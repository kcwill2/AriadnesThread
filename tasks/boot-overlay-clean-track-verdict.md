# Verdict - boot-overlay clean track (quick-mode cosmetic fix)

- Handoff: replace dithered U+2591 step-bar remainder with solid cells in
  `<span class="step-bar-remainder">`; CLI `format_step_bar` byte-frozen (DP-1).
- Commit under review: `c00576d` (sole commit in `cdf3f4c..HEAD`), branch main.
- Reviewer session: fresh; all gates re-run, no journal claim trusted.

## VERDICT: APPROVED

## Per-commit review (oldest-first)

One commit in range. `git show "c00576d"`: main.js +15/-6, styles.css +9/-0,
static_html.rs +6/-4, journal +103. Judge it alone:

- main.js:290-302 `buildStepBarNodes(percent)`: clamp `Math.min(percent, 100)`,
  fill `Math.floor(20 * percent / 100)` (I7 literal intact), returns nodes
  `[` text, fill text node `"\u2588".repeat(filled)` (inherits the per-state
  `.step-bar` color), remainder `<span class="step-bar-remainder">` with
  `"\u2588".repeat(remainder)`, `]` text. Bar keeps `[` + 20 cells + `]`;
  caller (main.js:334) appends nodes instead of one flat textContent string.
- styles.css:1798-1806: rule appended at the end of the step-row block
  (same top-level block as `.step-row` at 1741; no `@media` wrapper - the
  4-space indent is the file's tail convention). Body is verbatim the
  prescribed `color: var(--progress-track); text-shadow: none;` - token-only,
  no `#`. `--progress-track` is declared in all four theme blocks
  (themes.css:111, 243, 383, 521).

## Commit-vs-touch-set

`git diff --name-only cdf3f4c..HEAD` returned exactly the four declared paths
(main.js, styles.css, static_html.rs, journal). No path outside the touch-set.
PASS.

## Plan-literal checks

- CSS rule matches the prescribed literal verbatim (styles.css:1804-1805).
- I7 arithmetic lock `Math.floor(20 * percent / 100)` intact in code
  (main.js:292) and in the test lock (static_html.rs:554).
- I8 CSS token-only check untouched (static_html.rs:565-567) and passing;
  `theme_contract::test_styles_css_carries_no_color_literals` passes in the
  fresh 209-suite run.
- ASCII purity: main.js gained zero non-ASCII bytes. All 13 non-ASCII lines
  (lines 28-410, em dashes / middle dots) are pre-existing and outside both
  diff hunks. Zero `\\u2591` and zero raw block glyphs remain in main.js;
  `\\u2588` appears only as escapes (comment at 286, code at 295, 298).
- DP-1 CLI freeze: `git diff cdf3f4c HEAD -- crates/cli` empty AND
  `git diff HEAD -- crates/cli` empty (worktree). `format_step_bar` untouched.

## CSS cascade audit (the named risk)

- `.step-active .step-bar` / `.step-done .step-bar` (styles.css:1787-1796,
  specificity 0,2,0) match the parent `.step-bar` span only; nothing matches
  the remainder span except `.step-bar-remainder` itself. A direct
  declaration always wins over an inherited value regardless of the
  inheriting rule's specificity, so track color and `text-shadow: none` hold
  on active and done rows. The journal's reasoning is correct on the merits.
- Exhaustive selector sweep: every `.step-*` descendant rule was enumerated
  (labels/details/bar/percent only); the two universal `*` blocks
  (styles.css:7, 31) set only scrollbar and box-model properties, no color or
  text-shadow. Nothing can paint the remainder span.
- Pending rows keep `.step-pending { opacity: 0.45 }` (row-level, unchanged;
  pre-accepted by the brief).

## Deviation dispositions

1. Gate command form (`--distill -p ariadne-ui` instead of the brief's
   literal `--distill cargo test -p ariadne-ui`): ACCEPTED. The contract form
   was run fresh by the reviewer and works (see gate evidence). The brief's
   wording is the defect, not the deviation. Follow-up (below).
2. Condensed lock block to stay under R2: ACCEPTED. Final
   static_html.rs = 999 lines (R2 ceiling is < 1000); `hygiene_check.py`
   emitted no R1/R2 rows. All five required assertions verified present:
   `\\u2588` presence + `\\u2591` absence (static_html.rs:559),
   `step-bar-remainder` and `createElement('span')` shape lock on
   `body_slice("function buildStepBarNodes")` (560-562), I7 formula (554),
   I8 var()-only block (565-567). `body_slice` (524-529) panics if the
   builder is renamed or removed and slices to the next `function`
   definition, so the JS-scoped slice lock is strictly stronger than the
   dropped html-level `contains` (slice membership implies html membership)
   - the implementer's claim holds. Reverting to a flat textContent string
   would fail the slice assertion. Module extraction would indeed have
   broken the hard touch-set; condensing is the correct call.
3. `buildStepBarCells` -> `buildStepBarNodes` rename: ACCEPTED. Grep across
   the repo: zero live references to the old name (only historical
   tasks/*.md records and the journal); the new name is referenced exactly
   at main.js:290 (definition), main.js:334 (caller), static_html.rs:560
   (test lock). The lock would panic on a rename.

No undeclared deviations found.

## Findings

1. MINOR (environment, not this commit): `python tools/testrun.py` full
   workspace run crashes at `tools/testrun.py:262` (`_echo` ->
   `sys.stdout.write`) with `UnicodeEncodeError: 'charmap' codec can't
   encode characters in position 3992-4011` under a cp1252 console; tests
   themselves completed fine and the capture file was written. Workaround:
   `$env:PYTHONIOENCODING = "utf-8"`. Worth a one-line fix (reconfigure
   stdout) in a future tooling handoff.
2. MINOR (configuration): the literal gate `python tools/ui_smoke.py` was
   refused by the bash sandbox (verbatim refusal recorded in
   execution-results). The gate was still run through the documented
   distilling entry point and PASSED. Reviewer/orchestrator allowlist could
   name `python tools/ui_smoke.py` directly.
3. MINOR (journal evidence): the journal quotes digest verdict lines and
   exit codes but not capture paths (distillation contract prefers them);
   the path is fixed (`target/testrun-capture.txt`), so evidence remains
   recoverable. Non-blocking.

No blockers. Findings 1-3 do not gate this commit.

## Gate evidence (fresh runs, this session)

- Scoped suite: `python tools/testrun.py --distill -p ariadne-ui` ->
  `### cargo test PASS (exit 0)` / `209 passed / 0 failed / 0 ignored / 3 suites`
- Full workspace: `1255 passed / 0 failed / 13 ignored / 91 suites`
  (with `PYTHONIOENCODING=utf-8`; see finding 1)
- Tool suites: `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 219 tests in 19.082s` / `OK`
- `python tools/hygiene_check.py`: exit 0; R3 warn rows only, none in any
  touched file (main.js/styles.css absent; static_html.rs absent from list)
- `python tools/encoding_guard.py --all`: silent (zero violations)
- `python tools/theme_token_check.py --check-added --base HEAD` and
  `--base cdf3f4c`: both silent (zero hex literals added, including by the
  commit itself)
- UI smoke: `python tools/distill.py -- python tools/ui_smoke.py` ->
  `### generic PASS (exit 0)` / `boot+route checks clean (cold fixture,
  attempt 1, 1.0s)` / `ui_smoke: PASS`
- CLI DP-1: both CLI diffs empty (above).

## Follow-up dispositions (for tasks/milestone_ledger.md deferred-findings)

- F1: static_html.rs sits at 999/1000 lines; a module split deserves its own
  handoff (implementer flagged; endorsed).
- F2: handoff wording should use the contract form
  `testrun.py --distill -p <crate>` (testrun prepends `cargo test` itself).
- F3: testrun `_echo` stdout-encoding crash on cp1252 consoles
  (tools/testrun.py:262); reconfigure stdout to UTF-8.
- F4: allowlist `python tools/ui_smoke.py` for reviewer sessions.
