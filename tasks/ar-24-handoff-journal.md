# AR-24 Handoff Journal - Typed errors at the plugin seam (A7)

Landing: `8497a67 refactor(lang-api): typed error enum at the plugin seam
(A7)` - single cross-crate commit, 18 paths, touch-set + declared riders R1-3
exact (journal-only docs rider commit follows per AR-15/AR-16 precedent).
Post-commit verification on the committed tree: lang-cpp
`155 passed / 0 failed / 0 ignored / 2 suites`;
`encoding_guard --all` silent.

Base HEAD: `cefbd6f` (branch shining-enthusiasm). One cross-crate commit
(AR-18 precedent, dispatch mandate): lang-api + lang + impl crates +
workspace Cargo.toml + Cargo.lock + compile-forced mock riders + this journal.

## Summary

`CompilerValidator::validate_compile` was the ONLY trait method returning
`Result<_, String>` at the plugin seam (reached through
`LanguagePlugin::validator()`). Converted to
`Result<(), PluginError>` with a new thiserror enum in
`crates/lang-api/src/error.rs`. Every CLI-visible error STRING is
byte-identical (byte-locks captured pre-change, re-green post-change,
non-vacuity proven by two reverted mutations). STRICT COMPILE-GREEN held
after every edit (`cargo check --workspace --all-targets`).

## Per-method typed-vs-anyhow decision table

| Site | Today's signature | Decision | Justification |
| --- | --- | --- | --- |
| `CompilerValidator::validate_compile` (lang trait, the plugin seam) | `Result<(), String>` | **typed `PluginError`** | Structure adds a real distinction: a tier-3 consumer must separate "validation could not run" (report honestly / Skipped) from "the compiler ran and said no" (Fail, rollback). Kind-branching + AR-25 downcast both need the type. |
| `CompilerValidator::toolchain_status` | `ToolchainStatus` (no Result) | no decision needed | already typed data |
| `LanguagePlugin::{detector, analyzer, resolver, constraints, extractor, rewriter, validator, id, modeling, capabilities, analysis_fingerprint, shaper}` | no Result | n/a | value-returning accessors; no String error traffic exists |
| `LanguagePlugin::{aggregation_filter, target_path_for}` (AR-18 hooks) | `Option<...>` | n/a | documented Option idiom, no error strings exist |
| `cli::registry::parse_language_arg` | `Result<SourceLanguage, String>` | STAYS String | CLI-arg parsing surface, not the plugin trait seam; clap's `value_parser` contract just wants `Err: Display`; conversion is AR-25 territory. cli lock stays green UNEDITED. |
| `cli::registry::incremental_backend` | `Result<&dyn LanguagePlugin, String>` | STAYS String | cli-side dispatch helper outside the touch-set ("DO NOT re-wrap yet"); the single `.map_err(anyhow::Error::msg)` re-wrap at analyze.rs:39 STAYS per AR-25 ownership. Its locks stay green UNEDITED. |
| scheduler `worker.rs:79` / engine / pipeline `stage_index` / `for_incremental` "unhandled language" texts | `anyhow::Result` | NO change (already typed = anyhow) | not `Result<_, String>`; AR-16 contract text locked by scheduler suite + m18 parity + m24 -- all green UNEDITED. |
| worker-core `WorkerBackend::analyze_task` | `anyhow::Result` | NO change | already anyhow; the lang trait change did NOT force worker-core (analyze_task never reaches `validate_compile`). Dispatch's "any such case" => none occurred. |

## PluginError shape (lang-api/src/error.rs, thiserror)

- `ValidationNotWired { message }` - plugin has not implemented validation yet.
- `ToolchainUnavailable { message }` - toolchain absent / deferred / vanished.
- `CompileFailure { message }` - compiler ran and reported failure.
- `Display` renders `{message}` verbatim for every variant (pure passthrough).
- Rationale (message-carrying kinds, not language-specific variants): a
  neutral API must not hardcode one language's spelling -- the same rule
  that keeps `Other` out of `LanguageId`. The plugin owns wording, the
  seam owns kind. Kind discrimination is exactly what the old String
  could not give; three kinds answer the one consumer question:
  "could the validation run at all?".
- Derives `Debug, Clone, PartialEq, Eq, Error` -- `PartialEq` keeps the
  lang-scala `assert_eq!(result, Ok(()))` lock byte-intact.

## Kind mapping decisions

| Forced error | Kind | Note |
| --- | --- | --- |
| C++ M23 placeholder | `ValidationNotWired` | not-wired, never a compile failure |
| Scala offline deferral (D-M23-12) | `ToolchainUnavailable` | could not run (probe-backed) |
| Scala vanished between probe and compile | `ToolchainUnavailable` | nothing compiled |
| Scala compiler report | `CompileFailure` | the compiler said no |

## String-stability evidence (same forced errors, pre -> post)

Byte-locks landed FIRST against the pre-change `Result<(), String>` API
(`error.to_string()` renders identically pre and post) and re-verified
after conversion. Capture runs quote the suite lines verbatim below.

| Forced scenario | The locked string (byte-identical through Display) |
| --- | --- |
| C++ placeholder | `C++ CompilerValidator not wired yet (M23)` |
| Scala offline deferral | `Scala compile deferred (D-M23-12): no usable Scala toolchain (scalac/scala-cli/sbt) is present; offline path stays Missing and tier 3 reports Skipped` |
| Scala failure (stderr-first) | `scalac exited unsuccessfully: error: expected type` |
| Scala failure (stdout fallback) | `scalac exited unsuccessfully: warning-only output` |
| Scala failure (no output) | `scalac exited unsuccessfully: no compiler output` |
| Scala vanished toolchain | `scalac became unrunnable between probe and compile` |

AR-16 cross-language missing-language contract text (NOT converted --
anyhow paths, their locks stayed UNEDITED and green): scheduler
`unhandled language for {path:?}` (worker.rs:79, AR-16 locks at
:478-542), pipeline/engine/stage_index
`unhandled language for {} (no registered plugin claims it)`
(m18 parity :57, m24 :379 contain-locks).

Non-vacuity mutation proofs (reverted after capture, AR-16 precedent):

- Mutation A: CompileFailure Display mutated to prefix `AR24-MUTATION ` ->
  lang-scala `224 passed / 3 failed / 0 ignored / 1 suites` -- exactly the
  three compiler-failure byte-locks, every other test green. Reverted.
- Mutation B: failure path constructed as `ToolchainUnavailable` with the
  BYTE-IDENTICAL message -> `5 passed / 1 failed / 0 ignored / 1 suites`
  (ar24 filter) -- only the kind test, byte-locks green. Text and kind are
  locked independently. Reverted; restored `227 passed / 0 failed /
  0 ignored / 2 suites`.

## New locks (error-kind branching -- match on variant, not string)

- `crates/lang-api/tests/plugin_error_seam_test.rs` (4 tests): identical
  texts still discriminate by kind; Display passthrough for every variant
  (sentinel text, independent of plugin wording); downcast through the
  anyhow chain (the AR-25 transport); downcast as `&dyn std::error::Error`.
- lang-cpp `ar24_validate_compile_error_branches_by_kind` - forced through
  `&dyn CompilerValidator`, the actual plugin seam.
- lang-scala `ar24_error_kinds_discriminate_unrunnable_from_failed_validation`.

## Consumer edits

No production call site pattern-matches a `validate_compile` error STRING:
workspace-wide grep of `validate_compile` call sites found ZERO production
callers (the tier validator never invokes it -- documented D-M23-12 state;
only tests and the lang-scala internals). So no site needed a typed match
in this handoff; the capability is demonstrated by the new kind tests and
consumed by AR-25. No cli compile-edit was required at all (cli suite
`114/0/8/6` matches baseline with cli untouched).

Compile-forced mock `impl CompilerValidator` signature adaptations
(E0053 is unavoidable for ANY trait-signature change; every body stayed
`Ok(())`, every word lock stayed untouched -- declared riders):

- `crates/lang/src/registry.rs` MockValidator (in-lane lang test)
- `crates/graph/src/builder.rs` builder_seam_tests SeamValidator - signature only
- `crates/validate/src/lib.rs` FixedToolchain - signature only (toolchain_status untouched)
- `crates/scheduler/src/worker.rs` MockValidator - signature only (AR-16 word locks untouched)
- `tests/integration/tests/m17_plugin_seam_test.rs` SeamValidator - signature only
- `tests/integration/tests/m24_language_identity_test.rs` NeutralValidator - signature only

In-lane test adaptations (lang-scala `src/validator/tests.rs`): three
existing contains-locks changed `error.contains(..)` ->
`error.to_string().contains(..)` (same literals, same assertions);
`assert_eq!(result, Ok(()))` unchanged via `PartialEq`.

## Deliberately NOT done

- No `.map_err(anyhow::Error::msg)` removal (analyze.rs:39 stays; AR-25).
- No cli String-helper conversion (decision table).
- worker-core untouched (already anyhow, not forced).
- m18 parity + m16 suites: ZERO diff (byte-parity authorities).
- No new production deps beyond the sanctioned thiserror (below).

## Dependencies added (declared)

- R1: workspace `thiserror = "2.0"` + lang-api dependency + Cargo.lock
  (thiserror 2.0.21 resolved from local cache, offline-safe). Dispatch
  pre-authorized this addition ("workspace Cargo.toml dep addition
  allowed for it").
- R2: lang-api DEV-dependency `anyhow.workspace` - transport proof only
  (downcast test); production seam stays thiserror-typed. Journal deviation D1.

## Gate lines (own-run stdout; testrun capture path printed per digest: target/testrun-capture.txt; distill capture: target/distill-capture.txt)

Pre-change baselines:

- lang-api `37 passed / 0 failed / 0 ignored / 3 suites`
- lang `35 passed / 0 failed / 1 ignored / 2 suites` (dispatch baseline 35/0/1i confirmed)
- lang-cpp `153 passed / 0 failed / 0 ignored / 2 suites` (dispatch 153/0/0/2 confirmed)
- lang-scala `221 passed / 0 failed / 0 ignored / 2 suites` (dispatch 221/0/0/2 confirmed)
- cli `114 passed / 0 failed / 8 ignored / 6 suites`; scheduler `26 passed / 0 failed / 0 ignored / 2 suites` (dispatch correction to 26 post-AR-20a confirmed)
- m18 parity `23 passed / 0 failed / 0 ignored / 1 suites`; m17 seam `8 passed / 0 failed / 0 ignored / 1 suites`; m24 `8 passed / 0 failed / 0 ignored / 1 suites`; m16 traversal `2 passed / 0 failed / 0 ignored / 1 suites`; m16 clang-worker `3 passed / 0 failed / 0 ignored / 1 suites`
- `cargo check --workspace --all-targets` ok (exit 0, 21 warnings - all pre-existing)

Characterization capture (locks green BEFORE conversion):
lang-cpp `154 passed / 0 failed / 0 ignored / 2 suites`;
lang-scala `226 passed / 0 failed / 0 ignored / 2 suites`.

Post-conversion (final, after the last byte-level comment edits):

- lang-api `41 passed / 0 failed / 0 ignored / 4 suites` (+4 seam tests, +1 suite)
- lang `35 passed / 0 failed / 1 ignored / 2 suites` (exact baseline match)
- lang-cpp `155 passed / 0 failed / 0 ignored / 2 suites` (+2: byte-lock + kind)
- lang-scala `227 passed / 0 failed / 0 ignored / 2 suites` (+6: 5 byte-locks + 1 kind)
- cli `114 passed / 0 failed / 8 ignored / 6 suites` (baseline exact; zero cli edits)
- scheduler `26 passed / 0 failed / 0 ignored / 2 suites` (AR-16 locks green)
- m18 parity `23 passed / 0 failed / 0 ignored / 1 suites` (file ZERO diff)
- m17 seam `8 passed / 0 failed / 0 ignored / 1 suites` (mock signature line only; transitive guard rows green)
- m24 identity `8 passed / 0 failed / 0 ignored / 1 suites` (mock signature line only)
- m16 traversal `2 passed / 0 failed / 0 ignored / 1 suites`; m16 clang-worker `3 passed / 0 failed / 0 ignored / 1 suites` (both ZERO diff)
- rider crates: graph `129 passed / 0 failed / 0 ignored / 6 suites`; validate `16 passed / 0 failed / 0 ignored / 5 suites`
- post-conversion `cargo check --workspace --all-targets` ok (exit 0, 21 warnings - identical count, none from this diff)
- `python tools/hygiene_check.py`: zero R1/R2 blocking rows (all printed rows are
  warn-only R3 on pre-existing untouched lines; none inside this diff)
- `python tools/encoding_guard.py --all`: silent (clean); `--staged` after staging: silent

## Deviations

- D1: lang-api dev-dependency `anyhow` added (transport proof for the
  AR-25 downcast path); dev-only, reason above.
- D2: the "UNEDITED lock suite" constraint collides with STRICT
  COMPILE-GREEN at the mock `impl` signatures (E0053 admits no
  design that keeps `Result<(), String>` impls compiling). Resolution:
  the LOCKS (assertions/wording) are byte-untouched in every lock suite;
  m18 parity and both m16 files have ZERO diff; m17/m24 carry the one
  forced mock signature line each. Suite lines above prove no assertion
  moved.
- D3: lang/lib.rs gained the one-line `pub use ariadne_lang_api::error::
  PluginError;` re-export, following the established targeted re-export
  idiom (ToolchainStatus precedent) so implementor crates resolve the
  type through `ariadne_lang::`.
