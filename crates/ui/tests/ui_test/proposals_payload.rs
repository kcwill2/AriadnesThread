use ariadne_ui::{ExtractionProposalDto, GraphExportPayload, ProposedTargetFileDto, SummaryStats};
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
        target_files: vec![],
        target_files_truncated: false,
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
            "target_files",
            "target_files_truncated",
            "total_symbols",
        ]
    );
}

#[test]
fn test_target_file_keys_emitted_when_fields_set() {
    let json =
        serde_json::to_value(payload_with(vec![sample_proposal("PLAN-001")])).unwrap();
    assert!(
        json["extraction_proposals"][0]
            .get("target_files")
            .is_some(),
        "target_files key must be emitted for every proposal row"
    );
}

#[test]
fn test_legacy_payload_without_target_file_keys_decodes() {
    let mut json =
        serde_json::to_value(payload_with(vec![sample_proposal("PLAN-001")])).unwrap();
    let legacy_row = json
        .get_mut("extraction_proposals")
        .and_then(|rows| rows.get_mut(0))
        .and_then(|row| row.as_object_mut())
        .expect("sample payload carries one proposal row");
    legacy_row.remove("target_files");
    legacy_row.remove("target_files_truncated");

    let back: GraphExportPayload = serde_json::from_value(json).unwrap();
    assert_eq!(
        back.extraction_proposals[0].target_files,
        vec![],
        "payloads written before the target-file fields must decode to empty"
    );
    assert!(
        !back.extraction_proposals[0].target_files_truncated,
        "absent target_files_truncated must decode to false"
    );
}

#[test]
fn test_target_files_round_trip_preserves_content() {
    let destination = ProposedTargetFileDto {
        path: "include/thing_extracted.h".to_string(),
        symbol_names: vec!["Thing".to_string(), "ThingBuilder".to_string()],
        symbol_count: 3,
    };
    let mut proposal = sample_proposal("PLAN-001");
    proposal.target_files = vec![destination.clone()];
    proposal.target_files_truncated = true;

    let json = serde_json::to_value(payload_with(vec![proposal.clone()])).unwrap();
    let back: GraphExportPayload = serde_json::from_value(json).unwrap();

    assert_eq!(back.extraction_proposals, vec![proposal]);
    assert_eq!(back.extraction_proposals[0].target_files, vec![destination]);
}
