//! M22 D22 witness: the plain `build_dependency_graph` entry selects its
//! unit shaper from the repository root's build markers — an sbt repo is
//! shaped through the second-registered plugin's shaper with NO injected
//! shaper. Before D22 the entry used the registry's first-wins selection
//! (C++ registers first), which recognized none of this repo's markers and
//! shaped zero units, so the walk below produced an edgeless graph.

use ariadne_graph::{
    build_dependency_graph, build_dependency_graph_with_unit_shaper, IncludeGraph, IncludeKind,
};
use ariadne_lang::LanguageRegistry;
use ariadne_repo::{FileId, SourceLanguage};
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE: &str = "scala-basic";
const ORDER_REL: &str = "src/main/scala/com/example/models/Order.scala";
const ORDER_SERVICE_REL: &str = "src/main/scala/com/example/services/OrderService.scala";

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(FIXTURE)
}

/// Throwaway copy of the fixture — never build inside the repo fixture dir
/// (the m19/m20 isolation pattern).
fn isolated_repo_copy(label: &str) -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().expect("temp repo");
    let root = temp.path().join(label);
    copy_tree(&fixture_path(), &root).expect("copy fixture");
    (temp, root)
}

fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
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

/// Mirrors the production default registry's registration order (D9
/// contract: C++ first, Scala second); the production enumeration itself
/// lives in `ariadne_pipeline::build_default_registry`, which depends on
/// graph and so cannot be a graph dev-dependency.
fn production_order_registry() -> LanguageRegistry {
    LanguageRegistry::new()
        .register(Box::new(ariadne_lang_cpp::CppPlugin::new()))
        .register(Box::new(ariadne_lang_scala::ScalaPlugin::new()))
}

#[test]
fn unfiltered_graph_on_an_sbt_repo_auto_selects_the_scala_shaper() {
    let (_temp, root) = isolated_repo_copy("auto-select");
    let graph = build_dependency_graph(&root, &production_order_registry()).expect("build");

    let service = FileId::from_relative_path(ORDER_SERVICE_REL);
    let order = FileId::from_relative_path(ORDER_REL);
    let named_import = graph.edges.iter().any(|edge| {
        edge.source == service
            && edge.destination == Some(order)
            && edge.kind == IncludeKind::Import
            && edge.language == SourceLanguage::Scala
    });
    assert!(
        named_import,
        "the plain entry point walks the sbt repo without an injected shaper \
         ({} edges, expected an Import edge {ORDER_SERVICE_REL} -> {ORDER_REL})",
        graph.edges.len()
    );
}

#[test]
fn auto_selected_graph_matches_the_injected_scala_shaper_graph() {
    let (_auto_temp, auto_root) = isolated_repo_copy("auto");
    let (_inj_temp, inj_root) = isolated_repo_copy("injected");
    let registry = production_order_registry();

    let auto = build_dependency_graph(&auto_root, &registry).expect("auto build");
    let injected = build_dependency_graph_with_unit_shaper(
        &inj_root,
        &registry,
        &ariadne_lang_scala::ScalaUnitShaper,
    )
    .expect("injected build");

    assert!(
        !auto.edges.is_empty(),
        "the auto-selected shaper seeds the walk at all"
    );
    assert_eq!(
        edge_shape(&auto),
        edge_shape(&injected),
        "marker-based auto-selection lands on exactly the shaper m20 injects"
    );
}

/// Comparable projection of a graph's edges (source, destination, kind).
fn edge_shape(graph: &IncludeGraph) -> Vec<(u64, Option<u64>, String)> {
    let mut shape: Vec<(u64, Option<u64>, String)> = graph
        .edges
        .iter()
        .map(|edge| {
            (
                edge.source.0,
                edge.destination.map(|id| id.0),
                format!("{:?}", edge.kind),
            )
        })
        .collect();
    shape.sort();
    shape
}
