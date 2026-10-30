# R-9 Handoff - rust-language: planning hooks + probe-only validator + the final all-true capability flip + the symbol-isolation suite

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). Hand 9
of 10, serial. Shape sec. 5's 5b-half: the last two components (hooks,
validator) land, ALL EIGHT capability bits flip true behind existing
components, and the plan/preview/apply/rollback suite (Scala m23-shape
mirror) proves the whole parity surface dispatches. Start rev: your dispatch
head (R-8's landing + verdict via duty 1).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/rust-language-shape.md` sec. 2 INV-5 (final literal - byte-exact),
   INV-10 (probe validator a/b/c proofs), sec. 3.5 hooks + validator rows
   (VERBATIM contract), sec. 3.6 (why the three `CyclePolicy::Reject`
   literals do NOT change), sec. 6 RISK-MODTREE's grown-oracle assertion.
   Kickoff L5 (m23 zero-diff), A3/D-2 (probe-only).
3. Model files: `crates/lang-scala/src/planning.rs` (hook pair + sanitize
   discipline) and `crates/lang-scala/src/validator.rs` (CommandRunner
   injection seam + probe tests - mirror its file 1-for-1 so probe tests need
   no real spawns), `tests/integration/tests/m23_symbol_isolation_test.rs`
   (suite idiom - zero-diff model), `crates/cli/src/registry.rs` `planning_hooks`
   dispatch (:276 - registry-dispatched, your overrides enroll with ZERO CLI
   edit - AR-19 read-verified).
4. The toolchain-probe seam facts (INV-10): `TOOLCHAIN_PROBE_ORDER ==
   ["cargo", "rustc"]` (mission-wording order); `validate_compile` uses
   `rustc` specifically, options-before-sources, with the single-input
   honesty doc (rustc takes one main input; multi-file out of scope;
   TierValidator never calls it while offline - D-M23-12 class); Tier 3
   `Skipped` is `crates/validate`'s existing neutral behavior - no edit.
5. Your duty-1 targets: `tasks/rust-language-r8-verdict.md` + ledger.

## Duty 1
Land R-8's verdict + ledger flip (`R-8 LANDED <SHA> APPROVED <date>, census
anchor <line> @ <SHA>, R-9 dispatched.`).

## Touch-set (hard)
- tasks/rust-language-r9-journal.md (create)
- tasks/rust-language-r8-verdict.md + tasks/milestone_ledger.md (duty 1 only)
- crates/lang-rust/src/planning.rs (NEW: hook impls + sanitize helpers)
- crates/lang-rust/src/validator.rs (stub -> probe-only real, injection seam)
- crates/lang-rust/src/plugin.rs (the hook-method wiring consuming
  `planning.rs` + the FINAL caps literal + pin rename; the validator
  accessor wiring if its type changed)
- crates/lang-rust/src/lib.rs (one `mod` line)
- tests/integration/tests/rust_language_symbol_isolation_test.rs (NEW suite)

You MUST NOT touch: `crates/cli/**` (registration into planning is the trait
override, not a CLI edit - shape sec. 3.5 says it plainly), the three
`CyclePolicy::Reject` literals (`commands/plan.rs:203`, `optimize.rs:305`,
`commands/ui.rs:419` - ZERO hunk), `crates/cluster/**` (planner - neutral),
`crates/validate/**` (tier-3 Skipped behavior exists), every other lang-rust
file, fixtures (read-only), m18-m23 suites (m23 is your zero-diff model),
`tools/**`, goldens, Cargo.toml/lock (no new deps). No `git stash`/branch
change.

## What to build (shape sec. 3.5 - VERBATIM contract)

1. `planning.rs` (the ADR-0002 Decision 3 pair; plugin override enrolls,
   registry dispatch answers):
   - `aggregation_filter(repo_root, path) -> Option<bool>`: `Some(true)` iff
     the file content parses (syn) AND declares >= 1 NON-INLINE `mod x;`
     item (disk-level aggregation edge); inline `mod x { .. }` blocks are NOT
     aggregation children; unreadable/unparseable -> `Some(false)` (the
     filter invents nothing - Scala parity).
   - `target_path_for(path, symbol) -> Option<PathBuf>`: `Some` the
     `{sanitized_symbol}.rs` beside the aggregation file; sanitize discipline
     mirrors `scala_target_path_for` (read it; mirror, don't invent).
2. `validator.rs`: `TOOLCHAIN_PROBE_ORDER == ["cargo", "rustc"]`;
   `toolchain_status()` -> `Available(first banner)` | `Missing` via the
   injected `CommandRunner` seam mirroring the Scala file 1-for-1;
   `validate_compile` per the seam facts in binding input 4 (offline error
   names the deferral; spawns nothing beyond the probe walk - INV-10c).
   INV-10's three scripted-runner proofs become in-src tests: (a) no usable
   probe -> `Missing`; (b) EVERY spawn through the injected runner is
   `(probe-order entry, ["--version"])`; (c) offline `validate_compile`
   errors naming the deferral, spawning nothing beyond the probe walk.
3. FINAL caps literal (INV-5, byte-exact, D-M23-6-style pin renamed
   `capabilities_match_the_parity_flip_literal`):
   `LanguageCapabilities { discovery: true, dependency_graph: true,
   semantic_index: true, load_metric: true, planning: true, extraction:
   true, rewriting: true, validation: true, requires_toolchain: false }`
   - all eight true BEHIND EXISTING components (analyzer R-5, resolver R-6,
   load data R-7, extractor+rewriter R-8, hooks+validator this hop);
   `requires_toolchain: false` at every hop including this one (A3/D-2).
   Also flip the hook methods ON in `plugin.rs` (the `aggregation_filter` /
   `target_path_for` overrides delegate to `planning.rs`).
4. Suite `rust_language_symbol_isolation_test.rs` (m23 idiom, fixture-driven,
   milestone-named): plan (aggregation candidates = the `mod`-declaring files
   via your hook), preview (dry-run set INCLUDING the parent file as MODIFIED
   - the grown-oracle / RISK-MODTREE assertion: parent-file modification
   bytes present, `mod <extracted>;` line in the previewed parent), apply
   (parent gains the declaration ONCE - re-apply idempotence; consumer `use`
   paths rewritten per spec), rollback (bytes back; re-plan converges to the
   pre-apply plan - the m23 convergence discipline), tier-3 offline stance:
   validation tier reports `Skipped` (probe-only, INV-10 class), NEVER a
   spawned compile beyond `--version` probes.

## RGB protocol (true Red)
1. Red: suite + the parity caps pin authored pre-implementation: the pin
   fails `left: planning: false, right: true`; the suite's plan-stage
   aggregation-candidate assert fails (hook default None -> honestly not an
   aggregation unit); validator probe-walk tests fail on the stub; journal
   the red list.
2. Green: items 1-4; re-run.
3. Refactor; re-run.

## Hygiene
planning ~250, validator ~200, suite ~550 (Scala's 831 shrinks honestly -
no extends channel); all < 1000; functions < 100; banned-name clean; pure
ASCII; `hygiene_check.py` exit 0.

## Frozen locks
- L5: m23 (`m23_symbol_isolation_test.rs`) in NO hunk; your suite is the
  Rust mirror, not an edit.
- m18-m22 + m24 absent; cli in no hunk; cluster/validate absent; L7/L8/L9
  plugin/tool zero-diff; L2/L3 goldens+fixtures empty-diff; L4 m00 green
  UNEDITED (census); L1 repo absent; L12 encoding exit 0.
- Caps monotonicity witness: the diff of plugin.rs shows the literal gained
  EXACTLY four bits (planning, extraction, rewriting, validation - the first
  three had components shipped at R-8, `load_metric` flipped at R-7 and must
  NOT re-appear in this hunk) - reviewer counts bit-lines in the hunk; a bit
  true without a shipped component behind it = blocker (INV-5).

## Gates (distilling entry points only, 600000 ms)
1. `python tools/testrun.py --distill -p ariadne-lang-rust` - baseline.
2. same - Red (named failures incl. the parity pin).
3. same - Green (hooks + validator + pin attributed by name).
4. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   rust_language_symbol_isolation_test` - Red then Green (journal both).
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m23_symbol_isolation_test` - UNEDITED green (Scala isolation zero-diff).
6. `python tools/testrun.py --distill -p ariadne-cli` - UNEDITED green
   (dispatch consumed, not edited).
7. `python tools/testrun.py --distill` - full census: failed 0, ignored 13,
   suites +1, deltas attributed.
8. `python tools/hygiene_check.py` - exit 0.
9. Commit cycle: `python tools/encoding_guard.py --staged` (or `--all`) then
   `python tools/commit_scoped.py --scope crates/lang-rust --scope
   tests/integration/tests/rust_language_symbol_isolation_test.rs --scope
   tasks/rust-language-r9-journal.md -m "feat(lang-rust): planning hooks,
   probe-only validator, final all-true capabilities, isolation suite
   (rust-language R-9)"`.

Declared gate budget: 8 + commit trio.

## Bench stance (reviewer's acceptance pass)
`python tools/distill.py -- python tools/bench_compare.py` RUN fresh
(`crates/lang-rust/src` touched; kickoff sec. 4; WARNs not attributed; no
baseline/corpus change - D-3).

## Discipline clauses
UTF-8 Write/Edit-only; touch-set hard; journal per Green gate; RISK-MODTREE:
if R-8's mechanism turns out to mis-interact with apply at the suite level,
STOP per the shape's clause (journal inventory, return; planner amendment,
never silent narrowing); STOP + `checkpoint_guard.py resume --spec
tasks/rust-language-r9-handoff.md --scope crates/lang-rust --cmd ...` on
exhaustion; SHA relay (R-10's duty 1 cites); F14.

## Report back
Duty-1 + landing SHAs + `git show --stat`; red list (both red stages); green
canonical lines + capture path; INV-10 a/b/c proof citation; deviations;
out-of-scope notes.

## Reviewer acceptance pass
bench_compare; per-commit vs touch-set; the caps hunk bit-count audit
(exactly four bits gained, all-true + toolchain-false final); INV-10 a/b/c
tests green UNEDITED form audit; m23 zero-diff; cli zero-diff; grown-oracle
parent-modified assert present and meaningful (not vacuously true - reviewer
checks the assertion names the parent path AND the inserted line); census
attribution.

Reviewer-gate reference: `review-gate` skill. Extra MUSTs: every spawn in
probe tests is `--version`-shaped (INV-10b) - a test spawning `rustc
--crate-name` etc. is a blocker; the `Skipped` tier mapping consumed from
`crates/validate`, not reimplemented.

Sizing (planner, itemized): planning ~250, validator ~200, plugin wiring +
literal ~40, suite ~550, journal ~12 - total ~1000-1200 produced lines; 8
gates + trio; one crate dir + one new suite file (declared vertical, SM-4
pattern). No split: hooks + validator + flip + acceptance are the shape's
pre-authorized 5b unit; further cuts land a false all-true literal or an
unaccepted flip.
