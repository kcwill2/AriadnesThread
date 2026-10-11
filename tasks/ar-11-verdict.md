# AR-11 Verdict - Escape-at-source + data-id delegation (S2/S4)

Commit adjudicated: `adb54ea` ("fix(ui): escape-at-source + data-id delegation
across asset JS (S2/S4)"), branch `shining-enthusiasm`, single commit.
Contract `tasks/ar-11-handoff.md`; journal `tasks/ar-11-handoff-journal.md`.
Reviewer: review-gate skill, fresh verification, own stdout throughout.
Gates addendum: `tasks/ar-11-execution-results.md`.

## VERDICT: APPROVED

Findings 1-4 are minor (records/attribution/one accepted behavior delta);
none require rework of `adb54ea`.

## 1. Touch-set - EXACT

`git diff --name-status "adb54ea^..adb54ea"` lists 9 paths:
`crates/ui/assets/{api,file-view,graph,hotspots,main,traversal}.js`,
`crates/ui/tests/ui_test/asset_injection_hardening.rs` (A),
`crates/ui/tests/ui_test/main.rs` (+1 mod registration),
`tasks/ar-11-handoff-journal.md` (A).
Every path is inside the declared touch-set. Zero `crates/ui/src/**` —
the server.rs chain is untouched by this commit (its R1/R2 rows remain
AR-13-owned pre-existing state). No undeclared deviations at the scope axis.

## 2. Oracle integrity - CONFIRMED (central check)

- Attribute count: grep `#[tokio::test]|#[test]` in the committed oracle = 9
  (8 `#[test]` + 1 `#[tokio::test]`), file syntactically clean
  (`"#[^#]` = 0 matches — no malformed-close residue).
- Registration proof: fresh scoped run (reviewer stdout) shows all 9 oracle
  tests by name executing and passing, and the canonical line
  `265 passed / 0 failed / 0 ignored / 3 suites` = 256 baseline + 9.
  Both formerly-phantom tests —
  `test_apijs_declared_sites_escaped_and_delegated` and
  `test_traversaljs_declared_option_sites_escaped_without_degree_regression`
  — match the journal's named swallowed tests exactly.
- Assertion genuineness: all 9 bodies read end-to-end. Locks are verbatim
  markup strings (positive AND negative), escaper-character sets, exact
  delegated call forms, hostile-payload serve/export checks
  (`HOSTILE_FILENAME = "<svg onload=x>"quote'.h` — tag + both quote classes,
  per contract). No tautologies; no weakened text vs contract intent. The
  two repaired tests assert exactly the contract's declared sites
  (api :304/:324 + entrypoints delegation; traversal :44/:131 + degree
  regression locks).
- Delimiter-repair adjudication (accepted, judged on mechanics, not the log):
  simulating `"#` closes at exactly the five journal-cited lines reproduces
  the claimed failure signature exactly — 211 swallows until line 225's
  proper `"##` (consuming graph.js-test's close + the apijs test's
  `#[test] fn` declaration); 239 swallows until line 255's `"##`
  (consuming the traversaljs test's declaration); 315 terminates at line
  316's proper `"##` (syntax stays valid, hotspots test registers with a
  merged check; mainjs unaffected). Exactly 2 unregistered, 7 registered —
  matching the RED line `1 passed / 6 failed` (the one pass =
  `test_no_nested_escaper_calls`, trivially green pre-change) and the
  resume probe `257 passed / 6 failed` = 256 + 7. Closing-delimiters-only
  is mechanically consistent; the oracle file was NEW (untracked pre-commit
  state not diffable), so this internal-consistency audit is the strongest
  available evidence and it is strong. Assets bent to oracle, never reverse.

## 3. Escape discipline - PASS (6-asset diff review)

Every repo-derived interpolation at touched sites now passes through the
shared escapers or rides inert data-* attributes:
- graph.js: upstream/downstream rows -> `data-select-id` + `escapeHtml(name)`
  spans; collateral row -> `data-path-from/to` + `escapeAttribute` on both
  title labels + `escapeHtml(entry.label)`.
- api.js: path-node text `escapeHtml(n)` (:304); entrypoints row
  `escapeAttribute(ep)` + `escapeHtml(ep)` (:324); preview attrs switched
  escapeHtml -> escapeAttribute (quoted-attr contexts).
- hotspots.js: `escapeHtml` quote-extended to all five characters (& < >
  &quot; &#39;); row/button/card payloads -> data-*; `targetLabel` is
  escaper-bound at definition (hotspots.js:541), single-pass.
- traversal.js: the two declared option sites only; `value` attrs via
  escapeAttribute, paths via escapeHtml.
- file-view.js: weaker 3-char `escapeHtml` duplicate REMOVED. Its two call
  sites (sourceLoadError :32, renderSourceCode :92) are inside click/render
  handlers only — never at load time — and resolve to the hotspots.js
  global (classic-script global resolution at call time). Load order
  verified in index.html:592-602 (api, graph, traversal, file-view,
  hotspots, preview, provenance, main, theme) — matches the journal
  exactly; the export surface carries exactly one escapeHtml definition
  (locked by the tokio oracle test, green in the 265 run).
- main.js: no new interpolation; delegation only.
No raw-concat innerHTML remains at touched sites; composed escaper calls
are locked absent (`test_no_nested_escaper_calls`).

## 4. Behavior preservation - PASS (one accepted delta, finding 1)

- Journal mapping table vs diff: all six mappings exact — same functions,
  same arguments (`selectNode`, `illuminatePath(pathFrom, pathTo)`,
  `selectHotspotRow`, `openTraversalForRoot`, `closeModals`). All four
  delegated container ids exist statically in index.html (:147, :340, :361,
  :369); container/handler globals exist at main.js wire time (loads last).
- Delegated suppression: traversal-button branch is FIRST with early
  return — row/card selection is never reached from a button click,
  reproducing the locked suppression of the row handler. Disclosure and
  preview buttons keep their own `stopPropagation()` paths (oracle-locked
  as shipped), so they never reach the delegated branches.
- traversal.js byte-untouched proof: `git show "adb54ea" --
  crates/ui/assets/traversal.js` = exactly two hunks, one line each, at
  :44 and :131 (stat 2+/2-). The degree-setting bug zone is outside the
  diff by definition of byte-level diffing: `dirSelect.value = 'out'`,
  the subtree row `data-node-id="${n.id}" onclick="selectNode('${n.id}')"`,
  and the :128/:135 dropdown block appear only as context or not at all.
  The known-open `ui.traversal_view_degree_setting_bug` was NOT silently
  fixed; both byte-present locks are green in the fresh run.
- Pinned survivors honored: `jumpToCrumb(${i})` (numeric index) sole
  main.js survivor; `selectNode('${n.id}')` sole traversal survivor —
  exactly as the oracle asserts, zero-diff constraint respected.

## 5. theme_token_check disposition - INFORMATIONAL-ONLY, explicitly

Reproduced against actual parent `0741074`: exactly the implementer's 3
flags (index.html 6>4, main.js 1>0, traversal.js 1>0), all from
`scan_frozen_counts`. Zero `new hex literal added since` violations — the
commit-scoped `scan_added_literals` scan (the check AR-11 is responsible
for) is CLEAN. Attribution verified independently: the flagged literals
are HTML numeric entities matched by the naive hex regex
(`&#8250;` -> `#8250` at main.js:308; `&#128161;` -> `#128161` at
traversal.js:390 — both OUTSIDE every diff hunk; index.html blob unchanged
in adb54ea), and baseline `tools/theme_token_baseline.json` (frozen at
`d76a559`: index.html=4, main.js=0, traversal.js=0) is byte-untouched.
DISPOSITION: the `--check-added` semantics do NOT require a baseline commit
from this handoff. The frozen-count scan is base-rev-independent
workspace-wide state; the flags are inherited drift from earlier lanes
that shipped entities without `--update-baseline`. Regenerating the
byte-locked, approval-gated manifest inside AR-11 would (a) violate
AR-11's declared touch-set and (b) launder attribution of other lanes'
drift into this commit. Correct resolution is a separate deliberate
`--update-baseline` commit owned by the orchestrator — routed to
closeout/deferred-findings (finding 3).

## 6. graph.js budget - PASS

`test_graph_js_line_budget` (static_html_sections.rs:221, cap 990 over
`GRAPH_JS.lines().count()`) green in the reviewer's own fresh run;
graph.js measured 961 lines (tail-read). 961 <= 990 with headroom intact;
cap not raised.

## Numbered findings

1. MINOR (accepted behavior delta; wording correction owed): journal
   "reproduces the inline stopPropagation suppression exactly" overstates.
   The suppression of the ROW handler is exactly reproduced; but the old
   inline `event.stopPropagation()` also prevented the pre-existing
   document-level QuickOpen closer (hotspots.js:634-636) from ever seeing
   traversal-button clicks. With delegation, such clicks reach it:
   activating the traversal button while the QuickOpen palette is open now
   also closes the palette (previously it stayed open on top). Grep-verified
   this is the only document/window-level CLICK listener in shipped JS
   (others are keydown/mousemove/pointer or target/container-scoped), so
   the delta's blast radius is that one overlap edge, in the
   behavior-consistent direction (outside-click closes the palette — like
   every other click does). Accepted, no rework; correct the "exactly"
   wording on the file's next touch.
2. MINOR (record accuracy): journal says file-view.js "+4/-4"; actual is
   +4/-3 (stat total 7: 3-line escaper function removed, 4 comment lines
   added). No code impact.
3. CONFIG (routed to closeout): handoff gate 5 promised `--check-added`
   "silent" — unsatisfiable at launch. The frozen-count drift was already
   present at the launch base (base-rev-independent scan; implementer's own
   run at 0d7ecd3 and reviewer's at 0741074 produce identical 3 flags).
   Needs a dedicated approval-gated `theme_token_baseline.json`
   regeneration commit (entities-vs-hex regex may deserve a tools-ticket
   alongside AR-30's testattr-count ticket).
4. RECORD NOTE: actual parent is `0741074` (AR-30 docs landing) vs
   journal's finish base `0d7ecd3` — mid-wave landing precedent
   (AR-02/AR-18). 0741074 is docs/ledger-only, zero crates/ui content,
   so every content anchor and the theme baseline state are unchanged
   across the gap.
5. ENVIRONMENT (configuration note, per gate-discipline rules): commands
   actually run vs refused. Refused by allowlist (catch-all
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`
   reported — boilerplate does not identify the cause): `python -c "..."`
   (pattern-level; script-path invocation
   `python tools/<tool>.py ...` works normally, all five gates +
   checkpoint_guard ran). Exit codes of hygiene/theme are not echoable
   under the same allowlist; adjudicated on printed contracts + exact
   output shape, matching AR-01/AR-02/AR-03/AR-08 precedent. All other
   expected gates ran fresh on reviewer stdout (see execution-results).
6. INFORMATIONAL (foreign, not attributable): the ui lib build prints 2
   pre-existing warnings (unused import `TraversalQuery`, unused variable
   `gen` at server.rs:984) — server.rs chain, AR-13/AR-02 territory.

## Deviation dispositions (implementer-declared)

- D1 oracle delimiter repair: ACCEPTED (section 2 mechanics; the only
  mutation consistent with every recorded arithmetic, and assertion text
  untouched — all locks pin contract-declared strings).
- D2 theme drift flag-not-fix, baseline untouched: ACCEPTED with the
  explicit disposition in section 5.

## Follow-up dispositions (routed, not dropped)

- Baseline regeneration commit (finding 3) -> orchestrator/closeout;
  candidate ledger deferred row next to golden_guard row.
- Journal wording corrections (findings 1, 2) -> next touch of the
  journal/asset, not a re-gate.
- Board-banked tools-ticket (main, `board_94890de5`): testrun.py
  testattr-vs-registered count check — endorsed; discovered by
  AR-11's own process failure class.
