# Milestone Ledger Archive

Verbatim rows discharged from `tasks/milestone_ledger.md` per its archive
rule (worktree-census rows whose disposition was carried out; deferred
findings executed or dropped). Live ledger stays lean; this file is the
audit trail. Discharge evidence lives in the referenced milestone rows /
gates.

## Discharged worktree-census rows

| Worktree | State at 2026-10-01 | Disposition |
|---|---|---|
| `.kilo/worktrees/load-tail-phase-performance` | merged via `7a58e1f` | abandon with row |

Row moved verbatim 2026-10-04 (Phase 4 closeout): its disposition -- merged --
was carried out, so the row is discharged. The three rows still live in
`tasks/milestone_ledger.md` (`troubled-concrete`, `linen-mice`, `jeweled-neon`,
plus M23.5's own `tarjan-s-scc` added at closeout) have NOT been carried out.

| `.kilo/worktrees/linen-mice` | M19 branch, complete | merge or abandon with row |

The linen-mice census row DISCHARGED by merge, archived by the AR-30 ledger
truth-sync 2026-10-10: branch head `32ecf33` is an ancestor of `main`
(`git branch --contains 32ecf33` -> `main`); the merge is `f9064f4` ("Merge
branch 'multi-language-support' into main (M24 Part A)", first merge on the
ancestry path `32ecf33..main`); `crates/lang-scala` present on `main`
(`git ls-tree main crates/lang-scala` -> tree `e27dd9c`; first `main` commit
`d0c6499` feat(m19.1,m19.2)); walkthrough on `main`
(`walkthroughs/m19-scala-discovery-walkthrough.html`). Residual operator
cleanup (not a milestone disposition): the worktree DIRECTORY still exists on
disk - `git worktree remove` pending. Cited by key/title ("linen-mice census
row"), never by row number.

## Discharged deferred findings

| Finding | Target | Source |
|---|---|---|
| `testrun.parse_cargo_output` matches unstripped lines; colorized cargo -> false `parse format drift`; fix = ANSI-strip at raw parser entry, red-gated | next tooling milestone | tool-distill A gate |
| Optional test hardening: distill-mode baseline-load-error direct assertion (header `(exit 2)` + rc 2 together); class currently covered by shared-mechanism + adjacent pins | next tooling milestone | tool-distill B gate |
| Stale UI/tool process holds `target\debug\tool.exe` (`error: failed to remove file ... Access is denied. (os error 5)`): unscoped census and bench attempts die against the normal target (testrun DRIFT exit 2 / cargo bench exit 101); clean census `1181 passed / 0 failed / 4 ignored / 85 suites` was obtained with `CARGO_TARGET_DIR` redirected to a temp dir (code-equivalent). Close the stale process before the next normal-target build, `bench_compare`, or `--baseline` run | operator environment cleanup, before next bench/census | found during ui-extraction-card-story T1 reviewer hop (verdict finding 2) |
| M27 profile-lever landing (OPEN obligation, NOT closed by recording): DEBUG is the de-facto launch profile through EVERY documented route (AGENTS.md `cargo build` default; ui_smoke builds AND launches `target/debug`; extension docs `cargo run`) - T1a launch-profile finding verified at `42d4a6e` with E3 DEBUG proof (t0-evidence :32/:76-78); part of E1/E3 slowness was profile, not algorithm. Shipping a release-profile launch route (doc line / smoke or smoke-adjacent flag) is worth more than any plausible T3 win on current data. | named small doc/flag hop or T1b rider, before M27 closeout; RETARGETED 2026-10-05 (M28 planner registration): doc-routes half landed APPROVED at `5b517e6` (M27 T5; verdict `tasks/tu-index-hang-throughput-t5-profile-lever-verdict.md` :198 discharge-readiness - the ledger update was owed); ratified remaining half (DP-1 YES 2026-10-05: ui_smoke release route) = M28 T3 `tasks/stage5-collateral-memory-ceiling-t3-handoff.md` dispatch-ready; row stays OPEN until T3 gates APPROVED, then archived verbatim per the Archive rule | tu-index-hang-throughput T1a verdict (launch-profile record + follow-up dispositions; measurement doc sec. (c)) |
| OPERATOR disk (2026-10-06): C: free space cannot hold an N=1280 artifact save - `There is not enough space on the disk (os error 112)` killed the release RSS curve twice at the T1 review and precluded any N=1280 witness at T2. FREE DISK BEFORE: T0b's N=1280 curve re-run, the LIVE `--bench stage5-memory` gate, and any post-fix N=1280 witness; combine with the PR-2 `--update-baseline` VERIFIED-IDLE session. (Residual: T1 reviewer's two failed-run temp dirs survive - `Remove-Item` denied to reviewers.) PARTIALLY DISCHARGED 2026-10-06: operator reports disk freed (planner relay at T0c authoring); the fresh >=16 GiB free-space witness is T0c gate 0 (handoff-encoded) - row stays OPEN until that witness lands in the measurement doc. | operator, prerequisite of T0b live duties + post-fix witnesses | M28 T1 verdict sec. 7 + finding 3; T2 verdict sec. 8 + Explicitly-NOT-run |
| graph.js line-count budget lock `ariadne_ui::GRAPH_JS.lines().count() <= 990` hides inside the unrelatedly-named `test_extraction_impact_preview_overlay` (`static_html_sections.rs:215`): sizing greps miss it and it fired mid-hop 2026-10-08 (hotspot-canvas-direction T1; planner sizing erratum recorded in that track row; discharged via the handoff's AMENDMENT A1 one-line raise to 998). Residual: the assert-message literal still reads "(cap 990)" against the raised 998 bound - kept stale BY A1 AUTHORIZATION. Fix both: move the budget assert to a dedicated, findable lock test (or drop it in favor of the R2 hygiene gate) and correct the message, when the heat/legend extraction handoff (named in `tasks/hotspot-canvas-direction-shape.md` R1: graph.js heat/legend module extraction, planned together with the asset-roster cost) is next authored | heat/legend extraction follow-up handoff (shape R1 named) | hotspot-canvas-direction T1 implementer BLOCKER report-only observation + planner sizing erratum 2026-10-08 |
| M23.5 companion parity test (T3-B companion item) -- **HARD EXPIRY: must land before unit E.** T3-B's two-process export lock is APPROVED and its two-sample residual was explicitly ACCEPTED (do not re-open it), but its reviewer left one companion item open and unassigned: a `crates/ui` test proving `build_include_elements_from_groups(g, .., &[], ..)` is byte-equal to `build_include_elements(g, .., &[], ..)` on an acyclic graph -- i.e. the evidence that E's deletion is behaviour-preserving. **AUTHORIZED 2026-10-04 as inserted unit D2, `tasks/m235-t3d2-handoff.md`, NOT started.** Why it cannot wait: E DELETES the legacy builder (after which the comparison cannot be written at all) and pins "you add and remove no test"; units C (`tests/integration`) and D (`crates/graph`) cannot host a `crates/ui` file; T4.1 is after E | lands with D2; only then may E be dispatched. Archive this row (verbatim) once D2 is APPROVED | T3-B verdict sec5.6 + finding 3; T3-D verdict finding 13 |

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
The third row (M23.5 companion parity test) was discharged by inserted unit D2
(`tasks/m235-t3d2-handoff.md`): test commit `42bb44e`, reviewer records
`4d0f245` and `b0a91d2`; review-gate APPROVED with 0 blocking findings
(12 non-blocking) per `tasks/m235-t3d2-verdict.md`. The row's own hard expiry
("Archive this row (verbatim) once D2 is APPROVED") is therefore satisfied,
and unit E's precondition ("only then may E be dispatched") is cleared -- E is
AUTHORIZED, repaired in place, dispatch-pending. The row is reproduced exactly
as written while it was live: its "NOT started" clause records the state at
authoring time and is superseded by the commits cited above. Its ACCEPTED
two-sample residual stays closed -- do not re-open it.

### Discharged 2026-10-04 (Phase 4 closeout): M23.5 unit E, SUPERSEDED

Moved VERBATIM from the M23.5 detail sub-block in `tasks/milestone_ledger.md`.
Its obligation is discharged: unit E was never started, it was replaced by E1
and E2, and **both are APPROVED** (`5addad7` and `378b240`+`4bc3b88`+`dcd681b`).
Its "NOT started" clauses record the state at authoring time and are superseded
by those commits. Retained for audit; **do not re-open and do not treat any
count or line reference below as current** -- the live sub-block carries the
current state and the live F14 row carries the supersession of record.

- **Unit E (`tasks/m235-t3e-handoff.md`): SUPERSEDED 2026-10-04. NOT started,
  zero commits, nothing landed.** Its implementer stopped at preflight with an
  executed probe: renaming `dto.rs:637` alone made
  `cargo check -p ariadne-cli --all-targets` fail (exit 101,
  `error[E0425]: cannot find function build_include_elements_from_groups in crate
  ariadne_ui` at `crates/cli/src/commands/ui.rs:328:40`) while the contract
  froze `crates/cli/**` byte-unchanged and `crates/ui/src/lib.rs:6` is a bare
  `pub use dto::*;`. The contract shipped unsatisfiable -- see deferred row
  **F14** and its sub-block. **Replaced by two units, both NOT started:**
  - **E1** `tasks/m235-t3e1-handoff.md` -- crate:ui only. Deletes the legacy
    `build_include_elements` (`dto.rs:608`) and its only helper
    `IncludeCycleClass::from_paths`, repoints the NINE in-crate call sites,
    collapses four two-name import lists, converts parity test P1 to a
    frozen-expectation byte lock. It carries P1's conversion **because it
    deletes the legacy builder**: once that builder is gone, P1 compares the
    survivor against itself. Adds `cargo check -p ariadne-cli --all-targets`
    as a gate. Sizing: 1 crate, 7 code paths + journal, 7 gate commands /
    15 invocations (itemised in the handoff), ~250 produced lines.
  - **E2** `tasks/m235-t3e2-handoff.md` -- renames the survivor to the plan's
    canonical public name (plan line 69). The only unit with a declared
    2-crate exception; touches `crates/cli/src/commands/ui.rs:328` and
    `crates/cli/tests/payload_tail_bench.rs:325` (plus one stale comment clause
    at `:64`); gates BOTH reverse dependencies (`-p ariadne-cli --all-targets`,
    `-p ariadne-integration-tests --all-targets`). Sizing: 2 crates, 7 code
    paths + journal, 8 gate commands / 14 invocations, ~70 produced lines.
  - Split rationale: the deletion and the rename are separable across a
    compile-green boundary (verified in both directions), and only the rename
    needs `crates/cli`. E1 cannot carry the rename, because after E1 the sole
    surviving name is the one `crates/cli` calls.
  - **Superseded contract disposition: SUPERSEDED, retained byte-unchanged on
    disk, not archived and not deleted.** E1 commit 1 lands it as the record of
    what was dispatched and rejected; this sub-block and row F14 are the
    supersession of record.
  - D2 findings **F1** (stale counts/census) and **F2** (P1 tautology) stay
    discharged: F1 by E1's re-derived caller census, F2 by E1's P1 conversion.
  - Test counts move in NEITHER unit: `--test ui_test` stays `186 / 0 / 0 / 1`
    and `-p ariadne-ui` stays `186 / 0 / 0 / 3` (186 - 0 + 0 in both). D2's
    `+2` to the workspace line remains a PREDICTION, never a measurement.

**Outcome (for orientation only, not part of the verbatim row):** the
prediction held. D2 landed `+2`, E1 and E2 each landed `+0`, and the chain ends
`1180/0/4/88` at `dcd681b`, reviewer-measured. The single public builder is now
`crates/ui/src/dto.rs:621`, so every `dto.rs` line number in the verbatim row
above is historical.

### Discharged 2026-10-28 (rust-language closeout hygiene sweep: deferred rows
executed/closed, moved verbatim per the ledger Archive rule)
| ui-traversal-back-button F1 (2026-10-09): SECOND pre-existing ui-suite red at TRACK_BASE - `static_html_sections::test_inspector_panel_is_wider_and_collapsible` (assert at `static_html_sections.rs:411-413` requires `onclick="toggleInspector()"` present in INDEX_HTML) - attributable to the operator WIP landing `20e3fb8`: the assertion is old (introduced `b78ec19`; last carrier-file edit `c17d182` = green ledger anchor 0 failed), the ONLY commit touching `index.html` in `c17d182..20e3fb8` is `20e3fb8`, and `toggleInspector` now exists only as the orphaned handler at `main.js:388` (reviewer pickaxe + path-scoped log attribution CONFIRMED; verdict F1). Sibling red `test_extraction_impact_preview_overlay` (graph.js 1051 > 998) maps to the existing graph.js budget-lock deferred row - no new row for it. Operator disposition: restore the collapse control in the WIP's own follow-up, OR amend the frozen lock deliberately. RESOLVED 2026-10-09 via disposition 1 (restore control): `index.html:301` restored to `onclick="toggleInspector()"` (byte-identical to the green anchor `c17d182`); second WIP defect found and fixed in the same follow-up - `main.js` `collapseInspector`/`expandInspector` had been repointed to `getElementById('sidebarPanelInspector')`, an id that exists in NO asset (the aside is `inspectorPanel`), so even the WIP's rewired button was a dead handler; both lookups restored to `inspectorPanel`. Scoped ui gate: `234 passed / 0 failed / 0 ignored / 3 suites` (F1 + sibling graph.js red both clear of the failing set) | CLOSED 2026-10-09 (operator WIP follow-up, restore-control form) | ui-traversal-back-button T1 verdict F1 + execution-results disposition 1 |
| ui-traversal-back-button F3b (2026-10-09): frozen-COUNT hex-baseline violations from WIP-introduced entities - `index.html 6 > 4` (base inflation 5 > 4 already at TRACK_BASE `20e3fb8`, before this track added any line; the +6th is the track's C1 entity, a phantom misread that disappears once F3a lands) and `traversal.js 1 > 0` (WIP emoji entity); operator disposition: deliberate `--update-baseline` in a dedicated commit with per-new-literal justification per the byte-locked-sets table rule, OR entity removal from the WIP assets. CLOSED 2026-10-11: disposition 1 EXECUTED at `a1129eb` (branding-assets T-A D5 regen, user-approved; per-hunk justification = journal D5 + ta-verdict D3(c); pins now true-state index.html 6 / main.js 1 / traversal.js 1; the misread COUNTS are pinned - F3a regex fix remains OPEN); see the dated regen row; archive per Archive rule at next closeout | CLOSED 2026-10-11 (executed at `a1129eb`) | ui-traversal-back-button T1 verdict F3 sub-finding + execution-results disposition 3 |
| branding-assets D5 baseline regen (dated 2026-10-11, LANDED): `a1129eb` regenerated `tools/theme_token_baseline.json` via `--update-baseline`, user-approved; root cause per ta-verdict finding 1 - shape D4/R2 premise false against shipped code (`scan_frozen_counts` has no `TOKEN_FILES` branch; exemption is scan-1 only) - plus the stale `d76a559` pins already red at TRACK_BASE (3 violations on untouched files). Regen cured the 4-item set (index.html 4->6, main.js 0->1, traversal.js unkeyed->1, icon.svg 0->7; 286+7+2+1+1-1=296) + 2 zero-diff-proven TRACK_BASE-pre-existing stale-pin corrections: graph.js 1->0 (sanctioned `#8250` literal relocated to main.js:308) and themes.css:188 recolor `#15803d`->`#166534` (count held 281). Canonical both-scan gate = combined `--check-added --base <rev>` form (bare `theme_token_check.py` = usage-error exit 2 by design). Authorization + per-hunk justification: journal D5 row; ledger:107 byte-lock rule satisfied; executes the ledger:90 F3b disposition | CLOSED 2026-10-11 at `a1129eb` (evidence: ta-verdict D3(c) + findings 1-2; execution-results T-A criterion 9; journal D5) | branding-assets T-A D3/D5 + verdict findings 1-2 |
| ui-extraction-card-clarity T-A/T-B disposition (dated 2026-10-12, recorded by the ui-extraction-wizard kickoff; the clarity milestone row marker points here): T-A LANDED on main and REVIEWED, but the ledger stayed silent until this row - landing `8a254ef` (parent `55a85d9`), review rider `548decf`, merged via `c75d4932` ("merge(ui-extraction-card-clarity): T-A contrast oracle + value fix (8a254ef) and review rider findings 1-3 (548decf) into main"). Adjudication: verdict APPROVE WITH SUGGESTIONS, findings 1-3 all fixed in the user-authorized rider (reject-loudly duplicate-declaration guard in the shared `declared_value`; WCAG math + parsing helpers consolidated into `ui_test/common.rs`); NO `tasks/ui-extraction-card-clarity-ta-verdict.md` artifact was ever written - the adjudication record lives in the ta-journal "Review rider" section (orphaned-verdict class; noted so the next kickoff does not re-open it). Disk proof at HEAD `97dd836`: `extraction_contrast.rs` present (329 lines post-rider), `main.rs` mod line after `extraction_card_story`, light `--green` = `#166534` in the themes.css light block - all inherited as frozen locks by ui-extraction-wizard (W8). T-B SUPERSEDED BY `ui-extraction-wizard` - DO NOT DISPATCH `tasks/ui-extraction-card-clarity-tb-handoff.md`: its per-file content shape (C2/C3) transfers into the wizard steps and its four-lock amendment table transfers to wizard shape section 5 C5 (re-justified by the wizard restructure); its card-detail placement does not ship | CLOSED 2026-10-12 (disposition recorded as fact - nothing to execute; archive per the Archive rule at the next closeout) | ui-extraction-wizard kickoff ledger duty 4 + shape section 6; ta-journal Review rider section; commits `8a254ef` / `548decf` / `c75d4932` |
| M23.5 F12 (housekeeping) -- **CLOSED by T4.1 at `b3f33a9` (2026-10-04).** Fix landed: `.gitattributes` now pins `tests/fixtures/**/artifacts/**/*.json text eol=lf`. **Proof per contract:** after a full workspace test pass run POST-pin, `git status --porcelain -- tests/fixtures/basic/artifacts/includes/report.json` printed NOTHING; workspace line `1180/0/4/88` held; the fixture appears in NO commit and stayed byte-equal to HEAD (`git hash-object` == HEAD blob `501b19bd`). **Mechanism correction measured:** the pin alone does not clear an already-stale index entry -- one `git checkout -- <path>` normalization (byte-identical re-smudge) was needed once because the entry's CRLF-checkout expectation predates the pin; the following full run proved the pin prevents recurrence. The pre-run churn signature (status ` M`, `git diff` empty) reproduced a TENTH time at T4.1 preflight and is now defunct. Original row text retained in-line after this disposition. `tests/fixtures/basic/artifacts/includes/report.json` is UNPINNED: `.gitattributes` (7 lines) sets `text eol=lf` for exactly two globs, `crates/ui/tests/fixtures/*.json` and `tests/golden/**/*.json`, and neither covers this path; `core.autocrlf=true` re-dirties its STAT ENTRY on every full test run even when content is unchanged. Measured **nine** times (T2-A f9, T2-B f10, T3-A sec 7, T3-B sec 6, T3-C, T3-D sec 3.4, T3-E2 f13 -- clean there -- plus the Phase 4 planner's own measurement, which re-reproduced the exact signature: `git status` reports the path `modified` while `git diff` for it returns NO hunks). Content is byte-equal to HEAD; only the stat cache is invalidated. Pre-existing test behaviour, not a milestone defect -- but it poisons any unstaged `commit_scoped.py` | Extend `.gitattributes` to `tests/fixtures/**/artifacts/**/*.json` (**preferred: config-only, no writer change**), prove it by re-running a full test pass and showing `git status` clean for the path, and pass an EXPLICIT `--scope` on every `commit_scoped.py`. T4.1 owns this; do NOT let the fixture appear in any commit -- it must be byte-equal to HEAD | M23.5 T1 verdict finding 12; T2-A f9; T2-B f10; T3-E2 f13 |
| ui-extraction-wizard W-3a handoff amendment (dated 2026-10-12, planner): the W-3a implementer halted pre-Red at the boot audit with ZERO edits (journal `tasks/ui-extraction-wizard-w3a-journal.md`, journal commit `e22f4a6`, parent `1f561a0` which carried the already-landed W-2a verdict tracking; baseline at halt `294 passed / 0 failed / 0 ignored / 3 suites`). Root cause: the R7 registration chain was under-counted - the 5-site enumeration missed TWO asset-count lock sites that the mandatory `index.html` script tag detonates: the `standalone_template` inline tuple in `crates/ui/src/export.rs:35-48` (a 10th `<script src>` survives the replacement loop, failing the export lock at `static_html.rs:992`) and `crates/ui/tests/ui_test/theme_contract.rs:428-449` (script-tag count `9` -> `10` + one `tag_start_is_asset` whitelist row). DISCHARGED same day as a doc fix, NOT a rework: the handoff is amended in place at HEAD `e22f4a6` with an AMENDMENT section (playbook C5 / clarity T-B keep-literal discipline - exact before/after per lock; the count assertion is mechanically opposed to required test 1, so NO keep-literal path exists), touch-set +2 files, gate-9 scope list +2; gates stay at 10, one module dir, diff +~10 lines. `static_html.rs:949`'s export lock verified to survive UNEDITED (generic no-survivor lock + per-asset presence markers, no count or list). Further consumers of the export tuple entry swept CLEAN by reading: `export_order.rs` (payload order, no asset surface), `asset_injection_hardening.rs` (fixed 9-element arrays; the wizard must NOT re-declare `escapeHtml` - required test 8 enforces), `d27_node_provenance.rs:280` (provenance-specific), `static_html.rs:760` (relative order walk) and `:792` (fixed 8-list), `ui_smoke_tests.py:86-95` (builds from `ROUTE_ASSERTIONS`), `payload_tail_bench.rs:571` (two-process equality, both sides move identically), integration m15/m16/m17 (presence markers), goldens/fixtures (no HTML-export goldens). The `ui-extraction-wizard` milestone row stays PLANNED - NOT flipped. The next implementer hop's Duty 1 is a docs-only commit of the amended handoff + this ledger row; the W-2a verdict artifacts (`1f561a0`) are already landed and NOT in that command | the amended handoff (discharged as a doc fix - nothing to re-execute); the successor implements per the AMENDMENT table | W-3a halt journal `tasks/ui-extraction-wizard-w3a-journal.md` (D1-D6) + planner verification reads 2026-10-12 (theme_contract.rs:384-449, index.html:591-601, export.rs:35-48, static_html.rs:949-998) |
