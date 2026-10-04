# M23.5 T3-B Implementer Journal - CLI Consumer Rewiring + Stage-5 Bench Re-Pin (crate:cli)

**Handoff:** `tasks/m235-t3b-handoff.md` (Phase 3 unit B of 5, plan T3.1 label + T3.2 wiring)
**Worktree:** `.kilo/worktrees/tarjan-s-scc`, branch `tarjan-s-scc`, base HEAD `ea82422`
**Authorizations:** this handoff + `tasks/m235-kickoff.md` + M23.5 milestone row
(`tasks/milestone_ledger.md`) + `tasks/m235-t3a-handoff.md` APPROVED (unit A
landed) + orchestrator task message adding the T3-A verdict's deferred
cross-process export-stability discharge (verdict sec 8 / finding 1, owner
unit B) to this unit and declaring the two T3-A reviewer records into this
unit's commit scope.

## Checkpoint 0 - preflight (before any edit)

`python tools/checkpoint_guard.py status` at session start:
- HEAD `ea82422`, branch `tarjan-s-scc`.
- DIRTY (numstat): `(none)`.
- UNTRACKED: `tasks/m235-t3a-execution-results.md`, `tasks/m235-t3a-verdict.md`
  (the T3-A reviewer's records; orchestrator declared them into this unit's
  commit scope; NOT touched by the implementer).
- Worktree census row for this worktree reads `dirty=3` (porcelain):
  2 untracked records + 1 F12 stat-cache entry. See fixture check below.
- Fixture preflight (handoff step 0.2,
  `tests/fixtures/basic/artifacts/includes/report.json`): the numstat dirty
  section does NOT list the path BEFORE any test run -> BEFORE-STATE clean,
  no restoration needed. F12 predicts the stat entry may re-dirty after a
  full run; after-state checks scheduled post-gates (gates 5/6).
- Step 0.3 (unit A landed): `build_include_elements_from_groups` CONFIRMED
  at `crates/ui/src/dto.rs:637` with the 5-arg shape
  `(graph, closures, &[ariadne_graph::CycleGroup], file_languages,
  file_unit_kinds)`. Not dispatched out of order.
- Call-site re-verification against the live tree (handoff warns line refs
  are pinned at `b950963`): B1 `ui.rs:310`, B2 `ui.rs:316`,
  B3 `payload_tail_bench.rs:295`, B4 `:298`, B5 `includes.rs:113`,
  B6 `ui.rs:103`, plus bench docs `:20/:39/:361` ALL match the quoted text
  byte-for-byte. No drift. `detect_cycle_groups` confirmed at
  `crates/graph/src/cycle_groups.rs:39`. Grep over `crates/cli/**` finds
  `detect_cycles`/`build_include_elements` ONLY at the six handoff sites -
  no additional consumers.
- Item-2 feasibility probe (per orchestrator directive): the two-process
  export lock is EXPRESSIBLE inside the touch-set -
  `crates/cli/tests/payload_tail_bench.rs` is an integration-test target of
  `ariadne-cli`, which defines `[[bin]] name = "ariadne"`, so
  `env!("CARGO_BIN_EXE_ariadne")` yields a real binary path for two genuine
  `std::process::Command` invocations. Design facts verified: `ui
  <repo> --export <path>` runs `build_ui_payload` + `export_standalone_html`
  and never binds a port; the export embeds the payload via
  `serde_json::to_string` into a `window.__ARIADNE_DATA__` script
  (`crates/ui/src/export.rs:49-51`); no asset JS contains the token
  `cycle_group_size` (grep over `crates/ui/assets`), so a payload-region
  substring assertion is non-vacuous; all pipeline state (including the TU
  shard cache, `shard_dir = repo/artifacts/shards`) lives under
  `<repo>/artifacts/`, so wiping that directory between the two runs keeps
  both processes symmetrically cold. The same repo directory is used for
  both runs so no absolute-path difference can enter the bytes.

## Deviations inventory (running; final copy in the closing checkpoint)

- D1: handoff sec 6 "exactly one new #[test]" is superseded by the
  orchestrator-assigned Item 2 (T3-A verdict sec 8 + finding 1, assigned to
  unit B AFTER this handoff was authored at `b950963`). The second test -
  the true two-process export byte-stability lock - lands in
  `crates/cli/tests/payload_tail_bench.rs`, the only in-set integration
  location where a real process spawn via `CARGO_BIN_EXE_ariadne` is
  expressible. A lib unit test cannot spawn the CLI binary (no
  `CARGO_BIN_EXE` there) and a same-process double call is explicitly
  forbidden. Consequence: gate 1 measures `88 passed / 0 failed / 1 ignored
  / 3 suites`, +1 vs the sec 8 table; the delta is exactly this test.
  Declared here before the measurement exists.
- D5 (precedent, will recur): `encoding_guard.py --staged` cannot interpose
  between staging and committing because `commit_scoped.py` has no
  stage-only mode (T2-A/T2-B D-E, T3-A D6, both ACCEPTED). The stronger
  `--all` form runs before every commit; the literal `--staged` form runs
  after staging/committing.
- D2 (harness drift, measured): the handoff sec 8 gate-2 line
  `... --test payload_tail_bench -- --ignored --nocapture --test-threads=1`
  CANNOT pass libtest args through `testrun.py` as written:
  `parse_arguments` consumes the FIRST `--` itself (tools/testrun.py:188-190,
  "extend cargo_args with the remainder") and never re-inserts cargo's own
  separator, so cargo receives `--ignored` as a CARGO flag. First attempt
  verbatim per contract -> exit 2 `parse format drift`, capture 8 lines,
  verbatim cargo error: `error: unexpected argument '--ignored' found`
  (tip: `to pass '--ignored' as a value, use '-- --ignored'`).
  Remediation: DOUBLED `--` (`-- -- --ignored ...`) so one separator
  survives to cargo; the cargo command then equals the bench's own run
  instruction (`payload_tail_bench.rs:62-63`) verbatim. This is an
  invocation-shape correction at the entry point, not a substitute gate:
  same cargo argv, same ignored test, same release profile. The sec 8 line
  as written is a handoff-spec wording defect (report-only: fixing
  `tools/testrun.py` or the handoff table is outside this unit).
- D3 (measured): gate 1 suite count is 5, not the sec 8 table's 3. The
  crate defines TWO `[[bin]]` targets (`ariadne`, `tool`); cargo test runs
  a unittests binary for each even at 0 tests, so suites = lib + main.rs +
  tool.rs + payload_tail_bench + doc-tests = 5. Attribution verified from
  the capture: exactly five `Running`/`Doc-tests` headers, five
  `test result:` lines. This is pre-existing tree shape (nothing in this
  unit adds a test binary), so the handoff's "3 suites" arithmetic is a
  planner slip at `b950963` (  the ledger's `-p ariadne-cli` = `86/0/1/3`
  row shares it). passed/failed/ignored match every prediction exactly.

## Checkpoint 1 - Red-Green-Refactor on the label (item 1), plus rewiring

- SEAM EXTRACT (refactor, stayed Green): commit `9d831bf` adds
  `cycle_groups_label(_vocabulary)` returning the CURRENT bytes
  `"  Cycles Detected"` and repoints `includes.rs:113` through it (line
  shape matches its neighbours). Verified via the commit diff: the commit
  carries the OLD string and NO test, so the reviewer's sec 10 audit 1
  (helper-with-current-bytes first) is provable from history. Post-refactor
  gate form run: `86 passed / 0 failed / 1 ignored / 5 suites` (exit 0).
- RED (genuine, compiling, observed): added
  `includes_summary_labels_report_cycle_groups_not_cycle_paths` ->
  `python tools/testrun.py --distill --timeout 1200 -p ariadne-cli`
  -> `86 passed / 1 failed / 0 ignored / 1 suites` (exit 1; distiller
  reports only the failing suite). Verbatim failure:
  `panicked at crates\cli\src\commands\includes.rs:220:9: assertion
  left == right failed / left: "  Cycles Detected"` (right:
  `"  Cycle Groups Detected"`). Exactly the predicted semantic reason -
  an assertion failure naming both strings, not a compile error.
  NOTE: the handoff sec 4 predicted the at-that-point line as
  `86 passed / 0 failed / 1 ignored / 3 suites` which cannot coexist with
  "one failing"; the honest measured Red line is the one above
  (87 tests selected, 1 failed -> non-vacuous).
- GREEN: flipped only the helper's string literal ->
  `87 passed / 0 failed / 1 ignored / 5 suites` (exit 0, capture
  `target/testrun-capture.txt`). +1 passed exactly. Commit `52f8aa2`.
- REWIRE (evidence lock, NO Red claimed): B1 `ui.rs:310` ->
  `let cycle_groups = include_graph.detect_cycle_groups();`;
  B2 -> `build_include_elements_from_groups` with the SAME 5 arguments and
  `node_maps::build` untouched (the pre-existing column-0 indent on the
  `let node_maps` line was NOT touched - not my defect, handoff sec 2);
  B3/B4 bench `timed(|| (graph.generate_report(),
  graph.detect_cycle_groups()))` + from-groups builder; emitted column
  label renamed; `:20` phase note updated. IV-9 check: `ui.rs:378`
  `cycle_count: report.cycle_count` untouched (grep-verified after edit).
- ITEM 2 test added (the assigned two-process lock, see D1):
  `export_bytes_are_identical_across_two_separate_processes` in
  `payload_tail_bench.rs`. Two GENUINE OS processes: two separate
  `std::process::Command` invocations of the built `ariadne` binary
  (`env!("CARGO_BIN_EXE_ariadne")`, debug build for gate 1), each running
  `ui <repo> --export <path>` to completion on a cycle-bearing fixture
  (a.h <-> b.h mutual includes -> one SCC of size 2, one TU,
  compile_commands.json). Between the runs `<repo>/artifacts/` (every
  persisted stage state incl. the TU shard cache) is deleted so the second
  process recomputes symmetrically cold; the repo directory is shared so
  no path can differ between runs. Non-vacuity: asserts the payload region
  (after `window.__ARIADNE_DATA__ = `) contains `cycle_group_size` - a key
  no asset JS contains, so it can only come from grouped nodes; then
  asserts full-file byte equality with the first-diverging-byte reported.
  Gate 1 after rewiring + test: `88 passed / 0 failed / 1 ignored /
  5 suites`; capture line 106: `test
  export_bytes_are_identical_across_two_separate_processes ... ok`, bench
  target `1 passed; 0 failed; 1 ignored; 0 filtered out`. The +1 over the
  87 line is exactly this test (D1 attribution). Commit `a55091c`.
- This payload-behaviour change (intra-group chord now classed `cycle` in
  the export bytes) is unit A's proof (verdict sec 5.1-5.2), NOT a claim
  of this unit; my test locks only byte-STABILITY of whatever the payload
  carries, plus the presence check above.

## Checkpoint 2 - gate 2: release bench re-pin (item 3)

- Gate 2 first attempt EXACTLY as sec 8 wrote it -> testrun exit 2,
  `parse format drift`, 8-line capture, verbatim cargo error:
  `error: unexpected argument '--ignored' found` (see D2; harness eats the
  separator). NOT a gate abandonment - fixed the invocation shape (doubled
  `--`) and re-ran the same cargo argv.
- Gate 2 GREEN: `python tools/testrun.py --distill --timeout 3600
  --release -p ariadne-cli --test payload_tail_bench -- -- --ignored
  --nocapture --test-threads=1` -> `1 passed / 0 failed / 0 ignored /
  1 suites` (exit 0, capture `target/testrun-capture.txt`, 147 lines).
  Not vacuous: the run prints `TAIL_BENCH headers=520 tus=40
  reduced_symbols=2160 file_metrics=560 justification_entries=8960
  plans=520 conservative_rows=520` (full pipeline over the stress fixture).
- MEASURED RELEASE ROW (quoted here so it survives capture overwrite,
  handoff sec 8): `generate_report+detect_cycle_groups (combined) | 5.6`
  ms on 2026-10-04. Same-run neighbours for context: build_include_elements
  5.7, path_to_id+file_collateral 30.3, ensure_load total 2067.3.
- BASELINE COMPARISON: against THIS BENCH's own recorded row `5.8`
  (2026-09-29) -> 5.6, parity within single-run noise, the expected
  linear-SCC outcome. The plan/ledger quote `4.7 ms` (`ui.rs:103`) as "the
  generate_report+detect_cycles baseline"; `4.7` and `5.8` were split in
  the tree BEFORE this unit (same claimed calibration date, two tables,
  never reconciled - other rows differ too: build_include_elements 5.1 vs
  5.4, file_pass 75.4 vs 89.5, save 1417.8 vs 1799.7). I did NOT invent a
  reconciliation: measured 5.6 replaces the figure at BOTH sites per
  handoff sec 3.3, and the pre-existing split is recorded here and in both
  comments.
- UNVERIFIED SENTENCE (handoff sec 5, carried into both comments): `5.6`
  is a SINGLE LOCAL MEASUREMENT on this machine, NOT a bench-compare-
  verified result. `python tools/bench_compare.py` was NOT run (refused per
  ledger M23.5 F10; per handoff sec 5/9 "Do NOT run it / not yours") and
  NOT routed through `distill.py` to evade the denial. No perf-regression
  evidence for `detect_cycle_groups` exists beyond this single measurement.
- Gate 6 (post-gate-2 `checkpoint_guard status`): recorded after this
  commit run; proves the F12 fixture state after the heaviest run.

## Checkpoint 3 - hygiene, records, independent refusal measurement

- Gate 3 `python tools/hygiene_check.py`: ZERO R1/R2 lines workspace-wide;
  every printed line is an R3 warn. Files I authored: `includes.rs` and
  `payload_tail_bench.rs` carry ZERO flags. `ui.rs` R3s (:557, :656, :659,
  :665, :732: single-letter `p`, banned `bar`) are pre-existing lines in
  `swap_into_state`/`make_bar_progress`/`make_boot_progress`/error path,
  shifted +12 from their originals (:545, :644, :647, :653, :720) by my
  comment block only; none sits in an authored region (:103, :107-120,
  :322, :328-334).
- RECORD COMMIT (orchestrator-declared; planners/reviewers cannot commit):
  `tasks/m235-t3a-verdict.md` + `tasks/m235-t3a-execution-results.md`
  landed as `9f30c9a`, zero production content.
- INDEPENDENT ui_smoke measurement (verdict finding 9 escalated its absence
  to material for exactly this unit, and F10 due-date reads "before unit B
  repoints the CLI"): attempted ONCE, directly, NOT routed through
  `distill.py`. REFUSED, verbatim: `The user has specified a rule which
  prevents you from using this specific tool call. Here are some of the
  relevant rules {"permission":"bash","pattern":"*","action":"deny",
  "source":"global"}`. Identical text to T3-A sec 10.2 -> F10 unchanged,
  operator action still owed; there is STILL no runtime UI evidence for the
  new payload bytes (this unit changes what the server serves; a later
  authorized session or the amended allowlist must close F10 before T4.1).
- `python tools/bench_compare.py`: NOT run at all (handoff sec 5: "Do NOT
  run"; ledger F10 refusal already pinned verbatim three times), NOT routed
  through `distill.py`. `python tools/testrun.py --baseline`: not this
  unit's gate (sec 9), not run, no workaround; F9's verbatim usage error
  stands. No gate of MY sec 8 set was dropped: gates 1-7 all ran (gate 2
  after the D2 invocation-shape fix, gate 7 discharged as five scoped
  commits, D6).

## Checkpoint 4 - commits (this unit's SHAs, self-attested)

- `9d831bf` refactor(cli): extract cycle_groups_label seam with the
  current label bytes (includes.rs +9/-1; NO test, OLD string - proves the
  Red ordering for sec 10 audit 1).
- `52f8aa2` feat(cli): label includes summary line as Cycle Groups
  Detected (includes.rs +12/-1: the Red test + the string flip together;
  the Red itself is never committable - "Never commit a Red state" - the
  verbatim failure is Checkpoint 1 above, T3-A precedent).
- `a55091c` feat(cli): wire UI payload and includes summary to cycle
  groups (gate-7 message verbatim; ui.rs +3/-3 = B1+B2 only; bench
  +127/-5 = B3, B4, :20, :361 label, stability test).
- `9194c25` docs(cli): re-pin stage-5 bench row and calibration to
  detect_cycle_groups 5.6ms (ui.rs +13/-1: B6 figure + ASCII provenance
  note; bench +29/-2: :39 row, table header date note, re-pin + UNVERIFIED
  notes).
- `9f30c9a` docs(tasks): land M23.5 T3-A reviewer records (216 + 952
  lines, records only).
- Journal commit follows this file's own save (its SHA is by definition not
  self-listable here; read RECENT COMMITS in checkpoint_guard).
- `tests/fixtures/basic/artifacts/includes/report.json` appears in NONE of
  the commits above (per-commit numstat printed by every
  `commit_scoped.py` run; only in-set paths staged, never `git add .`).

## Checkpoint 5 - after-state, fixture, item dispositions

- Fixture AFTER the heaviest runs (gate 2 full 520-header pipeline + two
  repeat gate-1 runs): `checkpoint_guard status` numstat DIRTY section
  `(none)` -> the forbidden path never went content-dirty, so the Read/
  Write restore was never needed at any point. BEFORE-STATE: clean;
  AFTER-STATE: clean (numstat). Census porcelain `dirty` counts only
  untracked files + the known F12 stat entry (verdict sec 7 mechanism,
  third-plus reproduction).
- Item 1 (label): DONE with a genuine Red/Green (Checkpoint 1).
- Item 2 (two-process export byte-stability): LOCKED as a real committed
  test, and it PASSED - export bytes from two separate `ariadne` process
  invocations over the cyclic fixture are byte-identical, with the payload
  carrying cycle-group data (non-vacuity assert green too). Discharge
  evidence for the verdict-sec-8 deferred row (owner unit B, due before
  T4.1). The ledger row itself is outside this unit's touch-set - the
  milestone-lifecycle step should mark it discharged citing this journal
  and commit `a55091c`. Residual honesty: (a) two samples only - a
  pathological hash-seed collision between the two processes could in
  theory mask an order bug, the classic limitation of a two-run lock (the
  T3-A structural argument in verdict sec 8d remains the backstop); (b) the
  verdict's recommended COMPANION parity test (`from_groups` vs legacy
  builder byte-equality on an acyclic graph) is NOT added - it needs no
  spawn but needs a second `#[test]`, every extra test deepens D1, and the
  orchestrator assigned only the two-process item to this unit. It stays
  open for unit E or T4.1.
- Item 3 (bench re-pin): DONE; measured `5.6` ms release; compared against
  this bench's own `5.8` (parity within single-run noise); the split
  `4.7`-vs-`5.8` pre-existing baseline is recorded in three places (both
  comments + this journal), NOT silently reconciled; the number is
  UNVERIFIED against `bench_compare.py` in the journal and in both file
  comments, verbatim sentence present.
- WHAT IS NOT CLAIMED: no Red exists for the rewiring or the stability test
  (evidence locks on new wiring, per the orchestrator's honesty clause);
  the intra-group chord classing is unit A's proof, not this unit's;
  `detect_cycles` and `dfs_cycle` are INTACT (unit D owns the delete -
  grep confirms crates/cli no longer calls `detect_cycles()`, but
  `crates/graph/src/graph.rs` untouched and its internal callers and the
  `tests/` m20/m22 consumers all still compile - gate 1 proves the crate
  builds and the full-workspace build is unaffected since no signature was
  removed). IV-9: `cycle_count: report.cycle_count` byte-unchanged (now at
  `ui.rs:390`, shifted only by comment lines above it).
- Out-of-scope observations (report-only): the emitted bench row still
  names the DTO row "build_include_elements" (`:362/:40`) while B4 calls
  the from-groups builder - sec 3.4's list is closed and did not name it,
  so it stays for unit E/closeout to rename; the pre-2026-09-29 calibration
  disagreement affects several table rows (file_pass, save tail,
  justification), all left as history per the A6 precedent; F12 `.gitattributes`
  pin and F9 `--baseline` fix remain closeout items, untouched here.

## Deviations - final (consolidated; the running list above is the record)

| # | Deviation | Reason / status |
|---|---|---|
| D1 | Second new `#[test]` beyond sec 6's "exactly one"; gate 1 line `88 passed / ... / 5 suites` vs table's `87 ... / 3` | Item 2 assigned to unit B by the T3-A verdict AFTER this handoff was authored; only in-set integration file; delta fully attributed to that one test |
| D2 | Gate 2 run with DOUBLED `--` | `testrun.py:188-190` eats the single separator (verbatim first-attempt failure recorded); cargo argv executed equals the bench's own run instruction |
| D3 | Gate 1 suite count 5, not 3 | Two `[[bin]]` targets each add a 0-test unittests suite; pre-existing tree shape; planner arithmetic slip |
| D4 | Red digest line differs from sec 4's predicted line | sec 4's "86 passed / 0 failed ... one failing" is self-contradictory; measured `86 passed / 1 failed / 0 ignored / 1 suites` reported honestly |
| D5 | `encoding_guard.py --staged` runs post-commit | no stage-only mode in `commit_scoped.py`; `--all` (stronger) ran before EVERY commit; T2-A/T2-B D-E + T3-A D5 accepted precedent |
| D6 | sec 8 gate 7's single commit split into five scoped commits | one scoped commit per logical step; REQUIRED for audit 1 (seam commit must show old bytes without the test); gate-7 message preserved verbatim on `a55091c`; every `--scope` explicit; fixture path in no commit |
