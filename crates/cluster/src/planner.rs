use ariadne_graph::IncludeGraph;
use ariadne_graph::reduced::ReducedSemanticModel;
use ariadne_repo::{FileId, FileRecord, OptimizationPolicy};
use ariadne_graph::load::LoadAnalyzer;
use crate::plan::{OptimizationPlan, PlanRejection, PlanId, ProposedExtraction, RiskLevel};
use crate::frontier::FrontierAnalyzer;
use crate::checker::ConstraintChecker;
use crate::scc;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use ariadne_index::types::SymbolId;

pub struct OptimizationPlanner<'a> {
    pub include_graph: &'a IncludeGraph,
    pub model: &'a ReducedSemanticModel,
    pub file_records: &'a [FileRecord],
    pub tu_file_ids: &'a [FileId],
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
        let mut plans = Vec::new();
        let load_analyzer = LoadAnalyzer::new(self.include_graph, self.model);
        let frontier_analyzer = FrontierAnalyzer::new();
        
        let mut file_policies = HashMap::new();
        let mut file_ownerships = HashMap::new();
        let mut path_to_policy = HashMap::new();
        for record in self.file_records {
            let path = self.include_graph.file_paths.get(&record.id).cloned().unwrap_or_default();
            file_policies.insert(record.id, record.optimization_policy);
            file_ownerships.insert(record.id, record.ownership.clone());
            if !path.as_os_str().is_empty() {
                path_to_policy.insert(path, record.optimization_policy);
            }
        }
        
        let checker = ConstraintChecker::new(path_to_policy, HashSet::new());
        let mut plan_id_counter = 1;
        let mut sorted_headers: Vec<FileId> = self.include_graph.file_paths.keys().copied().collect();
        sorted_headers.sort();

        let header_consumers = self.include_graph.map_header_consuming_tus(self.tu_file_ids);

        for file_id in sorted_headers {
            let path = self.include_graph.file_paths.get(&file_id).unwrap();
            let metric_res = load_analyzer.analyze_file_load(path);
            if metric_res.is_err() {
                continue;
            }
            let metric = metric_res.unwrap();
            
            if metric.collateral_load == 0 {
                continue;
            }
            let lcs = match &metric.largest_collateral_source {
                Some(src) => src,
                None => continue,
            };

            let policy = file_policies.get(&file_id).copied().unwrap_or(OptimizationPolicy::Optimizable);
            if policy != OptimizationPolicy::Optimizable {
                continue;
            }

            let file_syms = self.model.file_symbols.get(&file_id).cloned().unwrap_or_default();
            let mapped_edges: Vec<ariadne_index::types::SemanticEdge> = self.model.edges.iter().map(|e| {
                ariadne_index::types::SemanticEdge {
                    src_symbol: e.src_symbol.clone(),
                    dst_symbol: e.dst_symbol.clone(),
                    kind: e.kind,
                    src_file: FileId(0),
                    dst_file: FileId(0),
                }
            }).collect();
            let sccs = scc::compute_symbol_sccs(&file_syms, &mapped_edges);

            let mut proposed_extractions = Vec::new();
            
            for scc in &sccs {
                for sym_id in &scc.symbols {
                    if let Some(record) = self.model.symbols.get(sym_id) {
                        let new_header_path = path.with_file_name(format!("{}_extracted.h", record.name));
                        if checker.validate_extraction(sym_id, &new_header_path, scc.symbols.len() > 1, false, None).is_ok() {
                            proposed_extractions.push(ProposedExtraction {
                                symbol_name: record.name.clone(),
                                symbol_id: sym_id.clone(),
                                target_header: new_header_path,
                            });
                        }
                    }
                }
            }
            
            if proposed_extractions.is_empty() {
                continue;
            }

            let optimization_frontier = frontier_analyzer.find_frontiers(
                file_id, 
                lcs.file_id, 
                self.include_graph, 
                &file_policies, 
                &file_ownerships
            );

            let plan_id = format!("PLAN-{:03}", plan_id_counter);
            plan_id_counter += 1;

            let validation_required_tus = header_consumers.get(&file_id)
                .map(|tus| tus.iter().filter_map(|id| self.include_graph.file_paths.get(id).cloned()).collect())
                .unwrap_or_default();

            let rationale = format!(
                "File {} has {} collateral symbols mostly from {}. Extracting independent symbols.",
                path.display(),
                metric.collateral_load,
                lcs.path.display()
            );

            plans.push(OptimizationPlan {
                id: PlanId(plan_id),
                target_file: path.clone(),
                target_file_id: file_id,
                problem: format!("{} symbols of collateral load", metric.collateral_load),
                collateral_load_before: metric.collateral_load,
                collateral_load_after: 0,
                proposed_extractions,
                optimization_frontier,
                risk: RiskLevel::Conservative,
                validation_required_tus,
                rationale,
            });
        }

        plans
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
        checker.validate_extraction(symbol, target_file, splits_scc, creates_include_cycle, unresolved_dep)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_graph::IncludeGraph;
    use ariadne_graph::reduced::{ReducedSemanticModel, ReducedSymbolRecord};
    use ariadne_repo::{FileId, FileRecord, FileOwnership};
    use ariadne_index::types::{SymbolKind};
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
        model.symbols.insert(sym_a.clone(), ReducedSymbolRecord {
            id: sym_a.clone(),
            name: "A".to_string(),
            kind: SymbolKind::Class,
            defining_file: Some(FileId(1)),
            declarations: vec![],
            definition: None,
        });
        model.file_symbols.insert(FileId(1), vec![sym_a.clone()]);
        
        let sym_b = SymbolId::new("B");
        model.symbols.insert(sym_b.clone(), ReducedSymbolRecord {
            id: sym_b.clone(),
            name: "B".to_string(),
            kind: SymbolKind::Class,
            defining_file: Some(FileId(2)),
            declarations: vec![],
            definition: None,
        });
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
            }
        ];
        let tu_file_ids = vec![];

        let planner = OptimizationPlanner::new(&include_graph, &model, &file_records, &tu_file_ids);
        let plans = planner.plan();
        
        assert_eq!(plans.len(), 1, "Should generate exactly one plan");
        assert_eq!(plans[0].id.0, "PLAN-001");
        assert_eq!(plans[0].target_file_id, FileId(1));
    }
}
