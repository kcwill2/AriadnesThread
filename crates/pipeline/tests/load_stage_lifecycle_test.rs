//! M28 T2 stage-5 lifecycle locks over the PUBLIC pipeline API: the I6
//! recorded event-stream literal (the byte-equality leash for the
//! take/restore + chunking change), the exactly-one-`CollateralFile`-per-
//! file contract, and the public `RunStats` witness that the index result
//! is released before stage 5 and reloads fingerprint-fresh afterwards.
//!
//! The fixture replicates the `internal_tests.rs` temp-dir shape (journal
//! deviation D5 - the equivalence oracle file stays UNEDITED).

use std::fs;
use std::sync::{Arc, Mutex};

use ariadne_pipeline::{Pipeline, ProgressEvent, StageReuse, STAGE_INDEX};
use tempfile::TempDir;

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

fn recorder() -> (Arc<Mutex<Vec<ProgressEvent>>>, impl FnMut(&ProgressEvent)) {
    let events: Arc<Mutex<Vec<ProgressEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();
    (events, move |event| sink.lock().unwrap().push(event.clone()))
}

/// I6 characterization FLOOR (M28 T2 RGB step 1a - a green-first lock, not
/// a red): the FULL `ensure_load` progress stream on the locked fixture,
/// captured from the pre-M28-T2 tree and replayed literally. Stages 1-3
/// are warmed silently by a first pipeline (the warm-load branches emit no
/// events), so the recording pipeline's stream is exactly the stage-5
/// output. `workers = Some(1)` pins the scoped pool to one thread, which
/// makes the per-file `CollateralFile` arrival order deterministic (the
/// pre-change tree collects one unchunked pass; the post-change chunk loop
/// keeps the same pool and, below the chunk width, the same single pass) -
/// that is what lets kinds+order+payloads be pinned as a literal.
/// Any add, drop, reorder, or payload change in the take/restore or the
/// chunked passes fails here; the literal is the law (I6 has zero budget).
#[test]
fn test_stage_five_event_stream_matches_recorded_literal() {
    let dir = setup_repo();
    {
        let mut warmer = Pipeline::new(dir.path(), None).unwrap();
        warmer.ensure_include_graph().unwrap();
        warmer.ensure_index().unwrap();
        warmer.ensure_reduced().unwrap();
    }

    let (events, cb) = recorder();
    let mut pipeline = Pipeline::new(dir.path(), Some(1))
        .unwrap()
        .with_progress(cb);
    pipeline.ensure_load().unwrap();

    let recorded = events.lock().unwrap();
    // The event enum is read-only material for this hop (no PartialEq), so
    // the byte-equality lock runs on the frames' Debug rendering: identical
    // kinds + order + payloads render byte-identically, anything else does
    // not. The literal below was captured from the pre-M28-T2 tree (T2
    // journal, R1 capture run).
    let recorded_frames: Vec<String> = recorded
        .iter()
        .map(|event| format!("{event:?}"))
        .collect();
    let expected: Vec<&str> = vec![
        "LoadMilestone(SymbolsAnalyzed { symbols: 5 })",
        "CollateralFile { done: 0, total: 5 }",
        "CollateralFile { done: 1, total: 5 }",
        "CollateralFile { done: 2, total: 5 }",
        "CollateralFile { done: 3, total: 5 }",
        "CollateralFile { done: 4, total: 5 }",
        "CollateralFile { done: 5, total: 5 }",
        "LoadMilestone(ArtifactsSaved)",
    ];
    assert_eq!(
        recorded_frames, expected,
        "stage-5 event stream must stay byte-equal to the literal recorded from the pre-M28-T2 tree (I6: same kinds, same order, same payloads)"
    );
    let milestone_count = recorded
        .iter()
        .filter(|event| matches!(event, ProgressEvent::LoadMilestone(_)))
        .count();
    assert_eq!(
        milestone_count, 2,
        "the stream must carry exactly SymbolsAnalyzed and ArtifactsSaved"
    );
}

/// I6 per-file clause: exactly one `CollateralFile` event per collateral
/// file plus the terminal `done == total` completion frame, with the
/// `done` values covering `0..=total` exactly once each (the
/// `AtomicUsize::fetch_add` contract). Order-independent by design, so it
/// runs on the global pool (`workers = None`) to also witness the parallel
/// arrival path. Green-first floor (characterizes today's contract).
#[test]
fn test_exactly_one_collateral_event_per_file_plus_terminal_frame() {
    let dir = setup_repo();
    let (events, cb) = recorder();
    let mut pipeline = Pipeline::new(dir.path(), None)
        .unwrap()
        .with_progress(cb);
    pipeline.ensure_load().unwrap();

    let recorded = events.lock().unwrap();
    let file_events: Vec<(usize, usize)> = recorded
        .iter()
        .filter_map(|event| match event {
            ProgressEvent::CollateralFile { done, total } => Some((*done, *total)),
            _ => None,
        })
        .collect();
    assert!(
        !file_events.is_empty(),
        "a cold rebuild must emit CollateralFile events"
    );
    let total = file_events[file_events.len() - 1].1;
    for (_done, event_total) in &file_events {
        assert_eq!(
            *event_total, total,
            "every CollateralFile frame must carry the same total"
        );
    }
    let terminal = file_events[file_events.len() - 1];
    assert_eq!(
        terminal.0, terminal.1,
        "the last file frame must be the terminal done == total completion"
    );
    assert_eq!(
        file_events.len(),
        total + 1,
        "exactly one CollateralFile per collateral file, plus the terminal done == total frame"
    );
    let mut done_values: Vec<usize> = file_events.iter().map(|(done, _)| *done).collect();
    done_values.sort();
    let expected_done_values: Vec<usize> = (0..=total).collect();
    assert_eq!(
        done_values, expected_done_values,
        "done values must cover 0..=total exactly once each (fetch_add uniqueness + one caller-side terminal)"
    );
}

/// (b) release assertion, public witness (M28 T2): after a cold
/// `ensure_load`, a later `ensure_index` must take the fingerprint-fresh
/// DISK branch - `RunStats` then records the index stage as `Loaded`. On
/// the pre-fix tree the resident in-memory cache still answers the second
/// `ensure_index` (the stage-2 `Rebuilt` record survives untouched), i.e.
/// "index result still resident through stage 5". This is the machine
/// check that the release neither breaks nor skips the reload contract.
#[test]
fn test_cold_stage_five_releases_index_and_reloads_fresh() {
    let dir = setup_repo();
    let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
    pipeline.ensure_load().unwrap();

    pipeline.ensure_index().unwrap();
    assert_eq!(
        pipeline.stats.stage_reuse(STAGE_INDEX),
        Some(StageReuse::Loaded),
        "index result still resident through stage 5: the ensure_index AFTER the cold ensure_load hit the in-memory cache instead of reloading fingerprint-fresh from disk (RunStats still records the stage-2 Rebuilt; A5 residency means the release never happened)"
    );
    let index = pipeline
        .get_or_build_index()
        .expect("the post-release index stage must still serve the same public contract");
    assert_eq!(
        index.tu_analyses.len(),
        2,
        "the fingerprint-fresh reload must carry both fixture translation units"
    );
}
