//! UTA-T6 item 6 (pin-only) planner locks: the union-merge discipline of
//! `merge_colocated_batches` (anchor position, reverse-append order,
//! transitive group joining) asserted on deterministic exact membership
//! order, the duplicate-metric winner of `screening_facts_from_metrics`,
//! and the accept/frozen pair of `check_candidate_safety`.
//!
//! This lives in its own file because `planner.rs` sits near the 1000-line
//! ceiling (handoff: extract oversized test modules rather than breach it).

use super::{merge_colocated_batches, screening_facts_from_metrics, OptimizationPlanner};
use crate::plan::{PlanRejection, RejectionReason};
use crate::planner::HeaderScreeningFacts;
use ariadne_graph::reduced::ReducedSemanticModel;
use ariadne_graph::{ContributingSource, IncludeGraph, LoadMetric};
use ariadne_index::types::SymbolId;
use ariadne_lang::CyclePolicy;
use ariadne_repo::{FileId, OptimizationPolicy};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

fn sym(name: &str) -> SymbolId {
    SymbolId::new(name)
}

// ---------------------------------------------------------------------------
// merge_colocated_batches
// ---------------------------------------------------------------------------

#[test]
fn merge_keeps_anchor_position_and_appends_last_intersected_batch_first() {
    // Three batches intersect one colocated group. The FIRST intersected
    // batch is the anchor and stays at its index; the others are removed
    // back-to-front (so earlier indices stay valid) and appended in reverse
    // encounter order - last-intersected batch's symbols land FIRST.
    // Asserted as an exact full-vec match, not a set.
    let mut batches = vec![
        vec![sym("X")],
        vec![sym("G")],
        vec![sym("H")],
        vec![sym("K")],
    ];
    merge_colocated_batches(&mut batches, &[vec![sym("G"), sym("H"), sym("K")]]);

    assert_eq!(
        batches,
        vec![vec![sym("X")], vec![sym("G"), sym("K"), sym("H")]],
        "pin: untouched batch keeps index 0, anchor keeps index 1 with the \
         anchor symbol first, then K (last intersected) before H"
    );
}

#[test]
fn merge_joins_groups_transitively_through_a_shared_symbol_and_ignores_non_intersecting() {
    // Group 1 merges {A-batch, D-batch} into the A anchor. Group 2 shares D
    // with the merged anchor batch, so it pulls B into the SAME anchor -
    // groups are applied sequentially and a shared symbol transitively
    // joins them. A third group intersecting nothing must be a no-op.
    let mut batches = vec![
        vec![sym("A")],
        vec![sym("B")],
        vec![sym("C")],
        vec![sym("D")],
    ];
    merge_colocated_batches(
        &mut batches,
        &[
            vec![sym("A"), sym("D")],
            vec![sym("D"), sym("B")],
            vec![sym("Q")],
        ],
    );

    assert_eq!(
        batches,
        vec![vec![sym("A"), sym("D"), sym("B")], vec![sym("C")]],
        "pin: transitive join via shared D lands B behind D in the A anchor; \
         the untouched C batch shifts up to index 1; the Q group changes nothing"
    );
}

// ---------------------------------------------------------------------------
// screening_facts_from_metrics - duplicate metric winner
// ---------------------------------------------------------------------------

fn metric_for(load: usize, source: FileId, source_path: &str) -> LoadMetric {
    LoadMetric {
        target_name: "dup.h".to_string(),
        target_file: Some(PathBuf::from("dup.h")),
        semantic_load: 0,
        include_load: 0,
        collateral_load: load,
        total_load: load,
        semantic_symbols: vec![],
        collateral_symbols: vec![],
        collateral_sources: vec![],
        largest_collateral_source: Some(ContributingSource {
            file_id: source,
            path: PathBuf::from(source_path),
            symbol_count: 0,
            symbols: vec![],
        }),
        semantic_sources: vec![],
    }
}

#[test]
fn pin_screening_facts_later_duplicate_metric_wins() {
    // Two file_metrics rows resolve to the SAME FileId (same target path).
    // The map is built by `collect`, so the LAST duplicate overwrites the
    // earlier one deterministically in slice order. Pinned winner: later row.
    let mut graph = IncludeGraph::default();
    graph.register_file(FileId(9), PathBuf::from("dup.h"));
    graph.register_file(FileId(20), PathBuf::from("big1.h"));
    graph.register_file(FileId(30), PathBuf::from("big2.h"));

    let facts = screening_facts_from_metrics(
        &graph,
        &[
            metric_for(10, FileId(20), "big1.h"),
            metric_for(99, FileId(30), "big2.h"),
        ],
    );

    assert_eq!(facts.len(), 1, "duplicates collapse to one entry");
    assert_eq!(
        facts[&FileId(9)],
        HeaderScreeningFacts {
            collateral_load: 99,
            largest_collateral_source: Some(FileId(30)),
        },
        "pin: the later metric in slice order is the stable winner"
    );
}

// ---------------------------------------------------------------------------
// check_candidate_safety - accept + frozen-target pair
// ---------------------------------------------------------------------------

/// Minimal borrowed snapshot: `check_candidate_safety` builds its own
/// checker from the passed policies and touches none of the other fields.
struct EmptyPlannerFixture {
    graph: IncludeGraph,
    model: ReducedSemanticModel,
    records: Vec<ariadne_repo::FileRecord>,
    tu_file_ids: Vec<FileId>,
}

impl EmptyPlannerFixture {
    fn new() -> Self {
        Self {
            graph: IncludeGraph::default(),
            model: ReducedSemanticModel {
                repo_root: PathBuf::from("/"),
                symbols: BTreeMap::new(),
                edges: Vec::new(),
                tu_usage: BTreeMap::new(),
                file_symbols: BTreeMap::new(),
                extends_edges: Vec::new(),
                sealed_symbols: Vec::new(),
            },
            records: Vec::new(),
            tu_file_ids: Vec::new(),
        }
    }

    fn planner(&self) -> OptimizationPlanner<'_> {
        fn accept_unit(path: &Path) -> bool {
            path.file_name().is_some()
        }
        fn unit_template(path: &Path, symbol: &str) -> PathBuf {
            path.with_file_name(format!("{symbol}_unit"))
        }
        OptimizationPlanner::new(
            &self.graph,
            &self.model,
            &self.records,
            &self.tu_file_ids,
            &accept_unit,
            &unit_template,
            None,
            CyclePolicy::Reject,
        )
    }
}

#[test]
fn check_candidate_safety_accepts_then_rejects_the_frozen_target_pair() {
    let fixture = EmptyPlannerFixture::new();
    let planner = fixture.planner();
    let mut policies = HashMap::new();
    policies.insert(PathBuf::from("ok.h"), OptimizationPolicy::Optimizable);
    policies.insert(PathBuf::from("frozen.h"), OptimizationPolicy::Frozen);
    let movable = sym("Movable");

    planner
        .check_candidate_safety(&movable, Path::new("ok.h"), false, false, None, policies.clone())
        .expect("an optimizable target with clean flags must be accepted");

    let rejection: PlanRejection = planner
        .check_candidate_safety(&movable, Path::new("frozen.h"), false, false, None, policies)
        .expect_err("a frozen target must be rejected");
    assert!(
        matches!(rejection.reason, RejectionReason::FrozenFile(_)),
        "frozen targets reject with FrozenFile, got {:?}",
        rejection.reason
    );
    assert_eq!(rejection.target_file, PathBuf::from("frozen.h"));
}
