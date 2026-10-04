# M23.5 T3-D Journal - IV-11 Policy Port + ATOMIC DELETE (crate:graph)

**Unit:** M23.5 Phase 3 unit D (plan T3.1 report + policy seam, the D1-B delete).
**Handoff:** `tasks/m235-t3d-handoff.md`. Base HEAD `6c0c2b6` (T3-C tip), branch
`tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`.
**Commits (recorded as they land):** (pending)

## Preflight (step 0.1-0.4)

- `python tools/checkpoint_guard.py status`: HEAD `6c0c2b6`, branch
  `tarjan-s-scc`, DIRTY FILES (numstat) `(none)`, UNTRACKED
  `tasks/m235-t3c-execution-results.md` + `tasks/m235-t3c-verdict.md` (the two
  orchestrator-declared reviewer records; the handoff names no other untracked
  planning artifact). Census shows this worktree `dirty=3` while numstat is
  clean - the known F12 porcelain-only signature plus the two untracked files.
- Step 0.2 fixture: `tests/fixtures/basic/artifacts/includes/report.json` was
  NOT content-dirty at session start (numstat clean), so the Read/Write restore
  was never needed and none was attempted.
- Step 0.3 ABORTING GREP - `detect_cycles|dfs_cycle` over the whole tree
  (Kilo grep tool, 200-match cap re-run scoped per-directory to confirm no
  truncation): production references exist ONLY inside `crates/graph/src/`:
  `graph.rs:234` (`pub fn detect_cycles`), `:246` + `:275` (`Self::dfs_cycle`
  calls), `:260` (`fn dfs_cycle`), `:356` (report call), `:534` (unit-test
  call), `closure_equivalence_tests.rs:515` (cyclicity predicate).
  Zero references under `tests/**`. Other hits are non-callers:
  the frozen test-local oracle `old_detect_cycles`/`old_dfs_cycle`
  (`cycle_groups_equivalence_test.rs:90/:116`, IV-6, must survive), doc prose
  (`cycle_groups_test.rs:5/:101/:142`, `closure_equivalence_tests.rs:67`
  fixture label string, `cycle_groups.rs:64`, `payload_tail_bench.rs:62/:64`),
  `crates/transform` `detect_cycles_in_units` (different function, T3-B
  handoff line 71 warning honoured), and bench label strings in `tools/**`.
  **Gate PASS - the delete is authorised.**
- Step 0.4: every `commit_scoped.py` call carries an explicit `--scope`; the
  fixture path appears in no commit.

## Line-drift observations (FILE wins)

- The six `informational_cycle_count` tests are at `graph.rs:612-680`
  (handoff-corrected range CONFIRMED in-tree; plan's `~:614-687` is stale).
  The four absolute `cycle_count == 1` assertions are at `:617/:631/:640/:647`
  exactly as the T3-C verdict states.
- All other handoff pins (`:234/:246/:260/:275/:356/:369/:534`, adjacency
  `:78-95`, helper `:297-303`) verified byte-true at `6c0c2b6`.

## IV-10 due diligence before touching `:515`

Read `cycle_groups_equivalence_test.rs` (oracle `:90-143` uses only
`graph.file_paths`/`graph.edges` - calls nothing on the live method, so the
delete cannot break it) and `closure_equivalence_tests.rs` in full. Key fact:
`random_graph` registers EVERY file `1..=file_count` (`:420-424`) and every
`link` stays inside the registered set, so the corpus at `:515` lies entirely
inside the registered-endpoint universe where Phase 2's IV-2 equality is proven
(the single named divergence needs an UNregistered cycle endpoint, which this
corpus does not contain). `cyclic = !detect_cycles().is_empty()` and
`cyclic = !detect_cycle_groups().is_empty()` therefore have identical truth
values on every graph in the corpus; the predicate's role is a corpus
regime-sanity check, and the suite's actual proof (closures == BFS oracle) is
untouched by the repoint. No construct-or: same predicate shape, same corpus,
same downstream assertions.

## Checkpoint 1 - RED (genuine, both observed)

Command: `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --lib`
-> `101 passed / 2 failed / 0 ignored / 1 suites`, exit 1,
capture `target/testrun-capture.txt` (140 lines).

Hand-trace of the frozen oracle on `double_reporting_scc()` (edges 1->2, 2->3,
3->1, 3->2; sorted adjacency 1:[2], 2:[3], 3:[1,2]; seeds [1,2,3] from
file_paths, all registered): `dfs_cycle(1)` -> push 1; neighbour 2 unvisited ->
push 2; neighbour 3 unvisited -> push 3. `3`'s neighbours: `1` is on
`rec_stack` at path position 0 -> report `path[0..] = [1,2,3]`; `2` is on
`rec_stack` at position 1 -> report `path[1..] = [2,3]`. Unwind; 2 and 3 now
visited, outer loop skips. Old result `[[1,2,3],[2,3]]` - TWO reported paths
for the ONE SCC `{1,2,3}`; `detect_cycle_groups()` returns ONE group.

- R1 `cycle_count_counts_entangled_groups_not_reported_paths` failed at
  `graph.rs:639` verbatim:
  `assertion \`left == right\` failed: the SCC {1, 2, 3} is ONE entangled group even though the old DFS path enumeration reported two paths ([1, 2, 3] and [2, 3]) for it`
  with `left: 2` / `right: 1` (capture lines 113-114). Exactly the predicted
  semantic reason.
- R2 `double_reporting_scc_with_one_rejecting_member_is_blocking` failed at
  `graph.rs:662` verbatim:
  `assertion \`left == right\` failed: a group containing a rejecting member is blocking even if some old DFS path through it (here [2, 3]) was all-tolerating`
  with `left: 1` / `right: 0` (capture lines 121-122). Under the old per-path
  rule `cycle_is_informational([1,2,3])` is false (FileId(1) rejects) but
  `cycle_is_informational([2,3])` is TRUE, so the old report says 1. Both R2
  assertions fail under the old rule; the observed Red is the first assertion
  (the panic stops the test before the JSON check, which fails by the same
  mechanism since a non-zero count serializes the key - pinned independently
  by the ported test 5 at `:664`).

## Checkpoint 2 - GREEN Half One (seam port), gates verbatim

Port applied: `group_is_informational` (the name plan sec1.4 mandated and T2-A
deviation E deferred to this unit) + report body switched to
`detect_cycle_groups()` (`detected_groups.len()` for `cycle_count`, filter over
`Self::group_is_informational`); `generate_report()` untouched as the
all-`Reject` wrapper; callback signature untouched; `types.rs` untouched, so
`informational_cycle_count` name + skip-when-zero serde shape are byte-intact.
R1/R2 were already written against `generate_report*()`, so no call-site switch
was needed - both were observed Red against the UNPORTED report above. The six
IV-11 tests keep every assertion VALUE (`1/1`, `1/0`, `1/1`, `1/0`, both serde
locks byte-for-byte); only test 1's misleading message ("IV-2: cycle_count
keeps counting all cycles" -> "IV-9: ... every entangled group") changed, plus
a suite header comment naming the group unit and pointing at the two
discriminating tests.

- Gate 3 `-p ariadne-graph`: `118 passed / 0 failed / 0 ignored / 5 suites`
  (expected 118 = lib 101 + 2 new + 9 + 4 + 2; matches, zero arithmetic drift).
- Gate 1 `--test cycle_groups_test`: `9 passed / 0 failed / 0 ignored / 1 suites`.
- Gate 2 `--test cycle_groups_equivalence_test`: `4 passed / 0 failed / 0
  ignored / 1 suites` - the frozen oracle still green with the live report on
  group semantics.
- Gate 4 `cargo build -p ariadne-graph`: `ok (exit 0, 0 warnings emitted)`.
- Gate 5 (IV-9, run AFTER the port - discharges T3-C finding 5):
  `3 passed / 0 failed / 0 ignored / 1 suites`, carrier
  `test_m3_edge_cases_duplicates_system_missing_cycles ... ok`, `0 filtered
  out`. UNEDITEDNESS VERIFIED: `m3_include_graph_test.rs` is in no numstat
  dirty section across all runs (checkpoint_guard above), was never a target of
  any Edit/Write call this session, and is absent from the staged set; the
  reviewer re-proves blob identity. `graph.rs:369` now reads
  `cycle_count: detected_groups.len()`, so this run is the first in the
  milestone where `cycle_count > 0` is witnessed carrying a GROUP count - the
  falsifiable half of IV-9.

Half One commit: `adb53a3` (scoped `crates/graph/src/graph.rs`, +96/-14).
`encoding_guard.py --all` silent before the commit; `--staged` run after each
commit (index empty post-commit; same treatment adjudicated as T3-C Dev-3).

## Checkpoint 3 - HALF TWO (atomic delete + repoints + carried findings)

- Repointed `graph.rs` `test_cycle_detection` to `detect_cycle_groups()`
  (fixture `a.h <-> b.h` = one SCC, one group; assertion truth unchanged, no
  number moved) and `closure_equivalence_tests.rs:515` to
  `!graph.detect_cycle_groups().is_empty()` (one line; IV-10 rationale above).
- Deleted `pub fn detect_cycles` + private `dfs_cycle` (55 lines incl. doc).
- F4: `adjacency()` doc now names all three post-delete production call sites
  (`compute_closures` here, `detect_cycle_groups` in `cycle_groups.rs`,
  `LoadAnalyzer`'s `file_adjacency` memo in `load_caches.rs`), keeps the
  no-duplicate-loop prohibition, and states that the `#[cfg(test)]` callers
  (`compute_closures_bfs_oracle`, `closure_equivalence_tests.rs`) are separate
  deliberate consumers. Authored in pure ASCII (the prior line carried an
  em-dash; the replacement uses "-").
- F5: PREFERRED FIX APPLIED - `explore_from` now matches all three `FrameStep`
  arms (`Descended | EdgeVisited => continue`, `NeighborsExhausted =>
  retire_frame`). Behaviour-identical; `cargo build -p ariadne-graph` reports
  `0 warnings emitted` both before and after, which is the load-bearing
  dead-code evidence either way. No deferral needed.

Post-delete abort-grep re-run (same tool, same pattern, crates + tests):
ZERO live references. Survivors are exactly the allowed classes: frozen oracle
`old_detect_cycles`/`old_dfs_cycle` (calls nothing on the deleted method - it
operates on `graph.file_paths`/`graph.edges` via its own mirrored adjacency),
doc comments (`cycle_groups_test.rs:5/:101/:142`, `cycle_groups.rs:64`,
`closure_equivalence_tests.rs:67` fixture label string), the unrelated
`crates/transform::detect_cycles_in_units`, and stale prose in
`crates/cli/tests/payload_tail_bench.rs:62/:64` (report-only: claims
`detect_cycles` "still exists"; that file is outside this unit's touch-set).

Final-state gates:
- Gate 3: `118 passed / 0 failed / 0 ignored / 5 suites`; capture names
  `test graph::tests::cycle_count_counts_entangled_groups_not_reported_paths
  ... ok`, `... double_reporting_scc_with_one_rejecting_member_is_blocking ...
  ok`, `test_cycle_detection ... ok`, all eight
  `graph::closure_equivalence::*` tests ok (IV-10 witness green with the
  repointed predicate), lib decomposition `103 passed` with `0 filtered out`.
- Gate 1: `9 passed / 0 failed / 0 ignored / 1 suites`.
- Gate 2 (frozen oracle suite, unchanged file): `4 passed / 0 failed /
  0 ignored / 1 suites`.
- Gate 4: `ok (exit 0, 0 warnings emitted)`.
- Gate 6 (delete blast-radius proof): `ok (exit 0, 0 warnings emitted)`.
- Gate 5 (IV-9, final state): `3 passed / 0 failed / 0 ignored / 1 suites`.
- Gate 7 hygiene: zero R1/R2; every printed line an R3 warn in pre-existing
  legacy lines. Zero warns in `graph.rs` and `cycle_groups.rs`; the
  `closure_equivalence_tests.rs` warns are untouched fixture-builder lines,
  not the one line this unit edited.
- Gate 8: `encoding_guard.py --staged` no output; `--all` silent before both
  commits.
- Gate 9 checkpoint: numstat DIRTY `(none)` after the heaviest runs; fixture
  never content-dirty at any point this unit.

Half Two commit: `acea7f2` (scopes: `graph.rs` +9/-61,
`closure_equivalence_tests.rs` +1/-1, `cycle_groups.rs` +3/-2 - all three in
ONE commit, atomic with the delete per reviewer audit 1).

## graph.rs size

704 -> **734** lines. The handoff estimated ~679 (704 - 55 + ~30); the ~30
addition estimate under-counts the work it mandated: the two Red tests plus
their discriminating-fixture doc are ~75 lines, the IV-11 suite header ~9,
the wrapped helper signature +2, the expanded `adjacency()` doc +4, the
report-doc rewrite +2. 734 is meaningfully below the 1000 ceiling (266 lines
of headroom, same posture as `cycle_groups.rs` at 527); recorded as a planner
arithmetic observation, not a defect. All authored functions remain < 100
lines; longest source lines are far under 2000 chars.

## IV-9 uneditedness verification (`m3_include_graph_test`)

The path was never a target of any Edit/Write call this session; it appears in
no numstat DIRTY section at any checkpoint (session start, mid-run, final); it
is in no commit of this unit (scoped commits only, three declared scopes +
tasks/ records). The gate 5 run compiled and executed the committed blob:
capture names `test_m3_edge_cases_duplicates_system_missing_cycles ... ok`
with `0 filtered out`. This is the milestone's first run where `graph.rs`
already computed `cycle_count` as `detected_groups.len()` - the semantic-shift
half of IV-9 is now witnessed, not just the field-name half.

## F5 disposition

FIXED (preferred form, three-arm match). Justification: behaviour-identical,
costs four lines, makes the enum say what it means at its one call site; the
`if let` reading left two constructed variants load-bearing nowhere. No
deferral taken, so the three-element deferral requirement is moot; gate 4's
zero-warning result is recorded either way as the reviewer's load-bearing
evidence.

## IV-7 residual recursion note (handoff sec1, out of scope)

The one recursion left in `crate:graph` after this unit is
`format_explain_node` (`graph.rs`, the explain-tree pretty-printer). It is
out of this milestone's scope per the handoff; NOT touched.

## Report-only observations (not defects of this unit)

- `crates/cli/tests/payload_tail_bench.rs:62/:64` prose is now stale (says
  `detect_cycles` "still exists until" a later unit; it is deleted by
  `acea7f2`). Owner: whoever next touches `crates/cli`.
- `tasks/m235-t2a-handoff.md:122` `[c0, 0]` typo (T2-A finding 7) remains -
  outside this unit's write grant, still open.
- Plan's six-test citation `~:614-687` is stale vs the tree's `:612-680`
  (recorded above; same class as Phase 2's drift findings).
- `tools/bench_compare.py:52` + `bench_baseline.json` still label the metric
  `generate_report+detect_cycles (combined)` - a bench row label, `tools/**`
  is out of this unit's scope; F10 stands.
- Ledger M23.5 row remains 3 units stale (T3-C reviewer already filed the
  correction for the planner).
- The T3-B reviewer's "companion parity test expires at unit E" and the
  `payload_tail_bench.rs:389` row-label rename remain unit-E/closeout items.

## Deviations

- Dev-1: `graph.rs` landed 734 lines vs the handoff's ~679 estimate (cause
  enumerated in the size section above). No ceiling pressure; file, functions,
  and line lengths all pass hygiene.
- Dev-2: `group_is_informational` signature wrapped across lines (the
  one-line spelling with the longer name would exceed the repo's 100-col
  rustfmt width). Body and semantics exactly as handoff sec3.1.
- Dev-3: R2's second assertion (JSON key absence) is not directly OBSERVED
  failing because the first assertion's panic stops the test; the failure of
  both under the old rule is proven by the hand-off's own reasoning + the
  ported test 5's `:664` pin. Disclosed honestly per handoff sec10 audit 2.
- Dev-4: `encoding_guard.py --staged` necessarily runs on an empty index
  (commit_scoped.py stages+commits atomically; raw git staging is denied);
  the substantive pre-commit check was `--all` (silent both times), matching
  the treatment the T3-C verdict accepted as Dev-3.
- Dev-5: gate-count budget - the handoff lists 10 invocations; because the
  unit commits in two halves (per the orchestrator's reviewability
  instruction), gates 1/2/3/4/5 ran once per half (10 test/build invocations
  total incl. re-runs), all verbatim lines reported. Nothing skipped.

## Refactor pass

The port itself is the refactor: no speculative helpers, no extra
configuration, no second feature. `cycle_is_informational` deleted in favour
of `group_is_informational` (its only caller migrated in the same edit);
R1/R2 needed no call-site switch since they were written against the report
surface from the start. Post-Green re-runs of gates 1-5 confirmed no
regression.

## Commits (final record)

- `adb53a3` refactor(graph): count and classify entangled groups in the
  report seam (IV-9, IV-11) - Half One (seam port + Reds + six-test port).
- `acea7f2` refactor(graph)!: delete detect_cycles/dfs_cycle, repoint last
  callers (D1-B, IV-7, IV-10); FrameStep three-arm match (F5) - Half Two.
- (this journal + the two orchestrator-declared T3-C reviewer records land in
  the docs commit that follows; SHA recorded by the reviewer's diff.)
