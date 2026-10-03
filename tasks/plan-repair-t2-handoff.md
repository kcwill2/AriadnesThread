# Plan Repair T2 Handoff — D.2: ADR-0001, the language plugin boundary

Binding inputs (read in this order, targeted reads only):

1. This file (complete contract).
2. `plans/multi-language-support-implementation_plan.md` -- read ONLY these ranges:
   - **lines 78-167** (section 3.1 crate layout + 3.2 the neutral vocabulary) -- the dependency-direction rule
   - **lines 239-256** (section 3.4, the C++-vs-Scala lifecycle mapping table) -- this is the ADR's core
   - **lines 272-293** (section 3.6 + 3.6.1, isolation primitive and provenance)
   - **lines 517-522** (review rules R1-R4)
   - **lines 21-31** (assumptions A1-A5, incl. A2 and A5)
3. `crates/cli/src/registry.rs` -- read the module doc comment (first 12 lines) and
   `capability_backend_for_language` / `plan_backend` / `plugin_for_catalog_file`. This
   is the single enumeration seam the ADR must describe accurately.
4. `tests/integration/tests/m17_plugin_seam_test.rs` -- read ONLY the two guard
   functions `core_crates_do_not_name_languages` and
   `plugin_crates_do_not_depend_on_core_pipeline` and the `GUARD_FORBIDDEN` const.
   These are the machine-checkable form of rules R1/R2.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `docs/adr/0001-language-plugin-boundary.md` (create, including the `docs/adr/` directory)
- `tasks/plan-repair-t2-handoff.md` (this file)
- `tasks/plan-repair-t2-journal.md` (checkpoint journal; always in-set)

You MUST NOT touch: `crates/**`, `tests/**`, `tools/**`, `Plan.md`, any other file
under `docs/`, or any file under `plans/`. ADR-0002 is a separate task (T3) and must
not be created or edited by you. No `git stash`, no branch change, no `git add .`.

## What to build

`docs/adr/0001-language-plugin-boundary.md` -- an Architecture Decision Record that
fixes **where the language boundary sits** and **what a core crate may and may not
know about a language**. `docs/adr/` does not exist yet; you create it.

Use this exact section order (standard ADR shape, adapted to this repo):

```markdown
# ADR-0001: The language plugin boundary

- Status: Accepted
- Date: 2026-10-03
- Applies to: `crates/lang-api`, `crates/lang`, `crates/lang-cpp`, `crates/lang-scala`, and every core crate that receives `&dyn` language capabilities
- Supersedes: nothing
- Spec source: `plans/multi-language-support-implementation_plan.md` sections 3.1-3.4, 3.6, and review rules R1-R3

## Context
## Decision
## The lifecycle mapping (C++ vs Scala)
## What each crate may know
## Machine-checkable enforcement
## Consequences
## What this ADR does not decide
```

### Required content

**Context.** Why a boundary exists: before the multi-language work, a new language
required edits in at least six crates plus two binaries, and the `ariadne-lang` trait
scaffold was dead code. State the goal in one paragraph: *new language = 1 new crate +
1 registry entry + fixtures/tests, with 0 edits to the core pipeline crates.* That
sentence is the decision's whole point; keep it verbatim.

**Decision.** Three parts, each tight:

1. *Vocabulary lives in `ariadne-lang-api`.* The eleven index value types
   (`SymbolId`, `DeclId`, `SymbolKind`, `DependencyKind`, `FileLocation`, `SymbolDecl`,
   `SymbolRecord`, `SemanticEdge`, `CallSite`, `SymbolReference`, `TuAnalysis`) were
   moved out of `ariadne-index` into a new `ariadne-lang-api` crate, with
   `crates/index/src/types.rs` reduced to a re-export so every existing call path keeps
   resolving. Explain WHY in one sentence: the `lang -> index` dependency direction made
   the alternative (pushing traits down into `index`) a permanent cycle.
2. *Plugin crates depend inward; core crates never depend on plugin crates.* Reproduce
   the crate layout as a fenced code block (it is worth carrying verbatim from spec
   lines 82-95). State that the ONLY place plugins are enumerated is
   `build_default_registry` in `crates/pipeline` (reached through
   `ariadne_pipeline`), with `crates/cli/src/registry.rs` delegating to it. Registration
   order (C++ first, then Scala) is a contract, not an accident -- it preserves
   unit-shaper and language-detection precedence.
3. *Core crates reach capabilities only as `&dyn`.* They never name a plugin type, a
   compiler binary, a file extension, or a directive syntax.

**The lifecycle mapping table.** Carry the section-3.4 table (spec lines 241-256) as a
markdown table, but TRANSPOSE it so rows are the *concepts* and the C++ / Scala
columns say where each lives NOW (post-landing), not where it was planned to live. The
spec's table is written in planned-move tense; an ADR records the decision and its
current reality. Add a short note that the spec remains the historical record.

**What each crate may know.** A table of the neutral vocabulary the core crates are
allowed to use: *unit*, *aggregation*, *directive*, *dependency*, *external boundary*,
*isolation unit*, *symbol placement*. State the rule from review rule R2 verbatim in
substance: core crates may not name a language, a compiler binary, a file extension, or
a directive syntax; the only C++ vocabulary lives in `crates/lang-cpp`. Cite the
exceptions honestly: `ToolchainStatus` and `ResolveOutcome` live in `lang-api` so core
crates can match on results without importing a plugin, and D5-frozen artifact field
names (`include_load`, `tu_file_id`, `target_header`) are retained for compatibility.

**Machine-checkable enforcement.** This is the section that makes the ADR worth
writing. Describe the two guard tests in `m17_plugin_seam_test.rs` by name and by what
they assert, and note that the second one asserts against the plugin manifests
(`plugin_crates_do_not_depend_on_core_pipeline` parses `crates/lang-cpp/Cargo.toml` and
`crates/lang-scala/Cargo.toml`). State the consequence explicitly: because plugins are
compile-time crates whose manifests can be read, rule R1 is *verifiable* rather than
asserted. Add one sentence that this is exactly the property a runtime-loaded
(dylib/WASM) plugin host would weaken -- the boundary is only auditable while plugins
are statically linked. That sentence is the reason this ADR exists.

**Consequences.** Positive: adding a language touches no core crate; C++ behaviour is
provably preserved by the M18 parity suite; the C++-first registration order is locked
by a unit test. Negative / accepted costs: two registries must be kept in agreement
(the `registries_enumerate_identical_languages` test in `crates/cli/src/registry.rs`
enforces it); the C++ plugin is an *extraction* of core code, not an independently
authored plugin, so rule R1 has never been falsified by a genuinely foreign language.

**What this ADR does not decide.** Explicitly out of scope, so a later reader does not
assume otherwise: dynamic/out-of-process plugin loading (`libloading`/WASM) is NOT
decided here and is currently unparked -- it is gated on a plugin needing something the
in-process model cannot give it (heavyweight third-party dependency, fault isolation, or
independent release cadence), and it would require its own ADR because it changes how
rule R1 is enforced. Also out of scope: a third language, and any change to
`Plan.md` milestones M0-M16.

## Accuracy discipline (hard)

- Every crate name, file path, type name, and test-function name you write must exist.
  Verify with `Test-Path` / `Select-String` before you commit. Report anything that does not resolve.
- Do not restate the spec's planned state as if it were delivered. Where the spec's
  planned table diverged from what landed, the ADR records what landed.
- Do not claim rule R1 has been proven by a third language. It has not. See
  Consequences.

## Verification gates (run in this order)

No TDD surface -- documentation only. Acceptance is zero-source-diff plus
reference resolution.

1. `git status --porcelain` -- must show ONLY `docs/adr/0001-language-plugin-boundary.md`
   as untracked/added (plus this handoff and the journal). Any `crates/` or `tests/`
   path = blocker, revert it.
2. `git diff --cached --stat` after staging -- exactly one content file.
3. Non-ASCII scan of your new file: it must be pure ASCII.
   `Select-String -Path docs\adr\0001-language-plugin-boundary.md -Pattern '[^\x00-\x7F]'` -- must return nothing.
4. `python tools/hygiene_check.py` -- must exit 0.
5. Reference check: every `crates/...`, `tests/...`, `plans/...` path and every type /
   test-function name cited must resolve. Report any that do not.
6. The section order above must be present, in order. Verify by grepping the `^## ` lines.

**Do NOT run `cargo test` / `cargo build` / `cargo clippy`.** No source changed.

## Available tooling (IMPORTANT)

`tools/` contains ONLY `hygiene_check.py` and `hygiene_check_tests.py`. These do NOT
exist on this base and must not be attempted: `tools/testrun.py`, `tools/distill.py`,
`tools/encoding_guard.py`, `tools/golden_guard.py`, `tools/commit_scoped.py`,
`tools/checkpoint_guard.py`.

Commit with plain git, scoped to the touch-set (never `git add .`):

```powershell
git add docs/adr/0001-language-plugin-boundary.md tasks/plan-repair-t2-handoff.md tasks/plan-repair-t2-journal.md
git commit -m "docs(adr): D.2 add ADR-0001 fixing the language plugin boundary, the lifecycle mapping, and R1-R3 enforcement"
```

## Discipline clauses

- **UTF-8 file edits:** use the Write/Edit tools ONLY. Never `Set-Content`, `Out-File`,
  `>`, or `Get-Content -Raw | -replace`.
- Pure ASCII content. Use `--` for dashes, never an en-dash or em-dash character.
- Touch only the touch-set. On discovering out-of-scope breakage: report, do not fix.
- After the Green gate, append a 1-3 line checkpoint to
  `tasks/plan-repair-t2-journal.md` (create it alongside this handoff).
- If context exhaustion looms: STOP and return a disk-state inventory. A dead session
  is never resumed.

## Report back (final message)

1. Commit hash + `git show --stat`.
2. Gate evidence: the six verification results as canonical lines + exit codes.
3. Section-order confirmation (paste the `^## ` heading list only -- it is short).
4. Every cited path/name that failed to resolve (report-only).
5. Deviations from this handoff + reason.

Never paste raw tool output into the report. Cite command + canonical summary line + exit code.
