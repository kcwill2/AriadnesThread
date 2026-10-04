# T1 Handoff - ui-dark-grey-theme: register the fourth theme `dark-grey`

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. Shape (source of truth for intent/invariants): `.kilo/plans/1791105717628-ui-dark-grey-theme-shape.md`
   - read fully; invariants I1-I11 and Contracts are binding.
3. `crates/ui/assets/themes.css` lines 1-38 (D6 semantic/decorative header - read, do not edit),
   lines 40-166 (default/dark block - the verbatim source for the semantic layer),
   lines 298-311 (the insertion point: after light's closing `}`, before the steampunk comment).
4. `crates/ui/assets/theme.js` lines 15-17 (registry + defaults).
5. `crates/ui/assets/index.html` lines 8-20 (anti-flash bootstrap), 195-204 (#themeSelect group),
   471 (Help "Theme:" paragraph).
6. `crates/ui/tests/ui_test/theme_contract.rs` lines 89-122 (export test to extend),
   lines 178-243 (parser helper idioms to COPY locally into the new test file).
7. `crates/ui/tests/ui_test/main.rs` lines 18-19 (mod registration site).
8. `tools/theme_token_check.py` lines 37-40 and 136-147 (TOKEN_FILES exemption; frozen-count semantics).
9. `docs/specs/tool-output-distillation.md` (distilling entry points - which commands you may run).

User-confirmed decisions (recorded so zero verbal briefing is needed; all four are settled):
- Theme id `dark-grey`, display label `Dark (Grey)`. The id is PERMANENT localStorage state
  (key `ariadne_theme`) once shipped - spell it exactly, never `dark-gray`.
- Keep the sky-cyan decorative accent (default block's `--cyan*`, `--brand*`, `--primary-btn*`,
  `--tag-pill*`, `--tour-glow` values copied verbatim into the grey block).
- Keep the default saturated node-kind card tints for v1: `--node-header/source/class/function/other-*`
  and the proposed-hypothetical family `--preview-hyp-bg`, `--preview-hyp-bg-active`,
  `--preview-proposed-bg/text/stroke` copy the DEFAULT block values verbatim.
- Registry/dropdown/block order: `light, dark, dark-grey, dark-steampunk` - grey adjacent to Dark.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/ui-dark-grey-theme-t1-journal.md` (checkpoint journal; always in-set)
- `crates/ui/assets/themes.css` (append ONE new block at the insertion point above)
- `crates/ui/assets/theme.js` (exactly the `THEMES` array literal, 1 line)
- `crates/ui/assets/index.html` (bootstrap comment sentence + validator line, one `<option>`,
  Help "Theme:" paragraph)
- `crates/ui/tests/ui_test/theme_contract.rs` (extend the export assertion at lines 104-107
  and its message wording ONLY; everything else in this file byte-identical, including the
  "(3-way parity)" wording at line 375 - stale-but-out-of-scope, report it, do not fix)
- `crates/ui/tests/ui_test/theme_dark_grey_palette.rs` (NEW file, the four grey oracles)
- `crates/ui/tests/ui_test/main.rs` (+1 `mod theme_dark_grey_palette;` line, alphabetical:
  after `mod theme_contract;`)
- `tools/theme_token_baseline.json` (regenerated via `--update-baseline`, SAME commit as the
  themes.css block - never in a separate commit)

You MUST NOT touch: `crates/ui/assets/styles.css`, every other file under
`crates/ui/assets/` (api.js, graph.js, main.js, preview.js, hotspots.js, provenance.js,
file-view.js, theme_js consumers), anything under `crates/ui/src/**` (surveyed: no Rust site
enumerates theme ids; if you find one that fails, that is a FINDING TO ESCALATE, not to absorb),
`crates/ui/tests/ui_test/common.rs` and all other test files, `tools/theme_token_check.py`,
`tools/ui_smoke.py`, `tests/fixtures/**`, `crates/pipeline/**`,
`crates/cli/src/commands/progress.rs`, `Cargo.toml`/`Cargo.lock` (no dependency changes;
if cargo regenerates the lock, report, do not commit it). No `git stash`, no branch change.

## What to build

Make `dark-grey` a full fourth theme across the four mirror surfaces (registry, select,
CSS block, bootstrap). Pure token definition: zero rule changes, zero Rust changes.

Exact literals (shape Contracts, verbatim):

1. theme.js line 15 becomes exactly:

       const THEMES = ['light', 'dark', 'dark-grey', 'dark-steampunk'];

   `DEFAULT_THEME`, `THEME_STORAGE_KEY`, `resolveTheme`, `applyTheme`, `readStoredTheme`,
   `initThemeControl` byte-identical. No new JS. The zero-hex lock on theme.js
   (`theme_js_zero_hex.rs`) must keep passing - ids contain no `#`.

2. index.html bootstrap validator line (keep it ONE line, `&&` chain intact, never rewrap):

       if (bootTheme !== 'light' && bootTheme !== 'dark' && bootTheme !== 'dark-grey' && bootTheme !== 'dark-steampunk') bootTheme = 'dark';

   The leading comment sentence "the three accepted ids mirror ..." becomes
   "the four accepted ids mirror ..." and keeps its mirror-warning sentence (reword
   "adding a fourth theme there" to "adding another theme there" so it stays honest).
   Rest of the head untouched (the new lines carry no colors - I9 sweep covers them).

3. index.html select, between the `dark` and `dark-steampunk` options:

       <option value="dark-grey">Dark (Grey)</option>

   `dark` keeps `selected`. The option-comment stays accurate (it points at the
   test-locked ids, not a count).

4. themes.css: ONE new block `:root[data-theme="dark-grey"] { ... }` inserted after the
   light block's closing `}` and BEFORE the steampunk header comment. It declares exactly
   the default block's token names (~106) in the default block's order and grouping, with the
   same comment rhythm. Per-token value rules:
   - SEMANTIC layer - copy the DEFAULT block's value strings VERBATIM (byte-identical,
     including alpha digits and spacing): the I6 token list in "Required tests" below.
     This theme retints the table, not the food: verdicts, heat ramp, cycle red, preview
     verdicts keep their exact hues (stronger than steampunk's comment-only guarantee).
   - DECORATIVE accents - copy the DEFAULT (dark) block's values verbatim for the
     `--cyan*` family, `--brand-*`, `--primary-btn*`, `--tag-pill-*`, `--tour-glow`
     (user decision: keep sky-cyan).
   - NODE-KIND identities + proposed family - copy DEFAULT values verbatim (user decision Q3).
   - CHROME (the I7 list in Required tests) - new pure-neutral ramp you author under the
     bounds: near-zero hue (zinc/neutral direction, not warm greige, not cool graphite);
     `--bg-main` around #141414 with visible card/border steps; `--text-main` around #f5f5f5;
     muted/sub step down through neutral greys while holding the I8 floors. Every hex the
     chrome tokens declare must satisfy `max(r,g,b) - min(r,g,b) <= 4`; every `rgba()` the
     chrome tokens declare must have `r == g == b` (so `--overlay-panel-bg`, `--tile-bg`,
     `--progress-track`, `--grid-dot`, shadows, scrims are neutral greys, not slate).
   - The block's header comment states the D6 discipline (chrome-only retint; semantic hues
     are information). All NEW lines pure ASCII.

5. index.html Help "Theme:" paragraph: list Light / Dark / Dark (Grey) / Dark (Steampunk);
   "keep their meaning in all four themes". Prose is deliberately NOT test-locked (drift-
   tolerant by the select test's own comment) - it is a reviewer checklist item.

6. `tools/theme_token_baseline.json`: regenerate with `python tools/theme_token_check.py
   --update-baseline` AFTER the grey block exists, in the SAME commit as the block
   (themes.css count rises 211 -> ~281; only the `themes.css` count and its `locations`
   entries may change). The commit message carries the justification sentence:
   "dark-grey theme block adds one hex per hex-declared token, mirroring the default
   block's structure; zero literals added to any rule file".

## Sequencing precondition (verify before touching anything)

This handoff lands AFTER `ui-progress-parity` T3 merged. At authoring the precondition is MET:
the ledger's `ui-progress-parity` row is CLOSED 2026-10-04, all three handoffs gate-APPROVED
(T3 `17e2299`), G1-G7 met, and `themes.css`/`theme.js`/baseline were held untouched through
that run (G4 zero-added-hex was silent) - so those three files are released to THIS milestone.
Caveat: the planner's asset line numbers were read from a pre-T3 tree snapshot, so the line
ranges in this file may have shifted in the merged tree - edit by the exact content anchors
quoted above, never by line number alone.
FIRST action at session start: `python tools/checkpoint_guard.py status`, then verify the
starting tree actually carries T3's changes (e.g. `crates/ui/src/static_html.rs` present /
T3 step-row markup in index.html). If they are not visible (stale branch or worktree), STOP
and report - do not start from a pre-T3 tree. If any content anchor quoted in this file is
not found verbatim (a T3 rework reworded it), STOP and report - do not improvise relocation.

## RGB protocol (in this file order)

1. Red: author `theme_dark_grey_palette.rs` (all four tests below) + the `main.rs` mod line
   + the extended export assertion in `theme_contract.rs` FIRST. Run gate 1. Expected: the
   suite COMPILES and exactly 5 named tests fail (assertion/panic failures, NOT compile
   errors). Capture the failing test list in the journal. NEVER a todo!()/skeleton as "red".
2. Green: add the themes.css block, then theme.js, then index.html (one pass - partial
   application breaks the I1-I3 mirror tests mid-run), regenerate the baseline, re-run
   until green. NEVER weaken an assertion to pass.
3. Refactor: dedupe test helpers, honest names, doc comments; re-run.

## Required tests

`theme_contract.rs` (extension, Red-compatible): `test_standalone_export_inlines_theme_tokens`
gains `content.contains("[data-theme=\"dark-grey\"]")` beside the steampunk assertion, and the
message wording updates from "all three theme blocks" to "all four theme blocks". Option
presence rides the existing export-markup assertions - no new export test.

New `theme_dark_grey_palette.rs` (four tests; copy the private-helper idiom from
theme_contract.rs `parse_js_string_array`/`css_block_body`/`declared_tokens`/`data_theme_ids`
LOCALLY - do not refactor common.rs or theme_contract.rs):

1. `test_dark_grey_block_sits_in_mirror_order` (I5): on comment-stripped THEMES_CSS, the byte
   offsets of `[data-theme="light"]` < `[data-theme="dark-grey"]` < `[data-theme="dark-steampunk"]`,
   and `THEMES[2] == "dark-grey"`. RED failure: panic "themes.css must contain a block for
   [data-theme=\"dark-grey\"]" (or the positional assert).
2. `test_dark_grey_semantic_layer_equals_default_block` (I6): per-token byte-string compare
   (trimmed value) of grey vs DEFAULT block. Names enumerated in the test:
   `--green --amber --red --green-glow --red-wash --red-wash-soft --red-wash-hover --red-glow
   --red-glow-strong --red-text --amber-wash --amber-border --amber-glow --amber-text
   --heat-ramp-a --heat-0-bg/-border/-text --heat-1-bg/-border/-text --heat-2-bg/-border/-text
   --heat-3-bg/-border/-text --heat-4-bg/-border/-text --node-cycle-bg --node-cycle-text
   --node-cycle-border --preview-eliminated-bg/-text/-glow --preview-kept-bg/-text/-glow
   --preview-still-bg --preview-still-text --preview-amber-edge --preview-amber-text
   --preview-amber-glow --preview-amber-outline`. RED failure: missing grey block panic.
   Copy default values verbatim in Green (watch: default writes `0.1` where steampunk writes
   `0.10` - byte compare is exact).
3. `test_dark_grey_chrome_is_achromatic` (I7): for every token in the chrome list
   `--bg-main --bg-canvas --bg-surface --bg-sub --bg-card --bg-card-hover --border
   --border-light --text-main --text-muted --text-sub --text-strong --on-accent --grid-dot
   --shadow-soft --shadow-node --shadow-ambient --shadow-lift --overlay-scrim --tour-scrim
   --overlay-panel-bg --tile-bg --progress-track --edge-neutral --edge-incomplete --code-text`:
   each hex it declares (parse 3/4/6/8-digit forms) has `max(r,g,b) - min(r,g,b) <= 4`; each
   `rgba()` it declares has `r == g == b` (alpha exempt). Exempt from the check (hue allowed):
   `--cyan*`, `--brand*`, `--primary-btn*`, `--tag-pill*`, `--tour-glow`, node-kind and
   proposed/violet tokens. Distinguishability corollary in the same test: the DEFAULT block's
   `--bg-main` has `b - r >= 8` (today 0x17-0x0b = 12) - characterization lock for the slate
   tint this theme exists to differ from. RED failure: missing grey block panic.
4. `test_dark_grey_contrast_floors` (I8): WCAG contrast from parsed hex - relative luminance
   linearizes sRGB per channel (`c/255`; `c <= 0.04045 ? c/12.92 : ((c+0.055)/1.055)^2.4`;
   `L = 0.2126R + 0.7152G + 0.0722B`), `contrast = (L_light + 0.05) / (L_dark + 0.05)`.
   Floors on the grey block: `--text-main` vs `--bg-main`/`--bg-canvas`/`--bg-surface`/`--bg-card`
   each >= 7.0; `--text-muted` vs `--bg-main` >= 4.5; `--text-sub` vs `--bg-main` >= 3.0.
   First numeric legibility lock in the repo (grey-only to start). RED failure: missing grey
   block panic.

Existing oracles that must keep passing with NO edit (they auto-detect the new block):
`test_theme_select_options_mirror_the_theme_registry`,
`test_css_theme_ids_and_registry_agree_both_ways`,
`test_bootstrap_theme_ids_mirror_the_theme_registry`,
`test_every_theme_block_declares_the_full_token_set` (I4 - additions rejected as hard as
omissions: declare exactly the default token set), `test_token_definitions_live_only_in_themes_css`
(grey between light and steampunk keeps its positional assert true),
`test_styles_css_carries_no_color_literals`, `test_index_html_styling_carries_no_color_literals`,
`test_index_markup_has_no_inline_style_or_script_bodies`, `theme_js_zero_hex.rs` suite,
I10 degradation semantics (validators still resolve unknown ids to `dark`; `dark` keeps
`selected`). Do not "fix" any of these - if one fails, your surface edit is incomplete.

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines (themes.css 437 -> ~565, ceiling safe;
theme_contract.rs stays ~485 because grey assertions live in the new file, ~140 lines there),
no banned names (tmp/res/foo/bar/test1), no single-letter locals, intention-revealing names,
one responsibility per function; test helpers named like `hex_channel`, `relative_luminance`,
`contrast_ratio`. `python tools/hygiene_check.py` must stay exit 0. No new line anywhere near
2000 chars; the bootstrap validator line (~150 chars) must stay ONE line.

## Frozen locks

| Lock | Proof command |
|---|---|
| `styles.css` + every non-theme asset byte-identical; zero color literals outside themes.css | reviewer touch-set check + `test_styles_css_carries_no_color_literals` / `test_index_html_styling_carries_no_color_literals` green |
| `tools/theme_token_baseline.json` hex snapshot: regenerate ONLY in the themes.css commit with the justification sentence | `python tools/theme_token_check.py --check-added --base <parent-rev>` exit 0 (themes.css is TOKEN_FILES-exempt from the added scan; the frozen COUNT scan is what the regeneration satisfies) |
| `tests/fixtures/**` frozen; `crates/pipeline/**` + `crates/cli/src/commands/progress.rs` byte-identical | reviewer touch-set check (files must not appear in the diff) |
| `themes.css`/`theme.js`/baseline were frozen to `ui-progress-parity` through its run; released to THIS milestone by its 2026-10-04 closeout | precondition section above - verify T3 markup in the starting tree before mutating these files |
| Authored content pure ASCII; no PowerShell file rewrites | `python tools/encoding_guard.py --staged` (CI: `--all`) |
| R1/R2 hygiene | `python tools/hygiene_check.py` exit 0 |

## Gates (exact commands, timeout 600000 ms each; scoped only - ~7 invocations total)

1. `python tools/testrun.py --distill -p ariadne-ui` - RED run. Expect compile-clean,
   exactly the 5 named failures. Canonical summary line + digest header verbatim in report.
2. `python tools/testrun.py --distill -p ariadne-ui` - GREEN run after all surfaces + baseline.
   Canonical line must read 0 failed.
3. `python tools/theme_token_check.py --update-baseline` - after the grey block exists;
   paste its one-line count output (themes.css 211 -> ~281).
4. `python tools/theme_token_check.py --check-added --base <parent-rev>` - exit 0, with the
   regenerated baseline staged in the same commit.
5. `python tools/hygiene_check.py` - exit 0.
6. `python tools/encoding_guard.py --staged` - exit 0 before committing.
7. `python tools/commit_scoped.py --scope "tasks/ui-dark-grey-theme-t1-*" --scope "crates/ui/assets/*"
   --scope "crates/ui/tests/ui_test/*" --scope "tools/theme_token_baseline.json"
   -m "feat(ui): register dark-grey fourth theme (neutral chrome, locked semantics)"`
   - message body carries the baseline justification sentence. Never `git add .`.

Workspace-wide gates (full `--baseline` census, `ui_smoke.py`, manual browser pass) belong to
the reviewer acceptance pass below - NOT to this handoff.

## Discipline clauses

- UTF-8: never Set-Content/Out-File/>/Get-Content-replace; Write/Edit tools only; authored
  content pure ASCII (new CSS comment lines use `--`, not em-dash) - see AGENTS.md File-Edit
  Discipline.
- Touch only the touch-set; on discovering out-of-scope breakage (e.g. a Rust site that turns
  out to enumerate theme ids): report, do not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full `unittest discover`,
  `ui_smoke`, `bench_compare`) in an implementer session are a reviewer blocker; run them
  through the distilling entry points (`docs/specs/tool-output-distillation.md`).
- After EACH per-module Green gate, append a 1-3 line checkpoint to
  `tasks/ui-dark-grey-theme-t1-journal.md` (created alongside this handoff): module + commit
  hash, any deviation, any lock/contract finding. The journal is the dead session's decision
  log; without it, recovery restores code (git) but loses reasoning.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead session is NEVER
  resumed - successor runs `python tools/checkpoint_guard.py resume --spec
  tasks/ui-dark-grey-theme-t1-handoff.md --scope crates/ui --cmd "python tools/testrun.py
  --distill -p ariadne-ui"`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence (first failing test list, the 5 names).
3. Green evidence (distill-mode digest header + canonical summary line verbatim + capture
   path). 4. Baseline delta line (211 -> actual). 5. Any deviation + reason.
6. Out-of-scope observations (report-only).
Never paste raw tool output (cargo/git dumps) into the report: cite the command + the
canonical summary line + exit code. The reviewer re-runs gates against the commits.

## Reviewer gate reference

Adjudicated via the `review-gate` skill (fresh full verification, per-commit `git show`
oldest-first, commit-vs-touch-set enforcement against the touch-set above - so the baseline
regeneration MUST be in the same commit as the themes.css block). Reviewer-carried items
beyond the implementer gates, from the shape's acceptance list:
- `python tools/ui_smoke.py` exit 0 (default-theme boot unchanged).
- `python tools/theme_token_check.py --check-added --base <parent-of-landing>` exit 0.
- grep `index.html` for leftover "three"/"all three" prose (risk R7 - deliberately no test).
- styles.css / non-theme assets / pipeline / progress.rs byte-identical (touch-set check).
- Manual browser pass in `Dark (Grey)`: select switches with graph re-render (node cards
  repaint, not just chrome); survives reload (localStorage) and appears in a standalone
  export; a cycle card still reads red; heat ramp still reads cool -> hot; verdict spine
  green/amber/red legible; node-kind cards still show the default saturated tints.
- Orchestrator-owned (not implementer, not reviewer): in the landing commit window, the
  memory row `ui.three_theme_set` -> four-theme set (light, dark, dark-grey,
  dark-steampunk) and the `ui-dark-grey-theme` milestone row flip in
  `tasks/milestone_ledger.md`, per the ledger authority rule.
