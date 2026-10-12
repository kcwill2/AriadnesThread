# M23.5 T3-E2 Execution Results (addendum to the reviewer verdict)

Unit: `m235-t3e2` -- rename the surviving include-element builder to the canonical name.
Base `5addad7` (E1 tip), tip `dcd681b`, branch `tarjan-s-scc`.
Verdict: **APPROVED** (`tasks/m235-t3e2-verdict.md`). Reviewer measurement session; every number
below was produced by the reviewer, not copied from the implementer's journal.

## 1. Acceptance criteria, one by one, with the evidence line for each

| # | Contract obligation | Status | Evidence |
|---|---|---|---|
| 1 | Census closed sum is the exhaustiveness proof | **MET** | Independently re-derived. At base: 33 total = 30 `crates/` + 3 `tools/` + 0 `tests/`, decomposing as `1 + 19 + 5 + 2 + 3 = 30`. At HEAD: 29 = `1 + 19 + 5 + 1 + 3`; the one-line delta is `dto.rs:601`, whose D1 rewrite removed the token. Arithmetic closes with **zero residual**. |
| 2 | Contract's sec0.4 wording slip adjudicated | **MET (defect confirmed, planner)** | `` ...from_groups `` NOT followed by `(` returns **6**, not the predicted 7, because `dto.rs:601` spelled the PLAIN name. The slip is in the row's wording; the closed sum is unaffected because it is expressed over the union pattern. |
| 3 | Zero surviving occurrences of the old name | **MET** | `build_include_elements_from_groups` -> **no files found** under `crates/`; **no files found** under `tools/`; `tests/` carried zero hits for `build_include_elements` from the first census. |
| 4 | Exactly one `pub fn build_include_elements` in `crates/ui/src/dto.rs` | **MET** | `dto.rs:621`. Read at HEAD. |
| 5 | Third parameter `&[CycleGroup]`, 5-argument shape intact | **MET** | `cycle_groups: &[ariadne_graph::CycleGroup]` at `:624`; params `graph`, `closures`, `cycle_groups`, `file_languages`, `file_unit_kinds`; body a single `emit_include_elements(...)` call with `IncludeCycleClass::from_groups`. Plan line 69 satisfied verbatim. |
| 6 | Rename plus comments only, no behavioural diff | **MET** | Every hunk of `4bc3b88` read individually. All 19 call-site edits and all 5 import-list edits are callee-token-only with every argument a CONTEXT line. No assertion, key spelling, emission order or type moved. |
| 7 | A real Red was executed | **MET** | Reported `error[E0425]` at `crates/cli/src/commands/ui.rs:328:40`. Corroborated structurally: the pre-rename line was `ariadne_ui::build_include_elements_from_groups(`, and I counted the callee identifier to begin at **column 40 exactly**. The named caller is the out-of-crate **production** call in `build_ui_payload` -- the E blocker's blind spot. |
| 8 | Revert byte-for-byte by SHA256 | **MET** | `804680f3cf2e728d456b9730691cc4b61929489e58afd0a9efc4547092b15d94`, identical pre-Red and post-revert, recorded in both positions. Independently unnecessary: gate 1 is green against a frozen byte constant, which a partial revert could not survive. |
| 9 | Predicted error-count identity | **CONTRACT DEFECT (planner), 5th F14 instance** | Observed **5 errors, all `E0432`**, plus `due to 5 previous errors`. Structurally unobtainable: every in-crate caller imports the name, so rustc aborts resolution before enumerating call sites, and sec4 mandates repointing nothing. Substituted argument judged **sound and stronger** than the identity requested. |
| 10 | D8-D13 / the "do not edit" list respected | **MET** | D8 label `payload_tail_bench.rs:390`, D9 table row `:41`, D10 sentence `:62-63`, D11 row `ui.rs:103`, D12 at `payload_tail_bench.rs:80-87` **and** `ui.rs:115-119`, D13 module doc and generic prose: all absent from the diff and read byte-unchanged at HEAD. |
| 11 | The three `tools/` bench-baseline keys untouched | **MET** | No `tools/**` path in any of the three commits. Keys still at `bench_compare.py:53`, `bench_baseline.json:6`, `bench_compare_tests.py:37`. Because D8 was left alone, the emitted label and the keys that match on it still agree. |
| 12 | D1-D7 actually fixed | **MET** | All seven present in the diff. D7's citation **verifies against git**: `git show --stat acea7f2` is the `delete detect_cycles/dfs_cycle` commit and `git log --oneline adb53a3..aebfa9b` confirms the approved range. D7's parentheses balance; no timing number moved. |
| 13 | Contested judgments `:62` no-edit / `:63-64` fix / `ui.rs:103` no-edit | **MET, both agreed** | `:62-63` is historical narration of a re-pin -- both names correct, rewriting it would destroy the only record the column was ever named otherwise. `:63-64` was a **live false present-tense statement** ("`detect_cycles` still exists until..."), correctly fixed. `ui.rs:103` becomes *true* by the rename; its `5.1`-vs-`5.4` residual is a perf claim and F10 is open, so not reconciling it was required, not optional. |
| 14 | No test added or removed | **MET** | No `#[test]` added/removed and no test `fn` added/removed/renamed in the diff. Measured `186` at base and `186` at final; workspace delta `+0`. |
| 15 | P1 lock, single-invocation, P2 `assert_ne!` intact | **MET** | `ACYCLIC_EMPTY_CURRENCY_PAYLOAD` pure CONTEXT (byte-unchanged); P1 body calls the builder **once** at `:423` with the anti-tautology comment intact; guards at `:433`/`:451`/`:455` still **ahead** of the byte lock at `:471`; P2's `assert_ne!` at `:327` intact with `:311` (`&[]`) vs `:318` (`&groups`) on the same name -- still a real cross-currency comparison. |
| 16 | Commit contains exactly the in-set paths, no fixture | **MET for `4bc3b88`**; two `tasks/**`-only commits adjudicated separately | `4bc3b88` = the 8 in-set code files + journal, exactly the nine `--scope`s. `378b240` and `dcd681b` are `tasks/**`-only, planner-declared, disclosed, precedent-backed -- APPROVED, with the standing boundary recorded that an out-of-band commit is benign only while it carries no code. |

## 2. Scoped gate lines, verbatim

```
186 passed / 0 failed / 0 ignored / 1 suites
186 passed / 0 failed / 0 ignored / 3 suites
ok (exit 0, 0 warnings emitted)          # cargo check -p ariadne-cli --all-targets
ok (exit 0, 21 warnings emitted)         # cargo check -p ariadne-integration-tests --all-targets
ok (exit 0, 0 warnings emitted)          # cargo build -p ariadne-ui
```

Hygiene: **R1 0 lines, R2 0 lines**, output entirely R3 warns, none newly introduced in any
touched file (proved structurally: this unit's edits are one-line-for-one-line in every
touched file except three with mandated doc rewraps, so no line number shifted, and
`dto.rs:330/:418/:445` sit far above the first edit at `:599` while `ui.rs`'s sit below its
only edit at `:328`). Encoding `--all` and `--staged`: **no output**; `--all` is the
load-bearing form, `--staged` is necessarily silent at review time on a clean index.

None of the five test/compile lines is vacuous. Gate 5's zero-warning result is the specific
proof that no import or reference was missed.

## 3. Workspace-wide acceptance pass -- measured, and the delta

Canonical line, verbatim:

```
1180 passed / 0 failed / 4 ignored / 88 suites
```

| | passed | failed | ignored | suites |
|---|---|---|---|---|
| last measurement (E1 tip `5addad7`) | 1180 | 0 | 4 | 88 |
| **measured at the T3-E2 tip `dcd681b`** | **1180** | **0** | **4** | **88** |
| **delta** | **+0** | **+0** | **+0** | **+0** |

Zero on all four axes is the correct answer for a rename that adds and removes no test, and it
is the independent confirmation of criterion 14 at workspace scope rather than crate scope.

Tools suites (what CI runs):

```
Ran 184 tests in 20.778s

OK
```

Workspace `hygiene_check.py`: R1 0, R2 0 (same invocation as the scoped gate; workspace-wide by
construction).

## 4. Explicitly NOT run in this session

- **`python tools/testrun.py --baseline`** -- broken, F9, reproduced verbatim: `usage error:
  missing value for --baseline`. Not retried, not fixed, not worked around; the workspace pass
  ran without the flag and was compared by hand against the ledger's baseline chain, which is
  F9's own protocol.
- **`python tools/ui_smoke.py`** -- REFUSED, F10, verbatim:
  `The user has specified a rule which prevents you from using this specific tool call. Here
  are some of the relevant rules
  {"permission":"bash","pattern":"*","action":"deny","source":"global"}`
- **`python tools/bench_compare.py`** -- REFUSED, F10, identical verbatim text.
- **`rg` / `git grep` census commands** -- refused verbatim (4 invocations). The census was
  completed instead with the dedicated `grep`/`read` tools and the commit diff's deleted side.
- **`theme_token_check.py`** -- not run, correctly: no `crates/ui/assets/**` path exists in any
  of the three commits, so "zero new hex literals" is structural rather than asserted.

Each refusal was attempted **independently**, exactly once, and **none was routed through
`distill.py`**. No gate was dropped because a different invocation was refused.

## 5. What the Phase 4 closeout planner must carry forward

1. **M23.5 acceptance criterion 1 is FULLY DISCHARGED for production code.** `CycleGroup` is the
   only cycle currency; `from_paths` and `build_include_elements_legacy_paths` are zero hits;
   the public `IncludeGraph::detect_cycles()` has zero production hits; exactly one public
   include-element builder survives (`pub fn build_include_elements`, `dto.rs:621`, third param
   `&[CycleGroup]`), its emission tail `emit_include_elements` is **private**, and
   `crates/ui/src/lib.rs` is still the bare `pub use dto::*;` so **no alias or shim survives**.
   Record it with two caveats: the **deliberate test-local oracle**
   `crates/graph/tests/cycle_groups_equivalence_test.rs`'s frozen `old_detect_cycles` /
   `old_dfs_cycle` is retained **by design** as the equivalence evidence and must not be
   deleted; and `crates/transform/src/engine/mod.rs:133`'s `detect_cycles_in_units` is an
   unrelated subsystem guard, outside M23.5.
2. **F10 is the most material unclosed gate and this review does NOT close it.** Both
   `ui_smoke.py` and `bench_compare.py` are refused. There is still zero runtime evidence that
   the UI cold-boots, routes, serves `Cache-Control: no-store` and returns a group-count
   report, and no bench-compare evidence for the payload timing after T3-D's SCC swap. D12's
   `UNVERIFIED` paragraph is still the only in-tree statement of that gap. **A green
   compiler-level pass does not substitute for it**, and the milestone is not closed until an
   operator runs both.
3. **Ledger row F13 needs an operator edit before closeout.** `tasks/milestone_ledger.md:85`
   still describes two builders at `dto.rs:608`/`:637` -- both names and both line numbers are
   E1-era and now describe a state this milestone deliberately destroyed. F13 also still owns
   P2's stale locals `legacy_elements` / `grouped_elements` and its stale test name
   `the_parity_comparison_detects_a_cycle_classification_difference`, which this unit was
   **forbidden** to rename (sec5) and therefore correctly left (implementer observation O-3).
4. **Five contract defects attributed to the planner**, all instances of F14 ("an unverified
   claim about the tree", here extended to claims about compiler output), all non-blocking:
   sec4's unobtainable error-count identity; sec0.4's census-row wording slip; sec0.4's
   files-vs-token-hits conflation for `crates/cli`; sec7's non-deterministic gate-4 warning
   pin; and F14/2's still-ambiguous trigger wording. The standing remedy is F14's own written
   one -- state every claim with the command that establishes it.
5. **The `tasks/**` out-of-band landing waiver is now the fourth-plus occurrence.** It remains
   necessary because the reviewer role holds read-only git, but `handoff-spec` should carry the
   standing clause. Standing boundary for the next reviewer: an out-of-band commit is benign
   **only** while it carries no code path.
