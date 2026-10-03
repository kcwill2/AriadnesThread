//! M20 acceptance suite (plan:398, kickoff acceptance #2): the Scala
//! fixture graph built through the REAL registry path —
//! `build_dependency_graph_with_unit_shaper` + the CLI `default_registry`
//! + `ScalaUnitShaper` (D14) — locks the Import edge kind (IV-1), the
//! External wildcard-boundary edge (IV-2), the deliberate models <->
//! services cycle classified informational only under a tolerate policy
//! (IV-3, D11/D13), and the untouched external boundary (IV-2, mirroring
//! m19's `external_frozen_boundary_lock`).

use anyhow::Result;
use ariadne_graph::{build_dependency_graph_with_unit_shaper, IncludeGraph, IncludeKind};
use ariadne_lang_api::modeling::CyclePolicy;
use ariadne_lang_scala::ScalaUnitShaper;
use ariadne_repo::{FileId, SourceLanguage};
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE: &str = "scala-basic";

const ORDER_REL: &str = "src/main/scala/com/example/models/Order.scala";
const USER_REL: &str = "src/main/scala/com/example/models/User.scala";
const ORDER_SERVICE_REL: &str = "src/main/scala/com/example/services/OrderService.scala";

/// The resolver strips the `._` spelling from a wildcard edge, so the raw
/// path on the External edge is the package, not the directive text.
const MUTABLE_PACKAGE: &str = "scala.collection.mutable";

struct IsolatedScalaRepo {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fixtures")
        .join(name)
}

/// Throwaway copy of the fixture — never build inside the repo fixture dir
/// (the m19 isolation pattern).
fn isolated_scala_repo(label: &str) -> Result<IsolatedScalaRepo> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join(label);
    copy_tree(&fixture_path(FIXTURE), &root)?;
    Ok(IsolatedScalaRepo { _temp: temp, root })
}

fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn normalized_rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// The plan:398 production path: default registry, explicitly injected Scala
/// shaper (the registry's first-wins shaper selection stays C++-bound).
fn scala_graph(root: &Path) -> Result<IncludeGraph> {
    let registry = ariadne_cli::registry::default_registry();
    build_dependency_graph_with_unit_shaper(root, &registry, &ScalaUnitShaper)
}

fn rel_of(graph: &IncludeGraph, id: FileId) -> Option<String> {
    graph
        .file_paths
        .get(&id)
        .map(|path| normalized_rel(path))
}

fn is_boundary_rel(rel: &str) -> bool {
    rel.starts_with("lib/") || rel.starts_with("target/")
}

#[test]
fn graph_emits_import_edges_for_order_service() -> Result<()> {
    let repo = isolated_scala_repo("edges")?;
    let graph = scala_graph(&repo.root)?;
    let service = FileId::from_relative_path(ORDER_SERVICE_REL);

    for expected_dest in [ORDER_REL, USER_REL] {
        let matching: Vec<_> = graph
            .edges
            .iter()
            .filter(|edge| {
                edge.source == service
                    && edge
                        .destination
                        .and_then(|dest| rel_of(&graph, dest))
                        .as_deref()
                        == Some(expected_dest)
            })
            .collect();
        assert_eq!(
            matching.len(),
            1,
            "exactly one edge resolves to {expected_dest}"
        );
        let edge = matching[0];
        assert!(
            edge.destination.is_some(),
            "the named directive resolves to a repository file"
        );
        assert_eq!(edge.kind, IncludeKind::Import, "plan:398 edge kind");
        assert_eq!(edge.language, SourceLanguage::Scala.into());
    }
    Ok(())
}

#[test]
fn wildcard_package_import_is_external() -> Result<()> {
    let repo = isolated_scala_repo("wildcard")?;
    let graph = scala_graph(&repo.root)?;
    let service = FileId::from_relative_path(ORDER_SERVICE_REL);

    let matching: Vec<_> = graph
        .edges
        .iter()
        .filter(|edge| edge.source == service && edge.raw_path == MUTABLE_PACKAGE)
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "`import scala.collection.mutable._` yields exactly one edge, spelled with the package"
    );
    assert_eq!(
        matching[0].kind,
        IncludeKind::External,
        "an out-of-repo package prefix is an External edge (D10b)"
    );
    assert_eq!(
        matching[0].destination, None,
        "External is an honest boundary marker: never a destination"
    );
    assert_eq!(matching[0].language, SourceLanguage::Scala.into());
    Ok(())
}

#[test]
fn fixture_cycle_is_informational_under_tolerate_policy() -> Result<()> {
    let repo = isolated_scala_repo("cycle-policy")?;
    let graph = scala_graph(&repo.root)?;

    let cycles = graph.detect_cycles();
    assert!(
        cycles.len() >= 1,
        "the deliberate models <-> services fixture cycle is detected"
    );

    let tolerated =
        graph.generate_report_with_cycle_policies(&|_file_id| CyclePolicy::TolerateWithinPackage);
    assert!(
        tolerated.informational_cycle_count >= 1,
        "IV-3: the cycle is non-blocking when every member file tolerates"
    );
    assert_eq!(
        tolerated.informational_cycle_count, tolerated.cycle_count,
        "an all-tolerate stance makes every detected cycle informational"
    );

    let blocking = graph.generate_report();
    assert_eq!(
        blocking.informational_cycle_count, 0,
        "the all-Reject delegate classifies nothing informational"
    );
    assert_eq!(
        blocking.cycle_count, tolerated.cycle_count,
        "cycle_count keeps counting every detected cycle (m3 lock)"
    );

    let blocking_json = serde_json::to_string(&blocking).expect("report serializes");
    assert!(
        !blocking_json.contains("informational_cycle_count"),
        "skip-when-zero keeps the pre-M20 artifact shape: {blocking_json}"
    );
    let tolerated_json = serde_json::to_string(&tolerated).expect("report serializes");
    assert!(
        tolerated_json.contains("informational_cycle_count"),
        "a non-zero informational count surfaces in the artifact: {tolerated_json}"
    );
    Ok(())
}

#[test]
fn no_edges_point_into_external_boundary() -> Result<()> {
    let repo = isolated_scala_repo("boundary")?;
    // Same materialization as m19's boundary lock: even with a real
    // generated file on disk, neither boundary dir may enter the graph.
    let generated = repo
        .root
        .join("target")
        .join("scala-2.13")
        .join("classes")
        .join("Generated.scala");
    fs::create_dir_all(generated.parent().unwrap())?;
    fs::write(&generated, "// generated by a hypothetical sbt run\n")?;

    let graph = scala_graph(&repo.root)?;
    for (id, path) in &graph.file_paths {
        let rel = normalized_rel(path);
        assert!(
            !is_boundary_rel(&rel),
            "no graph record lives under an external boundary dir: {rel} (id {})",
            id.0
        );
    }
    for edge in &graph.edges {
        let mut spellings: Vec<String> = edge
            .destination
            .and_then(|dest| rel_of(&graph, dest))
            .into_iter()
            .collect();
        if let Some(resolved) = &edge.resolved_path {
            spellings.push(normalized_rel(resolved));
        }
        for spelling in spellings {
            assert!(
                !is_boundary_rel(&spelling),
                "no edge destination points into an external boundary dir: {spelling}"
            );
        }
    }
    Ok(())
}

#[test]
fn deliberate_cycle_is_import_edges_in_both_directions() -> Result<()> {
    let repo = isolated_scala_repo("cycle-edges")?;
    let graph = scala_graph(&repo.root)?;
    let order = FileId::from_relative_path(ORDER_REL);
    let service = FileId::from_relative_path(ORDER_SERVICE_REL);

    assert_import_edge(&graph, service, order);
    assert_import_edge(&graph, order, service);
    Ok(())
}

fn assert_import_edge(graph: &IncludeGraph, source: FileId, destination: FileId) {
    let found = graph.edges.iter().any(|edge| {
        edge.source == source && edge.destination == Some(destination) && edge.kind == IncludeKind::Import
    });
    assert!(
        found,
        "expected an Import edge {} -> {}",
        source.0,
        destination.0
    );
}
