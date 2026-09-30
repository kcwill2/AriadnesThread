//! Isolation engine: grouping planned extractions into generated units
//! (plan §3.6). The engine owns grouping, inter-symbol dependency edges,
//! the unit-cycle check, and the disk write; the new-file *text* is built by
//! a language plugin through the neutral `SourceExtractor` hook.

use crate::{ExtractedSymbol, ExtractionResult, GeneratedUnit};
use ariadne_cluster::plan::OptimizationPlan;
use ariadne_graph::reduced::ReducedSemanticModel;
use ariadne_index::SymbolKind;
use ariadne_lang::{SourceExtractor, SymbolBody};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub struct IsolationEngine<'a> {
    model: &'a ReducedSemanticModel,
}

impl<'a> IsolationEngine<'a> {
    pub fn new(model: &'a ReducedSemanticModel) -> Self {
        Self { model }
    }

    pub fn extract(&self, plan: &OptimizationPlan) -> ExtractionResult {
        let mut groups: BTreeMap<PathBuf, Vec<&ariadne_cluster::plan::ProposedExtraction>> =
            BTreeMap::new();
        let mut symbol_to_unit: BTreeMap<ariadne_index::types::SymbolId, PathBuf> =
            BTreeMap::new();

        for ext in &plan.proposed_extractions {
            groups
                .entry(ext.target_header.clone())
                .or_default()
                .push(ext);
            symbol_to_unit.insert(ext.symbol_id.clone(), ext.target_header.clone());
        }

        let mut units = Vec::new();

        for (target_path, extractions) in groups {
            units.push(self.build_generated_unit(
                target_path,
                extractions,
                &symbol_to_unit,
            ));
        }

        let mut errors = vec![];

        if Self::detect_cycles_in_units(&units) {
            errors.push("Cycle detected in generated headers".to_string());
        }

        ExtractionResult {
            plan_id: plan.id.0.clone(),
            headers: units,
            errors,
        }
    }

    fn build_generated_unit(
        &self,
        target_path: PathBuf,
        extractions: Vec<&ariadne_cluster::plan::ProposedExtraction>,
        symbol_to_unit: &BTreeMap<ariadne_index::types::SymbolId, PathBuf>,
    ) -> GeneratedUnit {
        let mut symbols = Vec::new();
        let mut required_deps = BTreeSet::new();

        for ext in extractions {
            let kind = self
                .model
                .find_symbol(&ext.symbol_id.0)
                .map(|r| r.kind)
                .unwrap_or(SymbolKind::Other);

            let body = format!("// Symbol: {} ({:?})", ext.symbol_name, kind);

            symbols.push(ExtractedSymbol {
                name: ext.symbol_name.clone(),
                kind,
                body,
            });

            for edge in self.model.get_dependencies(&ext.symbol_id) {
                if let Some(dep_unit) = symbol_to_unit.get(&edge.dst_symbol) {
                    if dep_unit != &target_path {
                        required_deps.insert(dep_unit.clone());
                    }
                }
            }
        }

        GeneratedUnit {
            path: target_path,
            symbols,
            required_deps: required_deps.into_iter().collect(),
        }
    }

    fn detect_cycles_in_units(units: &[GeneratedUnit]) -> bool {
        let mut adj: BTreeMap<PathBuf, BTreeSet<PathBuf>> = BTreeMap::new();
        for u in units {
            adj.insert(
                u.path.clone(),
                u.required_deps.iter().cloned().collect(),
            );
        }

        fn has_cycle(
            node: &PathBuf,
            adj: &BTreeMap<PathBuf, BTreeSet<PathBuf>>,
            visited: &mut BTreeSet<PathBuf>,
            stack: &mut BTreeSet<PathBuf>,
        ) -> bool {
            visited.insert(node.clone());
            stack.insert(node.clone());

            if let Some(neighbors) = adj.get(node) {
                for n in neighbors {
                    if !visited.contains(n) {
                        if has_cycle(n, adj, visited, stack) {
                            return true;
                        }
                    } else if stack.contains(n) {
                        return true;
                    }
                }
            }
            stack.remove(node);
            false
        }

        let mut visited = BTreeSet::new();
        let mut stack = BTreeSet::new();
        let mut cycle_found = false;

        for u in units {
            if !visited.contains(&u.path) {
                if has_cycle(&u.path, &adj, &mut visited, &mut stack) {
                    cycle_found = true;
                    break;
                }
            }
        }

        cycle_found
    }
}

impl ExtractionResult {
    pub fn write_to_disk(
        &self,
        output_dir: &std::path::Path,
        extractor: &dyn SourceExtractor,
    ) -> anyhow::Result<Vec<PathBuf>> {
        let mut written_paths = Vec::new();
        std::fs::create_dir_all(output_dir)?;

        for unit in &self.headers {
            written_paths.push(write_unit(output_dir, unit, extractor)?);
        }

        Ok(written_paths)
    }
}

fn write_unit(
    output_dir: &std::path::Path,
    unit: &GeneratedUnit,
    extractor: &dyn SourceExtractor,
) -> anyhow::Result<PathBuf> {
    let bodies: Vec<SymbolBody> = unit
        .symbols
        .iter()
        .map(|sym| SymbolBody {
            name: sym.name.clone(),
            body: sym.body.clone(),
        })
        .collect();
    let deps: Vec<String> = unit
        .required_deps
        .iter()
        .map(|dep| dep.display().to_string())
        .collect();

    let generated = extractor.extract(&bodies, "", &deps);
    let file_path = output_dir.join(&unit.path);
    std::fs::write(&file_path, generated.content)?;
    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_unit_fields() {
        let sym = ExtractedSymbol {
            name: "MyClass".to_string(),
            kind: ariadne_index::SymbolKind::Class,
            body: "// body".to_string(),
        };
        let unit = GeneratedUnit {
            path: std::path::PathBuf::from("my_class.h"),
            symbols: vec![sym],
            required_deps: vec![std::path::PathBuf::from("other.h")],
        };
        assert_eq!(unit.path.to_str().unwrap(), "my_class.h");
        assert_eq!(unit.symbols[0].name, "MyClass");
        assert_eq!(unit.required_deps.len(), 1);
    }

    fn dummy_model() -> ReducedSemanticModel {
        ReducedSemanticModel {
            repo_root: std::path::PathBuf::from("/"),
            symbols: std::collections::BTreeMap::new(),
            edges: vec![],
            tu_usage: std::collections::BTreeMap::new(),
            file_symbols: std::collections::BTreeMap::new(),
        }
    }

    #[test]
    fn test_extract_basic() {
        use ariadne_cluster::plan::{PlanId, ProposedExtraction, RiskLevel};
        use ariadne_index::types::SymbolId;
        use ariadne_repo::FileId;

        let plan = OptimizationPlan {
            id: PlanId("PLAN-001".to_string()),
            target_file: std::path::PathBuf::from("thing.h"),
            target_file_id: FileId::from_relative_path("thing.h"),
            problem: "problem".to_string(),
            collateral_load_before: 10,
            collateral_load_after: 5,
            proposed_extractions: vec![
                ProposedExtraction {
                    symbol_name: "A".to_string(),
                    symbol_id: SymbolId::new("A"),
                    target_header: std::path::PathBuf::from("a.h"),
                },
                ProposedExtraction {
                    symbol_name: "B".to_string(),
                    symbol_id: SymbolId::new("B"),
                    target_header: std::path::PathBuf::from("b.h"),
                },
            ],
            optimization_frontier: vec![],
            risk: RiskLevel::Conservative,
            validation_required_tus: vec![],
            rationale: "rat".to_string(),
        };

        let model = dummy_model();
        let extractor = IsolationEngine::new(&model);
        let result = extractor.extract(&plan);

        assert_eq!(result.plan_id, "PLAN-001");
        assert_eq!(result.headers.len(), 2);

        let a_unit = result
            .headers
            .iter()
            .find(|u| u.path == std::path::PathBuf::from("a.h"))
            .unwrap();
        assert_eq!(a_unit.symbols.len(), 1);
        assert_eq!(a_unit.symbols[0].name, "A");
        assert_eq!(a_unit.symbols[0].body, "// Symbol: A (Other)");
    }

    #[test]
    fn test_no_symbol_lost_invariant() {
        use ariadne_cluster::plan::{PlanId, ProposedExtraction, RiskLevel};
        use ariadne_index::types::SymbolId;
        use ariadne_repo::FileId;

        let plan = OptimizationPlan {
            id: PlanId("PLAN-002".to_string()),
            target_file: std::path::PathBuf::from("thing.h"),
            target_file_id: FileId::from_relative_path("thing.h"),
            problem: "problem".to_string(),
            collateral_load_before: 10,
            collateral_load_after: 5,
            proposed_extractions: vec![
                ProposedExtraction {
                    symbol_name: "X".to_string(),
                    symbol_id: SymbolId::new("X"),
                    target_header: std::path::PathBuf::from("x.h"),
                },
                ProposedExtraction {
                    symbol_name: "Y".to_string(),
                    symbol_id: SymbolId::new("Y"),
                    target_header: std::path::PathBuf::from("x.h"),
                },
            ],
            optimization_frontier: vec![],
            risk: RiskLevel::Conservative,
            validation_required_tus: vec![],
            rationale: "rat".to_string(),
        };

        let model = dummy_model();
        let extractor = IsolationEngine::new(&model);
        let result = extractor.extract(&plan);

        // Invariant: every SymbolId from plan appears exactly once across all headers
        let mut extracted_names = vec![];
        for u in &result.headers {
            for sym in &u.symbols {
                extracted_names.push(sym.name.clone());
            }
        }
        extracted_names.sort();
        assert_eq!(extracted_names, vec!["X", "Y"]);
    }

    #[test]
    fn test_acyclic_includes() {
        use ariadne_cluster::plan::{PlanId, ProposedExtraction, RiskLevel};
        use ariadne_graph::reduced::ReducedSemanticEdge;
        use ariadne_index::types::{DependencyKind, SymbolId};
        use ariadne_repo::FileId;

        let plan = OptimizationPlan {
            id: PlanId("PLAN-003".to_string()),
            target_file: std::path::PathBuf::from("thing.h"),
            target_file_id: FileId::from_relative_path("thing.h"),
            problem: "cycle".to_string(),
            collateral_load_before: 10,
            collateral_load_after: 5,
            proposed_extractions: vec![
                ProposedExtraction {
                    symbol_name: "A".to_string(),
                    symbol_id: SymbolId::new("A"),
                    target_header: std::path::PathBuf::from("a.h"),
                },
                ProposedExtraction {
                    symbol_name: "B".to_string(),
                    symbol_id: SymbolId::new("B"),
                    target_header: std::path::PathBuf::from("b.h"),
                },
            ],
            optimization_frontier: vec![],
            risk: RiskLevel::Conservative,
            validation_required_tus: vec![],
            rationale: "rat".to_string(),
        };

        // Create a model with a cycle A -> B and B -> A
        let mut model = dummy_model();
        model.edges.push(ReducedSemanticEdge {
            src_symbol: SymbolId::new("A"),
            dst_symbol: SymbolId::new("B"),
            kind: DependencyKind::Complete,
        });
        model.edges.push(ReducedSemanticEdge {
            src_symbol: SymbolId::new("B"),
            dst_symbol: SymbolId::new("A"),
            kind: DependencyKind::Complete,
        });

        let extractor = IsolationEngine::new(&model);
        let result = extractor.extract(&plan);

        // Check if there is an error
        let has_errors = !result.errors.is_empty();

        // Or if graph is acyclic
        let mut a_reqs_b = false;
        let mut b_reqs_a = false;

        for u in &result.headers {
            if u.path == std::path::PathBuf::from("a.h") {
                if u.required_deps
                    .contains(&std::path::PathBuf::from("b.h"))
                {
                    a_reqs_b = true;
                }
            } else if u.path == std::path::PathBuf::from("b.h") {
                if u.required_deps
                    .contains(&std::path::PathBuf::from("a.h"))
                {
                    b_reqs_a = true;
                }
            }
        }

        let has_cycle = a_reqs_b && b_reqs_a;
        assert!(
            has_errors || !has_cycle,
            "Extractor must either emit an error or resolve the cycle"
        );
    }

    #[test]
    fn test_write_to_disk() {
        use ariadne_lang_cpp::CppSourceExtractor;

        let sym1 = ExtractedSymbol {
            name: "MyClass".to_string(),
            kind: ariadne_index::SymbolKind::Class,
            body: "// body1".to_string(),
        };
        let unit1 = GeneratedUnit {
            path: std::path::PathBuf::from("h1.h"),
            symbols: vec![sym1],
            required_deps: vec![std::path::PathBuf::from("other.h")],
        };

        let sym2 = ExtractedSymbol {
            name: "MyFunc".to_string(),
            kind: ariadne_index::SymbolKind::Function,
            body: "// body2".to_string(),
        };
        let unit2 = GeneratedUnit {
            path: std::path::PathBuf::from("h2.h"),
            symbols: vec![sym2],
            required_deps: vec![],
        };

        let result = ExtractionResult {
            plan_id: "PLAN".to_string(),
            headers: vec![unit1, unit2],
            errors: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        let written = result
            .write_to_disk(dir.path(), &CppSourceExtractor)
            .unwrap();
        assert_eq!(written.len(), 2);

        let h1_path = dir.path().join("h1.h");
        let h2_path = dir.path().join("h2.h");

        assert!(h1_path.exists());
        assert!(h2_path.exists());

        let h1_content = std::fs::read_to_string(h1_path).unwrap();
        assert!(h1_content.contains("#pragma once"));
        assert!(h1_content.contains("#include \"other.h\""));
        assert!(h1_content.contains("// body1"));

        let h2_content = std::fs::read_to_string(h2_path).unwrap();
        assert!(h2_content.contains("#pragma once"));
        assert!(h2_content.contains("// body2"));
        assert!(!h2_content.contains("#include"));
    }

    #[test]
    fn test_write_to_disk_byte_parity_full_content() {
        use ariadne_lang_cpp::CppSourceExtractor;

        let unit = GeneratedUnit {
            path: std::path::PathBuf::from("h.h"),
            symbols: vec![
                ExtractedSymbol {
                    name: "A".to_string(),
                    kind: ariadne_index::SymbolKind::Class,
                    body: "// body1".to_string(),
                },
                ExtractedSymbol {
                    name: "B".to_string(),
                    kind: ariadne_index::SymbolKind::Function,
                    body: "// body2".to_string(),
                },
            ],
            required_deps: vec![std::path::PathBuf::from("a.h"), std::path::PathBuf::from("b.h")],
        };
        let result = ExtractionResult {
            plan_id: "PLAN".to_string(),
            headers: vec![unit],
            errors: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        result.write_to_disk(dir.path(), &CppSourceExtractor).unwrap();

        let content = std::fs::read_to_string(dir.path().join("h.h")).unwrap();
        assert_eq!(
            content,
            "#pragma once\n\n#include \"a.h\"\n#include \"b.h\"\n\n// body1\n// body2\n"
        );
    }
}
