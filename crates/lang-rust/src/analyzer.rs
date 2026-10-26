//! [`RustAnalyzer`] - the `syn`-backed [`SemanticAnalyzer`] (rust-language
//! R-5; replaces the never-dispatched stub in the commit that also lands the
//! `"rust-1"` fingerprint, INV-8).
//!
//! D-2 mechanism: `syn::parse_file` over the written source; every line and
//! column comes from `proc_macro2` with the `span-locations` feature (the
//! INV-7 pins fail if that feature ever vanishes - without it the accessors
//! do not exist at all, so the crate compiling with spans consumed is the
//! secondary oracle, RISK-SYN-SPAN). The walker is pre-split per shape sec.
//! 7: written items -> [`crate::parser_items`], `use` facts ->
//! [`crate::parser_uses`]; this file only orchestrates.
//!
//! Honesty contract (D-2): macro content is parsed only as far as `syn`
//! accepts it - nothing unexpanded is invented (the fixture-driven macro
//! caveat in the integration suite pins it). A file `syn` cannot parse gets
//! the neutral EMPTY analysis: an unparseable file has no honest symbol
//! surface, and fabricating one from a failed parse is the violation this
//! track names. The file id, context, and language stamp are always carried.

use crate::parser_items::{self, ROOT_PREFIX};
use crate::parser_uses;
use ariadne_lang::{SemanticAnalyzer, TuAnalysis};
use ariadne_repo::{FileId, SourceLanguage};
use syn::Item;

/// Stateless Rust analysis entry point wired into
/// [`crate::plugin::RustPlugin`] as of R-5; with `capabilities().semantic_index`
/// flipped true the pipeline dispatches here for every Rust unit.
pub struct RustAnalyzer;

impl RustAnalyzer {
    pub fn new() -> Self {
        RustAnalyzer
    }
}

impl Default for RustAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer for RustAnalyzer {
    fn analyze(&self, file_id: FileId, source: &str, context: &str) -> TuAnalysis {
        let mut analysis = TuAnalysis {
            tu_file_id: file_id,
            context_name: context.to_string(),
            language: SourceLanguage::Rust,
            ..TuAnalysis::default()
        };
        if let Ok(parsed) = syn::parse_file(source) {
            walk_items(&parsed.items, ROOT_PREFIX, file_id, &mut analysis);
        }
        analysis
    }
}

/// Source-order walk: `use` statements leave through `parser_uses`, every
/// other item through `parser_items`; an inline `mod` block's written
/// content descends one prefix level (recursion is bounded by the file's own
/// written nesting, which `syn` already accepted to parse).
fn walk_items(items: &[Item], module_prefix: &str, file_id: FileId, analysis: &mut TuAnalysis) {
    for item in items {
        match item {
            Item::Use(use_item) => parser_uses::record_use(use_item, file_id, analysis),
            _ => {
                if let Some(inline_module) =
                    parser_items::record_item(item, module_prefix, file_id, analysis)
                {
                    walk_items(inline_module.items, &inline_module.prefix, file_id, analysis);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze_inline(source: &str) -> TuAnalysis {
        RustAnalyzer::new().analyze(FileId::from_relative_path("src/lib.rs"), source, "root")
    }

    #[test]
    fn analyze_stamps_file_context_and_language() {
        let file_id = FileId::from_relative_path("src/lib.rs");
        let analysis = analyze_inline("pub fn alpha() {}\n");
        assert_eq!(analysis.tu_file_id, file_id, "the parser owns tu_file_id");
        assert_eq!(analysis.context_name, "root", "context is stamped verbatim");
        assert_eq!(analysis.language, SourceLanguage::Rust);
        assert_eq!(analysis.symbols.len(), 1, "the written fn is on the surface");
    }

    #[test]
    fn analyze_unparseable_source_stays_neutrally_empty() {
        let analysis = analyze_inline("pub fn broken(");
        assert!(
            analysis.symbols.is_empty()
                && analysis.declarations.is_empty()
                && analysis.references.is_empty()
                && analysis.edges.is_empty(),
            "a failed parse has no honest symbol surface - nothing is invented"
        );
        assert_eq!(analysis.context_name, "root", "the stamps stay honest anyway");
        assert_eq!(analysis.language, SourceLanguage::Rust);
    }

    /// R-5 in-src span pin (INV-7): named written items carry their EXACT
    /// 1-based name-token line and column. Authored red against the stub
    /// (`left: []`); green makes it literal-exact.
    #[test]
    fn analyze_carries_exact_name_spans_on_written_items() {
        let source = "pub fn alpha() {}\npub struct Beta { pub field: u32 }\n";
        let analysis = analyze_inline(source);
        let spans: Vec<(String, usize, usize)> = analysis
            .declarations
            .iter()
            .map(|decl| {
                (
                    decl.symbol_id.0.clone(),
                    decl.location.line,
                    decl.location.column,
                )
            })
            .collect();
        assert_eq!(
            spans,
            vec![
                ("crate::alpha".to_string(), 1, 8),
                ("crate::Beta".to_string(), 2, 12),
            ],
            "INV-7: every written item's declaration sits at its exact 1-based \
             name-token position (proc_macro2 span-locations, column+1)"
        );
    }

    #[test]
    fn analyzer_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RustAnalyzer>();
    }
}
