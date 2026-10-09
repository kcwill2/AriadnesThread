# Shape - UI Dark-Grey Theme (fourth registered theme)

Location note: the orchestrator requested `docs/specs/`, but the architect
permission profile denies edits outside `tasks/**` and `.kilo/plans/**`; the
artifact lives here, next to its format reference
(`1791093392957-ui-cli-progress-parity-shape.md`). Move it into `docs/specs/`
with an authorized editor if a canonical location is wanted.

Role boundary: this is the architect's shape, not a task list. It folds the
kickoff essentials (intent, scope, locks, gates) into the single artifact the
orchestrator asked for. The planner sizes handoffs from it; the implementer
executes. Milestone key proposed: `ui-dark-grey-theme` (M-number and dates are
assigned by the milestone-lifecycle kickoff, never guessed here).

## Kickoff context

Intent. Add one more selectable color theme, "dark grey", to the graph UI.
Behavior change: the registered theme set grows 3 -> 4; a user can pick
`Dark (Grey)` from the toolbar select, it persists across reloads and exports,
and the standalone export carries it too. Blast radius: three static asset
files under `crates/ui/assets/`, one regenerated drift-guard baseline, the
theme contract suite, and one memory/ledger constraint row. No Rust logic, no
protocol, no CLI surface.

Why: the existing `dark` is slate - its surfaces carry a blue tint
(`--bg-main: #0b0f17`, blue channel strictly above red). Some users read slate
as "blue" and want a true neutral dark chrome. The theme mechanism
(plan 1790656207440) was explicitly built for this: the `index.html` bootstrap
comment already says "adding a fourth theme there REQUIRES mirroring it here",
and the parity test iterates discovered blocks rather than a hardcoded theme
list, precisely so a fourth theme is forced into parity.

Constraint decision recorded here: the saved project constraint
`ui.three_theme_set` ("UI must provide light, dark, dark-steampunk") is
extended to a FOUR-theme set: light, dark, dark-grey, dark-steampunk. Per the
ledger authority rule, kilo-memory must be updated in the same commit that
lands this change; the orchestrator owns that edit (the architect may not touch
ledger or memory).

Milestone posture: opens its own milestone. It must NOT ride
`ui-progress-parity`, whose ledger row explicitly freezes
`themes.css` / `theme.js` / `tools/theme_token_baseline.json` as untouched.
Sequencing: `ui-progress-parity` T3 also edits `crates/ui/assets/**` and
`crates/ui/static_html.rs`; the shared file with this change is `index.html`
(bootstrap line, `<option>`s, Help paragraph). Land this milestone after
`ui-progress-parity` T3 merges, or accept a small `index.html` rebase.

### Scope in

- `crates/ui/assets/themes.css` - one new `[data-theme="dark-grey"]` block.
- `crates/ui/assets/theme.js` - one token of the `THEMES` array literal.
- `crates/ui/assets/index.html` - bootstrap validator line, one `<option>`,
  the bootstrap comment's "three accepted ids" prose, the Help theme paragraph.
- `crates/ui/tests/ui_test/theme_contract.rs` - export assertion extended.
- New `crates/ui/tests/ui_test/theme_dark_grey_palette.rs` + `main.rs` mod line.
- `tools/theme_token_baseline.json` - regenerated in the same commit (see
  frozen locks) with the justification sentence from Contracts.
- Same-commit orchestrator edits: `ui.three_theme_set` memory row -> four-theme
  set; a new milestone row in `tasks/milestone_ledger.md`.

### Scope out (with reason)

- `crates/ui/assets/styles.css` and every other asset JS - the single-definition
  site invariant means a new theme is pure token definition; `gui_scrollbar_style.rs`
  proves every rule already reads `var(--token)`. A rule change here would be
  a design bug.
- All of `crates/ui/src/**` - verified by survey, no Rust site enumerates theme
  ids: `lib.rs` is `include_str!` constants, `server.rs` has fixed routes
  (`/themes.css`, `/app/theme.js`), `export.rs` inlines the whole
  `THEMES_CSS`/`THEME_JS`/`INDEX_HTML` strings with per-asset substitution and
  no theme-id list. Adding a theme cannot require a Rust change; if the
  implementer finds one, that is a finding to escalate, not to absorb.
- `tools/theme_token_check.py`, `tools/ui_smoke.py` - the checker exempts
  `themes.css` (TOKEN_FILES) by design; `ui_smoke.py` contains zero theme
  references (grep-verified), it only cold-boots the default theme.
- New token names - the grey block must declare exactly the default block's
  token set; the N-way parity test rejects additions as hard as omissions.

### Inherited frozen locks (each with its proving command)

| Lock | Source | Proof command |
|---|---|---|
| `tools/theme_token_baseline.json` hex snapshot: regenerate only in a commit justifying every new literal | ledger "Byte-locked sets" | `python tools/theme_token_check.py --check-added --base <parent-rev>` exit 0, with the regenerated baseline in the same commit |
| `tests/fixtures/**` frozen | ledger | reviewer touch-set check (files must not appear in the diff) |
| `crates/pipeline/**` + `crates/cli/src/commands/progress.rs` byte-identical (while `ui-progress-parity` runs) | ledger `ui-progress-parity` row | untouched by this scope by construction; reviewer touch-set check |
| `themes.css`/`theme.js`/baseline untouched *by ui-progress-parity* | ledger same row | this change may touch them only as its OWN milestone |
| Authored content pure ASCII; no PowerShell file rewrites | AGENTS.md file-edit discipline | `python tools/encoding_guard.py --staged` (CI: `--all`) |
| R1/R2 hygiene | `tasks/milestone_ledger.md` graduation rows | `python tools/hygiene_check.py` |

### Milestone acceptance gate

1. `python tools/testrun.py --distill -p ariadne-ui` - full suite green; quote
   the canonical line verbatim in the handoff report.
2. `python tools/theme_token_check.py --check-added --base <parent-rev>` exit 0.
3. `python tools/encoding_guard.py --staged` clean.
4. `python tools/ui_smoke.py` exit 0 (default-theme boot unchanged).
5. `python tools/hygiene_check.py` no new blockers.
6. Manual browser pass in `Dark (Grey)`: select switches with graph re-render
   (node cards repaint, not just chrome); survives reload (localStorage) and
   appears in a standalone export; a cycle card still reads red; the heat ramp
   still reads cool -> hot; verdict spine green/amber/red legible.
7. Same-commit: memory row `ui.three_theme_set` replaced by the four-theme set;
   new ledger milestone row (orchestrator-owned).

## Invariants (each independently checkable)

I1. `THEMES` (theme.js) equals the `#themeSelect` `<option value>`s exactly and
    in order. Existing oracle: `test_theme_select_options_mirror_the_theme_registry`.

I2. The set of `[data-theme="..."]` ids in comment-stripped `themes.css` equals
    the `THEMES` set both ways. Existing oracle:
    `test_css_theme_ids_and_registry_agree_both_ways`.

I3. The anti-flash bootstrap's accepted ids (parsed from `!== '...'` literals)
    equal the `THEMES` set. Existing oracle:
    `test_bootstrap_theme_ids_mirror_the_theme_registry`.

I4. The `dark-grey` block declares exactly the default block's token-name set
    (no adds, no omissions). Existing oracle:
    `test_every_theme_block_declares_the_full_token_set` (iterates discovered
    blocks, so the new block is forced into parity with no test edit).

I5. Block source order keeps `default < light < dark-steampunk`, and the grey
    block sits between `light` and `dark-steampunk` (mirroring registry order).
    Existing oracle: `test_token_definitions_live_only_in_themes_css` asserts
    the first inequality by position and keeps passing unchanged; the grey
    placement is locked by the new file's
    `test_dark_grey_block_sits_in_mirror_order`.

I6. Semantic layer byte-equality: every semantically locked token declared in
    the grey block has byte-equal value to the default block. Token list:
    `--green --amber --red --green-glow --red-wash --red-wash-soft
    --red-wash-hover --red-glow --red-glow-strong --red-text --amber-wash
    --amber-border --amber-glow --amber-text --heat-ramp-a
    --heat-0..4-bg/border/text --node-cycle-bg/text/border
    --preview-eliminated-bg/text/glow --preview-kept-bg/text/glow
    --preview-still-bg/text --preview-amber-edge/text/glow/outline`.
    New oracle: `test_dark_grey_semantic_layer_equals_default_block`
    (byte-string compare per token, names enumerated in the test).
    This is stronger than steampunk's guarantee, which is a header comment only.

I7. Neutral chrome: every HEX literal declared by a chrome token in the grey
    block is achromatic - `max(r,g,b) - min(r,g,b) <= 4` - and every `rgba()`
    declared by a chrome token has `r == g == b`. Chrome token list:
    `--bg-main --bg-canvas --bg-surface --bg-sub --bg-card --bg-card-hover
    --border --border-light --text-main --text-muted --text-sub --text-strong
    --on-accent --grid-dot --shadow-soft/node/ambient/lift --overlay-scrim
    --tour-scrim --overlay-panel-bg --tile-bg --progress-track --edge-neutral
    --edge-incomplete --code-text`. Exempt (decorative accents may carry hue):
    `--cyan*`, `--brand*`, `--primary-btn*`, `--tag-pill*`, `--tour-glow`,
    node-kind and proposed/violet identity tokens.
    New oracle: `test_dark_grey_chrome_is_achromatic`.
    Distinguishability corollary, same test: the DEFAULT block's `--bg-main`
    has `b - r >= 8` (the slate tint this theme must differ from) - true today,
    locked as characterization.

I8. Legibility: contrast (WCAG relative luminance, computed from the parsed
    hex) of `--text-main` on `--bg-main`, `--bg-canvas`, `--bg-surface`,
    `--bg-card` each >= 7:1; `--text-muted` on `--bg-main` >= 4.5:1;
    `--text-sub` on `--bg-main` >= 3:1.
    New oracle: `test_dark_grey_contrast_floors`. No existing test computes
    contrast for any theme today (light's "clears 4.5:1" claim is prose); this
    is the first numeric legibility lock and it is grey-only to start.

I9. Rules unchanged: `styles.css` bytes and every other asset byte-identical;
    zero color literals outside `themes.css`. Existing oracles:
    `test_styles_css_carries_no_color_literals`,
    `test_index_html_styling_carries_no_color_literals` (the new `<option>` and
    bootstrap line carry no colors), plus the checker's added-literal scan.

I10. Validation semantics unchanged: a stored value that is none of the four
    ids still resolves to `dark` in BOTH validators (theme.js `resolveTheme`
    and the bootstrap); `DEFAULT_THEME` stays `dark`. Existing oracles: I1's
    `selected` assertion on `<option value="dark" selected>` and I3's set
    equality; the degradation logic itself has no behavior change to test.

I11. Export surface: the standalone export inlines the grey block and the grey
    `<option>`. Extended oracle: `test_standalone_export_inlines_theme_tokens`
    gains `content.contains("[data-theme=\"dark-grey\"]")`; option presence
    rides the existing export-markup assertions.

## Contracts

Theme id and label. Id: `dark-grey` (lowercase, hyphenated, like the existing
ids; persisted under `ariadne_theme`, so the string is permanent user state
once shipped - see Open question Q1). Label: `Dark (Grey)`, mirroring the
`Dark (Steampunk)` label grammar. Dropdown order (registry order, locked by
I1): `light, dark, dark-grey, dark-steampunk` - dark variants adjacent.

theme.js. Exactly one line changes:

    const THEMES = ['light', 'dark', 'dark-grey', 'dark-steampunk'];

`DEFAULT_THEME`, `THEME_STORAGE_KEY`, `resolveTheme`, `applyTheme`,
`readStoredTheme`, `initThemeControl`: byte-identical. No new JS.

index.html bootstrap. The validator line becomes (single line, ASCII):

    if (bootTheme !== 'light' && bootTheme !== 'dark' && bootTheme !== 'dark-grey' && bootTheme !== 'dark-steampunk') bootTheme = 'dark';

The bootstrap's leading comment says "four accepted ids" and keeps the
mirror-warning sentence. The rest of the head is untouched (I9's color sweep
covers it).

index.html select. Between the `dark` and `dark-steampunk` options:

    <option value="dark-grey">Dark (Grey)</option>

`dark` keeps `selected`. The option-comment stays accurate (it points at the
test-locked ids, not a count).

themes.css block. One new block `:root[data-theme="dark-grey"] { ... }`,
placed after the light block and before the steampunk block, declaring exactly
the default block's ~106 token names in the same order (I4), grouped with the
same comment rhythm. Palette direction (exact hexes are implementer choice
under the invariants):

- Chrome (I7 list): pure neutral ramp, near-zero hue (zinc/neutral family
  direction, not warm greige, not cool graphite): bg-main around #141414 with
  visible card/border steps, text-main around #f5f5f5, muted/sub stepping down
  through neutral greys while holding the I8 floors.
- Semantic layer: byte-equal to the default block (I6) - verdicts, heat ramp,
  cycle red, preview verdicts. The grey theme changes the table, not the food.
- Decorative accents (`--cyan` family, brand, primary button, tag pills): keep
  the dark theme's sky-cyan family values (recommended; see Q2). Node-kind
  card identities and the violet proposed-family: keep default values for v1
  (see Q3) - saturated identity cards read fine on neutral chrome.
- The block's own header comment states the D6 discipline in ASCII: this theme
  retints chrome only; semantic hues are information.

index.html Help paragraph. The "Theme:" paragraph lists
Light / Dark / Dark (Grey) / Dark (Steampunk) and says semantic colors keep
their meaning "in all four themes". Prose is deliberately NOT test-locked
(labels are drift-tolerant by the select test's own comment); it is a reviewer
checklist item - honest note: no test locks this paragraph today.

Baseline regeneration. The landing commit contains BOTH the new block and the
regenerated `tools/theme_token_baseline.json` (themes.css count rises by
~70 hex literals - the grey block mirrors the default block's hex/token
structure; total 211 -> ~281). Justification sentence for the commit message:
"dark-grey theme block adds one hex per hex-declared token, mirroring the
default block's structure; zero literals added to any rule file".

## Module map

Changed:
- `crates/ui/assets/themes.css` - +1 block (~126 lines, 437 -> ~565).
- `crates/ui/assets/theme.js` - 1 line.
- `crates/ui/assets/index.html` - bootstrap comment + line, 1 option, Help
  paragraph (~6 lines total).
- `crates/ui/tests/ui_test/theme_contract.rs` - export assertion + the stale
  "all three theme blocks" message wording.
- `crates/ui/tests/ui_test/theme_dark_grey_palette.rs` - NEW: I5-I8 tests
  (~140 lines, small helpers).
- `crates/ui/tests/ui_test/main.rs` - +1 `mod` line (new test files need
  registration - this is an easy-to-miss touch-site of the suite).
- `tools/theme_token_baseline.json` - regenerated.

Touched (read-only expectations, no edits): `crates/ui/src/lib.rs`,
`crates/ui/src/server.rs`, `crates/ui/src/export.rs`,
`crates/ui/src/static_html.rs`, `tools/theme_token_check.py`,
`tools/ui_smoke.py` - all verified to require no change (evidence in Scope out).

Must not change: `crates/ui/assets/styles.css`, any other asset JS,
`tests/**` fixtures, `crates/pipeline/**`, `crates/cli/src/commands/progress.rs`.

## Risks and oracles

| Risk | Mitigation | Oracle |
|---|---|---|
| R1 half-added theme (one of four surfaces forgotten) | the four lock tests I1-I3 are set/order comparisons, they fire on the first test run | `cargo test -p ariadne-ui` (theme_contract) |
| R2 frozen hex baseline rejects the new block | regenerate baseline in the SAME commit with the justification sentence | `theme_token_check.py --check-added --base <rev>` exits 1 until fixed |
| R3 "cohesion" retint of verdict/heat hues in the new block (the failure mode steampunk guards only with a comment) | copy semantic values from the default block verbatim and lock equality | `test_dark_grey_semantic_layer_equals_default_block` (I6) |
| R4 blue/warm tint creeps into "neutral" greys (the slate mistake repeated) | numeric neutrality bound | `test_dark_grey_chrome_is_achromatic` (I7) |
| R5 mid-grey chrome collapses the text hierarchy (card vs surface unreadable) | contrast floors computed from the declared hexes | `test_dark_grey_contrast_floors` (I8) + manual pass, gate 6 |
| R6 `index.html` conflict with `ui-progress-parity` T3 in flight | land after T3, or rebase; touch-set discipline in the handoff | reviewer commit-vs-touch-set check |
| R7 Help/label prose still says "three themes" | reviewer checklist; deliberately NOT a test | reviewer grep of "three"/"all three" in `index.html` |
| R8 mojibake from PowerShell rewrites of assets | Edit/Write tools only, ASCII-only authored content | `encoding_guard.py --staged` |
| R9 stale localStorage values | validators already degrade unknown ids to `dark`; Q1 spelling makes an id permanent once shipped | I3 + I10, existing behavior unchanged |

## Hygiene notes

- `themes.css` 437 -> ~565 lines (ceiling 1000; safe headroom for a fifth theme).
- `theme_contract.rs` 481 -> ~485 lines; grey-palette assertions live in their
  own new file so the contract file does not creep toward the ceiling.
- No line anywhere near the 2000-char truncation cliff; the new bootstrap line
  is ~150 chars and must stay ONE line with its `&&` chain intact - the
  bootstrap parser test scans the single inline script, wrapping it is fine for
  CSS elsewhere but never reformat the validator condition across lines.
- All new functions (test helpers) well under 100 lines; names
  (`test_dark_grey_*`, `declared_tokens`, `css_block_body`) reveal intent; the
  new file reuses the parsing helpers' idiom from `theme_contract.rs`.

## Open for the user

- Q1 Spelling: `dark-grey` (the user's own wording, becomes permanent localStorage state) vs `dark-gray` (repo uses American "color" in code). Recommend `dark-grey`.
- Q2 Accent in the grey theme: keep the sky-cyan decorative family (recommended - it keeps the theme usable and steampunk proves accent retint is optional) vs a desaturated steel accent.
- Q3 Node-kind card identities: keep the default saturated blue/indigo/violet/green tints on grey chrome for v1 (recommended, byte-parity with default) vs neutralized grey cards.
- Q4 Dropdown order: grey directly after Dark (recommended, dark variants adjacent) vs appended after Steampunk (smaller diff, weaker grouping).

## Next step

Planner: this is ONE handoff (one crate + one baseline file, ~6 gate commands,
well inside the Planning Law); author it once `ui-progress-parity` T3's
position on `index.html` is known. Orchestrator: flip the memory constraint row
and add the milestone row in the landing commit.
