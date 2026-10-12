//! Index stage of the artifact pipeline: shard-store TU reuse
//! (`ensure_index`, `rebuild_index_single_threaded`), the `--lang` unit
//! reshaping, and the plugin/closure/shard helpers the rebuild walks.
//!
//! Extracted VERBATIM from `lib.rs` (boot-progress-visibility V3a, hygiene
//! sec. 9: `lib.rs` sat AT the 1000-line ceiling). Same items, same
//! signatures, same bodies - a pure move landing before any behavior
//! change; M27 T3/T4 add into this module, not `lib.rs`.

use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Result};

use ariadne_build::{BuildDiscoverer, BuildDiscoveryResult, TranslationUnit};
use ariadne_index::{IncrementalStats, IndexResult, TuAnalysis, CURRENT_ANALYSIS_VERSION};
use ariadne_lang::{LanguageId, LanguagePlugin, LanguageRegistry, UnitShaper};
use ariadne_repo::SourceLanguage;
use ariadne_scheduler::{
    analyze_closure_with, compute_tu_closure_plugin, load_shard, save_shard, shard_dir,
    ProductionAnalyzer, SharedPluginWorkerBackend, SharedTuStoreContext, ShardCheckpoint,
    TuClosure, TuStoreContext,
};

use crate::{Pipeline, ProgressEvent, StageReuse, STAGE_INDEX};

impl Pipeline {
    // ------------------------------------------------------------------
    // Index stage (TU-level reuse via the shard store)
    // ------------------------------------------------------------------

    pub fn ensure_index(&mut self) -> Result<()> {
        if self.index_cache.is_some() {
            return Ok(());
        }

        if self.manager.stage_is_fresh("index", &self.fps.index) && self.language_filter.is_none() {
            if let Ok(loaded) = self.manager.load_index_checked(&self.fps.index) {
                self.index_cache = Some(loaded);
                self.stats.record(STAGE_INDEX, StageReuse::Loaded);
                return Ok(());
            }
        }

        let indexed_build = self.indexed_build()?;
        let (index_result, stats, production_stats) = match self.workers {
            Some(workers) => {
                // M18.11: the production path always runs the registry-backed
                // plugin backend and the plugin-driven shard prepass.
                let analyzer =
                    ProductionAnalyzer::new(&self.repo, &self.discovery, &indexed_build)
                        .with_workers(workers)
                        .with_backend(Arc::new(SharedPluginWorkerBackend::new(Arc::clone(
                            &self.registry,
                        ))))
                        .with_registry(Arc::clone(&self.registry));
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()?;
                let analysis_result = rt.block_on(analyzer.analyze())?;
                let production = analysis_result.stats;
                let reuse =
                    IncrementalStats::calculate(production.reused_tus, production.reanalyzed_tus);
                (analysis_result.index_result, reuse, Some(production))
            }
            None => {
                let (index_result, stats) = self.rebuild_index_single_threaded(&indexed_build)?;
                (index_result, stats, None)
            }
        };

        self.production_stats = production_stats;

        // A `--lang` restriction is a filtered view, never the authoritative
        // full-stage artifact; saving it under the full index fingerprint would
        // poison every later full run, so only the unfiltered result persists.
        if self.language_filter.is_none() {
            self.manager
                .save_index_artifacts(&index_result, &self.fps.index, Some(&stats))?;
        }
        self.stats.reused_tus = stats.reused_tus;
        self.stats.reanalyzed_tus = stats.reanalyzed_tus;
        self.index_cache = Some(index_result);
        self.stats.record(STAGE_INDEX, StageReuse::Rebuilt);
        Ok(())
    }

    /// The build result the index stage analyzes: full, or restricted to the
    /// `--lang` language (plan M18.3). Contexts are kept intact either way —
    /// closures resolve per TU context exactly as in a full run.
    ///
    /// A filtered run re-shapes through the filter's plugin shaper (D15):
    /// construction-time discovery ran the registry's marker-based selection
    /// (D22), which may recognize none of the filtered language's build
    /// markers at all (a repo shaped by another language's shaper yields
    /// zero units), so retention alone could never surface units of the
    /// filtered language.
    /// The unfiltered path returns the construction-time result unchanged —
    /// `select_unit_shaper`'s `None` branch is that same marker-based call,
    /// so `compute_inputs`' discovery and every derived fingerprint are
    /// exactly what the unfiltered run analyzes.
    fn indexed_build(&mut self) -> Result<BuildDiscoveryResult> {
        let mut build = match self.language_filter {
            None => self.build.clone(),
            Some(_) => {
                let reshaped = BuildDiscoverer::new(&self.repo.root)
                    .with_repo_discovery(&self.discovery)
                    .with_unit_shaper(select_unit_shaper(
                        &self.registry,
                        self.language_filter.clone(),
                        &self.repo.root,
                    ))
                    .discover()?;
                // The shard store context must see the contexts the shaping
                // named, exactly as construction-time shaping provided them
                // (context content is part of every TU cache fingerprint).
                self.store_ctx =
                    TuStoreContext::new(&self.discovery, reshaped.contexts.clone());
                reshaped
            }
        };
        if let Some(language) = self.language_filter.clone() {
            // M24 Part A: the `--lang` filter arrives as the closed file-kind
            // label, while detection answers in the open identity, so the
            // comparison goes through the one direction that exists.
            let selected = LanguageId::from(language);
            build
                .translation_units
                .retain(|tu| self.registry.detect_language(&tu.source_path).as_ref() == Some(&selected));
        }
        Ok(build)
    }

    fn rebuild_index_single_threaded(
        &mut self,
        build: &BuildDiscoveryResult,
    ) -> Result<(IndexResult, IncrementalStats)> {
        let store_dir = shard_dir(&self.repo.root);
        let store_ctx = self.store_ctx.clone();
        let repo_root = self.repo.root.clone();
        let mut progress = self.progress.take();
        let mut notify = |event: &ProgressEvent| {
            if let Some(cb) = progress.as_mut() {
                cb(event);
            }
        };

        let mut reused = 0usize;
        let mut reanalyzed = 0usize;
        let mut analyses = Vec::new();

        let total = build.translation_units.len();
        for (i, tu) in build.translation_units.iter().enumerate() {
            // R7 (V3): exactly one event per TU - TuStarted REPLACES the
            // per-TU TuIndex, carrying the repo-relative source path
            // verbatim (I10); strip_prefix falls back to the path as-is.
            // The terminal TuIndex below stays as the bar's final frame.
            notify(&ProgressEvent::TuStarted {
                done: i,
                total,
                source: tu
                    .source_path
                    .strip_prefix(&repo_root)
                    .unwrap_or(&tu.source_path)
                    .to_path_buf(),
            });

            let (analysis, was_reused) = index_unit_with_plugin(
                tu,
                &repo_root,
                &store_ctx,
                &store_dir,
                plugin_for_path(&self.registry, &tu.source_path)?,
            )?;
            if was_reused {
                reused += 1;
            } else {
                reanalyzed += 1;
            }
            analyses.push(analysis);
        }
        notify(&ProgressEvent::TuIndex { done: total, total });

        self.progress = progress;
        let index_result = IndexResult::new(repo_root, analyses);
        Ok((
            index_result,
            IncrementalStats::calculate(reused, reanalyzed),
        ))
    }

    pub fn get_or_build_index(&mut self) -> Result<IndexResult> {
        self.ensure_index()?;
        self.cached_index()
    }
}

/// Unit shaper for the index stage's unit shaping (D15 + D22): with an
/// explicit `--lang` filter the shaper comes from that language's plugin — a
/// plugin that provides no shaper honestly stays `None`, leaving build
/// discovery neutral instead of guessing another language's defaults;
/// without a filter the registry's marker-based selection against the
/// repository root applies (the same call `compute_inputs` made at
/// construction). Registry lookups only — no language-identity branching
/// (R3).
fn select_unit_shaper<'a>(
    registry: &'a LanguageRegistry,
    language_filter: Option<SourceLanguage>,
    repo_root: &Path,
) -> Option<&'a dyn UnitShaper> {
    match language_filter {
        Some(language) => registry
            .plugin_for_id(LanguageId::from(language))
            .and_then(|plugin| plugin.shaper()),
        None => registry.unit_shaper_for_repo(repo_root),
    }
}

/// Plugin that owns `path`'s language (plan §3.3 fact 3): the registry
/// detection is the dispatch key, and an unclaimed path is reported as
/// "unhandled language" rather than silently parsed with some other plugin.
fn plugin_for_path<'a>(
    registry: &'a LanguageRegistry,
    path: &Path,
) -> Result<&'a dyn LanguagePlugin> {
    let language = registry.detect_language(path).ok_or_else(|| {
        anyhow!(
            "unhandled language for {} (no registered plugin claims it)",
            path.display()
        )
    })?;
    registry
        .plugin_for_id(language.clone())
        .ok_or_else(|| anyhow!("language plugin not registered: {language:?}"))
}

/// Shard-backed analysis of one unit through the plugin resolver/analyzer
/// (plan M18.3, the only path since M18.11). `compute_tu_closure_plugin`
/// reproduces the canonical closure file order byte-for-byte (locked by the
/// vendored oracles in `m18_cpp_plugin_parity_test`), so shard envelopes and
/// merged analyses are identical to the pre-M11 hardcoded-C++ path.
fn index_unit_with_plugin(
    tu: &TranslationUnit,
    repo_root: &Path,
    store_ctx: &SharedTuStoreContext,
    store_dir: &Path,
    plugin: &dyn LanguagePlugin,
) -> Result<(TuAnalysis, bool)> {
    let closure = compute_tu_closure_plugin(
        tu,
        repo_root,
        store_ctx,
        CURRENT_ANALYSIS_VERSION,
        plugin.resolver(),
        // M18.10 fingerprint identity: `LanguageId` IS `SourceLanguage`
        // (plan §3.2), and for C++ these two values equal the constants the
        // pre-M11 walk hardcoded — keeping shard envelopes byte-identical.
        plugin.id(),
        &plugin.analysis_fingerprint(),
    )?;
    sharded_analysis(tu, &closure, store_dir, || {
        analyze_closure_with(tu, &closure, plugin.analyzer())
    })
}

/// Shared shard envelope contract of both unit paths: reuse the checkpoint
/// when the closure fingerprint still matches, else analyze and re-stamp.
fn sharded_analysis(
    tu: &TranslationUnit,
    closure: &TuClosure,
    store_dir: &Path,
    analyze: impl FnOnce() -> TuAnalysis,
) -> Result<(TuAnalysis, bool)> {
    if let Some(shard) = load_shard(store_dir, tu.file_id) {
        if shard.fingerprint == closure.fingerprint.combined_hash {
            return Ok((shard.analysis, true));
        }
    }
    let analysis = analyze();
    save_shard(
        store_dir,
        &ShardCheckpoint {
            fingerprint: closure.fingerprint.combined_hash.clone(),
            analysis: analysis.clone(),
        },
    )?;
    Ok((analysis, false))
}
