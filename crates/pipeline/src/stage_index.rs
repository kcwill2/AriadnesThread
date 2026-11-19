//! Index stage of the artifact pipeline: stage reuse (`ensure_index`), the
//! in-process store-once rebuild (`rebuild_index_in_process`, see
//! [`crate::stage_index_store_once`]), the `--workers` production path, and
//! the `--lang` unit reshaping.
//!
//! Extracted from `lib.rs` (boot-progress-visibility V3a, hygiene sec. 9:
//! `lib.rs` sat AT the 1000-line ceiling).

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;

use ariadne_build::{BuildDiscoverer, BuildDiscoveryResult};
use ariadne_index::{IncrementalStats, IndexResult};
use ariadne_lang::{LanguageId, LanguageRegistry, UnitShaper};
use ariadne_repo::SourceLanguage;
use ariadne_scheduler::{ProductionAnalyzer, SharedPluginWorkerBackend, TuStoreContext};

use crate::stage_index_store_once::{rebuild_index_store_once, StoreOnceInputs};
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
                let (index_result, stats) = self.rebuild_index_in_process(&indexed_build)?;
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

    /// In-process rebuild (no `--workers`): store-once indexing, so a header
    /// shared by many units is analyzed and stored once (see
    /// [`crate::stage_index_store_once`]). Runs on the global rayon pool.
    fn rebuild_index_in_process(
        &mut self,
        build: &BuildDiscoveryResult,
    ) -> Result<(IndexResult, IncrementalStats)> {
        let mut progress = self.progress.take();
        let mut notify = |event: &ProgressEvent| {
            if let Some(cb) = progress.as_mut() {
                cb(event);
            }
        };
        let inputs = StoreOnceInputs {
            repo_root: &self.repo.root,
            store_ctx: &self.store_ctx,
            registry: &self.registry,
            pool: None,
        };
        let outcome = rebuild_index_store_once(&build.translation_units, &inputs, &mut notify);
        self.progress = progress;
        outcome
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
