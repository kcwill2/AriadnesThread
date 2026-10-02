//! [`ScalaClusteringConstraints`] (M22.1a, decision D24) — the language
//! half of the clustering-constraint seam the planner injects
//! (`crates/cluster/src/planner.rs` `constraints`).
//!
//! Visible in the per-batch `&[SymbolRecord]` input (IV3):
//! - **Companion object↔class pairs**: D17 gives the companion object the
//!   id `<qualified-class-id>$`; the constraints group a `Class` record
//!   `X` with an `Object` record `X$` when both are in the batch.
//!
//! NOT visible in that input, and therefore never claimed (locked by
//! `sealed_pairing_scope_locked`):
//! - **Sealed trait ↔ sub-type pairing**: `SymbolRecord` carries no
//!   modifier flags (the parser consumes `sealed` as a plain modifier, see
//!   `parser::declarations::is_modifier_keyword`) and no parent edges —
//!   those live on `TuAnalysis::edges`, which the `ClusteringConstraints`
//!   trait never receives. A `sealed trait` and its sub-types cannot be
//!   recognized from records alone. Cross-file sealed members are doubly
//!   invisible: the offline parser cannot see same-package siblings in
//!   other files at all (D16 limitation), and a batch is only ever what
//!   the planner puts in it. Revisit when M23 lands real bodies or when
//!   the record slice grows modifiers/edges.
//! - `frozen_symbols` stays empty (plan §3.4: the policy layer owns
//!   freezing, same stance as C++).

use ariadne_lang::{ClusteringConstraints, SymbolId, SymbolKind, SymbolRecord};
use std::collections::{BTreeSet, HashSet};

/// Scala's clustering constraints: companion object↔class pairs are
/// atomic (D17 `$`-id convention, IV3); nothing is frozen at the language
/// layer.
pub struct ScalaClusteringConstraints;

/// Group every `Object` record whose id is `<id>$` with the `Class`
/// record `<id>` present in the same batch. Groups and group members are
/// emitted in id order, so the result never depends on batch order;
/// duplicate records fold into one group.
fn companion_groups(symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
    let class_ids: BTreeSet<&str> = symbols
        .iter()
        .filter(|record| record.kind == SymbolKind::Class)
        .map(|record| record.id.0.as_str())
        .collect();
    let mut groups: BTreeSet<Vec<&str>> = BTreeSet::new();
    for record in symbols
        .iter()
        .filter(|record| record.kind == SymbolKind::Object)
    {
        // False-pair guard: a D17 companion carries the `$` on its id only
        // (`name` stays the source identifier). An object literally named
        // `Foo$` is not a companion id — stripping here would pair it with
        // an unrelated `Foo`.
        if record.name.ends_with('$') {
            continue;
        }
        if let Some(class_id) = record.id.0.strip_suffix('$') {
            if class_ids.contains(class_id) {
                groups.insert(vec![class_id, record.id.0.as_str()]);
            }
        }
    }
    groups
        .into_iter()
        .map(|pair| pair.into_iter().map(SymbolId::new).collect())
        .collect()
}

impl ClusteringConstraints for ScalaClusteringConstraints {
    fn colocated_groups(&self, symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
        companion_groups(symbols)
    }

    fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> HashSet<SymbolId> {
        HashSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_lang::{SymbolId, SymbolKind, SymbolRecord};

    fn record(id: &str, kind: SymbolKind) -> SymbolRecord {
        SymbolRecord {
            id: SymbolId::new(id),
            name: id.rsplit('.').next().unwrap_or(id).to_string(),
            kind,
            ..SymbolRecord::default()
        }
    }

    fn group_ids(groups: &[Vec<SymbolId>]) -> Vec<Vec<String>> {
        groups
            .iter()
            .map(|g| g.iter().map(|s| s.0.clone()).collect())
            .collect()
    }

    #[test]
    fn companion_pairs_colocate() {
        // D17 parser shape: the companion object's `name` is the source
        // identifier `Foo`; only the id carries the `$`.
        let companion = SymbolRecord {
            name: "Foo".to_string(),
            ..record("com.example.Foo$", SymbolKind::Object)
        };
        let batch = vec![record("com.example.Foo", SymbolKind::Class), companion];
        let groups = ScalaClusteringConstraints.colocated_groups(&batch);
        assert_eq!(
            group_ids(&groups),
            vec![vec![
                "com.example.Foo".to_string(),
                "com.example.Foo$".to_string()
            ]],
            "IV3/D24: a class and its D17 `$` companion object in one batch are atomic"
        );
    }

    #[test]
    fn companion_pairing_only_fires_on_the_dollar_id_convention() {
        let batch = vec![
            record("com.example.Bar", SymbolKind::Class),
            record("com.example.Baz", SymbolKind::Object),
            record("com.example.Qux", SymbolKind::Class),
            record("com.example.Qux", SymbolKind::Object),
        ];
        let groups = ScalaClusteringConstraints.colocated_groups(&batch);
        assert!(
            groups.is_empty(),
            "unrelated and non-`$` symbols form no group: the planner leaves each in its own \
             singleton batch unchanged (a singleton is not a constraint), so unrelated symbols \
             stay colocated with nothing"
        );
    }

    #[test]
    fn lone_dollar_named_object_does_not_pair() {
        // An object whose SOURCE name is literally `Foo$` (D17 appends the
        // `$` only to the id of a class-paired companion, never to `name`)
        // must not pair an unrelated `Foo` class through `$`-stripping.
        let literal = SymbolRecord {
            name: "Foo$".to_string(),
            ..record("com.example.Foo$", SymbolKind::Object)
        };
        let batch = vec![record("com.example.Foo", SymbolKind::Class), literal];
        assert!(
            ScalaClusteringConstraints.colocated_groups(&batch).is_empty(),
            "a `$`-named object is not a D17 companion id — pairing it with an \
             unrelated class invents a constraint the source never declared"
        );
    }

    #[test]
    fn sealed_pairing_scope_locked() {
        let batch = vec![
            record("com.example.models.Shape", SymbolKind::Trait),
            record("com.example.models.Circle", SymbolKind::Class),
            record("com.example.models.Square", SymbolKind::Class),
        ];
        assert!(
            ScalaClusteringConstraints.colocated_groups(&batch).is_empty(),
            "locked limitation: a sealed trait and its sub-types produce no group because \
             SymbolRecord carries neither the `sealed` modifier nor parent edges — see the \
             module docs (cross-file sealed members are invisible to the offline parser anyway)"
        );
        assert!(
            ScalaClusteringConstraints
                .frozen_symbols(&batch)
                .is_empty(),
            "IV3: frozen stays empty — the policy layer owns freezing (plan §3.4)"
        );
    }
}
