//! boot-progress-visibility V3: pipeline-side boot-event locks.
//!
//! 1. `new_with_progress` records `BootMilestone::DiscoveryHashed` per
//!    hashed file during construction (repo V2 hook), while the
//!    `Pipeline::new` path stays a behavioral no-op: it buffers nothing and
//!    replays nothing through a later `with_progress` recorder (G1-5
//!    exemption basis).
//! 2. `BootMilestone::IncludeScan` fires only on the include-graph COLD
//!    rebuild branch - the four real substeps, monotone, `total` 4 - and
//!    the warm-load branch emits nothing.
//! 3. The single-threaded index rebuild fires `TuStarted` exactly once per
//!    TU (R7) carrying the repo-relative source path verbatim (I10), and
//!    the terminal `TuIndex { done: total, total }` stays.

use ariadne_pipeline::{BootMilestone, Pipeline, ProgressEvent};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tempfile::TempDir;

/// Events recorded through the Send `FnMut` sink shared with the test.
type RecordedEvents = Arc<Mutex<Vec<ProgressEvent>>>;

fn record_into(recorder: RecordedEvents) -> impl FnMut(&ProgressEvent) + Send {
    move |event| recorder.lock().unwrap().push(event.clone())
}

fn recorded(recorder: &RecordedEvents) -> Vec<ProgressEvent> {
    recorder.lock().unwrap().clone()
}

/// Two-header/two-TU repo modeled on `pipeline_test.rs::setup_repo`, with
/// one deliberate difference: the compile-database `directory` carries the
/// CANONICAL temp-root string. The pipeline locates a repo through
/// `std::fs::canonicalize` (`\\?\`-prefixed on Windows), so only a
/// canonical-string directory makes the TU `source_path` strip to a
/// repo-relative path on this OS - exactly the condition the
/// `TuStarted.source` lock below asserts.
fn setup_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("include/common.h"), "struct Common { int id; };").unwrap();
    fs::write(
        root.join("include/header_a.h"),
        "#include \"common.h\"\nstruct ThingA { Common c; };",
    )
    .unwrap();
    fs::write(
        root.join("include/header_b.h"),
        "struct ThingB { double x; };",
    )
    .unwrap();
    fs::write(
        root.join("src/tu1.cpp"),
        "#include \"header_a.h\"\nvoid use_a(ThingA a) {}",
    )
    .unwrap();
    fs::write(
        root.join("src/tu2.cpp"),
        "#include \"header_b.h\"\nvoid use_b(ThingB b) {}",
    )
    .unwrap();

    let canonical_root = fs::canonicalize(root).unwrap();
    let root_str = canonical_root.to_string_lossy().replace('\\', "\\\\");
    let compile_commands = format!(
        r#"[
    {{ "directory": "{root_str}", "command": "clang++ -Iinclude -c src/tu1.cpp", "file": "src/tu1.cpp" }},
    {{ "directory": "{root_str}", "command": "clang++ -Iinclude -c src/tu2.cpp", "file": "src/tu2.cpp" }}
]"#,
        root_str = root_str
    );
    fs::write(root.join("compile_commands.json"), compile_commands).unwrap();
    dir
}

fn discovery_hashed_counts(events: &[ProgressEvent]) -> Vec<(usize, Option<usize>)> {
    events
        .iter()
        .filter_map(|event| match event {
            ProgressEvent::BootMilestone(BootMilestone::DiscoveryHashed { done, total }) => {
                Some((*done, *total))
            }
            _ => None,
        })
        .collect()
}

fn include_scan_counts(events: &[ProgressEvent]) -> Vec<(usize, usize)> {
    events
        .iter()
        .filter_map(|event| match event {
            ProgressEvent::BootMilestone(BootMilestone::IncludeScan { done, total }) => {
                Some((*done, *total))
            }
            _ => None,
        })
        .collect()
}

fn tu_started(events: &[ProgressEvent]) -> Vec<(usize, usize, PathBuf)> {
    events
        .iter()
        .filter_map(|event| match event {
            ProgressEvent::TuStarted { done, total, source } => {
                Some((*done, *total, source.clone()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn test_discovery_hashed_events_fire_during_construction() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let events: RecordedEvents = Arc::new(Mutex::new(Vec::new()));
    let pipeline =
        Pipeline::new_with_progress(&root, None, Box::new(record_into(events.clone()))).unwrap();

    let hashed = discovery_hashed_counts(&recorded(&events));
    assert!(
        !hashed.is_empty(),
        "new_with_progress must record >= 1 DiscoveryHashed during construction, got 0"
    );
    for pair in hashed.windows(2) {
        assert!(
            pair[0].0 < pair[1].0,
            "done must strictly increase: {} then {}",
            pair[0].0,
            pair[1].0
        );
    }
    assert_eq!(
        hashed.last().unwrap().0,
        pipeline.discovery.files.len(),
        "last done must equal the discovered file count"
    );
    assert!(
        hashed.iter().all(|(_, total)| total.is_none()),
        "the single-pass walk has no bounded total: every event must carry None"
    );

    // Behavioral no-op lock (G1-5 exemption basis): the `Pipeline::new`
    // path has no callback during construction, must not buffer or replay
    // discovery milestones, and a recorder attached afterwards through
    // `with_progress` may see zero BootMilestone events over a real stage.
    // The index stage is the run to observe: unlike a cold include rebuild
    // (whose own stage-time IncludeScan is test 2's subject), it emits no
    // BootMilestone by design, so any BootMilestone seen here could only be
    // a buffered/replayed construction milestone - the exact leak this locks.
    let plain_events: RecordedEvents = Arc::new(Mutex::new(Vec::new()));
    let mut plain = Pipeline::new(&root, None)
        .unwrap()
        .with_progress(record_into(plain_events.clone()));
    plain.ensure_index().unwrap();
    assert!(
        !recorded(&plain_events)
            .iter()
            .any(|event| matches!(event, ProgressEvent::BootMilestone(_))),
        "Pipeline::new must emit no BootMilestone events (behavioral no-op)"
    );
}

#[test]
fn test_include_scan_events_cold_only_and_monotone() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let cold_events: RecordedEvents = Arc::new(Mutex::new(Vec::new()));
    let mut cold =
        Pipeline::new_with_progress(&root, None, Box::new(record_into(cold_events.clone())))
            .unwrap();
    cold.ensure_include_graph().unwrap();

    assert_eq!(
        include_scan_counts(&recorded(&cold_events)),
        vec![(1, 4), (2, 4), (3, 4), (4, 4)],
        "cold rebuild must fire the four real substeps, monotone, total 4"
    );
    drop(cold);

    // Warm: a fresh pipeline over the same artifacts loads, and the warm
    // branch emits nothing (warm runs already jump honestly).
    let warm_events: RecordedEvents = Arc::new(Mutex::new(Vec::new()));
    let mut warm =
        Pipeline::new_with_progress(&root, None, Box::new(record_into(warm_events.clone())))
            .unwrap();
    warm.ensure_include_graph().unwrap();
    assert_eq!(
        include_scan_counts(&recorded(&warm_events)),
        Vec::new(),
        "the warm include-load branch must emit zero IncludeScan events"
    );
}

#[test]
fn test_tu_started_fires_exactly_once_per_tu_with_relative_path() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let events: RecordedEvents = Arc::new(Mutex::new(Vec::new()));
    let mut cold =
        Pipeline::new_with_progress(&root, None, Box::new(record_into(events.clone()))).unwrap();
    cold.ensure_index().unwrap();

    let recorded_events = recorded(&events);
    let tu_count = cold.build.translation_units.len();
    assert_eq!(tu_count, 2, "fixture must carry exactly two TUs");

    let started = tu_started(&recorded_events);
    assert_eq!(
        started.len(),
        tu_count,
        "R7: exactly ONE TuStarted per TU (replace, not add): got {} for {tu_count} TUs",
        started.len()
    );
    let done_counts: Vec<usize> = started.iter().map(|(done, _, _)| *done).collect();
    assert_eq!(done_counts, vec![0, 1], "done runs 0.. over the TU order");
    for (done, total, source) in &started {
        assert_eq!(*total, tu_count);
        let tu = &cold.build.translation_units[*done];
        let expected_relative = Path::new("src").join(tu.source_path.file_name().unwrap());
        assert_eq!(
            source, &expected_relative,
            "I10: TuStarted.source must be the repo-relative TU path verbatim"
        );
    }

    let terminal = recorded_events
        .last()
        .expect("the index stage must end on the terminal TuIndex event");
    match terminal {
        ProgressEvent::TuIndex { done, total } => assert_eq!(
            (*done, *total),
            (tu_count, tu_count),
            "the terminal TuIndex {{ done: total, total }} stays"
        ),
        other => panic!("expected terminal TuIndex, got {other:?}"),
    }
}
