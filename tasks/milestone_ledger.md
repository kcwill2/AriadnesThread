# Milestone Ledger

**Authority rule:** this ledger is the source of truth for milestone state.
kilo-memory entries MIRROR it and must be updated in the same commit; on
conflict, trust this ledger and fix the memory. Milestone rows and deferred
findings predating 2026-10-01 were reconstructed from memory + git and should
be backfilled at the next kickoff if found inaccurate.

## Archive rule (added 2026-10-01, context hygiene)

The ledger carries only LIVE rows: kickoff reads it and kilo-memory mirrors it,
so growth is paid twice. At the closeout AFTER a row lands -- a worktree census
row whose disposition was carried out, or a deferred finding executed or
dropped -- move that row verbatim to `tasks/ledger_archive.md`. Milestone rows
stay (they are the audit trail); census and deferred rows do not.

**Line-width rule (added 2026-10-04, observability):** no line in this file may
exceed 2000 characters. Agent Read and grep silently truncate at 2000 chars per
line, so an over-long row hides its own tail from every workflow participant --
a milestone row once grew past the limit and its whole tail became invisible.
Keep table cells short, put per-unit detail in a sub-block beneath the table,
and point at the per-unit verdict artifacts under `tasks/` instead of
accumulating it here.

## Milestones

| Milestone | Status | Where (commit/branch) | Frozen locks | Notes |
|---|---|---|---|---|
| M18 header-dependency migration + UI story | COMPLETE (audit 2026-10-01: intent confirmed, zero regressions, 8 findings dispositioned) | `main` (merged; audit vs spec at df57491) | `tests/golden/**` byte-locked, manifest hash `0xddc3...` resync via `UPDATE_GOLDENS=1` only | M18.11 session-death incident -> T1 checkpoint_guard |
| M19 Scala plugin (`crates/lang-scala`) | COMPLETE on branch, NOT merged to `main` | `multi-language-support` @ `32ecf33` (worktree `.kilo/worktrees/linen-mice`) | same as M18 on that branch; `tests/golden/**` does NOT exist on `main` | 715 tests, cpp parity 22/22, walkthrough committed; handoffs `tasks/m19-t{a,b}-handoff.md` live on the branch |
| M20 Scala resolver | STARTING (independent session + own worktree per 2026-10-01 handoff) | new branch off `multi-language-support` | TBD at kickoff | StubResolver replacement |
| Process tooling codification | COMPLETE on `main` @ T11 feat `592de60` (T1-T5, T7-T14; 153 tool tests green, CI consolidated discover step) | `.kilo/plans/1790831557705-process-tooling-codification.md` | `tools/theme_token_baseline.json` (132 literals, 8 files), `tools/hygiene_r3_allowlist.txt` (150 legacy entries) | T6 golden_guard deferred to M19 merge (row below) |
| Tool output distillation (contract + distillers) | COMPLETE (A gated APPROVED `1b7bc3f`; B gated APPROVED `a2f94af`) | spec `.kilo/plans/1791044160000-tool-output-distillation.md`; `docs/specs/tool-output-distillation.md` | none | I8 SUPERSEDED 2026-10-04 by the five-role topology row below (enforcement now lives in the permission allowlist, not in tools or delegation prose); F1/F2 fixed in B; F3/F4 closed as process lessons; global `~/.agents/skills/tool-runner/SKILL.md` amended out-of-band 2026-10-02 (raw paste fields deleted, digest contract + routing added; vendoring declined to avoid shadow drift); handoff C discharged deferred O1 + B-gate pin (`f3dc53b`/`c600fb8`, gate APPROVED zero findings; rows archived to `tasks/ledger_archive.md`) |
| M23.5 cycle substrate: SCC condensation + iterative Tarjan | IN PROGRESS -- Phases 1, 2 and 3 **ALL units APPROVED 2026-10-04** (T1.1/T1.2, T2-A, T2-B, T3-A/B/C/D/D2/E1/E2; unit E SUPERSEDED at preflight, zero commits, replaced by E1+E2). Phase 4 (T4.1) EXECUTED 2026-10-04 (config+code `b3f33a9`; records + this ledger update in the follow-on `tasks/**` commit). **F12 CLOSED** (pin landed + full-run proof: path clean after a post-pin full test pass). **F13 CLOSED** (P2 renames landed; `186/0/0/1` survived). **NOT CLOSABLE: F10 runtime/perf evidence is unclosed and blocks the closeout verdict.** Per-unit detail: sub-block below | branch `tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`; base `2708425`; HEAD moves `dcd681b` -> `b3f33a9` -> T4.1 records commit; NOT merged to `main` | IV-1..IV-12; R1/R2 hygiene; UTF-8 mojibake; zero new UI hex literals; planner byte-identity UNEDITED; `tests/golden/**` LF; cycle-policy seam; D1-B (full list in sub-block) | Baseline chain ends `1180/0/4/88` at `dcd681b` (reviewer-measured). Criterion 1 DISCHARGED for production with 2 recorded caveats. Open: F9, F10 (BLOCKS closeout), F11, F14 below; F12 and F13 CLOSED by T4.1 |
| Agent topology: five-role, permission-enforced | COMPLETE (verified end-to-end by 4-hop synthetic smoke test, 2026-10-04) | plan `.kilo/plans/1791072847000-five-role-agent-topology.md`; agents `~/.config/kilo/agents/*.md` (GLOBAL, affects all Kilo projects) | none in-repo; global config backups `kilo.jsonc.bak-20261004`, `kilo.jsonc.pre-agents-strip`, `agents/code-reviewer.md.bak-20261004` | Replaces the four-role hybrid: `tool-caller`, `review-guide`, `code-reviewer` DELETED. Roster: orchestrator (primary) + architect/planner/implementer/reviewer/walkthrough-builder (subagents), all hops via `task`. Four probed constraints: (1) `TaskTool` stamps `task: deny` on every child, so subagents cannot nest -- a subagent keeps `task` only as a top-level session; (2) `mode: primary` agents cannot be task-spawned at all; (3) permission objects are LAST-match-wins, so the catch-all `"*": deny` must come FIRST; (4) a caller's `edit: deny` is inherited as a hard ceiling that strips `write`+`edit` from every child, so the orchestrator holds `edit: ask`. `background_process` and mutating `git_git_*` are gateable and denied in all six. Known limits: `distill.py` is an argv pass-through so it bounds OUTPUT not command scope; permission denials always report the catch-all rule, which caused one reviewer to skip five permitted gates (role prompts now instruct per-gate attempts). M23.5 handoffs + plan repointed off `tool-caller`/`Code-reviewer` same day |

### M23.5 detail (per-unit detail lives here and in the verdicts, never back in the table row)

- **Phase 1 (T1.1+T1.2, crate:cluster):** APPROVED 2026-10-04, 0 blocking / 12 non-blocking. `dd3db11` (parity oracle + iterative frame machine), `e2a05e2` (journal).
- **Phase 2 (T2.1-T2.3, crate:graph):** APPROVED and CLOSED 2026-10-04. T2-A `34f5fe4`+`7c94af3`, T2-B `08741b8`+`b950963`, 0 blocking each.
- **Phase 3 (T3.1/T3.2) units A/B/C/D/E1/E2:** all APPROVED 2026-10-04. A `31513b9..ea82422`, B `9d831bf..91967c9`, C `b774362..6c0c2b6`, D `adb53a3..aebfa9b` (0 blocking / 13 non-blocking), E1 `5addad7`, E2 `378b240`+`4bc3b88`+`dcd681b` (0 blocking / 14 findings, **9 planner-attributed** -- see F14).
- **Unit D2 (inserted companion, crate:ui legacy/group parity lock, one test file):** APPROVED, 0 blocking, 12 findings. `42bb44e`+`4d0f245`+`b0a91d2`.
- **Unit E (`tasks/m235-t3e-handoff.md`): SUPERSEDED 2026-10-04. NOT started, zero commits, nothing landed; replaced by E1 + E2, both since APPROVED.** It shipped unsatisfiable -- its preflight probe proved renaming the grouped builder alone fails `cargo check -p ariadne-cli --all-targets` (exit 101, `E0425` at `crates/cli/src/commands/ui.rs:328:40`) while the contract froze `crates/cli/**`; see **F14**. Sizing, split rationale and the full superseded-state record moved verbatim to `tasks/ledger_archive.md`.
  - **E1** `tasks/m235-t3e1-handoff.md` -- crate:ui only. Deleted the legacy `build_include_elements` + its `from_paths` helper, repointed the in-crate call sites, collapsed four two-name import lists, converted parity test P1 to a frozen-expectation byte lock. It carries P1's conversion **because it deletes the legacy builder**: once that builder is gone, P1 compares the survivor against itself. Added `cargo check -p ariadne-cli --all-targets` as a gate.
  - **E2** `tasks/m235-t3e2-handoff.md` -- renamed the survivor to the plan's canonical public name (plan line 69). The only 2-crate unit; gated BOTH reverse dependencies (`-p ariadne-cli`, `-p ariadne-integration-tests`, `--all-targets`).
  - **Superseded contract disposition: SUPERSEDED, retained byte-unchanged on disk, not archived and not deleted** (landed by `ab59bb4`). This sub-block and row F14 are the supersession of record.
  - D2 findings **F1** (stale counts/census) and **F2** (P1 tautology) stay discharged: F1 by E1's re-derived caller census, F2 by E1's P1 conversion.
  - Test counts moved in NEITHER unit: `--test ui_test` `186/0/0/1` and `-p ariadne-ui` `186/0/0/3` at both ends, and the workspace line moved **+0/+0/+0/+0** across E1+E2 (reviewer-measured at `dcd681b`) -- the independent proof that no test was added, removed, renamed or stopped running.
- **Phase 4 (T4.1):** AUTHORIZED, in progress -- `tasks/m235-t4-handoff.md`. Acceptance criterion 1 is DISCHARGED for production code. **The milestone cannot be declared CLOSED until three items no role can execute are done:** (i) F10's allowlist amendment + `ui_smoke.py` / `bench_compare.py` runs, (ii) the kilo-memory mirror of this ledger (the authority rule above: memory MIRRORS the ledger and must move in the same commit; not delegable to any role), (iii) the worktree census disposition for `.kilo/worktrees/tarjan-s-scc` (branch `tarjan-s-scc` is NOT merged to `main`). Plus the `milestone-lifecycle` walkthrough item: `walkthroughs/**` holds NO M23.5 deck (only `ui-theming-`, `m21-`, `m19-`), so one must be generated and committed ON THIS BRANCH (E10/E16 loss).
  - **T4.1 EXECUTED 2026-10-04** (`tasks/m235-t4-handoff.md`, journal `tasks/m235-t4-journal.md`). Landed in `b3f33a9` + records commit: F12 pin `tests/fixtures/**/artifacts/**/*.json text eol=lf` with full-run proof; F13 P2 renames in `crates/ui/tests/ui_test/export_order.rs` (`the_non_vacuity_witness_detects_a_cycle_classification_difference`, `empty_currency_elements`, `populated_currency_elements`; assertions/bytes untouched, `186/0/0/1` intact). Goldens VERIFIED byte-clean, re-pin UNNECESSARY and NOT done (`git diff HEAD -- tests/golden` empty after two full runs; `UPDATE_GOLDENS` never run here). Plan file confirmed TRACKED (discharges the UNVERIFIED caveat in the strays row). Workspace `1180/0/4/88` re-measured twice; hygiene R1/R2 zero; encoding `--all`/`--staged` silent; theme-token guard silent. Bench (ignored, `--release -- --ignored`) measured: `generate_report+detect_cycle_groups` combined `8.0` ms and `build_include_elements` `8.0` ms on the 520-header fixture -- recorded, NOT reconciled against the `5.1`/`5.4`/`5.6` doc split (F10 open; T3-E2 verdict finding 6 do-not-re-open stands).
  - **F12 finding of mechanism (corrects the handoff's mental model):** the `.gitattributes` pin alone does NOT clear an ALREADY-dirtied entry -- git keeps the stale checkout-EOL expectation cached in the index, so status stayed ` M` even across a full run post-pin. Closure required ONE normalization of the worktree entry (`git checkout -- <path>`, byte-identical, verified by `git hash-object` == HEAD blob `501b19bd`), after which the pin holds: the NEXT full run left the path clean. Future clones/first checkouts are covered by the pin directly.
  - **Determinism report (plan T4.1 wording, unmet-as-literally-stated, REPORTED not built):** two NON-ignored locks cover cycle-bearing export determinism on THEIR fixtures (`payload_tail_bench.rs:571` two separate processes; `graph_api.rs:742` two in-process calls). The 520-header SCC-dense fixture has NO determinism assertion (only the `#[ignore]`d size-asserting `payload_tail_phase_timing`). A cross-process lock on that fixture would be a NEW TEST = new RGB unit, not closeout work.
- **Baseline chain** (all reviewer-measured, every delta attributed, zero residual): `1156/0/4/86` pre-Phase 1 -> `1165/0/4/87` T2-A (+9/+1) -> `1169/0/4/88` T2-B (+4/+1) -> `1174/0/4/88` T3-A (+5/+0) -> `1176/0/4/88` T3-B (+2/+0) -> T3-C (+0/+0) -> `1178/0/4/88` at `aebfa9b` T3-D (+2/+0) -> **D2 +2/+0 -> E1 +0/+0 -> `1180/0/4/88` at `dcd681b` (E2 +0/+0)**. Tools suite unmoved throughout: `Ran 184 tests ... OK`.
- **Crate-scoped lines:** crate:cluster (Phase 1); crate:graph (T2-A, T2-B, T3-C, T3-D); crate:ui (D2 one test file; E1 whole crate; E2 the sole 2-crate unit).
- **Frozen locks (full):** IV-1..IV-12 (plan sec1.2); R1/R2 hygiene (`hygiene_check.py`, blocking); UTF-8 mojibake (`encoding_guard.py`); zero new UI hex literals (`theme_token_check.py`); planner byte-identity UNEDITED (`planner_facts_parity`, `planner_characterization`, `m00_facts_path_parity_test`); `tests/golden/**` LF via `.gitattributes`; cycle-policy seam (`generate_report_with_cycle_policies`, `informational_cycle_count` skip-when-zero serde); D1-B = no dual cycle representation.
- **Acceptance criterion 1 (`CycleGroup` is the only cycle currency; one public builder name):** DISCHARGED for production code at `dcd681b`, reviewer-verified, with **two caveats of record**: (a) `crates/graph/tests/cycle_groups_equivalence_test.rs:90`/`:116` deliberately retain `old_detect_cycles` / `old_dfs_cycle` as a FROZEN test-local oracle (module doc at `:4`) proving the SCC path reproduces DFS semantics -- the correct place for a dual representation, since deleting it would delete the equivalence evidence; **never delete it**; (b) `crates/transform/src/engine/mod.rs:133`'s private `detect_cycles_in_units` is an unrelated subsystem guard over generated units (called at `:72`), already proven a different function, out of M23.5 scope. Also verified: exactly ONE public builder `crates/ui/src/dto.rs:621` `pub fn build_include_elements`, third param `&[CycleGroup]`, the emission tail `emit_include_elements` (`:643`) is PRIVATE, and `crates/ui/src/lib.rs` is a bare `pub use dto::*;` so no alias or shim survives.
- **Accepted, do NOT re-open:** T3-B's two-process export lock is APPROVED and its two-sample residual was explicitly ACCEPTED (T3-B verdict sec5.6). E2's residual `5.1` vs `5.4` perf split (`crates/cli/src/commands/ui.rs:103` vs `payload_tail_bench.rs:41`) is also do-not-re-open: reconciling it while F10 is open would manufacture perf evidence, so O-1 was correctly reported and left alone.
- **Open deferred rows (detail in the table below):** F9 (`testrun.py --baseline` unrunnable; the hand-comparison protocol WORKED, the tool defect stays open), F10 (**BLOCKS the closeout verdict** -- `ui_smoke.py` + `bench_compare.py` refused by the reviewer allowlist; OPERATOR action), F11 (unattributed `+6` pre-Phase-1 residual), F12 (`.gitattributes` pin missing for `includes/report.json`; T4.1 closes it), F13 (CORRECTED 2026-10-04 by the T3-E2 review -- see table; now also owns P2's stale locals and stale test name), F14 (the "unverified claim about the tree" class: **six** contract defects this milestone; three standing rules below).
- **Where per-unit detail now lives:** `tasks/m235-t1-verdict.md`, `-t2a-`, `-t2b-`, `-t3a-`, `-t3b-`, `-t3c-`, `-t3d-`, `-t3d2-`, `-t3e1-`, `-t3e2-verdict.md` (each unit's `-journal.md` and, where one exists, its `-execution-results.md`), the **superseded** unit-E handoff `tasks/m235-t3e-handoff.md` with its preflight-stop journal, the replacement handoffs `tasks/m235-t3e1-handoff.md` and `tasks/m235-t3e2-handoff.md`, the Phase 4 contract `tasks/m235-t4-handoff.md`, and `tasks/m235-kickoff.md`. No `tasks/m235-t3e-execution-results.md` exists on disk and none is referenced.

## Worktree census obligations (E16)

Every participating worktree must end merged, or abandoned WITH a ledger row
below. Closeout checks run `python tools/checkpoint_guard.py status` (census).

| Worktree | State at 2026-10-01 | Disposition |
|---|---|---|
| `.kilo/worktrees/troubled-concrete` | DETACHED HEAD @ `1aa03b3`, own branch deleted; held verified-but-never-committed theming work (lost) | needs `git worktree remove` at next kickoff once operator confirms nothing to salvage |
| `.kilo/worktrees/linen-mice` | M19 branch, complete | merge or abandon with row |
| `.kilo/worktrees/jeweled-neon` | at `main` tip, no branch work | abandon with row |
| `.kilo/worktrees/tarjan-s-scc` (M23.5's own) | branch `tarjan-s-scc` @ `dcd681b`, all units APPROVED, NOT merged to `main` | **OPEN -- CLOSEOUT BLOCKER.** Must end merged to `main`, or abandoned with a row, before M23.5 is CLOSED. This worktree holds the ONLY copy of the verified milestone work (E16: that is how the theming implementation was lost) |

## Deferred findings (with target milestone)

| Finding | Target | Source |
|---|---|---|
| Execute theming plan `1790656207440` (three themes, D1-D8) - implementation VERIFIED IN WORKTREE THEN LOST (E16); its own T8 parity + zero-hex Rust tests own theme-token Phase B | next UI capacity window; redo required | E16 / T13 |
| lang-scala guard gaps | M23 | M19 kickoff (D9) |
| `test_output.txt`, `test_results.txt`, `parse.py` are UTF-16 PowerShell-captured debris committed ~2026-09-25; invisible to encoding_guard's binary (NUL) skip | delete at next hygiene pass | found during T2 build 2026-10-01 |
| `.css` invisible to R2 size gate (`styles.css` 1422 lines and growing; `R2_SCANNED_SUFFIXES` excludes `.css`) | separate decision (day-one failure risk) | plan residual risk |
| `bench_baseline.json` has 3 phases baselined at 0.0 ms (both `get_or_build_load hand-out` rows, `conservative filter pass`); any future nonzero median fires WARN +inf% -- add a min-abs floor if noisy; bench doc-comment phase labels are STALE (live rows print `hand-out`/`facts+closures`, not `clone`/`cold`) | fix when bench table next touched | T11 build 2026-10-01 |
| T6 `tools/golden_guard.py` - specced, unimplemented: `tests/golden/**` exists only on `multi-language-support` | at/before M19 merge | D5 |
| Three untracked strays (`.kilo/plans/1790697359283-systems-layer-engine-room.md`, `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md`, `.kilo/skills/deslop/`) need owner decision: commit or discard. **PARTIALLY RESOLVED 2026-10-04 at Phase 4:** none of the three appears in `git status`'s untracked list at `dcd681b`, and `.gitignore` carries no `.kilo` rule -- so each is either TRACKED or covered by a global/exclude ignore. Whether the M23.5 plan file is tracked is **UNVERIFIED** (bash denied to the planner this session); `repo.tracks_plan_files` says plan files are tracked, so T4.1 must confirm and, if tracked, cite it as the contract of record. **DISCHARGED by T4.1 (2026-10-04): `git ls-files --error-unmatch .kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` exits 0 -- the plan file IS tracked; cited as the contract of record; no landing needed** | operator; T4.1 confirms the M23.5 plan file | tool-distill A gate |
| M23.5 F9 (planner/configuration): `python tools/testrun.py --baseline` is UNRUNNABLE as documented in `AGENTS.md`, plan T4.1, and the reviewer agent definition -- `--baseline` requires a FILE argument and no `N/M/K/S` baseline file exists anywhere in the repo. Verbatim: `usage error: missing value for --baseline` (exit 2). **REPRODUCED AGAIN 2026-10-04 in the T3-E2 review, verbatim, and again to this planner.** **The PROTOCOL WORKED: every milestone gate in this milestone was compared BY HAND against that row's baseline chain (the chain ends `1180/0/4/88` at `dcd681b`), and every delta was attributed; the tool defect never once blocked a gate, because the protocol does not use the flag.** The defect is a documentation/UX bug in three places that still advertise an unrunnable command | fix `testrun.py` to default the argument, OR land a committed baseline file holding the CURRENT canonical line (**the M23.5 milestone row is the single copy -- do not restate the chain in the baseline file; it is dated and will rot**), AND correct the three documents that order the broken form. Not required for the T4.1 verdict | M23.5 T1 verdict finding 9; T3-E2 verdict "Gates refused / not run" |
| M23.5 F10 (configuration) -- **THE MOST MATERIAL UNCLOSED GATE IN THE MILESTONE, AND IT BLOCKS THE CLOSEOUT VERDICT.** The reviewer bash allowlist omits `python tools/ui_smoke.py` and `python tools/bench_compare.py`, yet the same agent file and plan T4.1 both order them; both refused verbatim with the catch-all rule in seven units, none routed through `distill.py`. **There is still ZERO runtime evidence that the UI cold-boots, routes, serves `Cache-Control: no-store` and returns a group-count report, and NO verified perf evidence after T3-D changed what `cycle_count` and `generate_report()` compute.** T3-E2's rename does not make this safe to defer -- the ONE surviving public builder's output is exactly what `ui_smoke.py` exercises end-to-end. Detail + escalation: sub-block below | **OPERATOR ACTION, and no role can discharge it: amend the permission allowlist** (enforcement lives in the allowlist, not in agent prose), then run both in an authorized session. T4.1 must NOT reconcile the `5.1`/`5.4`/`5.6` perf split and must NOT route either command through `distill.py`. **Until both run, M23.5 may not be described as CLOSED** | M23.5 T1 verdict f10; T2-A f9; T2-B f9; T3-B f6; T3-D f9 + exec-results sec7; T3-E2 f14 |
| M23.5 F12 (housekeeping) -- **CLOSED by T4.1 at `b3f33a9` (2026-10-04).** Fix landed: `.gitattributes` now pins `tests/fixtures/**/artifacts/**/*.json text eol=lf`. **Proof per contract:** after a full workspace test pass run POST-pin, `git status --porcelain -- tests/fixtures/basic/artifacts/includes/report.json` printed NOTHING; workspace line `1180/0/4/88` held; the fixture appears in NO commit and stayed byte-equal to HEAD (`git hash-object` == HEAD blob `501b19bd`). **Mechanism correction measured:** the pin alone does not clear an already-stale index entry -- one `git checkout -- <path>` normalization (byte-identical re-smudge) was needed once because the entry's CRLF-checkout expectation predates the pin; the following full run proved the pin prevents recurrence. The pre-run churn signature (status ` M`, `git diff` empty) reproduced a TENTH time at T4.1 preflight and is now defunct. Original row text retained in-line after this disposition. `tests/fixtures/basic/artifacts/includes/report.json` is UNPINNED: `.gitattributes` (7 lines) sets `text eol=lf` for exactly two globs, `crates/ui/tests/fixtures/*.json` and `tests/golden/**/*.json`, and neither covers this path; `core.autocrlf=true` re-dirties its STAT ENTRY on every full test run even when content is unchanged. Measured **nine** times (T2-A f9, T2-B f10, T3-A sec 7, T3-B sec 6, T3-C, T3-D sec 3.4, T3-E2 f13 -- clean there -- plus the Phase 4 planner's own measurement, which re-reproduced the exact signature: `git status` reports the path `modified` while `git diff` for it returns NO hunks). Content is byte-equal to HEAD; only the stat cache is invalidated. Pre-existing test behaviour, not a milestone defect -- but it poisons any unstaged `commit_scoped.py` | Extend `.gitattributes` to `tests/fixtures/**/artifacts/**/*.json` (**preferred: config-only, no writer change**), prove it by re-running a full test pass and showing `git status` clean for the path, and pass an EXPLICIT `--scope` on every `commit_scoped.py`. T4.1 owns this; do NOT let the fixture appear in any commit -- it must be byte-equal to HEAD | M23.5 T1 verdict finding 12; T2-A f9; T2-B f10; T3-E2 f13 |
| M23.5 F11 residual (configuration/planner): the workspace delta `1147 -> 1156` carries an UNATTRIBUTED residual `+6 passed`. It cannot have been produced by Phase 1 (compile-time-fixed `#[test]` counts; one added file with three test functions; `scc.rs` diff confined above `#[cfg(test)]`). Most likely cause is the `main`-vs-branch skew: the only `main` commit absent from `tarjan-s-scc` is `e276910` (a Python tooling fix) | SCOPE NARROWED 2026-10-04: T2-A (+9/+1), T2-B (+4/+1) and **all four Phase 3 deltas** (A +5/+0, B +2/+0, C +0/+0, D +2/+0) were FULLY attributed, so the residual is confined to the Phase 1 step and does NOT block Phase 3 or D2. identify the +6 before T4.1 re-pins the baseline; until identified the milestone row records `1156/0/4/86` as MEASURED, not as an explained delta. Do not absorb silently | M23.5 T1 verdict finding 11 |
| M23.5 F13 (documentation, DOC-ONLY -- no code change) -- **CLOSED by T4.1 at `b3f33a9` (2026-10-04): the rename item is DONE.** `crates/ui/tests/ui_test/export_order.rs` now reads `the_non_vacuity_witness_detects_a_cycle_classification_difference` (test, was `the_parity_comparison_...`), `empty_currency_elements` (was `legacy_elements`, the `&[]` call) and `populated_currency_elements` (was `grouped_elements`, the `&groups` call); assertions, arguments, expected values and payload bytes untouched; `-p ariadne-ui --test ui_test` re-measured at `186/0/0/1` after the rename. The ledger-row correction itself was applied by the T3-E2 review cycle; the prior text was WRONG on all four counts** (it named two builders at `dto.rs:608` and `:637`, both E1-era names and line numbers, describing a state this milestone deliberately destroyed). Correct state at `dcd681b`: ONE public builder, `crates/ui/src/dto.rs:621` `pub fn build_include_elements`, third param `&[CycleGroup]`, and its tail `emit_include_elements` (`:643`) is private. The original slip stands unchanged: two SIGNED T3-D records name a non-existent `build_include_elements_legacy_paths`; the bad name occurs **ONLY under `tasks/`**. Detail + remaining item: sub-block below | **ITEM DONE by T4.1 (2026-10-04, `b3f33a9`):** F13 also owned P2's stale-identifier debris in `crates/ui/tests/ui_test/export_order.rs` -- locals `legacy_elements` (`:311`) and `grouped_elements` (`:318`), test name `the_parity_comparison_detects_a_cycle_classification_difference` (`:298`) -- which E2's contract FORBADE renaming, so E2 correctly left it. **T4.1 renamed all three; the `186/0/0/1` count pin held across the rename.** Never cite a `dto.rs` line under any name other than the one at that line today | M23.5 T3-D2 verdict section D + finding F1; T3-E2 verdict f9 + f10 |
| M23.5 F14 (planner/process; the "unverified claim about the tree" class -- **six contract defects this milestone** (the original, four recurrences, and the "fifth" stale line estimate); full evidence + three standing rules in the F14 sub-block below) | planner discipline, binding on every handoff in this milestone and the next one; ALSO the correction of record for the stale trailing sentence in `tasks/ledger_archive.md:27-36`. **COST SO FAR: one unit HELD at preflight with zero commits (unit E), one unit SPLIT IN TWO (E -> E1 + E2), and THREE reviewer repairs.** | `tasks/m235-t3e-journal.md` BLOCKER section (planner-authored); T3-B verdict finding 1; T3-D verdict finding 1; T3-E2 verdict findings 1-4 |
| M23.5 handoff sizing arithmetic (planner / `handoff-spec`, standing): T3-D's handoff predicted `crates/graph/src/graph.rs` at ~679; it landed at **734** over a measured base of 704 (+105/-75 = net +30). The growth is **71 lines of MANDATED ported tests** plus ~7 of mandated doc work against 56 lines of deleted production recursion -- nothing is wrong with the delivered file (266 lines of headroom, zero R1/R2, every function well under 100 lines). **ATTRIBUTION: planner, NOT the implementer** (T3-D Dev-1, ACCEPTED; T3-B Dev-3/D4 are the same defect class). The error is pricing additions without pricing the mandated Red test bodies and their fixtures | `handoff-spec` must require every addition estimate to be itemised FROM the mandated tests, as `tasks/m235-t3d2-handoff.md` sec11 does; never re-derive a count or a suite total from a ledger line | T3-D verdict finding 1 + Dev-1 |

### F10 detail -- why this BLOCKS the milestone verdict

**The escalation (T3-D verdict/exec-results sec7, 2026-10-04).** T3-D changed
what `IncludeReport::cycle_count` *MEANS*: it is now an entangled-group count,
and it is exactly the number the UI include view and
`crates/cli/src/commands/includes.rs`'s "Cycle Groups Detected" label display.
Before T3-D those two were **inconsistent by construction** (two
representations, never compared); after T3-D they agree by construction, which
is a real improvement -- and still unobserved at runtime. Perf is the same
shape: T3-D changed what `generate_report()` computes internally (iterative
Tarjan, not DFS path enumeration), so every timing in
`crates/cli/tests/payload_tail_bench.rs` has changed meaning again and remains
**UNVERIFIED**.

**Why a green compiler-level pass does not substitute.** T3-E2 measured
`1180/0/4/88`, zero R1/R2 and a clean encoding guard, and none of those
observes a socket. The only in-tree statement of the gap is the `UNVERIFIED`
paragraph at `payload_tail_bench.rs:80-87` and `crates/cli/src/commands/ui.rs:115-119`,
both verified byte-intact at `dcd681b`. **Do not weaken either.** The residual
`5.1` (`ui.rs:103`) vs `5.4` (`payload_tail_bench.rs:41`) split for the same
phase is a perf claim; reconciling it would manufacture exactly the unverified
timing evidence this milestone has been disciplined about refusing. T3-E2
recorded it as observation O-1 and left it alone, and that was correct.

### F13 detail -- the corrected record, and what T4.1 still owes (ALL DISCHARGED 2026-10-04)

**What was wrong.** The prior row named two builders at `dto.rs:608` and
`:637`. E1 deleted the path-based builder and its `IncludeCycleClass::from_paths`
helper; E2 renamed the survivor to the plan's canonical name. Both names and
both line numbers therefore described a state this milestone deliberately
destroyed, and the row would have misled the next reader about the UI layer's
shape. The T3-E2 review required the correction (finding 9); this paragraph
discharges that requirement.

**What was NOT wrong, and must not be rewritten.** `tasks/m235-t3d-verdict.md:107`
and `tasks/m235-t3d-execution-results.md:24` call a non-existent
`build_include_elements_legacy_paths`. That bad name occurs **ONLY under
`tasks/`** (zero hits under `crates/**` and `tools/**`), and both records cite
the correct SPAN with a wrong IDENTIFIER -- a transcription slip, not code
drift, first caught at `tasks/m235-t3d2-journal.md` Checkpoint 0. **They are
signed adjudication artifacts: do NOT silently rewrite them.** This row plus
`tasks/m235-t3d2-verdict.md` section D is the correction of record.

**What T4.1 still owes.** P2 in `crates/ui/tests/ui_test/export_order.rs` was
written against the two-builder era and carries three stale identifiers:
`legacy_elements` (`:311`), `grouped_elements` (`:318`) and the test name
`the_parity_comparison_detects_a_cycle_classification_difference` (`:298`).
E2's contract forbade the rename (it bars test renames and permits only
callee-only body edits), so E2 correctly left all three and reported it as
observation O-3. **Nothing forbids the rename now**, and it is the only reason
F13 is still open rather than discharged.

**DISCHARGED by T4.1 (2026-10-04, `b3f33a9`).** All three renamed in place:
test -> `the_non_vacuity_witness_detects_a_cycle_classification_difference`
(its own doc calls it the non-vacuity witness for the frozen lock, so the
name no longer falsely claims a "parity" comparison); `legacy_elements` ->
`empty_currency_elements` (the `&[]` call); `grouped_elements` ->
`populated_currency_elements` (the `&groups` call). Assertions, arguments,
expected values and the frozen payload literal untouched; the `186/0/0/1`
count pin re-measured after the rename. F13 is CLOSED.

### F14 detail -- the repaired E contract shipped UNSATISFIABLE, caught pre-commit

**THE THREE STANDING RULES EXISTED AND WERE LOAD-BEARING.** The first
occurrence was caught only because the implementer ran an executed preflight
probe instead of trusting the contract. Every rule below was written by the
planner AFTER that incident and then bound the two replacement units; this row
is not archival history, it is the live discipline for the next milestone.

**THE CLASS RECURRED FOUR MORE TIMES, and a FIFTH contract defect was a stale
line estimate.** All five are planner-attributed, all were caught in review
rather than in production, and none was the implementer's fault:

| # | defect | unit | caught by |
|---|---|---|---|
| 1 | E's touch-set froze `crates/cli/**` while mandating a rename that breaks it | E | **implementer's executed preflight probe, zero commits** |
| 2 | E2's contract froze a touch-set that excluded a file it then had to edit (`common.rs:3` named the dead symbol) | E2 | E1's reviewer, blocking |
| 3 | E2's census predicted **19** call sites in a form that returns **11** (pattern/category mismatch: `dto.rs:601` spelled the plain name and cannot match a `_from_groups` pattern) | E2 | implementer, reconciled in journal |
| 4 | E2's Red predicted a rustc error-count identity (`17 in-crate + 2 cli = 19`) that is **structurally unobtainable**: every in-crate caller imports the name, so rustc emits the 5 `E0432` import errors and stops -- and sec4 mandated "repoint nothing", so the predicted `E0425`s could never appear | E2 | implementer, substituted a STRONGER argument; reviewer upheld |
| 5 | E2's gate 7 pinned a rustc **warning count** for a crate whose warnings all live outside the touch-set; the count is a function of cargo's diagnostic cache, so the identical command at the identical commit returned **21** to the reviewer and **9** to the implementer, both exit 0 | E2 | implementer, disclosed; reviewer adjudicated |
| 6 | handoff line estimates undercount additions because the mandated Red test bodies were never priced (T3-D: `graph.rs` predicted ~679, landed **734**) -- this is the "FIFTH contract defect" | T3-D | implementer Dev-1, ACCEPTED |

**COST: one unit HELD (E, zero commits), one unit SPLIT IN TWO, three
reviewer repairs.** The pattern is uniform -- every one was a claim about the
tree, or about an output derived from the tree, asserted without the command
that establishes it.

**WHAT CHANGES TO PREVENT A SIXTH.** Four changes, all in the planner's own
authoring step, all mechanical:

1. **Every census row carries its own command.** No `expected N` without
   `python tools/distill.py -- <grep command> ; expect N`. A count stated
   without the command that produces it is not a pin, it a guess with
   arithmetic.
2. **Never pin a number the contract cannot force to be observable.** Check the
   step ordering first: if acceptance requires output that only a later step
   can produce, pin the OUTCOME (green + byte lock + zero warnings) instead of
   the intermediate count. Defect 4 was unfixable by any implementer.
3. **Pin warning COUNTS only for gates that re-check the whole crate from
   cold**; otherwise pin exit status alone. A cache-dependent tally is a
   false-positive generator, and this milestone now has four behind it.
4. **Price every addition FROM the mandated Red test bodies**, itemised, per
   `tasks/m235-t3d2-handoff.md` sec11. Never re-derive a count or a suite
   total from a ledger line.

**What happened.** `tasks/m235-t3e-handoff.md` (949 lines, repaired once
already) mandated, in its sec3.3, that the name
`build_include_elements_from_groups` cease to exist (rename to
`build_include_elements`), while sec2 and sec8 froze `crates/cli/**`
byte-unchanged. Three tree facts make those clauses unsatisfiable together:

1. `crates/cli/src/commands/ui.rs:328` (production `build_ui_payload`) calls
   `ariadne_ui::build_include_elements_from_groups`.
2. `crates/cli/tests/payload_tail_bench.rs:325` (inside `run_assembly`) calls it
   too. Those are the ONLY two code references outside `crates/ui`.
3. `crates/ui/src/lib.rs:6` is a bare `pub use dto::*;` -- the sole export path,
   no alias and no shim, so a name removed from `dto.rs` is gone workspace-wide.

**How it was caught, and why that is the finding.** The implementer did not
report a guess: it executed a preflight probe -- rename `dto.rs:637` to a
throwaway name, one line, everything else untouched -- and
`cargo check -p ariadne-cli --all-targets` failed with exit 101,
`error[E0425]: cannot find function build_include_elements_from_groups in crate
ariadne_ui` at `crates/cli/src/commands/ui.rs:328:40`; it then reverted
byte-for-byte, proven by matching `crates/ui/src/dto.rs` SHA256 before and after,
and committed nothing. Evidence: `tasks/m235-t3e-journal.md`, BLOCKER section.

**Two structural causes, both now standing rules.**

- The preflight grep targeted the LEGACY name's callers and therefore missed the
  grouped-name hazard entirely; the contract's own caller census called the two
  `crates/cli` occurrences "report-only, do not touch" while mandating the
  rename that would break them.
- The unit's gate set never type-checked `crates/cli` (gates were `-p ariadne-ui`
  scoped plus a `cargo build -p ariadne-ui`), so had the implementer not probed,
  the unit would have landed **all-green and broken the workspace**, first
  surfacing at T4.1 as a milestone-level regression.

**THREE STANDING RULES, binding on every handoff from here.**

0. **A standing rule that has never fired is not a rule -- it is a hope.**
   Rules 1-3 below were written after defect 1 and did their job on E1/E2
   (E1 carried `cargo check -p ariadne-cli --all-targets` precisely because
   rule 2 named that reverse dependency; E2 gated BOTH reverse dependencies at
   BOTH ends). Keep firing them.
1. **A caller census greps EVERY name the unit adds, removes or renames.** One
   name is not a census. Enumerate from the tree at the dispatch base, in the
   same pass that writes the contract, and state an expected count plus an
   explicit STOP condition when it is not met.
2. **A unit that changes a public function name must build its REVERSE
   dependencies**, enumerated from the workspace `Cargo.toml` files rather than
   assumed. For `crates/ui` those are `ariadne-cli` (`crates/cli/Cargo.toml:34`,
   the only crate with live references) and `ariadne-integration-tests`
   (`tests/integration/Cargo.toml:25`, a declared path dependency with zero
   textual references today). Gate `-p ariadne-cli --all-targets` AND
   `-p ariadne-integration-tests --all-targets`; `-p ariadne-ui` alone proves
   nothing about them.
3. **A required deletion's blast radius joins the touch-set.** `from_paths` is
   `pub(crate)` in a `pub(crate)` module with exactly one caller, inside the
   deleted function, so deleting the caller without the callee makes gate
   "zero warnings" unreachable -- a contract that requires an unreachable gate
   has handed the reviewer a contradiction, not a task.

**Precedents, same defect class, all planner-attributed:** T3-B finding 1 (a Red
line reporting `0 failed` while describing a failing test) and T3-D finding 1 (a
size estimate undercounted by 71 lines because additions were priced without the
mandated test bodies). T3-D2 Dev-1/Dev-3 are the same class again. This row is
the named row the milestone needed for the class.

**Also the correction of record for one stale sentence.** The trailing note in
`tasks/ledger_archive.md:27-36` still says "E is AUTHORIZED, repaired in place,
dispatch-pending". That is false as of this row. **Do NOT rewrite it:** archive
rows and signed reviewer records are verbatim audit trail, corrected by ledger
row -- the same treatment F13 mandates for `tasks/m235-t3d-verdict.md:107` and
`tasks/m235-t3d-execution-results.md:24`. E1's commit 1 lands
`tasks/ledger_archive.md` byte-as-found.

## Graduation backlog (D4: warn-only -> blocking flips; owner = next kickoff runner)

| Check | Current mode | Flip condition (propose dated target at kickoff) |
|---|---|---|
| `hygiene_check.py` R3 naming rule (`tmp`/`res`/`foo`/`bar`/`test1`, single-letter locals) | warn-only; per-file+line allow-list for legacy | flip after one full milestone with zero new R3 findings |
| clippy `-D warnings` (125-warning baseline at `f7ace60`, ci.yml comment) | warn-only (plain clippy) | flip when baseline reaches 0 |
| `tools/bench_compare.py` stage-5 25% median regression | local gate only; GitHub-runner timings incomparable | CI adoption needs a dedicated self-hosted runner class |
| `tools/theme_token_check.py --check-added` hex drift | baseline-committed drift guard | Phase B token parity belongs to the theming plan's own Rust tests, NOT here |

## Byte-locked sets (as of 2026-10-01)

| Set | Branch | Regeneration protocol |
|---|---|---|
| `tests/golden/**` | `multi-language-support` only | `UPDATE_GOLDENS=1`, then manifest hash resync; no new files inside `tests/fixtures/basic` / `tests/fixtures/expensive` |
| `tests/fixtures/**` | all branches | frozen; changes need explicit review approval |
| `tools/theme_token_baseline.json` hex-location snapshot | `main` | regenerate only in a commit that justifies each new/removed literal |