# T-A Handoff - ui-extraction-card-clarity: contrast oracle + token value fixes

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ui-extraction-card-clarity-shape.md` sections 1 (I2, I4, I9, I13),
   2 (C4 pair table verbatim, C5 named muddy pairs, C6 fix list).
3. `tasks/ui-extraction-card-clarity-kickoff.md` sections "Frozen locks" and
   acceptance gate G1/G4 (context only; this file is the hand contract).
4. Model file to mirror in structure and helper idiom:
   `crates/ui/tests/ui_test/theme_dark_grey_palette.rs` (comment-stripping,
   block-body, declared-value, hex-run, WCAG luminance + ratio helpers are
   copied per-file by house idiom - do not import across test modules).
5. Registry authority: `crates/ui/assets/theme.js` (`THEMES` array, parsed,
   never hardcoded); token authority: `crates/ui/assets/themes.css`.

## Objective and preserved invariants

Land a deterministic four-theme contrast oracle for the extraction-section
token pairs, then apply the minimal `themes.css` value fixes it demands.
Invariants this hand must preserve:
- I2 body ink: ratio(--text-main, --bg-card) >= 7.0; ratio(--text-muted,
  --bg-card / --bg-sub / --bg-surface / --bg-main) >= 4.5, every theme.
- I4 mass floor: ratio(--bg-card, --bg-surface) >= 1.12;
  ratio(--border-light, --bg-card) >= 1.35;
  ratio(--border-light, --bg-surface) >= 1.50, every theme.
- I9 hex discipline: zero new hex literals; `themes.css` edits change VALUES
  only - the literal count per theme block is unchanged, so
  `tools/theme_token_baseline.json` stays diff-empty.
- I13 four-theme coverage: the oracle iterates the parsed `THEMES` registry
  from theme.js; a fifth theme is forced into the pair table automatically;
  a registry id without a themes.css block panics the test (missing pair row
  is a test bug by construction - the table loops the registry).
- Verdict hues (--green/--amber/--red) may be retinted for CONTRAST ONLY,
  never style (themes.css D6 rule): each changed value must name its red
  oracle pair row as justification in the journal.
- Grey/steampunk value edits are presumptively wrong: C6 expects an empty fix
  list for dark-grey/dark-steampunk beyond what the oracle proves; the grey
  achromatic (spread <= 4) and semantic-byte-equality locks stay UNEDITED and
  will trip any careless retint.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/ui-extraction-card-clarity-ta-journal.md (checkpoint journal; create
  it; FIRST entry records the start rev `git rev-parse HEAD` and the baseline
  scoped-ui test line)
- crates/ui/tests/ui_test/extraction_contrast.rs (new; the oracle)
- crates/ui/tests/ui_test/main.rs (+1 mod line ONLY: insert
  `mod extraction_contrast;` immediately AFTER `mod extraction_card_story;`,
  BEFORE `mod file_view;` - true alphabetical site; do NOT touch the known
  `extraction_card_playbook`-before-`export_order` mod-order erratum region)
- crates/ui/assets/themes.css (value edits only, driven by the oracle)

You MUST NOT touch: tools/theme_token_baseline.json, theme.js, any other file
under crates/ui/assets/, any other test file, crates/*/src/**, tools/**,
tests/fixtures/**. No `git stash`, no branch change.

## What to build

`extraction_contrast.rs` (~180-260 lines, pure std, no new deps):
- Copy the helper idiom from theme_dark_grey_palette.rs: strip comments,
  extract a theme block body by registry id, read a declared token value,
  collect hex runs, compute WCAG relative luminance and ratio.
- Parse `THEMES` from theme.js (the registry list of ids); parse each theme
  block from themes.css. Composite rule: translucent declarations composite
  over --bg-surface per channel (`a*c + (1-a)*under`); opaque values pass
  through (C4 convention, state it in a doc comment).
- Pair table exactly per shape C4 (9 rows): --text-main on --bg-card/--bg-sub
  floor 7.0; --text-muted on --bg-card/--bg-sub/--bg-surface/--bg-main floor
  4.5; --text-sub on --bg-main floor 3.0; --amber on --bg-card floor 4.5;
  --green on --bg-card floor 4.5; --red-text on --bg-card floor 4.5;
  ratio(--bg-card, --bg-surface) floor 1.12; ratio(--border-light,
  --bg-card) floor 1.35; ratio(--border-light, --bg-surface) floor 1.50.
- Required tests (each failure message names theme, pair, measured ratio,
  floor): `test_registry_ids_all_have_theme_blocks` (panics per I13),
  `test_body_ink_floors`, `test_offcard_ink_floor`,
  `test_verdict_hues_on_card`, `test_mass_separation`,
  `test_border_separation`, `test_pair_table_loops_parsed_registry` (asserts
  the evaluated row count equals registry-size x 9; proves no hardcoding).

Then the themes.css fixes - oracle-driven, minimal:
- Expected (hand-computed, NOT binding - the oracle decides): deepen light
  `--green` (C6 candidate `#166534`, hue preserved); lighten dark
  `--border-light` (current `#334155` measures ~1.2 against dark --bg-card,
  floor 1.35; choose the nearest value clearing the floor with >= 5% margin,
  hue preserved). Expected empty for grey and steampunk.
- Every changed value is justified in the journal by its red pair row.

## RGB protocol (in this file order)
1. Baseline: run gate 1 below BEFORE any edit; journal the verbatim scoped-ui
   test line (delta accounting - this repo reports deltas, never hardcoded
   totals).
2. Red: add extraction_contrast.rs + mod line; run scoped ui. The oracle
   compiles cleanly and fails on below-floor assertions. A test that fails to
   compile is NOT Red. Journal quotes the failing pair lines verbatim (that
   table supersedes the shape's hand-computed C5 table).
   STOP-AND-REPORT case: if the oracle comes back all-green before any
   themes.css edit, the milestone premise is falsified or the parser is
   wrong - report, do not invent fixes.
3. Green: apply the value edits; re-run scoped ui to exit 0. NEVER weaken an
   assertion or floor to pass.
4. Refactor: honest helper names, doc comment on the composite convention;
   re-run.

## Frozen locks (with proving command)
- `python tools/theme_token_check.py --check-added --base <start-rev>` silent
  (counts literals; value-only edits must stay silent). If it flags the
  changed lines, STOP and report - do NOT regenerate the baseline.
- `git diff <start-rev> -- tools/theme_token_baseline.json` empty.
- UNEDITED and green (they appear in the scoped run): theme_contract.rs,
  theme_js_zero_hex.rs, theme_dark_grey_palette.rs (incl. grey achromatic
  spread <= 4 and grey semantic-byte-equality-to-default locks).
- `git diff <start-rev> -- crates/*/src` empty (bench-exemption proof).
- tests/fixtures/** frozen; no path under tools/** touched.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only)
1. `python tools/testrun.py --distill -p ariadne-ui` (baseline; again at Red,
   then Green - canonical line `N passed / 0 failed / ...` must appear
   verbatim in the report; Green N = baseline N + 7)
2. `python tools/theme_token_check.py --check-added --base <start-rev>` (silent)
3. `python tools/hygiene_check.py` (exit 0)
4. `python tools/encoding_guard.py --staged` (exit 0 before committing)
5. `python tools/commit_scoped.py --scope crates/ui/tests/ui_test/extraction_contrast.rs --scope crates/ui/tests/ui_test/main.rs --scope crates/ui/assets/themes.css --scope tasks/ui-extraction-card-clarity-ta-journal.md -m "test(ui): four-theme contrast oracle for extraction token pairs + oracle-driven value fixes"`
   (single landing commit, house precedent; never `git add .`)
~7 invocations with one Green re-run allowance. Any further overshoot is
declared as a dated D-row in the journal (standing ledger law, M28 T0b
finding 1).

## Hygiene (review-enforced)
Functions < 100 lines, files < 1000 lines, no banned names
(tmp/res/foo/bar/test1), no single-letter locals, one responsibility per
helper; `python tools/hygiene_check.py` exit 0. Ceiling at risk: the ratio
helpers copied from the grey palette model - keep each comparison row's
evaluation in one small function, no monolith.

## Discipline clauses
- UTF-8: Write/Edit tools only; pure-ASCII content (no em dashes or section
  signs); never Set-Content/Out-File/redirect/Get-Content-replace (AGENTS.md
  File-Edit Discipline).
- Touch only the touch-set; out-of-scope breakage: report, don't fix.
- Raw `cargo test`/`build`/`clippy` in this session is a reviewer blocker -
  distilling entry points only; full capture path from the digest
  (`target/testrun-capture.txt`) is cited, never pasted.
- After EACH Green step, append a 1-3 line checkpoint to the journal
  (module + commit hash + deviations). If context exhaustion looms: STOP and
  return a disk-state inventory; successor boots via
  `python tools/checkpoint_guard.py resume --spec tasks/ui-extraction-card-clarity-ta-handoff.md`.

## Acceptance (reviewer runs; verdict lines expected)
- `python tools/testrun.py --distill -p ariadne-ui` exit 0, delta +7 vs the
  journal-recorded baseline; zero existing test amended (this hand adds only).
- `python tools/theme_token_check.py --check-added --base <start-rev>` silent;
  `git diff <start-rev> -- tools/theme_token_baseline.json` empty.
- Per-commit touch-set enforcement via review-gate (commit_scoped set
  equality); themes.css diff contains value changes ONLY (same literal count
  per theme block; no added/removed declaration lines).
- `python tools/hygiene_check.py` + `python tools/encoding_guard.py --all`
  exit 0.
- Bench: EXEMPT with proof (empty `crates/*/src` diff, standing precedent).
- Reviewer gate reference: adjudicated via the `review-gate` skill (fresh
  full verification, per-commit git show oldest-first, commit-vs-touch-set,
  plan-literal check against this file + shape C4/C6).

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence: the verbatim failing
pair lines from the first oracle run (this is kickoff G1's machine evidence).
3. Green evidence: distill digest header + canonical line + capture path.
4. Each themes.css value change with its justifying pair row. 5. Any
deviation + reason. 6. Out-of-scope observations (report-only).
Never paste raw tool output; cite command + canonical line + exit code.
