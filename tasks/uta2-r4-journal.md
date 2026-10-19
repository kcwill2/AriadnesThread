# UTA2-R4 Journal - sweep-B remainder (worker-core drop pin + small crate completions)

Handoff: tasks/uta2-r4-handoff.md. Binding spec: docs/specs/unit-test-audit.md
"Round 2" (drop pin + rows 1, 8-18). All pins; no behavior changes.
Start state: main at tip b43d6da (uta2 R3 landed). Tree carried only reviewer
artifacts outside this touch-set (tasks/uta-execution-results.md modified,
uta2-r1/r2/r3 verdicts + r2 probes untracked) - deliberately not touched.

## Baselines (before, per-crate canonical lines, `python tools/testrun.py --distill -- -p <crate>`)

- worker-core: `19 passed / 0 failed / 0 ignored / 2 suites`
- scheduler:   `40 passed / 0 failed / 0 ignored / 2 suites`
- build:       `28 passed / 0 failed / 0 ignored / 2 suites`
- cluster:     `67 passed / 0 failed / 0 ignored / 8 suites`
- artifact:    `57 passed / 0 failed / 0 ignored / 2 suites`
- transform:   `32 passed / 0 failed / 0 ignored / 4 suites`
- lang:        `44 passed / 0 failed / 1 ignored / 2 suites`
- validate:    `31 passed / 0 failed / 0 ignored / 5 suites`

## Crate 1 (GREEN): worker-core

- New `src/uta2_r4_pins.rs` (child of `tu_store` via `#[path]`, the
  `tu_store_t5_pins.rs` model - keeps legal access to the private
  `edge_destinations`): `pin_edge_destination_drops_unknown_id` covers the
  whole drop shape: Resolved(ghost) drops SILENTLY (no error/counter/fallback
  - the ghost's bytes are planted on disk AFTER discovery to prove the
  Resolved/Wildcard arms never reach the `on_disk_include_fallback`);
  Wildcard fan-out drops only the unknown ids, keeps known survivors in
  resolver order with their discovery paths; all-unknown fan-out visits
  nothing.
- REPORT (UTA-B6 class, no fix - shape decision owed): recommend a surfaced
  dropped-id counter (+reasons) on the closure-walk output, mirroring how
  `External`/`Unresolved` keep their on-disk fallback visible.
- New `src/worker_backend_uta2_r4_pins.rs` (child of `worker_backend`):
  `pin_arc_backend_delegates_to_inner_once_with_verbatim_arguments` -
  delegating spy with call-count + argument recording, pinned through both
  live shapes: `Arc<Concrete>` (WorkerPool storage) and `Arc<dyn
  WorkerBackend>` (M18.3 registry wiring). Exactly one inner call per outer
  call, verbatim task id + repo root, inner's Ok passed back unchanged.
- First gate run hit a COMPILE error (my harness typo: `Arc::clone` inference
  against the `Arc<dyn>` let-binding), not a semantic red - fixed the
  coercion site; second run green.
- Gate: `python tools/testrun.py --distill -- -p ariadne-worker-core` ->
  `21 passed / 0 failed / 0 ignored / 2 suites` (19 baseline + 2 pins;
  capture: target/testrun-capture.txt).

## Crate 2 (GREEN): scheduler

- New `src/uta2_r4_pins.rs` (child of `incremental.rs`):
  - `pin_save_artifacts_surfaces_stage_io_failure_with_no_manifest_written` -
    plain file planted on `artifacts/repository` (the FIRST stage's dir slot)
    fails its create_dir_all; Err surfaces naming the failing creation; no
    manifest exists for repository, index, or reduced (later stages never
    attempted). DEVIATION (mechanics, disclosed): the handoff names a
    "read-only dir" injection; read-only dir attributes are advisory for our
    own writes on this Windows host, so the pin uses the t5
    `save_shard_surfaces_create_dir_all_failure_as_error` blocker-file
    precedent - same IO-failure class, deterministic cross-platform.
  - `pin_analyze_surfaces_shard_save_failure_keeping_earlier_shards` -
    second TU's shard FINAL path planted as a DIRECTORY (rename-over-dir is
    rejected on Windows and Unix alike): first shard persists complete
    (envelope reloads), planted target untouched, exactly one `.tmp` staging
    residue survives (pin: no cleanup, no rollback), Err surfaces before any
    stats/index work. Partial-save state locked.
- First run: COMPILE error only (`expect_err` needs `Debug` on
  `IncrementalAnalysisResult`); switched to the t4-shape match-panic (no
  production derive added - pins only). Second run green.
- Gate: `python tools/testrun.py --distill -- -p ariadne-scheduler` ->
  `42 passed / 0 failed / 0 ignored / 2 suites` (40 baseline + 2 pins).

## Crate 3 (GREEN): build

- New `src/uta2_r4_pins.rs` (crate-level child, the `uta_t6_pins.rs` model):
  - `pin_conflicting_databases_losers_never_parsed_and_malformed_winner_never_falls_through` -
    the round-1 t6 suite already locks the plain deletion ladder, so R4 adds
    the CONFLICT dimension: (a) root valid + build/ + out/ UNPARSEABLE ->
    root parses alone (proves the ladder short-circuits at `exists()` and
    never touches losers); (b) root absent + build/ malformed + out/ VALID -
    > discovery Errs naming the build rung; NO fall-through to the parseable
    out/ (first-existing-candidate wins regardless of content).
  - `pin_malformed_compile_db_err_names_the_offending_file` - Err chain
    carries "Failed to parse compile_commands.json at" + the offending
    file's full Debug-rendered path (escaped-backslash form on Windows), not
    just the basename.
  - `pin_measure_tu_simulation_attributes_wall_clock_to_frontend_cpu` -
    honest attribution name: empty `arguments` skips the spawn entirely; the
    fallback copies `wall_clock_time` (with its 1ms floor) into
    `frontend_cpu_time` verbatim; preprocessing 0, memory mock 1MiB, and
    exit code 0 (a TU that never ran is reported as SUCCESS).
- Gate: -> `31 passed / 0 failed / 0 ignored / 2 suites` (28 baseline + 3 pins).

## Crate 4 (GREEN): cluster

- `plan.rs` gained a compact inline `mod format_summary_uta2_r4_pin` (file
  had ample headroom; round-1 tests byte-untouched):
  `pin_format_summary_single_shape` -     exact shape `{id}: {before} -> {after}
  symbols`, ASCII `->` and NO thousands separators (contrasting
  format_detail's comma groups and its own non-ASCII arrow, both locked by
  round-1), zero-count
  boundary, no-newline lock, and the field-set lock: two plans differing in
  every field except id/counts summarize IDENTICALLY.
- Gate: -> `68 passed / 0 failed / 0 ignored / 8 suites` (67 baseline + 1).

## Crate 5 (GREEN): artifact

- New `src/manager/uta2_r4_pin_tests.rs` (sibling of `uta_t6_pin_tests`,
  reusing its `manager_in_fresh_tempdir` helper):
  `pin_load_checked_variants_share_the_fingerprint_mismatch_verdict` drives
  ALL EIGHT variants (repository, contexts, index, includes, reduced,
  callgraph, load, plans) with a planted manifest carrying a foreign
  fingerprint, asserting each variant's exact verdict string
  `stale artifact for stage '<stage>': input fingerprint mismatch` + the
  StaleArtifact downcast. Payload-free stage dirs double-lock: the manifest
  gate runs before any payload read (no IO error leaks first) and pins each
  variant's stage-directory name.
- CONFIRMED shared-helper equivalence (grep-lock note, handoff item 5's
  second option): `self.check_manifest(` has exactly 8 production call
  sites, one per variant, each as the function's first statement -
  discovery_io.rs:71/143, index_io.rs:64, graph_io.rs:85/243/290,
  metrics_io.rs:323/383 - all matching the 8 `load_*_checked` definitions.
  `transform_io.rs` has NO checked variant (load_migration_artifacts is
  unchecked), so eight is the full set.
- Gate: -> `58 passed / 0 failed / 0 ignored / 2 suites` (57 baseline + 1).

## Crate 6 (GREEN): transform

- `placement.rs` gained inline `mod uta2_r4_truth_table_pin`:
  `pin_is_noop_false_for_identical_lists_carrying_preservations` completes
  the t4 truth table: identical old/new lists + NON-EMPTY preservations ->
  false; a preservation foreign to both lists still -> false (any
  non-empty list fails the gate); the same lists with the preservation
  dropped -> true (control restating the round-1 row) - preservations alone
  flip the verdict.
- Gate: -> `33 passed / 0 failed / 0 ignored / 4 suites` (32 baseline + 1).

## Crate 7 (GREEN): lang

- `extractor.rs` gained inline `mod uta2_r4_render_pins`:
  `pin_render_unit_routes_namespace_and_returns_extract_bytes_verbatim` -
  instrumented extractor double rendering a deterministic mini-header;
  pins (a) Some(source): the `unit_namespace` hook IS consulted, its answer
  reaches `extract`, default imports passthrough survives, bodies keep input
  order, and `render_unit` returns the extractor's bytes unchanged (no
  emitter-side decoration); (b) None: the hook is NOT called AT ALL (call
  log empty - the D-M23-2 rule pinned at call level, beyond round-1's
  byte-level locks) and the namespace is empty.
- Gate: -> `45 passed / 0 failed / 1 ignored / 2 suites` (44 baseline + 1;
  the 1 ignored is pre-existing).

## Crate 8 (GREEN): validate

- New `src/uta2_r4_pins.rs` (crate child, the `orchestration_tests` model)
  pinning the two IO-error observations the UTA-T4 review shipped:
  - `pin_capture_of_directory_target_surfaces_bare_io_error_before_any_mutation`
    - capture()'s typed `std::io::Result` binding itself pins the BARE
      error shape (no anyhow context/path naming); the directory's OS error
      KIND is platform text (Windows denied vs Unix is-a-directory), not
      contract, so no kind is asserted. Through `apply_with`: the chain
      carries NO "rolled back" decoration (capture precedes the AR-04
      path) and nothing is mutated (no `generated/` dir).
  - `pin_rollback_restores_files_but_keeps_created_intermediate_dirs` -
    nested unit two's mkdir fails through unit one's file; clean rollback
    removes both unit paths but `generated/deep/nested` survives - the
    applier never tracks created dirs. Count "rolled back 3 files"
    (2 snapshot entries + 1 created file) locked too.
- Gate: -> `33 passed / 0 failed / 0 ignored / 5 suites` (31 baseline + 2).

## No red-exposed wrong behavior

Every pin passed on its first semantic run: no unlisted wrong behavior was
encountered, so nothing beyond the spec's own claims was pinned. The two
compile-only failures (worker-core coercion typo, scheduler `expect_err`
Debug bound) were harness-shape errors fixed in the test code; no
production line changed.

## Gate record (distilling entry points, digests verbatim)

1. `python tools/testrun.py --distill -- -p ariadne-worker-core` ->
   `21 passed / 0 failed / 0 ignored / 2 suites`
2. `... -p ariadne-scheduler` -> `42 passed / 0 failed / 0 ignored / 2 suites`
3. `... -p ariadne-build` -> `31 passed / 0 failed / 0 ignored / 2 suites`
4. `... -p ariadne-cluster` -> `68 passed / 0 failed / 0 ignored / 8 suites`
5. `... -p ariadne-artifact` -> `58 passed / 0 failed / 0 ignored / 2 suites`
6. `... -p ariadne-transform` -> `33 passed / 0 failed / 0 ignored / 4 suites`
7. `... -p ariadne-lang` -> `45 passed / 0 failed / 1 ignored / 2 suites`
8. `... -p ariadne-validate` -> `33 passed / 0 failed / 0 ignored / 5 suites`
   (captures: target/testrun-capture.txt, overwritten per run)
9. hygiene: `python tools/hygiene_check.py` -> exit 0 (witnessed via
   `python tools/distill.py -- python tools/hygiene_check.py` ->
   `### generic PASS (exit 0)`, capture 173 lines). R1/R2: zero findings.
   Every R3 warn is PRE-EXISTING (not one sits in a line this hand added -
   the three warns inside touched FILES - discovery.rs:320 `c`,
   plan.rs:74 `s`, validate lib.rs:104 `p`, tu_store.rs:54-62 `tmp` - are
   all in untouched round-0 production lines).
10. encoding: `python tools/encoding_guard.py --all` -> exit 0 (silent,
    full-tree witness run post-commit - `commit_scoped.py` stages+commits
    atomically, so `--staged` post-commit only ever sees the empty index;
    the sequencing mirrors the earlier hands. All authored content was
    verified pure ASCII (grep -n '[^\x00-\x7F]') BEFORE the commit; the one
    non-ASCII character this hand had introduced (a U+2192 in a plan.rs pin
    comment) was caught by that self-check and rewritten to words BEFORE
    staging; the remaining non-ASCII hits in touched files are pre-existing
    legitimate `->`/em-dash producer bytes in untouched lines).
    Cluster's gate was re-run after that comment-only fix:
    `68 passed / 0 failed / 0 ignored / 8 suites` (unchanged).
11. commit: single scoped commit via commit_scoped.py (hash in the
    handback; `git show --stat HEAD`).

## Skips

- No platform-conditional arms needed: every injection (blocker file,
  rename-onto-dir, directory-target capture) is deterministic on both
  Windows and Unix; no cfg(windows) gating was required.
- The scheduler "read-only dir" wording was realized as the blocker-file
  injection (disclosed deviation above - same IO-failure class, the
  established cross-platform precedent).

## Out-of-scope observations (report only)

- `format_summary` ignores `target_file`/`problem`/`risk` ENTIRELY (locked
  by the field-set arm) - two distinct plans on one id with equal counts
  summarize byte-identically; fine as a one-line summary, but any UI
  grouping by summary string could collide across plans. No change proposed.
- validate rollback leaves EMPTY intermediate dirs (pinned); the created-dir
  set is not tracked anywhere, so a `verify_restored`-style check that only
  compares files cannot see it - already t4-review-adjacent, added here to
  the deferred-findings candidate list.

## Reports (pin-and-report class; no fixes - shape decisions owed)

- worker-core `edge_destinations` (spec pin-and-report row): Resolved /
  Wildcard ids absent from `path_by_id` drop with NO counter, warning, or
  error - same class as UTA-B6. RECOMMENDATION: surface a dropped-id count
  (+reason) on the closure-walk output so silent under-visit is diagnosable.
- build measure fallback (context for the honest name): in simulation mode
  the reported front-end CPU IS the wall clock, and a TU whose command never
  ran reports exit code 0 - consumers of `format_report` cannot distinguish
  simulated from measured runs. Surfacing (e.g. a `simulated` flag on
  TuMeasurement) is an owner decision, not this hand's.
- validate capture polarity: capture aborts ALL-or-nothing on the first
  bare IO error (no snapshot survives earlier good files) and the error
  names no path - consistent with the t4-review disposition, pinned only.

## Deliberately NOT done (with reasons)

- No production line changed anywhere (pins only; the scheduler fixture
  keeps all round-1 inline suites byte-identical).
- Did not fix the drop-silence, simulation attribution, or capture polarity
  (all report/owner-decision class per the handoff).
- Did not re-pin build's plain deletion ladder (round-1 t6 covers it; R4
  adds only the conflict dimension).
- Touched only the handoff's touch-set: the eight `crates/*/src/**` trees +
  this journal. No round-1 pin file, fixture, or golden artifact modified.
