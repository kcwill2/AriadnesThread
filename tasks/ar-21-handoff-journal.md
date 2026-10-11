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
