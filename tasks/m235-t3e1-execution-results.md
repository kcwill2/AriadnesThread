# M23.5 T3-E1 Execution Results

Unit: `m235-t3e1` -- retire the path-based include-element DTO (crate `crates/ui`).
Dispatch base `b0a91d2`; tip `5addad7`; branch `tarjan-s-scc`.
Verdict: **APPROVED** -- `tasks/m235-t3e1-verdict.md`, zero blocking findings against the unit.
Companion record: `tasks/m235-t3e1-journal.md` (implementer), `tasks/m235-t3e1-handoff.md`
(contract).

This addendum states what actually happened against each acceptance criterion, with the evidence
line for each, and names explicitly what was not run. It is the workspace-wide evidence the
handoff's sec7 forbade the implementer from producing; producing it is the reviewer's job.

---

## 1. Acceptance criterion-by-criterion

| # | Criterion (handoff sec) | Outcome | Evidence |
|---|---|---|---|
| 1 | Census re-derived at base, three greps, no STOP condition | **MET** | Re-derived at the tip and closed arithmetically: 20 hits = 19 call sites + 1 definition; legacy-name call sites **0**; `from_paths` **0** hits under `crates/`; `pub fn build_include_elements` exactly one at `dto.rs:618`. Journal's census matched line-for-line. |
| 2 | F12 fixture handled per "if and only if it prints rows" | **MET** | `git diff --numstat HEAD -- <path>` printed **zero content rows** (only the autocrlf stat warning). Nothing restored, nothing written. Path in **NO** commit. Measured an 8th time. |
| 3 | Delete legacy entry point + its doc paragraph; move the D27 paragraph onto the survivor | **MET** | `dto.rs` 4/23. Legacy `pub fn` and its 5-line doc paragraph gone; D27 paragraph now at `:599-606` on the survivor. `dto.rs:625-626`-equivalent left ALONE as sec3.1 required (E2's obligation). |
| 4 | Delete `from_paths` + doc + separating blank; amend module doc `:1-9`; nothing else in that file | **MET** | `cycle_groups_view.rs` 6/36. `from_groups` and everything below untouched. No `#[allow(dead_code)]`, no suppression of any kind. Module doc no longer names a deleted function. |
| 5 | Repoint the nine in-crate call sites, callee name only | **MET** | All nine now call `..._from_groups` (17 grouped call sites in `ui_test/`). Zero type annotations changed; one local rename (`dto_builders.rs:13`) expressly permitted by sec3.3. |
| 6 | Collapse the four two-name import lists | **MET (as corrected, Dev-2)** | Legacy name dropped from `export_order.rs:15`, `dto_builders.rs:4`, `graph_api.rs:4`, `d27_node_provenance.rs:9`. Each list names exactly one builder. `common.rs:3` edited too (Dev-1, contract-forced). |
| 7 | P1 is CONVERTED, not deleted; exactly ONE builder invocation; structural lock DELETED not ported; guards AHEAD of the byte lock; P2 keeps its `assert_ne!` | **MET** | `acyclic_include_elements_match_the_frozen_acyclic_payload` at `export_order.rs:391`; single call at `:422`; no `assert_eq!` over two builder results anywhere; four guards at `:432-465` upstream of the lock at `:471`; P2 keeps `assert_ne!` at `:326` with `&[]` vs `&groups`. Discharges D2 finding F2. |
| 8 | Exactly one test changed; none added, none removed | **MET** | Gate 1 `186/0/0/1` at base and at tip; gate 2 `186/0/0/3`. 186 - 0 + 0 = 186. |
| 9 | R1 executed: a compile FAILURE proving the nine-site set was complete | **MET** | `error[E0425]: cannot find function `build_include_elements` in this scope` at `export_order.rs:91:20`; `could not compile ariadne-ui (test "ui_test") due to 1 previous error`. **Exactly one** error = no missed site. Never committed. |
| 10 | R2 executed: one-character mutation of the frozen expectation, failing AT the byte lock | **MET** | `translation-unit` -> `transmation-unit`, counted first as exactly one occurrence inside the const (`:348`; the second is the input map at `:417`). Result `185 passed / 1 failed / 0 ignored / 1 suites`. Panic at `export_order.rs:471:5` -- which in the committed tree IS the byte lock, itself proof the mutation preserved line count. Message names the frozen-payload contract. |
| 11 | Revert byte-for-byte | **MET** | SHA256 `34ef1356...e2ae2f` pre-mutation and post-revert; final green gate 1 confirms the committed constant equals live output. |
| 12 | Capture obtained honestly, temporary line removed | **MET** | One temporary line writing to the OS temp dir, run once, deleted before any commit. `export_order.rs` numstat `80/50` contains no capture code. Provenance (committed parity lock at `42bb44e`, not the capture alone) stated in the new test header at `:365-389`. |
| 13 | Gate 3 ran at base AND after, green both times | **MET** | `ok (exit 0, 0 warnings emitted)` at the tip; implementer recorded the same at base. The systemic lesson of the E blocker is discharged. |
| 14 | Gate 4 zero warnings at both ends | **MET** | `ok (exit 0, 0 warnings emitted)`. The proof that `from_paths` is really gone and no dangling import survived. |
| 15 | Gate 5 zero R1/R2, no NEW R3 in a touched file | **MET** | R1: 0 lines. R2: 0 lines. No R3 line names `export_order.rs`, `cycle_groups_view.rs`, `common.rs`, `dto_builders.rs` or `d27_node_provenance.rs`. The `dto.rs:330/:418/:445` and `graph_api.rs` R3 lines are provably pre-existing: `dto.rs`'s every deletion is at `:604`+ and `graph_api.rs` is +3/-3, so no line number shifted. |
| 16 | Gate 6 encoding clean | **MET** | `--all` no output; `--staged` no output. Covers the pure-ASCII requirement on the new constant and every authored doc line. |
| 17 | Two commits, explicit `--scope`, byte-as-found records | **MET** | `ab59bb4` = 7 `tasks/**` paths, zero code; `5addad7` = the 7 code paths + the journal. No `git add .`, no amend, no Red committed. |
| 18 | Touch-set hard; anything outside is a blocker | **MET** | Zero paths outside the declared touch-set in either commit. `crates/cli/**`, `crates/graph/**`, `crates/report/**`, `tests/**`, `tools/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`, `crates/ui/src/lib.rs`, `crates/ui/tests/ui_test/main.rs` all absent. No `assets/**` path, so zero new hex literals is structural. |
| 19 | `theme_token_check.py` omitted, with the structural reason | **MET, reasoning HOLDS** | The touch-set has no `assets/**` path, and I ran the tool anyway: `--check-added --base b0a91d2` returns **no output**. Zero added hex literals, confirmed rather than assumed. |
| 20 | Workspace line: not the implementer's to measure | **N/A by contract, and it was not measured** | See section 3 -- measured here instead, because this unit deleted production code. |

## 2. Canonical gate lines, verbatim

Scoped, each run directly and in its own turn:

```
gate 1  186 passed / 0 failed / 0 ignored / 1 suites
gate 2  186 passed / 0 failed / 0 ignored / 3 suites
gate 3  ok (exit 0, 0 warnings emitted)
gate 4  ok (exit 0, 0 warnings emitted)
gate 5  R1: 0 lines   R2: 0 lines   (R3 warns only, none new in a touched file)
gate 6  no output (--all)   no output (--staged)
```

Two gates added by this review, beyond the contract's set:

```
F14/2   cargo check -p ariadne-integration-tests --all-targets
        ok (exit 0, 21 warnings emitted)
theme   theme_token_check.py --check-added --base b0a91d2
        no output
```

The 21 warnings are pre-existing in `tests/integration/**`, which this unit's touch-set cannot
reach.

## 3. Workspace-wide acceptance -- RUN, not deferred

The handoff forbade the implementer to measure the workspace line, which was correct for a
test-only unit. This unit deleted production code and changed a crate's public surface, so the
deferral does not apply to the reviewer. It ran.

```
1180 passed / 0 failed / 4 ignored / 88 suites
```
(`### cargo test PASS (exit 0)`, capture `target/testrun-capture.txt`, 2070 lines.)

```
Ran 184 tests in 18.423s

OK
```
(`python -m unittest discover -s tools -p "*_tests.py"` -- what CI runs.)

### Measured delta against the ledger baseline

| | passed | failed | ignored | suites |
|---|---|---|---|---|
| baseline, measured at `aebfa9b` | 1178 | 0 | 4 | 88 |
| **measured at `5addad7`** | **1180** | **0** | **4** | **88** |
| delta | **+2 / +0 / +0 / +0** | | | |

Zero residual, and it closes a deferral this milestone had been carrying: D2's `+2` was a
**prediction** (`1180/0/4/88`) throughout, because D2's reviewer deferred workspace measurement
to T4.1. This run converts it to a **measurement** -- 1180 is exactly `1178 + 2`, so D2's two new
tests are confirmed present in the workspace line and not only in the crate-scoped one. E1's own
contribution is `+0` as predicted: one test rewritten and renamed, none added or removed.
`failed`, `ignored` and `suites` unmoved is the signal that the deletion removed no test and the
conversion removed no suite. Ledger row **F11**'s Phase-1 `+6` residual remains confined to
Phase 1 and is untouched by this measurement.

## 4. Not run, with verbatim evidence

Each attempted **once**, independently. No refusal was retried, worked around, or routed through
`distill.py`.

**F10 -- `ui_smoke.py` and `bench_compare.py`, both permission-refused.** Verbatim, identical for
both:

```
The user has specified a rule which prevents you from using this specific tool call.
Here are some of the relevant rules
{"permission":"bash","pattern":"*","action":"deny","source":"global"}
```

**This is the most material unclosed gate in the milestone and it is NOT closed by this review.**
There is still **zero runtime evidence** that the UI cold-boots, serves its routes, returns
`Cache-Control: no-store`, and reports a group count -- and T3-D changed what
`IncludeReport::cycle_count` *means*, which is exactly the number the UI include view and the
CLI "Cycle Groups Detected" label display. Likewise there is still **no `bench_compare.py`
evidence** for the payload timing after T3-D changed what `generate_report()` computes. Both must
be closed by an operator who amends the permission allowlist, or by an authorized session, before
T4.1 closeout.

**F9 -- `testrun.py --baseline`, broken not refused.** Verbatim: `usage error: missing value for
--baseline`. Not attempted again, not fixed, not worked around. The workspace pass was run
without `--baseline` and compared **by hand** against the ledger's baseline chain, which is the
protocol F9 itself prescribes.

**Session-start refusals, recorded because they shaped the method.** `Test-Path ...` and
`git status --porcelain` were both refused with the same catch-all text before any gate ran.
Neither caused a gate to be skipped: `python tools/hygiene_check.py` then ran unmodified on the
next attempt, and read-only git was obtained through the `git_git_*` MCP tools (`log`, `show`,
`diff`, `status`). The allowlist is narrow, not inert -- a boilerplate denial is never a reason
to skip the next gate.

## 5. What the next unit must be told

`tasks/m235-t3e2-handoff.md` **cannot land as written.** Full detail in the verdict's finding 1;
the short form:

1. It freezes `crates/ui/tests/ui_test/common.rs` "byte-unchanged" on the premise that its import
   at `:3` already names `build_include_elements`. It does not -- E1's contract-forced Dev-1
   resolution left it naming `build_include_elements_from_groups`. E2's rename would leave a
   dangling import behind an explicit "MUST NOT touch".
2. Its 0.3 census pins **11** `..._from_groups(` call sites; the measured count is **19**, and
   its table cites `export_order.rs:382`, which does not exist (P1's call is at `:422`).
3. Its sec2 touch-set names only 9 of the 19 call sites needing the rename, because it inherited
   E1's inverted-name error.

All three surface as ordinary compile failures that E2's own gates 1 and 3 would catch, so they
cannot reproduce the silent milestone-level regression of the E blocker -- but they should be
repaired before E2 is dispatched rather than diagnosed by its implementer.