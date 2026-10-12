use super::{build_extraction_proposals, conservative_proposals};
use ariadne_cluster::plan::{OptimizationPlan, PlanId, ProposedExtraction, RiskLevel};
use ariadne_graph::{IncludeGraph, ReducedSemanticModel};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

fn plan_with(id: &str, before: usize, after: usize, risk: RiskLevel) -> OptimizationPlan {
    OptimizationPlan {
        id: PlanId(id.to_string()),
        target_file: PathBuf::from("include/graph.h"),
        target_file_id: FileId(7),
        problem: format!("{before} symbols of collateral load"),
        collateral_load_before: before,
        collateral_load_after: after,
        proposed_extractions: vec![ProposedExtraction {
            symbol_name: "Thing".to_string(),
            symbol_id: SymbolId::new("sym_1"),
            target_header: PathBuf::from("graph_types.h"),
            source_file: Some(FileId(7)),
        }],
        optimization_frontier: vec![
            PathBuf::from("inc/graphics.h"),
            PathBuf::from("inc/config.h"),
        ],
        risk,
        validation_required_tus: vec![PathBuf::from("a.cc"), PathBuf::from("b.cc")],
        rationale: String::new(),
    }
}

#[test]
fn test_conservative_plan_maps_every_dto_field() {
    let out = conservative_proposals(vec![plan_with(
        "PLAN-001",
        300,
        40,
        RiskLevel::Conservative,
    )]);
    assert_eq!(out.len(), 1);
    let dto = &out[0];
    assert_eq!(dto.plan_id, "PLAN-001");
    assert_eq!(dto.target_file, "graph.h");
    assert_eq!(dto.target_file_id, "7");
    assert_eq!(dto.problem, "300 symbols of collateral load");
    assert_eq!(dto.collateral_load_before, 300);
    assert_eq!(dto.collateral_load_after, 40);
    assert_eq!(dto.symbol_names, vec!["Thing".to_string()]);
    assert_eq!(dto.total_symbols, 1);
    assert_eq!(
        dto.required_headers,
        vec!["config.h".to_string(), "graphics.h".to_string()]
    );
    assert_eq!(dto.consumer_tu_count, 2);
    assert_eq!(dto.risk, "Conservative");
}

#[test]
fn test_moderate_and_aggressive_plans_are_excluded() {
    let out = conservative_proposals(vec![
        plan_with("PLAN-001", 100, 10, RiskLevel::Moderate),
        plan_with("PLAN-002", 200, 20, RiskLevel::Conservative),
        plan_with("PLAN-003", 300, 30, RiskLevel::Aggressive),
        plan_with("PLAN-004", 400, 40, RiskLevel::Conservative),
    ]);
    let ids: Vec<&str> = out.iter().map(|p| p.plan_id.as_str()).collect();
    assert_eq!(ids, vec!["PLAN-004", "PLAN-002"]);
    assert!(
        out.iter().all(|p| p.risk == "Conservative"),
        "I8: only Conservative plans may reach the payload"
    );
}

#[test]
fn test_cap_keeps_top_twelve_and_drops_lowest_delta() {
    let plans: Vec<OptimizationPlan> = (1..=13)
        .map(|n| {
            let mut p = plan_with(
                &format!("PLAN-{n:02}"),
                n * 100,
                n * 10,
                RiskLevel::Conservative,
            );
            p.collateral_load_after = 0;
            p
        })
        .collect();
    let out = conservative_proposals(plans);
    assert_eq!(out.len(), 12);
    assert_eq!(out[0].plan_id, "PLAN-13");
    assert_eq!(out[11].plan_id, "PLAN-02");
    assert!(
        out.iter().all(|p| p.plan_id != "PLAN-01"),
        "the lowest-delta plan must be the one dropped by the cap"
    );
}

#[test]
fn test_equal_deltas_sorted_by_plan_id_ascending() {
    let out = conservative_proposals(vec![
        plan_with("PLAN-002", 500, 100, RiskLevel::Conservative),
        plan_with("PLAN-001", 500, 100, RiskLevel::Conservative),
    ]);
    assert_eq!(
        out.iter().map(|p| p.plan_id.as_str()).collect::<Vec<_>>(),
        vec!["PLAN-001", "PLAN-002"]
    );
}

#[test]
fn test_symbol_names_capped_at_twelve_with_true_total() {
    let mut p = plan_with("PLAN-001", 900, 100, RiskLevel::Conservative);
    p.proposed_extractions = (1..=15)
        .map(|n| ProposedExtraction {
            symbol_name: format!("Sym{n:02}"),
            symbol_id: SymbolId::new(format!("sym_{n:02}")),
            target_header: PathBuf::from("graph_types.h"),
            source_file: Some(FileId(7)),
        })
        .collect();
    let out = conservative_proposals(vec![p]);
    assert_eq!(out[0].symbol_names.len(), 12);
    assert_eq!(out[0].symbol_names[0], "Sym01");
    assert_eq!(out[0].symbol_names[11], "Sym12");
    assert_eq!(out[0].total_symbols, 15);
}

#[test]
fn test_mapping_is_deterministic() {
    let plans = || {
        vec![
            plan_with("PLAN-002", 400, 40, RiskLevel::Moderate),
            plan_with("PLAN-001", 500, 50, RiskLevel::Conservative),
            plan_with("PLAN-003", 500, 50, RiskLevel::Conservative),
        ]
    };
    assert_eq!(
        conservative_proposals(plans()),
        conservative_proposals(plans())
    );
}

#[test]
fn test_planner_wiring_on_empty_inputs_yields_no_proposals() {
    let model = ReducedSemanticModel {
        repo_root: PathBuf::from("."),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };
    // AR-19 (A1 part 2): the proposals seam takes the registry-dispatched
    // hooks now (shared with plan/optimize); on empty inputs the dispatch
    // has nothing to claim, so the witness is unchanged.
    let registry = crate::registry::default_registry();
    let out = build_extraction_proposals(
        &IncludeGraph::new(),
        &model,
        &[],
        &[],
        &[],
        &HashMap::new(),
        &registry,
        std::path::Path::new("."),
    );
    assert!(out.is_empty());
}

#[test]
fn test_zero_collateral_proposals_are_filtered() {
    let out = conservative_proposals(vec![
        plan_with("PLAN-001", 100, 100, RiskLevel::Conservative),
        plan_with("PLAN-002", 200, 50, RiskLevel::Conservative),
        plan_with("PLAN-003", 300, 300, RiskLevel::Conservative),
        plan_with("PLAN-004", 400, 40, RiskLevel::Conservative),
    ]);
    let ids: Vec<&str> = out.iter().map(|p| p.plan_id.as_str()).collect();
    assert_eq!(ids, vec!["PLAN-004", "PLAN-002"]);
    assert!(
        out.iter()
            .all(|p| p.collateral_load_before != p.collateral_load_after),
        "Zero-collateral proposals (before == after) must be filtered out"
    );
}

#[test]
fn test_self_reference_removed_from_required_headers() {
    let mut plan = plan_with("PLAN-001", 300, 40, RiskLevel::Conservative);
    plan.target_file = PathBuf::from("include/graph.h");
    plan.optimization_frontier = vec![
        PathBuf::from("include/graph.h"),
        PathBuf::from("inc/graphics.h"),
        PathBuf::from("inc/config.h"),
    ];
    let out = conservative_proposals(vec![plan]);
    assert_eq!(out.len(), 1);
    let dto = &out[0];
    assert_eq!(
        dto.required_headers,
        vec!["config.h".to_string(), "graphics.h".to_string()]
    );
    assert!(
        !dto.required_headers.contains(&"graph.h".to_string()),
        "Target file should not appear in its own required_headers"
    );
}
