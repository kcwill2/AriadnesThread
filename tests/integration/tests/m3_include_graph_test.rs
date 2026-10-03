use ariadne_cli::{analyze_include_graph, explain_include_graph_for_file};
use ariadne_graph::IncludeKind;
use ariadne_pipeline::Pipeline;
use ariadne_repo::FileId;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn get_basic_fixture_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("../fixtures/basic")
}

#[test]
fn test_m3_basic_fixture_include_graph() {
    let repo_root = get_basic_fixture_dir();
    assert!(repo_root.exists(), "basic fixture path must exist");

    let mut pipeline =
        Pipeline::new(&repo_root, None).expect("pipeline init failed for basic fixture");
    let (graph, closures) = pipeline
        .get_or_build_include_graph()
        .expect("build_include_graph failed for basic fixture");
    let repo = &pipeline.repo;
    let report = graph.generate_report();

    assert_eq!(
        repo.root.canonicalize().unwrap(),
        repo_root.canonicalize().unwrap()
    );
    assert!(
        report.total_edges >= 3,
        "Fixture must contain at least 3 include edges"
    );

    let id_main = FileId::from_relative_path("src/thing.cpp");
    let id_thing = FileId::from_relative_path("include/thing.h");
    let id_types = FileId::from_relative_path("include/thing_types.h");
    let id_heavy = FileId::from_relative_path("include/heavy.h");

    let thing_closure = closures
        .get(&id_thing)
        .expect("Closure for thing.h must exist");
    assert_eq!(
        thing_closure.closure_size, 2,
        "thing.h closure must include thing_types.h and heavy.h"
    );
    assert!(thing_closure.transitive_includes.contains(&id_types));
    assert!(thing_closure.transitive_includes.contains(&id_heavy));

    let main_closure = closures
        .get(&id_main)
        .expect("Closure for thing.cpp must exist");
    assert_eq!(
        main_closure.closure_size, 3,
        "thing.cpp closure must include thing.h, thing_types.h, heavy.h"
    );

    // Check artifacts created by explicitly saving them
    let artifacts = ariadne_report::ArtifactManager::prepare_for_repo(&repo).unwrap();
    artifacts
        .save_include_artifacts(&graph, &closures, &report, "m3-test-fingerprint")
        .unwrap();
    let artifact_dir = repo_root.join("artifacts/includes");
    assert!(
        artifact_dir.join("edges.bin").exists(),
        "edges.bin must exist"
    );
    assert!(
        artifact_dir.join("closures.bin").exists(),
        "closures.bin must exist"
    );
    assert!(
        artifact_dir.join("report.json").exists(),
        "report.json must exist"
    );
}

#[test]
fn test_m3_edge_cases_duplicates_system_missing_cycles() {
    let dir = tempdir().expect("Failed to create tempdir");
    let root = dir.path();

    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    // Cyclic includes: a.h -> b.h and b.h -> a.h
    fs::write(
        root.join("include/a.h"),
        "#pragma once\n#include \"b.h\"\n#include <vector>\n#include \"missing.h\"",
    )
    .unwrap();
    fs::write(
        root.join("include/b.h"),
        "#pragma once\n#include \"a.h\"\n#include \"a.h\"",
    )
    .unwrap(); // duplicate include
    fs::write(root.join("src/main.cpp"), "#include \"a.h\"").unwrap();

    let compile_commands_json = format!(
        r#"[
            {{
                "directory": "{}",
                "command": "cl.exe /c /Iinclude src/main.cpp",
                "file": "src/main.cpp"
            }}
        ]"#,
        root.display().to_string().replace('\\', "/")
    );
    fs::write(root.join("compile_commands.json"), compile_commands_json).unwrap();

    let mut pipeline = Pipeline::new(&root, None).expect("pipeline init failed");
    let (graph, closures) = pipeline
        .get_or_build_include_graph()
        .expect("build_include_graph failed");
    let report = graph.generate_report();

    assert!(
        report.cycle_count > 0,
        "Cycle between a.h and b.h must be detected"
    );
    assert!(
        report.system_headers_count > 0,
        "System header <vector> must be recorded"
    );
    assert!(
        report.unresolved_includes_count > 0,
        "Missing header missing.h must be recorded"
    );

    let id_a = FileId::from_relative_path("include/a.h");
    let id_b = FileId::from_relative_path("include/b.h");

    let closure_a = closures.get(&id_a).expect("Closure for a.h must exist");
    assert!(closure_a.transitive_includes.contains(&id_b));

    // Verify system edge
    let sys_edge = graph
        .edges
        .iter()
        .find(|e| e.raw_path == "vector")
        .expect("System include vector must exist");
    assert_eq!(sys_edge.kind, IncludeKind::System);
}

#[test]
fn test_m3_cli_commands() {
    let repo_root = get_basic_fixture_dir();
    let res = analyze_include_graph(&repo_root);
    assert!(res.is_ok(), "handle_includes should succeed");

    let explain_res = explain_include_graph_for_file(&repo_root, &PathBuf::from("include/thing.h"));
    assert!(
        explain_res.is_ok(),
        "handle_includes_explain should succeed"
    );
}
