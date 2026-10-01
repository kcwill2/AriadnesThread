# IDD/RGB-TDD Workflow for AriadnesThread

This project uses the Intent-Driven Development (IDD) workflow with strict Red-Green-Refactor (RGB) TDD and enforced code hygiene.

## How the Workflow Works

The workflow activates automatically when you ask for development work (implement, add, build, create, fix features). It uses skill orchestration to coordinate specialized agents:

1. **Architect/Orchestrator** - Clarifies intent, creates specifications, breaks down tasks
2. **Implementer** - Follows strict TDD (Red-Green-Refactor) with code hygiene enforcement
3. **Tool-runner** - Executes tests, builds, and linters
4. **Reviewer** - Enforces code hygiene and quality standards
5. **Walkthrough Guide** - Generates interactive HTML documentation

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
- Tests are organized by crate in `crates/*/tests/` and `tests/`
- Integration tests are in `tests/integration/`

### Build
- Use `cargo build` to build the project
- Use `cargo build --release` for release builds

### Documentation
- Project intent is documented in `Intent.md`
- Technical specification is in `Spec.md`
- Implementation plan is in `Plan.md`

## Workflow Integration

The workflow integrates with existing project documentation:

- **Intent documents**: Created/updated in `Intent.md` or `docs/specs/<feature>.md`
- **Planning documents**: Created in `tasks/plan.md` and `tasks/todo.md`
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
- `/tool-runner` - Execute tools
- `/reviewer-hygiene` - Code review with hygiene enforcement
- `/walkthrough-guide` - Generate walkthrough
- `/quick-solo` - Quick implementation for simple tasks

## Custom Modes

The project has custom modes in `.roomodes` (architect, implementer, reviewer, tool-caller), but the skill-based workflow is the primary approach. The custom modes may be simplified or removed in favor of the skill orchestration.

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