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
}
