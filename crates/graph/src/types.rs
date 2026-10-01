//! Data models for include graph, edges, closures, and reports.

use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// `IncludeKind`, `SourceLocation`, and `IncludeEdge` are the include-graph
// artifact DTOs. They live in `ariadne-lang-api` (the shared artifact
// vocabulary) since M18.1 because the `ariadne-lang-cpp` resolver produces
// them and plugin crates may not depend on this one. The re-export keeps
// every existing path (`ariadne_graph::IncludeEdge`, ...) and the serde/
// bincode shapes unchanged.
pub use ariadne_lang_api::types::{IncludeEdge, IncludeKind, SourceLocation};

/// Transitive include closure for a specific file or translation unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncludeClosure {
    pub file_id: FileId,
    pub direct_includes: Vec<FileId>,
    pub transitive_includes: Vec<FileId>,
    pub closure_size: usize,
}

impl IncludeClosure {
    pub fn new(
        file_id: FileId,
        mut direct_includes: Vec<FileId>,
        mut transitive_includes: Vec<FileId>,
    ) -> Self {
        direct_includes.sort();
        direct_includes.dedup();
        transitive_includes.sort();
        transitive_includes.dedup();

        let closure_size = transitive_includes.len();

        Self {
            file_id,
            direct_includes,
            transitive_includes,
            closure_size,
        }
    }
}

/// Aggregated summary report of repository-wide include graph analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncludeReport {
    pub total_edges: usize,
    pub unique_headers: usize,
    pub local_headers_count: usize,
    pub system_headers_count: usize,
    pub external_headers_count: usize,
    pub unresolved_includes_count: usize,
    pub cycle_count: usize,
    /// Cycles whose every member file tolerates cycles per its language's
    /// `CyclePolicy` (M20 D11). Only ever non-zero through
    /// `generate_report_with_cycle_policies`; skipped when 0 so every
    /// pre-M20 artifact (all C++ paths) keeps a byte-identical `report.json`.
    #[serde(default, skip_serializing_if = "informational_cycles_absent")]
    pub informational_cycle_count: usize,
    pub top_headers_by_closure: Vec<(FileId, PathBuf, usize)>,
}

/// Serde skip predicate for `IncludeReport::informational_cycle_count`: the
/// key is absent from serialized reports exactly when no cycle is
/// informational (the shape every artifact had before M20).
fn informational_cycles_absent(count: &usize) -> bool {
    *count == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_closure_normalization() {
        let f0 = FileId(1);
        let f1 = FileId(2);
        let f2 = FileId(3);
        let closure = IncludeClosure::new(f0, vec![f2, f1, f1], vec![f2, f1, f2]);

        assert_eq!(closure.file_id, f0);
        assert_eq!(closure.direct_includes, vec![f1, f2]);
        assert_eq!(closure.transitive_includes, vec![f1, f2]);
        assert_eq!(closure.closure_size, 2);
    }
}
