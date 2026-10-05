# V2 Journal - M26 boot-progress-visibility: additive `discover_with_progress` hook

Handoff: `tasks/boot-progress-visibility-v2-handoff.md`
Crate: `ariadne-repo` (single-crate hop). V3 consumes this API next.

## Bootstrap

- Start rev: `f325beef9a1b36ab0ae49af4e4ea1d885a3e6589` (= V1 landing
  `f325bee`, APPROVED per `tasks/boot-progress-visibility-v1-verdict.md`).
  Recorded via `python tools/distill.py -- git rev-parse HEAD`.
- Binding inputs read in order: this handoff (complete), shape sec. 3.1 +
  sec. 4 V2 row, `crates/repo/src/lib.rs`, `crates/repo/src/discovery.rs`,
  `crates/repo/src/types.rs`, `docs/specs/tool-output-distillation.md`.
- V1 verdict findings 6/7 NOT read in depth: this hop touches neither
  `ui.rs` nor `stage_spans.rs` (repo-crate discovery hook per the shape).
  Finding 4 (commit route) applies: direct `commit_scoped.py`, staging via
  explicit distill-wrapped `git add -- <touch-set>`.
- Call-site census (report-only, no switch required): outside `crates/repo`
  every consumer uses `Repository::discover()` or the public free fn
  `discover_repository_files`; both keep byte-identical behavior under the
  delegation. No call site changes in this hop.

## Plan

1. Red: append `progress_hook_tests` mod to `discovery.rs`; add the
   `discover_with_progress` seam in `lib.rs` that compiles but discards the
   hook (delegates to the plain walk). Gate 1 expects tests 2-4 failing at
   runtime (hook count 0), test 1 passing.
2. Green: add private `discover_repository_files_with_progress(root,
   on_hashed)` in `discovery.rs`; `discover_repository_files` delegates with
   a no-op closure; `Repository::discover` delegates to
   `discover_with_progress` no-op. Gate 2 expects 0 failed.
3. Hygiene gate; encoding gate after staging; scoped commit.

## Red (gate 1)

`python tools/testrun.py --distill -p ariadne-repo` -> `### cargo test FAIL
(exit 1)`, canonical `12 passed / 3 failed / 0 ignored / 1 suites`. Failing
list = exactly tests 2-4, each `left: 0` (hook discarded in the seam);
test 1 (identity) green on the stub as the handoff predicted.

## Deviations / decisions

- Test 4 shipped as `test_hook_order_independent_of_path_sort` (the handoff
  ELSE branch): unreadable-but-listable files are not cheaply expressible
  via std::fs on Windows (no chmod; icacls is a process call, not a unit
  test). The fallback locks hook-fires-while-walking + path-sorted result.
- Gate 1 first invocation used `; echo EXITCODE` and was allowlist-refused
  (boilerplate catch-all deny; compound forms are the documented V1 finding
  1 family). Re-ran verbatim; the digest header carries the exit code.

## Green (gate 2)

`python tools/testrun.py --distill -p ariadne-repo` -> `### cargo test PASS
(exit 0)`, canonical `15 passed / 0 failed / 0 ignored / 2 suites`.
Threading: `discover_repository_files_with_progress` (pub(crate), not
re-exported) carries the hook; `discover_repository_files` delegates with a
no-op closure; `Repository::discover` delegates to `discover_with_progress`
no-op. Walk/skip rules, sort, and all pre-existing tests untouched.

## Refactor pass

`discover_repository_files_with_progress` is ~72 lines - well under the
100-line ceiling, so the named `record_for_path` extraction was NOT needed
(handoff says extract only if threading pushes past ~90). No duplication
beyond the one-line delegations, which ARE the contract. No further edits.
