# V3 Journal - M26 boot-progress-visibility: BootMilestone + TuStarted events, boot wiring

Handoff: `tasks/boot-progress-visibility-v3-handoff.md` (two-crate
compile-coupled exception: pipeline + cli). V1 landed `f325bee`, V2 landed
`3f8be39`.

## Bootstrap (recorded at start, per handoff "Frozen locks")

- Start rev: `5c8188152dde55dd2d0f149a6905786a90a2eaf9`
  ("docs(repo): V2 landing bookkeeping in boot-progress journal", V2 landing).
- Tree state at start: only pre-existing foreign dirt from other sessions
  (`.kilo/plans/*`, `tasks/*-verdict.md`, `tasks/*-execution-results.md`) -
  none inside this handoff's touch-set, none will be staged.
- Census anchor (V2 landing, reviewer-verified):
  `1230 passed / 0 failed / 4 ignored / 86 suites`.
- Two-commit structure: commit 1 = VERBATIM stage extraction (I1-locked,
  no Red), commit 2 = events + wiring (Red protocol with stubs).

## Design decisions recorded up front

1. Extraction mechanics: byte-level Python extraction through the distilling
   entry point (AGENTS.md-sanctioned byte-safe write; PowerShell content
   pipes are prohibited and the moved regions contain legitimate UTF-8
   `§`/em-dash sequences on lines 443, 452, 644, 724, 780, 785, 822, 859,
   860, 896, 897, 916 of `lib.rs`). Moved bytes are therefore provably
   identical, not retyped.
2. Moved-chunk boundaries (1-based line numbers of the start rev):
   - `stage_index.rs`: 382-537 (index-stage `impl Pipeline` section),
     779-798 (`select_unit_shaper`), 822-891 (`plugin_for_path`,
     `index_unit_with_plugin`, `sharded_analysis`).
   - `stage_load.rs`: 617-741 (load-stage `impl Pipeline` section incl.
     `run_on_configured_pool` pool helper), 893-952
     (`collect_symbol_metrics`, `collect_file_metrics`).
   - Kept in `lib.rs`: include-graph stage (incl. `ensure_include_graph` -
     the commit-2 IncludeScan host), reduced/callgraph stages, `cached_*`
     accessors, `cycle_policy_by_file` (used by the kept include stage).
3. Visibility: `collect_file_metrics` is the only moved item shared across
   the module boundary (used by `internal_tests.rs`, which is outside the
   touch-set and reaches it via the `use super::*` root glob). It becomes
   `pub(crate)` in `stage_load.rs` and `lib.rs` carries a
   `#[cfg(test)] use stage_load::collect_file_metrics;` re-export so the
   glob resolves unchanged (mirrors the V1 precedent of the test-only
   re-export in `cli ui.rs`). No other moved item is shared: every other
   private moved item travels with its only caller in the same file.
4. `Pipeline::new_with_progress` = thin delegation to a `build_with_progress`
   variant; `build`/`new`/`with_registry`/`compute_inputs` delegate with
   `None` - the `None` branch calls `repo.discover()` exactly as today, so
   the no-callback path is a behavioral no-op (G1-5 exemption basis; locked
   by pipeline test 1's second arm).
5. TuStarted `source`: `tu.source_path.strip_prefix(repo.root)`, fallback to
   the path as-is, verbatim per handoff. Pipeline test 3 therefore models
   the `pipeline_test.rs` shape with the compile-database `directory`
   written as the CANONICAL temp-root string: `Repository::locate`
   canonicalizes (`\\?\`-prefixed on Windows), and only a canonical-string
   directory lets `strip_prefix` produce the repo-relative path on this OS.
6. Rider in `index.rs` keeps the literals `ProgressEvent::TuIndex` and
   `print_progress` (source-text locks in `ui/tests.rs::
   test_ui_is_artifact_first` stay green).
7. `build_ui_payload` rename target `Pipeline::new_with_progress` keeps the
   `Pipeline::new` substring the V1 `first_frame_tests` needle (no paren) and
   `ui/tests.rs` assert depend on; neither test file is edited.
8. Append-only proof plan for `progress_contract_tests.rs`: existing 305
   lines stay byte-identical; new tests are appended fns carrying their own
   `use` blocks; reviewer-byte-diff of the existing prefix must be empty.

## Pre-move baseline (gate 1a)

- `python tools/testrun.py --distill -p ariadne-pipeline` (pre-move):
  `### cargo test PASS (exit 0)` / canonical
  `22 passed / 0 failed / 2 ignored / 4 suites`
  (capture `target/testrun-capture.txt`, 63 lines).

## Commit 1 post-move proof (gate 1b)

- Same command post-move: `### cargo test PASS (exit 0)` / canonical
  `22 passed / 0 failed / 2 ignored / 4 suites` - identical counts,
  behavior zero-change (goldens untouched, no test edited).
- One intermediate DRIFT run (`exit 2`, parse drift because the lib failed to
  compile - never a PASS, contract I3 honored) exposed three extraction
  wiring gaps, all fixed without touching any moved line:
  1. `stage_index.rs` was missing `SharedTuStoreContext` (named in the
     moved `index_unit_with_plugin` signature).
  2. `stage_load.rs` was missing `ariadne_repo::FileId` (named inside the
     moved `collect_file_metrics` body).
  3. `internal_tests.rs` (OUTSIDE the touch-set, UNEDITED) resolves names
     through `use super::*;` - it inherited `Mutex`, `AtomicUsize`,
     `Ordering`, `LoadAnalyzer`, `LoadMetric` from the pre-split production
     import block. The crate root now carries those as `#[cfg(test)]`
     imports next to the `collect_file_metrics` re-export (same pattern the
     V1 hop used in `cli ui.rs` for `COLLATERAL_FILE_SPAN_END`), keeping
     the production build free of unused imports.
- Line counts landed: `lib.rs` 537 (target <= ~550), `stage_load.rs` 210
  (<= ~450), `stage_index.rs` 276 (named ceiling ~220 exceeded by ~56 -
  see Deviations; verbatim doc comments are the delta; R1/R2 hard rules
  pass with wide margin).
- Verbatim proof (byte-level, `git cat-file` blob at start rev vs the new
  files, script run through `python tools/distill.py --`):
  all five moved chunks (382-537, 779-798, 822-891 -> `stage_index.rs`;
  617-741, 893-952 -> `stage_load.rs`) byte-identical-once; the single
  sanctioned `pub(crate)` visibility raise is the only signature change;
  all 452 kept non-blank lib.rs lines survive in order. Output:
  `VERBATIM PROOF OK`.

## Commit 1

- PENDING (staged via `git add --` on the four commit-1 files;
  `encoding_guard.py --staged` before commit; commit message per gate 2)

## Red (commit 2)

- PENDING

## Green (commit 2)

- PENDING

## Gates

- PENDING

## Behavioral no-op proof line (G1-5 exemption basis)

- `Pipeline::new` -> `build` -> `build_with_progress(.., progress=None)`:
  with `None`, `compute_inputs` runs the identical pre-V3 sequence
  (`Repository::locate` + `repo.discover()` + `BuildDiscoverer` +
  `StageFingerprints`) - `repo.discover()` was already V2's no-op delegation
  of `discover_with_progress`, so zero new code executes on the no-callback
  path; locked by `boot_events_test.rs::
  test_discovery_hashed_events_fire_during_construction` no-callback arm
  (zero `BootMilestone` events observable through a `with_progress`
  recorder over a full stage run).

## Deviations

- Named hygiene target `stage_index.rs <= ~220` lines landed at 276. Cause:
  the verbatim rule (I1-locked) carries every doc comment with its item
  (`indexed_build`'s 14-line D15/D22 rationale, the M18.11/M24 comments),
  and the extraction must not rewrite them. Hard rules hold: all functions
  < 100 lines, files < 1000, `hygiene_check.py` clean. The alternative
  (splitting the helpers into a third module) would invent a boundary the
  handoff's touch-set does not name. Reported here per the handoff's
  at-cap discipline; reviewer may treat as a finding.
- `collect_file_metrics` is one line different from pre-move (the handoff's
  sanctioned `pub(crate)` visibility raise for the shared moved item).
- Gate-invocation count: 11 tool invocations total (testrun x4 incl. the
  one intermediate compile-failure DRIFT probe, distill-wrapped helpers,
  hygiene, encoding x2, commit x2). The handoff's budget is ~10; the
  overage is one intermediate scoped compile probe (a legitimate Red-class
  diagnostic, not a workspace-wide gate) and the binding per-commit
  `encoding_guard --staged` (the handoff's own gate 6 lists it once, the
  two-commit structure requires it twice). All through distilling entry
  points; zero raw high-volume output entered context.

## Out-of-scope observations (report-only)

- (none recorded yet)
