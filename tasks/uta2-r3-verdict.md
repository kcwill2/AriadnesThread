# UTA2-R3 Verdict - repo + lang-api (UTB-8 fix + catalog/policy matrices)

Hand: `uta2-r3`, single commit `b43d6da` on base `eaeaf2d`. Handoff:
`tasks/uta2-r3-handoff.md`. Journal: `tasks/uta2-r3-journal.md`. Spec:
`docs/specs/unit-test-audit.md` Round 2 (UTB-8; repo/lang-api pin rows).

## VERDICT: **APPROVED** (3 minor findings, none blocking; all 3 deviations ACCEPTED)

## Fresh canonical lines (reviewer runs, quoted verbatim)

- repo: `python tools/testrun.py --distill --timeout 240 -- -p ariadne-repo` ->
  `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 69 lines]` /
  `38 passed / 0 failed / 0 ignored / 5 suites` (matches handoff expectation 38/0/0/5)
- lang-api: `python tools/testrun.py --distill --timeout 240 -- -p ariadne-lang-api` ->
  `47 passed / 0 failed / 0 ignored / 5 suites` (matches expectation 47/0/0/5)
- loop test under `--nocapture` (vacuous-skip exclusion): `1 passed / 0 failed / 0 ignored / 1 suites`,
  `test discovery_terminates_on_directory_loop_and_records_loop_target_once ... ok ... finished in 0.02s`
- m0-era integration locks over discovery output:
  `python tools/testrun.py --distill --timeout 240 -- -p ariadne-integration-tests --test m0_skeleton_test --test m00_golden_baseline_test --test m00_facts_path_parity_test`
  -> `5 passed / 0 failed / 0 ignored / 3 suites`
- tools CI: `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 296 tests in 23.910s` / `OK`
- hygiene: `python tools/hygiene_check.py` R3 warn-only, clean return (exit 0); zero warns on this
  hand's added lines
- encoding: `python tools/encoding_guard.py --all` silent, clean exit (exit 0)

## Gate results

1. **Per-commit review**: single commit `b43d6da`, `git show "b43d6da"` reviewed file-by-file
   (+774/-2; the 2 deletions are exactly the `use` line and the unguarded `stack.push(path);`).
2. **Commit-vs-touch-set**: `git diff --name-only eaeaf2d..HEAD` -> 8 paths: 4 under
   `crates/repo/src/`, 2 under `crates/repo/tests/`, 1 under `crates/lang-api/src/`, journal.
   All inside the declared hard touch-set. No Cargo, no other crate, no tools.
3. **Frozen locks**: existing suites green untouched - ar27 byte lock in the 38 run; m00 golden
   baseline + facts-path parity green in the fresh scoped run.
4. **Report evidence**: journal is digest-shaped - verbatim red/green canonical lines, capture
   path on gate 1, distilling entry points, zero raw dumps.

## Key independent confirmations

- **THE FIX (item 1) is plan-literal**: `walked_dirs: HashSet<PathBuf>`, root canonicalized at
  seed, every dir `path.canonicalize().unwrap_or_else(|_| path.clone())` AT PUSH TIME, admission
  via first-sight `insert`, RAW path pushed/walked - mirroring the file-side precedent
  (discovery.rs:79; base :70 exactly as the handoff cites). Because the raw path is walked,
  `strip_prefix(root)` -> FileId/record paths unchanged on loop-free trees: PROVEN by
  m00 golden-baseline + facts parity + ar27 byte lock all green (see canonical lines).
- **Watchdog structure read directly** (test file, not journal): walk on a spawned thread, judged
  via `mpsc::Receiver::recv_timeout(15s)`; timeout arms panic; a libtest panicking test cannot
  keep the process alive (orphan thread dies at main exit), so no reviewer rerun can hang. The
  green path terminates in milliseconds (0.02 s measured).
- **The test genuinely ran here**: junction creation is host-privileged; the `--nocapture` rerun
  showed NO skip eprintln and a 0.02 s real walk. The skip arm is loud and returns only on
  creation failure.
- **Dead-arm report (item 2) independently confirmed**: `classify_ownership_and_policy(` has
  exactly ONE production call site - `discovery.rs:87` with hardcoded `false`. All other hits are
  tests. Arm semantics pinned (`pin_toolchain_flag_arm_is_production_dead`).
- **Policy matrix completeness**: production table has 5 third-party + 6 generated keywords;
  the pin table covers all 11, leading + mid-path, plus folded-case hits and whole-component
  near-miss anti-pins (`vendorish`, `vendor_libs`, `targeting`, `outsized`, `third-partydeps`);
  first-matching-component precedence and keyword-loop-before-contract verified against
  policy.rs:24-49.
- **FileKind divergence pinned exactly** (3-row spot-check by reviewer against code):
  `.H` -> from_path FOLDS to Header (types.rs:45-59) yet `is_includable_header` (types.rs:197-205,
  case-SENSITIVE `ends_with`) -> false; `.hh` -> Header/false; dotfile `.h` -> Other (no Path
  extension) yet includable TRUE. Not unified, reported. Matches spec requirement.
- **by_basename insertion-order pin is honest**: production uses `entry(..).or_insert`
  (catalog.rs:50-54) - first insert wins; the pin asserts DIFFERENT winners under flipped
  construction order, proving the key is not key-internal; B4-class report is truthful
  (determinism today rests on discovery's path sort, discovery.rs:107).
- **Catalog boundary rules hand-traced**: `ends_on_module_boundary` (catalog.rs:219-221) makes
  `models.User` impossible to satisfy from `src.xmodels.User` (left char is `x`, not `.`) on BOTH
  the file arm and the parent arm; `""`, `.`, `/`, `..` all `unify_segments` to empty -> empty
  result (catalog.rs:126-130, 195-200); `by_path` canonical arm resolves `..`-carrying absolute
  candidates through the dot-collapsing `canonical_key` - pins match code exactly.
- **locate Err lock**: context text + Debug rendering match production verbatim
  (lib.rs:29 `format!("Failed to locate repository path: {:?}", path.as_ref())`).
- **No unification performed**: walk polarity code untouched; the proposal exists only in the
  journal (item 4) - handoff-compliant.

## Deviation dispositions

- **D1 (item 4, declared): adjacency argument for `entry?`/`metadata?` instead of a deletion-race
  injection - ACCEPTED.** Independent mechanics: `fs::metadata` is a metadata/attribute query not
  gated by file share modes; `read_dir` entry errors are enumeration-level, not file-lockable; a
  dangling link fails `is_file`/`is_dir` BEFORE the `?` sites. So share_mode(0) has exactly one
  deterministic propagation target - the hashing `fs::read` - which the shipped pin trips, and the
  pin proves propagate-never-skip for the same `?` sites in the same loop. A deletion race would
  be flaky and violates the repo's T7a deterministic-injection doctrine. The characterization note
  is accurate and honestly named.
- **D2 (item 5, declared): `{:?}`-rendering assert (first draft asserted the raw lossy string and
  failed) - ACCEPTED.** Test-shape error corrected before shipping, not a behavior deviation: the
  production context embeds the path via Debug (`{:?}`), so asserting the Debug rendering is the
  truthful contract lock.
- **D3 (item 1, disclosed in journal): pre-fix red was the 70-level duplicate-record blow-up
  (truncated by the OS path ceiling), not the anticipated watchdog timeout - ACCEPTED as an
  acceptable bounded-red demonstration.** The handoff said "demonstrate via the bound failing";
  strictly the bound did not trip on this host. Merits: the failing assertion IS the regression
  assertion, for exactly the missing-visited-set cause; pre-fix behavior is independently proven
  by the base code (unguarded `stack.push(path)`, no admission check) - the duplicate shape is the
  mechanically necessary pre-fix outcome; both red shapes (timeout OR duplicates) fail the same
  single assert and both stem from the missing set; the walk was bounded throughout (testrun
  --timeout 240 + the 15 s internal bound); zero harness error. The test's own safety contract
  covers both shapes, and my nocapture rerun proves the post-fix path is the genuine loop walk.

## Findings (numbered, all minor - no required changes)

1. **Journal evidence gap (minor, now closed by this review)**: the handoff line "existing repo
   integration suites (m01 etc.) green" was never journalled as RUN - the journal's green scoping
   (`27 passed / ... / 4 suites`) is repo-crate only. The claim is TRUE (reviewer ran
   m0_skeleton + m00_golden_baseline + m00_facts_path_parity -> `5 passed / 0 failed / 0 ignored /
   3 suites`), but the hand's own record did not establish it. Future hands: journal integration
   claims with the run that proves them.
2. **Journal gate 2 omits the capture path** (gate 1 carries it). Recurring bookkeeping wart.
3. **policy.rs case-fold pin derives expected ownership via a probe heuristic**
   (`probe.contains('_') || probe.starts_with("VENDOR")`) instead of per-probe literals. True and
   green for the four shipped probes, but a reader must re-derive intent; a literal table would be
   self-evident. Non-blocking.

## Ledger closeout routing (deferred-findings candidates from this hand)

- `is_toolchain=true` production-dead arm: wiring OR removal = owner decision (reported + pinned).
- FileKind case-fold vs `is_includable_header` case-sensitivity unification = owner decision
  (pinned exactly, incl. `.H` and the dotfile contrast).
- Walk error-polarity unification policy proposal (journal item 4) = owner decision.
- by_basename/by_module_path collision key-internal tie-break (B4-class; today deterministic only
  via the upstream path sort) = owner decision.
- Alias-first-walked LIFO determinism on case-insensitive filesystems (journal's reviewer-
  candidate row): which of two aliases to one dir answers first after the visited-set fix is
  push-order dependent - pin or owner decision at closeout.

## Not run (explicit statement)

- `python tools/testrun.py --baseline`, `python tools/ui_smoke.py`, `python tools/bench_compare.py`:
  NOT run at this hand - per-hand precedent (T1-T7b, R1, R2) and the AGENTS.md sizing rule
  (workspace-wide final gates run only in a fresh session / the orchestrator's own bounded run).
  OWED at UTA2 closeout. Nothing here substitutes.
- `encoding_guard.py --staged`: vacuous post-commit (clean tree); superseded by the `--all` run
  at HEAD.
- `golden_guard.py`: not applicable (no golden tool in this hand's lock prose; byte-lock
  equivalent = ar27 + m00 suites green, run fresh).

## Refusals observed (configuration note, not skipped gates)

Catch-all deny boilerplate (`{"permission":"bash","pattern":"*","action":"deny",
"source":"global"}`) hit two shapes: the `;`-chained `hygiene_check.py; echo $LASTEXITCODE` /
`encoding_guard.py --all; echo` attempts (BOTH gates reran plain - hygiene R3-only clean return,
encoding silent clean - exit 0 witnessed by clean return per R1/R2 precedent), and one compound
`cmd /c` junction-creation probe (superseded by the strictly better nocapture rerun that exercises
the test's own junction path). All expected gate evidence obtained; no gate skipped.

## Reviewer artifacts

`tasks/uta2-r3-verdict.md` + the Round-2 R3 addendum appended to
`tasks/uta-execution-results.md` - LEFT UNTRACKED per precedent (reviewer git-write denied);
orchestrator folds into a docs commit at convenience.
