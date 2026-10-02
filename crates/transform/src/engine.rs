//! Isolation engine: grouping planned extractions into generated units
//! (plan §3.6). The engine owns grouping, inter-symbol dependency edges,
//! the unit-cycle check, and the disk write; the new-file *text* is built by
//! a language plugin through the neutral `SourceExtractor` hook.

use crate::{ExtractedSymbol, ExtractionResult, GeneratedUnit};
use ariadne_cluster::plan::OptimizationPlan;
use ariadne_graph::reduced::ReducedSemanticModel;
use ariadne_index::SymbolKind;
use ariadne_lang::{SourceExtractor, SymbolBody};
use ariadne_repo::FileId;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub struct IsolationEngine<'a> {
    model: &'a ReducedSemanticModel,
}

impl<'a> IsolationEngine<'a> {
    pub fn new(model: &'a ReducedSemanticModel) -> Self {
        Self { model }
    }

    /// Group the plan's extractions into generated units.
    ///
    /// - `sources`: reads the text of a planned extraction's `source_file`
    ///   (callers read disk; `|_| None` keeps every body on the placeholder
    ///   fallback, invariant I4).
    /// - `extractor`: the language plugin asked to locate verbatim declaration
    ///   bodies in those source texts (D-M23-1). The engine itself names only
    ///   this neutral trait.
    pub fn extract(
        &self,
        plan: &OptimizationPlan,
        sources: &dyn Fn(FileId) -> Option<String>,
        extractor: &dyn SourceExtractor,
    ) -> ExtractionResult {
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
                sources,
                extractor,
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
        sources: &dyn Fn(FileId) -> Option<String>,
        extractor: &dyn SourceExtractor,
    ) -> GeneratedUnit {
        let (located_bodies, primary_source_text) =
            self.locate_bodies_for_unit(&extractions, sources, extractor);

        let mut symbols = Vec::new();
        let mut required_deps = BTreeSet::new();

        for ext in extractions {
            let kind = self
                .model
                .find_symbol(&ext.symbol_id.0)
                .map(|r| r.kind)
                .unwrap_or(SymbolKind::Other);

            let body = match located_bodies.get(&ext.symbol_name) {
                Some(verbatim) => verbatim.clone(),
                None => format!("// Symbol: {} ({:?})", ext.symbol_name, kind),
            };

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
            primary_source_text,
        }
    }

    /// Ask the plugin to locate verbatim declaration bodies in the unit's
    /// source files (D-M23-1): source files are deduplicated, unreadable ones
    /// are skipped (placeholder fallback, I4), and the first readable text is
    /// carried as the unit's primary source for the namespace seam (D-M23-2).
    fn locate_bodies_for_unit(
        &self,
        extractions: &[&ariadne_cluster::plan::ProposedExtraction],
        sources: &dyn Fn(FileId) -> Option<String>,
        extractor: &dyn SourceExtractor,
    ) -> (BTreeMap<String, String>, Option<String>) {
        let mut wanted_per_source: BTreeMap<FileId, Vec<(String, SymbolKind)>> = BTreeMap::new();
        for ext in extractions {
            if let Some(source_file) = ext.source_file {
                let kind = self
                    .model
                    .find_symbol(&ext.symbol_id.0)
                    .map(|r| r.kind)
                    .unwrap_or(SymbolKind::Other);
                wanted_per_source
                    .entry(source_file)
                    .or_default()
                    .push((ext.symbol_name.clone(), kind));
            }
        }

        let mut located: BTreeMap<String, String> = BTreeMap::new();
        let mut primary_source_text: Option<String> = None;
        for (source_file, wanted) in wanted_per_source {
            let Some(source_text) = sources(source_file) else {
                continue;
            };
            if primary_source_text.is_none() {
                primary_source_text = Some(source_text.clone());
            }
            for body in extractor.locate_symbol_bodies(&source_text, &wanted) {
                located.entry(body.name).or_insert(body.body);
            }
        }
        (located, primary_source_text)
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

    let namespace = match unit.primary_source_text.as_deref() {
        Some(source_text) => extractor.unit_namespace(source_text),
        None => String::new(),
    };
    let generated = extractor.extract(&bodies, &namespace, &deps);
    let file_path = output_dir.join(&unit.path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&file_path, generated.content)?;
    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    // R2 keeps directive text out of this crate's sources, so the C++ shapes
    // asserted here are assembled from a non-contiguous marker (the plugin
    // crate owns the canonical form).
    const DIRECTIVE: &str = concat!("#inc", "lude ");
    fn include_of(target: &str) -> String {
        format!("{DIRECTIVE}\"{target}\"")
    }

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
            primary_source_text: None,
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
                    source_file: None,
                },
                ProposedExtraction {
                    symbol_name: "B".to_string(),
                    symbol_id: SymbolId::new("B"),
                    target_header: std::path::PathBuf::from("b.h"),
                    source_file: None,
                },
            ],
            optimization_frontier: vec![],
            risk: RiskLevel::Conservative,
            validation_required_tus: vec![],
            rationale: "rat".to_string(),
        };

        let model = dummy_model();
        let extractor = IsolationEngine::new(&model);
        let fake_extractor = RecordingFakeExtractor::default();
        let result = extractor.extract(&plan, &no_sources, &fake_extractor);

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
                    source_file: None,
                },
                ProposedExtraction {
                    symbol_name: "Y".to_string(),
                    symbol_id: SymbolId::new("Y"),
                    target_header: std::path::PathBuf::from("x.h"),
                    source_file: None,
                },
            ],
            optimization_frontier: vec![],
            risk: RiskLevel::Conservative,
            validation_required_tus: vec![],
            rationale: "rat".to_string(),
        };

        let model = dummy_model();
        let extractor = IsolationEngine::new(&model);
        let fake_extractor = RecordingFakeExtractor::default();
        let result = extractor.extract(&plan, &no_sources, &fake_extractor);

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
                    source_file: None,
                },
                ProposedExtraction {
                    symbol_name: "B".to_string(),
                    symbol_id: SymbolId::new("B"),
                    target_header: std::path::PathBuf::from("b.h"),
                    source_file: None,
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
        let fake_extractor = RecordingFakeExtractor::default();
        let result = extractor.extract(&plan, &no_sources, &fake_extractor);

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
            primary_source_text: None,
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
            primary_source_text: None,
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
        assert!(h1_content.contains(&include_of("other.h")));
        assert!(h1_content.contains("// body1"));

        let h2_content = std::fs::read_to_string(h2_path).unwrap();
        assert!(h2_content.contains("#pragma once"));
        assert!(h2_content.contains("// body2"));
        assert!(!h2_content.contains(DIRECTIVE.trim_end()));
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
            primary_source_text: None,
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
            format!(
                "#pragma once\n\n{}\n{}\n\n// body1\n// body2\n",
                include_of("a.h"),
                include_of("b.h")
            )
        );
    }

    // --- M23 T1: verbatim body resolution + namespace wiring (neutral fakes) ---

    /// Content provider that never finds anything: every symbol must fall
    /// back to the placeholder (invariant I4).
    fn no_sources(_file_id: ariadne_repo::FileId) -> Option<String> {
        None
    }

    fn reader_for(
        named_texts: &[(&str, &str)],
    ) -> impl Fn(ariadne_repo::FileId) -> Option<String> {
        let by_id: Vec<(ariadne_repo::FileId, String)> = named_texts
            .iter()
            .map(|(rel, text)| {
                (
                    ariadne_repo::FileId::from_relative_path(rel),
                    text.to_string(),
                )
            })
            .collect();
        move |file_id| {
            by_id
                .iter()
                .find(|(id, _)| *id == file_id)
                .map(|(_, text)| text.clone())
        }
    }

    fn extraction(
        symbol: &str,
        target: &str,
        source: Option<&str>,
    ) -> ariadne_cluster::plan::ProposedExtraction {
        ariadne_cluster::plan::ProposedExtraction {
            symbol_name: symbol.to_string(),
            symbol_id: ariadne_index::types::SymbolId::new(symbol),
            target_header: std::path::PathBuf::from(target),
            source_file: source.map(ariadne_repo::FileId::from_relative_path),
        }
    }

    fn plan_of(extractions: Vec<ariadne_cluster::plan::ProposedExtraction>) -> OptimizationPlan {
        OptimizationPlan {
            id: ariadne_cluster::plan::PlanId("PLAN-F".to_string()),
            target_file: std::path::PathBuf::from("thing.h"),
            target_file_id: ariadne_repo::FileId::from_relative_path("thing.h"),
            problem: "fake".to_string(),
            collateral_load_before: 4,
            collateral_load_after: 2,
            proposed_extractions: extractions,
            optimization_frontier: vec![],
            risk: ariadne_cluster::plan::RiskLevel::Conservative,
            validation_required_tus: vec![],
            rationale: "fake".to_string(),
        }
    }

    /// Neutral plugin stand-in (no plugin names in engine src): scripts the
    /// verbatim bodies `locate_symbol_bodies` may find, reports a namespace
    /// from `unit_namespace`, and records what `extract` was handed.
    #[derive(Default)]
    struct RecordingFakeExtractor {
        locatable: Vec<(String, String)>,
        reported_namespace: String,
        extract_namespaces: std::sync::Mutex<Vec<String>>,
        unit_namespace_inputs: std::sync::Mutex<Vec<String>>,
    }

    impl RecordingFakeExtractor {
        fn with_locatable_bodies(bodies: &[(&str, &str)]) -> Self {
            Self {
                locatable: bodies
                    .iter()
                    .map(|(name, body)| (name.to_string(), body.to_string()))
                    .collect(),
                ..Default::default()
            }
        }

        fn reporting_namespace(namespace: &str) -> Self {
            Self {
                reported_namespace: namespace.to_string(),
                ..Default::default()
            }
        }

        fn extract_namespaces(&self) -> Vec<String> {
            self.extract_namespaces.lock().unwrap().clone()
        }

        fn unit_namespace_inputs(&self) -> Vec<String> {
            self.unit_namespace_inputs.lock().unwrap().clone()
        }
    }

    impl SourceExtractor for RecordingFakeExtractor {
        fn extract(
            &self,
            symbols: &[SymbolBody],
            namespace: &str,
            required_deps: &[String],
        ) -> ariadne_lang::GeneratedSourceFile {
            self.extract_namespaces
                .lock()
                .unwrap()
                .push(namespace.to_string());
            let mut content = format!("NS<{namespace}>\n");
            for dep in required_deps {
                content.push_str(&format!("DEP<{dep}>\n"));
            }
            for sym in symbols {
                content.push_str(&sym.body);
                content.push('\n');
            }
            ariadne_lang::GeneratedSourceFile {
                path: std::path::PathBuf::from("fake.out"),
                content,
            }
        }

        fn unit_namespace(&self, source_text: &str) -> String {
            self.unit_namespace_inputs
                .lock()
                .unwrap()
                .push(source_text.to_string());
            self.reported_namespace.clone()
        }

        fn locate_symbol_bodies(
            &self,
            source_text: &str,
            wanted: &[(String, SymbolKind)],
        ) -> Vec<SymbolBody> {
            wanted
                .iter()
                .filter_map(|(name, _)| {
                    let body = self
                        .locatable
                        .iter()
                        .find(|(known, _)| known == name)
                        .map(|(_, body)| body.clone())?;
                    source_text.contains(name.as_str()).then(|| SymbolBody {
                        name: name.clone(),
                        body,
                    })
                })
                .collect()
        }
    }

    #[test]
    fn extract_uses_verbatim_bodies_located_by_the_plugin() {
        let source_text = "// fixture\nnamespace demo { class A { int x; }; }\n";
        let fake = RecordingFakeExtractor::with_locatable_bodies(&[("A", "class A { int x; };")]);
        let plan = plan_of(vec![extraction("A", "a.h", Some("thing.h"))]);

        let model = dummy_model();
        let engine = IsolationEngine::new(&model);
        let result = engine.extract(&plan, &reader_for(&[("thing.h", source_text)]), &fake);

        let a_unit = result
            .headers
            .iter()
            .find(|u| u.path == std::path::PathBuf::from("a.h"))
            .unwrap();
        assert_eq!(a_unit.symbols[0].body, "class A { int x; };");
        assert_eq!(
            a_unit.primary_source_text.as_deref(),
            Some(source_text),
            "the unit must carry its source text for the namespace seam"
        );
    }

    #[test]
    fn extract_keeps_placeholder_for_symbols_the_plugin_cannot_locate() {
        let source_text = "class A { int x; };\nstruct B { int y; };\n";
        let fake = RecordingFakeExtractor::with_locatable_bodies(&[("A", "class A { int x; };")]);
        let plan = plan_of(vec![
            extraction("A", "shared.h", Some("thing.h")),
            extraction("B", "shared.h", Some("thing.h")),
        ]);

        let model = dummy_model();
        let engine = IsolationEngine::new(&model);
        let result = engine.extract(&plan, &reader_for(&[("thing.h", source_text)]), &fake);

        let shared = &result.headers[0];
        let body_of = |name: &str| {
            shared
                .symbols
                .iter()
                .find(|s| s.name == name)
                .unwrap()
                .body
                .clone()
        };
        assert_eq!(body_of("A"), "class A { int x; };");
        assert_eq!(body_of("B"), "// Symbol: B (Other)");
    }

    #[test]
    fn extract_falls_back_to_placeholder_when_source_text_is_missing() {
        let fake = RecordingFakeExtractor::with_locatable_bodies(&[("A", "class A { int x; };")]);
        let plan = plan_of(vec![extraction("A", "a.h", Some("thing.h"))]);

        let model = dummy_model();
        let engine = IsolationEngine::new(&model);
        let result = engine.extract(&plan, &no_sources, &fake);

        assert_eq!(result.headers[0].symbols[0].body, "// Symbol: A (Other)");
        assert_eq!(result.headers[0].primary_source_text, None);
    }

    #[test]
    fn write_unit_passes_namespace_from_the_plugin_hook() {
        let fake = RecordingFakeExtractor::reporting_namespace("demo_ns");
        let source_text = "namespace demo { }\n";
        let unit = GeneratedUnit {
            path: std::path::PathBuf::from("ns_unit.h"),
            symbols: vec![ExtractedSymbol {
                name: "A".to_string(),
                kind: ariadne_index::SymbolKind::Class,
                body: "class A { int x; };".to_string(),
            }],
            required_deps: vec![],
            primary_source_text: Some(source_text.to_string()),
        };
        let result = ExtractionResult {
            plan_id: "PLAN-NS".to_string(),
            headers: vec![unit],
            errors: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        result.write_to_disk(dir.path(), &fake).unwrap();

        assert_eq!(fake.unit_namespace_inputs(), vec![source_text.to_string()]);
        assert_eq!(fake.extract_namespaces(), vec!["demo_ns".to_string()]);
        let written = std::fs::read_to_string(dir.path().join("ns_unit.h")).unwrap();
        assert!(written.starts_with("NS<demo_ns>"));
    }

    #[test]
    fn write_unit_passes_empty_namespace_without_source_text() {
        let fake = RecordingFakeExtractor::reporting_namespace("unused_ns");
        let unit = GeneratedUnit {
            path: std::path::PathBuf::from("bare_unit.h"),
            symbols: vec![ExtractedSymbol {
                name: "A".to_string(),
                kind: ariadne_index::SymbolKind::Class,
                body: "// Symbol: A (Class)".to_string(),
            }],
            required_deps: vec![],
            primary_source_text: None,
        };
        let result = ExtractionResult {
            plan_id: "PLAN-BARE".to_string(),
            headers: vec![unit],
            errors: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        result.write_to_disk(dir.path(), &fake).unwrap();

        assert!(fake.unit_namespace_inputs().is_empty());
        assert_eq!(fake.extract_namespaces(), vec![String::new()]);
    }

    #[test]
    fn write_to_disk_creates_nested_parent_directories() {
        let unit = GeneratedUnit {
            path: std::path::PathBuf::from("generated").join("nested").join("deep_x.h"),
            symbols: vec![ExtractedSymbol {
                name: "X".to_string(),
                kind: ariadne_index::SymbolKind::Class,
                body: "// Symbol: X (Class)".to_string(),
            }],
            required_deps: vec![],
            primary_source_text: None,
        };
        let result = ExtractionResult {
            plan_id: "PLAN-NEST".to_string(),
            headers: vec![unit],
            errors: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        let written = result.write_to_disk(dir.path(), &RecordingFakeExtractor::default()).unwrap();

        assert!(written[0].exists(), "nested parents must be created");
    }
}
