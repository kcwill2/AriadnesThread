# AR-21 Execution Results - reviewer gate addendum

Reviewer: ses_ed4bdfc9cffeBJHlaBVbLNXEaZ, 2026-10-11. All lines quoted from
this session's OWN stdout (shared-capture rule; AR-25 lane live). Ground truth
at review start (`checkpoint_guard.py status`): HEAD b7136db, branch
shining-enthusiasm, tracked tree clean, no uncommitted changes to AR-21 paths.

## Scoped gates (fresh, own stdout, verbatim canonical lines)

| # | Contract criterion | Command | Verbatim result |
|---|--------------------|---------|-----------------|
| 1 | lang-cpp default features (re-anchored 155) | `python tools/testrun.py --distill -- -p ariadne-lang-cpp` | `### cargo test PASS (exit 0)` / `155 passed / 0 failed / 0 ignored / 2 suites` |
| 2 | m18 parity UNEDITED, both sides execute | `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` | `### cargo test PASS (exit 0)` / `23 passed / 0 failed / 0 ignored / 1 suites` |
| 3 | feature-on configuration (count parity) | `python tools/testrun.py --distill -- -p ariadne-lang-cpp --features test-resolver` | `### cargo test PASS (exit 0)` / `155 passed / 0 failed / 0 ignored / 2 suites` |
| 4a | m00 pair UNEDITED | `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test` | `### cargo test PASS (exit 0)` / `4 passed / 0 failed / 0 ignored / 2 suites` |
| 4b | m17 cone guard | `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m17_plugin_seam_test` | `### cargo test PASS (exit 0)` / `8 passed / 0 failed / 0 ignored / 1 suites` |
| 5 | strict compile-green, all targets | `python tools/distill.py -- cargo check --workspace --all-targets` | `ok (exit 0, 31 warnings emitted)` |
| 5b | production absence proof | `python tools/distill.py -- cargo check -p ariadne-pipeline` | `ok (exit 0, 0 warnings emitted)` |
| 5c | feature-on lib self-warning-free (D2) | `python tools/distill.py -- cargo check -p ariadne-lang-cpp --features test-resolver` | `ok (exit 0, 0 warnings emitted)` |
| 6 | hygiene (zero blocking regime) | `python tools/hygiene_check.py` | exit 0; printed rows ALL R3 (warn); ZERO blocking rows; `resolver.rs` absent from findings |
| 6b | encoding | `python tools/encoding_guard.py --all` | exit 0, silent (no output) |

All five test-suite baselines byte-equal to the brief's anchors: lang-cpp
155/0/0/2, m18 23/0/0/1, m17 8/0/0/1, m00 4/0/0/2. No drift observed; the
foreign AR-25 landing (2aeae5e..b7136db, cli +4 tests/1 suite) does not move
any scoped anchor above (each run pinned to its own package/suite).

## Legacy-side execution proof (both oracle sides run)

`python tools/distill.py -- cargo test -p ariadne-integration-tests --test
m18_cpp_plugin_parity_test legacy` -> `ok (exit 0, 12 warnings emitted)`;
own capture (read immediately, before clobber):

```
running 6 tests
test m18_cpp_plugin_parity::resolver_parity::resolve_target_outcome_table_matches_legacy_resolver_expensive ... ok
test m18_cpp_plugin_parity::resolver_parity::plugin_closure_matches_vendored_legacy_bfs_expensive ... ok
test m18_cpp_plugin_parity::resolver_parity::resolve_target_outcome_table_matches_legacy_resolver_basic ... ok
test m18_cpp_plugin_parity::resolver_parity::plugin_closure_matches_vendored_legacy_bfs_basic ... ok
test m18_cpp_plugin_parity::external_boundary_parity::resolve_target_outcome_table_matches_legacy_resolver_external_boundary ... ok
test m18_cpp_plugin_parity::external_boundary_parity::plugin_closure_matches_vendored_legacy_external_boundary ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 17 filtered out; finished in 0.02s
```

All 6 IncludeResolver-driven left-side tests green. The same capture shows the
consumer-side deprecation clock live: dated warnings
`use of deprecated struct ariadne_lang_cpp::IncludeResolver: ... dated 2026-10-11`
at exactly `resolver_parity.rs:215`, `:290`, `external_boundary_parity.rs:228`,
`:399` (no blanket suppression).

## Workspace warning attribution (31, exit 0)

Own capture of the all-targets check, classified:
- AR-21 intended dated signal: `ariadne-lang-cpp (lib test) generated 10
  warnings` (unit-test use sites resolver.rs:400/406/412/418/424/430/497/587/654/858)
  + `m18_cpp_plugin_parity_test generated 12 warnings (7 duplicates)` (oracle
  use sites + common dead_code duplicates). All carry the dated AR-21 note;
  oracles are uneditable by contract - warnings ARE the designed signal.
- `ariadne-lang-cpp (lib)` (feature-on under dev-dep unification): NO warning
  summary = zero (D2 confinement confirmed in-workspace, matching the 5c
  standalone check).
- Everything else pre-existing, none from AR-21 paths: ui `unused variable
  gen` x1 (+ lib-test duplicate), cli `unused_mut` x1 (pre-existing class),
  `common/mod.rs` dead_code x20 (duplicates per test target), m10 unused
  imports x2, m6 unread field x1, m7/m23 unused vars x2.
- Production targets (pipeline, clang-worker, cli lib, tool): zero
  deprecation warnings; no production target sees the legacy surface.

## Production compiles-it-out - structural confirmation

`python tools/distill.py -- cargo tree -e features -i ariadne-lang-cpp` ->
`ok (exit 0, 0 warnings emitted)`; own capture (101 lines): the ONLY enable
edge for `ariadne-lang-cpp feature "test-resolver"` is
`[dev-dependencies] -> ariadne-integration-tests`; every production consumer
(pipeline, clang-worker, cli, report, ...) reaches lang-cpp exclusively under
`feature "default"` (default-off confirmed: `test-resolver = []`, no `default`
key in `crates/lang-cpp/Cargo.toml`). Source-level: every legacy item behind
`#[cfg(any(test, feature = "test-resolver"))]`.

## Commit-vs-touch-set (mechanical)

- 7797edb: `A tasks/ar-21-handoff-journal.md` - in set.
- 3e874ab: `M crates/lang-cpp/Cargo.toml`, `M crates/lang-cpp/src/lib.rs`,
  `M crates/lang-cpp/src/resolver.rs`, `M tasks/ar-21-handoff-journal.md` -
  in set; `M tests/integration/Cargo.toml` - the single out-of-set path, the
  declared D1. Cargo.lock NOT touched; no dependency edges added.
- 9fa9a1c: `M tasks/ar-21-handoff-journal.md` - in set.
- `git diff 7797edb 9fa9a1c --name-only -- tests/integration/tests` ->
  `tests/integration/tests/m24_language_identity_test.rs` ONLY, from foreign
  AR-25 commit 2f3b5e7 (not an AR-21 commit); m18/m00 oracle dirs byte-identical
  across the entire review range.

## Not run (explicit, per orchestrator constraint)

- `python tools/testrun.py --baseline`: NOT run. Errata: FILE-mode with no
  baseline file in repo; bare form exits 2. Workspace-wide gate belongs to the
  orchestrator's fresh-session run; AR-25 lane live in this worktree.
- `python -m unittest discover -s tools -p "*_tests.py"`: NOT run - workspace-
  wide; none of the three AR-21 commits touches `tools/` (path lists above).
- `python tools/ui_smoke.py`, `python tools/bench_compare.py`: NOT run - not
  required by this handoff (zero UI / zero hot-path surface; feature-gate only,
  live plugin path untouched, m17 guard green).
- Workspace census anchor (1393/0/13/102 + expected AR-25 foreign drift): NOT
  re-measured for the same lane-collision reason; scoped anchors above cover
  every AR-21-facing package.

## Refused commands (verbatim)

- `git show "3e874ab" -- crates/lang-cpp/src/resolver.rs | Select-Object -Skip 130`
  -> `The user has specified a rule which prevents you from using this specific
  tool call. Here are some of the relevant rules {"permission":"bash",
  "pattern":"*","action":"deny","source":"global"}`. Boilerplate catch-all on a
  piped variant; no gate was refused. Workaround: read working-tree file
  (clean vs HEAD).
