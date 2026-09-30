use ariadne_graph::{
    ContributingSource, IncludeEdge, IncludeGraph, IncludeKind, LoadMetric, ReducedSemanticEdge,
    ReducedSemanticModel, ReducedSymbolRecord, SourceLocation,
};
use ariadne_index::{DependencyKind, SymbolId, SymbolKind};
use ariadne_repo::FileId;
use ariadne_ui::{
    build_file_justification, build_file_justification_detail, GraphExportPayload, SummaryStats,
};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

fn sym(id: &str, file: u64) -> ReducedSymbolRecord {
    ReducedSymbolRecord {
        id: SymbolId(id.to_string()),
        name: id.to_string(),
        kind: SymbolKind::Function,
        defining_file: Some(FileId(file)),
        declarations: vec![],
        definition: None,
    }
}

fn model_with(symbols: Vec<ReducedSymbolRecord>, edges: Vec<(&str, &str)>) -> ReducedSemanticModel {
    ReducedSemanticModel {
        repo_root: PathBuf::from("."),
        symbols: symbols
            .into_iter()
            .map(|r| (r.id.clone(), r))
            .collect::<BTreeMap<_, _>>(),
        edges: edges
            .into_iter()
            .map(|(src, dst)| ReducedSemanticEdge {
                src_symbol: SymbolId(src.to_string()),
                dst_symbol: SymbolId(dst.to_string()),
                kind: DependencyKind::Complete,
            })
            .collect(),
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),
    }
}

fn graph_with_paths(paths: &[(u64, &str)]) -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    for (id, path) in paths {
        graph.file_paths.insert(FileId(*id), PathBuf::from(path));
    }
    graph
}

fn path_to_id(graph: &IncludeGraph) -> HashMap<PathBuf, u64> {
    graph
        .file_paths
        .iter()
        .map(|(id, path)| (path.clone(), id.0))
        .collect()
}

fn source(file: u64, path: &str, symbols: Vec<&str>) -> ContributingSource {
    let symbols = symbols.into_iter().map(SymbolId::new).collect::<Vec<_>>();
    ContributingSource {
        file_id: FileId(file),
        path: PathBuf::from(path),
        symbol_count: symbols.len(),
        symbols,
    }
}

fn source_with_count(file: u64, path: &str, prefix: &str, count: usize) -> ContributingSource {
    let symbols = (0..count)
        .map(|i| SymbolId::new(format!("{prefix}_{i:03}")))
        .collect::<Vec<_>>();
    ContributingSource {
        file_id: FileId(file),
        path: PathBuf::from(path),
        symbol_count: count,
        symbols,
    }
}

fn metric_with(
    target: &str,
    semantic_sources: Vec<ContributingSource>,
    collateral_sources: Vec<ContributingSource>,
) -> LoadMetric {
    let semantic_load = semantic_sources.iter().map(|s| s.symbol_count).sum();
    let collateral_load = collateral_sources.iter().map(|s| s.symbol_count).sum();
    LoadMetric {
        target_name: target.to_string(),
        target_file: Some(PathBuf::from(target)),
        semantic_load,
        include_load: semantic_load + collateral_load,
        collateral_load,
        total_load: semantic_load + collateral_load,
        semantic_symbols: vec![],
        collateral_symbols: vec![],
        collateral_sources,
        largest_collateral_source: None,
        semantic_sources,
    }
}

fn empty_payload(
    file_justification: HashMap<String, HashMap<String, ariadne_ui::JustificationEntry>>,
) -> GraphExportPayload {
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
        file_justification,
        extraction_proposals: vec![],
    }
}

/// Fixture: graph.h(1) selects jobserver.h(2); X,Y,Z justified there require
/// a.h(5) and b.h(6); col1,col2 ride along as collateral.
fn basic_fixture() -> (
    IncludeGraph,
    ReducedSemanticModel,
    HashMap<PathBuf, u64>,
    Vec<LoadMetric>,
) {
    let graph = graph_with_paths(&[(1, "graph.h"), (2, "jobserver.h"), (5, "a.h"), (6, "b.h")]);
    let model = model_with(
        vec![
            sym("X", 2),
            sym("Y", 2),
            sym("Z", 2),
            sym("col1", 2),
            sym("col2", 2),
            sym("A", 5),
            sym("B", 6),
        ],
        vec![("X", "A"), ("Z", "B")],
    );
    let metrics = vec![metric_with(
        "graph.h",
        vec![source(2, "jobserver.h", vec!["X", "Y", "Z"])],
        vec![source(2, "jobserver.h", vec!["col1", "col2"])],
    )];
    let ids = path_to_id(&graph);
    (graph, model, ids, metrics)
}

#[test]
fn test_basic_entry_names_and_uncapped_counts() {
    let (graph, model, ids, metrics) = basic_fixture();
    let out = build_file_justification(&metrics, &graph, &model, &ids);

    let entry = out
        .get("1")
        .and_then(|root| root.get("2"))
        .expect("root 1 -> source 2 entry");
    assert_eq!(entry.justified_symbols, vec!["X", "Y", "Z"]);
    assert_eq!(entry.collateral_symbols, vec!["col1", "col2"]);
    assert_eq!(entry.justified_count, 3);
    assert_eq!(entry.collateral_count, 2);
    assert!(!entry.justified_truncated);
    assert!(!entry.collateral_truncated);
    assert_eq!(entry.required_headers, vec!["a.h", "b.h"]);
}

#[test]
fn test_name_cap_12_sets_truncation_flag() {
    let names: Vec<String> = (1..=13).map(|i| format!("n{i:02}")).collect();
    let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let graph = graph_with_paths(&[(1, "root.cc"), (2, "big.h")]);
    let model = model_with(names.iter().map(|n| sym(n, 2)).collect(), vec![]);
    let metrics = vec![metric_with(
        "root.cc",
        vec![source(2, "big.h", name_refs)],
        vec![],
    )];
    let out = build_file_justification(&metrics, &graph, &model, &path_to_id(&graph));

    let entry = out.get("1").unwrap().get("2").unwrap();
    assert_eq!(entry.justified_symbols.len(), 12);
    assert_eq!(entry.justified_symbols[0], "n01");
    assert_eq!(entry.justified_symbols[11], "n12");
    assert!(!entry.justified_symbols.contains(&"n13".to_string()));
    assert!(entry.justified_truncated);
    assert_eq!(entry.justified_count, 13, "count must stay uncapped");
    assert!(entry.collateral_symbols.is_empty());
    assert!(!entry.collateral_truncated);
    assert_eq!(entry.collateral_count, 0);
}

#[test]
fn test_top_16_contributors_kept_by_combined_count() {
    // 17 sources with combined counts 100..84; the two lowest must drop.
    let mut paths: Vec<(u64, &str)> = vec![(1, "root.cc")];
    let file_names: Vec<String> = (10..=26).map(|i| format!("src{i}.h")).collect();
    for (i, name) in (10..=26).zip(file_names.iter()) {
        paths.push((i, name.as_str()));
    }
    let graph = graph_with_paths(&paths);

    let mut symbols = Vec::new();
    let mut sources = Vec::new();
    for (idx, file) in (10..=26).enumerate() {
        let count = 100 - idx;
        let prefix = format!("f{file}");
        for i in 0..count {
            symbols.push(sym(&format!("{prefix}_{i:03}"), file));
        }
        sources.push(source_with_count(
            file,
            &format!("src{file}.h"),
            &prefix,
            count,
        ));
    }
    let model = model_with(symbols, vec![]);
    let metrics = vec![metric_with("root.cc", sources, vec![])];
    let out = build_file_justification(&metrics, &graph, &model, &path_to_id(&graph));

    let root = out.get("1").expect("root present");
    assert_eq!(root.len(), 16, "exactly 16 source entries survive the cap");
    let dropped: Vec<String> = (10..=26)
        .map(|i| i.to_string())
        .filter(|k| !root.contains_key(k))
        .collect();
    assert_eq!(
        dropped,
        vec!["26".to_string()],
        "the single lowest combined count (84) is the one dropped"
    );
    assert_eq!(
        root.get("25")
            .expect("85 must survive at the boundary")
            .justified_count,
        85
    );
    for (key, entry) in root {
        assert!(
            entry.justified_count >= 85,
            "kept entry {key} has count {}, below the top-16 floor",
            entry.justified_count
        );
    }
}

#[test]
fn test_build_is_deterministic_and_headers_sorted() {
    let (graph, model, ids, metrics) = basic_fixture();
    let first = build_file_justification(&metrics, &graph, &model, &ids);
    let second = build_file_justification(&metrics, &graph, &model, &ids);
    assert_eq!(first, second);

    let entry = first.get("1").unwrap().get("2").unwrap();
    let mut sorted = entry.required_headers.clone();
    sorted.sort();
    assert_eq!(entry.required_headers, sorted);
}

#[test]
fn test_empty_inputs_produce_empty_map_and_key_is_omitted() {
    let graph = IncludeGraph::new();
    let model = model_with(vec![], vec![]);
    let out = build_file_justification(&[], &graph, &model, &HashMap::new());
    assert!(out.is_empty());

    let serialized = serde_json::to_string(&empty_payload(HashMap::new())).unwrap();
    assert!(
        !serialized.contains("file_justification"),
        "empty file_justification is skipped during serialization; clients must default it"
    );
    let back: GraphExportPayload = serde_json::from_str(&serialized).unwrap();
    assert!(back.file_justification.is_empty());
}

#[test]
fn test_required_headers_exclude_source_basename_and_sort() {
    let graph = graph_with_paths(&[
        (1, "graph.h"),
        (2, "jobserver.h"),
        (5, "a.h"),
        (6, "b.h"),
        (7, "vendor/inc/util.h"),
    ]);
    // X,Y,Z live in jobserver.h itself; their closure reaches util.h, a.h, b.h
    // and a same-file symbol K that must never surface as a required header.
    let model = model_with(
        vec![
            sym("X", 2),
            sym("Y", 2),
            sym("Z", 2),
            sym("K", 2),
            sym("A", 5),
            sym("B", 6),
            sym("U", 7),
        ],
        vec![("X", "A"), ("Y", "K"), ("Y", "U"), ("Z", "B")],
    );
    let metrics = vec![metric_with(
        "graph.h",
        vec![source(2, "jobserver.h", vec!["X", "Y", "Z"])],
        vec![],
    )];
    let out = build_file_justification(&metrics, &graph, &model, &path_to_id(&graph));

    let entry = out.get("1").unwrap().get("2").unwrap();
    assert!(
        !entry.required_headers.contains(&"jobserver.h".to_string()),
        "source header must never appear in its own required set: {:?}",
        entry.required_headers
    );
    assert_eq!(entry.required_headers, vec!["a.h", "b.h", "util.h"]);
}

#[test]
fn test_symbols_missing_from_model_fall_back_to_id_string() {
    let graph = graph_with_paths(&[(1, "root.cc"), (2, "dep.h")]);
    let model = model_with(vec![sym("Real", 2)], vec![]);
    let metrics = vec![metric_with(
        "root.cc",
        vec![source(2, "dep.h", vec!["Ghost", "Real"])],
        vec![],
    )];
    let out = build_file_justification(&metrics, &graph, &model, &path_to_id(&graph));

    let entry = out.get("1").unwrap().get("2").unwrap();
    assert_eq!(entry.justified_symbols, vec!["Ghost", "Real"]);
    assert_eq!(entry.justified_count, 2);
}

#[test]
fn test_payload_round_trip_carries_justification_entries() {
    let (graph, model, ids, metrics) = basic_fixture();
    let mut payload = empty_payload(build_file_justification(&metrics, &graph, &model, &ids));
    let serialized = serde_json::to_string(&payload).unwrap();
    assert!(serialized.contains("file_justification"));

    payload
        .file_justification
        .get_mut("1")
        .unwrap()
        .get_mut("2")
        .unwrap()
        .required_headers
        .push("z.h".to_string());
    let back: GraphExportPayload = serde_json::from_str(&serialized).unwrap();
    let round = back.file_justification.get("1").unwrap().get("2").unwrap();
    assert_eq!(round.required_headers, vec!["a.h", "b.h"]);
    assert_eq!(round.justified_symbols, vec!["X", "Y", "Z"]);
}

// --- E2: bounded ImpactSummary on payload entries (plan 1790521470000) -----------------

fn include_edge(from: u64, to: u64) -> IncludeEdge {
    IncludeEdge {
        source: FileId(from),
        destination: Some(FileId(to)),
        context_name: String::new(),
        source_location: SourceLocation { line: 1, column: 1 },
        kind: IncludeKind::Local,
        raw_path: String::new(),
        resolved_path: None,
        language: ariadne_repo::SourceLanguage::Cpp,
    }
}

fn json_keys(value: &serde_json::Value) -> Vec<String> {
    let mut keys: Vec<String> = value
        .as_object()
        .expect("expected a JSON object")
        .keys()
        .cloned()
        .collect();
    keys.sort();
    keys
}

/// Sibling-path shape replicated from load_impact_tests at the DTO level:
/// root.cc(1) -> fat.h(2), fat.h -> base.h(3), fat.h -> c.h(4),
/// c.h -> d.h(5), c.h -> e.h(6), plus the sibling edge root.cc -> d.h(5).
/// FatA/FatB justified on fat.h need only base.h, so the slim edge sheds
/// {c, d, e} but d.h still arrives via the sibling edge.
fn impact_fixture() -> (
    IncludeGraph,
    ReducedSemanticModel,
    HashMap<PathBuf, u64>,
    Vec<LoadMetric>,
) {
    let mut graph = graph_with_paths(&[
        (1, "root.cc"),
        (2, "fat.h"),
        (3, "base.h"),
        (4, "c.h"),
        (5, "d.h"),
        (6, "e.h"),
        (7, "onlycol.h"),
        (8, "onlyjust.h"),
    ]);
    for (from, to) in [(1u64, 2u64), (1, 5), (2, 3), (2, 4), (4, 5), (4, 6)] {
        graph.edges.push(include_edge(from, to));
    }
    let mut model = model_with(
        vec![
            sym("FatA", 2),
            sym("FatB", 2),
            sym("ColA", 2),
            sym("BaseT", 3),
            sym("Csym", 4),
            sym("Dsym", 5),
            sym("Esym", 6),
            sym("OC1", 7),
            sym("OJ1", 8),
            sym("SelfJ", 1),
            sym("SelfC", 1),
        ],
        vec![("FatA", "BaseT"), ("FatB", "BaseT")],
    );
    model.file_symbols = vec![
        (
            FileId(2),
            vec![
                SymbolId::new("FatA"),
                SymbolId::new("FatB"),
                SymbolId::new("ColA"),
            ],
        ),
        (FileId(3), vec![SymbolId::new("BaseT")]),
        (FileId(4), vec![SymbolId::new("Csym")]),
        (FileId(5), vec![SymbolId::new("Dsym")]),
        (FileId(6), vec![SymbolId::new("Esym")]),
    ]
    .into_iter()
    .collect::<BTreeMap<_, _>>();
    let metrics = vec![metric_with(
        "root.cc",
        vec![
            source(2, "fat.h", vec!["FatA", "FatB"]),
            source(1, "root.cc", vec!["SelfJ"]),
            source(8, "onlyjust.h", vec!["OJ1"]),
        ],
        vec![
            source(2, "fat.h", vec!["ColA"]),
            source(1, "root.cc", vec!["SelfC"]),
            source(7, "onlycol.h", vec!["OC1"]),
        ],
    )];
    let ids = path_to_id(&graph);
    (graph, model, ids, metrics)
}

#[test]
fn test_impact_present_on_eligible_entry_with_correct_drop_counts() {
    let (graph, model, ids, metrics) = impact_fixture();
    let out = build_file_justification(&metrics, &graph, &model, &ids);
    let entry = out.get("1").unwrap().get("2").expect("fat.h entry");

    // Hand computation (strict-closure convention, seed excluded):
    //   closure(root.cc) = {fat, base, c, d, e}                    -> before 5
    //   R = {base}; closure_after = {fat, base, d} (d via sibling) -> after 3
    //   dropped_from_root = {c.h, e.h}; still_reachable = {d.h}
    //   symbols_no_longer_parsed = Csym(1) + Esym(1) = 2
    let expected = ariadne_ui::ImpactSummary {
        dropped_from_root_count: 2,
        still_reachable_count: 1,
        dropped_sample: vec!["c.h".to_string(), "e.h".to_string()],
        dropped_sample_truncated: false,
        root_closure_before: 5,
        root_closure_after: 3,
        symbols_no_longer_parsed: 2,
    };
    assert_eq!(entry.impact.as_ref(), Some(&expected));
}

#[test]
fn test_impact_absent_on_ineligible_entries() {
    let (graph, model, ids, metrics) = impact_fixture();
    let out = build_file_justification(&metrics, &graph, &model, &ids);
    let root = out.get("1").unwrap();

    assert!(
        root.get("1").is_none(),
        "root.cc self row is dropped by the header-only source filter (E-I9 mirror)"
    );
    assert!(
        root.get("7").expect("collateral-only row").impact.is_none(),
        "collateral_count == 0 is not eligible (E-I4)"
    );
    assert!(
        root.get("8").expect("justified-only row").impact.is_none(),
        "justified-only rows carry no collateral to shed; E-I4 excludes them"
    );
}

#[test]
fn test_impact_capped_to_eight_entries_per_root_by_rank() {
    // 9 eligible sources (justified>0 and collateral>0 on the same file),
    // combined counts 10..2 so rank order is id order: files 10..17 must
    // carry impact (top 8), file 18 (lowest rank) must not.
    let mut paths: Vec<(u64, &str)> = vec![(1, "root.cc")];
    let names: Vec<String> = (10..=18).map(|i| format!("s{i}.h")).collect();
    for (i, name) in (10..=18).zip(names.iter()) {
        paths.push((i, name.as_str()));
    }
    let graph = graph_with_paths(&paths);

    let mut semantic = Vec::new();
    let mut collateral = Vec::new();
    let znames: Vec<String> = (10..=18).map(|i| format!("z{i}")).collect();
    for (idx, i) in (10..=18u64).enumerate() {
        semantic.push(source(i, &format!("s{i}.h"), vec![znames[idx].as_str()]));
        collateral.push(source_with_count(
            i,
            &format!("s{i}.h"),
            &format!("c{i}"),
            19 - i as usize,
        ));
    }
    let model = model_with(vec![], vec![]);
    let metrics = vec![metric_with("root.cc", semantic, collateral)];
    let out = build_file_justification(&metrics, &graph, &model, &path_to_id(&graph));
    let root = out.get("1").unwrap();

    let with_impact: Vec<&String> = root
        .iter()
        .filter(|(_, e)| e.impact.is_some())
        .map(|(k, _)| k)
        .collect();
    assert_eq!(
        with_impact.len(),
        8,
        "exactly 8 entries per root carry impact"
    );
    for i in 10..=17 {
        assert!(
            root.get(&i.to_string()).unwrap().impact.is_some(),
            "ranked-eligible source {i} must carry impact"
        );
    }
    assert!(
        root.get("18").unwrap().impact.is_none(),
        "the 9th eligible source (lowest rank) must be None"
    );
}

#[test]
fn test_impact_dropped_sample_caps_at_eight_with_truncation_flag() {
    // fat9.h drops 9 headers (d0..d8): sample == first 8 path-asc, truncated.
    // fat8.h drops exactly 8 (e0..e7): sample == 8, not truncated.
    let dnames: Vec<String> = (0..9).map(|i| format!("d{i}.h")).collect();
    let enames: Vec<String> = (0..8).map(|i| format!("e{i}.h")).collect();
    let mut paths: Vec<(u64, &str)> = vec![(1, "root.cc"), (2, "fat9.h"), (3, "fat8.h")];
    for (idx, name) in dnames.iter().enumerate() {
        paths.push((20 + idx as u64, name));
    }
    for (idx, name) in enames.iter().enumerate() {
        paths.push((30 + idx as u64, name));
    }
    let mut graph = graph_with_paths(&paths);
    graph.edges.push(include_edge(1, 2));
    graph.edges.push(include_edge(1, 3));
    for i in 0..9 {
        graph.edges.push(include_edge(2, 20 + i));
    }
    for i in 0..8 {
        graph.edges.push(include_edge(3, 30 + i));
    }
    // F9/F8 have no semantic edges: R = {} and the whole child set drops.
    let model = model_with(
        vec![sym("F9", 2), sym("C9", 2), sym("F8", 3), sym("C8", 3)],
        vec![],
    );
    let metrics = vec![metric_with(
        "root.cc",
        vec![
            source(2, "fat9.h", vec!["F9"]),
            source(3, "fat8.h", vec!["F8"]),
        ],
        vec![
            source(2, "fat9.h", vec!["C9"]),
            source(3, "fat8.h", vec!["C8"]),
        ],
    )];
    let out = build_file_justification(&metrics, &graph, &model, &path_to_id(&graph));
    let root = out.get("1").unwrap();

    let nine = root
        .get("2")
        .unwrap()
        .impact
        .as_ref()
        .expect("9-drop impact");
    assert_eq!(nine.dropped_from_root_count, 9);
    assert_eq!(
        nine.dropped_sample,
        vec!["d0.h", "d1.h", "d2.h", "d3.h", "d4.h", "d5.h", "d6.h", "d7.h"]
    );
    assert!(
        nine.dropped_sample_truncated,
        "9 > 8 marks the sample truncated"
    );
    assert_eq!(nine.root_closure_before, 19);
    assert_eq!(nine.root_closure_after, 10);

    let eight = root
        .get("3")
        .unwrap()
        .impact
        .as_ref()
        .expect("8-drop impact");
    assert_eq!(eight.dropped_from_root_count, 8);
    assert_eq!(
        eight.dropped_sample,
        vec!["e0.h", "e1.h", "e2.h", "e3.h", "e4.h", "e5.h", "e6.h", "e7.h"]
    );
    assert!(
        !eight.dropped_sample_truncated,
        "exactly 8 must not truncate"
    );
}

#[test]
fn test_impact_summary_serde_key_lock_and_none_omission() {
    let (graph, model, ids, metrics) = impact_fixture();
    let out = build_file_justification(&metrics, &graph, &model, &ids);
    let root = out.get("1").unwrap();

    let some_json = serde_json::to_value(root.get("2").unwrap()).unwrap();
    assert_eq!(
        json_keys(&some_json),
        vec![
            "collateral_count",
            "collateral_symbols",
            "collateral_truncated",
            "impact",
            "justified_count",
            "justified_symbols",
            "justified_truncated",
            "required_headers",
        ]
    );
    assert_eq!(
        json_keys(&some_json["impact"]),
        vec![
            "dropped_from_root_count",
            "dropped_sample",
            "dropped_sample_truncated",
            "root_closure_after",
            "root_closure_before",
            "still_reachable_count",
            "symbols_no_longer_parsed",
        ],
        "ImpactSummary field names are a locked wire contract"
    );

    let none_json = serde_json::to_value(root.get("7").unwrap()).unwrap();
    assert!(
        none_json.get("impact").is_none(),
        "None impact must be ABSENT from the JSON (skip_serializing_if)"
    );
}

// --- Header-only source filter (E-I9 mirror in the UI justification rows) -------------

/// root.cc(1) inspects: jobserver.h(2) justified X + collateral col1;
/// root.cc(1) self row justified SelfJ + collateral SelfC (compute_load
/// inserts the root into its own closure); other.cc(9) collateral-only
/// defining file; onlycol.h(7) collateral-only header (unnecessary include).
fn mixed_path_fixture() -> (
    IncludeGraph,
    ReducedSemanticModel,
    HashMap<PathBuf, u64>,
    Vec<LoadMetric>,
) {
    let graph = graph_with_paths(&[
        (1, "root.cc"),
        (2, "jobserver.h"),
        (7, "onlycol.h"),
        (9, "other.cc"),
    ]);
    let model = model_with(
        vec![
            sym("X", 2),
            sym("col1", 2),
            sym("SelfJ", 1),
            sym("SelfC", 1),
            sym("OC1", 7),
            sym("OC2", 7),
            sym("OtherC", 9),
        ],
        vec![],
    );
    let metrics = vec![metric_with(
        "root.cc",
        vec![
            source(2, "jobserver.h", vec!["X"]),
            source(1, "root.cc", vec!["SelfJ"]),
        ],
        vec![
            source(2, "jobserver.h", vec!["col1"]),
            source(1, "root.cc", vec!["SelfC"]),
            source(9, "other.cc", vec!["OtherC"]),
            source(7, "onlycol.h", vec!["OC1", "OC2"]),
        ],
    )];
    let ids = path_to_id(&graph);
    (graph, model, ids, metrics)
}

#[test]
fn test_payload_rows_are_header_files_only() {
    let (graph, model, ids, metrics) = mixed_path_fixture();
    let out = build_file_justification(&metrics, &graph, &model, &ids);
    let root = out.get("1").expect("root present");

    assert!(
        root.contains_key("2"),
        "justified+collateral header survives"
    );
    assert!(
        root.contains_key("7"),
        "collateral-only header must survive: it IS the unnecessary-include signal"
    );
    assert!(
        !root.contains_key("1"),
        "root.cc self row must not pollute the justification list"
    );
    assert!(
        !root.contains_key("9"),
        "other.cc defining-file row must not pollute the justification list"
    );
    assert_eq!(root.len(), 2, "only the two header sources remain");
}

#[test]
fn test_detail_rows_are_header_files_only_and_totals_exclude_dropped() {
    let (graph, model, _ids, metrics) = mixed_path_fixture();
    let analyzer = ariadne_graph::LoadAnalyzer::new(&graph, &model);
    let detail = build_file_justification_detail("1", FileId(1), &metrics[0], &analyzer, &model);

    assert_eq!(
        detail
            .entries
            .iter()
            .map(|e| e.source_label.as_str())
            .collect::<Vec<_>>(),
        vec!["jobserver.h", "onlycol.h"],
        "combined counts tie at 2, so source_id asc orders jobserver before onlycol"
    );
    assert!(
        !detail
            .entries
            .iter()
            .any(|e| e.source_label.ends_with(".cc") || e.source_id == "1" || e.source_id == "9"),
        "no .cc source rows may survive: {:?}",
        detail
            .entries
            .iter()
            .map(|e| (&e.source_id, &e.source_label))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        detail.total_justified, 1,
        "SelfJ (root.cc self row) must not count"
    );
    assert_eq!(
        detail.total_collateral, 3,
        "SelfC (root.cc) and OtherC (other.cc) must not count; col1+OC1+OC2 do"
    );
}

#[test]
fn test_collateral_only_header_row_survives_zero_justified_on_both_builders() {
    let (graph, model, ids, metrics) = mixed_path_fixture();
    let payload = build_file_justification(&metrics, &graph, &model, &ids);
    let only_col = payload
        .get("1")
        .and_then(|root| root.get("7"))
        .expect("collateral-only header row must not be filtered for having 0 justified");
    assert_eq!(only_col.justified_count, 0);
    assert_eq!(only_col.collateral_count, 2);
    assert!(only_col.justified_symbols.is_empty());
    assert_eq!(only_col.collateral_symbols, vec!["OC1", "OC2"]);

    let analyzer = ariadne_graph::LoadAnalyzer::new(&graph, &model);
    let detail = build_file_justification_detail("1", FileId(1), &metrics[0], &analyzer, &model);
    let row = detail
        .entries
        .iter()
        .find(|e| e.source_id == "7")
        .expect("unnecessary-include row present on the live route too");
    assert_eq!(row.entry.justified_count, 0);
    assert_eq!(row.entry.collateral_count, 2);
    assert!(
        detail.total_collateral >= 2,
        "collateral-only row counts toward totals: got {}",
        detail.total_collateral
    );
}
