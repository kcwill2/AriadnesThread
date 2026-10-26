//! [`RustClusteringConstraints`] - the FINAL honest shape already
//! (rust-language R-3, shape sec. 3.7). This file does not grow.
//!
//! Rust has no companion-object idiom: nothing is colocated by a language
//! rule, so `colocated_groups` answers empty vecs (the default
//! `colocated_groups_with_edges` inherits the same emptiness). There is no
//! sealed-hierarchy pairing either: the offline `syn` analysis cannot see
//! cross-file coherence that the compiler enforces crate-wide anyway.
//! `frozen_symbols` stays empty (IV3: the policy layer owns freezing -
//! Scala/C++ parity, plan sec. 3.4 stance).

use ariadne_lang::{ClusteringConstraints, SymbolId, SymbolRecord};
use std::collections::HashSet;

/// Rust's clustering constraints: no language-level colocation rule exists,
/// and nothing is frozen at the language layer.
pub struct RustClusteringConstraints;

impl ClusteringConstraints for RustClusteringConstraints {
    fn colocated_groups(&self, _symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
        Vec::new()
    }

    fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> HashSet<SymbolId> {
        HashSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_lang::{SymbolId, SymbolKind};

    #[test]
    fn no_colocated_groups_and_no_frozen_symbols() {
        let batch = vec![
            SymbolRecord {
                id: SymbolId::new("crate::greeter"),
                name: "greeter".to_string(),
                kind: SymbolKind::Function,
                ..SymbolRecord::default()
            },
            SymbolRecord {
                id: SymbolId::new("crate::Greeter"),
                name: "Greeter".to_string(),
                kind: SymbolKind::Struct,
                ..SymbolRecord::default()
            },
        ];
        assert!(
            RustClusteringConstraints.colocated_groups(&batch).is_empty(),
            "shape sec. 3.7: no companion-object idiom in Rust - inventing a \
             group would fabricate a constraint the language never declared"
        );
        assert!(
            RustClusteringConstraints
                .colocated_groups_with_edges(&batch, &[])
                .is_empty(),
            "the edges-aware default inherits the same honest emptiness"
        );
        assert!(
            RustClusteringConstraints.frozen_symbols(&batch).is_empty(),
            "IV3: frozen stays empty - the policy layer owns freezing (Scala parity)"
        );
    }
}
