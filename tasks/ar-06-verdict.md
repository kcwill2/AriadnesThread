# AR-06 Verdict - O(1) edge dedup in `add_edge` (P2)

Date: 2026-10-10. Reviewer: Kilo (review-gate). Implementation commit: `70e1699`
(`perf(graph): O(1) edge dedup in add_edge (P2)`), branch `shining-enthusiasm`.
Reviewed jointly with AR-07 (`8ea8d0d`, `602c176`) - shared gates, see
`tasks/ar-07-verdict.md`; shared gate evidence cross-referenced below.

## VERDICT: APPROVED

## Ground truth

- `checkpoint_guard status`: HEAD `70e1699`, branch `shining-enthusiasm`.
  Dirty: `crates/scheduler/src/worker.rs` (+547, foreign AR-16 WIP).
- Commit range for this handoff: `70e1699` alone.

## Commit-vs-touch-set (step 4)

`git show --stat 70e1699` touches exactly
`crates/graph/src/graph.rs` (+29/-4) and `tasks/ar-06-handoff-journal.md`.
Both inside the declared touch-set. EXACT.

Attribution note adjudicated: the 7-test parity suite
`crates/graph/tests/ar06_edge_dedup_test.rs` (184 lines) lands in the repo via
AR-07's `8ea8d0d` (shared-worktree glob sweep, disclosed on both sides).
`git diff 8ea8d0d..70e1699 -- crates/graph/tests/ar06_edge_dedup_test.rs`
returns EMPTY - the file the AR-06 implementer tested is byte-identical to the
committed bytes; no silent evolution. Working tree carries no dirty copy.

## Key-choice adjudication (plan's open question) - ACCEPTED, verified independently

- Plan anchor `.kilo/plans/1791568620000-audit-remediation.md:121` writes
  `HashSet<(FileId, Option<FileId>... key type per Edge)>` - the ellipsis
  defers the key type to Edge's own equality semantics; the full-edge key is
  plan-consistent, not a literal break.
- `crates/graph/src/builder.rs:298` (inside `include_edge`, lines 286-304)
  stamps `source_location: SourceLocation::new(raw.line, raw.column)` per
  directive - same-endpoint/different-line edges legitimately coexist, and a
  pair key would silently drop the second (changing `total_edges`,
  `direct_includes` multiplicities). Journal claim CONFIRMED at source.
- `IncludeEdge` derives `Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash`
  (`crates/lang-api/src/types.rs:293`) - Hash/Eq are derived over exactly the
  same fields as the retired `Vec::contains` PartialEq, so set-membership and
  struct-equality cannot drift. No DTO change was needed.
- Parity locks are genuine:
  `same_endpoints_different_metadata_edges_are_both_kept` (lines 56-79: lines
  3 vs 9 both kept, plus a `kind` variant) FAILS under the pair key;
  `edges_vec_is_byte_identical_to_reference_interleave_fixture` locks Vec and
  serialized-bytes order; `artifact_rebuild_loop_dedups_and_keeps_first_occurrence_order`
  locks the graph_io replay; `serialized_graph_shape_never_carries_the_dedup_set`
  locks the `{edges, file_paths}` JSON shape.

## COW deviation adjudication - ACCEPTED on merits

Plain `HashSet` field tripped the mandated 1.25x bench ratio (journal truth
rows: clone cold 2.1 vs 1.3, warm 1.6 vs 1.0, exit 1). Fix verified in
`graph.rs`: `seen_edges: Arc<HashSet<IncludeEdge>>` +
`#[serde(skip)]`, `add_edge` via `Arc::make_mut(...).insert(...)`;
`Clone` derive shares the Arc (refcount bump), `Default` derive yields empty.
Every load path rebuilds through `add_edge`
(`crates/report/src/manager/graph_io.rs:104-106` re-runs the persisted
`Vec<IncludeEdge>`), so the serde-skipped set is correctly empty after
`Deserialize`; my grep finds NO whole-graph `Deserialize` site
(`from_str/from_slice/from_reader/from_value` with `IncludeGraph`: zero hits),
confirming the journal claim. Desync audit: all direct `graph.edges.push`
sites are UI test fixtures (`crates/ui/tests/ui_test/*`), and
`crates/ui/tests` contains ZERO `add_edge` calls - the two styles never meet,
verified by grep, exactly as the journal claims.

## Gates (fresh, reviewer stdout) - see `tasks/ar-06-execution-results.md`

- graph scoped: `116 passed / 0 failed / 0 ignored / 5 suites` (matches
  implementer's GREEN + post-refactor line)
- bench: `bench compare: 3 run(s), no regressions past 1.25x baseline`
- m00 byte-identity consumers green UNEDITED (last edit `f9064f4`, pre-wave)
- workspace census: `1326 passed / 0 failed / 13 ignored / 94 suites`
- encoding `--all` silent exit 0; hygiene = exactly the 2 pre-existing
  `server.rs` R1/R2 rows, zero rows on `graph.rs` or the test file.

## Findings

1. (minor, record) Parity suite enters history via `8ea8d0d` (AR-07's commit),
   not this handoff's commit - shared-worktree glob sweep. Disclosure complete
   (both journals + board INFO/RESULT), file verified byte-identical
   (empty diff) and green at both commits; AR-07's verdict records the same
   event as its own finding. Use `8ea8d0d` as the introduction commit in wave
   reconciliation. No rework.
2. (minor, deferred candidate -> `tasks/milestone_ledger.md`) Residual desync
   surface: `edges` remains a public field, so future code that pushes
   directly AND calls `add_edge` on the same graph would bypass `seen_edges`
   (legacy `Vec::contains` covered both). Current state safe - verified zero
   mixing today. Ledger candidate if the field ever gains mixed production
   use; type-level enforcement is out of AR-06's touch-set.
3. (watch, not blocking) Fresh bench clone rows 1.6/1.2 ms vs 1.3/1.0 ms
   baseline = 1.23x/1.20x - PASSES but the cold row sits close to the 1.25x
   ratio; relevant if AR-09 (same-file chain) further touches this file.

## Deviation dispositions

- Full-edge key vs plan's pair suggestion: ACCEPTED (plan defers key type;
  semantic evidence verified at builder.rs:298; locked by test).
- `Arc` COW clone shape (not in handoff text): ACCEPTED (mandated bench gate
  forced it; shape verified; clone rows pass).
- RED phase green pre-change (characterization suite): ACCEPTED - correct
  shape for a behavior-preserving perf refactor; O(1) property gated by the
  mandated `bench_compare`, not unit timing.
