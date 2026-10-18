//! UTA-T1 item 15 (hand H1): external degrees of a SELF-LOOP singleton
//! cycle group - only pairs with exactly one endpoint in the group count;
//! the self-pair stays internal. Mirrors the fixture style of
//! `crates/graph/tests/cycle_groups_test.rs`.

use ariadne_graph::{IncludeEdge, IncludeGraph, IncludeKind, LanguageId, SourceLocation};
use ariadne_repo::FileId;
use std::path::PathBuf;

fn edge(source: u64, destination: u64) -> IncludeEdge {
    IncludeEdge {
        source: FileId(source),
        destination: Some(FileId(destination)),
        context_name: "selfloop-degrees".into(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: format!("f{destination}.h"),
        resolved_path: None,
        language: LanguageId::cpp(),
    }
}

#[test]
fn self_loop_group_counts_only_external_pairs() {
    // Group {5} exists via the 5 -> 5 self-include. 8 -> 5 and 9 -> 5
    // enter the group, 5 -> 7 leaves it; the self-pair must never be
    // counted on either side.
    let mut graph = IncludeGraph::new();
    for id in [5u64, 7, 8, 9] {
        graph.register_file(FileId(id), PathBuf::from(format!("g{id}.h")));
    }
    for (source, destination) in [(5, 5), (8, 5), (9, 5), (5, 7)] {
        graph.add_edge(edge(source, destination));
    }

    let groups = graph.detect_cycle_groups();
    assert_eq!(groups.len(), 1, "only the self-include entangles");
    let group = &groups[0];
    assert_eq!(group.member_files, vec![FileId(5)]);
    assert_eq!(
        group.representative_cycle,
        vec![FileId(5), FileId(5)],
        "self-loop groups keep the short-circuited witness"
    );
    assert_eq!(
        group.external_in_count, 2,
        "the (8,5) and (9,5) distinct pairs enter"
    );
    assert_eq!(
        group.external_out_count, 1,
        "only (5,7) leaves - the (5,5) self-pair is internal"
    );
}
