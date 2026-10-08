# T-A journal - ui-extraction-card-clarity (contrast oracle + token value fixes)

Handoff: `tasks/ui-extraction-card-clarity-ta-handoff.md`.
This hop ended at checkpoint 0 (BLOCKED before Red).
No oracle code was authored, no themes.css edit was made, no landing commit exists.

## Duty 1 (orchestrator-mandated planning-artifact tracking) - DONE

- Commit 1: `5c3e0a3` - kickoff/shape/plan tracked (exactly 3 files; 25/168/293 added).
- Commit 2: `6937633` - ledger PLANNED row + ta/tb handoffs tracked
  (exactly 3 files; ledger +16/-4, ta 168, tb 236).
- `git diff tasks/milestone_ledger.md` inspected before commit 2.
  Foreign uncommitted hunks existed; every one is ledger-sync class (no non-ledger edits):
  - `+` M28 `stage5-collateral-memory-ceiling` registration row (2026-10-08 dispatch-state finalization).
  - `+` `ui-hotspot-canvas-direction` PLANNED row (the other live change track's registration).
  - `~` bench-baseline stale deferred row: 2026-10-08 PR-2 state appended.
  - `~` reviewer-throwaway purge row: t0b/t0c/t4b/t4 probe scripts appended.
  - `~` graduation-backlog `bench_compare` cell: M28 DP-3 local-gate-only note appended.
  - `-` M27 profile-lever deferred row removed; discharge text lands in
    `tasks/ledger_archive.md`, which stays foreign-uncommitted (out of my scopes).
  - `+` nine new deferred rows (M28 T2 D1 interim API; M28 cap economics; M28 T2
    journal erratum; M28 I7c counter-test decision; M28 T0b gate-count declaration
    law; M28 T4 snapshot hygiene; bench_compare exit-2 docstring drift;
    handoff-spec gate-budget vs commit-convention collision; clarity shape/kickoff
    erratum adjudicated at authoring).
  The ledger commit is necessarily file-granular (house precedent: playbook T-A
  `f2e1790`/`4a65b6f` carried foreign ledger rows the same way).
  `commit_scoped` used `git commit --only -- <paths>`: each commit carried EXACTLY
  its scoped paths (numstat verified per commit; review-gate set-equality holds).
- Foreign artifacts excluded from both commits (left untouched in the tree):
  dark-grey verdict/execution-results + shape plan, workflow-observer plan,
  stage5 kickoff/shape/handoff/verdict set + plans, hotspot-canvas
  kickoff/shape/t1-handoff + plan, boot-overlay verdict/execution-results,
  engine-room plan edit (M), `tools/bench_baseline.json` (M),
  `tasks/ledger_archive.md` (M), v3-review-*/t*-review-probe.py throwaways.

## Checkpoint 0 - BLOCKED before baseline (hop-0 blocker; zero authored code)

- Start rev: `6937633894227018af7b3b3ac705f2dc9c0d9b95` (`git rev-parse HEAD`
  recorded immediately after Duty 1, before any edit).
- Gate 1 baseline attempt (run BEFORE any edit of mine), command
  `python tools/testrun.py --distill -p ariadne-ui`:
  - verbatim line: `216 passed / 1 failed / 0 ignored / 2 suites` (exit 1;
    capture `target/testrun-capture.txt`, 243 lines).
  - Failing test: `static_html_sections::test_extraction_impact_preview_overlay`.
  - Panic verbatim: `graph.js must stay within the spine-plan line budget (cap 990)`.
- This line is NOT a valid baseline: the shared worktree carries a live foreign
  hand mid-flight (hotspot-canvas-direction T1).
  - At session start (2026-10-08 ~08:05Z) `git status` showed ZERO dirt under
    `crates/**`.
  - By gate-1 time it showed `M crates/ui/assets/graph.js`,
    `M crates/ui/assets/hotspots.js`, `M crates/ui/assets/index.html`,
    `M crates/ui/tests/ui_test/main.rs`,
    `?? crates/ui/tests/ui_test/hotspot_direction.rs`,
    `?? tasks/hotspot-canvas-direction-journal.md`.
  - The compiled binary that ran included the 8 `hotspot_direction::*` tests
    (explains 209 -> 217 against the planner's `c00576d` verification) and the
    embedded `GRAPH_JS` was their WIP file past the cap.
  - At the start rev the committed `graph.js` is exactly 990 lines
    (`git show "HEAD:crates/ui/assets/graph.js"` capture, 990 lines; unchanged
    since `98919a4`), so a CLEAN tree passes that cap test (`<= 990`).
    The failure is purely their uncommitted delta; their own ledger row
    prescribes the pending cap amendment (990 -> <=998) - they are mid-RGB,
    which is legitimate for their hand.
- Why I stopped instead of proceeding:
  - `crates/ui/tests/ui_test/main.rs` is in MY touch-set and simultaneously in
    THEIRS (each hand adds one mod line). `commit_scoped` commits
    working-tree content per path, so staging `main.rs` now would either drag
    their WIP mod line into my landing commit (breaks commit-vs-touch-set and
    standalone compilation of my commit) or have my line dragged into theirs.
    That is the repo-incident E5 pollution class this tool exists to prevent.
  - The shared `ui_test` binary couples every scoped run to their WIP: the Red
    evidence "compiles cleanly and fails on below-floor assertions" could be
    polluted or blocked by their mid-flight compile errors at any moment.
  - The Green acceptance (`exit 0`, delta +7 vs a journal-recorded baseline) is
    unattainable while their WIP is in the tree, and fixing it is out of my
    scope by law (`static_html_sections.rs`, `graph.js` are must-not-touch).
  - The shape section 6 concurrency precondition ("no live hand touches
    crates/ui/assets -> conflict surface is effectively zero") is empirically
    false at execution time; serializing (hotspot T1 lands first, then clarity
    T-A on clean main) is the orchestrator's call, not mine to improvise.
- Disk-state inventory at stop:
  - Authored and committed: this journal only (checkpoint commit).
  - NOT created: `crates/ui/tests/ui_test/extraction_contrast.rs`.
  - Untouched: `main.rs`, `themes.css`, all other assets/tests/src/tools paths.
  - Zero hex literals authored; `tools/theme_token_baseline.json` untouched.
- Deviations (numbered; standing ledger law M28 T0b finding 1):
  - D1: hop ends pre-Red, BLOCKED. Reason: live foreign hand overlaps the
    touch-set (`main.rs`) and the scoped test target; stop-and-report per the
    handoff discipline clause ("out-of-scope breakage: report, don't fix") and
    the checkpoint clause.
  - D2: the baseline line above was taken from a contaminated tree; it is
    journal-recorded verbatim as blocker evidence, annotated as invalid.
    The successor MUST re-baseline on a clean tree.
  - D3: this journal lands as a checkpoint commit, not the handoff's single
    landing commit (the landing commit cannot exist while the tree is
    contaminated). Precedent: `resume-checkpoint` skill; the playbook T-A
    journal records that the single-commit rule reports landing hashes from
    the return report, not a self-hash append.
  - D4: beyond the handoff's ~7-gate budget, ~10 distill-proxied `git`
    read-calls (status/log/show/diff) were used for the foreign-hunk duty and
    blocker attribution; declared here per the same M28 T0b finding-1 law.
    Gate commands proper used so far: testrun x1, commit_scoped x3 (two are
    Duty-1 docs commits mandated by the orchestrator, one is this checkpoint).
- Successor bootstrap (after hotspot-canvas-direction T1 lands on main and
  `crates/**` is clean vs HEAD):
  1. Re-run gate 1 `python tools/testrun.py --distill -p ariadne-ui` at the
     NEW start rev; journal that as the true baseline (expect 217 green if the
     hotspot hop lands its 8 tests; the Green target stays baseline + 7).
  2. The mod-line site instruction is unaffected: insert
     `mod extraction_contrast;` immediately after `mod extraction_card_story;`,
     before `mod file_view;` (their `mod hotspot_direction;` sorts elsewhere).
  3. Oracle expectations for orientation ONLY (hand-computed, NOT evidence;
     the oracle decides): predicted single Red row is
     `light: --green on --bg-card ~4.07 < 4.5` (C6 candidate `#166534`,
     hue-preserving deepen, predicted ~5.8 on `#e2e8f0`).
     Predicted green everywhere else - all dark/grey/steampunk rows included,
     notably the dark `--border-light` rows that shape C6 expected to fail
     (hand-calc ~1.2 vs my recomputation ~1.41 on the 1.35 floor): if the
     oracle comes back green there, NO dark edit - journal the non-materialized
     expectation instead of forcing a change ("expected-but-oracle-decides").
     Predicted empty fix list for grey/steampunk; the achromatic spread <= 4
     and semantic-byte-equality locks stay UNEDITED as tripwires.
  4. Resume route:
     `python tools/checkpoint_guard.py resume --spec tasks/ui-extraction-card-clarity-ta-handoff.md`.
- Hygiene/encoding gates at this checkpoint: `encoding_guard --staged` run
  before each commit (exit 0; staged set was empty pre-commit per the
  commit_scoped internal-add route - the journal itself is pure-ASCII by
  authoring via the Write tool). `hygiene_check` / `encoding_guard --all` are
  deferred to the real landing hop; nothing but this Markdown was authored.
