# AR-19 Handoff Journal - Dispatch planning via registry; drop cpp hardwires (A1 part 2)

Implementer session lane. Depends verified: AR-18 landed at `d937ac0`
(`feat(lang): planning hooks on LanguagePlugin, cpp+scala overrides (A1 part 1)`)
with the approved verdict (`tasks/ar-18-verdict.md`). Wave 6, crate = cli +
lang-cpp rider + integration lock.

## What was done

1. **New registry seam** (`crates/cli/src/registry.rs`): `PlanningHooks<'a>` +
   `planning_hooks(&registry, &repo_root)`. Both hooks resolve the answering
   plugin per path through `registry.plugin_for(path)` - the same
   first-claiming-detector lookup `detect_language` performs and the catalog
   dispatch uses (D-A5: registry is the dispatch authority) - and consume the
   AR-18 trait methods `LanguagePlugin::aggregation_filter(repo_root, path)` /
   `target_path_for(aggregation_path, symbol_name)`. Unclaimed path: honestly
   not an aggregation unit (`false`) and candidate returned UNCHANGED (the
   `PathBuf` hook cannot error). No C++ default anywhere.
2. **optimize.rs switch**: the `use ariadne_lang_cpp::{cpp_aggregation_filter,
   cpp_target_path_for};` import is GONE. `plan_safe_extractions` now feeds
   the planner `crate::registry::planning_hooks(...)` (the exact form the
   `commands/plan.rs` lane already used). `analyze_collateral_opportunities`
   replaced its post-aggregation `.filter(cpp_aggregation_filter(name))`
   with per-file plugin dispatch through `plugin_for_catalog_file` + the
   trait hook (see deviation D5 for the C++ equivalence argument).
3. **ui.rs switch**: `build_extraction_proposals` takes
   `registry: &ariadne_lang::LanguageRegistry` + `repo_root: &Path` and feeds
   the planner the registry hooks; the positional cpp fn references at the
   old :395-396 are gone. `build_ui_payload` hoists ONE
   `default_registry()` per payload build (node maps + planning hooks share
   it; the value is identical to the inline call it replaces).
4. **plan.rs 2-arm match deleted**: the D-M23-7 i `PlanningHooks` struct,
   the `planning_hooks` 2-arm closures, and `language_of_path` are removed
   from `commands/plan.rs`; `plan_extractions` now calls
   `crate::registry::planning_hooks`. One dispatch for all three lanes.
5. **New integration lock**
   `tests/integration/tests/m24a_scala_plan_path_test.rs` (sibling-mod
   naming per handoff): copies `tests/fixtures/scala-isolation` (the m23
   witness fixture sized >=1 Scala extraction) to a temp repo and asserts
   (a) the `tool optimize` lane surfaces `legacy.scala` collateral and
   proposes >=1 safe plan, (b) the UI payload at `/api/export` carries a
   non-empty `extraction_proposals` for `legacy.scala` whose `target_files`
   rows are Scala-named (`LegacyReport.scala` beside the shared file, never
   `{symbol}_extracted.h`).

## Rider (AR-18 verdict): hook coexistence resolution

`CppPlugin` carried TWO hook paths: the inherent M18.6 pair
`aggregation_filter(&self, path) -> bool` / `target_path_for(&self,
aggregation_path, symbol_name) -> PathBuf` (plugin.rs impl block) and the
AR-18 trait overrides (returning `Some(...)`, delegating to the same
`crate::planning` free functions). Determination of which the planner
consumes: **the TRAIT pair.** Every live call site in the workspace resolves
through `&dyn LanguagePlugin` (registry-dispatched `plugin_for` returns, the
AR-18 parity tests via `as_plugin`, and the lang MockPlugin locks) - a
workspace grep of `\.aggregation_filter(` / `\.target_path_for(` found zero
callers of the inherent forms. The inherent pair was additionally a shadowing
hazard: on a concrete `CppPlugin` receiver the inherent methods win method
resolution, so future code could bypass the parity-locked trait silently.
Resolution taken: **deleted the dead inherent pair** (rider sanction: "delete
the redundant inherent path IF dead"); the trait override is now the single
hook source, its doc note points at the free-function delegation and the
parity lock. Zero tests removed (test-attr count reconciled below).

## Dead-code determination (handoff item 3)

`crates/lang-cpp/src/plan.rs` DOES NOT EXIST (never did; `git ls-files
crates/lang-cpp/**` confirms). The handoff's intended target is
`crates/lang-cpp/src/planning.rs`. Determination: **ALIVE, kept.** After the
switch it is still consumed by (a) the `CppPlugin` trait-hook delegation
(plugin.rs), (b) the `lib.rs` re-export `pub use planning::{...}` which is
the byte-identity oracle used by cli unit tests (plan.rs lane test,
registry.rs hook locks), `crates/cli/tests/payload_tail_bench.rs`, the UNEDITED
`m00_facts_path_parity_test.rs` and the UNEDITED m18 parity suite. Removing
it would break every C++ byte-identity authority the handoff itself locks.
Consequence: handoff gate 4 ("run -p ariadne-lang-cpp IF plan.rs was removed")
is N/A in its conditional form; the suite was run anyway for the rider and
is green (see gates).

## Cargo dep status (handoff invariants)

`ariadne-lang-cpp` and `ariadne-lang-scala` MOVED from `[dependencies]` to
`[dev-dependencies]` in `crates/cli/Cargo.toml`. Production cli src
references neither anymore (grep: remaining references are cfg(test) unit
tests and `tests/payload_tail_bench.rs`, both linked with dev-deps). The
binaries are unaffected: plugin enumeration is pipeline-side (D9) and
`ariadne-pipeline` carries its own lang-cpp/lang-scala deps. They survive as
dev-dependency oracles for the byte-identity locks. The stale `[dependencies]`
comment claiming registry.rs enumerates `CppPlugin` was corrected (it
delegated to pipeline since D9). One doc-comment intra-doc link to
`ariadne_lang_scala::ScalaPlugin` was de-linked (the crate left the non-dev
dep graph); wording preserved.

## TDD evidence (canonical lines, own stdout; capture files shared-worktree-race-prone)

- RED (before any src switch, only the new lock present):
  `0 passed / 2 failed / 0 ignored / 1 suites`
  optimize witness: `Opportunities seen: []`; UI witness: payload keys
  without `extraction_proposals` - the silent-empty bug demonstrated through
  the production lanes.
- GREEN (after the switch): `2 passed / 0 failed / 0 ignored / 1 suites`.

## Gates (verbatim digest lines)

1. `-p ariadne-cli`: `117 passed / 0 failed / 8 ignored / 6 suites`.
   Reconciles from the 118/0/8/6 baseline: 5 plan.rs unit tests retired into
   the seam move (-5), 1 plan-lane witness added (+1), 3 registry
   `planning_hooks_*` seam locks added (+3): 118 - 5 + 1 + 3 = 117.
2. m00 golden + facts-path parity UNEDITED + m18 parity UNEDITED + m24
   identity UNEDITED: `35 passed / 0 failed / 0 ignored / 4 suites`.
3. New scala suite re-gated with all UNEDITED locks in one run:
   `37 passed / 0 failed / 0 ignored / 5 suites`.
4. `-p ariadne-lang-cpp` (run for the rider, not for a removal):
   `153 passed / 0 failed / 0 ignored / 2 suites`; `git grep -c -F "#[test]"`
   per file is byte-equal between HEAD and the worktree (15 files, same
   counts) - the rider deleted zero tests.
5. `python tools/encoding_guard.py --all`: silent (exit 0).
6. `python tools/hygiene_check.py`: exactly the 2 inherited rows
   (`crates/ui/src/server.rs` R2 1172 + `api_graph_traversal` R1 161 - the
   AR-13 pair every Wave-6 journal cites). One transient NEW R1 was self-
   inflicted and fixed: `build_ui_payload` reached 104 lines with the hoist;
   compacting the node_maps call + a one-line comment returns it under the
   ceiling (gate re-run clean). All other rows are pre-existing R3 warnings
   (warn-only), none from files this handoff authored; the new integration
   file triggers no rows and sits far under the 1000-line ceiling (~250).

## Deliberately NOT done

- No edits to crates/pipeline, crates/lang, crates/lang-scala, crates/ui
  (only READ for the trait shape, shaper, DTO, routes).
- `crates/lang-cpp/src/planning.rs` kept intact (determination above).
- No fixture/golden regeneration of any kind.
- The optimize-lane `generated/` staging relocation inside `apply_safe_plan`
  was left as-is (pre-existing optimize machinery now reachable for Scala;
  changing its shape is out of contract).

## Deviations (with reasons)

- D1: `crates/cli/src/commands/ui/builder_tests.rs:156` call site updated
  (registry + root args added). The file is not on the handoff touch-set
  list, but the `build_extraction_proposals` signature change the handoff
  mandates makes ANY other outcome a compile error (violating the
  strict-compile-green constraint). Covered by the commit scope
  `crates/cli/src/**`; the test's semantics are unchanged (empty inputs
  yield no proposals under the dispatch too).
- D2: plan.rs unit-test consolidation (5 deleted / 1 added): the two
  `language_of_path` locks died with their function; the 2-arm-table and
  route-scala locks were re-expressed as the 3 registry-seam locks (cpp
  bytes, scala routing, honest no-claim answers incl. the D-A4 third-
  language and unclaimed-path assertions they locked) plus the plan-lane
  witness. Coverage moved to the seam all lanes share; net lock count +0
  at the seam, -4 in plan.rs, gate 1 reconciles.
- D3: handoff names `crates/lang-cpp/src/plan.rs`; the real file is
  `crates/lang-cpp/src/planning.rs` and it is ALIVE (see determination).
- D4: `build_ui_payload` now shares ONE `default_registry()` between
  node_maps and the planning hooks instead of calling it inline for
  node_maps only (same value, zero payload impact - m00 goldens UNEDITED
  green; R1-driven compaction kept the fn under 100 lines).
- D5: collateral-opportunity filtering moved from post-aggregation
  extension check on the file NAME to per-file plugin dispatch. C++
  equivalence: `cpp_aggregation_filter` is extension-only and a file's
  name shares its path's extension, so both orders accept/reject exactly
  the same keys (the old code aggregated non-header keys only to filter
  them out; the new code never aggregates them). Byte-equality authorities
  (m00 goldens, m14/m18 suites, cli unit regression lock
  `test_collateral_opportunities_skip_symbols_without_defining_file`) are
  green UNEDITED/unchanged.

## Addendum (post-commit)

- Main commit: `5de3884` (nine in-scope paths, `git status --short` over the
  touched scopes EMPTY after it; foreign lane churn - ui assets, report.json,
  AGENTS.md, other ar-* docs - left untouched and unstaged, as required in a
  shared worktree).
- `encoding_guard --staged` ran after staging/commit: silent (clean). Honest
  semantics note: raw `git add` is denied in this session, so staging happens
  inside `commit_scoped.py`; the new integration file was therefore guarded
  BEFORE the commit by `encoding_guard --all` (tracked set, silent) plus
  per-file byte scans proving zero non-ASCII bytes in every authored file
  (mojibake cannot arise from pure-ASCII content), and AFTER by the silent
  `--staged` run. All authored content is pure ASCII by construction.
- Mid-session landings by other lanes (`bcc08fc` AR-22 pipeline ctor,
  `15ffbfd` its journal wording) are ancestors of `5de3884`; the final
  `cargo check --workspace --all-targets` (ok, 1 pre-existing lib.rs:804
  warning, zero from this diff) and every gate line above ran on top of
  them.

