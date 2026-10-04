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

## Milestones

| Milestone | Status | Where (commit/branch) | Frozen locks | Notes |
|---|---|---|---|---|
| M18 header-dependency migration + UI story | COMPLETE (audit 2026-10-01: intent confirmed, zero regressions, 8 findings dispositioned) | `main` (merged; audit vs spec at df57491) | `tests/golden/**` byte-locked, manifest hash `0xddc3...` resync via `UPDATE_GOLDENS=1` only | M18.11 session-death incident -> T1 checkpoint_guard |
| M19 Scala plugin (`crates/lang-scala`) | COMPLETE on branch, NOT merged to `main` | `multi-language-support` @ `32ecf33` (worktree `.kilo/worktrees/linen-mice`) | same as M18 on that branch; `tests/golden/**` does NOT exist on `main` | 715 tests, cpp parity 22/22, walkthrough committed; handoffs `tasks/m19-t{a,b}-handoff.md` live on the branch |
| M20 Scala resolver | STARTING (independent session + own worktree per 2026-10-01 handoff) | new branch off `multi-language-support` | TBD at kickoff | StubResolver replacement |
| Process tooling codification | COMPLETE on `main` @ T11 feat `592de60` (T1-T5, T7-T14; 153 tool tests green, CI consolidated discover step) | `.kilo/plans/1790831557705-process-tooling-codification.md` | `tools/theme_token_baseline.json` (132 literals, 8 files), `tools/hygiene_r3_allowlist.txt` (150 legacy entries) | T6 golden_guard deferred to M19 merge (row below) |
| Tool output distillation (contract + distillers) | COMPLETE (A gated APPROVED `1b7bc3f`; B gated APPROVED `a2f94af`) | spec `.kilo/plans/1791044160000-tool-output-distillation.md`; `docs/specs/tool-output-distillation.md` | none | I8 SUPERSEDED 2026-10-04 by the five-role topology row below (enforcement now lives in the permission allowlist, not in tools or delegation prose); F1/F2 fixed in B; F3/F4 closed as process lessons; global `~/.agents/skills/tool-runner/SKILL.md` amended out-of-band 2026-10-02 (raw paste fields deleted, digest contract + routing added; vendoring declined to avoid shadow drift); handoff C discharged deferred O1 + B-gate pin (`f3dc53b`/`c600fb8`, gate APPROVED zero findings; rows archived to `tasks/ledger_archive.md`) |
| M23.5 cycle substrate: SCC condensation + iterative Tarjan | IN PROGRESS -- Phase 1 (T1.1+T1.2, crate:cluster) APPROVED 2026-10-04, 0 blocking / 12 non-blocking; Phase 2 (T2.1-T2.3, crate:graph) APPROVED and CLOSED 2026-10-04 -- T2-A `34f5fe4`+`7c94af3`, T2-B `08741b8`+`b950963`, 0 blocking each; Phase 3 (T3.1/T3.2) AUTHORIZED 2026-10-04 as FIVE units `tasks/m235-t3{a,b,c,d,e}-handoff.md`, not started; Phase 4 (T4.1) NOT authorized | branch `tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`; base `2708425`; Phase 1 = `dd3db11` (parity oracle + iterative frame machine) + `e2a05e2` (journal); HEAD `b950963`; NOT merged to `main` | IV-1..IV-12 (plan sec1.2); R1/R2 hygiene (`hygiene_check.py`, blocking); UTF-8 mojibake (`encoding_guard.py`); zero new UI hex literals (`theme_token_check.py`); planner byte-identity UNEDITED (`planner_facts_parity`, `planner_characterization`, `m00_facts_path_parity_test`); `tests/golden/**` LF via `.gitattributes`; cycle-policy seam (`generate_report_with_cycle_policies`, `informational_cycle_count` skip-when-zero serde); D1-B = no dual cycle representation | **BASELINE CHAIN (all reviewer-measured, fully attributed, zero residual):** `1156/0/4/86` pre-Phase 1 -> `1165/0/4/87` after T2-A (+9/+1) -> **`1169/0/4/88` at `b950963`** after T2-B (+4/+1). Crate-scoped: `-p ariadne-graph` = `116/0/0/5`; `-p ariadne-ui --test ui_test` = `179/0/0/1`; `-p ariadne-cli` = `86/0/1/3`. F11 (the Phase 1 +6) stays OPEN but is NOT a Phase 3 blocker. **Phase 3 shape (2026-10-04):** 5 one-crate units A `crates/ui` (additive DTO) -> B `crates/cli` (incl. bench re-pin) -> C `tests/integration` -> D `crates/graph` (IV-11 port + **atomic delete** of `detect_cycles`/`dfs_cycle` with its last repoint) -> E `crates/ui` (drop the path-based DTO); C is file-disjoint from A and B. `tests/golden/*/includes_report.json` hold `"cycle_count": 0` only -- no golden carries a concrete cycle list, so a Phase 3 golden re-pin is expected to be a NO-OP and a mismatch is a FINDING. |
| Agent topology: five-role, permission-enforced | COMPLETE (verified end-to-end by 4-hop synthetic smoke test, 2026-10-04) | plan `.kilo/plans/1791072847000-five-role-agent-topology.md`; agents `~/.config/kilo/agents/*.md` (GLOBAL, affects all Kilo projects) | none in-repo; global config backups `kilo.jsonc.bak-20261004`, `kilo.jsonc.pre-agents-strip`, `agents/code-reviewer.md.bak-20261004` | Replaces the four-role hybrid: `tool-caller`, `review-guide`, `code-reviewer` DELETED. Roster: orchestrator (primary) + architect/planner/implementer/reviewer/walkthrough-builder (subagents), all hops via `task`. Four probed constraints: (1) `TaskTool` stamps `task: deny` on every child, so subagents cannot nest -- a subagent keeps `task` only as a top-level session; (2) `mode: primary` agents cannot be task-spawned at all; (3) permission objects are LAST-match-wins, so the catch-all `"*": deny` must come FIRST; (4) a caller's `edit: deny` is inherited as a hard ceiling that strips `write`+`edit` from every child, so the orchestrator holds `edit: ask`. `background_process` and mutating `git_git_*` are gateable and denied in all six. Known limits: `distill.py` is an argv pass-through so it bounds OUTPUT not command scope; permission denials always report the catch-all rule, which caused one reviewer to skip five permitted gates (role prompts now instruct per-gate attempts). M23.5 handoffs + plan repointed off `tool-caller`/`Code-reviewer` same day |

## Worktree census obligations (E16)

Every participating worktree must end merged, or abandoned WITH a ledger row
below. Closeout checks run `python tools/checkpoint_guard.py status` (census).

| Worktree | State at 2026-10-01 | Disposition |
|---|---|---|
| `.kilo/worktrees/troubled-concrete` | DETACHED HEAD @ `1aa03b3`, own branch deleted; held verified-but-never-committed theming work (lost) | needs `git worktree remove` at next kickoff once operator confirms nothing to salvage |
| `.kilo/worktrees/linen-mice` | M19 branch, complete | merge or abandon with row |
| `.kilo/worktrees/jeweled-neon` | at `main` tip, no branch work | abandon with row |
| `.kilo/worktrees/load-tail-phase-performance` | merged via `7a58e1f` | abandon with row |

## Deferred findings (with target milestone)

| Finding | Target | Source |
|---|---|---|
| Execute theming plan `1790656207440` (three themes, D1-D8) - implementation VERIFIED IN WORKTREE THEN LOST (E16); its own T8 parity + zero-hex Rust tests own theme-token Phase B | next UI capacity window; redo required | E16 / T13 |
| lang-scala guard gaps | M23 | M19 kickoff (D9) |
| `test_output.txt`, `test_results.txt`, `parse.py` are UTF-16 PowerShell-captured debris committed ~2026-09-25; invisible to encoding_guard's binary (NUL) skip | delete at next hygiene pass | found during T2 build 2026-10-01 |
| `.css` invisible to R2 size gate (`styles.css` 1422 lines and growing; `R2_SCANNED_SUFFIXES` excludes `.css`) | separate decision (day-one failure risk) | plan residual risk |
| `bench_baseline.json` has 3 phases baselined at 0.0 ms (both `get_or_build_load hand-out` rows, `conservative filter pass`); any future nonzero median fires WARN +inf% -- add a min-abs floor if noisy; bench doc-comment phase labels are STALE (live rows print `hand-out`/`facts+closures`, not `clone`/`cold`) | fix when bench table next touched | T11 build 2026-10-01 |
| T6 `tools/golden_guard.py` - specced, unimplemented: `tests/golden/**` exists only on `multi-language-support` | at/before M19 merge | D5 |
| Three untracked strays (`.kilo/plans/1790697359283-systems-layer-engine-room.md`, `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md`, `.kilo/skills/deslop/`) need owner decision: commit or discard | operator | tool-distill A gate |
| M23.5 F9 (planner/configuration): `python tools/testrun.py --baseline` is UNRUNNABLE as documented in `AGENTS.md`, plan T4.1, and the reviewer agent definition -- `--baseline` requires a FILE argument and no `N/M/K/S` baseline file exists anywhere in the repo. Verbatim: `usage error: missing value for --baseline` (exit 2) | fix `testrun.py` to default the argument, OR land a committed baseline file holding the CURRENT canonical line `1169/0/4/88`; until then every milestone gate compares BY HAND against the M23.5 milestone row's baseline chain `1156/0/4/86` -> `1165/0/4/87` -> `1169/0/4/88`. Before T4.1 closeout | M23.5 T1 verdict finding 9 |
| M23.5 F10 (configuration): the reviewer bash allowlist omits `python tools/ui_smoke.py` and `python tools/bench_compare.py`, yet the same agent file and plan T4.1 both order them -- both refused verbatim with the catch-all rule, reconfirmed independently in T2-A finding 9 and T2-B finding 9 (neither routed through `distill.py`). CONSEQUENCE: there is still NO ui-smoke evidence and NO perf-regression evidence for `detect_cycle_groups`, which is a real surface (`payload_tail_bench.rs` times the cycle path, and `compute_symbol_sccs` sits in the planner's per-header hot path) | operator amends the permission allowlist (enforcement lives in the allowlist, not in agent prose) BEFORE T4.1 closeout, or an authorized session runs both. `tasks/m235-t3b-handoff.md` re-pins `payload_tail_bench.rs` against the recorded baseline and MUST label the comparison UNVERIFIED (no `bench_compare.py`); it must not imply perf evidence that does not exist. Do NOT route either through `distill.py` to evade the denial | M23.5 T1 verdict finding 10; T2-A finding 9; T2-B finding 9 |
| M23.5 F12 (housekeeping): `tests/fixtures/basic/artifacts/includes/report.json` is UNPINNED (`.gitattributes` sets `text eol=lf` only for `crates/ui/tests/fixtures/*.json` and `tests/golden/**/*.json`) and `core.autocrlf=true` (measured) re-dirties its STAT ENTRY on every full test run even when content is unchanged. Measured three times (T2-A finding 9, T2-B finding 10): `git ls-files --eol` stays `i/lf w/lf`, no `\r` byte is present, and `git diff --numstat HEAD -- <path>` returns no rows. Content is byte-equal to HEAD; only the stat cache is invalidated. Pre-existing test behaviour, not a milestone defect -- but it poisons any unstaged `commit_scoped.py` | RESTORE TO LF if it appears: preflight step 0 of every Phase 3 handoff (byte-safe Kilo Read/Write only; mutating git is denied to every role). EVERY `commit_scoped.py` in Phase 3 MUST carry an EXPLICIT `--scope`, and this path MUST appear in NO Phase 3 commit. Extend `.gitattributes` to `tests/fixtures/**/artifacts/**/*.json`, or make the artifact writer emit LF, at closeout BEFORE T4.1 | M23.5 T1 verdict finding 12; T2-A finding 9; T2-B finding 10 |
| M23.5 F11 residual (configuration/planner): the workspace delta `1147 -> 1156` carries an UNATTRIBUTED residual `+6 passed`. It cannot have been produced by Phase 1 (compile-time-fixed `#[test]` counts; one added file with three test functions; `scc.rs` diff confined above `#[cfg(test)]`). Most likely cause is the `main`-vs-branch skew: the only `main` commit absent from `tarjan-s-scc` is `e276910` (a Python tooling fix) | SCOPE NARROWED 2026-10-04: T2-A (+9/+1) and T2-B (+4/+1) were both FULLY attributed, so the residual is confined to the Phase 1 step and does NOT block Phase 3. identify the +6 before T4.1 re-pins the baseline; until identified the milestone row records `1156/0/4/86` as MEASURED, not as an explained delta. Do not absorb silently | M23.5 T1 verdict finding 11 |

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
