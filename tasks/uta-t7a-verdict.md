# UTA-T7a Verdict - cli error-swallow and residual-arithmetic fixes

Handoff: `tasks/uta-t7a-handoff.md`. Commits: `60b76f3` (fix) + `ff47047`
(journal rider, +7 journal-only) on base `e95a6d5`. Reviewer session ran the
full review-gate procedure fresh.

## VERDICT: APPROVED

0 blockers. 5 minor findings, none requiring rework. All 5 declared deviations
ACCEPTED on independent merits.

Canonical fresh gate line (reviewer's own run, `python tools/testrun.py
--distill -- -p ariadne-cli`):

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 242 lines]
147 passed / 0 failed / 8 ignored / 8 suites
```

Matches the expected 147/0/8/8; the 8 ignores are the pre-existing bench
tests (journal-disclosed, consistent with the H6-era cli suite shape).

## 1. Commit-vs-touch-set - PASS

`git diff --name-only e95a6d5..HEAD` -> exactly 6 paths:

```
crates/cli/src/commands/discover.rs   (+20/-16)
crates/cli/src/commands/measure.rs    (+15/-7)
crates/cli/src/commands/transform.rs  (+38/-5)
crates/cli/src/optimize.rs            (+126/-14)
crates/cli/tests/uta_t7a_discovery_error_test.rs (+64/-0)
tasks/uta-t7a-journal.md              (+110, rider +7)
```

The four named-by-basename src files are EXACTLY the touched ones; no scope
widening. Glob of `crates/cli/src/*.rs` shows only `registry.rs`,
`optimize.rs`, `main.rs`, `lib.rs` at that level - the handoff's literal
`src/transform.rs|measure.rs|discover.rs` do not exist, so the `commands/`
subdir is the only possible home of the audit-cited lines, and the diff
hunk headers confirm the cited lines resolve inside those files at base
(transform.rs 230-233 = the `.or_else(..).unwrap_or_default()` block,
measure.rs:11 and discover.rs:87 = the two `discover().ok()` lines).
No other crate, no Cargo.toml/lock, no other cli src touched (H7b's half is
untouched). Commit message matches the handoff gate-4 `-m` literal verbatim.

## 2. Per-commit review, oldest-first - PASS

- `60b76f3` reviewed file-by-file in full (+373/-42 total incl. journal).
  Deletions (42) are ALL production swallow/prefix lines: the bare
  `committed_transformations` counter, the `take(committed)` prefix, the
  `if let Ok(json)` + `let _ =` pair, the `.or_else(..).unwrap_or_default()`
  eraser, and the two `.ok()`/Option-dance/`unwrap_or_default()` fallbacks.
  ZERO deletions or edits inside any existing test -> frozen locks byte-kept.
- `ff47047` journal-only (+7), records the gate checkpoint.

## 3. Independent adjudication of the five required items

### Fix 1 (UTA-B8 residual arithmetic) - CONFIRMED CORRECT

Reviewer recomputation: opportunities 600+400=1000; old code
`safe_plans.take(committed_transformations=2)` subtracted the plan1+plan2
reductions (90+180)=270 -> residual 730. The journal's red observed EXACTLY
730 - the prefix-count defect model reproduced to the digit. Committed-set
{plan1,plan3} subtracts 90+270=360 -> 640, pinned by the shipped test plus
all-committed (460) and zero-committed (1000) sanity pins. Commit-loop
tracking verified: `committed_plans.push(plan.clone())` fires ONLY inside
`if outcome.validation_passed`; `committed_transformations` is now DERIVED
from `committed_plans.len()`, so count and set cannot drift or double-count
by construction. Single caller of `residual_collateral_load`, updated.

### Fixes 2-4 (UTA-B7 error propagation) - REDS GENUINE, FIXES MINIMAL

- Fix 2 (`save_optimization_report`): pre-fix `let _ = fs::write(..)`
  reported Ok over an un-written report; red occupied `report.json` with a
  directory (deterministically unwritable) and observed the unqualified Ok.
  Post-fix both serde and write propagate via `?`; the CALLER
  (optimize.rs:198) already routes through `?`, so the Err fails the command
  - no new swallow at the call site. Handoff's "Choose Err" honored; doc
  comment records the "Ok means written" contract.
- Fix 3 (`load_plan_for_action`): the old `.or_else(..).unwrap_or_default()`
  erased the replan Err to an empty vec, so the reload error ALONE surfaced -
  the journal's red ("chain held only 'Failed to read plans.bin'") is the
  exact mechanical consequence of that code, verified against the deleted
  lines. New form: replan Err is the base error, `with_context` embeds the
  load error text -> the chain carries BOTH the replan root cause and the
  reload error, per the handoff's "context-chain the reload error with the
  replan error text". The test asserts `chain.contains("Failed to locate
  repository path")` - text independently confirmed live at
  `crates/repo/src/lib.rs:29`. Err surfaces the ROOT cause, not a generic
  wrap. Replan-success behavior preserved (empty vec -> reload of the
  just-replanned artifacts).
- Fix 4 (measure.rs / discover.rs `.ok()`): both propagate via
  `with_context(...)?`; the Option dance and the `unwrap_or_default()`
  empty-hash fallback are DELETED, not bypassed - the empty-file
  fingerprint path no longer exists at the cli sites. Red: `0 passed /
  2 failed / 0 ignored / 1 suites`, both legs at the expect-Err line
  (Ok over failed discovery) - the audit's exact symptom. Journal records
  what discover.rs:87 gated: (a) `with_repo_discovery` catalog view,
  (b) the `Fingerprint::inputs` hash list feeding `contexts_fingerprint`
  - both verified in the diff.

### The share_mode(0) injection - honest deterministic mechanism, NOT flaky

Independently verified against the upstream walker:
`crates/repo/src/discovery.rs:43-46` swallows ONLY `read_dir` errors
(`Err(_) => continue`); per-file `fs::read` failures PROPAGATE via
`compute_file_hash` (:16-22) at the `?` on line 77. An exclusive handle
(share_mode 0) on a walked header therefore makes `Repository::discover()`
fail with ERROR_SHARING_VIOLATION, deterministically. The test is also
SELF-VALIDATING: post-fix it can only pass if the injected condition truly
errors `discover()`, and the stage-specific chain assertion
(`contains("repository discovery failed")`) rules out an incidental early
Err (locate/prepare_for_repo) making it pass. `#![cfg(windows)]` is the
semantically required gate (sharing mode is a Win32-only semantic); CI runs
on this Windows host (disclosed as deviation 2). See finding 5 for the
closeout note.

### Deviations - all five ACCEPTED

- D1 `commands/` subdir paths: ACCEPTED. Named paths do not exist as written;
  citations resolve at the cited line numbers in the real files; four named
  basenames == four touched files; commit `--scope` globs mirror the real
  paths. Spec-side citation drift is finding 2 (owner side, not implementer).
- D2 Windows-gated integration tests: ACCEPTED. Mechanism determinism proven
  above; gating is honest disclosure, not flake cover.
- D3 "uses plan1 only" correction: ACCEPTED. The handoff's parenthetical was
  imprecise - the actual pre-fix value is the first-N prefix (plan1+plan2 =
  730, reviewer-recomputed). The journal's correction is right and the 640
  assertion discriminates under either model.
- D4 No synthetic replan seam: ACCEPTED on merits. The handoff's "inject
  failing replan seam" was the suggested means; the machine-checkable
  requirement (chain contains the replan cause) is met by a REAL failing
  replan (real `plan_extractions` against a vanished repo, fails fast in
  `Repository::locate`). A seam would have tested the seam; the real path
  tests production, and the fix stayed smaller.
- D5 Test file placement in `crates/cli/tests/`: ACCEPTED - explicitly
  allowed ("tests in crates/cli/tests/ if needed").

### Honest absences - both verified true

- Serde-leg "unfakeable" claim: TRUE. Every `OptimizationReport` field
  (incl. `Duration`) serializes plainly; forcing a serde failure would
  require a hostile `Serialize` impl - a test of serde_json, not of this
  code. The leg is nonetheless covered by the same `?`. No synthetic test
  invented, disclosed.
- `measure.rs` `load_load_artifacts().ok()` untouched: verified UNCHANGED at
  line 36; it degrades `collateral_load` to `Option::None` (a documented
  artifact-absence fallback), a different error class from the discovery
  swallow; not one of the handoff's named sites. Correctly left for the
  ledger/H7b owner rather than scope-crept.
- The repo-crate `Err(_) => continue` walk tolerance: outside touch-set,
  reported as out-of-scope observation (a) -> routed to ledger below.

## 4. Frozen locks - held verbatim

- transform.rs C1 lock (`test_apply_plan_with_is_the_shared_transactional_
core`): byte-identical in current file vs diff context; new test appended
  AFTER it; the fix's added lines contain no needle text (`format!`-built
  needles are self-avoiding).
- extraction_apply test-11 lock (`apply_plan_with(` exactly 2x in
  transform.rs): current count == 2 (lines 139, 162); zero occurrences in
  the fix's added lines -> base count was also 2; extraction_apply.rs not in
  the commit at all.
- lib.rs `test_handle_measure` (lib.rs:672): file absent from the commit;
  it constructs unlocked temp repos, so fix 4 cannot affect it; passes in
  the fresh green run.
- All three locks pass in the reviewer's fresh 147/0 run.

## 5. Findings (minor, non-blocking)

1. Journal gate checkpoint states "+273/-42" for `60b76f3`; the code+tests
   diff is +263/-42 and the whole commit (incl. the journal's own 110 lines)
   is +373/-42. Minor self-report misstatement - matches the T2/T3/T5
   numstat-slip class; no behavior impact.
2. Path-citation drift at the contract level: handoff items 2-4 and the
   spec's UTA-B7 rows cite `crates/cli/src/{transform,measure,discover}.rs`,
   which do not exist (files live under `commands/`). Adjudicated on the
   implementer side, but `docs/specs/unit-test-audit.md` rows 57-61 (+ the
   handoff template that copied them) should get the `commands/` prefix at
   closeout - same class as T6's manager.rs:170->172 finding.
3. Handoff item 4's "corrupt/unreadable ROOT scenario" does not literally
   surface even post-fix: an unreadable root DIRECTORY still yields an
   empty-Ok walk upstream (`discovery.rs:43-46`). The shipped scenario (a
   locked file INSIDE the root) genuinely errors the contractual
   `repository.discover()` call at both named sites, so the contract's
   named-site rule is fully enforced; the residual upstream class is
   honestly reported (journal observation (a)) and routes to ledger
   deferred-findings: neutralize the empty-fingerprint class in
   `crates/repo::discovery`.
4. Allowlist configuration: the byte-level `python -c` probe was refused
   with the catch-all deny boilerplate
   (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`) -
   same shape as T5/T6. Evidence was obtained with equal rigor via the grep
   tool (lock needle count), diff line numbers (base cite resolution), and
   the passing lock tests in the fresh capture. No gate skipped.
5. `#![cfg(windows)]` means the UTA-B7c integration coverage compiles to
   zero tests on a non-Windows host. Honest today (CI is this host); if CI
   ever moves, this coverage vanishes SILENTLY - worth one closeout line.

## 6. Report-evidence check - PASS

Journal is digest-shaped throughout: canonical lines, failing lists, capture
paths (`target/testrun-capture.txt`), exit witnesses, zero raw dumps. Gates
used the distilling entry point (`testrun.py --distill`). Gate-3 sequencing
(`--staged` post-commit after `commit --only` reset the index, `--all`
pre-commit carrying the real weight) mirrors the accepted T1-T6 precedent;
reviewer's fresh `--all` at HEAD exit 0 supersedes it.

## 7. Not run / owed

- `testrun.py --baseline`, `ui_smoke.py`, `bench_compare.py`: NOT run at
  this hand - milestone-closeout debt, orchestrator's fresh-session gates
  (T1-T6 precedent; H7b still mutates the tree).
- `encoding_guard.py --staged`: vacuous post-commit (clean tree); superseded
  by the reviewer's `--all` exit 0.
- `golden_guard.py`: not applicable (prose lock clause; enforced via the
  42-deletion audit - all deletions are production fix lines).

## 8. Ledger / closeout routing

- Deferred-findings candidates: upstream `discover_repository_files`
  empty-Ok-on-unreadable-entry class (crates/repo, journal observation (a));
  `load_load_artifacts().ok()` owner decision (journal "deliberately NOT
  done").
- Spec corrections: finding 2 (commands/ prefix in UTA-B7 citations).
- Windows-coverage note: finding 5.
- Worktree census: the 8 untracked `tasks/*.md` files are the prior reviewers'
  verdict/results artifacts (T1-T6 precedent, disclosed), not this hand's
  working state.
