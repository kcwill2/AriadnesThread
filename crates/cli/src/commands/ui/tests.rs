#[test]
fn test_ui_is_artifact_first() {
    let ui_code = include_str!("../ui.rs");
    assert!(ui_code.contains("Pipeline::new"));
    assert!(ui_code.contains("ensure_include_graph"));
    assert!(ui_code.contains("get_or_build_callgraph"));
    assert!(ui_code.contains("get_or_build_load"));
    assert!(ui_code.contains("spawn_watcher"));

    let index_code = include_str!("../index.rs");
    assert!(index_code.contains("ProgressEvent::TuIndex"));
    assert!(index_code.contains("print_progress"));

    // The extraction-proposal builder lives inline in `ui.rs` on this branch,
    // so the A5 file-split greps cannot apply. The A1 invariant still can: the
    // UI must screen candidates from the stage's load metrics through the
    // facts constructor instead of building its own planner from scratch.
    assert!(
        ui_code.contains("with_screening_facts"),
        "ui.rs must carry the A1 screening-facts planner wiring"
    );
    assert!(
        ui_code.contains("screening_facts_from_metrics"),
        "ui.rs must build screening facts from the stage file metrics"
    );
}
