# W-1 Verdict - ui-extraction-wizard: dto.rs module split

Handoff: `tasks/ui-extraction-wizard-w1-handoff.md`
Journal: `tasks/ui-extraction-wizard-w1-journal.md`
Range under review: `5cc4ac3..82e5ffe` (landing `b27aa46`, parent `5cc4ac3`; journal rider `82e5ffe`)

## Verdict: APPROVED

Zero blockers. Six numbered findings, all minor or relay; none blocks the
landing. W-1 discharges the R2 1000-line wall exactly as planned: `dto.rs`
1000 -> 953 lines, `dto/extraction.rs` NEW at 68 lines, every public path
survives, the frozen ui suite is byte-identical and green.

## Ground truth (before judging)

`python tools/checkpoint_guard.py status`:

- HEAD `82e5ffe`, branch `main`, dirty=0, untracked=0.
- Commits: `82e5ffe` (docs rider), `b27aa46` (landing), `5cc4ac3` (duty-1
  planning, docs-only + handoffs, outside this handoff's range).
- Worktree census: 5 worktrees, none owned by this milestone; the only dirty
  entries are foreign-track files in `mergetest` (272) and
  `tarjan-s-scc` (2) / `shining-enthusiasm` (1), all pre-existing.

## Fresh verification (re-run by the reviewer; nothing trusted)

| Gate | Command | Result |
|---|---|---|
| G1 baseline | journal record (start rev) | `283 passed / 0 failed / 0 ignored / 3 suites` |
| G2 post-move | `python tools/testrun.py --distill -p ariadne-ui` | `### cargo test PASS (exit 0) [capture: ...testrun-capture.txt, 312 lines]` / `283 passed / 0 failed / 0 ignored / 3 suites` / `no failures` |
| G3 F14 cli | `python tools/distill.py -- cargo check -p ariadne-cli --all-targets` | `ok (exit 0, 0 warnings emitted) -> capture: ...distill-capture.txt` |
| G4 F14 integration | `python tools/distill.py -- cargo check -p ariadne-integration-tests --all-targets` | `ok (exit 0, 21 warnings emitted) -> capture: ...distill-capture.txt` |
| G5 hygiene | `python tools/hygiene_check.py` (via distiller for the exit code) | `### generic PASS (exit 0)`; 179 capture lines, all R3 warn-only |
| G6 encoding pre-commit | `encoding_guard.py --staged` | not reproducible post-hoc (staging empty; `commit_scoped.py` owns staging - standing verdict precedent); superseded by G8 |
| G7 landing | exact handoff command | `b27aa46` stat: dto.rs 7+/54-, dto/extraction.rs 68+/0-, journal 125+/0- |
| G8 encoding post-commit | `python tools/encoding_guard.py --all` (via distiller) | `### generic PASS (exit 0)`; capture 0 lines |

The 21 integration warnings in G4 are pre-existing in untouched files
(golden-helper dead code, AR-21 `IncludeResolver` deprecation, `unused
variable: gen` at `crates/ui/src/routes/api_graph.rs:719`); none names
`dto.rs` or `dto/extraction.rs`.

## Commit-vs-touch-set (enforced)

`git diff --name-only 5cc4ac3..HEAD` -> exactly three paths:

1. `crates/ui/src/dto.rs`
2. `crates/ui/src/dto/extraction.rs`
3. `tasks/ui-extraction-wizard-w1-journal.md`

All inside the declared touch-set. Per-commit, oldest first:

- `b27aa46`: modifies `crates/ui/src/dto.rs` (+7/-54), adds
  `crates/ui/src/dto/extraction.rs` (+68), adds the journal (+125). Nothing
  else.
- `82e5ffe`: journal only (+26/-2), replacing two "(pending) Gate 7/Gate 8"
  placeholders with the landing records. Docs-only rider, inside the
  touch-set.

No commit lands outside the touch-set. No `Cargo.lock` regeneration, no
`tests/golden/**`, no `tests/fixtures/**`, no `tools/theme_token_baseline.json`
(git verified: `git diff --name-only 97dd836 HEAD` lists only tasks/**, one
.kilo/plans doc, and the two dto paths).

## Pure-move proof (independent, not the implementer's script)

The sanctioned byte path (`git cat-file` inside Python) was refused by the
session allowlist, so the proof was re-established with allowlisted
blob-vs-blob diffs plus the Read tool on the checkout:

1. `git diff --numstat "5cc4ac3:crates/ui/src/dto.rs"
   "82e5ffe:crates/ui/src/dto/extraction.rs"` -> `12 944
   crates/ui/src/{dto.rs => dto/extraction.rs}`: of extraction.rs's 68 lines,
   exactly 56 are shared with the pre-move dto.rs (53 moved content lines +
   3 matched blanks); exactly 12 lines are new (the module doc and the two
   `use` lines).
2. `git diff -U0` of the same pair shows the moved region (old dto.rs lines
   233-286) as unchanged context between deletion-only hunks, i.e. the moved
   block is byte-identical, including its two U+2014 em dashes (which render
   in the session console and are gated by `encoding_guard --all` exit 0).
3. `git diff --numstat "97dd836:crates/ui/src/dto.rs"
   "HEAD:crates/ui/src/dto.rs"` -> `7 54`: dto.rs added only the module doc
   plus `pub mod extraction;` / `pub use extraction::*;` and deleted the
   block region. Read-to-EOF on the checkout confirms dto.rs = 953 lines and
   extraction.rs = 68 lines.
4. Accounting: the 54 deletions = the 53 byte-identical block lines + one
   separator blank line (old line 287) collapsed outside the moved block
   (rustfmt-consistent; see Finding 2). No content edit inside the moved
   block.

## Plan-literal compliance

- Shape section 5, W-1 row: "dto.rs (move out), NEW dto/extraction.rs, no
  test change; anything else must not change; behavior byte-identical" - met
  exactly.
- Kickoff section 10 item 1: "dto.rs under 1000 lines with dto/extraction.rs
  holding the proposal family; scoped ui suite green UNEDITED; hygiene exit
  0" - met: 953 lines, family moved, `crates/ui/tests` byte-identical
  between base and HEAD (`git diff --numstat` on the whole tree returns
  empty), hygiene exit 0.
- Kickoff section 6 / shape R8: dto.rs at exactly 1000 lines forced the
  split; W-2b's wizard DTOs now have the new submodule (68 lines) as
  headroom. Risk discharged.
- Invariants: `crates/ui/src/lib.rs:2` `pub mod dto;` and `:8`
  `pub use dto::*;` are unchanged (lib.rs not in any diff). Every moved name
  is still consumed live through the `ariadne_ui::` root - cli
  (`ariadne_ui::ExtractionProposalDto`, `ProposedTargetFileDto`,
  `EXTRACTION_PROPOSAL_MAX`, `..._MAX_NAMES`, `..._MAX_HEADERS`,
  `..._MAX_TARGET_FILES`) and the frozen ui tests (`proposals_payload.rs`,
  `payload_etag.rs`, `dto_builders.rs`) - and both compile and pass. The
  plain-display-data rule survives as the new module's doc (extraction.rs
  lines 7-10) and inside the moved struct docs. No rename, reorder, or
  doc rewrite inside the moved block; `JUSTIFICATION_MAX_NAMES` /
  `JUSTIFICATION_MAX_HEADERS` stay private in dto.rs and are imported by the
  submodule.

## Acceptance items (handoff "Required tests" set), one by one

| # | Item | Verdict | Evidence |
|---|---|---|---|
| 1 | `testrun.py --distill -p ariadne-ui` at the start rev; if not green, STOP | PASS | Journal records `283 passed / 0 failed / 0 ignored / 3 suites` exit 0. Independently corroborated: `crates/ui/tests` is byte-identical base..HEAD (empty numstat), so the suite composition - and therefore the start-tree canonical line - is provably unchanged. |
| 2 | Same command after the move: exit 0, identical canonical line | PASS | Fresh reviewer run: `283 passed / 0 failed / 0 ignored / 3 suites`, exit 0, capture 312 lines. Identical to item 1. |
| 3 | `distill.py -- cargo check -p ariadne-cli --all-targets` exit 0 (public-path proof) | PASS | `ok (exit 0, 0 warnings emitted)`. The live reverse dependency compiles against the unchanged `ariadne_ui::` surface. |
| 4 | `distill.py -- cargo check -p ariadne-integration-tests --all-targets` exit 0 | PASS | `ok (exit 0, 21 warnings emitted)`; all 21 pre-existing in untouched files. |
| 5 | Pure-move proof in the landing commit | PASS | 53/53 byte-identical moved lines (numstat `12/944` with the block as unchanged context); only the module doc + submodule lines + two mechanical `use` lines are new. One formatting blank collapsed outside the block (Finding 2) - no content edit inside the block. |

Handoff gates 1-8: all discharged (see the verification table; G6 via the
G8 superset with the standing precedent).

## Deviation adjudication (independent)

- **D1 Red-skip - ACCEPTED.** The handoff's RGB section pre-declares the
  skip with its reason; the "no test change" bound is in the handoff and the
  shape module map; editing a test to manufacture a failing assertion would
  violate the touch-set. The substitute oracle is stronger than a vacuous
  test: the byte-identical frozen suite (item 2) plus two F14 compile proofs
  (items 3-4) plus the workspace census at the ledger anchor.
- **D2 two mechanical `use` lines - ACCEPTED.** extraction.rs:12
  `use serde::{Deserialize, Serialize};` is required by the verbatim derive
  attributes (lines 31, 46); extraction.rs:14
  `use super::{JUSTIFICATION_MAX_HEADERS, JUSTIFICATION_MAX_NAMES};` is
  required because the moved cap definitions (lines 22-23) reference the
  parent's private justification caps. Both parent consts keep their exact
  declaration text and stay private in dto.rs; widening them to `pub` would
  have been the worse change. Minimal and necessary.
- **D3 moved block carries the pre-existing em dashes byte-identical -
  ACCEPTED.** Verified independently (the moved block is byte-identical; the
  em dashes moved with it; `encoding_guard --all` exit 0). The authored-ASCII
  rule governs newly authored lines, and every newly authored line here is
  pure ASCII.
- **D4 shape's dto.rs anchor drift - ACCEPTED.** The handoff's own "verify on
  disk before cutting" clause governs; the rule lives at disk 248/262, not at
  the cited 182-184, and it survives in the new module doc. Recorded as a
  planning erratum (Finding 3), not an implementation deviation.
- **D5 journal rider - ACCEPTED.** The rider is inside the touch-set,
  docs-only, and resolves the chicken-and-egg of recording the landing hash
  (the journal honestly carried "(pending)" placeholders at `b27aa46`).
  Precedent exists in the ledger (branding-assets' two declared riders;
  ui-degree-opacity-ramp docs riders). NOTE: D5 is not numbered in the
  journal (Finding 1).
- **D6 capture-overwrite re-run - ACCEPTED with a provenance caveat.**
  `target/testrun-capture.txt` is a fixed path, so the gate-1 "before"
  capture was overwritten by the gate-2 "after" run. The before/after
  equivalence nevertheless holds by three independent proofs: the whole
  `crates/ui/tests` tree is byte-identical between base and HEAD, the fresh
  post-move run reproduces `283 passed / 0 failed / 0 ignored / 3 suites`,
  and the workspace census matches the ledger anchor exactly. NOTE: D6 is
  not numbered in the journal (Finding 1).

Undeclared deviations found in per-commit review: none.

## Findings

1. **Minor** - `tasks/ui-extraction-wizard-w1-journal.md` declares only
   D1-D4. The rider commit and the capture-overwrite re-run (the dispatch's
   D5/D6) are real deviations that exist only implicitly in the journal
   text; per the handoff's own D-numbering discipline (M28 T0b finding-1
   dated-row law) they should have been numbered. No scope or behavior
   impact; fix at the next journal touch.
2. **Minor** - pure-move accounting nuance: dto.rs deletions (54) exceed the
   moved block (53) by one separator blank line (old line 287, between the
   block and `GraphExportPayload`); the leading separator blank was kept. The
   result is what rustfmt produces and nothing inside the block changed, but
   the handoff's wording "the dto.rs deletions equal to the extraction.rs
   additions" is literally true only for the 53 content lines, not for the
   whole file pair. Recorded for the record; no change required.
3. **Relay** - shape/kickoff cite the plain-display-data rule at pre-split
   `dto.rs:182-184`; on disk it lives at 248/262 inside the moved struct
   docs. A docs erratum to fix at a future `tasks/**` touch (out of W-1's
   touch-set: only the journal is in-set).
4. **Relay (bench gate only; not a W-1 blocker)** - `tools/bench_compare.py:84`
   pins DOCUMENTED_PHASES entry `generate_report+detect_cycles (combined)`
   while `crates/cli/tests/payload_tail_bench.rs:389` prints
   `generate_report+detect_cycle_groups (combined)` since the M23.5 rename
   (commit `9194c25`, 2026-10-04). The stage5-timing gate therefore exits 2
   (`parse format drift: documented phase row(s) missing`) on every run on
   this machine, before and after W-1. Zero-diff proof for W-1: the bench
   source, `tools/bench_compare.py` and `tools/bench_baseline.json` are all
   byte-identical between `5cc4ac3` and HEAD (empty numstat on each), and the
   W-1 range contains no path under `crates/cli/**` or `tools/**`. The fix is
   a one-line tuple edit plus a dated ledger row, owed to the orchestrator
   (tools/** is forbidden to this hand and to the reviewer).
5. **Observation** - produced-lines estimate vs actual: shape section 7 and
   the plan estimate ~300-450 produced lines for W-1; the actual source diff
   is 129 lines (7 + 68). Well inside the sizing-law ceiling (~1200) and the
   8-gate bound; noted only as estimate calibration for later hands.
6. **Configuration note (session allowlist)** - the following were refused
   with the verbatim catch-all text
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`:
   `python -c ...` (byte-proof attempts, twice), a heredoc + `Set-Content`
   script form, `git grep ...` (twice), `cmd /c ...` (exit-code probe),
   `python tools/ui_smoke.py` (direct), `python tools/bench_compare.py`
   (direct). The Write tool to the temp dir was refused with
   `{"permission":"edit","pattern":"*","action":"deny","source":"agent"}`.
   Consequences and substitutions: (a) the sanctioned byte-level
   `git cat-file`-inside-Python proof was unavailable, so the pure-move proof
   was re-established with allowlisted blob-vs-blob `git diff` plus the Read
   tool on the checkout (stronger, not weaker); (b) `ui_smoke` and
   `bench_compare` were re-run through the documented distilling entry point
   (the kickoff's own fallback for exactly this allowlist case); (c) the
   user-stated unittest pattern `-p "*_test*_tests.py"` matches zero files
   (`Ran 0 tests` / `NO TESTS RAN`), so the documented CI form
   `-p "*_tests.py"` was used instead (219 tests, OK). Every gate the hop
   owed has evidence; nothing was skipped silently.

## Follow-up dispositions (recorded, never dropped)

- Deferred ledger row owed: bench_compare DOCUMENTED_PHASES tuple fix
  (Finding 4) - one line, tools/**, operator/orchestrator duty.
- Deferred docs erratum: shape/kickoff `dto.rs:182-184` citation
  (Finding 3).
- Journal deviation numbering (Finding 1) - next handoff's journal author
  should number D5/D6-style riders explicitly.

## Duty 1 for the next implementer hop

The orchestrator should route exactly this command to the next implementer
hop to land this verdict and its execution-results (both files are in the
reviewer's edit grant; handoffs/ledger are excluded because they already
landed in `5cc4ac3`):

```
python tools/commit_scoped.py --scope tasks/ui-extraction-wizard-w1-verdict.md --scope tasks/ui-extraction-wizard-w1-execution-results.md -m "docs(tasks): W-1 gate APPROVED (dto.rs split, 953 lines) + execution-results"
```
