# M24-Part-A T1 Journal — Open `LanguageId`

Base `fe1d649`. Branch `multi-language-support`. Executed in the architect session because a
global permission rule denies `bash` to every subagent (`{"permission":"bash","action":"deny",
"pattern":"*","source":"global"}`), which makes implementer/tool-caller delegation impossible
on this branch. Verified by probe against a `tool-caller` subagent.

## Red

`cargo test -p ariadne-lang-api --no-run` → **exit 101, 12 compile errors**, all of the form
"no variant or associated item named `new`/`as_str`/`cpp`/`scala`/`other`/`CPP`/`SCALA`/`OTHER`
found for enum `ariadne_repo::SourceLanguage`", plus "`SourceLanguage` doesn't implement
`std::fmt::Display`". The open type was genuinely absent.

## Green

`cargo test --workspace --no-fail-fast` → **exit 0, TOTAL passed=1092 failed=0 ignored=3**
(baseline at `fe1d649` was 1088/0/3; +4 is the net of 7 identity tests replacing 3).

## Compiler-error census (D-A4 discovery mechanism)

Sites the type split broke, and how each resolved:

| Site | Resolution |
|---|---|
| `build/src/discovery.rs:245,264` (`ctx.language`, `tu.language`) | use `record.language` (ADAPT-8) — both fields stay `SourceLanguage` |
| `graph/src/builder.rs` `edge_from_outcome`/`resolved_edge`/`include_edge` + 3 guard sites | param type → `LanguageId`; guard sites → `LanguageId::cpp()` |
| `graph/src/graph.rs` `edge()` helper + 5 `IncludeEdge` literals + 3 `let cpp` | → `LanguageId` / `LanguageId::cpp()`; `cpp.clone()` at call sites |
| `graph/src/{traversal,load_*_tests}.rs` 12 `IncludeEdge` literals | → `crate::types::LanguageId::cpp()` |
| `cluster/src/planner.rs:128` | `gap.language.clone()` (no longer `Copy`) |
| `cluster/src/planner.rs:678` | `matches!` const pattern impossible (`Cow` is not structural-match) → equality guard |
| `cluster/src/frontier.rs` 3 sites | → `ariadne_lang::LanguageId::cpp()` |
| `report/src/manifests.rs` 4 `languages` fields + `collect_stage_languages` | → `Vec<LanguageId>`, generic over `Into<LanguageId>` |
| `report/src/manager/{graph_io,metrics_io}.rs` | `.clone()` / `LanguageId::from(..)` |
| `index/src/cache.rs` `TuCacheFingerprint.language` + `compute` | → `LanguageId` (ADAPT-5), `Default` = Cpp, epoch NOT bumped |
| `scheduler/src/tu_store.rs:357` `compute_tu_closure_plugin` | `language: LanguageId` (ADAPT-5); 5 `plugin.id()` callers unchanged |
| `scheduler/src/{engine,worker}.rs`, `pipeline/src/lib.rs` (4 sites), `cli/*` (9 sites) | `.clone()` or `LanguageId::from(..)` |
| `lang-cpp/src/{resolver,shaper}.rs`, `lang/src/registry.rs:388,392` | `IncludeEdge.language` → `LanguageId::Cpp`; `self.language.clone()` |

**Not** `LanguageId`-typed and correctly left alone: `FileRecord.language`,
`TuAnalysis.language`, `TranslationUnit.language`, `CompilationContext.language`.

## Frozen-lock status

- `tests/golden/**` — **empty diff**. `m00_golden_baseline_test` green (2/2).
- `Cargo.lock` — **empty diff**, 0 packages added. `crates/repo` — **empty diff**.
- `python tools/hygiene_check.py` — exit 0.
- `m17_plugin_seam_test.rs` — **unedited**, green (6/6), including
  `core_crates_do_not_name_languages`. Its `LanguageId::Scala` value uses are why ADAPT-4 keeps
  PascalCase value constants. It caught two of my own violations
  (`crates/report:243,281` — `LanguageId::Cpp` inside a guarded crate); fixed to `cpp()`.
- **4 frozen integration tests needed a value-preserving type edit** (12 lines total) —
  `m18_cpp_plugin_parity/{external_boundary_parity,resolver_parity}.rs`,
  `m20_scala_dependency_graph_test.rs`, `m22_scala_load_analysis_test.rs`,
  `m23_symbol_isolation_test.rs`, plus non-frozen `m19`, `m21`. All pass. Reported in closeout.

## Deviations

1. `crates/cli/src/optimize.rs:523` **was** touched, contrary to the kickoff's scope note.
   The kickoff states Part A "does not surface `optimize.rs`". False: its
   `test_include_edge` helper builds an `IncludeEdge`, whose `language` is now a `LanguageId`.
   Only the one literal changed — no Part D hook redesign.
2. `crates/ui/Cargo.toml` NOT modified; `LanguageId` re-exported from `graph::types` so
   `ariadne-ui` reaches it with no new dependency edge and no `Cargo.lock` change.