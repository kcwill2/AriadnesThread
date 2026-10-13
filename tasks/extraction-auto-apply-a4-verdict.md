# A-4 Verdict - extraction-auto-apply: `EngineBackedExtractionApplier`, the swap install, the provider proofs

**Verdict: APPROVED.**

Reviewer hop, read-only with respect to source. Review range `7451c36..0fa6c39`,
HEAD `0fa6c39`, branch `main`, worktree clean at start and at end of the hop
(`python tools/checkpoint_guard.py status`).

This is the safety-critical hand - the first writer path behind a browser-facing
surface - so the adjudication below leans hardest on the four things that make it
safe: the delegated single write path, the dry-run/apply parity proof, the forced
rollback proof, and the token/mutex gate. All four hold. Three minor,
non-blocking findings are recorded; none requires a re-land.

## 1. Ground truth and commit-vs-touch-set (oldest first)

`python tools/checkpoint_guard.py status`: HEAD `0fa6c39`, branch `main`, dirty 0,
untracked none. Worktree census unchanged from the pre-hop record (the six
registered worktrees plus the temp pool entry; none of them is this hand's).

Range: `git log --oneline "7451c36..0fa6c39"` = `0fa6c39`, `ad33b11` (two commits;
`38457f2` and `7451c36` are the already-adjudicated A-3 verdict and journal
appendix and sit below the base).

**`ad33b11` (parent `7451c36`)** - judged alone. `3 files changed, 1248
insertions(+)`:
- `crates/cli/src/commands/ui.rs` `+7 0`
- `crates/cli/src/commands/ui/extraction_apply.rs` `+999 0` (NEW)
- `tasks/extraction-auto-apply-a4-journal.md` `+242 0`

The diff of `ui.rs` is exactly three hunks and nothing else: the module line
(`+mod extraction_apply;` at :36, directly after `mod extraction_reader;`), the
clone-before-move line (`+let applier_plans = reader_plans.clone();` immediately
after `let reader_plans = out.extraction_plans;`), and the install (four comment
lines plus `extraction_apply::install_with_generation(state, reader_epoch,
applier_plans);`) placed after `state.install_extraction_reader(Arc::new(reader));`.
Zero removed lines. The reader construction block itself is byte-identical, as the
handoff required. Measured at HEAD: `ui.rs` is **877 lines**, at or below the
frozen 880 ceiling; the A-1 lock
`crates/cli/src/commands/ui/boot_progress.rs::test_ui_rs_is_under_the_extraction_ceiling`
(asserts `line_count <= 880` over `include_str!("../ui.rs")`) is green inside the
gate-3 run, so the ceiling is machine-proven, not asserted.

**`0fa6c39` (parent `ad33b11`)** - journal only, `1 file changed, 28
insertions(+)`, the landing record plus gates 7-9 and the producer zero-diff
proof. Nothing outside `tasks/**`.

`git diff --name-only "7451c36..0fa6c39"` = exactly the three touch-set paths.
**No commit lands outside the declared touch-set.** This also proves, by
exclusion, every frozen lock in the handoff: `extraction_reader.rs` byte-identical,
the seven frozen cli test modules under `crates/cli/src/commands/ui/`
byte-identical, `crates/transform/**`, `crates/validate/**`, `crates/artifact/**`
untouched, `tests/golden/**` and `tests/fixtures/**` byte-frozen,
`tools/theme_token_baseline.json` diff-empty, no `Cargo.toml`/`Cargo.lock` change,
`resolve_bind_addr` and `--expose` in no diff (A8).

Pure ASCII: grep for non-ASCII bytes over `extraction_apply.rs` returns no
matches, and the seven added `ui.rs` lines are ASCII. `encoding_guard.py --all`
is silent exit 0 (fresh, below).

## 2. Deep adjudication of the named items

### 2.1 D6, the headline deviation - ACCEPTED, with one caveat recorded as finding 2

**The m20 characterization, independently verified.**
`tests/integration/tests/m20_apply_migrate_characterization_test.rs:179`
`apply_thing_plan_locks_transaction_outcome_and_disk_effects` provisions exactly
the fixture A-4's required test 1 uses (`tests/fixtures/basic`, :27) and asserts,
at :224-230:

> `assert_eq!(consumer_before, consumer_after, "current behavior: apply commits
> although the old-include replacement is a no-op (it searches for
> include/thing.h while sources contain the bare thing.h) - latent
> vacuous-rewrite bug, characterization only");`

plus `thing_impl_before == thing_impl_after`. That is a frozen suite asserting,
by name and with the reason spelled out, that an apply on this fixture modifies
no consumer. Independently confirmed on disk: `tests/fixtures/basic/tests/consumer.cpp:1`
and `tests/fixtures/basic/src/thing.cpp:1` both hold the bare `#include "thing.h"`,
while the plan target is `include/thing.h` (`plans.json`); `crates/cli/src/commands/transform.rs:330-334`
is where the mismatch is manufactured - consumer discovery pushes the discovery
target's spelling (`vec![target_file.clone()]`) as the rewrite's `old_includes`,
so the rewriter searches for `#include "include/thing.h"` that is not there.

**The delegation is faithful.** `apply_plan_with` (`transform.rs:162-223`) is the
only production path the applier uses. It builds the migration with the same
pieces the dry-run builds it with - `plan_rewrite_targets` then
`IncludeRewriter::new(...).build_preview(...)` (:181-187), the twin of the frozen
reader's `compute_dry_run` (`extraction_reader.rs:159-193`) - and hands it to
`TransactionApplier::apply_with`. The vacuous rewrite therefore lives entirely
inside `crates/transform`'s rewriter and the consumer-discovery spelling, three
layers below the applier. The applier cannot influence it from its layer without
authoring a second rewrite implementation, which the handoff forbids and A1
exists to prevent. Verified: `extraction_apply.rs:140` is the module's single
delegation call, inside `run_transaction`, and it passes exactly the arguments
the contract names (`&self.repo_root, plan, &self.generation.reduced_model`).

**The literal clause is unsatisfiable.** The handoff's required test 1 demands
"the set of MODIFIED files equals the dry-run's rewrites' `consumer_file` set"
while also demanding the approved dry-run carry consumer rows. On this fixture
m20 pins the first set to empty and the second set to two files. No honest test
can assert both. The implementer's diagnosis is correct and complete.

**The shipped proof.** Test 1 asserts, in order: `written` (the tree-walk
difference, artifact tree excluded) is non-empty; `written == approved_new_files`
field by field; `approved_consumers` is non-empty (non-vacuity of the dry-run's
consumer rows); `modified` is a subset of the approved consumer set, so an apply
that touched anything the user did not approve fails;
`outcome.new_files == written`; `committed == true`;
`output_text.ends_with("\nCommitted.")`; `plan_id` and `generation_epoch` round
trip. That is the strongest form of the A6 non-negotiable obtainable inside the
touch-set, and the safety-relevant half of the consumer clause (no unapproved
modification) is machine-locked.

Where I part company slightly with the implementer is the phrase "the strongest
obtainable inside the touch-set". The dry-run DTO does expose
`old_includes` per rewrite row (`crates/ui/src/dto/extraction.rs:167`), so a
positive faithfulness oracle was available at no cost: expected-modified = the
approved consumers whose on-disk text actually contains an `old_includes`
spelling. That would distinguish "the engine faithfully applied a vacuous
rewrite" from "the applier dropped the consumer rewrites", and it would
auto-restore to equality the day the bug is fixed. It was not built. This is
finding 2 below, minor, because the negative safety property is locked and the
bug itself is already characterized tree-wide by the frozen m20 suite.

**D6 is ACCEPTED.** The bug fix was deliberately not attempted, correctly: it
belongs to `crates/transform` and would require amending a frozen
characterization, both outside the touch-set.

### 2.2 The E2E proof (acceptance 1 and 2)

Required test 1 (`extraction_apply.rs:527-613`) boots a real fixture copy through
`boot_swapped_fixture` -> `build_ui_payload` -> `swap_into_state`, reads the
dry-run through the installed reader, mints through the installed applier,
applies, and compares whole-tree walks before and after. `written_relative` is
"present after, absent before", excluding `.ariadne` and `artifacts` components -
correct, because the outcome record lands under `artifacts/transaction`. The
`!written.is_empty()` guard makes the equality non-vacuous, so it cannot pass by
comparing two empty sets.

Required test 3 (`:669-747`) is the rollback proof and it is genuinely strong. It
fabricates an extraction unit whose `required_deps` name a header that exists
nowhere, runs the real machinery - `TransactionApplier::apply_with`,
`TierValidator::validate_structural`, `validate_generated_headers`, then
`TransactionApplier::rollback(&snapshot, &new_paths)` - and asserts four things:
`!new_paths.is_empty()` (the transaction really wrote), `assert_ne!(tree_hash,
before)` (the tree really changed), `verify_restored(...).is_ok()`, and
`tree_hash(&repo_root) == before`. The anti-vacuity pair means the final equality
cannot pass on a no-op. Both oracles are real content comparisons:
`TransactionApplier::verify_restored` compares a SHA-256 fingerprint over
path bytes, a presence marker and file content bytes
(`crates/validate/src/lib.rs:114-122`), and the test's own `tree_hash` is FNV-1a/64
over the sorted walk's path bytes, presence marker and content bytes.

The "budget-driven swap" between oracles is confirmed and correctly resolved: the
journal's Refactor section records that test 3's whole-tree hash oracle was
briefly removed for the line budget and then RESTORED. At HEAD it is present, and
tests 4 and 5 keep the content-map walk as their oracle, which is what the
handoff names for them ("the tree was not mutated a second time"). No proof was
quietly traded away.

### 2.3 A1 single write path (acceptance 5)

`grep -c "apply_plan_with(" crates/cli/src` = **3** at HEAD: the definition
(`transform.rs:162`), the CLI wrapper's call (`transform.rs:139`), and exactly one
delegation site in the applier (`extraction_apply.rs:140`). Required test 11
locks this by text, building the needle with `format!` so the test cannot match
itself. Production `TransactionApplier::apply_with` sites are exactly two -
`transform.rs:195` (inside the A-2 core) and the pre-existing
`crates/cli/src/optimize.rs:406`, which this hand never touched and which is not
in the range diff. `crates/ui/src` names no write token and gained no write path
in this range (A-3's module, already adjudicated, is unchanged here). The only
`TransactionApplier::apply_with` inside the applier is at `:713`, inside
`#[cfg(test)]` for required test 3, which the handoff mandated verbatim.

### 2.4 A4 mutex

`apply_lock_guard` (`:112-118`) is `try_lock` only: `WouldBlock` -> `None` ->
`Err(InProgress)` at the call site, `Poisoned` -> recovered inner guard via the
crate idiom. Applies are never queued, and the refusal happens at step (2),
before `consume_token` at step (3), so a refused apply cannot burn the approval.
The route runs the transaction on `spawn_blocking` (`crates/ui/src/routes/api_extraction_apply.rs:80`),
so a multi-second apply never blocks the async runtime. Required test 5
(`:782-815`) pre-holds the lock through the mandated `apply_lock_handle()` probe,
asserts `InProgress`, asserts the tree is untouched, drops the guard, and asserts
the SAME token then applies and commits - the honest-retry proof, complete.
Epoch-swap-mid-apply behaviour matches shape C5 by construction (the applier owns
its `Arc<AnalysisGeneration>`; a later apply hits a fresh registry) and the
required test 6 proves the epoch binding of consent; a literal "swap a generation
while a transaction is in flight" test is not in the handoff's list and is not
required.

### 2.5 A2/A4 token semantics

Registry shape `Mutex<HashMap<String, (String, u64)>>` exactly as contract C4.
Consumption is lookup plus remove under one lock hold (`consume_token`,
`:123-134`), and a wrong binding is refused with the entry left in place, so one
request's bad token cannot burn somebody else's approval - required test 7 locks
that with the `token_registry_contains` probe. Mint point returns `Some` only for
a plan resolvable in this generation (`plan_for`), tested for both directions in
required test 9. Token value is `apply-{epoch}-{plan_id}-{counter}`, counter
1-based per fresh applier, with the two-appliers-same-epoch determinism asserted
as a feature. `NotApproved` covers missing, unknown, replayed and foreign-plan;
`UnknownPlan` is checked before the lock and the registry is untouched
(required test 8, probe-verified).

### 2.6 ui.rs lock and the install-via-helper deviation

Acceptance 6 is met in full: 877 lines (<= 880) and the diff is the module line
plus the install and nothing else. D1 is the ceiling-safe route and I accept it:
the handoff's own snippet would have taken `ui.rs` from 870 to 885, past the
frozen 880 lock, while `install_with_generation` preserves every mandated element
verbatim - the epoch comes from the swap (`reader_epoch`), the plans from the
pre-move clone, the generation from `state.analysis()` with the same `expect`
message, and the install through `state.install_extraction_applier`. The
handoff's "~855" estimate base was stale at this rev; the A-1 commit and journal
say 870, and 870 + 7 = 877 is what landed.

## 3. Deviation dispositions

| Deviation | Disposition | Reasoning |
|---|---|---|
| D1 - install via `extraction_apply::install_with_generation(...)` | **ACCEPTED** | The inline snippet breaches the frozen 880 ceiling at the 870 base; the helper preserves every mandated element verbatim and lands at 877. The ceiling is a frozen lock and the estimate in the handoff was drift. |
| D2 - Red panic shapes | **ACCEPTED** | Three of the four mandated "verbatim" failure texts assume always-Ok answers from a skeleton the handoff also mandates to refuse unconditionally. Mutually unsatisfiable. The stems are kept verbatim and the values interpolate. The Red set is still exactly four named tests. A-3 D3 precedent. |
| D3 - literal mod site after `extraction_reader` | **ACCEPTED** | The handoff's "alphabetically after `mod extraction_reader;`" is internally contradictory (apply sorts before reader); the literal site - directly after the reader line, which the handoff names - is what landed. No behavior difference. |
| D4 - second probe `token_registry_contains` | **ACCEPTED** | Required test 8's registry-untouched claim must pass at Red under the "exactly four" clause, and it is unobservable without the probe. Test-only, `#[cfg(test)]`, documented as having no production caller. |
| D5 - `apply_lock: Arc<Mutex<()>>` | **ACCEPTED** | Forced by the handoff's own mandated probe signature `apply_lock_handle(&self) -> Arc<std::sync::Mutex<()>>`. Shared ownership is the only way to hand out the handle. |
| D6 - subset parity instead of consumer equality | **ACCEPTED** | Section 2.1. The literal clause is unsatisfiable against the frozen m20 characterization; the fix is out of touch-set; the A6 non-negotiable is untouched. Caveat is finding 2. |
| D7 - journal rider commit | **ACCEPTED** | Precedent W-1 D5 and A-2 D2. The landing commit carries the pre-commit journal state, so post-commit gate records legitimately ride a second journal-only commit. |
| R - refactor: comment compression + `LanguageCapabilities` annotation | **ACCEPTED** | The annotation `capability: fn(&ariadne_lang::LanguageCapabilities) -> bool` matches `crates/cli/src/registry.rs:93-98`'s `plan_backend` signature exactly. Comment compression is prose only; no assertion or behavior changed (the +11 test census and the whole-tree hash oracle both confirm). The file lands at 999 lines against the 1000 gate. |

## 4. Findings (all minor, non-blocking; none requires a re-land)

1. **MINOR - `crates/cli/src/commands/ui/extraction_apply.rs:146-170` and `:177-187`
   (A9 path carve-out).** The handoff's A9 clause says "Every string the applier
   puts into the DTO is sanitized to ASCII", but `outcome_dto` sanitizes
   `output_text`, `detail` and the fail/skip reasons and leaves `plan_id` and the
   `repo_relative` rendering of `new_files` unsanitized. This is not a blocker for
   three reasons I checked rather than assumed: shape A9's enumeration of record
   names token, tier rows, output text and refusal details and not paths; the
   frozen reader renders `new_files` with the same lossy `to_string_lossy()`
   (`extraction_reader.rs:398-408`), so sanitizing here would break acceptance 1's
   field-by-field parity; and required test 10 covers the mapping for both
   outcomes. The `plan_id` half is a one-line fix that nothing depends on.
   Disposition: the planner records the path carve-out as a dated ledger note;
   `ascii_only(&plan_id)` can ride any later touch of this file.

2. **MINOR - `crates/cli/src/commands/ui/extraction_apply.rs:559-590` (D6's
   justification is slightly overstated).** Test 1's consumer clause is negative
   only. The dry-run DTO exposes `old_includes` per rewrite row
   (`crates/ui/src/dto/extraction.rs:167`), so a positive faithfulness oracle was
   obtainable inside the touch-set at near-zero cost: expected-modified = the
   approved consumers whose on-disk text contains an `old_includes` spelling. That
   would prove the rewrite was faithfully attempted rather than silently dropped,
   and would restore equality automatically when the bug is fixed. Not built. Not
   a blocker because the safety-relevant property (no modification outside the
   approved set) is machine-locked and the bug itself is already characterized
   tree-wide by the frozen m20 suite. Disposition: planner records the oracle as a
   follow-up item attached to the D6 deferred row below.

3. **MINOR - `crates/cli/src/commands/ui/extraction_apply.rs:243-248`
   (cosmetic).** The doc comment on `ascii_only` is indented four spaces past the
   `fn`, and the body sits one level flatter than the rest of the module. No
   rustfmt or clippy gate exists in this repo and `hygiene_check.py` is exit 0, so
   nothing enforces it; recording it for the next touch of this file.

Observations recorded but not numbered as findings, because they are not defects
against the contract: test 1 does not assert the absence of deleted files (the
before/after maps are already in hand, so a one-line `before`-only-keys check
would extend the whole-tree guarantee); and no test drives a generation swap
while a transaction is in flight (shape C5's "not cancelled" claim follows from
the applier owning its `Arc<AnalysisGeneration>`, and the handoff's test list
does not ask for it).

## 5. Fresh gate results (reviewer hop, run now)

- `python tools/testrun.py --distill` - `### cargo test PASS (exit 0) [capture:
  C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 2828 lines]`
  **`1513 passed / 0 failed / 13 ignored / 112 suites`**
  `no failures`
  Canonical line quoted verbatim. Reconciled against the anchor
  `1502 passed / 0 failed / 13 ignored / 112 suites` = exactly **+11** from A-4,
  the eleven required tests, suites held at 112, zero movement elsewhere. All
  eleven confirmed by name in the capture, all `... ok`.
- `python -m unittest discover -s tools -p "*_tests.py"` - **`Ran 219 tests in
  23.086s` / `OK`**.
- `python tools/ui_smoke.py` (via `python tools/distill.py -- python tools/ui_smoke.py`) -
  `### generic PASS (exit 0)`, `boot+route checks clean (cold fixture, attempt 1,
  1.1s)`, `ui_smoke: PASS`.
- `python tools/hygiene_check.py` - exit 0. R1/R2 zero. The R3 rows are the
  pre-existing warn-only baseline; zero rows name `extraction_apply.rs`, and the
  two `ui.rs` `p` rows (:643, :783) are pre-existing lines outside this hand's
  hunks.
- `python tools/encoding_guard.py --all` - `(no output)`, exit 0. Also fresh
  `--staged` exit 0.
- `python tools/theme_token_check.py --check-added --base 7451c36` -
  `### generic PASS (exit 0)`, capture 0 lines (silent exit 0).
- Producer zero-diff proof (bench stance): `python tools/distill.py -- git diff
  --numstat "7451c36..0fa6c39" -- crates/cli/tests` -> `### generic PASS (exit 0)`,
  capture 0 lines. The producer set is byte-identical across the hand's range.
- `python tools/bench_compare.py` (via the distilling entry point) -
  `### generic FAIL (exit 2)`, `ERROR parse format drift: documented phase row(s)
  missing from bench output (never substituting zeros)`. Deterministically
  blocked by the `DOCUMENTED_PHASES` tuple at `tools/bench_compare.py:84`; the
  ledger already carries this as an OPERATOR blocker and `tools/**` is forbidden
  to this hop, so I did not edit it and no timing number is attributed to this
  hand.
- Compensating gate, fresh: `python tools/distill.py -- python tools/bench_compare.py
  --bench stage5-memory --runs 6` -> `### generic PASS (exit 0)`. Verbatim
  headline lines from the capture:
  `stage5-memory sample accounting: accepted 6 of 6 invocations, rejected 0
  (> 8% peak-operand divergence from per-slot median)`
  `stage5 peak RSS (debug, N=640)                                   |     1067.2`
  `stage5 peak RSS (debug, N=1280)                                  |     4370.0`
  `stage5 RSS growth ratio (debug, N=640->1280)                     |        4.1`
  `warm deserialize peak (debug)                                    |        7.2`
  plus `WARNING no baseline for machine class
  'Windows-AMD64-AMD64_Family_26_Model_68_Stepping_0_AuthenticAMD::stage5-memory';
  regression compare skipped.` - the expected no-baseline state on this machine,
  so this is a fresh 6-run memory sample, not a regression verdict.

Refused commands (verbatim catch-all, per the reviewer rule): `python tools/hygiene_check.py`
with output redirection plus chained exit-code echoes; `python tools/bench_compare.py`
direct; `python -c ...` byte scans; `git status --porcelain`; `git grep -c "apply_plan_with(" <rev>`.
Each was worked around through the distilling entry point (`distill.py -- <cmd>`),
plain `git status`, and the dedicated grep tool over the clean worktree, which
equals HEAD. No gate I was expected to run was skipped, and no weaker evidence
was substituted.

## 6. Execution-results addendum (against the acceptance criteria)

1. **E2E proof non-negotiable** - MET. `test_a_valid_token_applies_the_plan_end_to_end`
   ... ok (capture line 211). WRITTEN == `new_files` field by field, non-vacuous,
   through real boot + swap + applier on an isolated fixture copy.
2. **Rollback proof non-negotiable** - MET. `test_a_failed_tier_restores_the_tree_byte_identically`
   ... ok (capture line 131). `verify_restored` ok AND whole-tree hash equality,
   with the anti-vacuity pair so the equality is not a no-op.
3. **Consumed token, held lock** - MET. Tests 4 and 5 ok (capture lines 212, 210).
4. **Epoch / foreign plan / unknown plan** - MET. Tests 6, 7, 8 ok (capture lines
   203, 201, 206), including registry-untouched on both refusal paths.
5. **Caller census = 3** - MET. Measured at HEAD; locked by test 11 ok (capture
   line 127). `crates/cli/src/optimize.rs:406` pre-existing and untouched.
6. **`ui.rs` <= 880, diff = module line + install** - MET. 877 lines measured;
   diff is 7 added, 0 removed, exactly the two edit points.
7. **Gates 3, 4, 5 exit 0; hygiene exit 0; encoding exit 0** - MET by the fresh
   workspace run above, which subsumes and exceeds the scoped runs.
8. **Commit touch-set exactly the three paths** - MET for both commits
   (section 1).

Not run in this hop, stated explicitly: `cargo clippy` (no clippy gate exists for
this milestone and the handoff's gate list does not name it); the wizard
front-end and C8 amendment work, which is A-5's and is unlanded; and any
release-profile (`--release`) smoke or bench form, which the handoff does not
mandate.

## 7. Statement on the latent vacuous-rewrite bug (D6) - for the planner's closeout write

**Yes. This must become a dated ledger deferred row.** It is a real, reproducible
product defect with a named location, and it is currently recorded only inside a
frozen characterization test's assertion message - which means nobody planning
future work will find it by reading the ledger.

The row should read, approximately: `extraction-auto-apply A-4 (reviewer verdict,
2026-10-13) - vacuous consumer rewrite: an apply's consumer rewrites are a no-op
on the basic fixture because consumer discovery reports the plan target's
spelling \`include/thing.h\` as the rewrite's \`old_includes\`
(\`crates/cli/src/commands/transform.rs:330-334\`) while the sources hold the bare
\`#include \"thing.h\"\`; locked as characterization only by
\`m20_apply_migrate_characterization_test.rs:179\`
\`apply_thing_plan_locks_transaction_outcome_and_disk_effects\`. A6's consumer half
(\"the consumer rewrites match\", shape section 2) is therefore not satisfiable and
A-4 required test 1 asserts subset parity plus a non-vacuity guard instead
(journal D6). Fix owner: \`crates/transform\` (\`IncludeRewriter\`) plus the
consumer-discovery \`old_includes\` spelling; requires amending the m20
characterization. Follow-up oracle recommended: expected-modified = approved
consumers whose on-disk text contains an \`old_includes\` spelling.`

Two further dated notes belong in the same closeout write, both from this verdict:
the A9 path carve-out (finding 1) and the `plan_id` sanitization one-liner.

## 8. Handoff back to the planner

A-4 is approved for the ledger as landed at `0fa6c39` (landing `ad33b11` + journal
rider `0fa6c39`). The three findings are non-blocking and are recorded rather
than fixed, per the reviewer's read-only grant. The D6 deferred row, the A9
carve-out note and the follow-up oracle go into the milestone closeout write.
The next implementer hop commits this verdict as its duty 1.
