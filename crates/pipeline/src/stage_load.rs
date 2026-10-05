//! Load stage of the artifact pipeline: `ensure_load` with its A4
//! milestone bracketing, the configured-pool runner, and the parallel
//! order-preserving symbol/file metric passes.
//!
//! Extracted VERBATIM from `lib.rs` (boot-progress-visibility V3a, hygiene
//! sec. 9). Same items, same signatures, same bodies - a pure move landing
//! before any behavior change. `collect_file_metrics` is `pub(crate)` only
//! because `internal_tests` reaches it through the crate-root glob
//! (the handoff's sanctioned visibility raise for a shared moved item).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use rayon::prelude::*;

use ariadne_graph::{IncludeGraph, LoadAnalyzer, LoadMetric, ReducedSemanticModel};
use ariadne_repo::{FileId, SourceLanguage};
use ariadne_report::LoadReport;

use crate::{LoadMilestone, Pipeline, ProgressCb, ProgressEvent, StageReuse, STAGE_LOAD};

impl Pipeline {
    // ------------------------------------------------------------------
    // Load stage
    // ------------------------------------------------------------------

    pub fn ensure_load(&mut self) -> Result<()> {
        if self.load_cache.is_some() {
            return Ok(());
        }

        if self.manager.stage_is_fresh("load", &self.fps.load) {
            if let Ok(loaded) = self.manager.load_load_checked(&self.fps.load) {
                self.load_cache = Some(Arc::new(loaded));
                self.stats.record(STAGE_LOAD, StageReuse::Loaded);
                return Ok(());
            }
        }

        self.ensure_include_graph()?;
        self.ensure_reduced()?;
        let (graph, closures) = self.cached_include_graph()?;
        let reduced = self.cached_reduced()?;

        // Phase 4 (plans/LoadPerfImprovements): both analysis loops are
        // CPU-bound and independent per item, so they run under rayon with
        // `par_iter().filter_map().collect()`, which preserves the
        // sequential item order element-for-element. ONE memoized
        // `LoadAnalyzer` is shared via `&` across all workers (it is
        // `Send + Sync` — compile-asserted in the graph crate) so the
        // append-only semantic-closure memo keeps warming across seeds;
        // building an analyzer per item would lose that cross-seed reuse.
        //
        // Deliberate deviation from the plan: there is NO cargo feature
        // flag for parallel vs sequential. A flag would ship a default
        // path that no CI run ever exercises; instead the `workers` knob
        // (already the parallelism control for the index stage) selects
        // the pool size, and the test
        // `test_load_report_matches_sequential_reference` locks the
        // parallel output against an in-test sequential reference.
        let analyzer = LoadAnalyzer::with_closures(&graph, &reduced, closures);
        let total_files = reduced.file_symbols.len();
        let repository_root = self.repo.root.clone();
        // A4: the two analysis passes run as separate pool sections so the
        // symbol-pass boundary can notify through `self.notify` while the
        // callback is still on `self`; the file pass then takes the callback
        // into a Mutex for the rayon workers, mirroring
        // `rebuild_index_single_threaded`. Field order of the old `LoadReport`
        // struct literal already guaranteed symbols-before-files; the split
        // keeps that order and makes it a notifiable milestone.
        let symbol_metrics =
            self.run_on_configured_pool(|| collect_symbol_metrics(&analyzer, &reduced))?;
        self.notify(&ProgressEvent::LoadMilestone(LoadMilestone::SymbolsAnalyzed {
            symbols: symbol_metrics.len(),
        }));

        // take/restore the callback around the parallel section, mirroring
        // `rebuild_index_single_threaded`; the Mutex lets every worker share
        // one `FnMut` sink and guards are only held for a single event.
        let progress = Mutex::new(self.progress.take());
        let completed_files = AtomicUsize::new(0);
        let file_metrics = self.run_on_configured_pool(|| {
            collect_file_metrics(
                &analyzer,
                &graph,
                &reduced,
                total_files,
                &progress,
                &completed_files,
            )
        })?;

        self.progress = progress.into_inner().unwrap_or_else(|err| err.into_inner());
        self.notify(&ProgressEvent::CollateralFile {
            done: total_files,
            total: total_files,
        });

        let report = LoadReport {
            repository_root,
            symbol_metrics,
            file_metrics,
        };
        // M22 D25: the load report itself carries no language data, so the
        // stage feeds the manifest from discovery records; an all-default
        // repository still writes byte-identical pre-M22 manifest bytes.
        let stage_languages: Vec<SourceLanguage> =
            self.discovery.files.iter().map(|f| f.language).collect();
        self.manager.save_load_artifacts_with_languages(
            &report,
            &self.fps.load,
            &stage_languages,
        )?;
        self.notify(&ProgressEvent::LoadMilestone(LoadMilestone::ArtifactsSaved));
        self.load_cache = Some(Arc::new(report));
        self.stats.record(STAGE_LOAD, StageReuse::Rebuilt);
        Ok(())
    }

    /// A3: hands out a shared `Arc<LoadReport>` handle, not a copy.
    pub fn get_or_build_load(&mut self) -> Result<Arc<LoadReport>> {
        self.ensure_load()?;
        self.cached_load()
    }

    /// Runs `work` on a rayon-managed thread set honoring the `workers`
    /// knob: `Some(n)` builds a scoped pool of exactly `n` threads and
    /// `install`s the work into it; `None` runs it inline, letting the
    /// `par_iter` calls inside use the global pool. Same knob the index
    /// stage already uses — the deliberate no-feature-flag design is
    /// documented at the call site in `ensure_load`.
    fn run_on_configured_pool<T, F>(&self, work: F) -> Result<T>
    where
        T: Send,
        F: FnOnce() -> T + Send,
    {
        match self.workers {
            Some(num_threads) => {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(num_threads.max(1))
                    .build()
                    .map_err(|err| anyhow!("failed to build rayon thread pool: {err}"))?;
                Ok(pool.install(work))
            }
            None => Ok(work()),
        }
    }
}

/// Parallel, order-preserving symbol metric pass for `ensure_load`. The
/// BTreeMap `.values()` view is materialized into a `Vec` first (b-tree
/// value iterators are not rayon parallel iterators); rayon's
/// `par_iter().filter_map().collect()` then yields the same vector — and
/// the same order — as the old sequential loop.
fn collect_symbol_metrics(
    analyzer: &LoadAnalyzer<'_>,
    reduced: &ReducedSemanticModel,
) -> Vec<LoadMetric> {
    let symbol_names: Vec<String> = reduced
        .symbols
        .values()
        .map(|record| record.name.clone())
        .collect();
    symbol_names
        .par_iter()
        .filter_map(|name| analyzer.analyze_symbol_load(name).ok())
        .collect()
}

/// Parallel, order-preserving file metric pass for `ensure_load`. Each
/// worker emits one `CollateralFile` progress event after its file
/// completes; `done` comes from `AtomicUsize::fetch_add`, so the values
/// stay unique (0..total-1) while event arrival may interleave — the UI
/// only reads done/total. The `done == total` completion event is the
/// caller's. `completed` is incremented once per file id (matching the
/// sequential `enumerate`), even when the id has no registered path.
pub(crate) fn collect_file_metrics(
    analyzer: &LoadAnalyzer<'_>,
    graph: &IncludeGraph,
    reduced: &ReducedSemanticModel,
    total_files: usize,
    progress: &Mutex<Option<ProgressCb>>,
    completed: &AtomicUsize,
) -> Vec<LoadMetric> {
    let file_ids: Vec<FileId> = reduced.file_symbols.keys().copied().collect();
    file_ids
        .par_iter()
        .map(|&file_id| {
            // By-id analysis drops the per-file O(V) `find_file_id` string
            // scan the old path round-trip paid (plan LoadPerf A2). The
            // registration check preserves the old skip contract: an id
            // without a path yields None and disappears via `flatten()`.
            let metric = if graph.file_paths.contains_key(&file_id) {
                analyzer.analyze_file_load_by_id(file_id).ok()
            } else {
                None
            };
            let done = completed.fetch_add(1, Ordering::SeqCst);
            if let Some(cb) = progress.lock().unwrap().as_mut() {
                cb(&ProgressEvent::CollateralFile {
                    done,
                    total: total_files,
                });
            }
            metric
        })
        .flatten()
        .collect()
}
