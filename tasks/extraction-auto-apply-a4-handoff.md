# A-4 Handoff - extraction-auto-apply: `EngineBackedExtractionApplier`, the swap install, the provider proofs

Depends on A-1, A-2 AND A-3 all landed. This is the hand that writes a real
file through a browser-facing surface, so it carries the milestone's
non-negotiable proofs: the fixture end-to-end apply and the forced rollback.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-auto-apply-shape.md` sections 2 (A1-A6 oracles), 3
   (C1 `apply_plan_with`, C4 token semantics, C5 mutex and epoch model), 5
   (module map, A-4 row), 6 (the split contingency), 8 (hygiene notes), 9
   (names). `tasks/extraction-auto-apply-kickoff.md` sections 3 (items 4-5),
   7, 10 (item 4 and item 9).
3. `crates/cli/src/commands/ui/extraction_reader.rs` - the structural model
   for this hand: struct + `new` (:44-68), the `plan_for` idiom (:79-81),
   the read-only delegation style, and the test module (:429-839) with its
   `fixture_copy` (:515-526), `boot_swapped_fixture` (:472-492) and
   `tree_hash` (:544+) idioms.
4. `crates/cli/src/commands/ui.rs` `swap_into_state` (:606-633) - the install
   site - and the module list (:25-28).
5. `crates/ui/src/routes/api_extraction_apply.rs` (A-3's trait, refusal enum
   and handler) and `crates/ui/src/state.rs` (A-3's applier seam).
6. `crates/validate/src/lib.rs` :150-201 (`apply_with`), :297-331 (`rollback`,
   `verify_restored`), :400-489 (the three tiers) - READ ONLY.

## Objective and preserved invariants

Implement the mutation seam in the cli crate and install it with the
generation, so the A-3 route has a real applier behind it. Every mutation
delegates to the extracted `apply_plan_with` core (A-2) - this hand authors
no second transactional implementation.

The contract this hand implements (shape C4/C5, binding):

- **Token value**: deterministic ASCII, epoch-bound and registry-keyed:
  `format!("apply-{epoch}-{plan_id}-{counter}")` with a per-applier
  `AtomicU64` counter. It is a gate, not a credential.
- **Registry**: `Mutex<HashMap<String, (String, u64)>>` inside the applier,
  keyed token to (plan_id, epoch). An epoch swap installs a new applier and
  therefore discards every outstanding token.
- **Mint point**: `mint_apply_token(plan_id)` returns `Some(token)` when the
  plan resolves in this generation, else `None`.
- **Serialization**: the applier holds `apply_lock: std::sync::Mutex<()>`;
  the apply path `try_lock`s it. A held lock yields `InProgress`
  immediately - applies are never queued.
- **Single use**: consumption happens atomically when the apply transaction
  starts. A lock refusal does not consume the token, so an honest retry
  works; a successful apply consumes it, so a second apply requires a fresh
  dry-run fetch.
- **Epoch binding**: only the live epoch can apply (the route refuses 409
  otherwise), and the token carries its epoch, so a generation swap discards
  every outstanding approval.

The apply order inside the implementation (derived from C4/C5; the tests
lock it):

1. resolve the plan id in this generation -> `Err(UnknownPlan)` when absent;
2. `try_lock` the generation's apply lock -> `Err(InProgress)` when held,
   leaving the token registry untouched;
3. atomically consume the token (lookup + remove under the registry mutex)
   -> `Err(NotApproved)` when it is missing, unknown, replayed or bound to
   another plan or epoch;
4. run the transaction through `apply_plan_with(&self.repo_root, plan,
   &self.generation.reduced_model)`;
5. an `Err` from the transaction is `Err(Failed(<ascii text>))` - the tree
   is already rolled back by validate's AR-04 machinery;
6. an `Ok(report)` maps into the DTO: `committed`, the three tier rows, the
   created files rendered repo-relative (the same lossy rendering
   `staged_write_paths` uses), and the verbatim `format_output()` text.

Invariants this hand must preserve:

- **Single write path (A1).** At the end of this hand
  `grep -c "apply_plan_with(" crates/cli/src` returns 3: the definition and
  the CLI wrapper's call (A-2's census of record, expected 2 after A-2) plus
  exactly ONE delegation site - this applier. The milestone census is
  therefore "exactly one delegation site per caller, no third caller", and
  it is re-stated in the acceptance section.
- **A8, bind default unchanged.** No route, handler or flag widens the
  exposure surface. `resolve_bind_addr` and `--expose` appear in no diff;
  this hand adds no bind code.
- **`ui.rs` headroom (the W-2a F2 discharge).** The install is the ONLY
  `ui.rs` growth, it is bounded to the `swap_into_state` block plus one
  module line, and the file must stay at or below 880 lines - the ceiling
  A-1's lock proves.
- **ASCII (A9).** Every string the applier puts into the DTO is sanitized to
  ASCII, including the verbatim `format_output()` text and any refusal
  detail.
- **No new dependency edge.** No `Cargo.toml` changes; the applier uses only
  what `extraction_reader.rs` already uses.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- tasks/extraction-auto-apply-a4-journal.md (checkpoint journal)
- crates/cli/src/commands/ui/extraction_apply.rs (NEW: the applier, the
  token registry, the mutex, the DTO mapping, and its test module)
- crates/cli/src/commands/ui.rs (ONE module line `mod extraction_apply;` in
  the existing block at :25-28, and the install inside `swap_into_state`,
  +~14 lines total)

You MUST NOT touch: `crates/cli/src/commands/transform.rs`,
`crates/cli/src/commands/ui/extraction_reader.rs`, every other file under
`crates/cli/**`, `crates/ui/**`, `tests/**`, `tools/**`, any asset,
`Cargo.toml`, `Cargo.lock`. No `git stash`, no branch change.

## Frozen locks

- `crates/cli/src/commands/ui/extraction_reader.rs` BYTE-IDENTICAL (the
  shape's module map forbids it): this hand duplicates its fixture-test
  helpers into the new module rather than sharing them. That duplication is
  the priced, deliberate trade; a shared cli test-helpers extraction is a
  separate hygiene hop and must NOT be attempted here.
- `crates/validate/**`, `crates/transform/**`, `crates/artifact/**` READ
  ONLY - no second transactional implementation exists.
- `tests/golden/**` byte-locked (`UPDATE_GOLDENS=1` FORBIDDEN);
  `tests/fixtures/**` byte-frozen. Every test operates on an isolated
  fixture COPY in a temp dir; the tracked fixture is never written.
- `tools/theme_token_baseline.json` diff-empty; zero added hex (no asset is
  touched).
- The seven frozen cli test modules under `crates/cli/src/commands/ui/` stay
  BYTE-IDENTICAL (A-1's discipline carries forward).

## What to build

1. `mod extraction_apply;` in `ui.rs`'s module block (alphabetically after
   `mod extraction_reader;`).
2. `EngineBackedExtractionApplier` (shape section 9's name is binding):
   fields `epoch: u64`, `repo_root: PathBuf`, `plans: Vec<OptimizationPlan>`,
   `generation: Arc<AnalysisGeneration>`,
   `token_registry: Mutex<HashMap<String, (String, u64)>>`,
   `apply_lock: std::sync::Mutex<()>`, `token_counter: AtomicU64`. The
   `new(epoch, repo_root, plans, generation)` constructor mirrors
   `EngineBackedExtractionReader::new` exactly (same parameter order and
   doc style).
3. `impl ExtractionPlanApplier for EngineBackedExtractionApplier` with
   `epoch`, `mint_apply_token` and `apply` per the six-step order above, and
   the private helpers `plan_for`, `token_registry_guard`,
   `apply_lock_guard`, `consume_token`, `outcome_dto`, `tier_row`,
   `repo_relative`, `ascii_only`. Poison recovery uses the
   `unwrap_or_else(|poisoned| poisoned.into_inner())` idiom the crate
   already uses (never `unwrap()`).
4. The `swap_into_state` install. TWO edit points, because
   `ui.rs:609` binds `let reader_plans = out.extraction_plans;` and
   `ui.rs:618` MOVES it into the reader - cloning after that point is a
   use-after-move compile error:
   - immediately after the `let reader_plans = ...;` line, add
     `let applier_plans = reader_plans.clone();`;
   - immediately after `state.install_extraction_reader(Arc::new(reader));`,
     add:
     ```rust
     // A-4: the applier belongs to THIS generation exactly like the reader -
     // installed in the same sequence, from the same plans and the same
     // generation Arc. Every UI-triggered mutation goes through it, and it
     // delegates to the `apply_plan_with` core (A1: one write path).
     let applier = extraction_apply::EngineBackedExtractionApplier::new(
         reader_epoch,
         state.repo_root.clone(),
         applier_plans,
         state
             .analysis()
             .expect("install_analysis publishes the generation it just installed"),
     );
     state.install_extraction_applier(Arc::new(applier));
     ```
   The reader construction itself stays byte-identical. Total `ui.rs` growth:
   the module line plus the clone line plus this block (~+14 lines), and the
   file must stay at or below 880 lines.
5. A `#[cfg(test)] pub fn apply_lock_handle(&self) -> Arc<std::sync::Mutex<()>>`
   test-only probe (the `consumer_discovery_runs` precedent at
   `extraction_reader.rs:74-77`: doc comment must state that no production
   caller exists, so an ungated inherent method would be dead code in every
   non-test build). This is how the held-lock test pre-holds the lock.

## RGB protocol (per module, in this file order)

Red: land the compile-complete skeleton FIRST - the struct with every
field, `new`, the trait impl whose `apply` answers `Err(NotApproved)`
unconditionally and whose `mint_apply_token` returns a deterministic token
for a resolvable plan - plus the `swap_into_state` install and the module
line. Then author the test module below and run gate 2. Expected failures,
verbatim shape:

- `test_a_valid_token_applies_the_plan_end_to_end`:
  `expected an approved apply, got Err(NotApproved)`
- `test_a_consumed_token_cannot_apply_twice`:
  `the second apply with the same token must be refused, got Ok`
- `test_a_held_lock_refuses_without_consuming_the_token`:
  `expected InProgress while the lock is held, got Ok`
- `test_a_token_minted_for_another_epoch_is_refused`:
  `expected NotApproved for a token from another epoch, got Ok`

A compile failure is NOT the Red. The 503/404 route-level shapes are A-3's
already-green locks and must stay green here untouched.

Green: implement the six-step order, the token registry, the mutex and the
DTO mapping. Re-run until exit 0.

Refactor: `apply` and the mapping helpers stay under 100 lines; the test
module's helpers stay under 100 lines each.

## Required tests (in `extraction_apply.rs`'s `#[cfg(test)] mod tests`)

Providers are built through the REAL boot + `swap_into_state` wiring (the
`boot_swapped_fixture` idiom, duplicated here on purpose - see Frozen
locks). Fixture: the `basic` C++ fixture, PLAN-001.

1. `test_a_valid_token_applies_the_plan_end_to_end` (A6 - THE proof): boot
   an isolated `basic` copy, walk the tree, read the dry-run through the
   installed reader for the first proposal id, mint a token through the
   installed applier, call `apply(plan_id, &token)`, walk the tree again,
   and assert the set of files the transaction ACTUALLY WROTE equals the
   approved dry-run's `new_files` field by field (repo-relative,
   lossy-rendered, sorted). The written set is the tree-walk difference
   EXCLUDING the repo's artifact tree (`.ariadne` / `artifacts` components):
   the apply's own outcome record lands under `artifacts/transaction` and is
   not a created source file. Also assert the set of MODIFIED files equals
   the dry-run's rewrites' `consumer_file` set, that the outcome DTO's
   `new_files` equals the written set, that `committed` is true, and that
   `output_text` ends with the committed line of `format_output()`. The
   tree-walk helpers are the `tree_hash`/`collect_tree` idiom from
   `extraction_reader.rs:542+`, duplicated per the Frozen-locks note.
2. `test_a_rolled_back_outcome_maps_to_the_honest_dto` (A5, DTO half):
   fabricate `ApplyOutcome::RolledBack { failed_tier: <a tier-2 failure>,
   tiers_passed: <a tier-1 pass> }` and feed it through the applier's own
   `outcome_dto` mapping. Assert `committed == false`, exactly three tier
   rows with the contract names ("Structural", "GeneratedHeaders",
   "AffectedTUs"), the failing row's status is "fail" with the failure
   reason in `detail`, and `output_text` equals
   `outcome.format_output()` byte for byte.
3. `test_a_failed_tier_restores_the_tree_byte_identically` (A5,
   restoration half; the fabricated-extraction oracle): on an isolated
   fixture copy, fabricate an `ExtractionResult` whose units carry a
   `required_deps` entry naming a header that does NOT exist on disk (so
   tier 2 fails), run the REAL machinery -
   `TransactionApplier::apply_with`, `TierValidator::validate_structural`,
   `TierValidator::validate_generated_headers`, then
   `TransactionApplier::rollback(&snapshot, &new_paths)` - and assert
   `TransactionApplier::verify_restored(&snapshot).is_ok()` AND the
   whole-tree hash equals the pre-apply hash (the m10 `hash_dir` idiom).
   This is the proof the rollback story is real, not asserted.
4. `test_a_consumed_token_cannot_apply_twice` (A2): apply once with a fresh
   token (succeeds), then apply again with the SAME token and assert
   `Err(NotApproved)` - and assert the tree was not mutated a second time
   (the consumer rewrites did not change again).
5. `test_a_held_lock_refuses_without_consuming_the_token` (A4): take the
   `apply_lock_handle()`, lock it in the test thread, call `apply` and
   assert `Err(InProgress)`; drop the guard and assert the SAME token now
   applies successfully. This is the honest-retry proof.
6. `test_a_token_minted_for_another_epoch_is_refused` (A3): build a second
   applier for a different epoch, mint from it, and apply through the first
   applier: `Err(NotApproved)`.
7. `test_a_foreign_plan_token_is_refused` (A2): mint for plan A, apply with
   plan B's id: `Err(NotApproved)`.
8. `test_an_unknown_plan_is_refused` (A1's boundary): mint/apply an id the
   generation does not hold: `Err(UnknownPlan)`, and the token registry is
   untouched.
9. `test_the_token_value_is_deterministic_epoch_and_plan_bound` (C4): the
   minted token equals `format!("apply-{epoch}-{plan_id}-{counter}")` for
   the counter sequence, and two appliers for the same epoch mint the same
   first token (determinism is a feature, not a bug - the gate is the
   single-use registry).
10. `test_every_dto_string_is_ascii` (A9): for both the committed and the
    rolled-back mapping, every string in the DTO is ASCII.
11. Lock (A1, the caller census of record):
    `include_str!("../transform.rs")` contains exactly TWO occurrences of
    `apply_plan_with(` - the definition and the CLI wrapper's call - and
    `include_str!("extraction_apply.rs")` (this file, by text) contains
    exactly ONE. A third site in either file forks the transactional path.

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned identifier names, no
single-letter locals, intention-revealing names (shape section 9's name
list is binding). Measured estimates against HEAD `023558c` (post A-1):
`extraction_apply.rs` NEW ~480-600 lines including the duplicated test
helpers; `ui.rs` ~855 -> ~869. Both far under 1000. `python
tools/hygiene_check.py` exit 0.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/testrun.py --distill -p ariadne-cli` (baseline canonical
   line recorded verbatim in the journal)
2. `python tools/testrun.py --distill -p ariadne-cli` (Red: exit 1; exactly
   the four named failing tests, verbatim panic lines, in the journal)
3. `python tools/testrun.py --distill -p ariadne-cli` (Green: exit 0;
   canonical line = baseline + the new tests, no other movement)
4. `python tools/testrun.py --distill -p ariadne-ui` (exit 0: the consumer
   crate is unbroken by the install; its canonical line is recorded)
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m21_run_ui_smoke_test` (exit 0: the real `run_ui` boot path through
   `swap_into_state` still reaches ready with an applier installed; this is
   the F14 reverse-dependency proof for the wiring)
6. `python tools/hygiene_check.py` (exit 0)
7. `python tools/encoding_guard.py --staged` (exit 0)
8. `python tools/commit_scoped.py --scope crates/cli/src/commands/ui/extraction_apply.rs
   --scope crates/cli/src/commands/ui.rs --scope
   tasks/extraction-auto-apply-a4-journal.md -m "feat(cli): engine-backed
   extraction applier (token registry, apply mutex, dry-run-then-apply
   delegation) installed with the generation"` (never `git add .`)
9. `python tools/encoding_guard.py --all` (post-commit, exit 0)

Gate count: 9 (the commit-cycle trio is gates 7-9). Bench stance: this hand
touches `crates/cli/src` so the DOCUMENTED_PHASES dated row applies - the
producer set under `crates/cli/tests/` is byte-identical across this hand's
range (zero-diff producer proof in the journal), the timing gate is
deterministically blocked at `tools/bench_compare.py:84`, and NO timing
number is attributed to this hand. The OPTIONAL compensating
`--bench stage5-memory --runs 6` form (the W-2a precedent,
`tasks/ui-extraction-wizard-w2a-verdict.md`) MAY be run through the
distilling entry point; if it is, the journal must declare it as a dated
overshoot row per the M28 T0b finding-1 law. It is not part of the gate
budget and it is not required.

## Acceptance (machine-checkable)

1. **The fixture end-to-end proof is non-negotiable**: required test 1 shows
   the files an apply WRITES equals the approved dry-run's `new_files`
   (field by field), on an isolated copy of the tracked fixture, through the
   real boot + swap + applier path. A weaker proof (an in-memory comparison,
   a hand-built expectation, or a skipped tier) fails this hand.
2. **The rollback proof is non-negotiable**: required test 3 shows a forced
   tier-2 failure restoring the tree byte-identically, asserted through
   `TransactionApplier::verify_restored` AND a whole-tree hash equality.
3. A consumed token cannot apply twice (required test 4); a held lock
   refuses without consuming the token and the same token then succeeds
   (required test 5).
4. A token minted for another epoch or another plan is refused (required
   tests 6-7); an unknown plan is refused with the registry untouched
   (required test 8).
5. `grep -c "apply_plan_with(" crates/cli/src` returns 3 - the definition,
   the CLI wrapper's call, and exactly ONE delegation site in the applier -
   and `crates/cli/src/commands/ui/extraction_apply.rs` contains exactly
   one occurrence (required test 11).
6. `crates/cli/src/commands/ui.rs` is at or below 880 lines and its diff is
   the module line plus the `swap_into_state` install and nothing else.
7. Gates 3, 4 and 5 exit 0; `python tools/hygiene_check.py` exit 0;
   `python tools/encoding_guard.py --all` exit 0.
8. The commit's touch-set is exactly the three files in gate 8 (reviewer
   commit-vs-touch-set enforcement).

## Discipline clauses

- UTF-8: Write/Edit tools only; never `Set-Content`/`Out-File`/`>`/
  `Get-Content | -replace`; pure-ASCII authored content; no em dashes.
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`) are a reviewer
  blocker: distilling entry points only.
- NEVER run an apply against the working repository or any path outside a
  temp dir. Every mutation in this hand happens inside a
  `tempfile::TempDir` fixture copy.
- If this hand exceeds its budget, the split line is INSIDE the provider:
  the token registry and the mutex first, the `apply_plan_with` delegation
  second (shape section 6) - because the gate is what makes the milestone
  safe rather than merely functional. STOP-and-report rather than
  compressing the proofs.
- Checkpoint the journal after each Green gate (module, commit hash,
  deviation, lock finding, and the measured `ui.rs` line count).
- Context exhaustion: STOP with a disk-state inventory; successor resumes
  via `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-auto-apply-a4-handoff.md --scope ... --cmd ...`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence (the four named failing
   tests with verbatim panic lines). 3. Green evidence (distill digest
   header lines + canonical lines for gates 3, 4 and 5 + capture path).
   4. The end-to-end proof's written-file list and the dry-run's
   `new_files`, quoted. 5. The measured `ui.rs` line count. 6. Deviations
   + reasons. 7. Out-of-scope observations. No raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the touch-set above exactly.
