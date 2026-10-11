# AR-26a2 Handoff - Atomic `manager/*` move into `ariadne-artifact` + report shim (A3 chain, hop 2/3)

Successor of `tasks/ar-26a-handoff.md` (crate skeleton + manifests move);
dispatch ONLY after that hop is APPROVED and landed, and re-baseline every
suite line at YOUR dispatch HEAD. Evidence base:
`tasks/ar-26-handoff-journal.md` F1/F3/F4/F5 - numstat-proven, do not
re-measure.

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

Invariants preserved: `report` depends DOWN on `artifact` only (F2: the
manifests hop already made this direction real); scheduler cone unchanged
this hop (still `scheduler -> report -> artifact`); bincode wire bytes
byte-identical; ZERO test edits anywhere.

## Baselines (dispatch-HEAD = AR-26a landing; re-measure and journal BEFORE
any mutation)

- report `47 passed / 0 failed / 0 ignored / 4 suites` (decomposition: lib
  40 = 24 `tests.rs` + 11 `m22_manifest_languages_tests` + 5
  `payload_reshape_tests`; bincode_bounded_decode 6; extraction_artifacts
  1; Doc-tests 0).
- scheduler `26/0/0/2`; ariadne-worker-core `8/0/0/2`; m17 `8/0/0/1`;
  m00 `4/0/0/2`; cli `118/0/8/7`; lang-cpp `155/0/0/2`.
- hygiene ZERO blocking rows. Census at HEAD `1397/0/13/103` (reviewer's
  pass).

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
  `1397`; suite count +0 versus hop-1 state (artifact already contributed
  its two empty targets; its lib target gains the 40 tests, report's lib
  target goes to 0 tests but still counts as a suite). Reviewer's fresh
  census: `1397/0/13/105` versus HEAD `.../103` - the +2 is
  artifact's lib + Doc-tests targets; journal verbatim, do not force it.

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
- `crates/report/src/lib.rs` (final content below - the 4 shim lines + doc
  comment; the ONLY authored edits in report)
- `crates/report/Cargo.toml` (UNTOUCHED except: keep everything as landed
  by hop 1; do NOT prune orphaned deps this hop - cargo does not error on
  unused deps; a journal note suffices)
- workspace `Cargo.toml`, `crates/report/src/manifests.rs`: UNTOUCHED
  (already landed in hop 1)
- `Cargo.lock` (cargo regenerates if dep additions change resolution)

You MUST NOT touch: `crates/report/tests/**`, `crates/scheduler/**`,
`crates/cli/**`, `crates/pipeline/**`, `crates/lang-cpp/**` (AR-31 live
lane), `tests/**` (every suite and golden UNEDITED), `crates/ui/**`,
`tasks/ar-26-handoff.md`, `tasks/ar-26-handoff-journal.md`. No `git stash`,
no branch change.

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

`crates/report/src/lib.rs`, in full:

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
its new home; its `use crate::manifests::*` consumers inside the IO
modules now resolve against ARTIFACT's crate root (no shim needed inside
artifact). Report-internal `crate::manifests` resolution through the shim
binding is AR-20a-proven (hop 1 mechanism, unchanged).

## RGB protocol (extraction hop - RED-first NOT mandated; byte-locks are
the regression)

1. Journal dispatch HEAD + all baseline lines BEFORE mutation.
2. Byte-safe move: create `crates/artifact/src/manager/`, then `git mv`
   all 11 files (stage the renames FIRST so `--find-renames` sees them).
   NEVER recreate files via Edit round-trips (LF-slip precedent).
3. Shim load-bearingness mutation proof (uncommitted, AR-20a precedent):
   with shims in place, comment out the two `manager` shim lines in
   `crates/report/src/lib.rs`, run
   `python tools/distill.py -- cargo check -p ariadne-scheduler`, capture
   the expected `error[E0433]`/`error[E0432]` (cannot find
   `ArtifactManager` in `ariadne_report`, at
   `crates/scheduler/src/incremental.rs:23`), restore byte-exact,
   `git diff` clean against the intended final shim. NEVER commit a red
   state.
4. Green: all gates below. 5. Refactor: NONE on moved code; hygiene review
   covers only the two lib.rs files and Cargo.toml deltas.

## Hygiene (review-enforced)

The ceiling at risk: `crates/artifact/src/manager/tests.rs` is 910 lines
against the 1000-line file ceiling - it moves whole and must NOT grow
(zero hunks allowed anyway). `metrics_io.rs` 386 and `graph_io.rs` 307
stay well under. Post-move `report/src/lib.rs` is 8 lines.
`python tools/hygiene_check.py` exit 0 with ZERO blocking rows.

## Frozen locks (verify + quote)

- `python tools/distill.py -- git diff <dispatch-HEAD> -- tests crates/report/tests` -> EMPTY except the rename pairs of the 11 moved files (no test content change, no golden change).
- `git status --porcelain -- tests/golden` empty.
- m00 golden pair + m23 artifact suite + `crates/report/tests/bincode_bounded_decode_test.rs` + `crates/report/tests/extraction_artifacts_test.rs`: UNEDITED, byte-identical (bincode wire bytes authority); their green runs ARE the byte-lock proof.
- m17 guard file UNTOUCHED this hop:
  `python tools/distill.py -- git diff <dispatch-HEAD> -- tests/integration/tests/m17_plugin_seam_test.rs` -> EMPTY (AR-26b owns it).

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/distill.py -- cargo check --workspace --all-targets`
   -- GREEN first (proves all 22 consumers + report's own test files
   compile UNEDITED through the shim).
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
6. `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test` -- `4 passed / 0 failed / 0 ignored / 2 suites` UNEDITED + golden status porcelain empty.
7. `python tools/testrun.py --distill -p ariadne-integration-tests --test m23_symbol_isolation_test` -- matches journalled dispatch-HEAD baseline, UNEDITED.
8. `python tools/hygiene_check.py` -- exit 0, zero blocking rows.
9. `python tools/encoding_guard.py --staged` -- exit 0 before committing.
10. `python tools/encoding_guard.py --all` -- exit 0, journal the run.

Commit (single cross-crate commit, AR-18/AR-20a precedent):
`python tools/commit_scoped.py --scope "crates/artifact/**" --scope "crates/report/**" --scope "Cargo.lock" --scope "tasks/ar-26a2-handoff-journal.md" -m "refactor(artifact): move manager IO + tests verbatim to ariadne-artifact; report re-export shim (AR-26a2)"`
(never `git add .`).

## Rename-completeness proof clause (AR-20a D7 - mandatory)

`commit_scoped.py` cannot see staged `git mv` deletes. Immediately after
the commit, via distiller: `git show --stat -M HEAD` must list EXACTLY the
11 rename pairs (old path deleted / new path added, or R100 renames) plus
the two lib.rs and Cargo.toml deltas and the journal - no other path, no
stale copy; then `git ls-files crates/report/src/manager crates/report/src/manager.rs`
must return NOTHING (nothing tracked at the old paths). If any old-path
deletion fell out of the scoped commit (the D7 hazard), land an immediate
second scoped commit for the stray deletions ONLY and journal it. Stat
paths outside the touch-set: self-declared blocker, report - do not amend
silently.

## Discipline clauses

- UTF-8: never Set-Content/Out-File/>/Get-Content-replace; Write/Edit tools
  only; authored content pure ASCII (AGENTS.md File-Edit Discipline).
- VERBATIM only: all 11 moved files zero content hunks (journal evidence:
  `python tools/distill.py -- git diff <dispatch-HEAD> --find-renames -M --stat`).
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
- Serialization: hop 1 must be APPROVED and landed before dispatch; this
  hop runs alone (no other 26-chain hop may overlap - shared
  `crates/report/**` + lockfile); AR-31 (lang-cpp) may run concurrently,
  rebase + regenerate `Cargo.lock` on collision.

## Report back (final message)

1. Commit hash + distilled `git show --stat -M` + D7 proof outputs.
2. Baseline lines (pre-mutation) + mutation-proof capture (command +
   expected E0433 + restore confirmation). 3. Green evidence: gates 2-7
   canonical lines verbatim from own stdout + capture paths + the
   conservation equation. 4. Any deviation + reason. 5. Out-of-scope
   observations (report-only). Never paste raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the list above (including
`Cargo.lock` precisely because cargo regenerates it).
