# AR-01 Journal - Path-traversal containment + export script escape

Handoff: `tasks/ar-01-handoff.md` (S1 CRIT, S3 HIGH, S4-partial MED).
Executed strict Red-Green-Refactor. All authored content pure ASCII.

## What was done

### S1 CRIT - `/api/source` containment (`crates/ui/src/server.rs`)

- `api_source` now canonicalizes BOTH ends (`repo_root` and the candidate
  path built from `file=` / `local_prefix=`) and requires
  `contained.starts_with(canonical_root)` before reading.
- Absolute `file=` values, `..` traversal, and escaping `local_prefix=`
  resolve outside the canonical root and are rejected with 403 FORBIDDEN.
- Canonicalization failure = missing path: unchanged 404 (contained
  missing-file reads behave exactly as before).
- `repo_root` itself failing to canonicalize is 500 (fail-closed; the
  server cannot prove containment without a canonical root).
- Bonus from `canonicalize` semantics: in-repo symlinks pointing outside
  the root are also contained (the resolution happens before the check).
- `local_prefix` branch kept its exact path-construction; the containment
  applies to its result ("sandbox prefix into repo" per the handoff
  option b). The legitimate mapping case (prefix == server root, file =
  absolute server path) stays 200 byte-identical, locked by test.

### S3 HIGH - export JSON script escape (`crates/ui/src/export.rs`)

- New private helper `escape_json_for_script` post-serializes
  `<` -> `\u003c`, U+2028 -> `\u2028`, U+2029 -> `\u2029` in the
  `window.__ARIADNE_DATA__` payload of `export_standalone_html`.
- Sound because none of the three can appear in JSON structural syntax
  (only inside string values), and the introduced escape sequences contain
  no further matchable characters: single-pass substitution is total.
- Parse-identical: round-trip asserted in the lock test. Clean payloads
  are byte-identical to the pre-fix serialization (locked).

### S4-partial MED - attribute-safe helper (`crates/ui/assets/hotspots.js`)

- Added `escapeAttribute(value)` (escapes `& < > " '`), the shared-renderer
  helper named by the handoff objective.
- Wired at the two payload-derived HTML attributes in the extraction card
  renderer: `data-preview-root` / `data-preview-source` (strictly stronger
  than the previous `escapeHtml` call: adds `'`).

### Lock tests (new mod `crates/ui/tests/ui_test/source_containment.rs`)

8 tests: contained byte-identical read; absolute-path reject; `../`
traversal reject; escaping `local_prefix` reject (each reject also asserts
no outside-repo secret leaks in the body); prefix==root mapping still 200;
export hostile fixture (raw `<` / U+2028 / U+2029 absent, round-trip equal);
export clean-payload byte-identical rider; `escapeAttribute` ships in
HOTSPOTS_JS. Hostile separators built from byte values / Unicode escapes
(pure-ASCII source rule); query params percent-encoded so Windows path
separators survive the URI parser.

## TDD evidence

- Red: `237 passed / 5 failed / 0 ignored / 2 suites` - the 5 failures were
  exactly the predicted semantic ones: the three S1 escapes answered
  `left: 200` (vulnerable code actually served the outside-repo secrets),
  export carried a raw `<`, helper absent.
- Green: `242 passed / 0 failed / 0 ignored / 3 suites`
  (baseline 234 + 8 new locks; capture `target/testrun-capture.txt`).
- Refactor pass: helpers already extracted at Red time
  (`encode_query_component`, `assert_no_outside_content_leaked`,
  `escape_json_for_script`); no further change needed.

## Deliberately NOT done

- Hotspots inline `onclick="...('${id}')"` sites (the other S4 carriers):
  unfixable by ANY escaper, because HTML entity-decoding restores quotes
  before the JS parser evaluates the attribute. The correct fix is the
  data-id + addEventListener refactor owned by AR-11 (plan says
  verbatim). Left untouched.
- `file-view.js` untouched: its repo-derived values reach the DOM via
  `textContent` or the text-node `escapeHtml`; it has no applicable
  attribute interpolation for the helper ("if applicable" was false).
  Quote-extension of `escapeHtml` is AR-11 per the plan.
- No `escapeHtml` unification across assets (AR-11 territory).

## Deviations (with reason)

1. Locks live in a new mod `source_containment.rs`, not in
   `server_api.rs`: that file is 942 lines; ~130 lock lines would cross
   the R2 1000-line blocking ceiling. The handoff explicitly allows
   "(or new mod)". Registering it required a ONE-LINE amendment to
   `crates/ui/tests/ui_test/main.rs` - outside the hard touch-set,
   implied-allowed by the "(or new mod)" option; flagged for reviewer.
2. Hygiene gate exits 1 at baseline and after: the two pre-existing
   violations are both in `server.rs` (R2 1120 -> 1137 lines, R1
   `api_graph_traversal` 161 lines), already recorded in the remediation
   plan census and owned by AR-13's decomposition. AR-01 added ZERO new
   R1/R2/R3 entries (verified by diffing the violation sets before/after;
   only line numbers of pre-existing R3 warns shifted). The handoff gate
   "hygiene exit 0" is unreachable until AR-13 lands and was pre-known
   false at dispatch (plan census says hygiene FAIL exit 1).
3. `ui_smoke` ran through `python tools/distill.py -- python tools
   ui_smoke.py` (direct invocation is not on the bash allowlist);
   `ui_smoke: PASS`.

## Gates (acceptance commands)

- `python tools/testrun.py --distill -- -p ariadne-ui` ->
  `242 passed / 0 failed / 0 ignored / 3 suites`
- `python tools/ui_smoke.py` (via distill) -> `ui_smoke: PASS`
- `python tools/hygiene_check.py` -> unchanged pre-existing set only
- `python tools/encoding_guard.py --staged` -> see commit transcript
