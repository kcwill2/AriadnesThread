//! UTA2-R5 line 1: `IncludeGraph::map_aggregation_consumers` -- the neutral
//! aggregation-consumer query (graph.rs:321), previously ZERO coverage.
//!
//! The consumer map answers "which aggregation units (TUs today) consume this
//! file, directly or transitively" -- the planner's alternate arm when the
//! caller did not supply a closure map
//! (`crates/cluster/src/planner.rs:171`). Pins: transitive consumer listing,
//! FileId-sorted consumer vectors regardless of input order, unknown-tu skip,
//! and the empty-in/empty-out edge.

use ariadne_graph::{IncludeEdge, IncludeGraph, IncludeKind, LanguageId, SourceLocation};
use ariadne_repo::FileId;
use std::path::PathBuf;

const LOW_TU: u64 = 5;
const TU: u64 = 10;
const HDR: u64 = 20;
const MID: u64 = 30;
const UNKNOWN_TU: u64 = 999;

fn edge(source: u64, destination: u64) -> IncludeEdge {
    IncludeEdge {
        source: FileId(source),
        destination: Some(FileId(destination)),
        context_name: "Default".into(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: format!("dep{destination}.h"),
        resolved_path: Some(PathBuf::from(format!("include/dep{destination}.h"))),
        language: LanguageId::cpp(),
    }
}

/// The 3-file chain fixture tu -> hdr -> mid: `tu` includes `hdr` directly
/// and reaches `mid` only transitively.
fn chain_graph() -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    graph.add_edge(edge(TU, HDR));
    graph.add_edge(edge(HDR, MID));
    graph
}

#[test]
fn transitive_consumer_listing_covers_the_whole_chain() {
    let consumers = chain_graph().map_aggregation_consumers(&[FileId(TU)]);
    assert_eq!(
        consumers.len(),
        2,
        "both the direct include and the transitive include gain a consumer entry"
    );
    assert_eq!(
        consumers[&FileId(HDR)],
        vec![FileId(TU)],
        "the directly included header lists the TU"
    );
    assert_eq!(
        consumers[&FileId(MID)],
        vec![FileId(TU)],
        "the chain-middle header is consumed TRANSITIVELY by the TU even though \
         the TU never includes it directly"
    );
}

#[test]
fn consumer_vectors_are_sorted_by_file_id_regardless_of_input_order() {
    // Two TUs both reach hdr and mid through their own hdr edges; the input
    // order (high id first) must not leak into the consumer vectors. The
    // consumer set is a HashSet internally, so iteration order is
    // hash-seed-random between runs - only the sort makes the vector stable.
    let mut graph = chain_graph();
    graph.add_edge(edge(LOW_TU, HDR));
    let consumers = graph.map_aggregation_consumers(&[FileId(TU), FileId(LOW_TU)]);
    assert_eq!(
        consumers[&FileId(HDR)],
        vec![FileId(LOW_TU), FileId(TU)],
        "consumer ids must be FileId-sorted ascending despite descending input"
    );
    assert_eq!(
        consumers[&FileId(MID)],
        vec![FileId(LOW_TU), FileId(TU)],
        "the transitive header sees both TUs, sorted"
    );
}

#[test]
fn tu_id_missing_from_the_closure_map_is_skipped() {
    // UNKNOWN_TU has no edges and no registration, so compute_closures never
    // produces an entry for it; it must contribute nothing - no panic, no
    // empty-key artifact, no invented consumer rows.
    let graph = chain_graph();
    let with_unknown = graph.map_aggregation_consumers(&[FileId(TU), FileId(UNKNOWN_TU)]);
    let without_unknown = graph.map_aggregation_consumers(&[FileId(TU)]);
    assert_eq!(
        with_unknown, without_unknown,
        "an unknown tu_id must be skipped exactly"
    );
    assert!(
        !with_unknown.contains_key(&FileId(UNKNOWN_TU)),
        "the unknown id must never surface as a key"
    );
    for consumers in with_unknown.values() {
        assert!(
            !consumers.contains(&FileId(UNKNOWN_TU)),
            "the unknown id must never surface as a consumer"
        );
    }
}

#[test]
fn empty_input_maps_to_empty_output() {
    assert!(
        chain_graph()
            .map_aggregation_consumers(&[])
            .is_empty(),
        "no queried TU means no consumer rows, even with edges present"
    );
    assert!(
        IncludeGraph::new().map_aggregation_consumers(&[]).is_empty(),
        "the edge-less empty graph maps empty too"
    );
}
