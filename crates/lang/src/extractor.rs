//! Source file generation trait for language plugins.

use ariadne_lang_api::types::SymbolKind;
use std::path::PathBuf;

/// A symbol's raw source text, ready to be written into a new file.
#[derive(Debug, Clone)]
pub struct SymbolBody {
    /// The symbol's simple name (e.g. `"Order"`, `"hash_combine"`).
    pub name: String,
    /// The verbatim source text of the symbol's definition block.
    pub body: String,
}

/// A new source file produced by the extraction engine.
#[derive(Debug, Clone)]
pub struct GeneratedSourceFile {
    /// Relative path for the file within the repository.
    pub path: PathBuf,
    /// Complete text content ready to be written to disk.
    pub content: String,
}

/// Generates new source files from a group of extracted symbols.
///
/// For C++, this writes a new `.h` file with `#pragma once`, required
/// `#include` directives, and the symbol bodies.
/// For Scala, this writes a new `.scala` file with the `package` declaration,
/// minimal `import` statements, and the class/trait/object bodies.
pub trait SourceExtractor: Send + Sync {
    /// Build a [`GeneratedSourceFile`] from `symbols`.
    ///
    /// - `namespace`: the package or namespace context (e.g. `"com.example.domain"`)
    /// - `required_deps`: dependency directives to prepend (includes or imports)
    fn extract(
        &self,
        symbols: &[SymbolBody],
        namespace: &str,
        required_deps: &[String],
    ) -> GeneratedSourceFile;

    /// Namespace the engine should pass back into [`SourceExtractor::extract`]
    /// for units cut from `source_text` (plan M23, D-M23-2). The default is
    /// empty — C++ headers need no namespace and ignores the `extract`
    /// argument anyway (locked by `namespace_argument_is_ignored`); Scala
    /// re-derives the file's package clause.
    fn unit_namespace(&self, _source_text: &str) -> String {
        String::new()
    }

    /// Verbatim declaration blocks for the named symbols located inside
    /// `source_text` (plan M23, D-M23-1). Plugins locate bodies in their own
    /// syntax at extract time; names that cannot be located are simply
    /// omitted from the result, and the engine keeps its placeholder body
    /// for them (invariant I4). The default finds nothing.
    fn locate_symbol_bodies(
        &self,
        _source_text: &str,
        _wanted: &[(String, SymbolKind)],
    ) -> Vec<SymbolBody> {
        Vec::new()
    }

    /// The dependency list [`SourceExtractor::extract`] receives for one
    /// generated unit (plan M23 T7, D-M23-13 as refined by the orchestrator's
    /// replace-passthrough ruling).
    ///
    /// `unit_paths` are the engine's inter-unit file-path deps (today's
    /// `required_deps`); the hook **replaces** them — it does not merge — so
    /// a language whose import syntax is not file paths (Scala: FQNs,
    /// D-M23-5) can emit what its compiler actually understands. The default
    /// is the verbatim passthrough of `unit_paths` — exactly today's
    /// behavior — so C++ keeps its byte locks unchanged.
    ///
    /// `primary_source_text` is `None` when the unit has no source file
    /// (placeholder-only renders): honest producers keep the passthrough
    /// then, because inventing directives without the file would fabricate.
    fn unit_imports(
        &self,
        _primary_source_text: Option<&str>,
        _bodies: &[SymbolBody],
        unit_paths: &[String],
    ) -> Vec<String> {
        unit_paths.to_vec()
    }

    /// Whether generated units land at their **repo-relative plan target**
    /// path (in-place) instead of under the applier's `generated/` staging
    /// directory (plan M23, T8b; D-M23-5 naming already resolves targets
    /// beside the source). The default is `false` — C++ copy semantics keep
    /// staging under `generated/` byte-identically (I1); move semantics
    /// (Scala) require the file to live where the plan says, or the move
    /// would strand the symbol in a staging dir the compiler never reads.
    fn places_generated_units_in_place(&self) -> bool {
        false
    }
}

/// Render the complete text of one generated unit — **the single emitter**
/// for generated-file bytes (plan M23, invariant I3 / D-M23-4). The
/// isolation engine's disk write and the transactional applier both route
/// through here, so the namespace-seam rule lives in exactly one place:
/// [`SourceExtractor::unit_namespace`] is consulted only when
/// `primary_source_text` is `Some` (D-M23-2); otherwise no hook call is
/// made and the namespace stays empty (C++ bytes stay unchanged).
/// Dependency text follows the same single path: [`SourceExtractor::unit_imports`]
/// transforms `required_deps` before `extract` sees them (D-M23-13, T7
/// replace-passthrough ruling); the default hook is the identity, so C++
/// bytes through this seam are unchanged.
pub fn render_unit(
    extractor: &dyn SourceExtractor,
    symbol_bodies: &[SymbolBody],
    primary_source_text: Option<&str>,
    required_deps: &[String],
) -> String {
    let namespace = match primary_source_text {
        Some(source_text) => extractor.unit_namespace(source_text),
        None => String::new(),
    };
    let deps = extractor.unit_imports(primary_source_text, symbol_bodies, required_deps);
    extractor.extract(symbol_bodies, &namespace, &deps).content
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bodies(names: &[&str]) -> Vec<SymbolBody> {
        names
            .iter()
            .map(|name| SymbolBody {
                name: name.to_string(),
                body: format!("class {name} {{}}"),
            })
            .collect()
    }

    fn paths(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    struct PassthroughProbe;
    impl SourceExtractor for PassthroughProbe {
        fn extract(
            &self,
            _symbols: &[SymbolBody],
            _namespace: &str,
            required_deps: &[String],
        ) -> GeneratedSourceFile {
            GeneratedSourceFile {
                path: PathBuf::new(),
                content: required_deps.join("|"),
            }
        }
    }

    struct RoutingProducer;
    impl SourceExtractor for RoutingProducer {
        fn extract(
            &self,
            _symbols: &[SymbolBody],
            _namespace: &str,
            required_deps: &[String],
        ) -> GeneratedSourceFile {
            GeneratedSourceFile {
                path: PathBuf::new(),
                content: required_deps.join("|"),
            }
        }

        fn unit_imports(
            &self,
            primary_source_text: Option<&str>,
            _bodies: &[SymbolBody],
            unit_paths: &[String],
        ) -> Vec<String> {
            match primary_source_text {
                Some(_) => paths(&["a.b.C", "scala.collection.mutable._"]),
                None => unit_paths.to_vec(),
            }
        }
    }

    #[test]
    fn unit_imports_default_passes_unit_paths_through_verbatim() {
        let probe = PassthroughProbe;
        let unit_paths = paths(&["gen/a.h", "gen/b.h"]);
        assert_eq!(
            probe.unit_imports(Some("int x;"), &bodies(&["X"]), &unit_paths),
            unit_paths,
            "LOCK (D-M23-13 replace-passthrough): the default hands back the \
             engine's unit paths byte-identical WITH a source text — the C++ \
             extract deps are unchanged"
        );
        assert_eq!(
            probe.unit_imports(None, &bodies(&["X"]), &unit_paths),
            unit_paths,
            "LOCK: same passthrough without a source text"
        );
        assert!(
            probe
                .unit_imports(Some("x"), &bodies(&["X"]), &[])
                .is_empty(),
            "empty unit paths stay empty"
        );
    }

    #[test]
    fn render_unit_default_route_keeps_required_deps_byte_identical() {
        let unit_paths = paths(&["gen/a.h", "gen/b.h"]);
        assert_eq!(
            render_unit(&PassthroughProbe, &bodies(&["X"]), Some("src"), &unit_paths),
            "gen/a.h|gen/b.h",
            "LOCK (I1): with the default hook the bytes extract sees are \
             exactly the required_deps the engine passed"
        );
        assert_eq!(
            render_unit(&PassthroughProbe, &bodies(&["X"]), None, &unit_paths),
            "gen/a.h|gen/b.h"
        );
    }

    #[test]
    fn render_unit_hands_unit_imports_output_to_extract() {
        assert_eq!(
            render_unit(&RoutingProducer, &bodies(&["X"]), Some("package p"), &paths(&["src/main/scala/p/Legacy.scala"])),
            "a.b.C|scala.collection.mutable._",
            "D-M23-13 (T7 ruling): extract must receive the hook's output, \
             not the raw engine unit paths"
        );
        assert_eq!(
            render_unit(&RoutingProducer, &bodies(&["X"]), None, &paths(&["gen/a.h"])),
            "gen/a.h",
            "no source text: the producer's own passthrough survives"
        );
    }

    #[test]
    fn pin_neutral_extractor_defaults_find_nothing_and_stage_under_generated() {
        assert!(
            PassthroughProbe
                .locate_symbol_bodies("class X {}", &[("X".to_string(), SymbolKind::Class)])
                .is_empty(),
            "D-M23-1 / I4: the default locates nothing - the engine keeps its \
             placeholder body; only syntax-aware plugins (Scala) implement this"
        );
        assert!(
            !PassthroughProbe.places_generated_units_in_place(),
            "T8b: the default is `false` - copy semantics (C++) keep the `generated/` \
             staging byte-identically (I1); move semantics must opt in explicitly"
        );
        assert_eq!(
            PassthroughProbe.unit_namespace("package p"),
            String::new(),
            "D-M23-2: the neutral namespace seam answers empty"
        );
    }
}
