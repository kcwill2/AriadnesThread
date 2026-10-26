//! [`RustImportResolver`] - honest never-dispatched stub (rust-language R-3).
//!
//! `capabilities().dependency_graph` is `false` at this hop, so the graph
//! layer never dispatches here. The empty edge vec is the honest answer - a
//! compiling, tested neutral return, not a panic (SM-1's red rule). The real
//! resolver (`mod x;` file mapping + `use`-path resolution through the
//! discovery catalog, carrying line/col on every `RawDepEdge`) lands at R-6.

use ariadne_lang::{DependencyResolver, RawDepEdge, ResolutionContext};
use ariadne_repo::FileId;
use std::path::Path;

/// Never-dispatched resolver stub: emits no edges.
pub struct RustImportResolver;

impl DependencyResolver for RustImportResolver {
    fn resolve_edges(
        &self,
        _file_id: FileId,
        _rel_path: &Path,
        _source: &str,
        _ctx: &ResolutionContext<'_>,
    ) -> Vec<RawDepEdge> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_edges_returns_no_edges() {
        let edges = RustImportResolver.resolve_edges(
            FileId::from_relative_path("src/lib.rs"),
            Path::new("src/lib.rs"),
            "use serde::Serialize;\nmod greeting;\n",
            &ResolutionContext::default(),
        );
        assert!(
            edges.is_empty(),
            "dependency_graph is false, so these edges are never consumed; the \
             empty vec is honest - the mod-tree resolver lands at R-6"
        );
    }
}
