//! FIX-F lock (plan M23 §8): the additive `extends_edges`/`modifiers` fields
//! omit empty values from human-readable JSON (m00 golden bytes) but must
//! ALWAYS be present for positional formats (bincode). A format-conditional
//! omission desynchronizes the derived positional reader and every non-empty
//! `tu_analyses.bin` stops loading, which broke the warm stage trust path
//! (m00 warm manifest stats, m18/pipeline warm-load assertions).

use ariadne_lang_api::types::{SymbolId, SymbolRecord, TuAnalysis};

fn record_with_modifiers(name: &str, modifiers: Vec<&str>) -> SymbolRecord {
    SymbolRecord {
        id: SymbolId::new(name),
        name: name.to_string(),
        kind: ariadne_lang_api::types::SymbolKind::Trait,
        owning_file: ariadne_repo::FileId(7),
        declarations: Vec::new(),
        definition: None,
        modifiers: modifiers.into_iter().map(String::from).collect(),
    }
}

#[test]
fn bincode_round_trips_non_empty_analysis_vectors() {
    let analyses = vec![TuAnalysis::default(), TuAnalysis::default()];
    let bytes = bincode::serialize(&analyses).unwrap();
    let back: Vec<TuAnalysis> = bincode::deserialize(&bytes).expect(
        "two default TuAnalysis records must survive bincode round-trip \
         (warm-load path deserializes exactly this shape)",
    );
    assert_eq!(back, analyses);
}

#[test]
fn bincode_round_trips_sealed_modifier_and_extends_channel() {
    let mut analysis = TuAnalysis::default();
    analysis.symbols = vec![record_with_modifiers("p.Shape", vec!["sealed"])];
    analysis.extends_edges = vec![(
        SymbolId::new("p.Circle"),
        SymbolId::new("p.Shape"),
    )];
    let bytes = bincode::serialize(&vec![analysis.clone()]).unwrap();
    let back: Vec<TuAnalysis> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(back, vec![analysis], "D-M23-8 channels survive bincode");
}

#[test]
fn bincode_round_trips_non_empty_symbol_record_vectors() {
    let records = vec![
        record_with_modifiers("p.Shape", vec!["sealed"]),
        record_with_modifiers("p.Item", vec![]),
    ];
    let bytes = bincode::serialize(&records).unwrap();
    let back: Vec<SymbolRecord> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(back, records);
}

#[test]
fn json_omits_empty_channels_and_keeps_populated_ones() {
    let empty_json = serde_json::to_string(&TuAnalysis::default()).unwrap();
    assert!(
        !empty_json.contains("extends_edges"),
        "m00 mechanism: empty extends channel stays out of JSON bytes"
    );
    let record_json = serde_json::to_string(&record_with_modifiers("p.X", vec![])).unwrap();
    assert!(
        !record_json.contains("modifiers"),
        "m00 mechanism: empty modifiers stay out of JSON bytes"
    );

    let mut analysis = TuAnalysis::default();
    analysis.extends_edges = vec![(SymbolId::new("a"), SymbolId::new("b"))];
    analysis.symbols = vec![record_with_modifiers("a", vec!["sealed"])];
    let full = serde_json::to_string(&analysis).unwrap();
    let back: TuAnalysis = serde_json::from_str(&full).unwrap();
    assert_eq!(back, analysis, "populated channels round-trip through JSON");
}

#[test]
fn json_without_channel_keys_still_deserializes() {
    let legacy = r#"{"tu_file_id":1,"context_name":"","symbols":[],"declarations":[],
        "edges":[],"references":[],"call_sites":[],"language":"Cpp"}"#;
    let parsed: TuAnalysis = serde_json::from_str(legacy).unwrap();
    assert!(parsed.extends_edges.is_empty());
}
