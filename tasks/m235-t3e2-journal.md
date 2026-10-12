# M23.5 T3-E2 Journal - Rename the surviving include-element builder to the canonical name

Contract: `tasks/m235-t3e2-handoff.md` (the REWRITTEN version, uncommitted at
dispatch time -- see Commits). Dispatch base: HEAD `5addad7`, branch
`tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`, confirmed by
`checkpoint_guard.py status` before any edit.

Outcome: COMPLETE. Three scoped commits -- commit 1 `378b240` = the planner
records the dispatch waived me to land; commit 2 `4bc3b88` = the eight code
paths plus this journal; commit 3 = this journal alone, carrying `4bc3b88`
(sec9 asks for your own SHA in the journal, which a commit body cannot do for
itself; precedent is T3-E1 verdict "the journal's own SHA correctly absent
from its own commit table").

Per sec4, stated plainly: **there is NO production-behaviour Red for a
rename.** This unit changes no behaviour, adds no test, removes no test. The
executed Red is R1 below: a mechanical compile failure.

## Checkpoint 0 - preflight (sec0.4 / sec0.5), run before any edit

- `checkpoint_guard.py status`: HEAD `5addad7`, branch `tarjan-s-scc`.
  DIRTY: `tasks/m235-t3e2-handoff.md` 283/405 (the planner's rewrite).
  UNTRACKED: `tasks/m235-t3e1-verdict.md`,
  `tasks/m235-t3e1-execution-results.md`. No `crates/**` dirt.
- **0.4 CENSUS -- run and closed by me, no STOP needed.** Grep
  `build_include_elements` (the `grep` tool; raw bash grep is off the
  allowlist):
  - total **33** = **30** under `crates/` + **3** under `tools/`
    (`bench_compare.py:53`, `bench_baseline.json:6`,
    `bench_compare_tests.py:37`) + **0** under `tests/`.
  - the 30 close as **1 + 19 + 5 + 2 + 3**:
    - **1** definition: `dto.rs:618` `pub fn build_include_elements_from_groups`.
    - **19** call sites: `common.rs:158`;
      `d27_node_provenance.rs:141/:152/:174`;
      `dto_builders.rs:15/:470/:501/:588`;
      `export_order.rs:91/:108/:167/:310/:317/:422`;
      `graph_api.rs:136/:752/:759`; `cli/src/commands/ui.rs:328`;
      `cli/tests/payload_tail_bench.rs:325` (17 in `crates/ui`, 2 in `crates/cli`).
    - **5** import lists: `common.rs:3`, `d27_node_provenance.rs:9`,
      `dto_builders.rs:4`, `export_order.rs:15`, `graph_api.rs:4`.
    - **2** doc name-refs: `dto.rs:601` (plain spelling, the D1 sentence) and
      `export_order.rs:368` (grouped spelling).
    - **3** prose/label: `payload_tail_bench.rs:41` (header table row),
      `payload_tail_bench.rs:389` (emitted bench LABEL),
      `cli/src/commands/ui.rs:103` (stage-5 calibration row).
  - Wording imprecision in the sec0.4 table (documentation defect, planner,
    no STOP): the row "`build_include_elements_from_groups` NOT followed by
    `(`" returns **6** grouped-name lines (5 imports + `export_order.rs:368`),
    not 7, because the second doc name-ref (`dto.rs:601`) spells the name
    WITHOUT the `_from_groups` suffix. The closed decomposition of the 30 --
    which sec0.4 names as the real exhaustiveness proof -- matches exactly:
    10 non-paren lines = 5 import + 2 doc + 3 prose.
  - `ariadne-ui` in any `Cargo.toml`: **3** hits -- package decl
    `crates/ui/Cargo.toml:2`, `crates/cli/Cargo.toml:34`,
    `tests/integration/Cargo.toml:25` => exactly two reverse dependencies.
  - `tests/`: **zero** hits for `ariadne_ui` AND for `build_include_elements`
    -- the integration-tests crate is a declared path dependency that
    textually never mentions the API, which is why it still gets gated (F14/2).
  - `crates/cli`: the token `ariadne_ui` occurs 17 times spread over exactly
    **2 files** (`src/commands/ui.rs`, `tests/payload_tail_bench.rs`), and
    references to the renamed symbol are exactly **2** (`:328`, `:325`). The
    sec0.4 row says "2 under `crates/cli`" naming those two files: true for
    files and for builder references, false only if read as raw token hits.
- **0.5 F12 fixture** `tests/fixtures/basic/artifacts/includes/report.json`:
  `git diff --numstat HEAD -- <path>` (through `distill.py`) printed **zero
  numstat rows**, only the autocrlf stat warning. Content byte-equal, so per
  "if and only if it prints rows" NOTHING was restored and nothing written.
  Re-checked after both full test runs: `checkpoint_guard.py status` still
  does not list it as dirty. It appears in **no** commit of this unit.
- Entry line counts (before): `dto.rs` 882, `export_order.rs` 476,
  `dto_builders.rs` 660, `graph_api.rs` 790, `d27_node_provenance.rs` 293,
  `common.rs` 227, `cli/src/commands/ui.rs` 766,
  `cli/tests/payload_tail_bench.rs` 621 -- all as sec0 measured, all far
  under the 1000-line ceiling.

## Checkpoint 1 - base state of the gates (sec7 "base runs")

| Gate | Command | Verbatim |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test` | `186 passed / 0 failed / 0 ignored / 1 suites` (capture `target/testrun-capture.txt`, 193 lines) |
| 3 | `python tools/distill.py -- cargo check -p ariadne-cli --all-targets` | `ok (exit 0, 0 warnings emitted)` (capture `target/distill-capture.txt`) |
| 4 | `python tools/distill.py -- cargo check -p ariadne-integration-tests --all-targets` | `ok (exit 0, 21 warnings emitted)` |
| 5 | `python tools/distill.py -- cargo build -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` |

## R1 - the executed Red (definition only, nothing repointed) then revert

- Pre-edit content hash `certutil -hashfile crates\ui\src\dto.rs SHA256`
  (through `distill.py`):
  `804680f3cf2e728d456b9730691cc4b61929489e58afd0a9efc4547092b15d94`
- Red edit: one identifier at `dto.rs:618`, `build_include_elements_from_groups`
  -> `build_include_elements`. No call site, no import, no doc touched.
- **Gate 3 FAILED as contracted** -- `python tools/distill.py -- cargo check -p
  ariadne-cli --all-targets` -> `### cargo build FAIL (exit 101)`, verbatim
  from the 12-line capture `target/distill-capture.txt`:

  ```
  error[E0425]: cannot find function `build_include_elements_from_groups` in crate `ariadne_ui`
     --> crates\cli\src\commands\ui.rs:328:40
  error: could not compile `ariadne-cli` (lib) due to 1 previous error
  error: could not compile `ariadne-cli` (lib test) due to 1 previous error
  ```

  The named caller is OUTSIDE `crates/ui` -- the production call in
  `build_ui_payload` -- which is the whole point of this Red (the E blocker's
  gate set never built this crate). The bench target
  (`payload_tail_bench.rs:325`, identical textual form) was not reached
  because the lib failed first, exactly as the held E unit's probe observed.
- **Gate 1 FAILED too, but its error count is NOT the 17 the contract
  predicted** -- `### cargo test DRIFT (exit 2) [capture:
  target/testrun-capture.txt, 33 lines]` / "parse format drift: no test result
  line matched the pinned parse spec" (a compile failure emits no test result
  line; precedent T3-E1 Dev-5). The capture holds **5** errors, code
  **E0432** (unresolved import), not 17 of E0425 class, closing with
  `error: could not compile `ariadne-ui` (test "ui_test") due to 5 previous
  errors`:

  ```
  error[E0432]: unresolved import `ariadne_ui::build_include_elements_from_groups`
    --> crates\ui\tests\ui_test\common.rs:3:26
  error[E0432]: unresolved import `ariadne_ui::build_include_elements_from_groups`
    --> crates\ui\tests\ui_test\d27_node_provenance.rs:9:18
  error[E0432]: unresolved import `ariadne_ui::build_include_elements_from_groups`
    --> crates\ui\tests\ui_test\dto_builders.rs:4:26
  error[E0432]: unresolved import `ariadne_ui::build_include_elements_from_groups`
    --> crates\ui\tests\ui_test\export_order.rs:15:5
  error[E0432]: unresolved import `ariadne_ui::build_include_elements_from_groups`
    --> crates\ui\tests\ui_test\graph_api.rs:4:26
  ```

  Cause: every in-crate caller imports the name, and rustc emits the import
  errors and aborts name resolution before it enumerates the 17 call sites.
  So the predicted identity "17 in-crate + 2 cli = 19" cannot be observed this
  way. The census is nevertheless still closed and the compiler corroborates
  it at file granularity: the 5 E0432 files are EXACTLY the census's 5
  import-list files. See Dev-1 (documentation defect in the contract,
  planner-side).
- **Revert, byte-for-byte.** The exact inverse one-identifier Edit. Post-revert
  `certutil` SHA256 of `dto.rs`:
  `804680f3cf2e728d456b9730691cc4b61929489e58afd0a9efc4547092b15d94` --
  identical to the pre-Red hash. A partial revert would have failed the final
  Green (gate 1 `186/0/0/1` against the frozen byte constant).
- R1 was never committed.

## Green - as landed (`git diff --numstat HEAD` at commit time)

| Path | +/- | What changed |
|---|---|---|
| `crates/ui/src/dto.rs` | 10/7 | definition renamed (identifier only; body, the `emit_include_elements` call and the `from_groups` view byte-unchanged); D1, D2, D3 fixed; D4 added |
| `crates/ui/tests/ui_test/export_order.rs` | 15/14 | import `:15` + 6 calls repointed by identifier; D5 (`:368` name); D6 (P2 doc: former P1 name -> current P1 name, "parity lock" -> frozen-expectation wording) |
| `crates/ui/tests/ui_test/dto_builders.rs` | 5/5 | import `:4` + 4 calls (`:15/:470/:501/:588`) |
| `crates/ui/tests/ui_test/graph_api.rs` | 4/4 | import `:4` + 3 calls (`:136/:752/:759`) |
| `crates/ui/tests/ui_test/d27_node_provenance.rs` | 4/4 | import `:9` + 3 calls (`:141/:152/:174`) |
| `crates/ui/tests/ui_test/common.rs` | 2/2 | import `:3` + 1 call (`:158`) -- in-set per sec1 item 6, NOT frozen |
| `crates/cli/src/commands/ui.rs` | 1/1 | the one production call `:328`, callee token only |
| `crates/cli/tests/payload_tail_bench.rs` | 4/3 | the one bench call `:325`, callee token only; D7 clause |

No argument, no line break, no reformat, no type annotation moved. Zero new
hex literals (no `crates/ui/assets/**` path in the touch-set, so it is
structural). Line counts after: `dto.rs` 885 (+3, D4 line),
`export_order.rs` 477 (+1, D6 rewrap), `payload_tail_bench.rs` 622 (+1, D7
rewrap), the other five unchanged (each replacement is one line by one line).
Largest file 885, ceiling 1000; no module extracted; no function grew (the
renamed function is 15 lines).

## Doc obligations -- done and deliberately NOT done

DONE: **D1** the "group-based sibling of `build_include_elements`" sentence is
now "the ONE include-element entry point in `crates/ui` (the path-based one was
retired in T3-E by E1)", 5-argument D27 shape claim kept. **D2** "both
include-element entry points" -> "the one include-element entry point". **D3**
"the only input differing between the entry points" -> "The
cycle-classification view is this layer's only cycle-classification input",
no-duplicated-loop rationale kept. **D4** added: the third parameter was
`&[Vec<FileId>]` (reported DFS path enumeration) before M23.5 and is now
`&[CycleGroup]` (the SCC partition), so the earlier type must not be restored.
**D5** `export_order.rs` frozen-lock header names the builder by its current
name. **D6** P2's doc comment names the CURRENT P1
`acyclic_include_elements_match_the_frozen_acyclic_payload` instead of the
former `acyclic_..._under_both_cycle_currencies` and no longer calls it a
parity lock ("frozen-expectation lock" / "the frozen expectation honest"); the
historical R2 narrative is byte-unchanged. **D7** the false clause
"`detect_cycles` still exists until the crates/graph unit D deletes it" now
reads "crates/graph unit D deleted the public `detect_cycles()` in `acea7f2`,
approved range `adb53a3..aebfa9b`"; no timing number and no other clause
touched.

NOT TOUCHED (NO-EDIT list honoured): **D8** the emitted bench row label
`("build_include_elements", include_elements_ms)` at
`payload_tail_bench.rs:389` -- it names the timed call, becomes correct by the
rename, and is the key `tools/bench_baseline.json` and `bench_compare.py`
match on. **D9** the header baseline table row
`build_include_elements | 5.4` at `:41`. **D10** the
`generate_report+detect_cycles` -> `generate_report+detect_cycle_groups`
sentence (both names correct there as history). **D11** the `ui.rs:103`
stage-5 calibration row naming `build_include_elements` (see observation O-1).
**D12** the `UNVERIFIED against bench_compare.py` paragraph at
`payload_tail_bench.rs:79-86` -- byte-unchanged, and the `ui.rs:115-119`
counterpart too. **D13** `cycle_groups_view.rs` module doc and the generic
"include-element builder" prose in `export_order.rs` / `graph_api.rs`.
`crates/ui/src/lib.rs` (bare `pub use dto::*;` re-export) untouched.

Post-state locks: `build_include_elements_from_groups` -> **zero** hits under
`crates/` and `tests/`; `pub fn build_include_elements` -> exactly **1**
(`dto.rs:621`) with third parameter `cycle_groups: &[ariadne_graph::CycleGroup]`
at `:624`; the 3 `tools/` bench keys unchanged; `crates/graph/**`,
`crates/report/**`, `crates/pipeline/**`, `tests/**`, `tools/**`,
`Cargo.toml`, `Cargo.lock`, `.gitattributes`, `lib.rs`, `main.rs` and
`assets/**` absent from the diff.

## Checkpoint 2 - final gates (sec7), all DIRECT form

| # | Command | Verbatim |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test` | `186 passed / 0 failed / 0 ignored / 1 suites` (capture `target/testrun-capture.txt`, 194 lines) |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui` | `186 passed / 0 failed / 0 ignored / 3 suites` (capture `target/testrun-capture.txt`, 206 lines) |
| 3 | `python tools/distill.py -- cargo check -p ariadne-cli --all-targets` | `ok (exit 0, 0 warnings emitted)` (capture `target/distill-capture.txt`; the capture shows `Checking ariadne-ui` + `Checking ariadne-cli` + `Finished`, i.e. it really re-checked, not cached-silent) |
| 4 | `python tools/distill.py -- cargo check -p ariadne-integration-tests --all-targets` | `ok (exit 0, 9 warnings emitted)` (181-line capture; every warning is in `tests\integration\tests\**`, none in `ariadne-ui`) |
| 5 | `python tools/distill.py -- cargo build -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` -- zero warnings proves no import or reference was missed |
| 6 | `python tools/hygiene_check.py` | R1 **0** lines, R2 **0** lines, **120** R3 warn lines -- the same 120 T3-D2/T3-E1 measured at this base; none introduced in a file I touched (line numbers in my files are unshifted or above my edits: `dto.rs:330/:418/:445` sit above my `:599`+ edits, `ui.rs:557+` below my `:328`, `graph_api.rs` `res` lines unchanged) |
| 7 | `python tools/encoding_guard.py --all` (x2, immediately before each commit) then `--staged` (x1, immediately after the code commit) | no output, all three forms (Dev-3 ordering) |
| 8 | `python tools/commit_scoped.py --scope ... -m ...` | two commits, explicit `--scope` per path; F12 fixture in neither |

Non-vacuity: gate 1 executes 186 with 0 ignored in 1 suite; gate 2's 3 suites
are lib + `ui_test` + doc-tests (`crates/ui/src/` carries zero `#[test]`);
gate 3/4 `--all-targets` covers lib, bin, tests and the bench target; gate 3
is the very command that failed in R1 minutes earlier. Count identity,
re-derived from my own capture: 186 - 0 + 0 = 186 (base and final identical).
**Workspace-wide line NOT measured, per sec7** (last measured
`1180/0/4/88` at this tip by the E1 reviewer). Not run and not worked around:
`testrun.py --baseline` (F9, broken), `unittest discover -s tools`,
`ui_smoke.py`, `bench_compare.py` (F10). `theme_token_check.py` not run: the
touch-set has no `crates/ui/assets/**` path, so zero new hex literals is
structural.

## Deviations

- **Dev-1 (contract documentation defect, planner; tree wins).** Sec4 predicted
  gate 1's Red error count as "17 in-crate + 2 in `crates/cli` = 19", with
  E0425-class errors naming callers. The observed count is **5 errors, code
  E0432** (`unresolved import`), because rustc reports the 5 failed imports and
  aborts resolution before enumerating the 17 call sites. I did NOT treat this
  as a census failure: the sec0.4 abort gate is the census arithmetic, and that
  closed exactly at 33 and at 1+19+5+2+3=30 before I edited anything. The
  compiler's own list of 5 files matches the census's 5 import-list files, and
  the final Green (186 tests + zero warnings in gates 3/5) proves the 17
  call-site repoints are complete. Correcting source: the 33-line capture and
  `cargo`'s own `due to 5 previous errors` line. This is the milestone's known
  defect class (F14: an unverified claim about the tree) applied to a claim
  about compiler output; remedy is the same -- state the claim with the
  command that establishes it.
- **Dev-2 (gate 4 warning count differs between its two runs).** Base `21
  warnings emitted`, final `9`. Both exit 0. All warnings in both runs are
  pre-existing in `tests/integration/**` (outside my touch-set); the count
  differs because cargo replays cached diagnostics only for the targets it
  re-checks, so the digest's warning tally depends on cache state, not on my
  edit. The contracted pin for gate 4 is `ok (exit 0)`, met both times; the
  zero-warning requirement belongs to gate 5, which reported `0 warnings
  emitted`.
- **Dev-3 (gate 7 ordering, pre-authorised by sec7).** `commit_scoped.py`
  stages and commits atomically and raw git staging is denied, so `--staged`
  cannot interpose. Ran the strictly stronger `--all` immediately before each
  commit and the literal `--staged` immediately after the code commit.
  Precedents: T3-D2 Dev-4 and T3-E1 Dev-4 (both APPROVED).
- **Dev-4 (two commits, not one).** Sec8's one commit carries EXACTLY the nine
  in-set paths (eight code + journal). The dispatch additionally waived me to
  land planner records that no other role can commit (reviewers hold read-only
  git); those went into their own `tasks/**`-only commit, precedent `ab59bb4`
  (T3-E1 commit 1), `4d0f245` (T3-D), `9f30c9a` (T3-B). The F12 fixture appears
  in neither.
- **Dev-5 (the dispatch's land-list does not match the tree).** The dispatch
  named `tasks/m235-t3d2-verdict.md` for landing; the tree says it is already
  committed in `ab59bb4` (T3-E1 commit 1), so nothing was owed. What the tree
  actually had uncommitted: `tasks/m235-t3e1-verdict.md` and
  `tasks/m235-t3e1-execution-results.md` (untracked) and the rewritten
  `tasks/m235-t3e2-handoff.md` (modified, the contract I am executing). I
  landed those three. Landing the contract is the same class as `ab59bb4`,
  which landed `m235-t3e1-handoff.md` and the superseded `m235-t3e-handoff.md`
  as records.
- **Sanctioned, not deviations:** the identifier-only repoints in the five
  `ui_test` files (sec1 item 6 puts `common.rs` in-set), the mandated D1-D7 doc
  edits, and the D7/D6 rewraps that necessarily change doc line lengths.

## Report-only observations (not acted on)

- **O-1 (D11 divergence, deliberately not reconciled).** `ui.rs:103` records
  `build_include_elements 5.1ms` while the bench header row at
  `payload_tail_bench.rs:41` records `5.4` for the same phase, and the bench's
  own T3.2 note already records an unreconciled `4.7`-vs-`5.8` split for the
  adjacent cycle row. Reconciling a figure is a perf claim, not a rename, and
  this milestone has no bench-compare evidence (F10). Left as-is per D11.
- **O-2.** `crates/cli/src/commands/ui.rs:323` (`let node_maps =
  node_maps::build(`) starts at column 1 while its continuation lines are
  indented. Pre-existing at `5addad7`, one line above my `:328` edit, compiles
  clean with zero warnings. Out of scope; a formatting-only fix belongs to
  whoever owns that file next.
- **O-3.** P2's locals are still `legacy_elements` / `grouped_elements` and its
  test name still says "parity comparison". Both were true of the pre-E1
  two-currency comparison; sec5 permits a callee-only change in the body, and
  the test name is a rename of a test, which sec5 forbids. Ledger F13's
  stale-identifier class; left for a disposition I cannot make.
- **O-4.** `tasks/milestone_ledger.md:85` (deferred row F13) still describes the
  two builders at `dto.rs:608`/`:637`; those lines are E1-era and both the
  names and the line numbers have moved. Ledger rows are operator-owned.

## Refusals encountered (verbatim, per sec9)

None. Every command in this session ran on its first independent attempt:
`checkpoint_guard.py status` (x3: preflight, after the test runs, after the
commits), `testrun.py` (x4), `distill.py` wrapping `cargo check`/`cargo build`
(x6), `certutil` (x2) and `git diff --numstat`
through `distill.py`, `hygiene_check.py`, `encoding_guard.py`,
`commit_scoped.py` (x2). The census greps ran through the `grep` tool because
raw `grep`/`git`/`cargo` are off the allowlist by contract, not by refusal.

## Commits

| SHA | Scope | Content |
|---|---|---|
| `378b240` | 3 explicit `--scope`s under `tasks/` | planner records: `m235-t3e2-handoff.md` 283/405 (the rewritten contract), pure additions `m235-t3e1-verdict.md` (699/0) and `m235-t3e1-execution-results.md` (151/0) |
| `4bc3b88` | 9 explicit `--scope`s = sec1/sec8 exactly | the rename, the 19 repointed call sites, the 5 import lists, D1-D7, and this journal as written through Checkpoint 2 |
| (this journal-only follow-up) | `tasks/m235-t3e2-journal.md` alone | sec9's "your own SHA in it": it records `4bc3b88`, which cannot sit in its own commit body, plus the two gate-count corrections above. Its own SHA: read `git log -1`. Precedent for a journal's SHA living outside the commit that carries it: T3-E1, whose verdict called that "correctly absent". |

Neither commit contains `tests/fixtures/basic/artifacts/includes/report.json`
(F12), `crates/graph/**`, `crates/report/**`, `crates/pipeline/**`,
`tests/**`, `tools/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`,
`crates/ui/src/lib.rs`, `crates/ui/tests/ui_test/main.rs` or any
`crates/ui/assets/**` path.
