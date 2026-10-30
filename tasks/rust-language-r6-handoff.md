# R-6 Handoff - rust-language: the resolver (mod tree + use paths), dependency_graph flip, graph-shape truth

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). Hand 6
of 10, serial. Replaces the resolver stub with the real `mod x;`/`use`-path
resolver, flips `dependency_graph`, and lands the graph-shape suite including
the RISK-CYCLE report-data assertion. Start rev: your dispatch head (R-5's
landing + verdict via duty 1).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/rust-language-shape.md` sec. 2 INV-5/INV-8, sec. 3.3 cycle_policy
   row, sec. 3.7 resolver row (the contract), sec. 6 RISK-CYCLE (oracle:
   existing SCC-sharing lock UNEDITED + report-classification assertion).
   Kickoff L5/L6; "Not this milestone" scope-out (cross-crate path-dependency
   edges stay external).
3. Model files: `crates/lang-scala/src/resolver.rs` (ResolutionContext
   consumption + external-boundary stance) and
   `tests/integration/tests/m20_scala_dependency_graph_test.rs` (suite idiom -
   zero-diff model).
4. The neutral seam: `crates/lang` `DependencyResolver` trait + `RawDepEdge`
   (line/col fields carried) + `ResolutionContext` (via the Scala resolver's
   imports).
5. Your duty-1 targets: `tasks/rust-language-r5-verdict.md` + ledger.

## Duty 1
Land R-5's verdict + ledger flip (`R-5 LANDED <SHA> APPROVED <date>, census
anchor <line> @ <SHA>, R-6 dispatched.`), same commit shape as before.

## Touch-set (hard)
- tasks/rust-language-r6-journal.md (create)
- tasks/rust-language-r5-verdict.md + tasks/milestone_ledger.md (duty 1 only)
- crates/lang-rust/src/resolver.rs (stub -> real)
- crates/lang-rust/src/plugin.rs (caps literal flip + pin rename ONLY)
- tests/integration/tests/rust_language_dependency_graph_test.rs (NEW suite)

You MUST NOT touch: every other lang-rust file, `crates/graph/**`,
`crates/report/**` (RISK-CYCLE consumes `cycle_policy_by_file` -
`crates/pipeline/src/lib.rs:672-687` lookup, existing data path, ZERO core
edits), `crates/{index,cluster,transform,validate,artifact,scheduler,ui}/**`,
fixtures (read-only), m18-m23 suites, `tools/**`, goldens, Cargo.toml/lock
(no new deps; a cargo rewrite must be reverted). No `git stash`/branch change.

## What to build (shape sec. 3.7 resolver row - the contract)
1. `resolve_edges`: `mod x;` resolves to the child file - `x.rs` or
   `x/mod.rs` (both fixture forms exist post-R-4); `use` paths resolve to
   repo-discovered targets through `ResolutionContext`; external/registry
   paths fall to the external-boundary markers (R-3's `["target", "vendor",
   ".cargo"]`); out-of-repo `::serde::...` resolves to NOTHING (mirrors C++
   system-header / Scala-lib stance - inventing an edge is the violation);
   every `RawDepEdge` carries line/col (INV-7 spans honored end to end).
   Cross-crate path-dependency edges beyond the repo's discovered file set:
   NOT this milestone (kickoff sec. 9) - external stance, pinned by a test.
2. Caps flip: `discovery + semantic_index + dependency_graph` true, all else
   false, `requires_toolchain: false`; pin renamed
   `capabilities_match_the_dependency_graph_flip_literal` (byte-exact).
3. Suite `rust_language_dependency_graph_test.rs` (m20 idiom, fixture-driven,
   milestone-named): mod-tree edges both layouts (`src/models/mod.rs` sibling
   form and `src/greeting.rs`); `use`-edges over fixture paths; `vendor/`
   external-boundary assertion (marker, no unit edges); out-of-repo path
   resolves to nothing; RISK-CYCLE report-data assertion: a fixture with a
   legal intra-crate `use` cycle keeps the planner stance informational and
   the report-side consumes `TolerateWithinPackage` through the existing
   `cycle_policy_by_file` data path (assert manifest/report-data field; zero
   core edit, zero UI edit); edge spans carry the analyzer's line/col.

## RGB protocol (true Red)
1. Red: suite authored against the stub resolver - expected failure: the
   edge-vector asserts fail on EMPTY vecs (`left: []`, right: the named edge
   set); journal the red list.
2. Green: items 1-3; re-run.
3. Refactor; re-run.

## Hygiene
resolver.rs ~400 (< 1000, Scala's is 422); suite ~280; functions < 100;
banned-name clean; pure ASCII; `hygiene_check.py` exit 0.

## Frozen locks
- L5/L6: m20 (and all m18-m23) paths in NO hunk (m20 is the read-only model).
- L7/L8/L9: lang-cpp/lang-scala/tools absent; graph/report/validate/artifact
  absent (neutrality census-witnessed).
- L2/L3: goldens + fixtures empty-diff; L4 m00 green UNEDITED; L1 repo absent.
- L12: `encoding_guard --all` exit 0.

## Gates (distilling entry points only, 600000 ms)
1. `python tools/testrun.py --distill -p ariadne-lang-rust` - baseline.
2. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   rust_language_dependency_graph_test` - Red (empty-edge failures).
3. same command - Green.
4. `python tools/testrun.py --distill -p ariadne-lang-rust` - Green + caps
   pin attributed.
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m20_scala_dependency_graph_test` - UNEDITED green.
6. `python tools/testrun.py --distill` - full census: failed 0, ignored 13,
   suites +1, deltas attributed; anchor from duty-1 clause.
7. `python tools/hygiene_check.py` - exit 0.
8. Commit cycle: `python tools/encoding_guard.py --staged` (or `--all`) then
   `python tools/commit_scoped.py --scope crates/lang-rust --scope
   tests/integration/tests/rust_language_dependency_graph_test.rs --scope
   tasks/rust-language-r6-journal.md -m "feat(lang-rust): mod-tree and
   use-path resolver, dependency_graph flip (rust-language R-6)"`.

Declared gate budget: 7 + commit trio.

## Bench stance (reviewer's acceptance pass)
`python tools/distill.py -- python tools/bench_compare.py` RUN fresh
(`crates/lang-rust/src` touched; kickoff sec. 4; WARNs not attributed).

## Discipline clauses
UTF-8/Write-Edit-only; touch-set hard; journal per Green gate; STOP +
`checkpoint_guard.py resume --spec tasks/rust-language-r6-handoff.md --scope
crates/lang-rust --cmd ...` on exhaustion; landing SHA + reflog relay (R-7's
duty 1 cites); F14.

## Report back
Duty-1 + landing SHAs + `git show --stat`; red list; green canonical lines
(gates 3-6) + capture path; deviations; out-of-scope notes. No raw output.

## Reviewer acceptance pass
bench_compare; per-commit vs touch-set; the invented-edge audit (every
asserted edge traced to written source; external stance pinned); caps
literal byte-check (exactly three true); m20 UNEDITED; census attribution.

Reviewer-gate reference: `review-gate` skill. Extra MUSTs: no
`crates/pipeline/src/lib.rs` hunk (the `cycle_policy_by_file` lookup is
consumed, not edited); fixture bytes unchanged.

Sizing (planner, itemized): resolver ~400 (stub 55 replaced), plugin ~12,
suite ~280, journal ~10 - total ~700-900 produced lines; 7 gates + trio; one
crate dir + one new suite file (declared vertical, same SM-4-style
declaration). No split: resolver and its graph suite are one capability hop.
