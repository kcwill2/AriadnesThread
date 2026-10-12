//! AR-06 invariant suite: `add_edge` dedup semantics and edge-order parity.
//!
//! These tests lock the contract the O(1) `HashSet` dedup must preserve
//! byte-for-byte from the legacy `Vec::contains` scan: full-struct equality
//! (metadata differences keep same-endpoint edges distinct), first-occurrence
//! insertion order, and a dedup set that never leaks into serialized shapes.

use ariadne_graph::{IncludeEdge, IncludeGraph, IncludeKind, LanguageId, SourceLocation};
use ariadne_repo::FileId;
use std::path::PathBuf;

fn edge(source: u64, destination: u64, line: u32) -> IncludeEdge {
    IncludeEdge {
        source: FileId(source),
        destination: Some(FileId(destination)),
        context_name: "Default".into(),
        source_location: SourceLocation::new(line, 1),
        kind: IncludeKind::Local,
        raw_path: format!("dep{destination}.h"),
        resolved_path: Some(PathBuf::from(format!("include/dep{destination}.h"))),
        language: LanguageId::cpp(),
    }
}

#[test]
fn exact_duplicate_edge_added_twice_yields_one_edge() {
    let mut graph = IncludeGraph::new();
    graph.add_edge(edge(1, 2, 7));
    graph.add_edge(edge(1, 2, 7));
    assert_eq!(
        graph.edges.len(),
        1,
        "an exact-duplicate edge must be rejected"
    );
}

#[test]
fn first_occurrence_insertion_order_is_preserved() {
    let mut graph = IncludeGraph::new();
    let first = edge(1, 2, 1);
    let second = edge(2, 3, 1);
    let third = edge(3, 4, 1);
    graph.add_edge(first.clone());
    graph.add_edge(second.clone());
    graph.add_edge(first.clone());
    graph.add_edge(third.clone());
    graph.add_edge(second.clone());
    assert_eq!(
        graph.edges,
        vec![first, second, third],
        "duplicates must never reorder: the Vec keeps first-occurrence order"
    );
}

#[test]
fn same_endpoints_different_metadata_edges_are_both_kept() {
    // Key-choice lock: dedup is FULL-STRUCT equality, exactly the legacy
    // `Vec::contains` semantics. The builder emits one edge per include
    // directive carrying its own `source_location`, so a file including the
    // same header on two different lines is TWO edges, not one. A
    // `(source, destination)` pair key would silently drop the second.
    let mut graph = IncludeGraph::new();
    graph.add_edge(edge(1, 2, 3));
    graph.add_edge(edge(1, 2, 9));
    assert_eq!(
        graph.edges.len(),
        2,
        "same endpoints with different source_location must both survive"
    );
    assert_eq!(graph.edges[0].source_location.line, 3);
    assert_eq!(graph.edges[1].source_location.line, 9);

    // Metadata variants beyond location: differing kind and raw_path must
    // likewise keep the edges distinct.
    let mut by_kind = edge(1, 2, 3);
    by_kind.kind = IncludeKind::System;
    graph.add_edge(by_kind);
    assert_eq!(graph.edges.len(), 3, "kind participates in edge equality");
}

#[test]
fn edges_vec_is_byte_identical_to_reference_interleave_fixture() {
    // Build-order parity lock: the resulting Vec (and its serialized bytes)
    // must be identical to the independently constructed reference order of
    // first occurrences for a fixed interleaved insert sequence.
    let inserts: Vec<IncludeEdge> = [
        edge(10, 11, 1),
        edge(10, 12, 2),
        edge(11, 12, 1),
        edge(10, 11, 1),
        edge(12, 13, 4),
        edge(10, 12, 2),
        edge(11, 12, 1),
        edge(13, 10, 2),
        edge(12, 13, 4),
    ]
    .into_iter()
    .collect();

    let mut graph = IncludeGraph::new();
    for candidate in &inserts {
        graph.add_edge(candidate.clone());
    }

    let expected: Vec<IncludeEdge> = vec![
        edge(10, 11, 1),
        edge(10, 12, 2),
        edge(11, 12, 1),
        edge(12, 13, 4),
        edge(13, 10, 2),
    ];
    assert_eq!(graph.edges, expected, "Vec order must match reference");

    let actual_bytes = serde_json::to_vec(&graph.edges).expect("serialize actual");
    let expected_bytes = serde_json::to_vec(&expected).expect("serialize expected");
    assert_eq!(
        actual_bytes, expected_bytes,
        "serialized edge bytes must be identical to the reference order"
    );
}

#[test]
fn artifact_rebuild_loop_dedups_and_keeps_first_occurrence_order() {
    // Mirrors the `report::manager::graph_io` load path: a fresh graph fed the
    // persisted edge list (which may itself contain repeats) through
    // `add_edge`; the rebuilt Vec must equal the dedup-keep-first view.
    let persisted: Vec<IncludeEdge> = vec![
        edge(1, 2, 1),
        edge(2, 3, 1),
        edge(1, 2, 1),
        edge(1, 2, 5),
        edge(2, 3, 1),
    ];
    let mut graph = IncludeGraph::new();
    for candidate in persisted {
        graph.add_edge(candidate);
    }
    assert_eq!(
        graph.edges,
        vec![edge(1, 2, 1), edge(2, 3, 1), edge(1, 2, 5)],
        "rebuild via add_edge must dedup identically to the build path"
    );
}

#[test]
fn scale_insert_with_repeats_yields_exact_deduped_order() {
    // 3000 unique edges each replayed three times at offsets that interleave
    // repeats between first occurrences; correctness only, no timing claim.
    const UNIQUE: u64 = 3000;
    let mut graph = IncludeGraph::new();
    for round in 0..3u64 {
        for id in 0..UNIQUE {
            let shifted = (id + round * 7) % UNIQUE;
            graph.add_edge(edge(shifted, 1000 + shifted, (shifted % 13) as u32 + 1));
        }
    }
    assert_eq!(graph.edges.len(), UNIQUE as usize);
    for (position, stored) in graph.edges.iter().enumerate() {
        // First-occurrence order: round 0 visits ids in the rotated order
        // (id + 0*7) % UNIQUE == id, so edge(id, 1000+id) at line id%13+1.
        let id = position as u64;
        assert_eq!(*stored, edge(id, 1000 + id, (id % 13) as u32 + 1));
    }
}

#[test]
fn serialized_graph_shape_never_carries_the_dedup_set() {
    // The dedup HashSet is a build-time index: serde-skipped, so the JSON
    // shape of the whole graph stays {edges, file_paths} exactly as today.
    let mut graph = IncludeGraph::new();
    graph.register_file(FileId(1), PathBuf::from("a.h"));
    graph.add_edge(edge(1, 2, 1));
    let shape = serde_json::to_value(&graph).expect("serialize graph");
    let keys: Vec<&String> = shape
        .as_object()
        .expect("graph serializes as an object")
        .keys()
        .collect();
    assert_eq!(
        keys,
        vec!["edges", "file_paths"],
        "no dedup-set field may appear in the serialized shape"
    );
}
