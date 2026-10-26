//! [`RustClusteringConstraints`] - Rust's clustering constraints seam
//! (rust-language R-3, shape sec. 3.7; the F2 planning freeze was ADDED at
//! R-9b per the R9 AMENDMENT-1 sec. 2 ruling, 2026-10-26 - the R-3 "this
//! file does not grow" claim was superseded by that ruling and is
//! reworded here honestly, in the landing that grows it).
//!
//! Rust has no companion-object idiom: nothing is colocated by a language
//! rule, so `colocated_groups` answers empty vecs (the default
//! `colocated_groups_with_edges` inherits the same emptiness). There is no
//! sealed-hierarchy pairing either: the offline `syn` analysis cannot see
//! cross-file coherence that the compiler enforces crate-wide anyway.
//!
//! `frozen_symbols` freezes two provable classes at the language seam. The
//! IV3 stance, recorded as amended (AMENDMENT-1 sec. 2): the policy layer
//! owns freezing symbol INSTANCES; this rule freezes a symbol KIND plus
//! provably-ambiguous folds at the language layer - which is exactly what
//! the M18.7 constraints seam exists for ("what cannot move" is language
//! grammar):
//! - `kind == SymbolKind::Module`: a `mod x;` declaration is structure, its
//!   "body" IS the child file at its declared path, so an extraction
//!   targeting it is either a frozen-child clobber or a self-referential
//!   stub;
//! - `declarations.len() >= 2`: the flat-id sharing class. The analyzer
//!   gives every written item exactly one `<qualified-id>#L<line>` DeclId
//!   and the fold dedupes by DeclId identity, so two or more declarations
//!   are PROOF of a shared fold - deterministic regardless of fold order or
//!   folded kind.
//!
//! Honest limit (dated 2026-10-26, closeout deferral row + R-10 rider
//! material): shared-id symbols are never plannable; safe support would
//! require path-aware ids. That is the ACCEPTED cost of the flat-id
//! analyzer (R-5 D5), neutralized at the planning seam by this freeze, not
//! by re-qualification.

use ariadne_lang::{ClusteringConstraints, SymbolId, SymbolKind, SymbolRecord};
use std::collections::HashSet;

/// Rust's clustering constraints: no language-level colocation rule exists;
/// Module-kind records and provably-ambiguous shared folds (>= 2
/// declarations) are planning-frozen at this seam.
pub struct RustClusteringConstraints;

impl ClusteringConstraints for RustClusteringConstraints {
    fn colocated_groups(&self, _symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
        Vec::new()
    }

    /// Union of the two frozen classes from the module docs: Module-kind
    /// structure and >= 2-declaration shared folds. Everything else -
    /// ordinary fn/struct records - stays unfrozen (the freeze must not
    /// blast fn batches; the named pins lock both sides).
    fn frozen_symbols(&self, symbols: &[SymbolRecord]) -> HashSet<SymbolId> {
        symbols
            .iter()
            .filter(|record| {
                record.kind == SymbolKind::Module || record.declarations.len() >= 2
            })
            .map(|record| record.id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_lang::{DeclId, SymbolId, SymbolKind};

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

    /// F2 positive pin (amendment sec. 2): a `mod x;` declaration record is
    /// Module-kind structure - its "body" IS the child file, so extraction
    /// targeting it is clobber or a self-referential stub. Language grammar
    /// freezes it at the plugin's constraints seam (zero cluster/CLI edits);
    /// `colocated_groups` stays honestly empty.
    #[test]
    fn module_kind_records_are_planning_frozen() {
        let module_batch = vec![SymbolRecord {
            id: SymbolId::new("crate::models"),
            name: "models".to_string(),
            kind: SymbolKind::Module,
            ..SymbolRecord::default()
        }];
        let frozen = RustClusteringConstraints.frozen_symbols(&module_batch);
        assert!(
            frozen.contains(&SymbolId::new("crate::models")),
            "shape sec. 3.7 as amended: Module-kind records are planning-frozen at the language seam"
        );
        assert!(
            RustClusteringConstraints.colocated_groups(&module_batch).is_empty(),
            "the freeze is not a colocation group - no_colocated_groups stands"
        );
    }

    /// F2 positive pin (amendment sec. 2): a record folded from >= 2
    /// declarations is the flat-id sharing class - two written items folded
    /// to one id cannot honestly be extracted as one unit. >= 2 declarations
    /// is proof-of-shared-fold, deterministic regardless of fold order or
    /// folded kind (the analyzer gives every written item exactly one
    /// `<qualified-id>#L<line>` DeclId; the fold dedupes by DeclId
    /// identity).
    #[test]
    fn two_declaration_fold_records_are_planning_frozen() {
        let shared_fold = vec![SymbolRecord {
            id: SymbolId::new("crate::greeting"),
            name: "greeting".to_string(),
            kind: SymbolKind::Function,
            declarations: vec![
                DeclId::new("crate::greeting#L4"),
                DeclId::new("crate::greeting#L6"),
            ],
            ..SymbolRecord::default()
        }];
        let frozen = RustClusteringConstraints.frozen_symbols(&shared_fold);
        assert!(
            frozen.contains(&SymbolId::new("crate::greeting")),
            ">= 2 declarations proves a shared fold - planning-frozen (R-5 D5 neutralized at the planning seam)"
        );
        let single_declaration = vec![SymbolRecord {
            id: SymbolId::new("crate::fixture_marker"),
            name: "fixture_marker".to_string(),
            kind: SymbolKind::Function,
            declarations: vec![DeclId::new("crate::fixture_marker#L9")],
            ..SymbolRecord::default()
        }];
        assert!(
            RustClusteringConstraints.frozen_symbols(&single_declaration).is_empty(),
            "one declaration is an ordinary item - the freeze must not blast fn batches"
        );
    }
}
