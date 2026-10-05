# Verdict - boot-progress-visibility V3 (M26 closing hop): APPROVED

Reviewer gate per `review-gate` skill. Handoff
`tasks/boot-progress-visibility-v3-handoff.md`, journal
`tasks/boot-progress-visibility-v3-journal.md`, start rev `5c81881`,
commits reviewed oldest-first: `1819b8f` (verbatim extraction), `8eedb05`
(events + wiring), `1e81d7e` (journal bookkeeping). HEAD `1e81d7e`, branch
`main`, ahead 0 / behind 0; dirty/untracked set is pre-existing foreign
dirt only (`.kilo/plans/*`, other milestones' verdict/execution files) -
nothing of this hop uncommitted.

Workspace gates (this hop owns them): all fresh, all pass. Canonical lines:

- scoped pipeline: `25 passed / 0 failed / 2 ignored / 5 suites` (exit 0)
- scoped cli: `110 passed / 0 failed / 1 ignored / 5 suites` (exit 0)
- m00 parity: `2 passed / 0 failed / 0 ignored / 1 suites` (UNEDITED)
- m18 parity: `23 passed / 0 failed / 0 ignored / 1 suites` (UNEDITED)
- census: `1235 passed / 0 failed / 4 ignored / 87 suites` (exit 0) =
  V2 anchor `1230/0/4/86` + 3 pipeline + 2 cli; suites +1 (new binary)
- tools: `Ran 188 tests in 24.728s` / `OK`
- smoke: `boot+route checks clean (cold fixture, attempt 1, 0.7s)` +
  `ui_smoke: PASS`
- hygiene_check exit 0 (R3 warn-only, all pre-existing, zero on
  hop-changed lines); `encoding_guard --all` exit 0 (silent)

Full evidence, byte-level proofs, and refused-form log:
`tasks/boot-progress-visibility-execution-results.md` (V3 section).

## Commit-vs-touch-set: PASS

`git diff --name-only 5c81881..HEAD` = exactly the 9 touch-set files.
Commit 1 staged exactly its four gate-2 files; commit 2 its seven gate-7
scopes + the declared stage_index.rs rider (D1); commit 3 journal-only
(journal-declared, V2 precedent `5c81881`). No path outside the touch-set
anywhere. The two-crate compile-coupling exception is REAL:
`stage_progress_callback`'s exhaustive `match` would not compile with
pipeline-side variants and cli-side arms split across hops.

## Plan-literal checks: PASS

Enum block byte-identical with variants APPENDED after the existing three
(`BootMilestone(BootMilestone)`, then
`TuStarted { done, total, source: std::path::PathBuf }`); `BootMilestone`
derives `Debug, Clone, PartialEq, Eq`; `new_with_progress` signature
verbatim; `None` arms route `repo.discover()` unchanged (behavioral no-op);
`DiscoveryHashed { done, total: None }` per hashed file via the V2 hook;
`IncludeScan { done: 1..=4, total: 4 }` after the four named substeps,
cold-rebuild branch only (warm branch early-returns, emits nothing);
per-TU `TuIndex` notify REPLACED by `TuStarted` (R7: one event per TU,
`strip_prefix(&repo_root)` fallback verbatim), terminal
`TuIndex { done: total, total }` intact; messages `Hashing {done} files`,
`{done}/{total} include scan`, `{done}/{total} TUs  {}` (I10 double space,
locked byte-exactly by contract test 5 `"3/9 TUs  src/some TU.cpp"`);
`discovery_hashed_percent` = `map_or(stage_start_percent(1), ...)`,
`include_scan_percent` over `[10, 19]` (`INCLUDE_SCAN_SPAN_END = 19`),
stage-end `(1, 20)` ownership preserved (I9); index.rs or-pattern rider
keeps the same `print_progress` call and both locked literals;
`build_ui_payload` keeps the V1 frame-1-first statement and the
paren-less `Pipeline::new` needle substring through the rename - the V1
test file is untouched and green.

## Verbatim extraction (commit 1): PROVEN, reviewer-owned

Five chunks byte-identical-once between start-rev `lib.rs` and the landing
modules (382-537, 779-798, 822-891 -> `stage_index.rs`; 617-741, 893-952 ->
`stage_load.rs`); the `collect_file_metrics` region carries exactly ONE
delta line - the sanctioned `pub(crate)` raise (old:920 ->
stage_load:178); commit-1 `lib.rs` opcode diff reduces to 430 in-chunk
deletions + import-block re-pruning (22 lines, all imports travelling with
moved items, + boundary blanks) + 30 seam additions (re-pruned imports, two
`mod` declarations, `#[cfg(test)]` companion set mirroring the V1
precedent for `internal_tests`' glob). No moved body or signature line
changed. Line counts verified: `lib.rs` 959 -> 537 -> 634; `stage_index.rs`
276/288; `stage_load.rs` 210 (frozen after commit 1); `stage_spans.rs`
144 <= ~180.

## Frozen locks: all hold (byte-level)

`progress.rs` byte-identical; `progress_contract_tests.rs` append-only
(single insert opcode at line 305, 137/0, existing 305 lines incl. top use
block and `emitted_milestones()` byte-identical); stage-5 13-frame replay +
I2 oracles green UNEDITED; goldens/fixtures/`tools/**` zero-diff;
`crates/{ui,graph,repo,scheduler}` zero-diff; the five V1 frozen test
modules zero-diff and green; `stage_spans.rs` single deletion is exactly
the `#[allow(dead_code)]` whose consumer this hop landed.

## Deviation dispositions

- D1 (commit-2 scope + `stage_index.rs`): ACCEPTED. The file is named in
  the handoff's own touch-set; the R7 emission point the handoff pinned at
  "lib.rs ~508" physically lives there after commit 1's mandated move -
  gate 7's list predated its own commit-1. Editing it in commit 2 is the
  plan-literal landing, not scope creep.
- D2 (test-1 no-op arm `ensure_include_graph` -> `ensure_index`):
  ACCEPTED. The handoff's literal ("the `Pipeline::new` path emits
  nothing") is self-contradictory when run through a cold include rebuild,
  which the handoff's own emission contract SAYS fires four honest
  `IncludeScan` events there - test 2's subject. The switch preserves the
  leak-lock exactly: the assertion is zero `BootMilestone`; `ensure_index`
  provably emits no `BootMilestone` (verified in `stage_index.rs`: the
  index stage emits only `TuStarted`/terminal `TuIndex`), so any
  `BootMilestone` seen would be a buffered/replayed construction milestone
  - the precise G1-5 leak under lock. Cold-include behavior remains fully
  covered by test 2 (cold sequence + warm zero). Division sound.
- D3 (ceilings `lib.rs` 634, `stage_index.rs` 276/288, `ui.rs` 853 vs
  named ~550/~220/~700): ACCEPTED. R2 (< 1000) holds with wide margin
  everywhere and hygiene exits 0. `lib.rs`: commit 1 met ~550 at landing
  (537); the commit-2 boot seam (`build_with_progress`,
  `compute_inputs_with_progress`, the kept include stage) can only land in
  `lib.rs` - a third module is a boundary the touch-set does not name and
  the handoff forbids inventing. `stage_index.rs`: the overage is verbatim
  doc comments the I1 lock FORBIDS rewriting. `ui.rs`: already 832 at V1
  landing; this hop is net +21 of handoff-mandated arms. All three misses
  are consequences of stronger handoff constraints, each reported per the
  handoff's at-cap discipline. M27 T3/T4 headroom remains ample.
- D4 (gate volume ~13 vs ~10): ACCEPTED. The overage is one intermediate
  compile probe (Red-class diagnostic on a never-PASS DRIFT run, I3
  honored), the second `encoding_guard --staged` binding required by the
  two-commit structure, and the Red pair RE-RUN forced by the transport
  loss. All through distilling entry points; zero raw output; reported.
- D5 (staging via per-path `git add --` where gates 2/7 declared
  `commit_scoped.py`): ACCEPTED, recorded. The per-commit stats prove the
  staged sets are byte-exactly the declared scopes (set-equality), which is
  the guard `commit_scoped.py` exists to enforce (it is advisory-only per
  the skill); precedent accepted in the extraction-card T-A D2 row.

## Transport-loss Red evidence: capture-grounded

The Red section is re-run evidence, not recall: journal states the resume
re-ran both gates on the identical tree before any Green edit; every
failure string matches the committed assert-message templates exactly
("got 0 for 2 TUs" against the `{} for {tu_count} TUs` template;
`left: "3/9 TUs"` against test 5's assertion and the committed count-only
arm shape; "bounded hashing ends at the D split point [0, 10]" verbatim at
contract test line 328); cli Red arithmetic reconciles to green (108+2 =
110). Open limitation, stated: the pipeline fail-mode digest counts
(12/3/0/2) cannot be recomputed post-Green; the journal itself flags
distill's green/fail digest-shape difference. Acceptable under the
lost-evidence-must-be-re-established precedent.

## G1-5 bench exemption statement (required in-verdict)

The written behavioral-no-op exemption HOLDS, re-verified at this hop:
`Pipeline::new` delegates `build -> build_with_progress(.., None) ->
compute_inputs_with_progress(.., None)` whose `None` arm calls
`repo.discover()` exactly as pre-V3 (green lock: test 1's no-callback arm);
the entire stage-5 producer module `stage_load.rs` is byte-identical from
start-rev code to HEAD (one sanctioned visibility line; zero commits-2/3
change); planner/justification code untouched; remaining boot-path changes
are callback-gated or event-count-preserving. `bench_compare` NOT RUN per
the M26 ledger row ("NO bench gate ... PR-2 bench refresh belongs to M27
only"); a run would inherit the dated stale-baseline flap (escalated row)
and belongs to M27 T1b on a VERIFIED-IDLE machine.

## Findings (numbered; none blocking)

1. (minor, tooling-config) Chained bash forms (`git log; echo ...; git
   diff`, `git show A; echo; git show B`) hit the boilerplate
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`
   while the same commands run fine individually; `git ls-files`,
   `git status --porcelain`, `Remove-Item`, and agent-grant-external `Write`
   are likewise denied. Every gate cleared via permitted forms; append the
   recurrence to ledger row 60 at closeout.
2. (minor, hygiene-of-review) Reviewer scratch scripts could not be
   deleted (`Remove-Item` denied): `tasks/v3-review-verbatim.py`,
   `tasks/v3-review-debug2.py`, `tasks/v3-review-debug3.py`,
   `tasks/v3-review-debug4.py`, `tasks/v3-review-final.py` - untracked,
   scan-visible to hygiene R3; add to the operator purge queue (M25
   "reviewer throwaways" precedent).
3. (minor, evidence-completeness) Pipeline Red fail-mode digest is not
   recomputable post-Green; accepted as disclosed residual with the
   name-level grounding above.
4. (note) The dispatch brief's closeout list is stale on one point: the
   V3 journal is TRACKED (landed in `1819b8f`/`8eedb05`/`1e81d7e`); the
   actually untracked boot-progress artifacts are
   `boot-progress-visibility-execution-results.md` (until this hop's append -
   still untracked), `boot-progress-visibility-v1-verdict.md`,
   `boot-progress-visibility-v2-verdict.md`, plus this hop's verdict +
   execution-results.

## M26-closeout readiness

After merging this hop (already on `main`), milestone closeout remains, per
`milestone-lifecycle` and the M26 ledger row:

1. Ledger flip: row 32 PLANNED -> CLOSED, quoting this hop's gates verbatim,
   D1-D5 ACCEPTED, census `1235/0/4/87`; kilo-memory mirror row saved in the
   same commit.
2. Artifact commits (currently untracked):
   `tasks/boot-progress-visibility-v1-verdict.md`,
   `tasks/boot-progress-visibility-v2-verdict.md`,
   `tasks/boot-progress-visibility-execution-results.md` (now V1+V2+V3),
   `tasks/boot-progress-visibility-v3-verdict.md` (this file); kickoff and
   shape plan docs are already tracked.
3. Kickoff-doc residuals from the ledger row: `checkpoint_guard.py status`
   census statement (this hop: 8 worktrees, none M26-owned - the hop worked
   on `main`; `tarjan-s-scc` carries 2 foreign dirty lines unrelated),
   graduation-backlog audit.
4. Bench: record exemption + premise re-proof (above); PR-2 refresh stays
   with M27 T1b / operator on VERIFIED-IDLE.
5. Errata recurrence: `--baseline` FILE-form unrunnable, 4th documented
   encounter (census substitute `--distill` used again).
6. Release the V3-BEFORE-T3 ordering: the M27 row's T3 block "M26 V3
   landed" half is now satisfied (the written T1-sufficiency-FAIL half is
   M27's own).
7. Worktree census clean statement at closeout; walkthrough per the
   milestone-lifecycle closeout checklist.
