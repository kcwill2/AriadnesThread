use crate::checker::ConstraintChecker;
use crate::frontier::FrontierAnalyzer;
use crate::plan::{OptimizationPlan, PlanId, PlanRejection, ProposedExtraction, RiskLevel};
use crate::scc;
use ariadne_graph::load::LoadAnalyzer;
use ariadne_graph::reduced::ReducedSemanticModel;
use ariadne_graph::{IncludeClosure, IncludeGraph, LoadMetric};
use ariadne_index::types::SymbolId;
use ariadne_repo::{FileId, FileOwnership, FileRecord, OptimizationPolicy};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn is_header_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext, "h" | "hpp" | "hxx" | "hh" | "inl"))
        .unwrap_or(false)
}

/// Precomputed candidate-screening inputs for one header, decoupled from the
/// load DTOs: the stage's load pass already knows a header's collateral load
/// and dominant collateral source, so `with_screening_facts` never re-runs
/// the cold `LoadAnalyzer` scan per header. Field types mirror `LoadMetric`
/// (`collateral_load: usize`, largest source keyed by `FileId` — its path is
/// re-resolved from the include graph, the same way `LoadAnalyzer` derived
/// it in `contributing_sources`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderScreeningFacts {
    pub collateral_load: usize,
    pub largest_collateral_source: Option<FileId>,
}

/// Build the header screening facts map from a stage `LoadReport`'s
/// `file_metrics`: each metric's `target_file` path is resolved to its
/// `FileId` through the include graph, and metrics whose target path is
/// unregistered (or absent) are skipped — an unregistered target can never
/// be a candidate. Metrics exist exactly for files with symbols, so headers
/// without symbols end up fact-less and are screened as load 0, matching the
/// cold path's outcome of zero proposals (no own symbols ⇒ no extraction).
pub fn screening_facts_from_metrics(
    include_graph: &IncludeGraph,
    file_metrics: &[LoadMetric],
) -> HashMap<FileId, HeaderScreeningFacts> {
    let id_by_path: HashMap<&Path, FileId> = include_graph
        .file_paths
        .iter()
        .map(|(id, path)| (path.as_path(), *id))
        .collect();
    file_metrics
        .iter()
        .filter_map(|metric| {
            let file_id = *id_by_path.get(metric.target_file.as_deref()?)?;
            Some((
                file_id,
                HeaderScreeningFacts {
                    collateral_load: metric.collateral_load,
                    largest_collateral_source: metric
                        .largest_collateral_source
                        .as_ref()
                        .map(|source| source.file_id),
                },
            ))
        })
        .collect()
}

pub struct OptimizationPlanner<'a> {
    pub include_graph: &'a IncludeGraph,
    pub model: &'a ReducedSemanticModel,
    pub file_records: &'a [FileRecord],
    pub tu_file_ids: &'a [FileId],
    /// `Some` on the UI fast path: per-header screening inputs and the
    /// stage's include closures, precomputed by the pipeline stages.
    screening_facts: Option<HashMap<FileId, HeaderScreeningFacts>>,
    stage_closures: Option<&'a HashMap<FileId, IncludeClosure>>,
}

/// Optimization policy and ownership views built once per planning run.
struct FilePolicyMaps {
    by_id: HashMap<FileId, OptimizationPolicy>,
    ownership_by_id: HashMap<FileId, FileOwnership>,
    policy_by_path: HashMap<PathBuf, OptimizationPolicy>,
}

/// A header that passed the load, collateral-source, and policy screens and
/// is eligible for extraction proposals.
struct HeaderCandidate {
    file_id: FileId,
    path: PathBuf,
    collateral_load: usize,
    collateral_source_id: FileId,
    collateral_source_path: PathBuf,
}

/// A screened header plus its validated extractions, awaiting sequential plan
/// assembly and dense `PLAN-{:03}` numbering. Kept id-free so the per-header
/// step is pure and the parallel collect can renumber afterwards.
struct PendingPlan {
    candidate: HeaderCandidate,
    proposed_extractions: Vec<ProposedExtraction>,
}

/// Read-only inputs shared by every per-header planning step, built once per
/// planning run. Holds the hoisted model-wide symbol adjacency (A2: the SCC
/// pass no longer rebuilds it per header) and the cold analyzer, which is
/// `None` on the facts path.
struct PlanningContext<'a> {
    cold_analyzer: Option<LoadAnalyzer<'a>>,
    frontier_analyzer: FrontierAnalyzer,
    policies: FilePolicyMaps,
    checker: ConstraintChecker,
    adjacency: HashMap<&'a SymbolId, Vec<&'a SymbolId>>,
    header_consumers: HashMap<FileId, Vec<FileId>>,
}

impl<'a> OptimizationPlanner<'a> {
    /// Cold path (CLI `tool optimize`/`plan`): screening re-analyzes each
    /// header serially with a fresh `LoadAnalyzer`. Behavior is untouched by
    /// the facts fast path below.
    pub fn new(
        include_graph: &'a IncludeGraph,
        model: &'a ReducedSemanticModel,
        file_records: &'a [FileRecord],
        tu_file_ids: &'a [FileId],
    ) -> Self {
        Self {
            include_graph,
            model,
            file_records,
            tu_file_ids,
            screening_facts: None,
            stage_closures: None,
        }
    }

    /// Facts path (UI payload build): candidate screening reads the stage's
    /// precomputed [`HeaderScreeningFacts`] and consumer mapping reuses the
    /// stage's include `closures`, so `plan()` never constructs the cold
    /// `LoadAnalyzer` and never recomputes closures. Plan output is parity-
    /// locked against the cold path by `tests/planner_facts_parity.rs`.
    pub fn with_screening_facts(
        include_graph: &'a IncludeGraph,
        model: &'a ReducedSemanticModel,
        file_records: &'a [FileRecord],
        tu_file_ids: &'a [FileId],
        screening_facts: HashMap<FileId, HeaderScreeningFacts>,
        closures: &'a HashMap<FileId, IncludeClosure>,
    ) -> Self {
        Self {
            include_graph,
            model,
            file_records,
            tu_file_ids,
            screening_facts: Some(screening_facts),
            stage_closures: Some(closures),
        }
    }

    /// Screening + proposal discovery run in parallel over sorted header ids
    /// (`par_iter` collect preserves input order, mirroring
    /// `collect_file_metrics` in the pipeline); assembly and dense
    /// `PLAN-{:03}` numbering then follow collected order, so output is
    /// byte-identical to the pre-A2 serial loop — locked against
    /// [`plan_serial_for_test`] by `tests/planner_facts_parity.rs`.
    pub fn plan(&self) -> Vec<OptimizationPlan> {
        let context = self.build_planning_context();
        let pending: Vec<PendingPlan> = self
            .sorted_header_ids()
            .par_iter()
            .filter_map(|&file_id| self.header_proposal(file_id, &context))
            .collect();
        self.number_pending_plans(pending, &context)
    }

    /// Characterization driver reproducing the pre-A2 serial loop verbatim
    /// through the same shared steps, for the parallel-parity lock. Gate-less
    /// `doc(hidden)`: integration tests compile against the non-test cfg
    /// crate, so `#[cfg(test)]`/`pub(crate)` would not be visible to
    /// `tests/planner_facts_parity.rs`.
    #[doc(hidden)]
    pub fn plan_serial_for_test(&self) -> Vec<OptimizationPlan> {
        let context = self.build_planning_context();
        let pending: Vec<PendingPlan> = self
            .sorted_header_ids()
            .into_iter()
            .filter_map(|file_id| self.header_proposal(file_id, &context))
            .collect();
        self.number_pending_plans(pending, &context)
    }

    fn build_planning_context(&self) -> PlanningContext<'a> {
        // Facts path must never construct the cold analyzer: building it and
        // scanning every header serially is exactly the tail A1 removes.
        let cold_analyzer = if self.screening_facts.is_some() {
            None
        } else {
            Some(LoadAnalyzer::new(self.include_graph, self.model))
        };
        let policies = self.collect_file_policies();
        let checker = ConstraintChecker::new(policies.policy_by_path.clone(), HashSet::new());
        PlanningContext {
            cold_analyzer,
            frontier_analyzer: FrontierAnalyzer::new(),
            policies,
            checker,
            adjacency: self.symbol_adjacency(),
            header_consumers: self.map_header_consumers(),
        }
    }

    /// The per-header step with no id assignment (pure over `context`):
    /// screen, then propose extractions from the shared adjacency. Rejects
    /// (unscreenable headers, zero-proposal candidates) collapse to `None`,
    /// exactly like the old `continue`s.
    fn header_proposal(
        &self,
        file_id: FileId,
        context: &PlanningContext<'a>,
    ) -> Option<PendingPlan> {
        let candidate =
            self.candidate_header(file_id, context.cold_analyzer.as_ref(), &context.policies)?;
        let proposed_extractions =
            self.propose_extractions(&candidate, &context.adjacency, &context.checker);
        if proposed_extractions.is_empty() {
            return None;
        }
        Some(PendingPlan {
            candidate,
            proposed_extractions,
        })
    }

    /// Sequential assembly after collection: enumerating the collected
    /// pending plans reproduces the serial loop's dense `plans.len() + 1`
    /// numbering exactly.
    fn number_pending_plans(
        &self,
        pending: Vec<PendingPlan>,
        context: &PlanningContext<'a>,
    ) -> Vec<OptimizationPlan> {
        pending
            .into_iter()
            .enumerate()
            .map(|(index, plan)| {
                self.assemble_plan(
                    &plan.candidate,
                    plan.proposed_extractions,
                    index + 1,
                    &context.frontier_analyzer,
                    &context.policies,
                    &context.header_consumers,
                )
            })
            .collect()
    }

    /// Consumer TUs per header: reuse the stage closure map on the facts
    /// path; recompute closures on the cold path, as before.
    fn map_header_consumers(&self) -> HashMap<FileId, Vec<FileId>> {
        match self.stage_closures {
            Some(closures) => self
                .include_graph
                .map_header_consuming_tus_with_closures(self.tu_file_ids, closures),
            None => self
                .include_graph
                .map_header_consuming_tus(self.tu_file_ids),
        }
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

    fn sorted_header_ids(&self) -> Vec<FileId> {
        let mut sorted_headers: Vec<FileId> = self
            .include_graph
            .file_paths
            .iter()
            .filter(|(_, path)| is_header_file(path))
            .map(|(id, _)| *id)
            .collect();
        sorted_headers.sort();
        sorted_headers
    }

    fn candidate_header(
        &self,
        file_id: FileId,
        cold_analyzer: Option<&LoadAnalyzer>,
        policies: &FilePolicyMaps,
    ) -> Option<HeaderCandidate> {
        let path = self.include_graph.file_paths.get(&file_id)?.clone();
        let (collateral_load, collateral_source_id) = match &self.screening_facts {
            Some(facts) => self.screen_from_facts(file_id, facts)?,
            None => self.screen_cold(&path, cold_analyzer?)?,
        };
        if collateral_load == 0 {
            return None;
        }

        let policy = policies
            .by_id
            .get(&file_id)
            .copied()
            .unwrap_or(OptimizationPolicy::Optimizable);
        if policy != OptimizationPolicy::Optimizable {
            return None;
        }

        Some(HeaderCandidate {
            file_id,
            path,
            collateral_load,
            collateral_source_id,
            collateral_source_path: self.collateral_source_path(collateral_source_id),
        })
    }

    /// Facts screening: no metric means the header has no symbols, which on
    /// the cold path yields no proposals and therefore no plan — screening
    /// it out here is the same observable outcome.
    fn screen_from_facts(
        &self,
        file_id: FileId,
        facts: &HashMap<FileId, HeaderScreeningFacts>,
    ) -> Option<(usize, FileId)> {
        let fact = facts.get(&file_id)?;
        Some((fact.collateral_load, fact.largest_collateral_source?))
    }

    /// Cold screening: the original per-header `analyze_file_load` scan,
    /// kept verbatim so `new()`-constructed plans stay byte-identical.
    fn screen_cold(&self, path: &Path, cold_analyzer: &LoadAnalyzer) -> Option<(usize, FileId)> {
        let metric = cold_analyzer.analyze_file_load(path).ok()?;
        let source = metric.largest_collateral_source.as_ref()?;
        Some((metric.collateral_load, source.file_id))
    }

    /// Same path derivation `LoadAnalyzer::contributing_sources` used when
    /// it built `largest_collateral_source`, keeping the rationale text
    /// byte-identical between the cold and facts paths.
    fn collateral_source_path(&self, source_id: FileId) -> PathBuf {
        self.include_graph
            .file_paths
            .get(&source_id)
            .cloned()
            .unwrap_or_else(|| PathBuf::from("unknown"))
    }

    /// Model-wide symbol adjacency with file endpoints ignored (SCC uses only
    /// symbol symbols), built once per planning run. Neighbor lists follow
    /// `model.edges` iteration order — identical to the adjacency the old
    /// per-candidate rebuild derived from the endpoint-erased edge clone.
    fn symbol_adjacency(&self) -> HashMap<&'a SymbolId, Vec<&'a SymbolId>> {
        let mut adjacency: HashMap<&'a SymbolId, Vec<&'a SymbolId>> = HashMap::new();
        for edge in self.model.edges.iter() {
            adjacency
                .entry(&edge.src_symbol)
                .or_default()
                .push(&edge.dst_symbol);
        }
        adjacency
    }

    fn propose_extractions(
        &self,
        candidate: &HeaderCandidate,
        adjacency: &HashMap<&'a SymbolId, Vec<&'a SymbolId>>,
        checker: &ConstraintChecker,
    ) -> Vec<ProposedExtraction> {
        let file_syms = self
            .model
            .file_symbols
            .get(&candidate.file_id)
            .cloned()
            .unwrap_or_default();
        let sccs = scc::compute_symbol_sccs_from_adj(&file_syms, adjacency);

        let mut proposed_extractions = Vec::new();
        for scc in &sccs {
            for sym_id in &scc.symbols {
                if let Some(record) = self.model.symbols.get(sym_id) {
                    let new_header_path = candidate.path.with_file_name(format!(
                        "{}_extracted.h",
                        sanitize_header_stem(&record.name)
                    ));
                    if checker
                        .validate_extraction(
                            sym_id,
                            &new_header_path,
                            scc.symbols.len() > 1,
                            false,
                            None,
                        )
                        .is_ok()
                    {
                        proposed_extractions.push(ProposedExtraction {
                            symbol_name: record.name.clone(),
                            symbol_id: sym_id.clone(),
                            target_header: new_header_path,
                        });
                    }
                }
            }
        }
        proposed_extractions
    }

    fn assemble_plan(
        &self,
        candidate: &HeaderCandidate,
        proposed_extractions: Vec<ProposedExtraction>,
        plan_number: usize,
        frontier_analyzer: &FrontierAnalyzer,
        policies: &FilePolicyMaps,
        header_consumers: &HashMap<FileId, Vec<FileId>>,
    ) -> OptimizationPlan {
        let optimization_frontier = frontier_analyzer.find_frontiers(
            candidate.file_id,
            candidate.collateral_source_id,
            self.include_graph,
            &policies.by_id,
            &policies.ownership_by_id,
        );

        let validation_required_tus = header_consumers
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

/// Turn a symbol name into a filesystem-safe header stem. Closure-merged
/// models surface qualified/template names (`std::vector`, `Thing<T>`) that
/// are invalid in file names on Windows.
fn sanitize_header_stem(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut pending_underscore = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            if pending_underscore {
                out.push('_');
                pending_underscore = false;
            }
            out.push(ch);
        } else {
            pending_underscore = true;
        }
    }
    let stem = out.trim_matches('_').to_string();
    if stem.is_empty() {
        "symbol".to_string()
    } else {
        stem
    }
}

#[cfg(test)]
#[path = "planner_characterization_tests.rs"]
mod planner_characterization;

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_graph::reduced::{ReducedSemanticModel, ReducedSymbolRecord};
    use ariadne_graph::IncludeGraph;
    use ariadne_index::types::SymbolKind;
    use ariadne_repo::{FileId, FileOwnership, FileRecord};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    #[test]
    fn test_planner_basic() {
        let mut include_graph = IncludeGraph::new();
        include_graph.register_file(FileId(1), PathBuf::from("start.h"));
        include_graph.register_file(FileId(2), PathBuf::from("col.h"));

        use ariadne_graph::types::{IncludeEdge, IncludeKind, SourceLocation};
        include_graph.add_edge(IncludeEdge {
            source: FileId(1),
            destination: Some(FileId(2)),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "col.h".into(),
            resolved_path: Some(PathBuf::from("col.h")),
        });

        let mut model = ReducedSemanticModel {
            repo_root: PathBuf::from("/"),
            symbols: BTreeMap::new(),
            edges: vec![],
            tu_usage: BTreeMap::new(),
            file_symbols: BTreeMap::new(),
        };

        let sym_a = SymbolId::new("A");
        model.symbols.insert(
            sym_a.clone(),
            ReducedSymbolRecord {
                id: sym_a.clone(),
                name: "A".to_string(),
                kind: SymbolKind::Class,
                defining_file: Some(FileId(1)),
                declarations: vec![],
                definition: None,
            },
        );
        model.file_symbols.insert(FileId(1), vec![sym_a.clone()]);

        let sym_b = SymbolId::new("B");
        model.symbols.insert(
            sym_b.clone(),
            ReducedSymbolRecord {
                id: sym_b.clone(),
                name: "B".to_string(),
                kind: SymbolKind::Class,
                defining_file: Some(FileId(2)),
                declarations: vec![],
                definition: None,
            },
        );
        model.file_symbols.insert(FileId(2), vec![sym_b.clone()]);

        let file_records = vec![
            FileRecord {
                id: FileId(1),
                path: PathBuf::from("start.h"),
                canonical_path: PathBuf::from("start.h"),
                kind: ariadne_repo::FileKind::Header,
                size: 0,
                content_hash: String::new(),
                ownership: FileOwnership::Repository,
                optimization_policy: OptimizationPolicy::Optimizable,
            },
            FileRecord {
                id: FileId(2),
                path: PathBuf::from("col.h"),
                canonical_path: PathBuf::from("col.h"),
                kind: ariadne_repo::FileKind::Header,
                size: 0,
                content_hash: String::new(),
                ownership: FileOwnership::ThirdParty,
                optimization_policy: OptimizationPolicy::Frozen,
            },
        ];
        let tu_file_ids = vec![];

        let planner = OptimizationPlanner::new(&include_graph, &model, &file_records, &tu_file_ids);
        let plans = planner.plan();

        assert_eq!(plans.len(), 1, "Should generate exactly one plan");
        assert_eq!(plans[0].id.0, "PLAN-001");
        assert_eq!(plans[0].target_file_id, FileId(1));
    }
}
