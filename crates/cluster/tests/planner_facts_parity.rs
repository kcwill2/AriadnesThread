//! A1 semantics lock: `OptimizationPlanner::plan()` fed with precomputed
//! screening facts + stage closures (the UI path) must produce a
//! byte-identical `Vec<OptimizationPlan>` to the cold `new()` path (the CLI
//! `tool optimize`/`plan` path), including order and `PLAN-001..` ids.
//!
//! The fixture is a synthetic include graph built through the public
//! graph/model APIs (same construction style as the in-crate planner
//! characterization fixture, scaled to a fanout of 48 headers + TUs + the
//! screening edge classes: no-symbol umbrella, zero-collateral, phantom-only
//! symbol lists, non-cyclic single-file SCCs, and a frozen-policy header).

use ariadne_cluster::plan::OptimizationPlan;
use ariadne_cluster::planner::{
    screening_facts_from_metrics, HeaderScreeningFacts, OptimizationPlanner,
};
use ariadne_graph::reduced::{ReducedSemanticEdge, ReducedSymbolRecord};
use ariadne_graph::types::{IncludeEdge, IncludeKind, LanguageId, SourceLocation};
use ariadne_graph::{
    IncludeClosure, IncludeGraph, LoadAnalyzer, LoadMetric, ReducedSemanticModel,
};
use ariadne_index::types::{DependencyKind, SymbolId, SymbolKind};
use ariadne_lang::CyclePolicy;
use ariadne_repo::{
    FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage,
};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

const HEADER_COUNT: usize = 48;
const FANOUT_STEP_A: usize = 11;
const FANOUT_STEP_B: usize = 23;
const HEADER_ID_BASE: u64 = 100;
const TU_ID_BASE: u64 = 200;

fn header_id(index: usize) -> FileId {
    FileId(HEADER_ID_BASE as u64 + index as u64)
}

fn header_name(index: usize) -> String {
    format!("h{index:02}.h")
}

struct ParityFixture {
    graph: IncludeGraph,
    symbols: BTreeMap<SymbolId, ReducedSymbolRecord>,
    edges: Vec<ReducedSemanticEdge>,
    file_symbols: BTreeMap<FileId, Vec<SymbolId>>,
    records: Vec<FileRecord>,
    tu_file_ids: Vec<FileId>,
}

impl ParityFixture {
    fn new() -> Self {
        Self {
            graph: IncludeGraph::new(),
            symbols: BTreeMap::new(),
            edges: Vec::new(),
            file_symbols: BTreeMap::new(),
            records: Vec::new(),
            tu_file_ids: Vec::new(),
        }
    }

    fn register(
        &mut self,
        id: FileId,
        name: &str,
        kind: FileKind,
        policy: OptimizationPolicy,
    ) {
        self.graph.register_file(id, PathBuf::from(name));
        self.records.push(FileRecord {
            id,
            path: PathBuf::from(name),
            canonical_path: PathBuf::from(name),
            kind,
            size: 0,
            content_hash: String::new(),
            ownership: FileOwnership::Repository,
            optimization_policy: policy,
            language: SourceLanguage::Cpp,
        });
    }

    fn includes(&mut self, source: FileId, dest: FileId) {
        let raw_path = self
            .graph
            .file_paths
            .get(&dest)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        self.graph.add_edge(IncludeEdge {
            source,
            destination: Some(dest),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: raw_path.clone(),
            resolved_path: Some(PathBuf::from(raw_path)),
            language: LanguageId::Cpp,
        });
    }

    /// Registers a symbol on the file; with `record_registered = false` the
    /// symbol lands in `file_symbols` without a model record (a phantom —
    /// `contributing_sources` skips it, so collateral it carries is
    /// unattributable, exactly like metrics whose source lacks a record).
    fn defines(&mut self, file: FileId, name: &str, record_registered: bool) {
        let sym = SymbolId::new(name);
        if record_registered {
            self.symbols.insert(
                sym.clone(),
                ReducedSymbolRecord {
                    id: sym.clone(),
                    name: name.to_string(),
                    kind: SymbolKind::Class,
                    defining_file: Some(file),
                    declarations: vec![],
                    definition: None,
                    declaring_files: Vec::new(),
                },
            );
        }
        self.file_symbols.entry(file).or_default().push(sym);
    }

    fn depends(&mut self, src: &str, dst: &str) {
        self.edges.push(ReducedSemanticEdge {
            src_symbol: SymbolId::new(src),
            dst_symbol: SymbolId::new(dst),
            kind: DependencyKind::Complete,
        });
    }

    /// Registers the seven screening-class headers and the three TUs.
    fn register_special_files(&mut self) {
        for special_id in 1u64..=7 {
            self.register(
                FileId(special_id),
                special_name(special_id),
                FileKind::Header,
                if special_id == 2 {
                    OptimizationPolicy::Frozen
                } else {
                    OptimizationPolicy::Optimizable
                },
            );
        }
        for slot in 0..3u64 {
            self.register(
                FileId(TU_ID_BASE + slot),
                &format!("tu{slot}.cc"),
                FileKind::Source,
                OptimizationPolicy::Optimizable,
            );
            self.tu_file_ids.push(FileId(TU_ID_BASE + slot));
        }
    }

    fn model(&self) -> ReducedSemanticModel {
        ReducedSemanticModel {
            repo_root: PathBuf::from("/"),
            symbols: self.symbols.clone(),
            edges: self.edges.clone(),
            tu_usage: BTreeMap::new(),
            file_symbols: self.file_symbols.clone(),
            extends_edges: Vec::new(),
            sealed_symbols: Vec::new(),
        }
    }
}

/// Builds the shared fixture. Symbol edges are strictly decreasing-index, so
/// every within-header SCC is a singleton and proposals survive validation;
/// include edges fan out cyclically so closures (and collateral) are dense.
///
/// Screening classes exercised:
/// - `zero.h` (FileId 1): symbols, no includes ⇒ collateral 0 on both paths.
/// - `frozen_guard.h` (2): collateral > 0 but policy Frozen ⇒ screened out
///   after the facts lookup (facts must never bypass the policy check).
/// - `orphan_umbrella.h` (3): no own symbols, includes real headers ⇒ cold
///   treats it as load-positive but it yields zero extractions; the facts
///   path never sees it at all (no metric ⇒ screened load 0).
/// - `lonely.h` (4): no symbols, no includes ⇒ load 0 on both paths.
/// - `cyclic_pair.h` (5): two mutually dependent symbols ⇒ single-file SCC
///   is rejected (QUIRK: whole header dropped, id number not burned).
/// - `ghost_collateral.h` (6): collateral is one phantom symbol (no model
///   record) ⇒ collateral_load 1 but largest_collateral_source None on both.
/// - `phantom.h` (7): file_symbols is only the phantom symbol ⇒ metric
///   exists with collateral 0 on both paths.
fn build_fixture() -> ParityFixture {
    let mut fixture = ParityFixture::new();

    for index in 0..HEADER_COUNT {
        fixture.register(
            header_id(index),
            &header_name(index),
            FileKind::Header,
            OptimizationPolicy::Optimizable,
        );
    }
    for index in 0..HEADER_COUNT {
        let target_a = (index + FANOUT_STEP_A) % HEADER_COUNT;
        let target_b = (index + FANOUT_STEP_B) % HEADER_COUNT;
        fixture.includes(header_id(index), header_id(target_a));
        fixture.includes(header_id(index), header_id(target_b));
    }
    fixture.register_special_files();

    // Real fanout headers: 3 symbols each, strictly-decreasing semantic edges
    // keep every SCC a singleton.
    for index in 0..HEADER_COUNT {
        let stem = format!("S{index:02}");
        fixture.defines(header_id(index), &format!("{stem}_0"), true);
        fixture.defines(header_id(index), &format!("{stem}_1"), true);
        fixture.defines(header_id(index), &format!("{stem}_2"), true);
        if index >= 2 {
            fixture.depends(&format!("S{index:02}_0"), &format!("S{:02}_0", index - 1));
        }
        if index >= 12 {
            fixture.depends(&format!("S{index:02}_1"), &format!("S{:02}_0", index - 12));
        }
        fixture.depends(&format!("S{index:02}_2"), &format!("S{index:02}_1"));
    }

    // zero.h: one symbol, no includes.
    fixture.defines(FileId(1), "Zero", true);
    // frozen_guard.h: one symbol, includes a real header (collateral > 0).
    fixture.defines(FileId(2), "Guard", true);
    fixture.includes(FileId(2), header_id(0));
    // orphan_umbrella.h / lonely.h: deliberately absent from file_symbols.
    fixture.includes(FileId(3), header_id(0));
    fixture.includes(FileId(3), FileId(7));
    // cyclic_pair.h: mutual cycle inside one file, plus collateral source.
    fixture.defines(FileId(5), "CycleA", true);
    fixture.defines(FileId(5), "CycleB", true);
    fixture.depends("CycleA", "CycleB");
    fixture.depends("CycleB", "CycleA");
    fixture.includes(FileId(5), header_id(1));
    // ghost_collateral.h: its only collateral is the phantom symbol.
    fixture.defines(FileId(6), "Ghost", true);
    fixture.includes(FileId(6), FileId(7));
    // phantom.h: symbol list contains a phantom with no model record.
    fixture.defines(FileId(7), "Phantom", false);

    // TU consumer edges: exercise validation_required_tus via closures.
    fixture.includes(FileId(TU_ID_BASE), header_id(0));
    fixture.includes(FileId(TU_ID_BASE), header_id(1));
    fixture.includes(FileId(TU_ID_BASE), header_id(2));
    fixture.includes(FileId(TU_ID_BASE + 1), header_id(3));
    fixture.includes(FileId(TU_ID_BASE + 1), header_id(4));
    fixture.includes(FileId(TU_ID_BASE + 2), header_id(HEADER_COUNT - 1));
    fixture.includes(FileId(TU_ID_BASE + 2), header_id(0));

    fixture
}

fn special_name(id: u64) -> &'static str {
    match id {
        1 => "zero.h",
        2 => "frozen_guard.h",
        3 => "orphan_umbrella.h",
        4 => "lonely.h",
        5 => "cyclic_pair.h",
        6 => "ghost_collateral.h",
        7 => "phantom.h",
        _ => unreachable!("fixture specials are FileId 1..=7"),
    }
}

/// Stage file-metrics exactly as `collect_file_metrics` builds them: one
/// by-id metric per `file_symbols` key that is registered in `file_paths`.
fn stage_file_metrics(
    graph: &IncludeGraph,
    model: &ReducedSemanticModel,
) -> Vec<LoadMetric> {
    let analyzer = LoadAnalyzer::new(graph, model);
    model
        .file_symbols
        .keys()
        .filter(|file_id| graph.file_paths.contains_key(file_id))
        .filter_map(|file_id| analyzer.analyze_file_load_by_id(*file_id).ok())
        .collect()
}

fn empty_metric(target_file: Option<PathBuf>, collateral_load: usize) -> LoadMetric {
    LoadMetric {
        target_name: target_file
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        target_file,
        semantic_load: 0,
        include_load: collateral_load,
        collateral_load,
        total_load: collateral_load,
        semantic_symbols: vec![],
        collateral_symbols: vec![],
        collateral_sources: vec![],
        largest_collateral_source: None,
        semantic_sources: vec![],
    }
}

fn plan_ids(plans: &[OptimizationPlan]) -> Vec<String> {
    plans.iter().map(|plan| plan.id.0.clone()).collect()
}

#[test]
fn plans_from_precomputed_facts_match_cold() {
    let fixture = build_fixture();
    let model = fixture.model();

    let cold_plans = OptimizationPlanner::new(
        &fixture.graph,
        &model,
        &fixture.records,
        &fixture.tu_file_ids,
        &(|path: &std::path::Path| path.extension().is_some_and(|ext| ext == "h")),
        &(|path: &std::path::Path, symbol: &str| {
            path.with_file_name(format!("{}_extracted.h", symbol))
        }),
        None,
        CyclePolicy::Reject,
    )
    .plan();

    let file_metrics = stage_file_metrics(&fixture.graph, &model);
    let closures = fixture.graph.compute_closures();
    let facts = screening_facts_from_metrics(&fixture.graph, &file_metrics);
    let facts_plans = OptimizationPlanner::with_screening_facts(
        &fixture.graph,
        &model,
        &fixture.records,
        &fixture.tu_file_ids,
        &(|path: &std::path::Path| path.extension().is_some_and(|ext| ext == "h")),
        &(|path: &std::path::Path, symbol: &str| {
            path.with_file_name(format!("{}_extracted.h", symbol))
        }),
        None,
        CyclePolicy::Reject,
        facts,
        &closures,
    )
    .plan();

    assert!(
        cold_plans.len() > 40,
        "fixture must produce many candidate plans, got {}",
        cold_plans.len()
    );
    assert_eq!(
        plan_ids(&facts_plans),
        (1..=cold_plans.len())
            .map(|n| format!("PLAN-{n:03}"))
            .collect::<Vec<String>>(),
        "facts-path plan ids must be dense and ordered"
    );
    assert_eq!(facts_plans, cold_plans);
}

#[test]
fn headers_without_symbols_are_non_candidates_on_both_paths() {
    let fixture = build_fixture();
    let model = fixture.model();

    let cold_plans = OptimizationPlanner::new(
        &fixture.graph,
        &model,
        &fixture.records,
        &fixture.tu_file_ids,
        &(|path: &std::path::Path| path.extension().is_some_and(|ext| ext == "h")),
        &(|path: &std::path::Path, symbol: &str| {
            path.with_file_name(format!("{}_extracted.h", symbol))
        }),
        None,
        CyclePolicy::Reject,
    )
    .plan();
    let file_metrics = stage_file_metrics(&fixture.graph, &model);
    let closures = fixture.graph.compute_closures();
    let facts = screening_facts_from_metrics(&fixture.graph, &file_metrics);

    for no_symbol_id in [FileId(3), FileId(4)] {
        assert!(fixture.graph.file_paths.contains_key(&no_symbol_id));
        assert!(!fixture.file_symbols.contains_key(&no_symbol_id));
        assert!(
            !facts.contains_key(&no_symbol_id),
            "no FileMetric must exist for symbol-free {no_symbol_id:?}"
        );
    }

    let facts_plans = OptimizationPlanner::with_screening_facts(
        &fixture.graph,
        &model,
        &fixture.records,
        &fixture.tu_file_ids,
        &(|path: &std::path::Path| path.extension().is_some_and(|ext| ext == "h")),
        &(|path: &std::path::Path, symbol: &str| {
            path.with_file_name(format!("{}_extracted.h", symbol))
        }),
        None,
        CyclePolicy::Reject,
        facts,
        &closures,
    )
    .plan();

    assert_eq!(facts_plans, cold_plans);
    for symbol_free in ["orphan_umbrella.h", "lonely.h"] {
        assert!(
            !cold_plans
                .iter()
                .chain(facts_plans.iter())
                .any(|plan| plan.target_file == PathBuf::from(symbol_free)),
            "{symbol_free} has no own symbols and must never produce a plan"
        );
    }
}

#[test]
fn unregistered_metric_targets_are_never_candidates() {
    let fixture = build_fixture();
    let model = fixture.model();

    let cold_plans = OptimizationPlanner::new(
        &fixture.graph,
        &model,
        &fixture.records,
        &fixture.tu_file_ids,
        &(|path: &std::path::Path| path.extension().is_some_and(|ext| ext == "h")),
        &(|path: &std::path::Path, symbol: &str| {
            path.with_file_name(format!("{}_extracted.h", symbol))
        }),
        None,
        CyclePolicy::Reject,
    )
    .plan();

    let mut file_metrics = stage_file_metrics(&fixture.graph, &model);
    let registered_count = file_metrics.len();
    file_metrics.push(empty_metric(
        Some(PathBuf::from("ghost_unregistered.h")),
        999,
    ));
    file_metrics.push(empty_metric(None, 999));

    let facts = screening_facts_from_metrics(&fixture.graph, &file_metrics);
    assert_eq!(
        facts.len(),
        registered_count,
        "metrics with unregistered or absent target paths must be skipped"
    );
    assert!(
        facts
            .keys()
            .all(|file_id| fixture.graph.file_paths.contains_key(file_id)),
        "every fact key must be a registered file"
    );

    let closures = fixture.graph.compute_closures();
    let facts_plans = OptimizationPlanner::with_screening_facts(
        &fixture.graph,
        &model,
        &fixture.records,
        &fixture.tu_file_ids,
        &(|path: &std::path::Path| path.extension().is_some_and(|ext| ext == "h")),
        &(|path: &std::path::Path, symbol: &str| {
            path.with_file_name(format!("{}_extracted.h", symbol))
        }),
        None,
        CyclePolicy::Reject,
        facts,
        &closures,
    )
    .plan();
    assert_eq!(facts_plans, cold_plans);

    // Even a hand-built fact keyed by an unregistered FileId cannot create a
    // candidate: screening only ever consults facts for graph-registered ids.
    let mut stray_facts: HashMap<FileId, HeaderScreeningFacts> = HashMap::new();
    stray_facts.insert(
        FileId(999_999),
        HeaderScreeningFacts {
            collateral_load: 4242,
            largest_collateral_source: Some(header_id(0)),
        },
    );
    let stray_plans = OptimizationPlanner::with_screening_facts(
        &fixture.graph,
        &model,
        &fixture.records,
        &fixture.tu_file_ids,
        &(|path: &std::path::Path| path.extension().is_some_and(|ext| ext == "h")),
        &(|path: &std::path::Path, symbol: &str| {
            path.with_file_name(format!("{}_extracted.h", symbol))
        }),
        None,
        CyclePolicy::Reject,
        stray_facts,
        &closures,
    )
    .plan();
    assert!(
        stray_plans.is_empty(),
        "a fact for an unregistered FileId must not produce any plan: {stray_plans:?}"
    );
}

// ---------------------------------------------------------------------------
// A2 locks: the rayon header loop (adjacency hoisted + order-preserving
// collect + collect-then-enumerate PLAN numbering) must be byte-identical to
// the pre-A2 serial loop, on the cold and the facts path alike.
// ---------------------------------------------------------------------------

/// > 64 header work items so the parallel driver must chunk and interleave
/// across threads; screening classes are interleaved across the index range.
const STRESS_HEADER_COUNT: usize = 84;
const STRESS_ZERO_COLLATERAL_EVERY: usize = 13;
const STRESS_FROZEN_EVERY: usize = 17;
const STRESS_CYCLIC_EVERY: usize = 11;
const STRESS_CYCLIC_REMAINDER: usize = 4;

/// Order-stress fixture: 84 headers with cyclic fanout includes (dense
/// closures ⇒ collateral > 0), three symbols per candidate header joined by
/// strictly decreasing semantic edges (singleton SCCs ⇒ multiple extraction
/// proposals per plan). Rejected classes are interleaved, never appended:
/// - `index % 13 == 5`: symbol-rich but include-free ⇒ collateral 0, screened
///   out before any SCC work.
/// - `index % 17 == 7`: collateral > 0 but Frozen policy ⇒ screened out at
///   the policy gate.
/// - `index % 11 == 4`: its two symbols form one whole-file SCC ⇒ screened
///   candidate that yields zero proposals ⇒ whole header dropped without
///   burning a plan number (QUIRK locked by the A1 characterization).
fn build_stress_fixture() -> ParityFixture {
    let mut fixture = ParityFixture::new();

    for index in 0..STRESS_HEADER_COUNT {
        fixture.register(
            header_id(index),
            &header_name(index),
            FileKind::Header,
            if index % STRESS_FROZEN_EVERY == 7 {
                OptimizationPolicy::Frozen
            } else {
                OptimizationPolicy::Optimizable
            },
        );
    }
    for index in 0..STRESS_HEADER_COUNT {
        if index % STRESS_ZERO_COLLATERAL_EVERY == 5 {
            continue; // include-free: zero-collateral reject
        }
        let target_a = (index + 11) % STRESS_HEADER_COUNT;
        let target_b = (index + 23) % STRESS_HEADER_COUNT;
        fixture.includes(header_id(index), header_id(target_a));
        fixture.includes(header_id(index), header_id(target_b));
    }

    for slot in 0..2u64 {
        fixture.register(
            FileId(TU_ID_BASE + slot),
            &format!("tu{slot}.cc"),
            FileKind::Source,
            OptimizationPolicy::Optimizable,
        );
        fixture.tu_file_ids.push(FileId(TU_ID_BASE + slot));
    }
    fixture.includes(FileId(TU_ID_BASE), header_id(0));
    fixture.includes(FileId(TU_ID_BASE), header_id(5));
    fixture.includes(FileId(TU_ID_BASE + 1), header_id(STRESS_HEADER_COUNT - 1));

    for index in 0..STRESS_HEADER_COUNT {
        let stem = format!("T{index:02}");
        if index % STRESS_CYCLIC_EVERY == STRESS_CYCLIC_REMAINDER {
            fixture.defines(header_id(index), &format!("{stem}_0"), true);
            fixture.defines(header_id(index), &format!("{stem}_1"), true);
            fixture.depends(&format!("{stem}_0"), &format!("{stem}_1"));
            fixture.depends(&format!("{stem}_1"), &format!("{stem}_0"));
        } else {
            fixture.defines(header_id(index), &format!("{stem}_0"), true);
            fixture.defines(header_id(index), &format!("{stem}_1"), true);
            fixture.defines(header_id(index), &format!("{stem}_2"), true);
            if index >= 1 {
                fixture.depends(&format!("T{index:02}_0"), &format!("T{:02}_0", index - 1));
            }
            fixture.depends(&format!("T{index:02}_2"), &format!("T{index:02}_1"));
        }
    }

    fixture
}

fn stress_planner<'a>(
    fixture: &'a ParityFixture,
    model: &'a ReducedSemanticModel,
) -> OptimizationPlanner<'a> {
    OptimizationPlanner::new(
        &fixture.graph,
        model,
        &fixture.records,
        &fixture.tu_file_ids,
        &(|path: &std::path::Path| path.extension().is_some_and(|ext| ext == "h")),
        &(|path: &std::path::Path, symbol: &str| {
            path.with_file_name(format!("{}_extracted.h", symbol))
        }),
        None,
        CyclePolicy::Reject,
    )
}

fn facts_planner<'a>(
    fixture: &'a ParityFixture,
    model: &'a ReducedSemanticModel,
    closures: &'a HashMap<FileId, IncludeClosure>,
) -> OptimizationPlanner<'a> {
    let file_metrics = stage_file_metrics(&fixture.graph, model);
    let facts = screening_facts_from_metrics(&fixture.graph, &file_metrics);
    OptimizationPlanner::with_screening_facts(
        &fixture.graph,
        model,
        &fixture.records,
        &fixture.tu_file_ids,
        &(|path: &std::path::Path| path.extension().is_some_and(|ext| ext == "h")),
        &(|path: &std::path::Path, symbol: &str| {
            path.with_file_name(format!("{}_extracted.h", symbol))
        }),
        None,
        CyclePolicy::Reject,
        facts,
        closures,
    )
}

#[test]
fn parallel_output_matches_serial() {
    // (a) the existing 48-header fixture, cold and facts paths.
    let fixture = build_fixture();
    let model = fixture.model();
    let closures = fixture.graph.compute_closures();
    assert_eq!(
        stress_planner(&fixture, &model).plan(),
        stress_planner(&fixture, &model).plan_serial_for_test(),
        "48-header cold path must match the serial driver"
    );
    assert_eq!(
        facts_planner(&fixture, &model, &closures).plan(),
        facts_planner(&fixture, &model, &closures).plan_serial_for_test(),
        "48-header facts path must match the serial driver"
    );

    // (b) the 84-header order-stress fixture.
    let stress = build_stress_fixture();
    let stress_model = stress.model();
    let parallel = stress_planner(&stress, &stress_model).plan();
    let serial = stress_planner(&stress, &stress_model).plan_serial_for_test();
    assert_eq!(parallel, serial, "stress fixture must match serial exactly");

    let expected_ids: Vec<String> = (1..=parallel.len())
        .map(|n| format!("PLAN-{n:03}"))
        .collect();
    assert_eq!(
        plan_ids(&parallel),
        expected_ids,
        "ids must be dense PLAN-001.. over collected order"
    );

    // The stress fixture really exercises every class.
    assert!(
        parallel.len() > 60,
        "most headers must plan, got {}",
        parallel.len()
    );
    assert!(
        parallel.iter().any(|plan| plan
            .proposed_extractions
            .iter()
            .filter(|ext| ext.symbol_name.starts_with('T'))
            .count()
            >= 3),
        "candidate headers must yield multiple SCC proposals each"
    );
    let planned_ids: Vec<u64> = parallel.iter().map(|plan| plan.target_file_id.0).collect();
    assert!(
        (0..STRESS_HEADER_COUNT)
            .filter(|index| index % STRESS_CYCLIC_EVERY == STRESS_CYCLIC_REMAINDER)
            .all(|index| !planned_ids.contains(&(HEADER_ID_BASE + index as u64))),
        "single-SCC headers must be dropped without burning ids"
    );
    assert!(
        (0..STRESS_HEADER_COUNT)
            .filter(|index| index % STRESS_FROZEN_EVERY == 7)
            .all(|index| !planned_ids.contains(&(HEADER_ID_BASE + index as u64))),
        "frozen headers must never plan"
    );
    assert!(
        (0..STRESS_HEADER_COUNT)
            .filter(|index| index % STRESS_ZERO_COLLATERAL_EVERY == 5)
            .all(|index| !planned_ids.contains(&(HEADER_ID_BASE + index as u64))),
        "include-free headers must screen out on zero collateral"
    );
}

#[test]
fn plan_is_deterministic_across_repeats() {
    // Guards rayon ordering flakiness: repeated plan() calls on one planner
    // must return byte-equal output on both paths.
    let stress = build_stress_fixture();
    let stress_model = stress.model();
    let closures = stress.graph.compute_closures();

    let cold = stress_planner(&stress, &stress_model);
    let cold_first = cold.plan();
    let facts = facts_planner(&stress, &stress_model, &closures);
    let facts_first = facts.plan();

    for repeat in 1..10 {
        assert_eq!(
            cold.plan(),
            cold_first,
            "cold plan() must be byte-equal on repeat {repeat}"
        );
        assert_eq!(
            facts.plan(),
            facts_first,
            "facts plan() must be byte-equal on repeat {repeat}"
        );
    }
}
