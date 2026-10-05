//! P1 feed-truth oracle (ui-extraction-card-playbook T-B): the DTO's
//! `target_files` rows must equal the plan's symbol -> `target_header`
//! mapping (paths ascending via BTreeMap; symbols sorted by `symbol_id`,
//! mirroring the existing `symbol_names` sort; per-file name cap and
//! per-proposal file-row cap with the truncated flag set iff the group
//! count exceeds the cap). Constructed plans only: no planner, no I/O.

use super::extraction_proposal_dto;
use ariadne_cluster::plan::{OptimizationPlan, PlanId, ProposedExtraction, RiskLevel};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use std::path::PathBuf;

fn extraction(symbol_name: &str, symbol_id: &str, target_header: &str) -> ProposedExtraction {
    ProposedExtraction {
        symbol_name: symbol_name.to_string(),
        symbol_id: SymbolId::new(symbol_id),
        target_header: PathBuf::from(target_header),
        source_file: Some(FileId(7)),
    }
}

fn plan_extracting(extractions: Vec<ProposedExtraction>) -> OptimizationPlan {
    OptimizationPlan {
        id: PlanId("PLAN-001".to_string()),
        target_file: PathBuf::from("include/graph.h"),
        target_file_id: FileId(7),
        problem: "300 symbols of collateral load".to_string(),
        collateral_load_before: 300,
        collateral_load_after: 40,
        proposed_extractions: extractions,
        optimization_frontier: vec![
            PathBuf::from("inc/graphics.h"),
            PathBuf::from("inc/config.h"),
        ],
        risk: RiskLevel::Conservative,
        validation_required_tus: vec![PathBuf::from("a.cc"), PathBuf::from("b.cc")],
        rationale: String::new(),
    }
}

fn one_symbol_per_file(count: usize) -> Vec<ProposedExtraction> {
    (1..=count)
        .map(|n| {
            extraction(
                &format!("Sym{n:02}"),
                &format!("sym_{n:02}"),
                &format!("hdrs/h{n:02}.h"),
            )
        })
        .collect()
}

#[test]
fn test_target_files_group_by_path_ascending() {
    let plan = plan_extracting(vec![
        extraction("Zeta", "sym_z", "zeta.h"),
        extraction("Mid2", "sym_m2", "mid.h"),
        extraction("Alpha", "sym_a", "alpha.h"),
        extraction("Mid1", "sym_m1", "mid.h"),
    ]);
    let dto = extraction_proposal_dto(&plan);
    let paths: Vec<&str> = dto.target_files.iter().map(|row| row.path.as_str()).collect();
    assert_eq!(paths, vec!["alpha.h", "mid.h", "zeta.h"]);
    assert_eq!(dto.target_files[1].symbol_names, vec!["Mid1", "Mid2"]);
    let counts: Vec<usize> = dto.target_files.iter().map(|row| row.symbol_count).collect();
    assert_eq!(counts, vec![1, 2, 1]);
    assert!(!dto.target_files_truncated);
}

#[test]
fn test_symbol_file_assignment_preserved() {
    let plan = plan_extracting(vec![
        extraction("Alpha", "sym_1", "alpha.h"),
        extraction("Beta", "sym_2", "beta.h"),
        extraction("Gamma", "sym_3", "alpha.h"),
    ]);
    let dto = extraction_proposal_dto(&plan);
    let rows: Vec<(&str, Vec<&str>)> = dto
        .target_files
        .iter()
        .map(|row| {
            (
                row.path.as_str(),
                row.symbol_names.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![("alpha.h", vec!["Alpha", "Gamma"]), ("beta.h", vec!["Beta"])],
        "each symbol must appear under exactly its own target_header"
    );
}

#[test]
fn test_per_file_name_cap_and_uncapped_count() {
    let crowded = (1..=15)
        .map(|n| extraction(&format!("Sym{n:02}"), &format!("sym_{n:02}"), "graph_types.h"))
        .collect();
    let dto = extraction_proposal_dto(&plan_extracting(crowded));
    assert_eq!(dto.target_files.len(), 1);
    let row = &dto.target_files[0];
    assert_eq!(row.symbol_names.len(), ariadne_ui::EXTRACTION_PROPOSAL_MAX_NAMES);
    assert_eq!(row.symbol_count, 15, "symbol_count is the uncapped member count");
    assert_eq!(row.symbol_names.first().map(String::as_str), Some("Sym01"));
    assert_eq!(row.symbol_names[ariadne_ui::EXTRACTION_PROPOSAL_MAX_NAMES - 1], "Sym12");
    assert!(!dto.target_files_truncated);
}

#[test]
fn test_file_row_cap_sets_truncated_flag() {
    let over_cap = extraction_proposal_dto(&plan_extracting(one_symbol_per_file(9)));
    assert_eq!(over_cap.target_files.len(), ariadne_ui::EXTRACTION_PROPOSAL_MAX_TARGET_FILES);
    assert!(over_cap.target_files_truncated, "9 groups exceed the cap of 8");
    assert_eq!(over_cap.target_files[0].path, "hdrs/h01.h", "cap keeps the path-asc head");
    assert_eq!(over_cap.target_files[7].path, "hdrs/h08.h");
    let at_cap = extraction_proposal_dto(&plan_extracting(one_symbol_per_file(8)));
    assert_eq!(at_cap.target_files.len(), 8);
    assert!(!at_cap.target_files_truncated, "8 groups do not exceed the cap");
}

#[test]
fn test_existing_fields_unchanged_by_the_fill() {
    let plan = plan_extracting(vec![
        extraction("Zeta", "sym_z", "zeta.h"),
        extraction("Alpha", "sym_a", "alpha.h"),
    ]);
    let dto = extraction_proposal_dto(&plan);
    assert_eq!(dto.plan_id, "PLAN-001");
    assert_eq!(dto.target_file, "graph.h");
    assert_eq!(dto.target_file_id, "7");
    assert_eq!(dto.problem, "300 symbols of collateral load");
    assert_eq!(dto.collateral_load_before, 300);
    assert_eq!(dto.collateral_load_after, 40);
    assert_eq!(dto.symbol_names, vec!["Alpha".to_string(), "Zeta".to_string()]);
    assert_eq!(dto.total_symbols, 2);
    assert_eq!(
        dto.required_headers,
        vec!["config.h".to_string(), "graphics.h".to_string()]
    );
    assert_eq!(dto.consumer_tu_count, 2);
    assert_eq!(dto.risk, "Conservative");
}
