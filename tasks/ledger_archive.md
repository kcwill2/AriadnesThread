# Milestone Ledger Archive

Verbatim rows discharged from `tasks/milestone_ledger.md` per its archive
rule (worktree-census rows whose disposition was carried out; deferred
findings executed or dropped). Live ledger stays lean; this file is the
audit trail. Discharge evidence lives in the referenced milestone rows /
gates.

## Discharged worktree-census rows

| Worktree | State at 2026-10-01 | Disposition |
|---|---|---|

## Discharged deferred findings

| Finding | Target | Source |
|---|---|---|
| `testrun.parse_cargo_output` matches unstripped lines; colorized cargo -> false `parse format drift`; fix = ANSI-strip at raw parser entry, red-gated | next tooling milestone | tool-distill A gate |
| Optional test hardening: distill-mode baseline-load-error direct assertion (header `(exit 2)` + rc 2 together); class currently covered by shared-mechanism + adjacent pins | next tooling milestone | tool-distill B gate |
| Stale UI/tool process holds `target\debug\tool.exe` (`error: failed to remove file ... Access is denied. (os error 5)`): unscoped census and bench attempts die against the normal target (testrun DRIFT exit 2 / cargo bench exit 101); clean census `1181 passed / 0 failed / 4 ignored / 85 suites` was obtained with `CARGO_TARGET_DIR` redirected to a temp dir (code-equivalent). Close the stale process before the next normal-target build, `bench_compare`, or `--baseline` run | operator environment cleanup, before next bench/census | found during ui-extraction-card-story T1 reviewer hop (verdict finding 2) |
| M27 profile-lever landing (OPEN obligation, NOT closed by recording): DEBUG is the de-facto launch profile through EVERY documented route (AGENTS.md `cargo build` default; ui_smoke builds AND launches `target/debug`; extension docs `cargo run`) - T1a launch-profile finding verified at `42d4a6e` with E3 DEBUG proof (t0-evidence :32/:76-78); part of E1/E3 slowness was profile, not algorithm. Shipping a release-profile launch route (doc line / smoke or smoke-adjacent flag) is worth more than any plausible T3 win on current data. | named small doc/flag hop or T1b rider, before M27 closeout; RETARGETED 2026-10-05 (M28 planner registration): doc-routes half landed APPROVED at `5b517e6` (M27 T5; verdict `tasks/tu-index-hang-throughput-t5-profile-lever-verdict.md` :198 discharge-readiness - the ledger update was owed); ratified remaining half (DP-1 YES 2026-10-05: ui_smoke release route) = M28 T3 `tasks/stage5-collateral-memory-ceiling-t3-handoff.md` dispatch-ready; row stays OPEN until T3 gates APPROVED, then archived verbatim per the Archive rule | tu-index-hang-throughput T1a verdict (launch-profile record + follow-up dispositions; measurement doc sec. (c)) |
| OPERATOR disk (2026-10-06): C: free space cannot hold an N=1280 artifact save - `There is not enough space on the disk (os error 112)` killed the release RSS curve twice at the T1 review and precluded any N=1280 witness at T2. FREE DISK BEFORE: T0b's N=1280 curve re-run, the LIVE `--bench stage5-memory` gate, and any post-fix N=1280 witness; combine with the PR-2 `--update-baseline` VERIFIED-IDLE session. (Residual: T1 reviewer's two failed-run temp dirs survive - `Remove-Item` denied to reviewers.) PARTIALLY DISCHARGED 2026-10-06: operator reports disk freed (planner relay at T0c authoring); the fresh >=16 GiB free-space witness is T0c gate 0 (handoff-encoded) - row stays OPEN until that witness lands in the measurement doc. | operator, prerequisite of T0b live duties + post-fix witnesses | M28 T1 verdict sec. 7 + finding 3; T2 verdict sec. 8 + Explicitly-NOT-run |
| graph.js line-count budget lock `ariadne_ui::GRAPH_JS.lines().count() <= 990` hides inside the unrelatedly-named `test_extraction_impact_preview_overlay` (`static_html_sections.rs:215`): sizing greps miss it and it fired mid-hop 2026-10-08 (hotspot-canvas-direction T1; planner sizing erratum recorded in that track row; discharged via the handoff's AMENDMENT A1 one-line raise to 998). Residual: the assert-message literal still reads "(cap 990)" against the raised 998 bound - kept stale BY A1 AUTHORIZATION. Fix both: move the budget assert to a dedicated, findable lock test (or drop it in favor of the R2 hygiene gate) and correct the message, when the heat/legend extraction handoff (named in `tasks/hotspot-canvas-direction-shape.md` R1: graph.js heat/legend module extraction, planned together with the asset-roster cost) is next authored | heat/legend extraction follow-up handoff (shape R1 named) | hotspot-canvas-direction T1 implementer BLOCKER report-only observation + planner sizing erratum 2026-10-08 |

The first two rows above executed by handoff C (`tasks/tool-distill-c-handoff.md`):
`f3dc53b` (Red-gated ANSI-strip at parser entry; drift never hidden),
`c600fb8` (exit-honesty pin); review-gate APPROVED zero findings at
`c600fb8`; suites `Ran 30 tests ... OK` / discover `Ran 184 tests ... OK`.

The tool.exe row discharged 2026-10-04 at the ui-extraction-card-playbook
closeout: T-B's reviewer hop ran the unscoped census against the NORMAL
target dir with the verdict noting "no lock" (`1204 passed / 0 failed / 4
ignored / 85 suites` exit 0), and T-C's hop repeated clean normal-target
census (`1213 ...`) + `ui_smoke` (0.7s boot) + bench attempts - the os-error-5
signature never recurred, so the stale process was closed before the T-B hop.
The bench FAILs at those hops are the stale-baseline machine-drift class,
tracked by the dated escalation row, not this lock.

The M27 profile-lever row FULLY DISCHARGED 2026-10-06 (M28 planner, per
`tasks/stage5-collateral-memory-ceiling-t3-verdict.md` relay): doc-routes
half shipped APPROVED at `5b517e6` (M27 T5); smoke half `ui_smoke
--release` shipped at `9385acf`, reviewer APPROVED at `9f02645`.
Workspace-gate evidence (`tasks/stage5-collateral-memory-ceiling-execution-results.md`,
verdict-quoted): census `### cargo test PASS (exit 0)` / `1235 passed /
0 failed / 6 ignored / 88 suites` - byte-identical to the `bd4fb1a`
anchor, zero delta to reconcile; tools discover CI form
`-p "*_tests.py"`: `Ran 194 tests in 20.462s` / `OK` (= 188 prior + 6 new,
exact); both smoke witnesses PASS - default `boot+route checks clean (cold
fixture, attempt 1, 0.6s)` / `ui_smoke: PASS` (unchanged-behavior witness)
and `--release` `boot+route checks clean (cold fixture, attempt 1, 0.3s)`
/ `ui_smoke: PASS` (I8 rider witness). This row is hereafter cited by its
key/title ("M27 profile-lever landing"), never by row number (T3 verdict
finding 2).

The M28 OPERATOR disk row FULLY DISCHARGED 2026-10-07 (M28 planner;
witnesses, in order: (1) operator relay - disk freed, recorded in the row's
own PARTIALLY DISCHARGED note 2026-10-06 at T0c authoring; (2) the row's
required fresh witness, T0c gate-0 run BEFORE any cargo: `DISK C: free 161
GiB of 1862 GiB - PASS` exit 0
(`tasks/stage5-collateral-memory-ceiling-t0c-measurement.md` sec. 1);
(3) reviewer re-witness at review time: `DISK C: free 162 GiB of 1862 GiB -
PASS` via `python tasks/t0c_run_invocation.py --disk-witness
--min-free-gib 16` (`tasks/stage5-collateral-memory-ceiling-t0c-verdict.md`
sec. 1 + finding 5, "Row 70 has discharge evidence"). The `os error 112`
prerequisite is cleared for the LIVE `--bench stage5-memory` gate and all
N=1280 witnesses including T4's in-handoff curve gate. Scope discipline:
this row discharges its DISK duty only - PR-2 (`--update-baseline` on
VERIFIED-IDLE) rides its own dated bench-baseline row and stays OUTSTANDING
(T0c verdict finding 5: rows 62/69 stay open). Residual, operator purge
queue class (non-gate): the T1 reviewer's two failed-run temp dirs survive
(`Remove-Item` denied to reviewers). Cited by key/title ("M28 OPERATOR
disk"), never by row number.

The graph.js line-budget-lock row (moved verbatim from the live ledger's
deferred-findings table immediately above) is DISCHARGED 2026-10-09 by the
ui-degree-opacity-ramp closeout, discharge landing `3c426eb` (gated APPROVED,
verdict `tasks/ui-degree-opacity-ramp-tr-verdict.md`). Both named defects are
fixed there: the budget assert moved out of `test_extraction_impact_preview_
overlay` into a dedicated, findable `test_graph_js_line_budget()` cap 990, and
the message is honest - it names its own number, retiring the stale
"(cap 990)"-against-998 mismatch (the 998-assert removal was the only semantic
relaxation and it LOWERS the bound; handoff C2 audit + verdict blob proof).
The relocation held: T1 shipped graph.js 961 with the dedicated lock green, 29
lines of headroom (`tasks/ui-degree-opacity-ramp-t1-verdict.md` own byte-level
blob count). The heat/legend extraction handoff named in shape R1 no longer
carries this lock duty. Cited by key/title ("graph.js line-budget lock"),
never by row number.
