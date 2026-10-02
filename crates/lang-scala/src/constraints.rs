//! [`ScalaClusteringConstraints`] (M22.1a, decision D24) — the language
//! half of the clustering-constraint seam the planner injects
//! (`crates/cluster/src/planner.rs` `constraints`).
//!
//! Visible in the per-batch `&[SymbolRecord]` input (IV3):
//! - **Companion object↔class pairs**: D17 gives the companion object the
//!   id `<qualified-class-id>$`; the constraints group a `Class` record
//!   `X` with an `Object` record `X$` when both are in the batch.
//!
//! - **Sealed trait ↔ sub-type pairing**: realized by
//!   [`ClusteringConstraints::colocated_groups_with_edges`] (plan M23,
//!   D-M23-8): the planner feeds the candidate file's typed extends channel
//!   and the records carry the `sealed` modifier, so a sealed trait groups
//!   with its direct sub-types (locked positively by
//!   `sealed_trait_pairs_direct_subtypes_with_edges`). The records-only
//!   [`ClusteringConstraints::colocated_groups`] path keeps the M22 gap
//!   locked (`sealed_pairing_needs_both_modifier_and_edge`). Cross-file
//!   sealed members stay invisible: the offline parser cannot see
//!   same-package siblings in other files at all (D16 limitation), and a
//!   batch is only ever what the planner puts in it.
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

    fn sealed_batch() -> Vec<SymbolRecord> {
        vec![
            SymbolRecord {
                modifiers: vec!["sealed".to_string()],
                ..record("com.example.models.Shape", SymbolKind::Trait)
            },
            record("com.example.models.Circle", SymbolKind::Class),
            record("com.example.models.Square", SymbolKind::Class),
            record("com.example.models.Base", SymbolKind::Trait),
            record("com.example.models.Lone", SymbolKind::Class),
        ]
    }

    fn sealed_edges() -> Vec<(SymbolId, SymbolId)> {
        vec![
            (
                SymbolId::new("com.example.models.Circle"),
                SymbolId::new("com.example.models.Shape"),
            ),
            (
                SymbolId::new("com.example.models.Square"),
                SymbolId::new("com.example.models.Shape"),
            ),
            (
                SymbolId::new("com.example.models.Lone"),
                SymbolId::new("com.example.models.Base"),
            ),
        ]
    }

    #[test]
    fn sealed_trait_pairs_direct_subtypes_with_edges() {
        let groups =
            ScalaClusteringConstraints.colocated_groups_with_edges(&sealed_batch(), &sealed_edges());
        assert_eq!(
            group_ids(&groups),
            vec![vec![
                "com.example.models.Circle".to_string(),
                "com.example.models.Shape".to_string(),
                "com.example.models.Square".to_string(),
            ]],
            "D-M23-8 positive lock: sealed trait and its extends-edge sub-types are atomic; \
             the unsealed Base/Lone pair (edge exists, no modifier) stays out"
        );
    }

    #[test]
    fn sealed_pairing_needs_both_modifier_and_edge() {
        // Records-only input (the M22 seam) keeps the gap locked.
        assert!(
            ScalaClusteringConstraints.colocated_groups(&sealed_batch()).is_empty(),
            "colocated_groups still cannot see modifiers+edges jointly"
        );
        // Sealed trait WITHOUT any extends edge groups nothing.
        assert!(
            ScalaClusteringConstraints
                .colocated_groups_with_edges(&sealed_batch(), &[])
                .is_empty(),
            "the modifier alone is not a constraint; sub-typing must be evidenced by an edge"
        );
        assert!(
            ScalaClusteringConstraints
                .frozen_symbols(&sealed_batch())
                .is_empty(),
            "IV3: frozen stays empty - the policy layer owns freezing (plan 3.4)"
        );
    }

    #[test]
    fn companion_pairs_survive_the_with_edges_path() {
        let companion = SymbolRecord {
            name: "Foo".to_string(),
            ..record("com.example.Foo$", SymbolKind::Object)
        };
        let mut batch = sealed_batch();
        batch.push(record("com.example.Foo", SymbolKind::Class));
        batch.push(companion);
        let groups = ScalaClusteringConstraints.colocated_groups_with_edges(&batch, &sealed_edges());
        let mut got = group_ids(&groups);
        for g in got.iter_mut() {
            g.sort();
        }
        got.sort();
        let mut expected: Vec<Vec<String>> = vec![
            vec![
                "com.example.models.Circle",
                "com.example.models.Shape",
                "com.example.models.Square",
            ],
            vec!["com.example.Foo", "com.example.Foo$"],
        ]
        .into_iter()
        .map(|g| g.iter().map(|s| s.to_string()).collect())
        .collect();
        for g in expected.iter_mut() {
            g.sort();
        }
        expected.sort();
        assert_eq!(
            got, expected,
            "D-M23-8: the with_edges path is a superset - companion logic is preserved"
        );
    }
}
