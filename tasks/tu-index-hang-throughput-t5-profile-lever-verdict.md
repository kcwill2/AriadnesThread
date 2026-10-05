# T5 profile-lever verdict - M27 tu-index-hang-throughput

VERDICT: **APPROVED**

FLAG: **ACCEPT** - `[profile.release] overflow-checks = true` ships. The
measured steady-state cost is zero-to-noise across 8 stages (two post-build
sessions) and the loud-panics rationale is not merely historical: the
correction ledger carries a SAME-DAY (2026-10-05) M25-class brace-depth
underflow observation, and this very hop makes the release binary the
normal human launch - making that profile loud is the correct pairing.

Hop: `tasks/tu-index-hang-throughput-t5-profile-lever-handoff.md`. Landing
commit `5b517e6` (parent `41f7ad4` - verified via `git rev-parse "5b517e6^"`).
Journal: `tasks/tu-index-hang-throughput-t5-profile-lever-journal.md`.
Reviewer acceptance-pass evidence:
`tasks/tu-index-hang-throughput-execution-results.md` addendum 5.
Note for orchestrator finalization: the handoff file itself is UNTRACKED
(not part of the locked touch-set); track it with the verdict at closeout.

## Per-commit review (oldest-first; single commit in range)

`git log --oneline 41f7ad4..HEAD` -> exactly one commit:
`5b517e6 feat(release): overflow-checks=true for release profile + release
launch route (M27 T5 profile lever; cost table in measurement sec. (e))`.

`git show --stat "5b517e6"` -> 6 files changed, 219 insertions(+),
3 deletions(-). Reviewer judged the whole diff hunk-by-hunk (single commit):

1. `Cargo.toml` (+7): stanza + 4-line rationale comment byte-faithful to the
   handoff "What to build" item 1 literal (plus one blank separator line).
   Appended at EOF as specified; flag names ONLY `[profile.release]` (I1
   literal holds).
2. `AGENTS.md` (+4): bullet verbatim from handoff item 2, anchored after
   `- Use \`cargo build --release\` for release builds` as required. The
   "~20x slower" claim is evidence-bound (151.4 ms/TU release vs E1
   ~3.1 s/TU debug; both dated to T1a/E3).
3. `EXTENSIONS.md` (+11/-2): both blocks (:22, :113) get the two release
   lines inside the block; debug form kept below as dated labeled
   alternative. Matches item 3 literally, including "same command plus
   `--release`" and the dated quick-debug note.
4. `extensions/ariadne-extension/README.md` (+5/-1): same treatment; emoji
   headings untouched (edit anchored inside the bash block); new release
   lines use the EXTENSIONS.md-correct `--features` form (= the handoff's
   own item-3 literal), debug line preserved verbatim including its stray
   `--` separator (disclosed report-only; correct under the discipline
   clause).
5. `tasks/tu-index-hang-throughput-t1-measurement.md` (+34/-0): pure EOF
   append, ZERO deletion lines in the hunk - I4 (append-only over the frozen
   T1a artifact) verified on the diff itself, not just the journal.
6. Journal (+158, new file): in-set, always allowed.

## Commit-vs-touch-set

`git diff --name-only 41f7ad4..HEAD` -> exactly the six declared paths
(Cargo.toml, AGENTS.md, EXTENSIONS.md, extension README, measurement doc,
journal). Zero paths outside. `Cargo.lock` zero-diff (single-pathspec
`git diff 41f7ad4..HEAD --stat -- Cargo.lock` -> empty). `tools/**`,
`crates/**`, `tests/**` byte-identical (absent from the range diff - covers
every named frozen lock: `ui_smoke.py`, `bench_compare.py`,
`bench_baseline.json`, `boot_stages_bench.rs`, `common/mod.rs`,
`progress.rs` + contract tests, goldens, fixtures incl.
`m27-nonascii-underflow/`).

## Plan-literal checks

- Bench invocation in sec. (e) method line: `python tools/distill.py --
  cargo test --release -p ariadne-pipeline --test boot_stages_bench --
  boot_stages_release_timing --ignored --nocapture --test-threads=1` -
  verbatim equal to t1a-handoff gate 1 (:161-163). PASS.
- All nine sec. (a) reference numbers cited in sec. (e) (8 medians + the
  include-graph cold spread 460.9-491.2) match the frozen table at :26-33.
  All 8 delta-% values re-derived by hand: +3.4/0.0/0.0/+167.1/+4.1/-3.6/
  +0.6/-3.0 - every one correct.
- MUST lines: gates 1/2/4 printed their required tuples (reviewer fresh,
  below). Census UNCHANGED (I3): `1235 passed / 0 failed / 6 ignored /
  88 suites` - suites 88 and ignored 6 hold exactly.
- I2: `tools/ui_smoke.py` absent from the range diff; behaviorally
  re-confirmed `ui_smoke: PASS` (debug route unaffected).
- Invocation ceiling: 7 enumerated + 1 confirming re-run = 8 <= 8. PASS
  (support call disclosed as D2; see dispositions).

## FLAG ACCEPTANCE adjudication (reviewer-owned call)

Trip-resolution exchange: protocol-faithful, line-for-line.
(a) Run-1 include-graph cold 1240.3 ms vs median 464.3 -> +167.1%, over the
+10% bar -> implementer STOPPED (journal TRIP record: "NOT committing until
adjudicated"), spent exactly ONE confirming re-run of the allowed form, got
478.3 ms INSIDE the recorded spread 460.9-491.2 -> NOT A TRIP per the
handoff's own rule, recorded BOTH numbers in journal AND sec. (e), then
committed. No commit between trip and resolution. The handoff rule
("if the re-run lands inside the T1a recorded run spread for that stage,
that is NOT a trip") was applied exactly as written, not stretched.

Mechanical plausibility: run 1 was the FIRST launch of freshly rebuilt
release binaries under the new profile (the handoff itself anticipated the
full rebuild); the reviewer's independent THIRD data point - second
post-build invocation on this session - printed `include-graph cold total
(release) | 490.9` (inside 460.9-491.2; identical machine header,
`profile=release debug_assertions=false`, exit 0, `finished in 162.71s`).
Every post-build observation (478.3, 490.9) clusters inside the T1a spread;
only the first-ever cold launch is an outlier. Integer-arithmetic checks
slowing an I/O-dominated 464 ms stage 2.7x is not a credible mechanism;
first-touch page-cache state is.

Steady-state cost: across two post-build sessions the 8 labels sit in
-4.5%..+13.5% single-value deltas; every value is inside its own T1a
recorded run spread (the largest, warm-load +13.5%, rides that row's own
37.2-51.3 spread, which itself spans ~+/-16% around the median - a noisy
row, not a cliff). No stage-level cliff. This is the handoff's own
acceptance shape ("a uniform +2..+8% cost is acceptable on the loud-panics
rationale, a stage-level cliff is not") - and the observed cost is even
better than the acceptable shape.

Rationale sufficiency: the flag is not insurance against a hypothetical.
M25 brace-depth and M27 tokenizer were both silent-wrapped in release, and
the correction ledger records a same-day (2026-10-05) runtime underflow
observation of the M25 class at `crates\lang-cpp\src\parser.rs:61` - the
class is still live. This hop simultaneously makes release the NORMAL
launch route; shipping the flag converts silent column/depth corruption
into visible failure in exactly the profile every user is now pointed at.
Cost ~0 in steady state, benefit demonstrated recurring. ACCEPT.

Sequencing: landing BEFORE PR-2/T1b is per the ratified order (T5 ->
baseline refresh -> T1b) so the fresh baseline inherits this profile regime;
no re-open risk of the row-61 flapping-gate saga from this side.

## Deviation dispositions

- D1 (bar tripped, resolved by the sanctioned exchange): **ACCEPTED**. See
  FLAG ACCEPTANCE above - adjudicated on merits with an independent third
  data point, not rubber-stamped.
- D2 (+1 distill support call to stage the six paths for a non-vacuous
  `encoding_guard --staged`): **ACCEPTED**. Disclosed; avoids the vacuous
  empty-index pass; set-equality staging precedent (M26 V3 D1/D5); gate
  ceiling still 8/8, the extra call is support, not a gate.
- D3 (chained PowerShell exit-code-echo forms refused; gates attempted
  independently): **ACCEPTED - config family, not implementer fault**.
  This reviewer hop hit the identical family (log below) and substituted
  the same way; nothing was skipped on either side.
- Start-rev note (`41f7ad4`, one docs-only commit above the `bd4fb1a`
  anchor): **NOTED, no disposition needed** - the reviewer's fresh census
  proves the anchor holds across both commits.
- Report-only doc drifts (`--bin cli`, `cd crates/ui`, stray `--`):
  **ACCEPTED as report-only** - adjudicated independently: the handoff's
  discipline clause names the `--bin cli` case VERBATIM and orders journal
  + report, do NOT fix; item 3 locks the edit surface to "same command plus
  `--release`"; a correct fix requires choosing between declared bins
  (`ariadne` vs `tool`) - a decision outside this hop's ratified scope.
  The release lines are no more broken than the debug lines the docs shipped
  for months; this hop neither created nor widened the breakage. Preserving
  verbatim was RIGHT. Follow-up routed, not dropped (disposition 1).

## Findings (numbered; no blockers; no required implementer changes)

1. MINOR (journal completeness) -
   `tasks/tu-index-hang-throughput-t5-profile-lever-journal.md` Gate log:
   gates 1,2,3,confirm,4,5 carry exit-quoted lines, but gate 6
   (`encoding_guard.py --staged`) has NO recorded line/exit code; it is
   implied only via the D2 staging rationale. Superseded by the reviewer's
   own `python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)`
   on the landing tree (strictly broader than a pre-commit index check), so
   no blob risk. Record the omission in closeout bookkeeping; no re-run.
2. MINOR (config, not the implementer) - reviewer gate-surface refusals by
   the boilerplate catch-all (`{"permission":"bash","pattern":"*","action":
   "deny","source":"global"}`): `git show | findstr`, `unittest ... 2>&1 |
   Select-Object`, `cmd; echo $LASTEXITCODE` chains, inline `python -c`,
   bare `python tools/ui_smoke.py`. Every affected gate completed via a
   permitted equivalent (standalone run, distill digest attestation,
   Read-tool capture read); no gate skipped. Recurrence of the row-59/60/65
   family (open config row persists).
3. MINOR (planner observation, discharge scope - not an implementer
   finding) - ledger row 67's fact base names THREE debug routes (AGENTS.md,
   ui_smoke builds AND launches `target/debug`, extension docs); the ratified
   hop closes the doc routes and - by I2 - deliberately leaves `ui_smoke`
   debug (PR-2/bench-bound). The row-67 discharge note MUST record that
   scope explicitly so "ui_smoke profile" is an on-ledger decision, not a
   silent gap.
4. MINOR (pre-existing doc defect, routed) - the `--bin cli` trio: no bin
   named `cli` is declared (`crates/cli/Cargo.toml`: `ariadne`, `tool`);
   `EXTENSIONS.md` block 1 `cd crates/ui` before a cli run (`crates/ui` is a
   pure lib); README :81 stray `--` argument separator. Documented
   report-only here by design; needs its own small named hop.

## Follow-up dispositions (routed, not dropped)

1. Finding 4 (doc-drift trio) -> `tasks/milestone_ledger.md`
   deferred-findings dated row at M27 closeout (planner-owned; reviewer
   cannot write the ledger), with a named fix hop after T1b.
2. Finding 3 -> discharge note wording for row 67 (planner, same commit).
3. Orchestrator: track `tasks/tu-index-hang-throughput-t5-profile-lever-
   handoff.md` (currently UNTRACKED) + this verdict + execution-results
   addendum 5 in the closeout commit.
4. `bench_compare` stays PR-2/T1b-bound - exemption stands (this hop ships
   zero code; its bench obligation was the ignored cargo-test bench, run 3x).

## M27-closeout delta

Row 67 DISCHARGE READINESS: YES - the named small doc hop landed APPROVED;
release launch route shipped at every doc site; flag landed with a measured,
reviewer-accepted cost table (sec. (e)); census anchor `1235/0/6/88`
re-proved at `5b517e6`. Remaining M27 closeout items: T1b (PR-2-bound:
operator `bench_compare --update-baseline` on VERIFIED-IDLE FIRST),
walkthrough-builder hop, ledger+memory finalization (row 67 flip with the
finding-3 scope note; row 33 status update), worktree census + operator
throwaway purge (row 66 queue), artifact tracking (disposition 3).

## Gates (reviewer-fresh, canonical lines verbatim)

- Gate 1 `python tools/testrun.py --distill -p ariadne-pipeline` exit 0:
  `25 passed / 0 failed / 4 ignored / 6 suites` (MUST verbatim).
- Gate 2 `python tools/testrun.py --distill -p ariadne-cli` exit 0:
  `110 passed / 0 failed / 1 ignored / 5 suites` (MUST verbatim).
- Gate 3 (bench, reviewer's own third run) exit 0: all 8 release labels;
  include-graph cold `490.9` inside the recorded spread; capture
  `target/distill-capture.txt` read fresh (19 lines,
  `finished in 162.71s`).
- Gate 4 `python tools/testrun.py --distill -- --workspace --no-fail-fast`
  exit 0: `1235 passed / 0 failed / 6 ignored / 88 suites` - UNCHANGED (I3).
- Gate 5 `python tools/hygiene_check.py` exit 0
  (`### generic PASS (exit 0)` distill attestation): R3 warn-only, zero
  R1/R2, zero findings in any touch-set file.
- Gate 6 substitute `python tools/encoding_guard.py --all` exit 0
  (`### generic PASS (exit 0)`, 0-line capture).
- Acceptance pass: tools `Ran 188 tests in 21.887s` / `OK`;
  `ui_smoke` (distill route): `boot+route checks clean (cold fixture,
  attempt 1, 0.9s)` / `ui_smoke: PASS`;
  `python tools/testrun.py --baseline`: `usage error: missing value for
  --baseline` (6th encounter, ledger row 59 errata; `--distill` census
  substitute quoted above).

Full evidence with refusals log: execution-results addendum 5.
