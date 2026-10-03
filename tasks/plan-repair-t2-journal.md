# Plan Repair T2 journal -- D.2 ADR-0001, the language plugin boundary

Append-only checkpoint log. One short entry per gate discharged.

## Checkpoint 1 -- content written (post-Green)

Created `docs/adr/0001-language-plugin-boundary.md` (new `docs/adr/` directory) and this
journal. Both are in the T2 touch-set. No source file was touched, so no TDD surface applies.

## Checkpoint 2 -- gates discharged by inspection

Shell is denied in implementer sessions, so the six handoff gates were discharged with Read,
Grep, and Glob instead of being executed:

1. Touch-set: verified by tool-call ledger -- exactly two Write calls, both into the
   touch-set. `git status` itself NOT run (no shell); reported as such.
2. Staged-content count: N/A, nothing staged (orchestrator commits).
3. Pure ASCII: PASS. Grep `[^\x00-\x7F]` over the new file returned no matches, with a
   positive control (the same pattern found 12 non-ASCII lines in `crates/cli/src/registry.rs`).
4. `tools/hygiene_check.py` exit 0: not executed (no shell). Discharged logically instead:
   its `R2_SCANNED_SUFFIXES` is `.rs`/`.py`/`.html`/`.js`, so a Markdown-only change cannot
   alter the R1/R2 result, and this task changed no source file.
5. Reference resolution: every cited path, type, trait, and test-function name was resolved
   with Glob or Grep before it was written. No unresolved reference.
6. Section order: PASS. Grep `^##` returned the seven required `##` headings in the required
   order, interleaved with three `###` subsections under `## Decision`.

## Checkpoint 3 -- deliberate accuracy calls

Two claims from the handoff were softened rather than copied, both for honesty:

- The goal sentence "0 edits to the core pipeline crates" is reproduced verbatim, but is
  immediately qualified: the single registry entry lands in `crates/pipeline/src/registry.rs`,
  which IS a core-crate file (the guard-exempt composition root). The accurate claim is "zero
  edits to core stage logic".
- `registries_enumerate_identical_languages` is close to tautological today, because
  `crates/cli/src/registry.rs` delegates to `build_default_registry`, so both sides of the
  assertion are the same object. The ADR says so and explains why the test is still kept.

Files left uncommitted for the orchestrator, per the override in the task message.

## Checkpoint 4 -- follow-up amendment (CLI planning-hook exception)

Triggered by T4 (`docs/adding-a-language.md`), which found a language-identity dispatch that
the ADR did not mention. Purely additive amendment to
`docs/adr/0001-language-plugin-boundary.md`; no existing wording or section order changed.

Added one `### Known exception: per-language planning hooks in the CLI` subsection inside
`## Machine-checkable enforcement`, one accepted-cost bullet in `## Consequences`, and one
clause in `## What this ADR does not decide`. File grew 268 -> 325 lines.

Verified before writing, by Read/Grep only (no shell):
- `planning_hooks` is at `crates/cli/src/commands/plan.rs:62-88`, called from
  `pub fn plan_extractions` (line 192) at line 210 -- production, not test.
- The CLI registry doc comment forbids "a concrete plugin type"; `plan.rs` calls plugin-crate
  *functions*, not constructing a plugin. The ADR records both the literal reading and the
  intent rather than asserting a flat contradiction.
- `GUARD_CORE_CRATES` is 7 entries, `crates/cli` absent -- guard cannot see the file.
- Two ADDITIONAL plugin-crate reference sites found beyond the reported one:
  `crates/cli/src/optimize.rs:272-281` and `crates/cli/src/commands/ui.rs:359-368`, both
  hardcoding the C++ hook pair with no language `match`. Recorded as extra edit sites; they are
  NOT R3 violations.
- Narrowed the opening claim from "one place in the tree" to "one place in the CLI", because
  that is the scope actually verified by the grep.
- Non-ASCII re-scan of the amended file: clean.

Left uncommitted for the orchestrator.

## Checkpoint 5 -- known-exception subsection completed

A verification pass found the exception section was incomplete. Read-only re-verification in
`crates/cli/src` (Grep for the four hook-pair symbols, then Read of each hit and of the
enclosing `fn`/`mod tests` boundaries) confirmed 16 hits, classified 9 production dispatch hits
forming 5 sites across 3 files, 1 production import, and 6 test-side hits.

The missing site was `crates/cli/src/optimize.rs:247` -- inside
`analyze_collateral_opportunities` (declared line 212, `mod tests` begins line 461, so it is
production), a `.filter()` applying `cpp_aggregation_filter` unconditionally. Distinct from the
already-recorded `plan_safe_extractions` `OptimizationPlanner::new` call at lines 272-281; both
are live on the `optimize` path. Also added the `crates/cli/src/registry.rs:51-52` error-string
clause, which is explicitly NOT a dispatch site.

Amended only the known-exception subsection and the one Consequences bullet added in Checkpoint
4. File 325 -> 355 lines; no section order or pre-existing claim altered. Two arithmetic errors
in my own first draft ("nine of the sixteen" where five, and "Nine plus one plus six") were caught
on read-back and corrected before reporting.

Left uncommitted for the orchestrator.

## Checkpoint 6 -- sixth item recorded as a modelling literal (final amendment)

Recorded `crates/cli/src/commands/plan.rs:231`, which passes `ariadne_lang::CyclePolicy::Reject`
to the planner for every language from production `plan_extractions`. Typed explicitly as a
**modelling literal, not a dispatch site**, so the five-site count and the 16-hit accounting are
untouched.

Verified by Read before writing: line 231 and its inline comment at lines 227-230; the comment's
substance (informational to the planner; SCC co-proposal path shared by design; stance steers
report-side classification only); and that `cycle_policy_variants_share_the_scc_coproposal_path_for_now`
exists at `crates/cluster/tests/planner_injection.rs:334`, referenced from
`crates/cluster/src/planner.rs:43`. Modelling values re-confirmed from
`crates/lang-cpp/src/plugin.rs:208` (Reject) and `crates/lang-scala/src/plugin.rs:118`
(TolerateWithinPackage).

Note: the source comment at plan.rs:228 contains an em-dash, so the ADR paraphrases the comment
rather than quoting it, keeping the new file pure ASCII. No defect claim is made -- the ADR states
the shared SCC path is deliberate and test-locked, and frames the item as something a new language
needs to be told about.

Amended only the known-exception subsection and the one Consequences bullet. File 355 -> 375
lines. Final amendment; no further trivia escalated. Left uncommitted for the orchestrator.

## Checkpoint 7 -- corrective edit, false clause removed (closes the reviewer gate)

A reviewer found that Checkpoint 6 asserted a FALSE consequence: that "report-side cycle
classification therefore uses the C++ stance for every language". I independently confirmed the
reviewer right on every point, and the defect was mine -- I had verified the substance of the
source *comment* rather than the actual data path.

Verified by Read/Grep before editing:
- `cycle_policy` in `crates/cluster/src/planner.rs` has exactly five occurrences: 43 (doc), 49
  (field declaration), 96 (constructor parameter), 106 (field initializer), 261 (doc reference).
  No read anywhere in the crate. Write-only, as claimed.
- `cycle_policy_by_file` is defined at `crates/pipeline/src/lib.rs:748-763`; line 758 evaluates
  `registry.plugin_for_id(record.language).map(|plugin| plugin.modeling().cycle_policy)` with
  `.unwrap_or(CyclePolicy::Reject)` at 759 for unclaimed files. Called at 327, fed to
  `generate_report_with_cycle_policies` at 328.
- `docs/adr/0002-...:222` does state the shipped Scala cycle policy tolerates cycles within a
  package, so the false clause contradicted a sibling ADR.

Correction: the false clause was deleted and the paragraph now records the verified mechanism --
the hardcode is inert, report-side classification is registry-sourced, a Scala run already uses
`TolerateWithinPackage` -- and states the real forward-looking hazard (the literal becomes a live
override the moment any consumer reads the planner field). Non-defect framing preserved; the
inert field is intentional per the source comment. The Consequences bullet was aligned to the same
mechanism. Five-site count, 16-hit accounting, and modelling-literal typing all untouched.
File 375 -> 392 lines. Left uncommitted for the orchestrator.