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
| M23.5 cycle substrate: SCC condensation + iterative Tarjan | IN PROGRESS -- Phase 1, Phase 2 (CLOSED) and Phase 3 units A/B/C/D/D2 all APPROVED 2026-10-04; unit E SUPERSEDED (unsatisfiable contract, preflight STOP with zero commits) and replaced by units E1 + E2, both NOT started; Phase 4 (T4.1) NOT authorized. Per-unit detail: sub-block below | branch `tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`; base `2708425`; HEAD `b0a91d2`; NOT merged to `main` | IV-1..IV-12; R1/R2 hygiene; UTF-8 mojibake; zero new UI hex literals; planner byte-identity UNEDITED; `tests/golden/**` LF; cycle-policy seam; D1-B (full list in sub-block) | Baseline chain ends `1178/0/4/88` at `aebfa9b`. Open: units E1/E2 + deferred F9, F10, F11, F12, F13, F14 below |
| Agent topology: five-role, permission-enforced | COMPLETE (verified end-to-end by 4-hop synthetic smoke test, 2026-10-04) | plan `.kilo/plans/1791072847000-five-role-agent-topology.md`; agents `~/.config/kilo/agents/*.md` (GLOBAL, affects all Kilo projects) | none in-repo; global config backups `kilo.jsonc.bak-20261004`, `kilo.jsonc.pre-agents-strip`, `agents/code-reviewer.md.bak-20261004` | Replaces the four-role hybrid: `tool-caller`, `review-guide`, `code-reviewer` DELETED. Roster: orchestrator (primary) + architect/planner/implementer/reviewer/walkthrough-builder (subagents), all hops via `task`. Four probed constraints: (1) `TaskTool` stamps `task: deny` on every child, so subagents cannot nest -- a subagent keeps `task` only as a top-level session; (2) `mode: primary` agents cannot be task-spawned at all; (3) permission objects are LAST-match-wins, so the catch-all `"*": deny` must come FIRST; (4) a caller's `edit: deny` is inherited as a hard ceiling that strips `write`+`edit` from every child, so the orchestrator holds `edit: ask`. `background_process` and mutating `git_git_*` are gateable and denied in all six. Known limits: `distill.py` is an argv pass-through so it bounds OUTPUT not command scope; permission denials always report the catch-all rule, which caused one reviewer to skip five permitted gates (role prompts now instruct per-gate attempts). M23.5 handoffs + plan repointed off `tool-caller`/`Code-reviewer` same day |

### M23.5 detail (per-unit detail lives here and in the verdicts, never back in the table row)

- **Phase 1 (T1.1+T1.2, crate:cluster):** APPROVED 2026-10-04, 0 blocking / 12 non-blocking. `dd3db11` (parity oracle + iterative frame machine), `e2a05e2` (journal).
- **Phase 2 (T2.1-T2.3, crate:graph):** APPROVED and CLOSED 2026-10-04. T2-A `34f5fe4`+`7c94af3`, T2-B `08741b8`+`b950963`, 0 blocking each.
- **Phase 3 (T3.1/T3.2) units A/B/C/D:** all APPROVED 2026-10-04. A `31513b9..ea82422`, B `9d831bf..91967c9`, C `b774362..6c0c2b6`, D `adb53a3..aebfa9b` (0 blocking / 13 non-blocking).
- **Unit D2 (inserted companion, crate:ui legacy/group parity lock, one test file):** APPROVED, 0 blocking, 12 findings. `42bb44e`+`4d0f245`+`b0a91d2` (HEAD).
- **Unit E (`tasks/m235-t3e-handoff.md`): SUPERSEDED 2026-10-04. NOT started, zero commits, nothing landed.** Its implementer stopped at preflight with an executed probe: renaming `dto.rs:637` alone made `cargo check -p ariadne-cli --all-targets` fail (exit 101, `error[E0425]: cannot find function build_include_elements_from_groups in crate ariadne_ui` at `crates/cli/src/commands/ui.rs:328:40`) while the contract froze `crates/cli/**` byte-unchanged and `crates/ui/src/lib.rs:6` is a bare `pub use dto::*;`. The contract shipped unsatisfiable -- see deferred row **F14** and its sub-block. **Replaced by two units, both NOT started:**
  - **E1** `tasks/m235-t3e1-handoff.md` -- crate:ui only. Deletes the legacy `build_include_elements` (`dto.rs:608`) and its only helper `IncludeCycleClass::from_paths`, repoints the NINE in-crate call sites, collapses four two-name import lists, converts parity test P1 to a frozen-expectation byte lock. It carries P1's conversion **because it deletes the legacy builder**: once that builder is gone, P1 compares the survivor against itself. Adds `cargo check -p ariadne-cli --all-targets` as a gate. Sizing: 1 crate, 7 code paths + journal, 7 gate commands / 15 invocations (itemised in the handoff), ~250 produced lines.
  - **E2** `tasks/m235-t3e2-handoff.md` -- renames the survivor to the plan's canonical public name (plan line 69). The only unit with a declared 2-crate exception; touches `crates/cli/src/commands/ui.rs:328` and `crates/cli/tests/payload_tail_bench.rs:325` (plus one stale comment clause at `:64`); gates BOTH reverse dependencies (`-p ariadne-cli --all-targets`, `-p ariadne-integration-tests --all-targets`). Sizing: 2 crates, 7 code paths + journal, 8 gate commands / 14 invocations, ~70 produced lines.
  - Split rationale: the deletion and the rename are separable across a compile-green boundary (verified in both directions), and only the rename needs `crates/cli`. E1 cannot carry the rename, because after E1 the sole surviving name is the one `crates/cli` calls.
  - **Superseded contract disposition: SUPERSEDED, retained byte-unchanged on disk, not archived and not deleted.** E1 commit 1 lands it as the record of what was dispatched and rejected; this sub-block and row F14 are the supersession of record.
  - D2 findings **F1** (stale counts/census) and **F2** (P1 tautology) stay discharged: F1 by E1's re-derived caller census, F2 by E1's P1 conversion.
  - Test counts move in NEITHER unit: `--test ui_test` stays `186 / 0 / 0 / 1` and `-p ariadne-ui` stays `186 / 0 / 0 / 3` (186 - 0 + 0 in both). D2's `+2` to the workspace line remains a PREDICTION, never a measurement.
- **Phase 4 (T4.1):** NOT authorized.
- **Baseline chain** (all reviewer-measured, every delta attributed, zero residual): `1156/0/4/86` pre-Phase 1 -> `1165/0/4/87` T2-A (+9/+1) -> `1169/0/4/88` T2-B (+4/+1) -> `1174/0/4/88` T3-A (+5/+0) -> `1176/0/4/88` T3-B (+2/+0) -> T3-C (+0/+0) -> **`1178/0/4/88` at `aebfa9b`** T3-D (+2/+0).
- **Crate-scoped lines:** crate:cluster (Phase 1); crate:graph (T2-A, T2-B, T3-C, T3-D); crate:ui (D2 only -- one test file).
- **Frozen locks (full):** IV-1..IV-12 (plan sec1.2); R1/R2 hygiene (`hygiene_check.py`, blocking); UTF-8 mojibake (`encoding_guard.py`); zero new UI hex literals (`theme_token_check.py`); planner byte-identity UNEDITED (`planner_facts_parity`, `planner_characterization`, `m00_facts_path_parity_test`); `tests/golden/**` LF via `.gitattributes`; cycle-policy seam (`generate_report_with_cycle_policies`, `informational_cycle_count` skip-when-zero serde); D1-B = no dual cycle representation.
- **Accepted, do NOT re-open:** T3-B's two-process export lock is APPROVED and its two-sample residual was explicitly ACCEPTED (T3-B verdict sec5.6).
- **Open deferred rows (detail in the table below):** F9 (`testrun.py --baseline` unrunnable as documented), F10 (reviewer allowlist denies `ui_smoke.py` + `bench_compare.py` -- the most material unclosed gate; must close before T4.1), F11 (unattributed `+6` pre-Phase-1 residual), F12 (`includes/report.json` stat churn poisons unstaged `commit_scoped.py`), F13 (signed-record identifier slip, DOC-ONLY), F14 (a repaired handoff shipped unsatisfiable; three standing planner rules -- detail sub-block below).
- **Where per-unit detail now lives:** `tasks/m235-t1-verdict.md`, `-t2a-`, `-t2b-`, `-t3a-`, `-t3b-`, `-t3c-`, `-t3d-`, `-t3d2-verdict.md` (plus each unit's `-journal.md` and, where one exists, its `-execution-results.md`), the **superseded** unit-E handoff `tasks/m235-t3e-handoff.md` and its preflight-stop journal `tasks/m235-t3e-journal.md`, the replacement handoffs `tasks/m235-t3e1-handoff.md` and `tasks/m235-t3e2-handoff.md`, and `tasks/m235-kickoff.md`. No `tasks/m235-t3e-execution-results.md` exists on disk and none is referenced.

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
| M23.5 F9 (planner/configuration): `python tools/testrun.py --baseline` is UNRUNNABLE as documented in `AGENTS.md`, plan T4.1, and the reviewer agent definition -- `--baseline` requires a FILE argument and no `N/M/K/S` baseline file exists anywhere in the repo. Verbatim: `usage error: missing value for --baseline` (exit 2) | fix `testrun.py` to default the argument, OR land a committed baseline file holding the CURRENT canonical line (**the M23.5 milestone row is the single copy -- do not restate the chain here**; as of `aebfa9b` it ends `1178/0/4/88`); until then every milestone gate compares BY HAND against that row's baseline chain. Before T4.1 closeout | M23.5 T1 verdict finding 9 |
| M23.5 F10 (configuration) -- **THE MOST MATERIAL UNCLOSED GATE IN THE MILESTONE; must close before T4.1:** the reviewer bash allowlist omits `python tools/ui_smoke.py` and `python tools/bench_compare.py`, yet the same agent file and plan T4.1 both order them -- both refused verbatim with the catch-all rule, reconfirmed independently in T2-A finding 9, T2-B finding 9, T3-A, T3-B, T3-C and T3-D (six units; none routed through `distill.py`). **ESCALATED 2026-10-04 by the T3-D verdict/exec-results sec7:** T3-D changed what `IncludeReport::cycle_count` *MEANS* -- it is now an entangled-group count, and it is exactly the number the UI include view and `crates/cli/src/commands/includes.rs`'s "Cycle Groups Detected" label display. Before T3-D the UI's group-derived classification and the report path count were **inconsistent by construction** (two representations, never compared); after it they agree by construction, which is a real improvement -- yet there is still **ZERO runtime evidence that the UI cold-boots, routes, serves `Cache-Control: no-store`, and returns a group-count report**. Perf is the same shape: T3-D changed what `generate_report()` computes internally (iterative Tarjan, not DFS path enumeration), so the timing at `crates/cli/tests/payload_tail_bench.rs:322` has changed meaning again and remains **UNVERIFIED** | operator amends the permission allowlist (enforcement lives in the allowlist, not in agent prose) BEFORE T4.1 closeout, or an authorized session runs both. `tasks/m235-t3b-handoff.md` re-pins the bench row against the recorded baseline and MUST keep labelling that comparison UNVERIFIED (no `bench_compare.py`); it must not imply perf evidence that does not exist. Do NOT route either through `distill.py` to evade the denial | M23.5 T1 verdict finding 10; T2-A f9; T2-B f9; T3-B f6; T3-D f9 + exec-results sec7 |
| M23.5 F12 (housekeeping): `tests/fixtures/basic/artifacts/includes/report.json` is UNPINNED (`.gitattributes` sets `text eol=lf` only for `crates/ui/tests/fixtures/*.json` and `tests/golden/**/*.json`) and `core.autocrlf=true` (measured) re-dirties its STAT ENTRY on every full test run even when content is unchanged. Measured **six** times (T2-A finding 9, T2-B finding 10, T3-A sec 7, T3-B sec 6, T3-C, T3-D sec 3.4): `git ls-files --eol` stays `i/lf w/lf`, no `\r` byte is present, and `git diff --numstat HEAD -- <path>` returns no rows. Content is byte-equal to HEAD; only the stat cache is invalidated. Pre-existing test behaviour, not a milestone defect -- but it poisons any unstaged `commit_scoped.py` | RESTORE TO LF if it appears: preflight step 0 of every Phase 3 handoff (byte-safe Kilo Read/Write only; mutating git is denied to every role). EVERY `commit_scoped.py` in Phase 3 MUST carry an EXPLICIT `--scope`, and this path MUST appear in NO Phase 3 commit. Extend `.gitattributes` to `tests/fixtures/**/artifacts/**/*.json`, or make the artifact writer emit LF, at closeout BEFORE T4.1 | M23.5 T1 verdict finding 12; T2-A finding 9; T2-B finding 10 |
| M23.5 F11 residual (configuration/planner): the workspace delta `1147 -> 1156` carries an UNATTRIBUTED residual `+6 passed`. It cannot have been produced by Phase 1 (compile-time-fixed `#[test]` counts; one added file with three test functions; `scc.rs` diff confined above `#[cfg(test)]`). Most likely cause is the `main`-vs-branch skew: the only `main` commit absent from `tarjan-s-scc` is `e276910` (a Python tooling fix) | SCOPE NARROWED 2026-10-04: T2-A (+9/+1), T2-B (+4/+1) and **all four Phase 3 deltas** (A +5/+0, B +2/+0, C +0/+0, D +2/+0) were FULLY attributed, so the residual is confined to the Phase 1 step and does NOT block Phase 3 or D2. identify the +6 before T4.1 re-pins the baseline; until identified the milestone row records `1156/0/4/86` as MEASURED, not as an explained delta. Do not absorb silently | M23.5 T1 verdict finding 11 |
| M23.5 F13 (documentation, DOC-ONLY -- no code change): two SIGNED T3-D reviewer records name a legacy builder that does not exist. `tasks/m235-t3d-verdict.md:107` and `tasks/m235-t3d-execution-results.md:24` both call it `build_include_elements_legacy_paths`; **no such identifier exists anywhere in the tree**. Correcting source: **`crates/ui/src/dto.rs:608` `pub fn build_include_elements`** (the path-based legacy entry, third arg `cycles: &[Vec<FileId>]`); its group-aware sibling is `build_include_elements_from_groups` at `dto.rs:637` (third arg `cycle_groups: &[CycleGroup]`), and both share the ONE private tail `emit_include_elements` at `dto.rs:659`. The bad name occurs **ONLY under `tasks/` (zero hits under `crates/**` or `tools/**`)**, and both records cite the CORRECT SPAN (`dto.rs:604-622` = the legacy doc comment plus body) with a wrong IDENTIFIER -- a transcription slip, not code drift. First caught at `tasks/m235-t3d2-journal.md` Checkpoint 0. **Do NOT silently rewrite the two records: they are signed adjudication artifacts, and this row plus `tasks/m235-t3d2-verdict.md` section D is the correction of record.** `tasks/m235-t3e-handoff.md` needed **no** name fix -- it names `build_include_elements` correctly; its stale points were the gate counts, the caller census (seven -> NINE) and the line references, all repaired in place 2026-10-04 | nothing outstanding -- no code, test, manifest or lock change is required and no gate is owed. At T4.1 confirm the identifier stays out of NEW documents, and never cite `dto.rs:604-622` under any other name | M23.5 T3-D2 verdict section D + finding F1; T3-D2 journal Checkpoint 0 |
| M23.5 F14 (planner/process; **THIRD** "unverified claim about the tree" defect -- full evidence + three standing rules in the F14 sub-block below) | planner discipline, binding on every handoff in this milestone and the next one; ALSO the correction of record for the stale trailing sentence in `tasks/ledger_archive.md:27-36` | `tasks/m235-t3e-journal.md` BLOCKER section (planner-authored); T3-B verdict finding 1; T3-D verdict finding 1 |
| M23.5 handoff sizing arithmetic (planner / `handoff-spec`, standing): T3-D's handoff predicted `crates/graph/src/graph.rs` at ~679; it landed at **734** over a measured base of 704 (+105/-75 = net +30). The growth is **71 lines of MANDATED ported tests** plus ~7 of mandated doc work against 56 lines of deleted production recursion -- nothing is wrong with the delivered file (266 lines of headroom, zero R1/R2, every function well under 100 lines). **ATTRIBUTION: planner, NOT the implementer** (T3-D Dev-1, ACCEPTED; T3-B Dev-3/D4 are the same defect class). The error is pricing additions without pricing the mandated Red test bodies and their fixtures | `handoff-spec` must require every addition estimate to be itemised FROM the mandated tests, as `tasks/m235-t3d2-handoff.md` sec11 does; never re-derive a count or a suite total from a ledger line | T3-D verdict finding 1 + Dev-1 |

### F14 detail -- the repaired E contract shipped UNSATISFIABLE, caught pre-commit

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