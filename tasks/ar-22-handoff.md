# AR-22 Handoff - `Pipeline::for_incremental(...)` (A6 part 1)

Wave 6 / Architecture seams. Crate (`pipeline`). Purely ADDITIVE constructor;
depends on nothing; AR-23 (cli consumes + deletes the mirror) follows.

## Objective

Add `Pipeline::for_incremental(...)` encapsulating the
locate/discover/shaper/restrict sequence for the incremental path that the CLI
mirrors today at `crates/cli/src/commands/analyze.rs` (~lines 32-56 — content
anchor; also read the self-admitted mirror comment at ~:110-114). The
constructor must produce a pipeline configured IDENTICALLY to what analyze.rs
builds by hand today, including the optional `--lang` unit restriction and
artifact-authority handling that the mirror carries.

## Impact

None observable yet; creates the single authority site AR-23 switches to.

## Touch-set (hard)

- `crates/pipeline/src/lib.rs` (+ new module file if the ctor needs room for
  hygiene: one-responsibility private helpers <100 lines each)
- `crates/pipeline/tests/for_incremental_parity_test.rs` (NEW)
- `tasks/ar-22-handoff-journal.md`

## Constraints

- STRICT COMPILE-GREEN discipline (shared worktree): additive code must keep
  `cargo check --workspace` green at all times; red phases are test-assertion
  red only.
- PARITY TEST, not new semantics: the test builds a Pipeline via
  `for_incremental` and via the analyze.rs-mirrored sequence (replicated in
  the test) on a temp fixture repo, asserting equivalent observable
  configuration (stage set, restriction, artifact paths — whatever is
  inspectable; journal what parity means concretely per the type's surface).
- Pure ASCII; commit_scoped pipeline scopes only.

## Gate commands

1. `python tools/testrun.py --distill -- -p ariadne-pipeline`
2. `python tools/testrun.py --distill -- --test m00_golden_baseline_test` and
   m00 facts-path parity — UNEDITED green (do NOT run the binary BUILD if
   another lane is mid-flight; if build contention observed, rerun and note)
3. `python tools/encoding_guard.py --all`; `python tools/hygiene_check.py`
   (zero NEW beyond the 2 server.rs rows)

## Invariants / locks

- Existing Pipeline constructors' behavior byte-identical (ctor untouched).
- m00 parity + goldens UNEDITED.
- No cli edits — AR-23 owns the switch + mirror deletion.
