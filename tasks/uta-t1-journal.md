# UTA-T1 Journal - graph: traversal surface, deterministic lookup, iterative cycles

Handoff: tasks/uta-t1-handoff.md. Baseline: main @ 43e6ebe, clean tree.
Change tag: uta (docs/specs/unit-test-audit.md), hand H1.

## Plan (per handoff item order)

- Item 1 (UTA-B4 fix): red pin in graph.rs tests -> two-pass deterministic
  `find_file_id` (full-path pass, then smallest-FileId filename pass).
- Item 2 (UTA-B5 fix): red pin in callgraph.rs tests (two-SCC fixture +
  100k fused chain on 64 KiB thread) -> iterative frame-machine rewrite of
  `detect_recursion_cycles` (faithful translation of the old enumeration
  semantics; see journal note on undercount observation).
- Items 3-15: required behavior pins (expected green). Placement plan:
  3/4/10 inline traversal.rs; 5 inline callgraph.rs; 6+12 sibling
  load_guard_tests.rs wired from load.rs; 7 tests/report_top_headers_test.rs;
  8 tests/format_explain_snapshot_test.rs; 9 inline load_explain.rs;
  11 inline reduced.rs; 13 sibling closures_tests.rs wired from closures.rs;
  14 report-only; 15 reduced.rs + cycle_groups tests file + types.rs.

## Deviations / observations log

- Item 6: NOT newly authored - the guard behavior is already pinned verbatim
  by `test_impact_guards_return_none` in load_impact_tests.rs (verified
  present and green; handoff's "verify + note" path taken).
- Item 12: shipped as sibling `load_degeneracy_tests.rs` wired from load.rs
  via `#[path]` (plan line said "load_guard_tests.rs"; the degeneracy pin is
  the whole payload, so the file is named after it). Pin content per plan:
  `include_load == total_load` degeneracy at load.rs:548-550.
- Item 13: DEVIATION with reason - the handoff words "100k-node cycle
  compute_closures on a 64 KiB stack", but `compute_closures` materializes a
  full `transitive_includes` Vec per member, so a fused 100k cycle is
  O(V^2) element copies (~10^11 ops at V=100k): it cannot finish inside the
  600 s gate on ANY stack. The stress is split - the recursive-overflow risk
  surface (the Tarjan frame machine, the module's deep-DFS analogue) runs the
  FULL 100k fused cycle on the 64 KiB thread through the crate-internal
  `tarjan_components` seam (linear work), and the PUBLIC `compute_closures`
  pipeline (condensation DP + assembly) runs end-to-end on a 3k fused cycle
  on the same thread (a recursive walk dies within a few hundred frames of
  64 KiB, so depth coverage holds). Rationale also in the module header.
- Item 14: report-only per handoff - no test, no code change. Observed:
  builder.rs:89-92 swallows per-file index failures via `Err(_) => continue`
  with no counter and no log; a repo whose files all fail to parse yields an
  empty manifest that downstream sees as a clean empty repository.
  Report-only observation for the owner.
- Gate 3 sequencing note: `commit_scoped.py` stages+commits atomically with
  no hook point, so `encoding_guard.py --staged` ran against the index
  pre-commit (exit 0) AND `encoding_guard.py --all` ran over all tracked
  files for non-vacuous coverage of the new content (exit 0).

## Checkpoints

- Item 1 (UTA-B4) GREEN. Red digest: `108 passed / 1 failed / 0 ignored / 1
  suites` - the new pin failed with `left: Some(FileId(1)) right:
  Some(FileId(2))` (full path `b/util.h` lost to the basename match on
  `a/util.h`), capture target/testrun-capture.txt. Green digest:
  `145 passed / 0 failed / 0 ignored / 8 suites`. `find_file_id` rewritten as
  two deterministic passes (exact full-path smallest-id pass, then basename
  smallest-id pass); the test-only scan counter stays once per call so the
  LoadPerf A2 counter locks hold.
- Item 2 (UTA-B5) GREEN. Red evidence: the new 100k fused-chain pin aborted
  the lib test binary - `thread '<unknown>' (7984) has overflowed its stack`
  in the capture; testrun reported `parse format drift: no test result line
  matched the pinned parse spec` (exit 2) because the binary died before
  printing results (same "aborted binary" red class as M23.5 T2.1-R1).
  The two-SCC fixture passes against BOTH implementations (old DFS happens to
  enumerate that shape correctly) - it is the enumeration-equivalence guard
  for the rewrite, not the discriminator. Green digest:
  `147 passed / 0 failed / 0 ignored / 8 suites` (old `test_recursion_cycles`
  contract of 2 cycles intact). `detect_recursion_cycles` now walks on an
  explicit `(node, cursor)` frame stack (`explore_call_frames`, mirroring
  closures.rs `explore_frames`); traversal semantics, normalization, root and
  neighbor orders are a verbatim translation; the O(depth) on-path scan
  became an O(1) position map (required to keep the 100k fixture linear).
- Observation for reviewer: the global-visited skip is preserved verbatim, so
  the enumerator still misses cycles whose members were finished by an
  earlier DFS tree (diamond feed: edges 1->2, 1->3, 2->4, 3->4, 4->1 report
  [1,2,4] but not [1,3,4]). The audit bug-ledger words UTA-B5 as "stack risk
  + SCC undercount", but this handoff's item-2 build spec is the
  no-recursion replacement + its two fixtures, which the translation
  satisfies; a full elementary-cycle enumeration would change
  `CallGraphManifest.recursion_cycles` for real repos, an artifact-visible
   decision left to the owner (report-only).
- Items 3-15 GREEN (single gate). Digest:
  `170 passed / 0 failed / 0 ignored / 11 suites` (+23 tests, +3 test
  binaries vs the item-2 gate). No required-test red: no additional
  bug-ledger item was exposed by the pins, so no extra fix beyond items
  1-2. Placement: 3/4/10 inline traversal.rs; 5 inline callgraph.rs;
  7 tests/report_top_headers_test.rs; 8 tests/format_explain_snapshot_test.rs
  (box-drawing asserted via unicode escapes, source stays ASCII);
  9 inline load_explain.rs (`mod source_filter_pins`, 5 tests);
  11 + 15a/15b inline reduced.rs; 12 load_degeneracy_tests.rs;
  13 closures_tests.rs; 15c tests/cycle_groups_selfloop_degrees_test.rs;
  15d inline types.rs (`include_closure_empty_and_direct_only_shapes`).
  All 23 new test names verified present-and-ok in the capture file.
- Final gates: hygiene_check.py exit 0 (R1/R2 clean; listed R3 warns are all
  pre-existing lines, spot-verified e.g. reduced.rs:707-708 predates this
  hand); encoding_guard --staged exit 0; encoding_guard --all exit 0.


