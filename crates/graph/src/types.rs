//! Data models for include graph, edges, closures, and reports.

use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Classification of an included header file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum IncludeKind {
    /// Local header found via relative search or local `-I` directory.
    Local,
    /// System header found via `<...>` or system include paths.
    System,
    /// External third-party library header.
    External,
}

/// Source location of an `#include` directive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
}

impl SourceLocation {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

/// Directed include edge representing `#include` from source to destination.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IncludeEdge {
    pub source: FileId,
    pub destination: Option<FileId>,
    pub context_name: String,
    pub source_location: SourceLocation,
    pub kind: IncludeKind,
    pub raw_path: String,
    pub resolved_path: Option<PathBuf>,
}

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
    pub top_headers_by_closure: Vec<(FileId, PathBuf, usize)>,
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

    #[test]
    fn test_include_edge_serialization() {
        let edge = IncludeEdge {
            source: FileId(10),
            destination: Some(FileId(20)),
            context_name: "Debug|x64".to_string(),
            source_location: SourceLocation::new(4, 1),
            kind: IncludeKind::Local,
            raw_path: "thing_types.h".to_string(),
            resolved_path: Some(PathBuf::from("include/thing_types.h")),
        };

        let json = serde_json::to_string(&edge).unwrap();
        let decoded: IncludeEdge = serde_json::from_str(&json).unwrap();
        assert_eq!(edge, decoded);

        let bin = bincode::serialize(&edge).unwrap();
        let decoded_bin: IncludeEdge = bincode::deserialize(&bin).unwrap();
        assert_eq!(edge, decoded_bin);
    }
}
