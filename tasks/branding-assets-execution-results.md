# branding-assets Execution Results

Track-level execution results. Section 1 covers handoff T-B (extension
banner), gated APPROVED 2026-10-11 at landing commit `b7ba3a1`
(verdict `tasks/branding-assets-tb-verdict.md`). Handoff T-A (app side) has
not been gated yet; its addendum will be appended here at track closeout.
All lines below quoted verbatim from fresh reviewer-hop runs.

## T-B acceptance criteria vs actuals

| # | Criterion (handoff contract) | Actual + evidence line |
|---|---|---|
| 1 | Single landing commit inside the touch-set (EXACT) | `git log --oneline f23ca8a..HEAD` -> `b7ba3a1 feat(extension): README banner from canonical media + brand icon`; `git diff --name-only f23ca8a..HEAD` -> `extensions/ariadne-extension/README.md`, `extensions/ariadne-extension/media/banner.jpg`, `tasks/branding-assets-tb-journal.md` - no other path |
| 2 | I1 byte-identity banner.jpg == media/AT.jpg | `git hash-object media/AT.jpg extensions/ariadne-extension/media/banner.jpg` -> `2217a7bc25ae8e78fc80c8781fd41e303ada4d01` twice; runner line `### generic PASS (exit 0)` |
| 3 | I1 media/** zero-diff (source of truth) | `git diff --name-only f23ca8a..HEAD -- media ...` -> media absent from output; `f23ca8a` stat shows `media/AT.jpg Bin 0 -> 109528 bytes` |
| 4 | I2 encoding guard green with new blobs | `python tools/distill.py -- python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)`, zero violation lines; Risk R1 did not trip |
| 5 | I3 new strings ASCII + possessive | README line 3 = `![Ariadne's Thread](media/banner.jpg)`, straight ASCII apostrophe; exclusion grep `[^\x00-\x7F]` matches only pre-existing emoji/arrow lines (7, 14, 37, 60, 73, 87, 90, 113, 121, 127, 143), never an added line; journal: zero non-ASCII matches |
| 6 | I3 README additions-only lock | landing hunk `+2/-0` (`@@ -1,5 +1,7 @@`); zero `-` lines; emoji headers survive as unchanged context |
| 7 | I4 packaging validity (deferral branch) | DEFERRAL pair verified: `icon` grep on package.json (clean tree == HEAD) -> `No files found`; `extensions/ariadne-extension/media/` contains only `banner.jpg`; journal carries the dated deferral line; package.json has NO hunk in `b7ba3a1` |
| 8 | I5 zero collateral diff | name-only diff vs TRACK_BASE restricted to media + extensions/ariadne-extension + crates + tools + tests -> only README.md and media/banner.jpg; `.vsix`, `src/extension.ts`, `.vscodeignore`, `package-lock.json`, `crates/**`, `tools/**`, `tests/**`, ledger all zero-diff |
| 9 | Dirty-tree precondition cured at TRACK_BASE | `f23ca8a` lands the extension feature work as tracked content (README +35, package.json +23, src/extension.ts +263, dist/extension.js +225) - committed, not reverted; whole tree clean at re-dispatch |
| 10 | Rasterizer probe + sanctioned deferral | journal verbatim `ModuleNotFoundError: No module named 'PIL'`; deferral = completion per handoff C2 ("Handoff B is COMPLETE on the deferral path") |
| 11 | Gate-count discipline | overshoot declared as dated D5 row (~20 attempts, all read-only or declared) - accepted per ledger:81 rule |

## Workspace-wide acceptance gates (reviewer hop 2026-10-11, HEAD b7ba3a1)

- Workspace census (substitute for `testrun.py --baseline`, whose file form is
  the standing ledger:69 errata; reproduced verbatim this hop:
  `usage error: missing value for --baseline`, 6th encounter):
  `python tools/testrun.py --distill` ->
  `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 2263 lines]`
  canonical line: `1280 passed / 0 failed / 13 ignored / 91 suites`
- Tool unit suites (CI form): `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 219 tests in 22.568s` / `OK`
- UI runtime smoke (distiller route per ledger:70):
  `python tools/distill.py -- python tools/ui_smoke.py` ->
  `### generic PASS (exit 0)` /
  `boot+route checks clean (cold fixture, attempt 1, 0.7s)` / `ui_smoke: PASS`
- Hygiene: `python tools/distill.py -- python tools/hygiene_check.py` ->
  exit 1 identical to the pre-landing baseline; R1/R2 set exactly the inherited
  pair `crates/ui/src/server.rs: R2 1120 lines > 1000` and
  `crates/ui/src/server.rs:764: R1 fn \`api_graph_traversal\` is 161 lines (> 100)`;
  capture grep proves zero findings on this handoff's files; all other lines
  R3 warn-only
- Encoding: `encoding_guard --all` -> `### generic PASS (exit 0)`

## Explicitly not run

- `python tools/bench_compare.py` - EXEMPT, not run. Named precedent:
  ui-degree-opacity-ramp closeout row (ledger:41) "bench_compare exempt per
  zero-src-diff precedent (both verdicts)"; proof: diff `f23ca8a..b7ba3a1`
  contains no path under `crates/*/src` (data/assets/docs handoff only). The
  stale-baseline row (ledger:71) makes an off-premise run against an
  unrepaired baseline misleading; operator baseline refresh still owed there.
- `npm run compile` - declared handoff exemption: zero TypeScript diff in
  range; manifest surface untouched on the deferral path.
- `python tools/testrun.py --baseline` (file form) - unrunnable by the standing
  errata (6th encounter); `--distill` census substitute run above.

## Not reproducible pre-commit (recorded, not skipped)

- `encoding_guard --staged` with the new blobs staged - index state was
  consumed by the landing commit; the `--all` superset above is the arbiter
  (hotspot-canvas verdict gate-4 precedent, ledger:35). Journal carries the
  original gate 6 line: `### generic PASS (exit 0)`, zero violation lines.

## Open obligations relayed to track closeout

1. PNG marketplace icon re-derivation when a rasterizer is available (deferral
   recorded; `"icon"` correctly unset; ledger:43 "tb deferral = completion").
2. `.vsix` rebuild - operator duty (kickoff scope-out); stale tracked vsix
   verified zero-diff.
3. Journal post-commit tail (+19 lines uncommitted) - journal-rider commit due
   (precedents `5b4da58`, `7547dea`).
4. Memory mirror `ledger.milestone.branding-assets` + ledger row flip - owed at
   TRACK closeout in the same commit (ledger:43); NOT this hop: T-A pending,
   row stays PLANNED.

---

## T-A addendum (appended by the T-A reviewer hop, 2026-10-11, HEAD `ca5960f`)

Handoff T-A (app side: served brand icon), gated APPROVED at
`tasks/branding-assets-ta-verdict.md`. Chain over TRACK_BASE
`b7ba3a128a5af3a8f652ba96651c7f42f1183577`: `9ddf623` landing (11 files,
touch-set EXACT incl. the 2 declared tools/ riders), `1c9b466` journal tail,
`a1129eb` user-approved theme-token baseline regeneration (D5), `ca5960f` D5
journal rider. All lines below quoted verbatim from fresh reviewer-hop runs.

### T-A acceptance criteria vs actuals

| # | Criterion (handoff contract) | Actual + evidence line |
|---|---|---|
| 1 | Commit-vs-touch-set EXACT incl. riders | `git diff --name-only b7ba3a1..HEAD` -> 12 paths: the 11 touch-set entries EXACT + `tools/theme_token_baseline.json` (user-approved regen, verdict finding 2, ACCEPTED deviation); zero collateral anything else |
| 2 | I1 `/icon.svg` 200 + no-store + image/svg+xml + exact body | route test green: capture line 108 `test icon_asset::test_icon_route_serves_brand_art_with_no_store ... ok`; smoke row green (criterion 7) |
| 3 | I3 byte identity + media zero-diff | `git hash-object media/AT-svg.svg crates/ui/assets/icon.svg` -> `f05c2c0a8fa0cb18966ac3f568b4feeade771e5c` twice (`### generic PASS (exit 0)`); `git diff --name-only b7ba3a1..HEAD -- media ...` -> media absent; test 2 green (capture line 106) |
| 4 | I2 smoke green + `/icon.svg` row | `python tools/distill.py -- python tools/ui_smoke.py` -> `### generic PASS (exit 0)` / `boot+route checks clean (cold fixture, attempt 1, 0.7s)` / `ui_smoke: PASS`; grep: `tools/ui_smoke.py:70:     ("/icon.svg", "image/svg+xml", True),` |
| 5 | I4 token guard both scans | combined form `--check-added --base b7ba3a1...` -> `### generic PASS (exit 0)`, capture 0 lines (BOTH scans; scan 2 runs only in this form - code `theme_token_check.py:209-210`); gate-7 literal bare form -> `### generic FAIL (exit 2)` usage-error form by design (verdict finding 1); `themes.css` zero-diff; index.html/styles.css added zero hex (scan 1 silent) |
| 6 | I5/I7 ASCII + possessive | `encoding_guard.py --all` -> `### generic PASS (exit 0)` 0 lines; exclusion grep `[^\x00-\x7F]` zero matches in `icon_asset.rs` / `icon.svg` / journal; alt text `Ariadne's Thread logo` ASCII apostrophe, byte-equal across index.html / export.rs literal / test |
| 7 | I6 export closed-world + app page `<img>`-only | test 4 green (capture line 240): `neonGradient` inlined, zero of `/icon.svg`, `<img class="brand-mark"`, `<link rel="icon"`, `src="/app/`, `href="/styles.css"`, `href="/themes.css"`; test 3 green (line 112): page carries favicon + img, no bolt path, no `neonGradient`/`neonGlow` |
| 8 | I8 zero collateral + hygiene identity | hygiene exit 1 carrying ONLY the inherited server.rs pair: `crates/ui/src/server.rs: R2 1125 lines > 1000` + `crates/ui/src/server.rs:769: R1 fn \`api_graph_traversal\` is 161 lines (> 100)` (= 1120+5 / 764+5, journal claim matched); 9 server.rs warn lines at exactly baseline+5; zero finding lines for any track file; frozen locks (media, static_html.rs, static_html_sections.rs, 3 theme suites, other JS) diff-EMPTY |
| 9 | Frozen-count baseline regen (D5, user-approved) | `a1129eb` baseline-only commit; zero-diff proof `b7ba3a1..HEAD -- graph.js themes.css main.js traversal.js index.html` -> only index.html (both out-of-cure hunks TRACK_BASE-pre-existing); arithmetic verified from both blobs: old sum 286, new sum 296, `286+7+2+1+1-1=296`; ledger:107 justification rule + ledger:90 F3b disposition executed |
| 10 | Plan-literal C0-C7 | all before/after strings verbatim (git show per commit); sole reading-deviation D2 (test-2 path expression) ACCEPTED - handoff literal resolves outside the repo, machine-checkable acceptance reproduced: `238 passed / 0 failed / 0 ignored / 3 suites` = baseline 234 + 4 |
| 11 | Landing message + scoping | `9ddf623` message = handoff gate 10 verbatim; nine commit_scoped scopes per handoff; journal gates itemized (12 gate-class invocations, no undeclared overshoot) |

### Workspace-wide acceptance gates (T-A reviewer hop, fresh, HEAD ca5960f)

- Workspace census (substitute for the `--baseline` file form - 7th encounter
  reproduced verbatim: `usage error: missing value for --baseline`; standing
  errata ledger:69, decision still owed):
  `python tools/testrun.py --distill -- --workspace` ->
  `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 2270 lines]`
  canonical line: `1284 passed / 0 failed / 13 ignored / 91 suites`
  - reconciles EXACTLY: T-B anchor 1280 + the 4 new icon_asset tests; ignored
    13 and suites 91 held.
- Scoped ui suite: `python tools/testrun.py --distill -- -p ariadne-ui` ->
  `### cargo test PASS (exit 0) [capture: ...\target\testrun-capture.txt, 275 lines]` /
  `238 passed / 0 failed / 0 ignored / 3 suites`
- Tool unit suites (CI form): `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 219 tests in 20.747s` / `OK`
- UI runtime smoke: verbatim in criterion 4 above (raw form refused by the
  allowlist - sanctioned distiller route, fresh cold boot).
- Hygiene / encoding / token-guard: verbatim in criteria 5, 6, 8 above.

### Explicitly not run (T-A hop)

- `python tools/bench_compare.py` - EXEMPT, adjudicated NOT RUN with
  src-touch evidence (the zero-src-diff precedent alone does not cover this
  track: T-A touched `crates/ui/src/{lib,server,export}.rs`). Adjudication:
  bench_compare measures the stage-5 indexing tail only - it runs the ignored
  benches `crates/cli/tests/payload_tail_bench.rs` / `stage5_memory_bench.rs`
  and compares their pipeline phase medians (`bench_compare.py:6-17,64-69`);
  none of those crates is touched by the track. Proof:
  `git diff --name-only b7ba3a1..HEAD -- crates/cli crates/pipeline crates/index crates/graph crates/repo crates/lang* crates/build crates/scheduler crates/transform crates/validate crates/report tests fixtures` -> EMPTY. The sole Rust delta is `+5` router-table lines + a 3-line asset handler (runs only on `GET /icon.svg`, never in a bench phase) and one `include_str!` const (+2129 bytes of rodata in the ui crate). This matches the handoff's own exemption clause ("no pipeline/Rust hot-path producer changes beyond the ui crate's own serving; reviewer confirms via diff paths") - confirmed via diff paths. The stale-baseline flap row (ledger:71) further makes an off-premise run misleading; operator refresh still owed there.
- `python tools/testrun.py --baseline` (file form) - unrunnable by the standing
  errata (7th encounter); `--distill` census substitute run above.
- `npm run compile` and extension gates - T-B territory, unchanged, already
  reported in the T-B section above.

### Track-state summary after both handoffs

- T-B: APPROVED at `b7ba3a1` (verdict above); T-A: APPROVED at chain
  `9ddf623..ca5960f` (verdict `tasks/branding-assets-ta-verdict.md`).
- Open for closeout: ledger:43 flip + dated baseline-regen row + dated HOLD-lift
  note + memory mirror `ledger.milestone.branding-assets` (planner, same
  commit); operator eyeball (brand mark ~32px, favicon tab, all four shipped
  themes); `.vsix` rebuild; PNG icon rasterizer follow-up; `--baseline`
  file-form errata decision; F3a entity-misread regex fix and server.rs
  extraction ride their existing deferred rows.
