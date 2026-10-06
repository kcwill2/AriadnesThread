//! Load stage of the artifact pipeline: `ensure_load` with its A4
//! milestone bracketing, the single-copy take/restore lifecycle over the
//! stage inputs, the configured-pool runner, and the chunked, parallel,
//! order-preserving symbol/file metric passes.
//!
//! Extracted VERBATIM from `lib.rs` (boot-progress-visibility V3a, hygiene
//! sec. 9). M28 T2 changed the memory LIFECYCLE only: the cold span runs
//! with one live copy of the include graph and the reduced model (moved
//! out via `Pipeline::take_graph_and_reduced_for_load`, restored before
//! return, replacing the two A4 deep clones), releases the index result
//! before the passes run (A5), bounds the memo through the T1 boot caps,
//! drops the analyzer - and with it the memo's seeded closure table -
//! BEFORE the artifact save, and bounds the pass intermediates through the
//! `LOAD_PASS_CHUNK` chunking. The progress-event stream is byte-equal to
//! the pre-M28-T2 stream (I6; locked by the recording test in
//! `tests/load_stage_lifecycle_test.rs`). `collect_file_metrics` is
//! `pub(crate)` only because `internal_tests` reaches it through the
//! crate-root glob (the handoff's sanctioned visibility raise for a shared
//! moved item); its signature is unchanged (shape sec. 3.3 preferred
//! outcome - the pin test is green UNEDITED).

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use rayon::prelude::*;

use ariadne_graph::{
    IncludeClosure, IncludeGraph, LoadAnalyzer, LoadMemo, LoadMetric, ReducedSemanticModel,
};
use ariadne_repo::{FileId, SourceLanguage};
use ariadne_report::LoadReport;

use crate::{LoadMilestone, Pipeline, ProgressCb, ProgressEvent, StageReuse, STAGE_LOAD};

/// Chunk width of the two stage-5 passes (M28 T2 (c)): each rayon
/// `collect()` now owns at most a chunk's worth of intermediate elements,
/// so the pass intermediates scale with this width instead of the corpus.
/// 512 is a power of two, >= 16x a typical logical-core count (the pool
/// stays saturated per chunk), and <= ~9% of the failing corpus's file
/// count (5753), keeping chunk intermediates sub-dominant (T2 journal D3).
/// The chunk loop is sequential and rayon's `collect` preserves item order,
/// so output equals the pre-chunking full collect element-for-element
/// (locked by the UNEDITED sequential-reference oracles) and the event
/// stream is unchanged (locked by the I6 literal).
const LOAD_PASS_CHUNK: usize = 512;

impl Pipeline {
    // ------------------------------------------------------------------
    // Load stage
    // ------------------------------------------------------------------

    /// M28 T2 single-copy lifecycle: warm branches first (each releases the
    /// index result too), then the cold span moves the stage inputs out of
    /// their caches, analyzes through a capped boot memo, returns the
    /// inputs before the save, and saves without the index result, the
    /// analyzer, or the deep-cloned tables resident. `ensure_load` and
    /// every helper stay under the 100-line function ceiling (M28 T2 (e):
    /// extraction is the planned hygiene move at 92/100, not a rescue).
    pub fn ensure_load(&mut self) -> Result<()> {
        if self.load_cache.is_some() {
            return Ok(());
        }

        if let Some(loaded) = self.try_reuse_persisted_load()? {
            self.release_index_result_for_load();
            self.load_cache = Some(loaded);
            self.stats.record(STAGE_LOAD, StageReuse::Loaded);
            return Ok(());
        }

        self.ensure_include_graph()?;
        self.ensure_reduced()?;
        // A5 (M28 T2 (b)): stage 5 never reads the index result (the passes
        // consume include-graph and reduced-model data only), so it is
        // released here instead of sitting through both passes and the
        // save span at corpus scale. Later consumers reload it
        // fingerprint-fresh - contract documented at the seam.
        self.release_index_result_for_load();

        // A4 (M28 T2 (a)): MOVE the stage inputs out of their caches - the
        // pre-M28-T2 path deep-cloned the (graph, closures) pair and the
        // reduced model at this spot while the caches held the originals.
        let (graph, closures, reduced) = self.take_graph_and_reduced_for_load()?;
        let analyzer = build_stage5_analyzer(&graph, &reduced, &closures);
        let total_files = reduced.file_symbols.len();
        let repository_root = self.repo.root.clone();

        // Phase 4 (plans/LoadPerfImprovements), preserved from the
        // pre-split code: both analysis loops are CPU-bound and
        // independent per item, so they run under rayon with
        // `par_iter().filter_map().collect()`, which preserves the
        // sequential item order element-for-element. ONE memoized
        // `LoadAnalyzer` is shared via `&` across all workers (it is
        // `Send + Sync` - compile-asserted in the graph crate) so the
        // memo keeps warming across seeds; building an analyzer per item
        // would lose that reuse. There is deliberately NO cargo feature
        // flag for parallel vs sequential - the `workers` knob (the same
        // one the index stage uses) selects the pool, and
        // `test_load_report_matches_sequential_reference` locks the
        // parallel output against an in-test sequential reference.
        //
        // A4 milestone bracketing: the two passes stay separate pool
        // sections so the symbol-pass boundary can notify through
        // `self.notify` before the file pass takes the callback into a
        // Mutex for the rayon workers (mirroring
        // `rebuild_index_single_threaded`). The old `LoadReport` field
        // order already guaranteed symbols-before-files.
        let symbol_metrics =
            self.run_on_configured_pool(|| collect_symbol_metrics(&analyzer, &reduced))?;
        self.notify(&ProgressEvent::LoadMilestone(LoadMilestone::SymbolsAnalyzed {
            symbols: symbol_metrics.len(),
        }));

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

        // The analyzer is the sole owner of the shared memo (seeded
        // closure table + capped boot caches): dropping it BEFORE the save
        // span keeps that table off the base the JSON+bincode
        // serialization transient grows on top of (T0a evidence block 2).
        drop(analyzer);
        // The moved inputs return to their caches (move, no copy) - boot
        // hands out `take_include_graph` AFTER stage 5 and the warm-reuse
        // tests read both caches after the load stage.
        self.restore_stage5_inputs(graph, closures, reduced);

        let report = LoadReport {
            repository_root,
            symbol_metrics,
            file_metrics,
        };
        self.save_load_report_artifacts(&report)?;
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

    /// The load stage's warm branch: a persisted load artifact with a
    /// fresh fingerprint short-circuits the whole analysis; `None` means
    /// the cold rebuild runs. Mirrors the original inline branch exactly
    /// (the `ok()` swallow of a failed verified-load is the pre-existing
    /// rebuild-on-doubt behavior).
    fn try_reuse_persisted_load(&self) -> Result<Option<Arc<LoadReport>>> {
        if !self.manager.stage_is_fresh("load", &self.fps.load) {
            return Ok(None);
        }
        Ok(self.manager.load_load_checked(&self.fps.load).ok().map(Arc::new))
    }

    /// Persist the freshly built load report. M22 D25: the load report
    /// itself carries no language data, so the stage feeds the manifest
    /// from discovery records; an all-default repository still writes
    /// byte-identical pre-M22 manifest bytes.
    fn save_load_report_artifacts(&self, report: &LoadReport) -> Result<()> {
        let stage_languages: Vec<SourceLanguage> = self
            .discovery
            .files
            .iter()
            .map(|record| record.language)
            .collect();
        self.manager
            .save_load_artifacts_with_languages(report, &self.fps.load, &stage_languages)
            .map(|_artifact_root| ())
    }

    /// Runs `work` on a rayon-managed thread set honoring the `workers`
    /// knob: `Some(n)` builds a scoped pool of exactly `n` threads and
    /// `install`s the work into it; `None` runs it inline, letting the
    /// `par_iter` calls inside use the global pool. Same knob the index
    /// stage already uses - the deliberate no-feature-flag design is
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

/// Env name of the M28 T0c ablation lever: value `a,b` where each token is
/// a non-negative integer or `max` (-> `usize::MAX`, the uncapped control).
/// Read ONLY under `feature = "test-instrument"`; production builds do not
/// even read it, and the default path stays the landed T1 boot consts.
#[cfg(feature = "test-instrument")]
const CAPS_OVERRIDE_ENV: &str = "ARIADNE_LOAD_MEM_CAPS";

#[cfg(feature = "test-instrument")]
fn caps_override_usage_panic(raw_value: &str) -> ! {
    panic!(
        "{CAPS_OVERRIDE_ENV} must be '<non-negative integer|max>,<non-negative \
         integer|max>', got {raw_value:?} (no silent zeros, no silent defaults)"
    )
}

#[cfg(feature = "test-instrument")]
fn parse_caps_token(token: &str, raw_value: &str) -> usize {
    if token == "max" {
        return usize::MAX;
    }
    match token.parse::<usize>() {
        Ok(cap) => cap,
        Err(_) => caps_override_usage_panic(raw_value),
    }
}

/// Parse of the ablation env: `Some((semantic_cap, physical_cap))` when set,
/// `None` when unset (default path). Malformed values PANIC loudly, in bench
/// env style.
#[cfg(feature = "test-instrument")]
fn caps_override_from_env() -> Option<(usize, usize)> {
    let raw_value = std::env::var(CAPS_OVERRIDE_ENV).ok()?;
    let mut tokens = raw_value.split(',').map(str::trim);
    let semantic_token = tokens.next().unwrap_or("");
    let physical_token = tokens.next().unwrap_or("");
    if tokens.next().is_some() {
        caps_override_usage_panic(&raw_value);
    }
    Some((
        parse_caps_token(semantic_token, &raw_value),
        parse_caps_token(physical_token, &raw_value),
    ))
}

/// Single-copy stage-5 analyzer, wired to the T1 boot memo caps (M28 T2
/// cap wiring - the boot memo instance lives here). The cap literals are
/// T1's LANDED choices from commit `d6640a2` (`hop-fixed`, not review-
/// ratified): `SEMANTIC_CLOSURE_BOOT_CAP` = 512 and
/// `PHYSICAL_EXPANSION_BOOT_CAP` = 128, justified in
/// `tasks/stage5-collateral-memory-ceiling-t1-journal.md` ("Hop-fixed cap
/// literals") against the measured N=1280 memo attribution
/// (`tasks/stage5-collateral-memory-ceiling-t0a-evidence.md` block 4) and
/// the boot single-use query economics: boot queries each seed and each
/// def-file exactly once, so retained entries past the cap are pure
/// bloat. Query-result equality under any capacity is locked forever by
/// T1's capacity-invariance suite, so the caps trade time, never bytes.
///
/// The `closures.clone()` below is the ONE deep copy retained in the
/// stage-5 span (T2 journal deviation D1): `LoadMemo` consumes its seed
/// table by value and offers no extraction, while the include cache must
/// hold the pair again when `ensure_load` returns (boot's
/// `take_include_graph` after stage 5, `ui.rs:250`). The graph and the
/// reduced model, which the pre-M28-T2 code deep-cloned here too, arrive
/// by move and are only borrowed.
// `pub(crate)` is the M28 T0c route-test seam: the feature-gated caps tests
// in `stage_load_tests.rs` call the route directly (journal D1, the
// `collect_file_metrics` pub(crate)-raise precedent). The module is private,
// so the production surface is unchanged.
pub(crate) fn build_stage5_analyzer<'a>(
    graph: &'a IncludeGraph,
    reduced: &'a ReducedSemanticModel,
    closures: &'a HashMap<FileId, IncludeClosure>,
) -> LoadAnalyzer<'a> {
    // M28 T0c cap-0 ablation lever: with the env set (instrumented builds
    // only) the boot memo takes the env caps - `0,0` is the pure-recompute
    // ablation whose RSS separates memo retention from the save-span
    // transient. The witness line prints exactly once per process (loud
    // anti-fabrication marker: every cap-0 capture must carry it, no
    // landed-curve capture may). With the feature off this whole branch
    // compiles out and the default below is byte-identical to the landed
    // path.
    #[cfg(feature = "test-instrument")]
    if let Some((semantic_cap, physical_cap)) = caps_override_from_env() {
        static CAPS_WITNESS: std::sync::Once = std::sync::Once::new();
        CAPS_WITNESS.call_once(|| {
            eprintln!(
                "load-memo caps override active: semantic={semantic_cap} physical={physical_cap}"
            );
        });
        let memo =
            LoadMemo::with_closures_and_capacity(closures.clone(), semantic_cap, physical_cap);
        return LoadAnalyzer::with_memo(graph, reduced, Arc::new(memo));
    }
    let memo = LoadMemo::with_closures_and_capacity(
        closures.clone(),
        LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP,
        LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP,
    );
    LoadAnalyzer::with_memo(graph, reduced, Arc::new(memo))
}

/// Chunked, parallel, order-preserving symbol metric pass for
/// `ensure_load` (M28 T2 (c)). The BTreeMap `.values()` view is walked in
/// name chunks of `LOAD_PASS_CHUNK` (b-tree value iterators are not rayon
/// parallel iterators): each chunk is collected through
/// `par_iter().filter_map().collect()` - the same vector, and the same
/// order, the old full-size collect produced - and appended, so the
/// chunk-owned intermediates stay chunk-sized while the result keeps the
/// exact element order (sequential-reference oracles, green UNEDITED).
fn collect_symbol_metrics(
    analyzer: &LoadAnalyzer<'_>,
    reduced: &ReducedSemanticModel,
) -> Vec<LoadMetric> {
    let mut metrics: Vec<LoadMetric> = Vec::new();
    let mut name_chunk: Vec<String> = Vec::with_capacity(LOAD_PASS_CHUNK);
    for symbol_record in reduced.symbols.values() {
        name_chunk.push(symbol_record.name.clone());
        if name_chunk.len() == LOAD_PASS_CHUNK {
            metrics.extend(analyze_symbol_chunk(analyzer, &name_chunk));
            name_chunk.clear();
        }
    }
    if !name_chunk.is_empty() {
        metrics.extend(analyze_symbol_chunk(analyzer, &name_chunk));
    }
    metrics
}

/// One symbol chunk: analyze every name in parallel, skipping the names
/// whose analysis errors exactly as the pre-split `filter_map` did.
fn analyze_symbol_chunk(analyzer: &LoadAnalyzer<'_>, symbol_names: &[String]) -> Vec<LoadMetric> {
    symbol_names
        .par_iter()
        .filter_map(|name| analyzer.analyze_symbol_load(name).ok())
        .collect()
}

/// Parallel, order-preserving file metric pass for `ensure_load`, chunked
/// over `LOAD_PASS_CHUNK` file ids (M28 T2 (c)) with the signature pinned
/// by `internal_tests` kept UNCHANGED. Each worker emits one
/// `CollateralFile` progress event after its file completes; `done` comes
/// from `AtomicUsize::fetch_add`, so the values stay unique (0..total-1)
/// while event arrival may interleave - the UI only reads done/total. The
/// `done == total` completion event is the caller's. `completed` is
/// incremented once per file id (matching the sequential `enumerate`),
/// even when the id has no registered path. Chunks are processed
/// sequentially, so the collected metrics keep the exact `file_symbols`
/// key order the pre-chunking single collect produced.
pub(crate) fn collect_file_metrics(
    analyzer: &LoadAnalyzer<'_>,
    graph: &IncludeGraph,
    reduced: &ReducedSemanticModel,
    total_files: usize,
    progress: &Mutex<Option<ProgressCb>>,
    completed: &AtomicUsize,
) -> Vec<LoadMetric> {
    let mut metrics: Vec<LoadMetric> = Vec::new();
    let mut file_id_chunk: Vec<FileId> = Vec::with_capacity(LOAD_PASS_CHUNK);
    for file_id in reduced.file_symbols.keys() {
        file_id_chunk.push(*file_id);
        if file_id_chunk.len() == LOAD_PASS_CHUNK {
            metrics.extend(analyze_file_chunk(
                analyzer,
                graph,
                &file_id_chunk,
                total_files,
                progress,
                completed,
            ));
            file_id_chunk.clear();
        }
    }
    if !file_id_chunk.is_empty() {
        metrics.extend(analyze_file_chunk(
            analyzer,
            graph,
            &file_id_chunk,
            total_files,
            progress,
            completed,
        ));
    }
    metrics
}

/// One file chunk: analyze every registered id in parallel, skipping the
/// ids without a registered path. By-id analysis drops the per-file O(V)
/// `find_file_id` string scan the old path round-trip paid (plan LoadPerf
/// A2); the registration check preserves the old skip contract: an id
/// without a path yields None and disappears via `flatten()`.
fn analyze_file_chunk(
    analyzer: &LoadAnalyzer<'_>,
    graph: &IncludeGraph,
    file_ids: &[FileId],
    total_files: usize,
    progress: &Mutex<Option<ProgressCb>>,
    completed: &AtomicUsize,
) -> Vec<LoadMetric> {
    file_ids
        .par_iter()
        .map(|&file_id| {
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
