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

// UTA-T7b row 1: build_file_collateral behavior pins (LoadMetric fixture).

fn collateral_source(rel: &str, symbol_count: usize) -> ariadne_graph::ContributingSource {
    ariadne_graph::ContributingSource {
        file_id: ariadne_repo::FileId::from_relative_path(rel),
        path: std::path::PathBuf::from(rel),
        symbol_count,
        symbols: Vec::new(),
    }
}

fn load_metric_with(
    target_file: Option<&str>,
    sources: Vec<ariadne_graph::ContributingSource>,
) -> ariadne_graph::LoadMetric {
    ariadne_graph::LoadMetric {
        target_name: "pinned.target".to_string(),
        target_file: target_file.map(std::path::PathBuf::from),
        semantic_load: 0,
        include_load: 0,
        collateral_load: sources.iter().map(|s| s.symbol_count).sum(),
        total_load: 0,
        semantic_symbols: Vec::new(),
        collateral_symbols: Vec::new(),
        collateral_sources: sources,
        largest_collateral_source: None,
        semantic_sources: Vec::new(),
    }
}

#[test]
fn pin_build_file_collateral_drops_zeroes_skips_unmapped_and_keys_by_id_string() {
    // Handoff row 1: symbol_count==0 dropped, target_file None skipped,
    // key = id string. Outer keys are the NUMERIC id strings that
    // `path_to_id` stores (never the path text), inner keys the collateral
    // FileId's numeric string.
    let mut path_to_id: std::collections::HashMap<std::path::PathBuf, u64> =
        std::collections::HashMap::new();
    path_to_id.insert(std::path::PathBuf::from("include/target.h"), 42);
    path_to_id.insert(std::path::PathBuf::from("include/other.h"), 43);

    let metrics = vec![
        // target_file None -> skipped even with live collateral.
        load_metric_with(None, vec![collateral_source("src/kept.h", 3)]),
        // target present but not in path_to_id -> skipped.
        load_metric_with(
            Some("include/ghost.h"),
            vec![collateral_source("src/kept.h", 7)],
        ),
        // known target: zero-count source dropped, live source kept.
        load_metric_with(
            Some("include/target.h"),
            vec![
                collateral_source("src/kept.h", 5),
                collateral_source("src/zero.h", 0),
            ],
        ),
        // known target whose ONLY source is zero-count: the whole row
        // disappears (empty inner map is never inserted).
        load_metric_with(
            Some("include/other.h"),
            vec![collateral_source("src/zero2.h", 0)],
        ),
    ];

    let file_collateral = super::build_file_collateral(&path_to_id, &metrics);

    assert_eq!(
        file_collateral.keys().cloned().collect::<Vec<_>>(),
        vec!["42".to_string()],
        "the outer key is the numeric id string from path_to_id, and only \
         the one row with live collateral survives"
    );
    let inner = &file_collateral["42"];
    let kept_key = ariadne_repo::FileId::from_relative_path("src/kept.h")
        .0
        .to_string();
    let zero_key = ariadne_repo::FileId::from_relative_path("src/zero.h")
        .0
        .to_string();
    assert_eq!(inner.get(&kept_key).copied(), Some(5));
    assert!(
        !inner.contains_key(&zero_key),
        "a symbol_count==0 contributor must never appear in the row"
    );
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
