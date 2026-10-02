//! Orchestration + body-resolution characterization suite for the isolation
//! engine (the write/render tests live in `engine/write.rs`).

use super::*;
use crate::ExtractedSymbol;
use ariadne_lang::SymbolBody;

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

        extends_edges: Vec::new(),
        sealed_symbols: Vec::new(),
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
            if u.required_deps.contains(&std::path::PathBuf::from("b.h")) {
                a_reqs_b = true;
            }
        } else if u.path == std::path::PathBuf::from("b.h") {
            if u.required_deps.contains(&std::path::PathBuf::from("a.h")) {
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

// --- M23 T1: verbatim body resolution + namespace wiring (neutral fakes) ---

/// Content provider that never finds anything: every symbol must fall
/// back to the placeholder (invariant I4).
fn no_sources(_file_id: ariadne_repo::FileId) -> Option<String> {
    None
}

fn reader_for(named_texts: &[(&str, &str)]) -> impl Fn(ariadne_repo::FileId) -> Option<String> {
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
/// from `unit_namespace`, and records what `extract` was handed. Shared
/// with the write-render tests in `engine/write.rs`.
#[derive(Default)]
pub(super) struct RecordingFakeExtractor {
    locatable: Vec<(String, String)>,
    reported_namespace: String,
    extract_namespaces: std::sync::Mutex<Vec<String>>,
    unit_namespace_inputs: std::sync::Mutex<Vec<String>>,
}

impl RecordingFakeExtractor {
    pub(super) fn with_locatable_bodies(bodies: &[(&str, &str)]) -> Self {
        Self {
            locatable: bodies
                .iter()
                .map(|(name, body)| (name.to_string(), body.to_string()))
                .collect(),
            ..Default::default()
        }
    }

    pub(super) fn reporting_namespace(namespace: &str) -> Self {
        Self {
            reported_namespace: namespace.to_string(),
            ..Default::default()
        }
    }

    pub(super) fn extract_namespaces(&self) -> Vec<String> {
        self.extract_namespaces.lock().unwrap().clone()
    }

    pub(super) fn unit_namespace_inputs(&self) -> Vec<String> {
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

/// F4 carry-forward (M23 T3): `locate_bodies_for_unit` iterates
/// `wanted_per_source` (a `BTreeMap<FileId, _>`), so with several readable
/// source files the unit's `primary_source_text` must be the LOWEST FileId,
/// independent of plan enumeration order, and the first readable one when a
/// lower FileId is unreadable. This matters because the namespace seam
/// (D-M23-2/D-M23-4) feeds `primary_source_text` to
/// `SourceExtractor::unit_namespace` on write: a non-deterministic choice
/// would make Scala's `unit_namespace` (T5/T7) — and thus the generated
/// package clause — depend on extraction ordering.
#[test]
fn primary_source_text_is_the_lowest_readable_file_id_not_the_first_listed() {
    use ariadne_cluster::plan::ProposedExtraction;
    use ariadne_index::types::SymbolId;
    use ariadne_repo::FileId;

    let low_text = "// lowest file id\n";
    let high_text = "// higher file id\n";
    let extraction = |name: &str, source: FileId| ProposedExtraction {
        symbol_name: name.to_string(),
        symbol_id: SymbolId::new(name),
        target_header: std::path::PathBuf::from("shared.h"),
        source_file: Some(source),
    };
    // Deliberately enumerate the higher FileId first: BTreeMap order must win.
    let plan = plan_of(vec![
        extraction("Beta", FileId(2)),
        extraction("Alpha", FileId(1)),
    ]);
    let reader = |file_id: FileId| match file_id {
        FileId(1) => Some(low_text.to_string()),
        FileId(2) => Some(high_text.to_string()),
        _ => None,
    };

    let model = dummy_model();
    let engine = IsolationEngine::new(&model);
    let result = engine.extract(&plan, &reader, &RecordingFakeExtractor::default());
    assert_eq!(
        result.headers[0].primary_source_text.as_deref(),
        Some(low_text),
        "lowest FileId must win when several sources are readable"
    );

    // Unreadable lowest id: the FIRST READABLE id becomes primary.
    let skip_low = |file_id: FileId| (file_id == FileId(2)).then(|| high_text.to_string());
    let result = engine.extract(&plan, &skip_low, &RecordingFakeExtractor::default());
    assert_eq!(
        result.headers[0].primary_source_text.as_deref(),
        Some(high_text),
        "first readable id wins when the lowest is unreadable"
    );
}
