# UTA-T6 Journal - cluster + artifact + build + pipeline pins

Handoff: `tasks/uta-t6-handoff.md`. Base: 34f3f87. Pins only (no behavior
changes); every adjudicated smell gets an honest named pin + report.

## Plan of record

- Crate order: artifact, cluster, build, pipeline (per handoff).
- Artifact test growth lands in NEW modules (tests.rs is at 910 lines; the
  1000-line ceiling forbids growing it): inline `mod bounded_decode_pins` in
  `bincode_options.rs`, inline `mod streaming_write_tests` in `metrics_io.rs`,
  and `manager/uta_t6_pin_tests.rs` for items 2/3/4. Only src edits are
  appended `#[cfg(test)]` test modules + one mod declaration line in
  `manager.rs`. Zero production-body edits.
- Pins are green-by-design by nature; per the RGB protocol a pin that goes
  RED against behavior NOT adjudicated in the spec stops that row.

## Checkpoints

- cluster Green: `67 passed / 0 failed / 0 ignored / 8 suites` (13 new pins,
  one run, no Red detour). Item 5: five frontier path shapes + the two
  adjudicated smells named (`pin_frontier_breaks_at_first_frozen`,
  `pin_frontier_ignores_ownership`). Item 6: merge-anchor/reverse-append and
  transitive-join exact-order pins, duplicate screening-metric later-wins pin,
  check_candidate_safety accept+frozen pair. Item 7: exact cross-SCC emission
  order (acyclic reverse-topological + cycle-then-detached, repeated-run byte
  equal), duplicate-seed pin. Item 10 (REPORT ONLY, no test): planner.rs:477
  rebuilds the full-graph SCC adjacency once per candidate (wrapper `scc::
  compute_symbol_sccs`), O(candidates x all edges); `build_symbol_adjacency`
  + `compute_symbol_sccs_from_adj` exist for reuse but the hot loop does not
  use them. planner pins live in new file `planner/uta_t6_pins.rs` because
  planner.rs is at ~945 lines (ceiling-avoidance per handoff).
- build Green: `28 passed / 0 failed / 0 ignored / 2 suites` (5 new pins,
  one run). All in new `crates/build/src/uta_t6_pins.rs` + one mod decl in
  lib.rs. Pins: `pin_normalize_drops_duplicate_named_context` (adjudicated
  types.rs:97 - loser dropped WHOLE, input-order-first wins), discovery
  precedence root>build>out (three-tier deletion walk), plain-relative-entry
  branch (directory field never enters the FileId), Windows verbatim-
  canonical-root string-fallback branch (pinned via the repo-relative FileId
  with a counterfactual guard), and format_report number boundaries
  0/1,000/100,000/1,000,000 (arrow char referenced by code point, file stays
  ASCII). DIVERGENCE NOTED (no unify, per handoff): cluster/src/plan.rs:73-83
  `format_number` is a second, independent digit-grouping implementation
  (string-reverse) vs build/measure.rs:78-94 (arithmetic); results agree at
  all pinned boundaries, the code is not shared.
- artifact Green: `57 passed / 0 failed / 0 ignored / 2 suites` (17 new pins).
  Items 1-4 shipped: bounded-decode module pins (both variants, slice+reader,
  string+seq, + honest pass-through), streaming-write failure arms (emit-Err
  tmp-removal + rename-fail), check_manifest branches (root mismatch,
  key-absent skip, corrupt json), same_path table, measurement fallback
  ladder, index reuse-stats round-trip, extraction fingerprint "" pin.
- DEVIATION: the first same_path table attempt went RED at manager level:
  `Repository::locate` canonicalizes, so the manager root is `\\?\` verbatim
  and rewriting its slashes (`//?/C:/...`) breaks same_path's string-prefix
  verbatim strip - behavior NOT in the adjudicated table. Resolved without
  stopping the row: the adjudicated table is pinned at the `same_path`
  function itself (all four cosmetic dimensions + sibling control), and the
  verbatim-edge consequence is pinned as a second honest test
  (`pin_verbatim_manager_root_rejects_the_slash_rewritten_spelling`).
  REPORTED for owner decision: separator-cosmetic equality does not survive
  combination with the verbatim prefix; manifests written with forward-slash
  roots against canonicalized Windows roots mismatch.
- Pin honesty note: rename-failure pin asserts the staged tmp file is LEFT
  BEHIND (cleanup arm covers emit-error only) - pin of an admitted wart.

## Successor checkpoint (fresh-session verification + pipeline Green)

- The prior session died uncommitted (empty compaction summary, never
  resumed). This checkpoint comes from the resume-checkpoint bootstrap at
  HEAD 34f3f87 over the same working tree; disk state matched the resume
  report exactly (8 dirty files, 4 untracked pin modules).
- pipeline Green (FIRST-EVER run of the item-9 pin module): `46 passed /
  0 failed / 4 ignored / 9 suites` (6 new pins, one run, no Red detour).
  9a corrupt report.bin behind a fresh manifest -> Ok + Rebuilt + re-persisted
  decodable blob (the `.ok()` swallow contract, stage_load.rs:162-172);
  9b `--lang` filtered index retains both units and writes NO full-fingerprint
  index manifest, the following full run still Rebuilds (filtered run poisons
  nothing); 9c `for_incremental` bails on a unit no registered plugin claims,
  error names `unhandled language` + the offending unit, with an all-claimed
  Ok control; 9d `RunStats::record` replaces per stage + `format_summary`
  empty/filled branches byte-exact; 9e lib.rs `cycle_policy_by_file` falls
  back to `Reject` for an unclaimed language. All five rows were green as
  written; ZERO rows stopped. The 4 ignores are pre-existing bench fixtures
  (load_perf_bench x2, boot_stages_bench x2), untouched.
- SCRATCH FILE: the resume brief listed `crates/artifact/same_path_probe.py`
  for deletion; it is NOT on disk (verified three ways: directory listing,
  glob scan, checkpoint_guard untracked census). Nothing to remove.
- Fresh re-run of all four scoped gates in this successor session; every crate
  matches its recorded baseline: artifact `57 passed / 0 failed / 0 ignored /
  2 suites`, cluster `67 passed / 0 failed / 0 ignored / 8 suites`, build
  `28 passed / 0 failed / 0 ignored / 2 suites`, pipeline `46 passed /
  0 failed / 4 ignored / 9 suites`. H6 total: 41 new pins (17 artifact +
  13 cluster + 5 build + 6 pipeline), zero rows stopped.
- Gate record: hygiene_check exit 0 (all findings R3 warn-only, every one
  pre-existing in files outside this hand's diff); encoding_guard --staged
  exit 0 with the scope pre-staged via the distill entry point for a
  meaningful staged-blob scan; encoding_guard --all exit 0; one scoped
  commit via commit_scoped over the handoff's five globs.
- Closeout rider (base 2a8e9dc, R2): the workspace guard `core_crates_do_not_name_languages` flagged the pin assert message at `crates/pipeline/src/uta_t6_pins.rs:126` for naming a language; reworded that single string literal to "pin: the language filter retains both discoverable units", no other byte touched (`SourceLanguage::Cpp` enum path stays, exempt via neutral vocab).
- Rider gates: Red reproduced (`0 passed / 1 failed`, offender `crates/pipeline:126`); Green pipeline `46 passed / 0 failed / 4 ignored / 9 suites` (baseline unchanged) and guard `1 passed / 0 failed / 0 ignored / 37 suites`; hygiene_check exit 0 (R3 warns all pre-existing); encoding_guard --all exit 0.
