# AR-11 Handoff - Escape-every-interpolation + data-id refactor (S2 HIGH, S4 MED)

Wave 3 / UI injection hardening. Crate (`ui/assets` + `ui_test`). Precondition
MET: `ui-degree-opacity-ramp` CLOSED 2026-10-09 (graph.js at 961 < 1000).
NEVER run parallel with another UI hand (AR-02/13/15 or clarity T-A/T-B) —
UI hands serialize on `main`, one at a time.

## Objective

1. Apply the AR-01 attribute-safe escaper at every repo-derived interpolation:
   `graph.js` (4 sites near :596-:640 at audit base — re-anchor at start),
   `api.js` (:304,:324), `traversal.js` (:44,:131), `hotspots.js` (:275,:278,
   partial already done in AR-01 at data-preview-*); extend `escapeHtml` with
   quote escaping.
2. Replace inline `onclick="...('${id}')"` with `data-id` +
   `addEventListener` delegation (graph/api/hotspots). Behavior-preserving —
   click handlers locked.
3. New oracle tests in `ui_test` (NEW mod, do not cross server_api.rs line
   ceiling): fixture-served filename containing `<svg onload>` + attribute
   quotes; assert inert rendering (single-pass string lock, not browser).

## Impact

Closes the confirmed XSS cluster: hostile analyzed-repo content cannot execute
in the analyst's browser. Visible behavior unchanged for normal names.

## Touch-set (hard)

- `crates/ui/assets/graph.js` (line budget lock: dedicated
  `test_graph_js_line_budget()` cap 990 must stay green — 29 lines headroom at
  base; if the change cannot fit, move escaping into the shared helper file,
  do NOT raise the cap)
- `crates/ui/assets/api.js`, `traversal.js`, `hotspots.js`, `file-view.js`,
  `main.js` (delegation wiring where handlers move)
- `crates/ui/tests/ui_test/` NEW mod + 1-line `main.rs` registration
- `tasks/ar-11-handoff-journal.md`

## Constraints

- TDD red-first: hostile-fixture oracle FAILS first against today's markup.
- Anchors are content-anchored (line numbers advisory); STOP on undeclared red.
- `ui.traversal_view_degree_setting_bug` is known-open: do NOT regress degree
  handling in traversal.js (zero-diff there except the two interpolation sites).
- Pure ASCII; commit_scoped; distilling gates.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-ui`
2. `python tools/distill.py -- python tools/ui_smoke.py`
3. `python tools/encoding_guard.py --all`
4. `python tools/hygiene_check.py` (zero NEW; server.rs pair AR-13-owned)
5. `python tools/theme_token_check.py --check-added --base <start-rev>` (silent)

## Invariants / locks

- graph.js dedicated line-budget test green (cap 990).
- Click-delegation swap preserves every handler's behavior (locked by
  characterization tests over the shipped asset strings).
- All extraction-card / traversal / hotspot story tests UNEDITED and green.
