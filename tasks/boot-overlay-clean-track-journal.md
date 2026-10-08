# Boot-overlay clean track - implementer journal

Handoff: quick-mode cosmetic fix (spec = orchestrator prompt; no planner doc).
Date: 2026-10-08. Touch-set: `crates/ui/assets/main.js`,
`crates/ui/assets/styles.css`, `crates/ui/tests/ui_test/static_html.rs`,
this journal.

## What was done

- main.js: `buildStepBarCells` became `buildStepBarNodes` (it returns DOM
  nodes, not a string). The bar is constructed as text node '[', a text node
  of the fill run (`"\u2588".repeat(filled)`, keeps the inherited per-state
  color), a dedicated `<span class="step-bar-remainder">` containing
  `"\u2588".repeat(remainder)`, and text node ']'. U+2591 is removed from
  both code and comment. The caller appends the nodes instead of assigning
  `bar.textContent` a single string.
- styles.css (end of the step-row block, I8 token-only):
  `.step-bar-remainder { color: var(--progress-track); text-shadow: none; }`.
  `text-shadow: none` is required because text-shadow inherits and
  `.step-active .step-bar` carries the cyan glow; the rule targets the span
  directly, so no `.step-active .step-bar` / `.step-done .step-bar` color
  (which matches the parent) can win for the child. Pending rows keep the
  0.45 row opacity: the rgba track composites to roughly 0.09 alpha over the
  canvas - quiet but visible, which the brief pre-accepted.
- static_html.rs (intended spec change): I7 arithmetic lock
  (`Math.floor(20 * percent / 100)`) untouched in code and test. Glyph lock
  is now `contains("\\u2588") && !contains("\\u2591")`; new shape lock:
  `body_slice("function buildStepBarNodes")` contains both
  `"step-bar-remainder"` and `"createElement('span')"` (JS-scoped, fails if
  someone reverts to one flat textContent string). I8 CSS block check
  untouched and passing (no '#', has 'var(--').

## TDD trail

- Red (locks updated, before JS/CSS change):
  `### cargo test FAIL (exit 1)` / `208 passed / 1 failed / 0 ignored / 2 suites`
  panic at static_html.rs:562 - `assertion failed: !html.contains("\\u2591")`
  - the exact failure the handoff predicted.
- Green and re-run after refactor (condensed lock block):
  `209 passed / 0 failed / 0 ignored / 3 suites` - no failures, no
  regressions. (Suite count shows 2 in the red run because the panic abort
  ends that binary's listing early.)

## Gates (digests)

- `python tools/testrun.py --distill -p ariadne-ui` ->
  `209 passed / 0 failed / 0 ignored / 3 suites`
- `python tools/hygiene_check.py` -> R1/R2 clean (no R1 or R2 rows at all);
  R3 warn list contains only pre-existing rows in files outside the
  touch-set; none introduced here.
- `python tools/theme_token_check.py --check-added --base HEAD` -> no
  output (zero hex literals added; the CSS rule is token-driven).
- `python tools/encoding_guard.py --staged` -> pending, digest appended
  below before commit.
- CLI byte format (DP-1): `git diff HEAD -- crates/cli` via distiller ->
  empty capture (0 lines). Frozen files untouched.
- Acceptance greps: `u2591|░` zero hits in main.js; `step-bar-remainder`
  present in main.js, styles.css, and static_html.rs.
- No byte-format snapshot broke: the full scoped suite shows no snapshot
  diffs. No fixtures, baselines, or theme token files were touched.

## Deviations (with reasons)

1. Testrun invocation: the brief's literal
   `python tools/testrun.py --distill cargo test -p ariadne-ui` produced
   `parse format drift` (exit 2): testrun.py prepends `cargo test` itself,
   so the literal extra words became stray positional args. Ran the
   documented contract form `python tools/testrun.py --distill
   -p ariadne-ui` (same scope, same suite).
2. R2 ceiling on static_html.rs: the file sat at 997 lines; my first
   (verbose) Red-phase block reached 1008 and R2 is blocking. Extracting a
   module would create a file outside the hard touch-set (review failure),
   so I condensed the glyph/track lock block to six content lines while
   keeping every required assertion. Final file: 999 lines. Flag for the
   orchestrator: this file is at the ceiling; a future split deserves its
   own handoff.
3. The standalone `html.contains("step-bar-remainder")` assert was folded
   into the `body_slice` shape lock (slice membership implies html-level
   presence, and the slice check is strictly stronger and JS-scoped).
   Acceptance rg still hits all three files.
4. Builder renamed `buildStepBarCells` -> `buildStepBarNodes` to match the
   returned shape; the old name had no other references (verified by grep),
   and the new name is referenced only by the test's shape lock.
5. A standalone `html.contains("step-bar-remainder")`-only assertion on the
   whole bundle was dropped as redundant (see 3); the brief allowed shaping
   the durable lock to the authored shape.

## Deliberately not done

- Did not touch `crates/cli/**` (DP-1 byte format), `themes.css`
  (`--progress-track` already defined in all four theme sections), any other
  test, fixture, or baseline.
- Did not build new test infrastructure: no jsdom-style behavior harness for
  this renderer existed; the brief prescribes source locks only.
- Did not change the 0.45 pending-row row-opacity behavior.

## Encoding guard digest (staged, pre-commit)

`python tools/encoding_guard.py --staged` -> silent, exit 0 (the tool
prints one line per violation and returns 1 on any; no output = zero
mojibake/non-UTF-8 violations across the four staged touch-set blobs).
Staged set held only this handoff's four files (foreign worktree entries
were unstaged `M` at session start, verified by the initial status census).
