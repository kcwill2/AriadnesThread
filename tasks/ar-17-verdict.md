# AR-17 Verdict - APPROVED

Handoff: `tasks/ar-17-handoff.md` (T4 fixture paths / T5 polled readiness /
T6 m21 deterministic lock). Landing commit: `55c93c8` (single commit,
reviewed whole; dispatch base `b49ed06`, parent `14abc55` - other lanes'
commits in between belong to their own adjudications).

Reviewer ran every gate fresh on HEAD `55c93c8`; no implementer-run line was
trusted where I could produce my own.

## Gate evidence (reviewer-owned stdout, canonical lines verbatim)

Scoped integration suites (`python tools/testrun.py --distill -- -p ariadne-integration-tests --test <suite>`):

- m10_validation_test: `5 passed / 0 failed / 0 ignored / 1 suites`
- m15_graph_ui_test: `1 passed / 0 failed / 0 ignored / 1 suites`
- m16_traversal_test: `2 passed / 0 failed / 0 ignored / 1 suites`
- m17_dir_nav_test: `1 passed / 0 failed / 0 ignored / 1 suites`
- m18_cache_epoch_test: `2 passed / 0 failed / 0 ignored / 1 suites`
- m18_persistence_incremental_test: `7 passed / 0 failed / 0 ignored / 1 suites`
- m21_run_ui_smoke_test: `1 passed / 0 failed / 0 ignored / 1 suites`

`python tools/encoding_guard.py --all`: zero output = zero violations (exit 0
is the silent-pass signature per `tools/encoding_guard.py:173`
`return 1 if violations else 0`).

`python tools/hygiene_check.py`: blocking rows are exactly the two expected
AR-13 rows in the foreign-lane file:

```
crates/ui/src/server.rs: R2 1172 lines > 1000
crates/ui/src/server.rs:816: R1 fn `api_graph_traversal` is 161 lines (> 100)
```

No R1/R2 row touches any AR-17 file. The AR-17-introduced m15 R1 (104 lines)
is gone from the output - the helper-extraction fix holds.

`python -m unittest discover -s tools -p "*_tests.py"`: `Ran 219 tests in 19.119s` / `OK`.

## Checks adjudicated

1. **Touch-set EXACT.** `git diff --name-only "55c93c8^..55c93c8"` returned
   exactly the 8 declared paths (m10/m15/m16/m17/m18_cache/m18_persistence/
   m21 + journal). `common/mod.rs` untouched - the helper pre-existed, which
   the touch-set explicitly permits ("if missing"). Nothing outside the set.

2. **T4 fixture-path independence.** Diff replaces every CWD-relative
   literal in the touched suites; grep over `tests/integration` finds zero
   remaining `PathBuf::from("../../` forms - only m19/m20 keep the
   manifest-derived form (outside the glob touch-set, declared).
   `common::fixture_path` (common/mod.rs:42-53) resolves identically to the
   old manifest literals (`CARGO_MANIFEST_DIR/../..` + `tests/fixtures/<n>`),
   so the m18_cache/m21 conversions are resolution-neutral.
   **Alien-CWD reproduced by the reviewer**: the freshly built committed
   binary `target/debug/deps/m17_dir_nav_test-bc7420cf42f18b94.exe` executed
   via `python tools/distill.py` (which pins `cwd=REPO_ROOT`, an alien CWD
   where `../../tests/fixtures` resolves outside the repo) ->
   `### generic PASS (exit 0)`, capture:
   `test result: ok. 1 passed; 0 failed; 0 ignored; ...`
   (capture: `target/reviewer-m17-alien-cwd.txt`).
   The pre-fix FAIL was not re-executable under read-only git (finding 3).

3. **T5 sleep -> poll.** All three sites pre-fix vs post-fix compared against
   `55c93c8^` blobs: budgets byte-identical (`0..120` x 250 ms in m15/m16/
   m18_persistence), ready condition identical
   (`status["progress"]["state"] == Some("ready")`), terminal assertions
   context-unchanged in every diff. The change moves the sleep from
   leading to between-attempts only; every loop stays bounded
   (`for _ in 0..120`). No sleep merely shortened, no infinite-poll hazard.

4. **T6 (crux).**
   (a) Committed `provision_basic_repo_copy` copies with
   `copy_dir_all_excluding_artifacts` (skips `artifacts/`) and calls
   `seed_boot_workload` (20 TUs x 400 struct decls, pure ASCII). No stale
   `copy_dir_all` retained (no dual representation).
   (b) Both `if let Some(...)` soft gates are gone; committed code has
   `live_probe.expect("live route probe must fire during the loading
   window...")` (m21:345) and `trace...find(...).expect("trace must contain
   a loading snapshot...")` (m21:353-356), plus the unconditional
   `assert_live_probe_serves_index_gating_graph(&probe)`.
   (c) `server.rs` untouched by this hand: the commit touches 8 files, none
   in `crates/`; `git diff --stat 14abc55..55c93c8 -- crates/ui/src/server.rs`
   is empty -> the mutation revert was byte-exact and no weakening was
   smuggled. HEAD `api_graph_include` (server.rs:276-284) still ends
   `.ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)`. The mutation RED
   panic the journal cites (`m21_run_ui_smoke_test.rs:253 ... left: 200`)
   matches the committed line 253-255 assert
   (`probe.graph_include_status, 503, "graph route must stay gated until
   ready"`) exactly - internal consistency confirmed.
   Non-vacuity by code reading: if the probe never fires, the `.expect`
   panics (test red); if it fires but gating regressed, the 503 assert fails
   (test red). Neither branch can silently skip. The mutation itself could
   not be re-applied by the reviewer (source writes denied) - finding 4.

5. **m15 R1 repair.** Committed `poll_until_ready_status` = 18 lines;
   `test_m15_graph_ui_command` = 88 lines (file lines 33-120). All ten
   assertions present and semantically identical to the pre-fix blob.
   hygiene_check shows no m15 row.

6. **Report-evidence (distillation contract).** Journal gate transcript is
   digest-shaped: every command is a `testrun.py --distill` invocation,
   canonical lines quoted, exit codes and capture paths present, zero raw
   dumps; it correctly disclaims the shared capture file as non-owned under
   concurrent lanes.

## Findings (no blockers)

1. (minor, informational) Pre-fix alien-CWD FAIL was verified by
   consistency, not re-execution: read-only git cannot rebuild the deleted
   revision. Evidence chain accepted: pre-fix blob carries the relative
   literal at m17 line 7 with the cited assert at line 22; the mechanism
   (repo-root `../../` lands outside the repo) is mechanically sound; and
   the reviewer's post-fix alien-CWD PASS proves the property T4 claims.
2. (minor, informational) `hygiene_check.py` process exit code not captured:
   the `$LASTEXITCODE` capture form is refused by the bash allow-list
   (boilerplate refusal `{"permission":"bash","pattern":"*","action":"deny",
   "source":"global"}`). The blocking-row set itself was captured verbatim
   and matches the expected AR-13 baseline exactly.
3. (follow-up, ledger) `tools/distill.py` digest echo crashes on cp1252
   consoles (`UnicodeEncodeError` at distill.py:213) - reproduced by the
   reviewer's own alien-CWD run; verdict line prints before the crash and
   the capture remains authoritative. Pre-existing tooling bug, outside
   AR-17; route to `tasks/milestone_ledger.md` deferred findings.
4. (follow-up, ledger) Pre-existing in-place fixture-pollution class:
   m15/m16/m17 run `run_ui` against the live fixture dir, rewriting
   `tests/fixtures/basic/artifacts/**` in the worktree (journal observed an
   eol-only dirty `artifacts/includes/report.json`). The commit itself
   touches no fixture file (verified in the 8-file list). Outside AR-17's
   touch-set; route to ledger deferred findings.
5. (informational) Mutation re-run not possible for the reviewer (source
   edits denied). Verdict rests on journal RED evidence + independent
   code-reading proof of non-vacuity + byte-exact server.rs integrity, all
   independently confirmed.

## Declared deviations - dispositions

- D1 m21 poll cadence `400 x 100 ms` -> `800 x 50 ms`: **accepted**. Same
  nominal 40 s budget, error text preserved; finer sampling is the
  determinism mechanism and the handoff names m21's poll as the pattern to
  work on. Bounded loop retained.
- D2 m21 assertion-count increase (4 gating asserts + loading-window
  asserts now unconditional): **accepted**. Adjudicated on merit, not
  rubber-stamped: the handoff's objective #3 ("the lock must not be able to
  silently stop asserting") directly conflicts with the blanket
  "assertion count preserved" constraint clause; the change strengthens
  every affected assertion and removes none, which is the deliverable, not
  an adjacent reinterpretation.
- D3 m18_cache/m21 manifest-derived literals converted to `fixture_path`:
  **accepted**. "sibling relative forms" is inside objective 1 and the
  resolution was independently confirmed identical.
- D4 `m16_clang_worker_test.rs` `fixture_root()` left alone despite the
  `m16*.rs` glob: **accepted**. Verified CWD-independent at source
  (`CARGO_MANIFEST_DIR`-derived); the objective targets the CWD-relative
  pattern.
- D5 m19/m20 manifest-derived literals left: **accepted**. Outside the
  declared globs; correct scope discipline.
- D6 m10 pre-existing unused imports left; new `mod common;` dead-code
  warnings follow the m00 precedent: **accepted** (hygiene gate shows no
  blocking row).

## Rejected/declined reviewer commands (recorded verbatim, none blocked a gate)

`Get-ChildItem ...`, `python -c "..."`, a `git status --short`+`echo`
chain, a `--cmdlist` distill probe, and a `$LASTEXITCODE`-capturing
`encoding_guard` rerun each returned:
`The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}`
All expected gates (testrun, hygiene, encoding, distill, git read
show/log/diff, unittest discover) ran successfully.
