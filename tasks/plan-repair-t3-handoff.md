# Plan Repair T3 Handoff — D.4: ADR-0002, symbol placement and the isolation primitive

Binding inputs (read in this order, targeted reads only):

1. This file (complete contract).
2. `plans/multi-language-support-implementation_plan.md` -- read ONLY these ranges:
   - **lines 14-19** (section 0.1, the C++-vs-single-file refinement that motivates the whole thing)
   - **lines 272-293** (section 3.6 the isolation primitive, its division-of-labor table, and 3.6.1 the provenance fix)
   - **lines 430-436** (the Deferred M23 bullets -- this is what M23 was asked to do)
   - **lines 530-537** (decision points D1-D7, esp. D6 and D7)
3. `plans/m23-symbol-isolation-kickoff.md` -- read ONLY lines 1-10, 43-48 (the D-M23
   rulings that amended the spec), and 170-178 (Gate-G3 verdicts, the two
   confirmations, and the follow-up backlog). This is the record of what M23 actually
   decided; it supersedes the spec's planned table where they differ.
4. `crates/lang-api/src/modeling.rs` -- read `SymbolPlacement` and `LanguageModeling`
   only. Confirm the real variant names before you write them down.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `docs/adr/0002-symbol-placement-and-isolation.md` (create, including the `docs/adr/` directory if absent)
- `tasks/plan-repair-t3-handoff.md` (this file)
- `tasks/plan-repair-t3-journal.md` (checkpoint journal; always in-set)

You MUST NOT touch: `crates/**`, `tests/**`, `tools/**`, `Plan.md`, any other file
under `docs/` (ADR-0001 belongs to task T2 -- do not create or edit it), or any file
under `plans/`. No `git stash`, no branch change, no `git add .`.

## What to build

`docs/adr/0002-symbol-placement-and-isolation.md` -- an Architecture Decision Record
that fixes (a) how the model represents a symbol whose declaration and definition live
in different files, and (b) *why "move a symbol into its own file" is the
language-neutral primitive* rather than "extract a C++ header".

Use this exact section order:

```markdown
# ADR-0002: Symbol placement and the isolation primitive

- Status: Accepted
- Date: 2026-10-03
- Applies to: `crates/lang-api` (`SymbolPlacement`, `LanguageModeling`), `crates/graph` (reduced model), `crates/transform` (isolation engine), `crates/cluster` (planner), both plugin crates
- Supersedes: nothing
- Spec source: `plans/multi-language-support-implementation_plan.md` sections 0.1, 3.6, 3.6.1; decision points D6 and D7

## Context
## Decision
### 1. Symbol placement is modeled, not assumed
### 2. Isolation means "move the symbol into its own file"
### 3. The isolation unit differs per language
### 4. Symbol-to-file provenance is recorded, not inferred
## Consequences
## What this ADR does not decide
```

### Required content

**Context (2 short paragraphs).** First: C++ is special -- a symbol's *declaration*
lives in a header while its *definition* may live in a different source file. Most
modern languages are single-file: one top-level symbol per file, declaration and
definition inseparable. The original model papered over this by assuming the two were
the same ("usually"), which is a C++ accident dressed as a universal rule. Second: the
product's core move is therefore not "extract a C++ header". It is *move symbol S out
of shared file F into its own file F_S, so consumers that need only S stop depending on
F.* For C++ this shrinks the include closure; for single-file languages it shrinks
import granularity -- and in both cases it makes the symbol-to-file relation 1:1, which
is what makes the per-file load metric actionable at all.

**Decision 1 -- placement is modeled.** `LanguageModeling` carries a
`symbol_placement` field whose value is one of the `SymbolPlacement` variants. Confirm
the exact variant names from `crates/lang-api/src/modeling.rs` and use those; the
concept is C++ = split declaration/definition, other target languages = single file.
State that no extension test or `.h` suffix check appears anywhere in the core crates.

**Decision 2 -- the isolation primitive.** Keep the spec's division-of-labor table
(spec lines 278-284) but TRANSPOSED to past tense, recording what is where now:
the *engine* (grouping extractions per target file, computing inter-symbol dependency
edges over the reduced model, cycle-checking the target graph, writing files) stays
neutral in `crates/transform`; *new-file text rendering* is the plugin's
(`SourceExtractor`); *consumer redirect and directive-text rewriting* is the plugin's
(`DependencyRewriter`); *target file naming* is a plugin hook. State the invariant that
makes this safe: core crates must not learn a language's file-naming or directive
syntax to do any of that.

**Decision 3 -- the isolation unit differs per language.** This is the most
counter-intuitive part, so state it plainly in a two-row table: C++ isolates the
*declaration site* -- extract from the header, because that is what consumers actually
pay for; the definitions stay in their source files. Single-file languages isolate the
*whole top-level symbol* -- the file becomes its own file. Add the consequence: the same
plan, the same engine, and the same verification produce the right unit per language
because the plugin supplies the answer.

**Decision 4 -- provenance is recorded, not inferred.** The reduced model gains
`declaring_files` and a `defining_file` resolved from the *defining declaration's*
recorded file location, falling back to the owning file when the declaration id is
unknown -- replacing the old shortcut that simply equated the two. `declaring_files` is
deduped and sorted. Note the artifact-format consequence honestly and specifically: the
reduced symbol blob is **bincode, not JSON**, so `#[serde(default)]` does not rescue a
stale blob. It is safe only because the cache epoch was bumped, which forces
regeneration. That coupling is worth stating in an ADR because it is a trap.

**Consequences.** Positive: the load metric becomes trustworthy; C++ behaviour is
preserved byte-for-byte through M18 while the engine was neutralized; M23 could then
feed real symbol bodies for both languages. Accepted costs and known limits, from the
M23 record: extracted units are **not self-contained** -- they can reference types
outside their own dependency set -- so real compiler invocation was deliberately NOT
wired and tier-3 validation stays `Skipped` for both languages offline (amendment
D-M23-12, after reviewer finding F3); a `source_file` provenance refinement for the
planner was ruled incomplete-not-incorrect by both review lanes and deferred.

**What this ADR does not decide.** Real compiler invocation and the extraction
include-closure computation that would make it possible; a toolchain-backed Scala
analyzer; package-scoped cycle precision; anything about dynamic plugin loading (see
ADR-0001). State that these are follow-up work, not settled decisions, so a later
reader does not mistake absence for a ruling.

## Accuracy discipline (hard)

- Every crate name, file path, type name, and variant name you write must exist.
  Verify with `Test-Path` / `Select-String` before committing. Report anything that
  does not resolve.
- The M23 record in `plans/m23-symbol-isolation-kickoff.md` lines 170-178 **supersedes**
  the spec's planned table wherever the two differ. Where they differ, say so.
- Do not claim tier-3 validation runs a compiler. It does not.

## Verification gates (run in this order)

No TDD surface -- documentation only. Acceptance is zero-source-diff plus reference
resolution.

1. `git status --porcelain` -- must show ONLY your new ADR (plus this handoff and the
   journal). Any `crates/` or `tests/` path = blocker, revert it.
2. `git diff --cached --stat` after staging -- exactly one content file.
3. Non-ASCII scan: `Select-String -Path docs\adr\0002-symbol-placement-and-isolation.md -Pattern '[^\x00-\x7F]'` -- must return nothing.
4. `python tools/hygiene_check.py` -- must exit 0.
5. Reference check: every cited path / type / variant must resolve. Report failures.
6. The section order above must be present, in order. Verify by grepping `^#{2,3} ` lines.

**Do NOT run `cargo test` / `cargo build` / `cargo clippy`.** No source changed.

## Available tooling (IMPORTANT)

`tools/` contains ONLY `hygiene_check.py` and `hygiene_check_tests.py`. These do NOT
exist on this base and must not be attempted: `tools/testrun.py`, `tools/distill.py`,
`tools/encoding_guard.py`, `tools/golden_guard.py`, `tools/commit_scoped.py`,
`tools/checkpoint_guard.py`.

Commit with plain git, scoped to the touch-set (never `git add .`):

```powershell
git add docs/adr/0002-symbol-placement-and-isolation.md tasks/plan-repair-t3-handoff.md tasks/plan-repair-t3-journal.md
git commit -m "docs(adr): D.4 add ADR-0002 fixing symbol placement and why symbol isolation is the language-neutral primitive"
```

## Discipline clauses

- **UTF-8 file edits:** use the Write/Edit tools ONLY. Never `Set-Content`, `Out-File`,
  `>`, or `Get-Content -Raw | -replace`.
- Pure ASCII content. Use `--` for dashes, never an en-dash or em-dash character.
- Touch only the touch-set. On discovering out-of-scope breakage: report, do not fix.
- After the Green gate, append a 1-3 line checkpoint to
  `tasks/plan-repair-t3-journal.md` (create it alongside this handoff).
- If context exhaustion looms: STOP and return a disk-state inventory. A dead session
  is never resumed.

## Report back (final message)

1. Commit hash + `git show --stat`.
2. Gate evidence: the six verification results as canonical lines + exit codes.
3. Section-order confirmation (paste the heading list only -- it is short).
4. Every cited path/name/variant that failed to resolve (report-only).
5. Any place you had to depart from the spec's planned table in favour of the M23
   record -- cite both.
6. Deviations from this handoff + reason.

Never paste raw tool output into the report. Cite command + canonical summary line + exit code.
