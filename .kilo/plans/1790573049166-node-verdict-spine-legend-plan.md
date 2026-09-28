# Node Verdict Spine + Teach the Card Vocabulary

## Problem

With a file selected in the include graph, the node card outline has no stable meaning: kind color → heat tier ring → two ad-hoc verdict overrides (`.dep-justified` full-green outline, `.dep-mixed` `drop-shadow(-3px 0 0 green)` side marker that reads as a rendering bug). The node can also contradict its arrow: `util.h` (0/1 used, droppable) shows a calm cool card while its arrow is red-dashed "unused". The legend teaches the arrow vocabulary (line-style swatches) but says nothing about card outlines; help copy is stale ("green-outlined").

## Decision (confirmed with user)

1. **Verdict spine model**: cards carry the verdict as a visible left accent bar (real SVG rect, same idiom as `.extraction-card` border-left). Fill stays pure collateral heat.
2. **Full unification, flat cards**: ALL verdicts render ONLY as the spine — `dep-justified`'s full-green outline and `dep-mixed`'s shadow hack are deleted. Colored tier glows are deleted from dependency cards; glow is reserved for selection/root (cyan). One sentence to teach: **spine = how needed, fill = how heavy, cyan = you-are-here, arrow = same verdict as spine.**

## Target vocabulary (include tab, selection active)

| Channel | Value | Style |
|---|---|---|
| Left spine (new `<rect class="verdict-spine">`) | needed (`j>0, c=0`) | `var(--green)` #4ade80 |
| | partly needed (`j>0, c>0`) | `var(--amber)` #fb923c |
| | unused (`j=0, c>0`) | `#ef4444` |
| | neutral / no verdict data | no spine |
| Card fill | heat tier (`getHeatColor(c, max)`), applied to EVERY reachable dep incl. justified (j ⇒ c=0 ⇒ cool tier) | existing 5-tier palette, flat (no glow) |
| Card stroke | heat tier border color, 2.5px (rename class `heat-glow-N` → `heat-tier-N`; keeps stroke-width, drops `filter`) | existing tier colors |
| Glow | selected + heat-root only | existing cyan rules, unchanged |
| Badge text | verdict color (green/amber/red) like the spine; `+N col` neutral badge keeps tier color | — |

Spine geometry: `<rect x="0" y="6" width="4" height="30" rx="2">` inside the node group (inset flag bar, avoids fighting the card's rx=8 corners). Kind fills/outlines remain for non-include tabs, unselected include view, and faded/unreachable nodes; cycle nodes keep their red kind outline when unreachable. Preview-after-extraction hides spines (`.preview-hypothetical .verdict-spine { display: none; }`) — preview overlays already own the cards there.

## Files

- `crates/ui/assets/graph.js` — node heat pass (lines ~335-379): apply heat fill/stroke to justified nodes too; build spine via new `appendVerdictSpine(el, tier)` helper (keep functions <100 lines); color badges by verdict; emit `heat-tier-N`; stop adding `dep-justified`/`dep-mixed`.
- `crates/ui/assets/styles.css` — delete `.node-group.dep-justified rect` and `.node-group.dep-mixed rect` (lines ~443-451); rewrite `.heat-glow-N` block (lines ~437-441) as `.heat-tier-N rect { stroke-width: 2.5px; }` (no `filter`); keep the empty legacy `.heat-critical rect` selector and its graph.js class application (I5 lock); add `.verdict-spine` + three verdict color classes using the same tokens as `.edge-justified`/`.edge-leaving`/`.edge-unused`; add `.preview-hypothetical .verdict-spine { display:none; }`; add `.legend-card` swatch CSS.
- `crates/ui/assets/main.js` — `updateHeatLegend()` (lines ~147-183): after the existing arrow rows (keep their gating/order contract), append a node-card block: mini card-shaped swatches (`.legend-card` + `.spine-*`) — "Card: every symbol used — needed" / "Card: partly needed" / "Card: supplies nothing — this include can go" / "No spine — no verdict data"; retitle pill block intent with one muted line "Card fill = collateral weight"; keep footnote. Extend `verdictTooltipText()` to name the spine ("green spine: every supplied symbol is used" etc.). Update tour step 5 copy (line ~251).
- `crates/ui/assets/index.html` — rewrite the stale "Justified dependencies … render green-outlined" sentence in `#helpTab-legend` (line 417) to the new spine/fill/glow vocabulary.

## Tests (Red first, package `ariadne-ui`)

New `crates/ui/tests/ui_test/node_verdict_spine.rs` (register in `main.rs`), string-assert style of `edge_verdict_story.rs`:
- graph.js contains `verdict-spine`, `heat-tier-`, `appendVerdictSpine`; no longer adds `dep-justified`/`dep-mixed`.
- styles.css: `.verdict-spine.justified`/`.mixed`/`.unused` carry green/amber(#fb923c or var)/red; NO `.dep-justified` or `.dep-mixed` selector remains; every `.heat-tier-N rect` rule body contains `stroke-width` and NO `filter`; `.preview-hypothetical .verdict-spine` display:none present.
- legend: `updateHeatLegend` emits `.legend-card` spine swatch rows with needed/partly/unused copy; green+unused card rows gated on `verdictTiers` like the arrow rows.
- help + tour copy mention "spine" and no longer claim "green-outlined".
- tooltip copy names spine colors.

Update tests that lock the old design (they encode deleted behavior, rewrite deliberately in the Red step):
- `static_html.rs` ~244-256: replace dep-justified/dep-mixed rule assertions with spine assertions.
- `static_html_sections.rs` ~275-326: heat-glow glow assertions → heat-tier no-filter assertions (selector rename included); drop the `.dep-justified` 10px-glow assertion; keep `.heat-critical rect` existence + order + no-filter locks.
- `edge_verdict_story.rs` ~127: "dep-justified/dep-mixed must remain intact" → spine-class application assertion.

## Task order (RGB)

1. **Red**: add `node_verdict_spine.rs` + rewrite the enumerated old assertions; `cargo test -p ariadne-ui` fails only on new/updated expectations.
2. **Green — graph.js**: spine building, justified heat fill, class rename, badge verdict colors.
3. **Green — styles.css**: spine classes, delete verdict overrides, heat-tier flatten, preview hide, legend swatch CSS.
4. **Green — main.js + index.html**: legend card rows, tooltip/tour/help copy. `cargo test -p ariadne-ui` green.
5. **Refactor + hygiene**: extract helpers so no function ≥100 lines; run `python tools/hygiene_check.py` (generated-artifact allowlist already in place).
6. **Visual verification** (user-side): rebuild tool.exe, select `graph.h`, confirm against this checklist:
   - `jobserver.h`: flat crimson card + **amber** spine, red badge `4/15 used` (mixed).
   - `util.h`: cool fill + **red** spine, badge `0/1 used` — node now agrees with its red-dashed arrow.
   - `eval_env.h`: amber fill + amber spine (spine readable against the darker heat fill — verify; if muddy, widen spine to 5px only).
   - `dyndep.h`/`explanations.h`: green/amber spines per verdict; `timestamp.h`: no spine, `+0 col`.
   - Legend shows the new card rows with gating; Esc restores the standard legend; Preview-after-extraction hides spines.

## Risks / notes

- Spine-vs-rounded-corner readability: inset flag bar chosen; final look confirmed in step 6.
- Users trained on "green outline = good": justified cards stay strongly green (spine + badge + legend row); help copy bridges the change.
- Cascade-order test locks touch deleted/renamed selectors — updated intentionally, not regressions.
- Amber spine on amber-heat fill is the one readability tension (only when `0.15 < t ≤ 0.35…0.60` AND mixed); fallback is a 5px spine, decided by eye in step 6.

## Open questions

None — remaining details use the defaults above (universal heat fill for justified; no spine for neutral; legend rows appear only in heat view).
