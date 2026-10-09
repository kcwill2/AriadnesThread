# Execution results - ui-traversal-back-button track (reviewer-hop verification)

Track: `20e3fb8` (TRACK_BASE, operator WIP landing) .. `5b4da58` (HEAD).
Landing commit `9c0cd0c` + journal rider `5b4da58`.
Verified fresh in the reviewer session 2026-10-09 (no trust in implementer
scoped runs). Full adjudication:
`tasks/ui-traversal-back-button-t1-verdict.md` — **APPROVED**, 7 numbered
findings, none blocking.

## Against the kickoff acceptance gates (G1-G8)

- **G1 scoped ui suite + new lock green + pre-existing tests byte-identical**:
  MET. Fresh `python tools/testrun.py --distill -- -p ariadne-ui`:
  `218 passed / 2 failed / 0 ignored / 2 suites` (exit 1; both failures
  pre-existing by name identity with the journal BASELINE set).
  Capture lines 212/214/216: `test traversal_back::test_back_css_token_only
  ... ok`, `test traversal_back::test_back_control_markup ... ok`,
  `test traversal_back::test_return_capture_wiring ... ok`.
  `git diff --name-only 20e3fb8..HEAD -- crates/ui/tests` == exactly
  `ui_test/main.rs` + `ui_test/traversal_back.rs`.
- **G2 ui_smoke PASS**: MET.
  `python tools/distill.py -- python tools/ui_smoke.py` ->
  `ui_smoke: PASS`, `### generic PASS (exit 0)`.
- **G3 theme_token_check --check-added silent**: NOT MET AS WRITTEN —
  adjudicated collision, verdict Finding 3. Fresh run exits 1 with 3 lines;
  the only track-attributable one is the C1-mandated `&#8592;` entity
  misread as `#8592` (detector regex `tools/theme_token_check.py:42-47` has
  no exemption for numeric character references; the tool's own baseline
  sanctions 4 such misreads). Substantive theme safety MET:
  styles.css frozen count clean, `test_back_css_token_only` green. Ledger
  rows required: tool fix + WIP-entity baseline disposition.
- **G4 hygiene + encoding_guard --all exit 0**: PARTIAL, adjudicated.
  `encoding_guard --all` exit 0 MET. `hygiene_check.py` exit 1 NOT MET —
  3 R1/R2 findings (`graph.js: R2 1051 > 1000`;
  `server.rs: R2 1120 > 1000`; `server.rs:764: R1 fn
  api_graph_traversal is 161 lines`) all on files proven byte-identical to
  TRACK_BASE; zero findings in touch-set files. Red pre-dates the track
  (content-identity proof); verdict Finding 2.
- **G5 Rust-source zero-diff**: MET.
  `git diff --stat 20e3fb8..HEAD -- crates/ui/src` empty; manifests/lockfile
  empty.
- **G6 touch-set exactness**: MET.
  `git diff --name-only 20e3fb8..HEAD` = the 7 touch-set paths exactly;
  rider commit journal-only. I3 collateral list (api.js, main.js,
  preview.js, provenance.js, file-view.js, theme.js, themes.css, tools,
  tests, docs) diff-empty.
- **G7 manual-smoke matrix**: NOT EXECUTED — contract-compliant fallback.
  Journal states "manual smoke pending operator" with the full 4-row matrix.
  Runtime behavior of the Back control (return-to-origin from symbol/call/
  hotspot-card entries; degree behavior not worsened) remains UNVERIFIED
  pending an operator browser pass. Verdict Finding 6.
- **G8 workspace census reconciled from live tree**: MET.
  `python tools/testrun.py --distill -- --workspace`:
  `1250 passed / 2 failed / 12 ignored / 70 suites` (exit 1, same two
  pre-existing reds only). Track adds exactly 3 tests; TRACK_BASE live
  baseline therefore 1247/2; 1250 = baseline + 3. Anchor drift vs
  `1263/0/13/91 @ c17d182` (suites −21) is WIP-era, not track-attributable
  (track adds 0 suites) — report-only observation.

## Against the shape invariants (I1-I7)

- I1 MET: C1 markup byte-exact at `index.html:388`; `closeTraversal` /
  `traversal-close` absent from all of `crates/ui/assets/**`; the 3
  unrelated `title="Close"` buttons preserved (:132, :145, :234); locked by
  `test_back_control_markup`.
- I2 MET: contract block byte-exact at `traversal.js:398-416`; both entry
  points (`index.html:72`, `hotspots.js::openTraversalForRoot`) routed
  through `openTraversalTab()`; the only remaining `switchTab('traversal')`
  call site is inside `openTraversalTab` itself; locked by
  `test_return_capture_wiring`.
- I3 MET: zero-diff proofs above.
- I4 MET (static): `git diff 20e3fb8..HEAD -- crates/ui/assets/graph.js`
  empty; degree bug untouched and still OPEN (not claimed fixed).
- I5 MET: 9-script roster / no-store route / export inline roster green
  UNEDITED (G1 failing-set identity) + `ui_smoke` PASS; `static_html.rs`
  roster not extended (file zero-diff).
- I6 MET substantively; gate-silence clause adjudicated per Finding 3.
- I7 MET: test-dir diff = new module + one `mod` line at roster tail
  (`ui_test/main.rs:24`).

## Workspace-wide reviewer-hop gates

- `python tools/testrun.py --distill -- --workspace`:
  `1250 passed / 2 failed / 12 ignored / 70 suites` (exit 1).
  The `--baseline` FILE form was deliberately NOT used: the handoff/kickoff
  carry a standing 5-encounter errata row mandating live-tree
  reconciliation via `--distill`. The `--baseline` comparison suggested in
  the reviewer checklist is superseded by that handoff language.
- `python -m unittest discover -s tools -p "*_tests.py"`:
  `Ran 219 tests in 18.966s` / `OK` (exit 0).
- `python tools/ui_smoke.py` (via distiller): `ui_smoke: PASS` (exit 0).
- `python tools/bench_compare.py` (via distiller):
  `bench compare: 3 run(s), no regressions past 1.25x baseline` (exit 0).
- `python tools/hygiene_check.py`: exit 1 (see G4).
- `python tools/theme_token_check.py --check-added --base 20e3fb8`: exit 1
  (see G3).
- `python tools/encoding_guard.py --all`: exit 0.
- `python tools/checkpoint_guard.py status`: HEAD `5b4da58`, clean worktree
  vs HEAD for track files; 3 dirty files all from the parallel
  `ui-degree-opacity-ramp` track (docs/plan only, no asset/src overlap);
  9 worktrees total, none on this track's branches.

## Explicitly NOT run / evidence gaps

- `python tools/testrun.py --baseline` FILE form — intentionally superseded
  by handoff errata (see above); live-tree reconciliation done instead.
- Operator browser matrix for the Back control (G7) — pending operator;
  the only behavioral verification outstanding.
- Per-commit line-count attribution of `server.rs`/`graph.js` growth
  (`git show <rev>:path | Measure-Object -Line` refused by sandbox bash
  rules; no permitted alternative retained) — server.rs WIP attribution is
  highly likely (kickoff dirty list) but was not commit-proven in this hop.
- `git worktree add` isolated-checkout hygiene re-run (write-git denied);
  replaced by the content-identity argument in verdict Finding 2.
- `git grep` unavailable in this session (all forms refused); equivalent
  evidence obtained via `git log -S` pickaxe and path-scoped `git log`.

## Operator dispositions required in `tasks/milestone_ledger.md`

1. Two pre-existing ui-suite reds at TRACK_BASE (both arrived with or
   pre-date the WIP landing `20e3fb8`):
   `static_html_sections::test_extraction_impact_preview_overlay` (maps to
   the existing deferred graph.js 998 budget-lock row) and
   `static_html_sections::test_inspector_panel_is_wider_and_collapsible`
   (orphaned `toggleInspector` handler at `main.js:388`; restore control or
   amend the frozen lock deliberately).
2. hygiene server.rs R1/R2 (`1120` lines; `api_graph_traversal` 161 lines)
   — new deferred row; graph.js R2 rides the existing deferred row.
3. theme_token_check tool fix (regex must skip `&#\d+;` numeric character
   references) + deliberate frozen-baseline update for WIP-introduced
   entity matches (`index.html 5>4` base inflation, `traversal.js 1>0`).
4. G7 manual smoke matrix on the Back control (4 rows) before the track is
   fully consumed.
5. Optional: ledger anchor refresh note (census drift `91->70` suites since
   `c17d182`, WIP-era).
