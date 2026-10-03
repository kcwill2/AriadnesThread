//! A6 characterization lock: `build_file_justification` output must stay
//! byte-identical across the rayon parallelization. The golden fixture under
//! `tests/fixtures/` is generated from the pre-A6 SERIAL code (regenerate
//! with `ARIADNE_UPDATE_GOLDEN=1` only if the payload contract changes on
//! purpose — the review must then bless the diff). Serialization is
//! canonicalized through `serde_json::Value`, whose map is a BTreeMap, so
//! HashMap iteration order can never leak into the fixture.

use ariadne_graph::{
    ContributingSource, IncludeEdge, IncludeGraph, IncludeKind, LanguageId, LoadMetric,
    ReducedSemanticEdge, ReducedSemanticModel, ReducedSymbolRecord, SourceLocation,
};
use ariadne_index::{DependencyKind, SymbolId, SymbolKind};
use ariadne_repo::FileId;
use ariadne_ui::build_file_justification;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

const HEADER_COUNT: u64 = 48;
const TU_ID_BASE: u64 = HEADER_COUNT + 1;

fn header_path(index: u64) -> String {
    format!("include/h{index:02}.h")
}

fn tu_path(tu: u64) -> String {
    format!("src/tu{}.cpp", tu + 1)
}

/// Fanout targets of header index `i` (0-based; file ids are `index + 1`).
fn fanout(index: u64) -> (u64, u64) {
    ((index + 7) % HEADER_COUNT, (index + 13) % HEADER_COUNT)
}

fn sym(id: &str, file: u64) -> ReducedSymbolRecord {
    ReducedSymbolRecord {
        id: SymbolId(id.to_string()),
        name: id.to_string(),
        kind: SymbolKind::Function,
        defining_file: Some(FileId(file)),
        declarations: vec![],
        definition: None,
        declaring_files: vec![],
    }
}

fn include_edge(from: u64, to: u64) -> IncludeEdge {
    IncludeEdge {
        source: FileId(from),
        destination: Some(FileId(to)),
        context_name: String::new(),
        source_location: SourceLocation { line: 1, column: 1 },
        kind: IncludeKind::Local,
        raw_path: String::new(),
        resolved_path: None,
        language: LanguageId::Cpp,
    }
}

fn sem_edge(src: &str, dst: &str) -> ReducedSemanticEdge {
    ReducedSemanticEdge {
        src_symbol: SymbolId(src.to_string()),
        dst_symbol: SymbolId(dst.to_string()),
        kind: DependencyKind::Complete,
    }
}

fn source(file: u64, path: &str, symbols: Vec<String>) -> ContributingSource {
    let symbols = symbols.into_iter().map(SymbolId::new).collect::<Vec<_>>();
    ContributingSource {
        file_id: FileId(file),
        path: PathBuf::from(path),
        symbol_count: symbols.len(),
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

/// The 3 model symbols defined in header index `i` (file id `i + 1`).
fn header_symbols(index: u64) -> Vec<String> {
    vec![
        format!("h{index:02}_f"),
        format!("h{index:02}_u"),
        format!("h{index:02}_s"),
    ]
}

/// Justified symbols on root `index` for source `t`: the source's own symbols
/// (whose model edges make the closure cross into another header, so
/// required_headers is non-empty) plus the root's own symbol, a seed defined
/// in the ROOT and therefore excluded from required headers.
fn justified_at(root_index: u64, t: u64) -> Vec<String> {
    let mut symbols = header_symbols(t);
    symbols.push(format!("h{root_index:02}_s"));
    symbols
}

fn tu_roots(tu: u64) -> Vec<u64> {
    vec![(tu * 11) % 48, (tu * 11 + 17) % 48, (tu * 11 + 29) % 48]
}

fn build_fixture_graph() -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    for index in 0..HEADER_COUNT {
        graph
            .file_paths
            .insert(FileId(index + 1), PathBuf::from(header_path(index)));
        let (t0, t1) = fanout(index);
        graph.edges.push(include_edge(index + 1, t0 + 1));
        graph.edges.push(include_edge(index + 1, t1 + 1));
    }
    for tu in 0..4 {
        graph
            .file_paths
            .insert(FileId(TU_ID_BASE + tu), PathBuf::from(tu_path(tu)));
        for root in tu_roots(tu) {
            graph.edges.push(include_edge(TU_ID_BASE + tu, root + 1));
        }
    }
    // Shedding pin (ids 55-57): R -> S -> L with no back path, so the
    // edge R->S (justified symbol without closure edges) drops L from root
    // R — the only fixture case with a non-empty dropped_from_root sample.
    graph
        .file_paths
        .insert(FileId(55), PathBuf::from(header_path(54)));
    graph
        .file_paths
        .insert(FileId(56), PathBuf::from(header_path(55)));
    graph
        .file_paths
        .insert(FileId(57), PathBuf::from(header_path(56)));
    graph.edges.push(include_edge(56, 55));
    graph.edges.push(include_edge(57, 56));
    graph
}

fn build_fixture_model() -> ReducedSemanticModel {
    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("."),
        symbols: BTreeMap::new(),
        edges: Vec::new(),
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),
        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };
    for index in 0..HEADER_COUNT {
        let (t0, t1) = fanout(index);
        let registered = header_symbols(index)
            .into_iter()
            .map(|id| sym(&id, index + 1))
            .collect::<Vec<_>>();
        for record in &registered {
            model.symbols.insert(record.id.clone(), record.clone());
        }
        model.file_symbols.insert(
            FileId(index + 1),
            registered.into_iter().map(|r| r.id).collect(),
        );
        model
            .edges
            .push(sem_edge(&format!("h{index:02}_f"), &format!("h{t0:02}_s")));
        model
            .edges
            .push(sem_edge(&format!("h{index:02}_u"), &format!("h{t1:02}_u")));
    }
    // Truncation fixture: 12 extra registered symbols on h00 — with its 3
    // own symbols one entry carries 15 names against the 12-name cap.
    for extra in 0..12 {
        let id = format!("h00_x{extra:03}");
        model.symbols.insert(SymbolId(id.clone()), sym(&id, 1));
        model
            .file_symbols
            .get_mut(&FileId(1))
            .unwrap()
            .push(SymbolId(id));
    }
    for tu in 0..4 {
        let drive = format!("tu{tu}_drive");
        model
            .symbols
            .insert(SymbolId(drive.clone()), sym(&drive, TU_ID_BASE + tu));
        for root in tu_roots(tu).iter().take(2) {
            model.edges.push(sem_edge(&drive, &format!("h{root:02}_s")));
        }
    }
    // Shedding pin symbols (edgeless, so required_headers for the R->S edge
    // is empty and the whole S-subtree drops): leafsym def 55, mid_f/midsym
    // def 56.
    model
        .symbols
        .insert(SymbolId("leafsym".to_string()), sym("leafsym", 55));
    model
        .file_symbols
        .insert(FileId(55), vec![SymbolId("leafsym".to_string())]);
    for mid in ["mid_f", "midsym"] {
        model
            .symbols
            .insert(SymbolId(mid.to_string()), sym(mid, 56));
    }
    model.file_symbols.insert(
        FileId(56),
        vec![
            SymbolId("mid_f".to_string()),
            SymbolId("midsym".to_string()),
        ],
    );
    model
}

/// One header root: fanout justified sources + one collateral; root 0 adds a
/// 15-symbol self source and 14 further sources (crosses the 16-source cap
/// and the 12-name truncation), root 1 adds a self-edge entry, root 2 adds 12
/// both-side sources so all 8 impact slots are consumed with 4 eligible
/// entries left without a summary.
fn header_metric(index: u64) -> LoadMetric {
    let (t0, t1) = fanout(index);
    let mut justified = vec![
        source(t0 + 1, &header_path(t0), justified_at(index, t0)),
        source(t1 + 1, &header_path(t1), header_symbols(t1)),
    ];
    let mut collateral = vec![];
    if index == 0 {
        let mut wide: Vec<String> = (0..12).map(|i| format!("h00_x{i:03}")).collect();
        wide.extend(header_symbols(0));
        justified.push(source(1, &header_path(0), wide));
        for j in 0..14 {
            let extra = (5 + j * 3) % HEADER_COUNT;
            justified.push(source(
                extra + 1,
                &header_path(extra),
                header_symbols(extra),
            ));
        }
    }
    if index == 1 {
        justified.push(source(2, &header_path(1), header_symbols(1)));
    }
    if index == 2 {
        for m in 0..12u64 {
            let both = (4 + 3 * m) % HEADER_COUNT;
            justified.push(source(both + 1, &header_path(both), header_symbols(both)));
            collateral.push(source(both + 1, &header_path(both), header_symbols(both)));
        }
    }
    let col = (index + 23) % HEADER_COUNT;
    collateral.push(source(col + 1, &header_path(col), header_symbols(col)));
    metric_with(&header_path(index), justified, collateral)
}

fn tu_metric(tu: u64) -> LoadMetric {
    let justified = tu_roots(tu)
        .iter()
        .map(|root| source(root + 1, &header_path(*root), header_symbols(*root)))
        .collect();
    let col = (tu * 7 + 3) % HEADER_COUNT;
    metric_with(
        &tu_path(tu),
        justified,
        vec![source(col + 1, &header_path(col), header_symbols(col))],
    )
}

/// 48 headers + 4 TU roots, one metric each, plus two skip pins: an
/// unregistered target and a metric whose only source is a `.cpp` (filtered
/// by E-I9, leaving empty sides) on a root that already has a real metric.
fn build_fixture_metrics() -> Vec<LoadMetric> {
    let mut metrics = Vec::new();
    for index in 0..HEADER_COUNT {
        metrics.push(header_metric(index));
    }
    for tu in 0..4 {
        metrics.push(tu_metric(tu));
    }
    // Shedding-pin root (id 57): justified mid_f has no closure edges, so
    // h54.h (L) drops out of root R when edge 57->56 goes slim.
    metrics.push(metric_with(
        &header_path(56),
        vec![source(56, &header_path(55), vec!["mid_f".to_string()])],
        vec![
            source(56, &header_path(55), vec!["midsym".to_string()]),
            source(55, &header_path(54), vec!["leafsym".to_string()]),
        ],
    ));
    metrics.push(metric_with("gone.cpp", vec![], vec![]));
    metrics.push(metric_with(
        &header_path(5),
        vec![],
        vec![source(
            TU_ID_BASE + 1,
            &tu_path(1),
            vec!["tu1_drive".to_string()],
        )],
    ));
    metrics
}

fn fixture() -> (
    IncludeGraph,
    ReducedSemanticModel,
    HashMap<PathBuf, u64>,
    Vec<LoadMetric>,
) {
    let graph = build_fixture_graph();
    let ids: HashMap<PathBuf, u64> = graph
        .file_paths
        .iter()
        .map(|(id, path)| (path.clone(), id.0))
        .collect();
    (graph, build_fixture_model(), ids, build_fixture_metrics())
}

fn canonical_json(
    out: &HashMap<String, HashMap<String, ariadne_ui::JustificationEntry>>,
) -> String {
    serde_json::to_string_pretty(&serde_json::to_value(out).unwrap()).unwrap()
}

fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/justification_golden.json")
}

#[test]
fn file_justification_matches_golden_fixture() {
    let (graph, model, ids, metrics) = fixture();
    let serialized = canonical_json(&build_file_justification(&metrics, &graph, &model, &ids));

    let path = golden_path();
    if std::env::var("ARIADNE_UPDATE_GOLDEN").is_ok() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, format!("{serialized}\n")).unwrap();
    }
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("golden fixture {} missing: {e}", path.display()));
    assert_eq!(
        serialized,
        golden.trim_end(),
        "file justification payload drifted from the committed golden"
    );
}

#[test]
fn file_justification_is_deterministic_across_ten_runs() {
    let (graph, model, ids, metrics) = fixture();
    let first = canonical_json(&build_file_justification(&metrics, &graph, &model, &ids));
    for run in 1..10 {
        let again = canonical_json(&build_file_justification(&metrics, &graph, &model, &ids));
        assert_eq!(again, first, "run {run} diverged from run 0");
    }
}

#[test]
fn golden_fixture_covers_every_interesting_shape() {
    let (graph, model, ids, metrics) = fixture();
    let out = build_file_justification(&metrics, &graph, &model, &ids);

    assert_eq!(out.len(), 53, "48 headers + 4 TUs + shedding pin");
    let shed = &out["57"]["56"];
    let shed_impact = shed.impact.as_ref().expect("shedding pin carries impact");
    assert_eq!(shed.required_headers, Vec::<String>::new());
    assert_eq!(shed_impact.dropped_from_root_count, 1);
    assert_eq!(shed_impact.dropped_sample, vec!["h54.h".to_string()]);
    assert!(!shed_impact.dropped_sample_truncated);
    assert_eq!(shed_impact.still_reachable_count, 0);
    assert_eq!(shed_impact.symbols_no_longer_parsed, 1);
    assert_eq!(
        out["6"].len(),
        3,
        "unregistered/.cpp-only metrics add nothing"
    );

    let root1 = &out["1"];
    assert_eq!(root1.len(), 16, "source cap (I3) is in play");
    assert!(
        root1
            .values()
            .any(|e| e.justified_truncated && e.justified_symbols.len() == 12),
        "name truncation (12) is in play"
    );
    assert!(
        root1.values().any(|e| e.impact.is_some()),
        "impact summaries are in play"
    );
    assert_eq!(out["3"].len(), 14);
    assert_eq!(
        out["3"].values().filter(|e| e.impact.is_some()).count(),
        8,
        "impact slot cap (8/root) is in play"
    );
    assert!(
        out["3"]
            .values()
            .filter(|e| e.justified_count > 0 && e.collateral_count > 0 && e.impact.is_none())
            .count()
            >= 4,
        "eligible entries past the slot cap carry no summary"
    );
    assert!(
        out["2"]["2"].impact.is_none(),
        "self-edge stays impact-ineligible"
    );
    assert!(
        root1.values().any(|e| e.required_headers.len() == 8),
        "required-header cap (8) is in play"
    );
    assert!(
        out.values().all(|group| group.values().all(|e| {
            let mut sorted = e.required_headers.clone();
            sorted.sort();
            sorted.dedup();
            sorted == e.required_headers
        })),
        "required headers are canonicalized (sorted, deduped) everywhere"
    );
    assert_eq!(model.symbols.len(), 48 * 3 + 12 + 4 + 3);
    assert!(!graph.edges.is_empty());
    assert_eq!(metrics.len(), 55);
}
