# AR-26a2 Handoff - Atomic `manager/*` move into `ariadne-artifact` + report shim (A3 chain, hop 2/3)

Successor of `tasks/ar-26a-handoff.md` (crate skeleton + manifests move),
now APPROVED and landed: code `4794467`, journal tip `075e3bf`
(`tasks/ar-26a-verdict.md`; hop 1 landed as its own R099 + one authorized
visibility hunk per ruling board_12d48349 riders 1-3 - that supersedes the
hop-1 contract's R100 expectation; hop 2's 11 moved files keep the strict
R100 rule). FROZEN-LOCK DIFF BASE for this hop: `4794467` (code anchor;
journal tip `075e3bf` carries zero code delta - either proves the frozen
set identical). Re-baseline every suite line at YOUR dispatch HEAD.
Evidence base: `tasks/ar-26-handoff-journal.md` F1/F3/F4/F5 -
numstat-proven, do not re-measure.

Binding inputs added by the verdict (findings 1-2) - read in full before
dispatch: `tasks/ar-26a-verdict.md` F1, and `tasks/ar-26a-handoff-journal.md`
sections "ADJUDICATED EXECUTION" + "RIDER FOR 26a2" (ruling board_12d48349
option A + riders). The RIDER is a banked mandate and is EMBEDDED below as
machine-checkable acceptance (see "Rider close" - verdict F1 exists
because the first version of this contract (at `957c391`) predated the
ruling and carried no rider content; this amendment discharges F1).

## Objective

Move the ENTIRE manager IO stack from `crates/report` to
`crates/artifact` VERBATIM via byte-safe renames - `manager.rs` plus its 7
private IO submodules plus its 3 in-src test modules (11 files, 2879
lines) - and make `report` a pure re-export shell whose every public path
still resolves, so all 22 external consumer files and report's own 2
integration test files compile UNEDITED. Behavior-free: the bincode wire
bytes, manifest JSON, and every suite outcome are byte- and
count-identical; the byte-locks are the regression.

Why ATOMIC (do not attempt a finer split - both alternatives are dead):
1. E0116: the 7 IO modules carry `impl ArtifactManager` blocks (journal
   F1); an inherent impl for a foreign-crate type does not compile, so
   `manager.rs` and all 7 IO modules share one crate in every green state.
2. E0583: the 3 test modules are declared at `manager.rs:15-20`, so the
   files travel with `manager.rs`; docking their mod declarations to
   strand them in `report` would be test edits (forbidden) and would leave
   a half-built system between hops (the wrong seam, Planning Law).
3. `bincode_options.rs` stays a private `mod` inside `manager.rs`
   (`manager.rs:8`, AR-03): grep-proven zero external path references, so
   it needs NO new public path - it moves byte-identical.

SECOND DUTY - close the AR-26a rider (ruling board_12d48349 rider 3,
verdict F1): once manager/* lands same-crate, `collect_stage_languages`
reverts to `pub(crate)` at its definition and the report shim's rider line
+ allow pair + rationale comment are all deleted. Terminal public surface:
the 12 manifest DTOs only (flat + module-qualified); no gate-visible
`pub` helper. This is machine-checked by gate 7 - see "Rider close".

Line budget: the 2879 moved lines exceed the ~1200 produced-lines bound -
PRE-ACCEPTED by the orchestrator (inventory, not authored lines: `git mv`
renames land R100 with ~0 churn; the session authors ~20 lines of shims and
manifests). Do NOT split further to chase the budget; the atomicity proof
above supersedes it.

Invariants preserved: `report` depends DOWN on `artifact` only (F2: the
manifests hop already made this direction real); scheduler cone unchanged
this hop (still `scheduler -> report -> artifact`); bincode wire bytes
byte-identical; ZERO test edits anywhere; `ariadne-report` compiles with
its 4-line shim only (rider trio gone).

## Baselines (dispatch-HEAD = AR-26a landing `4794467`/`075e3bf`;
re-measure and journal BEFORE any mutation)

- CENSUS ANCHOR (AR-26a reviewer-measured, fully attributed - the
  planner's earlier `1397/.../103` projection is SUPERSEDED by verdict
  finding 2): `1398 passed / 0 failed / 0 ignored / 105 suites`
  (reconciliation: 1393/0/13/102 @AR-24 -> AR-25 +4 tests/+1 suite ->
  AR-31 +1 test -> AR-26a +0 tests/+2 suites).
- report `47 passed / 0 failed / 0 ignored / 4 suites` (decomposition
  unchanged: lib 40 = 24 `tests.rs` + 11 `m22_manifest_languages_tests` + 5
  `payload_reshape_tests`; bincode_bounded_decode 6; extraction_artifacts
  1; Doc-tests 0).
- artifact `0 passed / 0 failed / 0 ignored / 2 suites` (empty lib +
  Doc-tests target, as landed).
- m17 `8/0/0/1`; m23_symbol_isolation_test `7/0/0/1` (pinned by the AR-26a
  acceptance pass); m00 pair `4/0/0/2`; scheduler `26/0/0/2`;
  cli `118/0/8/7`.
- hygiene ZERO blocking rows (post-AR-13 baseline; quote the run, never an
  R3 count).

## Per-hop census/suite reconciliation (the contract's exact arithmetic)

Moved suites land under `-p ariadne-artifact`:

- Gate 2 (`-p ariadne-artifact`) expected:
  `40 passed / 0 failed / 0 ignored / 2 suites` (moved lib suite 40 + the
  crate's own empty Doc-tests target).
- Gate 3 (`-p ariadne-report`) expected:
  `7 passed / 0 failed / 0 ignored / 3 suites` (bincode_bounded_decode 6 +
  extraction_artifacts 1 + report's empty lib target with 0 tests + its
  Doc-tests target - quote verbatim; suite-count tolerance per-crate: the
  authority is PASSED conservation).
- Conservation equation (journal it explicitly):
  `40 + 7 = 47` passed, `0` failed both sides, ZERO tests authored, moved,
  edited, or deleted (cross-check: `#[test]` attribute count diff across
  `crates/report` + `crates/artifact` = 0).
- Census projection after this hop lands: passed total conserved at
  `1398`, suite count unchanged at `105` (artifact already contributed its
  two targets at hop 1; its lib target gains the 40 tests, report's lib
  target drops to 0 tests but still counts as a suite). Reviewer's fresh
  census expected `1398/0/13/105`; journal verbatim, do not force it -
  PASSED conservation (`40 + 7 = 47`) is the authority.
- Rider close adds/moves ZERO tests: `collect_stage_languages` keeps its
  three in-crate callers (now same-crate), so no dead_code warning and no
  suite delta (gate 7 proves the surface; the 40-tests artifact line above
  already includes their coverage).

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/ar-26a2-handoff-journal.md` (checkpoint journal; always in-set)
- `crates/artifact/src/manager.rs` + `crates/artifact/src/manager/` (the 11
  files MOVED VERBATIM via `git mv` from `crates/report/src/manager.rs`
  and `crates/report/src/manager/`: `manager.rs`, `apply_io.rs`,
  `bincode_options.rs`, `discovery_io.rs`, `graph_io.rs`, `index_io.rs`,
  `metrics_io.rs`, `transform_io.rs`, `tests.rs`,
  `m22_manifest_languages_tests.rs`, `payload_reshape_tests.rs`)
- `crates/artifact/src/lib.rs` (+`pub mod manager;` +`pub use manager::*;`
  - final content below)
- `crates/artifact/Cargo.toml` (add exactly what the moved modules
  reference: `anyhow.workspace = true`, `serde_json.workspace = true`,
  `bincode.workspace = true`, `rayon.workspace = true`,
  `ariadne-build = { path = "../build" }`,
  `ariadne-index = { path = "../index" }`,
  `ariadne-cluster = { path = "../cluster" }`,
  `ariadne-transform = { path = "../transform" }`,
  `ariadne-validate = { path = "../validate" }`; dev-deps:
  `tempfile.workspace = true` - add only what E0432/E0433 proves needed,
  journal each addition. The cluster/transform/validate edges are the
  honest transitive shape the moved IO signatures impose and MANDATORY per
  `tasks/ar-26b-shape-ruling.md` - no implementer may "clean up" or
  feature-hide them.)
- `crates/artifact/src/manifests.rs` (ONE hunk only - rider close: line 22
  `pub fn collect_stage_languages` -> `pub(crate) fn
  collect_stage_languages`. This is the ONLY content hunk authorized this
  hop, and it is on a NOT-moved file (it moved at hop 1); every other byte
  of this file is frozen)
- `crates/report/src/lib.rs` (final content below - doc comment + the 4
  shim lines ONLY: remove the rider trio of hop 1 - the rationale comment
  block, the `#[allow(hidden_glob_reexports, unused_imports)]` attribute,
  and the `pub(crate) use
  ariadne_artifact::manifests::collect_stage_languages;` line (currently
  `lib.rs:9-16`). The two `pub use ariadne_artifact::manifests` 12-DTO
  shim lines and the two `manager` shim lines STAY. These are the ONLY
  authored edits in report)
- `crates/report/Cargo.toml` (UNTOUCHED except: keep everything as landed
  by hop 1; do NOT prune orphaned deps this hop - cargo does not error on
  unused deps; a journal note suffices)
- workspace `Cargo.toml`: UNTOUCHED (member line landed at hop 1). Note:
  `crates/report/src/manifests.rs` does not exist anymore - hop 1 moved it
  to `crates/artifact/src/manifests.rs`, which is why that path is in-set
  above for the one-hunk re-privatization.
- `Cargo.lock` (cargo regenerates if dep additions change resolution)

You MUST NOT touch: `crates/report/tests/**`, `crates/scheduler/**`,
`crates/cli/**`, `crates/pipeline/**`, `crates/lang-cpp/**` (frozen
post-AR-31), `tests/**` (every suite and golden UNEDITED), `crates/ui/**`,
`crates/artifact/src/manifests.rs` beyond the single authorized re-
privatization hunk, `tasks/**` except your own journal
(`tasks/ar-26a2-handoff-journal.md`). No `git stash`, no branch change.

## The exact final shims

`crates/artifact/src/lib.rs`, in full:

```rust
//! Artifact-store IO: the ArtifactManager, manifest DTOs, and the bounded
//! bincode wire options that guard them (AR-26a extraction from
//! ariadne-report).

pub mod manager;
pub mod manifests;

pub use manager::*;
pub use manifests::*;
```

`crates/report/src/lib.rs`, in full - doc comment + 4 shim lines and
NOTHING else. Reaching this exact content IS the rider close on the report
side: the hop-1 rider trio (rationale comment + allow attribute +
`pub(crate) use ...::collect_stage_languages;`) is deleted in the same
commit; the 12-DTO `pub use ariadne_artifact::manifests` pair STAYS:

```rust
//! Presentation/report surface (AR-26a2: the artifact-store IO lives in
//! ariadne-artifact; every path preserved by re-export).

pub use ariadne_artifact::manager;
pub use ariadne_artifact::manager::*;
pub use ariadne_artifact::manifests;
pub use ariadne_artifact::manifests::*;
```

`manager.rs` itself is byte-identical (mod declarations and
`#[cfg(test)]` mounts included) - its `mod bincode_options;` resolves at
its new home.

CONSUMER IMPORT LINES - the `crate::` REBINDING ruling (what `4794467`
actually shipped, decided here per VERBATIM): all 9
`use crate::manifests::{...}` lines in the 11 moved files
(`apply_io.rs:10`, `discovery_io.rs:12`, `graph_io.rs:12`,
`index_io.rs:10`, `metrics_io.rs:13`, `transform_io.rs:10`, `tests.rs:2`,
`m22_manifest_languages_tests.rs:6`, `payload_reshape_tests.rs:20`) are
BYTE-UNCHANGED by this hop. At their new home `crate::` REBINDS to the
artifact crate root, where the REAL `pub mod manifests` provides the 12
DTOs and - for the three fn-importers (`discovery_io`, `metrics_io`,
`graph_io`) - the re-privatized `pub(crate)` fn via same-crate visibility.
The import TEXT does not change; only its resolution target does, from the
report shim binding to the same-crate module. Verbatim means the textual
rule; the rebinding is resolution-time, the same E-class the AR-20a shim
rule handled - journal the choice with these exact 9 sites. VERIFY at
dispatch HEAD that all 9 sites name `crate::manifests`; if any site names
`ariadne_artifact::manifests` instead, that is contract drift versus what
hop 1 landed - STOP, report, do not rewrite. No consumer import edit is
part of this contract.

## RGB protocol (extraction hop - RED-first NOT mandated; byte-locks are
the regression)

1. Journal dispatch HEAD + all baseline lines BEFORE mutation.
2. Byte-safe move: create `crates/artifact/src/manager/`, then `git mv`
   all 11 files (stage the renames FIRST so `--find-renames` sees them).
   NEVER recreate files via Edit round-trips (LF-slip precedent).
3. RIDER CLOSE (ruling board_12d48349 rider 3; machine-checked by gate 7,
   terminal state pinned in the shim section above). In the SAME landing
   commit, once the 11 files are in place:
   (a) re-privatize the definition:
   `crates/artifact/src/manifests.rs:22`
   `pub fn collect_stage_languages` ->
   `pub(crate) fn collect_stage_languages` - after the move the fn has
   three SAME-CRATE callers, so `pub(crate)` compiles and no dead_code
   warning can appear;
   (b) delete the rider trio from `crates/report/src/lib.rs` (rationale
   comment + `#[allow(hidden_glob_reexports, unused_imports)]` + rider
   line) while KEEPING the two 12-DTO manifests shim lines.
   Terminal artifact public surface = the 12 DTOs (flat + module-qualified);
   the `collect_stage_languages` symbol must exist in the source tree ONLY
   at its `pub(crate)` definition and the three same-crate consumer
   imports/call sites.
4. Shim load-bearingness mutation proof (uncommitted, AR-20a precedent):
   with shims in place, comment out the two `manager` shim lines in
   `crates/report/src/lib.rs`, run
   `python tools/distill.py -- cargo check -p ariadne-scheduler`, capture
   the expected `error[E0433]`/`error[E0432]` (cannot find
   `ArtifactManager` in `ariadne_report`, at
   `crates/scheduler/src/incremental.rs:23`), restore byte-exact,
   `git diff` clean against the intended final shim. NEVER commit a red
   state.
5. Green: all gates below. 6. Refactor: NONE on moved code; hygiene review
   covers only the two lib.rs files, the manifests.rs hunk, and Cargo.toml
   deltas.

## Hygiene (review-enforced)

The ceiling at risk: `crates/artifact/src/manager/tests.rs` is 910 lines
against the 1000-line file ceiling - it moves whole and must NOT grow
(zero hunks allowed anyway). `metrics_io.rs` 386 and `graph_io.rs` 307
stay well under. Post-move `report/src/lib.rs` is 7 lines (2 doc + blank +
4 shim); the `manifests.rs` hunk keeps that file at 222 lines.
`python tools/hygiene_check.py` exit 0 with ZERO blocking rows.

## Frozen locks (verify + quote)

Frozen-lock diff base: hop-1 landing - code anchor `4794467`, journal tip
`075e3bf` (zero code delta between them; use either, journal which).

- `python tools/distill.py -- git diff 4794467 -- tests crates/report/tests` ->
  EMPTY (this hop moves and edits NOTHING under either path - the 11-file
  rename lives under `crates/report/src/manager*`, verified R100 via the
  `--find-renames` discipline clause, not via this lock).
- `git status --porcelain -- tests/golden` empty.
- m00 golden pair + m23 artifact suite + `crates/report/tests/bincode_bounded_decode_test.rs` + `crates/report/tests/extraction_artifacts_test.rs`: UNEDITED, byte-identical (bincode wire bytes authority); their green runs ARE the byte-lock proof. Neither report test file imports `collect_stage_languages` (verified 2026-10-11), so the re-privatization cannot touch them.
- m17 guard file UNTOUCHED this hop:
  `python tools/distill.py -- git diff 4794467 -- tests/integration/tests/m17_plugin_seam_test.rs` -> EMPTY (AR-26b owns it).
- `crates/artifact/src/manifests.rs` frozen except the line-22 hunk:
  `python tools/distill.py -- git diff 4794467 -- crates/artifact/src/manifests.rs`
  -> exactly 1+1-, the `pub` -> `pub(crate)` visibility line.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/distill.py -- cargo check --workspace --all-targets`
   -- GREEN first (proves all 22 consumers + report's own test files
   compile UNEDITED through the shim, with the rider trio gone and the fn
   re-privatized). Zero-warning attribution clause: no warning lines
   attributable to `ariadne-report`/`ariadne-artifact` in own capture (the
   pre-existing 31-warning census of other crates is the hop-1 landed
   baseline - a rider-left-behind or reprivatize-missed failure surfaces
   here).
2. `python tools/testrun.py --distill -p ariadne-artifact` --
   `40 passed / 0 failed / 0 ignored / 2 suites` (reconciliation above).
3. `python tools/testrun.py --distill -p ariadne-report` --
   `7 passed / 0 failed / 0 ignored / 3 suites` (conservation equation
   `40 + 7 = 47` journalled).
4. `python tools/testrun.py --distill -p ariadne-scheduler` --
   `26 passed / 0 failed / 0 ignored / 2 suites` UNEDITED (proof the
   scheduler -> report -> artifact shim resolves with zero scheduler edits).
5. `python tools/testrun.py --distill -p ariadne-cli` -- `118 passed /
   0 failed / 8 ignored / 7 suites` UNEDITED (the 7 cli consumer files
   ride the shim).
6. `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test --test m23_symbol_isolation_test` -- `11 passed / 0 failed / 0 ignored / 3 suites` (decomposition m00 pair 4 + m23 pinned 7; UNEDITED) + `git status --porcelain -- tests/golden` empty.
7. RIDER CLOSE grep (one invocation):
   `python tools/distill.py -- rg -n "collect_stage_languages|hidden_glob_reexports|unused_imports" crates/report/src/lib.rs crates/artifact/src/manifests.rs`
   -- expected output: EXACTLY ONE line,
   `crates/artifact/src/manifests.rs:22:pub(crate) fn collect_stage_languages`
   (a `pub fn` prefix in the output = re-privatization missed = FAIL), and
   ZERO lines from the `crates/report/src/lib.rs` path (rider line, allow
   pair, and rationale comment all gone; if the comment survived it is
   caught by the allow-name patterns and by the pinned 7-line final
   content). Acceptance corollary stated for the reviewer: the terminal
   public surface of `ariadne-artifact` is the 12 DTOs (flat + module-
   qualified) + manager exports; `collect_stage_languages` exists in
   tracked source ONLY as the `pub(crate)` definition plus the three
   same-crate consumers. Journal the capture path + own stdout.
8. `python tools/hygiene_check.py` -- exit 0, zero blocking rows.
9. `python tools/encoding_guard.py --staged` -- exit 0 before committing.
10. `python tools/encoding_guard.py --all` -- exit 0, journal the run.

Commit (single cross-crate commit, AR-18/AR-20a precedent):
`python tools/commit_scoped.py --scope "crates/artifact/**" --scope "crates/report/**" --scope "Cargo.lock" --scope "tasks/ar-26a2-handoff-journal.md" -m "refactor(artifact): move manager IO + tests verbatim to ariadne-artifact; report re-export shim (AR-26a2)"`
(never `git add .`).

## Rename-completeness proof clause (AR-20a D7 - mandatory)

`commit_scoped.py` cannot see staged `git mv` deletes (hop 1 proved the
AR-20a recipe: leave the old-path deletion UNSTAGED so the ls-files-based
scope resolution sees it - landed all pairs in ONE commit). Immediately
after the commit, via distiller: `git show --stat -M HEAD` must list
EXACTLY the 11 rename pairs (R100 - zero content hunks on moved files;
hop 1's manifests move landed R099 by ruling, this hop's 11 files carry NO
authorized hunk) plus the deltas of `crates/artifact/src/lib.rs`,
`crates/artifact/Cargo.toml`, `crates/artifact/src/manifests.rs` (exactly
1+1-: the re-privatization), `crates/report/src/lib.rs`, `Cargo.lock`, and
the journal - no other path, no stale copy; then
`git ls-files crates/report/src/manager crates/report/src/manager.rs`
must return NOTHING (nothing tracked at the old paths). If any old-path
deletion fell out of the scoped commit (the D7 hazard), land an immediate
second scoped commit for the stray deletions ONLY and journal it. Stat
paths outside the touch-set: self-declared blocker, report - do not amend
silently.

## Discipline clauses

- UTF-8: never Set-Content/Out-File/>/Get-Content-replace; Write/Edit tools
  only; authored content pure ASCII (AGENTS.md File-Edit Discipline).
- VERBATIM only: all 11 moved files zero content hunks AND all 9
  `use crate::manifests::{...}` import lines byte-unchanged (the `crate::`
  rebinding does the work; see the shim section - a "repointing" edit to a
  consumer import is OUT OF CONTRACT, not in it). The ONLY content hunk
  anywhere is the mandated rider-close re-privatization on
  `crates/artifact/src/manifests.rs:22` - pre-mandated by ruling
  board_12d48349 rider 3, so NOT a deviation. Journal evidence:
  `python tools/distill.py -- git diff 4794467 --find-renames -M --stat`.
- ZERO test edits: the 40 + 7 conservation equation is the lock; any test
  file content change is a self-declared blocker.
- Touch only the touch-set; out-of-scope breakage: STOP, report, do not fix.
- No raw high-volume commands; distilling entry points only
  (`docs/specs/tool-output-distillation.md`); quote canonical lines from OWN
  stdout (capture-race ruling).
- After EACH Green gate: 1-3 line checkpoint to
  `tasks/ar-26a2-handoff-journal.md` (module + commit hash, deviations,
  findings). Context exhaustion: STOP, return disk-state inventory; dead
  sessions are NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ar-26a2-handoff.md
  --scope ... --cmd ...`.
- Serialization: hop 1 is APPROVED and landed (`4794467`) - this hop runs
  alone (no other 26-chain hop may overlap - shared `crates/report/**` +
  lockfile). AR-31 has LANDED (census attribution +1 test); no live lane
  touches `crates/report`/`crates/artifact` at dispatch. Should any
  foreign lane exist at dispatch HEAD: journal it, keep your cone checks
  green, rebase + regenerate `Cargo.lock` on collision, do not stage
  foreign files.

## Report back (final message)

1. Commit hash + distilled `git show --stat -M` + D7 proof outputs.
2. Baseline lines (pre-mutation) + mutation-proof capture (command +
   expected E0433 + restore confirmation). 3. Green evidence: gates 2-7
   canonical lines verbatim from own stdout + capture paths + the
   conservation equation; gate 7's expected single `pub(crate) fn
   collect_stage_languages` line quoted as the rider-close acceptance
   proof. 4. Any deviation + reason. 5. Out-of-scope observations
   (report-only). Never paste raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the list above (including
`Cargo.lock` precisely because cargo regenerates it).
