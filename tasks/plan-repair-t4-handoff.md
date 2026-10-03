# Plan Repair T4 Handoff — D.3: "Adding a language" page

Binding inputs (read in this order, targeted reads only):

1. This file (complete contract).
2. `crates/lang-scala/Cargo.toml` (complete -- 18 lines) and `crates/lang-scala/src/lib.rs`
   (module list only). This is the concrete shape a new plugin crate takes.
3. `crates/pipeline/src/registry.rs` -- find the `build_default_registry` function and
   read it plus its surrounding doc comment ONLY. This is the one registration site.
4. `crates/lang/src/registry.rs` -- read the `LanguagePlugin` trait definition ONLY, to
   list the methods a new plugin must supply. Do not read the whole file.
5. `crates/lang/src/unit.rs` -- read the `UnitShaper` trait definition ONLY.
6. `tests/integration/tests/m17_plugin_seam_test.rs` -- read the two guard functions
   and the `GUARD_FORBIDDEN` const ONLY.
7. `tests/fixtures/scala-basic/` -- list the tree (`Get-ChildItem -Recurse -File`) to
   describe the fixture shape. Read ONE `.scala` file and `build.sbt` at most.
8. `docs/adr/0001-language-plugin-boundary.md` -- read ONLY its "What this ADR does not
   decide" and "Machine-checkable enforcement" sections. This page must be consistent
   with the ADR and must not restate it.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `docs/adding-a-language.md` (create)
- `tasks/plan-repair-t4-handoff.md` (this file)
- `tasks/plan-repair-t4-journal.md` (checkpoint journal; always in-set)

You MUST NOT touch: `crates/**`, `tests/**`, `tools/**`, `Plan.md`, `docs/adr/**` (ADRs
belong to tasks T2 and T3), or any file under `plans/`. No `git stash`, no branch
change, no `git add .`.

## Preconditions

Tasks T1-T3 run before you. Verify before you start:

```powershell
git log --oneline -4
Test-Path docs/adr/0001-language-plugin-boundary.md   # must be True
Test-Path docs/adr/0002-symbol-placement-and-isolation.md   # must be True
```

If either ADR is missing, STOP and report -- do not create it.

## What to build

`docs/adding-a-language.md` -- a working, copy-adaptable guide for an engineer adding a
language plugin. It must be usable by someone who has not read the multi-language
plan. Target 150-230 lines.

```markdown
# Adding a language to Ariadnes

## Before you start
## The five steps
### Step 1 -- Create the plugin crate
### Step 2 -- Implement the trait surface
### Step 3 -- Register the plugin
### Step 4 -- Add a fixture repository
### Step 5 -- Add the test suite
## The guard will check you
## What the plugin may not do
## Honest limits
## Where to read more
```

### Required content

**Before you start (short).** State the promise in one sentence -- a new language is
1 new crate, 1 registration, 1 fixture, 1 suite, and **0 edits** to `crates/index`,
`crates/graph`, `crates/cluster`, `crates/transform`, `crates/validate`,
`crates/report`, `crates/scheduler`. State that this promise is *machine-checked*, not
merely intended, and that the checks will fail your build if you violate them. State
that the plugin crate must depend inward only and must NOT list `ariadne-graph`,
`ariadne-cluster`, `ariadne-transform`, or `ariadne-validate`.

**The five steps**, each concrete enough to follow:

1. *Create the plugin crate* -- `crates/lang-<x>/` with `Cargo.toml` listing
   `ariadne-repo`, `ariadne-lang-api`, `ariadne-lang` and nothing else. Show the
   manifest as a fenced code block modelled on `crates/lang-scala/Cargo.toml`. Note
   that `docs/adr/0001` explains why those three and no more. Tell the reader to add
   the crate to the root workspace `members` list. -- This is the ONLY core-adjacent
   file they touch, and it is the workspace manifest, not a core crate.

2. *Implement the trait surface* -- `LanguagePlugin` supplies `id`, `modeling`,
   `capabilities`, `analysis_fingerprint`, and optionally `shaper`; from there you
   supply the capability traits the pipeline actually calls. Confirm every method
   name and signature against `crates/lang/src/registry.rs` and
   `crates/lang/src/unit.rs` -- do not write a method that does not exist, and do not
   omit one that does. Call out the two highest-leverage decisions: `modeling()`
   (unit kind, aggregation kind, duplicate-definition guard, `symbol_placement`,
   external path markers, cycle policy) and `capabilities()` (which pipeline stages
   your language enables). State that leaving a capability `false` produces an honest
   rejection rather than a silent empty result, and that a language with no registered
   plugin is an "unhandled language" error, never a C++ default.

3. *Register the plugin* -- add exactly one line to `build_default_registry` in
   `crates/pipeline/src/registry.rs`. State that this is the single enumeration site
   and that `crates/cli/src/registry.rs` delegates to it. State that **registration
   order is a contract**: the first plugin wins unit-shaper and language-detection
   precedence, which is why C++ is registered first. Warn that appending a language
   that claims paths the C++ plugin claims will change detection precedence and can
   alter C++ results.

4. *Add a fixture repository* -- under `tests/fixtures/`, mirroring
   `tests/fixtures/scala-basic/`: a build-manifest file your shaper recognises, sources
   under a conventional source root, at least one test-side file, and a deliberately
   out-of-repo or generated directory to exercise the external/frozen boundary. Keep it
   small and deterministic -- the whole suite must run with no toolchain installed.

5. *Add the test suite* -- a file in `tests/integration/tests/` that drives the CLI
   handlers on the fixture and asserts on the returned models plus the
   `artifacts/**/manifest.json` files. Name the existing suites as the pattern
   (`m19_scala_discovery_test.rs`, `m20_scala_dependency_graph_test.rs`,
   `m21_scala_semantic_index_test.rs`, `m22_scala_load_analysis_test.rs`,
   `m23_symbol_isolation_test.rs`). Warn about the suite-prefix collision: an `mNN`
   prefix does not mean plan milestone NN -- `m17_dir_nav_test.rs` is a UI feature and
   `m19_optimize_characterization_test.rs` has nothing to do with the Scala milestone.

**The guard will check you.** Name `core_crates_do_not_name_languages` and
`plugin_crates_do_not_depend_on_core_pipeline` from `m17_plugin_seam_test.rs`, say what
each asserts, and note that the neutral-word rule is enforced against production *and*
in-`src` test lines. Give the neutral vocabulary to reach for: *unit*, *aggregation*,
*directive*, *dependency*, *external boundary*, *isolation unit*, *symbol placement*.
Tell the reader to run `python tools/hygiene_check.py` and the guard suite before
opening a review.

**What the plugin may not do.** Four bullets, each with a reason: do not depend on a
core pipeline crate (it inverts the dependency direction); do not make a core crate
name your language; do not require a toolchain to be installed for the suite to pass;
do not write a `match` on language identity outside `lang-api` and the registry
builder without an ADR.

**Honest limits.** This page exists because a promise is only as good as its evidence,
so state the evidence honestly in three bullets: the C++ "plugin" was produced by
*extracting* core code into a plugin crate, not authored independently, so it proves the
seam is neutral but not that a foreign language can be added without friction; the
guard test enforces the boundary continuously but has only ever seen two plugins, both
added in the same repository by the same team; and dynamic/out-of-process plugin
loading is not available and not decided (ADR-0001). Do not hedge this section away --
it is the most useful paragraph in the document.

**Where to read more.** `docs/adr/0001-language-plugin-boundary.md`,
`docs/adr/0002-symbol-placement-and-isolation.md`, and
`plans/multi-language-support-implementation_plan.md`.

## Accuracy discipline (hard)

- Every trait name, method name, signature, file path, and suite filename must exist.
  Verify with `Test-Path` / `Select-String` before committing. Report any that do not
  resolve. A method you invent is worse than a method you omit.
- Do not state that the boundary has been proven by an independent language.

## Verification gates (run in this order)

No TDD surface -- documentation only. Acceptance is zero-source-diff plus reference
resolution.

1. `git status --porcelain` -- must show ONLY `docs/adding-a-language.md` as new (plus
   this handoff and the journal). Any `crates/` or `tests/` path = blocker, revert it.
2. `git diff --cached --stat` after staging -- exactly one content file.
3. Non-ASCII scan: `Select-String -Path docs\adding-a-language.md -Pattern '[^\x00-\x7F]'` -- must return nothing.
4. `python tools/hygiene_check.py` -- must exit 0.
5. Reference check: every `crates/...`, `tests/...`, `docs/...`, `plans/...` path, every
   trait/method name, and every suite filename cited must resolve. Report failures.
6. Length: `(Get-Content docs\adding-a-language.md | Measure-Object -Line).Lines` -- must be 150-230.

**Do NOT run `cargo test` / `cargo build` / `cargo clippy`.** No source changed.

## Available tooling (IMPORTANT)

`tools/` contains ONLY `hygiene_check.py` and `hygiene_check_tests.py`. These do NOT
exist on this base and must not be attempted: `tools/testrun.py`, `tools/distill.py`,
`tools/encoding_guard.py`, `tools/golden_guard.py`, `tools/commit_scoped.py`,
`tools/checkpoint_guard.py`.

Commit with plain git, scoped to the touch-set (never `git add .`):

```powershell
git add docs/adding-a-language.md tasks/plan-repair-t4-handoff.md tasks/plan-repair-t4-journal.md
git commit -m "docs(guide): D.3 add 'Adding a language' - five steps, the guard, and the honest limits"
```

## Discipline clauses

- **UTF-8 file edits:** use the Write/Edit tools ONLY. Never `Set-Content`, `Out-File`,
  `>`, or `Get-Content -Raw | -replace`.
- Pure ASCII content. Use `--` for dashes, never an en-dash or em-dash character.
- Touch only the touch-set. On discovering out-of-scope breakage: report, do not fix.
- After the Green gate, append a 1-3 line checkpoint to
  `tasks/plan-repair-t4-journal.md` (create it alongside this handoff).
- If context exhaustion looms: STOP and return a disk-state inventory. A dead session
  is never resumed.

## Report back (final message)

1. Commit hash + `git show --stat`.
2. Gate evidence: the six verification results as canonical lines + exit codes.
3. Line count.
4. Every cited path/method/suite that failed to resolve (report-only).
5. Any trait method you found yourself unable to describe because the trait surface is
   ambiguous -- report, do not guess.
6. Deviations from this handoff + reason.

Never paste raw tool output into the report. Cite command + canonical summary line + exit code.
