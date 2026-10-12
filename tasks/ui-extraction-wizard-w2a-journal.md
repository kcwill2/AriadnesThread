# W-2a Journal - read-only engine-backed extraction provider (crates/cli)

Handoff: `tasks/ui-extraction-wizard-w2a-handoff.md`. Milestone
`ui-extraction-wizard`. This journal is authored by the RESUME session
(implementer); the first W-2a implementer session died with an empty
result and left unknown on-disk state. Per the resume-checkpoint class,
everything below starts from on-disk ground truth, never from remembered
intent.

## 0. Step-0 state report (ground truth at boot, before any mutation)

`python tools/checkpoint_guard.py status` verbatim:

```
RULE: Sessions whose compaction summary was empty are NEVER resumed. This report is the fresh-session bootstrap.
== HEAD ==
sha: a9d8655
branch: main
== DIRTY FILES (git diff HEAD --numstat: worktree + index vs HEAD) ==
[crates/cli]
  94	8	crates/cli/src/commands/ui.rs
== UNTRACKED FILES ==
crates/cli/src/commands/ui/extraction_reader.rs
== RECENT COMMITS (last 5) ==
a9d8655 docs(tasks): W-2b gate APPROVED (reader trait + wizard DTOs + two GET routes, 503/404/409) + execution-results
27cb9de docs(tasks): W-2b journal rider - gates 7-8 landing records (f5a7d80)
f5a7d80 feat(ui): extraction manifest/dry-run routes behind a read-only reader trait (503/404/409)
7b2238c docs(tasks): W-1 gate APPROVED (dto.rs split, 953 lines) + execution-results
82e5ffe docs(tasks): W-1 journal rider - gates 7-8 landing records (b27aa46)
== WORKTREE CENSUS (git worktree list + .kilo/worktrees union) ==
* C:/Users/Kurt/Code/AriadnesThread branch=main ahead=0 behind=0 dirty=2
* C:/Users/Kurt/AppData/Local/Temp/kilo/mergetest branch=DETACHED ahead=0 behind=84 dirty=272
* C:/Users/Kurt/Code/AriadnesThread/.kilo/worktrees/purrfect-cod branch=create-readme ahead=0 behind=82 dirty=0
* C:/Users/Kurt/Code/AriadnesThread/.kilo/worktrees/shining-enthusiasm branch=shining-enthusiasm ahead=1 behind=55 dirty=1
* C:/Users/Kurt/Code/AriadnesThread/.kilo/worktrees/tarjan-s-scc branch=tarjan-s-scc ahead=0 behind=193 dirty=2
```

W-2b artifact tracking: `git ls-files` answers BOTH files tracked:

```
tasks/ui-extraction-wizard-w2b-execution-results.md
tasks/ui-extraction-wizard-w2b-verdict.md
```

HEAD hash: `a9d8655` - which IS the W-2b verdict commit (its subject line
is the Duty-1 message), so Duty 1 was already discharged by the reviewer
commit; nothing is owed.

crates dirt inventory: exactly two entries, both under `crates/cli`:

- `crates/cli/src/commands/ui.rs` modified, unstaged (+94/-8); staged set
  EMPTY (`git diff --cached --stat` = 0 lines).
- `crates/cli/src/commands/ui/extraction_reader.rs` untracked (506 lines).
- Everything else diff-EMPTY: `git diff -- crates/ui crates/ui/src` =
  0-line capture; `git status --porcelain=v1` lists only the two entries.

`ui.rs` diff content (read in full from
`target/distill-capture.txt`, 207 lines): `mod extraction_reader;`
declaration; `UiBuildResult` gains `extraction_plans` + `install_epoch`
(the plan feed the handoff demands: planner output travels with the
build result instead of being dropped); `build_ui_payload` rewired
through new `build_extraction_proposals_and_plans` /
`conservative_proposals_with_plans` wrappers that keep the old single
return via `.0`; `swap_into_state` constructs
`EngineBackedExtractionReader` and calls `install_extraction_reader`
right after `install_analysis`; watcher gains an `AtomicU64` starting at
2 and stamps `out.install_epoch` per successful swap; the boot closure
sets `install_epoch = 1`. All of it is inside the handoff touch-set and
is exactly the wiring the handoff's "What to build" describes.

`extraction_reader.rs` content: the handoff's Red protocol literally
implemented - an honest-absent `EngineBackedExtractionReader` (manifest
returns `Some(dto)` with empty `files`, dry_run returns `Some(dto)` with
empty `rewrites`/`new_files`, `includes_unavailable: true`, no
`todo!()`/`unimplemented!()`) plus the seven required tests, including
the W9 byte oracle captured verbatim from live code (the
`CAPTURED_PROPOSALS_JSON` constant with Windows-lossy backslash
separators, matching the `basic` fixture's PLAN-001).

### Rule applied: (c)

The `crates/**` dirt IS the dead session's partial W-2a work: both files
are inside the handoff touch-set, their content matches the handoff's Red
protocol and wiring spec with no other author's signature, the staged
index is empty, and nothing outside `crates/cli` moved. No foreign hand
(rule b) and no untracked W-2b artifacts (rule a) apply; the tree is not
clean (rule d). Per rule (c): KEEP the inherited state and continue from
it.

### Resume note (inherited state, per the handoff's checkpoint law)

Inherited: complete ui.rs wiring + Red-scaffold provider + the seven
required tests, zero gates recorded (no baseline, no Red run evidence,
no journal). The resume session does not know whether the dead session
ever compiled this state; the first action is a compile probe, then the
gate sequence runs from Gate 1 unchanged.

## 1. Compile probe of the inherited state

`python tools/distill.py -- cargo check -p ariadne-cli --all-targets`
before any edit: exit 0 (digest line `ok (exit 0, 0 warnings emitted)`;
the distiller's warning tally under-reads - the capture
`target/distill-capture.txt` records 7 lib-target + 6 lib-test warnings,
all scaffold dead-code/unused-import warnings that Green consumes, plus
the two wrapper dead-code rows analyzed in section 3). The Red scaffold
compiles; the state is coherent.

## 2. Gate 1 - baseline canonical line

Baseline was owed at the START rev `a9d8655` (the worktree carries the
dead session's Red dirt, so the baseline cannot be taken here honestly).
Taken in an isolated detached worktree at exactly the start rev
(`git worktree add ... a9d8655 --detach`, temp path
`C:\Users\Kurt\AppData\Local\Temp\kilo\w2a-baseline`, shared
`CARGO_TARGET_DIR` to reuse unchanged dependency artifacts):

`python tools/testrun.py --distill -p ariadne-cli` (workdir = baseline
worktree):

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\AppData\Local\Temp\kilo\w2a-baseline\target\testrun-capture.txt, 211 lines]
120 passed / 0 failed / 8 ignored / 7 suites
```

Gate 1 baseline canonical line: `120 passed / 0 failed / 8 ignored / 7 suites`.

## 3. Gate 2 - Red

First Red attempt from the main worktree printed `120 passed / 0 failed /
8 ignored / 7 suites` exit 0 - a false artifact: the baseline build in
the shared target dir had newer mtimes than the inherited files, so
cargo judged the HEAD-derived cli test binaries fresh and skipped
rebuilding. Caught by comparing digests; `python tools/distill.py --
cargo clean -p ariadne-cli` cleared the poisoned fingerprints (deviation
D3). Re-run, verbatim digest:

```
### cargo test FAIL (exit 1) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 307 lines]
117 passed / 4 failed / 0 ignored / 1 suites
```

`cargo test` halts at the first failing test binary, so only the
lib-test suite reports; 121 executed = the lib suite plus the 7 scaffold
tests. Named failing tests with their verbatim panic lines:

1. `commands::ui::extraction_reader::tests::test_manifest_lists_engine_sourced_includes_for_the_fixture_plan`
   - `assertion \`left == right\` failed: the manifest rows must equal the plan's destination-file rows` `left: []`
2. `commands::ui::extraction_reader::tests::test_dry_run_rewrites_match_the_cli_migration_preview`
   - `assertion \`left == right\` failed: the dry-run must answer with exactly the CLI preview's rewrites` `left: 0`
3. `commands::ui::extraction_reader::tests::test_dry_run_caches_consumer_discovery_per_plan_epoch`
   - `assertion \`left == right\` failed: the expensive consumer-discovery half must run exactly once` `left: 0`
4. `commands::ui::extraction_reader::tests::test_dry_run_reports_copy_only_honesty_on_the_cpp_fixture`
   - `a C++ dry-run must report copy-only semantics`

All four fail for the predicted semantic reason (W3 rows missing, C4
rewrites empty, R3 counter never moves, W4 flag false). The three
pre-mutation oracles (W2 read-only, W9 byte lock, epoch/install) pass at
Red, as the protocol requires.

## 4. Wrapper dead-code disposition (pre-Green decision)

`build_extraction_proposals` and `conservative_proposals` are dead in
the lib build but are the import surface of
`crates/cli/src/commands/ui/builder_tests.rs`, which is OUTSIDE this
hand's touch-set (grep of `tests/**` finds only a doc-comment mention,
no call). To keep the lib build warning-clean without editing
`builder_tests.rs`, the wrappers first got `#[cfg(test)]` - which broke
the string lock
`commands::ui::tests::test_run_ui_binds_resolved_addr_never_hardcoded_wildcard`:
it cuts ui.rs's production section at the FIRST `#[cfg(test)]` literal in
the file (a string `find`, so even the literal inside a doc comment
counts). `ui/tests.rs` is outside the touch-set; the contract stands, the
wrappers adapt to `#[allow(dead_code)]` with the reason documented in
their doc comments. Deviation D2.

## 5. Gate 3 - Green

Provider Green composition (all inside `extraction_reader.rs`):
`run_engine` (IsolationEngine over the generation's reduced model,
`sources` reads nothing - bodies never cross this boundary and
`required_deps` is model-edge-computed, so the I4 placeholder bodies are
the honest cost), `unit_manifest_row` (symbol-id ordered members, capped
names, uncapped count, lossy `required_deps`, `includes_unavailable` =
any member symbol absent from the reduced model),
`discover_plan_consumers` (the same public pieces the CLI's private
`find_plan_consumers` composes), `compute_dry_run` (C4:
`plan_rewrite_targets` + `IncludeRewriter::build_preview` verbatim, W4
count computed by offering `remove_top_level_symbols` to the plan
rewriter read-only, `staged_write_paths` follows the plugin staging
decision), the (plan id, epoch) cache in `dry_run`, degraded answers
never cached.

Two test corrections inside the touch-set test module (deviations D4/D5,
section 7). Iteration to green took three runs: the first two exposed
the two dead-session test flaws plus my own doc comment carrying the cfg
literal (see the D2 note above).

Final Gate 3 digest, verbatim:

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 201 lines]
127 passed / 0 failed / 8 ignored / 7 suites
```

Canonical line `127 passed / 0 failed / 8 ignored / 7 suites` = baseline
`120/0/8/7` plus exactly the seven scaffold tests. (`m00_facts_path_parity`
does NOT run under `-p ariadne-cli` - it lives in the separate
`ariadne-integration-tests` package; it ran green UNEDITED in the
reverse-dependency pass of section 8.)

## 6. Gate 4 - consumer crate

`python tools/testrun.py --distill -p ariadne-ui` verbatim:

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 324 lines]
294 passed / 0 failed / 0 ignored / 3 suites
```

Exactly the W-2b landing line (`294/0/0/3`): the install call sites
change nothing for the ui crate.

## 7. Refactor phase (hygiene-forced) and Gates 5/6

The inherited state + my first refactor pass surfaced four blocking
hygiene rows, all in `ui.rs`: R1 `build_ui_payload` 102 lines, R1
`run_ui` 101 lines, then my own 25-line planner helper pushed the file
to R2 `1016 > 1000`. Final composition (fee discipline - no net growth
from the refactor itself):

- `build_boot_generation` (11 lines): the boot epoch-stamp closure
  extracted from `run_ui`, which returns to a 3-line closure (98 lines).
  This helper also serves as the named home for the boot epoch contract.
- the planner call inside `build_ui_payload` compacts to a 5-line
  multi-argument call (function 97 lines) instead of a separate helper -
  the first extraction's helper cost more lines than it saved and broke
  R2; the file lands at **986 lines** (885 at HEAD, net +101).
- `extraction_reader.rs` lands at 839 lines (< 1000); every function in
  both files is under 100 lines (R1 rows zero).
- Deviation D6: the `ui.rs` fee is +101 lines against the handoff's
  "about 30 lines of call sites" target - the plan feed (`UiBuildResult`
  fields + docs, the `*_and_plans` twins, the boot-generation stamp, the
  install call, the two wrapper `allow` attributes) is the cost. The
  STOP clause fires only when the honest wiring needs MORE than the
  1000-line ceiling; it does not (986), so the work continued.

Gate 6 `python tools/distill.py -- python tools/hygiene_check.py`:

```
### generic PASS (exit 0)
```

179 capture lines, all `R3 (warn)` pre-existing rows (grep of the
capture for `R1|R2`: zero matches). Gate 5
`python tools/distill.py -- cargo check -p ariadne-cli --all-targets`:
`ok (exit 0, 0 warnings emitted)`; the capture shows the ONLY warnings
as the two pre-existing ones (`crates/ui/src/routes/api_graph.rs:719`
unused `gen` from the W-2b era, `crates/cli/src/lib.rs:804` unused
`mut`), zero in the touched files.

## 8. Gate 5 expansion - workspace and reverse-dependency compile proofs

The hand touches `crates/cli`, so the workspace must compile at landing.

- `python tools/distill.py -- cargo check --workspace --all-targets`:
  `ok (exit 0, 11 warnings emitted)` (final post-refactor run; capture
  enumerates 42 warning sites, ALL in files this hand does not touch:
  `lang-cpp/resolver.rs`, `ui/api_graph.rs`, `cli/lib.rs:804`,
  integration-test files).
- The one real reverse-dependency package on `ariadne-cli` is
  `ariadne-integration-tests` (its Cargo.toml depends on `ariadne-cli`;
  it hosts `m00_facts_path_parity`). Scoped run
  `python tools/testrun.py --distill -p ariadne-integration-tests`:

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 929 lines]
154 passed / 0 failed / 0 ignored / 38 suites
```

  including `facts_path_reproduces_production_plans_basic ... ok` and
  `facts_path_reproduces_production_plans_expensive ... ok` - the m00
  lock green UNEDITED (the file is diff-empty; it never entered the
  porcelain dirt list at any point).

## 9. Frozen-lock evidence

- `crates/ui/**` diff-empty: porcelain lists only the two `crates/cli`
  files plus this journal, at every status check.
- `tests/fixtures/**`, `tests/golden/**`: never in porcelain; the W2
  test reads fixtures and writes nothing (test 4 green: before/after
  FNV tree hash identical across manifest + dry-run);
  `UPDATE_GOLDENS=1` never invoked.
- `m00_facts_path_parity`: UNEDITED and green (section 8).
- `tools/theme_token_baseline.json`: diff-empty; zero asset touched;
  zero hex literal authored.
- W9 byte lock: test 6 green - serialized `extraction_proposals` bytes
  for the boot of `basic` equal the Red-time capture verbatim (the boot
  payload grew by nothing; the planner run count is unchanged:
  `build_extraction_proposals_and_plans` wraps the SAME single
  `conservative_proposals_with_plans(planner.plan())` call).

## 10. The engine -> provider -> DTO data-flow proof (the central deliverable)

The data the CLI preview boundary dropped (capability row 4: the
preview path passes `required_deps: vec![]` at
`crates/cli/src/commands/transform.rs:216-235`), now reaches the wizard:

1. ENGINE: `IsolationEngine::build_generated_unit`
   (`crates/transform/src/engine/mod.rs:83-131`) derives each unit's
   `required_deps` from `model.get_dependencies(&ext.symbol_id)` mapped
   through the plan's symbol->unit table (lines 116-122). The engine was
   not modified; the provider calls it.
2. PROVIDER: `EngineBackedExtractionReader::run_engine`
   (`crates/cli/src/commands/ui/extraction_reader.rs`) runs exactly that
   engine over `self.generation.reduced_model` - the reduced model of
   the generation whose epoch the routes check. `sources` reads nothing
   (bodies never cross this boundary; `required_deps` is model-edge
   computed, so the I4 placeholder bodies do not touch W3).
3. DTO: `unit_manifest_row` renders `unit.required_deps` lossy into
   `ExtractionUnitManifestDto.required_includes` and sets
   `includes_unavailable` only from the engine's own resolution failure
   (a member symbol absent from the reduced model).

Runtime proof (test 1, green): the boot-installed reader answers
PLAN-001 with 4 destination rows equal to the payload's target-file
rows; `Thing_extracted.h` reports required_includes NON-EMPTY naming
`ThingConfig_extracted.h` (the known inter-unit dependency),
`includes_unavailable: false`; and every rendered include is proven
non-fabricated by the test's own assertion (any include not among the
plan's destination files fails the run). The dry-run's C4 half runs the
CLI's own pieces (`plan_rewrite_targets`, `plan_consumer_discovery_targets`
+ `plan_target_consumers`, `IncludeRewriter::build_preview`) and test 2
compares the DTO to `artifacts/migration/preview.json` from
`build_migration_preview` itself, field by field, on one tree.

## 11. Deviations (numbered, complete)

1. **D1 - resume bootstrap**: Gate 1 baseline taken in an isolated
   detached worktree at start rev `a9d8655` (temp path
   `...\kilo\w2a-baseline`) sharing `CARGO_TARGET_DIR` with the main
   tree, because the working tree necessarily carries the inherited Red
   dirt. Post-gate, the worktree was removed.
2. **D2 - wrapper disposition**: `build_extraction_proposals` /
   `conservative_proposals` get `#[allow(dead_code)]` + reason docs.
   The first attempt (`#[cfg(test)]`) broke the string lock in
   `commands::ui::tests::test_run_ui_binds_resolved_addr_never_hardcoded_wildcard`
   (it cuts at the FIRST `#[cfg(test)]` literal - even inside a comment);
   `ui/tests.rs` is outside the touch-set, so the wrappers adapted. The
   rationale doc comments initially carrying the literal were rewritten
   to avoid it.
3. **D3 - poisoned fingerprints**: the shared target dir made the first
   post-baseline Red run execute HEAD-derived test binaries (mtime
   freshness). `python tools/distill.py -- cargo clean -p ariadne-cli`
   (exit 0) restored honest rebuilds; the Red evidence of section 3 is
   the clean run.
4. **D4 - test predicate fix (touch-set test module)**: the dead
   session's `find(|row| row.path.ends_with("Thing_extracted.h"))` also
   matches `Thing_Thing_extracted.h` (suffix collision); masked at Red
   behind the earlier rows assertion. Now file-NAME equality via
   `rsplit(['\\\\', '/'])`. The check itself (symbol_id-ordered single
   member `Thing`) is unchanged in intent.
5. **D5 - parity on one tree**: the dead session's two-copy parity is
   not satisfiable on this host: the discovery stack renders
   `consumer_file` absolute and canonicalized (`\\?\C:\...` - proven by
   the first Green run's assertion sides, differing only by temp root),
   so the two copies can never match byte-for-byte. The preview now runs
   against the SAME booted temp tree (stronger parity: identical
   strings, one root); its plan/migration artifacts land inside that
   test's own temp dir; the W2 read-only proof keeps its own separate
   boot (test 4 is untouched and green).
6. **D6 - ui.rs fee**: +101 lines net (885 -> 986) vs the "~30 call
   sites" target, driven by the plan feed; under the 1000 ceiling so the
   STOP clause does not fire (section 7).
7. **D7 (rider-style, interpretation note)** -
   `includes_unavailable` operationalization: "the engine could not
   resolve deps" is implemented as "a member symbol is absent from the
   generation's reduced model" - the only state in which the engine has
   no edges to resolve (documented at `unit_manifest_row`). A resolved
   symbol with no cross-unit dependency yields an empty, available list.
   The degraded answers (backend/discovery failure) use the Red-protocol
   honest-absent shape and are never cached.

## 12. Bench duty

This hand touches `crates/*/src`, so the assets-only bench exemption
does NOT apply; per the handoff ("bench duty is the reviewer/operator
hop per the kickoff lock table") no `bench_compare` run is claimed here.
The stale-baseline operator row (4 prior encounters) remains open and
the boot path was not slowed by design: the reader construction in
`swap_into_state` is three allocations, and no planner run was added
(m00 green).
