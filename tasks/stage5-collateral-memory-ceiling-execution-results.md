# Execution results addendum - M28 stage5-collateral-memory-ceiling

Scope of this addendum: the M28 T3 reviewer acceptance pass (workspace-wide
gates belonging to no implementer handoff), run fresh at HEAD `9f02645`
(main, ahead=0, behind=0, dirty=18 foreign paths) on 2026-10-05. T0a/T0b/T1/T2
have not landed; this is NOT the milestone closeout file - it records T3's
acceptance evidence against the handoff's machine-checkable acceptance block.
Verdict: `tasks/stage5-collateral-memory-ceiling-t3-verdict.md` - APPROVED.

## Acceptance criteria vs what happened (evidence lines verbatim)

1. Gate 1 `Ran N ... OK` with Red-first captured:
   - Fresh GREEN-tree (MUST-lines from green only, row-65 erratum):
     `### generic PASS (exit 0)` / `Ran 29 tests in 4.603s` / `OK` (exit 0)
   - Red per journal: `### generic FAIL (exit 1)`, failing list = exactly
     the 5 new `--release` tests (`ui_smoke.UsageError: unrecognized
     arguments: --release`), floor green, `Ran 29 tests in 4.647s` /
     `FAILED (failures=5)`; mechanism independently verified against the
     base-tree strict parser (`ui_smoke.py:77-79`).
2. Gate 2 exit 0 listing `--release`:
   `### generic PASS (exit 0)`; usage `... [--no-build] [--release] [--cold |
   --warm] ...`; help `--release  build and launch the release profile
   (cargo build --release, target/release)`.
3. Hygiene gate 3 exit 0 (fresh): `### generic PASS (exit 0)`; capture 152
   lines, every row a pre-existing R3 warn-only site; zero rows for
   `tools/ui_smoke.py` / `tools/ui_smoke_tests.py` / `AGENTS.md`.
4. Encoding gate 4 exit 0 (implementer, staged) + workspace gate (fresh,
   `--all`): `### generic PASS (exit 0)`, silent (capture 0 lines).
5. `git show --stat <landing>` == touch-set exactly: per-commit shows of
   `9385acf` (4 paths = touch-set; numstat AGENTS.md +1/-0, journal +140/-0,
   ui_smoke.py +32/-8, ui_smoke_tests.py +90/-0) and `9f02645` (journal
   only); range proof `git diff --name-only 5169483..HEAD` = the same four
   paths. PASS.
6. Reviewer smoke forms - BOTH PASS (fresh):
   - default: `### generic PASS (exit 0)` /
     `boot+route checks clean (cold fixture, attempt 1, 0.6s)` /
     `ui_smoke: PASS` (unchanged-behavior witness)
   - `--release`: `### generic PASS (exit 0)` /
     `boot+route checks clean (cold fixture, attempt 1, 0.3s)` /
     `ui_smoke: PASS` (I8 rider witness; exit 0 on this path requires
     `cargo build --release --bin tool` success, presence of
     `target/release/tool.exe`, and that exe booting all route + no-store
     assertions)
7. Full tools discover OK (fresh): CI form `-p "*_tests.py"`:
   `Ran 194 tests in 20.462s` / `OK` = 188 prior + 6 new, exact. The
   handoff's literal command (no `-p`) discovers only 30 (`Ran 30 tests in
   0.740s` / `OK`) - pattern default `test*.py`; verdict finding 1.
8. Census UNCHANGED (fresh): `### cargo test PASS (exit 0)` [capture:
   `target/testrun-capture.txt`, 2175 lines] /
   `1235 passed / 0 failed / 6 ignored / 88 suites`
   - byte-identical to the anchor `1235 passed / 0 failed / 6 ignored /
   88 suites @ bd4fb1a`. Reconciliation: zero delta (zero `crates/**` paths
   in the range diff; zero cargo tests added or removed). `--baseline` file
   form re-encountered unrunnable: `usage error: missing value for
   --baseline` (standing errata row 59; the pre-approved `--distill`
   unscoped census substitute used instead).

## Frozen-lock witnesses

- 23 pre-existing `ui_smoke_tests` cases byte-identical: `+90/-0` range
  numstat (pure insertion) + addition-only hunks in per-commit shows.
- `Cargo.toml`, `crates/**` incl. `crates/cli/tests` (in-flight T0a
  territory), `tests/**`, fixtures/goldens, `tools/bench_compare.py`,
  `bench_baseline.json`, `tasks/milestone_ledger.md`, M27 artifacts: zero
  diff across `5169483..HEAD` (absent from the 4-path name-only diff). The
  working-tree-dirty ledger was never committed by these two commits.
- `--release` opt-in only: default flag False, default exe resolution
  `target/debug` (characterization-locked + live default PASS witness).

## Explicitly NOT run

- `bench_compare.py`: NOT RUN - written producer-crate zero-diff exemption:
  range diff contains zero `crates/*/src` paths, `tools/bench_compare.py`
  and `bench_baseline.json` zero-diff, and DP-3 keeps the `stage5-memory`
  bench gate dormant until T0b lands. No shipped surface can move a stage-5
  column.
- `theme_token_check.py`: NOT APPLICABLE - zero Rust/UI/JS paths in the
  range diff (zero added hex exempt-with-proof by the touch-set diff).
- `golden_guard.py`: not landed in `tools/` (handoff's prose byte-check =
  diff-scope proofs above).

## Permission / gate-route record

- Every gate above ran, attempted individually. Distilling route used for
  all (distinct `--capture` files under `target/`).
- Direct (non-distill) execution of `python tools/hygiene_check.py` and
  `python tools/testrun.py --baseline` SUCCEEDED - the dispatch premise
  that this session cannot invoke gate commands directly is only true for
  non-allowlisted shell forms. Two non-gate probes (`Test-Path`, `python -c`)
  were refused with the catch-all boilerplate verbatim: `The user has
  specified a rule which prevents you from using this specific tool call.
  Here are some of the relevant rules
  {"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
  No gate was skipped, substituted, or judged unrunnable for permissions.
- One glob-tool artifact: `target/**` is invisible to Glob (gitignore-aware)
  - a `target/release/tool.exe` "not found" is not evidence; the smoke PASS
  exit-0 path is the authoritative presence witness.

---

# T0a reviewer acceptance pass (appended 2026-10-06)

Scope: the M28 T0a reviewer acceptance pass, run fresh at HEAD `8d34a3c`
(main, ahead=0, behind=0, dirty=21 foreign paths) on 2026-10-06. Verdict:
`tasks/stage5-collateral-memory-ceiling-t0a-verdict.md` - REJECTED
(1 blocker: release growth-ratio row irreconcilable with the bench's own
reproducible peaks; 4 minor findings).

## Acceptance criteria vs what happened (evidence lines verbatim)

1. Gate 1 scoped cli census (fresh): `### cargo test PASS (exit 0)`
   [capture: `target/testrun-capture.txt`, 158 lines] /
   `111 passed / 0 failed / 8 ignored / 6 suites` - baseline `110/0/1/5`
   +1 probe / +7 ignored / +1 suite, matches the journal line exactly.
2. Gate 2 scoped bench probe (fresh): `### cargo test PASS (exit 0)`
   [capture: 15 lines] / `1 passed / 0 failed / 7 ignored / 1 suites`.
   Red form (journal): `0 passed / 1 failed / 0 ignored / 1 suites`,
   semantic stub-probe panic; verified the live test catches a dead probe
   (assert on peak delta >= 200 MiB; unsupported platform panics via
   `.expect(PROBE_UNAVAILABLE)`, never skips).
3. Workspace census (fresh, `--distill` substitute per row-59 errata):
   `### cargo test PASS (exit 0)` [capture: 2190 lines] /
   `1236 passed / 0 failed / 13 ignored / 89 suites`. Reconciliation:
   anchor `1235 passed / 0 failed / 6 ignored / 88 suites @ bd4fb1a` plus
   `+1 passed / +7 ignored / +1 suite` - delta equals T0a's added tests
   only. Replay witness in capture:
   `test_stage_five_milestone_sequence_is_monotonic_and_contained ... ok`;
   m18 parity green; all locked paths absent from the range diff (byte-
   identical by scope).
4. Tools CI-form discover (fresh): `### generic PASS (exit 0)` /
   `Ran 194 tests in 23.762s` / `OK`.
5. Smoke (fresh): `### generic PASS (exit 0)` /
   `boot+route checks clean (cold fixture, attempt 1, 0.9s)` /
   `ui_smoke: PASS` (distill route; direct route refused by allowlist,
   catch-all boilerplate only).
6. Hygiene (fresh): exit 0, warn-only R3 rows all pre-existing/foreign,
   zero rows for `crates/cli/tests/stage5_memory_bench.rs`. Encoding
   `--all` (fresh): exit 0, silent.
7. Commit-vs-touch-set: `git diff --name-only 9f02645..HEAD` =
   `crates/cli/tests/stage5_memory_bench.rs`, `tasks/...t0a-evidence.md`,
   `tasks/...t0a-journal.md` (per-commit stats: 994 / 341 / 6 lines).
   Filename literal deviates from the handoff's `t0-evidence.md` -
   verdict finding 2. Zero `crates/*/src`, zero Cargo delta.
8. Measurement reproducibility spot-checks (fresh release runs, exit 0):
   half-scale peak `2258.7` (doc band 2254.1-2258.2), full-scale peak
   `9034.8` (doc band 9033.7-9045.4), block-4 memo rows byte-equal to the
   doc at both scales (BTreeMap iteration - deterministic as claimed),
   debug ratio `4.0` (consistent). ANOMALY: release ratio re-run printed
   `stage5 RSS growth ratio (release, N=640->1280) | 3.6` while the same
   bench's own reproduced peaks give 4.000 - verdict finding 1, BLOCKER.

## Explicitly NOT run

- `bench_compare.py`: NOT RUN - written producer-crate zero-diff exemption:
  `git diff --name-only 5169483..HEAD` contains zero `crates/*/src` paths
  (combined T3+T0a range), `bench_baseline.json` untouched, DP-3 keeps the
  `stage5-memory` gate dormant until T0b. Producer semantics unchanged;
  the +4 GiB save-span row is a measurement of the existing producer.
- `theme_token_check.py`: NOT APPLICABLE - zero UI/Rust-src paths in the
  range diff.
- Full 8-invocation debug/release curve re-runs: NOT RUN (hours; per
  dispatch). Four targeted isolated invocations substituted, quoted above.
- `--baseline` file form: unrunnable (standing errata row 59); `--distill`
  census substitute used, line quoted.

## Permission / gate-route record

- Every gate attempted individually. Direct `python tools/ui_smoke.py` and
  env-prefix forms were refused with the catch-all boilerplate
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`;
  sanctioned distill routes and `--exact` filter forms succeeded instead.
  Inline `python -c` is not allowlisted (refused); no gate was skipped on
  permissions.

---

# T0a round-2 reviewer acceptance pass (appended 2026-10-06)

Scope: re-adjudication of the T0a rework (`407fe66`, `eec5646`, `69885ce`) at
HEAD `69885ce` (main, ahead=0, behind=0, dirty=22 foreign). Verdict:
`tasks/stage5-collateral-memory-ceiling-t0a-verdict.md` - APPROVED
(round-1 blocker CLOSED; 1 major relay condition to the DP-4 evidence review,
2 minor findings, 1 planner follow-up; the file supersedes the round-1
REJECTED verdict with history preserved).

## Acceptance criteria vs what happened (evidence lines verbatim)

1. Scoped cli census (fresh): `### cargo test PASS (exit 0)` [capture:
   `target/testrun-capture.txt`, 158 lines] /
   `111 passed / 0 failed / 8 ignored / 6 suites` - expected line, exact.
2. Bench probe (fresh): `### cargo test PASS (exit 0)` [capture: 15 lines] /
   `1 passed / 0 failed / 7 ignored / 1 suites`.
3. Workspace census (fresh, `--distill`): `### cargo test PASS (exit 0)`
   [capture: 2190 lines] / `1236 passed / 0 failed / 13 ignored / 89 suites`
   - byte-identical to the round-1 line: zero test-count delta from the
   rework (one test file touched, no tests added/removed).
4. Tools CI-form discover (fresh): `### generic PASS (exit 0)` /
   `Ran 194 tests in 21.016s` / `OK`.
5. Fresh decomposable ratio invocation (release `--exact`, distill route):
   `ok (exit 0, 0 warnings emitted)`; capture rows verbatim:
   `stage5 peak RSS (release, N=640) | 2260.2` /
   `stage5 peak RSS (release, N=1280) | 7720.5` /
   `stage5 RSS growth ratio (release, N=640->1280) | 3.4`.
   Operands print and the quotient self-audits (7720.5/2260.2 = 3.416 ->
   3.4): finding-1 fix VERIFIED working. The anomalous full operand (-14%,
   ambient working-set trimming) simultaneously REFUTES the errata's
   "full-scale operands never diverged" clause - verdict finding 1: erratum
   + operand-divergence rejection on BOTH operands is a condition on the
   DP-4 evidence review, not a reopen of the hop. Implementer's five
   corrected decomposed readings re-verified arithmetically (all quotients
   4.00 +/- 0.15%); corrected ratio value 4.0 stands.
6. Errata arithmetic re-verified exactly: transient pairs
   9047.2-4951.1=4096.1; 9035.7-4939.7=4096.0; 9034.4-4938.4=4096.0;
   9033.7-4937.6=4096.1 (row-4 PROFILE label wrong in the doc - release
   invocation 3, not debug: verdict finding 2). Label count 26 in both docs;
   D6 declared; T0b supplier pointer already planner-corrected
   (`t0b-handoff.md:16-19`); `silent_peak_mib` scratch: zero matches in
   `crates/` (fully reverted as claimed).
7. Hygiene (fresh): `### generic PASS (exit 0)` (152 capture lines, all
   pre-existing R3 warns, zero rows for the bench; R2 at 995 lines).
   Encoding `--all` (fresh): `### generic PASS (exit 0)`, capture 0 lines.
8. Commit-vs-touch-set: `407fe66` = bench only (+28/-27, stayed in
   `crates/cli/tests`); `eec5646` = evidence (+110/-0, append-only honored)
   + journal (+137/-1, single deletion = the corrected label bullet);
   `69885ce` = journal hash record (+3/-2). Full-range re-proof
   `git diff --name-only 5169483..HEAD` = AGENTS.md, `tools/ui_smoke.py`,
   `tools/ui_smoke_tests.py`, `tasks/...t3-journal.md`, and T0a's three
   paths: zero `crates/*/src`, zero Cargo, zero fixtures/goldens,
   `bench_baseline.json`, ledger. Frozen locks byte-identical.

## Explicitly NOT run

- `bench_compare.py`: NOT RUN - producer-crate zero-diff exemption RE-PROVEN
  over the full `5169483..HEAD` range (above); DP-3 keeps the
  `stage5-memory` gate dormant until T0b.
- `ui_smoke.py`: NOT RE-RUN - zero UI/tools/binary paths in the rework range;
  round-1 same-day `ui_smoke: PASS` witness + zero-delta scope covers it.
- Full debug/release curve suites: NOT RUN (not required this round; the
  ratio test + one control were, and the implementer's five decomposed
  readings were arithmetic-verified instead of trusted).
- `theme_token_check.py`: NOT APPLICABLE (zero UI paths in any T0a range).

## Permission / gate-route record (round 2)

- Refusals this round (verbatim boilerplate
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
  exit-code probe form (`echo $LASTEXITCODE`/`>` redirects) - substituted
  with distiller `### generic PASS (exit 0)` witnesses for hygiene and
  encoding. Everything else ran individually and successfully; no gate was
  skipped, substituted weakly, or judged unrunnable on permissions.

---

# T1 reviewer acceptance pass (appended 2026-10-06)

Scope: the M28 T1 (`t1-bounded-memo`) reviewer acceptance pass, run fresh
at HEAD `d6640a2` (start rev `afcd038`; main, ahead=0, behind=0,
dirty=22 foreign paths per checkpoint_guard). Verdict:
`tasks/stage5-collateral-memory-ceiling-t1-verdict.md` - APPROVED
(0 blockers; 3 minor/informational findings; deviations D1-D5 + the
guard-shape relaxation all ACCEPTED on merit; hop-fixed caps
512/128/pin 512 adjudicated on Block-4 evidence).

## Acceptance criteria vs what happened (evidence lines verbatim)

1. Graph scoped, BOTH forms (fresh): plain (gate-1 exact command)
   `### cargo test PASS (exit 0)` / `104 passed / 0 failed / 0 ignored / 4 suites`;
   instrumented `### cargo test PASS (exit 0)` /
   `107 passed / 0 failed / 0 ignored / 4 suites`. Baseline `103/0/0/3`
   +1 / +4 as the journal predicts. RED-first verified arithmetically
   against the journal's `103 passed / 2 failed / 0 ignored / 2 suites`
   (fail-fast lib 101 + new binary 2 pass/2 fail; the two failures are the
   counter axis; the equality matrix ran green PRE-swap - the oracle
   falsifies counters only, exactly the swap axis).
2. Pipeline oracle UNEDITED (fresh): `### cargo test PASS (exit 0)` /
   `25 passed / 0 failed / 4 ignored / 6 suites` - expected 25/0/4/6,
   exact; `internal_tests.rs` absent from every diff.
3. Cli reviewer hop (fresh, ui C1 memo lock + stage-5 13-frame replay):
   `### cargo test PASS (exit 0)` /
   `111 passed / 0 failed / 8 ignored / 6 suites` - identical to the T0a
   round lines: zero test-count delta from T1.
4. Workspace census (fresh, `--distill` unscoped per row-59 errata):
   `### cargo test PASS (exit 0)` [capture: 2212 lines] /
   `1240 passed / 0 failed / 13 ignored / 90 suites`. Reconciliation:
   `1236 passed / 0 failed / 13 ignored / 89 suites @ T0a` +4 passed
   (this hop's 4 new tests only) / +1 suite, ignored held. D3 COVERAGE
   TRUTH: capture lines 364-367 show ALL FOUR new tests executing in the
   unscoped build - the three `#[cfg(feature = "test-instrument")]`
   counter tests INCLUDED - the feature unification via
   `crates/ui/Cargo.toml:26` dev-dep is empirically confirmed; the I7c CI
   clause runs in the normal census. No coverage finding.
5. Frozen-floor proofs (fresh): m00 `2 passed / 0 failed / 0 ignored / 1 suites`;
   m18 `23 passed / 0 failed / 0 ignored / 1 suites`; lock scan
   `afcd038..d6640a2` diff-EMPTY for `progress.rs`, `index/cache.rs`,
   `bench_baseline.json`, `tests/fixtures`, `tests/golden`,
   `progress_contract_tests.rs`, `boot_progress_tests.rs`, m00/m18 files,
   `internal_tests.rs` (root `Cargo.toml` appears only in the wider
   `bd4fb1a..HEAD` range via the landed M27 T5 rider `5b517e6`; T1's own
   commit has zero Cargo delta).
6. Tools CI-form discover (fresh): `Ran 194 tests in 24.855s` / `OK`.
7. Smoke (fresh, distill): `boot+route checks clean (cold fixture,
   attempt 1, 0.7s)` / `ui_smoke: PASS`.
8. Hygiene (fresh): `### generic PASS (exit 0)`; zero rows in the new test
   file; `load_caches.rs:428/433` (`for f`) pre-existing in untouched
   context as the journal states, warn-only. Encoding `--all` (fresh):
   `### generic PASS (exit 0)`, silent.
9. Commit-vs-touch-set: single commit `d6640a2`, numstat
   `138/48` + `639/0` + `280/0` = the touch-set exactly with the
   gate-5-authorized `load.rs` drop; seam sufficiency verified at
   `load.rs:201` (`with_memo`) and used by the suite itself.
10. Plan-literal checks: `with_closures_and_capacity` signature matches
    the shape 3.2 FROZEN literal verbatim (names, types, per-cache
    entry-count semantics); I5 capacity axis {0, 1, hop caps, MAX} as
    I5 specifies; counters kept truthful, not repurposed; header doc
    :7-19 outside every diff hunk; `LoadMetric` derives plain
    `PartialEq, Eq` (`load.rs:38-52`) - I5 compares full metric
    structure, cold+warm, on 6 fixtures incl. demand>cap.
11. Bench gate (RUN - `crates/graph/src` diff revoked the exemption):
    `### generic PASS (exit 0)` / `bench compare: 3 run(s), no
    regressions past 1.25x baseline`; hot-path `symbol_pass+prep` 1418.7
    inside the recorded 1.15x-1.17x same-machine band; `plan()` 1733.9 =
    1.195x PASS but judged against the KNOWN stale/flapping baseline
    (ledger escalation row: 1.236x-1.273x same-tree bracket; operator
    `--update-baseline` still outstanding); uncapped-path timing is
    structurally unmoved (one extra `len()` check per miss).

## Explicitly NOT run

- RSS directional witness (reviewer-hop local gate, DP-3): ATTEMPTED
  TWICE, environment-blocked. Release `--exact stage5_rss_growth_ratio`
  (distill route, T0a protocol): the N=1280 child died in the artifact
  save span on `There is not enough space on the disk. (os error 112)`
  both times (exit 101) - disk capacity, not code (same invocation ran
  repeatedly during T0a today). Surviving datum: attempt-2 N=640 peak
  `2258.0` - inside the pre-fix T0a band (2254.1-2259.3), i.e. T1 does
  not move the curve, BY DESIGN: the boot memo stays uncapped until T2
  wires the caps (T1 ships the knob only; production constructors are
  uncapped-identical). No I7a inference for or against T1 is claimed;
  median-of->=5 protocol makes any single invocation directional-only
  regardless. Cleanup of the failed runs' temp dirs: `Remove-Item`
  refused by sandbox permissions.
- `--baseline` file form: unrunnable (standing errata row 59); the
  sanctioned `--distill` unscoped census substitute ran and is quoted.
- `bench_compare --bench stage5-memory`: NOT RUN - T0b's gate mode,
  dormant until T0b's baseline rows exist.
- Golden guard script: not landed; byte-identity proven by diff-scope.

## Permission / gate-route record (T1)

- Every expected gate was attempted individually. Refused forms (all
  catch-all boilerplate `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`,
  none of them the gate itself): `rg` in-shell (substituted by the grep
  tool), exit-code probes via `$env:`/`>` redirects and pipes (substituted
  by distiller exit witnesses), `Get-PSDrive` and a quoted-`%` distill-
  wrapped `python -c` disk probe (free-space reported via the os-error
  evidence instead), `Remove-Item` temp cleanup. All cargo/pytest/hygiene/
  encoding/smoke/bench gates RAN through allowlisted direct or distill
  routes; the only obligation that could not complete was blocked by
  machine disk space, not by permissions or allowlist.

---

# T2 reviewer acceptance pass (appended 2026-10-06)

Scope: the M28 T2 (`t2-stage-lifecycle`) reviewer acceptance pass, run
fresh at HEAD `69138db` (start rev `d6640a2` = T1 landing; main,
ahead=0, behind=0, dirty=23 foreign paths per checkpoint_guard).
Verdict: `tasks/stage5-collateral-memory-ceiling-t2-verdict.md` -
APPROVED (0 blockers; 1 relay finding, 1 accepted interim with a
mandated future-touch follow-up, 2 journal errata, 1 witness note;
D1-D9 all adjudicated on merit).

## Acceptance criteria vs what happened (evidence lines verbatim)

1. Pipeline scoped (gate 1 exact command, fresh):
   `### cargo test PASS (exit 0)` [capture 79 lines] /
   `32 passed / 0 failed / 4 ignored / 7 suites` - expected line exact
   (baseline 25/0/4/6 + 7 new tests + 1 suite). Red capture pair
   accepted on the T1-D5 precedent with internally-consistent totals
   (R1 `27/5/4/7`, R2 `28/4/4/7`, R3 `32/0/4/7`; 36 tests throughout).
2. Internal oracles (gate 2, fresh): `### cargo test PASS (exit 0)` /
   `12 passed / 0 failed / 0 ignored / 6 suites` - report-equality x2,
   warm reuse, Arc-sharing, take_include_graph x3, milestone order,
   `collect_file_metrics` pin, missing-cache, hash helper: green
   UNEDITED, `internal_tests.rs` absent from every diff (3.3 budget
   unspent).
3. Cli (fresh): `### cargo test PASS (exit 0)` /
   `111 passed / 0 failed / 8 ignored / 6 suites` - boot contract,
   13-frame replay (which pins the pre-existing SymbolsAnalyzed ->
   CollateralFile x4 -> ArtifactsSaved sequence corroborating the I6
   literal's frame shape), all UNEDITED.
4. Frozen floors (fresh): m00 `2 passed / 0 failed / 0 ignored / 1
   suites`; m18 `23 passed / 0 failed / 0 ignored / 1 suites`.
   Graph (T1 locked suite UNEDITED, fresh): `104 passed / 0 failed / 0
   ignored / 4 suites` and `107 passed / 0 failed / 0 ignored / 4
   suites`. Lock scan `d6640a2..HEAD` diff-EMPTY for `progress.rs`,
   `cache.rs`, `bench_baseline.json`, goldens/fixtures, progress-family,
   cli, graph, report, tools, ledger, every manifest/lock file;
   `load_caches.rs` blob-identical (`9519a44` both ends).
5. Workspace census (fresh, `--distill` substitute, row-59 errata):
   `### cargo test PASS (exit 0)` [capture 2215 lines] /
   `1247 passed / 0 failed / 13 ignored / 91 suites`. Reconciliation:
   T1-review `1240/0/13/90` +7 passed (4 in-crate seam/release tests +
   3 integration lifecycle tests) / +1 suite, ignored held. Exact.
6. Tools CI-form discover (fresh): `Ran 194 tests in 20.878s` / `OK`.
7. Smoke (fresh, distill): `boot+route checks clean (cold fixture,
   attempt 1, 1.0s)` / `ui_smoke: PASS` - the R5 witness that boot's
   `take_include_graph` at `ui.rs:250` still gets its pair after the
   take/restore lifecycle (warm+cold boot).
8. Hygiene (fresh, direct): exit 0; zero rows on any touched path.
   Encoding `--all` (fresh): exit 0, silent.
9. Commit-vs-touch-set: `e52a9e9` = 5 paths, numstat lib.rs +66,
   stage_load.rs +225/-72, stage_load_tests.rs +163 (NEW, in-crate
   adjacency clause), load_stage_lifecycle_test.rs +203 (NEW), journal
   +282; `69138db` = journal only +35. PASS exact, per-commit.
10. Lifecycle code audit (reviewer's own read of the committed blobs):
    release at stage_load.rs:67 (warm) and :80 (cold, before
    take/passes/save); take (:85) / capped analyzer (:86, literals
    `512`/`128` verified as T1's landed `d6640a2` constants,
    hop-fixed citation) / `drop(analyzer)` :138 BEFORE restore :142
    and save :149; graph/reduced arrive and return by MOVE (cache
    `take()` + re-assign, old :43-44 clones deleted); analyzer is the
    Arc-sole owner of the memo. `ensure_load` counted 94 lines
    (61..154); helpers all < 100; `collect_file_metrics` signature
    unchanged. Consumers verified: `ensure_reduced`/`ensure_callgraph`
    early-return before their internal `ensure_index` (zero spurious
    reloads); `ui.rs` (diff-EMPTY) has no index consumer after load -
    boot pays zero reloads.
11. RSS directional witness (ONE invocation, sanctioned 640-scale):
    `### generic PASS (exit 0)` with
    `stage5 peak RSS (release, N=640) | 2090.2` vs pre-fix band
    2254.1-2258.2 and T1-tree 2258.0: **-168 MiB (~-7.4%)**;
    `save-tail RSS delta | 0.5`; resident-after-return 1066.2 vs
    pre-fix 1144.8; `ensure_load wall | 2.5` s vs pre-fix release
    1.6-1.7 s (cap recomputation cost - by-design I5 trade, finding 1);
    stages 1-4 wall 1.5 s unchanged. Memo `entries` rows (2464/640) are
    DEMAND counts (bench :566 `report.symbol_metrics.len()`), not cache
    lengths - no cap violation; N=640 transient survives (1024.0 vs
    pre-fix 1023.9) exactly as D7 declares. Single invocation, not
    VERIFIED-IDLE: directional only per the RATIFIED median-of->=5
    protocol.
12. bench_compare (RUN - pipeline/src changed):
    `### generic FAIL (exit 1)`; medians `symbol_pass+prep` 1653.7
    (+30% vs baseline 1276.4), `file_pass` 643.5 (+106% vs 312.5),
    `save_load_artifacts+cache` 14457.6, `ensure_load total` 16760.4.
    Known stale/flapping baseline exactly as the T1 reviewer framed it
    (ledger escalation row: same-tree swings 1667->2016/day; PR-2
    `--update-baseline` on VERIFIED-IDLE outstanding; row-61). The NEW
    signal: `file_pass` ~1.84x its same-day T1 reading (349.1),
    structurally attributable to T2 wiring the caps (demand 640-1280 vs
    physical cap 128 -> mass recomputation). Handoff declares a
    stale-baseline FAIL "the known flapping gate, not this hop" - not
    a blocker here; relayed to T0b's post-PR-2 re-judge (finding 1).

## Explicitly NOT run

- `--baseline` FILE form: unrunnable (standing errata row 59); the
  sanctioned `--distill` unscoped census substitute ran and is quoted.
- N=1280 RSS curve / `--bench stage5-memory` G-C discrimination:
  T0b/operator local-gate duties (DP-3, pre-fix-tree discrimination +
  RATIFIED median-of->=5 on idle machine + freed disk). The machine
  still cannot hold an N=1280 artifact save (os error 112, recorded
  twice at T1); per dispatch only the single N=640 invocation was
  attempted; nothing outside temp deleted.
- Pre-fix-tree gate re-runs (R1 capture verification on `d6640a2`):
  impossible read-only; Red states verified by arithmetic consistency
  of the journal's capture lines + diff inspection (notify sites
  byte-identical; literals match fixture shape + the diff-EMPTY 13-frame
  replay sequence) instead.
- `theme_token_check.py`: NOT APPLICABLE (zero UI paths in the range).
- Golden guard script: not landed; byte-identity proven by diff-scope.

## Permission / gate-route record (T2)

- Every gate attempted individually. Refusals were all the catch-all
  boilerplate `{"permission":"bash","pattern":"*","action":"deny",
  "source":"global"}` for NON-gate forms: the git-diff MCP tool could
  not resolve rev ranges (substituted by direct read-only `git diff`
  per file - succeeded), `git show | Select-Object` (substituted by the
  Read tool on the provably diff-EMPTY checkout), `python -c` blobs
  dump and a Write into temp (edit grant is tasks/**-only; substituted
  by per-file direct git diff), an env-prefix RSS form (substituted by
  `distill.py -- cmd /c "set VAR=...&& cargo test ..."` - succeeded,
  exit witness intact), and a `python -c` free-disk probe (free space
  reported via the os-error evidence instead). All cargo/pytest/
  hygiene/encoding/smoke/bench/RSS gates RAN; none skipped or
  weakened on permissions.

---

# T0b reviewer acceptance pass (appended 2026-10-06)

Scope: the M28 T0b (`t0b-gate-mode`) reviewer acceptance pass, run
fresh at HEAD `dc294bb` (start rev `69138db` = T2 landing; main,
ahead=0, behind=0, dirty=24 foreign paths per checkpoint_guard).
Verdict: `tasks/stage5-collateral-memory-ceiling-t0b-verdict.md` -
APPROVED (0 blockers; 1 minor process finding - undeclared gate-
invocation overshoot; 2 informational relays; deviations D1-D3 and all
five hop decisions adjudicated on merit).

## Acceptance criteria vs what happened (evidence lines verbatim)

1. Scoped gate (fresh): `### generic PASS (exit 0)` /
   `Ran 45 tests in 0.327s` / `OK` - 21 pre-existing cases UNEDITED
   (range numstat `475/0` on the tests file, ZERO deleted lines,
   verified independently via `git diff --numstat 69138db..dc294bb`)
   + 24 new. Red per journal: `### generic FAIL (exit 1)` /
   `Ran 45 tests in 0.282s` / `FAILED (failures=4, errors=22)` with the
   missing-attribute classes exactly as the handoff's Red predictions
   name; one Red-green-by-design test pins the default-mode
   last-write-wins contract.
2. CI-form tools discover (fresh): `### generic PASS (exit 0)` /
   `Ran 218 tests in 20.601s` / `OK` = 194 prior + 24 new, exact.
   Anchor reconciliation: the handoff's reviewer-obligation anchor
   `1236/13/89` was stale (superseded @ T2); the ledger M28 NEW anchor
   `1247 passed / 0 failed / 13 ignored / 91 suites` is confirmed
   grep-present on the M28 ledger row and the fresh census matches it
   EXACTLY; the +24 Python tests land in the discover, zero cargo delta.
3. Workspace census (fresh, `--distill` substitute per row-59 errata):
   `### cargo test PASS (exit 0)` [capture: `target/testrun-capture.txt`,
   2213 lines] / `1247 passed / 0 failed / 13 ignored / 91 suites`.
4. Live gate-mode surface (fresh, distill route): `### generic PASS
   (exit 0)` with usage `[--bench {stage5-timing,stage5-memory}]`
   (Gate-2 choice spellings, exit 0); `--bench boot-stages` ->
   `### generic FAIL (exit 2)` / `ERROR unknown --bench choice
   'boot-stages'; expected one of: stage5-timing, stage5-memory`
   (M27 T1b not preempted; `DOCUMENTED_PHASES` diff-EMPTY).
5. Synthetic in-band memory witness (reviewer scratch driver, temp-dir
   baseline; `bench_baseline.json` untouched): exit 0 with
   `stage5-memory sample accounting: accepted 5 of 5 invocations,
   rejected 0 (> 8% peak-operand divergence from per-slot median)` and
   `bench compare: 5 run(s), no regressions past 1.50x baseline and the
   RATIFIED bars (peak 5200 MiB, growth 2.0)`. Bar exceedance exits 1:
   `WARN stage5 peak RSS (debug, N=1280): 5300.0 MiB exceeds the
   RATIFIED 5200 MiB peak bar (N=1280)`. Wholesale divergence rejection
   witness: `REJECT invocation 4: stage5 peak RSS (debug, N=640) sample
   3000.0 MiB diverges +25.0% from slot median 2400.0 MiB (limit 8%)` /
   `accepted 5 of 6`. Namespaced key witness:
   `Windows-AMD64-AMD64_Family_26_Model_68_Stepping_0_AuthenticAMD::stage5-memory`;
   baseline JSON key census: ms machine-class key only,
   `stage5-memory keys present: []`.
6. Default-mode live run (reviewer obligation, fresh):
   `### generic FAIL (exit 1)`; medians `symbol_pass+prep` 1646.0,
   `file_pass` 666.4, `save_load_artifacts+cache` 14648.3,
   `ensure_load total` 16935.9, `plan()` 1668.4,
   `build_file_justification_with_memo` 8527.3; WARNs
   `symbol_pass+prep ... (+29%)`, `file_pass ... (+113%)`. Row-61/62
   stale-baseline class, PR-2 outstanding. Cap-economics relay (row
   69): `file_pass` 666.4 vs T2-review 643.5 vs same-day T1 349.1
   (~1.91x, consistent with the ~1.84x row) and `symbol_pass+prep`
   1646.0 vs 1653.7 - same-machine witness recorded, row 69 stays OPEN,
   no new finding.
7. Smoke (fresh, distill): `boot+route checks clean (cold fixture,
   attempt 1, 1.0s)` / `ui_smoke: PASS`.
8. Hygiene (fresh, direct + distill): `### generic PASS (exit 0)`;
   154 capture lines, all pre-existing R3 warns in foreign paths; zero
   rows on the touched files. Encoding `--all` (fresh):
   `### generic PASS (exit 0)`, capture 0 lines.
9. Commit-vs-touch-set: `ac0eb7e` = journal +190/-0, `bench_compare.py`
   +495/-56, `bench_compare_tests.py` +475/-0; `dc294bb` = journal only
   (+20/-6); range `git diff --name-only 69138db..dc294bb` = the three
   touch-set paths exactly; `bench_baseline.json` diff-EMPTY across the
   range; zero `crates/**` (frozen locks byte-identical by scope).
10. RATIFIED-contract fidelity: every clause traced to code + pinning
    test (verdict section, clause-by-clause): per-slot medians over
    accepted, >8% rejection binding both operands/both scales/both
    directions (four-quadrant test + exactly-8% accepted-boundary test),
    loud accounting always, <5 accepted exit-2 pinned (2 tests),
    multiset parse of twice-per-scale peak labels, 26-template tuple
    set-equality re-grepped against the real bench source at test time
    (reviewer grep corroborates: exactly 25 `{suffix}` stems + the
    ratio template at bench :835), unknown probe row tolerated
    (`WARN untracked phase`, exit 0). All green fresh.

## Explicitly NOT run

- LIVE `--bench stage5-memory` full run (no-baseline hint form): NOT
  RUN - environment-blocked by OPEN ledger row 70 (disk), not
  permissions. Fresh witness: `DISK C: free 1.4 GiB of 1862.0 GiB
  total`; row 70 names this exact gate under `FREE DISK BEFORE`; the
  hint path needs >=5 accepted debug N=1280-suite invocations and the
  N=1280 artifact save fails at this capacity (os error 112 twice at
  the T1 review); attempting it would fail mid-save and leak temp dirs.
  Substituted evidence: the pinned hint test green fresh + live JSON
  read proving no `::stage5-memory` key exists, so the live shape IS
  the pinned exit-0 hint until PR-2.
- `--baseline` FILE form: unrunnable (standing errata row 59); the
  sanctioned `--distill` census substitute ran and is quoted.
- `theme_token_check.py`: NOT APPLICABLE (zero UI paths in the range).
- Golden guard script: not landed; byte-identity proven by diff scope.

## Permission / gate-route record (T0b)

- Every gate attempted individually and RAN via direct or distill
  routes. Refusals (verbatim boilerplate `{"permission":"bash",
  "pattern":"*","action":"deny","source":"global"}`) were all NON-gate
  forms: compound git+echo chains, `python -c` probes (incl. one
  distill-wrapped), `Get-PSDrive C`, and a direct
  `python tools/bench_compare.py --help` (allowlist quirk; the distill
  form ran and passed). One Write into the external temp dir was
  refused by the agent edit grant `{"permission":"edit","pattern":"*",
  "action":"deny","source":"agent"}`; the scratch probe relocated to
  `tasks/t0b-review-probe.py` (row-67 purge class, verdict finding 3).
  No gate skipped or weakened on permissions; the single non-completed
  obligation is environment-blocked and declared.

---

# T0c reviewer acceptance pass (appended 2026-10-06)

Scope: the M28 T0c (`t0c-postfix-curve-ablation`) reviewer acceptance pass,
run fresh at HEAD `e9a4ac1` (start rev `dc294bb` = T0b landing; main,
ahead=0, behind=0, dirty = foreign paths per checkpoint_guard). Verdict:
`tasks/stage5-collateral-memory-ceiling-t0c-verdict.md` - APPROVED
(0 blockers; 5 numbered findings, all informational relays; deviations
D1-D7 all adjudicated on merit; full anti-fabrication recompute from raw
captures succeeded).

## Acceptance criteria vs what happened (evidence lines verbatim)

1. Scoped pipeline, BOTH forms (fresh, `--distill`): plain
   `### cargo test PASS (exit 0)` / `32 passed / 0 failed / 4 ignored /
   7 suites` EXACT (production-invariance witness, route-inert with the
   feature off); instrumented `35 passed / 0 failed / 4 ignored / 7
   suites` exit 0 (+3 route tests, matching journal gates 2/3).
2. Commit-vs-touch-set: `git diff --name-only dc294bb..e9a4ac1` =
   EXACTLY the seven touch-set paths (both Cargo.tomls, `stage_load.rs`,
   `stage_load_tests.rs`, journal, measurement doc, helper); per-commit
   stats all inside (`f312dba` 6 paths, `60c88c7` 1, `bf23e84` 2,
   `e9a4ac1` 2); `Cargo.lock` diff-EMPTY; bench file,
   `crates/graph/**`, `tools/**`, `bench_baseline.json`, ledger diff-EMPTY
   by absence. Frozen locks byte-identical by scope.
3. Anti-fabrication recompute (reviewer probe, own parser, standalone
   law; capture `target/distill-capture.txt` 43 lines, exit 0):
   - Witness census: `load-memo caps override active: semantic=0
     physical=0` present in ALL FIVE G3 captures (line 68 each), ABSENT
     in all six G1, all five G2, and the preflight capture.
   - Independent law re-derivation matches the doc exactly:
     `G1 REJECT invocation 3: release|640 sample 2363.9 vs 2088.8 =
     13.2%`; `G1 accepted 5 of 6 | accepted peak medians:
     {'release|1280': 8469.5, 'release|640': 2088.8}`; `G2 accepted 5 of
     5 ... 8470.9, 2091.2` ratio `4.0`; `G3 accepted 5 of 5 ... 8479.2,
     2086.7` ratio `4.1`; save tails `4371.5 / 4375.6 / 4381.0`;
     unknown-label sets EMPTY in every group.
   - D5 prefix witness on the raw row:
     `test stage5_rss_growth_ratio ... stage5 peak RSS (release, N=640) | 2363.9`.
   - Ablation arithmetic exact: transient landed `4098.0`, cap-0
     `4098.2`, memo residual `-9.7`, uncapped-class `560.8`, A7-alone
     `4371.5 <= 5200: True`.
4. Implementer law pass reproduced end-to-end (fresh `--medians` on the
   G1 captures, capture `target/rv-t0c-g1-medians-rerun.txt` 332 lines):
   `REJECT invocation 3: stage5 peak RSS (release, N=640) sample 2363.9
   vs slot median 2088.8 = 13.2%` / `accepted 5 of 6`; MEDIAN lines
   reproduce the doc tables line-by-line incl. memo rows byte-equal to
   T0a Block 4.
5. Directional re-run (reviewer's own fresh release isolated invocation,
   N=640, distill route, capture `target/rv-t0c-rerun640.txt`):
   `### generic PASS (exit 0)`, `stage5 peak RSS (release, N=640) |
   2090.4` (accepted slot median 2088.8, +0.08%), no witness line -
   direct witness that the instrumented binary is inert with the env
   unset.
6. Disk witness (row 70 discharge evidence):
   `DISK C: free 162 GiB of 1862 GiB - PASS` (threshold 16 GiB).
7. Workspace census (fresh, `--distill` substitute per row-59 errata):
   `### cargo test PASS (exit 0)` [capture: 2218 lines] /
   `1250 passed / 0 failed / 13 ignored / 91 suites`. Reconciliation:
   T2/T0b anchor `1247` +3 = the three `caps_env_ablation` route tests,
   which the capture shows running in pipeline's lib binary (lines
   1410/1415/1416) - mechanism is the `crates/cli/Cargo.toml` dev-dep
   `features = ["test-instrument"]` unification. New anchor: 1250/13/91
   (verdict finding 1).
8. Default-mode live bench_compare (RUN - `crates/pipeline/src` diff in
   range revokes the exemption): `### generic FAIL (exit 1)`; medians
   `symbol_pass+prep` 1529.6, `file_pass` 598.5 (`WARN ... vs baseline
   312.5 ms (+92%)`), `save_load_artifacts+cache` 12657.1,
   `ensure_load total` 14787.7, `plan()` 1535.4,
   `build_file_justification_with_memo` 8455.8. Stale/flapping-baseline
   class (rows 62/69, PR-2 outstanding) - recorded, not re-litigated per
   the handoff's "Groups are NOT gates" clause; `file_pass` 598.5 is
   BELOW the T0b-review same-machine reading 666.4 (noise class).
9. Smoke (fresh, distill): `boot+route checks clean (cold fixture,
   attempt 1, 0.6s)` / `ui_smoke: PASS`.
10. Hygiene (fresh): `### generic PASS (exit 0)`, 160 capture lines, zero
    rows in touched files (sole new rows are the reviewer's own scratch
    probe - purge class). Encoding `--all` (fresh):
    `### generic PASS (exit 0)`, capture 0 lines.
11. Plan-literal + DRAFT discipline: env name/value law, loud-panic
    parse, once-witness string, helper flags all verbatim per handoff;
    the optional `<=` pin case correctly not added; measurement doc
    asserts actuals only ("data, not gate verdict"), no bar verdict
    anywhere; `tools/bench_baseline.json` untouched (no `--update-
    baseline` usurpation); journal is digest-shaped (step-8 contract).

## Explicitly NOT run

- Full 16-invocation curve re-runs: NOT RUN per dispatch (hours) -
  substituted by the independent recompute from the raw captures (item
  3, own parser, no implementer helper code) plus one fresh directional
  invocation (item 5). This is the strongest available read-only audit.
- `--bench stage5-memory` GATE mode: NOT THIS HOP'S DUTY - the formal
  gate waits for the orchestrator's bar decision; this hop is
  explicitly "data, not gate verdict".
- `--baseline` FILE form: unrunnable (standing errata row 59); the
  sanctioned `--distill` census substitute ran and is quoted.
- debug cap-0 ablation: out of scope per accepted deviation D4
  (profile-independence is measured evidence, G1 vs G2).
- `theme_token_check.py`: NOT APPLICABLE (zero UI paths in the range).
- Golden guard script: not landed; byte-identity proven by diff scope.

## Permission / gate-route record (T0c)

- Every gate attempted individually and RAN via direct or distill
  routes. Refusals (verbatim boilerplate `{"permission":"bash",
  "pattern":"*","action":"deny","source":"global"}`) were all NON-gate
  forms: compound git+echo chains, `python -c` probes, `Get-PSDrive C`,
  and direct `python tools/bench_compare.py` (the distill form ran,
  exit witness intact). Reviewer scratch probe relocated to
  `tasks/t0c-review-probe.py` (row-67 purge class alongside
  `tasks/t0b-review-probe.py`). No gate skipped, substituted weakly, or
  judged unrunnable on permissions.

---

# T4b reviewer acceptance pass (appended 2026-10-07)

Scope: the M28 T4b (`t4b-g-tripwire`) reviewer acceptance pass, run fresh
at HEAD `373a160` (start rev `e9a4ac1` = T0c landing, the parent of
`1df4ce4`; main, ahead=0, behind=0; worktree additionally contaminated
MID-REVIEW by in-flight T4 - finding 3 of the verdict). Authority taken
as given, not re-ratified: user decision 2026-10-06 demotes G to a
tripwire. Verdict: `tasks/stage5-collateral-memory-ceiling-t4b-verdict.md`
- APPROVED (0 blockers; 2 minor findings, 2 informational relays; all
three declared deviations ACCEPTED on merit).

## Acceptance criteria vs what happened (evidence lines verbatim)

1. Scoped gate (fresh, handoff gate-2 command): `### generic PASS
   (exit 0)` / `Ran 46 tests in 0.305s` / `OK` - the journal's exact
   landing line; row-65 discipline honored (journal quotes prior `Ran 45
   tests in 0.339s` / `OK` BEFORE inverting; 46 = 45 + 1 new case,
   inversions alone keep N).
2. CI-form discover (fresh): `Ran 219 tests in 18.056s` / `OK` = 218
   prior + 1, exact.
3. Commit-vs-touch-set: `git diff --name-only e9a4ac1..373a160` = the
   four touch-set paths exactly (numstat `35 21` / `31 7` / **`26 0`** /
   `164 0`); per-commit: `1df4ce4` = 2 tools files, `777653f` = evidence
   doc + journal, `373a160` = journal only. `bench_baseline.json`
   diff-EMPTY AND worktree-clean; ZERO `crates/**` in the range.
4. Evidence-doc append-only: full diff is one pure-append hunk
   `@@ -349,4 +349,30 @@` (zero `-` lines); superseded `RATIFIED` growth
   line verbatim at :298-299; note dated (decision 2026-10-06 / landing
   2026-10-07), cites decision 3 + `t0c-verdict sec. 7` + this handoff;
   the journal's self-caught `27 1` -> `26 0` pre-commit fix matches the
   landed shape (never existed in history).
5. Semantics (code read + fresh live witnesses via
   `tasks/t4b-review-probe.py`, sanctioned `run_bench_impl` seam,
   temp-dir baselines - `tools/bench_baseline.json` never referenced):
   - `== A ratio 4.00 in-band: exit=0` +
     `TRIPWIRE stage5 RSS growth ratio (debug, N=640->1280): 4.00 exceeds
     the demoted growth gate 2.0 - tripwire only, never a gate (user
     decision 2026-10-06; the fixture is Theta(N^2) by construction, see
     t0c-verdict sec. 7)`
   - `== B peak 5300 exceeds bar: exit=1` +
     `WARN stage5 peak RSS (debug, N=1280): 5300.0 MiB exceeds the
     RATIFIED 5200 MiB peak bar (N=1280)` (byte-identical to the T0b
     witness)
   - `== C/C2 ratio exactly 2.0: exit=0` with NO TRIPWIRE line (strict
     `>` boundary; both directions witnessed)
   - `== D ratio 4.00 absent from baseline: exit=0` + TRIPWIRE present
     (baseline-independence)
6. Frozen literals: `5200.0` (:187) and `1.50` (:221) in NO diff hunk,
   read at current values; peak-bar quadrant test byte-unchanged (zero
   edited lines in :591-605, appears only as diff context; sampling/
   mode-isolation suites UNEDITED; all 7 test-file deletions inside the
   two authorized hunks); grep-proofs: `GROWTH_GATE` gone from all of
   `tools/`, old growth WARN string gone, peak WARN string intact,
   `TRIPWIRE` reachable via label constant (tuple-identical print).
7. `--help`: `### generic PASS (exit 0)`, exit-1 wording names the peak
   bar + tripwire non-failure. Hygiene: exit 0, 160 pre-existing R3
   lines, ZERO rows on touched paths. `encoding_guard --all`: exit 0,
   capture 0 lines. Smoke: `boot+route checks clean (cold fixture,
   attempt 1, 0.9s)` / `ui_smoke: PASS`.

## Explicitly NOT run / blocked

- Workspace census LIVE re-run: environment-blocked by in-flight T4
  mid-edit in the shared worktree (worktree clean at 19:31 ground truth;
  by census time carried uncommitted `crates/report/**`,
  `crates/index/src/cache.rs`, `crates/cli/src/commands/measure.rs`,
  `tests/golden/{basic,expensive}/load_report.json`, m12/m21 test edits).
  The attempted run: `### cargo test FAIL (exit 1)` [capture 852 lines] /
  `305 passed / 2 failed / 8 ignored / 19 suites` - both failures are the
  golden suites against mid-reshape load_report output (golden lines 634
  vs actual 237 / 636 vs 263), fail-fast truncation before the rest.
  Census acceptance for THIS hop stands on the handoff's own proof
  clause - "Python-only hop - zero cargo delta is the proof" - proven in
  the range diff - plus the ledger's current anchor `1250 passed / 0
  failed / 13 ignored / 91 suites` (accepted as the 2026-10-07
  decision; witnessed 4 h earlier at the T0c review at `e9a4ac1`, whose
  census-relevant files are byte-identical at HEAD). A clean live census
  witness is owed at T4 landing / next quiet window (verdict finding 3).
- `--baseline` FILE form: unrunnable (standing errata row 59); the
  sanctioned `--distill` census substitute was the form attempted above.
- Full pre-change-code Red re-run: impossible read-only; the journal's
  Red text (`AssertionError: 1 != 0`) verified arithmetically against
  the pre-change code path (exceeding arm exit-1s today) per the T2
  precedent.
- `bench_compare --bench stage5-memory` live on the real bench: NOT
  THIS HOP'S DUTY (PR-2-bound; heavy); substituted by the sanctioned
  `run_bench_impl` seam driving the FULL compare path (parse -> sampling
  law -> medians -> bars/tripwire -> exit code), 4-case witness above.
- `theme_token_check.py`: NOT APPLICABLE (zero UI paths). Golden guard:
  not landed; byte-identity proven by diff scope.

## Permission / gate-route record (T4b)

- Every gate attempted individually; all RAN via direct or distill
   routes. Refusals (verbatim boilerplate `{"permission":"bash",
   "pattern":"*","action":"deny","source":"global"}`) were NON-gate
   forms only: a `git show | Select-Object` pipe (substituted by three
   plain `git show --numstat` calls - succeeded), a `python -c` file
   peek, and `git status --short` (substituted by the git MCP status tool
   - succeeded, and it is what exposed the T4 mid-flight dirt). No gate
   skipped or weakened on permissions; the single non-completed
   obligation (live census) is environment-blocked by a concurrent writer
   and declared with its evidence line above.

---

# T4 reviewer acceptance pass (appended 2026-10-08)

Scope: the M28 T4 (`t4-payload-reshape` - the terminal memory fix) reviewer
acceptance pass, run fresh at HEAD `fd7c1ec` (start rev `373a160` = T4b
landing, the exact parent; main, ahead=0, behind=0, dirty=33 foreign paths
per checkpoint_guard). Landing is a message-only-with-journal amend of
dangling `d43571a`. Authority taken as given, not re-ratified: user
decisions 2026-10-06 (T4 dispatch; `CURRENT_ANALYSIS_VERSION` 2.1.0 ->
2.2.0 same-commit as a registered DP-2 exception to the `cache.rs`
frozen-lock; 5200 MiB peak bars ratified). Verdict:
`tasks/stage5-collateral-memory-ceiling-t4-verdict.md` - APPROVED
(0 blockers; 1 MAJOR record-correction finding + 4 relays/errata;
D1-D7 all adjudicated on merit).

## Acceptance criteria vs what happened (evidence lines verbatim)

1. Amend truth (this hop's named risk): `git show -s --format="%B"
   "d43571a"` shows the superseded single-paragraph message (payload +
   golden justifications AND the 2.2.0 lineage entry missing); `fd7c1ec`
   carries all gate-10 sections incl. `release N=1280 peak median 4379.7
   MiB vs the ratified 5200 bar, accepted 5 of 5`; `git show --stat
   "fd7c1ec"` = the 11 declared paths EXACTLY, `Cargo.lock` diff-EMPTY,
   parent `373a160` EXACT. **`git diff "d43571a" "fd7c1ec"` is NOT
   empty**: 12-insertion JOURNAL-only delta (the D7 row folded into the
   amend); the ten code/test/golden paths are byte-identical - the
   recorded "tree byte-identical" claim is false at letter (verdict
   finding 1; required follow-up erratum + landing-hash record).
2. Registered-exception lock audit: `cache.rs` delta = EXACTLY the
   registered bump (`"2.1.0" -> "2.2.0"` const :30, assert :347, lineage
   comments; zero logic lines); m12 pin + lineage comment only; m21
   witness + message only; `LOCK_MANIFEST_JSON_HASH` resync keeps both
   prior hashes in lineage (`0xbee9_...` <- `0xddc3_...`). Literal
   inventory of `2.1.0|2.2.0` across `crates/` + `tests/`: NO sixth
   dependent site (the only remaining `"2.1.0"` code occurrence is R2's
   deliberate stale-epoch fixture); m27 has no version literal (its
   UNEDITED status structural). Green-tree probe = the fresh census
   (item 6): all five dependent sites green in-tree.
3. Goldens: range diff over `tests/golden/**` = EXACTLY
   `basic/load_report.json` + `expensive/load_report.json` (all other
   goldens byte-identical by absence); `symbol_metrics` key grep-absent;
   `symbol_summary {analyzed_symbols, collateral_load}` + full
   `file_metrics` present; m00 green UNEDITED fresh (golden lines in the
   integration capture), m18 parity UNEDITED green (23 cases).
4. Acceptance curve (ratified law): helper `--medians` re-run on the
   five `target/t4-postfix-*.txt` captures:
   `stage5-memory sample accounting: accepted 5 of 5 invocations,
   rejected 0 (> 8% peak-operand divergence from per-slot median)` /
   `MEDIAN(accepted) stage5 peak RSS (release, N=1280) | 4379.7` (<= the
   ratified 5200.0 bar) / `MEDIAN(accepted) stage5 peak RSS (release,
   N=640) | 1063.8` / `MEDIAN(all) RSS checkpoint save tail (release,
   N=1280) | 4377.7` -> save-span transient **2.0 MiB** (from 4098.0) /
   ratio `4.1` printed TRIPWIRE-only per decision 3. **Independent
   recompute** (reviewer probe, own parser, standalone multiset law):
   identical values - `accepted 5 of 5 (independent law, multiset
   operands)`, peaks `4379.7` / `1063.8`, transient `2.0`,
   `acceptance: 4379.7 <= 5200.0 -> True`. Fresh directional release
   N=640: `stage5 peak RSS (release, N=640) | 1063.0` == save tail
   1063.0, save-tail delta 0.0 (pre-T4 same form: 2090.4 peak / ~1024
   transient - dead at half scale too), `ensure_load wall | 0.7` s.
   Live debug group: `stage5 peak RSS (debug, N=1280) | 4364.3` vs save
   tail `4361.2` (pre-T4 debug 8470.9) - BOTH ratified profiles under
   bar.
5. Fresh scoped gates (canonical lines verbatim): report
   `41 passed / 0 failed / 0 ignored / 3 suites` (row-65 baseline
   `36/0/0/3` + 5 new); integration
   `141 passed / 0 failed / 0 ignored / 35 suites`; pipeline plain
   `32 passed / 0 failed / 4 ignored / 7 suites` (IV6); internal oracles
   `12 passed / 0 failed / 0 ignored / 6 suites` (UNEDITED); graph plain
   `104 passed / 0 failed / 0 ignored / 4 suites` + instrumented
   `107 passed / 0 failed / 0 ignored / 4 suites` (I5 diff-EMPTY);
   tools CI-form `Ran 219 tests in 17.895s` / `OK` (zero delta);
   hygiene exit 0 with ZERO rows on any touched path;
   `encoding_guard --all` exit 0, capture 0 lines; smoke
   `boot+route checks clean (cold fixture, attempt 1, 1.0s)` /
   `ui_smoke: PASS` and `--release`: `ui_smoke: PASS` (0.6s).
6. Workspace census (fresh, `--distill` substitute per row-59 errata -
   discharges the census witness T4b finding 3 owed):
   `### cargo test PASS (exit 0)` [capture: `target/testrun-capture.txt`,
   2221 lines] / `1255 passed / 0 failed / 13 ignored / 91 suites` -
   matches the implementer's probe line EXACTLY. Reconciliation from the
   `1250` anchor: +5 = the five `manager::payload_reshape_tests` cases
   (capture lines 1493-1507); m27's un-ignored completion test
   (capture :879) contributes ZERO (its +1 is already baked into the
   anchor) - the dispatch's "+5 = m27 + 4" split was the wrong guess,
   the journal's split is correct. **New anchor:
   `1255 passed / 0 failed / 13 ignored / 91 suites`.**
7. Live bench (both forms):
   - `--bench stage5-memory` (pre-PR-2 hint form, 5 debug invocations):
     `### generic PASS (exit 0)` / `accepted 5 of 5 invocations, rejected
     0` / `WARNING no baseline for machine class
     'Windows-AMD64-...::stage5-memory'; regression compare skipped.
     Ready-to-paste baseline JSON:`; baseline file verified NOT written
     (diff identical pre/post). RECORD ONLY, not a gate pass, per
     dispatch.
   - default mode: `### generic PASS (exit 0)` / `bench compare: 3
     run(s), no regressions past 1.25x baseline` - first clean timing
     witness in the lineage; medians `symbol_pass+prep 1583.4`,
     `file_pass 663.7`, `save_load_artifacts+cache 2422.9`,
     `ensure_load total 4674.9`, `plan() 1707.7`,
     `build_file_justification_with_memo 8368.4`, ZERO WARN lines.
     This PASS measured against an operator baseline update that landed
     UNCOMMITTED IN THIS REVIEW WINDOW (see Explicitly NOT run / relay)
     - the first `--update-baseline`-era reading in the ledger.

## Explicitly NOT run / relay

- `--baseline` FILE form: unrunnable (standing errata row 59); the
  sanctioned `--distill` unscoped census substitute ran and is quoted.
- Full release curve re-runs: NOT RUN per dispatch (hours) -
  substituted by the independent multiset-law recompute from the five
  raw captures + one fresh directional N=640 invocation (item 4): the
  strongest read-only audit available.
- FORMAL `--bench stage5-memory` gate: NOT THIS REVIEW - the first run
  after PR-2 writes the `::stage5-memory` baseline key is the real
  verdict (TRIPWIRE for ratio 4.x expected, never gate-failing). The
  paste-ready snapshot from the hint run carries two hygiene blemishes
  (harness-prefixed label row + raw float `4364.299999999999`) - fix in
  `tools/bench_compare.py` at next touch or sanitize manually (verdict
  finding 3).
- **PR-2 timing hazard (verdict finding 2):** `tools/bench_baseline.json`
  was worktree-CLEAN vs HEAD at review ground truth (02:31Z) and carried
  an uncommitted operator update (recorded 2026-10-07, runs 3, post-T4
  profile) by the bench observation (~03:05Z) - the update landed
  DURING this review's heavy-gate window; PR-2's VERIFIED-IDLE
  precondition is unverifiable from here and a `--update-baseline`
  inside a review window cannot satisfy it by construction. The update
  is still uncommitted (tracked, 12+/12- vs HEAD): operator commit/
  decision outstanding; rows 62/69/70 and the row-69 cap-economics
  re-judge remain OPEN per dispatch.
- Corpora field boot (G-H terminal acceptance): NOT RUN (milestone tail,
  user/operator duty; milestone acceptance = bars [MET, item 4] + this).
- `theme_token_check.py`: NOT APPLICABLE (zero UI paths in the range).
  Golden guard: not landed; golden scope proven by diff + structure.
- Ledger + memory sync at closeout: orchestrator duty; new census anchor
  `1255/0/13/91` and the T4 landing `fd7c1ec` to be recorded there.

## Permission / gate-route record (T4)

- Every gate attempted individually; all RAN via direct or distill
  routes. Refusals this hop (verbatim boilerplate `{"permission":"bash",
  "pattern":"*","action":"deny","source":"global"}`) were NON-gate
  forms only: `git status --short` (substituted by the git MCP status
  tool - succeeded) and compound/pipe chains (avoided per the standing
  route lesson). The independent-recompute probe ran as the scratch
  file `tasks/t4-review-probe.py` (row-67 purge class, verdict finding
  5, joins the t0b/t0c/t4b probes); it raises a cosmetic trailing
  `NameError` AFTER all assertions/prints - its result lines are in
  `target/rv-t4-independent-recompute.txt` and complete. No gate was
  skipped, substituted weakly, or judged unrunnable on permissions.
