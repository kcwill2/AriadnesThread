# W-2a Verdict - ui-extraction-wizard: read-only engine-backed provider (cli)

Handoff: `tasks/ui-extraction-wizard-w2a-handoff.md`
Milestone: `ui-extraction-wizard`
Commits under review: `a9d8655` (start rev; W-2b verdict, docs-only) .. `90f4298`
- `cb51bf8` feat(cli): read-only extraction manifest/dry-run provider over the isolation engine
- `90f4298` docs(tasks): W-2a journal rider - gates 7-9 landing records (cb51bf8)

## VERDICT: APPROVED

Zero blocking findings. Three numbered minor findings/observations (F1-F3, all
non-blocking: F1 is an operator-side owed fix the hand correctly escalated,
F2/F3 are notes for downstream hands). All eight declared deviations
(D1-D8) adjudicated ACCEPTED.

## Ground truth at review

`python tools/checkpoint_guard.py status` (session start): HEAD `90f4298`,
branch `main`, dirty 0, untracked none. Worktree census contained no leftover
`w2a-baseline` worktree (the D1 bootstrap worktree was removed, confirming the
journal's post-landing record). Working tree clean for every gate run below
(re-confirmed via the read-only git tool: "nothing to commit, working tree
clean"), so all evidence is from exactly the state under review.

## Per-commit review (oldest first)

### 1. `cb51bf8` - the landing (parent `a9d8655`)

`git show --stat cb51bf8`:

```
commit cb51bf856cb6f27a34beb825714164afc704d1ba
    feat(cli): read-only extraction manifest/dry-run provider over the isolation engine

 crates/cli/src/commands/ui.rs                   | 133 +++-
 crates/cli/src/commands/ui/extraction_reader.rs | 839 ++++++++++++++++++++++++
 tasks/ui-extraction-wizard-w2a-journal.md       | 391 +++++++++++
 3 files changed, 1347 insertions(+), 16 deletions(-)
```

Reviews as one coherent hand:

- `extraction_reader.rs` (NEW, 839 lines) implements
  `ariadne_ui::ExtractionManifestReader` exactly as the W-2b trait froze it
  (`epoch` / `manifest` / `dry_run`, object-safe). Honest-absent Red scaffold
  completed to Green; no `todo!()`/`unimplemented!()`/`panic!()`/`println!()`
  anywhere in the file (grep-verified); the only failure surface is
  documented `expect`s and poison-tolerant mutex handling.
- `ui.rs` (+117/-16) is exactly the wiring the handoff's "What to build"
  names: the `mod extraction_reader;` line; `UiBuildResult` gains
  `extraction_plans` + `install_epoch` (the plan feed the reader resolves ids
  against); `build_ui_payload` routes through the
  `build_extraction_proposals_and_plans` / `conservative_proposals_with_plans`
  twins with ONE planner run (`conservative_proposals_with_plans(planner.plan())`
  - m00 parity structurally preserved); `swap_into_state` constructs
  `EngineBackedExtractionReader` and calls `install_extraction_reader` right
  after `install_analysis` from the just-published generation Arc (zero-copy);
  the watcher carries `next_epoch: AtomicU64` seeded at 2; boot stamps epoch 1
  through the extracted `build_boot_generation`. The frozen single-return
  wrappers survive with `#[allow(dead_code)]` as the `builder_tests` import
  surface (D2 below).
- The frozen sibling test modules needed no edits and got none - verified
  `boot_panic_tests.rs` only names `UiBuildResult` in always-fail closure
  return types (no struct literal), which is why two new struct fields could
  not break compilation of the frozen modules.

### 2. `90f4298` - the docs rider (parent `cb51bf8`)

`git show --stat 90f4298`: journal +24/-0 only. The rider records gates 7-9
(encoding --staged, the scoped landing commit, encoding --all) and the
worktree removal. The docs-only rider-as-commit-cycle-record vehicle is the
declared W-1 `82e5ffe` / W-2b `27cb9de` precedent (both verified docs-only).

## Commit-vs-touch-set check

`git diff --name-only a9d8655 90f4298` returns exactly:

```
crates/cli/src/commands/ui.rs
crates/cli/src/commands/ui/extraction_reader.rs
tasks/ui-extraction-wizard-w2a-journal.md
```

Every path is inside the declared touch-set
(`crates/cli/src/commands/ui.rs`, `crates/cli/src/commands/ui/extraction_reader.rs`,
journal). Per-commit: `cb51bf8` = the three paths; `90f4298` = journal only.

Frozen-lock byte-identity (independent, not taken from the journal):
- `crates/ui/**`: `git diff --numstat a9d8655 90f4298 -- crates/ui tests tools themes.css assets`
  returned only the two cli rows; `-- crates/ui/src/routes/api_extraction.rs
  tools/theme_token_baseline.json themes.css` returned EMPTY.
- `tests/**` (fixtures + goldens incl. `tests/golden/basic/plans_plans.json`):
  diff-empty (the only `tests/` row in the whole range diff is nothing; the
  three-path name list proves it).
- `m00_facts_path_parity`: host file diff-empty; both
  `facts_path_reproduces_production_plans_basic/expensive` ran green UNEDITED
  this review (capture lines 25-26 `... ok`).
- `tools/theme_token_check.py --check-added --base a9d8655`: PASS (exit 0,
  silent) - zero added hex.

## The data bet: engine -> provider -> DTO (independently verified at source)

1. ENGINE (unmodified; `crates/transform/**` diff-empty):
   `IsolationEngine::build_generated_unit` derives `required_deps` from
   `self.model.get_dependencies(&ext.symbol_id)` mapped through the plan's
   `symbol_to_unit` table (`crates/transform/src/engine/mod.rs:116-122`).
2. PROVIDER: `run_engine` runs exactly that engine over
   `self.generation.reduced_model` - the generation the routes epoch-check -
   with a nothing-reading `sources` closure (`&|_source_file| None`), so the
   W3 data path is model-edge-computed and independent of the I4
   placeholder bodies.
3. DTO: `unit_manifest_row` renders `unit.required_deps` lossily into
   `required_includes` with no transformation, filtering, or fabrication.

The runtime proof is non-tautological: test 1 asserts the known fixture edge
materialized (`Thing_extracted.h` carries a non-empty include naming
`ThingConfig_extracted.h` - empty would fail), asserts every rendered include
is one of the plan's destination files (a fabricated path fails), and asserts
row paths equal the payload's target-file rows with uncapped
`symbol_count` sums. Test names verified green in this review's capture:

```
test commands::ui::extraction_reader::tests::test_manifest_lists_engine_sourced_includes_for_the_fixture_plan ... ok
```

## Dry-run parity (D5 single-tree move did not hollow the comparison)

Test 2 still invokes the real CLI path: `build_migration_preview(&boot_repo,
&plan_id, false)` on the same booted tree, reads
`artifacts/migration/preview.json`, and compares field by field
(`consumer_file`, `old_includes`, `new_includes`,
`side_effect_preservations`), asserts `!preview.rewrites.is_empty()` (so the
comparison has teeth), equal rewrite counts, `rewrites_truncated` false, and
`new_files` equal to the apply write set under the plugin staging decision.
Source parity is literal: `discover_plan_consumers` composes the same public
pieces, in the same order and argument shape, as the CLI's private
`find_plan_consumers` (`transform.rs:309-339`): `Repository::locate` ->
`discover()` -> `BuildDiscoverer` with repo discovery + unit shaper ->
`plan_consumer_discovery_targets` -> `plan_target_consumers`. The single shared
root is the stronger equality claim on this host (the `\\?\` canonicalization
would make two-copy byte equality unassertable), and the W2 read-only proof
keeps its own separate boot so test 2's artifact writes cannot contaminate
the tree-hash oracle.

## Resume-scaffold legitimacy

The journal's step-0 inventory (ui.rs modified +94/-8 unstaged, staged set
empty; `extraction_reader.rs` untracked at 506 lines; nothing outside
`crates/cli` dirty) is consistent with the landed shape: the final +117/-16 is
the inherited +94/-8 plus the recovery session's completion, and 839 = 506
inherited scaffold lines plus the Green work. Rule (c) (keep the dead
session's in-touch-set partial work) was the correct call: both dirt files
were inside the touch-set, the content matched the handoff's Red protocol
with no foreign author signature, and the staged index was empty. The landing
commit is coherent as one hand: honest-absent scaffold completed to a real
implementation, no dead-session artifacts remain (clean tree at `90f4298`,
no stubs, no stray files). The deceased session's Duty-1 status was correctly
resolved too: `a9d8655` already IS the W-2b verdict commit, so Duty 1 was
discharged by the reviewer commit and nothing was owed.

## Deviation adjudications (D1-D8)

- **D1 - baseline via isolated detached worktree at `a9d8655`:** ACCEPTED.
  The working tree necessarily carried inherited Red dirt, so a same-tree
  baseline would have measured the wrong bytes. Cleanup verified (no leftover
  worktree in this review's census).
- **D2 - `#[allow(dead_code)]` wrappers (test-cfg gate breaks a string lock):**
  ACCEPTED after independent check. `ui/tests.rs:48-53` cuts ui.rs's
  production section at the FIRST `#[cfg(test)]` literal via a string `find`;
  `builder_tests.rs:1` (frozen, outside the touch-set) imports exactly
  `build_extraction_proposals` and `conservative_proposals`, so the wrappers
  are load-bearing for the frozen suite. Current ui.rs carries no
  `#[cfg(test)]` token before line 963 (comment) / 964 (first real
  attribute), so the production section and the bind-honesty lock are intact;
  the rationale comments were reworded to avoid embedding the literal. The
  lib build is warning-clean with this adaptation.
- **D3 - shared-target mtime false-green fixed by `cargo clean -p ariadne-cli`:**
  ACCEPTED. Standard fingerprint hazard; the honest Red evidence is the
  post-clean run. Noted as a recurring reviewer hazard.
- **D4 - touch-set test predicate suffix-collision fix:** ACCEPTED. The dead
  session's `ends_with("Thing_extracted.h")` also matches
  `Thing_Thing_extracted.h`; the fix (file-NAME equality via
  `rsplit(['\\', '/'])`) is correct and preserves the semantic check. The
  change is inside the hand's own test module.
- **D5 - parity on one shared tree:** ACCEPTED. Verified not hollowed (see
  above). The W2 proof's separate boot is preserved.
- **D6 - ui.rs fee +101 vs the ~30-line call-site target:** ACCEPTED. The
  STOP clause fires only when the honest wiring needs MORE than the 1000-line
  ceiling; it does not (ui.rs lands at 986 lines, verified by full-file read).
  The fee is the plan feed the reader requires (build-result fields, the
  `_and_plans` twins, the boot-generation stamp, the install call). See
  finding F2 for the downstream note.
- **D7 - `includes_unavailable` operationalization:** ACCEPTED. "The engine
  could not resolve deps" is implementable only as "a member symbol is
  absent from the generation's reduced model" - that is exactly when the
  engine has no dependency edges to resolve. A resolved symbol with no
  cross-unit dependency renders an empty, AVAILABLE list, which is honest.
  Documented at `unit_manifest_row`; degraded answers keep the Red-protocol
  honest-absent shape and are never cached. See finding F3 for the W-3b note.
- **D8 - docs-only journal rider as the pre-declared commit-cycle vehicle:**
  ACCEPTED. Precedent-verified (`82e5ffe`, `27cb9de`); `90f4298` rides no code.

## Findings (numbered)

1. **F1 (minor, operator-side; owed fix recorded).** The bench gate is OWED to
   this hand (kickoff lock-table row: W-2a touches `crates/*/src`, so the
   assets-only exemption does not apply). The timing mode deterministically
   fails with the documented parse drift: `missing documented phase:
   generate_report+detect_cycles (combined)`. Root cause verified on disk:
   `tools/bench_compare.py:84` (`DOCUMENTED_PHASES`) still carries the stale
   label, while the producer prints `generate_report+detect_cycle_groups
   (combined)` (`crates/cli/tests/payload_tail_bench.rs:389`; the M23.5 T3.2
   re-pin renamed the row when the public `detect_cycles()` was deleted).
   Exemption recorded per the operator-row protocol: zero-diff proof on the
   producer set vs `a9d8655` (`git diff --numstat a9d8655 90f4298 --
   crates/cli/tests/payload_tail_bench.rs crates/cli/tests/stage5_memory_bench.rs
   tools/bench_compare.py crates/pipeline crates/graph crates/transform
   crates/validate` -> EMPTY), so this hand can neither cause nor cure the
   drift. Compensating evidence obtained: `--bench stage5-memory --runs 6`
   PASS, `accepted 6 of 6 invocations, rejected 0`. Owed fix (operator hop):
   update the `DOCUMENTED_PHASES` tuple label in `tools/bench_compare.py` to
   match the producer, then re-run the timing gate on a verified-idle machine
   (the stale per-machine baseline is the separate, already-escalated PR-2
   row).
2. **F2 (minor, note for downstream hands).** `ui.rs` now sits at 986/1000
   lines. Any future hand needing ui.rs growth should extract a sibling
   module rather than spend the last 14 lines; the handoff's fee discipline
   held, but the margin is gone.
3. **F3 (minor, note for W-3b).** The degraded `honest_absent_dry_run` answers
   `copy_only: false` alongside `move_removals: 0`. That is the conservative,
   non-overclaiming choice (nothing computed, nothing claimed), but W-3b's
   copy must key its "copy-only" sentence off the computed path and treat
   `includes_unavailable` (not `copy_only`) as the degradation signal, or a
   degraded answer would render as "this apply removes symbols".

## Acceptance pass (per acceptance item)

| # | Criterion | Status | Evidence |
|---|---|---|---|
| A1 | Provider implements the W-2b trait over one generation (epoch/manifest/dry_run) | MET | `extraction_reader.rs:264-317`; trait at `crates/ui/src/routes/api_extraction.rs:30-34` unchanged |
| A2 | W3 engine-sourced includes; no fabrication; `includes_unavailable` honest | MET | engine `mod.rs:116-122` diff-empty; `unit_manifest_row:124-151`; test 1 green (non-empty ThingConfig include; membership assertion) |
| A3 | W2 read-only (no write call; tree-hash before/after) | MET | production code reads only (`fs::read_to_string` at :250; writes confined to `#[cfg(test)]` fixture setup); test 4 green |
| A4 | C4 dry-run parity with `build_migration_preview` (same targets, same discovery, same rewriter) | MET | `compute_dry_run:159-192`; `discover_plan_consumers:198-222` mirrors `transform.rs:309-339`; test 2 field-by-field green |
| A5 | R3 cache per (plan id, epoch) behind a mutex; discovery ran once | MET | `dry_run:300-316`; degraded answers never cached; test 3 green (counter == 1) |
| A6 | W4 copy-vs-move computed, not asserted (C++ -> 0 / copy_only) | MET | `move_removal_count:231-261` mirrors validate's `collect_move_removals` (`crates/validate/src/lib.rs:345-370`) read-only; test 5 green |
| A7 | W9 boot payload byte lock (Red-time capture) | MET | test 6 green; capture matches the frozen golden PLAN-001 shape (`tests/golden/basic/plans_plans.json`); one planner run |
| A8 | Epoch/install wiring: reader installed with the generation, boot and watcher | MET | `swap_into_state:606-633` (install right after `install_analysis`); boot epoch 1 (`build_boot_generation:856-860`), watcher from 2 (`:648`); test 7 green |
| A9 | Caps enforced (64 files / 200 symbols / 200 rewrites) with honest truncation flags | MET | `manifest:290-297`, `compute_dry_run:175-181`; test 1 asserts `!files_truncated`, test 2 asserts `!rewrites_truncated` |
| A10 | Gates 1-9 (Red exit 1 with named failures; Green exit 0; ui crate; check --all-targets; hygiene; encoding; scoped commit) | MET | re-run this review: `127 passed / 0 failed / 8 ignored / 7 suites`; `294 passed / 0 failed / 0 ignored / 3 suites`; check exit 0; hygiene exit 0; encoding exit 0 |
| A11 | Touch-set exact; frozen locks byte-identical | MET | name-list = exactly the 3 touch-set paths; `crates/ui`, fixtures, goldens, `theme_token_baseline.json` diff-empty; theme_token_check silent |
| A12 | `m00_facts_path_parity` green UNEDITED | MET | `154 passed / 0 failed / 0 ignored / 38 suites`; both `facts_path_reproduces_production_plans_*` `... ok` |
| A13 | Hygiene (R1/R2 zero; functions < 100; files < 1000) | MET | hygiene exit 0, zero R1/R2 rows; ui.rs 986 lines, extraction_reader.rs 839 lines; zero hygiene rows in the new file |
| A14 | Pure ASCII on newly authored lines | MET | non-ASCII grep on new file + journal: no matches; the only ui.rs non-ASCII lines are pre-existing in the `a9d8655` blob (context, not authored here) |
| A15 | Workspace gates (census, tools discover, ui_smoke, bench) | MET | `1465 passed / 0 failed / 13 ignored / 112 suites`; `Ran 219 tests in 22.313s` `OK`; `ui_smoke: PASS`; bench per F1 (timing: documented drift exemption with zero-diff proof; memory: PASS 6/6) |

## Workspace-gate digests (verbatim, this review)

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 200 lines]
127 passed / 0 failed / 8 ignored / 7 suites
no failures
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 323 lines]
294 passed / 0 failed / 0 ignored / 3 suites
no failures
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 926 lines]
154 passed / 0 failed / 0 ignored / 38 suites
no failures
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 2773 lines]
1465 passed / 0 failed / 13 ignored / 112 suites
no failures
ok (exit 0, 23 warnings emitted) -> capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt
### generic PASS (exit 0)
boot+route checks clean (cold fixture, attempt 1, 1.0s)
ui_smoke: PASS
### generic FAIL (exit 2)
ERROR parse format drift: documented phase row(s) missing from bench output (never substituting zeros):
  missing documented phase: generate_report+detect_cycles (combined)
### generic PASS (exit 0)
stage5-memory sample accounting: accepted 6 of 6 invocations, rejected 0 (> 8% peak-operand divergence from per-slot median)
```

Census arithmetic: the W-2b anchor line is `1458 passed / 0 failed / 13 ignored
/ 112 suites` (`tasks/ui-extraction-wizard-w2b-execution-results.md:58`).
This hand adds exactly the 7 new cli lib tests (individually verified green in
the scoped capture: all 7 `commands::ui::extraction_reader::tests::*` `... ok`;
scoped cli lib suite `121 passed / 0 failed / 0 ignored / 1 suites` = 114 + 7).
1458 + 7 = 1465 passed; failed 0 held; ignored 13 held; suites 112 held. No
delta to explain.

## Tool-allowlist substitutions (no silent skips)

The session's bash allowlist refused several non-allowlisted forms with the
catch-all `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`;
each was retried in an allowed form:

- `python -c ...` (line counts) -> substituted with Read-tool line counts
  (ui.rs 986; extraction_reader.rs 839).
- `git status --porcelain=v1` via bash -> substituted with the read-only git
  tool (clean tree).
- `git grep -nP "[^\x00-\x7F]" <rev> -- <paths>` -> substituted with the Grep
  tool (same regex over the working tree, which matches the reviewed commits).
- `... | Select-Object -Last 4` / `Out-Null` / `> file` pipes -> substituted
  with the documented plain CI forms.
- Not re-run by design (reviewer must not mutate the tree): handoff gate 2
  (the Red run against the honest-absent scaffold) and gate 1's baseline
  worktree. Red evidence was adjudicated from the journal plus the live Green
  state, and the +7 reconciliation is machine-proven by the census and the
  per-test capture. `tools/golden_guard.py` does not exist yet; the frozen
  locks were proven with the prose byte-check (`git diff --numstat` emptiness
  over the lock paths).

## Follow-ups recorded (never dropped)

- Operator row (F1): refresh the `DOCUMENTED_PHASES` label in
  `tools/bench_compare.py`, then re-run the timing bench on an idle machine;
  the stale per-machine baseline (PR-2) remains the separate escalated row.
- W-3b note (F3): key copy-only copy off the computed dry-run path.

## Next hop Duty-1 command (for the W-2a verdict-tracking commit)

```
python tools/commit_scoped.py --scope tasks/ui-extraction-wizard-w2a-verdict.md --scope tasks/ui-extraction-wizard-w2a-execution-results.md -m "docs(tasks): W-2a gate APPROVED (engine-backed read-only manifest/dry-run provider) + execution-results"
```
