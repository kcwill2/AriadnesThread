# AR-21 Journal - Dual resolver -> gated test oracle (A5 MED)

Implementer session ses_ed4fbc3beffeFHHPif3LO9iPqV. Contract:
`tasks/ar-21-handoff.md`. Worktree `shining-enthusiasm`. All gate lines
quoted from this session's own stdout (shared-capture rule; parallel
AR-25 lane active).

## Phase 0 - pre-change baselines (own stdout, verbatim)

- `python tools/testrun.py --distill -- -p ariadne-lang-cpp`
  -> `155 passed / 0 failed / 0 ignored / 2 suites` (re-briefed anchor;
  contract's 153 was stale, +2 from AR-24 kind tests).
- `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m18_cpp_plugin_parity_test`
  -> `23 passed / 0 failed / 0 ignored / 1 suites` (parity lock count).
- Worktree dirt at dispatch: only `tests/fixtures/basic/artifacts/includes/
  report.json` CRLF churn (suite-run byproduct, D5 precedent) left
  unstaged; all other porcelain entries are untracked foreign docs.

## Mechanism ruling (contract: "journal the mechanism choice")

Choice: **test-only Cargo feature `test-resolver`, default OFF**, items
gated `#[cfg(any(test, feature = "test-resolver"))]`, plus
`#[deprecated]` with the dated removal note on the legacy surface.

Rejected alternatives (machine-checked against this workspace):

1. `#[cfg(test)]` alone: the m18 parity binary links the lang-cpp lib
   compiled WITHOUT `cfg(test)` (integration tests are external
   consumers), so `ariadne_lang_cpp::IncludeResolver` would vanish from
   the very build the oracle uses -> m18 binary fails to compile. The
   handoff's "parity suites keep working UNEDITED" is unsatisfiable on
   this route.
2. Gating only the `lib.rs` re-export: `ariadne_lang_cpp::resolver::
   IncludeResolver` (pub module path) stays reachable and COMPILED in
   production -> violates "Production build must no longer compile a
   live second resolver".
3. Profile sniffing (`debug_assertions`) or build-script cfg emission:
   not a named mechanism in the contract; a debug-profile `cargo build`
   of the tool is a non-test production build and would still contain
   the resolver -> violates the invariant. A build.rs would also be a
   new file outside the touch-set.

Consumer census (grep-verified this session):
- Production (normal-dep) consumers of ariadne-lang-cpp: `crates/pipeline`
  and `tools/clang-worker` only. Neither references `IncludeResolver`
  (zero grep hits). The feature stays default-OFF for them.
- Legacy-symbol consumers: ONLY the m18 parity binary
  (`tests/integration/tests/m18_cpp_plugin_parity/resolver_parity.rs`
  lines 215/290 and `.../external_boundary_parity.rs` lines 228/399,
  both modules of the single `m18_cpp_plugin_parity_test` binary), plus
  lang-cpp's own `#[cfg(test)]` unit tests in `resolver.rs`.
- `resolver::normalize_path` (pub free fn, not re-exported in lib.rs):
  no consumer outside `resolver.rs`. Gated with the legacy surface.

## D1 - forced one-line wiring in tests/integration/Cargo.toml (deviation)

The touch-set names `crates/lang-cpp/Cargo.toml (feature wiring if
chosen)` and `resolver_parity.rs (FEATURE WIRING ONLY ...)`. A default-
OFF feature must be ENABLED by the consuming test package's manifest;
Cargo features cannot be activated from test source code, and the
`resolver_parity.rs` file therefore needs ZERO edits (satisfying "zero
assertion edits" trivially). The single mechanically forced line is the
existing dev-dep edge:

`ariadne-lang-cpp = { path = "../../crates/lang-cpp" }` ->
`ariadne-lang-cpp = { path = "../../crates/lang-cpp", features = ["test-resolver"] }`

Without it, `cargo test --workspace` (the census every lane runs) fails
to compile `m18_cpp_plugin_parity_test` (E0432) -> the RED trail below
is the machine proof that the wiring is forced, not chosen. Precedents
for mechanically forced wiring lines outside the literal glob, accepted
on merits by review: AR-03 D1 (mod-registration line), AR-24 D2
(mock-signature lines forced by E0053). The file is the manifest of a
tests-only package: zero production impact; production builds of
pipeline/clang-worker/tool compile the feature-free lib (legacy resolver
not compiled at all, not merely hidden). Parallel-lane collision risk:
none - AR-25 (in flight) owns crates/cli + crates/report; the last edit
of this manifest on main is AR-17's.

## Status / resume notes

- Phase 0 complete. Next: implement gating (resolver.rs + lib.rs +
  lang-cpp Cargo.toml), capture RED absence trail (no wiring), then add
  wiring line, run all gates, atomic commit.
- Frozen locks observed: m17 transitive cone guard ({build, lang,
  lang-api, repo, worker-core, index}) - this handoff adds NO cargo
  dependencies, only a feature; M25/M27 parser/tokenizer byte locks -
  untouched.

## Implementation (landed shape)

- `resolver.rs`: legacy region gated `#[cfg(any(test, feature =
  "test-resolver"))]` item-by-item (gated imports, `normalize_path`,
  `collect_include_candidates`, `find_in_discovery`, `IncludeResolver` +
  impl). Both deprecated surfaces carry dated notes: "remove when the
  m18_cpp_plugin_parity oracles retire - dated 2026-10-11". Module docs
  updated to the gated reality (the intra-doc `[`IncludeResolver`]` link
  was plain-coded to avoid an unresolved-link warning in feature-off doc
  builds).
- `lib.rs`: re-export split; the `IncludeResolver` line is
  `#[cfg(any(test, feature = "test-resolver"))]` + `#[allow(deprecated)]`
  (deliberate re-export of the deprecated oracle surface).
- `crates/lang-cpp/Cargo.toml`: `[features] test-resolver = []` with the
  consumer census comment. No dependency added (cone unchanged - m17
  guard green). Cargo.lock UNCHANGED (feature-only, no dep-graph edge).
- D2 (in-touch-set, disclosed): `#[allow(deprecated)]` on the deprecated
  surface's own implementation sites (`find_in_discovery`,
  `IncludeResolver` impl, lib re-export) - conventional shim hygiene:
  the gated code must not self-warn; CONSUMER-side use (lang-cpp unit
  tests, m18 oracle) keeps warning = the dated removal clock stays
  audible. Effect: feature-on lang-cpp LIB build = zero warnings.

## RED-to-verify trail (feature-gate off-state, locked red-first)

With gating landed and wiring ABSENT (transient, never committed), the
default integration build lost the legacy surface - the off-state
absence lock:

- `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m18_cpp_plugin_parity_test`
  -> `### cargo test DRIFT (exit 2) ... parse format drift` (compile
  failure before any test line), capture
  `target/testrun-capture.txt` (own run): 4x
  `error[E0433]: failed to resolve: could not find `IncludeResolver` in
  `ariadne_lang_cpp`` at EXACTLY the four legacy consumption sites
  (`resolver_parity.rs:215/:290`, `external_boundary_parity.rs:228/:399`).
  Machine proof: a feature-off (production-shaped) build does not merely
  HIDE the legacy resolver, it does not COMPILE it - and the only
  missing-symbol consumers in the workspace are the oracle files
  themselves (proof D1 is forced, not chosen).

## Gate trail (final state, own stdout, verbatim)

Capture paths cited are own-run digests (shared capture, capture-race
rule; lines quoted from stdout, not the file).

1. `python tools/testrun.py --distill -- -p ariadne-lang-cpp`
   -> `155 passed / 0 failed / 0 ignored / 2 suites` (default features;
   count byte-equal to the re-briefed 155 anchor - contract's 153 stale,
   +2 = AR-24 kind tests).
2. `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m18_cpp_plugin_parity_test`
   -> `23 passed / 0 failed / 0 ignored / 1 suites` - parity lock count
   UNCHANGED; both oracle module files BYTE-IDENTICAL to HEAD
   (`git diff --stat HEAD -- tests/integration/tests` lists only foreign
   AR-25 WIP `m24_language_identity_test.rs`; the m18 files absent).
   BOTH SIDES EXECUTE - legacy-side probe evidence (own capture, final
   run): `plugin_closure_matches_vendored_legacy_bfs_basic ... ok`,
   `..._bfs_expensive ... ok`,
   `resolve_target_outcome_table_matches_legacy_resolver_basic ... ok`,
   `..._expensive ... ok`, `plugin_closure_matches_vendored_legacy_external_boundary ... ok`,
   `resolve_target_outcome_table_matches_legacy_resolver_external_boundary ... ok`
   (all 6 IncludeResolver-driven left-side tests green; with the gate
   off they would not compile - RED trail above).
3. Feature route chosen, both configurations:
   `python tools/testrun.py --distill -- -p ariadne-lang-cpp --features test-resolver`
   -> `155 passed / 0 failed / 0 ignored / 2 suites` (parity of counts
   across configs).
4. m00 pair UNEDITED: `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test`
   -> `4 passed / 0 failed / 0 ignored / 2 suites`.
   m17 cone guard (invariant lock):
   `python tools/testrun.py --distill -- -p ariadne-integration-tests --test m17_plugin_seam_test`
   -> `8 passed / 0 failed / 0 ignored / 1 suites` (cone free of
   scheduler/new deps - feature-only change).
5. STRICT compile-green:
   `python tools/distill.py -- cargo check --workspace --all-targets`
   -> `ok (exit 0, 31 warnings emitted)` (capture
   `target/distill-capture.txt`). Warning attribution: ALL
   AR-21-attributable diagnostics carry the dated AR-21 note
   (`ariadne-lang-cpp (lib test) generated 10 warnings` = unit-test use
   sites; `m18_cpp_plugin_parity_test generated 12 warnings (7
   duplicates)` = oracle use sites - oracle files uneditable, warnings
   are the intended dated signal; feature-on `ariadne-lang-cpp (lib)`
   emits NO warning summary = zero). The remainder (ui/cli singletons,
   integration `common` golden-helper dead_code x20 duplicates, m10
   unused imports) = pre-existing classes, none AR-21.
   Production absence proof: `python tools/distill.py -- cargo check -p
   ariadne-pipeline` -> `ok (exit 0, 0 warnings emitted)` - the
   production graph (pipeline -> lang-cpp, feature-off, no cfg(test))
   compiles the legacy resolver out entirely.
6. `python tools/encoding_guard.py --all` -> silent (exit 0), pre-commit;
   `python tools/hygiene_check.py` -> ZERO blocking rows; every reported
   row is R3 (warn); `resolver.rs` absent from findings (file ~920
   lines < 1000, no function grown). Encoding `--staged` after the
   scoped commit per AR-20b D2 practice (raw `git add` allowlist-denied;
   commit_scoped stages + the `--all` superset covered the content).

## Oracle doc-comment note

The uneditable oracle files' doc comments still say the legacy resolver
is "public, non-deprecated" (written pre-AR-21). Public: still true;
non-deprecated: stale by mandate (this handoff deprecates it). Editing
those comments is outside FEATURE-WIRING-ONLY permission; banked as a
wording row for whoever retires the oracles (the dated notes point here).

## Foreign-tree handling

- `tests/fixtures/basic/artifacts/includes/report.json`: CRLF churn from
  suite runs, left unstaged (D5 precedent).
- `tests/integration/tests/m24_language_identity_test.rs`: appeared
  dirty mid-run - foreign AR-25 lane WIP; left unstaged, excluded from
  my scopes; not caused by and not affecting any gate above (all gates
  green throughout).

