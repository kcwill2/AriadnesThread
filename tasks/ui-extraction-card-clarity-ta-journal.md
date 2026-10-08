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

## Resume hop 1 (2026-10-08) - landing hop, isolated worktree

- Worktree: `.kilo/worktrees/ui-extraction-card-clarity-ta`, branch
  `ui-extraction-card-clarity-ta` (isolated to dodge the hop-0 blocker: the
  hotspot hand that contaminated shared main has since landed at `55a85d9`).
- Boot per `checkpoint_guard status`: branch HEAD `4970af3`, tree clean,
  0 ahead / 88 behind main. The boot brief declared the branch based on main
  at `55a85d9`, but on disk the branch sat at `4970af3` where neither this
  journal nor the handoff is tracked. The branch owned ZERO commits.
  Bootstrap per resume-checkpoint discipline (on-disk state, not remembered
  intent): advanced the empty branch to its declared base with
  `git merge --ff-only 55a85d9` (distill-proxied; fast-forward, lossless, no
  merge commit, no work of this hand involved) - deviation D1.
- Start rev (used for every `<start-rev>` proof below): `55a85d9`.

### Gate 1 - fresh baseline at the clean start rev (re-baseline per hop-0 D2)

- Command: `python tools/testrun.py --distill -p ariadne-ui` - exit 0.
- Verbatim: `217 passed / 0 failed / 0 ignored / 3 suites`
  (capture `target/testrun-capture.txt`, 333 lines).
- Matches the hop-0 successor expectation (217 = the hotspot 8 tests landed).
  Green acceptance target: 217 + 7 = 224.

### Red - oracle + mod line only, ZERO themes.css edit

- Authored `crates/ui/tests/ui_test/extraction_contrast.rs`: the seven
  required tests; the C4 pair table as nine const rows looping the PARSED
  theme.js registry (I13); composite convention documented in the header doc
  comment (translucent composites over `--bg-surface` per channel
  `a*c + (1-a)*under`, opaque passes through - every table token is opaque
  today, so the composite arm never rescues a current declaration).
- `main.rs`: inserted `mod extraction_contrast;` immediately AFTER
  `mod extraction_card_story;`, BEFORE `mod file_view;` (the handoff anchor;
  the `export_order` mod-order erratum region untouched - the handoff anchor
  supersedes the hop-0 note's alphabetical phrasing).
- Same command, exit 1. Verbatim digest:

  ```
  ### cargo test FAIL (exit 1) [capture: ...\target\testrun-capture.txt, 252 lines]
  223 passed / 1 failed / 0 ignored / 2 suites
  ---- extraction_contrast::test_verdict_hues_on_card ----
  thread 'extraction_contrast::test_verdict_hues_on_card' (25384) panicked at crates\ui\tests\ui_test\extraction_contrast.rs:301:5:
  [verdict] contrast floors violated:
  light: --green vs --bg-card measured 4.07:1, below the 4.50 floor
  223 passed / 1 failed / 0 ignored / 2 suites
  ```

- The single Red pair row is exactly the hop-0 predicted row (light `--green`
  on `--bg-card` 4.07 < 4.5). Compiles cleanly; the other six oracle tests
  pass (223 = 217 + 6). Per kickoff G1 this measured table supersedes the
  shape's hand-computed C5 table.

### Green - the one value edit the oracle demands

- themes.css light block: `--green: #15803d;` -> `--green: #166534;`.
  Justifying red pair row: `light: --green vs --bg-card measured 4.07:1,
  below the 4.50 floor`. After-fix measured 5.78 (+28% margin). C6 candidate
  value, hue-preserving deepen, contrast-only per D6; value-only edit,
  literal count unchanged. No other token touched anywhere.
- Same command, exit 0. Verbatim: `224 passed / 0 failed / 0 ignored / 3
  suites` (capture 244 lines) = baseline + 7 exactly.

### Per-theme oracle outcomes (measured; full 52-pair table cited at
`target/distill-capture.txt` from the ratio-run capture)

| pair | floor | light | dark | dark-grey | dark-steampunk |
|---|---|---|---|---|---|
| --text-main / --bg-card | 7.0 | 14.48 | 13.98 | 12.46 | 11.61 |
| --text-main / --bg-sub | 7.0 | 14.48 | 15.81 | 13.88 | 13.11 |
| --text-muted / --bg-card | 4.5 | 6.15 | 5.71 | 5.38 | 6.43 |
| --text-muted / --bg-sub | 4.5 | 6.15 | 6.45 | 6.00 | 7.26 |
| --text-muted / --bg-surface | 4.5 | 7.58 | 6.92 | 6.76 | 7.90 |
| --text-muted / --bg-main | 4.5 | 6.92 | 7.48 | 7.30 | 8.90 |
| --text-sub / --bg-main | 3.0 | 5.81 | 4.03 | 5.34 | 5.16 |
| --amber / --bg-card | 4.5 | 5.75 | 6.46 | 6.00 | 6.17 |
| --green / --bg-card | 4.5 | 4.07 -> 5.78 FIXED | 8.40 | 7.79 | 8.02 |
| --red-text / --bg-card | 4.5 | 5.25 | 5.29 | 4.91 | 5.05 |
| --bg-card / --bg-surface | 1.12 | 1.23 | 1.21 | 1.25 | 1.23 |
| --border-light / --bg-card | 1.35 | 2.08 | 1.41 | 1.74 | 1.96 |
| --border-light / --bg-surface | 1.50 | 2.56 | 1.71 | 2.18 | 2.40 |

- light: FIXED (one value). dark / dark-grey / dark-steampunk: fix-empty.
- All 52 pair measurements green after the single fix.

### Oracle-refused expectation (non-materialized; NO edit forced)

- Shape C6 predicted the dark `--border-light` row FAIL (~1.2 vs the 1.35
  floor) and prescribed "lighten" as the expected dark fix. The oracle
  measured 1.41 >= 1.35: GREEN. The predicted dark edit is therefore
  ORACLE-REFUSED; per hop-0 successor instruction 3 this non-materialization
  is journalled and NO dark value was touched. The companion row
  `--border-light` on `--bg-surface` measured 1.71 >= 1.50 (also green). The
  shape's ~1.11 mass falsifier likewise did not materialize on declared
  values (dark `--bg-card`/`--bg-surface` = 1.21 >= 1.12).
- dark-grey and dark-steampunk fix lists empty as expected (tightest rows:
  grey `--red-text` 4.91, steampunk `--border-light` 1.96 - all above
  floors). The grey achromatic spread<=4 and semantic-byte-equality locks
  stayed UNEDITED and green in the scoped run; tripwires intact.

### Refactor

- Evaluated NO-OP: the handoff's refactor deliverables (honest helper names,
  composite-convention doc comment) were authored in the Red write itself;
  no duplication left to extract (effective_color vs opaque_color carry
  distinct alpha contracts). Zero content edits after the Green run, so the
  Green digest stands as post-refactor verification; the Green re-run
  allowance was spared (interpretation declared as deviation D3).

### Frozen-lock proofs (this hop)

- `python tools/theme_token_check.py --check-added --base 55a85d9`: SILENT.
- `git diff 55a85d9 --name-only` (pre-commit): exactly
  `crates/ui/assets/themes.css`, `crates/ui/tests/ui_test/main.rs` (plus the
  new oracle + this journal entering via the landing commit); therefore
  `tools/theme_token_baseline.json` diff-empty and `crates/*/src` diff-empty
  (bench-exemption proof) hold by construction.
- `python tools/hygiene_check.py`: exit 0 (distill-confirmed
  `### generic PASS (exit 0)`); every finding is a PRE-EXISTING R3 warn in
  untouched files; zero findings in any path this hand authored.

### Deviations (numbered; M28 T0b finding-1 dated-D-row law)

1. D1 boot reconciliation: the provisioned branch sat at `4970af3`, not the
   declared `55a85d9`, with the handoff/journal untracked there. Fast-forward
   `4970af3 -> 55a85d9` (`git merge --ff-only`, distill-proxied) before any
   edit. The branch had zero owned commits and a clean tree, so the ff is
   lossless; this establishes the base the handoff assumes and is NOT the
   prohibited mid-hand merge of main (no work commits of this hand were
   involved). Start rev = `55a85d9` post-ff.
2. D2 gate budget: 9 distill-entry-point gate invocations vs the handoff's
   ~7 (+1 Green re-run allowance, unspent): testrun x3 (baseline/Red/Green),
   theme_token_check x1, hygiene x1, encoding_guard --staged x1,
   commit_scoped x1, encoding_guard --all x1 (post-commit --all is mandated
   by the boot contract beyond the handoff's five-gate list), plus one
   hygiene re-run through distill solely for exit-code confirmation (direct
   PowerShell pipe confirmation was allowlist-refused). Also ~8 distill-
   proxied read-only invocations for boot reconciliation + proofs
   (rev-parse/cat-file/log/branch/ls-tree/merge --ff-only/diff --name-only)
   and one scratch WCAG ratio script run from the temp dir THROUGH
   `tools/distill.py` (direct python invocation was allowlist-refused; the
   catch-all deny rule reports boilerplate). Precedent: hop-0 D4.
3. D3 refactor-step no-op with the Green re-run allowance unspent (reason in
   the Refactor section above).
4. D4 the single landing commit carries oracle + mod line + themes.css value
   fix + this journal (handoff gate 5); its hash is NOT self-appended here -
   it returns via the report contract (hop-0 D3 precedent: landing hashes
   are reported from the return report, not self-hashed into the journal).
5. Landing-enclosure results (`encoding_guard --staged` pre-commit,
   `commit_scoped` hash + `git show --stat`, `encoding_guard --all`
   post-commit) necessarily postdate this journal's final byte; they are
   carried verbatim in the return report instead.

### Gate ledger (commands as actually run, in order)

1. `python tools/testrun.py --distill -p ariadne-ui` - baseline exit 0
   (217 passed), Red exit 1 (223/1), Green exit 0 (224 passed).
2. `python tools/theme_token_check.py --check-added --base 55a85d9` - silent.
3. `python tools/hygiene_check.py` - exit 0 (R3 warns only, all pre-existing).
4. `python tools/distill.py -- git diff --name-only 55a85d9` - two paths
   (touch-set conformance proof).
5. `python tools/distill.py -- git add -- <the four touch-set paths>` then
   `python tools/encoding_guard.py --staged` - pre-commit, exit reported in
   the return report.
6. `python tools/commit_scoped.py --scope crates/ui/tests/ui_test/extraction_contrast.rs --scope crates/ui/tests/ui_test/main.rs --scope crates/ui/assets/themes.css --scope tasks/ui-extraction-card-clarity-ta-journal.md -m "test(ui): four-theme contrast oracle for extraction token pairs + oracle-driven value fixes"` - single landing commit.
7. `python tools/encoding_guard.py --all` - post-commit, exit reported in the
   return report.
