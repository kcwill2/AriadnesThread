//! Refactoring and header extraction engine.

mod extractor;
mod placement;

pub use extractor::HeaderExtractor;
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

#[derive(Debug, Clone)]
pub struct ExtractedSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub forward_declarable: bool,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct GeneratedHeader {
    pub filename: PathBuf,
    pub pragma_once: bool,
    pub symbols: Vec<ExtractedSymbol>,
    pub required_includes: Vec<PathBuf>,
}

pub struct ExtractionResult {
    pub plan_id: String,
    pub headers: Vec<GeneratedHeader>,
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
