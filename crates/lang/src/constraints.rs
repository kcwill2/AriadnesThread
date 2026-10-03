//! Clustering constraint trait for language plugins.

use ariadne_lang_api::types::{SymbolId, SymbolRecord};
use std::collections::HashSet;

/// Language-specific rules that constrain the SCC-based extraction planner.
///
/// The planner calls these methods before emitting any [`ProposedExtraction`]
/// to ensure that language invariants are not violated. Examples:
/// - Scala: a class and its companion object must remain in the same file.
/// - Scala: a `sealed trait` and all its direct sub-types must stay together.
/// - C++: currently no hard constraints beyond policy (Contract/Frozen).
pub trait ClusteringConstraints: Send + Sync {
    /// Returns groups of symbol IDs that are **atomic** — they must always be
    /// extracted together into the same output file.
    ///
    /// The planner merges any proposed SCCs that overlap with these groups.
    fn colocated_groups(&self, symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>>;

    /// Returns symbols that must **not** be extracted regardless of policy.
    ///
    /// The planner skips any proposed extraction that includes a frozen symbol.
    fn frozen_symbols(&self, symbols: &[SymbolRecord]) -> HashSet<SymbolId>;

    /// [`ClusteringConstraints::colocated_groups`] with the candidate file's
    /// semantic edges attached (plan M23, D-M23-8): languages whose
    /// colocation rules need relations records cannot see (Scala:
    /// sealed-trait ↔ direct-subtype `extends` edges) get them here. The
    /// default **delegates to `colocated_groups` and ignores the edges** —
    /// exactly today's behavior — so C++ (and any language not opting in)
    /// is untouched. The planner call-site switch ships with the edge feed
    /// (M23 T7, gated on the sealed-edge ruling).
    fn colocated_groups_with_edges(
        &self,
        records: &[SymbolRecord],
        _edges: &[(SymbolId, SymbolId)],
    ) -> Vec<Vec<SymbolId>> {
        self.colocated_groups(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_lang_api::types::{SymbolId, SymbolRecord};

    struct OnlyNameGroups;
    impl ClusteringConstraints for OnlyNameGroups {
        fn colocated_groups(&self, symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
            symbols
                .iter()
                .filter(|r| r.name == "twin")
                .map(|r| vec![r.id.clone()])
                .collect()
        }
        fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> HashSet<SymbolId> {
            HashSet::new()
        }
    }

    #[test]
    fn colocated_groups_with_edges_default_delegates_and_ignores_edges() {
        let records = vec![SymbolRecord {
            id: SymbolId::new("a.twin"),
            name: "twin".to_string(),
            ..SymbolRecord::default()
        }];
        let edges = [(SymbolId::new("a.twin"), SymbolId::new("a.base"))];
        assert_eq!(
            OnlyNameGroups.colocated_groups_with_edges(&records, &edges),
            OnlyNameGroups.colocated_groups(&records),
            "D-M23-8: the default is byte-identity with colocated_groups"
        );
    }
}
