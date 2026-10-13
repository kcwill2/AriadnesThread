# A-2 Handoff - extraction-auto-apply: extract the `apply_plan_with` transactional core in `commands/transform.rs`

Depends on nothing. This hand de-risks the milestone's central bet: the CLI
and the UI must later share ONE transactional implementation, so the core
exists before any applier can call it.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-auto-apply-shape.md` sections 2 (invariant A1, its
   caller-census oracle), 3 (C1 verbatim - the struct and function
   signatures), 5 (module map, A-2 row), 8 (hygiene notes).
   `tasks/extraction-auto-apply-kickoff.md` section 3 (item 10).
3. `crates/cli/src/commands/transform.rs` lines 1-9 (imports), 129-192
   (`apply_transform_plan`), 194-212 (`load_plan_for_action`), 216-235
   (`plan_rewrite_targets`), 309-339 (`find_plan_consumers`), 341-415
   (`commit_or_rollback`, `report_rollback`).
4. The three frozen consumers of the wrapper:
   `tests/integration/tests/m10_validation_test.rs:67-86` (success path,
   asserts the `Committed` transaction artifact),
   `tests/integration/tests/m20_apply_migrate_characterization_test.rs:180-260`
   (the `Plan {} not found` error text and the missing-path error),
   `tests/integration/tests/m23_symbol_isolation_test.rs:580-600`.
5. `crates/validate/src/lib.rs:47-56` (`ApplyOutcome`), `:58-81`
   (`format_output`), `:150-201` (`TransactionApplier::apply_with`),
   `:297-321` (rollback) - READ ONLY, never edited.

## Objective and preserved invariants

Extract one transactional core out of `apply_transform_plan` and make the
CLI verb a thin caller of it. No behavior change to the CLI.

The contract of record, verbatim from shape C1:

```rust
/// One apply transaction's report: the tier outcome plus the files the
/// transaction created. Returned, never printed - the CLI wrapper prints
/// it, the UI route serializes it.
pub struct ApplyTransactionReport {
    pub outcome: ariadne_validate::ApplyOutcome,
    pub new_files: Vec<PathBuf>,
}

pub fn apply_plan_with(
    repo_path: &PathBuf,
    plan: &ariadne_cluster::plan::OptimizationPlan,
    reduced_model: &ariadne_graph::ReducedSemanticModel,
) -> Result<ApplyTransactionReport>
```

`reduced_model` is a parameter on purpose: the CLI wrapper loads the
artifact model (today's behavior) while A-4's UI applier passes the
generation's in-memory model - the exact model the approved dry-run was
computed from (shape A6 / risk R10).

Invariants this hand must preserve:

- `apply_transform_plan` keeps its EXACT signature
  (`repo_path: &PathBuf, plan_id: &str`) and becomes exactly: locate
  repository, prepare artifacts, `load_plan_for_action`, load the reduced
  model, print `Applying {}...`, call the core, print
  `outcome.format_output()`.
- Success-path stdout is BYTE-IDENTICAL to today: the `Applying {}...`
  line followed by `ApplyOutcome::format_output()`. Nothing else prints.
- The unknown-plan error text `Plan {} not found` is preserved verbatim (the
  m20 lock asserts its content).
- The core PRINTS NOTHING. `commit_or_rollback` and `report_rollback` stop
  printing; they return the report and still write the artifact records
  (`artifacts/transaction`, today's `transform.rs:394`, `:413`).
- Only ONE transactional implementation exists: no second call to
  `TransactionApplier::apply_with` is authored anywhere
  (`crates/validate/**` is not edited at all).
- Single write path (shape A1): at the end of this hand the cli crate holds
  exactly TWO occurrences of `apply_plan_with(` - the definition and the
  wrapper's one call. A third site forks the transactional path.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- tasks/extraction-auto-apply-a2-journal.md (checkpoint journal)
- crates/cli/src/commands/transform.rs (the core extraction; +~90/-~25 net)

You MUST NOT touch: `crates/cli/src/commands/ui.rs`, every other file under
`crates/cli/**`, `crates/ui/**`, `crates/validate/**`, `crates/transform/**`,
`crates/artifact/**`, `tests/**`, `tools/**`, any asset, `Cargo.toml`,
`Cargo.lock`. No `git stash`, no branch change.

## Frozen locks

- `tests/integration/tests/m10_validation_test.rs`,
  `m20_apply_migrate_characterization_test.rs` and
  `m23_symbol_isolation_test.rs` stay BYTE-IDENTICAL and green. Their
  canonical line must be identical before and after this hand (this is a
  behavior-preserving extraction).
- `tests/golden/**` byte-locked (`UPDATE_GOLDENS=1` FORBIDDEN);
  `tests/fixtures/**` byte-frozen. The apply writes only inside the
  integration tests' own temp copies.
- `crates/validate/src/lib.rs` (929 lines) is READ ONLY by this hand.
- `tools/theme_token_baseline.json` diff-empty; zero added hex (no asset is
  touched).

## What to build

1. `ApplyTransactionReport` as written above (doc comment verbatim).
2. `apply_plan_with(repo_path, plan, reduced_model)` holding the whole
   transaction: locate the repository, `ArtifactManager::prepare_for_repo`,
   `IsolationEngine::new(reduced_model)`, `repository_source_reader`, the
   plan-language extraction backend, `plan_rewrite_targets`,
   `find_plan_consumers`, `IncludeRewriter::build_preview`, the
   plan-language rewriting backend, `TransactionApplier::apply_with`, the
   plugin staging decision for the tier-2 header directory, and
   `commit_or_rollback` - now returning `ApplyTransactionReport`. Every
   existing comment and inline citation inside the moved code moves with it.
3. `apply_transform_plan` becomes the thin wrapper described above. The
   `Applying {}...` print moves BEFORE the transaction, which is why the
   error path now prints it first; the m10/m20/m23 suites assert error
   CONTENT and success-path stdout only - verify this at gate 5.
4. `commit_or_rollback` / `report_rollback` lose their `println!` calls and
   return the outcome; the wrapper prints `outcome.format_output()` once.
   The artifact-record writes stay exactly where they are.
5. Keep the public surface additive: `repository_source_reader`,
   `extract_plan_headers`, `build_migration_preview`, `plan_rewrite_targets`,
   `plan_consumer_discovery_targets`, `plan_target_consumers`,
   `load_plan_for_action` and their signatures are UNCHANGED (the reader
   provider and the frozen characterization suites depend on them).

## RGB protocol (per module, in this file order)

Red: author the new `#[cfg(test)] mod tests` at the bottom of
`transform.rs` FIRST. It reads `transform.rs` as text (its own file) and
fails on the pre-extraction tree. Expected failures, verbatim:

- `test_apply_plan_with_is_the_shared_transactional_core` panics with
  `C1: transform.rs must define the shared transactional core
  apply_plan_with`
- the second assertion in the same test panics with
  `C1: the core must return ApplyTransactionReport, never print it`

Both needles are built with `format!` (the
`progress_contract_tests.rs:73` idiom) so the test's own source cannot
match itself, and no doc comment in this file may contain the contiguous
text `pub fn apply_plan_with` or `pub struct ApplyTransactionReport`. A
compile failure is NOT the Red.

Green: perform the extraction per "What to build". Re-run until exit 0.

Refactor: `apply_plan_with` and `commit_or_rollback` each stay under 100
lines (extract helpers if the tier sequence grows); one responsibility per
function; no single-letter locals; no banned identifiers.

## Required tests (new, in `transform.rs`'s `#[cfg(test)] mod tests`)

1. `test_apply_plan_with_is_the_shared_transactional_core`: assert the file
   contains `format!("pub fn {}", "apply_plan_with")` and
   `format!("pub struct {}", "ApplyTransactionReport")`. These two
   assertions are the Red and they stay green forever after, so the core
   cannot silently regress into a print-only wrapper.

That is the whole new test module for this hand: the behavioral proof is
the three UNEDITED integration apply suites, whose canonical line must not
move by a single test.

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned identifier names, no
single-letter locals, intention-revealing names (shape section 9's name
list is binding: `ApplyTransactionReport`, `apply_plan_with`).
`transform.rs` 415 -> ~470 lines, well under the ceiling. `python
tools/hygiene_check.py` exit 0.

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/testrun.py --distill -p ariadne-cli` (baseline canonical
   line recorded verbatim in the journal)
2. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m10_validation_test --test m20_apply_migrate_characterization_test
   --test m23_symbol_isolation_test` (baseline canonical line for the three
   frozen apply suites, recorded verbatim)
3. `python tools/testrun.py --distill -p ariadne-cli` (Red: exit 1; the two
   named assertions, verbatim panic lines, in the journal; no other failure)
4. `python tools/testrun.py --distill -p ariadne-cli` (Green: exit 0;
   canonical line = baseline + 1 passed)
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m10_validation_test --test m20_apply_migrate_characterization_test
   --test m23_symbol_isolation_test` (exit 0; canonical line MUST equal
   gate 2's line byte for byte - any movement is a behavior change and a
   STOP)
6. `python tools/hygiene_check.py` (exit 0)
7. `python tools/encoding_guard.py --staged` (exit 0)
8. `python tools/commit_scoped.py --scope crates/cli/src/commands/transform.rs
   --scope tasks/extraction-auto-apply-a2-journal.md -m "refactor(cli):
   extract the apply_plan_with transactional core shared by CLI and UI"`
   (never `git add .`)
9. `python tools/encoding_guard.py --all` (post-commit, exit 0)

Gate count: 9 (the commit-cycle trio is gates 7-9). Bench stance: this hand
touches `crates/cli/src` so the DOCUMENTED_PHASES dated row applies - the
producer set under `crates/cli/tests/` is byte-identical across this hand's
range (zero-diff producer proof in the journal), the timing gate is
deterministically blocked at `tools/bench_compare.py:84`, and NO timing
number is attributed to this hand. Do not run `bench_compare.py`; the
reviewer hop owns it.

## Acceptance (machine-checkable)

1. `git show --stat` shows exactly `crates/cli/src/commands/transform.rs`
   plus the journal.
2. `apply_transform_plan`'s signature is unchanged
   (`grep -n "pub fn apply_transform_plan" crates/cli/src/commands/transform.rs`
   prints the same line as the start rev).
3. `grep -c "apply_plan_with(" crates/cli/src/commands/transform.rs`
   returns 2 (the definition plus the wrapper call). This is the A1
   caller census of record; re-run it at preflight AND at the end.
4. Gate 5's canonical line is identical to gate 2's.
5. `python tools/hygiene_check.py` exit 0.
6. `python tools/encoding_guard.py --all` exit 0.
7. The m10/m20/m23 files appear in no hunk of this hand's diff.

## Discipline clauses

- UTF-8: Write/Edit tools only; never `Set-Content`/`Out-File`/`>`/
  `Get-Content | -replace`; pure-ASCII authored content; no em dashes.
- Touch only the touch-set; out-of-scope breakage: report, do not fix. In
  particular, if a `crates/ui` or `tests/` consumer of the new core breaks,
  report it - A-3/A-4 own those files.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`) are a reviewer
  blocker: distilling entry points only.
- F14 standing rule 1: the caller census for every name this hand adds
  (`apply_plan_with`, `ApplyTransactionReport`) is enumerated from the tree
  at the dispatch base with an expected count and an explicit STOP
  condition. At HEAD `023558c` both counts are 0 outside `transform.rs`.
- Checkpoint the journal after each Green gate.
- Context exhaustion: STOP with a disk-state inventory; successor resumes
  via `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-auto-apply-a2-handoff.md --scope ... --cmd ...`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence (the two named
   assertions with verbatim panic lines). 3. Green evidence (distill digest
   header lines + canonical lines for gates 2, 4 and 5 + capture path).
   4. The measured `transform.rs` line count and the census output from
   acceptance item 3. 5. Deviations + reasons. 6. Out-of-scope
   observations. No raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the touch-set above exactly.
