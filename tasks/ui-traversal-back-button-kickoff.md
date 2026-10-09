# Kickoff: UI traversal back button (key `ui-traversal-back-button`)

Authored 2026-10-08 by the architect role.
No M-number: milestone-lifecycle assigns numbers at ledger registration;
precedent rows `UI hotspot-canvas direction` (unregistered track, live row)
and the boot-overlay clean track show the pattern.
The ledger (`tasks/milestone_ledger.md`) stays numbering authority.
Shape artifact: `tasks/ui-traversal-back-button-shape.md`.

## Intent

Replace the traversal panel's `Close` (X) control with a `Back` control that
returns the user to the view they came from, instead of always jumping to the
include tab.
The user's stated motive is intuitiveness, not a functional bug.
The underlying why: the traversal view is a side-trip with two entry points
(the `#tabTraversal` tab button from any tab, and the hotspot-card traversal
button from the include tab), but the current control
(`crates/ui/assets/index.html:388`, `traversal.js::closeTraversal`) hardcodes
`switchTab('include')`.
Calling that behavior "Close" is honest; calling it "Back" while keeping it
would be dishonest, so the label change and a one-level return-to-origin
semantic move together.

Blast radius: UI assets only -- `index.html` (2 lines), `traversal.js`
(one function block replaced, three declarations added, exports updated),
`styles.css` (2 rule renames), `hotspots.js` (1-line entry swap), plus one
new ui_test lock module and its `mod` line.
Zero Rust source diff: `crates/ui/src/**` is NOT touched; embedded assets
(`lib.rs` `include_str!`, `export.rs` inline roster, `server.rs` route)
carry content changes without edits.

## Precondition (blocking dispatch, not this shape)

The working tree at kickoff time is NOT gate-clean and carries an uncommitted
traversal-view work-in-progress:
`crates/ui/assets/traversal.js` is untracked (new); `api.js`, `graph.js`,
`hotspots.js`, `index.html`, `main.js`, `styles.css`, `dto.rs`, `export.rs`,
`lib.rs`, `server.rs`, `theme_contract.rs` are modified;
`docs/specs/collateral-traversal-view.md` is untracked.
Saved project memory records a failed degree-setting fix attempt in this same
area, and the tree shows working-tree `graph.js` at 1051 lines against the
committed budget lock `GRAPH_JS.lines().count() <= 998`
(`crates/ui/tests/ui_test/static_html_sections.rs:215`), i.e. the scoped ui
suite is expected red in the working tree today.

Consequences:
- This track's baseline is the tree AFTER that WIP lands or is otherwise
  dispositioned by the operator/orchestrator (their commit duty; this track
  neither absorbs, reverts, nor rebuilds it).
- Handoff anchors must be content-anchored, not line-number anchored, in
  case the WIP moves lines before landing.

## Scope in

- The traversal-panel header control: class rename, label, icon glyph,
  tooltip, handler name (`index.html:386-389` region).
- Return-to-origin semantics: a one-level return tab captured at traversal
  entry, defaulting to `'include'` (`traversal.js`).
- Routing BOTH traversal entry points through the capturing entry function
  (`index.html` tab button onclick; `hotspots.js::openTraversalForRoot`).
- The CSS rename `.traversal-close` -> `.traversal-back`
  (`styles.css:1793-1805`), token-only, no color changes.
- One new ui_test lock module pinning the new contract (see shape C5 for
  why it must be a new file: `static_html.rs` sits at 999 lines, at the
  1000 ceiling).

## Scope out (each with reason)

- Degree-setting-not-respected-on-traversal-activation bug: open correction
  (`ui.traversal_view_degree_setting_bug`), previous fix attempt failed.
  Adjacent-but-separate; this track's only obligation is I4 (zero diff in
  the degree code path) -- it must not regress, absorb, or silently re-fix
  it, and it must not amend the failed WIP.
- `graph.js` line-budget relocation (deferred ledger row owned by the
  heat/legend extraction follow-up): this track keeps `graph.js`
  byte-identical instead of touching the lock.
- Multi-level back / navigation history stack: a one-level return satisfies
  the intuitiveness motive; a stack is over-engineering for a 4-tab app with
  a permanently visible tab bar.
- Esc-key or global shortcut close/back: none exists today for this panel;
  inventing one is beyond the request.
- A `panel-restore-tab` left-edge affordance (the hotspot pattern): the
  Traversal tab button stays visible in the tab bar, so there is nothing to
  restore from; the hotspot restore tab exists because hotspots have no tab.
- `themes.css` / `theme.js` / token baseline: not needed -- the control is
  restyled purely with existing `var()` tokens (lock L3).

## Invariants (summary; falsifiable forms with oracles in the shape)

1. The traversal panel renders a Back control (`traversal-back` +
   `backFromTraversal()`); the close contract strings are gone.
2. Back returns to the tab captured at the most recent traversal entry;
   default `'include'`; both entry points capture.
3. `graph.js`, `themes.css`, `theme.js`, `main.js`, all of `crates/ui/src/**`
   are byte-identical to the track baseline.
4. The degree code path (`switchTab` traversal branch,
   `parseAndRenderElements`, `filterAndExpandSubgraph`) is unchanged
   (implied by 3) and the degree bug's open status is untouched.
5. Asset transport unchanged: `/app/traversal.js` still served with
   `no-store`; still exactly 9 classic scripts; standalone export still
   inlines `traversal.js` and has no `src="/app/` references.
6. Zero added hex literals (token-only CSS); three (four) themes render the
   control through tokens with no theme-specific edits.
7. No test file other than the new lock module and the ui_test `main.rs`
   mod line is modified; all pre-existing ui tests survive unedited.

## Frozen locks inherited (with proof commands)

| Lock | Proof |
|---|---|
| 9-script roster + tag-start asset whitelist (`theme_contract.rs`, `tag_start_is_asset` incl. `/app/traversal.js`) unedited and green | scoped ui suite: `python tools/testrun.py --distill -- -p ariadne-ui` |
| graph.js line budget `<= 998` (`static_html_sections.rs:215`) -- preserved trivially by graph.js zero-diff; this track must NOT amend the number | scoped ui suite + `git diff <base> -- crates/ui/assets/graph.js crates/ui/tests/ui_test/static_html_sections.rs` empty |
| Asset byte-serving + `no-store` route test and `test_script_tag_order_preserves_original_execution_order` unedited (neither lists `traversal.js` content, so content change rides through) | scoped ui suite |
| Standalone export inlines split assets, no `src="/app/` (`test_standalone_export_inlines_all_split_assets`) unedited | scoped ui suite |
| Zero added hex vs landing parent | `python tools/theme_token_check.py --check-added --base <landing-parent>` silent |
| No mojibake / pure-ASCII authored content | `python tools/encoding_guard.py --all` exit 0 |
| Hygiene R1/R2 blocking | `python tools/hygiene_check.py` exit 0 |
| UI runtime smoke routes unchanged | `python tools/ui_smoke.py` PASS |
| Goldens (`m00` parity) and `tests/fixtures/**` untouched; extraction-card story/playbook suites untouched | `git diff` pathset + those suites green unedited |
| Census anchor: last ledger anchor `1263 passed / 0 failed / 13 ignored / 91 suites` @ `c17d182`; standing errata row (5 encounters) says reconcile from the live tree via `--distill`, never guess | `python tools/testrun.py --distill -- --workspace` |

## Acceptance gate for the milestone

Single handoff expected to be sufficient (1 crate, ~8 gates, est. ~300-450
produced lines -- within the Planning Law; planner confirms sizing).
Gate set (all scoped to `crates/ui` + tools guards; workspace census per
ledger practice):

- G1 Scoped ui suite green, with the new lock module present and green, and
  the pre-existing ui test set byte-identical (I7).
- G2 `ui_smoke` PASS.
- G3 `theme_token_check --check-added` silent (zero added hex).
- G4 `hygiene_check` + `encoding_guard --all` exit 0.
- G5 Rust-source zero-diff proof: `git diff <base> -- crates/ui/src crates/ui/src/**` empty
  (asset changes ride `include_str!`; no manifest/lockfile diff).
- G6 Touch-set exactness (reviewer commit-vs-touch-set per `review-gate`).
- G7 Journal carries the manual-smoke line: enter via tab button from
  symbol/call tab -> Back lands on that tab; enter via hotspot card -> Back
  returns to include with the hotspot panel restored; degree behavior is
  unchanged from baseline (still the known-open bug, not worsened).
- G8 Workspace census via `--distill`, count reconciled against the live
  baseline + the new test count (never against a guessed number).
