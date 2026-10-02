use super::*;
use crate::reduced::{ReducedSemanticEdge, ReducedSymbolRecord};
use crate::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_index::types::{DependencyKind, SymbolId, SymbolKind};
use ariadne_repo::FileId;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

// Thing.h semantically needs Helper symbols (via Thing) and Base symbols,
// while HugeThirdParty symbols (dragged in by Helper.h) stay unused.
// Semantic side: Thing.h, Helper.h, Base.h groups. Collateral side: HugeThirdParty.h.
fn setup_justification_fixture() -> (IncludeGraph, ReducedSemanticModel) {
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(1), PathBuf::from("Thing.h"));
    ig.register_file(FileId(2), PathBuf::from("Helper.h"));
    ig.register_file(FileId(3), PathBuf::from("HugeThirdParty.h"));
    ig.register_file(FileId(4), PathBuf::from("Base.h"));

    let include = |ig: &mut IncludeGraph, from: FileId, to: FileId, name: &str| {
        ig.add_edge(IncludeEdge {
            source: from,
            destination: Some(to),
            context_name: "".to_string(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: name.to_string(),
            resolved_path: Some(PathBuf::from(name)),
            language: ariadne_repo::SourceLanguage::Cpp,
        });
    };
    include(&mut ig, FileId(1), FileId(2), "Helper.h");
    include(&mut ig, FileId(1), FileId(4), "Base.h");
    include(&mut ig, FileId(2), FileId(3), "HugeThirdParty.h");

    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };

    let add_sym = |model: &mut ReducedSemanticModel, name: &str, file: FileId| {
        let id = SymbolId::new(name);
        model.symbols.insert(
            id.clone(),
            ReducedSymbolRecord {
                id: id.clone(),
                name: name.to_string(),
                kind: SymbolKind::Function,
                defining_file: Some(file),
                declarations: vec![],
                definition: None,
                declaring_files: vec![],
            },
        );
        id
    };

    let thing = add_sym(&mut model, "Thing", FileId(1));
    let helper_fn = add_sym(&mut model, "HelperFn", FileId(2));
    let helper_fn2 = add_sym(&mut model, "HelperFn2", FileId(2));
    let helper_util = add_sym(&mut model, "HelperUtil", FileId(2));
    let base_struct = add_sym(&mut model, "BaseStruct", FileId(4));
    add_sym(&mut model, "HugeThing", FileId(3));
    add_sym(&mut model, "HugeOther", FileId(3));

    model.file_symbols.insert(FileId(1), vec![thing.clone()]);
    model.file_symbols.insert(
        FileId(2),
        vec![helper_fn.clone(), helper_fn2.clone(), helper_util.clone()],
    );
    model.file_symbols.insert(
        FileId(3),
        vec![SymbolId::new("HugeThing"), SymbolId::new("HugeOther")],
    );
    model
        .file_symbols
        .insert(FileId(4), vec![base_struct.clone()]);

    let dep = |src: &SymbolId, dst: &SymbolId| ReducedSemanticEdge {
        src_symbol: src.clone(),
        dst_symbol: dst.clone(),
        kind: DependencyKind::Complete,
    };
    model.edges = vec![
        dep(&thing, &helper_fn),
        dep(&helper_fn, &helper_fn2),
        dep(&thing, &helper_util),
        dep(&thing, &base_struct),
    ];

    (ig, model)
}

fn source_names(sources: &[CollateralSource]) -> Vec<(String, usize)> {
    sources
        .iter()
        .map(|s| {
            (
                s.path.file_name().unwrap().to_string_lossy().to_string(),
                s.symbol_count,
            )
        })
        .collect()
}

fn symbol_names(symbols: &[SymbolId]) -> Vec<String> {
    let mut names: Vec<String> = symbols.iter().map(|id| id.0.clone()).collect();
    names.sort();
    names
}

fn symbol_name_set(sources: &[CollateralSource]) -> HashSet<String> {
    sources
        .iter()
        .flat_map(|s| s.symbols.iter().map(|id| id.0.clone()))
        .collect()
}

#[test]
fn test_semantic_sources_grouped_by_defining_file_sorted() {
    let (ig, model) = setup_justification_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let metric = analyzer
        .analyze_file_load(Path::new("Thing.h"))
        .expect("analyze_file_load failed");

    // (symbol_count desc, path asc): Helper.h 3, then Base.h 1 before Thing.h 1.
    assert_eq!(
        source_names(&metric.semantic_sources),
        vec![
            ("Helper.h".to_string(), 3),
            ("Base.h".to_string(), 1),
            ("Thing.h".to_string(), 1),
        ]
    );
    assert_eq!(
        symbol_names(&metric.semantic_sources[0].symbols),
        vec!["HelperFn", "HelperFn2", "HelperUtil"]
    );

    // Each justified symbol appears exactly once across all groups.
    let mut flat: Vec<String> = metric
        .semantic_sources
        .iter()
        .flat_map(|s| s.symbols.iter().map(|id| id.0.clone()))
        .collect();
    let total = flat.len();
    flat.sort();
    flat.dedup();
    assert_eq!(total, 5);
    assert_eq!(flat.len(), 5);
    assert_eq!(
        HashSet::from_iter(flat),
        ["Thing", "HelperFn", "HelperFn2", "HelperUtil", "BaseStruct"]
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<String>>()
    );
}

#[test]
fn test_partition_invariant_i1_semantic_plus_collateral() {
    let (ig, model) = setup_justification_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let metric = analyzer
        .analyze_file_load(Path::new("Thing.h"))
        .expect("analyze_file_load failed");

    let sem_sum: usize = metric.semantic_sources.iter().map(|s| s.symbol_count).sum();
    let col_sum: usize = metric
        .collateral_sources
        .iter()
        .map(|s| s.symbol_count)
        .sum();
    assert_eq!(sem_sum + col_sum, metric.include_load);

    let sem_set = symbol_name_set(&metric.semantic_sources);
    let col_set = symbol_name_set(&metric.collateral_sources);
    let physical: HashSet<String> = [
        "Thing",
        "HelperFn",
        "HelperFn2",
        "HelperUtil",
        "BaseStruct",
        "HugeThing",
        "HugeOther",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let union: HashSet<String> = sem_set.union(&col_set).cloned().collect();
    assert_eq!(union, physical);
    assert!(sem_set.is_disjoint(&col_set));

    assert_eq!(
        sem_set,
        metric
            .semantic_symbols
            .iter()
            .map(|id| id.0.clone())
            .collect::<HashSet<String>>()
    );
    assert_eq!(
        col_set,
        metric
            .collateral_symbols
            .iter()
            .map(|id| id.0.clone())
            .collect::<HashSet<String>>()
    );
}

#[test]
fn test_semantic_sources_populated_for_symbol_load() {
    let (ig, model) = setup_justification_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let metric = analyzer
        .analyze_symbol_load("Thing")
        .expect("analyze_symbol_load failed");

    assert_eq!(
        source_names(&metric.semantic_sources),
        vec![
            ("Helper.h".to_string(), 3),
            ("Base.h".to_string(), 1),
            ("Thing.h".to_string(), 1),
        ]
    );
    let sem_sum: usize = metric.semantic_sources.iter().map(|s| s.symbol_count).sum();
    let col_sum: usize = metric
        .collateral_sources
        .iter()
        .map(|s| s.symbol_count)
        .sum();
    assert_eq!(sem_sum + col_sum, metric.include_load);
}

#[test]
fn test_semantic_sources_deterministic_across_runs() {
    let (ig, model) = setup_justification_fixture();

    let first = LoadAnalyzer::new(&ig, &model)
        .analyze_file_load(Path::new("Thing.h"))
        .expect("first run failed");
    let second = LoadAnalyzer::new(&ig, &model)
        .analyze_file_load(Path::new("Thing.h"))
        .expect("second run failed");

    assert_eq!(first.semantic_sources, second.semantic_sources);
}

#[test]
fn test_old_load_metric_json_without_semantic_sources_deserializes() {
    // I2 back-compat: artifacts written before semantic_sources existed must
    // still deserialize, with the new field defaulting to empty.
    let json = r#"{
        "target_name": "Thing",
        "target_file": "Thing.h",
        "semantic_load": 1,
        "include_load": 2,
        "collateral_load": 1,
        "total_load": 2,
        "semantic_symbols": ["Thing"],
        "collateral_symbols": ["HugeThing"],
        "collateral_sources": [
            {"file_id": 3, "path": "HugeThirdParty.h", "symbol_count": 1, "symbols": ["HugeThing"]}
        ],
        "largest_collateral_source": {
            "file_id": 3, "path": "HugeThirdParty.h", "symbol_count": 1, "symbols": ["HugeThing"]
        }
    }"#;

    let metric: LoadMetric = serde_json::from_str(json).expect("old JSON must deserialize");
    assert_eq!(metric.target_name, "Thing");
    assert!(metric.semantic_sources.is_empty());
    assert_eq!(metric.collateral_sources.len(), 1);
    assert_eq!(metric.collateral_sources[0].file_id, FileId(3));
    assert_eq!(
        metric.collateral_sources[0].path,
        PathBuf::from("HugeThirdParty.h")
    );
}

// ---- required_headers_for_symbols (projection API) ----

// Seeds S1, S2 and the isolated Lonely live in seed.h; S1 requires X1, X2
// (a.h) and X1 in turn requires Y1; S2 requires Y1 (b.h). The seeds' own
// defining file must never be reported as a required header.
fn setup_required_fixture() -> (IncludeGraph, ReducedSemanticModel) {
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(1), PathBuf::from("seed.h"));
    ig.register_file(FileId(2), PathBuf::from("a.h"));
    ig.register_file(FileId(3), PathBuf::from("b.h"));

    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };

    let add_sym = |model: &mut ReducedSemanticModel, name: &str, file: FileId| {
        let id = SymbolId::new(name);
        model.symbols.insert(
            id.clone(),
            ReducedSymbolRecord {
                id: id.clone(),
                name: name.to_string(),
                kind: SymbolKind::Function,
                defining_file: Some(file),
                declarations: vec![],
                definition: None,
                declaring_files: vec![],
            },
        );
        id
    };

    let s1 = add_sym(&mut model, "S1", FileId(1));
    let s2 = add_sym(&mut model, "S2", FileId(1));
    add_sym(&mut model, "Lonely", FileId(1));
    let x1 = add_sym(&mut model, "X1", FileId(2));
    let x2 = add_sym(&mut model, "X2", FileId(2));
    let y1 = add_sym(&mut model, "Y1", FileId(3));

    let dep = |src: &SymbolId, dst: &SymbolId| ReducedSemanticEdge {
        src_symbol: src.clone(),
        dst_symbol: dst.clone(),
        kind: DependencyKind::Complete,
    };
    model.edges = vec![dep(&s1, &x1), dep(&s1, &x2), dep(&x1, &y1), dep(&s2, &y1)];

    (ig, model)
}

// z.h is registered before b.h so graph insertion order contradicts the
// `(count desc, path asc)` contract; both contribute exactly one symbol.
fn setup_tiebreak_fixture() -> (IncludeGraph, ReducedSemanticModel) {
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(1), PathBuf::from("main.h"));
    ig.register_file(FileId(2), PathBuf::from("z.h"));
    ig.register_file(FileId(3), PathBuf::from("b.h"));

    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };

    let add_sym = |model: &mut ReducedSemanticModel, name: &str, file: FileId| {
        let id = SymbolId::new(name);
        model.symbols.insert(
            id.clone(),
            ReducedSymbolRecord {
                id: id.clone(),
                name: name.to_string(),
                kind: SymbolKind::Function,
                defining_file: Some(file),
                declarations: vec![],
                definition: None,
                declaring_files: vec![],
            },
        );
        id
    };

    let m = add_sym(&mut model, "M", FileId(1));
    let z = add_sym(&mut model, "Z", FileId(2));
    let b = add_sym(&mut model, "B", FileId(3));

    let dep = |src: &SymbolId, dst: &SymbolId| ReducedSemanticEdge {
        src_symbol: src.clone(),
        dst_symbol: dst.clone(),
        kind: DependencyKind::Complete,
    };
    model.edges = vec![dep(&m, &z), dep(&m, &b)];

    (ig, model)
}

fn header_rows(rows: &[(FileId, PathBuf, usize)]) -> Vec<(String, usize)> {
    rows.iter()
        .map(|(_, path, count)| {
            (
                path.file_name().unwrap().to_string_lossy().to_string(),
                *count,
            )
        })
        .collect()
}

#[test]
fn test_required_headers_combines_two_seed_closures_excluding_seed_file() {
    let (ig, model) = setup_required_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let rows = analyzer.required_headers_for_symbols(&[SymbolId::new("S1"), SymbolId::new("S2")]);

    assert_eq!(
        header_rows(&rows),
        vec![("a.h".to_string(), 2), ("b.h".to_string(), 1)]
    );
    assert_eq!(rows[0].0, FileId(2));
    assert_eq!(rows[1].0, FileId(3));
    assert!(rows.iter().all(|(id, _, _)| *id != FileId(1)));
}

#[test]
fn test_required_headers_empty_input_returns_empty() {
    let (ig, model) = setup_required_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);
    assert!(analyzer.required_headers_for_symbols(&[]).is_empty());
}

#[test]
fn test_required_headers_isolated_seed_returns_empty() {
    let (ig, model) = setup_required_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);
    let rows = analyzer.required_headers_for_symbols(&[SymbolId::new("Lonely")]);
    assert!(rows.is_empty());
}

#[test]
fn test_required_headers_unknown_symbols_ignored() {
    let (ig, model) = setup_required_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    assert!(analyzer
        .required_headers_for_symbols(&[
            SymbolId::new("NoSuchSymbol"),
            SymbolId::new("AlsoMissing")
        ])
        .is_empty());

    let known_only = analyzer.required_headers_for_symbols(&[SymbolId::new("S1")]);
    assert_eq!(
        header_rows(&known_only),
        vec![("a.h".to_string(), 2), ("b.h".to_string(), 1)]
    );
    let mixed = analyzer
        .required_headers_for_symbols(&[SymbolId::new("NoSuchSymbol"), SymbolId::new("S1")]);
    assert_eq!(mixed, known_only);
}

#[test]
fn test_required_headers_deterministic_and_path_tiebreak() {
    let (ig, model) = setup_required_fixture();
    let seeds = [SymbolId::new("S1"), SymbolId::new("S2")];
    let first = LoadAnalyzer::new(&ig, &model).required_headers_for_symbols(&seeds);
    let second = LoadAnalyzer::new(&ig, &model).required_headers_for_symbols(&seeds);
    assert_eq!(first, second);

    let (ig2, model2) = setup_tiebreak_fixture();
    let rows = LoadAnalyzer::new(&ig2, &model2).required_headers_for_symbols(&[SymbolId::new("M")]);
    assert_eq!(
        header_rows(&rows),
        vec![("b.h".to_string(), 1), ("z.h".to_string(), 1)]
    );
}

#[test]
fn test_required_headers_seed_defining_file_dropped_wholesale() {
    let (ig, model) = setup_required_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let baseline = analyzer.required_headers_for_symbols(&[SymbolId::new("S1")]);
    assert_eq!(
        header_rows(&baseline),
        vec![("a.h".to_string(), 2), ("b.h".to_string(), 1)]
    );

    // Adding X1 (defined in a.h) as a seed drops a.h wholesale, even though
    // the non-seed closure symbol X2 is still defined there: exclusion is
    // per seed defining-file, not per symbol.
    let with_seed =
        analyzer.required_headers_for_symbols(&[SymbolId::new("S1"), SymbolId::new("X1")]);
    assert_eq!(header_rows(&with_seed), vec![("b.h".to_string(), 1)]);
    assert!(with_seed.iter().all(|(id, _, _)| *id != FileId(2)));
}

// ---- E-I9: needs-sets contain includable headers only ----

// Intent (E-I9, plan 1790521470000): a semantic closure symbol defined in a
// translation unit (.cpp/.cc) contributes NO include requirement — it is
// satisfied by a declaration elsewhere or by linking, never by inclusion.
// S1@seed.h reaches worker.cpp AND a header-only path through b.h;
// S2@a.h is a seed whose closure re-reaches a.h plus b.h and worker.cpp;
// S3@seed.h reaches ONLY source defining files.
fn setup_ei9_fixture() -> (IncludeGraph, ReducedSemanticModel) {
    let mut ig = IncludeGraph::new();
    ig.register_file(FileId(1), PathBuf::from("seed.h"));
    ig.register_file(FileId(2), PathBuf::from("a.h"));
    ig.register_file(FileId(3), PathBuf::from("b.h"));
    ig.register_file(FileId(4), PathBuf::from("worker.cpp"));
    ig.register_file(FileId(5), PathBuf::from("impl.cc"));

    let mut model = ReducedSemanticModel {
        repo_root: PathBuf::from("/"),
        symbols: BTreeMap::new(),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
    };

    let add_sym = |model: &mut ReducedSemanticModel, name: &str, file: FileId| {
        let id = SymbolId::new(name);
        model.symbols.insert(
            id.clone(),
            ReducedSymbolRecord {
                id: id.clone(),
                name: name.to_string(),
                kind: SymbolKind::Function,
                defining_file: Some(file),
                declarations: vec![],
                definition: None,
                declaring_files: vec![],
            },
        );
        id
    };

    let s1 = add_sym(&mut model, "S1", FileId(1));
    let s3 = add_sym(&mut model, "S3", FileId(1));
    let s2 = add_sym(&mut model, "S2", FileId(2));
    let a_loc = add_sym(&mut model, "ALoc", FileId(2));
    let y = add_sym(&mut model, "Y", FileId(3));
    let z = add_sym(&mut model, "Z", FileId(3));
    let w = add_sym(&mut model, "W", FileId(4));
    let v = add_sym(&mut model, "V", FileId(5));

    let dep = |src: &SymbolId, dst: &SymbolId| ReducedSemanticEdge {
        src_symbol: src.clone(),
        dst_symbol: dst.clone(),
        kind: DependencyKind::Complete,
    };
    model.edges = vec![
        dep(&s1, &w),
        dep(&s1, &y),
        dep(&y, &z),
        dep(&s2, &a_loc),
        dep(&s2, &y),
        dep(&s2, &w),
        dep(&s3, &w),
        dep(&s3, &v),
    ];

    (ig, model)
}

#[test]
fn test_required_headers_mixed_header_and_source_closure_returns_header_only() {
    let (ig, model) = setup_ei9_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let rows = analyzer.required_headers_for_symbols(&[SymbolId::new("S1")]);

    // W is defined in worker.cpp: includable via nothing, so only b.h's row
    // (Y + Z = 2 symbols) may appear.
    assert_eq!(header_rows(&rows), vec![("b.h".to_string(), 2)]);
    assert!(
        rows.iter().all(|(id, _, _)| *id != FileId(4)),
        "worker.cpp must be absent from needs per E-I9: {:?}",
        header_rows(&rows)
    );
}

#[test]
fn test_required_headers_source_only_closure_returns_empty() {
    let (ig, model) = setup_ei9_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    let rows = analyzer.required_headers_for_symbols(&[SymbolId::new("S3")]);

    assert!(
        rows.is_empty(),
        "a closure reaching only .cpp/.cc defining files needs no includes per E-I9: {:?}",
        header_rows(&rows)
    );
}

#[test]
fn test_required_headers_seed_exclusion_still_applies_alongside_header_filter() {
    let (ig, model) = setup_ei9_fixture();
    let analyzer = LoadAnalyzer::new(&ig, &model);

    // Seed S2 is defined in a.h; ALoc (non-seed) sits in a.h too (dropped by
    // the wholesale seed-file exclusion), Y/Z land in b.h, W in worker.cpp.
    let rows = analyzer.required_headers_for_symbols(&[SymbolId::new("S2")]);

    assert_eq!(header_rows(&rows), vec![("b.h".to_string(), 2)]);
    assert!(rows.iter().all(|(id, _, _)| *id != FileId(2)));
    assert!(rows.iter().all(|(id, _, _)| *id != FileId(4)));
}

// The `is_includable_header` decision-table test moved to
// `ariadne-repo::types` alongside the relocated predicate (plan §6.3 R2:
// header-suffix taxonomy lives in the repo taxonomy crate).
