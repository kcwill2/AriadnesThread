use crate::checker::ConstraintChecker;
use crate::frontier::FrontierAnalyzer;
use crate::plan::{OptimizationPlan, PlanId, PlanRejection, ProposedExtraction, RiskLevel};
use crate::scc;
use ariadne_graph::load::LoadAnalyzer;
use ariadne_graph::reduced::ReducedSemanticModel;
use ariadne_graph::IncludeGraph;
use ariadne_index::types::SymbolId;
use ariadne_repo::{FileId, FileOwnership, FileRecord, OptimizationPolicy};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn is_header_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext, "h" | "hpp" | "hxx" | "hh" | "inl"))
        .unwrap_or(false)
}

pub struct OptimizationPlanner<'a> {
    pub include_graph: &'a IncludeGraph,
    pub model: &'a ReducedSemanticModel,
    pub file_records: &'a [FileRecord],
    pub tu_file_ids: &'a [FileId],
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

impl<'a> OptimizationPlanner<'a> {
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
        }
    }

    pub fn plan(&self) -> Vec<OptimizationPlan> {
        let load_analyzer = LoadAnalyzer::new(self.include_graph, self.model);
        let frontier_analyzer = FrontierAnalyzer::new();
        let policies = self.collect_file_policies();
        let checker = ConstraintChecker::new(policies.policy_by_path.clone(), HashSet::new());
        let mapped_edges = self.model_edges_without_file_endpoints();
        let header_consumers = self
            .include_graph
            .map_header_consuming_tus(self.tu_file_ids);

        let mut plans = Vec::new();
        for file_id in self.sorted_header_ids() {
            let Some(candidate) = self.candidate_header(file_id, &load_analyzer, &policies) else {
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
                &header_consumers,
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
        load_analyzer: &LoadAnalyzer,
        policies: &FilePolicyMaps,
    ) -> Option<HeaderCandidate> {
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

        Some(HeaderCandidate {
            file_id,
            path,
            collateral_load: metric.collateral_load,
            collateral_source_id: largest_collateral_source.file_id,
            collateral_source_path: largest_collateral_source.path.clone(),
        })
    }

    /// Semantic edges with their file endpoints erased, as consumed by the
    /// per-header SCC computation.
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

    fn propose_extractions(
        &self,
        candidate: &HeaderCandidate,
        mapped_edges: &[ariadne_index::types::SemanticEdge],
        checker: &ConstraintChecker,
    ) -> Vec<ProposedExtraction> {
        let file_syms = self
            .model
            .file_symbols
            .get(&candidate.file_id)
            .cloned()
            .unwrap_or_default();
        let sccs = scc::compute_symbol_sccs(&file_syms, mapped_edges);

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
            language: ariadne_repo::SourceLanguage::Cpp,
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
                language: ariadne_repo::SourceLanguage::Cpp,
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
                language: ariadne_repo::SourceLanguage::Cpp,
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
