# M19 Kickoff — Scala Plugin: Discovery + Build Context

Status: READY TO EXECUTE · Prepared 2026-10-01 · Base: branch `multi-language-support` @ `e4ab68f` (workspace 679/0/3, hygiene gate green)

Authoritative spec: `plans/multi-language-support-implementation_plan.md` — §5 "M19" table (lines 377–387) is the per-file change list and acceptance; §3.2/§3.4 the modeling/capability contracts; §9 lines 565–567 the task list; §7 R1–R4 the review rules. This document adds only what the plan cannot know: milestone state, decisions from the M18 audit, and the orchestration protocol.

## Scope — hard boundary

M19.1 `crates/lang-scala` + `ScalaPlugin` + workspace members · M19.2 `ScalaDetector` + `ScalaUnitShaper` + unit tests · M19.3 register + `tests/fixtures/scala-basic/**` + `m19_scala_discovery_test.rs`. **Nothing from M20+** — no import resolver, no graph, no indexing. Capabilities (plan:381): `dependency_graph`/`semantic_index`/`load_metric` are declared `true` but their machinery arrives later; `planning`/`extraction`/`rewriting`/`validation` are `false`.

R1 touch-set: `crates/lang-scala/**`, `crates/cli/src/registry.rs` (+ `crates/pipeline/src/lib.rs` **only** for the registry helper in D9 below), `tests/fixtures/scala-basic/**`, `tests/integration/**`. Any other core-crate edit needs justification against §3.1 first.

## Decisions carried in from the M18 audit

- **D8 — compile_commands residue (decided):** the `cl.exe`/`x64`/`language: Cpp` residues in `crates/build/src/discovery.rs` are permanent-by-design on the shaper-less path — `compile_commands.json` *is* a C/C++-specific artifact (comment at discovery.rs:172). Scala resolves units through the injected `ScalaUnitShaper`, so per plan:387 the M19.3 acceptance assertion *"the compile_commands path is not entered (no C++ diagnostics)"* is the lock. No core-crate change in M19.
- **D9 — registry duplication (mandatory M19 work item, plan note (c), plan:560):** `pipeline::default_registry()` duplicates `cli::registry::default_registry()` (pipeline/src/lib.rs:41–47). Registering Scala in only one site would make registry-aware pipeline paths silently miss Scala. **Preferred fix: share one helper; minimum fix: update both sites + a test asserting both registries enumerate the identical language set.**
- M18 locks are frozen: the `m18_cpp_plugin_parity` suite, its vendored captures, `tests/golden/**`, and `basic`/`expensive` fixtures must stay byte-identical; `m00` runs named fixtures only, so new fixture dirs are safe, new files inside `basic`/`expensive` are not.

## Acceptance (machine-checkable)

1. `cargo test -p ariadne-lang-scala` green (detector + shaper unit tests).
2. `cargo test -p ariadne-integration-tests --test m19_scala_discovery_test` green per plan:387: `.scala` discovery via `Repository::discover`; `lib/`+`target/` → ThirdParty/Generated + Frozen via existing `policy.rs`; shaper yields one unit per `src/main/scala` file with `CompilationContext { language: Scala, compiler: "scalac" }`, context_name from nearest `*.sbt` ancestor; compile_commands path untouched.
3. D9 registries-identical test green.
4. No regressions: `m17_plugin_seam_test` 5/5 (both guards), `m00_golden_baseline_test` 2/2, `m18_cpp_plugin_parity_test` 22/22.
5. Full `cargo test --workspace` green + `python tools/hygiene_check.py` exit 0.
6. No Scala toolchain may be invoked (A3 offline) — `Get-Command scalac` must stay irrelevant.

## Orchestration protocol (non-negotiable — from the M18 stalls)

- Two bounded implementer handoffs: **T-A** = M19.1+M19.2 (new crate only — zero blast radius); **T-B** = M19.3 + D9 (the only wiring + test task). Each ends green on its scoped commands and **commits immediately** before the next begins.
- Every handoff carries: exact touch-set, scoped test commands (never `--workspace` until the final gate), "targeted diffs over full-file reads, checkpoint report beats context overflow" clauses, and a verified disk-state inventory. A session whose compaction reports "empty summary" is unrecoverable — start fresh with a disk-state inventory, never resume.
- Reviewer gate after each commit: APPROVED before proceeding; R1/R2/R4 + plan:381 capability-literal check (capabilities must match the plan table exactly — a `planning: true` slip is a blocker).
- Hygiene gate: `tools/hygiene_check.py` (fn < 100, file < 1000) + meaningful names; `parser.rs` does not exist yet — M21 owns the split prediction.
- Never script-edit UTF-8 files through `Set-Content`/`Out-File`/`>`; Read/Edit tools or byte-level UTF8 strict writes only, verified with `git diff --numstat`.

## Known environment facts

Fixtures are copied to temp dirs for runs (see `m18_cpp_plugin_parity/mod.rs::isolated_repo`); integration suites call `ariadne_cli` handlers directly (plan §6.3); `tests/integration/Cargo.toml` gains `ariadne-lang` + `ariadne-lang-scala` dev-deps; PowerShell 5.1, long cargo builds — timeouts 600000 ms.
