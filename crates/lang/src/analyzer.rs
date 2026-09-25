//! Semantic analysis trait for language plugins.

use ariadne_index::TuAnalysis;
use ariadne_repo::FileId;

/// Analyses source text and returns a language-agnostic [`TuAnalysis`].
///
/// Each language plugin provides a concrete implementation that parses the
/// source and extracts symbols, declarations, and dependency edges.
pub trait SemanticAnalyzer: Send + Sync {
    /// Parse `source` (the full text of the file identified by `file_id`) and
    /// return a [`TuAnalysis`] containing symbols, declarations, edges, and
    /// call sites.
    ///
    /// `context` is a human-readable build-context name (e.g. `"Debug|x64"`
    /// for C++ or the Bloop project name for Scala).
    fn analyze(&self, file_id: FileId, source: &str, context: &str) -> TuAnalysis;
}
