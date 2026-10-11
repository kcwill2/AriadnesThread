# branding-assets T-A Verdict

Reviewer hop: 2026-10-11 (fresh session, review-gate skill).
Handoff: `tasks/branding-assets-ta-handoff.md`. Journal: `tasks/branding-assets-ta-journal.md`
(includes the D5 baseline-regeneration rider row; the D6 numbering skip is
deliberate and self-explained in the D5 row).
TRACK_BASE `b7ba3a128a5af3a8f652ba96651c7f42f1183577`. Chain over base:
`9ddf623` (landing, 11 files = touch-set EXACT incl. both declared riders) ->
`1c9b466` (journal tail + D4) -> `a1129eb` (user-approved baseline regen,
D5) -> `ca5960f` (D5 journal rider, HEAD).

## VERDICT: APPROVED

Zero blockers. Numbered findings below are all minor/relay.

## Gate record (all run fresh this hop, HEAD `ca5960f`)

| Gate | Result |
|---|---|
| Ground truth (`checkpoint_guard.py status`) | HEAD `ca5960f`, main; dirty set = T-B residue only (tb-journal +19, tb-verdict, execution-results untracked) - none inside the T-A touch-set |
| Per-commit `git show` oldest-first | 4 commits reviewed individually; see per-commit notes below |
| Commit-vs-touch-set (`git diff --name-only b7ba3a1..HEAD`) | 12 paths = the 11 touch-set entries (10 files + journal) EXACT, incl. DECLARED RIDER 1 `tools/ui_smoke.py` (+1 tuple at `ROUTE_ASSERTIONS` tail, line 70) and DECLARED RIDER 2 `tools/theme_token_check.py` (`TOKEN_FILES += "icon.svg"` + C7 comment verbatim), PLUS one adjudicated extra: `tools/theme_token_baseline.json` (user-approved regen, finding 2) |
| Frozen locks zero-diff (`git diff --name-only b7ba3a1..HEAD -- <paths>`) | `media/**`, `static_html.rs`, `static_html_sections.rs`, `theme_contract.rs`, `theme_js_zero_hex.rs`, `theme_dark_grey_palette.rs`, and all other app JS (api/file-view/preview/provenance/hotspots/theme) - all EMPTY; sole hit in the probed set was `styles.css` (in-touch-set, C4 replacement) |
| Stale-pin zero-diff proof (D5) | `git diff --name-only b7ba3a1..HEAD -- graph.js themes.css main.js traversal.js index.html` -> ONLY `crates/ui/assets/index.html`; graph.js and themes.css provably untouched by the track, so their baseline hunks are TRACK_BASE-pre-existing stale pins |
| I3 byte identity (fresh) | `git hash-object media/AT-svg.svg crates/ui/assets/icon.svg` via distiller -> `f05c2c0a8fa0cb18966ac3f568b4feeade771e5c` twice, EQUAL and equal to the journal Step-0 hash |
| Scoped ui suite fresh | `python tools/testrun.py --distill -- -p ariadne-ui` -> `### cargo test PASS (exit 0) [capture: target\testrun-capture.txt, 275 lines]` / `238 passed / 0 failed / 0 ignored / 3 suites`; capture lines 106/108/112/240: all four `icon_asset` tests `ok`; failing set EMPTY = journal BASELINE; passed = 234 + 4 exactly |
| I2 smoke fresh (undistilled form refused by allowlist; sanctioned distiller route per ledger:70) | `python tools/distill.py -- python tools/ui_smoke.py` -> `### generic PASS (exit 0)` / `boot+route checks clean (cold fixture, attempt 1, 0.7s)` / `ui_smoke: PASS`; table carries `("/icon.svg", "image/svg+xml", True)` at `tools/ui_smoke.py:70` |
| I1 serving (runtime, two independent oracles) | route test green in-suite (200 + `Cache-Control: no-store` + `Content-Type: image/svg+xml` + body == `ICON_SVG`); smoke row green |
| I4 token guard (fresh at HEAD) | `python tools/distill.py -- python tools/theme_token_check.py --check-added --base b7ba3a1...` -> `### generic PASS (exit 0)`, capture 0 lines - BOTH scans silent (scan 2 only runs inside this form; code-verified `theme_token_check.py:209-210`). Gate-7 literal bare form re-run: `### generic FAIL (exit 2)` + usage text - usage-error form by script design (`parse_mode`:188-195), finding 1 |
| I8 hygiene (fresh) | `python tools/distill.py -- python tools/hygiene_check.py` -> exit 1, capture 175 lines; BLOCKING = ONLY the inherited pair: `crates/ui/src/server.rs: R2 1125 lines > 1000` (= 1120 + the journal's +5 claim) and `crates/ui/src/server.rs:769: R1 fn \`api_graph_traversal\` is 161 lines (> 100)` (= 764 + 5). Warn identity: exactly 9 server.rs R3 lines at 419/489/524/536/545/549/621/731/1013 = baseline+5; grep proves ZERO finding lines (any rule) for `icon_asset.rs`, `lib.rs`, `export.rs`, `icon.svg`, `index.html`, `styles.css`, or the two rider files. Finding-set identity CONFIRMED |
| I7/encoding | `python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)`, 0 lines; exclusion grep `[^\x00-\x7F]`: zero matches in `icon_asset.rs`, `icon.svg`, `ta-journal.md`; the two added index.html lines and the export.rs comment block are pure ASCII (diff-verified); possessive `Ariadne's Thread` in the alt text (ASCII apostrophe), matching the C5 literal byte-for-byte |
| I6 export closed-world | C5 code matches the contract verbatim (both replace literals byte-equal to the C3 markup; favicon link dropped, `ICON_SVG.trim_end()` inlined at the img site); `test_standalone_export_closes_icon_references` green proves `neonGradient` inlined and zero of `/icon.svg`, `<img class="brand-mark"`, `<link rel="icon"`, `src="/app/`, `href="/styles.css"`, `href="/themes.css"`; app page keeps `<img>` (test 3 green: page contains no `neonGradient`/`neonGlow`) |
| C1-C7 literal compliance | C1 const line, C2 route + handler, C3 both edits, C4 rule replacement, C5 import site + both replaces, C6 four tests + doc style, C7 rider - all verbatim against the handoff except the D2 path-expression reading (disposition below); `mod icon_asset;` at the true alphabetical site (hotspot_direction / justification_builders) |
| Landing-commit message | `feat(ui): serve brand icon at /icon.svg with favicon and brand mark` - handoff gate 10 message verbatim; nine `commit_scoped` scopes match the handoff's nine |

## Workspace acceptance gates (this reviewer hop; verbatim lines)

- `python tools/testrun.py --baseline` -> `usage error: missing value for --baseline`
  - 7TH encounter of the standing errata (ledger:69); sanctioned substitute ran below.
- Workspace census: `python tools/testrun.py --distill -- --workspace` ->
  `### cargo test PASS (exit 0) [capture: target\testrun-capture.txt, 2270 lines]`
  canonical line: `1284 passed / 0 failed / 13 ignored / 91 suites`
  - reconciles EXACTLY: T-B-hop anchor `1280 / 0 / 13 / 91` + the four new `icon_asset`
    tests; ignored 13 and suites 91 held.
- Tool unit suites (CI form): `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 219 tests in 20.747s` / `OK` (exit 0, direct form)
- UI runtime smoke: verbatim in the gate table above.
- `python tools/bench_compare.py` -> EXEMPT, adjudicated not run WITH src-touch
  evidence (this hop touches `crates/*/src`, so the zero-src-diff precedent alone
  does not apply; adjudication in the "Explicitly not run" section of
  `tasks/branding-assets-execution-results.md`).

## Refused commands (verbatim catch-all, per running-gates duty)

All refusals reported the boilerplate
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`:

1. `git diff --name-only <range> -- <paths>; echo ...; git diff <range> -- <more>` (chained) - refused; split, the `git diff` halves PASSED standalone (the chained form is the known ledger:70 failure mode, matching the M26 V3 recurrence).
2. `git hash-object media/AT-svg.svg crates/ui/assets/icon.svg` (bare) - refused; PASSED via the sanctioned `tools/distill.py` route with identical output.
3. `python tools/ui_smoke.py` (raw undistilled) - refused; PASSED via the sanctioned distiller route (fresh cold-boot process; the distiller bounds output only).

No gate was skipped: every expected gate ran fresh or was replaced by the
sanctioned equivalent route. Recurrence of the standing allowlist row
(ledger:70) - relayed as finding 5, not an implementer issue.

## Deviation dispositions (independent adjudication)

- HOLD LIFT (dispatch precondition "waits for AR-13", ledger:43): ACCEPTED as a
  recorded deviation, not a lock breach. The lift came by explicit user
  instruction ("Dispatch T-A now"), not implementer improvisation. The HOLD's
  purpose (anchor drift from the AR-13 split) is moot: the journal Step-0 row
  records re-verification of every verbatim anchor at its authored location on
  a tree clean inside the touch-set (AR-13 has NOT landed), and the `9ddf623`
  diff confirms every anchor site matched (route inserted after the
  `/themes.css` line at the 131-133 roster, handler after `asset_themes_css`
  at 191). The planner's closeout row should date the lift.
- D1 (permission-layer refusal of `python -c` shapes; C0 ran as a
  single-purpose byte-level script via the distiller): ACCEPTED. The C0
  contract (byte-level Python, never a PowerShell text pipe, digest-only
  output) is preserved; I independently re-proved the byte identity with a
  fresh `git hash-object` equality, and C6 test 2 remains the machine lock in
  the suite. Same host hazard the T-B hop recorded (tb-verdict finding 3,
  ledger:70).
- D2 (C6 test-2 path expression `{}/../assets/icon.svg` implemented as the
  `{}/assets/icon.svg` join idiom): ACCEPTED. Independent verification: for an
  integration test with `CARGO_MANIFEST_DIR = crates/ui`, the handoff's literal
  resolves to `crates/assets/icon.svg` - outside the repo, can never exist.
  The handoff is internally inconsistent: its named file, its assert message,
  the Red-phase expectation, and invariant I3 all name
  `crates/ui/assets/icon.svg`. The implementer matched the named file and the
  suite's own precedent idiom (`d27_node_provenance.rs` `.join("assets/provenance.js")`);
  the source-of-truth path `../../media/AT-svg.svg` is kept semantically
  verbatim. The machine-checkable acceptance (all four tests green,
  baseline + 4) is reached and I reproduced it. Root cause is a handoff-text
  defect: finding 1's family (fix-at-next-touch of the handoff template).
- D3 (gate 6/7 spec defect + already-red baseline; STOP-report, then
  user-approved holistic regen at `a1129eb`): ACCEPTED on the merits, with the
  adjudication chain independently re-derived:
  (a) `scan_frozen_counts` (`theme_token_check.py:139-150`) has NO `TOKEN_FILES`
      branch - the shape D4/R2 parenthetical "exempt from BOTH scans / 
      --update-baseline NOT needed" is FALSE against shipped code; TOKEN_FILES
      exempts scan 1 only (`:124`). The new icon.svg could not pass scan 2 at
      count 0 by any in-touch-set means.
  (b) The frozen baseline was ALREADY red at TRACK_BASE for three untouched
      files. Proven, not accepted on assertion: old pins `index.html 4 /
      main.js 0 / traversal.js unkeyed (default 0)`; the true TRACK_BASE state
      is 6/1/1 - `main.js` and `traversal.js` are zero-diff over the whole
      range, and I read the TRACK_BASE `index.html` blob directly: all six
      scan-visible "literals" are decimal-entity misreads (`&#128293;`,
      `&#128161;` x2, `&#9888;`, `&#128279;`, `&#8592;`) that predate the track
      (the ledger:90 F3b row named this exact red: "index.html 6 > 4 ...
      operator disposition: deliberate --update-baseline in a dedicated commit
      with per-new-literal justification").
  (c) The regen executed exactly the ledger-prescribed remedy: dedicated,
      baseline-only commit, message + D5 journal row justifying every hunk -
      satisfying the byte-locked-sets rule (ledger:107). The 2 out-of-cure
      hunks are TRACK_BASE-pre-existing stale-pin corrections by the zero-diff
      proof above (graph.js 1->0: the sanctioned `#8250` lives at `main.js:308`
      - the degree-opacity-ramp row carried it there; themes.css:188
      `#15803d`->`#166534`: themes.css zero-diff means the tree already
      carried the new value). Hand-narrowing the JSON would have pinned counts
      contradicting locations - the guard exists to catch exactly that.
      Orchestrator disposition ACCEPTED.
  (d) Arithmetic re-verified from both JSON blobs: old counts sum
      1+4+281 = 286; new sum 7+6+1+281+1 = 296; delta +7 (icon.svg) +2
      (index.html) +1 (main.js) +1 (traversal.js) -1 (graph.js) = 296. Matches
      the journal's `286+7+2+1+1-1=296`. `generated_from_head` d76a559->1c9b466;
      assets unchanged between `1c9b466` and HEAD, so counts are true at HEAD
      (and the fresh scan's exit 0 proves counts <= actual; the generator's
      determinism on the unchanged tree proves equality).
- D4/D5 (journal tails land in separate scoped commits; the landing commit
  cannot contain its own hash): ACCEPTED. The pattern is the established
  precedent chain (tb-verdict finding 2 explicitly requiring the journal-rider
  commit; precedents `5b4da58`, `7547dea`; M28 template-collision row ledger:84
  documents the cost). Scope set unchanged, no `git add .`, one scoped commit
  per logical step. The D5 numbering (D6 requested, D5 landed) is self-explained
  in the D5 row header - a deliberate skip, not an omission.
- Gate-7 literal command (`python tools/theme_token_check.py`, bare): the
  handoff's literal invocation is the usage-error form (exit 2 by `parse_mode`
  design); the scan it names runs only inside the combined `--check-added
  --base` form, which I re-ran silent at HEAD. This is a handoff/shape wording
  defect - finding 1.

## Findings (numbered, severity-tagged; zero blockers)

1. minor (shape/handoff wording defect; root cause of the D3 blocker;
   fix-at-next-touch): `tasks/branding-assets-shape.md` D4/R2 claims the
   TOKEN_FILES rider exempts icon.svg "from BOTH scans" and that
   `--update-baseline` is "NOT needed" - false against shipped code
   (`scan_frozen_counts` has no TOKEN_FILES branch, `theme_token_check.py:139-150`);
   and handoff gate 7's literal bare `theme_token_check.py` is the usage-error
   form (exit 2, `parse_mode`:188-195). Fix the shape/handoff-spec wording and
   make the combined `--check-added --base <rev>` form the canonical both-scan
   gate. The implementer's STOP was the correct response to the contradiction.
2. minor (declared out-of-touch-set edit; ACCEPTED deviation on record):
   `a1129eb` touches `tools/theme_token_baseline.json` - outside the declared
   touch-set and on the handoff's MUST-NOT-TOUCH list. Covered by: explicit
   user approval, journal D5 row, ledger:107 byte-lock rule satisfied (the
   commit justifies each new/removed literal), and execution of the
   ledger:90 F3b operator disposition. OBLIGATION: the planner owns the dated
   baseline-regen ledger row; this verdict feeds it, the row is NOT landed
   here.
3. minor (rider comment overstatement; fix-at-next-touch): the C7 comment
   added to `tools/theme_token_check.py` says icon.svg is "exempt from BOTH
   scans exactly like the token file" - mechanically, TOKEN_FILES exempts scan
   1 only; the scan-2 pass holds via the pinned count (7), the same way
   themes.css passes (pinned 281), not via exemption. Behavior green; reword
   on the next touch of the file to avoid teaching the next track the shape
   error from finding 1.
4. relay (report-only, journal-carried): `index.html <title>` still reads
   "AriadnesThread Graph Visualizer" (pre-existing, out of handoff scope);
   `--brand-grad-a/b` remain defined in themes.css behind the now-opaque icon
   (kickoff scope-out); the ui_smoke docstring roster line stays drifted (known
   relay). All three go to closeout/deferred-findings, none is a rework.
5. relay (operator config): reviewer plumbing refused this hop (bare
   `git hash-object`, raw `ui_smoke`, chained `git diff; echo`) - recurrence of
   the standing allowlist row (ledger:70); every gate ran or was equivalently
   substituted via the sanctioned distiller route, none skipped.
6. relay (config): `testrun.py --baseline` file-form unrunnable again (7th
   encounter reproduced verbatim); standing errata decision (ledger:69) still
   owed.

## Out-of-scope observations disposition

Journal report-only items (a)/(b)/(c) recorded in finding 4; ledger deferred
rows F3a (entity misread regex fix) and the server.rs extraction (AR-13/F2)
remain open and untouched - this track added its planned +5 serve lines and
did not refactor, per the handoff's explicit out-of-scope section.

## Open obligations after this handoff (explicit)

- Track closeout (planner-owned): ledger:43 row flip, dated baseline-regen row,
  dated HOLD-lift note, memory mirror `ledger.milestone.branding-assets` (same
  commit, milestone-lifecycle ritual). NOT this hop.
- Operator eyeball rider (journal line "MANUAL EYEBALL PENDING OPERATOR"):
  header brand mark at ~32px, browser-tab favicon, brand mark render under
  light / dark / dark-grey / dark-steampunk. Unexecuted, not checked off.
- `.vsix` rebuild: operator duty (kickoff scope-out; unchanged from T-B).
- PNG marketplace icon: open follow-up from T-B (no rasterizer; deferral =
  completion).
- `--baseline` file-form errata decision: still owed (7th encounter).
