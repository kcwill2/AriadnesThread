//! UTA-T1 item 7 (hand H1): locks the `generate_report` top-headers branch
//! (crates/graph/src/graph.rs): ordering (closure size DESC, display path
//! ASC), the take(10) truncation, and the `{:016x}` display-path fallback
//! for ids that exist only in edges (never registered).

use ariadne_graph::{IncludeEdge, IncludeGraph, IncludeKind, LanguageId, SourceLocation};
use ariadne_repo::FileId;
use std::path::PathBuf;

fn include_edge(source: FileId, destination: Option<FileId>) -> IncludeEdge {
    IncludeEdge {
        source,
        destination,
        context_name: "report".into(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: "leaf.h".into(),
        resolved_path: None,
        language: LanguageId::cpp(),
    }
}

#[test]
fn top_headers_sort_size_desc_path_asc_truncate_to_ten_and_hex_project_edge_only_ids() {
    let mut graph = IncludeGraph::new();
    let leaf = FileId(500);
    graph.register_file(leaf, PathBuf::from("zzz/leaf.h"));
    // Twelve registered rows each include the shared leaf: 12 TIED closure
    // sizes, discriminating only by path.
    for row in 1..=12u64 {
        let id = FileId(row);
        graph.register_file(id, PathBuf::from(format!("rows/r{row:02}.h")));
        graph.add_edge(include_edge(id, Some(leaf)));
    }
    // FileId(900) is an EDGE-ONLY id: never registered, only an edge
    // source - its display path must project through the {:016x} fallback.
    let edge_only = FileId(900);
    graph.add_edge(include_edge(edge_only, Some(leaf)));

    let report = graph.generate_report();
    assert_eq!(
        report.top_headers_by_closure.len(),
        10,
        "the branch truncates to ten of the thirteen size-1 rows"
    );

    // All thirteen size-1 rows tie on size: path order puts the hex
    // fallback ("0000...", leading '0') BEFORE every "rows/..." path, then
    // r01..r12 ascending; the size-0 leaf never enters the top ten.
    let mut expected = vec![(
        edge_only,
        PathBuf::from(format!("{:016x}", edge_only.0)),
        1usize,
    )];
    expected.extend(
        (1..=9u64).map(|row| {
            (
                FileId(row),
                PathBuf::from(format!("rows/r{row:02}.h")),
                1usize,
            )
        }),
    );
    assert_eq!(report.top_headers_by_closure, expected);
    assert!(
        !report
            .top_headers_by_closure
            .iter()
            .any(|(_, path, _)| *path == PathBuf::from("zzz/leaf.h")),
        "the zero-closure leaf must not displace a size-1 row"
    );
}
