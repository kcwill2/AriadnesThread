# W-1 Handoff - ui-extraction-wizard: dto.rs module split (hygiene-forced refactor)

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ui-extraction-wizard-shape.md` sections 5 (module map), 7 (sizing), 9
   (hygiene notes); `tasks/ui-extraction-wizard-kickoff.md` section 6.
3. `crates/ui/src/dto.rs` in full (exactly 1000 lines at HEAD `97dd836`).
4. `crates/ui/src/lib.rs` (lines 2 and 8: `pub mod dto;` and `pub use dto::*;`).

## Objective and preserved invariants

Move the extraction-proposal family out of `crates/ui/src/dto.rs` into a new
submodule `crates/ui/src/dto/extraction.rs`, so `dto.rs` drops under the
1000-line R2 ceiling and the wizard DTOs (W-2b) have headroom. This is a pure
refactor: behavior must be byte-identical.

Invariants this hand must preserve:
- Every public path survives: `crates/ui/src/lib.rs:8` is a bare
  `pub use dto::*;`, so `ariadne_ui::ExtractionProposalDto` and every sibling
  export must keep resolving with the same names, generics and field sets. The
  split works only because `dto.rs` will re-export the submodule
  (`pub use extraction::*;`).
- The "plain display data only" rule documented in the `dto.rs` module doc
  (shape-cited pre-split `dto.rs:182-184`) survives as the moved module's doc.
- Zero behavior change: no test file is edited (shape module map: "no test
  change"); the scoped ui suite must report the SAME canonical line before and
  after.
- No renames, no reordering, no doc rewrites of the moved code: a move plus the
  module header, nothing else.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- tasks/ui-extraction-wizard-w1-journal.md (checkpoint journal; always in-set)
- crates/ui/src/dto.rs (the move-out plus the two submodule lines)
- crates/ui/src/dto/extraction.rs (NEW: the moved family, verbatim)

You MUST NOT touch: `crates/ui/tests/**`, `crates/cli/**`, every other asset
and source path, `tools/**`, `tests/**`. No `git stash`, no branch change.

## What to build

The moved family is the extraction-proposal family as it exists in `dto.rs` at
the start rev (shape-measured anchors, verify on disk before cutting):

- the display caps `EXTRACTION_PROPOSAL_MAX_TARGET_FILES` and
  `EXTRACTION_PROPOSAL_MAX_NAMES` (shape-cited `dto.rs:244-259`);
- `ExtractionProposalDto` (shape-cited `dto.rs:261-286`) and the per-file
  group types it owns;
- the builder/helper functions whose ONLY consumers are that family (read the
  file; the boundary is "everything the proposal payload needs and nothing
  else" - if a helper is shared with another family it stays in `dto.rs`).

`dto.rs` keeps: `pub mod extraction;` and `pub use extraction::*;` (a short
module doc naming the split and the R2 reason), and everything that is not the
proposal family. `extraction.rs` opens with a module doc that carries the
plain-display-data rule forward, then the moved code verbatim.

## RGB protocol (per module, in this file order)

Red: deliberately SKIPPED, with reason - this hand is a pure module move under
a "no test change" bound (shape module map), so there is no failing assertion
to author: editing a test to manufacture one would violate the touch-set, and
a vacuous one would be theater. The refactor oracle is the UNEDITED suite
staying green plus the reverse-dependency compile proofs below. Declared here
per the M28 T0b finding-1 dated-D-row law (the journal records it as D1).
Green: perform the move; re-run until the canonical line matches the baseline
line exactly (same passed / failed / ignored / suites).
Refactor: none available by contract (the move itself is the refactor). If a
helper turns out to be shared across families, leave it in `dto.rs` and report
the observation - do not duplicate it.

## Required tests

None new, none edited. The acceptance set is:
1. `python tools/testrun.py --distill -p ariadne-ui` at the start rev (record
   the canonical line in the journal). If the start tree is not green, STOP
   and report: every wizard hand assumes a clean base.
2. The same command after the move: exit 0 with an identical canonical line.
3. `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`:
   exit 0. This is the public-path proof (F14 standing rule 2: `ariadne-cli`
   is the live reverse dependency of `ariadne_ui`'s surface).
4. `python tools/distill.py -- cargo check -p ariadne-integration-tests
   --all-targets`: exit 0 (F14 standing rule 2, second enumerated reverse
   dependency).
5. Pure-move proof for the reviewer: in the landing commit,
   `git show -- <commit> -- crates/ui/src/dto.rs crates/ui/src/dto/extraction.rs`
   must show the dto.rs deletions equal to the extraction.rs additions, with
   only the two submodule lines and the module doc added on either side. Any
   content edit inside the moved block is a blocker.

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned identifier names
(tmp/res/foo/bar/test1), no single-letter locals, intention-revealing names,
one responsibility per function. Expect `dto.rs` to land well under 1000
(shape estimate ~300-450 produced lines) and `extraction.rs` under 1000.
`python tools/hygiene_check.py` must stay exit 0 with zero NEW findings in the
two touched files.

## Frozen locks

The scoped ui suite is the lock: it must run green UNEDITED (every existing
ui_test lock, including `proposals_payload.rs`, `static_html.rs`,
`export_order.rs`, the card-family locks, `extraction_contrast.rs` and the
theme oracles). `tests/golden/**` and `tests/fixtures/**` are byte-locked and
must not appear in the diff. `tools/theme_token_baseline.json` diff-empty.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/testrun.py --distill -p ariadne-ui` (baseline; record the
   canonical line verbatim in the journal)
2. `python tools/testrun.py --distill -p ariadne-ui` (post-move; canonical
   line must be identical to gate 1; exit 0)
3. `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`
   (exit 0)
4. `python tools/distill.py -- cargo check -p ariadne-integration-tests
   --all-targets` (exit 0)
5. `python tools/hygiene_check.py` (exit 0)
6. `python tools/encoding_guard.py --staged` (exit 0 before committing)
7. `python tools/commit_scoped.py --scope crates/ui/src/dto.rs --scope
   crates/ui/src/dto/extraction.rs --scope
   tasks/ui-extraction-wizard-w1-journal.md -m "refactor(ui): move the
   extraction-proposal family into dto/extraction.rs (R2 headroom for the
   wizard DTOs)"` (never `git add .`; scope globs are the touch-set exactly)
8. `python tools/encoding_guard.py --all` (post-commit, exit 0)

Gate count: 8 (within the ~10 bound; the commit-cycle trio is gates 6-8 and is
itemized here per the M28 T0b finding-1 journal law).

## Discipline clauses

- UTF-8: never Set-Content/Out-File/>/Get-Content-replace; Write/Edit tools
  only; all authored content pure ASCII (see AGENTS.md File-Edit Discipline).
- Touch only the touch-set; on discovering out-of-scope breakage: report, do
  not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`) in an implementer session
  are a reviewer blocker; run them through the distilling entry points
  (`docs/specs/tool-output-distillation.md`). This hand is Rust-source-only,
  so no ui_smoke or bench gate applies (zero assets/JS diff proof holds by
  construction - state it in the journal).
- After each Green gate, append a 1-3 line checkpoint to
  `tasks/ui-extraction-wizard-w1-journal.md`.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - the successor runs `python
  tools/checkpoint_guard.py resume --spec tasks/ui-extraction-wizard-w1-handoff.md
  --scope ... --cmd ...`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. The Red-skip declaration with reason.
3. Green evidence (distill digest header + canonical line, before and after,
   identical + capture path). 4. Any deviation + reason. 5. Out-of-scope
observations (report-only). Never paste raw tool output: cite the command, the
canonical summary line and the exit code.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the touch-set above exactly, so
every file this task may legitimately change must stay listed (including
Cargo.lock if cargo regenerates it).
