# UTA-T5 Verdict - reviewer gate (2026-10-18)

Hand: H5 `uta-t5` - scheduler + worker-core + index (UTA-B2 fix, UTA-B6
surfacing, boundary pins). Handoff: `tasks/uta-t5-handoff.md`.
Commit under review: `3e48c51` on base `7c9805d` (single commit).
Journal: `tasks/uta-t5-journal.md`. Spec rows: UTA-B2 (audit:31), UTA-B6
(audit:51), tmp-race row (audit:90), H5 plan row (audit:127).

## VERDICT: APPROVED

All gates fresh-run by the reviewer; all numbers match the journal verbatim.
No blockers. Five minor findings (all non-blocking, routed). All seven declared
deviations adjudicated ACCEPTED on independent merits.

Canonical reviewer lines (fresh, this session):

- `-p ariadne-index`: `22 passed / 0 failed / 0 ignored / 3 suites`
- `-p ariadne-worker-core`: `19 passed / 0 failed / 0 ignored / 2 suites`
- `-p ariadne-scheduler`: `40 passed / 0 failed / 0 ignored / 2 suites`
- consumer check `distill.py -- cargo check -p ariadne-cli -p ariadne-pipeline`:
  `ok (exit 0, 0 warnings emitted) -> capture: target/distill-capture.txt`
- hygiene: `### generic PASS (exit 0)` (R3 warn-only, zero R1/R2)
- encoding `--all`: `### generic PASS (exit 0)` (capture 0 lines)
- tools CI suite: `Ran 232 tests in 19.505s` / `OK`

## Gate record (reviewer-run)

1. `python tools/checkpoint_guard.py status` - HEAD `3e48c51`, branch main,
   tracked tree clean, untracked = T1-T4 verdict artifacts only. Census carries
   the pre-existing pool/temp worktrees from earlier milestones (unchanged).
2. Scoped suites re-run fresh via `testrun.py --distill` - all three match
   the expected and journal lines exactly (above).
3. `git log --oneline 7c9805d..3e48c51` -> single commit, reviewed via full
   `git show "3e48c51"` (1628 diff lines, file-by-file).
4. Commit-vs-touch-set: `git diff --name-only 7c9805d..3e48c51` -> 11 paths:
   `crates/index/src/{fingerprint,types}.rs`,
   `crates/scheduler/src/{engine,pool,queue,shard,worker}.rs`,
   `crates/worker-core/src/{task,tu_store,tu_store_t5_pins}.rs`,
   `tasks/uta-t5-journal.md`. ALL inside the declared touch-set; no
   cli/pipeline/Cargo touches. PASS.
5. Frozen locks: `git diff --numstat` -> total deletions in the range = 13
   lines (fingerprint.rs -2 = the old signature + old `unwrap_or_default()`
   body; engine.rs -11 = the swallowed checkpoint loop at the fix site).
   Zero deletions in any existing test -> test contracts byte-untouched. PASS.
6. Hygiene / encoding / tools suite: exit 0 / exit 0 / OK (above). The R3
   warns on the touched files (`fingerprint.rs:155,156,166`, `pool.rs:793-794`,
   `tu_store.rs:54,61,62`) sit OUTSIDE the single diff hunk of each file
   (untouched lines, shifted numbers) - verified against the hunks: pre-existing.
7. Report-evidence check: journal is digest-shaped - canonical lines, capture
   path, exit witnesses, zero raw dumps; gates used the distilling entry
   points; commit went through `commit_scoped.py` with the 3 declared scopes. PASS.

## Adjudication of the directed audit focuses

### 1. UTA-B2 fix (item 1) - VERIFIED

- Shipped: `.unwrap_or_else(|error| format!("SERIALIZE-FAILED:{error}"))` -
  output identical in shape to the handoff literal (inline-format arg vs
  positional; same string). Fail-closed: stale -> rebuild.
- Determinism: the material is a pure function of the inputs (serde_json
  error Display is input-determined); pinned directly by recompute-and-equal
  (`contexts_material_surfaces_serialize_failure_instead_of_empty_string`).
- Honest-empty distinction: pinned three ways - `starts_with
  ("SERIALIZE-FAILED")`, fingerprint != honest-empty-material fingerprint,
  and fingerprint != `""` (legacy material). Distinct failing states produce
  distinct materials (`..._distinct_failures_do_not_share_one_material`),
  which the old `unwrap_or_default()` silently collided.
- AR-05 mirror: cache.rs:451 defines the same `InjectedUnserializableContext`
  seam type over a generic `context_hash_of`; fingerprint.rs mirrors the name,
  the generic-for-the-seam reason, and the alpha/beta distinctness pattern.
- Consumer byte-consistency: BOTH fingerprint consumers route through the
  untouched `StageFingerprints::from_inputs` (scheduler `incremental.rs:135`,
  pipeline `lib.rs:366`); the one direct caller (`cli discover.rs:133`) passes
  concrete slices through the generic unchanged. The success-path bytes are
  untouched (`serde_json::to_string(&(contexts, tus))` unchanged); only the
  failure branch differs. Consumer check `cargo check -p ariadne-cli -p
  ariadne-pipeline` exit 0, 0 warnings, zero caller edits in the diff.

### 2. UTA-B6 fix (item 3) - VERIFIED + journal-BLOCKER legitimate

- `persist_shard_checkpoints` preserves the old loop on every non-fix path:
  same `pending_fingerprints.remove` per analysis, `completed_analyses.push`
  on BOTH arms (foreign analyses still flow into the reduction - no data
  loss), same save call, same map residue. Old inline loop fully replaced
  (no dual representation).
- No silent drop: foreign id -> per-incident `eprintln!` WARNING + skip;
  save failure -> WARNING with IO error + count; aggregate WARNING totals
  line at run end. Non-silent channel shipped.
- Red evidence: journal red `27 passed / 1 failed / 0 ignored / 1 suites` is
  sound - at base the foreign shard WAS written under `unwrap_or_default()`
  empty fp, so the shipped test's on-disk assertions (`load_shard(foreign)
  .is_none()` + zero `.json` in the dir) necessarily failed. Red-proven
  empty-fp shard write reproduced as directed.
- `checkpoint_save_failure_surfaces_but_does_not_kill_the_run` passes at
  base BY CONSTRUCTION (a swallowed failure has no in-process observable -
  the journal states this honestly); the handoff's "red test, per item" is
  satisfied at item granularity by the foreign-id red; the save-failure
  assertions pin the post-fix contract.
- journal-BLOCKER on return-value count fields: INDEPENDENTLY CONFIRMED
  legitimate scope discipline, not a contract gap. `cli analyze.rs:102`
  constructs `ProductionAnalysisStats` as a 9-field struct literal (no
  `..`), `analyze.rs:119` constructs `ProductionAnalysisResult` as a
  struct literal, `pipeline lib.rs:184` holds `Option<ProductionAnalysis
  Stats>`. Any new field breaks the cli build = outside touch-set, which the
  handoff declares "blocker, report don't fix". The handoff itself offered
  "count surfaced in return/LOG line" as the shape; the log-line shape
  shipped in-touch-set. Correct invocation of the contract.

### 3. tmp-name pre-existing claim (item 2) - TRUE -> spec row needs downgrade

- Reviewer byte-verified `git show "0861db0:crates/worker-core/src/tu_store.rs"`:
  `save_shard` at the ADD commit already wrote
  `.{}.{}.{}.tmp` = file_id + `process::id()` + process-local AtomicU64.
- `git log --oneline 0861db0..HEAD -- <file>` -> ONLY `3e48c51` (and that
  touched only the test-module tail, `@@ -702,4 +702,13 @@`). The tmp naming
  is unchanged since 0861db0. The proposed audit remedy (pid + "millesecond")
  is in fact superseded by the shipped scheme: pid + monotonic counter
  cannot collide across processes or threads without clock granularity.
- Therefore the audit's tmp-race "fix" rows are wrong: see finding 1.
  The implementer's lock-not-fix decision (no-code-without-red, T2 precedent)
  and the commit message refusing to claim a race fix are both correct.

### 4. Pool first-error / queue-restore pins (item 4) - honest characterization

- `pool.rs` actual behavior verified in source: collector keeps the FIRST
  received fatal error only (`if fatal_error.is_none()`, lines 93-97); the
  caller's queue is ALWAYS restored from the shared mutex (lines 105-107);
  workers drain until the queue is empty and break on exhausted retries
  (lines 136-173). Pins match this exactly:
  - `two_permanent_failures_surface_only_the_first_error_while_
    successes_complete` (3 workers, 2 fatal + 3 good): asserts XOR of which
    fatal is named - the correct formulation, since which worker reaches its
    fatal first is interleaving-dependent; asserting a specific winner would
    be a flaky over-claim. Exactly-one + "failed after 0 retries" + all-3-
    goods-executed + drained queue match the code path for every schedule.
  - `queue_is_restored_with_residue_when_both_workers_break_on_fatal_tasks`
    (2 workers, 2 fatal): good task unpopped, restored verbatim - matches
    lines 105-107 unconditionally.
- Both green at base (journal says so; no pool code changed in the commit -
  the pool.rs diff is a single pure-addition hunk `@@ -401,6 +401,236 @@`).
  Handoff rule "fix only if current behavior deviates" honored. The
  UTA-B6 "queue-restore unasserted" gap is now asserted.

### 5. Commit message truthfulness - ACCEPTED (orchestrator-authorized)

- Handoff gate-6 literal claims "checkpoint race" - which item 2 proved was
  never a bug. Shipping it verbatim would have claimed an unsailed fix.
  Shipped message (`fix(index,scheduler): uta-t5 fingerprint fail-closed +
  checkpoint foreign-skip/error surfacing; test(index,scheduler,worker-core):
  boundary pins + concurrency lock`) names exactly what shipped: fixes only
  for the two red->green sites (index, scheduler), worker-core listed under
  `test:` only (lock + pins, no fix). Truthful, journal-disclosed, T2
  precedent, orchestrator-authorized. Accepted.

### 6. tu_store_t5_pins.rs extraction is test-only - VERIFIED

- Declared `#[cfg(test)] #[path = "tu_store_t5_pins.rs"] mod t5_pins;` - not
  compiled in production. The file holds only `#[test]` fns + test helpers
  (`pins_ctx`, `ScriptedOutcomeResolver`, `scoped_fixture_context`).
- `tu_store.rs` diff = the tail of the existing `mod tests` (+ comment, + mod
  decl); production bytes byte-untouched (current lines 49-64 match the
  0861db0 blob). tu_store.rs now 714 < 1000 lines; zero test deletions
  (frozen locks intact). Child-module extraction keeps legal access to the
  private seams; as a `#[path]` sibling it adds no src surface.

### 7. "Good task starved at fatal==worker count" - PIN, not a latent bug

- The starvation is the inherent consequence of every worker dying on a
  fatal task: no survivor can drain. The pool surfaces it LOUDLY (Err +
  restored queue), and the pin asserts residue preservation with an explicit
  message ("not starved silently - it remains queued for the caller to
  retry"). Nothing silent, no wrong answer, no lost claim: a re-run
  re-analyzes from honest checkpoints. Handoff directed exactly this
  assertion. As a pin: honest. The engine-side consequence (local queue +
  `?` discards the restored residue when analyze() fails) goes to the
  ledger as observation - finding 3.

## Deviation dispositions (all declared in the journal)

- D1 generic `contexts_material` signature (harness step for the seam):
  ACCEPTED. Required to mirror the cache.rs AR-05 seam style the handoff
  itself demands; concrete callers compile unchanged (consumer check exit 0);
  red targets the bug, not the signature.
- D2 item 2 ships as lock, not fix: ACCEPTED. Pre-existing fix independently
  byte-verified at 0861db0; no red possible; T2 no-code-without-red.
  Consequence -> finding 1 (spec row downgrade, owner-side).
- D3 item 3 return-count fields journal-BLOCKED: ACCEPTED. Independently
  verified cli/pipeline coupling; handoff makes out-of-touch-set edits
  blockers-by-definition. Consequence -> finding 2.
- D4 save-failure test green-at-base (post-fix contract pin): ACCEPTED.
  Swallow has no in-process observable; the item-level red shipped on the
  foreign-id half; disclosed.
- D5 commit-message deviation from gate-6 literal: ACCEPTED. Truthfulness
  beats literalism; the literal would have lied. Orchestrator-authorized.
- D6 encoding gate run as `--all` (post-commit) instead of `--staged`:
  ACCEPTED. `--staged` is vacuous with commit_scoped's atomic staging;
  `--all` is the substantive superset; T1-T4 precedent; reviewer re-ran
  `--all` at HEAD: exit 0.
- D7 refactors forced by hygiene (persist_shard_checkpoints extraction;
  t5_pins extraction): ACCEPTED. No behavior delta (verified path-by-path);
  R1/R2 clean at HEAD.

## Findings (numbered; none blocking)

1. MINOR / closeout-routing: `docs/specs/unit-test-audit.md:90` ("shard
   tmp-name race ... include pid/millesecond in tmp name - fix") and the H5
   plan row `audit:127` ("fix UTA-B2, tmp-race") are factually wrong and
   need the UTA-B3-style downgrade: NOT REPRODUCIBLE - the tmp name carried
   pid + atomic counter from its birth commit `0861db0`, unchanged since
   (reviewer byte-check + follow-log). Spec is outside H5's touch-set;
   implementer correctly reported instead of fixing. Owner/orchestrator to
   downgrade both rows in the ledger docs commit (mirror of UTA-B3 handling).
2. MINOR / ledger deferred-findings: the UTA-B6 "counts in returned value"
   half is journal-BLOCKED (cli struct-literal coupling verified at
   `analyze.rs:102/119`, `pipeline lib.rs:184`). If the counts must reach
   the CLI report, a cross-crate handoff (scheduler + cli + pipeline) is
   required. Record in `tasks/milestone_ledger.md` deferred-findings.
3. MINOR / ledger observation: `ProductionAnalyzer::analyze` propagates the
   pool fatal error with `?` over a queue LOCAL to the function, so the
   restored residue (pinned in item 4) is discarded in production - the run
   fails loudly and a re-run redoes all TUs. Defensible (honest failure,
   checkpoints intact); record as observation only.
4. MINOR / journal thinness: final gate record line 208 ends "encoding_guard:
   `--all` exit 0 (recorded below)" with nothing below - dangling anchor; the
   gate-record note (line 186) likewise defers to a never-written line. The
   reviewer's own `--all` at HEAD (exit 0, capture 0 lines) substitutes.
5. MINOR / configuration: catch-all deny boilerplate
   (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`)
   refused this review's compound `;`+`echo` command, `python -c` probes, and
   `;`-chained exit-code capture. Every expected gate was still run - plain
   individual commands plus `distill.py`-carried exit codes. Same allowlist
   gap flagged by the T4 reviewer; an allowlist row for `python -c` (or
   exit-code echo) would remove the distill detour.

## Not run (explicit)

- `testrun.py --baseline`, `ui_smoke.py`, `bench_compare.py`: milestone
  closeout gates (AGENTS.md: fresh session / orchestrator only; H5 of 8
  hands). OWED at UTA closeout; nothing here substitutes.
- `encoding_guard.py --staged`: vacuous post-commit (clean tree); superseded
  by `--all` at HEAD (T1-T4 precedent).
- `golden_guard.py`: not applicable (prose lock clause; enforced via the
  13-deletion audit, all at fix sites).

Artifacts left UNTRACKED per T1-T4 precedent (reviewer git-write denied):
this file + the H5 addendum in `tasks/uta-execution-results.md`.
