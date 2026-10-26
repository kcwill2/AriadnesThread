//! [`RustAnalyzer`] - honest never-dispatched stub (rust-language R-3).
//!
//! `capabilities().semantic_index` is `false` at this hop, so the pipeline
//! never dispatches here. The neutral empty [`TuAnalysis`] is the honest
//! answer - a compiling, tested neutral return, NOT `todo!()`/`unimplemented!()`
//! (SM-1's red rule applies to shipped code too). The real `syn` analyzer
//! lands at R-5 together with the fingerprint bump "rust-1" (INV-8).

use ariadne_lang::{SemanticAnalyzer, TuAnalysis};
use ariadne_repo::FileId;

/// Never-dispatched analyzer stub: returns the neutral empty analysis.
pub struct RustAnalyzer;

impl RustAnalyzer {
    pub fn new() -> Self {
        RustAnalyzer
    }
}

impl SemanticAnalyzer for RustAnalyzer {
    /// Dispatch is gated off (`semantic_index: false`); the file id is the
    /// only content carried back, everything else stays neutral-empty.
    fn analyze(&self, file_id: FileId, _source: &str, _context: &str) -> TuAnalysis {
        TuAnalysis {
            tu_file_id: file_id,
            ..TuAnalysis::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyze_returns_the_neutral_empty_analysis() {
        let file_id = FileId::from_relative_path("src/lib.rs");
        let analysis = RustAnalyzer::new().analyze(file_id, "fn main() {}", "root");
        assert_eq!(analysis.tu_file_id, file_id, "the file id is echoed verbatim");
        assert!(
            analysis.symbols.is_empty()
                && analysis.declarations.is_empty()
                && analysis.edges.is_empty(),
            "semantic_index is false, so this analysis is never dispatched; the \
             neutral empty surface is what shipped code must return - never todo!()"
        );
    }
}
