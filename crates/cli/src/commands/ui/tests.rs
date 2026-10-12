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

// AR-02 (S6): the web UI binds localhost by default; `--expose <addr>` is the
// single opt-in to a wider bind address, and using it must warn at startup.

#[test]
fn test_resolve_bind_addr_defaults_to_localhost() {
    assert_eq!(super::resolve_bind_addr(None), "127.0.0.1");
    assert_eq!(
        super::resolve_bind_addr(Some("   ")),
        "127.0.0.1",
        "a blank --expose value must not fall outside the localhost default"
    );
}

#[test]
fn test_resolve_bind_addr_honors_expose_override() {
    assert_eq!(super::resolve_bind_addr(Some("0.0.0.0")), "0.0.0.0");
    assert_eq!(super::resolve_bind_addr(Some(" 10.0.0.5 ")), "10.0.0.5");
}

#[test]
fn test_run_ui_binds_resolved_addr_never_hardcoded_wildcard() {
    let ui_code = include_str!("../ui.rs");
    let production_end = ui_code
        .find("#[cfg(test)]")
        .expect("ui.rs must have a test module to cut the production section at");
    let production = &ui_code[..production_end];
    assert!(
        production.contains("TcpListener::bind(format!(\"{bind_addr}:{port}\"))"),
        "run_ui must bind the --expose-resolved address, not a hardcoded host"
    );
    assert!(
        !production.contains("0.0.0.0"),
        "production ui.rs must not hardcode the wildcard bind address (S6)"
    );
    assert!(
        production.contains("warning:") && production.contains("--expose"),
        "--expose use must print a LAN-exposure warning at startup (S6)"
    );
}
