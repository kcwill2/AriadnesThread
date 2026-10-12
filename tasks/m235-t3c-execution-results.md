# M23.5 T3-C Execution Results - workspace-wide acceptance addendum

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Verdict file:** `tasks/m235-t3c-verdict.md` (APPROVED, 0 blocking / 7 non-blocking)
**Handoff:** `tasks/m235-t3c-handoff.md`
**Range:** `91967c9..6c0c2b6`, branch `tarjan-s-scc`, worktree
`.kilo/worktrees/tarjan-s-scc`
**Commits:** `b774362`, `d11a819`, `6c0c2b6`

This addendum records what **actually happened** against each acceptance criterion,
with the evidence line for each, and an explicit statement of anything that was not
run. It does not restate the audit; the reasoning lives in the verdict.

---

## 1. Ground truth at review time

`python tools/checkpoint_guard.py status`, reviewer-run:

| Signal | Value |
|---|---|
| HEAD | `6c0c2b6` |
| branch | `tarjan-s-scc` |
| DIRTY FILES (`git diff HEAD --numstat`) | `(none)` |
| UNTRACKED FILES | `(none)` (later: this review's own two `tasks/` files) |
| census `dirty=` this worktree | `1` = the F12 stat-cache entry |

Base `91967c9` confirmed as the T3-B tip and as the commit immediately preceding
`b774362`; the reviewed range contains exactly the three declared commits.

---

## 2. Acceptance criteria, outcome against evidence

### Plan criterion 6 - "Cycle-policy seam preserved (IV-11): callback signature,
`informational_cycle_count` name + skip-when-zero serde shape, all-`Reject` C++
artifact byte-stability; m20/m22 migrated with stance preserved"

**MET IN FULL. This is the whole of this unit's deliverable and it is clean.**

| Element | Evidence |
|---|---|
| callback signature preserved | `generate_report_with_cycle_policies(&dyn Fn(FileId) -> CyclePolicy)` still called from `m20:161` and `m22:448`; `crates/**` absent from every commit |
| `informational_cycle_count` name preserved | `crates/graph/src/types.rs:62`, untouched |
| skip-when-zero serde shape preserved | `types.rs:61-62` (`#[serde(default, skip_serializing_if = "informational_cycles_absent")]`) over the predicate at `types.rs:69-71`, all untouched |
| all-`Reject` C++ artifact byte-stability | goldens byte-identical (blob SHA-1 unchanged); gate 2 green; `crates/graph/src/graph.rs` in no commit |
| m20 migrated with stance preserved | gate 3 = `5 passed / 0 failed / 0 ignored / 1 suites`, migrated test `fixture_cycle_is_informational_under_tolerate_policy ... ok` |
| m22 migrated with stance preserved | gate 4 = `6 passed / 0 failed / 0 ignored / 1 suites`, migrated test `auto_shaped_graph_keeps_the_cycle_informational ... ok` |

**Zero weakening, proved structurally:** the entire range is **four hunks at zero
context** - `m20:154`, `m20:156-157`, `m22:442`, `m22:444-445`. The assertions the
handoff sec1 froze (`m20:160-189`, `m22:447-453`) are not even diff CONTEXT lines;
they are untouched bytes. The only truth-condition change in the whole unit is
`cycles.len() >= 1` -> `!cycle_groups.is_empty()`, which is the **same predicate**.

### Plan criterion 2, second clause - "`m3_include_graph_test` green without edit"
(IV-9 proof)

**MET. This is the headline proof and it holds byte-for-byte.**

| Element | Evidence |
|---|---|
| the file is unedited | **identical blob SHA-1** `b6a54d5881dcf3d82ae8fcf56885fabd7e892a23` at `91967c9` and at `HEAD`; `git diff 91967c9..HEAD -- <path>` empty; absent from all three per-commit stat lists |
| it is green | gate 1 = `3 passed / 0 failed / 0 ignored / 1 suites` |
| the assertion actually ran | capture names `test_m3_edge_cases_duplicates_system_missing_cycles ... ok` - the exact test carrying `report.cycle_count > 0` at `m3_include_graph_test.rs:118-121` |
| nothing was filtered out | capture line 7 reads `0 filtered out` |
| the lock is non-vacuous | `> 0` is load-bearing: `>= 0` would be a tautology; a direct `detect_cycle_groups()` call would bypass the field NAME, which is the contract IV-9 exists to witness |

### Plan criterion 2, first clause - "`cycle_count` reports entangled-group count"

**NOT YET, BY DESIGN - not this unit's scope.** `crates/graph/src/graph.rs:369`
still reads `cycle_count: detected_cycles.len()` over DFS paths, and `:356` still
calls `detect_cycles()`. Unit D owns that port. **Action recorded as verdict
finding 5 for unit D's reviewer:** re-run `m3_include_graph_test` after the port and
treat that run as the real milestone-level IV-9 gate; re-pin the four absolute
`assert_eq!(report.cycle_count, 1, ...)` at `graph.rs:617/:631/:640/:647`.

### Plan T3.1 golden instruction - "re-pin any golden containing concrete cycle
lists" and "`m3_include_graph_test` passes without edit"

**The re-pin was provably unnecessary and was NOT made. Zero golden bytes touched.**

| Element | Evidence |
|---|---|
| the grep | `cycle_count` over `tests/golden/**` = **exactly 4 hits**, all `"cycle_count": 0` (`basic/includes_report.json:2`, `expensive/includes_report.json:2`, `basic/includes_manifest.json:3`, `expensive/includes_manifest.json:3`) |
| byte-identity | blob SHA-1 `dff8018df38d9fe42d540a1dbc882eb263b4d091` identical at `91967c9` and `HEAD`; `git diff --name-only 91967c9..HEAD -- tests/golden` **empty** (so all four, not just the one hashed) |
| no cycle list can hide under another key | `IncludeReport` (`types.rs:47-64`) has **no cycle-list field**; `CycleGroup` is unreachable from it at this HEAD |
| `UPDATE_GOLDENS=1` never run | switch documented at `m00_golden_baseline_test.rs:9`, read at `common/mod.rs:63-65`; byte-identity proves no write occurred; gate 2 green against the untouched blobs |
| fixtures untouched | `git diff --name-only 91967c9..HEAD -- tests/fixtures` **empty**; numstat `(none)` after the heaviest run; no file added under `tests/fixtures/basic` or `tests/fixtures/expensive` |
| why the byte is stable, not just equal today | both golden fixtures are **acyclic**, so 0 paths implies 0 nontrivial SCCs (IV-2), hence 0 groups: the byte is representation-**invariant** |

### Handoff sec9 frozen locks - all HOLD

Every row verified; full table in verdict section 12. Summary of the ones the
milestone's sequencing depends on:

- `detect_cycles` EXISTS at `graph.rs:234`; `dfs_cycle` EXISTS at `graph.rs:260`.
- **Both still CALLED** from all three surviving sites: `graph.rs:356`,
  `graph.rs:534`, `closure_equivalence_tests.rs:515`. This is unit D's delete-gate
  precondition and it is intact.
- **Zero** `detect_cycles()` occurrences remain under `tests/**` - this unit's
  migration is complete, which is exactly what unit D needs.
- IV-9 `cycle_count` retained (`types.rs:56`); IV-11 `informational_cycle_count`
  retained with skip-when-zero serde intact; the six `graph.rs:614-687` IV-11 unit
  tests unedited; the policy callback signature untouched.
- `crates/lang-scala/**` untouched (IV-12 plugin-side-free): `-- crates` empty.
- `m00_facts_path_parity_test.rs` (IV-8) and `m23_symbol_isolation_test.rs` both
  unedited and both green in the whole-crate run.
- The unpinned F12 fixture `tests/fixtures/basic/artifacts/includes/report.json` is
  in **no** commit of the range.

### Handoff sec2 touch-set - PASS

`git diff --name-only 91967c9..HEAD` = exactly five paths: the two touch-set test
files, `tasks/m235-t3c-journal.md`, and the two orchestrator-declared T3-B reviewer
records. `-- crates`, `-- tools`, `-- tests/golden`, `-- tests/fixtures` all empty.
Every commit carried an explicit `--scope`.

### Handoff sec5 "no new tests" - CONFIRMED

| File | Attributes at HEAD | Total | Gate |
|---|---|---|---|
| `m3_include_graph_test.rs` | `#[test]` `:14`, `:79`, `:146` | 3 | `3/0/0/1` |
| `m00_golden_baseline_test.rs` | `#[test]` `:39`, `:44` | 2 | `2/0/0/1` |
| `m20_scala_dependency_graph_test.rs` | `#[test]` `:85`, `:120`, `:149`, `:194`, `:236` | 5 | `5/0/0/1` |
| `m22_scala_load_analysis_test.rs` | `#[test]` `:119`, `:183`, `:247`, `:413`, `:436` + `#[tokio::test]` `:300` | 6 | `6/0/0/1` |

All eight+ line numbers match handoff sec8 exactly - **zero line drift** against the
handoff's stated `b950963` pins. No attribute appears on any `+`/`-` line anywhere
in the range, so the test inventory is provably unchanged.

### Handoff sec6 hygiene targets - MET

Zero R1/R2 workspace-wide; every printed line is `R3 (warn)`. Neither file this unit
edited appears in the hygiene output at all (the scanner demonstrably reaches
`tests/integration/**` - it flags `common/mod.rs:218`, `m5_reduced_model_test.rs:86`
and `m3_include_graph_test.rs:149`/`:150` in the same tree). No `tmp`/`res`/`foo`/
`bar`/`test1` introduced, no single-letter local authored, no line >= 2000 chars.
File sizes 258 (`m20`) and 455 (`m22`), both well under the 1000-line ceiling and
both net-smaller per handoff sec6.

### Handoff sec4 "no Red" - ACCEPTED as the correct call

No genuine failing-first moment exists; every premise verified independently
(`detect_cycle_groups` public at `cycle_groups.rs:39`; plan line 108 independently
forbids deleting `detect_cycles` from a non-`graph`-scoped unit). Red-equivalent
evidence: gate-5 identity pair `139 / 0 / 0 / 34` before and after, plus **both
migrated call paths named PASSING** - a positive witness, not merely a non-failure.

---

## 3. Canonical lines, reviewer-measured, verbatim

| Gate | Command | Line |
|---|---|---|
| 1 (IV-9) | `testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m3_include_graph_test` | `3 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `... --test m00_golden_baseline_test` | `2 passed / 0 failed / 0 ignored / 1 suites` |
| 3 | `... --test m20_scala_dependency_graph_test` | `5 passed / 0 failed / 0 ignored / 1 suites` |
| 4 | `... --test m22_scala_load_analysis_test` | `6 passed / 0 failed / 0 ignored / 1 suites` |
| 5 (crate) | `testrun.py --distill --timeout 3600 -p ariadne-integration-tests` | `139 passed / 0 failed / 0 ignored / 34 suites` |
| **workspace** | `python tools/testrun.py --distill --timeout 3600` | **`1176 passed / 0 failed / 4 ignored / 88 suites`** |
| tool suites | `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 184 tests in 18.152s` / `OK` |
| hygiene | `python tools/hygiene_check.py` | zero R1/R2; all lines `R3 (warn)` |
| encoding | `python tools/encoding_guard.py --all` and `--staged` | no output, exit 0 (both) |

### The workspace delta, attributed

**COMPLETED** - `1176 / 0 / 4 / 88`, delta **+0 / +0 / +0 / +0** against the ledger
baseline the brief named, exactly as expected for a unit that adds no test.

| | passed | failed | ignored | suites | delta |
|---|---|---|---|---|---|
| `b950963` (ledger chain end, pre-T3-A) | 1169 | 0 | 4 | 88 | - |
| `ea82422` (T3-A measured) | 1174 | 0 | 4 | 88 | +5 |
| `91967c9` (T3-B measured) | 1176 | 0 | 4 | 88 | +2 |
| **`6c0c2b6` (T3-C reviewer-measured)** | **1176** | **0** | **4** | **88** | **+0** |

`+0` is correct and fully explained: T3-C's diff adds and removes no test
anywhere, which the zero-context diff proves structurally. **Zero unattributed
residual.**

Non-vacuity of the workspace line: the capture contains exactly **88 suite headers**
(74 `Running ...` + 14 `Doc-tests ...`) and **zero** `test result: FAILED` /
`failures:` lines. Capture `target/testrun-capture.txt`, 2064 lines, exit 0.

Gate 5 non-vacuity: 34 `test result:` lines, **all 34 reading `0 filtered out`**;
`1 (lib) + 32 (targets) + 1 (doc-tests) = 34` suites; the 34 `passed` fields sum to
**139** with nothing left over. The crate's per-target counts agree with the four
scoped gates read from the same capture.

---

## 4. What was NOT run - stated plainly

| Item | Why | Attempt |
|---|---|---|
| `python tools/testrun.py --baseline` | **BROKEN** (ledger M23.5 F9). Verbatim: `usage error: missing value for --baseline` | once, verbatim recorded, **not** treated as a gate failure, **not** worked around. The invocation it wraps WAS run (section 3). |
| `python tools/ui_smoke.py` | **REFUSED** (ledger M23.5 F10). Verbatim: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}` | once, directly. **Not** retried, **not** routed through `distill.py`. |
| `python tools/bench_compare.py` | **REFUSED** (ledger M23.5 F10), identical verbatim text | once, directly. **Not** retried, **not** routed through `distill.py`. |
| The implementer's BEFORE gate-5 capture | never committed (correct), and `target/` is ephemeral | the line survives only in the committed journal. **Not a shortfall:** the AFTER side was reproduced exactly and the inventory delta is proven structurally 0. |
| The temporal ordering of that BEFORE measurement | a read-only reviewer cannot replay another session's edit timeline | **taken on report** - recorded in verdict section 9 as the one element not independently reproducible. |
| Any commit by this reviewer | write git is denied to the reviewer role | n/a |

**Impact of the two refusals for this unit: unchanged from T3-B.** This unit makes no
UI change and pins no timing. The standing gaps remain: **no runtime UI evidence**
that the UI cold-boots / routes / serves the group-currency payload its bytes now
carry, and **no verified perf evidence** for `detect_cycle_groups` (the pinned `5.6`
remains a single local measurement, labelled as such at both sites by T3-B).

**A refusal never narrowed the acceptance set.** Hygiene, `encoding_guard --all`,
`encoding_guard --staged`, `checkpoint_guard status` (three times), all four scoped
gates, the whole-crate integration run, the full-workspace run and the Python tool
suite all ran. Each refusal above reports the catch-all deny rule regardless of the
real cause; the real cause here is that `ui_smoke.py` and `bench_compare.py` are not
on the reviewer bash allowlist, **not** that bash is unavailable.

---

## 5. Follow-up dispositions (recorded, not dropped)

| # | Disposition | Owner | Trigger |
|---|---|---|---|
| verdict finding 1 | `handoff-spec` gates template: gate 9 is unsatisfiable alongside the header's journal-SHA instruction | planner | before T3-D |
| verdict finding 2 | ledger row `-p ariadne-cli = 86/0/1/3` is wrong (true value **5**); reconfirmed at workspace-capture lines 1979-1980 | planner | M23.5 closeout |
| verdict finding 3 | ledger M23.5 row is three units stale ("Phase 3 not started", `HEAD b950963`, chain ending `1169/0/4/88`); add the missing crate-scoped row `-p ariadne-integration-tests = 139/0/0/34` | planner / orchestrator | **before T3-D is dispatched** |
| verdict finding 4 | golden no-op reasoning should name both `includes_report.json` and `includes_manifest.json` | planner | when reused |
| verdict finding 5 | unit D must re-run `m3_include_graph_test` post-port as the real IV-9 gate, and re-pin `graph.rs:617/:631/:640/:647` | unit D's reviewer | unit D |
| verdict finding 6 | F10 (`ui_smoke`, `bench_compare`) close | configuration / operator | before T4.1 |
| verdict finding 7 | F9 (`testrun.py --baseline` requires a missing FILE) | configuration | before T4.1 |
| **carried from T3-B finding 3** | the `build_include_elements` vs `_from_groups` companion parity test has a **hard expiry at unit E**; it needs its own `crates/ui`-only unit sequenced BETWEEN D and E | planner | **before unit E is dispatched** |
| **carried, ledger F12** | pin `tests/fixtures/basic/artifacts/includes/report.json` in `.gitattributes`, or make the writer emit LF. Fifth identical reproduction this unit | configuration | before T4.1 |

---

## 6. Verdict

**APPROVED** - 0 blocking / 7 non-blocking. Range `91967c9..6c0c2b6`, three commits,
all inside the declared touch-set plus the orchestrator-declared records, forbidden
fixture in none of them. See `tasks/m235-t3c-verdict.md` for the full adjudication.