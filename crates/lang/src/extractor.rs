//! Source file generation trait for language plugins.

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
}
