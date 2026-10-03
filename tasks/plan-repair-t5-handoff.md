# Plan Repair T5 Handoff — amend the multi-language plan to match what landed

Binding inputs (read in this order, targeted reads only):

1. This file (complete contract).
2. `plans/multi-language-support-implementation_plan.md` (596 lines) -- read ONLY these
   ranges. Do NOT read the whole file:
   - **lines 1-4** (the Status/Date/Baseline header)
   - **lines 60-75** (section 2 Goals / Non-goals / Success criteria)
   - **lines 430-437** (the "Deferred, specified but not executed here" subsection holding M23 and M24)
   - **lines 538-596** (the end of section 8, all of section 9 the todo list, the sequencing note, and the trailing lines)
3. `plans/m23-symbol-isolation-kickoff.md` -- read ONLY lines 1-10, 43-48, and 170-178
   (base, D-M23 amendments, Gate-G3 verdicts, final gate, follow-up backlog).
4. `Plan.md` -- confirm task T1 landed: `git log --oneline -1` and the M15-M23 records
   at the end of the file. Do not edit it.
5. `docs/adr/0001-language-plugin-boundary.md` and `docs/adr/0002-symbol-placement-and-isolation.md`
   -- read ONLY their headings and their "What this ADR does not decide" sections. You
   will link to them; you must not restate them.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `plans/multi-language-support-implementation_plan.md` (the only file you may change)
- `tasks/plan-repair-t5-handoff.md` (this file)
- `tasks/plan-repair-t5-journal.md` (checkpoint journal; always in-set)

You MUST NOT touch: `crates/**`, `tests/**`, `tools/**`, `docs/**` (including the two
ADRs and the guide, which tasks T2-T4 own), `Plan.md`, or any other file under
`plans/`. No `git stash`, no branch change, no `git add .`.

## Preconditions

Tasks T1-T4 run before you. Verify before you start:

```powershell
git log --oneline -5
Test-Path docs/adr/0001-language-plugin-boundary.md                 # must be True
Test-Path docs/adr/0002-symbol-placement-and-isolation.md           # must be True
Test-Path docs/adding-a-language.md                                  # must be True
```

If any is missing, STOP and report -- do not create it.

## What to build

Six edits. Each is a *reconciliation*, not a rewrite: this document is the historical
record of the plan, so preserve its planning-tense prose and add dated notes rather than
silently rewriting history.

### Edit 1 -- the Status/Date/Baseline header (lines 3-4)

It currently reads `Status: **DRAFT for approval** (no source changed yet)` against
baseline `664e014`. That is now false on both counts. Replace the status with something
like:

```markdown
Status: **EXECUTED -- closed through M23** (2026-10-03). Plan approved and executed
M17-M23; M24 remains parked (see "Deferred" in section 5 and "Sequencing after M23"
below). Final gate at M23 closure: `cargo test --workspace` = 1088 pass / 0 fail /
3 ignored, verified at `f9bcf69`.
```

Keep the `Date:` line and extend it with the amendment trail rather than replacing it,
so the reader can see when each phase was decided.

### Edit 2 -- section 2 non-goals (line 68)

The non-goal line lists both `Scala transforms (M23)` and `dynamic plugin loading
(M24)`. M23 has since been executed, so listing it there is stale. **Do not delete the
line** -- it records what the plan's execution window excluded. Annotate it instead, in
a way that makes the scope distinction explicit: M23 was excluded from *this plan's
window* and was subsequently executed as its own gated milestone; M24 remains excluded
and is parked. Add a single sentence, no more.

### Edit 3 -- the Deferred subsection (lines 430-436), M23 bullet

Add a dated completion note to the M23 bullet stating that it was executed as a gated
milestone under `plans/m23-symbol-isolation-kickoff.md`, closed at `36b182a`, final gate
1088/0/3 at `f9bcf69`, with both Gate-G3 lanes APPROVED. Name the two things it
deliberately did not do, because they are the reason its follow-up backlog exists: real
compiler invocation was not wired (amendment D-M23-12, after reviewer finding F3 that
extracted units are not self-contained), and a `source_file` provenance refinement was
ruled incomplete-not-incorrect and deferred. Do not restate the whole kickoff.

### Edit 4 -- the M24 bullet (line 437): reclassify it

This is the substantive edit. The bullet currently reads:

> **M24 -- out-of-process plugin host** (`libloading` / WASM). Worth it only when a
> plugin is built by another team; the trait boundary here is the precondition, nothing
> more.

Replace it with a version that: (a) states plainly that M24 is **parked, not scheduled**;
(b) **corrects the trigger**, because "another team" is a proxy for the three conditions
that would actually justify it, and none of the three currently holds:

| Condition | Status in this repository |
| --- | --- |
| The plugin needs a heavyweight third-party dependency that must not be taken in-process (binary size, compile time, licensing) | Not present. `ariadne-lang-scala` has no third-party runtime dependency; its parser is hand-written by design (assumption A3). |
| Fault isolation -- a plugin crash must not kill an analysis run | Not present. Analysis is in-process and single-threaded. |
| Independent release cadence -- a plugin ships without relinking the main binary | Not present. Both plugins are built and released with this repository. |

(c) notes the cost, which is the reason it is a decision and not a chore: a
runtime-loaded plugin escapes the compile-time dependency graph, so review rule R1's
`plugin_crates_do_not_depend_on_core_pipeline` check -- which reads the plugin
`Cargo.toml` manifests -- can no longer police it. A dylib or WASM host therefore
**weakens the machine-checkable boundary** this plan spent M17-M18 building, and
requires its own ADR plus an explicit answer to how the guard is preserved. Point to
`docs/adr/0001-language-plugin-boundary.md`.

Keep the table to 3 rows and the whole bullet under ~25 lines.

### Edit 5 -- section 9: add the missing M23 block and check off Docs

Section 9 currently has M17, M18, M19-M22, and Docs. **M23 is absent from the todo list
entirely** -- it was specified in section 5 and executed under a separate kickoff, and
the todo list was never updated. Add an `**M23 -- Symbol isolation, shared engine
(C++ + Scala)**` block with `[x]` items for the slices that actually landed. Derive the
slice list from `plans/m23-symbol-isolation-kickoff.md` (its task breakdown and the
Gate-G3 record) and from the M23 commit subjects -- do not invent slice names.

Then change the four `Docs` checkboxes D.1, D.2, D.3, D.4 from `- [ ]` to `- [x]`, and
append the landing provenance for each (which commit carried it). Verify the commit
hashes with `git log` before writing them; if a hash cannot be resolved, mark the item
done and write `landed: <file>` instead of guessing a hash.

### Edit 6 -- add a "Sequencing after M23" note

Append it after the existing `**Sequencing note:**` paragraph near the end of section 9.
It records the decision taken at M23 closure, in at most 12 lines:

1. **Plan repair first.** The project plan had no record of M15-M23 -- `Plan.md`
   stopped at M14 -- so there was no single source of truth for milestone state. The
   ADRs and the "adding a language" guide did not exist. That gap is closed; see
   `Plan.md` Part II, `docs/adr/0001-language-plugin-boundary.md`,
   `docs/adr/0002-symbol-placement-and-isolation.md`, and `docs/adding-a-language.md`.
2. **Next milestone: prove the mechanism, do not extend it.** Review rule R1 promises
   that a new language touches no core crate, but it has never been falsifiable: both
   existing plugins were added in this repository by the team that wrote the core
   crates, and the C++ "plugin" was produced by extracting core code rather than
   authored independently. The next milestone adds a small third language as an
   independent plugin crate to test that promise honestly.
3. **M24 stays parked until that evidence exists.** The third-language milestone is
   what makes the M24 decision decidable: if a new plugin needs weight the binary
   cannot carry, or the guard cannot hold, the triggers in section 5 fire with evidence
   rather than speculation. Re-decide M24 after it lands.

Do not assign the next milestone a number. It has no plan yet, and inventing an ID
here is exactly the incoherence this task is curing.

## Accuracy discipline (hard)

- Every commit hash you write must have appeared in `git log` output you actually ran.
  Never reconstruct a hash from memory.
- Do not restate the ADRs. Link to them.
- Do not delete the original planning-tense text of the plan. Add dated notes.
- Do not claim the C++ goldens are byte-identical to a pre-M17 baseline without
  checking `tests/golden/`; the recorded invariant is that the M00 golden suite is the
  behavioural lock, and M23 carried a deliberate golden refresh.

## Verification gates (run in this order)

No TDD surface -- documentation only. Acceptance is zero-source-diff plus internal
consistency.

1. `git status --porcelain` -- must show ONLY
   `plans/multi-language-support-implementation_plan.md` modified (plus this handoff
   and the journal). Any `crates/`, `tests/`, `docs/`, or `Plan.md` path = blocker,
   revert it.
2. `git diff --stat` -- exactly one file changed by the commit.
3. Non-ASCII check on your ADDED lines only:
   `git diff -- plans/multi-language-support-implementation_plan.md | Select-String -Pattern '^\+.*[^\x00-\x7F]'`
   -- must return nothing. Pre-existing non-ASCII bytes in untouched lines are not your
   concern and must not be "fixed".
4. `python tools/hygiene_check.py` -- must exit 0.
5. Consistency check -- all must hold:
   - the status line no longer says DRAFT
   - section 9 contains an M23 block and all four Docs items are `[x]`
   - the M24 bullet contains the word "parked" and links to ADR-0001
   - a "Sequencing after M23" heading exists
6. Every `docs/...` and `Plan.md` path you cite must exist. Verify with `Test-Path`.

**Do NOT run `cargo test` / `cargo build` / `cargo clippy`.** No source changed.

## Available tooling (IMPORTANT)

`tools/` contains ONLY `hygiene_check.py` and `hygiene_check_tests.py`. These do NOT
exist on this base and must not be attempted: `tools/testrun.py`, `tools/distill.py`,
`tools/encoding_guard.py`, `tools/golden_guard.py`, `tools/commit_scoped.py`,
`tools/checkpoint_guard.py`.

Commit with plain git, scoped to the touch-set (never `git add .`):

```powershell
git add plans/multi-language-support-implementation_plan.md tasks/plan-repair-t5-handoff.md tasks/plan-repair-t5-journal.md
git commit -m "docs(plan): reconcile the multi-language plan with M15-M23 - closed-through-M23 status, M23 todo block, D.1-D.4 checked, M24 parked on real triggers"
```

## Discipline clauses

- **UTF-8 file edits:** use the Write/Edit tools ONLY. Never `Set-Content`, `Out-File`,
  `>`, or `Get-Content -Raw | -replace` -- on PowerShell 5.1 these corrupt UTF-8
  punctuation in tracked files.
- **Do not "fix" mojibake.** If PowerShell *displays* `Â§` or `â†"` in existing plan
  text, that is a console codepage artifact (UTF-8 read as ANSI), NOT file corruption.
  Leave those bytes alone. This matters most here: your edits are inside a file that
  already contains such characters.
- Touch only the touch-set. On discovering out-of-scope breakage: report, do not fix.
- After the Green gate, append a 1-3 line checkpoint to
  `tasks/plan-repair-t5-journal.md` (create it alongside this handoff).
- If context exhaustion looms: STOP and return a disk-state inventory. A dead session
  is never resumed.

## Report back (final message)

1. Commit hash + `git show --stat`.
2. Gate evidence: the six verification results as canonical lines + exit codes.
3. Confirmation of the four Edit-5/6 consistency predicates, one per line.
4. Any commit hash you could not resolve and therefore wrote as `landed: <file>`.
5. Any place you judged the M24 reclassification to misstate the M23 record -- report,
   do not soften the edit.
6. Deviations from this handoff + reason.

Never paste raw tool output into the report. Cite command + canonical summary line + exit code.
