# R-5 Handoff - rust-language: the syn analyzer, fingerprint "rust-1", semantic_index flip, span truth

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). Hand 5
of 10, serial. Replaces the never-dispatched analyzer stub with the REAL
`syn`-backed analyzer (pre-split into three files per shape sec. 7), lands
`analysis_fingerprint() == "rust-1"`, flips `semantic_index`, and makes D-2's
correctness guarantee falsifiable via exact line/column (INV-7). Start rev:
your dispatch head (R-4's landing + verdict via duty 1 - your duty 1 ALSO
carries the dated fixture-approval ledger note, R-4's clause).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/rust-language-shape.md` sec. 2 INV-5/INV-7/INV-8/INV-13, sec. 3.4
   (capability schedule), sec. 3.7 analyzer row (item set + qualified ids +
   `use`-edge stance + macro honesty), sec. 7 (pre-split rule: analyzer.rs
   crosses ~600 -> `parser_items.rs` + `parser_uses.rs`). Kickoff D-2 + L2.
3. Model files: `crates/lang-scala/src/{analyzer.rs,imports.rs,locator.rs}`
   (structure only - Scala's hand-rolled scanner is not your mechanism; your
   mechanism is `syn` + `proc_macro2` spans), `crates/lang-api`/`crates/lang`
   `SymbolRecord`/`TuAnalysis` shapes via `crates/lang-scala/src/analyzer.rs`
   usage.
4. The spans feature proof: root `Cargo.toml` `[workspace.dependencies]`
   already carries `proc-macro2` with `span-locations` (R-3). Without the
   feature the `start().line()` accessors do not exist at all - compile
   forces the feature (INV-7 secondary oracle; RISK-SYN-SPAN).
5. The warm/cold witnesses you keep green UNEDITED:
   `tests/integration/tests/m21_scala_semantic_index_test.rs` (warm/cold
   cache idiom), `m18_cache_epoch_test.rs`, `m18_persistence_incremental_test.rs`,
   `ar27_context_bytes_lock_test.rs`, CURRENT_ANALYSIS_VERSION (frozen
   `2.1.0`, M27) at `crates/index` cache.
6. Your duty-1 targets: `tasks/rust-language-r4-verdict.md` + ledger.

## Duty 1 (before any R-5 mutation)
Land R-4's verdict + ledger flip: `R-4 LANDED <SHA> APPROVED <date>, census
anchor <line> @ <SHA>, fixture rust-basic APPROVED <date> at <SHA> via R-4
verdict (L3 approval path discharged), R-5 dispatched.` (both clauses, one
commit, same shape as R-2's duty-1.)

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/rust-language-r5-journal.md (create)
- tasks/rust-language-r4-verdict.md + tasks/milestone_ledger.md (duty 1 only)
- crates/lang-rust/src/analyzer.rs (stub -> orchestrator)
- crates/lang-rust/src/parser_items.rs (NEW)
- crates/lang-rust/src/parser_uses.rs (NEW)
- crates/lang-rust/src/lib.rs (two `mod` lines)
- crates/lang-rust/src/plugin.rs (the `analysis_fingerprint` override + caps
  literal flip + pin-test rename; nothing else in the file)
- tests/integration/tests/rust_language_semantic_index_test.rs (NEW suite)

You MUST NOT touch: `crates/index/**` (CURRENT_ANALYSIS_VERSION lives there,
frozen `2.1.0` - ZERO bytes), `crates/repo/**`, `crates/cli/**`,
`crates/pipeline/**`, `crates/lang/**`, `crates/lang-cpp/**`,
`crates/lang-scala/**`, `crates/lang-rust/src/{detector,shaper,boundary,
resolver,constraints,extractor,dependency_rewriter,validator}.rs` (other
hands' files), `tests/fixtures/**` (rust-basic is frozen from R-4's approval
onward - your suite READS it, writes nothing), m18/m21/m22/m23/ar27 suites,
`tools/**`, goldens. Cargo.lock: no new deps (syn/proc-macro2/unicode-ident
already lock-generated at R-3; if cargo rewrites it, revert and journal).

## What to build (shape sec. 3.7 analyzer row - the contract)

1. `analyzer.rs`: `analyze(&self, file_id, source, context) -> TuAnalysis`
   driving `syn::parse_file`; delegates item-walking to `parser_items.rs` and
   `use`-statement extraction to `parser_uses.rs` (the pre-split - keep every
   file < ~600, hard ceiling 1000).
2. `parser_items.rs`: items -> `SymbolRecord`s for fn/struct/enum/trait/impl/
   type/const/static/mod, qualified ids `crate::<path>::<Name>` (shape sec.
   3.7 literal); every record carries the EXACT span line/col via
   `proc_macro2` `source_file().line()`/`column()` discipline (1-based) -
   INV-7.
3. `parser_uses.rs`: `use` statements -> usage edges, kind mirroring Scala's
   Incomplete stance (path targets resolve later through the resolver at
   R-6 - you emit the RAW path facts + spans, nothing invented).
4. Macro honesty (D-2 caveat made behavior): a file with
   `macro_rules!`/`include!` parses what syn accepts; unexpanded macro
   content produces NO invented symbols (RISK: fabricating symbols is the
   violation - pin it with a fixture-driven test over
   `rust-basic/src/macro_specimen.rs` asserting the specimen's macro-generated
   module does NOT surface as a symbol).
5. Fingerprint: `fn analysis_fingerprint(&self) -> String { "rust-1".to_string() }`
   + in-src literal pin `analysis_fingerprint_is_rust_1` (message: bumps ride
   future parser changes; C++/Scala fingerprint literals are separate digest
   inputs, unedited - INV-8). NEVER touch `CURRENT_ANALYSIS_VERSION`
   (frozen 2.1.0 per M27).
6. Caps flip: `semantic_index: true` (plus `discovery: true` carried); all
   else false; pin renamed `capabilities_match_the_semantic_index_flip_literal`
   (full-struct byte-exact `assert_eq!`).
7. Suite `rust_language_semantic_index_test.rs` (m21 idiom, fixture-driven;
   milestone-named): symbol surface truth on `rust-basic` (named fixture
   items appear with their qualified ids); INV-7 SPAN TRUTH - for named
   fixture items assert EXACT expected line and column (1-based) literals,
   e.g. the `mod models;` declaration in `src/lib.rs` pinned to its literal
   line - this is the test that fails if `span-locations` ever vanishes;
   macro-caveat assertion (no fabricated symbol); warm/cold: a second analyze
   pass on unchanged bytes is cache-served (m21's witness idiom), and the
   C++/Scala cache digests do not move (witnessed by m21/m18/ar27 green
   UNEDITED at gate 5's census - your fingerprint change must not perturb
   their inputs: the fingerprint is per-plugin digest input).

## RGB protocol (true Red)
1. Red: author the suite + the span/fingerprint in-src pins against the
   pre-hand tree; expected failures: `analysis_fingerprint_is_rust_1` fails
   `left: "v1", right: "rust-1"`; the suite's symbol-surface assertion fails
   on an EMPTY record vec (the stub answers empty - it compiles, this is a
   real Red, not a compile failure); a span pin fails on missing/zero
   coordinates. Journal the failing list. (The caps-literal pin flips WITH
   the flip commit - it is an amendment-style byte-pin, never red-in-
   isolation.)
2. Green: items 1-7; re-run until green. NEVER weaken an assertion to pass.
3. Refactor: dedupe walkers, honest names; re-run.

## Hygiene (review-enforced)
The pre-split is the plan: `analyzer.rs` must stay under ~600 once the walker
grows; all three files < 1000 (suite ~350); functions < 100 (the visitor
methods decompose - R1 blocking); no banned names; pure ASCII.
`python tools/hygiene_check.py` exit 0.

## Frozen locks (commands that prove them)
- `git diff --name-only <start>..HEAD -- crates/index` EMPTY +
  `CURRENT_ANALYSIS_VERSION` reads `2.1.0` (read-check, journal the value +
  file).
- L2/L3: goldens empty-diff; `git diff --name-only <start>..HEAD -- tests/fixtures`
  EMPTY (fixture read-only this hop).
- L7/L8/L9 + L5/L6: lang-cpp/lang-scala/tools/m18-m23 paths absent.
- L11/L1: repo absent.
- L4: m00 green UNEDITED (census).
- L12: `python tools/encoding_guard.py --all` exit 0.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only)
1. `python tools/testrun.py --distill -p ariadne-lang-rust` - clean baseline
   journal BEFORE the red edits.
2. `python tools/testrun.py --distill -p ariadne-lang-rust` - Red (the in-src
   pins above) - capture the named failing tests.
3. `python tools/testrun.py --distill -p ariadne-lang-rust` - Green: exit 0,
   passed = baseline + your new in-src pins, attributed by name.
4. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   rust_language_semantic_index_test` - exit 0.
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m21_scala_semantic_index_test` - UNEDITED green (warm/cold + cache-digest
   witness).
6. `python tools/testrun.py --distill` - full census: failed 0, ignored 13,
   suites +1; every passed delta attributed; anchor from duty-1 ledger clause.
7. `python tools/hygiene_check.py` - exit 0.
8. Commit cycle: `python tools/encoding_guard.py --staged` (or `--all`) then
   `python tools/commit_scoped.py --scope crates/lang-rust --scope
   tests/integration/tests/rust_language_semantic_index_test.rs --scope
   tasks/rust-language-r5-journal.md -m "feat(lang-rust): syn analyzer with
   span truth, fingerprint rust-1, semantic_index flip (rust-language R-5)"`.
   Journal inside.

Declared gate budget: 7 scoped invocations + commit trio.

## Bench stance (reviewer's acceptance pass)
`python tools/distill.py -- python tools/bench_compare.py` - RUN fresh
(`crates/lang-rust/src` touched; kickoff sec. 4 fully-run stance; no baseline
or corpus change - D-3; WARNs recorded not attributed - PR-2).

## Discipline clauses
- UTF-8 Write/Edit-only rule set; pure ASCII (incl. generated-looking test
  fixtures built via byte escapes if ever needed - none expected).
- Touch-set hard; report, don't fix.
- Journal checkpoint per Green gate.
- STOP on context exhaustion (analyzer hands are the classic E1 class - the
  shape names it); successor runs `checkpoint_guard.py resume --spec
  tasks/rust-language-r5-handoff.md --scope crates/lang-rust --cmd ...`.
- Landing SHA relay: report + reflog; R-6's duty 1 cites it.
- F14 rules.

## Report back (final message)
1. Duty-1 + landing SHAs + `git show --stat`. 2. Red evidence (named failing
tests + canonical line). 3. Green evidence (gates 3-6 canonical lines +
capture path). 4. Deviations. 5. Out-of-scope observations. No raw output.

## Reviewer acceptance pass
- bench_compare (above); per-commit review vs touch-set; CURRENT_ANALYSIS_
  VERSION read-proof in the verdict; span pins check against the fixture's
  actual line numbering (reviewer re-reads the fixture); macro-fabrication
  audit (no symbol without written source); caps literal byte-exact
  discovery+semantic_index true only.

Reviewer-gate reference: `review-gate` skill. Extra MUSTs: the fingerprint
literal lands in the FIRST commit shipping an analyzer (this one) - a later
fingerprint hop is a deviation to reject; `analyzer.rs` alone (no parser_*
split) is acceptable ONLY under ~600 lines, otherwise R1-style blocker.

Sizing (planner, itemized): analyzer ~120, parser_items ~420, parser_uses
~180, plugin ~25/-6, lib ~2, suite ~330, journal ~12 - total ~1050-1250
produced lines; 7 gates + trio; one crate dir + one new suite file (declared
vertical: the suite is the INV-7 oracle the crate ships against; SM-4-style
hard-chain declaration - the delta outside the crate is one NEW file, zero
edits to existing tests). No split: stub-analyzer + suite-without-fingerprint
or fingerprint-without-analyzer are both half-states the sizing law forbids.
