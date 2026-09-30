use super::*;
use ariadne_graph::reduced::{ReducedSemanticEdge, ReducedSymbolRecord};
use ariadne_graph::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_index::types::{DependencyKind, SymbolKind};
use ariadne_repo::{FileKind, FileOwnership};
use std::collections::BTreeMap;

/// Small-graph fixture builder for `OptimizationPlanner::plan` cases.
#[derive(Default)]
struct PlannerFixture {
    graph: IncludeGraph,
    model_symbols: BTreeMap<SymbolId, ReducedSymbolRecord>,
    model_edges: Vec<ReducedSemanticEdge>,
    file_symbols: BTreeMap<FileId, Vec<SymbolId>>,
    records: Vec<FileRecord>,
    tu_file_ids: Vec<FileId>,
}

impl PlannerFixture {
    fn new() -> Self {
        Self::default()
    }

    fn file(
        mut self,
        id: FileId,
        name: &str,
        kind: FileKind,
        ownership: FileOwnership,
        policy: OptimizationPolicy,
    ) -> Self {
        self.graph.register_file(id, PathBuf::from(name));
        self.records.push(FileRecord {
            id,
            path: PathBuf::from(name),
            canonical_path: PathBuf::from(name),
            kind,
            size: 0,
            content_hash: String::new(),
            ownership,
            optimization_policy: policy,
            language: ariadne_repo::SourceLanguage::Cpp,
        });
        self
    }

    fn header(self, id: FileId, name: &str, policy: OptimizationPolicy) -> Self {
        self.file(
            id,
            name,
            FileKind::Header,
            FileOwnership::Repository,
            policy,
        )
    }

    fn third_party_header(self, id: FileId, name: &str) -> Self {
        self.file(
            id,
            name,
            FileKind::Header,
            FileOwnership::ThirdParty,
            OptimizationPolicy::Frozen,
        )
    }

    fn translation_unit(mut self, id: FileId, name: &str) -> Self {
        self = self.file(
            id,
            name,
            FileKind::Source,
            FileOwnership::Repository,
            OptimizationPolicy::Optimizable,
        );
        self.tu_file_ids.push(id);
        self
    }

    fn includes(mut self, source: FileId, dest: FileId) -> Self {
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
            language: ariadne_repo::SourceLanguage::Cpp,
        });
        self
    }

    fn defines(mut self, file: FileId, name: &str) -> Self {
        let sym = SymbolId::new(name);
        self.model_symbols.insert(
            sym.clone(),
            ReducedSymbolRecord {
                id: sym.clone(),
                name: name.to_string(),
                kind: SymbolKind::Class,
                defining_file: Some(file),
                declarations: vec![],
                definition: None,
            },
        );
        self.file_symbols.entry(file).or_default().push(sym);
        self
    }

    fn depends(mut self, src: &str, dst: &str) -> Self {
        self.model_edges.push(ReducedSemanticEdge {
            src_symbol: SymbolId::new(src),
            dst_symbol: SymbolId::new(dst),
            kind: DependencyKind::Complete,
        });
        self
    }

    fn plan(&self) -> Vec<OptimizationPlan> {
        let model = ReducedSemanticModel {
            repo_root: PathBuf::from("/"),
            symbols: self.model_symbols.clone(),
            edges: self.model_edges.clone(),
            tu_usage: BTreeMap::new(),
            file_symbols: self.file_symbols.clone(),
        };
        OptimizationPlanner::new(&self.graph, &model, &self.records, &self.tu_file_ids).plan()
    }
}

fn collateral_header() -> PlannerFixture {
    PlannerFixture::new()
        .header(FileId(1), "start.h", OptimizationPolicy::Optimizable)
        .third_party_header(FileId(2), "col.h")
        .includes(FileId(1), FileId(2))
        .defines(FileId(1), "A")
        .defines(FileId(2), "B")
}

#[test]
fn plan_complete_structure_locks_current_output() {
    let plans = collateral_header().plan();

    assert_eq!(
        plans,
        vec![OptimizationPlan {
            id: PlanId("PLAN-001".to_string()),
            target_file: PathBuf::from("start.h"),
            target_file_id: FileId(1),
            problem: "1 symbols of collateral load".to_string(),
            collateral_load_before: 1,
            collateral_load_after: 0,
            proposed_extractions: vec![ProposedExtraction {
                symbol_name: "A".to_string(),
                symbol_id: SymbolId::new("A"),
                target_header: PathBuf::from("A_extracted.h"),
            }],
            optimization_frontier: vec![PathBuf::from("start.h")],
            risk: RiskLevel::Conservative,
            validation_required_tus: vec![],
            rationale: "File start.h has 1 collateral symbols mostly from col.h. \
                        Extracting independent symbols."
                .to_string(),
        }]
    );
}

#[test]
fn plan_skips_non_optimizable_target() {
    for policy in [OptimizationPolicy::Frozen, OptimizationPolicy::Contract] {
        let plans = PlannerFixture::new()
            .header(FileId(1), "start.h", policy)
            .third_party_header(FileId(2), "col.h")
            .includes(FileId(1), FileId(2))
            .defines(FileId(1), "A")
            .defines(FileId(2), "B")
            .plan();
        assert!(
            plans.is_empty(),
            "target policy {policy:?} must yield no plan"
        );
    }
}

#[test]
fn plan_skips_zero_collateral_header() {
    // col.h exists in the include graph but defines no symbols, so the
    // collateral load is zero and the header is skipped before SCC work.
    let plans = PlannerFixture::new()
        .header(FileId(1), "start.h", OptimizationPolicy::Optimizable)
        .third_party_header(FileId(2), "col.h")
        .includes(FileId(1), FileId(2))
        .defines(FileId(1), "A")
        .plan();
    assert!(plans.is_empty());
}

#[test]
fn plan_drops_all_extractions_when_file_symbols_form_single_scc() {
    // QUIRK: every symbol in a multi-symbol SCC is rejected because
    // splits_scc is inferred from `scc.symbols.len() > 1`, so a header whose
    // symbols are all mutually cyclic yields no plan at all, and the whole
    // header is dropped (empty proposals -> continue, no plan, no id burn).
    let plans = PlannerFixture::new()
        .header(FileId(1), "start.h", OptimizationPolicy::Optimizable)
        .third_party_header(FileId(2), "col.h")
        .includes(FileId(1), FileId(2))
        .defines(FileId(1), "A")
        .defines(FileId(1), "B")
        .defines(FileId(2), "C")
        .depends("A", "B")
        .depends("B", "A")
        .plan();
    assert!(plans.is_empty());
}

#[test]
fn plan_keeps_independent_symbols_alongside_a_cyclic_scc() {
    // C2 is outside the A<->B cycle, so its single-symbol SCC is proposed
    // even though the same file also contains a cycle.
    let plans = PlannerFixture::new()
        .header(FileId(1), "start.h", OptimizationPolicy::Optimizable)
        .third_party_header(FileId(2), "col.h")
        .includes(FileId(1), FileId(2))
        .defines(FileId(1), "A")
        .defines(FileId(1), "B")
        .defines(FileId(1), "C2")
        .defines(FileId(2), "C")
        .depends("A", "B")
        .depends("B", "A")
        .plan();
    assert_eq!(plans.len(), 1);
    assert_eq!(
        plans[0].proposed_extractions,
        vec![ProposedExtraction {
            symbol_name: "C2".to_string(),
            symbol_id: SymbolId::new("C2"),
            target_header: PathBuf::from("C2_extracted.h"),
        }]
    );
}

#[test]
fn plan_ids_follow_sorted_file_ids_and_skips_do_not_burn_numbers() {
    // FileId(1) is skipped (zero collateral), FileId(2) and FileId(3) both
    // include the frozen collateral source, so ids are dense in FileId order.
    let plans = PlannerFixture::new()
        .header(FileId(1), "zero.h", OptimizationPolicy::Optimizable)
        .header(FileId(2), "beta.h", OptimizationPolicy::Optimizable)
        .header(FileId(3), "alpha.h", OptimizationPolicy::Optimizable)
        .third_party_header(FileId(4), "col.h")
        .defines(FileId(2), "Beta")
        .defines(FileId(3), "Alpha")
        .defines(FileId(4), "Big")
        .includes(FileId(1), FileId(4))
        .includes(FileId(2), FileId(4))
        .includes(FileId(3), FileId(4))
        .plan();
    let ids: Vec<(String, u64)> = plans
        .iter()
        .map(|p| (p.id.0.clone(), p.target_file_id.0))
        .collect();
    assert_eq!(
        ids,
        vec![("PLAN-001".to_string(), 2), ("PLAN-002".to_string(), 3)]
    );
}

#[test]
fn plan_target_header_sanitizes_symbol_names() {
    // QUIRK: sanitized stems collide silently: "A_B" and "A::B" both target
    // A_B_extracted.h and the duplicate target headers are kept.
    let plans = PlannerFixture::new()
        .header(FileId(1), "start.h", OptimizationPolicy::Optimizable)
        .third_party_header(FileId(2), "col.h")
        .includes(FileId(1), FileId(2))
        .defines(FileId(2), "Big")
        .defines(FileId(1), "std::vector")
        .defines(FileId(1), "Thing<T>")
        .defines(FileId(1), ":::")
        .defines(FileId(1), "A_B")
        .defines(FileId(1), "A::B")
        .plan();
    assert_eq!(plans.len(), 1);
    let headers: Vec<String> = plans[0]
        .proposed_extractions
        .iter()
        .map(|ext| ext.target_header.to_string_lossy().to_string())
        .collect();
    assert_eq!(
        headers,
        vec![
            "std_vector_extracted.h",
            "Thing_T_extracted.h",
            "symbol_extracted.h",
            "A_B_extracted.h",
            "A_B_extracted.h",
        ]
    );
}

#[test]
fn plan_validation_required_tus_lists_consumer_paths() {
    let plans = collateral_header()
        .translation_unit(FileId(10), "main.cpp")
        .includes(FileId(10), FileId(1))
        .plan();
    assert_eq!(plans.len(), 1);
    assert_eq!(
        plans[0].validation_required_tus,
        vec![PathBuf::from("main.cpp")]
    );
}

#[test]
fn test_planner_only_considers_header_files() {
    let plans = PlannerFixture::new()
        .header(FileId(1), "start.h", OptimizationPolicy::Optimizable)
        .header(FileId(2), "col.h", OptimizationPolicy::Optimizable)
        .translation_unit(FileId(3), "impl.cc")
        .includes(FileId(1), FileId(2))
        .includes(FileId(3), FileId(1))
        .defines(FileId(2), "sym_1")
        .depends("sym_1", "sym_1")
        .plan();

    // Only header files should have proposals, not .cc files
    let target_files: Vec<_> = plans.iter().map(|p| p.target_file.clone()).collect();
    assert!(
        !target_files
            .iter()
            .any(|p| p.to_string_lossy().ends_with(".cc")),
        "Implementation files should not have extraction proposals"
    );
}
