//! UTA-T1 item 8 (hand H1): byte-snapshot of `IncludeGraph::format_explain`
//! (graph.rs `format_explain_node`) on a tree that exercises every renderer
//! branch: a two-child root (branch vs corner connectors, child-prefix
//! continuation), a nested single child, and a 2-cycle rendered once with
//! the " (cycle)" marker instead of recursion. Box-drawing characters are
//! written as unicode escapes so this source stays pure ASCII.

use ariadne_graph::{IncludeEdge, IncludeGraph, IncludeKind, LanguageId, SourceLocation};
use ariadne_repo::FileId;
use std::path::PathBuf;

const VERTICAL: char = '\u{2502}'; // | tree continuation
const BRANCH: char = '\u{251C}'; // +- mid-tree connector
const CORNER: char = '\u{2514}'; // +- last-child connector
const DASH: char = '\u{2500}'; // - horizontal bar

fn include_edge(source: FileId, destination: FileId) -> IncludeEdge {
    IncludeEdge {
        source,
        destination: Some(destination),
        context_name: "explain".into(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: "target".into(),
        resolved_path: None,
        language: LanguageId::cpp(),
    }
}

/// x.h -> {y.h, z.h} (two-child root); y.h -> w.h; w.h -> {y.h (cycle), v.h}
/// so the y<->w 2-cycle is reported once, marked, never descended.
fn cycle_tree_graph() -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    for (id, path) in [
        (1u64, "x.h"),
        (2, "y.h"),
        (3, "z.h"),
        (4, "w.h"),
        (5, "v.h"),
    ] {
        graph.register_file(FileId(id), PathBuf::from(path));
    }
    for (source, destination) in [(1, 2), (1, 3), (2, 4), (4, 2), (4, 5)] {
        graph.add_edge(include_edge(FileId(source), FileId(destination)));
    }
    graph
}

#[test]
fn format_explain_snapshot_locks_connectors_prefixes_and_cycle_marker() {
    let tree = cycle_tree_graph().format_explain(FileId(1));
    let expected = format!(
        "x.h\n\
         {branch}{dash}{dash} y.h\n\
         {vertical}   {corner}{dash}{dash} w.h\n\
         {vertical}       {branch}{dash}{dash} y.h (cycle)\n\
         {vertical}       {corner}{dash}{dash} v.h\n\
         {corner}{dash}{dash} z.h\n",
        branch = BRANCH,
        corner = CORNER,
        dash = DASH,
        vertical = VERTICAL,
    );
    assert_eq!(tree, expected);
}
