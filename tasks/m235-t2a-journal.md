# M23.5 T2-A Implementer Journal -- CycleGroup core + iterative graph Tarjan

**Unit:** T2.1 + T2.2 (`tasks/m235-t2a-handoff.md`), crate:graph only.
**Branch/worktree:** `tarjan-s-scc`, `.kilo/worktrees/tarjan-s-scc`, base HEAD `e2a05e2`.
**Implementer session:** 2026-10-04.

## Commits (recorded here per Phase 1 verdict finding 8)

| Commit | Scope | Content |
|---|---|---|
| COMMIT-1-SHA (filled by commit 2) | `crates/graph/src/cycle_groups.rs`, `crates/graph/src/lib.rs`, `crates/graph/tests/cycle_groups_test.rs`, `tasks/m235-t2a-journal.md` | feat(graph): iterative CycleGroup detection with closed-walk witness (gate 7 verbatim) |
| COMMIT-2-SHA (filled by checkpoint guard after commit 2) | `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md`, `tasks/milestone_ledger.md`, `tasks/m235-t2a-handoff.md`, `tasks/m235-t2b-handoff.md`, journal (SHA fill-in) | orchestrator-declared already-approved planning artifacts + journal SHA record |

SHAs are re-read from `python tools/checkpoint_guard.py status`
RECENT COMMITS after each commit; commit 2 fills commit 1's row here
(the file itself cannot carry its own SHA at authoring time).

## Preflight step 0 -- report.json LF restoration (F12)

- Read `tests/fixtures/basic/artifacts/includes/report.json` with the Kilo
  Read tool (36 lines, pure ASCII) and rewrote it byte-safely with the Kilo
  Write tool. No PowerShell writer, no `git checkout`/`git restore` (both
  denied to this role -- step 0.2 obeyed).
- First write showed `1 1` in checkpoint-guard numstat: the committed blob
  carries NO trailing newline, and the Write tool appended one. Second
  Write without the trailing newline returned the path to byte-identical
  with the HEAD blob.
- Gate 6 evidence (run twice around the writes): before, the dirty section
  listed only `tasks/milestone_ledger.md` (F12's CRLF dirt was NOT listed
  by `git diff HEAD --numstat` at session start); after the restoration the
  dirty section still does not list `tests/fixtures/basic/.../report.json`.
- Deviation note D-A: at session start the file was ALREADY not-listed-dirty
  (so it was already LF or autocrlf-normalized); the orchestrator-declared
  step 0 was executed anyway and the file's final state is LF, no trailing
  newline, byte-equal to the committed blob. The path is in NO commit.

## T2.1-R1 (Red) -- IV-7 deep-chain stack overflow, against `detect_cycles()`

- Test `test_deep_include_chain_completes_on_default_stack`: 100 000
  registered files f0..f99999, edges fi -> f(i+1) + f99999 -> f0, run on a
  `std::thread::Builder::new().stack_size(64 * 1024)` thread
  (`SMALL_STACK_BYTES`, matching `crates/cluster/tests/scc_iterative_parity.rs`).
- Gate 1 verbatim (staged file held ONLY this test):
  `### cargo test DRIFT (exit 2)` -- zero result lines survive the abort,
  exactly the predicted drift signature. Capture (transient, overwritten
  by the next run): `target/testrun-capture.txt`, line 7
  `thread '<unknown>' (5896) has overflowed its stack`, line 11
  `(exit code: 0xc00000fd, STATUS_STACK_OVERFLOW)`.
- Deviation note D-B: the handoff predicted `0xc0000409 /
  STATUS_STACK_BUFFER_OVERRUN`; the actual abort code is
  `0xc00000fd / STATUS_STACK_OVERFLOW`. Same overflow class (the IV-7
  predicate: the 64 KiB thread cannot hold 100 000 recursive frames); the
  specific NTSTATUS differs from the Phase 1 record. No behavior change.

## T2.1-R2 (Red) -- IV-3 chord-blind edge, against `detect_cycles()`

- Staged run temporarily dropped the R1 test (its abort kills the whole
  binary and would swallow this assertion -- noted in the staging comment).
- Gate 1 verbatim:
  `0 passed / 1 failed / 0 ignored / 1 suites` with
  `edge FileId(1) -> FileId(3) (a -> c chord shape): both endpoints are
  reported cyclic but this edge is on no reported cycle (IV-3)` --
  the failure names the chord `a -> c`, as the handoff requires.

## T2.1-G (Green) -- the module + the call-site switch (explicit step)

- Created `crates/graph/src/cycle_groups.rs` implementing plan sec1.3
  (frozen struct + signature), D-A1 (union seed universe from
  `compute_closures` semantics, sorted ascending), D-A2 (Kahn on the
  condensation, min-heap keyed on smallest member, sinks first), D-A3
  (members sorted ascending), D-A4 (distinct-pair external degrees), D-A6
  (frame machine mirrors `crates/cluster/src/scc.rs` shape: PUSH in
  `enter_vertex`, ADVANCE in `advance_frame`, POP in `retire_frame` =
  `emit_group_if_root` FIRST then `finish_child`), D-A7 (no cluster
  dependency -- Cargo.toml byte-unchanged), D-A8 (predicate-per-step
  decomposition; names `member_files`, `representative_cycle`,
  `external_out_count`, `on_stack`, `child_completed`).
- Added the two `lib.rs` lines (`pub mod cycle_groups;`,
  `pub use cycle_groups::*;`) mirroring the `graph` wiring.
- CALL-SITE SWITCH: R1 and R2 re-pointed from `detect_cycles()` to
  `detect_cycle_groups()` in the same test functions, assertions NOT
  weakened. R1 passes because the iterative machine is O(1) stack; R2
  passes because IV-3 is structural: every intra-group edge is on a cycle.
  R2's Green assertion form: every edge with both endpoints cyclic has
  both endpoints in one `member_files` (IV-3's iff); the Red form was the
  consecutive-pair membership against old reported paths. The IV-3
  semantic is identical; only the representation differs (D1-B).
- Gate 1 Green verbatim: `9 passed / 0 failed / 0 ignored / 1 suites`
  (capture `target/testrun-capture.txt`; note: the capture file is
  transient and was overwritten by gate 2's run afterwards).

## T2.2 -- the witness

- D-A5 implemented inside T2.1-G: the T2.1 fixture list itself pins
  witnesses (two-cycle `[a,b,a]`, self-loop `[a,a]`), and a stub would
  have fired the Phase 1 verdict-finding-5 dead_code rejection.
- Deviation note D-C (authorized by the handoff's own escape clause): the
  intermediate Red where tests 8/9 see empty witnesses was NOT observable
  -- T2.1-G necessarily fills every witness because its own fixture list
  asserts witness exactness for two-cycle/self-loop. Per the handoff sec 5,
  recorded honestly instead of pretending. The witness verifier
  (`assert_witness_is_edge_backed`) is the load-bearing IV-4 lock: closed
  walk, every consecutive pair re-checked against `test_adjacency`
  (public-edge rebuild of `graph.rs:83-95` semantics, cited in-file), no
  step leaves the group.
- Tests added: `test_seeded_digraphs_witnesses_are_edge_backed_closed_walks`
  (8 locked-seed fixtures in the `closure_equivalence_tests.rs` style:
  PureDag / BigCycle x2 / TwoSccsBridge / SelfLoops x2 / FullBlob x2, with
  unresolved-edge sprinkles) and
  `test_known_two_cycle_survives_in_random_tangle` (known 101<->102 pair
  in a 200-node seeded tangle; existence not exactness, D-A5).

## Refactor

- Renamed the copied `random_graph`/`cyclic_chain` loop locals
  (i/j/k -> source_index/destination_index/chain_position) so both NEW
  files carry ZERO R3 warnings (stronger than the Phase 1
  copied-body precedent requires; the copied LCG constants are untouched,
  so the seeded corpus is unchanged -- re-ran gate 1 after the rename:
  still `9 passed / 0 failed / 0 ignored / 1 suites`).
- `cycle_groups.rs`: 527 lines (< 1000; the ~390 estimate was exceeded by
  doc comments, still far under the ceiling). Test file landed 564 lines
  vs the ~325 sizing estimate; produced implementation+test total is 1093,
  inside the ~1200 sizing-law bound. `graph.rs` UNTOUCHED at 704 lines
  (sec 2.1). Longest functions well under 100 lines.

## Gate ledger (7 invocations, direct form; verbatim results)

| # | Gate | Result |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_test` | R1: `### cargo test DRIFT (exit 2)`; R2: `0 passed / 1 failed / 0 ignored / 1 suites` naming a -> c; Green (twice, pre/post refactor): `9 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph` | `112 passed / 0 failed / 0 ignored / 4 suites` -- matches the pinned line; IV-10 proof: `closure_equivalence` ran inside it, `graph.rs` unedited |
| 3 | `python tools/distill.py -- cargo build -p ariadne-graph` | first two runs FAIL exit 101 (my own typo: a `>` typed as `]` in `group_is_cyclic` + two stray-paren signatures -- fixed, re-run); final `ok (exit 0, 0 warnings emitted)`. A `2>&1 | Select-Object` variant of this command was REFUSED with the catch-all rule `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`; the plain form ran fine, recorded per sec 10 |
| 4 | `python tools/hygiene_check.py` | zero R1/R2 (blocking rules pass); R3 warns are legacy-only EXCEPT my staging file's copied-body i/j/k, which I then renamed to zero. Post-refactor new files: zero R3 |
| 5 | `python tools/encoding_guard.py --staged` | clean (see final section) |
| 6 | `python tools/checkpoint_guard.py status` | dirty section does not list the fixture path after step 0; HEAD progression recorded above |
| 7 | `python tools/commit_scoped.py --scope ... -m ...` | committed, SHA table above; explicit scopes only |

## Locks touched / untouched

- `graph.rs` 704 lines, byte-unchanged (IV-11 unit tests, adjacency()
  single source -- production code calls `self.adjacency()`, the only
  adjacency loop in production; the test rebuild is cited as a mirror).
- `crates/graph/Cargo.toml` byte-unchanged; no cluster<->graph dependency.
- `detect_cycles`/`dfs_cycle` NOT deleted (orchestrator deviation
  declaration 1+2: three of seven call sites are outside crate:graph;
  T2-A strictly additive; D1-B end state lands at T3.1). `graph.rs` stays
  704 lines through this unit; ceiling discharged by the extraction.
- `representative_cycle_for` lives in `cycle_groups.rs`; the handoff's
  `[c0, 0]` comment drift at sec 3 line 124 vs plan's `[c0, c0]` was
  resolved toward the plan (D-A5 and test 4 pin `[c0, c0]`).
- Not added: `group_is_informational` -- policy classification is
  Phase 3 (IV-11, report layer); an unused helper here would fire
  dead_code (Phase 1 verdict finding 5 precedent).
- `detect_cycle_groups` NOT wired into
  `generate_report_with_cycle_policies` (no third representation, no
  premature rewiring).

## Out-of-scope observations (report-only)

- Plan T2.1 refactor note ("graph.rs returns toward <= ~550 lines by
  deleting dfs_cycle/detect_cycles") is unexecutable in a graph-only unit;
  the handoff sec 2.1 already records this -- the plan text should get a
  T3.1 pointer at the next planning touch. Planner artifact, not mine.
- F12 root cause (artifact writer emits CRLF on Windows; `.gitattributes`
  does not cover `tests/fixtures/**/artifacts/**/*.json`) remains open per
  ledger; every commit here carries explicit `--scope`.
