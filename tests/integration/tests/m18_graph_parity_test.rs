//! M18.5 registry-driven include-graph parity proof (plan §5 M18 table +
//! §9 "M18.5"): `build_dependency_graph(repo, &registry)` must reproduce the
//! **pre-M18 BFS** byte-for-byte on BOTH fixtures — same edge list (source,
//! destination, kind, language, context, raw/resolved paths, source location)
//! IN ORDER, same node set, same cycle order, same closures/report.
//!
//! The left-hand side is `legacy_bfs` below: the pre-M18 breadth-first walk
//! vendored verbatim from commit `ae3cae7` (`git show ae3cae7:crates/graph/src/
//! builder.rs`), driven by the transitional `IncludeResolver` shim and the old
//! `cpp_default_shaper()` unit shaping. It is NOT `build_include_graph` — that
//! entry point now *delegates* to the same `dependency_graph` the right-hand
//! side calls, so comparing against it would be a tautology (Finding 2). The
//! m00 golden normalization re-sorts arrays as multisets and so cannot lock the
//! pre-M18 walk's order; this test asserts `Vec<IncludeEdge>` order-sensitively
//! (a reorder, a dropped angle-style/include-dir fallback, or an External-drop
//! divergence fails it — demonstrated by perturbing `graph::builder` and seeing
//! this suite go red).
//!
//! Plus the §3.3 fact 3 negative case: an EMPTY registry honestly produces an
//! edgeless graph, never a silently C++-parsed one. Kept until the M18.11
//! final parity suite absorbs it. Runs on throwaway fixture copies like the
//! M18.3 interim suite.

mod common;

use anyhow::{Context, Result};
use ariadne_build::BuildDiscoverer;
use ariadne_cli::registry::default_registry;
use ariadne_graph::{build_dependency_graph, build_include_graph, IncludeGraph};
use ariadne_lang::LanguageRegistry;
use ariadne_repo::{FileId, Repository};
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

struct IsolatedRepo {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

#[test]
fn registry_dependency_graph_matches_legacy_on_basic() -> Result<()> {
    graph_parity("basic")
}

#[test]
fn registry_dependency_graph_matches_legacy_on_expensive() -> Result<()> {
    graph_parity("expensive")
}

#[test]
fn empty_registry_dependency_graph_is_edgeless_never_cpp() -> Result<()> {
    // §3.3 fact 3 from the graph side: with no plugins the walk has no units
    // and no plugin — an honest edgeless graph, never a hidden C++ default.
    let repo = isolated_repo("basic", "empty")?;
    let empty = build_dependency_graph(&repo.root, &LanguageRegistry::new())?;
    assert!(
        empty.edges.is_empty(),
        "empty registry must not invent C++ edges: {:?}",
        empty.edges
    );
    assert!(empty.detect_cycles().is_empty());

    // Control on the same copy: the C++-capable path DOES see the fixture's
    // edges, so the empty result above is honest emptiness, not a fixture or
    // walk bug.
    let (_repo, legacy, _closures, _report) = build_include_graph(&repo.root)?;
    assert!(
        !legacy.edges.is_empty(),
        "control failed: fixture produced no edges even with the C++ registry"
    );
    Ok(())
}

fn graph_parity(fixture: &str) -> Result<()> {
    let repo = isolated_repo(fixture, "graph")?;

    let registry = default_registry();
    let driven = build_dependency_graph(&repo.root, &registry)?;
    let vendored = legacy_bfs(&repo.root)?;

    assert_eq!(
        vendored.file_paths, driven.file_paths,
        "node set diverged for {fixture}"
    );
    // Order-sensitive: `edges` is a `Vec`, compared element-by-element. The
    // pre-M18 BFS appends edges as it pops the FIFO scan queue, so a reorder or
    // a dropped fallback edge fails here (the golden's array-multiset sort does
    // not catch a pure reorder).
    assert_eq!(
        vendored.edges, driven.edges,
        "edge list ORDER (source/destination/kind/language/location) diverged for {fixture}"
    );
    assert_eq!(
        vendored.detect_cycles(),
        driven.detect_cycles(),
        "cycle ORDER diverged for {fixture}"
    );
    assert_eq!(
        vendored.compute_closures(),
        driven.compute_closures(),
        "closures diverged for {fixture}"
    );
    assert_eq!(
        vendored.generate_report(),
        driven.generate_report(),
        "include report diverged for {fixture}"
    );
    Ok(())
}

/// The pre-M18 breadth-first include-graph walk, vendored verbatim from commit
/// `ae3cae7` (`git show ae3cae7:crates/graph/src/builder.rs`). It is the honest
/// left-hand side for the parity assertion above: the transitional
/// `IncludeResolver` shim (deprecated, removed M18.11) plus the old
/// `cpp_default_shaper()` unit shaping the pre-M18 `BuildDiscoverer` implied.
#[allow(deprecated)]
fn legacy_bfs(root: &Path) -> Result<IncludeGraph> {
    let repository = Repository::locate(root)?;
    let repo_discovery = repository.discover()?;

    let build_result = BuildDiscoverer::new(&repository.root)
        .with_repo_discovery(&repo_discovery)
        .with_unit_shaper(Some(ariadne_lang_cpp::cpp_default_shaper()))
        .discover()?;

    let mut graph = IncludeGraph::new();
    for file_record in &repo_discovery.files {
        graph.register_file(file_record.id, file_record.path.clone());
    }

    let mut scanned_files: HashSet<FileId> = HashSet::new();
    let mut scan_queue: VecDeque<(FileId, PathBuf, Option<String>)> = VecDeque::new();
    for tu in &build_result.translation_units {
        scan_queue.push_back((
            tu.file_id,
            tu.source_path.clone(),
            Some(tu.context_name.clone()),
        ));
    }

    while let Some((file_id, rel_path, ctx_name_opt)) = scan_queue.pop_front() {
        if scanned_files.contains(&file_id) {
            continue;
        }
        scanned_files.insert(file_id);

        let abs_path = repository.root.join(&rel_path);
        let content = match fs::read_to_string(&abs_path) {
            Ok(content) => content,
            Err(_) => continue,
        };

        let target_ctx = ctx_name_opt
            .as_ref()
            .and_then(|name| build_result.contexts.iter().find(|c| &c.name == name))
            .or_else(|| build_result.contexts.first());

        let mut resolver =
            ariadne_graph::IncludeResolver::new(&repository.root).with_repo_discovery(&repo_discovery);
        if let Some(ctx) = target_ctx {
            resolver = resolver.with_context(ctx);
        }

        let edges = resolver.resolve_file_includes(file_id, &rel_path, &content);
        for edge in edges {
            if let Some(dest_id) = edge.destination {
                if !scanned_files.contains(&dest_id) {
                    if let Some(dest_path) = graph.file_paths.get(&dest_id).cloned() {
                        scan_queue.push_back((dest_id, dest_path, ctx_name_opt.clone()));
                    }
                }
            }
            graph.add_edge(edge);
        }
    }

    Ok(graph)
}

/// Copies `tests/fixtures/<name>` into a fresh temp dir (skipping regenerable
/// caches), same cold-run contract as the M18.3 interim suite.
fn isolated_repo(fixture: &str, label: &str) -> Result<IsolatedRepo> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join(format!("{fixture}-{label}"));
    copy_dir_filtered(&common::fixture_path(fixture), &root)
        .with_context(|| format!("failed to copy fixture {fixture} for {label}"))?;
    Ok(IsolatedRepo { _temp: temp, root })
}

fn copy_dir_filtered(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "artifacts" || name_str == ".ariadne" || name_str.starts_with('.') {
            continue;
        }
        let target = dst.join(name);
        if entry.file_type()?.is_dir() {
            copy_dir_filtered(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
