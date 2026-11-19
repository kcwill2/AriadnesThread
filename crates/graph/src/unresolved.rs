//! Breakdown of unresolved include edges.
//!
//! The include report's "Unresolved Includes" figure counts EDGES, so one
//! `#include <vector>` repeated across ten thousand files counts ten
//! thousand times. On a large repo that single number hides whether the
//! misses are toolchain headers AT has no search path for (`<vector>`,
//! `<windows.h>`) or the repo's own headers behind a missing include dir.
//! This breakdown splits the misses by spelling and ranks the spellings.

use crate::graph::IncludeGraph;
use crate::types::IncludeKind;
use std::collections::HashMap;

/// Unresolved edges split by how the directive was spelled, plus the most
/// frequent unresolved spellings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedBreakdown {
    /// `#include <...>` edges with no destination.
    pub angled_edges: usize,
    /// `#include "..."` edges with no destination.
    pub quoted_edges: usize,
    /// Distinct unresolved spellings (angled and quoted together).
    pub distinct_spellings: usize,
    /// `(spelling, is_angled, edge count)`, most frequent first, ties by
    /// spelling; at most the requested number of rows.
    pub top_spellings: Vec<(String, bool, usize)>,
}

impl IncludeGraph {
    /// Tallies the unresolved edges. An unresolved edge's kind records its
    /// spelling (angled => `System`, quoted => `Local`; see the builder's
    /// `style_kind`), so no re-parse is needed.
    pub fn unresolved_breakdown(&self, top_rows: usize) -> UnresolvedBreakdown {
        let mut counts: HashMap<(&str, bool), usize> = HashMap::new();
        let mut angled_edges = 0;
        let mut quoted_edges = 0;
        for edge in self.edges.iter().filter(|edge| edge.destination.is_none()) {
            let is_angled = edge.kind == IncludeKind::System;
            if is_angled {
                angled_edges += 1;
            } else {
                quoted_edges += 1;
            }
            *counts.entry((edge.raw_path.as_str(), is_angled)).or_insert(0) += 1;
        }
        let distinct_spellings = counts.len();
        let mut ranked: Vec<(String, bool, usize)> = counts
            .into_iter()
            .map(|((spelling, is_angled), count)| (spelling.to_string(), is_angled, count))
            .collect();
        ranked.sort_by(|left, right| right.2.cmp(&left.2).then_with(|| left.0.cmp(&right.0)));
        ranked.truncate(top_rows);
        UnresolvedBreakdown {
            angled_edges,
            quoted_edges,
            distinct_spellings,
            top_spellings: ranked,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::IncludeEdge;
    use ariadne_lang_api::identity::LanguageId;
    use ariadne_lang_api::types::SourceLocation;
    use ariadne_repo::FileId;

    fn edge(raw_path: &str, kind: IncludeKind, resolved: bool) -> IncludeEdge {
        IncludeEdge {
            source: FileId(1),
            destination: resolved.then_some(FileId(2)),
            context_name: "Default|x64".to_string(),
            source_location: SourceLocation::new(1, 1),
            kind,
            raw_path: raw_path.to_string(),
            resolved_path: None,
            language: LanguageId::cpp(),
        }
    }

    #[test]
    fn splits_unresolved_edges_by_spelling_and_ranks_them() {
        let mut graph = IncludeGraph::new();
        for _ in 0..3 {
            graph.edges.push(edge("vector", IncludeKind::System, false));
        }
        graph.edges.push(edge("windows.h", IncludeKind::System, false));
        graph.edges.push(edge("Common/Foo.h", IncludeKind::Local, false));
        graph.edges.push(edge("Common/Foo.h", IncludeKind::Local, false));
        graph.edges.push(edge("found.h", IncludeKind::Local, true));

        let breakdown = graph.unresolved_breakdown(2);

        assert_eq!(breakdown.angled_edges, 4);
        assert_eq!(breakdown.quoted_edges, 2);
        assert_eq!(breakdown.distinct_spellings, 3);
        assert_eq!(
            breakdown.top_spellings,
            vec![
                ("vector".to_string(), true, 3),
                ("Common/Foo.h".to_string(), false, 2),
            ]
        );
    }

    #[test]
    fn a_fully_resolved_graph_has_an_empty_breakdown() {
        let mut graph = IncludeGraph::new();
        graph.edges.push(edge("found.h", IncludeKind::Local, true));
        let breakdown = graph.unresolved_breakdown(10);
        assert_eq!(breakdown.angled_edges + breakdown.quoted_edges, 0);
        assert!(breakdown.top_spellings.is_empty());
    }
}
