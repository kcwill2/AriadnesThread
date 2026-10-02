use crate::checker::ConstraintChecker;
use crate::frontier::FrontierAnalyzer;
use crate::plan::{
    OptimizationPlan, PlanId, PlanRejection, ProposedExtraction, RejectionReason, RiskLevel,
};
use crate::scc;
use ariadne_graph::load::LoadAnalyzer;
use ariadne_graph::reduced::ReducedSemanticModel;
use ariadne_graph::IncludeGraph;
use ariadne_index::types::SymbolId;
use ariadne_lang::{ClusteringConstraints, CyclePolicy, LanguageId, SymbolRecord};
use ariadne_repo::{FileId, FileOwnership, FileRecord, OptimizationPolicy};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Builds extraction plans for one repository snapshot (plan §3.4).
///
/// The engine is language-neutral (plan §7 rule R2): which files are
/// aggregation units and what an extraction target is called are injected
/// per language (`aggregation_filter`, `target_path_for`); for C++ the
/// plugin supplies `ariadne_lang_cpp::{cpp_aggregation_filter,
/// cpp_target_path_for}`.
pub struct OptimizationPlanner<'a> {
    pub include_graph: &'a IncludeGraph,
    pub model: &'a ReducedSemanticModel,
    pub file_records: &'a [FileRecord],
    pub tu_file_ids: &'a [FileId],
    /// Only files passing this filter become candidate aggregation units
    /// (plan M18.6); the pipeline supplies it from the language plugin.
    pub aggregation_filter: &'a dyn Fn(&Path) -> bool,
    /// Produces the proposal target path for one co-proposed batch, called
    /// with (candidate aggregation file path, batch anchor symbol name)
    /// (plan M18.6, §3.6 "Target file naming").
    pub target_path_for: &'a dyn Fn(&Path, &str) -> PathBuf,
    /// Optional language clustering constraints (plan M18.7): colocated
    /// groups union-merge SCC proposal batches; frozen symbols are skipped.
    pub constraints: Option<&'a dyn ClusteringConstraints>,
    /// Cycle stance (plan M18.7): `Reject` rejects any proposal that would
    /// split an SCC via `ConstraintChecker`. `Tolerate` and
    /// `TolerateWithinPackage` share that path here — SCC members are already
    /// co-proposed as one batch, so cycles are never split, and the planner
    /// keeps behaving identically under every policy (locked by
    /// `tests/planner_injection.rs:315` `cycle_policy_variants_share_the_scc_coproposal_path_for_now`).
    /// What shipped in M20 (D11): the stance is consumed on the *report* side
    /// — `IncludeGraph::generate_report_with_cycle_policies` classifies a
    /// cycle as informational when every member's policy is non-`Reject`.
    /// Package-scoped cycle precision is superseded by the M23.5
    /// group-condensation milestone, not by this field.
    pub cycle_policy: CyclePolicy,
    /// Languages injected by the caller whose plugins report
    /// `capabilities().planning == false` (plan M22, D23). Each gap yields one
    /// explanatory rejection from `plan_with_rejections`, so planning output is
    /// never silently empty. Empty (the default) keeps `plan()` and
    /// `plan_with_rejections().0` byte-identical to today.
    planning_gaps: Vec<PlanningGap>,
}

/// A language present in the repository whose plugin disables the planning
/// stage (plan M22, D23). Computed by the caller from discovery records and
/// the registry — the engine never consults a registry itself (rule R2): the
/// language arrives as neutral `LanguageId` data with a representative
/// discovered path so the resulting rejection has an honest `target_file`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningGap {
    pub language: LanguageId,
    pub sample_path: PathBuf,
}

/// Optimization policy and ownership views built once per planning run.
struct FilePolicyMaps {
    by_id: HashMap<FileId, OptimizationPolicy>,
    ownership_by_id: HashMap<FileId, FileOwnership>,
    policy_by_path: HashMap<PathBuf, OptimizationPolicy>,
}

/// A file that passed the load, collateral-source, and policy screens and
/// is eligible for extraction proposals.
struct AggregationCandidate {
    file_id: FileId,
    path: PathBuf,
    collateral_load: usize,
    collateral_source_id: FileId,
    collateral_source_path: PathBuf,
}

impl<'a> OptimizationPlanner<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        include_graph: &'a IncludeGraph,
        model: &'a ReducedSemanticModel,
        file_records: &'a [FileRecord],
        tu_file_ids: &'a [FileId],
        aggregation_filter: &'a dyn Fn(&Path) -> bool,
        target_path_for: &'a dyn Fn(&Path, &str) -> PathBuf,
        constraints: Option<&'a dyn ClusteringConstraints>,
        cycle_policy: CyclePolicy,
    ) -> Self {
        Self {
            include_graph,
            model,
            file_records,
            tu_file_ids,
            aggregation_filter,
            target_path_for,
            constraints,
            cycle_policy,
            planning_gaps: Vec::new(),
        }
    }

    /// Injects the caller-computed planning gaps (plan M22, D23) without
    /// widening the constructor; returns `self` for builder-style use.
    pub fn with_planning_gaps(&mut self, planning_gaps: Vec<PlanningGap>) -> &mut Self {
        self.planning_gaps = planning_gaps;
        self
    }

    /// Plans exactly as `plan()` and additionally returns one explanatory
    /// rejection per injected planning gap (plan M22, D23), independent of the
    /// candidate pass. With no gaps injected the rejection channel is empty
    /// and the plans are byte-identical to `plan()`.
    pub fn plan_with_rejections(&self) -> (Vec<OptimizationPlan>, Vec<PlanRejection>) {
        let rejections = self
            .planning_gaps
            .iter()
            .map(|gap| PlanRejection {
                target_file: gap.sample_path.clone(),
                reason: RejectionReason::PlanningUnsupported(gap.language),
                explanation: format!(
                    "Planning is not supported for {:?}: the registered plugin disables the planning stage, so no extraction plans are proposed for {:?} files such as {}.",
                    gap.language,
                    gap.language,
                    gap.sample_path.display()
                ),
            })
            .collect();
        (self.plan(), rejections)
    }

    pub fn plan(&self) -> Vec<OptimizationPlan> {
        let load_analyzer = LoadAnalyzer::new(self.include_graph, self.model);
        let frontier_analyzer = FrontierAnalyzer::new();
        let policies = self.collect_file_policies();
        let checker = ConstraintChecker::new(policies.policy_by_path.clone(), HashSet::new());
        let mapped_edges = self.model_edges_without_file_endpoints();
        let aggregation_consumers = self
            .include_graph
            .map_aggregation_consumers(self.tu_file_ids);

        let mut plans = Vec::new();
        for file_id in self.sorted_candidate_file_ids() {
            let Some(candidate) =
                self.candidate_aggregation_file(file_id, &load_analyzer, &policies)
            else {
                continue;
            };
            let proposed_extractions =
                self.propose_extractions(&candidate, &mapped_edges, &checker);
            if proposed_extractions.is_empty() {
                continue;
            }
            plans.push(self.assemble_plan(
                &candidate,
                proposed_extractions,
                plans.len() + 1,
                &frontier_analyzer,
                &policies,
                &aggregation_consumers,
            ));
        }

        plans
    }

    fn collect_file_policies(&self) -> FilePolicyMaps {
        let mut by_id = HashMap::new();
        let mut ownership_by_id = HashMap::new();
        let mut policy_by_path = HashMap::new();
        for record in self.file_records {
            let path = self
                .include_graph
                .file_paths
                .get(&record.id)
                .cloned()
                .unwrap_or_default();
            by_id.insert(record.id, record.optimization_policy);
            ownership_by_id.insert(record.id, record.ownership.clone());
            if !path.as_os_str().is_empty() {
                policy_by_path.insert(path, record.optimization_policy);
            }
        }
        FilePolicyMaps {
            by_id,
            ownership_by_id,
            policy_by_path,
        }
    }

    fn sorted_candidate_file_ids(&self) -> Vec<FileId> {
        let mut sorted: Vec<FileId> = self
            .include_graph
            .file_paths
            .iter()
            .filter(|(_, path)| (self.aggregation_filter)(path))
            .map(|(id, _)| *id)
            .collect();
        sorted.sort();
        sorted
    }

    fn candidate_aggregation_file(
        &self,
        file_id: FileId,
        load_analyzer: &LoadAnalyzer,
        policies: &FilePolicyMaps,
    ) -> Option<AggregationCandidate> {
        let path = self.include_graph.file_paths.get(&file_id)?.clone();
        let metric = load_analyzer.analyze_file_load(&path).ok()?;
        if metric.collateral_load == 0 {
            return None;
        }
        let largest_collateral_source = metric.largest_collateral_source.as_ref()?;

        let policy = policies
            .by_id
            .get(&file_id)
            .copied()
            .unwrap_or(OptimizationPolicy::Optimizable);
        if policy != OptimizationPolicy::Optimizable {
            return None;
        }

        Some(AggregationCandidate {
            file_id,
            path,
            collateral_load: metric.collateral_load,
            collateral_source_id: largest_collateral_source.file_id,
            collateral_source_path: largest_collateral_source.path.clone(),
        })
    }

    /// Semantic edges with their file endpoints erased, as consumed by the
    /// per-file SCC computation.
    fn model_edges_without_file_endpoints(&self) -> Vec<ariadne_index::types::SemanticEdge> {
        self.model
            .edges
            .iter()
            .map(|e| ariadne_index::types::SemanticEdge {
                src_symbol: e.src_symbol.clone(),
                dst_symbol: e.dst_symbol.clone(),
                kind: e.kind,
                src_file: FileId(0),
                dst_file: FileId(0),
            })
            .collect()
    }

    /// One proposal batch per SCC (union-merged by colocated constraints);
    /// every batch is co-proposed to one shared target path. Members of a
    /// multi-symbol SCC keep today's `splits_scc` rejection regardless of
    /// `cycle_policy` — see the field docs for the M18.7 stance.
    fn propose_extractions(
        &self,
        candidate: &AggregationCandidate,
        mapped_edges: &[ariadne_index::types::SemanticEdge],
        checker: &ConstraintChecker,
    ) -> Vec<ProposedExtraction> {
        let file_symbols = self
            .model
            .file_symbols
            .get(&candidate.file_id)
            .cloned()
            .unwrap_or_default();
        let sccs = scc::compute_symbol_sccs(&file_symbols, mapped_edges);
        let cycle_members: HashSet<&SymbolId> = sccs
            .iter()
            .filter(|component| component.symbols.len() > 1)
            .flat_map(|component| &component.symbols)
            .collect();

        let mut batches: Vec<Vec<SymbolId>> = sccs
            .iter()
            .map(|component| component.symbols.clone())
            .collect();
        let colocated_groups = self
            .constraints
            .map(|constraints| constraints.colocated_groups(&self.symbol_records(&file_symbols)))
            .unwrap_or_default();
        merge_colocated_batches(&mut batches, &colocated_groups);
        let frozen = self.frozen_symbol_set(&file_symbols);

        let mut proposed_extractions = Vec::new();
        for batch in &batches {
            proposed_extractions.extend(self.propose_batch(
                batch,
                candidate,
                &cycle_members,
                &frozen,
                checker,
            ));
        }
        proposed_extractions
    }

    /// Emits one [`ProposedExtraction`] per batch member, all targeting the
    /// path produced from the batch's anchor symbol (its first member with a
    /// symbol record). A batch holding a frozen symbol is skipped whole:
    /// colocated members must move together, and one immovable member makes
    /// that impossible.
    fn propose_batch(
        &self,
        batch: &[SymbolId],
        candidate: &AggregationCandidate,
        cycle_members: &HashSet<&SymbolId>,
        frozen: &HashSet<SymbolId>,
        checker: &ConstraintChecker,
    ) -> Vec<ProposedExtraction> {
        if batch.iter().any(|symbol| frozen.contains(symbol)) {
            return Vec::new();
        }
        let members: Vec<(&SymbolId, _)> = batch
            .iter()
            .filter_map(|symbol| Some((symbol, self.model.symbols.get(symbol)?)))
            .collect();
        let Some((_, anchor)) = members.first() else {
            return Vec::new();
        };
        let target_path = (self.target_path_for)(&candidate.path, &anchor.name);
        members
            .into_iter()
            .filter_map(|(symbol, record)| {
                let splits_scc = cycle_members.contains(symbol);
                checker
                    .validate_extraction(symbol, &target_path, splits_scc, false, None)
                    .ok()?;
                Some(ProposedExtraction {
                    symbol_name: record.name.clone(),
                    symbol_id: symbol.clone(),
                    target_header: target_path.clone(),
                    source_file: Some(candidate.file_id),
                })
            })
            .collect()
    }

    /// The neutral symbol views handed to the language constraints trait.
    fn symbol_records(&self, file_symbols: &[SymbolId]) -> Vec<SymbolRecord> {
        file_symbols
            .iter()
            .filter_map(|id| self.model.symbols.get(id))
            .filter_map(|record| {
                let owning_file = record.defining_file?;
                Some(ariadne_index::types::SymbolRecord {
                    id: record.id.clone(),
                    name: record.name.clone(),
                    kind: record.kind,
                    owning_file,
                    declarations: record.declarations.clone(),
                    definition: record.definition.clone(),

                    modifiers: Vec::new(),
                })
            })
            .collect()
    }

    fn frozen_symbol_set(&self, file_symbols: &[SymbolId]) -> HashSet<SymbolId> {
        match self.constraints {
            Some(constraints) => constraints.frozen_symbols(&self.symbol_records(file_symbols)),
            None => HashSet::new(),
        }
    }

    fn assemble_plan(
        &self,
        candidate: &AggregationCandidate,
        proposed_extractions: Vec<ProposedExtraction>,
        plan_number: usize,
        frontier_analyzer: &FrontierAnalyzer,
        policies: &FilePolicyMaps,
        aggregation_consumers: &HashMap<FileId, Vec<FileId>>,
    ) -> OptimizationPlan {
        let optimization_frontier = frontier_analyzer.find_frontiers(
            candidate.file_id,
            candidate.collateral_source_id,
            self.include_graph,
            &policies.by_id,
            &policies.ownership_by_id,
        );

        let validation_required_tus = aggregation_consumers
            .get(&candidate.file_id)
            .map(|tus| {
                tus.iter()
                    .filter_map(|id| self.include_graph.file_paths.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default();

        let rationale = format!(
            "File {} has {} collateral symbols mostly from {}. Extracting independent symbols.",
            candidate.path.display(),
            candidate.collateral_load,
            candidate.collateral_source_path.display()
        );

        OptimizationPlan {
            id: PlanId(format!("PLAN-{:03}", plan_number)),
            target_file: candidate.path.clone(),
            target_file_id: candidate.file_id,
            problem: format!("{} symbols of collateral load", candidate.collateral_load),
            collateral_load_before: candidate.collateral_load,
            collateral_load_after: 0,
            proposed_extractions,
            optimization_frontier,
            risk: RiskLevel::Conservative,
            validation_required_tus,
            rationale,
        }
    }

    pub fn check_candidate_safety(
        &self,
        symbol: &SymbolId,
        target_file: &Path,
        splits_scc: bool,
        creates_include_cycle: bool,
        unresolved_dep: Option<String>,
        file_policies: HashMap<PathBuf, OptimizationPolicy>,
    ) -> Result<(), PlanRejection> {
        let checker = ConstraintChecker::new(file_policies, HashSet::new());
        checker.validate_extraction(
            symbol,
            target_file,
            splits_scc,
            creates_include_cycle,
            unresolved_dep,
        )
    }
}

/// Union-merge proposal batches whose symbol sets intersect a colocated
/// group (plan M18.7): the first intersected batch stays in place as the
/// merge anchor, later intersected batches are appended in reverse
/// encounter order (last-intersected batch first; removing back-to-front
/// keeps earlier batch indices valid). Groups are applied sequentially, so
/// a symbol shared by two groups transitively merges them.
fn merge_colocated_batches(batches: &mut Vec<Vec<SymbolId>>, colocated_groups: &[Vec<SymbolId>]) {
    for group in colocated_groups {
        let group_symbols: HashSet<&SymbolId> = group.iter().collect();
        let intersecting: Vec<usize> = batches
            .iter()
            .enumerate()
            .filter(|(_, batch)| batch.iter().any(|symbol| group_symbols.contains(symbol)))
            .map(|(index, _)| index)
            .collect();
        let Some((&anchor, rest)) = intersecting.split_first() else {
            continue;
        };
        for &merge_index in rest.iter().rev() {
            let merged = batches.remove(merge_index);
            let anchor_batch = &mut batches[anchor];
            for symbol in merged {
                if !anchor_batch.contains(&symbol) {
                    anchor_batch.push(symbol);
                }
            }
        }
    }
}

#[cfg(test)]
mod planning_gap_tests {
    use super::*;
    use ariadne_graph::reduced::ReducedSymbolRecord;
    use ariadne_graph::types::{IncludeEdge, IncludeKind, SourceLocation};
    use ariadne_index::types::SymbolKind;
    use ariadne_lang::LanguageId;
    use ariadne_repo::{FileKind, SourceLanguage};
    use std::collections::BTreeMap;

    /// Snapshot the planner borrows: one optimizable aggregation candidate
    /// with collateral load from a frozen source (neutral `.agg` fixture,
    /// mirroring `tests/planner_injection.rs`).
    struct GapFixture {
        graph: IncludeGraph,
        model: ReducedSemanticModel,
        records: Vec<FileRecord>,
        tu_file_ids: Vec<FileId>,
    }

    impl GapFixture {
        fn empty() -> Self {
            Self {
                graph: IncludeGraph::default(),
                model: ReducedSemanticModel {
                    repo_root: PathBuf::from("/"),
                    symbols: BTreeMap::new(),
                    edges: Vec::new(),
                    tu_usage: BTreeMap::new(),
                    file_symbols: BTreeMap::new(),

                    extends_edges: Vec::new(),
                    sealed_symbols: Vec::new(),
                },
                records: Vec::new(),
                tu_file_ids: Vec::new(),
            }
        }

        fn record(
            id: FileId,
            name: &str,
            ownership: FileOwnership,
            policy: OptimizationPolicy,
        ) -> FileRecord {
            FileRecord {
                id,
                path: PathBuf::from(name),
                canonical_path: PathBuf::from(name),
                kind: FileKind::Other,
                size: 0,
                content_hash: String::new(),
                ownership,
                optimization_policy: policy,
                language: SourceLanguage::Other,
            }
        }

        fn symbol(&mut self, file: FileId, name: &str) {
            let symbol_id = SymbolId::new(name);
            self.model.symbols.insert(
                symbol_id.clone(),
                ReducedSymbolRecord {
                    id: symbol_id.clone(),
                    name: name.to_string(),
                    kind: SymbolKind::Class,
                    defining_file: Some(file),
                    declarations: Vec::new(),
                    definition: None,
                    declaring_files: Vec::new(),
                },
            );
            self.model
                .file_symbols
                .entry(file)
                .or_default()
                .push(symbol_id);
        }

        fn collateral_pair() -> Self {
            let mut fixture = Self::empty();
            fixture
                .graph
                .register_file(FileId(1), PathBuf::from("pkg.agg"));
            fixture
                .graph
                .register_file(FileId(2), PathBuf::from("col.agg"));
            fixture.graph.add_edge(IncludeEdge {
                source: FileId(1),
                destination: Some(FileId(2)),
                context_name: "Default".into(),
                source_location: SourceLocation::new(1, 1),
                kind: IncludeKind::Local,
                raw_path: "col.agg".to_string(),
                resolved_path: Some(PathBuf::from("col.agg")),
                language: SourceLanguage::Other,
            });
            fixture.records.push(Self::record(
                FileId(1),
                "pkg.agg",
                FileOwnership::Repository,
                OptimizationPolicy::Optimizable,
            ));
            fixture.records.push(Self::record(
                FileId(2),
                "col.agg",
                FileOwnership::ThirdParty,
                OptimizationPolicy::Frozen,
            ));
            fixture.symbol(FileId(1), "Alpha");
            fixture.symbol(FileId(1), "Delta");
            fixture.symbol(FileId(2), "Big");
            fixture
        }
    }

    fn accept_unit(path: &Path) -> bool {
        path.file_name().is_some()
    }

    fn unit_template(path: &Path, symbol: &str) -> PathBuf {
        path.with_file_name(format!("{symbol}_unit"))
    }

    fn planner_over(fixture: &GapFixture) -> OptimizationPlanner<'_> {
        OptimizationPlanner::new(
            &fixture.graph,
            &fixture.model,
            &fixture.records,
            &fixture.tu_file_ids,
            &accept_unit,
            &unit_template,
            None,
            CyclePolicy::Reject,
        )
    }

    fn neutral_gap(sample: &str) -> PlanningGap {
        PlanningGap {
            language: LanguageId::Other,
            sample_path: PathBuf::from(sample),
        }
    }

    #[test]
    fn gaps_produce_rejections_per_language() {
        let fixture = GapFixture::collateral_pair();
        let mut planner = planner_over(&fixture);
        planner.with_planning_gaps(vec![neutral_gap("payload.unknown_ext")]);

        let (plans, rejections) = planner.plan_with_rejections();

        assert_eq!(plans.len(), 1, "gaps do not disturb the candidate pass");
        assert_eq!(
            rejections.len(),
            1,
            "exactly one rejection per injected gap language"
        );
        let rejection = &rejections[0];
        assert!(matches!(
            rejection.reason,
            RejectionReason::PlanningUnsupported(LanguageId::Other)
        ));
        assert_eq!(
            rejection.target_file,
            PathBuf::from("payload.unknown_ext"),
            "the gap's representative path is the honest target_file"
        );
        assert!(
            rejection.explanation.contains("Other"),
            "explanation names the language by data (Debug of LanguageId): {}",
            rejection.explanation
        );

        let empty_fixture = GapFixture::empty();
        let mut bare = planner_over(&empty_fixture);
        bare.with_planning_gaps(vec![neutral_gap("payload.unknown_ext")]);
        let (bare_plans, bare_rejections) = bare.plan_with_rejections();
        assert!(bare_plans.is_empty());
        assert_eq!(
            bare_rejections.len(),
            1,
            "rejections are independent of the candidate pass"
        );
    }

    #[test]
    fn no_gaps_rejections_empty() {
        let fixture = GapFixture::collateral_pair();
        let planner = planner_over(&fixture);

        let (plans, rejections) = planner.plan_with_rejections();

        assert!(rejections.is_empty(), "no gaps, no rejections");
        assert_eq!(
            plans,
            planner.plan(),
            "delegation equality: gap-free plans are identical"
        );
    }

    #[test]
    fn plan_delegates_unchanged() {
        let fixture = GapFixture::collateral_pair();
        let mut with_gaps = planner_over(&fixture);
        with_gaps.with_planning_gaps(vec![neutral_gap("payload.unknown_ext")]);
        let without_gaps = planner_over(&fixture);

        assert_eq!(
            with_gaps.plan(),
            without_gaps.plan(),
            "plan() drops the rejection channel: gap-free plans are identical"
        );
        assert_eq!(
            with_gaps.plan(),
            with_gaps.plan_with_rejections().0,
            "plan() equals the plans channel of plan_with_rejections()"
        );
    }
}
