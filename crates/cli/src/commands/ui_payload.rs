//! Payload-assembly DTO builders, extracted from `ui.rs` (plan
//! load-tail A5). Pure data mapping from stage output to UI payload DTOs:
//! nothing here constructs a `Pipeline` or runs load analysis — the
//! artifact-first invariant across the split is asserted by
//! `ui.rs::tests::test_ui_is_artifact_first`, which also greps this file.
//! `build_ui_payload` keeps orchestration and hands these builders the
//! stage results.

use std::collections::HashMap;
use std::path::PathBuf;

/// Shared path→numeric-FileId lookup used by every per-file payload builder.
pub(crate) fn build_path_to_id(include_graph: &ariadne_graph::IncludeGraph) -> HashMap<PathBuf, u64> {
    include_graph
        .file_paths
        .iter()
        .map(|(id, path)| (path.clone(), id.0))
        .collect()
}

/// Map each target file (by numeric FileId) to the symbol counts contributed
/// by its collateral sources, keeping only files with non-zero contributors.
pub(crate) fn build_file_collateral(
    path_to_id: &HashMap<PathBuf, u64>,
    file_metrics: &[ariadne_graph::LoadMetric],
) -> HashMap<String, HashMap<String, usize>> {
    let mut file_collateral: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for metric in file_metrics {
        let Some(path) = metric.target_file.as_ref().and_then(|p| path_to_id.get(p)) else {
            continue;
        };
        let mut collat = HashMap::new();
        for src in &metric.collateral_sources {
            if src.symbol_count > 0 {
                collat.insert(src.file_id.0.to_string(), src.symbol_count);
            }
        }
        if !collat.is_empty() {
            file_collateral.insert(path.to_string(), collat);
        }
    }
    file_collateral
}

/// Conservative extraction proposals for the payload (I8): runs the same
/// planner as `tool optimize` and applies the identical Conservative filter
/// as `plan_safe_extractions`; display-only, no behavior coupling. The
/// header screening itself consumes the stage's file metrics and closures
/// (plan load-tail A1) instead of re-running the load analysis cold.
pub(crate) fn build_extraction_proposals(
    include_graph: &ariadne_graph::IncludeGraph,
    reduced_model: &ariadne_graph::ReducedSemanticModel,
    discovery_files: &[ariadne_repo::FileRecord],
    tu_file_ids: &[ariadne_repo::FileId],
    file_metrics: &[ariadne_graph::LoadMetric],
    closures: &HashMap<ariadne_repo::FileId, ariadne_graph::IncludeClosure>,
) -> Vec<ariadne_ui::ExtractionProposalDto> {
    let planner = ariadne_cluster::planner::OptimizationPlanner::with_screening_facts(
        include_graph,
        reduced_model,
        discovery_files,
        tu_file_ids,
        ariadne_cluster::planner::screening_facts_from_metrics(include_graph, file_metrics),
        closures,
    );
    conservative_proposals(planner.plan())
}

/// Pure filter/sort/cap stage (I6: `(delta desc, plan_id asc)`; I3: top 12),
/// kept separate from the planner so the ordering contract is testable
/// without planner machinery.
fn conservative_proposals(
    plans: Vec<ariadne_cluster::plan::OptimizationPlan>,
) -> Vec<ariadne_ui::ExtractionProposalDto> {
    let mut ranked: Vec<_> = plans
        .into_iter()
        .filter(|p| {
            matches!(p.risk, ariadne_cluster::plan::RiskLevel::Conservative)
                && p.collateral_load_before != p.collateral_load_after
        })
        .collect();
    ranked.sort_by(|a, b| {
        proposal_delta(b)
            .cmp(&proposal_delta(a))
            .then(a.id.0.cmp(&b.id.0))
    });
    ranked
        .into_iter()
        .take(ariadne_ui::EXTRACTION_PROPOSAL_MAX)
        .map(|p| extraction_proposal_dto(&p))
        .collect()
}

fn proposal_delta(plan: &ariadne_cluster::plan::OptimizationPlan) -> usize {
    plan.collateral_load_before
        .saturating_sub(plan.collateral_load_after)
}

fn extraction_proposal_dto(
    plan: &ariadne_cluster::plan::OptimizationPlan,
) -> ariadne_ui::ExtractionProposalDto {
    let total_symbols = plan.proposed_extractions.len();
    let mut extractions: Vec<&ariadne_cluster::plan::ProposedExtraction> =
        plan.proposed_extractions.iter().collect();
    extractions.sort_by(|a, b| a.symbol_id.cmp(&b.symbol_id));
    let symbol_names: Vec<String> = extractions
        .into_iter()
        .take(ariadne_ui::EXTRACTION_PROPOSAL_MAX_NAMES)
        .map(|e| e.symbol_name.clone())
        .collect();
    let mut required_headers: Vec<String> = plan
        .optimization_frontier
        .iter()
        .filter(|p| *p != &plan.target_file)
        .map(|p| proposal_basename(p))
        .collect();
    required_headers.sort();
    required_headers.dedup();
    required_headers.truncate(ariadne_ui::EXTRACTION_PROPOSAL_MAX_HEADERS);
    ariadne_ui::ExtractionProposalDto {
        plan_id: plan.id.0.clone(),
        target_file: proposal_basename(&plan.target_file),
        target_file_id: plan.target_file_id.0.to_string(),
        problem: plan.problem.clone(),
        collateral_load_before: plan.collateral_load_before,
        collateral_load_after: plan.collateral_load_after,
        symbol_names,
        total_symbols,
        required_headers,
        consumer_tu_count: plan.validation_required_tus.len(),
        risk: plan.risk.to_string(),
    }
}

/// Basename with the graph crate's whole-path fallback for component-less
/// paths (mirrors dto.rs `path_basename`).
fn proposal_basename(path: &std::path::Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

#[cfg(test)]
mod builder_tests {
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
        assert_eq!(conservative_proposals(plans()), conservative_proposals(plans()));
    }

    #[test]
    fn test_planner_wiring_on_empty_inputs_yields_no_proposals() {
        let graph = IncludeGraph::new();
        let model = ReducedSemanticModel {
            repo_root: PathBuf::from("."),
            symbols: BTreeMap::new(),
            edges: vec![],
            tu_usage: BTreeMap::new(),
            file_symbols: BTreeMap::new(),
        };
        let out = build_extraction_proposals(&graph, &model, &[], &[], &[], &HashMap::new());
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
            out.iter().all(|p| p.collateral_load_before != p.collateral_load_after),
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
}
