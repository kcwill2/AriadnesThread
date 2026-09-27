use ariadne_graph::{
    ContributingSource, IncludeGraph, LoadMetric, ReducedSemanticEdge, ReducedSemanticModel,
    ReducedSymbolRecord,
};
use ariadne_index::{DependencyKind, SymbolId, SymbolKind};
use ariadne_repo::FileId;
use ariadne_ui::{build_file_justification, GraphExportPayload, SummaryStats};
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

fn empty_payload(file_justification: HashMap<String, HashMap<String, ariadne_ui::JustificationEntry>>) -> GraphExportPayload {
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
    let graph = graph_with_paths(&[
        (1, "graph.h"),
        (2, "jobserver.h"),
        (5, "a.h"),
        (6, "b.h"),
    ]);
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
    assert!(
        !root.contains_key("25") && !root.contains_key("26"),
        "the two lowest combined counts (85 and 84) must be dropped"
    );
    for (key, entry) in root {
        assert!(
            entry.justified_count >= 86,
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

    payload.file_justification.get_mut("1").unwrap().get_mut("2").unwrap().required_headers.push("z.h".to_string());
    let back: GraphExportPayload = serde_json::from_str(&serialized).unwrap();
    let round = back.file_justification.get("1").unwrap().get("2").unwrap();
    assert_eq!(round.required_headers, vec!["a.h", "b.h"]);
    assert_eq!(round.justified_symbols, vec!["X", "Y", "Z"]);
}
