//! M28 T1 capacity-invariance oracle (I5) and the deterministic CI cap
//! clause (I7c) for the load memo's two append-only caches.
//!
//! The milestone's swap oracle: every `analyze_*` result is `==`-identical
//! across memo capacities {0, 1, hop boot caps, usize::MAX} on every
//! fixture here - eviction may trade time, never bytes. The capacity-0 vs
//! capacity-MAX equality is self-referential (no pre-swap snapshot) and
//! falsifies exactly the axis the bounded-capacity fix moves. Authored
//! RED against the uncapped append-only behavior (journal
//! `tasks/stage5-collateral-memory-ceiling-t1-journal.md`) and kept
//! forever as a forever-lock file: later hops re-run it UNEDITED.
//!
//! The counter-consuming tests are gated on `feature = "test-instrument"`
//! because the memo's test-only counters compile only under
//! `cfg(any(test, feature = "test-instrument"))` and `Cargo.toml` is
//! frozen for this hop (journal deviation D3): run the scoped suite with
//! `--features test-instrument` to execute them; a workspace-wide test
//! build enables the feature via the ui dev-dependency.
//!
//! The cap literals asserted here are HOP-FIXED (`HOP_CAP`/pin, never
//! review-ratified - landing reviewer adjudicates): see
//! `LoadMemo::{SEMANTIC_CLOSURE_BOOT_CAP, PHYSICAL_EXPANSION_BOOT_CAP,
//! I7C_CACHE_LEN_PIN}` and the journal justification citing Block 4 of
//! `tasks/stage5-collateral-memory-ceiling-t0a-evidence.md`.

use ariadne_graph::{
    IncludeEdge, IncludeGraph, IncludeKind, LanguageId, LoadAnalyzer, LoadMemo, LoadMetric,
    ReducedSemanticEdge, ReducedSemanticModel, ReducedSymbolRecord, SourceLocation,
};
use ariadne_index::types::{DependencyKind, SymbolId, SymbolKind};
use ariadne_repo::FileId;
use std::path::PathBuf;
use std::sync::Arc;

/// One locked fixture: an include graph plus its reduced semantic model,
/// replicated in-code from the crate's existing locked fixture shapes
/// (zero fixture demand; journal "Fixture models").
struct Fixture {
    name: &'static str,
    graph: IncludeGraph,
    model: ReducedSemanticModel,
}

fn empty_model() -> ReducedSemanticModel {
    let mut model = ReducedSemanticModel::default();
    model.repo_root = PathBuf::from("/");
    model
}

fn local_include(from: FileId, to: FileId, name: &str) -> IncludeEdge {
    IncludeEdge {
        source: from,
        destination: Some(to),
        context_name: String::new(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: name.to_string(),
        resolved_path: Some(PathBuf::from(name)),
        language: LanguageId::cpp(),
    }
}

fn add_symbol(model: &mut ReducedSemanticModel, name: &str, file: FileId) -> SymbolId {
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
}

fn semantic_dep(src: &SymbolId, dst: &SymbolId) -> ReducedSemanticEdge {
    ReducedSemanticEdge {
        src_symbol: src.clone(),
        dst_symbol: dst.clone(),
        kind: DependencyKind::Complete,
    }
}

/// Mirrors `setup_chain_fixture` (`crates/graph/src/load_perf_tests.rs:13`,
/// same shape as `load_justification_tests.rs:12`): Thing.h semantically
/// needs Helper.h and Base.h symbols, HugeThirdParty.h stays collateral;
/// Helper.h owns several symbols so file-level analysis takes multi-seed
/// input whose members warm individually first.
fn chain_fixture() -> Fixture {
    let mut graph = IncludeGraph::new();
    graph.register_file(FileId(1), PathBuf::from("Thing.h"));
    graph.register_file(FileId(2), PathBuf::from("Helper.h"));
    graph.register_file(FileId(3), PathBuf::from("HugeThirdParty.h"));
    graph.register_file(FileId(4), PathBuf::from("Base.h"));
    graph.add_edge(local_include(FileId(1), FileId(2), "Helper.h"));
    graph.add_edge(local_include(FileId(1), FileId(4), "Base.h"));
    graph.add_edge(local_include(FileId(2), FileId(3), "HugeThirdParty.h"));

    let mut model = empty_model();
    let thing = add_symbol(&mut model, "Thing", FileId(1));
    let helper_fn = add_symbol(&mut model, "HelperFn", FileId(2));
    let helper_fn2 = add_symbol(&mut model, "HelperFn2", FileId(2));
    let helper_util = add_symbol(&mut model, "HelperUtil", FileId(2));
    let base_struct = add_symbol(&mut model, "BaseStruct", FileId(4));
    add_symbol(&mut model, "HugeThing", FileId(3));
    add_symbol(&mut model, "HugeOther", FileId(3));

    model.file_symbols.insert(FileId(1), vec![thing.clone()]);
    model.file_symbols.insert(
        FileId(2),
        vec![
            helper_fn.clone(),
            helper_fn2.clone(),
            helper_util.clone(),
        ],
    );
    model
        .file_symbols
        .insert(FileId(3), vec![SymbolId::new("HugeThing"), SymbolId::new("HugeOther")]);
    model.file_symbols.insert(FileId(4), vec![base_struct.clone()]);

    model.edges = vec![
        semantic_dep(&thing, &helper_fn),
        semantic_dep(&helper_fn, &helper_fn2),
        semantic_dep(&thing, &helper_util),
        semantic_dep(&thing, &base_struct),
    ];

    Fixture {
        name: "chain",
        graph,
        model,
    }
}

/// Mirrors `setup_seed_fixture` (`crates/graph/src/load_perf_tests.rs:104`):
/// seeds reach shared symbols across files and the model registers no
/// file_symbols at all, exercising the empty-file-metrics path.
fn seed_fixture() -> Fixture {
    let mut graph = IncludeGraph::new();
    graph.register_file(FileId(1), PathBuf::from("seed.h"));
    graph.register_file(FileId(2), PathBuf::from("a.h"));
    graph.register_file(FileId(3), PathBuf::from("b.h"));

    let mut model = empty_model();
    let s1 = add_symbol(&mut model, "S1", FileId(1));
    let s2 = add_symbol(&mut model, "S2", FileId(1));
    add_symbol(&mut model, "Lonely", FileId(1));
    let x1 = add_symbol(&mut model, "X1", FileId(2));
    let x2 = add_symbol(&mut model, "X2", FileId(2));
    let y1 = add_symbol(&mut model, "Y1", FileId(3));

    model.edges = vec![
        semantic_dep(&s1, &x1),
        semantic_dep(&s1, &x2),
        semantic_dep(&x1, &y1),
        semantic_dep(&s2, &y1),
    ];

    Fixture {
        name: "seed",
        graph,
        model,
    }
}

/// Mirrors `setup_concurrency_fixture`
/// (`crates/graph/src/load_perf_tests.rs:260`): the chain plus the alpha
/// and beta include sub-trees - multi-root include structure.
fn concurrency_fixture() -> Fixture {
    let mut graph = IncludeGraph::new();
    graph.register_file(FileId(1), PathBuf::from("Thing.h"));
    graph.register_file(FileId(2), PathBuf::from("Helper.h"));
    graph.register_file(FileId(3), PathBuf::from("HugeThirdParty.h"));
    graph.register_file(FileId(4), PathBuf::from("Base.h"));
    graph.register_file(FileId(5), PathBuf::from("alpha.h"));
    graph.register_file(FileId(6), PathBuf::from("alpha_helper.h"));
    graph.register_file(FileId(7), PathBuf::from("alpha_leaf.h"));
    graph.register_file(FileId(8), PathBuf::from("beta.h"));
    graph.register_file(FileId(9), PathBuf::from("beta_helper.h"));
    graph.add_edge(local_include(FileId(1), FileId(2), "Helper.h"));
    graph.add_edge(local_include(FileId(1), FileId(4), "Base.h"));
    graph.add_edge(local_include(FileId(2), FileId(3), "HugeThirdParty.h"));
    graph.add_edge(local_include(FileId(5), FileId(6), "alpha_helper.h"));
    graph.add_edge(local_include(FileId(6), FileId(7), "alpha_leaf.h"));
    graph.add_edge(local_include(FileId(8), FileId(9), "beta_helper.h"));

    let mut model = empty_model();
    let thing = add_symbol(&mut model, "Thing", FileId(1));
    let helper_fn = add_symbol(&mut model, "HelperFn", FileId(2));
    let helper_fn2 = add_symbol(&mut model, "HelperFn2", FileId(2));
    let helper_fn3 = add_symbol(&mut model, "HelperFn3", FileId(2));
    let helper_util = add_symbol(&mut model, "HelperUtil", FileId(2));
    let base_struct = add_symbol(&mut model, "BaseStruct", FileId(4));
    let base_util = add_symbol(&mut model, "BaseUtil", FileId(4));
    add_symbol(&mut model, "HugeThing", FileId(3));
    add_symbol(&mut model, "HugeOther", FileId(3));
    let alpha = add_symbol(&mut model, "Alpha", FileId(5));
    let alpha_helper = add_symbol(&mut model, "AlphaHelper", FileId(6));
    let alpha_leaf = add_symbol(&mut model, "AlphaLeaf", FileId(7));
    add_symbol(&mut model, "AlphaLeaf2", FileId(7));
    let beta = add_symbol(&mut model, "Beta", FileId(8));
    let beta_helper = add_symbol(&mut model, "BetaHelper", FileId(9));
    add_symbol(&mut model, "BetaStandalone", FileId(8));

    model.edges = vec![
        semantic_dep(&thing, &helper_fn),
        semantic_dep(&helper_fn, &helper_fn2),
        semantic_dep(&helper_fn2, &helper_fn3),
        semantic_dep(&thing, &helper_util),
        semantic_dep(&thing, &base_struct),
        semantic_dep(&base_struct, &base_util),
        semantic_dep(&alpha, &alpha_helper),
        semantic_dep(&alpha_helper, &alpha_leaf),
        semantic_dep(&beta, &beta_helper),
    ];

    Fixture {
        name: "concurrency",
        graph,
        model,
    }
}

/// Mirrors `setup_impact_fixture(&[], false)`
/// (`crates/graph/src/load_impact_tests.rs:73`): a `.cpp` TU root over a
/// hub fan-in (fat.h -> base/c/d/e) - TU roots and the
/// non-includable-defining-file filtering path.
fn impact_fixture() -> Fixture {
    let main = FileId(11);
    let fat = FileId(12);
    let base = FileId(13);
    let c_header = FileId(14);
    let d_header = FileId(15);
    let e_header = FileId(16);

    let mut graph = IncludeGraph::new();
    graph.register_file(main, PathBuf::from("main.cpp"));
    graph.register_file(fat, PathBuf::from("fat.h"));
    graph.register_file(base, PathBuf::from("base.h"));
    graph.register_file(c_header, PathBuf::from("c.h"));
    graph.register_file(d_header, PathBuf::from("d.h"));
    graph.register_file(e_header, PathBuf::from("e.h"));
    graph.add_edge(local_include(main, fat, "fat.h"));
    graph.add_edge(local_include(fat, base, "base.h"));
    graph.add_edge(local_include(fat, c_header, "c.h"));
    graph.add_edge(local_include(c_header, d_header, "d.h"));
    graph.add_edge(local_include(c_header, e_header, "e.h"));

    let mut model = empty_model();
    let fat_a = add_symbol(&mut model, "FatA", fat);
    let fat_b = add_symbol(&mut model, "FatB", fat);
    let base_t = add_symbol(&mut model, "BaseT", base);
    let c_symbol = add_symbol(&mut model, "Csym", c_header);
    let d_symbol = add_symbol(&mut model, "Dsym", d_header);
    let d_symbol2 = add_symbol(&mut model, "Dsym2", d_header);
    let e_symbol = add_symbol(&mut model, "Esym", e_header);

    model
        .file_symbols
        .insert(fat, vec![fat_a.clone(), fat_b.clone()]);
    model.file_symbols.insert(base, vec![base_t.clone()]);
    model.file_symbols.insert(c_header, vec![c_symbol]);
    model
        .file_symbols
        .insert(d_header, vec![d_symbol.clone(), d_symbol2]);
    model.file_symbols.insert(e_header, vec![e_symbol]);

    model.edges = vec![semantic_dep(&fat_a, &base_t), semantic_dep(&fat_b, &base_t)];

    Fixture {
        name: "impact",
        graph,
        model,
    }
}

/// Hub-heavy pipeline-scale sweep model (243 symbols, 41 files): every
/// leaf header includes `hub.h`, `main.cpp` includes everything - the
/// in-code analogue of the E1 hub shape at fixture scale, so semantic
/// closures are corpus-shaped relative to the fixture and the sweep
/// touches every cache path the stage-5 passes do.
fn hub_scale_fixture() -> Fixture {
    const LEAF_HEADER_COUNT: u64 = 39;
    const HUB_SYMBOL_COUNT: u64 = 8;
    const LEAF_SYMBOL_COUNT: u64 = 6;
    let hub = FileId(100);
    let main = FileId(0);

    let mut graph = IncludeGraph::new();
    graph.register_file(main, PathBuf::from("main.cpp"));
    graph.register_file(hub, PathBuf::from("hub.h"));
    for leaf_index in 1..=LEAF_HEADER_COUNT {
        graph.register_file(
            FileId(leaf_index),
            PathBuf::from(format!("hdr_{leaf_index}.h")),
        );
    }
    graph.add_edge(local_include(main, hub, "hub.h"));
    for leaf_index in 1..=LEAF_HEADER_COUNT {
        graph.add_edge(local_include(
            main,
            FileId(leaf_index),
            &format!("hdr_{leaf_index}.h"),
        ));
        graph.add_edge(local_include(FileId(leaf_index), hub, "hub.h"));
    }

    let mut model = empty_model();
    let mut hub_symbols = Vec::new();
    for hub_index in 1..=HUB_SYMBOL_COUNT {
        hub_symbols.push(add_symbol(&mut model, &format!("Hub{hub_index}"), hub));
    }
    let mut leaf_symbols_by_file: Vec<(FileId, Vec<SymbolId>)> = Vec::new();
    for leaf_index in 1..=LEAF_HEADER_COUNT {
        let leaf_file = FileId(leaf_index);
        let mut leaf_symbols = Vec::new();
        for symbol_index in 1..=LEAF_SYMBOL_COUNT {
            leaf_symbols.push(add_symbol(
                &mut model,
                &format!("Hdr{leaf_index}_{symbol_index}"),
                leaf_file,
            ));
        }
        leaf_symbols_by_file.push((leaf_file, leaf_symbols));
    }
    let main_fn = add_symbol(&mut model, "MainFn", main);

    let mut edges = vec![
        semantic_dep(&hub_symbols[0], &hub_symbols[1]),
        semantic_dep(&hub_symbols[1], &hub_symbols[2]),
        semantic_dep(&main_fn, &hub_symbols[0]),
    ];
    for (leaf_position, leaf_symbols) in leaf_symbols_by_file.iter().map(|(_, symbols)| symbols).enumerate() {
        edges.push(semantic_dep(
            &leaf_symbols[0],
            &hub_symbols[leaf_position % HUB_SYMBOL_COUNT as usize],
        ));
        edges.push(semantic_dep(&leaf_symbols[1], &main_fn));
    }
    model.edges = edges;

    model.file_symbols.insert(hub, hub_symbols.clone());
    for (leaf_file, leaf_symbols) in leaf_symbols_by_file {
        model.file_symbols.insert(leaf_file, leaf_symbols);
    }
    model.file_symbols.insert(main, vec![main_fn]);

    Fixture {
        name: "hub_scale",
        graph,
        model,
    }
}

/// Wide demand model with 700 independent single-symbol files - demand
/// strictly above BOTH hop caps, the falsifier for the counter cap "at
/// all times" clause (journal deviation D4). No include or semantic edges:
/// the sweep is cheap and the demand arithmetic is exact.
fn adversarial_wide_fixture() -> Fixture {
    const WIDE_FILE_COUNT: u64 = 700;
    let mut graph = IncludeGraph::new();
    let mut model = empty_model();
    for wide_index in 1..=WIDE_FILE_COUNT {
        let wide_file = FileId(wide_index);
        graph.register_file(wide_file, PathBuf::from(format!("wide_{wide_index}.h")));
        let wide_symbol = add_symbol(&mut model, &format!("Wide{wide_index}"), wide_file);
        model.file_symbols.insert(wide_file, vec![wide_symbol]);
    }
    Fixture {
        name: "adversarial_wide",
        graph,
        model,
    }
}

fn all_fixtures() -> Vec<Fixture> {
    vec![
        chain_fixture(),
        seed_fixture(),
        concurrency_fixture(),
        impact_fixture(),
        hub_scale_fixture(),
        adversarial_wide_fixture(),
    ]
}

/// Full pipeline-scale query sweep in stage-5 order: the symbol pass
/// (every symbol metric over `model.symbols.values()`, the exact ordering
/// of the pipeline report-equality oracle) then the file pass (every
/// `model.file_symbols.keys()` id). Deterministic by construction:
/// BTreeMap iteration order only.
fn pipeline_scale_sweep(analyzer: &LoadAnalyzer) -> Vec<LoadMetric> {
    let mut metrics = Vec::new();
    for symbol_record in analyzer.model.symbols.values() {
        metrics.push(
            analyzer
                .analyze_symbol_load(&symbol_record.name)
                .expect("registered symbol with a defining file analyzes"),
        );
    }
    for file_id in analyzer.model.file_symbols.keys() {
        metrics.push(
            analyzer
                .analyze_file_load_by_id(*file_id)
                .expect("registered file id analyzes"),
        );
    }
    metrics
}

/// Analyzer over a capped memo with the include-closure table pre-seeded
/// through the additive constructor (the boot-pass pattern).
fn analyzer_with_caps<'f>(
    fixture: &'f Fixture,
    semantic_cap: usize,
    physical_cap: usize,
) -> LoadAnalyzer<'f> {
    let memo = Arc::new(LoadMemo::with_closures_and_capacity(
        fixture.graph.compute_closures(),
        semantic_cap,
        physical_cap,
    ));
    LoadAnalyzer::with_memo(&fixture.graph, &fixture.model, memo)
}

/// Analyzer over today's uncapped API surface (`LoadAnalyzer::new`) - the
/// pre-M28 behavior the oracle pins every capacity to.
fn uncapped_reference_analyzer(fixture: &Fixture) -> LoadAnalyzer<'_> {
    LoadAnalyzer::new(&fixture.graph, &fixture.model)
}

/// The I5 capacity axis: both caches at the same axis point, with the
/// hop caps as their per-cache literals.
fn capacity_points() -> Vec<(&'static str, usize, usize)> {
    vec![
        ("cap-0", 0, 0),
        ("cap-1", 1, 1),
        (
            "hop-cap",
            LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP,
            LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP,
        ),
        ("cap-uncapped", usize::MAX, usize::MAX),
    ]
}

/// I5 (the swap oracle): every capacity point - cold and warm sweep -
/// equals the uncapped reference on every fixture. Pairwise equality
/// follows transitively (all points equal the reference). Counter-free:
/// runs in every build.
#[test]
fn analyze_results_are_identical_across_every_memo_capacity() {
    for fixture in all_fixtures() {
        let reference_analyzer = uncapped_reference_analyzer(&fixture);
        let reference = pipeline_scale_sweep(&reference_analyzer);
        assert!(
            !reference.is_empty(),
            "fixture {} must produce metrics",
            fixture.name
        );
        for (point_label, semantic_cap, physical_cap) in capacity_points() {
            let analyzer = analyzer_with_caps(&fixture, semantic_cap, physical_cap);
            let cold_sweep = pipeline_scale_sweep(&analyzer);
            let warm_sweep = pipeline_scale_sweep(&analyzer);
            assert_eq!(
                reference, cold_sweep,
                "fixture {} @ {}: cold sweep must equal the uncapped reference",
                fixture.name, point_label
            );
            assert_eq!(
                reference, warm_sweep,
                "fixture {} @ {}: warm sweep (memo serving admits) must equal the uncapped reference",
                fixture.name, point_label
            );
        }
    }
}

/// I7c (CI deterministic clause): after a full pipeline-scale query run
/// at the hop caps, both test-instrument counters stay `<=` their cache's
/// hop-fixed cap and `<=` the hop-fixed I7c pin. Machine-independent, no
/// wall time - the only memory clause allowed in CI (DP-3).
#[cfg(feature = "test-instrument")]
#[test]
fn cache_counters_stay_within_hop_caps_and_ci_pin() {
    let fixture = hub_scale_fixture();
    let analyzer = analyzer_with_caps(
        &fixture,
        LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP,
        LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP,
    );
    pipeline_scale_sweep(&analyzer);
    pipeline_scale_sweep(&analyzer);
    let memo = analyzer.memo();
    assert!(
        memo.semantic_closure_cache_len() <= LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP,
        "I7c: semantic_closure_cache_len {} exceeded the hop cap {}",
        memo.semantic_closure_cache_len(),
        LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP
    );
    assert!(
        memo.physical_expansion_cache_len() <= LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP,
        "I7c: physical_expansion_cache_len {} exceeded the hop cap {}",
        memo.physical_expansion_cache_len(),
        LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP
    );
    assert!(
        memo.semantic_closure_cache_len() <= LoadMemo::I7C_CACHE_LEN_PIN,
        "I7c pin: semantic_closure_cache_len {} exceeded the hop-fixed pin {}",
        memo.semantic_closure_cache_len(),
        LoadMemo::I7C_CACHE_LEN_PIN
    );
    assert!(
        memo.physical_expansion_cache_len() <= LoadMemo::I7C_CACHE_LEN_PIN,
        "I7c pin: physical_expansion_cache_len {} exceeded the hop-fixed pin {}",
        memo.physical_expansion_cache_len(),
        LoadMemo::I7C_CACHE_LEN_PIN
    );
}

/// I7c "at all times" falsifier (journal deviation D4): with demand
/// strictly above BOTH caps (700 seeds / 700 def-files), the counters
/// must sit exactly at the caps - they cannot grow past them - while the
/// served results stay equal to the uncapped reference (the cap axis
/// inside the I5 claim).
#[cfg(feature = "test-instrument")]
#[test]
fn counters_stay_capped_when_demand_exceeds_every_cap() {
    let fixture = adversarial_wide_fixture();
    assert!(adversarial_wide_demand() > LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP);
    assert!(adversarial_wide_demand() > LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP);

    let reference_analyzer = uncapped_reference_analyzer(&fixture);
    let reference = pipeline_scale_sweep(&reference_analyzer);

    let analyzer = analyzer_with_caps(
        &fixture,
        LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP,
        LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP,
    );
    let capped_sweep = pipeline_scale_sweep(&analyzer);
    let memo = analyzer.memo();
    assert_eq!(
        reference, capped_sweep,
        "over-demand capped memo must serve results identical to the uncapped reference"
    );
    assert_eq!(
        memo.semantic_closure_cache_len(),
        LoadMemo::SEMANTIC_CLOSURE_BOOT_CAP,
        "demand {} exceeds the cap: the semantic-closure counter must sit exactly AT the cap, never above",
        adversarial_wide_demand()
    );
    assert_eq!(
        memo.physical_expansion_cache_len(),
        LoadMemo::PHYSICAL_EXPANSION_BOOT_CAP,
        "demand {} exceeds the cap: the physical-expansion counter must sit exactly AT the cap, never above",
        adversarial_wide_demand()
    );
}

/// Demand of the adversarial fixture (distinct seeds == distinct
/// def-files == file count).
fn adversarial_wide_demand() -> usize {
    let fixture = adversarial_wide_fixture();
    fixture.model.symbols.len()
}

/// Cap-0 ablation contract (T0b reuses this mode): the zero-cap memo
/// answers identically AND stays empty - every query is real work. The
/// physical-expansion tally must grow on a repeated sweep by exactly the
/// number of expansion-serving queries (memo off = no query is served
/// from memory). RED witness against the uncapped behavior: the empty
/// counter assertions name the cap-0 case.
#[cfg(feature = "test-instrument")]
#[test]
fn cap_zero_memo_is_pure_recompute_ablation() {
    for fixture in [chain_fixture(), hub_scale_fixture()] {
        let analyzer = analyzer_with_caps(&fixture, 0, 0);
        let reference_analyzer = uncapped_reference_analyzer(&fixture);

        let first_sweep = pipeline_scale_sweep(&analyzer);
        let expansions_after_first = analyzer.memo().physical_expansion_count();
        let repeat_sweep = pipeline_scale_sweep(&analyzer);
        let expansions_after_repeat = analyzer.memo().physical_expansion_count();

        assert_eq!(
            first_sweep, repeat_sweep,
            "cap 0: repeated queries must serve identical answers for fixture {}",
            fixture.name
        );
        assert_eq!(
            pipeline_scale_sweep(&reference_analyzer),
            first_sweep,
            "cap 0: answers must equal the uncapped reference for fixture {}",
            fixture.name
        );
        assert_eq!(
            analyzer.memo().semantic_closure_cache_len(),
            0,
            "cap 0 (fixture {}): semantic-closure memo must stay EMPTY - a non-zero \
             length means memoization is still on at cap 0",
            fixture.name
        );
        assert_eq!(
            analyzer.memo().physical_expansion_cache_len(),
            0,
            "cap 0 (fixture {}): physical-expansion memo must stay EMPTY - a non-zero \
             length means memoization is still on at cap 0",
            fixture.name
        );

        // Every sweep query is a real single-def-file expansion: one per
        // symbol metric plus one per file metric.
        let expansions_per_sweep =
            fixture.model.symbols.len() + fixture.model.file_symbols.len();
        assert!(expansions_per_sweep > 0);
        assert_eq!(
            expansions_after_first, expansions_per_sweep,
            "cap 0 (fixture {}): first sweep must run one real expansion per query",
            fixture.name
        );
        assert_eq!(
            expansions_after_repeat - expansions_after_first,
            expansions_per_sweep,
            "cap 0 (fixture {}): repeated sweep must run REAL expansions again (memo off = every query is a real expansion)",
            fixture.name
        );
        assert!(
            analyzer.memo().semantic_bfs_traversal_count() > 0,
            "cap 0 (fixture {}): a recompute-only memo must run real BFS traversals",
            fixture.name
        );
    }
}
