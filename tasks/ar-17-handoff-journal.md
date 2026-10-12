# AR-17 Handoff Journal - Integration fixture paths + polling + m21 deterministic lock (T4/T5/T6)

Implementer: Implementer (RGB-TDD)
Handoff: `tasks/ar-17-handoff.md`
Base at dispatch: `b49ed06` (AR-02 landed).
Final gates run on: HEAD `14abc55` (graph perf P3/P4; other lanes landed during this handoff).

## Incident note (mid-handoff session death + resume)

The first session died on a provider timeout after the m21 determinism work and
five of seven scoped gate runs had completed.
Resume followed `checkpoint_guard.py status` + `git diff --stat -- tests/integration` disk-state
inventory rather than memory (resume-checkpoint discipline).
Inventory found all seven file edits intact on disk and `crates/ui/src/server.rs` clean
(confirming the mutation revert had landed byte-exact before the crash).
One regression surfaced at resume: the pre-crash poll-loop expansion pushed
`test_m15_graph_ui_command` to 104 lines - a NEW R1 hygiene violation introduced by this
handoff (committed version was 99).
Fixed at resume by extracting the readiness poll into `poll_until_ready_status()`;
the m15 suite gate was then re-run green at HEAD `14abc55`.
All seven scoped gates were (re-)run on the resumed tree so every canonical line below
describes the same tree that was committed.

## T4 - CWD-independent fixture paths

`common::fixture_path` already existed in `tests/integration/tests/common/mod.rs`
(derives from `CARGO_MANIFEST_DIR` via `repo_root()`); no helper was added.
Replacements (each file gained `mod common;` + `use common::fixture_path;`, matching the
existing precedent style of `m00_facts_path_parity_test.rs`, which carries the same
unused-helper dead-code warnings without any allow attributes):

- `m10_validation_test.rs`: `setup_fixture()` `PathBuf::from("../../tests/fixtures/basic")` -> `fixture_path("basic")`.
- `m15_graph_ui_test.rs`: `repo_dir` -> `fixture_path("basic")`.
- `m16_traversal_test.rs`: two sites (`spawn_fixture_ui_server`, export test) -> `fixture_path("basic")`.
- `m17_dir_nav_test.rs`: `repo_dir` -> `fixture_path("basic")`.
- `m18_cache_epoch_test.rs`: the `env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/basic")`
  literal pattern appears here; converted to `fixture_path("basic")` (identical resolution).
- `m21_run_ui_smoke_test.rs`: same manifest-derived literal converted to `fixture_path("basic")`.

Deliberately NOT changed:
- `m16_clang_worker_test.rs` `fixture_root()`: already `CARGO_MANIFEST_DIR`-derived with a
  different join form; not the CWD-relative pattern; left untouched to keep the diff minimal.
- `m19/m20` manifest-derived literals: outside the touch-set.
- No fixture file moved or modified.

Red/Green demo (CWD dependence is not observable through `cargo test`, which pins the test
CWD to the package root, so the compiled binary was executed directly via the distiller
from an alien CWD = repo root, where `../../tests/fixtures/basic` resolves outside the repo):

- Pre-fix binary, alien CWD: `### generic FAIL (exit 101)`,
  `test result: FAILED. 0 passed; 1 failed; 0 ignored; ...`,
  panic at `m17_dir_nav_test.rs:22` ("run_ui failed for export").
- Post-fix same binary, same alien CWD: `### generic PASS (exit 0)`,
  `test result: ok. 1 passed; 0 failed; 0 ignored; ...` (verified in the distill capture
  because the distiller's stdout echo died on a console cp1252 print bug - a pre-existing
  tooling quirk, verdict line and capture are authoritative).

## T5 - Fixed 250 ms sleeps -> poll-until-ready

Three sites, all m21-idiom (probe first, sleep only between attempts; assertion meaning and
the 120 x 250 ms budget preserved verbatim; terminal readiness assertions unchanged):

- `m15_graph_ui_test.rs`: leading `sleep(250)` inside the readiness loop removed; at resume
  the loop moved to helper `poll_until_ready_status()` to stay under the 100-line function cap.
- `m16_traversal_test.rs`: `wait_for_ready_status()` restructured poll-first
  (its readiness contract remains characterised by the subsequent graph-route assertions).
- `m18_persistence_incremental_test.rs`: warm-start status loop restructured poll-first.

These loops already polled; the flake class removed is the fixed leading delay (first probe
now happens immediately at t=0, ready is detected without a mandatory 250 ms tax per boot).

## T6 - m21 serve-while-loading / gating determinism

Problem: `LiveProbe` was `Option` and both gating assertions ran behind `if let Some(...)`,
so the lock could silently stop asserting.
Root cause found during this handoff: the tracked `basic` fixture SHIPS a pre-built
`artifacts/` tree, so copying it into the temp boot dir produced a near-instant warm boot;
the loading window was frequently shorter than the 100 ms sample interval.
Two count-assert-only attempts measured it: with 50 ms sampling the probe still missed
(`0 passed / 1 failed` - probe-assert RED), proving the window cannot be trusted to be
observable without forcing it.

Boot seam chosen (test-side only, since no seam exists in cli/ui and staging excludes crates):
`provision_basic_repo_copy()` now (a) skips the fixture's `artifacts/` dir when copying and
(b) seeds a synthetic workload (`seed_boot_workload`: 20 TUs x 400 struct decls, ASCII) so
`run_ui` must run the real 5-stage pipeline before reporting ready.
Measurement with temporary instrumentation (removed before commit):
`DIAG(AR-17): snapshots=14 loading_snapshots=13 probe_fired=true`.
The loading window is now ~1 s of real pipeline work against a 50 ms sampler.

Lock changes in `m21_run_ui_smoke_test.rs`:
- `poll_until_ready`: 400 x 100 ms -> 800 x 50 ms (same nominal 40 s window, error text kept).
- The two `if let Some(...)` blocks became hard assertions:
  `live_probe.expect("live route probe must fire during the loading window ...")` and
  `trace ... .expect("trace must contain a loading snapshot (probe fired during loading)")`.
- Doc comments updated: `LiveProbe` and `provision_basic_repo_copy` no longer promise
  "best-effort" semantics.

Assertion-count deviation (deliberate, handoff-mandated): the four gating asserts and the
loading-window asserts are now unconditional where they were conditional.
That strengthening IS the deliverable ("the lock must not be able to silently stop
asserting"); every pre-existing assertion's meaning is untouched.

## Mutation proof (never committed)

1. Mutation applied locally to `crates/ui/src/server.rs` `api_graph_include`: the
   `ok_or(SERVICE_UNAVAILABLE)` gating replaced by serving an empty payload (200) while loading.
2. `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m21_run_ui_smoke_test`
   -> `### cargo test FAIL (exit 1)` / `0 passed / 1 failed / 0 ignored / 1 suites`,
   panic at `m21_run_ui_smoke_test.rs:253`:
   `assertion left == right failed: graph route must stay gated until ready; left: 200`.
   The lock goes RED when gating is weakened - it cannot be vacuous.
3. Reverted with the byte-safe Edit tool; `git status --short` confirmed
   `crates/ui/src/server.rs` absent from the dirty list (byte-exact revert).
4. m21 re-run after revert: `1 passed / 0 failed / 0 ignored / 1 suites`.

## Deliberately NOT done

- Did not add a boot-delay parameter to `run_ui` (staging limited to tests/integration).
- Did not strip `m10`'s two pre-existing unused imports (`GeneratedUnit`, `FileSnapshot`);
  verified via `git show "HEAD:..."` that both pre-date this change (import-only occurrences).
- Did not touch `m21_scala_semantic_index_test.rs` (no pattern present), `m19/m20`, golden
  fixtures, or byte-locked manifests.
- Did not relocate the m15/m16/m17 in-place fixture-dir runs to throwaway copies: the m15
  server test runs `run_ui` against the live fixture dir, so pipeline runs rewrite
  `tests/fixtures/basic/artifacts/**` in the worktree.
  This session observed `artifacts/includes/report.json` flagged dirty with a CONTENT-EMPTY
  diff (git reported only a LF/CRLF eol warning, zero content lines changed).
  Left unstaged and untouched; flagged here for the reviewer - the in-place pattern is a
  pre-existing fixture-pollution class outside AR-17's touch-set.
- Foreign lane dirt present at commit time (`crates/ui/tests/ui_test/main.rs` +1 line,
  untracked task docs): never staged.

## Gate transcript (canonical lines, verbatim; each run owned its own stdout; capture files
at `target/testrun-capture.txt` are shared with concurrent lanes and were NOT treated as mine)

Scoped runs at HEAD `14abc55` (one run per touched suite binary):

```
python tools/testrun.py --distill -- -p ariadne-integration-tests --test m15_graph_ui_test
1 passed / 0 failed / 0 ignored / 1 suites
python tools/testrun.py --distill -- -p ariadne-integration-tests --test m10_validation_test
5 passed / 0 failed / 0 ignored / 1 suites
python tools/testrun.py --distill -- -p ariadne-integration-tests --test m16_traversal_test
2 passed / 0 failed / 0 ignored / 1 suites
python tools/testrun.py --distill -- -p ariadne-integration-tests --test m17_dir_nav_test
1 passed / 0 failed / 0 ignored / 1 suites
python tools/testrun.py --distill -- -p ariadne-integration-tests --test m18_cache_epoch_test
2 passed / 0 failed / 0 ignored / 1 suites
python tools/testrun.py --distill -- -p ariadne-integration-tests --test m18_persistence_incremental_test
7 passed / 0 failed / 0 ignored / 1 suites
python tools/testrun.py --distill -- -p ariadne-integration-tests --test m21_run_ui_smoke_test
1 passed / 0 failed / 0 ignored / 1 suites
```

Mutation-cycle lines (pre-gate evidence, quoted where section "Mutation proof" cites them):
GREEN baseline `1 passed / 0 failed / 0 ignored / 1 suites`; mutation RED
`0 passed / 1 failed / 0 ignored / 1 suites`; post-revert GREEN
`1 passed / 0 failed / 0 ignored / 1 suites`.

`python tools/hygiene_check.py`: the only blocking rows after the m15 fix are the two
pre-existing AR-13 rows (`crates/ui/src/server.rs` R2 1172 lines; `api_graph_traversal`
R1 161 lines), both in a foreign-lane file this handoff never committed.
The AR-17-introduced m15 R1 row (104 lines) is gone.
R3 rows touching my files (`m10` single-letter `p` x2, `common/mod.rs` `i`) pre-date HEAD.

`python tools/encoding_guard.py --all`: see verdict below.

## Encoding / compile hygiene notes

- All authored content pure ASCII.
- Each new `mod common;` consumer inherits the established ~18 dead-code warnings for
  golden helpers it does not use (exact precedent: `m00_facts_path_parity_test.rs` emits 20);
  no allow attributes added, matching repo precedent.
- Two pre-existing m10 unused-import warnings (`GeneratedUnit`, `FileSnapshot`) confirmed
  present at HEAD before this change; left in place.
