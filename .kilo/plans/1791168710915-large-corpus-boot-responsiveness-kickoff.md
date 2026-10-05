# Kickoff (architect artifact): `large-corpus-boot-responsiveness` -> TWO milestones

Status: PLANNED (rework 3, 2026-10-05T04:44Z). Folds the CONFIRMED root
cause from the live specimen's self-reported panic (E3): the hang was a
CONSEQUENCE of a swallowed worker panic, T0 is landed by field evidence,
and T2 has confirmed content. Prior reworks superseded in place; the
two-milestone topology (DP-0 answer) and the resolved DP-1/DP-2/DP-3
standing are unchanged. Governing shape sibling:
`.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md`
(contracts, confirmed mechanism, hypothesis ledger, and the TWO
ready-to-register ledger rows in its sec. 10).

The M-numbers and dates are assigned by the `milestone-lifecycle` kickoff
runner per milestone. Ledger registration, the `tasks/` kickoff docs, the
`checkpoint_guard.py status` census, and the graduation audit are
kickoff-runner steps per the skill; the architect does not commit the
ledger.

## 1. Intent, symptoms, and the confirmed root cause (motivating evidence)

- S1 (visibility): at `tool ui` boot the first row "Loading artifacts" sits
  idle/pending long while substantial work runs underneath with zero
  visible movement; the app looks frozen.
- S2 (throughput): "Indexing translation units..." (stage 2/5) very slow on
  large corpora.
- E1, 2026-10-05T04:11Z (field): stage 2 ~30% after 90+ minutes (5744 TUs;
  ~3.1 s/TU) -- projected multi-hour cold boot.
- E2, 2026-10-05T04:17Z (ESCALATED as hang): frozen at 1780/5744, ~20%
  total CPU, UI server alive, no build CPU.
- E3, 2026-10-05T04:22Z (ROOT CAUSE CONFIRMED, specimen self-report):
  `thread 'tokio-rt-worker' panicked at crates\lang-cpp\src\tokenizer.rs:331:21:
  attempt to subtract with overflow`, with a per-core graph (several cores
  active, NONE pinned) matching "worker died mid-run, siblings wind down"
  -- not a deadlock. Verified mechanism (shape sec. 0): the tokenizer
  counts `column` in CHARS but subtracts BYTE length at BOTH flush sites
  (`tokenizer.rs:311` word, `:331` punct -- the panic site), while `end`
  correctly uses `chars().count()`; any non-ASCII bytes in a run underflow
  the column. DEBUG (overflow-checks on -- which E3 proves the user's run
  used) PANICS; RELEASE WRAPS SILENTLY, corrupting token positions with
  no signal. Same underflow class as the M25 brace-depth incident, whose
  one-site guards the corrections row still records as incomplete -- the
  audit obligation in T2 exists because of that precedent.

Structural facts underneath (verified, line cites in shape sec. 0): all of
`Pipeline::new` (serial SHA-256 discovery, build discovery, fingerprints)
runs before any progress callback exists; stage 1 emits exactly two
frames; UI boot passes `workers: None` (single-threaded index loop -- no
rayon, no Mutex on this path); the stage-2 message carries only
`{done}/{total} TUs`, never a path; and the UI boot's `spawn_blocking`
has no `catch_unwind` and drops the JoinHandle -- THE SWALLOW that made a
dead worker look like a hang. E3 names the arithmetic bug; M1 names the
swallow. Both are fixed, in their own milestone, independently valuable.

Intent: milestone 1 makes boot progress honest and named (frames from the
first second, current TU path in the stage-2 row, dead boot surfaces as
error); milestone 2 kills the confirmed panic class (both sites + audit +
release-profile characterization), measures release-profile reality
(incl. whether release+single-thread already suffices), then does
throughput machinery only if the measurement justifies it -- never
trading dishonest percents or byte-drifted output for speed (A4 honesty
precedent, m00/golden locks).

## 2. Milestone topology (DP-0 resolved: TWO milestones)

- Milestone 1 `boot-progress-visibility` -- S1 only. Frames from the first
  second; stage-1 sub-progress; stage-2 row names the current TU path; a
  dead boot must surface as error, never as frozen loading. No bench gate.
- Milestone 2 `tu-index-hang-throughput` -- the hang is a CONSEQUENCE, not
  its own bug: T0 is LANDED by E3 field evidence. Remaining order: T0
  report bookkeeping + minimized RED repro -> T1 RELEASE-profile
  measurement (incl. launch-profile record and the release-already-single-
  thread SUFFICIENCY GATE) and T2 the confirmed fix, then T3 throughput
  machinery ONLY IF T1's sufficiency gate fails, then conditional scope.

Inheritance (recorded by milestone 2's kickoff doc): milestone 1's frames
and append-only contract tests (`stage_spans.rs` tests, the TuStarted
path-in-message lock, the first-frame-before-construction lock, the
boot-error-surface lock) exist on `main` before milestone 2's parallel
path lands; milestone 2 accepts the obligation to show them UNEDITED and
GREEN under parallel event arrival (DP-2 completion-count semantics),
without re-opening milestone 1.

Sequencing verdict (E3 supersedes the pre-E3 reading; shape sec. 1): the
specimen emergency is CLOSED -- the panic self-reported file:line, so
nothing waits on a dump. Milestone 2 priority is T0 report -> T1 and T2 in
parallel -> T3 only on a sufficiency-gate FAIL. Milestone 1 dispatches IN
PARALLEL throughout: its V3 hop must land before milestone 2's T3 workers
wiring so the parallel path inherits the TuStarted contract, and before
milestone 2 closeout so the inheritance obligation is checkable. E3 also
STRENGTHENS milestone 1's motivation: I11 (dead-boot error surface) is now
PROVEN necessary by a live incident, and I10 (path-in-row) is what will
name the offending file unaided next time.

## 3. Milestone 1 scope: `boot-progress-visibility`

In: boot progress sequencing in `crates/cli/src/commands/ui.rs`
(first frame before `Pipeline::new`; panic/error bridge for the boot task);
stage-1 sub-progress events (pipeline + an additive `crates/repo`
discovery hook); `ProgressEvent::TuStarted` so the active stage-2 row
names the current TU's repo-relative path (binding requirement from the
hang incident -- new message payloads allowed, the stage sequence and the
stage-5 13-frame replay stay untouched); append-only boot contract tests.

Out (reasons): any analysis-output or performance change (observability
only; I1 zero-drift); any `crates/ui` asset change (rows are generic; DP
history `ui-progress-parity`); parallelism (milestone 2); stage 3/4
sub-progress (no symptom); SSE (none exists; the UI polls).

Handoffs (each within the sizing law; details shape sec. 4): V1 cli
(frame-1-first + boot panic->error bridge + hygiene extraction of the
stage-span constants to `ui/stage_spans.rs`); V2 repo (additive
`discover_with_progress` hook); V3 pipeline+cli compile-coupled
(`BootMilestone` stage-1 events + `TuStarted` + `new_with_progress` +
mapping + append-only contract tests + pipeline stage-module extraction,
with written pipeline-touch justification).

## 4. Milestone 2 scope: `tu-index-hang-throughput`

In: T0 report bookkeeping (E3 evidence in, minimized RED repro of the
`:331` panic authored alongside T2; offender-TU identification via the
first-missing/newest-mtime shard rule for the fixture); T0b approved
minimized regression fixture (non-ASCII inside word AND punct runs;
documented frozen-set approval path); measurement (`boot_stages` bench +
`bench_compare --bench boot-stages`, RELEASE profile explicit, plus the
record of how the app is NORMALLY launched -- E3 proves the user's 90-
minute run was DEBUG, so part of "forever" is profile, not algorithm --
and the release+single-thread SUFFICIENCY GATE); the CONFIRMED fix:
`tokenizer.rs` :311 AND :331 both-site correction + same-class usize
subtraction audit across `tokenizer.rs`/`parser.rs` (M25 precedent:
one-site guards missed the class) + release-profile characterization of
the old wrapping semantics; then throughput wiring per resolved DP-3
(`tool ui --workers`, auto default, watcher inherits) with DP-2
completion-count percent + monotonic clamp and parallel-path `TuStarted`
emission -- dispatched ONLY if T1's sufficiency gate fails; DP-4
conditional scope (repo hashing / lang-cpp parse) stays OPEN, evidence-
gated. Data-integrity disposition: artifacts/shards from RELEASE-profile
runs over non-ASCII corpora (including the completed 30%) carry silently-
wrapped columns and will be RELOADED (not rebuilt) post-fix by
fingerprint match -- re-index affected corpora after T2; clean global
invalidation = `CURRENT_ANALYSIS_VERSION` bump landing with T2 = user
decision DP-6; a column-vs-line-length shard scan is a candidate detector
only, not a commitment.

Out (reasons): shard-envelope FIELD changes (I7; DP-6 covers only the
version CONSTANT, the escape hatch the evidence legitimately opened);
real-corpus gates (bench on synthetic scale; field numbers quoted in
evidence docs only); throughput machinery before the T1 sufficiency gate
(measurement-before-prescription remains the governing rule -- now with
the added twist that the measurement includes WHICH PROFILE was
measured).

Handoffs: T0 report (LANDED root cause + RED repro; no fix code), T0b
minimized fixture (fixtures approval path), T1 bench (pipeline tests) +
gate mode (tools) + profile record + sufficiency verdict [PR-2 binds],
T2 CONFIRMED fix + audit + characterization (one crate: lang-cpp;
version-bump rider only if DP-6 YES), T3 workers/percent semantics
(cli+pipeline(+scheduler), conditional), T4 conditional (DP-4).

## 5. Invariants by milestone (oracles in shape sec. 3)

Milestone 1: I1 zero analysis-output drift; I2 stage-5 13-frame sequence
UNEDITED (frozen lock per resolved DP-1); I3 `progress.rs` byte-identical
(frozen lock per resolved DP-1); I5 bounded first visible movement; I6
terminal/web row parity UNEDITED; I9 honest in-span percents; I10 stage-2
active row names the current TU path; I11 a failed/panicked boot surfaces
state=error, never a frozen loading row -- PROVEN necessary by E3 (the
UI sat frozen while its worker died; only console stderr told the truth);
hygiene/encoding gates exit 0.

Milestone 2: I1 (incl. parallel-vs-sequential byte parity; ASCII-path
tokenization bytes must not shift -- the fix only corrects non-ASCII
column values); I4 per-stage percent monotonic under BOTH arrival
patterns (DP-2 binding: completion count + clamp); I7 shard envelope
fields unchanged (version constant per DP-6); I12 cold index of the
minimized (non-ASCII) corpus completes; I14 the unit-mismatch class is
dead in BOTH profiles (release characterization pins it); inheritance
obligation: milestone 1's appended tests UNEDITED and green.

## 6. Frozen locks (both milestones; proofs in the shape)

Inherited from the ledger: `tests/golden/**` byte-locked;
`tests/fixtures/**` frozen (milestone 2 may add one minimized specimen
ONLY via the documented approval path -- reviewer-approved commit + dated
ledger note; nothing inside `basic`/`expensive`); theme assets + token
baseline untouched; m00/m18 parity UNEDITED; distillation contract for all
gate commands.

Resolved DP-1, now binding lock rows: `crates/cli/src/commands/progress.rs`
byte-identical for BOTH milestones; the stage-5 13-frame sequence
UNEDITED for both; boot progress contract tests APPEND-ONLY (new tests and
new rows may be added by milestone 1; milestone 2 may not edit them at
all, only run them). Milestone-2-only: shard envelope FIELDS byte-
identical; `CURRENT_ANALYSIS_VERSION` byte-identical EXCEPT the pending
DP-6 answer (a YES lands the bump with T2 as a cache invalidation for the
silent-wrap class -- sec. 4 data-integrity flag; a NO routes the
re-index instruction to affected corpora instead).

## 7. Preconditions routed before implementers boot

- PR-0 (CLOSED by E3; formerly "capture the live specimen NOW"): the
  worker self-reported the panic at `tokenizer.rs:331:21`; the dump/stack
  protocol (shape sec. 7) is demoted to OPTIONAL post-mortem. Residual
  user routing: (a) preserve the console scrollback / panic line and the
  per-core graph observation in the T0 doc; (b) DATA-INTEGRITY notice --
  do not trust shards/index artifacts produced by RELEASE-profile runs
  over non-ASCII corpora (they carry silent column wraps and will be
  RELOADED, not rebuilt); plan a re-index after T2 lands (clean
  invalidation via the DP-6 decision); (c) record how the app is normally
  launched/built (profile) -- E3 proves the observed run was debug.
- PR-1 (milestone 1 kickoff runner only): ledger row 1 registration,
  census, M-number, graduation audit. Milestone 1 runs WITHOUT any bench
  precondition (it makes no perf claim; see sec. 8 G1-5).
- PR-2 (operator, milestone 2 bench gates ONLY):
  `python tools/bench_compare.py --update-baseline` on a VERIFIED-IDLE
  machine (stale/escalated 2026-10-04 ledger row). Does not gate T0/T0b;
  gates T1 onward.
- PR-3 (both milestone kickoff runners): row 2 registration, census,
  M-number, and the inheritance record (sec. 2) in the kickoff doc.
- PR-4 (handoff authors): every gate invocation via `tools/distill.py`
  (standing subagent bash-denial ledger row).
- PR-5 (planner): T1/T2 may author from this plan directly (root cause is
  CONFIRMED); T3's handoff is authored ONLY on a written T1 sufficiency-
  gate FAIL (release+single-thread not acceptable at corpus scale). The
  measurement-before-prescription rule now protects against PREMATURE
  parallelism, not against an unknown cause.

## 8. Acceptance gates (summary; full text shape sec. 8)

Milestone 1: G1-1 scoped cli/repo/pipeline suites exit 0; G1-2 frozen
locks diff-EMPTY incl. the DP-1 rows; G1-3 boot-recording oracle: stage 1
>= 2 distinct row states, first Active frame before input computation
completes, stage-2 active message contains the current TU path,
panic-injection test ends with state=error; G1-4 `ui_smoke` PASS; G1-5
`bench_compare` NOT RUN with a written exemption (no stage-5 producer path
changes; behavioral no-op delegation proof). If a reviewer insists on a
run while the stale-baseline row stands, one informational run may be
recorded but its exit-1 cannot block milestone 1.

Milestone 2: G2-1 T0 report lands (E3 evidence in; minimized RED repro
authored with T2 -- root cause already CONFIRMED, not gated); G2-2 T1:
RELEASE-profile bench + gate mode green, pre-fix table + debug-context
row + launch-profile record, PR-2 refreshed stage-5 baseline clean, and a
WRITTEN release+single-thread sufficiency verdict (decides whether T3
runs at all); G2-3 T2 (CONFIRMED content): RED repro green at BOTH flush
sites, I14 release+debug characterization green, class audit over
tokenizer.rs+parser.rs dispositioned, data-integrity disposition executed
per DP-6, ASCII-path byte parity intact (m00/m18 UNEDITED); G2-4 if T3
runs: minimized-corpus completes (I12), byte parity I1/I7, `boot_stages`
affected-phase improvement vs T1 by ratified K; G2-5 inheritance
obligation: milestone 1's appended tests UNEDITED and green under W>1
arrival + the I4 shuffled-arrival table; G2-6 shared hygiene/encoding/
census (`--distill` substitute, standing errata) + `ui_smoke` on wiring
hops.

## 9. Resolved user decisions and what remains open

Resolved: DP-0 two milestones (this rework); DP-1 frozen-floor lock rows
(sec. 6); DP-2 completion-count percent + monotonic clamp (milestone 2
binding invariant I4); DP-3 `--workers` flag, auto default, watcher
inherits (milestone 2 binding decision T3).

Open: NEW DP-6 `CURRENT_ANALYSIS_VERSION` bump with T2 -- recommended YES
(sole reliable cache-invalidation for the silent-wrap corruption; a NO
must name the re-index route); whether T3 runs at all -- the T1
release+single-thread sufficiency gate decides; DP-4 conditional `crates/
repo`/`crates/lang-cpp` scope -- T1 evidence review; improvement target K
and worker setting N -- from T1's table (now RELEASE-profile); the
minimized fixture's approval -- routed at T0b; the exact auto-workers cap
-- ratified at T3 with T1 data.
