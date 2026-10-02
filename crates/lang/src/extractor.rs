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
}

/// Render the complete text of one generated unit — **the single emitter**
/// for generated-file bytes (plan M23, invariant I3 / D-M23-4). The
/// isolation engine's disk write and the transactional applier both route
/// through here, so the namespace-seam rule lives in exactly one place:
/// [`SourceExtractor::unit_namespace`] is consulted only when
/// `primary_source_text` is `Some` (D-M23-2); otherwise no hook call is
/// made and the namespace stays empty (C++ bytes stay unchanged).
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
    extractor
        .extract(symbol_bodies, &namespace, required_deps)
        .content
}
