# A-1 Journal - extraction-auto-apply (boot-progress family extraction)

Implementer decision log for `tasks/extraction-auto-apply-a1-handoff.md`.
Session: implementer hop, re-dispatch (the prior session was cancelled
externally before making any change; the start tree was verified clean).
Date 2026-10-13. Track base (start rev): `d7f01de` on `main`.

## Boot and preflight (all PASS)

- `python tools/checkpoint_guard.py status`: HEAD `d7f01de`, branch `main`,
  dirty 0, untracked 0. Worktree census shows the pool/temp worktrees are
  not this session's; the primary worktree is the landing surface.
- `git status --porcelain` via the distill wrapper: empty. Tree clean under
  `crates/**` (and everywhere else).
- Pre-hand state check: `crates/cli/src/commands/ui.rs` measured 986 lines;
  the boot-progress family present (`read_state` :53, `stage_report` :99,
  `stage_progress_callback` :192, family block :710-860 with the bind
  helpers `resolve_bind_addr` :834 and `warn_if_expose_used` :843).
- Frozen-import surface verified at HEAD and matches the handoff
  enumeration exactly: `boot_progress_tests.rs:1-8` imports
  `make_boot_progress` via `use super::{...}`; `boot_panic_tests.rs:7-10`
  imports `drive_boot_build` + `make_boot_progress_with_sink`;
  `first_frame_tests`, `builder_tests`, `tests.rs`, `target_files_tests`
  and `progress_contract_tests` import none of the moved names.
- Caller census over `crates/**` and `tests/**` for the eight moved names:
  hits only in `ui.rs` (definitions plus the five production call sites at
  :878, :922, :930, :931, :936), `boot_panic_tests.rs` (:48, :72, :93,
  :107) and `boot_progress_tests.rs` (import line 2, call :535). Zero
  code references anywhere else; `tests/**` zero. The remaining grep hits
  repo-wide are docs/walkthroughs only. See deviation D4 on the census
  wording. No STOP condition.
- Alignment reads completed: shape sections 1 (row 14), 4, 5 (A-1 row), 6,
  8; kickoff sections 3 (item 9), 6, 9; ledger `extraction-auto-apply` row
  plus the dated W-2a F2 line-ceiling row.

## Gate ledger (canonical/distill digest lines, verbatim)

Capture files are reused across runs (`target/testrun-capture.txt`,
`target/distill-capture.txt`); the lines below are quoted from each gate's
own run before any later gate overwrote the capture (W-1 D6 precedent).

1. Baseline `python tools/testrun.py --distill -p ariadne-cli` (pre-change):
   exit 0, `127 passed / 0 failed / 8 ignored / 7 suites`, capture
   `target/testrun-capture.txt` (202 lines).
2. Red run (same command): exit 1,
   `121 passed / 3 failed / 0 ignored / 1 suites`
   (cargo fail-fast ran only the lib suite; the other six suites resume in
   the Green run). Exactly the three required tests failed, no others:
   - `test_boot_progress_family_lives_in_this_module` panicked:
     `boot_progress.rs must define fn make_bar_progress`
   - `test_boot_progress_family_is_gone_from_ui_rs` panicked:
     `ui.rs must no longer define fn make_bar_progress; the family belongs in boot_progress.rs`
   - `test_ui_rs_is_under_the_extraction_ceiling` panicked:
     `ui.rs must drop under the extraction ceiling, got 986 lines (the W-2a F2 deferred row)`
   Red compiles cleanly (the tests are source-text reads with no
   dependency on the move), so the failure is semantic, not a harness
   error.
3. Green run (same command): exit 0,
   `130 passed / 0 failed / 8 ignored / 7 suites`
   = baseline + exactly the three new tests, suites unchanged,
   capture `target/testrun-capture.txt` (204 lines).
   One interim Green attempt failed to compile with E0432 before this
   pass; see deviation D2 for the cause and fix.
4. `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`:
   exit 0, `0 warnings emitted` (F14 reverse-dependency build).
5. `python tools/hygiene_check.py`: exit 0 (exit code re-proved through
   `python tools/distill.py -- python tools/hygiene_check.py` = `generic
   PASS (exit 0)`). All findings are R3 warn-only. The five touch-set R3
   lines (`ui.rs:636` `p`, `ui.rs:776` `p`, `boot_progress.rs:74` `bar`,
   `boot_progress.rs:82` `p`, `boot_progress.rs:90` `bar`) are the SAME
   pre-existing identifiers moved verbatim: equal warning population
   before and after, zero new R3, zero R1/R2.
6. `python tools/encoding_guard.py --staged`: exit 0. See the staging
   note: `commit_scoped.py` stages atomically inside gate 7, so unless a
   prior `git add` exists the index scan is vacuous at this point; the
   content-level proof is gate 8 (`--all`), which strictly dominates it.
7. `python tools/commit_scoped.py --scope crates/cli/src/commands/ui.rs
   --scope crates/cli/src/commands/ui/boot_progress.rs --scope
   tasks/extraction-auto-apply-a1-journal.md -m "refactor(cli): extract
   the boot-progress family into commands/ui/boot_progress.rs
   (ui.rs 986 -> 870)"` - landing hash in the Landing section below.
8. `python tools/encoding_guard.py --all` (post-commit): exit 0.

Supporting commands beyond the eight named gates (logged for transparency,
all via distilling entry points): `git status --porcelain`,
`git diff --numstat/--stat` scope proofs (below), the producer zero-diff
proof, and one baseline probe `cargo fmt -p ariadne-cli --check` which
FAILS on untouched files at `d7f01de` (see D5 and observations).

Bench stance (DOCUMENTED_PHASES row, per handoff): zero-diff producer
proof - `git diff d7f01de --stat -- crates/cli/tests` produced an empty
capture (0 lines), so the producer set is byte-identical across this
hand's range. The timing gate is deterministically blocked at
`tools/bench_compare.py:84` (dated deferred row); `bench_compare.py` was
NOT run (reviewer hop owns it); NO timing number is attributed to this
hand.

## Scope proofs (measured)

- `git status --porcelain` after all edits: exactly the two source files
  (`crates/cli/src/commands/ui.rs`, `crates/cli/src/commands/ui/
  boot_progress.rs`) plus this journal. Nothing else dirtied.
- `git diff d7f01de --stat -- crates/cli/src/commands/ui/`:
  `boot_progress.rs | 212 ++++-`, 1 file changed. The seven frozen test
  modules are byte-identical to the start rev (acceptance 2).
- `git diff --numstat d7f01de`: `ui.rs` +19/-135; `boot_progress.rs`
  +210/-2.

## Measured line counts (measured, not predicted)

- `crates/cli/src/commands/ui.rs`: 986 -> **870** (acceptance band
  830-880 MET; ceiling test `<= 880` green). Above the handoff's
  "roughly 830-860" and the shape's "~830-850": see D3.
- `crates/cli/src/commands/ui/boot_progress.rs`: 115 -> **323** (+208).
  Above the handoff's "roughly 270-310": see D3.
- `resolve_bind_addr` and `warn_if_expose_used` still defined in `ui.rs`
  (grep-verifiable at :721 and :730 post-change); the S6 bind honesty
  strings stay inside the production section the frozen `tests.rs` lock
  cuts at the first `#[cfg(test)]` token (both new cfg(test) re-exports
  are placed after it, next to the `COLLATERAL_FILE_SPAN_END` precedent).

## Verbatim-move evidence

- The eight family functions moved with their doc comments; function
  bodies byte-identical to the `d7f01de` text. The ONLY textual delta is
  a `pub ` prefix on the six formerly private functions (D1). The two
  `pub(crate)` functions are byte-identical, signature included.
- `BootStepSnapshot` (the I2 terminal-parity oracle target) moved into
  the same file's later section UNCHANGED; the file's -2 in numstat is
  the two import lines that were folded into the enlarged import block,
  not a change to the type.
- No new public API name: the `boot_progress` module stays private to
  `commands::ui`, so `pub` inside it is not externally reachable
  (D1, `stage_spans` precedent verbatim: "Items are `pub` at the module
  boundary; the module itself is private to `commands::ui`").
- The moved bodies reference through `super::`: `build_ui_payload`,
  `step_percent`, `swap_into_state`, `write_state`, `ProgressFn`,
  `ProgressReport`, `Result`, `RUN_PHASES`, `STAGES_PER_RUN`,
  `UiBuildResult`, `UiState`; directly: `std::io::Write`,
  `std::path::PathBuf`, `std::sync::{Arc, Mutex}`,
  `ariadne_index::StageFingerprints`,
  `crate::commands::progress::{strip_stage_prefix, TerminalStepBars}`.
  `print_load_complete` stays fully qualified verbatim.
- `ui.rs` re-export surface: production block `use boot_progress::{
  build_boot_generation, drive_boot_build, make_bar_progress,
  make_boot_progress, surface_boot_failure};` mirroring the stage_spans
  comment idiom; test-only block `#[cfg(test)] use boot_progress::
  make_boot_progress_with_sink;` placed after ALL production code next to
  the existing `#[cfg(test)] use stage_spans::COLLATERAL_FILE_SPAN_END;`.
  `make_bar_progress_with_sink` and `panic_payload_text` are NOT
  re-exported (post-move their only callers live inside
  `boot_progress.rs`), exactly per the handoff.

## Deviations (numbered, with reasons)

1. **`pub` visibility marker on the six moved private functions.** The
   handoff demands the re-export surface in `ui.rs` (`use boot_progress::
   {...}`) AND "verbatim" private visibility; these are mutually
   unsatisfiable in Rust (parent modules cannot import private child
   items: E0603). Conservative resolution chosen: mark `pub` exactly like
   the cited `stage_spans` precedent, whose module doc records the same
   rule verbatim ("Items are `pub` at the module boundary; the module
   itself is private to `commands::ui`"). Because `mod boot_progress;` is
   private to `commands::ui`, the effective visibility set is UNCHANGED
   from pre-move (only `commands::ui` and its descendants can name the
   path), so "no new public API name" holds. `pub(super)` would be
   strictly narrower but deviates from the named precedent; the two
   `pub(crate)` functions were kept byte-identical.
2. **`ui.rs` import-surface prune** (the "nothing else in `ui.rs`
   changes" clause under-specified the move fee). The move orphans four
   bindings in `ui.rs`: `std::io::Write`, `TerminalStepBars` (from the
   line-1 pair import), `Mutex` (from the `std::sync` triple), and
   `step_percent` production reachability. Verified against every frozen
   test module's import block: the tests import `Write`/`Mutex`/
   `TerminalStepBars` DIRECTLY from their origin modules (`std::io`,
   `std::sync`, `crate::commands::progress`), never via `super`, so all
   three were pruned (`strip_stage_prefix` stays: live in
   `spawn_watcher`). `step_percent` MUST stay ungated in `ui.rs`: the
   child module `boot_progress.rs` itself consumes it via
   `use super::step_percent` in BOTH cfgs (pre-existing
   `BootStepSnapshot::update_current_row` plus the moved bar renderer) -
   an interim attempt to gate it `#[cfg(test)]` failed the production cfg
   with E0432 (the honest Red-for-the-wrong-reason step of the Green
   phase, captured at `target/testrun-capture.txt` mid-sequence), and it
   was reverted to the original production line. A parent `use` binding
   consumed via a child's `super::` path counts as used, so the restore
   emits zero unused-import warnings in either cfg (gate 4: `0 warnings
   emitted`). Rationale for pruning rather than leaving dead imports:
   the handoff itself treats new `unused_imports` warnings as a stop
   signal ("an unused re-export is an `unused_imports` warning"), and the
   `stage_spans` precedent comment states the house invariant: the
   surface is EXACTLY what the frozen tests import plus what production
   still calls. One 2-line comment records why `TerminalStepBars` left
   the line-1 import.
3. **Measured counts vs estimate bands.** `ui.rs` lands at 870 (handoff
   estimate "~855", refactor target "roughly 830-860"; shape "~830-850");
   `boot_progress.rs` lands at 323 (estimate "~270-310"; shape "~265").
   The hard acceptance band is 830-880 and the new ceiling test locks
   `<= 880` - both MET. The estimates could not hold: deleting the family
   is -132 lines while the handoff REQUIRES the mirrored-idiom re-export
   comments (+13 lines) and the mandated 3-test source-text module lives
   in `boot_progress.rs` (~60 lines). Trimming the required comments to
   chase the estimate band would gut their mandated content; the
   measured 870 leaves 130 lines under the 1000 ceiling, so A-4's +~14
   install fits with room to spare (which is what the F2 row protects).
   Reported measured, per the handoff's "measured, not predicted".
4. **Census wording vs the enumerated surface.** The handoff's frozen
   lock says "zero references to the eight moved names outside `ui.rs`
   and `boot_panic_tests.rs`", but `boot_progress_tests.rs` references
   `make_boot_progress` (import :2, call :535). The handoff's own
   frozen-import enumeration (lines 54-60) explicitly lists and allows
   that surface, and the enumeration matched HEAD exactly, so the
   enumeration is read as authoritative and the census sentence as
   shorthand. Not a STOP; recorded so the reviewer's census check uses
   the same reading.
5. **Package `cargo fmt` not run (pre-existing repo state).** Baseline
   probe at the untouched start rev: `cargo fmt -p ariadne-cli --check`
   FAILS with ~2167 diff lines in files outside the touch-set (e.g.
   `crates/cli/src/commands/analyze.rs`, `crates/cli/tests/
   stage5_memory_bench.rs`). Running the formatter would rewrite
   out-of-touch files (scope breach); skipping it matches main's current
   state. All lines authored by this hand follow the existing file style
   and fit the house 100-column idiom (widest new line: the
   `pub fn build_boot_generation` signature, 98 columns).
6. **Journal landing-hash rider.** The landing hash cannot exist inside
   its own commit; this journal is committed in the gate-7 commit, and a
   one-file journal rider commit records the landing hash + show --stat,
   the established W-3b pattern (`b8597f1` = journal rider for landing
   `24c2f7e`). Both commits stay strictly inside the touch-set. The
   gate-7 message quotes the MEASURED `986 -> 870` instead of the
   handoff's drafted `~855` (see D3): a commit message must not assert a
   number known false.

## Out-of-scope observations (report-only, nothing touched)

- Pre-existing compiler warnings surfaced incidentally during Green
  builds, both outside the touch-set and unrelated to this diff:
  `crates/ui/src/routes/api_graph.rs:719` unused variable `gen`;
  `crates/cli/src/lib.rs:804` unnecessary `mut` on `root_command`.
- The package fmt drift at D5 (main is not `cargo fmt --check` clean).
- Git printed the cosmetic notice "in the working copy of
  'crates/cli/src/commands/ui/boot_progress.rs', LF will be replaced by
  CRLF": the Edit-tool-inserted lines carry LF in the working copy; the
  index blob is LF-normalized like every other Rust blob here. No
  content impact; the byte-identity of the seven frozen modules is
  unaffected (they show no diff at all).

## Landing record (appended by the rider commit)

- Landing hash: `907db9d` - `refactor(cli): extract the boot-progress
  family into commands/ui/boot_progress.rs (ui.rs 986 -> 870)`.
- Gate 7 digest (numstat from `commit_scoped.py`, verbatim):
  `19  135  crates/cli/src/commands/ui.rs`
  `210  2   crates/cli/src/commands/ui/boot_progress.rs`
  `252  0   tasks/extraction-auto-apply-a1-journal.md`
  Exactly the three touch-set files; acceptance 7 MET.
- Gate 8 (`encoding_guard --all`, post-landing): exit 0 (silent).
- Rider commit for this section: the scoped journal-only follow-up per
  deviation D6 (W-3b `b8597f1` precedent).
