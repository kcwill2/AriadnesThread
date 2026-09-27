use ariadne_ui::{ExtractionProposalDto, GraphExportPayload, SummaryStats};
use std::collections::HashMap;

fn sample_proposal(plan_id: &str) -> ExtractionProposalDto {
    ExtractionProposalDto {
        plan_id: plan_id.to_string(),
        target_file: "graph.h".to_string(),
        target_file_id: "1".to_string(),
        problem: "300 symbols of collateral load".to_string(),
        collateral_load_before: 300,
        collateral_load_after: 40,
        symbol_names: vec!["Thing".to_string(), "ThingBuilder".to_string()],
        total_symbols: 2,
        required_headers: vec!["a.h".to_string(), "b.h".to_string()],
        consumer_tu_count: 23,
        risk: "Conservative".to_string(),
    }
}

fn payload_with(proposals: Vec<ExtractionProposalDto>) -> GraphExportPayload {
    GraphExportPayload {
        repo_name: "fixture-repo".to_string(),
        include_graph: vec![],
        symbol_graph: vec![],
        call_graph: vec![],
        stats: SummaryStats {
            total_files: 0,
            total_symbols: 0,
            total_calls: 0,
            cycle_count: 0,
        },
        file_collateral: HashMap::new(),
        file_justification: HashMap::new(),
        extraction_proposals: proposals,
    }
}

#[test]
fn test_extraction_proposals_round_trip_when_non_empty() {
    let proposals = vec![sample_proposal("PLAN-001"), sample_proposal("PLAN-002")];
    let payload = payload_with(proposals.clone());

    let json = serde_json::to_value(&payload).unwrap();
    let array = json["extraction_proposals"]
        .as_array()
        .expect("non-empty extraction_proposals must serialize");
    assert_eq!(array.len(), 2);
    assert_eq!(json["extraction_proposals"][0]["plan_id"], "PLAN-001");
    assert_eq!(json["extraction_proposals"][1]["risk"], "Conservative");

    let back: GraphExportPayload = serde_json::from_value(json).unwrap();
    assert_eq!(back.extraction_proposals, proposals);
}

#[test]
fn test_empty_extraction_proposals_key_absent_from_json() {
    let json = serde_json::to_value(payload_with(Vec::new())).unwrap();
    assert!(
        json.get("extraction_proposals").is_none(),
        "empty extraction_proposals is skipped during serialization; clients must default it"
    );
}

#[test]
fn test_payload_json_without_proposals_key_deserializes_to_empty() {
    let json = serde_json::to_value(payload_with(Vec::new())).unwrap();
    let back: GraphExportPayload = serde_json::from_value(json).unwrap();
    assert!(back.extraction_proposals.is_empty());
}

#[test]
fn test_proposal_dto_key_names_are_stable() {
    let json =
        serde_json::to_value(payload_with(vec![sample_proposal("PLAN-001")])).unwrap();
    let mut keys: Vec<&str> = json["extraction_proposals"][0]
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        vec![
            "collateral_load_after",
            "collateral_load_before",
            "consumer_tu_count",
            "plan_id",
            "problem",
            "required_headers",
            "risk",
            "symbol_names",
            "target_file",
            "target_file_id",
            "total_symbols",
        ]
    );
}
