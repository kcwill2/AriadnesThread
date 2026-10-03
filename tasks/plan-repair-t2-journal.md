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