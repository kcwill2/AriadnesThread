# AR-21 Handoff - Dual resolver -> gated test oracle (A5 MED)

Wave 6. Crate (`lang-cpp`). Dispatch after AR-19 AND AR-20 approved
(same-crate serialization; AR-20 owns Cargo.toml + lib.rs churn first).

## Objective

`crates/lang-cpp/src/resolver.rs` carries the legacy dual `IncludeResolver`
implementation that exists only to feed the parity oracles. Deprecate it
(`#[deprecated]` + dated removal note) and cfg-gate it behind a test-only
feature (or `#[cfg(test)]` + dev-dependency wiring for the parity harness —
journal the mechanism choice). Production build must no longer compile a
live second resolver; the parity suites keep working UNEDITED.

## Impact

One fewer same-responsibility live implementation; the drift risk between
the two resolvers becomes a locked, dated, test-only artifact instead of a
silent fork.

## Touch-set (hard)

- `crates/lang-cpp/src/resolver.rs` (attributes/gating)
- `crates/lang-cpp/src/lib.rs` (cfg-gated module declaration)
- `crates/lang-cpp/Cargo.toml` (feature wiring if chosen)
- `tests/integration/tests/m18_cpp_plugin_parity/resolver_parity.rs`
  (FEATURE WIRING ONLY if the oracle needs the feature enabled; zero
  assertion edits)
- `tasks/ar-21-handoff-journal.md`

## Constraints

- Parity oracle semantics UNCHANGED: the m18_cpp_plugin_parity comparison
  (legacy vs gated resolver) must still execute BOTH sides under the test
  feature — if gating accidentally deletes the oracle's left side, the
  handoff is void (this is the RED-to-verify: run the parity suite, prove
  both sides still run — journal the evidence line, e.g. legacy-side probe
  or coverage of the deprecated path).
- `#[deprecated]` + note format: include the dated removal condition
  (e.g. "remove when m18 parity oracle retires — dated 2026-10-xx").
- STRICT COMPILE-GREEN; default feature set must build with zero
  deprecation warnings ESCALATED into the 153-count (warnings ok, count
  must stay 153/0/0/2 — journal exact state).
- Pure ASCII; commit_scoped lang-cpp + integration (if wired) + journal.

## Gate commands

1. `-p ariadne-lang-cpp` (153/0/0/2 — default features)
2. m18 parity suite UNEDITED green (23/0/0/1)
3. `-p ariadne-lang-cpp --features <test-resolver feature>` if feature route
   chosen (journal the command; parity both configurations)
4. m00 golden + facts-path UNEDITED
5. `encoding_guard --all`; `hygiene_check.py` (current baseline — quote)

## Invariants / locks

- Production (non-test) build contains NO second resolver implementation
  reachable from lib consumers.
- m18 parity lock count unchanged (oracle did not weaken).
