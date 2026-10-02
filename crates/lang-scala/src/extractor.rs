//! [`ScalaSourceExtractor`] — generated Scala unit text (plan M23, T5).
//!
//! Byte shape (mirrors the C++ extractor's structure, Scala-flavored):
//! an optional `package <ns>\n\n` clause, then the referenced-imports block
//! (one `import <dep>` line per required dependency, sorted and deduped,
//! rendered verbatim — dependency selection is upstream; this renderer is
//! dumb on purpose), a blank separator when any import was emitted, then
//! each verbatim symbol body followed by exactly one `\n`. The target path
//! is chosen by the engine (`unit.path`), so [`GeneratedSourceFile::path`]
//! stays empty, exactly like the C++ extractor.

use ariadne_lang::{GeneratedSourceFile, SourceExtractor, SymbolBody};
use std::path::PathBuf;

/// Generates the text of a newly extracted Scala source file.
pub struct ScalaSourceExtractor;

impl SourceExtractor for ScalaSourceExtractor {
    fn extract(
        &self,
        symbols: &[SymbolBody],
        namespace: &str,
        required_deps: &[String],
    ) -> GeneratedSourceFile {
        let mut content = String::new();
        content.push_str(&render_package_clause(namespace));
        content.push_str(&render_imports(required_deps));
        for symbol in symbols {
            content.push_str(&symbol.body);
            content.push('\n');
        }
        GeneratedSourceFile {
            path: PathBuf::new(),
            content,
        }
    }

    /// The file's package clause, re-derived with the parser's own directive
    /// scan (D-M23-2): block/label packages yield no namespace, matching the
    /// analysis-time qualification exactly.
    fn unit_namespace(&self, source_text: &str) -> String {
        crate::locator::package_namespace(source_text)
    }

    /// Verbatim top-level regions sliced by [`crate::locator`] (D-M23-1).
    fn locate_symbol_bodies(
        &self,
        source_text: &str,
        wanted: &[(String, ariadne_lang::SymbolKind)],
    ) -> Vec<SymbolBody> {
        crate::locator::locate_bodies(source_text, wanted)
    }
}

/// `package <ns>\n\n` — omitted entirely for an empty namespace (LOCK: an
/// extraction cut from a package-less source stays package-less).
fn render_package_clause(namespace: &str) -> String {
    if namespace.is_empty() {
        String::new()
    } else {
        format!("package {namespace}\n\n")
    }
}

/// One `import <dep>\n` per required dependency, sorted and deduped, plus a
/// blank separator — the Scala twin of the C++ include block. The renderer
/// is dumb by contract (see `deps_are_sorted_deduped_and_rendered_verbatim`):
/// referenced-only selection is the planner's job upstream, wildcard
/// spellings are never dropped (undroppable without the source file), and
/// source-file import lines are never fabricated.
fn render_imports(deps: &[String]) -> String {
    let mut ordered: Vec<&String> = deps.iter().collect();
    ordered.sort_unstable();
    ordered.dedup();
    if ordered.is_empty() {
        return String::new();
    }
    let mut block = String::new();
    for dep in ordered {
        block.push_str("import ");
        block.push_str(dep);
        block.push('\n');
    }
    block.push('\n');
    block
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(name: &str, text: &str) -> SymbolBody {
        SymbolBody {
            name: name.to_string(),
            body: text.to_string(),
        }
    }

    fn extract(bodies: &[SymbolBody], namespace: &str, deps: &[&str]) -> String {
        ScalaSourceExtractor
            .extract(
                bodies,
                namespace,
                &deps.iter().map(|d| d.to_string()).collect::<Vec<String>>(),
            )
            .content
    }

    #[test]
    fn package_clause_imports_and_bodies_byte_exact_two_dep_unit() {
        assert_eq!(
            extract(
                &[body("A", "class A {}")],
                "com.example.models",
                &["scala.collection.mutable", "com.example.util.Str"],
            ),
            "package com.example.models\n\n\
             import com.example.util.Str\n\
             import scala.collection.mutable\n\n\
             class A {}\n"
        );
    }

    #[test]
    fn empty_namespace_omits_the_package_line() {
        assert_eq!(
            extract(&[body("A", "class A {}")], "", &["a.b"]),
            "import a.b\n\nclass A {}\n",
            "LOCK: no package clause when the source had no package"
        );
    }

    #[test]
    fn empty_deps_omit_the_import_block_and_its_separator() {
        assert_eq!(
            extract(&[body("A", "class A {}")], "p", &[]),
            "package p\n\nclass A {}\n"
        );
    }

    #[test]
    fn deps_are_sorted_deduped_and_rendered_verbatim() {
        assert_eq!(
            extract(&[], "", &["b.x", "a.y", "b.x", "com.example.models._"]),
            "import a.y\nimport b.x\nimport com.example.models._\n\n",
            "LOCK: the import block is the rendering of the deps the engine \
             passed (sorted, deduped, `import <dep>` verbatim). Referenced-\
             only selection lives upstream in the required_deps the planner \
             assembles; the renderer never fabricates source-file import \
             lines, and never drops wildcard spellings — an unreferenced \
             wildcard cannot be proven dead without the source file"
        );
    }

    #[test]
    fn nothing_at_all_produces_empty_content() {
        assert_eq!(extract(&[], "", &[]), "");
        assert_eq!(extract(&[], "p", &[]), "package p\n\n");
    }

    #[test]
    fn bodies_keep_order_with_one_trailing_newline_each() {
        assert_eq!(
            extract(
                &[body("A", "class A {}"), body("b", "val b = 1")],
                "p",
                &[],
            ),
            "package p\n\nclass A {}\nval b = 1\n",
            "one trailing `\\n` per body; located regions never end in a \
             newline themselves (the slicer guarantees it)"
        );
    }

    #[test]
    fn path_is_left_to_the_engine_like_cpp() {
        let file = ScalaSourceExtractor.extract(&[], "", &[]);
        assert_eq!(file.path, PathBuf::new());
    }

    #[test]
    fn namespace_and_bodies_wire_through_the_plugin_hooks() {
        let source = "package legacy.shop\n\nimport util.Clock\n\nclass Gadget {\n  val name = \"g\"\n}\n\nclass Unused {\n}\n";
        let namespace = ScalaSourceExtractor.unit_namespace(source);
        assert_eq!(namespace, "legacy.shop");
        let bodies = ScalaSourceExtractor.locate_symbol_bodies(
            source,
            &[("Gadget".to_string(), ariadne_lang::SymbolKind::Class)],
        );
        assert_eq!(bodies.len(), 1);
        assert_eq!(
            extract(&bodies, &namespace, &[]),
            "package legacy.shop\n\nclass Gadget {\n  val name = \"g\"\n}\n",
            "extract output = package <ns> + verbatim top-level region \
             (spec §5-T5 red shape); no source-file imports are fabricated"
        );
    }

    #[test]
    fn plugin_extractor_emits_real_scala_bytes_not_the_stub() {
        use ariadne_lang::LanguagePlugin;
        let plugin = crate::plugin::ScalaPlugin::new();
        let content = plugin.extractor().extract(
            &[body("A", "class A {}")],
            "p",
            &["a.b".to_string()],
        );
        assert_eq!(content.content, "package p\n\nimport a.b\n\nclass A {}\n");
        assert!(
            content.path.as_os_str().is_empty(),
            "the retired StubExtractor's `out.scala` path is gone; the \
             engine picks the target path"
        );
    }
}
