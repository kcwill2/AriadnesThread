# tu-index-hang-throughput execution results - T0-report reviewer hop

Addendum written at the T0-report reviewer hop (2026-10-05), NOT the milestone
closeout. Scope: what the reviewer actually ran against the handoff's
acceptance criteria for `tasks/tu-index-hang-throughput-t0-report-handoff.md`
(commit `62ce72b`, parent `b0f89f8`). Verdict: APPROVED -
`tasks/tu-index-hang-throughput-t0-report-verdict.md`.

## Acceptance criteria vs evidence (each with its evidence line)

1. Landing commit = the two touch-set files only.
   Evidence: `git diff --name-status b0f89f8..62ce72b` ->
   `A tasks/tu-index-hang-throughput-t0-evidence.md` +
   `A tasks/tu-index-hang-throughput-t0-report-journal.md`;
   `git show --stat 62ce72b` -> `2 files changed, 284 insertions(+)`.
   Zero paths outside `tasks/`; goldens/fixtures/progress.rs not in the diff.
2. Encoding + hygiene exit 0.
   Evidence: `python tools/hygiene_check.py` -> `HYGIENE_EXIT=0` (warn-only R3
   rows, legacy files); `python tools/encoding_guard.py --all` ->
   `ENC_ALL_EXIT=0`; `python tools/encoding_guard.py --staged` ->
   `ENC_STAGED_EXIT=0` with clean staging area (vacuous at review time;
   substituted by direct byte grep `[^\x00-\x7F]` -> no match in either
   committed file, since `encoding_guard.py:36` allow-lists `tasks/`).
3. Mandatory grep-present list (13 strings) in the committed doc.
   Evidence: 13/13 present (full-file review + ripgrep literal re-grep of
   `subtract with overflow`, `tokenizer.rs:331`, `tokenizer.rs:311`,
   `chars().count()`, `WRAPS SILENTLY`, `M25`, `SUFFICIENCY`, `DP-6`,
   `2.0.0 -> 2.1.0`, `RELOADED, not rebuilt`, `binary search`,
   `NOT outstanding work`, `DEBUG/dev-profile`).
4. E3 quoted verbatim. Evidence: report :32 byte-identical to shape
   sec. 0 line 74 (thread id 55408, `tokenizer.rs:331:21`), single line,
   pure ASCII.
5. Code citations vs live tree: `tokenizer.rs:49/:307/:311/:313/:327/:331/:333`
   all confirmed (byte/char unit split internal to the file; :331 col 21 =
   subtraction = E3 site); `ui.rs:786-805` confirmed no `catch_unwind`,
   handle discarded; watcher `:636` catch confirmed; `parser.rs:61` floor
   guard confirmed; `crates/index/src/cache.rs:16` = `"2.0.0"` (DP-6 bump
   rides T2); `pipeline/lib.rs:507` in-order TU loop confirms sec. 6 rule;
   commits `9ce140a`/`ad65747`/`af11113` all exist.
6. Sections 1-9 complete in contract order; hypothesis ledger H-A..H-E
   complete; discriminators (a)-(e) present with "NOT outstanding work"
   marker; DP-6 recorded RESOLVED YES.

## Canonical summary lines (quoted verbatim)

- Census substitute (`python tools/testrun.py --distill`, full workspace):
  `1213 passed / 0 failed / 4 ignored / 85 suites` (exit 0,
  `### cargo test PASS (exit 0) [capture: ...target\testrun-capture.txt,
  2085 lines]`). Matches the `816d09d` anchor - expected: pure doc additions
  add zero tests.
- Tool suites (`python -m unittest discover -s tools -p "*_tests.py"`):
  `Ran 188 tests in 20.243s` / `OK`.
- Literal `python tools/testrun.py --baseline`: NOT RUNNABLE -
  `usage error: missing value for --baseline`. Known open errata, ledger
  line 59 (FILE-value flag; no committed baseline file; recurrence +1).

## Gate reachability log (per-gate attempt discipline)

Ran successfully: `checkpoint_guard.py status`; `hygiene_check.py`;
`encoding_guard.py --all` and `--staged`; `testrun.py --distill`;
`unittest discover`; bare `git status/log/show/diff/rev-parse` forms.
Refused by the global catch-all
(`{"permission":"bash","pattern":"*","action":"deny","source":"global"}` -
boilerplate per ledger line 29, so each was attempted, then routed around):
`python -c` one-liners (byte checks -> done via Grep tool instead),
PowerShell pipe forms (`| Out-Null`, `| findstr`), `git ls-tree` (Red check
re-derived via `git diff --name-status` A/A).

## Explicitly NOT run at this hop

- `python tools/ui_smoke.py` and `python tools/bench_compare.py`: exempted by
  explicit coordinator ruling for this doc-only hop; the diff proves zero
  code change, so both would measure the byte-identical `b0f89f8` binaries.
  M27's real bench obligations remain bound to T1a/T1b (shape sec. 3.3).
- Literal `--baseline` run: unrunnable (above), substitute run and quoted.
- Anything mutating git: reviewer is read-only; the journal's staged-state
  non-vacuity claim for gate 1 could not be re-created post-commit and was
  replaced by the reviewer's own byte-level ASCII proof of the committed
  blobs (finding 3 in the verdict).

---

# Addendum 2 - T0b reviewer hop (fixture approval), 2026-10-05

Reviewer hop for `tasks/tu-index-hang-throughput-t0b-handoff.md`; commits
`d6267a3` + `4ef8412`, parent `62ce72b`. Verdict: APPROVED with the frozen-set
fixture approval GRANTED - `tasks/tu-index-hang-throughput-t0b-verdict.md`.

## Acceptance criteria vs evidence (each with its evidence line)

1. Touch-set exact both commits. Evidence: `git show --stat "d6267a3"` ->
   6 files, 599 insertions (all six touch-set paths); `git show --stat
   "4ef8412"` -> `1 file changed, 15 insertions(+), 3 deletions(-)` journal
   only; `git diff --name-only 62ce72b..HEAD` -> exactly the six paths.
2. Frozen-set byte-identity. Evidence: `git diff --stat 62ce72b..HEAD --
   tests/fixtures/basic tests/fixtures/expensive` -> EMPTY; `git diff
   --name-only 62ce72b..HEAD -- tests/fixtures` -> only the four
   `m27-nonascii-underflow` files (new sibling dir).
3. Red evidence reproduced fresh by the reviewer: `python tools/distill.py
   -- cargo test -p ariadne-integration-tests --test
   m27_nonascii_corpus_test -- --ignored --nocapture` -> `### cargo build
   FAIL (exit 101)`; capture `target/distill-capture.txt` (18 lines):
   `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured;
   1 filtered out; finished in 0.02s`; `panicked at
   crates\lang-cpp\src\tokenizer.rs:331:21: attempt to subtract with
   overflow` - exact E3 site (t0-evidence :32; dispatch brief's ":331:11"
   is a dispatcher typo, verdict finding 3). `:311` class confirmed
   statically on live `tokenizer.rs` (9-char/11-byte column-1 word ->
   10 - 11 underflow); punct TU is analyzed first, hence the captured site.
4. Fixture bytes single-encoded; authored content pure ASCII. Evidence:
   green byte lock asserts from_utf8 OK, flanked `C3 A9`/`C2 AB` shapes,
   `E2 82 AC` presence, no U+00C3, README+test zero bytes >= 0x80; reviewer
   codepoint grep `[\x{80}-\x{10FFFF}]` -> no match in test file, README,
   journal, runs.h; `^[\x{20AC}]` -> `punct_run.cpp:10`; `Ã`/`Â`/double-enc
   signatures -> none in fixture dir; `python tools/encoding_guard.py --all`
   -> `ENC_EXIT_0`. (Mandated `git grep -P` route REFUSED - see gate log;
   substituted with the permitted equivalents above.)
5. Contingency fidelity (crux): v1-literal no-panic is journal-attested and
   reviewer-confirmed by static arithmetic (2-byte chars cannot underflow at
   any position; one extra byte at column 1 floors at 0). Swap = punct
   driver U+20AC (E2 82 AC) at column 1 + word second `C3 A9` at column 1;
   all mandated run shapes and the named identifier retained. ADJUDICATED:
   declared mechanism honored, plan self-contradiction (literal spec vs
   acceptance 1) resolved within the approval hop; the word_run driver exceeds
   the contingency's letter but was fully declared for re-approval and lands
   exactly where re-approval authority sits. ACCEPTED (verdict D1).
6. Scoped green gate: `python tools/testrun.py --distill -p
   ariadne-integration-tests --test m27_nonascii_corpus_test` ->
   `1 passed / 0 failed / 1 ignored / 1 suites`, exit 0.
7. Hygiene: `python tools/hygiene_check.py` -> `HYGIENE_EXIT_0` (R3 warn
   rows only; zero in touched files). Test functions < 100 lines; .cpp
   fixtures 10 lines each.
8. UNEDITED proof: zero `crates/`, `tools/`, golden, oracle, common/mod,
   Cargo.toml paths in the range diff; those suites ran unedited to zero
   failures inside the workspace census.
9. Ignore attribute verbatim vs handoff lines 66-68 (file line 143); T2
   named as un-ignorer.

## Canonical summary lines (quoted verbatim)

- Green scoped gate: `1 passed / 0 failed / 1 ignored / 1 suites` (exit 0).
- Workspace census substitute (`python tools/testrun.py --distill`):
  `1214 passed / 0 failed / 5 ignored / 86 suites` (exit 0). Reconciles the
  declared delta against the last ledger anchor `1213 / 0 / 4 / 85` @
  `816d09d`: +1 passed / +1 ignored / +1 suite (new test binary). New M27
  census anchor: `... / 5 ignored / 86 suites`.
- Literal `python tools/testrun.py --baseline`: NOT RUNNABLE - `usage error:
  missing value for --baseline` (known FILE-flag errata, ledger line 59;
  recurrence logged).
- Tool suites: `Ran 188 tests in 20.477s` / `OK`.
- `ui_smoke` (distill route): `boot+route checks clean (cold fixture,
  attempt 1, 1.0s)` / `ui_smoke: PASS`.
- `bench_compare` (distill route, run despite exemption eligibility -
  fixture-only hop): `bench compare: 3 run(s), no regressions past 1.25x
  baseline`.

## Gate reachability log (per-gate attempt discipline)

Ran successfully: `checkpoint_guard.py status`; both scoped cargo gates via
`distill.py`/`testrun.py --distill`; `testrun.py --distill` (full workspace);
`hygiene_check.py`; `encoding_guard.py --all`; `unittest discover`;
`ui_smoke.py` and `bench_compare.py` via distill; bare `git log/show/diff`.
Refused by the global catch-all (`{"permission":"bash","pattern":"*","action":
"deny","source":"global"}` boilerplate - each attempted, then routed around):
`git grep` and `git grep -P` (all forms); inline `python -c`; `--help` usage
probes; `| findstr` pipe form. No gate was skipped; every refusal substituted
with a permitted equivalent that passed.

## Explicitly NOT run at this hop

- Literal `--baseline` FILE form: unrunnable (ledger line 59 errata;
  substitute quoted above).
- Mutation-side reproduction of the v1 (pre-swap) fixture completion: v1 bytes
  were never committed; accepted as journal-attested + reviewer-confirmed
  statically.

---

# Addendum 3 - T2 reviewer hop (root-cause fix + DP-6 rider), 2026-10-05

Reviewer hop for `tasks/tu-index-hang-throughput-t2-handoff.md`; commits
`ea21687` + `59f0512` + `bcc3bb0`, parent `de5622f`. Verdict:
**APPROVED-PENDING-RIDERS** - `tasks/tu-index-hang-throughput-t2-verdict.md`.
State at HEAD `bcc3bb0` is workspace-RED on two machine-pinned DP-6
dependents (findings 1-2); the pinned rider fixes are in the verdict.

## Acceptance criteria vs evidence (each with its evidence line)

1. Both flush sites fixed in one commit (plan-literal rule). Evidence:
   `git show "ea21687" -- crates/lang-cpp/src/tokenizer.rs` -> `- column:
   column - word.len()` -> `+ column: column - len` AND `- column: column -
   punct.len()` -> `+ column: column - len`; live code `tokenizer.rs:307/327`
   reuse `let len = <run>.chars().count()` - one unit for `column` and `end`.
2. Audit sound (12 sites). Evidence: reviewer-independent grep
   `\w+ - \w+|-= |\.len\(\) -` over both files @ HEAD -> exactly A1-A12
   (tokenizer :233/:269/:311/:331, parser :70/:283/:285/:287/:306/:368/:396/
   :483); every SAFE-BY-INVARIANT guard re-read from live code (e.g. :281
   `self.i >= 2`, :286 `self.i >= 3`, :301 `paren_depth > 0`, :260/:264
   monotone). FIXED x2 / SAFE-BY-INVARIANT x8 / ALREADY-GUARDED x1 - zero
   second bugs.
3. `parser.rs` byte-identical + M25 guard intact. Evidence: absent from
   `git diff --name-only de5622f..HEAD`; floor guard `parser.rs:61-69` live
   and unweakened; lang-cpp suite green UNEDITED.
4. I14 both profiles, same module, fresh reviewer runs. Debug:
   `152 passed / 0 failed / 0 ignored / 2 suites`; release:
   `152 passed / 0 failed / 0 ignored / 2 suites`. RED-in-both-profiles
   credibility adjudicated (post-landing, RED unreproducible without a
   revert): quoted panic sites `:311`/`:331:21` match live call-graph
   arithmetic; release wraps `18446744073709551615` = `2-3` and `10-11`
   exactly reproduce on the pinned sources; m27 corpus debug RED independently
   reproduced at the approved T0b hop. Tests are external bounds properties,
   not tautologies.
5. ASCII parity. Evidence: fix is bit-invariant on ASCII by construction
   (`len() == chars().count()`); parity test green both profiles; m00 goldens
   + m18 parity UNEDITED (absent from range diff) green in the component
   proof: `34 passed / 0 failed / 0 ignored / 5 suites` with
   `test_m27_nonascii_corpus_cold_index_completes ... ok` (I12: corpus
   COMPLETES un-ignored; anti-vacuity `test_specimen_bytes_are_single_encoded_
   utf8 ... ok`).
6. Rider discipline. Evidence: `git show "59f0512"` - cache.rs = const + doc
   sentence + assert literal ONLY (envelope fields byte-identical, I7; `§9`
   context byte-intact); m12 = literal + comments only; m27 = `1 deletion`,
   exactly the `#[ignore]` line. Constant: single definition `cache.rs:21` =
   `"2.1.0"`; assert `:336` = `"2.1.0"`.
7. Touch-set. Evidence: `git diff --name-only de5622f..HEAD` -> 6 paths, all
   declared; per-commit `--stat` = scoped files; journal-only completion
   commit `bcc3bb0` in-set.
8. Stop-and-report adjudicated: m21 witness sanctioned as rider R1; report
   byte-lock discovered by the reviewer's census - rider R2. Full rider
   instruction + landing re-check totals in the verdict.

## Canonical summary lines (quoted verbatim)

- lang-cpp debug scoped: `152 passed / 0 failed / 0 ignored / 2 suites`
  (`### cargo test PASS (exit 0)`).
- lang-cpp release scoped (I14): `152 passed / 0 failed / 0 ignored /
  2 suites` (exit 0).
- index scoped: `11 passed / 0 failed / 0 ignored / 2 suites` (exit 0).
- integration full scoped: `99 passed / 1 failed / 0 ignored / 24 suites`
  (exit 1) - single failure
  `scala_fingerprint_moves_only_the_scala_digest ... m21_scala_semantic_
  index_test.rs:499:5: assertion left == right failed: M18.10 cache epoch
  witness, left: "2.1.0"` (the declared stop-and-report; fail-fast abort
  confirms the journal's m22+-never-started note).
- integration component proof (m27+m12+m00+m18parity+m18epoch):
  `34 passed / 0 failed / 0 ignored / 5 suites` (exit 0).
- report scoped: `34 passed / 1 failed / 0 ignored / 1 suites` (exit 1) -
  failure `manager::tests::load_artifact_bytes_are_byte_locked ...
  manifest=0xbee91e361bbe1e4b` vs lock `0xddc3_f9ee_566b_5ea6`; json/bin
  hashes match their locks (finding 1).
- Unscoped census, default (fail-fast, rider-pending aborts it):
  `388 passed / 1 failed / 1 ignored / 38 suites` - TRUNCATED, reported, not
  used for reconciliation.
- Unscoped census, complete (`python tools/testrun.py --distill --
  --workspace --no-fail-fast`): `1217 passed / 2 failed / 4 ignored /
  86 suites`. Reconciliation vs anchor `1214 passed / 0 failed / 5 ignored /
  86 suites`: 1214 + 4 new underflow tests + 1 m27 un-ignore = 1219 total;
  observed 1217 + 2 failed = 1219; ignored 5 -> 4; suites 86 held. Post-rider
  landing expectation: `1219 passed / 0 failed / 4 ignored / 86 suites`.
- Tool suites: `Ran 188 tests in 20.446s` / `OK`.
- `python tools/hygiene_check.py` -> `### generic PASS (exit 0)` (distill
  route; R3 warn rows only, zero in touched files - tokenizer `:50/:53` sit
  in the untouched `tokenize_raw` loop).
- `python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)`,
  0-line capture.
- `ui_smoke` (distill route): `boot+route checks clean (cold fixture,
  attempt 1, 0.7s)` / `ui_smoke: PASS`.

## Gate reachability log (per-gate attempt discipline)

Ran successfully: `checkpoint_guard.py status`; bare `git log/show/diff`;
both-profile scoped `testrun.py --distill`; index/integration/component/
report scoped runs; `--baseline` attempt; unscoped census (default +
`-- --workspace --no-fail-fast`); `unittest discover`; `hygiene_check.py`;
`encoding_guard.py --all`; `ui_smoke.py` via distill. Refused by the global
catch-all (`{"permission":"bash","pattern":"*","action":"deny","source":
"global"}` boilerplate - each attempted once, then routed around; no gate
skipped): direct `rg` (substituted: grep tool with the audit regex),
compound conditional/redirect forms for `$LASTEXITCODE` (substituted:
distill route, which prints the exit code in its digest), `python -c`
byte-inspection one-liners (substituted: direct source reads). `python tools
/testrun.py --baseline` -> `usage error: missing value for --baseline` -
testrun usage error, not a permission denial (ledger row 59 errata,
recurrence +1; `--distill` substitute quoted).

## Explicitly NOT run at this hop

- `python tools/bench_compare.py`: exempted by dispatch ruling + handoff
  silence (no bench gate declared; M27's bench truth is bound to T1a/T1b and
  the PR-2 operator condition; the ledger's dated bench-baseline staleness
  row makes hop-1 bench flaky). Stated honestly: the hop DOES change a
  hot-path crate (`lang-cpp` tokenizer), so the M27 T1 bench verdict remains
  the binding performance evidence for this fix's landing.
- Literal `--baseline` run: unrunnable (above); substitute quoted.
- RED-side re-runs (both-profile panic/wrap captures): impossible without
  reverting the landed fix (reviewer is read-only); adjudicated on internal
  arithmetic + the T0b hop's independent RED reproduction.
- Anything mutating git: reviewer is read-only; riders are implementer/
  orchestrator duties pinned verbatim in the verdict.
