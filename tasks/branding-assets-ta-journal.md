# T-A journal - branding-assets: served brand icon in the web app (crates/ui)

Implementer journal per `tasks/branding-assets-ta-handoff.md`. All gate output
is quoted from the distilling entry points; captures are under `target/`.

## Step 0 (2026-10-11)

- TRACK_BASE b7ba3a128a5af3a8f652ba96651c7f42f1183577 (HEAD at dispatch;
  recorded via `python tools/distill.py -- git rev-parse HEAD`).
- Step 0 status via `python tools/distill.py -- git status --porcelain`:
  three entries, ALL outside the touch-set and all handoff-B (T-B) residue:
  `M tasks/branding-assets-tb-journal.md`, `?? tasks/branding-assets-execution-results.md`,
  `?? tasks/branding-assets-tb-verdict.md`. Zero dirty files inside the T-A
  touch-set -> no STOP. They are left untouched (I8; commit scoping excludes them).
- Anchor verification (HOLD was lifted, not cured; AR-13 has NOT landed -
  every anchor is at its authored location):
  - `server.rs` `build_router` at line 129; `.route("/themes.css", get(asset_themes_css))`
    at 133; `ui_asset()` at 173; `asset_themes_css` handler at 189-191.
  - `lib.rs` const list ends at `THEME_JS` (line 20).
  - `export.rs` `use crate::{...}` lists `HOTSPOTS_JS,` / `INDEX_HTML,` adjacently
    (line 3); `standalone_template()` at 12; STYLES_CSS replace block ends at 22;
    JS loop starts at 23.
  - `index.html` lines 21-22 stylesheet links; `.logo-badge` bolt block at 38-42.
  - `styles.css` `.logo-badge` 63-72; `.logo-badge svg` rule 73-77 (4-space
    indentation as in C4).
  - `tools/ui_smoke.py` `ROUTE_ASSERTIONS` at 61-70; `tools/theme_token_check.py`
    `TOKEN_FILES = ("themes.css",)` at 40 with comment block 37-39.
  - `main.rs` roster: `mod hotspot_direction;` (14) / `mod justification_builders;`
    (15) - alphabetical insertion site confirmed.
  - Common helpers exist: `common::loading_state` (common.rs:229),
    `common::request_raw_body` (343), `common::empty_export_payload` (196).
- `media/AT-svg.svg`: 55 lines, pure ASCII, blob hash
  f05c2c0a8fa0cb18966ac3f568b4feeade771e5c (`git hash-object` via distiller).
  Byte size recorded at the C0 step below.
- `crates/ui/src/server.rs` measured 1120 physical lines (Read tool total),
  matching the kickoff's inherited breach claim.

### Gate 1 - Step 0 baseline testrun

Command: `python tools/testrun.py --distill -- -p ariadne-ui`
Digest: `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 271 lines]`
Canonical line: `234 passed / 0 failed / 0 ignored / 3 suites`
BASELINE FAILING SET (pre-existing): EMPTY (distiller verdict line: `no failures`).

### Gate 2 - Step 0 hygiene baseline

Command: `python tools/hygiene_check.py` (exit 1 expected while the inherited
server.rs blocking pair stands; exit-1 semantics read from hygiene_check.py:707).
BLOCKING findings (R1/R2) - the complete blocking set, verbatim:

```
crates/ui/src/server.rs: R2 1120 lines > 1000
crates/ui/src/server.rs:764: R1 fn `api_graph_traversal` is 161 lines (> 100)
```

R3 warnings (warn-only, NOT blocking; baseline set for identity): 173 warn
lines (measured count from the complete gate-2 output) spread over crates/cli, crates/build, crates/cluster, crates/graph,
crates/lang-api, crates/lang-cpp, crates/lang-scala, crates/repo,
crates/scheduler, crates/transform, crates/ui (src/dto.rs, src/server.rs
lines 414/484/519/531/540/544/616/726/1008, tests/ui_test/graph_api.rs,
justification_builders.rs, justification_golden.rs, server_api.rs,
theme_contract.rs, theme_js_zero_hex.rs), crates/validate, tasks/*.py probes,
tests/integration. No new-file R3 line may appear from this handoff's files
(`icon_asset.rs`); the inherited server.rs R3 warn lines shift line numbers by
the insertion delta - identity is by file+rule+identifier.

Expected growth claim: this handoff adds exactly +5 physical lines to
server.rs (route +1, separator blank +1, handler +3), so at Green the R2
message must read `R2 1125 lines > 1000` and the R1 finding must still be
fn `api_graph_traversal` at line 769 (764 + 5).

## Red (gate 3)

Command: `python tools/testrun.py --distill -- -p ariadne-ui` (timeout 600000 ms)
First invocation after writing the test module hit DRIFT (exit 2): two E0716
temporary-lifetime compile errors in the new test file (borrowed `&format!` /
`&(...to_string())` paths outlived their temporaries). Not a Red; fixed the
harness in-file with owned `PathBuf` bindings (same paths, d27 `join` idiom)
and re-ran. Re-run result:

Digest: `### cargo test FAIL (exit 1) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 293 lines]`
Canonical line: `234 passed / 3 failed / 0 ignored / 2 suites`
Failing set = BASELINE (empty) UNION exactly the three named Red tests, each
first-failing at the predicted assertion:
- `icon_asset::test_icon_asset_matches_source_of_truth` - missing
  `crates/ui/assets/icon.svg` existence assert (message names the brand asset);
- `icon_asset::test_page_head_marks_favicon_and_brand_img` - missing favicon
  line on INDEX_HTML (first assert);
- `icon_asset::test_standalone_export_closes_icon_references` - missing
  `neonGradient` inlining in the export (first assert).
Observation: the suite count reads 2 because the distiller's suite count
counts OK-result lines and `ui_test` printed FAILED; exit 1 (FAIL), not exit 2
(DRIFT) - the parse stayed healthy.

## Green (gates C0 + 4a/4b)

C0 byte copy (deviation D1 form - single-purpose script
`ta_c0_copy_icon.py` via `python tools/distill.py -- python <script>`):
`source_bytes 2129 / target_bytes 2129`. Byte identity proof:
`python tools/distill.py -- git hash-object media/AT-svg.svg crates/ui/assets/icon.svg`
printed `f05c2c0a8fa0cb18966ac3f568b4feeade771e5c` twice (equal; also equal to
the Step 0 source-of-truth hash). `media/AT-svg.svg` byte size: **2129**.

Gate 4a (without route test 1): `### cargo test PASS (exit 0) [capture:
...\target\testrun-capture.txt, 275 lines]` / `237 passed / 0 failed / 0
ignored / 3 suites` - failing set empty = BASELINE; passed = baseline + 3.
Gate 4b (test 1 added): `### cargo test PASS (exit 0) [capture:
...\target\testrun-capture.txt, 276 lines]` / `238 passed / 0 failed / 0
ignored / 3 suites` - failing set name-identical to BASELINE (empty); passed
= baseline + 4, exactly as required. No assertion was weakened.

Refactor: re-read of `icon_asset.rs` (106 lines) - honest names, no duplication
worth extracting, no banned identifiers; zero code change since gate 4b, so
the gate-4b result carries the Refactor re-run.

## Smoke (gate 5)

Command: `python tools/distill.py -- python tools/ui_smoke.py`
Digest: `### generic PASS (exit 0)`; verdict lines
`boot+route checks clean (cold fixture, attempt 1, 0.7s)` and `ui_smoke: PASS`
- the table now carries the `("/icon.svg", "image/svg+xml", True)` row and the
probe asserted it green.

## Theme-token guard (gates 6+7) - BLOCKER, gate-7 spec defect (deviation D3)

Gate 6 command run as specified:
`python tools/theme_token_check.py --check-added --base b7ba3a128a5af3a8f652ba96651c7f42f1183577`
(via the distiller for the exit-code header). Result exit 1, four violations:
- added-literal scan (scan 1): ZERO violations - the C7 rider worked
  (icon.svg exempted; index.html/styles.css added no hex).
- frozen-count scan (scan 2): `crates/ui/assets/icon.svg: 7 hex literals
  exceeds frozen baseline count 0` PLUS THREE PRE-EXISTING violations on
  untouched files: `index.html: 6 ... count 4`, `main.js: 1 ... count 0`,
  `traversal.js: 1 ... count 0`.

Gate 7 literal form `python tools/theme_token_check.py` exits 2 by the
script's design (usage error; recorded verbatim via the distiller header
`### generic FAIL (exit 2)` + usage text). The frozen-count scan only runs
INSIDE `--check-added --base <rev>` (theme_token_check.py main():209-210), so
gates 6 and 7 collapse into the single gate-6 invocation above.

Measurements (recorded, not guessed) that falsify the handoff's premise:
- `scan_frozen_counts` (lines 139-150) has NO `TOKEN_FILES` branch - the C7
  rider exempts icon.svg from scan 1 ONLY. Shape R2's parenthetical
  "`--update-baseline` NOT needed ... exempted from frozen counts too" is
  false against the shipped guard code.
- The frozen-count scan was ALREADY RED at TRACK_BASE: the committed baseline
  records `generated_from_head: d76a559` (stale), pins `index.html` at 4 and
  `main.js` at 0, and has no `traversal.js` key (defaults 0); the TRACK_BASE
  tree already carries 6/1/1 literals. The 3-file violations are untouched by
  this handoff (zero-diff; Step 0 status proves it). The ONLY violation this
  commit adds is the icon.svg one, which the shape promised would be covered
  by the rider and is not.

Disposition per discipline ("out-of-scope breakage: STOP-report, do not fix"):
NO improvisation. The only compliant remedies - `--update-baseline` (forbidden
by the handoff gates AND by the standing rule that theme-token baseline
regeneration needs explicit approval) or a guard/scan change (outside the
touch-set; the reviewer hop repeats "never edit the guard") - are orchestrator
decisions, not implementer choices. The green work is committed (gate 10)
with this evidence so the reviewer hop adjudicates gate 6/7 acceptance
against the measured guard behavior.

## Hygiene (gate 8)

Command: `python tools/distill.py -- python tools/hygiene_check.py`
Digest: `### generic FAIL (exit 1)` (expected: the inherited pair stands).
Capture: `target/distill-capture.txt` (175 lines = 2 blocking + 173 warn).
BLOCKING: `crates/ui/src/server.rs: R2 1125 lines > 1000` (= 1120 + the
journal's exact +5 claim) and `crates/ui/src/server.rs:769: R1 fn
\`api_graph_traversal\` is 161 lines (> 100)` (= 764 + 5). Warn-set identity:
all 9 server.rs warn lines at exactly baseline+5 (419/489/524/536/545/549/
621/731/1013); the other 164 warn lines live in files with zero diff;
grep of the capture shows ZERO finding lines for `icon_asset.rs`,
`crates/ui/src/lib.rs`, `export.rs`, or the two rider files. Finding-set
identity vs gate 2: CONFIRMED (file+rule+identifier; no new file/rule).

## Deviations declared

- D1 (host hazard, T-B learned): `python -c` forms are refused by the
  permission layer in this session, both raw and via `tools/distill.py`
  (4 refusals observed: 2 raw, 2 distilled, incl. single-statement forms).
  The C0 copy therefore runs as a single-purpose byte-level Python script
  (authored pure-ASCII via the Write tool under the temp dir, invoked as
  `python tools/distill.py -- python <script>`), which preserves the C0
  contract (byte-level Python, never a PowerShell text pipe, digest-only
  output). Byte identity is still proven by `git hash-object` equality plus
  C6 test 2.
- D2 (handoff internal contradiction, adjudicated to the machine-checkable
  acceptance): C6 test 2's literal path expression
  `format!("{}/../assets/icon.svg", env!("CARGO_MANIFEST_DIR"))` resolves to
  `crates/assets/icon.svg` (outside the repo) for an integration test whose
  manifest dir is `crates/ui`; the same test's named file, its message text,
  the Red-phase expectation, and invariant I3 all name `crates/ui/assets/icon.svg`.
  The path expression is implemented as `{}/assets/icon.svg` (matching the
  d27 precedent `.join("assets/provenance.js")` in the same suite) while the
  source-of-truth path `{}/../../media/AT-svg.svg` is kept verbatim (it is
  correct).   Without this reading the Green acceptance (all four tests green,
  passed = baseline + 4) is unreachable. Flagged for reviewer adjudication.
- D3 (gate 6/7 acceptance defect, see the gates section): the frozen-count
  scan is not TOKEN_FILES-exempt (shape R2 premise false), gate 7's literal
  command is a usage-error form (exit 2 by script design) and the scan it
  names only runs under `--check-added`, and the committed baseline was
  already red at TRACK_BASE for three untouched files. No compliant in-scope
  remedy exists (baseline regeneration forbidden + needs explicit approval;
  guard edits out of scope). STOP-report for orchestrator/reviewer
  adjudication; the code itself is fully green.

## Gate count ledger (implementer hop)

1. testrun --distill baseline (gate 1) - 1x. 2. hygiene_check baseline
(gate 2) - 1x. 3. Red testrun - 2x (first = DRIFT exit 2, compile fix
declared above; second = the recorded Red). 4. Green testrun - 2x (4a/4b).
5. distill ui_smoke - 1x. 6/7. theme_token_check - 2x (combined form +
literal gate-7 form). 8. hygiene - 1x. 9. encoding_guard --staged - 1x.
10. commit_scoped - 1x (see landing section). Extras (non-gate):
rev-parse, status, hash-object x3, C0 script, line-count reads. Total
gate-class invocations: 12 - within bound once the two declared re-runs are
counted honestly; no D-row overshoot beyond those.

## Landing (gate 9 + gate 10)

- Gate 9: `python tools/encoding_guard.py --staged` via the distiller - exit 0,
  silent (0-line capture). Staging used explicit-path
  `git add -- <the 11 touch-set files>` via the distiller (no `git add .`).
  autocrlf emitted the repo-standard working-copy notices for 6 files;
  committed blobs are the normalized (LF) content, identical for the asset
  pair since normalization is content-deterministic.
- Gate 10: `python tools/commit_scoped.py` with the handoff's exact nine
  scopes and message - landed `9ddf623`
  "feat(ui): serve brand icon at /icon.svg with favicon and brand mark".
  Stat: icon.svg +55, index.html +2/-3, styles.css +4/-4, export.rs +13/-1,
  lib.rs +1, server.rs +5, icon_asset.rs +106, main.rs +1, journal +223,
  theme_token_check.py +4/-1, ui_smoke.py +1. Set equals the Touch-set EXACT
  (10 files + this journal), riders included, zero collateral.
- Post-commit scoped testrun: NOT re-run - zero working-tree delta between
  gate 4b and the commit (staging only); gate 4b's
  `238 passed / 0 failed / 0 ignored / 3 suites` carries the landing state.
- Human-eyeball rider (kickoff acceptance): MANUAL EYEBALL PENDING OPERATOR -
  this implementer session cannot open a browser. Not checked off: header
  brand mark at ~32px; browser-tab favicon; brand mark render under light /
  dark / dark-grey / dark-steampunk (fixed art by design; the fourth theme
  shipped via the ui-dark-grey-theme track). An unexecuted check is never
  checked off.
- Out-of-scope observations (report-only): (a) `index.html <title>` still
  reads "AriadnesThread Graph Visualizer" - pre-existing, out of handoff
  scope per the explicit out-of-scope section; (b) `--brand-grad-a/b` remain
  defined in themes.css behind the now-opaque icon - orphan cleanup is the
  kickoff's scope-out; (c) the ui_smoke.py docstring line "six /app/*.js
  scripts" has long been drifted from the roster - known relay, untouched.
- Frozen-lock zero-diff proof (single scoped invocation
  `git diff --name-only b7ba3a1 -- media themes.css main.js static_html.rs
  static_html_sections.rs theme_contract.rs theme_js_zero_hex.rs
  theme_dark_grey_palette.rs theme_token_baseline.json`): EMPTY output.
  Committed-set proof `git diff --name-only b7ba3a1..HEAD`: the 11 touch-set
  files exactly, nothing else.

## Deviation D4 (tail journal commit)

- The Landing section records the landing commit's own hash, so it cannot
  physically live inside `9ddf623`. It lands as one journal-only scoped
  commit (`--scope tasks/branding-assets-ta-journal.md`), declared here
  against the handoff's "one landing commit" parenthetical: one scoped commit
  per logical step (code landing / journal landing), never `git add .`,
  scope set unchanged.

## D5 (D6-style rider): user-authorized theme-token baseline regeneration

(Orchestrator asked for a "D6-style" rider row; D1-D4 were consumed by the
landing hop, so this lands as D5 - the next free number.)

- Trigger: the D3 STOP-report. The rider clause (verify the regen diff
  contains ONLY the four approved count updates, else STOP with the diff)
  fired on two extra hunks; the commits were held until disposition.
- Authorization chain: user explicitly approved regenerating the
  frozen-count baseline via `--update-baseline` (NOT a guard-code amendment),
  with the dated ledger row owned by the planner (the implementer did not
  touch `tasks/milestone_ledger.md`). The orchestrator's disposition for the
  two out-of-cure hunks, cited verbatim: "user's approval covered a holistic
  `--update-baseline` regeneration; your zero-diff evidence proves both extra
  hunks (graph.js 1->0, themes.css:188 value) are TRACK_BASE-pre-existing
  stale-pin corrections, not this track's content; a hand-narrowed JSON would
  ship counts contradicting locations -- precisely the drift class the guard
  exists to catch." (em dash transliterated to `--` for the pure-ASCII rule.)
- Regeneration (correct form from parse_mode, theme_token_check.py:188-190):
  `python tools/theme_token_check.py --update-baseline` -
  `baseline written: 13 files, 296 literals, 296 locations` (11 pinned files
  plus new keys `traversal.js`, `icon.svg`). Approved-cure counts exactly as
  predicted: index.html 4->6, main.js 0->1, traversal.js unkeyed->1,
  icon.svg 0->7. Stale-pin corrections (zero-diff-proven TRACK_BASE-pre-
  existing; `git diff --name-only b7ba3a1..HEAD -- graph.js themes.css
  main.js traversal.js index.html` printed ONLY `index.html`): graph.js 1->0
  (`#8250` now lives at main.js:308), themes.css:188 value `#15803d`->
  `#166534` (count stays 281), `generated_from_head` d76a559->1c9b466, and
  mechanical index.html location re-indexing. Arithmetic: 286+7+2+1+1-1=296.
- Landing: baseline-only scoped commit `a1129eb`
  "fix(tools): regenerate theme-token baseline (branding-assets rider,
  user-approved; stale-pin corrections graph.js 1->0 and themes.css:188)"
  (24/12), preceded by `encoding_guard.py --staged` exit 0; no amend of
  `9ddf623`/`1c9b466`.
- Post-fix acceptance (D3 closure), verbatim at HEAD a1129eb -
  `python tools/theme_token_check.py --check-added --base
  b7ba3a128a5af3a8f652ba96651c7f42f1183577` via the distiller:
  `### generic PASS (exit 0)`, capture 0 lines (both scans silent; the bare
  gate-7 form remains the usage-error form, exit 2 by script design).
  Scoped suite unchanged: `238 passed / 0 failed / 0 ignored / 3 suites`.
- Rider-hop gate tally: status 2, add 2, encoding_guard --staged 2, regen 1,
  combined scan 2, zero-diff proof 1, testrun 1, commit_scoped 2 - 13
  invocations, no gate-class overshoot beyond the per-commit hygiene pairs.
