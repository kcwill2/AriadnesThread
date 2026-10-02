//! Exact expected sets over the frozen 5-file `scala-basic` main corpus.
//! Read-only: `tests/fixtures/scala-basic/**` is locked by m19's 6-record
//! lock; this pass only reads it.

use super::tests::{edge_keys, parse_src, ref_ids};
use super::*;
use std::path::{Path, PathBuf};

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/scala-basic/src/main/scala/com/example")
}

fn parse_fixture(rel_path: &str) -> TuAnalysis {
    let full = fixture_dir().join(rel_path);
    let source = std::fs::read_to_string(&full)
        .unwrap_or_else(|e| panic!("fixture {rel_path} must be readable: {e}"));
    let normalized = full.to_string_lossy().replace('\\', "/");
    let file_id = FileId::from_relative_path(&normalized);
    ScalaSourceParser.parse(file_id, &full, &source)
}

#[test]
fn main_scala_yields_lone_object_symbol_and_raw_call_sites() {
    let analysis = parse_fixture("Main.scala");
    assert_eq!(symbol_id_and_kind(&analysis), vec![("com.example.Main", "Object")]);
    assert_eq!(analysis.declarations[0].location.line, 3);
    assert_eq!(analysis.declarations[0].location.column, 8);
    assert!(analysis.edges.is_empty());
    assert!(ref_ids(&analysis).is_empty(), "println/args are unresolvable lowercase: {analysis:?}");
    let callees: Vec<&str> = analysis
        .call_sites
        .iter()
        .map(|c| c.callee_name.as_str())
        .collect();
    assert_eq!(callees, ["println"], "`def main(` is a definition, not a call");
    assert_eq!(analysis.call_sites[0].location.line, 5);
    assert!(analysis
        .call_sites
        .iter()
        .all(|c| c.caller_symbol.0 == "com.example.Main"));
}

#[test]
fn user_scala_yields_case_class_symbol_only() {
    let analysis = parse_fixture("models/User.scala");
    assert_eq!(
        symbol_id_and_kind(&analysis),
        vec![("com.example.models.User", "Class")]
    );
    assert_eq!(analysis.declarations[0].location.line, 3);
    assert_eq!(analysis.declarations[0].location.column, 12);
    assert!(analysis.edges.is_empty());
    assert!(ref_ids(&analysis).is_empty(), "Int/String are intrinsic: {analysis:?}");
    assert!(analysis.call_sites.is_empty());
}

#[test]
fn order_scala_edges_the_imported_service_only() {
    let analysis = parse_fixture("models/Order.scala");
    assert_eq!(
        symbol_id_and_kind(&analysis),
        vec![("com.example.models.Order", "Class")]
    );
    assert_eq!(
        edge_keys(&analysis),
        vec![(
            "com.example.models.Order".into(),
            "com.example.services.OrderService".into(),
            DependencyKind::Complete
        )]
    );
    assert_eq!(ref_ids(&analysis), ["com.example.services.OrderService"]);
    assert_eq!(analysis.call_sites.len(), 1);
    assert_eq!(analysis.call_sites[0].callee_name, "summarize");
    assert_eq!(analysis.call_sites[0].caller_symbol.0, "com.example.models.Order");
}

#[test]
fn order_scala_user_reference_is_invisible_across_files_as_documented() {
    let analysis = parse_fixture("models/Order.scala");
    assert!(
        !ref_ids(&analysis)
            .iter()
            .any(|r| r.ends_with(".User")),
        "cross-file same-package siblings are a documented offline limitation"
    );
}

#[test]
fn all_scala_package_object_takes_namespace_id_and_object_kind() {
    let analysis = parse_fixture("models/all.scala");
    assert_eq!(
        symbol_id_and_kind(&analysis),
        vec![("com.example.models", "Object")]
    );
    assert_eq!(analysis.symbols[0].name, "models");
    assert_eq!(analysis.declarations[0].id.0, "com.example.models#L2");
    assert!(analysis.edges.is_empty());
    assert!(ref_ids(&analysis).is_empty());
}

#[test]
fn order_service_scala_exact_symbols_edges_and_references() {
    let analysis = parse_fixture("services/OrderService.scala");
    assert_eq!(
        symbol_id_and_kind(&analysis),
        vec![("com.example.services.OrderService", "Class")]
    );
    assert_eq!(analysis.declarations[0].location.line, 6);
    assert_eq!(analysis.declarations[0].location.column, 7);
    // D27 fixture evolution (`import com.example.models._` replaced the
    // grouped import): only the FIRST wildcard is attributable (D16), so all
    // three usages are Incomplete edges guessed into `com.example.models` —
    // the heuristic cannot see that ArrayBuffer's real home is the second
    // wildcard's package, and never fabricates a non-attributed edge.
    assert_eq!(
        edge_keys(&analysis),
        vec![
            (
                "com.example.services.OrderService".into(),
                "com.example.models.ArrayBuffer".into(),
                DependencyKind::Incomplete
            ),
            (
                "com.example.services.OrderService".into(),
                "com.example.models.User".into(),
                DependencyKind::Incomplete
            ),
            (
                "com.example.services.OrderService".into(),
                "com.example.models.Order".into(),
                DependencyKind::Incomplete
            ),
        ]
    );
    assert_eq!(
        ref_ids(&analysis),
        [
            "com.example.models.ArrayBuffer",
            "com.example.models.User",
            "com.example.models.Order"
        ]
    );
    assert_eq!(analysis.references[0].location.line, 7);
    assert_eq!(analysis.references[1].location.line, 13);
}

#[test]
fn order_service_scala_interpolated_splice_contributes_nothing() {
    let analysis = parse_fixture("services/OrderService.scala");
    let splice_line = 14;
    assert!(
        !analysis
            .references
            .iter()
            .any(|r| r.location.line == splice_line),
        "no reference may originate inside the s-interpolation of summarize"
    );
    assert!(
        !analysis
            .call_sites
            .iter()
            .any(|c| c.location.line == splice_line)
    );
    assert_eq!(
        analysis.call_sites.len(),
        0,
        "ArrayBuffer[Int]() and def-headed lines must not fabricate call sites"
    );
}

#[test]
fn corpus_edges_carry_the_analyzed_file_on_both_sides_like_lang_cpp() {
    let analysis = parse_fixture("services/OrderService.scala");
    for edge in &analysis.edges {
        assert_eq!(edge.src_file, analysis.tu_file_id);
        assert_eq!(edge.dst_file, analysis.tu_file_id);
    }
}

#[test]
fn malformed_and_truncated_sources_never_panic() {
    let hostile = [
        "package x {",
        "class",
        "class A {",
        "\"\"\"",
        "s\"${",
        "class A extends",
        "val x = /*",
        "'",
        "object O { def f(",
        "/*",
        "`",
        "case",
        "given",
        "import a.b.{",
        "package",
    ];
    for source in hostile {
        let analysis = parse_src(source);
        for symbol in &analysis.symbols {
            assert_eq!(symbol.declarations.len(), 1, "invariant for {source:?}");
            assert!(symbol.definition.is_some());
        }
    }
}

fn symbol_id_and_kind(analysis: &TuAnalysis) -> Vec<(&str, &'static str)> {
    analysis
        .symbols
        .iter()
        .map(|s| {
            let kind = match s.kind {
                ariadne_lang::SymbolKind::Class => "Class",
                ariadne_lang::SymbolKind::Object => "Object",
                ariadne_lang::SymbolKind::Trait => "Trait",
                ariadne_lang::SymbolKind::Enum => "Enum",
                ariadne_lang::SymbolKind::Function => "Function",
                ariadne_lang::SymbolKind::Variable => "Variable",
                _ => "Other",
            };
            (s.id.0.as_str(), kind)
        })
        .collect()
}
