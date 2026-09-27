//! Header extraction: building generated headers from optimization plans.

use crate::{ExtractedSymbol, ExtractionResult, GeneratedHeader};
use ariadne_cluster::plan::OptimizationPlan;
use ariadne_graph::reduced::ReducedSemanticModel;
use ariadne_index::SymbolKind;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub struct HeaderExtractor<'a> {
    model: &'a ReducedSemanticModel,
}

impl<'a> HeaderExtractor<'a> {
    pub fn new(model: &'a ReducedSemanticModel) -> Self {
        Self { model }
    }

    pub fn extract(&self, plan: &OptimizationPlan) -> ExtractionResult {
        let mut groups: BTreeMap<PathBuf, Vec<&ariadne_cluster::plan::ProposedExtraction>> =
            BTreeMap::new();
        let mut symbol_to_header: BTreeMap<ariadne_index::types::SymbolId, PathBuf> =
            BTreeMap::new();

        for ext in &plan.proposed_extractions {
            groups
                .entry(ext.target_header.clone())
                .or_default()
                .push(ext);
            symbol_to_header.insert(ext.symbol_id.clone(), ext.target_header.clone());
        }

        let mut headers = Vec::new();

        for (target_header, extractions) in groups {
            headers.push(self.build_generated_header(
                target_header,
                extractions,
                &symbol_to_header,
            ));
        }

        let mut errors = vec![];

        if Self::detect_cycle_in_headers(&headers) {
            errors.push("Cycle detected in generated headers".to_string());
        }

        ExtractionResult {
            plan_id: plan.id.0.clone(),
            headers,
            errors,
        }
    }

    fn build_generated_header(
        &self,
        target_header: PathBuf,
        extractions: Vec<&ariadne_cluster::plan::ProposedExtraction>,
        symbol_to_header: &BTreeMap<ariadne_index::types::SymbolId, PathBuf>,
    ) -> GeneratedHeader {
        let mut symbols = Vec::new();
        let mut required_includes = BTreeSet::new();

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
                forward_declarable: false,
                body,
            });

            for edge in self.model.get_dependencies(&ext.symbol_id) {
                if let Some(dep_header) = symbol_to_header.get(&edge.dst_symbol) {
                    if dep_header != &target_header {
                        required_includes.insert(dep_header.clone());
                    }
                }
            }
        }

        GeneratedHeader {
            filename: target_header,
            pragma_once: true,
            symbols,
            required_includes: required_includes.into_iter().collect(),
        }
    }

    fn detect_cycle_in_headers(headers: &[GeneratedHeader]) -> bool {
        let mut adj: BTreeMap<PathBuf, BTreeSet<PathBuf>> = BTreeMap::new();
        for h in headers {
            adj.insert(
                h.filename.clone(),
                h.required_includes.iter().cloned().collect(),
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

        for h in headers {
            if !visited.contains(&h.filename) {
                if has_cycle(&h.filename, &adj, &mut visited, &mut stack) {
                    cycle_found = true;
                    break;
                }
            }
        }

        cycle_found
    }
}

impl ExtractionResult {
    pub fn write_to_disk(&self, output_dir: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut written_paths = Vec::new();
        std::fs::create_dir_all(output_dir)?;

        for header in &self.headers {
            let file_path = output_dir.join(&header.filename);
            let mut content = String::new();

            if header.pragma_once {
                content.push_str("#pragma once\n\n");
            }

            for inc in &header.required_includes {
                content.push_str(&format!("#include \"{}\"\n", inc.display()));
            }

            if !header.required_includes.is_empty() {
                content.push('\n');
            }

            for sym in &header.symbols {
                content.push_str(&sym.body);
                content.push('\n');
            }

            std::fs::write(&file_path, content)?;
            written_paths.push(file_path);
        }

        Ok(written_paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_header_fields() {
        let sym = ExtractedSymbol {
            name: "MyClass".to_string(),
            kind: ariadne_index::SymbolKind::Class,
            forward_declarable: true,
            body: "// body".to_string(),
        };
        let header = GeneratedHeader {
            filename: std::path::PathBuf::from("my_class.h"),
            pragma_once: true,
            symbols: vec![sym],
            required_includes: vec![std::path::PathBuf::from("other.h")],
        };
        assert_eq!(header.filename.to_str().unwrap(), "my_class.h");
        assert!(header.pragma_once);
        assert_eq!(header.symbols[0].name, "MyClass");
        assert_eq!(header.required_includes.len(), 1);
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
        let extractor = HeaderExtractor::new(&model);
        let result = extractor.extract(&plan);

        assert_eq!(result.plan_id, "PLAN-001");
        assert_eq!(result.headers.len(), 2);

        let a_hdr = result
            .headers
            .iter()
            .find(|h| h.filename == std::path::PathBuf::from("a.h"))
            .unwrap();
        assert_eq!(a_hdr.symbols.len(), 1);
        assert_eq!(a_hdr.symbols[0].name, "A");
        assert_eq!(a_hdr.symbols[0].body, "// Symbol: A (Other)");
        assert!(a_hdr.pragma_once);
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
        let extractor = HeaderExtractor::new(&model);
        let result = extractor.extract(&plan);

        // Invariant: every SymbolId from plan appears exactly once across all headers
        let mut extracted_names = vec![];
        for h in &result.headers {
            for sym in &h.symbols {
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

        let extractor = HeaderExtractor::new(&model);
        let result = extractor.extract(&plan);

        // Check if there is an error
        let has_errors = !result.errors.is_empty();

        // Or if graph is acyclic
        let mut a_reqs_b = false;
        let mut b_reqs_a = false;

        for h in &result.headers {
            if h.filename == std::path::PathBuf::from("a.h") {
                if h.required_includes
                    .contains(&std::path::PathBuf::from("b.h"))
                {
                    a_reqs_b = true;
                }
            } else if h.filename == std::path::PathBuf::from("b.h") {
                if h.required_includes
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
        let sym1 = ExtractedSymbol {
            name: "MyClass".to_string(),
            kind: ariadne_index::SymbolKind::Class,
            forward_declarable: true,
            body: "// body1".to_string(),
        };
        let header1 = GeneratedHeader {
            filename: std::path::PathBuf::from("h1.h"),
            pragma_once: true,
            symbols: vec![sym1],
            required_includes: vec![std::path::PathBuf::from("other.h")],
        };

        let sym2 = ExtractedSymbol {
            name: "MyFunc".to_string(),
            kind: ariadne_index::SymbolKind::Function,
            forward_declarable: false,
            body: "// body2".to_string(),
        };
        let header2 = GeneratedHeader {
            filename: std::path::PathBuf::from("h2.h"),
            pragma_once: true,
            symbols: vec![sym2],
            required_includes: vec![],
        };

        let result = ExtractionResult {
            plan_id: "PLAN".to_string(),
            headers: vec![header1, header2],
            errors: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        let written = result.write_to_disk(dir.path()).unwrap();
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
}
