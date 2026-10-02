//! Symbol isolation engine and consumer-rewrite machinery (plan §3.6).
//!
//! The engine is language-neutral: it groups planned extractions into
//! [`GeneratedUnit`]s, computes inter-symbol dependency edges, checks the
//! unit graph for cycles, and writes files through a plugin-provided
//! [`SourceExtractor`](ariadne_lang::SourceExtractor) text hook. All
//! language-specific new-file text lives in the language plugins.

mod engine;
mod placement;

pub use engine::IsolationEngine;
pub use placement::{ConsumerRewrite, IncludeRewriter, MigrationPreview};

use ariadne_index::SymbolKind;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct TransformEngine;

impl TransformEngine {
    pub fn new() -> Self {
        Self
    }
}

/// One extracted symbol: its name, semantic kind, and body text.
#[derive(Debug, Clone)]
pub struct ExtractedSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub body: String,
}

/// One generated file: where it lands and what it must depend on.
#[derive(Debug, Clone)]
pub struct GeneratedUnit {
    /// Relative path of the file the engine will write.
    pub path: PathBuf,
    /// Symbols extracted into this unit.
    pub symbols: Vec<ExtractedSymbol>,
    /// Other units this unit depends on (rendered by the plugin's extractor).
    pub required_deps: Vec<PathBuf>,
    /// Raw text of the unit's primary source file, when the engine could read
    /// it at extract time (plan M23, D-M23-2/D-M23-3). Drives the
    /// `SourceExtractor::unit_namespace` seam on write; `None` means no source
    /// text was available (placeholder bodies, empty namespace).
    pub primary_source_text: Option<String>,
}

pub struct ExtractionResult {
    pub plan_id: String,
    pub headers: Vec<GeneratedUnit>,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_engine_init() {
        let _te = TransformEngine::new();
    }
}
