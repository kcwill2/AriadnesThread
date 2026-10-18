//! Semantic index value types.
//!
//! The shared plugin vocabulary types now live in `ariadne-lang-api`
//! (plan §3.1); this module re-exports them verbatim so every existing path
//! (`ariadne_index::types::SymbolId`, the crate-root `pub use types::*`, and
//! `ariadne_index::SymbolId`) keeps resolving with zero call-site churn.
//! `IndexResult` stays here — it is an index product, not plugin vocabulary.

pub use ariadne_lang_api::types::*;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexResult {
    pub repo_root: PathBuf,
    pub tu_analyses: Vec<TuAnalysis>,
    pub total_symbols: usize,
    pub total_edges: usize,
    pub complete_edges: usize,
    pub incomplete_edges: usize,
}

impl IndexResult {
    pub fn new(repo_root: PathBuf, tu_analyses: Vec<TuAnalysis>) -> Self {
        let mut total_symbols = 0;
        let mut total_edges = 0;
        let mut complete_edges = 0;
        let mut incomplete_edges = 0;

        let mut seen_symbols = std::collections::HashSet::new();

        for tu in &tu_analyses {
            for sym in &tu.symbols {
                if seen_symbols.insert(sym.id.clone()) {
                    total_symbols += 1;
                }
            }
            total_edges += tu.edges.len();
            for edge in &tu.edges {
                match edge.kind {
                    DependencyKind::Complete => complete_edges += 1,
                    DependencyKind::Incomplete => incomplete_edges += 1,
                }
            }
        }

        Self {
            repo_root,
            tu_analyses,
            total_symbols,
            total_edges,
            complete_edges,
            incomplete_edges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_repo::{FileId, SourceLanguage};

    #[test]
    fn test_types_and_index_result_aggregation() {
        let file1 = FileId::from_relative_path("src/foo.cpp");
        let file2 = FileId::from_relative_path("include/foo.h");

        let sym_a = SymbolId::new("A");
        let sym_b = SymbolId::new("B");

        let decl_a = DeclId::new("decl_a_1");

        let symbol_rec_a = SymbolRecord {
            id: sym_a.clone(),
            name: "A".into(),
            kind: SymbolKind::Class,
            owning_file: file2.clone(),
            declarations: vec![decl_a.clone()],
            definition: Some(decl_a),

            modifiers: Vec::new(),
        };

        let edge = SemanticEdge {
            src_symbol: sym_a,
            dst_symbol: sym_b,
            kind: DependencyKind::Complete,
            src_file: file1.clone(),
            dst_file: file2,
        };

        let tu = TuAnalysis {
            tu_file_id: file1,
            context_name: "Debug|x64".into(),
            symbols: vec![symbol_rec_a],
            declarations: vec![],
            edges: vec![edge],
            references: vec![],
            call_sites: vec![],
            language: SourceLanguage::Cpp,

            extends_edges: Vec::new(),
        };

        let result = IndexResult::new(PathBuf::from("/test"), vec![tu]);
        assert_eq!(result.total_symbols, 1);
        assert_eq!(result.total_edges, 1);
        assert_eq!(result.complete_edges, 1);
        assert_eq!(result.incomplete_edges, 0);
    }

    // ---- UTA-T5 item 14: IndexResult::new aggregation pins -----------------

    fn pin_file(rel: &str) -> FileId {
        FileId::from_relative_path(rel)
    }

    fn pin_symbol(name: &str, owner: FileId) -> SymbolRecord {
        SymbolRecord {
            id: SymbolId::new(name),
            name: name.into(),
            kind: SymbolKind::Class,
            owning_file: owner,
            declarations: vec![],
            definition: None,

            modifiers: Vec::new(),
        }
    }

    fn pin_edge(
        src: &str,
        dst: &str,
        kind: DependencyKind,
        src_file: FileId,
        dst_file: FileId,
    ) -> SemanticEdge {
        SemanticEdge {
            src_symbol: SymbolId::new(src),
            dst_symbol: SymbolId::new(dst),
            kind,
            src_file,
            dst_file,
        }
    }

    #[test]
    fn index_result_dedups_shared_symbol_and_tallies_edge_kinds_across_two_tus() {
        let header = pin_file("include/shared.h");
        let unit_a = pin_file("src/a.cpp");
        let unit_b = pin_file("src/b.cpp");

        // The same symbol id arrives from BOTH translation units (shared
        // header): the count must dedup by symbol id, not count per TU.
        let shared_symbol = pin_symbol("Shared", header.clone());
        let edge = pin_edge(
            "Shared",
            "Target",
            DependencyKind::Complete,
            unit_a.clone(),
            unit_b.clone(),
        );
        let incomplete_edge = pin_edge(
            "Shared",
            "Other",
            DependencyKind::Incomplete,
            unit_b.clone(),
            header.clone(),
        );

        let analysis_a = TuAnalysis {
            tu_file_id: unit_a,
            context_name: "default".into(),
            symbols: vec![shared_symbol.clone()],
            declarations: vec![],
            edges: vec![edge],
            references: vec![],
            call_sites: vec![],
            language: SourceLanguage::Cpp,

            extends_edges: Vec::new(),
        };
        let analysis_b = TuAnalysis {
            tu_file_id: unit_b,
            context_name: "default".into(),
            symbols: vec![shared_symbol, pin_symbol("Private", header.clone())],
            declarations: vec![],
            edges: vec![incomplete_edge],
            references: vec![],
            call_sites: vec![],
            language: SourceLanguage::Cpp,

            extends_edges: Vec::new(),
        };

        let result = IndexResult::new(PathBuf::from("/pin"), vec![analysis_a, analysis_b]);
        // Shared "Shared" counted once, "Private" once: two distinct ids.
        assert_eq!(result.total_symbols, 2);
        // Edges are NOT deduped: one per TU.
        assert_eq!(result.total_edges, 2);
        assert_eq!(result.complete_edges, 1);
        assert_eq!(result.incomplete_edges, 1);
        // The analyses themselves are kept verbatim.
        assert_eq!(result.tu_analyses.len(), 2);
    }

    #[test]
    fn index_result_over_empty_analyses_is_all_zero() {
        let result = IndexResult::new(PathBuf::from("/pin"), vec![]);
        assert_eq!(result.total_symbols, 0);
        assert_eq!(result.total_edges, 0);
        assert_eq!(result.complete_edges, 0);
        assert_eq!(result.incomplete_edges, 0);
        assert!(result.tu_analyses.is_empty());
        assert_eq!(result.repo_root, PathBuf::from("/pin"));
    }
}
