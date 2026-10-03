# Plan Repair T1 Handoff — D.1: restore the project plan as source of truth (M15-M23)

Binding inputs (read in this order, targeted reads only):

1. This file (complete contract).
2. `Plan.md` (1121 lines) -- read ONLY: lines 1-45 (Goal + Milestone Overview table) and the last ~25 lines (the closing prose part, to find the append point). Do NOT read sections `# 3`..`# 17` in full; instead read ONE existing milestone record in full (e.g. `# 17. M14`) to copy its shape.
3. `plans/multi-language-support-implementation_plan.md` -- the multi-language spec. Read ONLY:
   - lines 314-437 (the milestone spine M17-M22 + the Deferred M23/M24 subsection)
   - lines 543-585 (the todo list, to see which items are checked)
4. `plans/m23-symbol-isolation-kickoff.md` -- read lines 1-10 and 170-178 (M23 base, gate verdicts, final gate, follow-up backlog).

## Why this task exists (the defect being cured)

`Plan.md` is the project plan. It records M0-M14 and then STOPS. Verified: `Select-String -Path Plan.md -Pattern 'M15|M16'` returns **zero matches** -- M15 and M16 are absent from the body AND from the section-2 Milestone Overview table, despite the suites `m15_graph_ui_test.rs` and `m16_clang_worker_test.rs` / `m16_traversal_test.rs` existing and passing. M17-M23 are likewise absent.

The multi-language plan says "Numbering continues `Plan.md` (M0-M16 exist)" -- that is false. So there is no record anywhere of what M15-M23 delivered. This task restores it.

**Scope correction you must honour:** checkbox D.1 in the spec says "Append M17-M22 to Plan.md". That is INCOMPLETE. The real scope is **M15-M23 (nine milestones)**, because M15/M16 are equally missing and appending only M17+ would create a *new* incoherence (a plan that jumps M14 -> M17). Record M15-M23.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `Plan.md` (the only file you may change)
- `tasks/plan-repair-t1-handoff.md` (this file, if a correction is needed)
- `tasks/plan-repair-t1-journal.md` (checkpoint journal; always in-set)

You MUST NOT touch: any file under `crates/`, `tests/`, `tools/`, `docs/`, `plans/`, or any other root file. No `git stash`, no branch change, no `git add .`.

## What to build

### 1. Extend the section-2 Milestone Overview table

Add nine rows (M15..M23) to the existing table, matching the existing 3-column shape
(`| Milestone | Capability | User-visible result |`) and its column padding. The table currently ends at M14. Keep the ASCII-only hyphens already used there; do NOT introduce en-dashes.

### 2. Append a new part at end of file

The existing milestone sections are `# 3. M0` .. `# 17. M14`, after which come
`# 18.` .. `# 23. Definition of Done for the Product` (closing prose). **Do not renumber
those sections** and do not insert between them -- renumbering risks breaking internal
cross-references.

Append at end of file:

```markdown
# Part II -- Multi-Language and Plugin Milestones (M15-M23)

Part I (M0-M14) built the C++ dependency-optimization tool. Part II records the
milestones that made it language-neutral and added Scala as the first additional
language through the plugin mechanism. The governing spec is
`plans/multi-language-support-implementation_plan.md`; per-milestone kickoffs and
gate verdicts live in `plans/m19..m23-*-kickoff.md`.
```

Then one `## M<n> -- <title>` record per milestone, M15 through M23, in the same
section shape as the existing milestone records.

### 3. Record shape (per milestone)

Mirror the existing M-record sub-headings. Use exactly these, in this order:

- `## Objective` -- one or two sentences: what exists after this milestone.
- `## Capabilities` -- bullet list of what the system can now do.
- `## CLI` -- the user-visible command/surface, or `not user-visible` if none.
- `## Artifacts` -- artifact paths whose shape this milestone changed (or `none`).
- `## Tests` -- the integration suite filenames that are this milestone's evidence.
- `## Proves` -- what is now demonstrated.
- `## Does not prove` -- the honest limits; this is the most important section.
- `Landed:` -- one trailing line naming the closeout commit, formatted
  `Landed: <short-hash> (<what that commit did>).`

### 4. Verified milestone anchors

These were verified against `git log` at HEAD `36b182a`. Use them; do not re-derive.

| MS | Title | Landed anchor | Evidence suite(s) |
|---|---|---|---|
| M15 | Graph UI | **UNVERIFIED** -- see rule below | `m15_graph_ui_test.rs` |
| M16 | clang worker / traversal | **UNVERIFIED** -- see rule below | `m16_clang_worker_test.rs`, `m16_traversal_test.rs` |
| M17 | Language API seam | `57cc995` (moves index value types into `ariadne-lang-api`, adds neutral modeling/capability/catalog vocabulary, extends `LanguagePlugin` + registry); preceded by `b8a9de7` (M17.0 golden capture) | `m17_plugin_seam_test.rs` |
| M18 | C++ becomes a plugin | `e8422ef` (relocates the C++ engine into `ariadne-lang-cpp`); gate APPROVED at `3b06e54` | `m18_cpp_plugin_parity_test.rs` |
| M19 | Scala plugin: discovery + build context | `d0c6499` (creates `ariadne-lang-scala`: `ScalaPlugin`, `ScalaDetector`, `ScalaUnitShaper`); closeout `32ecf33` | `m19_scala_discovery_test.rs` |
| M20 | Scala dependency graph (imports -> edges) | closeout `77bd0a7`; gate APPROVED `75abb8e` at 767/0/3 | `m20_scala_dependency_graph_test.rs` |
| M21 | Scala semantic index (offline) | closeout `33e9950`; gate APPROVED `acbb1a9` at 842/0/3 | `m21_scala_semantic_index_test.rs` |
| M22 | Scala analysis outputs: aggregation, load, reports, Graph UI | closeout `76f619c`; Gate-C ACCEPTED at `09530af`; final gate 916/0/3 | `m22_scala_load_analysis_test.rs` |
| M23 | Symbol isolation, shared engine (C++ + Scala) | Gate-G3 closure `36b182a`; final gate 1088/0/3 verified at `f9bcf69` | `m23_symbol_isolation_test.rs` |

**Rule for M15/M16 (UNVERIFIED anchors):** search once with
`git log --oneline --all --grep=M15` (and `M16`). If you find no commit whose subject
identifies the milestone closeout, **omit the `Landed:` line entirely for that
milestone** and instead write `Landed: not isolated in commit history (pre-dates
milestone-titled commits).` That is an honest record. **Never invent a hash, and
never present a hash you have not seen in `git log` output.**

**Suite-prefix collision warning.** Several suites carry an `mNN` prefix that is NOT
this plan's milestone: `m17_dir_nav_test.rs` (a UI directory-navigation feature),
`m18_cache_epoch_test.rs` and `m18_persistence_incremental_test.rs` (M18.x supporting
suites), `m19_optimize_characterization_test.rs`, `m20_apply_migrate_characterization_test.rs`,
`m21_run_ui_smoke_test.rs`. Cite only the suites in the table above unless you have
verified a suite genuinely belongs to the milestone you are recording.

### 5. Accuracy discipline (hard)

Every factual claim must be traceable to ONE of: a file path in the tree, a suite
filename, a commit hash you read from `git log`, or a gate result recorded in
`plans/*.md`. Do **not** restate aspirational spec text as delivered fact. Where the
spec's planned table and the landed code diverge, record what landed, and put the
divergence in `Does not prove` or a short note. Specifically:

- M23 **deliberately did not** wire real compiler invocation (amendment D-M23-12, after
  reviewer finding F3: extracted headers are not self-contained). Tier 3 therefore stays
  `Skipped` for both languages. This belongs in M23's `Does not prove`.
- M23 **deferred** the `AuditTrail` `source_file` provenance fix: both Gate-G3 lanes
  ruled it incomplete-not-incorrect and deferred. Belongs in `Does not prove`.
- The spec's own A2 assumption (no `libloading`/WASM host) is why dynamic plugin
  loading was never built. Mention this in M17's `Does not prove`.

### 6. Length budget

`Plan.md` is 1121 lines. AGENTS.md hygiene targets files < 1000 lines and the spec's
R4 says < 1500. Target **<= 1470 lines** total after your edit. Keep each M-record
between 28 and 40 lines. If you are heading over budget, tighten prose -- do NOT drop a
`Does not prove` section, and do NOT drop a milestone.

## Verification gates (run in this order)

There is no TDD surface: this task changes documentation only. The acceptance is a
**zero-source-diff** proof plus internal-reference resolution.

1. `git status --porcelain` -- must show ONLY `Plan.md` modified (plus this handoff and
   the journal once created). Any `crates/` or `tests/` path appearing = blocker, revert it.
2. `git diff --stat` -- must show exactly one file changed by the commit.
3. `git diff -- Plan.md | Select-String -Pattern '^\+.*[^\x00-\x7F]'` -- must return
   NOTHING. The Plan.md append is pure ASCII.
4. `python tools/hygiene_check.py` -- must exit 0.
5. Reference check: every `plans/...` and `crates/...` path and every suite filename you
   cite must exist. Verify with `Test-Path` / `Get-ChildItem`. Report any that do not.
6. Line count: `(Get-Content Plan.md | Measure-Object -Line).Lines` -- must be <= 1470.

**Do NOT run `cargo test` / `cargo build` / `cargo clippy`.** No source changed; a
full workspace run is high-volume noise and a reviewer blocker. Note in your report
that you deliberately skipped it and why.

## Available tooling (IMPORTANT)

`tools/` contains ONLY `hygiene_check.py` and `hygiene_check_tests.py`. These tools
referenced by older handoffs **do not exist on this base** and you must not attempt
them: `tools/testrun.py`, `tools/distill.py`, `tools/encoding_guard.py`,
`tools/golden_guard.py`, `tools/commit_scoped.py`, `tools/checkpoint_guard.py`.

Commit with plain git, scoped to the touch-set only (never `git add .`):

```powershell
git add Plan.md tasks/plan-repair-t1-handoff.md tasks/plan-repair-t1-journal.md
git commit -m "docs(plan): D.1 record M15-M23 in Plan.md - restore the milestone source of truth"
```

## Discipline clauses

- **UTF-8 file edits:** use the Write/Edit tools ONLY. Never `Set-Content`, `Out-File`,
  `>`, or `Get-Content -Raw | -replace` -- on PowerShell 5.1 these corrupt UTF-8
  punctuation in tracked files.
- **Do not "fix" mojibake.** If PowerShell *displays* `Â§` or `â†"` in existing
  Plan.md text, that is a console codepage artifact (UTF-8 read as ANSI), NOT file
  corruption. Leave those bytes alone. Your own added text is pure ASCII, so this only
  matters if you rewrite surrounding lines -- do not rewrite them.
- Touch only the touch-set. On discovering out-of-scope breakage: report, do not fix.
- After the Green gate, append a 1-3 line checkpoint to
  `tasks/plan-repair-t1-journal.md` (create it alongside this handoff): what changed,
  final line count, any unverified anchor you omitted.
- If context exhaustion looms: STOP and return a disk-state inventory of what you have
  written so far. A dead session is never resumed.

## Report back (final message)

1. Commit hash + `git show --stat`.
2. Gate evidence: the six verification results, each as a canonical line + exit code.
3. Final `Plan.md` line count.
4. Any milestone record whose `Landed:` anchor you had to omit, and why.
5. Any claim you could not trace to a file/suite/commit/gate (report-only).
6. Deviations from this handoff + reason.

Never paste raw `git log`/`git diff` dumps into the report. Cite command + canonical
summary line + exit code.
