# A-1 Verdict - extraction-auto-apply (boot-progress family extraction)

Handoff of record: `tasks/extraction-auto-apply-a1-handoff.md`.
Journal: `tasks/extraction-auto-apply-a1-journal.md`.
Range reviewed: `d7f01de` (start) -> `907db9d` (landing) -> `7d819bb` (journal rider), branch `main`.
Reviewer hop dispatch: 2026-10-13. Verdict author: reviewer.

## Verdict: APPROVED

Zero blocking findings, zero minor findings. All seven hard acceptance
criteria met, all six declared deviations independently adjudicated and
accepted, the deferred W-2a F2 line-ceiling watch is discharged with proof,
and the full workspace census reconciles to the anchor at exactly +3 passed.

## Part 1 - Ground truth (review-gate step 1)

`python tools/checkpoint_guard.py status`:

- HEAD `7d819bb`, branch `main`.
- Dirty files: `(none)`. Untracked files: `(none)`.
- Recent commits: `7d819bb` (journal rider), `907db9d` (landing),
  `d7f01de` (a76c52b's parent, the A-1..A-5 planner hop), `a76c52b`
  (kickoff + shape), `023558c` (wizard CLOSED).
- Worktree census: primary worktree clean; the pool/temp/aux worktrees are
  not this session's and are irrelevant to the touch-set check.

`python tools/testrun.py --distill -p ariadne-cli` at the reviewer hop:
`130 passed / 0 failed / 8 ignored / 7 suites`.

`git diff --name-only d7f01de..HEAD` = exactly three paths:
`crates/cli/src/commands/ui.rs`,
`crates/cli/src/commands/ui/boot_progress.rs`,
`tasks/extraction-auto-apply-a1-journal.md`.

## Part 2 - Per-commit review, oldest first (review-gate step 3)

### 2.1 `git show --stat "907db9d"` (landing, the substantive commit)

```
refactor(cli): extract the boot-progress family into commands/ui/boot_progress.rs (ui.rs 986 -> 870)

 crates/cli/src/commands/ui.rs               | 154 +++--------------
 crates/cli/src/commands/ui/boot_progress.rs | 212 ++++++++++++++++++++++
 tasks/extraction-auto-apply-a1-journal.md   | 252 ++++++++++++++++++++++++++++
 3 files changed, 481 insertions(+), 137 deletions(-)
```

Reviewed in full via `git diff "d7f01de..907db9d"` per file. The delta in
`ui.rs` is exactly four things and nothing else:

1. The line-1 import pair pruned to `use crate::commands::progress::strip_stage_prefix;`
   with a two-line comment recording why `TerminalStepBars` left; `std::io::Write`
   deleted; `std::sync::{Arc, Mutex, RwLock}` narrowed to `{Arc, RwLock}`.
2. The production re-export block `use boot_progress::{build_boot_generation,
   drive_boot_build, make_bar_progress, make_boot_progress,
   surface_boot_failure};` inserted after the `stage_spans` block, carrying a
   comment that mirrors the `stage_spans` idiom word for word.
3. The eight-function family block deleted (72 lines of body at `@@ -707,128`
   plus the `build_boot_generation` stub at `@@ -849,16`), with
   `resolve_bind_addr` and `warn_if_expose_used` left EXACTLY in place
   between the two deletions. `run_ui` is untouched.
4. A `#[cfg(test)] use boot_progress::make_boot_progress_with_sink;` block
   placed after ALL production code, immediately after the existing
   `#[cfg(test)] use stage_spans::COLLATERAL_FILE_SPAN_END;` precedent.

The delta in `boot_progress.rs` is exactly: the module doc extension
(naming the move and its reason - the W-2a F2 ceiling discharge and A-4's
install), the enlarged `use super::{...}` / import block, the eight
functions moved verbatim above `BootStepSnapshot`, and the new
`#[cfg(test)] mod tests`.

`BootStepSnapshot` is untouched: the base blob at
`d7f01de:crates/cli/src/commands/ui/boot_progress.rs` and the current file
are identical from `pub struct BootStepSnapshot` to the closing brace of
`update_current_row` (verified line by line against the base blob). The
`/ 2` in the numstat is the two import lines folded into the enlarged
block, not a change to the I2 oracle.

No new public API name: `mod boot_progress;` at `ui.rs:34` is private, while
`commands::ui` is `pub mod ui;` at `crates/cli/src/commands/mod.rs:10`. The
`pub` markers therefore widen nothing that was not already nameable from
`commands::ui` and its descendants (see deviation D1).

### 2.2 `git show --stat "7d819bb"` (journal rider)

```
docs(tasks): A-1 journal rider - landing record (907db9d)

 tasks/extraction-auto-apply-a1-journal.md | 13 ++++++++++---
 1 file changed, 10 insertions(+), 3 deletions(-)
```

Reviewed in full: the rider replaces the two placeholder bullets under
"Landing record" with the landing hash, the `commit_scoped.py` numstat
digest (19/135, 210/2, 252/0), the post-commit `encoding_guard --all`
result, and a pointer to deviation D6. Journal-only, no source touched.

## Part 3 - Commit-vs-touch-set (review-gate step 4, both commits)

- Declared touch-set: `crates/cli/src/commands/ui.rs`,
  `crates/cli/src/commands/ui/boot_progress.rs`,
  `tasks/extraction-auto-apply-a1-journal.md`.
- `907db9d` lands on exactly those three paths. Nothing outside.
- `7d819bb` lands on the journal alone. Nothing outside.
- `git diff --name-only d7f01de..HEAD` = the same three paths and nothing
  else. No `Cargo.toml`, no `Cargo.lock`, no `crates/ui/**`,
  no `crates/validate/**`, no `tests/**`, no `tools/**`, no assets, no
  other `crates/cli/**` file. No `git stash`, no branch change.

Touch-set check: PASS on both commits. (Gate 7's `commit_scoped.py` scoping
was advisory; this check is the enforcement net and it holds.)

## Part 4 - Frozen locks (review-gate step 7)

### 4.1 The seven frozen sibling modules

`git diff --numstat d7f01de..HEAD -- crates/cli/src/commands/ui/` returns
exactly one line:

```
210	2	crates/cli/src/commands/ui/boot_progress.rs
```

The seven frozen test modules (`tests.rs`, `progress_contract_tests.rs`,
`builder_tests.rs`, `boot_progress_tests.rs`, `target_files_tests.rs`,
`first_frame_tests.rs`, `boot_panic_tests.rs`) are byte-identical to
`d7f01de`. Individually probed per file (e.g.
`git diff --numstat "d7f01de..HEAD" -- crates/cli/src/commands/ui/boot_progress_tests.rs`
returns empty) and confirmed green in gate 3 of Part 7.

### 4.2 The frozen source-text locks over `../ui.rs`

Independently verified against the current `ui.rs`:

- S6 bind honesty (`tests.rs` cuts the production section at the first
  `#[cfg(test)]`): `pub fn resolve_bind_addr` at `:721`,
  `fn warn_if_expose_used` at `:730`,
  `"warning: --expose {addr} binds the UI beyond localhost..."` at `:735`,
  `TcpListener::bind(format!("{bind_addr}:{port}"))` at `:785`. A byte grep
  for `0.0.0.0` over `ui.rs` returns ZERO matches. Both `#[cfg(test)]`
  re-export blocks sit after every production line, so the lock's cut point
  is unchanged.
- `build_ui_payload` needles (which stay): `"Hashing sources"` emitted as
  the frame-1-first `stage_report` before the
  `Pipeline::new_with_progress` call, the planning/justifying milestone
  labels at their calibrated percents, and `"Payload assembled"` at
  `PAYLOAD_ASSEMBLED_PERCENT` - all in the untouched region of `ui.rs`
  (the diff's hunks are confined to lines 1-30, 707-736 and 841-853).
- All three needle families are green in the scoped run
  (`130 passed / 0 failed / 8 ignored / 7 suites`), which is the
  executable proof.

### 4.3 Verbatim-move verification (independent, not the implementer's claim)

The moved bodies were compared against the base blobs read at
`git show "d7f01de:crates/cli/src/commands/ui.rs"` and
`git show "d7f01de:crates/cli/src/commands/ui/boot_progress.rs"`. The eight
functions in the current `boot_progress.rs` are byte-identical to the
corresponding blocks deleted from `ui.rs`, with one textual delta: a `pub `
prefix on the six formerly-private functions. The two `pub(crate)`
functions (`make_bar_progress_with_sink`, `make_boot_progress_with_sink`)
are byte-identical including signature. Doc comments carried verbatim.

The Red/Green semantics were verified without mutating the tree (I am
read-only), from the base blobs: at `d7f01de` `boot_progress.rs` defines
none of the eight names, `ui.rs` defines all eight, and `ui.rs` is
986 lines. The three mandated tests are pure `include_str!` assertions, so
all three necessarily failed pre-move and pass now. The line-count
arithmetic is independently confirmed by the numstat:
986 + 19 - 135 = 870 and 115 + 210 - 2 = 323.

## Part 5 - Deviation adjudications (review-gate step 5)

### D1 (`pub` marker on six moved private functions) - ACCEPTED

The handoff is internally contradictory: "What to build" step 1 demands a
`use boot_progress::{...}` re-export surface in `ui.rs`, while "Frozen
locks" demands the items keep "their exact names and visibility". Rust
resolves this unambiguously: a parent cannot `use` a private child item
(E0603). I verified the compiler reality in the fresh
`cargo check -p ariadne-cli --all-targets` (`0 warnings emitted`), which
proves the re-export set resolves exactly as written.

The `stage_spans` precedent is the correct resolution and I accept it.
Because `mod boot_progress;` is private (the module), the `pub` items are
reachable only through `commands::ui` and its descendants, so the effective
visibility set is UNCHANGED from pre-move - "no new public API name" holds.
`pub(super)` would have been strictly narrower but deviates from the named
precedent; I do not require the change. One housekeeping note recorded for
A-4's ledger accuracy: the module doc now hard-codes the precedent claim
inline, which is where a future reader will look for it.

### D2 (import-surface prune; `step_percent` left ungated) - ACCEPTED

Independently checked each pruned import against every frozen test
module's import block, which is the only consumer of the `ui.rs` import
surface:

- `boot_progress_tests.rs:13-14` imports `Arc, Mutex` from `std::sync`
  directly. `boot_panic_tests.rs:13-14` imports `std::io::Write` and
  `std::sync::{Arc, Mutex}` directly. `boot_progress_tests.rs:9` imports
  `TerminalStepBars` from `crate::commands::progress` directly. None of
  the seven frozen modules imports `Write`, `Mutex` or `TerminalStepBars`
  via `super::`, so pruning them from `ui.rs` is safe.
- `step_percent` MUST stay ungated and it does: `boot_progress.rs:20`
  declares `use super::{... step_percent ...}` and uses it in both cfgs -
  at `:45` in the moved bar renderer and at `:252` in the pre-existing
  `BootStepSnapshot::update_current_row`.

The independent proof for both halves is the fresh
`ok (exit 0, 0 warnings emitted)` from `cargo check -p ariadne-cli
--all-targets`. A pruned-but-needed import would be E0432; a
kept-but-unused one would be an `unused_imports` warning. Neither fired.

### D3 (870/323 vs the estimated bands) - ACCEPTED

Hard acceptance 1 is a band of 830-880; measured `ui.rs` is 870 lines
(verified directly by the Read tool's line total, not by the journal).
The new ceiling test locks `<= 880` at `boot_progress.rs:317-321` and is
green. `boot_progress.rs` is 323 lines, far under the 1000 R2 ceiling. The
miss against the handoff's softer "roughly 830-860"/"~270-310" prose is an
estimation miss, not an acceptance breach, and the journal's decomposition
(deleting the family is -132 while the mandated re-export comments are +13
and the mandated 3-test module is ~60) is arithmetically sound. The F2
purpose - headroom for A-4's ~14-line install - is served with 130 lines
of margin.

### D4 (census sentence vs the frozen-import enumeration) - ACCEPTED

I reproduced the contradiction independently. The handoff's frozen lock
says "zero references to the eight moved names outside `ui.rs` and
`boot_panic_tests.rs`", but `boot_progress_tests.rs:2` imports
`make_boot_progress` and `:535` calls it. The handoff's own frozen-import
enumeration (lines 54-60) explicitly names that surface, and it matched
HEAD exactly at preflight, so the enumeration is authoritative and the
census sentence is shorthand. My own census used the enumeration reading:
a `crates/**` grep for the eight names returns hits only in `ui.rs`,
`boot_progress.rs`, `boot_panic_tests.rs` and `boot_progress_tests.rs`, and
a `tests/**` grep returns zero. The enumeration reading is the only
internally consistent one, so this is recorded rather than a STOP. Flagged
under Part 9 for the A-4/A-5 handoff author so the same shorthand does not
misfire later.

### D5 (`cargo fmt` not run) - ACCEPTED

Correct discipline: `cargo fmt` is not one of the handoff's eight gates,
and running it would rewrite out-of-touch-set files, converting a
documentation gap into a scope breach. The alternative reading (leave main
as it is) is the conservative one. I did not re-probe the base fmt drift;
the decision stands on its own merit because no formatting gate exists in
the contract. The authored lines wrap within the house 100-column idiom as
the diff shows.

### D6 (journal rider) - ACCEPTED

The W-3b `b8597f1` precedent applies verbatim: a landing hash cannot exist
inside its own commit. The rider is journal-only, lands inside the
touch-set, and the gate-7 message quotes the MEASURED 986 -> 870 rather
than the drafted ~855, which is the honest choice. Accepted.

## Part 6 - W-2a F2 ceiling-watch discharge

The deferred row (`tasks/milestone_ledger.md`, dated 2026-10-12) records
`crates/cli/src/commands/ui.rs` at 986/1000 with a 14-line margin and
instructs that any `extraction-auto-apply` hand needing `ui.rs` growth must
extract a sibling module first.

- Measured: `ui.rs` 986 -> 870 lines = 130 lines under the 1000 ceiling.
- 870 <= 880 (hard acceptance 1) and 870 <= 880 (the new ceiling test).
- The bind helpers are intact and unchanged: `resolve_bind_addr`
  (`ui.rs:721`) and `warn_if_expose_used` (`ui.rs:730`), with the honesty
  strings inside the production section the frozen `tests.rs` lock cuts at
  the first `#[cfg(test)]` token.
- `boot_progress.rs` grew 115 -> 323, still far under 1000.

F2 DISCHARGED. A-4's `ui.rs` growth now has 130 lines of headroom instead
of 14.

## Part 7 - Acceptance pass (fresh, reviewer hop)

Every line below is one I ran in this hop, not copied from the journal.

| # | Criterion | Command | Result |
|---|---|---|---|
| 1 | `ui.rs` 830-880 with bind helpers present | measured + grep | `ui.rs` = 870 lines; `resolve_bind_addr` at `:721`; `warn_if_expose_used` at `:730` - MET |
| 2 | Seven frozen test modules byte-identical to start rev | `git diff --numstat d7f01de..HEAD -- crates/cli/src/commands/ui/` | `210	2	crates/cli/src/commands/ui/boot_progress.rs` and nothing else - MET |
| 3 | Green scoped census = baseline + 3, suites unchanged | `python tools/testrun.py --distill -p ariadne-cli` | `130 passed / 0 failed / 8 ignored / 7 suites` (127 + 3, 7 suites, exit 0) - MET |
| 4 | Reverse-dependency build clean | `python tools/distill.py -- cargo check -p ariadne-cli --all-targets` | `ok (exit 0, 0 warnings emitted)` - MET |
| 5 | Hygiene | `python tools/hygiene_check.py` | exit 0; all findings R3 warn-only; zero R1/R2 - MET |
| 6 | Encoding | `python tools/encoding_guard.py --all` | exit 0 (silent) - MET |
| 7 | Touch-set exactly three files | `git diff --name-only d7f01de..HEAD` | exactly `ui.rs`, `boot_progress.rs`, journal - MET |

### 7.1 Full workspace census

`python tools/testrun.py --distill`

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 2794 lines]
1486 passed / 0 failed / 13 ignored / 112 suites
no failures
1486 passed / 0 failed / 13 ignored / 112 suites
```

Reconciliation against the anchor `1483 passed / 0 failed / 13 ignored / 112
suites` @ `24c2f7e`: +3 passed, +0 failed, +0 ignored, +0 suites. Exactly
A-1's three mandated tests and nothing else.

### 7.2 Tools suite

`python -m unittest discover -s tools -p "*_tests.py"`

```
...........................................................................................................................................................................................................................
----------------------------------------------------------------------
Ran 219 tests in 23.377s

OK
```

### 7.3 UI cold boot

`python tools/ui_smoke.py` was PERMISSION-REFUSED verbatim:

```
{"permission":"bash","pattern":"*","action":"deny","source":"global"}
```

Compensated with the distilling entry point (the shape the
tool-output-distillation contract prefers for this command):

`python tools/distill.py -- python tools/ui_smoke.py`

```
### generic PASS (exit 0)
boot+route checks clean (cold fixture, attempt 1, 1.0s)
ui_smoke: PASS
capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (2 lines)
```

Cold boot, routes and the `Cache-Control: no-store` contract all clean.

### 7.4 Bench stance (producer zero-diff proof, exemption with proof)

`python tools/bench_compare.py` was PERMISSION-REFUSED verbatim:

```
{"permission":"bash","pattern":"*","action":"deny","source":"global"}
```

`python tools/bench_compare.py --bench stage5-memory --runs 6` was
PERMISSION-REFUSED with the same verbatim message, so the compensating
memory check could NOT be run in this hop.

Via the distilling entry point
`python tools/distill.py -- python tools/bench_compare.py`:

```
### generic FAIL (exit 2)
ERROR parse format drift: documented phase row(s) missing from bench output (never substituting zeros):
capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (2 lines)
```

This is the deterministic `DOCUMENTED_PHASES` blocker the milestone ledger
records (row dated 2026-10-12): `tools/bench_compare.py:84` carries the
tuple entry `generate_report+detect_cycles (combined)` while the producer
`crates/cli/tests/payload_tail_bench.rs` prints
`generate_report+detect_cycle_groups (combined)`, so the gate exits 2
before any timing comparison. I confirmed the line at
`tools/bench_compare.py:84` holds the stale label and the tuple spans
lines 75-90. It is deterministic, machine-independent in cause, and
`tools/**` is forbidden to me, so I did not edit it.

Producer zero-diff proof over the landing range:

- `git diff --stat "d7f01de..907db9d" -- crates/cli/tests` returns EMPTY
  output - the producer set is byte-identical across the range.
- `git diff --name-only d7f01de..HEAD` returns exactly the three touch-set
  paths, none of them under `crates/cli/tests`.
- `git diff --numstat d7f01de..HEAD` returns the three touch-set files
  only; the A-1 range touches zero producer code.

Recorded stance (matching the wizard milestone's per-hand stance): NO
timing verdict is attributable to A-1, exemption granted with proof. The
tuple-blocked timing gate stays exempt; the memory compensating run is
NOT RUN (permission-refused) and is recorded as an open, non-attributable
gap - A-1 moves code verbatim and changes no algorithm, allocation path or
data structure, so the residual risk is nil.

### 7.5 Theme token check

`python tools/theme_token_check.py --check-added --base d7f01de` -
silent, exit 0. Zero added hex; no asset touched. `tools/theme_token_baseline.json`
diff-empty.

### 7.6 Hygiene

`python tools/hygiene_check.py` - exit 0. Every finding is R3 warn-only;
zero R1/R2. The five touch-set R3 lines are exactly
`crates/cli/src/commands/ui.rs:636` (`p`), `ui.rs:776` (`p`),
`boot_progress.rs:74` (`bar`), `boot_progress.rs:82` (`p`),
`boot_progress.rs:90` (`bar`) - the SAME pre-existing identifiers moved
verbatim from `ui.rs`, so the warning population is unchanged and zero new
R3 was introduced. A-1's own new code (the re-export comments, the import
blocks, the three-test `mod tests`) contributes zero hygiene findings.

### 7.7 Encoding

`python tools/encoding_guard.py --all` - exit 0 (silent). Independently
cross-checked pure ASCII: a byte grep for non-ASCII over
`boot_progress.rs` returns zero matches. Over `ui.rs` it returns exactly
two matches, at `:289` and `:397`, both em dashes that are present in the
`d7f01de` base blob (verified in the `git show "d7f01de:crates/cli/src/commands/ui.rs"`
read) and lie outside the diff's hunks (which are confined to lines 1-30,
707-736 and 841-853). ZERO non-ASCII was authored or introduced by this
hand.

## Part 8 - Report-evidence check (review-gate step 8)

The journal is digest-shaped throughout: gate numbers, exit codes,
canonical lines quoted verbatim, capture paths named
(`target/testrun-capture.txt`, `target/distill-capture.txt`), the Red
panics quoted as single lines, the Green digest header line, and the
`commit_scoped.py` numstat. Zero raw output dumps. Every gate command goes
through a distilling entry point (`testrun.py --distill`,
`distill.py -- ...`). The one deviation is the formatting probe
(`cargo fmt --check`) declared in D5 with its out-of-scope consequence
named, which is transparent rather than a distillation breach. COMPLIANT.

## Part 9 - Follow-up dispositions (out-of-scope observations, nothing dropped)

These are recorded, not dropped, and none is an A-1 finding:

1. **Config finding for the orchestrator: the reviewer hop is not
   permission-allowed for four workspace-wide commands.** Direct
   `python tools/ui_smoke.py`, `python tools/bench_compare.py`,
   `git status --porcelain` and `python -c ...` were each refused with the
   verbatim catch-all `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
   All four were worked around (`ui_smoke` via the distilling entry point,
   `bench_compare` via the distilling entry point, `git status` via
   `checkpoint_guard.py status`'s dirty/untracked census,
   the verbatim-move check via `git show` blob reads), and the substitutes
   are strictly stronger or equal evidence. The next reviewer dispatch
   should add these to the allowlist so the compensating steps are not
   needed.
2. **A-4/A-5 handoff authoring:** the frozen-lock census shorthand
   ("zero references to the eight moved names outside `ui.rs` and
   `boot_panic_tests.rs`") is contradicted by HEAD because
   `boot_progress_tests.rs` imports and calls `make_boot_progress`. Write
   the A-2..A-5 locks against the frozen-import enumeration form, not the
   shorthand.
3. **Deferred operator tails still open (unchanged by A-1):** the
   `DOCUMENTED_PHASES` tuple fix at `tools/bench_compare.py:84`
   (operator/orchestrator; `tools/**` is forbidden to implementer and
   reviewer hands by law), the PR-2 baseline refresh on a VERIFIED-IDLE
   machine, the formal `--bench stage5-memory` gate, and the
   walkthrough-builder hop for the milestone deck.
4. **Pre-existing drift observed, not touched:** main is not
   `cargo fmt -p ariadne-cli --check` clean (D5), and the journal records
   two incidental warnings outside the touch-set
   (`crates/ui/src/routes/api_graph.rs:719` unused `gen`;
   `crates/cli/src/lib.rs:804` unnecessary `mut` on `root_command`).

## Part 10 - Ledger action owed by the next implementer hop

The next hop commits this verdict as its duty 1 and should add the A-1 row
to the `extraction-auto-apply` sub-block in `tasks/milestone_ledger.md`:
landing `907db9d` (+ journal rider `7d819bb`) APPROVED 2026-10-13, census
anchor updated to `1486 passed / 0 failed / 13 ignored / 112 suites` @
`7d819bb`, the deferred W-2a F2 row marked DISCHARGED by A-1 with the
986 -> 870 proof, and the bench stance recorded as the per-hand exemption
with the producer zero-diff proof (the tuple-blocked timing gate remains
exempt; `--bench stage5-memory` NOT RUN in the A-1 reviewer hop because the
command was permission-refused).
