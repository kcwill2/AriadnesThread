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
