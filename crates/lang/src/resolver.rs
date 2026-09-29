//! Inter-file dependency resolution trait for language plugins.

use ariadne_lang_api::catalog::{FileCatalog, ResolveOutcome};
use ariadne_repo::FileId;
use std::path::Path;

/// A single raw dependency directive parsed from source text, before any
/// path resolution to a concrete [`FileId`] is attempted.
///
/// For C++ this represents a `#include` directive; for Scala an `import`
/// statement; for other languages the equivalent concept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDepEdge {
    /// The file containing the directive.
    pub source_file: FileId,
    /// The raw target string as it appears in source (e.g. `"util/hash.h"` or
    /// `"com.example.models.User"`).
    pub raw_target: String,
    /// `true` for wildcard/glob imports (`import com.example._`,
    /// `#include <*>`).
    pub is_wildcard: bool,
    /// 1-based source line of the directive.
    pub line: u32,
}

/// Minimal context passed to a [`DependencyResolver`] during edge resolution.
pub struct ResolutionContext<'a> {
    /// Absolute path to the repository root.
    pub repo_root: &'a Path,
}

/// Parses dependency directives from source text and emits [`RawDepEdge`]s.
///
/// Resolution to concrete [`FileId`]s is intentionally left to the graph
/// layer so that this trait stays pure and testable without a live filesystem.
pub trait DependencyResolver: Send + Sync {
    /// Return all dependency edges found in `source`.
    fn resolve_edges(
        &self,
        file_id: FileId,
        rel_path: &Path,
        source: &str,
        ctx: &ResolutionContext<'_>,
    ) -> Vec<RawDepEdge>;

    /// Maps a raw directive to a repository file. Default: unresolved.
    /// C++: include-dir + basename search. Scala: package → directory mapping.
    fn resolve_target(
        &self,
        edge: &RawDepEdge,
        catalog: &FileCatalog,
        ctx: &ResolutionContext<'_>,
    ) -> ResolveOutcome {
        let _ = (edge, catalog, ctx);
        ResolveOutcome::Unresolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_dep_edge_fields() {
        let file_id = FileId::from_relative_path("src/main.scala");
        let edge = RawDepEdge {
            source_file: file_id,
            raw_target: "com.example.models._".into(),
            is_wildcard: true,
            line: 3,
        };
        assert!(edge.is_wildcard);
        assert_eq!(edge.line, 3);
    }

    struct ParseOnlyResolver;
    impl DependencyResolver for ParseOnlyResolver {
        fn resolve_edges(
            &self,
            _file_id: FileId,
            _rel_path: &Path,
            _source: &str,
            _ctx: &ResolutionContext<'_>,
        ) -> Vec<RawDepEdge> {
            vec![]
        }
    }

    #[test]
    fn default_resolve_target_is_unresolved() {
        let ctx = ResolutionContext {
            repo_root: Path::new("."),
        };
        let edge = RawDepEdge {
            source_file: FileId::from_relative_path("src/a.cpp"),
            raw_target: "b.h".into(),
            is_wildcard: false,
            line: 1,
        };
        assert_eq!(
            ParseOnlyResolver.resolve_target(&edge, &FileCatalog::default(), &ctx),
            ResolveOutcome::Unresolved
        );
    }
}
