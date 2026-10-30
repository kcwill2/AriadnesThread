# R-7 Handoff - rust-language: load_metric flip + the data-path proofs (manifest languages field, CLI vocabulary, UI node label - zero UI/report edits)

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). Hand 7
of 10, serial. The lightest hop: flips `load_metric` and lands the INV-14
proofs - "language is data": Rust flows through the existing manifest/UI/CLI
data paths with those crates' diffs EMPTY. Start rev: your dispatch head
(R-6's landing + verdict via duty 1).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/rust-language-shape.md` sec. 2 INV-5/INV-14, sec. 3.4 (schedule:
   this hop flips `load_metric`). Kickoff sec. 1 blast-radius sentence
   ("zero edits to ui/report/... production") + sec. 9 UI scope-out.
3. The data paths you PROVE, read-only: `collect_stage_languages`
   (additive `languages` manifest field path), `cycle_policy_by_file`
   (pipeline/src/lib.rs - already consumed at R-6), the bare-name `Debug`
   `format!("{:?}", ...)` node-language map idiom at
   `crates/cli/src/commands/ui/node_maps.rs` (READ-ONLY - the map prints the
   open-id name; `Rust` appears with ZERO edits, INV-14).
4. Model suite: `tests/integration/tests/m22_scala_load_analysis_test.rs`
   (load-metric idiom - zero-diff model).
5. Your duty-1 targets: `tasks/rust-language-r6-verdict.md` + ledger.

## Duty 1
Land R-6's verdict + ledger flip (`R-6 LANDED <SHA> APPROVED <date>, census
anchor <line> @ <SHA>, R-7 dispatched.`).

## Touch-set (hard)
- tasks/rust-language-r7-journal.md (create)
- tasks/rust-language-r6-verdict.md + tasks/milestone_ledger.md (duty 1 only)
- crates/lang-rust/src/plugin.rs (caps literal flip + pin rename ONLY)
- tests/integration/tests/rust_language_load_analysis_test.rs (NEW suite)

You MUST NOT touch: every other lang-rust file, `crates/ui/**`,
`crates/report/**`, `crates/artifact/**` (INV-14: ZERO bytes under those
three; your suite asserts their data answers), `crates/cli/**` (node_maps
read-only), fixtures (read-only), m18-m23 suites, `tools/**`, goldens,
Cargo.toml/lock (no new deps). No `git stash`/branch change.

## What to build
1. Caps flip: `discovery + semantic_index + dependency_graph + load_metric`
   true; `planning/extraction/rewriting/validation` false;
   `requires_toolchain: false`; pin renamed
   `capabilities_match_the_load_metric_flip_literal` (byte-exact).
2. Suite `rust_language_load_analysis_test.rs` (m22 idiom, fixture-driven,
   milestone-named): load metric over the `rust-basic` units (non-empty,
   finite, same shape class m22 asserts for Scala); INV-14 manifest proof: a
   Rust run's `artifacts/**/manifest.json` carries `"Rust"` in its additive
   `languages` field (via `collect_stage_languages`, existing path - no
   helper edit); CLI vocabulary print proof: the D26 vocabulary fields
   (`directive_label "Use"`, passive `used`, aggregation noun `mod
   aggregation`) surface through the graph/render data path on the same run
   (assert the returned models/manifest data, not a stdout scrape); UI node
   map data path: the node-language label derives from the hand-written bare
   name `Debug` - assert `format!("{:?}", language)` yields `Rust` for the
   run's language values AND pin the zero-edit fact by asserting the three
   named crates' diffs are empty (your own `git diff --name-only` in the
   journal).
3. Keep it honest: you flip a bit whose backing component (graph stage load
   computation) already exists and consumed the analyzer/resolver outputs
   from R-5/R-6 - that is why INV-5 permits this hop.

## RGB protocol (true Red)
1. Red: the suite authored against the pre-hand caps - expected failures:
   the load-metric stage answers an honest rejection / absent manifest field
   (`languages` missing or lacking `"Rust"`), and the caps-literal pin (if
   you run it first) fails `left: load_metric: false, right: true`. Journal
   the red list.
2. Green: items 1-2; re-run.
3. Refactor; re-run.

## Hygiene
Suite ~250 (< 1000); plugin delta ~10; functions < 100; banned-name clean;
pure ASCII; `hygiene_check.py` exit 0.

## Frozen locks (commands)
- INV-14 core proof: `git diff --name-only <start>..HEAD -- crates/ui
  crates/report crates/artifact` EMPTY (reviewer re-runs).
- L5/L6/L7/L8/L9: m18-m23 / lang-cpp / lang-scala / tools absent from any
  hunk; L2/L3 goldens+fixtures empty-diff; L4 m00 green UNEDITED; L1/L11
  repo absent; L12 encoding exit 0.

## Gates (distilling entry points only, 600000 ms)
1. `python tools/testrun.py --distill -p ariadne-lang-rust` - baseline.
2. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   rust_language_load_analysis_test` - Red.
3. same - Green.
4. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m22_scala_load_analysis_test` - UNEDITED green.
5. `python tools/distill.py -- python tools/ui_smoke.py` - expect
   `ui_smoke: PASS` (this hand asserts the UI data path; the smoke proves
   the zero-edit boot still serves routes + `no-store`).
6. `python tools/testrun.py --distill` - full census: failed 0, ignored 13,
   suites +1, deltas attributed.
7. `python tools/hygiene_check.py` - exit 0.
8. Commit cycle: `python tools/encoding_guard.py --staged` (or `--all`) then
   `python tools/commit_scoped.py --scope crates/lang-rust/src/plugin.rs
   --scope tests/integration/tests/rust_language_load_analysis_test.rs
   --scope tasks/rust-language-r7-journal.md -m "feat(lang-rust): load_metric
   flip + language-is-data manifest/vocabulary/UI-label proofs
   (rust-language R-7)"`.

Declared gate budget: 7 + commit trio.

## Bench stance (reviewer's acceptance pass)
`python tools/distill.py -- python tools/bench_compare.py` RUN fresh
(`crates/lang-rust/src` touched; kickoff sec. 4; WARNs recorded not
attributed).

## Discipline clauses
UTF-8 Write/Edit-only; touch-set hard; journal per Green gate; STOP +
`checkpoint_guard.py resume --spec tasks/rust-language-r7-handoff.md --scope
crates/lang-rust --cmd ...`; SHA relay + reflog (R-8's duty 1 cites); F14.

## Report back
Duty-1 + landing SHAs + `git show --stat`; red list; green canonical lines +
capture path; the three-crate EMPTY-diff proof quoted; deviations; notes.

## Reviewer acceptance pass
bench_compare; per-commit vs touch-set; the three-crate zero-diff re-run;
caps byte-check (exactly four true); m22 UNEDITED; `ui_smoke: PASS` in
verdict; census attribution.

Reviewer-gate reference: `review-gate` skill. Extra MUSTs: no stdout-scrape
assertion (data-model asserts only); `node_maps.rs` in no hunk.

Sizing (planner, itemized): plugin delta ~10, suite ~250, journal ~10 -
total ~270-330 produced lines; 7 gates + trio; one crate file + one suite
file (lightest declared vertical). No split.
