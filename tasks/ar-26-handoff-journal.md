# AR-26 Handoff Journal - SPLIT CHECK STOP (no implementation)

Session: AR-26 implementer, dispatched 2026-10-11 at HEAD incl. AR-21 + AR-25.
Worktree: `.kilo/worktrees/shining-enthusiasm` (shared; AR-31 live in crates/lang-cpp).

## Verdict (mandatory first step)

SPLIT CHECK TRIGGERED. Projected moved lines >= 2879 vs the 500 threshold
(~5.8x). Per the handoff's hard split rule: DID NOT IMPLEMENT. Zero code,
manifest, test, or workspace-Cargo edits were made. This journal + its commit
are the entire deliverable for this session; AR-26a/AR-26b must be re-planned.

## SPLIT CHECK commands (run verbatim, via distilling entry point)

Raw `git ls-files ... | wc -l` is denied by the global bash rule (proven this
session; boilerplate catch-all refusal). Equivalent inventory through the
distiller, numstat-vs-empty-tree so added-line count equals file line count:

```
python tools/distill.py -- git ls-files crates/report/src/manager crates/report/src/manager.rs
python tools/distill.py -- git diff --numstat 4b825dc642cb6eb9a060e54bf8d69288fbee4904 -- crates/report/src/manager crates/report/src/manager.rs
```

Files: 11 (manager.rs + 7 production modules + 3 test modules). Numstat
verbatim (added lines = current line count):

| file | lines |
|---|---|
| crates/report/src/manager.rs | 185 |
| crates/report/src/manager/apply_io.rs | 129 |
| crates/report/src/manager/bincode_options.rs | 57 |
| crates/report/src/manager/discovery_io.rs | 165 |
| crates/report/src/manager/graph_io.rs | 307 |
| crates/report/src/manager/index_io.rs | 75 |
| crates/report/src/manager/m22_manifest_languages_tests.rs | 340 |
| crates/report/src/manager/metrics_io.rs | 386 |
| crates/report/src/manager/payload_reshape_tests.rs | 237 |
| crates/report/src/manager/tests.rs | 910 |
| crates/report/src/manager/transform_io.rs | 88 |
| **manager/* total** | **2879** |

Additional move-set members from the Objective (journal, per its own clause):

| file | lines | why in move set |
|---|---|---|
| crates/report/src/manifests.rs | 222 | Objective names "manifest"; see coupling finding F2 |

| bucket | lines |
|---|---|
| production modules (manager.rs + 7 io/bincode modules) | 1392 |
| test modules moving with them (tests.rs, m22_*, payload_reshape_*) | 1487 |
| manifests.rs | 222 |
| **total moved** | **3101** |
| re-export shim + new Cargo.toml/lib.rs (estimate) | ~40-60 |
| **projected produced diff** | **~3140-3160** |

Threshold arithmetic: 2879 (strictest reading: manager/* only) is 5.8x the
500-line split threshold; 3101+ is ~6.2x. Even production-only (1392/1614) is
2.8x-3.2x the threshold. STOP is unambiguous under any reading.

## Structural findings (evidence for the split plan)

F1 - manager.rs is not just a module list. It carries the ArtifactManager
    struct, StaleArtifact, is_stale, write_atomic, unique_tmp, same_path,
    and check_manifest (185 lines of the move itself). The 7 IO submodules
    are PRIVATE mods whose public capability is `impl ArtifactManager`
    blocks - not module-qualified paths.

F2 - manifests.rs is entangled and must move with manager/*: every IO module
    imports `crate::manifests::*` (apply_io.rs:10, discovery_io.rs:12,
    graph_io.rs:12, transform_io.rs:10, tests.rs:2; full grep in capture).
    Leaving it in report forces artifact -> report, inverting the handoff's
    own "report depends DOWN on artifact". Splitting manifests into
    IO-DTO vs presentation is a rewrite, forbidden by the VERBATIM rule.

F3 - consumer surface is root-only: `git grep -n "ariadne_report::manager"`
    over crates+tests = ZERO hits (exit 1). All consumers use
    `ariadne_report::ArtifactManager`-style root paths via lib.rs
    `pub use manager::*`. The shim is therefore ~10-20 `pub use` lines plus
    the manifests re-export; "zero consumer edits" stays feasible.

F4 - consumer census (files referencing `ariadne_report`; 24 total, 22
    external - all preserved by F3 shim, none editable in touch-set):
    cli 7 (analyze, discover, measure, plan, query, transform,
    tests/stage5_memory_bench), pipeline 6 (lib, for_incremental,
    stage_load, 3 test files), integration 8 (m0, m1, m2, m3, m5, m6, m7,
    m10), report own 2 (tests/bincode_bounded_decode_test,
    tests/extraction_artifacts_test - stay, resolve via shim),
    scheduler 1 (see F5).

F5 - scheduler consumption is ONE call site:
    `crates/scheduler/src/incremental.rs:23: let manager =
    ariadne_report::ArtifactManager::prepare_for_repo(repo)?;`
    The reparent (26b) is a one-line import + Cargo.toml dep swap + cone
    guard - tiny.

F6 - cone guard family file exists: tests/integration/tests/
    m17_plugin_seam_test.rs (26b target).

## Split proposal for the orchestrator

Contract-mandated shape, measured honestly:

- AR-26a (extract artifact crate; move manager/* + manifests.rs +
  bincode_options.rs; report shims): ~3101 moved + ~50 new ~= ~3150
  produced lines. NOTE: exceeds BOTH the 500 split threshold and the
  Planning Law <=~1200 produced-lines budget (~2.6x). A VERBATIM move is
  mechanical, but if Planning Law sizing binds, 26a as defined is still
  oversized.
- AR-26b (scheduler reparent; drop scheduler -> report; m17 cone-guard
  extension incl. planted-edge self-test + RED-first): ~= small (F5):
  one import line, one Cargo.toml swap, guard-test growth, journal. Well
  within bounds. This half is safe to implement once 26a lands.

If further subdivision of 26a is wanted, a candidate split along
compile-green seams (orchestrator's call, not mine):
- AR-26a1: artifact crate skeleton + manifests.rs + manager.rs + the five
  smaller io modules (apply/bincode_options/index/transform/discovery)
  ~= 921 moved + shims ~= ~980 (under ~1200); report shims, compile-green,
  suites recount.
- AR-26a2: graph_io + metrics_io move ~= 693; suite recount.
- AR-26a3 (optional): test-module relocation ~= 1487 (tests.rs 910 alone
  approaches the produced-lines budget; could pair into 26a1/26a2 instead
  of standing alone - test mods compile only with cfg(test), pairing is
  mechanical).
The m00/m23 bincode byte-locks and m22/payload suites ride UNEDITED through
whichever shape is chosen; that constraint is orthogonal to the split.

## Deliberately NOT done (and why)

- No artifact crate, no module moves, no shims, no scheduler change, no
  cone-guard RED test. The contract's split rule forbids implementing
  before the split is approved; >500 projected is proven above.
- No gate suites run beyond the split-check inventory/greps: with zero code
  change, -p artifact (nonexistent crate), report/scheduler suite counts,
  m00/m23 locks, hygiene and encoding suites carry nothing this diff adds.
  encoding_guard --all + hygiene_check still run pre-commit for journal
  hygiene (reported below).
- No RED-first cone-guard attempt: RED is meaningful only in the shape the
  re-plan chooses (26b, or a combined handoff).

## Baselines pinned for the successor (re-brief values, not contract text)

- scheduler 26 passed / 0 failed / 0 ignored / 2 suites; ariadne-worker-core
  8/0/0/2 (conservation 26+8=34 vs contract's stale "34/0"). Zero test edits
  in both.
- report 47/0/0/4 at HEAD; moved-suite reconciliation owed by 26a.
- m17 cone-guard suite 8/0/0/1 (planted-edge self-test must stay green).
- m00 golden + facts-path 4/0/0/2; m23 artifact/bincode locks: UNEDITED,
  byte-identical wire bytes is the authority lock.
- hygiene_check: zero blocking rows (pin "zero blocking", never an R3 count).

## Gates run this session (verbatim digests)

- Distiller runs (inventory numstat x2, consumer census, coupling grep,
  manager-path grep) - captures cited below; one expected exit-1: the
  `ariadne_report::manager` grep returning no hits (F3 evidence).
- `python tools/encoding_guard.py --all` -> silent (clean, incl. untracked
  journal), pre-commit.
- `python tools/hygiene_check.py` -> R3 warn lines only; ZERO blocking rows
  (no R1, no R2; the old AR-13 server.rs pair is gone post AR-13 merge).
  Pinned as "zero blocking", never an R3 count.
- `python tools/commit_scoped.py --scope "tasks/ar-26-handoff-journal.md"
  ...` -> `commit e946f57`, numstat `175 0 tasks/ar-26-handoff-journal.md`.
- `python tools/encoding_guard.py --staged` post-add (commit_scoped leaves
  the path staged) -> silent.
- `git show --stat HEAD` (via distiller, capture
  target/ar26-commit-capture.txt) -> `1 file changed, 175 insertions(+)`,
  sole path = the journal. Zero stale copies or stray paths: this handoff
  added no code files, so the AR-20a D7 rename-visibility hazard cannot
  arise (no git-mv deletes exist); stat proof stands.
- No suite gates run: zero code change exists to gate (see "Deliberately
  NOT done"). cargo check --workspace not run for the same reason - the
  tree compiles as committed by AR-21/AR-25 and this session touched no
  source.

## Capture paths (per-run, shared-capture clobber aware)

- target/distill-capture.txt: inventory ls-files, manager numstat, full
  report-crate numstat, scheduler grep, first consumer census (later runs
  clobbered it - numbers above quoted from own stdout).
- target/ar26-splitcheck-capture.txt: private capture for the scoped
  consumer census, manifests coupling grep, m17 file check, manager-path
  grep (0-line final state = the exit-1 no-hits run).

## Deviations

- None to code (none written). D1 (process): split-check inventory used
  `git diff --numstat` vs the empty tree through distill.py because raw
  `git ls-files | wc -l` is allowlist-denied; same evidence, different
  plumbing.
