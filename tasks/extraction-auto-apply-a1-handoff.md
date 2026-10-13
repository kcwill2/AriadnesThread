# A-1 Handoff - extraction-auto-apply: extract the boot-progress family out of `commands/ui.rs`

Depends on nothing. This hand de-risks the milestone: the deferred W-2a F2
row forbids any `extraction-auto-apply` hand from spending `ui.rs`'s last
14 lines, so the extraction lands before any ui.rs growth exists.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-auto-apply-shape.md` sections 1 (capability row 14), 4
   (scope out), 5 (module map, A-1 row), 6 (hard edges), 8 (hygiene notes).
   `tasks/extraction-auto-apply-kickoff.md` sections 3 (item 9), 6 (the
   `ui.rs` 986/1000 margin and the extraction list), 9 (frozen-lock table).
3. `tasks/milestone_ledger.md` the `extraction-auto-apply` row plus the
   deferred row "ui-extraction-wizard W-2a verdict F2 - `crates/cli/src/
   commands/ui.rs` line-ceiling watch".
4. Target files: `crates/cli/src/commands/ui.rs` lines 1-28 (the `use`
   stage_spans re-export block that is the precedent), 53-58
   (`read_state`/`write_state`), 90-105 (`ProgressReport`, `ProgressFn`),
   710-860 (the family to move, with the bind helpers that must STAY),
   960-986 (the test-module list); `crates/cli/src/commands/ui/
   boot_progress.rs` (all 115 lines); `crates/cli/src/commands/ui/
   boot_progress_tests.rs` lines 1-14 and 530-540; `crates/cli/src/commands/
   ui/boot_panic_tests.rs` lines 1-14 and 88-112.

## Objective and preserved invariants

Move the boot-progress family verbatim out of `commands/ui.rs` into the
existing sibling `commands/ui/boot_progress.rs`, and add the re-export
surface in `ui.rs` so all seven frozen test modules compile and pass
UNEDITED. No behavior change, no new route, no new output.

The family to move (shape section 1 row 14 / kickoff section 6, verbatim
names): `make_bar_progress`, `make_bar_progress_with_sink`,
`make_boot_progress`, `make_boot_progress_with_sink`, `panic_payload_text`,
`surface_boot_failure`, `drive_boot_build`, `build_boot_generation`
(currently `ui.rs:710-860`).

Invariants this hand must preserve:

- **S6 bind surface is NOT part of the move.** `resolve_bind_addr`
  (`ui.rs:832-839`) and `warn_if_expose_used` (`ui.rs:841-850`) STAY in
  `ui.rs` verbatim. Two frozen source-text locks require them there:
  `tests.rs` cuts the production section at the first `#[cfg(test)]` and
  asserts it contains `TcpListener::bind(format!("{bind_addr}:{port}"))`,
  `warning:` and `--expose`, and NOT `0.0.0.0`. Moving either helper fails
  those locks AND violates invariant A8 of the shape.
- **Every other frozen source-text lock over `../ui.rs` stays green.** The
  needles live in `build_ui_payload` (which stays): `"Hashing sources"`
  (once, ordered before the first `Pipeline::new` - `first_frame_tests.rs`),
  the planning/justifying milestone labels (`progress_contract_tests.rs`),
  and `Payload assembled` (built via `format!` at
  `progress_contract_tests.rs:73` so the test cannot match itself). None of
  the moved family carries any of them.
- **The frozen test-import surface resolves without editing any test file.**
  Enumerated at HEAD `023558c` (verify at preflight; a mismatch is a STOP):
  `boot_progress_tests.rs:1-8` imports `make_boot_progress` from `super::`;
  `boot_panic_tests.rs:7-10` imports `drive_boot_build` and
  `make_boot_progress_with_sink` from `super::`. `first_frame_tests.rs`,
  `builder_tests.rs`, `tests.rs`, `target_files_tests.rs` and
  `progress_contract_tests.rs` import none of the moved names.
- The `stage_spans` precedent at `ui.rs:13-23` is the model: parent-private
  `use` bindings in `ui.rs` are visible to child modules through
  `use super::{...}`, so the frozen import lines stay UNEDITED.
- Pure ASCII authoring; no em dashes; `python tools/encoding_guard.py
  --staged` exit 0 before the commit.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- tasks/extraction-auto-apply-a1-journal.md (checkpoint journal)
- crates/cli/src/commands/ui.rs (family moves out; re-export surface added;
  net -130 to -160 lines)
- crates/cli/src/commands/ui/boot_progress.rs (receives the family plus its
  new test module; net +140 to +190 lines)

You MUST NOT touch: `crates/cli/src/commands/transform.rs`,
`crates/cli/src/commands/ui/extraction_reader.rs`, any other file under
`crates/cli/**`, all seven frozen test modules under
`crates/cli/src/commands/ui/`, `crates/ui/**`, `crates/validate/**`,
`tests/**`, `tools/**`, any asset, `Cargo.toml`, `Cargo.lock`. No
`git stash`, no branch change.

## Frozen locks

- The seven frozen test modules under `crates/cli/src/commands/ui/`
  (`tests.rs`, `progress_contract_tests.rs`, `builder_tests.rs`,
  `boot_progress_tests.rs`, `target_files_tests.rs`, `first_frame_tests.rs`,
  `boot_panic_tests.rs`) stay BYTE-IDENTICAL. Proof: reviewer per-commit
  diff against the start rev, plus their tests green in gate 3.
- `crates/cli/src/commands/ui/boot_progress.rs`'s existing `BootStepSnapshot`
  contract (the I2 terminal-parity oracle in `boot_progress_tests.rs`) is a
  move target only: the type's source moves verbatim, unchanged.
- `tests/golden/**` byte-locked (`UPDATE_GOLDENS=1` FORBIDDEN);
  `tests/fixtures/**` byte-frozen. This hand writes no serialization.
- `tools/theme_token_baseline.json` diff-empty; zero added hex (no asset is
  touched - still run the added scan at the reviewer hop).
- No new public API name: the moved helpers keep their exact names and
  visibility so `crate::commands::ui::*` callers elsewhere in the workspace
  see no change. Caller census at HEAD: zero references to the eight moved
  names outside `ui.rs` and `boot_panic_tests.rs` (verify at preflight; any
  hit is a STOP-and-report).

## What to build

1. `boot_progress.rs` receives the eight functions VERBATIM (doc comments
   included) at the top of the file, above `BootStepSnapshot`, with a module
   doc extension naming the move and its reason (the W-2a F2 ceiling
   discharge; A-4's install lands later in `ui.rs` and needs the headroom).
   Add the `use super::{...}` lines the moved bodies need. Verified at HEAD
   the moved bodies reference: `ProgressFn`, `ProgressReport`,
   `step_percent`, `STAGES_PER_RUN`, `RUN_PHASES`, `write_state`, `UiState`,
   `BootStepSnapshot` (now local), `TerminalStepBars`,
   `crate::commands::progress::{strip_stage_prefix, print_load_complete}`,
   `StageFingerprints`, `UiBuildResult`, `swap_into_state`,
   `build_ui_payload`, `Repository`-free `PathBuf`. Every one of those is
   reachable from a child module through `super::` (child modules see
   parent-private items, including the private `type ProgressFn`).
2. `ui.rs` gains the re-export surface, mirroring the `stage_spans` comment
   idiom exactly (the comment must say the names are exported for the frozen
   test imports plus the production callers here):
   - production block after the `use stage_spans::{...}` import:
     `use boot_progress::{build_boot_generation, drive_boot_build,
     make_bar_progress, make_boot_progress, surface_boot_failure};`
     (these five are the only moved names `ui.rs` production still calls:
     `ui.rs:878`, `:922`, `:930`, `:931`, `:936`)
   - test-only block, PLACED after ALL production code next to the existing
     `#[cfg(test)] use stage_spans::COLLATERAL_FILE_SPAN_END;` precedent:
     `#[cfg(test)] use boot_progress::make_boot_progress_with_sink;`
     (its only remaining callers are `boot_panic_tests.rs:93` and the moved
     family itself)
   Do NOT re-export `make_bar_progress_with_sink` or `panic_payload_text`:
   after the move their only callers are inside `boot_progress.rs`, and an
   unused re-export is an `unused_imports` warning.
3. Nothing else in `ui.rs` changes. The `mod boot_progress;` line already
   exists at `ui.rs:25`.

## RGB protocol (per module, in this file order)

Red: author the new `#[cfg(test)] mod tests` in `boot_progress.rs` FIRST
(NOT in any frozen test file). It reads its own source and `../ui.rs` as
text and fails on the pre-move tree. See "Required tests" for the exact
needles and messages. Run gate 2. A compile failure is NOT the Red: if the
new test fails to compile, the skeleton is wrong - fix it before
proceeding.

Green: perform the move (functions verbatim, doc comments included) and add
the two re-export blocks in `ui.rs`. Re-run gate 2 until exit 0, then gate 3.

Refactor: keep each moved function under 100 lines, one responsibility per
function, no single-letter locals, no banned identifiers. `ui.rs` must land
at roughly 830-860 lines and `boot_progress.rs` at roughly 270-310.

## Required tests (new, in `boot_progress.rs`'s `#[cfg(test)] mod tests`)

All needles are built with `format!` so the test's own source cannot match
itself.

1. `test_boot_progress_family_lives_in_this_module`: for each of the eight
   family names, assert `include_str!("boot_progress.rs")` contains
   `format!("fn {name}")`. Red failure message (pre-move):
   `boot_progress.rs must define fn make_bar_progress` (the first missing
   name reported).
2. `test_boot_progress_family_is_gone_from_ui_rs`: for each of the eight
   names, assert `include_str!("../ui.rs")` does NOT contain
   `format!("fn {name}")`. Red failure message:
   `ui.rs must no longer define fn make_bar_progress; the family belongs in boot_progress.rs`.
3. `test_ui_rs_is_under_the_extraction_ceiling`: assert
   `include_str!("../ui.rs").lines().count() <= 880`. Red failure message:
   `ui.rs must drop under the extraction ceiling, got 986 lines (the W-2a F2
   deferred row)`. 880 is chosen with headroom: the post-move estimate is
   855 and A-4's install is +~14.

Required Red evidence: gate 2 exit 1 with exactly those three failing tests
named in the journal, no others.

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned identifier names, no
single-letter locals, intention-revealing names. `python
tools/hygiene_check.py` exit 0 is the R2 proof of the F2 discharge:
`ui.rs` 986 -> ~855 and `boot_progress.rs` 115 -> ~290 are both far under
1000. `crates/cli/src/commands/transform.rs` is untouched by this hand.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/testrun.py --distill -p ariadne-cli` (baseline canonical
   line recorded verbatim in the journal)
2. `python tools/testrun.py --distill -p ariadne-cli` (Red: exit 1; the
   three named failing tests, verbatim panic lines, in the journal)
3. `python tools/testrun.py --distill -p ariadne-cli` (Green: exit 0;
   canonical line = baseline + the three new tests; suites unchanged)
4. `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`
   (exit 0: F14 standing rule 2, the reverse-dependency build)
5. `python tools/hygiene_check.py` (exit 0)
6. `python tools/encoding_guard.py --staged` (exit 0)
7. `python tools/commit_scoped.py --scope crates/cli/src/commands/ui.rs
   --scope crates/cli/src/commands/ui/boot_progress.rs --scope
   tasks/extraction-auto-apply-a1-journal.md -m "refactor(cli): extract the
   boot-progress family into commands/ui/boot_progress.rs (ui.rs 986 -> ~855)"`
   (never `git add .`)
8. `python tools/encoding_guard.py --all` (post-commit, exit 0)

Gate count: 8 (the commit-cycle trio is gates 6-8). Bench stance: this hand
touches `crates/cli/src` so the DOCUMENTED_PHASES dated row applies - the
producer set under `crates/cli/tests/` is byte-identical across this hand's
range (zero-diff producer proof in the journal), the timing gate is
deterministically blocked at `tools/bench_compare.py:84`, and NO timing
number is attributed to this hand. Do not run `bench_compare.py`; the
reviewer hop owns it.

## Acceptance (machine-checkable)

1. `crates/cli/src/commands/ui.rs` lands between 830 and 880 lines, with
   `resolve_bind_addr` and `warn_if_expose_used` still defined in it
   (`grep`-verifiable).
2. The seven frozen test modules are byte-identical to the start rev
   (`git diff <start-rev> -- crates/cli/src/commands/ui/` shows only
   `boot_progress.rs`).
3. `python tools/testrun.py --distill -p ariadne-cli` Green canonical line
   = baseline + 3 passed, +0 failed, suites unchanged, exit 0.
4. `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`
   exit 0.
5. `python tools/hygiene_check.py` exit 0.
6. `python tools/encoding_guard.py --all` exit 0.
7. The commit's touch-set is exactly the three files in gate 7 and nothing
   else (reviewer commit-vs-touch-set enforcement).

## Discipline clauses

- UTF-8: Write/Edit tools only; never `Set-Content`/`Out-File`/`>`/
  `Get-Content | -replace`; pure-ASCII authored content; no em dashes.
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`) are a reviewer
  blocker: distilling entry points only.
- Checkpoint the journal after each Green gate (module, commit hash,
  deviation, lock finding). The journal is the dead session's decision log.
- Context exhaustion: STOP with a disk-state inventory; successor resumes
  via `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-auto-apply-a1-handoff.md --scope ... --cmd ...`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence (the three named
   failing tests with verbatim panic lines). 3. Green evidence (distill
   digest header line + canonical line + capture path). 4. The measured
   `ui.rs` and `boot_progress.rs` line counts (measured, not predicted).
   5. Deviations + reasons. 6. Out-of-scope observations (report-only). No
   raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the touch-set above exactly.
