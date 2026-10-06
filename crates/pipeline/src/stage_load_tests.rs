//! M28 T2 stage-5 lifecycle locks that need pipeline-INTERNAL visibility
//! (private `include_cache` / `reduced_cache` / `index_cache` fields and the
//! `pub(crate)` take/restore seam). Kept in-src as the internal sibling of
//! `internal_tests.rs` (same D-M23-10 split rationale). This file stays
//! plugin-name-free like its sibling: the fixture directive text is
//! assembled from a non-contiguous marker and the compile command uses a
//! generic `cc` executable (the sanctioned temp-dir fixture replication of
//! the `internal_tests.rs` shape - the equivalence oracle file itself stays
//! UNEDITED, T2 journal deviation D5).

use super::*;
use std::fs;
use tempfile::TempDir;

/// Guard-clean sibling of the relocated `setup_repo` fixture: same tree as
/// `internal_tests.rs::setup_repo`, byte-identical directive text via the
/// non-contiguous marker, generic `cc` compile command.
fn setup_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("include/common.h"), "struct Common { int id; };").unwrap();
    fs::write(
        root.join("include/header_a.h"),
        concat!("#inc", "lude \"common.h\"\nstruct ThingA { Common c; };"),
    )
    .unwrap();
    fs::write(
        root.join("include/header_b.h"),
        "struct ThingB { double x; };",
    )
    .unwrap();
    fs::write(
        root.join("src/tu1.cpp"),
        concat!("#inc", "lude \"header_a.h\"\nvoid use_a(ThingA a) {}"),
    )
    .unwrap();
    fs::write(
        root.join("src/tu2.cpp"),
        concat!("#inc", "lude \"header_b.h\"\nvoid use_b(ThingB b) {}"),
    )
    .unwrap();

    let root_str = root.to_string_lossy().replace('\\', "/");
    let compile_commands = format!(
        r#"[
    {{ "directory": "{root_str}", "command": "cc -Iinclude -c src/tu1.cpp", "file": "src/tu1.cpp" }},
    {{ "directory": "{root_str}", "command": "cc -Iinclude -c src/tu2.cpp", "file": "src/tu2.cpp" }}
]"#,
        root_str = root_str
    );
    fs::write(root.join("compile_commands.json"), compile_commands).unwrap();
    dir
}

/// (a) single-copy structural witness (M28 T2): the stage-5 input seam must
/// MOVE the tables, not hand out clones while the caches keep the originals.
/// While the hand-out is alive, the caches must be empty - that is the
/// machine form of "stage 5 runs with one live copy": pipeline-side
/// residency during the pass is exactly the handed-out copy. The restore
/// leg locks the inverse contract that boot's `take_include_graph` after
/// stage 5 (`ui.rs:250`) depends on.
#[test]
fn test_take_graph_and_reduced_for_load_empties_stage_caches() {
    let dir = setup_repo();
    let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
    pipeline.ensure_include_graph().unwrap();
    pipeline.ensure_reduced().unwrap();

    let (graph, closures, reduced) = pipeline
        .take_graph_and_reduced_for_load()
        .expect("seam must hand out after both stages ensured");
    assert!(
        pipeline.include_cache.is_none(),
        "single-copy stage-5 lifecycle violated: include_cache still holds the graph/closures while the stage-5 hand-out exists - the A4 deep-clone path (cached_include_graph at stage_load.rs:43) keeps a second live copy resident through stage 5"
    );
    assert!(
        pipeline.reduced_cache.is_none(),
        "single-copy stage-5 lifecycle violated: reduced_cache still holds the reduced model while the stage-5 hand-out exists - the A4 deep-clone path (cached_reduced at stage_load.rs:44) keeps a second live copy resident through stage 5"
    );
    assert!(
        !graph.file_paths.is_empty() && !closures.is_empty() && !reduced.file_symbols.is_empty(),
        "the hand-out must carry the full populated stage data (move, not partial drain)"
    );

    pipeline.restore_stage5_inputs(graph, closures, reduced);
    let (restored_graph, restored_closures) = pipeline
        .cached_include_graph()
        .expect("restore must repopulate the include cache");
    let restored_reduced = pipeline
        .cached_reduced()
        .expect("restore must repopulate the reduced cache");
    assert!(
        !restored_graph.file_paths.is_empty()
            && !restored_closures.is_empty()
            && !restored_reduced.file_symbols.is_empty(),
        "restore must hand the full tables back to the stage caches (boot's take_include_graph after stage 5 depends on them)"
    );
}

/// (a) floor: the post-stage-5 state contract. After a cold `ensure_load`
/// the stage caches must again hand out populated graph/closures/reduced
/// (the boot path takes the include graph AFTER stage 5; the sequential
/// reference tests read both caches after the load stage). Green today
/// (caches never emptied); must stay green after the take/restore lands.
#[test]
fn test_cold_ensure_load_restores_stage_inputs() {
    let dir = setup_repo();
    let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
    pipeline.ensure_load().unwrap();

    let (graph, closures) = pipeline
        .cached_include_graph()
        .expect("include cache must be populated after ensure_load returns");
    let reduced = pipeline
        .cached_reduced()
        .expect("reduced cache must be populated after ensure_load returns");
    assert!(
        !graph.file_paths.is_empty() && !closures.is_empty() && !reduced.file_symbols.is_empty(),
        "stage 5 must leave the full stage inputs in the caches"
    );
}

/// (b) release assertion, size-counter form (M28 T2): after a cold
/// `ensure_load` the pipeline must not hold the index result. Stage 5 never
/// reads it (shape sec. 0 A5); the resident `IndexResult` through the
/// passes and the save span is pure dead weight at corpus scale. The
/// reload contract is locked in the integration sibling
/// (`load_stage_lifecycle_test.rs`) via the public `RunStats` witness.
#[test]
fn test_cold_ensure_load_releases_index_result() {
    let dir = setup_repo();
    let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
    pipeline.ensure_load().unwrap();

    assert!(
        pipeline.index_cache.is_none(),
        "index result still resident through stage 5: stage 5 never reads the IndexResult (A5), so ensure_load must release it before the stage-5 pass runs - the cold rebuild path still holds it after return"
    );
}

/// (b) release assertion, warm branch (M28 T2, journal D4): the on-disk
/// load path of `ensure_load` runs stage 5 too, and a warm boot keeps its
/// loaded-from-disk `IndexResult` resident the same way. A fresh pipeline
/// that ensured the index (stage 2) and then the load stage must not hold
/// it afterwards; the public reload leg lives in the integration sibling.
#[test]
fn test_warm_ensure_load_releases_index_result() {
    let dir = setup_repo();
    let mut cold = Pipeline::new(dir.path(), None).unwrap();
    cold.ensure_load().unwrap();
    drop(cold);

    let mut warm = Pipeline::new(dir.path(), None).unwrap();
    warm.ensure_index().unwrap();
    warm.ensure_load().unwrap();

    assert!(
        warm.index_cache.is_none(),
        "warm ensure_load must release the loaded-from-disk index result too: stage 5 loaded its artifacts from disk and never reads the IndexResult (A5), so the resident copy is dead weight through the warm span"
    );
}

// ---------------------------------------------------------------------------
// M28 T0c cap-0 ablation route locks (feature-gated). `ARIADNE_LOAD_MEM_CAPS`
// (value `a,b`, each token a non-negative integer or `max`) makes
// `build_stage5_analyzer` build the boot memo with the given caps - cap 0 is
// the pure-recompute ablation the T0c ranking measures (T1 verdict: cap 0 =
// ablation lever, LOCKED). With the feature off (every production build) the
// route compiles out and the env is not even read; these tests exist only in
// the instrumented build.
// ---------------------------------------------------------------------------

#[cfg(feature = "test-instrument")]
mod caps_env_ablation {
    use super::setup_repo;
    use crate::stage_load::build_stage5_analyzer;
    use crate::Pipeline;
    use ariadne_graph::LoadMemo;
    use std::panic::AssertUnwindSafe;
    use std::sync::{Mutex, MutexGuard};

    const CAPS_ENV: &str = "ARIADNE_LOAD_MEM_CAPS";

    /// Env discipline: `std::env` is process-global and this harness runs
    /// tests in parallel, so all three tests take ONE shared mutex around
    /// every set/run/unset section (poison-safe) and restore the unset
    /// state on every exit path via the Drop guard - including the
    /// malformed-value panic test and any failed assertion.
    static CAPS_ENV_LOCK: Mutex<()> = Mutex::new(());

    struct CapsEnvGuard {
        _env_lock: MutexGuard<'static, ()>,
    }

    impl CapsEnvGuard {
        fn with(value: Option<&str>) -> Self {
            let env_lock = CAPS_ENV_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match value {
                Some(raw) => std::env::set_var(CAPS_ENV, raw),
                None => std::env::remove_var(CAPS_ENV),
            }
            CapsEnvGuard { _env_lock: env_lock }
        }
    }

    impl Drop for CapsEnvGuard {
        fn drop(&mut self) {
            std::env::remove_var(CAPS_ENV);
        }
    }

    /// Warm the graph/reduced/closures trio through the public boot stages
    /// and hand the trio out through the (pub(crate)) stage-5 input seam -
    /// the same isolated trio `ensure_load` feeds the analyzer.
    fn stage_inputs(
        dir: &tempfile::TempDir,
    ) -> (
        ariadne_graph::IncludeGraph,
        std::collections::HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
        ariadne_graph::ReducedSemanticModel,
    ) {
        let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
        pipeline.ensure_include_graph().unwrap();
        pipeline.ensure_reduced().unwrap();
        pipeline
            .take_graph_and_reduced_for_load()
            .expect("the seam hands out the populated trio after both stages")
    }

    /// (a) the caps env reaches BOTH boot caches: at `0,0` the memo admits
    /// nothing (pure recompute) while queries keep returning results.
    #[test]
    fn caps_env_override_bounds_both_boot_caches() {
        let dir = setup_repo();
        let (graph, closures, reduced) = stage_inputs(&dir);
        let _env_guard = CapsEnvGuard::with(Some("0,0"));

        let analyzer = build_stage5_analyzer(&graph, &reduced, &closures);
        let memo = analyzer.memo();
        let mut seeds_queried = 0usize;
        for seed in reduced.symbols.keys().take(3) {
            let closure = memo.cached_single_seed_closure(&reduced, seed);
            assert!(
                !closure.is_empty(),
                "cap-0 recompute must still return the closure for every seed"
            );
            seeds_queried += 1;
        }
        let mut files_queried = 0usize;
        for file_id in reduced.file_symbols.keys().take(2) {
            let expansion = memo.cached_physical_expansion(&graph, &reduced, *file_id);
            assert!(
                !expansion.is_empty(),
                "cap-0 recompute must still return the expansion for every file"
            );
            files_queried += 1;
        }
        assert!(
            seeds_queried >= 1 && files_queried >= 1,
            "the shared fixture must offer queryable seeds and files, got {} seeds {} files",
            seeds_queried,
            files_queried
        );

        assert_eq!(
            memo.semantic_closure_cache_len(),
            0,
            "ARIADNE_LOAD_MEM_CAPS=0,0 must reach the semantic-closure cache; \
             actual cache lengths semantic={} physical={}",
            memo.semantic_closure_cache_len(),
            memo.physical_expansion_cache_len()
        );
        assert_eq!(
            memo.physical_expansion_cache_len(),
            0,
            "ARIADNE_LOAD_MEM_CAPS=0,0 must reach the physical-expansion cache; \
             actual cache lengths semantic={} physical={}",
            memo.semantic_closure_cache_len(),
            memo.physical_expansion_cache_len()
        );
    }

    /// (b) default-inertness pin: with the env UNSET the memo behaves as the
    /// landed consts (512/128 boot caps) - every demanded entry is admitted
    /// at bench-fixture demand levels (all demand stays below both caps), so
    /// retention tracks demand exactly and nothing reads the env.
    /// Green-by-design at Red (journal D2): the pin that the route cannot
    /// alter the default path.
    #[test]
    fn caps_env_absent_keeps_landed_const_caps() {
        let dir = setup_repo();
        let (graph, closures, reduced) = stage_inputs(&dir);
        let _env_guard = CapsEnvGuard::with(None);

        let analyzer = build_stage5_analyzer(&graph, &reduced, &closures);
        let memo = analyzer.memo();
        assert_eq!(
            LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP,
            512,
            "the landed semantic-closure boot cap is 512 (T1 hop-fixed, d6640a2)"
        );
        assert_eq!(
            LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP,
            128,
            "the landed physical-expansion boot cap is 128 (T1 hop-fixed, d6640a2)"
        );

        let mut seeds_queried = 0usize;
        for seed in reduced.symbols.keys().take(3) {
            memo.cached_single_seed_closure(&reduced, seed);
            seeds_queried += 1;
        }
        let mut files_queried = 0usize;
        for file_id in reduced.file_symbols.keys().take(2) {
            memo.cached_physical_expansion(&graph, &reduced, *file_id);
            files_queried += 1;
        }

        assert_eq!(
            memo.semantic_closure_cache_len(),
            seeds_queried,
            "env unset must keep memoization active under the landed caps: every \
             queried seed is below the cap and must be admitted"
        );
        assert_eq!(
            memo.physical_expansion_cache_len(),
            files_queried,
            "env unset must keep memoization active under the landed caps: every \
             queried file is below the cap and must be admitted"
        );
        assert!(
            memo.semantic_closure_cache_len() <= LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP
                && memo.physical_expansion_cache_len()
                    <= LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP,
            "cache lengths must stay within the landed caps"
        );
    }

    /// (c) a malformed value must PANIC loudly - no silent zeros, bench env
    /// style (a typo'd ablation env that quietly read as cap 0, or as the
    /// default, would poison the whole ranking table).
    #[test]
    fn caps_env_malformed_panics_loudly() {
        let dir = setup_repo();
        let (graph, closures, reduced) = stage_inputs(&dir);
        let _env_guard = CapsEnvGuard::with(Some("3,x"));

        let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
            build_stage5_analyzer(&graph, &reduced, &closures)
        }));

        assert!(
            outcome.is_err(),
            "malformed {} must panic loudly (no silent zeros, no silent \
             defaults); the route returned Ok - no panic occurred",
            CAPS_ENV
        );
    }
}
