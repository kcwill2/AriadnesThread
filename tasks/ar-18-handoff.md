# AR-18 Handoff - Planning hooks onto LanguagePlugin (A1 part 1)

Wave 6 / Architecture seams. MULTI-CRATE INTENTIONAL (seam change is
indivisible); gates stay per-crate scoped. Depends: none — purely additive;
defaults keep every current caller compiling. AR-19 (cli dispatch) switches
later.

## Objective

Add planning hooks to the `LanguagePlugin` trait with default "planning
unsupported" implementations:
- `aggregation_filter(&self, repo_root, path)` 
- `target_path_for(...)` (mirror the C++ function's real arity — read it)

C++ and Scala OVERRIDE with their existing free functions; normalize the
curried-vs-direct signature divergence between them.

## Known anchors (verify at start; content-anchor everything)

- Existing C++ free functions: `cpp_aggregation_filter`, `cpp_target_path_for`
  — consumed today at `crates/cli/src/commands/ui.rs` `build_extraction_proposals`
  (~line 395) passed positionally into the planner.
- Scala equivalents: look in `crates/lang-scala/src/planning.rs` +
  `plugin.rs`.
- Plugin trait definition: locate `LanguagePlugin` (crates/lang or lang-api);
  read the trait + both plugin impls before designing hook shapes.
- `crates/cli/src/optimize.rs` (~:102) imports the cpp fns directly (the A1
  hardwire) — NOT edited here; AR-19 consumes the seam.

## Impact

None observable (defaults keep callers compiling); removes the need for CLI
edits when a language joins — the seam now carries planning.

## Touch-set (hard)

- plugin trait file (find it; trait + default impls)
- `crates/lang-cpp/src/plugin.rs` (+ plan module exports if needed)
- `crates/lang-scala/src/plugin.rs`
- per-crate tests (trait default behavior lock; cpp/scala override parity vs
  the free functions they wrap)
- `tasks/ar-18-handoff-journal.md`

## Constraints

- TDD: override-parity locks first — plugin hook output must equal the free
  function output on the same inputs (cpp now; scala now); default impl locked
  as "planning unsupported" (define the shape: Option::None / explicit error
  variant — choose per the trait's existing error style, journal it).
- No behavior change anywhere; cli/cluster/planner call sites UNEDITED.
- Pure ASCII; commit_scoped with one scope per crate (NEVER sweep other
  lanes' files); distilling gates.

## Gate commands (per-crate scoped)

1. `python tools/testrun.py --distill -- -p ariadne-lang`
2. `python tools/testrun.py --distill -- -p ariadne-lang-cpp`
3. `python tools/testrun.py --distill -- -p ariadne-lang-scala`
4. m18 parity + m24 identity suites UNEDITED green (plugin contracts locked by
   integration)
5. `python tools/encoding_guard.py --all`; `python tools/hygiene_check.py`
   (zero NEW — only the 2 server.rs AR-13 rows blocking)

## Invariants / locks

- Existing free functions remain exported and working (AR-19 removes them).
- m17 seam guard green; plugin trait object stays dyn-compatible (no generic
  methods).
