# Verdict - ui-progress-parity T3: overlay step-row renderer (cell-glyph bars)

Reviewers hop 2026-10-04, via `review-gate`. Range under review:
`587c409..1bc50b8` (commits `e9cf10d` feature, `1bc50b8` journal evidence
follow-up), reviewed oldest-first, each as if alone. This hop additionally
owns the milestone-wide gates deferred from the T1/T2 hops.

## VERDICT (HOP 1): REJECTED -> see HOP 2 RE-ADJUDICATION at the end of
this file for the final state

One blocker (finding 1). Everything else in the handoff landed clean; the
remediation is small and the milestone gates are re-runnable evidence, not
rebuilt work.

## Ground truth and commit-vs-touch-set

- `python tools/checkpoint_guard.py status`: HEAD `1bc50b8`, branch `main`,
  dirty = planning docs only (none in the touch-set); worktree census
  unchanged from prior hops.
- `git diff --name-only 587c409..HEAD` = `api.js`, `index.html`, `main.js`,
  `styles.css`, `static_html.rs`, T3 journal = exactly the 6 declared
  paths. No `Cargo.lock`. `git show --stat "e9cf10d"` = 6 files,
  202 insertions / 4 deletions - the claimed figure, verified. `1bc50b8`
  touches only the journal (+11). All 4 deletions are the prescribed
  api.js replacement lines; zero deletions in `static_html.rs`.

## Frozen locks (reviewer's own runs, all empty diffs)

- T3 range `587c409..HEAD` vs `crates/cli`, `crates/pipeline`,
  `crates/ui/src`, `server_api.rs`, `common.rs`: EMPTY (T3 touched none).
- Milestone range `e276910..HEAD` vs `crates/pipeline` (I9),
  `crates/cli/src/commands/progress.rs`, `themes.css`, `theme.js`,
  `tools/theme_token_baseline.json`, `tests/fixtures`, `tests/golden`:
  EMPTY. All locks held across the whole milestone.

## Plan-literal checks

- DP-2: `Math.floor(20 * percent / 100)` present verbatim inside
  `buildStepBarCells` (prescribed refactor helper), percent clamped via
  `Math.min(percent, 100)` before flooring, remainder = 20 - filled,
  total always 20, `[` `]` wrap, monospace `.step-bar`; code glyphs are
  `"\u2588"`/`"\u2591"` escapes; test locks literal-backslash needles.
- DP-1: all rows render (none hidden); `step-pending` = dimmed
  (`opacity: 0.45`, `var(--text-muted)`) with EMPTY detail (`main.js:306`).
- DP-3: done rows accent-only (`var(--cyan)` on label/bar/percent); no
  green/amber/red or heat-ramp token anywhere in the appended block.
  All tokens used (`--text-main`, `--text-sub`, `--text-muted`, `--cyan`,
  `--cyan-glow`) are defined in all three theme blocks (themes.css
  51-56 / 182-187 / 321-326) - I8 token-reuse safe, steampunk legibility
  is token-guarded as shape R5 requires.
- Legacy contract: `showProgress`/`updateProgress`/`hideProgress` names and
  behavior intact (`main.js:232-246`; `hideProgress` gains only the
  prescribed `#loadingSteps` clear, `main.js:250-251`); legacy single-bar
  path reached via `renderProgressSnapshot` empty-steps delegation
  (`main.js:277-281`); `index.html` gained exactly one container div
  (line 32), no new inline styles; all 9 pre-existing overlay-test needles
  kept verbatim (`static_html.rs:526-537`); poll branch replaced the single
  named line with `renderProgressSnapshot(status.progress)` (api.js:56),
  `state === 'loading'` needle stays, ready/error branches unchanged;
  standalone branch feeds the ONE renderer via `standaloneBootSnapshot`.
  DOM composition is `textContent`-only, zero `innerHTML` on the row path.
- Hygiene ceilings: `main.js` 570 lines (was 513), `renderProgressSnapshot`
  ~38 lines, helper 6 lines; no line near 2000 chars. `styles.css`
  appended-only (diff hunk `@@ -1592,3 @@`).

## Report-evidence check

Journal is digest-shaped: canonical lines quoted verbatim, failing-needle
citation, exit-code semantics, zero raw dumps, distilling entry points only.
Evidence-shape gaps recorded as finding 3.

## Journaled deviation - independent adjudication

**Legacy hiding scope (bar row only vs shape.md:183 "hidden" broadly):
REJECTED with required change - finding 1 (blocker).**
- `main.js:284-285` sets `display:none` on the `loadingProgress` wrapper
  only. `loadingText` and `loadingDetail` (index.html:30-31) stay visible
  with FROZEN fallback text during row rendering: server path shows
  "Connecting to backend..." (api.js:43) + "Initializing" above five live
  step rows for the entire boot (seconds to minutes); standalone export
  shows "[1/3] Loading... 0%" + "Initializing" above the synthetic rows.
- Shape.md lines 182-184 (handoff binding input #2, lines 22-24, names it
  complete-contract input): the retained legacy fallback UI is
  "hidden when rows render". `loadingOverlay` necessarily hosts the rows,
  so the hiding obligation falls on the bar/text/detail trio, not just
  the bar. Handoff line 89 narrowed it to the bar row; the narrowing
  produces the exact two-stories-in-one-surface confusion the kickoff's
  intent section exists to remove. Adjudicated on merits, not on the
  declared-ness: the shape contract governs; the process gap (handoff
  narrowing a binding input) belongs to the handoff author, but the
  shipped result fails the contract.
- No test locks this either way (R1 DOM-less residual risk acknowledged by
  the shape); the finding is in the code.

## Findings (numbered)

1. BLOCKER - `crates/ui/assets/main.js:284-285`: row path hides only the
   bar wrapper; `loadingText`/`loadingDetail` remain visible with stale
   frozen text while step rows render, violating shape.md:182-184
   (binding input). See deviation adjudication above.
   Required change + commands in Remediation.
2. Minor, must-fix in the same rework - `crates/ui/assets/main.js:261`:
   the `buildStepBarCells` doc comment contains RAW block glyphs
   (literal U+2588 and U+2591 characters in the illustrative bar),
   byte-level confirmed (non-ASCII
   regex match on the committed line; the other assets matches are
   pre-existing). Violates the handoff discipline clause ("the block
   glyphs exist ONLY as `\u2588`/`\u2591` escapes in JS source, never as
   raw bytes in any file you author") and AGENTS.md pure-ASCII authored
   content. `encoding_guard` stays green (valid UTF-8), so this is a
   rule breach, not a tool failure. Note the comment asserts "Glyphs stay
   ASCII escapes" while itself carrying raw glyphs.
3. Minor - journal evidence shape vs the handoff report-back spec: no
   capture path cited for the testrun gates, and no `exit 0` lines for
   gates 3/5 (the journal instead argues silent==exit-0 from tool
   sources). The claim VERIFIED TRUE by the reviewer independently:
   `theme_token_check.py` prints violations only and returns
   `1 if violations else 0` (lines 211-213); `encoding_guard.py`
   identical shape (lines 171-173); both tools re-run clean by the
   reviewer this hop. Evidence-format finding only.
4. Disposition (accepted) - `standaloneBootSnapshot` done rows keep
   percent 25/65 (never 100 when marked done). Handoff lines 99-102 pin
   "25/65/100 per the existing calls", so this is plan-literal; cosmetic
   and transient (~100 ms). Carry to the walkthrough note (R7-style),
   no rework.
5. Observation (handoff-side, not implementer) - handoff line 51 says
   `styles.css` is 1422 lines; the pre-commit file is 1594 (diff hunk
   `@@ -1592,3 @@`). Stale planning note; the known ledger finding about
   `styles.css` R2-invisibility already covers the size class. No action.
6. Observation (historical Red) - journaled Red cites "2 suites" vs green
   "3 suites" for the same scoped package: third recurrence of this
   pattern (T1 finding 4, T2 finding 3). Not re-executable; predicted
   needle + canonical line meet the Red protocol; reviewer's fresh runs
   confirm 3 suites scoped and 85 suites workspace-wide. Awareness only.
7. Config/doc-errata (reviewer-side, orchestrator to triage into the
   ledger deferred-findings) - kickoff G7's literal
   `python tools/testrun.py --baseline` is NOT runnable: `--baseline` is
   a VALUE flag requiring an `N/M/K/S` baseline file (testrun.py:6,
   59-60, 206-207), and NO baseline file is committed anywhere in the
   repo (glob `**/*baseline*`: only theme/bench baselines). Reviewer ran
   the workspace census via `python tools/testrun.py --distill` and
   quotes the canonical line (below). Distinct from the T2 finding-2
   doc-errata row (14-globals vs 13-frames), whose ledger append remains
   gated on a T3 APPROVE and is NOT performed this hop.
8. Gate-config (reviewer sandbox) - `ui_smoke.py` (2 attempts, incl.
   explicit workdir) and `bench_compare.py` (1 attempt) were refused
   with verbatim: `The user has specified a rule which prevents you from
   using this specific tool call. Here are some of the relevant rules
   {"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
   All other gates ran. This matches the sizing-law carve-out
   ("`ui_smoke`/`bench_compare` run ONLY in a fresh session or the
   orchestrator's bounded own run"); the orchestrator must execute G3 at
   closeout. The T3 implementation's own claim that gate commands
   "ran verbatim and unrefused" is consistent with the journal's scoped
   set (it never ran these two - correctly).

## Gate evidence (reviewer's own fresh runs, this hop)

- `python tools/testrun.py --distill -p ariadne-ui` (exit 0, capture
  `target/testrun-capture.txt`): `184 passed / 0 failed / 0 ignored /
  3 suites` - matches journaled Green + post-refactor lines exactly.
- `python tools/hygiene_check.py` - R3 warn-only, none in any T3-touched
  file; exit 0 per harness non-zero-is-failure semantics.
- `python tools/encoding_guard.py --all` - silent = clean, exit 0
  (source-verified + no harness failure report).
- `python tools/theme_token_check.py --check-added --base main` - silent,
  zero added hex (note: post-merge on `main` this diff is trivially
  empty; re-run substantively below).
- `python tools/theme_token_check.py --check-added --base e276910` -
  silent, zero added hex across the WHOLE milestone touch set: G4 MET at
  milestone scope.
- `python -m unittest discover -s tools -p "*_tests.py"` -
  `Ran 188 tests in 19.895s` / `OK`.
- `python tools/testrun.py --distill` (workspace census; `--baseline`
  form unrunnable, finding 7) - exit 0: `1168 passed / 0 failed /
  4 ignored / 85 suites`.
- `python tools/ui_smoke.py` - NOT RUN here (refused; finding 8).
- `python tools/bench_compare.py` - NOT RUN here (refused; finding 8;
  not among kickoff G1-G7; T3 is assets-only with `crates/pipeline`
  byte-identical, so no expected load effect).

## Milestone gate scoreboard (kickoff G1-G7, as of this hop)

- G1 scoped cli+ui green: MET (cli per T2 verdict, ui re-run this hop;
  both quoted verbatim).
- G2 frozen oracle: MET (progress.rs EMPTY diff `e276910..HEAD` re-proven
  this hop).
- G3 ui_smoke: NOT PROVEN in this hop - refused in reviewer sandbox;
  orchestrator closeout run required.
- G4 zero added hex: MET (base e276910, silent).
- G5 hygiene + encoding: MET (exit 0 both).
- G6 tools discover: MET (`Ran 188 tests ... OK`).
- G7 workspace baseline census: CENSUS MET (`1168 passed / 0 failed /
  4 ignored / 85 suites`); baseline-FILE comparison unrunnable - no
  committed baseline (finding 7).

## Milestone closeable: NO

Reasons: (a) T3 REJECTED with blocker finding 1; (b) G3 `ui_smoke` unrun
(orchestrator/fresh-session hop required); (c) the T2 finding-2 ledger
errata row remains gated on T3 APPROVE and was therefore NOT appended.

## Remediation (same implementer session, exact)

1. `crates/ui/assets/main.js` `renderProgressSnapshot` row path: also set
   `display:none` on `loadingText` and `loadingDetail` (index.html:30-31)
   when rows render, and restore their display (e.g. `display = ''`) at
   the top of the empty-steps legacy delegation branch so the fallback
   stays honest against legacy fixtures. Optionally extend the overlay
   test with one source needle locking the text-row hiding (the handoff
   extension list permits additions; do not weaken anything).
2. `crates/ui/assets/main.js:261`: rewrite the comment with zero raw
   non-ASCII bytes, e.g. `// 20-cell block-glyph bar ([fill x N + light
   remainder], CLI format_step_bar look), with the ...` - block glyphs
   must appear ONLY as `\u2588`/`\u2591` escapes in the file.
3. Re-run, in order, quoting canonical lines in the journal:
   `python tools/testrun.py --distill -p ariadne-ui` (expect `0 failed`;
   if a new test needle was added, capture the Red first per RGB),
   `python tools/theme_token_check.py --check-added --base main`,
   `python tools/hygiene_check.py`.
4. `python tools/encoding_guard.py --staged`, then commit via
   `python tools/commit_scoped.py --scope "crates/ui/assets/index.html"
   --scope "crates/ui/assets/main.js" --scope "crates/ui/assets/api.js"
   --scope "crates/ui/assets/styles.css" --scope
   "crates/ui/tests/ui_test/static_html.rs" --scope
   "tasks/ui-progress-parity-t3-handoff-journal.md" -m "fix(ui): hide
   legacy text rows under step-row rendering; ASCII-only bar comment
   (ui-progress-parity T3)"` (omit any scope you did not touch; never
   `git add .`).
5. Carry-over (non-blocking, T2 finding 1): fix
   `tasks/ui-progress-parity-t2-handoff-journal.md:50` "124 lines" ->
   115 in a separate docs-only scoped commit
   (`python tools/commit_scoped.py --scope
   "tasks/ui-progress-parity-t2-handoff-journal.md" -m "docs: fix T2
   journal line-count note"`); the T2 journal is outside the T3
   touch-set, so never co-stage it with the finding 1/2 fix.
6. Append journal checkpoints for the rework Green + commit hash.

## Follow-up dispositions (orchestrator-side)

- Finding 7 (G7 `--baseline` flag-shape vs no committed baseline file):
  triage into `tasks/milestone_ledger.md` deferred-findings, or fix the
  kickoff wording at closeout.
- T2 finding-2 doc-errata row: NOT appended (T3 not approved); re-gated
  on the APPROVE hop after remediation.
- Finding 4 (standalone 25/65 done rows): walkthrough note.
- Finding 8: schedule G3 `ui_smoke` (+ optional `bench_compare`) in a
  fresh/orchestrator session; the milestone cannot close without G3.

---

# HOP 2 RE-ADJUDICATION (rework range 1bc50b8..a17952a, 2026-10-04)

Commits under review, git order (oldest first): `9d96272` (blocker fix +
needles + journal hop-2 section), `fe29dfc` (T2 journal carry-over,
docs-only), `a17952a` (journal hash recap). NOTE: the orchestrator brief
listed `a17952a` before `fe29dfc`; git order is as above - reviewed in
git order.

## VERDICT (HOP 2): REJECTED -> superseded by HOP 3 RE-ADJUDICATION at the
end of this file (APPROVED)

The hop-1 remediation is faithfully executed and all three hop-1
findings are CLOSED, plus the T2 carry-over. One NEW blocker surfaced by
full-diff interaction review: the error branch loses its visible error
text (new finding N1). Everything else is merge-ready.

## Hop-1 finding closures (reviewer-verified, not trusted)

- F1 (blocker, trio hiding): CLOSED. `setLegacyFallbackVisible(visible)`
  toggles the bar wrapper + `loadingText` + `loadingDetail` together
  (main.js:248-259); row path calls `(false)`; empty-steps legacy branch
  calls `(true)` before `updateProgress`; `hideProgress` calls `(true)`
  inside the fade timeout. `static_html.rs` adds 6 pure-addition
  needles: function presence, `(false)` on the row path, `(true)` count
  >= 2, and a hideProgress-body slice assert locking the restore site.
- F2 (raw block glyphs): CLOSED by reviewer byte scan: pattern for the
  U+2588/U+2591 CODEPOINTS finds zero hits across `crates/ui/assets`.
  (Scan self-validates: the literal-text form of the pattern would have
  matched the intentional `\u2588` source escapes; it matched none.)
  Rewritten comment is pure ASCII; escape sequences inside a comment are
  inert.
- F3 (journal format): CLOSED. Hop-2 section adds capture paths and the
  exit-0 lines for gates 3/5; also records the G3/bench refusals verbatim
  from the implementer session, matching the reviewer's own refusals.
- T2 finding 1 (journal:50 124 -> 115): CLOSED via `fe29dfc` - docs-only,
  exactly the T2 journal in scope, value now matches the committed blob.

## Deviation adjudication (journaled): restore inside the fade setTimeout

ACCEPTED on merit. (1) Eager restore would flash the stale trio through
the 300 ms fade (the overlay is visible for its duration; rows are
cleared immediately on hide entry, so deferred restore fades an empty,
honest overlay). (2) The theoretical race - a snapshot rendering during
the fade so the queued timeout restores the trio under rows - is
unreachable in shipped flows: `hideProgress` runs exactly when the poll
loop has stopped (ready-done, error, standalone end) and
`startLiveRefresh` never touches the renderer; mid-session overlay
re-open is kickoff-OUT. (3) The needle locks the restore inside the hide
body. Note N1 below is NOT caused by this placement: the error branch
never calls `hideProgress`, so the timeout never runs on that path.

## New findings (numbered; N1 is the blocker)

N1. BLOCKER - invisible error message after any mid-boot failure.
  `crates/ui/assets/api.js:92` (handoff-frozen error branch):
  `updateProgress(100, 'Error: ' + status.progress.message)` writes the
  error text into `loadingDetail`, whose `display:none` the hop-2 row
  path just applied; `updateProgress` (main.js:241-246) does not restore
  visibility and the error branch returns without `hideProgress`, so the
  restore never runs. T2's boot seed makes the row path the FIRST poll's
  path in every server boot, so any mid-boot failure necessarily lands
  with the trio hidden: the user sees frozen step rows forever and NO
  error text. Pre-milestone baseline (`e276910`) showed the error text
  (`updateProgress` wrote to visible elements) - this is a regression on
  the failure surface introduced by the milestone. Root cause is a
  handoff contract edge, not implementer slack: the clause set (hide
  trio under rows + "ready/error branches unchanged" + "legacy
  functions' behavior kept") is unsatisfiable together once the error
  branch bypasses the renderer; one clause must be consciously broken
  and recorded. Fix in either variant (both one-liners, `api.js`/
  `main.js` are in the T3 touch-set):
  - Variant A (preferred): in `updateProgress`, restore the trio's
    visibility (it is the legacy presentation function; making what it
    writes visible completes its original effect). Single site; also
    restores the ready-phase "Fetching graph endpoints..." feedback
    (finding N2) for free.
  - Variant B: in the api.js error branch, `setLegacyFallbackVisible(true);`
    before `updateProgress(100, 'Error: ' + ...)` (optionally also
    before the ready-branch `updateProgress(90, ...)`) - deviates from
    the frozen-branch clause; record the deviation naming the
    unsatisfiable trio.
  Either way: add one source-lock needle in
  `test_ui_loading_overlay_and_progress` proving the error path restores
  visibility before/while writing 'Error: ' (bundle-region contains-check
  in the established R1 idiom), re-run the scoped gate to `0 failed`,
  and journal the Red (extended needle) before the Green.
N2. minor (rides along with the N1 fix) - ready-phase text feedback
  (`updateProgress(90, 'Fetching graph endpoints...')`, api.js:66) is
  likewise written to hidden elements: during the post-ready data fetch
  the overlay shows frozen rows with no visible progress text until
  `hideProgress` fires. Variant A fixes both; Variant B should take the
  second line too. Cosmetic/transient vs N1's stuck-invisible error.

## Gate evidence (reviewer's own fresh runs, hop 2)

- `python tools/testrun.py --distill -p ariadne-ui` (exit 0, capture
  `target/testrun-capture.txt`): `184 passed / 0 failed / 0 ignored /
  3 suites` - matches the journaled Rework Green exactly (needles joined
  the existing test; count unchanged).
- `python tools/hygiene_check.py` - R3 warn-only, none in rework-touched
  files; exit 0 per harness semantics.
- `python tools/encoding_guard.py --all` - silent, exit 0.
- `python tools/theme_token_check.py --check-added --base main` - silent;
  substantive re-run `--base e276910` also silent (rework added no CSS).
- Frozen locks for the rework range `1bc50b8..HEAD`: EMPTY for
  `crates/cli`, `crates/pipeline`, `crates/ui/src`, `themes.css`,
  `theme.js`, `index.html`, `api.js`, `styles.css`, `server_api.rs`,
  `tools/theme_token_baseline.json`, `tests/fixtures`, `tests/golden` -
  confirms the hash-recap claim that api.js/index.html/styles.css were
  untouched this hop.
- Commit-vs-touch-set: `9d96272` = main.js + static_html.rs + T3 journal
  (declared scopes with the correct omissions per hop-1 remediation);
  `fe29dfc` = T2 journal only (authorized carry-over as a separate
  docs-only commit, per remediation step 5); `a17952a` = T3 journal
  only. No Cargo.lock anywhere. Range `name-only` = 4 paths, all legal.
- Hygiene ceilings: `setLegacyFallbackVisible` 9 lines; rework needles are
  pure additions (0 deletions in static_html.rs).

## Gate attempts (hop 2, one each, per orchestrator instruction)

- `python tools/ui_smoke.py` - REFUSED, verbatim: `The user has specified
  a rule which prevents you from using this specific tool call. Here are
  some of the relevant rules {"permission":"bash","pattern":"*","action":
  "deny","source":"global"}`. Matches the implementer's journaled
  refusal this hop. Stays open for a fresh human/orchestrator session.
- `python tools/bench_compare.py` - REFUSED, identical verbatim refusal.
  Not a kickoff gate; stays open.

## Ledger dispositions (hop 2)

REJECTED -> the two pre-authorized deferred-findings rows (a) shape/T2
stage-5 frame-count errata and (b) kickoff G7 `--baseline`
misconfiguration are NOT appended; authorization carries to the approval
hop. No ledger commit this hop. (Implementer-side journal edits and
`fe29dfc` touched no ledger - verified by the empty ledger diff in the
name-only lists.)

## Milestone closeable (hop 2): NO

Remaining open items, exact: (1) fix N1 (+N2) in main.js or api.js with a
source-lock needle and one scoped Green; re-approve T3; (2) on that
APPROVE, the reviewer appends ledger deferred rows (a)+(b) + the G7
census line, committed via commit_scoped (tasks/milestone_ledger.md
only); (3) G3 `python tools/ui_smoke.py` green in a fresh human/
orchestrator session (kickoff acceptance gate; sandbox-refused in both
worker sessions); (4) optional `bench_compare` (not a kickoff gate).

---

# HOP 3 RE-ADJUDICATION (range a17952a..HEAD incl. ledger commit,
2026-10-04)

Commits reviewed, oldest-first: `17e2299` (N1/N2 Variant A fix +
`body_slice` compaction + journal hop-3) and `24ecb70` (journal hash
recap, +1 line, verified). Reviewer's own ledger-only commit `cad6efb`
follows the APPROVE (authorized scope: tasks/milestone_ledger.md,
2 insertions / 0 deletions, per the commit tool's own numstat).

## VERDICT (HOP 3): APPROVED

N1 and N2 are CLOSED at a single restore site; the R2 compaction is
clean (zero pre-existing-line deletions); all gates green fresh. T3 is
approved: handoffs T1 `5eff824`, T2 `587c409`, T3 `17e2299`.

## Verification detail

- N1/N2 fix: `updateProgress` opens with `setLegacyFallbackVisible(true)`
  (5 added lines, main.js). Reviewer's independent caller audit: the
  ONLY callers are api.js ready(66,84)+error(92),
  renderProgressSnapshot's empty-steps legacy branch (main.js:305), and
  graph.js tab-switch (39,46). Every caller now yields visible output;
  the error branch writes 'Error: ...' only after the trio is restored;
  frozen api.js branches untouched.
- Deviation (a) graph.js audit: CONCLUSION ACCEPTED, premise corrected.
  Tab-switch does NOT run in an overlay-hidden context: it opens with
  `showProgress('Switching tab...', 20)` (graph.js:37), which flexes the
  overlay - it IS the legacy trio UX. The conclusion still holds for
  stronger reasons: during boot the opaque full-screen overlay
  (`inset:0; z-index:9999`) blocks all tab interaction, and post-boot
  `hideProgress`'s fade timeout has already restored the trio, so the
  eager restore is idempotent at that site; stale-trio-over-live-rows
  interleaving is unreachable.
- Deviation (b) intermediate DRIFT run (exit 2): ACCEPTED. Own test
  helper compile failure during the self-inflicted compaction; testrun
  correctly refused a census (drift protocol, exit 2, no fabricated
  line), fixed and re-run before commit; final Red (predicted failing
  needle, `183 passed / 1 failed / 0 ignored / 2 suites`) and Green
  intact. Evidence-format note: the intermediate DRIFT run is not
  journaled (grep confirms no DRIFT line in hop-3) - add a one-line
  journal note at next touch.
- Deviation (c) R2 compaction: ACCEPTED, verified. Whole-milestone diff
  `e276910..HEAD` on static_html.rs has ZERO deletion lines - every
  pre-existing needle byte-untouched; only the implementer's own hop-2
  inline slice was replaced by the shared `body_slice` helper
  (strictly safer: searches after the full marker); the needle set is a
  strict superset (all hop-1 required + trio block + new
  updateProgress-restore lock); file 997 lines (<1000); hygiene exit 0
  fresh.
- Brief correction (informational, orchestrator-side): "api.js zero
  hunks vs e276910" is true only for the ready/error branch LINES
  (byte-identical - absent from the whole-milestone diff); api.js
  legitimately carries the 2 hop-1-mandated hunks from `e9cf10d`
  (poll-branch feed replacement + standalone branch), which are the
  handoff's own spec. Proven correct claim: api.js zero hunks
  `e9cf10d..HEAD` (empty diff) - Variant A left api.js untouched as
  intended.
- Frozen locks fresh (reviewer's own): EMPTY diffs - api.js since
  `e9cf10d`; crates/cli since `587c409` (T2); crates/pipeline,
  progress.rs, themes.css, theme.js, theme_token_baseline.json,
  tests/fixtures across `e276910..HEAD`.
- Gate lines (reviewer fresh at `24ecb70`):
  - `python tools/testrun.py --distill -p ariadne-ui` (exit 0):
    `184 passed / 0 failed / 0 ignored / 3 suites`
  - `python tools/hygiene_check.py`: exit 0, R3 warn-only (none in
    touched files)
  - `python tools/encoding_guard.py --all`: silent, exit 0
  - `python tools/theme_token_check.py --check-added --base main`:
    silent, exit 0; substantive `--base e276910` also silent (G4 holds
    at milestone scope through all three hops)
- Ledger: rows (a) frame-count errata and (b) G7 `--baseline`
  misconfiguration appended in `cad6efb` (ledger-only scope).
  kilo-memory mirrored via key `ledger.milestone.ui_progress_parity`.
- Attempt log note: a compound `git show ... | Select-Object` form and
  a bare `git status --porcelain` were refused mid-hop by the catch-all
  rule (the former is also AGENTS.md-prohibited); ground truth came
  from checkpoint_guard at hop start + the commit tool's own numstat;
  no gate skipped for refusal.

## Milestone closeable (hop 3): NO - two items remain, neither in code

1. G3 `python tools/ui_smoke.py` must run GREEN in a fresh
   human/orchestrator session (kickoff acceptance gate; sandbox-refused
   in every agent session this milestone; verbatim refusals in this
   file and in the journal hop-2 section).
2. `python tools/bench_compare.py` (not a kickoff gate; optional local
   perf check) - same sandbox situation; expected no effect (assets-
   only milestone, `crates/pipeline` byte-identical).
Kickoff gates G1/G2/G4/G5/G6 + the G7 census line are met and quoted in
`tasks/ui-progress-parity-execution-results.md`.
