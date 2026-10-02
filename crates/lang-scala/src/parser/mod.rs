//! Offline heuristic Scala source parser (M21 T-A, decisions D16/D17/D18).
//!
//! [`ScalaSourceParser`] turns one Scala source string into a
//! [`TuAnalysis`] without any Scala toolchain. It is a hand-written
//! scanner+parser (own [`crate::scanner`], D18), deterministic, source-order
//! preserving, and total: malformed input never panics.
//!
//! Documented limitations (D16/D17/D18, each negatively tested where marked):
//! - Only top-level (brace-depth-0) declarations become symbols; nested
//!   declarations contribute no symbols, only occurrence context.
//! - `package x { ... }` blocks and Scala-3 `package x:` labels are skipped
//!   whole: no crash, and no symbols/edges/references from inside either. ✓
//! - Interpolation splices `s"...${expr}..."` are not analyzed (D18); a
//!   `"` inside a splice ends the literal early (invalid-input leak, the
//!   same concession M20 made). ✓
//! - `import` selector renames (`{C => D}`) are not mapped; the selector
//!   region is still consumed.
//! - Cross-file same-package siblings are invisible offline: a sibling
//!   reference resolves only when the sibling is declared in the same file
//!   (D16's "same-package siblings declared in this file"); a reference to a
//!   sibling living in another file of the package produces nothing here —
//!   index-level edge coverage comes from explicit imports.
//! - Unresolvable non-parent names produce neither edges nor references
//!   (the handoff's "pick one rule" resolved in favor of "nothing");
//!   `X(` / `new X` still record a raw-named [`ariadne_lang::CallSite`]
//!   when an enclosing tracked symbol exists. ✓
//! - `extends`/`with` parents always produce an edge: Complete when the
//!   target maps to a named import or an in-file declaration, else Incomplete
//!   with a `<package>.<Name>` qualified guess (D16 (1)). ✓
//! - Multiple wildcard imports: the first wildcard wins ("at most one active
//!   wildcard attribution" per D16); later wildcards are ignored.
//! - `given` without a leading lowercase name (anonymous givens like
//!   `given Ordering[Int] with ...`) produces no symbol.
//! - Reference `is_write` is always `false`; no write analysis is done.
//! - `rel_path` is intentionally unused: every output field is derivable
//!   from `source` + `file_id`; inventing path usage would be dishonest.
//! - Edge `src_file`/`dst_file` mirror the `lang-cpp` analyzer convention
//!   (both set to the analyzed TU file, which it resolves); no `FileId(0)`
//!   sentinel is needed.

#[cfg(test)]
mod corpus;
mod declarations;
mod directives;
mod occurrences;

use ariadne_lang_api::types::{DependencyKind, SemanticEdge, TuAnalysis};
use ariadne_repo::{FileId, SourceLanguage};
use crate::scanner::Token;
use declarations::{collect_top_level_decls, resolve_decl_ids, symbol_payloads};
use directives::scan_directives;
pub(crate) use declarations::ResolvedDecl;
use occurrences::scan_occurrences;
use std::collections::BTreeSet;

/// Tokenized source with its resolved top-level declarations and derived
/// package namespace (plan M23, T5): the shared view `locate_symbol_bodies`
/// and `unit_namespace` consume, so region slicing sees exactly the
/// analysis-time qualification — no second parser.
pub(crate) struct SourceProgram {
    pub namespace: String,
    pub tokens: Vec<Token>,
    pub decls: Vec<ResolvedDecl>,
}

/// Runs scanner → directives → declarations over `source` without building
/// an analysis (same passes [`ScalaSourceParser::parse`] uses, minus the
/// occurrence walk).
pub(crate) fn program_of(source: &str) -> SourceProgram {
    let tokens = crate::scanner::tokenize(source);
    let directives = scan_directives(&tokens);
    let decl_pass = collect_top_level_decls(&tokens, &directives);
    let decls = resolve_decl_ids(decl_pass.decls, &tokens, &directives);
    SourceProgram {
        namespace: directives.namespace,
        tokens,
        decls,
    }
}

/// Stateless offline Scala source parser (plan:407 contract).
pub struct ScalaSourceParser;

impl ScalaSourceParser {
    /// Hand-written offline scanner; deterministic; never panics on invalid
    /// input. `rel_path` is accepted per the plan signature but unused in
    /// the output (see module docs).
    pub fn parse(
        &self,
        file_id: FileId,
        rel_path: &std::path::Path,
        source: &str,
    ) -> ariadne_lang::TuAnalysis {
        let _ = rel_path;
        let tokens = crate::scanner::tokenize(source);
        let directives = scan_directives(&tokens);
        let decl_pass = collect_top_level_decls(&tokens, &directives);
        let resolved = resolve_decl_ids(decl_pass.decls, &tokens, &directives);
        let scope = occurrences::Scope {
            namespace: &directives.namespace,
            imports: &directives.named_imports,
            wildcard_base: directives.wildcard_base.as_deref(),
            decls: &resolved,
            skip: &directives.consumed_ranges,
            name_tokens: &decl_pass.name_tokens,
        };
        let occurrences = scan_occurrences(&tokens, &file_id, &scope);
        let (symbols, declarations) = symbol_payloads(&resolved, &file_id);
        TuAnalysis {
            tu_file_id: file_id,
            context_name: String::new(),
            symbols,
            declarations,
            edges: dedupe_edges(occurrences.edges),
            references: occurrences.references,
            call_sites: occurrences.call_sites,
            language: SourceLanguage::Scala,
        }
    }
}

fn dedupe_edges(edges: Vec<SemanticEdge>) -> Vec<SemanticEdge> {
    let mut seen: BTreeSet<(String, String, DependencyKind)> = BTreeSet::new();
    edges
        .into_iter()
        .filter(|edge| {
            seen.insert((
                edge.src_symbol.0.clone(),
                edge.dst_symbol.0.clone(),
                edge.kind,
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file_id() -> FileId {
        FileId::from_relative_path("src/test.scala")
    }

    pub(crate) fn parse_src(source: &str) -> TuAnalysis {
        ScalaSourceParser.parse(file_id(), std::path::Path::new("src/test.scala"), source)
    }

    fn symbol_ids(analysis: &TuAnalysis) -> Vec<String> {
        analysis.symbols.iter().map(|s| s.id.0.clone()).collect()
    }

    pub(crate) fn edge_keys(analysis: &TuAnalysis) -> Vec<(String, String, DependencyKind)> {
        analysis
            .edges
            .iter()
            .map(|e| (e.src_symbol.0.clone(), e.dst_symbol.0.clone(), e.kind))
            .collect()
    }

    pub(crate) fn ref_ids(analysis: &TuAnalysis) -> Vec<String> {
        analysis
            .references
            .iter()
            .map(|r| r.symbol_id.0.clone())
            .collect()
    }

    #[test]
    fn class_with_package_yields_qualified_symbol_and_decl() {
        let analysis = parse_src("package com.p\n\nclass Widget {\n  val size = 1\n}");
        assert_eq!(symbol_ids(&analysis), ["com.p.Widget"]);
        let symbol = &analysis.symbols[0];
        assert_eq!(symbol.name, "Widget");
        assert_eq!(symbol.kind, ariadne_lang::SymbolKind::Class);
        assert_eq!(symbol.owning_file, file_id());
        assert_eq!(
            symbol.declarations,
            vec![ariadne_lang::DeclId::new("com.p.Widget#L3")]
        );
        assert_eq!(symbol.definition, symbol.declarations.first().cloned());
        let decl = &analysis.declarations[0];
        assert!(decl.is_definition);
        assert_eq!(decl.location.line, 3);
        assert_eq!(decl.location.column, 7);
        assert_eq!(analysis.language, SourceLanguage::Scala);
    }

    #[test]
    fn case_class_is_class() {
        let analysis = parse_src("package p\n\ncase class Point(x: Int, y: Int)");
        assert_eq!(symbol_ids(&analysis), ["p.Point"]);
        assert_eq!(analysis.symbols[0].kind, ariadne_lang::SymbolKind::Class);
    }

    #[test]
    fn trait_is_trait() {
        let analysis = parse_src("package p\n\ntrait Shape");
        assert_eq!(analysis.symbols[0].kind, ariadne_lang::SymbolKind::Trait);
    }

    #[test]
    fn object_is_object() {
        let analysis = parse_src("package p\n\nobject Registry");
        assert_eq!(analysis.symbols[0].kind, ariadne_lang::SymbolKind::Object);
    }

    #[test]
    fn case_object_is_object() {
        let analysis = parse_src("package p\n\ncase object Singleton");
        assert_eq!(analysis.symbols[0].kind, ariadne_lang::SymbolKind::Object);
    }

    #[test]
    fn enum_is_enum() {
        let analysis = parse_src("package p\n\nenum Color {\n  case Red, Green\n}");
        assert_eq!(symbol_ids(&analysis), ["p.Color"]);
        assert_eq!(analysis.symbols[0].kind, ariadne_lang::SymbolKind::Enum);
    }

    #[test]
    fn top_level_def_is_function() {
        let analysis = parse_src("package p\n\ndef greet(): String = \"hi\"");
        assert_eq!(
            symbol_ids(&analysis),
            ["p.greet"],
            "SymbolRecord.name = simple identifier, id = <package>.<Name>"
        );
        assert_eq!(analysis.symbols[0].kind, ariadne_lang::SymbolKind::Function);
    }

    #[test]
    fn top_level_val_is_variable() {
        let analysis = parse_src("package p\n\nval limit = 4");
        assert_eq!(symbol_ids(&analysis), ["p.limit"]);
        assert_eq!(analysis.symbols[0].kind, ariadne_lang::SymbolKind::Variable);
    }

    #[test]
    fn given_value_is_variable() {
        let analysis = parse_src("package p\n\ngiven greeting: String = \"hi\"");
        assert_eq!(symbol_ids(&analysis), ["p.greeting"]);
        assert_eq!(analysis.symbols[0].kind, ariadne_lang::SymbolKind::Variable);
    }

    #[test]
    fn given_with_parameter_list_is_function() {
        let analysis = parse_src("package p\n\ngiven boxFor[T](t: T): T = t");
        assert_eq!(symbol_ids(&analysis), ["p.boxFor"]);
        assert_eq!(analysis.symbols[0].kind, ariadne_lang::SymbolKind::Function);
    }

    #[test]
    fn companion_pair_class_and_object_take_plain_and_dollar_ids() {
        let analysis = parse_src("package com.p\n\nclass Counter\nobject Counter");
        assert_eq!(symbol_ids(&analysis), ["com.p.Counter", "com.p.Counter$"]);
        assert!(analysis.symbols.iter().all(|s| s.name == "Counter"));
        assert_eq!(
            analysis.declarations[1].id,
            ariadne_lang::DeclId::new("com.p.Counter$#L4")
        );
    }

    #[test]
    fn lone_object_keeps_plain_id_without_dollar() {
        let analysis = parse_src("package com.p\n\nobject Main {\n  def go(): Unit = {}\n}");
        assert_eq!(symbol_ids(&analysis), ["com.p.Main"]);
    }

    #[test]
    fn tie_break_type_position_is_class_id_and_call_position_is_object_id() {
        let analysis = parse_src(
            "package com.p\n\nclass Box(val v: Int)\n\nobject Box {\n  def make(): Box = Box(1)\n}",
        );
        assert_eq!(ref_ids(&analysis), ["com.p.Box", "com.p.Box$"]);
        assert_eq!(
            edge_keys(&analysis),
            vec![
                ("com.p.Box$".into(), "com.p.Box".into(), DependencyKind::Complete),
                ("com.p.Box$".into(), "com.p.Box$".into(), DependencyKind::Complete),
            ]
        );
        assert_eq!(analysis.call_sites.len(), 1);
        assert_eq!(analysis.call_sites[0].callee_name, "Box");
        assert_eq!(analysis.call_sites[0].caller_symbol.0, "com.p.Box$");
    }

    #[test]
    fn extends_in_file_sibling_is_complete_edge() {
        let analysis = parse_src("package p\n\nclass Animal\nclass Dog extends Animal");
        assert_eq!(
            edge_keys(&analysis),
            vec![("p.Dog".into(), "p.Animal".into(), DependencyKind::Complete)]
        );
        assert_eq!(ref_ids(&analysis), ["p.Animal"]);
    }

    #[test]
    fn extends_unresolvable_is_incomplete_edge_to_qualified_guess() {
        let analysis = parse_src("package p\n\nclass Solo extends Base");
        assert_eq!(
            edge_keys(&analysis),
            vec![("p.Solo".into(), "p.Base".into(), DependencyKind::Incomplete)]
        );
        assert!(
            ref_ids(&analysis).is_empty(),
            "unresolvable parents get no reference"
        );
    }

    #[test]
    fn with_clause_adds_parents() {
        let analysis = parse_src("package p\n\nclass B\nclass C\nclass A extends B with C");
        assert_eq!(
            edge_keys(&analysis),
            vec![
                ("p.A".into(), "p.B".into(), DependencyKind::Complete),
                ("p.A".into(), "p.C".into(), DependencyKind::Complete),
            ]
        );
    }

    #[test]
    fn new_expression_yields_reference_edge_and_call_site() {
        let analysis = parse_src(
            "package p\n\nclass Thing\nobject Factory {\n  val made = new Thing\n}",
        );
        assert_eq!(ref_ids(&analysis), ["p.Thing"]);
        assert_eq!(
            edge_keys(&analysis),
            vec![("p.Factory".into(), "p.Thing".into(), DependencyKind::Complete)]
        );
        assert_eq!(analysis.call_sites.len(), 1);
        assert_eq!(analysis.call_sites[0].callee_name, "Thing");
        assert_eq!(analysis.call_sites[0].caller_symbol.0, "p.Factory");
    }

    #[test]
    fn named_import_feeds_complete_edge() {
        let analysis = parse_src(
            "package p\n\nimport a.b.Widget\n\nclass Use {\n  val w: Widget = null\n}",
        );
        assert_eq!(
            edge_keys(&analysis),
            vec![("p.Use".into(), "a.b.Widget".into(), DependencyKind::Complete)]
        );
        assert_eq!(ref_ids(&analysis), ["a.b.Widget"]);
    }

    #[test]
    fn grouped_import_selectors_feed_complete_edges() {
        let analysis = parse_src(
            "package p\n\nimport a.b.{Alpha, Beta}\n\nclass Use {\n  def mix(x: Alpha, y: Beta): Unit = ???\n}",
        );
        assert_eq!(
            edge_keys(&analysis),
            vec![
                ("p.Use".into(), "a.b.Alpha".into(), DependencyKind::Complete),
                ("p.Use".into(), "a.b.Beta".into(), DependencyKind::Complete),
            ]
        );
        assert_eq!(ref_ids(&analysis), ["a.b.Alpha", "a.b.Beta"]);
    }

    #[test]
    fn wildcard_import_yields_incomplete_edge_to_qualified_guess() {
        let analysis = parse_src(
            "package p\n\nimport scala.collection.mutable._\n\nclass Bag {\n  val xs = ArrayBuffer[Int]()\n}",
        );
        assert_eq!(
            edge_keys(&analysis),
            vec![(
                "p.Bag".into(),
                "scala.collection.mutable.ArrayBuffer".into(),
                DependencyKind::Incomplete
            )]
        );
        assert_eq!(ref_ids(&analysis), ["scala.collection.mutable.ArrayBuffer"]);
    }

    #[test]
    fn same_package_sibling_in_file_resolves_without_import() {
        let analysis = parse_src(
            "package com.example.models\n\nclass Order {\n  def link(u: User): Unit = {}\n}\n\nclass User",
        );
        assert_eq!(
            edge_keys(&analysis),
            vec![(
                "com.example.models.Order".into(),
                "com.example.models.User".into(),
                DependencyKind::Complete
            )]
        );
    }

    #[test]
    fn nested_package_block_is_graceful_negative() {
        let analysis = parse_src("package outer {\n  class Hidden\n}");
        assert!(analysis.symbols.is_empty());
        assert!(analysis.declarations.is_empty());
        assert!(analysis.edges.is_empty());
        assert!(analysis.references.is_empty());
    }

    #[test]
    fn scala3_package_label_is_graceful_negative() {
        let analysis = parse_src("package outer:\nclass Hidden");
        assert!(analysis.symbols.is_empty());
        assert!(analysis.edges.is_empty());
    }

    #[test]
    fn line_commented_decl_is_invisible() {
        let analysis = parse_src("// class Ghost {}\npackage p\n\nclass Real");
        assert_eq!(symbol_ids(&analysis), ["p.Real"]);
    }

    #[test]
    fn block_commented_decl_is_invisible() {
        let analysis = parse_src("/*\nclass Ghost {}\n*/\npackage p\n\nclass Real");
        assert_eq!(symbol_ids(&analysis), ["p.Real"]);
    }

    #[test]
    fn decl_text_inside_string_literals_is_invisible() {
        let analysis = parse_src(
            "package p\n\nimport a.b.Widget\n\nobject Doc {\n  val one = \"class Ghost {}\"\n  val two = \"\"\"object Phantom {}\"\"\"\n  val three = \"\"\"Widget(1)\"\"\"\n}",
        );
        assert_eq!(symbol_ids(&analysis), ["p.Doc"]);
        assert!(analysis.references.is_empty());
        assert!(
            analysis.edges.is_empty() && analysis.call_sites.is_empty(),
            "Gate-A note 3: a callable leak (`Widget(` of the named import) inside \
             a triple-quoted literal must produce neither edges nor call-sites"
        );
    }

    #[test]
    fn nested_block_comment_wrapping_a_decl_yields_nothing() {
        let analysis = parse_src("/* /* class Ghost */ */");
        assert!(analysis.symbols.is_empty());
        assert!(analysis.edges.is_empty());
        assert!(analysis.references.is_empty());
    }

    #[test]
    fn anonymous_given_yields_no_symbol() {
        let analysis = parse_src("package p\n\ngiven Ordering[Int] with {}");
        assert!(
            analysis.symbols.is_empty(),
            "Gate-A note 2: anonymous givens (non-lowercase name) are a documented \
             no-symbol limitation"
        );
    }

    #[test]
    fn export_selector_feeds_complete_edge_like_import() {
        let analysis = parse_src(
            "package p\n\nexport a.b.Helper\n\nclass Use {\n  val h: Helper = null\n}",
        );
        assert_eq!(
            edge_keys(&analysis),
            vec![("p.Use".into(), "a.b.Helper".into(), DependencyKind::Complete)],
            "Gate-A note 4: export participates in the edge map like import"
        );
    }

    #[test]
    fn interpolated_splice_produces_no_symbols_edges_or_references() {
        let analysis = parse_src(
            "package p\n\nclass Foo\n\nobject O {\n  def line(): String = s\"${Foo(1)} done\"\n}",
        );
        assert_eq!(symbol_ids(&analysis), ["p.Foo", "p.O"]);
        assert!(analysis.edges.is_empty(), "splices are not analyzed");
        assert!(analysis.references.is_empty());
        assert!(analysis.call_sites.is_empty());
    }

    #[test]
    fn repeated_resolvable_occurrences_keep_refs_but_dedupe_edges() {
        let analysis = parse_src(
            "package p\n\nimport a.b.Widget\n\nclass Use {\n  def f(w: Widget): Widget = ???\n  def g: Widget = ???\n}",
        );
        assert_eq!(edge_keys(&analysis).len(), 1);
        assert_eq!(
            ref_ids(&analysis),
            ["a.b.Widget", "a.b.Widget", "a.b.Widget"]
        );
    }

    #[test]
    fn unresolvable_names_never_edge_but_calls_are_recorded_raw() {
        let analysis = parse_src("package p\n\nclass C {\n  def use(): Unit = Whatever()\n}");
        assert!(analysis.edges.is_empty());
        assert!(analysis.references.is_empty());
        assert_eq!(analysis.call_sites.len(), 1);
        assert_eq!(analysis.call_sites[0].callee_name, "Whatever");
        assert_eq!(analysis.call_sites[0].caller_symbol.0, "p.C");
    }

    #[test]
    fn package_object_under_package_clause_uses_namespace_id_and_object_kind() {
        let analysis = parse_src("package com.example\n\npackage object models {}");
        assert_eq!(symbol_ids(&analysis), ["com.example.models"]);
        assert_eq!(analysis.symbols[0].kind, ariadne_lang::SymbolKind::Object);
        assert_eq!(analysis.declarations[0].id.0, "com.example.models#L3");
    }

    #[test]
    fn output_stays_in_source_order() {
        let analysis = parse_src("package p\n\nclass Alpha\nclass Beta");
        assert_eq!(symbol_ids(&analysis), ["p.Alpha", "p.Beta"]);
        assert_eq!(analysis.declarations[0].location.line, 3);
        assert_eq!(analysis.declarations[1].location.line, 4);
    }

    #[test]
    fn language_is_always_scala_even_for_garbage() {
        let analysis = parse_src("package x {");
        assert_eq!(analysis.language, SourceLanguage::Scala);
    }
}
