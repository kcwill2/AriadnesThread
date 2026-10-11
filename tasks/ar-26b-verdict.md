# AR-26b Verdict — APPROVED

Adjudicated 2026-10-11 via `review-gate` skill. Contract:
`tasks/ar-26b-handoff.md` (@69ab3f7 text, Option A wording per
`tasks/ar-26b-shape-ruling.md` — the ruling is the invariant authority).
Lane commits: `3064b77` (pre-mutation journal) → `8e08600` (guard+swap,
the contract's commit verbatim) → `ff526b6` (addendum). Chain frozen base
`0d348ee`; dispatch HEAD `8eec456`. Reviewer HEAD: `ff526b6` clean vs HEAD.

Evidence addendum: `tasks/ar-26b-execution-results.md` (every gate re-run
fresh, canonical lines quoted verbatim).

## Gate summary (reviewer-own runs)

- Touch-set: `git diff --name-only 8eec456..HEAD` = EXACTLY the 5 declared
  paths (Cargo.lock, scheduler Cargo.toml, scheduler incremental.rs,
  journal, m17 file). All 3 commits inside the touch-set.
- Gate 1 compile-green: `ok (exit 0, 31 warnings emitted)` — capture grep
  confirms zero warning sources from scheduler/artifact (warning-count
  differs per run vs the journal's 11; cache-state variance, not asserted
  by the contract).
- Gate 3 m17: `10 passed / 0 failed / 0 ignored / 1 suites`
- Gate 4 scheduler: `26 passed / 0 failed / 0 ignored / 2 suites` UNEDITED
- Gate 5 worker-core: `8 passed / 0 failed / 0 ignored / 2 suites` UNEDITED
- Gate 7 m00 pair: `4 passed / 0 failed / 0 ignored / 2 suites` UNEDITED
- m23: `7 passed / 0 failed / 0 ignored / 1 suites` UNEDITED
- Gate 6 (re-run myself): tree exit 0; exact-token `ariadne-report` grep
  of my own capture = ZERO matches; `ariadne-artifact` direct at capture
  line 3; cluster/transform/validate present (lines 52/89/106) — described
  per P2, never asserted absent.
- Frozen locks: `git diff 0d348ee -- crates/report crates/artifact
  crates/cli crates/pipeline tests/golden crates/lang-cpp` → EMPTY;
  `tests/integration/**` diff = m17 file only (79/2, deletions = the two
  const size lines); scheduler grep `ariadne[-_]report` = 0 matches.
- hygiene: R3 warn-only rows, ZERO blocking, none in touched files.
  encoding_guard `--all`: silent. Golden dir: no diff content (fixtures
  LF/CRLF warning is foreign churn, no content).
- CHAIN CENSUS: `1400 passed / 0 failed / 13 ignored / 105 suites`
  (exact match to expected; = 1398@26a2 + 2 new m17 tests; suites 105
  reconfirmed). Tools: `Ran 219 tests in 27.699s / OK`.

## Central adjudications

### 1. Guard non-vacuity — PASS

(a) `SCHEDULER_CONE_FORBIDDEN: [&str; 1] = ["ariadne-report"];` byte-exact
one-element (m17_plugin_seam_test.rs:676). The section-5 doc comment names
cluster/transform/validate only as EXPECTED cone members that must never
enter the const — no absence assertion of the three anywhere in the new
code, const, or gate. Ruling P2 honored; no over-broad guard.
(b) Non-vacuity self-test `scheduler_cone_guard_detects_planted_report_edge`
exists (file lines 719-740): planted in-memory manifest
`"[dependencies]\nariadne-report = { path = \"../report\" }\n"` (concat!
form is string-identical to the contract literal), served for a path ending
`planted-scheduler/Cargo.toml`, asserting the walker reports
`ariadne-report` reachable — mirror of AR-20b's
`transitive_guard_detects_planted_scheduler_edge`.
(c) Exact-token predicate: `parse_local_path_deps` (file lines 599-633)
extracts dep names as the full TOML key (trimmed key side of `name = ...`
inside `[dependencies]`/`[build-dependencies]` only) — package-name
position by construction; the guard and self-test compare with `==`
equality on those whole keys, never substring. A name such as
`ariadne-report-x` can never match `ariadne-report`. Helpers reused
unforked — the diff contains no helper changes.

### 2. RED trail authenticity — PASS

The journal RED block is coherent and machine-consistent with the committed
code: `9 passed / 1 failed / 0 ignored / 1 suites`, single failing test
`scheduler_cone_excludes_report`, panic location
`m17_plugin_seam_test.rs:708:5` — line 708 of the COMMITTED file is exactly
the guard's `assert!(offenders.is_empty(), ...)`, so the capture was
produced by the code that landed. Reachable set lists `ariadne-report`
from the scheduler root manifest — the only source being the
`Cargo.toml:16` report edge present at dispatch HEAD (journal attributes
the offender edge to it; contract predicted this line verbatim).
Arithmetic holds: 8 pre-existing + planted self-test = 9 pass, guard-proper
= 1 fail. Commit `8e08600` carries guard AND swap atomically (m17 +
scheduler/Cargo.toml + incremental.rs + Cargo.lock in one diff), so no
RED-in-history guard-only commit exists; the RED is journal-only, never
committed. Guard GREEN-once-in-history is the accepted class per the
AR-20b transitive-guard precedent.

### 3. Predicate both sides — PASS

BEFORE side is the journal's pre-swap capture (exact-token match at line
71, plus expected members at 73/76/96/113 — coherent, and the AFTER side
is the lock that binds at HEAD). AFTER side re-run by me at `ff526b6`:
`cargo tree -p ariadne-scheduler -e normal` exit 0; ZERO exact-token
`ariadne-report` matches in my own `target/distill-capture.txt`;
`ariadne-artifact` at capture line 3 as a direct normal edge (the
implementer's claimed line number reproduces exactly);
`ariadne-cluster`/`ariadne-transform`/`ariadne-validate` present. Both
mechanisms of Ruling 1 hold at the same commit.

### 4. D3 "seven core crates" doc line — ACCEPTED

The module doc (file line 10) still says "the seven core pipeline crates"
while `GUARD_CORE_CRATES` is now 8. The contract is NOT silent: its
touch-set entry for the m17 file enumerates the permitted change exhaustively
("guard extension ONLY: the two new tests + two const-array additions;
every existing test body byte-untouched") and the Discipline clause binds
"the two const-array edits are the only in-place changes" — a third
in-place edit, including that doc line, would have violated the contract.
Byte-intact is mandated, not merely sanctioned; the new section-5 doc
comment records the 8th addition explicitly (file lines 690-693). The
stale wording itself is real: dispositioned as ledger-wording candidate
(finding 2), report-only per the implementer's own observation.

### 5. D4 mis-targeted Edit calls — CLEAN

Two Edit calls aimed at nonexistent `tasks/placeholder-noop.md` were
refused by the tool and created nothing: `checkpoint_guard status` reports
ZERO dirty files at review start; the full untracked census contains no
placeholder file (nor anything unexpected outside the long-standing
tasks/*.md + .kilo/plans/* doc pattern); all 3 commits carry only the
touch-set paths. Zero side effects confirmed independently of the
journal's disclosure.

## Findings (numbered; all minor — no rework required on `8e08600`)

1. minor (config/process, orchestrator-side): `git status --porcelain`
   is allowlist-refused in the reviewer lane — refused twice, verbatim:
   `The user has specified a rule which prevents you from using this
   specific tool call. Here are some of the relevant rules
   {"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
   Ground truth substituted via `checkpoint_guard.py status` + targeted
   `git diff --stat` (both succeeded); no gate was skipped. Consider
   allowlisting `git status` for reviewer lanes.
2. minor (ledger-wording class): `tests/integration/tests/
   m17_plugin_seam_test.rs:10` "seven core crates" is stale vs
   `GUARD_CORE_CRATES` len 8 — contract-forbade the fix; disposition to
   `tasks/milestone_ledger.md` deferred-findings for the planner lane
   (already report-flagged in the journal at lines 229-231).
3. minor (process-record, report-only): handoff baseline line 47
   `report 7/0/0/3` is stale — I measured `7 passed / 0 failed /
   0 ignored / 4 suites`, agreeing with the implementer and the
   orchestrator dispatch pin. Same class: the handoff census anchor line
   `1398/0/0/105` omits the 13 ignored that the dispatch expectation and
   my census both carry. Handoff is outside every lane's touch-set; no
   fix; ledger-close wording.
4. minor (process, closeout doc lane): the invariant authority
   `tasks/ar-26b-shape-ruling.md` is UNTRACKED (`git log` for it is
   empty) — it exists only on disk, alongside the chain-wide pattern of
   untracked ar-* task docs. Closeout should commit the chain's task
   documents so the ruling survives with the verdicts.

## Deviation dispositions

- D1 (NONE against code shape): CONFIRMED — every literal (const arrays,
  guard body, planted test, panic message, `-e normal`, RED-first,
  single-commit bundling) matches Ruling 4 verbatim.
- D2 (stale handoff baseline row): ACCEPTED — report-only; measurement
  corroborated (`7 passed / 0 failed / 0 ignored / 4 suites`).
- D3 (stale "seven core crates" doc): ACCEPTED — contract-literal
  compliance required byte-intactness; ledger candidate finding 2.
- D4 (refused placeholder Edits): ACCEPTED — zero side effects verified
  on disk and in every commit.
- D5 (extra BEFORE-side gate 6 capture): ACCEPTED — same mandated
  command, journal-only, supplies the report-back item-2 evidence.

## Follow-up dispositions

Findings 2-3 → `tasks/milestone_ledger.md` deferred-findings at
orchestrator closeout. Finding 4 → closeout doc-hygiene (commit the
chain's task docs). Finding 1 → reviewer-lane allowlist note to the
orchestrator. `ui_smoke.py` / `bench_compare.py` not run here (dispatch:
closeout); `testrun.py --baseline` treated as erratum per dispatch
restriction (census run fresh with `--distill -- --workspace` instead).

## Chain closure

The AR-26 chain closes APPROVED at all three hops. AR-26a landed the
artifact crate skeleton and manifests; AR-26a2 moved the manager IO +
tests verbatim (11/11 blobs R100, hash-level blob identity) leaving the
report's terminal public surface at 12 DTOs behind re-export shims
(`tasks/ar-26a2-verdict.md:33`); AR-26b (this hop) reparented
`ariadne-scheduler` to `ariadne-artifact`, dropping the last
scheduler→report edge from source, manifest, and Cargo.lock, and
machine-locked `"ariadne-report" NOT-IN Cone(scheduler) AND NOT-IN
Tree(scheduler)` in AR-20b's own planted-edge style — guard RED-first
before the swap, atomic guard+swap landing, non-vacuity locked. The
scheduler cone is report-free while the cluster-stack is
intact-by-design (cone-content delta vs pre-chain HEAD is exactly zero;
artifact drags what report dragged). Audit A3 is PARTIALLY satisfied per
Ruling 2; the cluster/transform/validate residual is NOT a follow-up AR
but a DEFERRED LEDGER ROW at orchestrator closeout carrying the AR-26c
promotion charter verbatim (Ruling 3/P3). The conservation equation
closes chain-wide (scheduler 26 + worker-core 8, zero test edits) and
the CHAIN CENSUS for closeout is
`1400 passed / 0 failed / 13 ignored / 105 suites`.
