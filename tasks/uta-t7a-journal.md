# UTA-T7a Journal - cli error-swallow and residual-arithmetic fixes

Implementer: H7a (uta-t7a-handoff.md). Base: main @ e95a6d5 (post UTA-T6, clean).

## Path deviation (pre-red, declared)

The handoff touch-set names `crates/cli/src/transform.rs`,
`crates/cli/src/measure.rs`, `crates/cli/src/discover.rs`; those modules live
at `crates/cli/src/commands/` (the audit spec line cites - transform.rs:230-233,
measure.rs:11, discover.rs:87 - all resolve inside `commands/`). Same four
files the fixes name; no scope widening. The commit gate's `--scope` globs are
adjusted to the real paths for the same reason. optimize.rs needs no
adjustment.

## Design notes

- UTA-B8 (fix 1): `residual_collateral_load` takes the committed plans
  themselves; the commit loop tracks `committed_plans` (Vec of clones) instead
  of a bare count. The handoff parenthetical says current code "uses plan1
  only" for a middle-failure; the actual buggy value is the first-N prefix
  (plan1+plan2) - the Red assertion pins the correct value (plan1+plan3)
  either way.
- UTA-B7a (fix 2): serde failure and write failure both propagate via `?`
  (Err chosen, per handoff). Red forces the write-failure leg
  (`report.json` occupied by a directory); the serde leg is unfakeable for
  this report shape (Duration serializes fine) and rides the same `?`.
- UTA-B7b (fix 3): `load_plan_for_action` replan failure now chains its own
  root cause (anyhow `with_context` over the replan error, load error text
  embedded); replan-success keeps the reload-the-replanned-artifacts path
  unchanged. Red drives the real replan (no seam added): artifacts dir with
  no plans.bin + a vanished replan repo path -> plan_extractions fails fast
  in `Repository::locate` ("Failed to locate repository path").
- UTA-B7c (fix 4): both `discover().ok()` sites propagate the error loudly.
  What discover.rs:87 gates: (a) BuildDiscoverer's `with_repo_discovery`
  catalog view, (b) the file-hash input list for `Fingerprint::inputs` that
  feeds `contexts_fingerprint` - the swallow saved a fingerprint computed
  over an EMPTY file list while reporting Ok. Deterministic failure
  scenario (Windows-gated): an analyzable file opened `share_mode(0)` makes
  repo discovery's hashing fail (ERROR_SHARING_VIOLATION) - upstream
  discovery swallows only read_dir errors, not file-read errors. Pre-fix
  observation matches the audit: measure returns Ok with zero TUs
  (shaper shapes zero records; `measure_all([])` is Ok) and
  `discover_build_contexts` returns Ok over the empty-hash fingerprint.
- Grep-lock sweep: the only source-text locks touching my files are
  transform.rs's own C1 lock (`pub fn apply_plan_with` /
  `pub struct ApplyTransactionReport`) and extraction_apply.rs test 11
  (`apply_plan_with(` exactly 2x in transform.rs). All fixes leave those
  texts untouched. No include_str! locks reference optimize/measure/discover.

## Checkpoints

- [x] Red run 1 (all four items) - digest + failing list
  - lib: `136 passed / 3 failed / 0 ignored / 1 suites`
    (capture target/testrun-capture.txt, 214 lines). Reds:
    residual=730 vs expected 640 (prefix assumption, exact arithmetic shown);
    save_optimization_report returned Ok over an un-writable target;
    load_plan_for_action chain held only "Failed to read plans.bin" - replan
    cause erased.
  - integration target `--test uta_t7a_discovery_error_test`:
    `0 passed / 2 failed / 0 ignored / 1 suites`. Both failed at the
    expect-Err line: measure AND discover_build_contexts returned Ok over
    the failed discovery - the audit's exact symptom, observed.
  - cargo stops at the first failing test target; the two legs were captured
    as separate scoped runs. Two compile-harness fixes preceded the red
    (missing `use` lines in the new transform test), no assertion touched.
- [x] Green 1: fix 1 residual arithmetic - commit loop pushes `committed_plans`
  (plan clones); `residual_collateral_load` now takes the committed slice;
  test re-pinned to the same expected 640 plus all-committed (460) and
  zero-committed (1000) sanity pins. Red witness kept in the test comment:
  the old count-based call returned 730.
- [x] Green 2: fix 2 save_optimization_report - serde + write both via `?`;
  doc comment records the "Ok means written" contract. Serde-failure leg is
  unfakeable for this report shape (Duration serializes fine) - rides the
  same `?`, no synthetic-serde test invented.
- [x] Green 3: fix 3 load_plan_for_action - replan failure chained via
  `with_context` over the replan error, load error text embedded; replan
  success path byte-equal behavior (reload picks up replanned artifacts).
- [x] Green 4: fix 4 discovery propagation - measure.rs and discover.rs both
  `discover().with_context(...)?` + unconditional `with_repo_discovery`;
  the Option dance and `unwrap_or_default()` hash fallback are gone.
  discover.rs:87 gates (a) BuildDiscoverer catalog view, (b)
  Fingerprint::inputs hash list feeding contexts_fingerprint - journal above.
- [x] Refactor review: no changes needed (files 758/508 lines, functions
  small, no duplication introduced); green run stands.

## Final gate digest (verbatim)

- Green (all cli targets): `147 passed / 0 failed / 8 ignored / 8 suites`
  (capture target/testrun-capture.txt, 243 lines). All five new tests named
  ok in the capture; 8 ignores are the pre-existing bench tests. Frozen
  locks (transform.rs C1, extraction_apply test 11 `apply_plan_with(` == 2x,
  lib test_handle_measure) all pass in the run.
- Deviations: (1) commands/ path deviation - declared at top, commit scopes
  mirror real paths. (2) Windows-only gating of the UTA-B7c integration
  tests (share_mode injection is a Win32 semantic); CI runs on this host.
  (3) Handoff said the prefix bug "uses plan1 only" - actual pre-fix value
  is the first-N prefix (plan1+plan2 reduction, 730); assertion pins 640
  either way. (4) No replan-seam injection in fix 3 - the handoff's "inject
  failing replan seam" served as the suggested means; the red drives the
  real replan with a vanished repo path (fails fast in Repository::locate),
  keeping the fix smaller. (5) New integration test file lives in
  crates/cli/tests/ - explicitly allowed by the touch-set.
- Deliberately NOT done: measure.rs:28 `load_load_artifacts().ok()` (documented
  artifact-absence fallback, not a discovery swallow - out of the handoff's
  named sites); repo-crate `Err(_) => continue` walk tolerance
  (crates/repo - outside touch-set, logged as out-of-scope observation).
- Out-of-scope observations: (a) `discover_repository_files` treats an
  unreadable/FILE root as an empty Ok walk (read_dir Err => continue) - the
  empty-fingerprint class can also be neutralized upstream there;
  (b) `lib.rs` test_handle_measure constructs repos manually in temp - unaffected.
