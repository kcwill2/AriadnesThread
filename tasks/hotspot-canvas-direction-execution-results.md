# Execution results — ui-hotspot-canvas-direction (reviewer hop, 2026-10-08)

Adjudication: `tasks/hotspot-canvas-direction-t1-verdict.md` — **APPROVED** at `c17d182`.
Every line below was produced by THIS session's fresh runs at HEAD `c17d182`
(parent `7036c9e`; track baseline `c00576d4a574052e00c88ad4e36ca2f1b3aeefa0`), except where the
implementer's historical evidence (RED-state runs) is cited as such.

## Acceptance criteria vs. actual (handoff "Acceptance (machine-checkable)" + kickoff (a)-(e))

| # | Criterion | Result | Evidence line (verbatim) |
| --- | --- | --- | --- |
| 1 | Gate 1 GREEN `217 passed / 0 failed / 0 ignored / 3 suites`, exit 0; pre-existing tests unedited except A1 line | PASS | `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 236 lines]` / `217 passed / 0 failed / 0 ignored / 3 suites` (`python tools/testrun.py --distill -p ariadne-ui`) |
| 2 | Gate 2 hygiene exit 0, no new R1/R2 | PASS | `python tools/hygiene_check.py`: exit 0, zero R1/R2 rows; all R3 warns pre-existing in foreign files (e.g. `crates/graph/src/builder.rs:569: R3 (warn) banned identifier `tmp``) |
| 3 | Gate 3 theme-token silent | PASS | `python tools/theme_token_check.py --check-added --base c00576d4a574052e00c88ad4e36ca2f1b3aeefa0` -> no output |
| 4 | Gate 4 encoding exit 0 | PASS (superset form) | `python tools/encoding_guard.py --all` -> no output, exit 0 (`--staged` not reproducible post-commit; `--all` is the stronger workspace form and covers every added line) |
| 5 | Gate 5 CLI DP-1 empty capture | PASS | `python tools/distill.py -- git diff HEAD -- crates/cli` -> `### generic PASS (exit 0)` / `capture: ...target\distill-capture.txt (0 lines)` |
| 6 | Journal manual-smoke line + gate-invocation itemization | PASS | journal:107-122 (itemized 24), journal:157-163 (smoke line; R8 limit stated) |
| (a) | Frozen locks green | PASS | §1-§3 of verdict; dirSelect default `<option value="out" selected>` at index.html:159 untouched by commit; roster locks green unedited within the 217 |
| (b) | 8 new locks green | PASS | exact-named 8 tests in `hotspot_direction.rs` (33-216) — all green within the 217 |
| (c) | Suite green, zero pre-existing test edits except A1 | PASS | per-commit `git show`: only `static_html_sections.rs` hunk `@@ -212,7 +212,7 @@`, single line :215 `<= 990` -> `<= 998`; ":216 (cap 990)" literal unchanged BY AUTHORIZATION |
| (d) | ui_smoke boots clean on unchanged roster | PASS | `boot+route checks clean (cold fixture, attempt 1, 1.1s)` / `ui_smoke: PASS` (via `python tools/distill.py -- python tools/ui_smoke.py`; direct `python tools/ui_smoke.py` refused by session allowlist — verdict finding F1) |
| (e) | Manual smoke: burden row -> "Consumers (<-)" + badged payers, not faded | ADJUDICATED (code path + smoke; live click NOT RUN) | deterministic chain graph.js:474-475/606/609/904 + index.html:160 + hotspots.js:289-294/344-366; no DOM harness exists (shape R8); stated in verdict §7 |

## Graph/hygiene measurements (this hop)

- graph.js: `996` lines (Read-tool census), `c00576d` baseline `990` -> net **+6** <= HARD CAP +8;
  numstat `7 1`. A1 lock `<= 998` holds with 2 lines of margin.
- outbound heat block byte-identical (I5): all `7+/1-` accounted by the two contract hunks;
  block `const reachableDependencies`..`restoreStandardLegend()` unchanged blob context.
- `renderInboundHeatView`: hotspots.js:320-414 = **95 lines** (< 100; journal claim exact).
- hotspots.js total: **610** (claim 610; shape "~575" estimate — finding F5, no ceiling breached).
- index.html: one `<li>`, copy verbatim, entities `&larr;`/`&rarr;`, ASCII.
- main.rs: one `mod hotspot_direction;` at the alphabetical site (between
  `help_center_readability` and `justification_builders`).
- Commit-vs-touch-set: exactly the 7 declared scopes; foreign-owned dirty set
  (`tasks/milestone_ledger.md`, `tasks/ledger_archive.md`, `tools/bench_baseline.json`,
  `.kilo/plans/1790697359283-*`, ~40 untracked M28 artifacts) stayed OUT of the commit.
- Forbidden surfaces untouched by this commit: `crates/ui/src/**` (incl. `static_html.rs` at
  999/1000), `main.js`, `styles.css`, `themes.css`, `theme.js`, `tools/**`, fixtures/goldens.

## Workspace-wide gates (verifier-cited canonical lines)

- Scoped: `217 passed / 0 failed / 0 ignored / 3 suites` (exit 0)
- Workspace census: `1263 passed / 0 failed / 13 ignored / 91 suites` (exit 0,
  `python tools/testrun.py --distill` unscoped — substitute for the literal `--baseline`,
  which errored `usage error: missing value for --baseline`; standing 5th-encounter errata,
  handoff itself: "the literal `--baseline` errata stands")
- Tools suites: `Ran 219 tests in 18.356s` / `OK` (`python -m unittest discover -s tools -p "*_tests.py"`)
- UI smoke: `ui_smoke: PASS` / `boot+route checks clean (cold fixture, attempt 1, 1.1s)` (exit 0)
- DP-1: `### generic PASS (exit 0)` / capture `(0 lines)`

## Explicitly NOT run

1. `python tools/testrun.py --baseline` literal FILE-comparison form — **not runnable**:
   `--baseline` is a FILE-valued flag (tools/testrun.py:6, :206-207) and no test-count baseline
   file exists in the repo. Verbatim: `usage error: missing value for --baseline`.
   Substitute census run instead (canonical line above). Standing ledger errata row applies.
2. `python tools/bench_compare.py` — deliberately NOT run; N/A justified: zero `crates/*/src`
   paths in the commit (pure UI assets + UI tests), the measured stage-5 tail is byte-unaffected,
   the shape's acceptance gate list omits bench_compare, and the ui-extraction-card-playbook T-C
   zero-Rust-diff EXEMPTION is the direct precedent.
3. Live interactive browser click (kickoff acceptance (e)) — NOT run: no DOM/browser harness in
   this repo or environment; adjudicated from the deterministic code path + clean ui_smoke boot,
   as the shape's R8 anticipated. The journal's manual-smoke line stands as the implementer's
   recorded observation, not machine proof.
4. Direct `python tools/ui_smoke.py` invocation — REFUSED twice by the session bash allowlist
   (verbatim refusal: `The user has specified a rule which prevents you from using this specific
   tool call. Here are some of the relevant rules
   {"permission":"bash","pattern":"*","action":"deny","source":"global"}`). The gate itself was
   NOT skipped: it completed via the AGENTS.md distilling entry point with the PASS lines above.
5. RED-state reproduction — not re-run (historical evidence: journal:27-31,
   `209 passed / 8 failed / 0 ignored / 2 suites`, 8 failures = the 8 new locks; suites=2 is the
   A1-tolerated panic-abort artifact in failing runs only; passed/failed reconcile to 209+8).

## Census status (worktree/dirty attribution)

- `python tools/checkpoint_guard.py status` (hop start): HEAD `c17d182`, `main`, ahead=0 behind=0,
  dirty=42; 8 worktrees. All dirty/untracked surfaces are foreign-owned (handoff's named foreign
  set + prior tracks' docs/probe scripts) or this hop's closing artifacts; `tarjan-s-scc` dirty=2
  is the foreign pair already documented in the M26 closeout row.
  **No stray worktree or dirty surface attributable to this track.**
- Open orchestrator residuals (verdict finding F4): ledger row :35 status flip PLANNED ->
  LANDED/APPROVED with `c17d182` + memory mirror `ledger.milestone.ui-hotspot-canvas-direction`;
  commit of ledger deferred row :79 (recorded, uncommitted) and the track's planning docs.
