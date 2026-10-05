# IDD/RGB-TDD Workflow for AriadnesThread

This project uses the Intent-Driven Development (IDD) workflow with strict Red-Green-Refactor (RGB) TDD and enforced code hygiene.

## How the Workflow Works

The workflow activates automatically when you ask for development work (implement, add, build, create, fix features). It uses skill orchestration to coordinate specialized agents (D1 Hybrid Topology):

1. **Architect/Orchestrator** (primary) - Clarifies intent, creates specifications, breaks down tasks, delegates to subagents
2. **Implementer** (project subagent) - Follows strict TDD (Red-Green-Refactor) with code hygiene enforcement; routes tools through distilling entry points
3. **Reviewer** (project subagent) - Adjudicates handoffs via `review-gate` skill; enforces code hygiene and quality standards
4. **Tool-caller** (project subagent, optional) - One-shot bulk command absorber; returns digest-shaped output only
5. **Walkthrough Guide** (project subagent) - Generates interactive HTML documentation

## Automatic Activation

You don't need to do anything special to use this workflow. Simply make a request like:

- "Implement a feature to optimize header dependencies"
- "Add a new parser for C++ templates"
- "Fix the bug in the symbol resolution"
- "Create a new report generator"

The `idd-workflow` skill will auto-invoke and coordinate the entire process.

## Code Hygiene Rules

This project enforces strict code hygiene:

- **Functions < 100 lines** - All functions must be under 100 lines
- **Files < 1000 lines** - All files must be under 1000 lines
- **Meaningful names** - All variables, functions, and types must have descriptive names
- **Single responsibility** - Each function must do one thing well
- **Source lines < 2000 chars** - Never park a paragraph on one line: agent Read/grep silently truncate at 2000 chars, hiding content from every workflow participant. Wrap whitespace-collapsing prose (HTML/Markdown body) sentence-per-line; where newlines are significant (`<pre>`, string literals), split into concatenated parts instead

Violations will cause code review to fail. See `~/.config/devin/skills/idd-workflow/CODE_HYGIENE.md` for detailed guidelines.

## File-Edit Discipline (UTF-8 hazard — read before touching any file)

PowerShell 5.1 mangles UTF-8 in this repo: tracked files repeatedly accumulated double-encoded sequences of legitimate characters (`§`, `—`, `→` stored as their own mojibake). Hard rules:

- NEVER write or rewrite file content through `Set-Content`, `Out-File`, `>`, `>>`, or `Get-Content | -replace` pipelines. Use the Read/Edit/Write tools instead (byte-safe), or byte-level writes (`open(path, "wb")`) in scripts.
- When quoting git content, ALWAYS quote the whole revision:path argument: `git show "rev^:path"` — an unquoted `^` is misparsed by PowerShell and can silently compare the wrong blob.
- NEVER pipe `git show`/`git cat-file` through PowerShell into a file; PowerShell console decoding fakes "missing" or corrupt lines. Read blobs with the Read tool on a checkout, or byte-level `git cat-file` inside Python.
- All authored file contents are pure ASCII; test fixtures that need corruption signatures construct them via byte escapes (`b"\xc3\x82\xc2\xa7"`), never literals.
- Enforcement: `python tools/encoding_guard.py --all` (also `--staged` for pre-commit) detects double-encoded UTF-8 mojibake and non-UTF-8 blobs in tracked content; CI runs it.

## Quick Mode

For simple tasks (single-file changes, trivial fixes, configuration updates), the workflow will offer a "quick mode" option that bypasses full planning and review while still enforcing code hygiene.

## Project-Specific Context

### Project Structure

This is a Rust workspace for C++ dependency analysis:
- `crates/` - Rust crates (cli, lang, repo, build, scheduler, index, graph, cluster, transform, validate, report, pipeline, ui)
- `tools/` - Supporting tools (clang-worker)
- `tests/` - Integration tests
- `Intent.md` - Project intent document
- `Plan.md` - Implementation plan
- `Spec.md` - Technical specification

### Testing

- Use `cargo test` to run tests
- Canonical reporting: `python tools/testrun.py [scoped cargo test args]` prints one line `N passed / M failed / K ignored / S suites` (exit 2 = parse drift; use `--baseline` for milestone gates)
- Agents run tests via `python tools/testrun.py --distill [scoped args]` (bounded digest + full capture at `target/testrun-capture.txt`, path printed in the digest); raw mode remains for humans
- UI runtime smoke (one command): `python tools/ui_smoke.py` builds the tool, cold-boots the UI on an ephemeral port, asserts routes + `Cache-Control: no-store`
- Tool unit suites: `python -m unittest discover -s tools -p "*_tests.py"` (what CI runs)
- Tests are organized by crate in `crates/*/tests/` and `tests/`
- Integration tests are in `tests/integration/`

### Build

- Use `cargo build` to build the project
- Use `cargo build --release` for release builds
- Normal human launch uses the RELEASE build (`cargo build --release`
  first, or add `--release` to any documented `cargo run` line): debug
  indexing measured ~20x slower than release at TU scale (M27 T1a
  launch-profile finding 2026-10-05; E1's ~3.1 s/TU was the debug profile)
- Load-perf regression check (local gate): `python tools/bench_compare.py` (median-of-3 stage-5 tail vs per-machine baseline)

### Process Tooling (deterministic rituals; reference, don't re-author)

- `python tools/checkpoint_guard.py status` — ground truth at session start (incl. worktree census); `resume --spec tasks/<handoff>.md --scope … --cmd …` — fresh-session bootstrap after a dead implementer (skill `resume-checkpoint`)
- `python tools/encoding_guard.py --staged` before committing; `--all` in CI (mojibake/non-UTF-8 gate)
- `python tools/commit_scoped.py --scope <glob> … -m <msg>` — scoped commits in shared worktrees (never `git add .`)
- `python tools/theme_token_check.py --check-added --base <rev>` — UI hex-literal drift guard
- `python tools/hygiene_check.py` — R1/R2 blocking, R3 naming warn-only (flips via dated rows in `tasks/milestone_ledger.md`)
- `python tools/testrun.py` — canonical test summary line (quote verbatim in reports)
- `python tools/distill.py -- <cmd>` - distilling wrapper for build/clippy/generic commands (digest + `target/distill-capture.txt`, never raw output in agent context)
- Standing contract: `docs/specs/tool-output-distillation.md` - entry points, digest shapes, prohibitions, evidence rules
- Skills (`.kilo/skills/`): `handoff-spec` (author handoffs), `review-gate` (adjudicate handoffs; its commit-vs-touch-set check enforces scope), `milestone-lifecycle` (kickout/closeout; ledger is source of truth), `resume-checkpoint`, `walkthrough-commit`
- Milestone state: `tasks/milestone_ledger.md` is the source of truth; kilo-memory mirrors it in the same commit

### Handoff Sizing Rule (Planning Law)

A handoff unit MUST fit within these bounds:
- **Scope**: ≤ 1 Cargo crate OR ≤ 1 top-level module directory under `crates/`/`tools/`/`tests/`
- **Gate commands**: ≤ ~10 scoped gate commands (distilling entry point invocations)
- **Expected produced lines**: ≤ ~1200 lines of implementation+test diff output
- **Larger work splits** into multiple handoffs

Workspace-wide final gates (`--baseline` run, full `unittest discover`, `ui_smoke`, `bench_compare`) run ONLY in a fresh session or the orchestrator's bounded own run. Sizing axis = gate count × output volume.

This rule is also reflected in the `handoff-spec` skill template (gates section).

### Documentation

- Project intent is documented in `Intent.md`
- Technical specification is in `Spec.md`
- Implementation plan is in `Plan.md`

## Workflow Integration

The workflow integrates with existing project documentation:

- **Intent documents**: Created/updated in `Intent.md` or `docs/specs/<feature>.md`
- **Handoff documents**: Created in `tasks/<task>-handoff.md` via `handoff-spec` skill
- **Journals**: Created in `tasks/<task>-journal.md` by implementer
- **Walkthroughs**: Generated in `walkthroughs/<feature>-walkthrough.html`

## Existing Skills

The workflow leverages existing global skills:
- `dev-planning` - For task breakdown
- `tdd` - For TDD guidance
- `review-team` - For specialized review perspectives
- `execute-plan` - For implementing from existing plans

## Manual Skill Invocation

You can also invoke skills directly:

- `/idd-workflow` - Start the full IDD workflow
- `/implementer-rgb-tdd` - TDD implementation
- `/tool-runner` - Execute tools (distilling entry points)
- `/reviewer-hygiene` - Code review with hygiene enforcement
- `/walkthrough-guide` - Generate walkthrough
- `/quick-solo` - Quick implementation for simple tasks

## Getting Help

For questions about the workflow:
- See `~/.config/devin/skills/idd-workflow/COORDINATION.md` for coordination protocols
- See `~/.config/devin/skills/idd-workflow/CODE_HYGIENE.md` for hygiene rules
- See individual skill SKILL.md files for detailed instructions

## Contributing

When contributing to this project:
1. The workflow will guide you through intent clarification
2. Follow the TDD process (failing tests first)
3. Adhere to code hygiene rules strictly
4. Participate in the review process
5. Review the generated walkthrough

The workflow ensures all contributions meet quality standards before integration.