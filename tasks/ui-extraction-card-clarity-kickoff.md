# Milestone Kickoff — ui-extraction-card-clarity

Milestone key (verbatim, for planner/ledger): `ui-extraction-card-clarity`
M-number and dates are assigned by the milestone-lifecycle kickoff runner; do not guess.
Authored by the architect at HEAD (working tree 2026-10-08), read-only survey.
Prior-milestone artifacts (`ui-extraction-card-story`, `ui-extraction-card-playbook`,
both CLOSED) are background, not binding.

## Intent

The user's verdict (verbatim, treated as intent):

- "Proposed extractions look like shit. The card is dense, cloudy looking and it
  does not tell the story of what to do with the data."
- Scope of failure: BOTH the summary row and the expanded detail, plus the whole
  Proposed Extractions section "reads as one gray mass".
- "Cloudy" = LOW CONTRAST / MUDDY COLORS specifically — card background, borders
  and text blend; a styling problem, not just information overload.
- Story shape demanded: PER-FILE INSTRUCTION BLOCKS — one block per suggested new
  file: path heading, what goes in it, which symbols; commands after. Not the
  current stacked beats.

Underlying why: the card is the UI's only actionable output of the extraction
planner; if it cannot be scanned and followed, the analyze->apply spine is dead
even though the CLI spine (`tool apply`, `tool migrate --preview`) is real.

Blast radius: `crates/ui` assets (hotspots.js, styles.css) + `crates/ui` string-lock
tests + token values in `crates/ui/assets/themes.css`. Zero Rust logic change,
zero DTO change, zero producer (cli/cluster/planner) change, index.html untouched.

## Ground truth verified at HEAD

- Theme count is FOUR, not three: `theme.js` registry
  `['light', 'dark', 'dark-grey', 'dark-steampunk']` and four blocks in
  `themes.css`. The memory constraint `ui.three_theme_set` is stale; the
  `ui-dark-grey-theme` ledger row still reads PLANNED although its T1 landed at
  `41ec354` (themes.css dark-grey block exists at HEAD) — its row flip + memory
  update is that milestone's pending closeout bookkeeping, not this milestone's.
  This milestone inherits the four-theme reality.
- Renderer: `renderExtractionProposalCard` in `crates/ui/assets/hotspots.js:369`;
  summary-first with `.extraction-details` disclosure; shared with the inspector
  via `api.js:264 renderInspectorProposalCard`. The saved decision
  `ui.extraction_card_interaction_model` (summary-card-first + click-to-expand)
  STAYS — no justification to amend it; the user attacks density and contrast,
  not the disclosure interaction.
- Cards sit on `.hotspot-panel { background: var(--bg-surface) }` over
  `background: var(--tile-bg)` (translucent, 0.6/0.7 alpha) — effective card
  surface is a composite; the "gray mass" is mechanically confirmed (ratios
  below).
- Whole expanded detail inherits `color: var(--text-sub)` (styles.css:1142) —
  the lowest ink tier used as body text.
- `target_files` (path, symbol_count, symbol_names) + `target_files_truncated`
  are already in the payload (playbook DTO); per-file blocks need NO DTO change.
- `problem` is templated ("{N} symbols of collateral load") — a card-level why
  line, not per-file prose; there is no per-file purpose field (none is added).

## Scope in

1. A deterministic contrast oracle: a new Rust ui_test that parses the declared
   token table in themes.css for every theme in the registry and enforces floor
   ratios on the extraction-section token pairs (WCAG relative luminance; the
   helper idiom already exists in `theme_dark_grey_palette.rs`).
2. Token-value fixes in `themes.css` where the oracle goes red (fix list driven
   by the oracle; current hand-computed candidates below).
3. Renderer restructure: per-destination-file instruction blocks in the detail
   (path heading / contents+symbols / one-line rationale), footnote strip for
   stats, final commands block; summary row slimmed to scan-minimum.
4. styles.css restyle of the extraction section: opaque card surface, visible
   border, ink tiers, per-block whitespace and size scale, section-level
   grouping so cards + header stop reading as one mass.
5. Sanctioned amendments to `extraction_card_story.rs` / `extraction_card_playbook.rs`
   strictly per the amendment table in the shape (3 test functions amended;
   none deleted; everything else byte-identical).

## Scope out (each with reason)

- DTO or producer changes: all needed data is already in the payload; adding a
  per-file "purpose" field would chain cli+cluster+ui and is not needed for the
  fix.
- index.html / help copy: the locked legend copy already describes the new
  design ("collapsed summary", "lists the suggested destination files with the
  symbols each should hold, plus the exact tool apply command"); zero-touch
  also removes the index.html conflict surface entirely.
- One-click apply / mutation routes: prior milestone boundary (UI never mutates)
  stands; unchanged.
- styles.css module split: the file is 1807 lines (over the 1000 ceiling,
  invisible to R2 — deferred ledger row dated 2026-10-01). Splitting means a new
  embedded asset + lib.rs export + baseline regeneration; decision belongs to
  that deferred row, not smuggled into this milestone. Net growth here is capped
  (old stacked-beat rules are deleted as block rules land).
- Non-extraction surfaces (legend, justification rows, graph): contrast fixes
  must be expressed as themes.css token values only; no rule changes elsewhere.
- Preview control behavior (E3): byte-identical markup and behavior.

## Invariants that must hold (full contracts in the shape)

1. Summary-first interaction model preserved verbatim (playbook P4/P5 locks
   green UNEDITED).
2. Per-file blocks: one block per `target_files` entry in payload order; heading
   = escaped real path; symbols via the escaping chips helper; honest truncation
   and legacy-fallback behavior survive.
3. Command honesty: exactly `tool apply <repo-root> --plan <id>` and
   `tool migrate <repo-root> --plan <id> --preview` are named (P9 lock survives
   UNEDITED).
4. No new data sources; DTO byte-identical; `proposals_payload.rs` UNEDITED green.
5. Payload order rendered as-is (no sort); no comma run-on blobs (S4/S6 survive).
6. Zero new hex in rule files; `themes.css` remains the only hex site;
   `theme_token_baseline.json` counts unchanged (value edits only).
7. Contrast floors green for every theme in the registry (4 today).
8. All 11 existing story+playbook test functions: 8 survive UNEDITED, 3 amended
   per the named table, 0 deleted.

## Frozen locks inherited from the ledger (with proving command)

- Themes.css is the single color-definition site; styles.css/index.html carry no
  literals: `python tools/theme_token_check.py --check-added --base <start-rev>`
  silent + `cargo test -p ariadne-ui theme_contract` green.
- `tools/theme_token_baseline.json` byte-locked set: regenerate ONLY in a
  commit that justifies each literal move — expected NOT to appear in any diff
  of this milestone (counts unchanged).
- Semantic verdict hues (--green/--amber/--red) may only be retinted for
  contrast, never for style (themes.css D6 rule): any value change must name the
  red oracle pair as justification; the grey theme's byte-equality-to-default
  lock (`test_dark_grey_semantic_layer_equals_default_block`) further confines
  fixes to the light block (expected fix list touches light only).
- `theme_dark_grey_palette.rs` stays UNEDITED (grey chrome achromatic bound
  constrains any grey token change: keep spread <= 4).
- `tests/fixtures/**` frozen; goldens on the scala branch untouched (assets-only
  milestone).
- Census anchor rule: reconcile from the newest ledger anchor at milestone start
  (M28 T0c accepted 1250 as anchor; hands report deltas, not hardcoded totals).

## Acceptance gate (milestone as a whole)

- G1 Contrast oracle green for all four themes; its pair table and floors are
  the shape's contract C4 verbatim; red-listed pairs at first run are reported
  verbatim in the T-A journal.
- G2 Scoped ui `python tools/testrun.py --distill -- -p ariadne-ui` exit 0 with
  the amended literals in place; all amended/added test names listed in verdicts.
- G3 Workspace census via `--distill` (the literal `--baseline` file still does
  not exist — open ledger errata row, 4+ encounters; the substitute is the
  standing practice; counts reconcile from the start anchor).
- G4 `theme_token_check --check-added --base <start-rev>` silent; baseline JSON
  diff-empty; `theme_contract` + `theme_js_zero_hex` + `theme_dark_grey_palette`
  green UNEDITED.
- G5 `python tools/ui_smoke.py` PASS (distill route if the session's allowlist
  denies direct runs — known deferred row).
- G6 `python tools/hygiene_check.py` + `python tools/encoding_guard.py --all`
  exit 0.
- G7 `python -m unittest discover -s tools -p "*_tests.py"` unchanged
  (`Ran 188 ... OK`) — zero python changes expected.
- G8 bench: EXEMPT with proof, standing precedent (story T1, playbook T-C):
  zero paths under `crates/*/src` in the milestone diff (themes.css/assets and
  `crates/ui/tests` are not `src`).
- G9 Human gate: closeout walkthrough deck shows the redesigned section in all
  four themes (walkthrough-commit skill).

## Handoff sizing summary (detail in shape)

- T-A: contrast oracle test + themes.css value fixes (crates/ui only).
- T-B: renderer restructure + styles restyle + 3 test amendments + new clarity
  locks (crates/ui only). Depends on T-A.
- T-C: contingent split (section chrome / whitespace only) if T-B's measured
  estimate exceeds ~1200 lines or ~10 gates. Planner decides at authoring.
- Concurrency at kickoff: no live hand touches crates/ui/assets or index.html;
  M28 is operator-tail (bench baseline commits, Corpora field boot) file-disjoint;
  M27 T1b/PR-2 is operator bench duty; dark-grey closeout is docs-only.
  Clear to serialize: land T-A, T-B sequentially on main.
