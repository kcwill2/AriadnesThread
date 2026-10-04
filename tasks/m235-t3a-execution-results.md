# M23.5 T3-A Execution Results - reviewer acceptance addendum

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Range:** `b950963..ea82422`, HEAD `ea82422`, branch `tarjan-s-scc`
**Verdict:** `tasks/m235-t3a-verdict.md` - APPROVED, 0 blocking / 11 non-blocking

Written to `tasks/m235-t3a-execution-results.md`, NOT to
`tasks/m235-execution-results.md`, following the T2-A/T2-B precedent: the
latter path holds the Phase 1 acceptance record and is already committed.

This addendum is the "what actually happened against each acceptance criterion,
with the evidence line for each, and an explicit statement of anything that was
not run" record. Sections 1-4 quote canonical lines verbatim.

---

## 1. Canonical workspace line, verbatim

```
1174 passed / 0 failed / 4 ignored / 88 suites
```

`python tools/testrun.py --distill --timeout 3600`, exit 0, capture
`target/testrun-capture.txt` (2062 lines).

| | passed | failed | ignored | suites |
|---|---|---|---|---|
| T2-B baseline (`b950963`) | 1169 | 0 | 4 | 88 |
| **T3-A measured (`ea82422`)** | **1174** | **0** | **4** | **88** |
| Handoff sec7 prediction | 1174 | 0 | 4 | 88 |
| **Delta** | **+5** | **+0** | **+0** | **+0** |

Measured delta equals predicted delta with **zero unattributed residual** (the
F11 discipline). `+5` is this unit's five new `#[test]` functions, attributed
path by path from the tree rather than merely consistent in total:
`dto_builders.rs` +3 (attrs at 457, 490, 562), `graph_api.rs` +1 (attr at 741),
`export_order.rs` +1 (attr at 151), `d27_node_provenance.rs` +0 (extended in
place). `+0 suites` is correct because `ui_test` is an existing target.

## 2. Handoff acceptance criteria, one by one

### Sec 5 - exactly five `#[test]` functions, 179 -> 184

MET, and independently recounted rather than trusted. 184 `#[test]` /
`#[tokio::test]` attributes under `crates/ui/tests/ui_test/` at HEAD, zero
`#[ignore]` anywhere under `crates/ui`. Gate 1's capture carries
`test result: ok. 184 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
All five required names are named as PASSING in the capture. The D27 test was
EXTENDED, not replaced - `d27_node_provenance.rs` still has 10 test attributes.

### Sec 5 - D27 OMITS-key lock extended

MET. The `language` / `unit_kind` omission code at `dto.rs:711-716` is diff
CONTEXT in `31513b9`, i.e. zero bytes changed. The pre-existing test keeps both
original assertions and gains five, including that a node which IS in a cycle
group still omits both keys and does carry `cycle_group_size == 2`, a
`cycle_witness` array, and `classes == Some("cycle")`. No sixth `#[test]`.

### Sec 3.1 / 3.2 / 3.3 - API shape, edge rule, node keys

MET. `build_include_elements_from_groups` exists with the exact pinned
signature. Exactly ONE node loop (`dto.rs:685`) and ONE edge loop
(`dto.rs:729`), both inside the single shared emitter `emit_include_elements`.
`cycle_group_size` / `cycle_witness` are assigned only when the
`Option`-returning accessors yield `Some`, so an ungrouped node carries neither
key - not `null`, not `0`.

### Sec 4 - two genuine Reds

MET, adjudicated by causal proof rather than re-execution (the Red revision was
correctly never committed and `crates/**` is read-only for this role). Both
messages match the committed assertion text verbatim; both `left` values are
derived exactly from the OLD source recovered from the diff's `-` lines; and I
hand-executed `detect_cycles` on R1's fixture, which returns literally
`[[1, 2, 3]]` - so the hand-fed `vec![vec![a,b,c]]` is byte-identical to real
producer output, and `(1, 3)` is genuinely absent from the old `cycle_edges`
set while both endpoints carry `classes: Some("cycle")` inside a nontrivial SCC.
The re-point to the real producer via `detect_cycle_groups()` is present and
logged, and no assertion was weakened.

### Sec 6 - line-count discipline and the extraction branch

MET. `dto.rs` is **901 lines** at HEAD, under the 1000-line R2 ceiling, with
`hygiene_check.py` reporting zero R1/R2 workspace-wide. The sec6 branch was
taken and honoured in full: `crates/ui/src/cycle_groups_view.rs` is a NEW
149-line module holding classification only, and `crates/ui/src/lib.rs` carries
exactly ONE added line (`pub(crate) mod cycle_groups_view;`) with nothing else,
as the clause's "one line, no other file changes" requires. The split is a real
module boundary (how to classify vs how to emit) with **no logic duplicated**
across the two files. `emit_include_elements` is 87 lines including signature
and doc, under the 100-line rule.

### Sec 8 - frozen locks

ALL HOLD. `crates/ui/tests/ui_test/main.rs` byte-unchanged; path-based
`build_include_elements` signature intact (diff CONTEXT) and `crates/cli`
compiles clean; no `crates/report` path in any commit; zero new hex literals;
UTF-8 clean; no `Cargo.toml` / `Cargo.lock` change; no `tests/fixtures/**` or
`tests/golden/**` path; `detect_cycles` / `dfs_cycle` still at `graph.rs:234`
and `:260`; exactly two cycle representations exist and the new module holds a
CLASSIFICATION view, not a third detector.

### Sec 0 step 0.2 / 0.3 - the CRLF fixture

MET, and the after-state is confirmed by me as well as by the implementer.
`tests/fixtures/basic/artifacts/includes/report.json` appears in **NO** commit of
`b950963..ea82422`, and is **absent from the numstat dirty section both before
and after my own full workspace run**. The porcelain view still flags its stat
entry and the census reads `dirty=2` for this worktree: `1` = the F12
stat-cache entry, `1` = this review's own untracked verdict file. Nothing else.

## 3. Reviewer-run gates, all results

| Gate | Command | Result |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test` | `184 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui` | `184 passed / 0 failed / 0 ignored / 3 suites` |
| 3 | `python tools/distill.py -- cargo build -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` |
| extra | `python tools/distill.py -- cargo build -p ariadne-cli` | `ok (exit 0, 0 warnings emitted)` |
| 4 | `python tools/hygiene_check.py` | zero R1/R2; every printed line a legacy-file R3 warn |
| 5 | `python tools/theme_token_check.py --check-added --base b950963` | no output, exit 0 - zero added hex literals |
| 6a | `python tools/encoding_guard.py --all` | no output, exit 0 |
| 6b | `python tools/encoding_guard.py --staged` | no output, exit 0 |
| 7 | `python tools/checkpoint_guard.py status` | run twice (pre and post my full run): DIRTY FILES (none) both times |
| workspace | `python tools/testrun.py --distill --timeout 3600` | `1174 passed / 0 failed / 4 ignored / 88 suites` |
| tool suites | `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 184 tests in 18.495s` then `OK` |

Hygiene detail: the implementer's claim that every R3 warning in a file it
touched sits on a pre-existing line is CONFIRMED by line number -
`dto.rs:330/418/445` are far below the authored region `598-748`, and all 14
`graph_api.rs` `res` warnings sit at lines 52-231 while the authored test starts
at ~697. `dto_builders.rs`, `export_order.rs`, `d27_node_provenance.rs` and
`cycle_groups_view.rs` carry zero flags, and the scanner demonstrably does scan
sibling test files, so the zero is measured rather than a skip.

Gate 5 detail: no file under `crates/ui/assets/` and no CSS, JS or template is
anywhere in the range, so light / dark / dark-steampunk semantic verdict colors
(green / amber / red) and the cool-to-hot heat ramp are structurally untouched.
Only red-edge COVERAGE grows, which is the intent.

## 4. Explicitly NOT run

- `python tools/testrun.py --baseline` -> `usage error: missing value for
  --baseline` (exit 2). Ledger M23.5 F9, attempted ONCE, recorded once, NOT
  treated as a gate failure, NOT worked around. The full-workspace invocation
  it wraps WAS run and compared by hand (section 1).
- `python tools/ui_smoke.py` -> REFUSED. Verbatim: `The user has specified a
  rule which prevents you from using this specific tool call. Here are some of
  the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}`
- `python tools/bench_compare.py` -> REFUSED. Verbatim: identical text.
  Both attempted INDEPENDENTLY, once each; neither retried, neither routed
  through `distill.py`. Both are ledger M23.5 F10.
  **UI-smoke evidence is MISSING, and this is the milestone's first unit that
  materially changes UI DTO output**, so the gap is materially larger here than
  for the graph-only (T2-A) and test-only (T2-B) units: there is now no runtime
  evidence that the UI cold-boots, routes and serves the new payload, and the
  `Cache-Control: no-store` route contract is unverified for it.
- The two Red re-executions. `crates/**` is read-only for this role, so the
  Red revision cannot be reconstructed; both Reds were adjudicated by
  exact-message matching, exact `left`-value derivation from the OLD source, and
  a hand-execution of `detect_cycles` on the fixture instead.
- Bash-issued git. Refused throughout; the dedicated git tools were used
  instead. No `git show` or `git cat-file` was piped through PowerShell into a
  file; all blob content was read from the checkout with the Read tool.
- One self-inflicted refusal, recorded for honesty: my first
  `unittest discover` appended `2>&1 | Select-Object -Last 6` and that PIPELINE
  was refused with the same catch-all text. Recognising the refusal came from my
  own shell pipeline rather than from the command, I re-ran it in plain form and
  it succeeded. No gate was dropped: all ran, six first-attempt and the seventh
  after removing my own pipeline.

## 5. The cross-process export-stability item - disposition

`include_elements_export_is_stable_across_two_calls_with_cycle_fixtures` locks
the IN-PROCESS form only, and the contract (plan:128 Red, plan:147 acceptance 5,
IV-5 at plan:29) requires byte stability across TWO PROCESSES with cycle
fixtures. In-process stability does NOT satisfy that: an export that varies run
to run - map iteration order, hash seeds, address-derived ids - would pass the
in-process test and fail the contract.

**Classification: genuinely NON-BLOCKING for T3-A**, and the implementer's
statement of the limitation was correct behaviour, not a shortfall. Grounds and
full reasoning are in verdict section 8. In short: (a) handoff sec5 allocated
the cross-process half out of this unit and told the implementer to say so
rather than claim it; (b) it is UNREACHABLE at this HEAD, because
`crates/cli/src/commands/ui.rs:310` and `:316` still call `detect_cycles()` and
the legacy `build_include_elements`, so `--export` bytes carry no cycle-group
data and a two-process diff could not fail on anything this unit changed;
(c) the requirement currently has no owner and no executable command, since the
handoff delegated it to a "reviewer acceptance pass" that contains no such gate
(`--baseline` broken, `ui_smoke`/`bench_compare` refused); and (d) the residual
risk was measured structurally and is low - no hash iteration reaches payload
bytes in the new code, node order comes from a total-order sort on
`(path, file_id)`, edge order from a `Vec`, witness order from the
`representative_cycle` `Vec`, `serde_json::Value` objects serialise as
`BTreeMap` (alphabetical, proven by the Red's printed data map), there is no
clock, RNG or address-derived value anywhere in `crates/ui/src`, and the
payload's two `HashMap` fields already carry `serialize_sorted_nested_maps`
with an explicit cross-process doc comment at `dto.rs:202-206`.

**Recommended disposition: a milestone ledger deferred row with a named owner -
unit B**, the commit that repoints the CLI and therefore the first point at
which the group currency is present in `--export` bytes. Mandatory gate there:
two `--export` invocations over a cycle-bearing fixture into two files, then a
byte comparison. Must close before T4.1. **No work is demanded of this
implementer**: it would require a CLI invocation and/or a golden under
`tests/golden/**`, both byte-locked out of this unit's touch-set.

## 6. Worktree state left behind

HEAD `ea82422`, branch `tarjan-s-scc`, DIRTY FILES (none). Census unchanged at
8 worktrees. This review added two untracked files under `tasks/` by design -
this addendum and `tasks/m235-t3a-verdict.md`. They are mine to land, not the
implementer's. Nothing else in the tree was modified and no git write was
performed.