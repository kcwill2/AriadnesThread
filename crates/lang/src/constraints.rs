//! Clustering constraint trait for language plugins.

use ariadne_index::{SymbolId, SymbolRecord};
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
}
