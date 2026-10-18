# Unit-Test Audit and Gap-Fill - Spec

Change tag: `uta` (unit-test-audit). Started 2026-10-18. Not a numbered
milestone; hands are gated individually via the `review-gate` skill.

## Method

Seven read-only gap audits (one per crate cluster) inventoried every
non-trivial function in `crates/*/src` against its unit tests. Baseline at
audit time: HEAD 47b6211, clean tree, suite 1521/0/13/112 (per
`tasks/milestone_ledger.md` closeout row; re-verified at final gate).

Test-density census (tests per Kloc of crate src): worker-core 10.0,
scheduler 8.7, build 12.7, artifact 12.9 were the weakest; ui 103.4 was the
strongest. The audits found the exposure is *bimodal*: crates have strong
happy-path suites but thin or zero coverage on error/edge branches, plus a
crop of silent-error-swallow sites.

## Confirmed bugs (code read by orchestrator, pinned below)

- UTA-B1 `crates/transform/src/placement.rs:107-116` - CONFIRMED.
  `compute_rewrite` marks *every* `old_includes` entry covered whenever *any*
  referenced symbol maps to a generated header; the comment says "find which
  old_include this symbol came from" but no provenance is checked. A consumer
  referencing symbols from one of two old includes silently loses the other.
  Hand H4 CONFIRMED reachability upstream is missing: the origin datum
  `ProposedExtraction.source_file` is dropped at engine/mod.rs before the
  rewriter, so the fix needs an origin field + cli caller changes - shipped
  as journal BLOCKER with the data-shape proposal (deferred, reviewer-
  verified 8ef784a).
- UTA-B2 `crates/index/src/fingerprint.rs:22` - CONFIRMED (AR-05 class).
  `contexts_material` returns `""` via `unwrap_or_default()` on serde failure
  (reachable: non-UTF-8 paths on Windows), making `contexts_fingerprint`
  run-invariant - stale-after-change detection silently dies. Hand H5 fixes
  it mirroring `cache.rs:61-111`.
- UTA-B3 `crates/lang-cpp/src/directive_parser.rs:24` - DOWNGRADED to NOT
  REPRODUCIBLE by hand H2's red discipline: the missing word boundary is real,
  but line 28 requires the remainder after `include` to start with `"`/`<`,
  so `#includemy "a.h"` can never produce a directive. Specimens ship as
  regression locks; no code change.
- UTA-B4 `crates/graph/src/graph.rs:135-155` - CONFIRMED.
  `find_file_id` returns the *first* HashMap-iteration match, so (a) two
  files sharing a basename resolve nondeterministically per process and
  (b) a full-path match can lose to an earlier filename match. Hand H1 fixes
  it deterministically (full-path match preferred; tie broken by FileId).
- UTA-B5 `crates/graph/src/callgraph.rs:205-231` - CONFIRMED policy
  violation. `detect_recursion_cycles` is recursive DFS against the crate's
  explicit no-recursion policy (`closures.rs:27`, IV-7 100k-chain lock);
  deep call chains stack-overflow. Hand H1 converts it to the frame-machine
  pattern and tests multi-SCC cycle enumeration.
- UTA-B6 `crates/scheduler/src/engine.rs:268-275` +
  `crates/scheduler/src/pool.rs:82-113` - CONFIRMED swallows: foreign
  `tu_file_id` checkpoint written with an empty fingerprint via
  `unwrap_or_default()`, every `save_shard` IO failure dropped via
  `let _ =`, pool keeps only the first error and queue-restore is unasserted.
  Hand H5 tests + surfaces.
- UTA-B7 cli silent swallows: `optimize.rs:471-473` (`save_optimization_report`
  drops serde and write errors, returns Ok), `transform.rs:230-233`
  (`load_plan_for_action` erases the replan error), `measure.rs:11` +
  `discover.rs:87` (`discover().ok()` turns discovery failure into an
  *empty-file* fingerprint). Hand H7 pins/fixes.
- UTA-B8 `crates/optimize prefix assumption`
  `crates/cli/src/optimize.rs:457-459` - `residual_collateral_load` assumes
  committed plans are a prefix of `safe_plans` (`take(committed)`), but the
  commit loop accepts plans individually, so a middle-failure skews the
  residual. Hand H7 fixes with a red test.

## Adjudicated-as-pin-only (semantics undecided; pin current, report)

These are real smells but changing behavior needs an owner decision; hands
pin current behavior with a named test and list them as findings:

- `crates/cluster/src/frontier.rs:71-75` break-at-first-non-optimizable;
  `frontier.rs:20` `_file_ownerships` ignored (H6).
- `crates/artifact/src/manager.rs:170` root check skipped when manifest key
  absent; `apply_io.rs:94-109` silent stale-JSON fallback (H6).
- `crates/build/src/types.rs:97` `dedup_by(name)` drops whole context (H6).
- `crates/lang-cpp/src/backend.rs:189` missing source analyzed as empty Ok
  (H2 pins with a loud name).
- lang-cpp case divergence `.H`/`.CPP` across detector/planning/shaper
  (B3-C): pin the full matrix, report for owner decision (H2).
- lang-scala scanner-vs-directive_parser unterminated-string disagreement +
  doc claims EOF while scanner stops at newline (H3): fix the *docs* to
  reality, pin both scanners' real behavior, report the divergence.
- `crates/graph/src/load.rs:548-550` `include_load == total_load` degeneracy
  (H1 pins), `get_entrypoints` parallel-edge degree inflation (H1 pins).
- `crates/transform/src/engine/write.rs:11-52` absolute/`..` unit.path can
  escape output dir (H4: add containment rejection - this one is a fix,
  silent out-of-dir writes are not defensible).
- `crates/worker-core` shard tmp-name race (H5: include pid/millesecond in
  tmp name - fix).
- `crates/cluster/src/planner.rs:477` per-candidate SCC rebuild (perf,
  H6: report only).
- `crates/graph/src/builder.rs:89-92` `Err(_) => continue` on unreadable
  file (H1: report only; needs a diagnostic-policy decision).
- scheduler `memory_limit_bytes` dead knob, `IncrementalCache` dead public
  API (H5: report only; deletion is an API decision).

## Known-open, explicitly out of scope

- `ui.traversal_view_degree_setting_bug` (JS traversal view ignores Degree
  setting): prior fix attempts failed; existing test
  `ui_test/asset_injection_hardening.rs:252` locks it as byte-untouched.
  Untouched by this change.
- `crates/report` is a 7-line re-export shim; its gaps are artifact's (H6).

## Hands, touch-sets, gate budget

All hands: scoped gates only (`python tools/testrun.py --distill -- -p
<crate>` per touched crate, `hygiene_check.py`, `encoding_guard.py
--staged`, scoped commit); never bare cargo. Bug fixes follow RGB (red pin
test first). Existing tests are contracts: if a fix requires editing an
existing test that locks the bug, the edit needs a journal justification and
the reviewer adjudicates. Report-only anything outside the touch-set.

- H1 `uta-t1` graph: fix UTA-B4, UTA-B5; add traversal/callgraph/entrypoint/
  report-branch tests (audit graph rows 1-15). Touch: crates/graph.
- H2 `uta-t2` lang-cpp: fix UTA-B3; add parser/locator/tokenizer/backend
  tests (rows 1-12, 15, 16-20); pin B3-C matrix + backend empty-Ok (rows
  11, 15). Touch: crates/lang-cpp.
- H3 `uta-t3` lang + lang-api + lang-scala: add lexer/rewrite/removal pins
  (scala rows 1-12, lang rows 13/22/23, api row 14); fix the scanner *doc*
  claims only. Touch: crates/lang, crates/lang-api, crates/lang-scala.
- H4 `uta-t4` transform + validate: fix UTA-B1 + write-path containment; add
  tier-validator, move-removal, rollback-mismatch, cycle-pinning tests.
  Touch: crates/transform, crates/validate.
- H5 `uta-t5` scheduler + worker-core + index: fix UTA-B2, tmp-race; test
  UTA-B6 paths, pool first-error, TuTask::cmp, format boundaries, shard
  clamps, tu_store corrupt/concurrency. Touch: crates/scheduler,
  crates/worker-core, crates/index.
- H6 `uta-t6` cluster + artifact + build + pipeline: add bounded-decode
  reject, manifest branches, same_path table, frontier 5-shape, merge-order,
  normalize-dedup, corrupt-reuse, incremental-bail tests; pin the
  adjudicated set. Touch: crates/cluster, crates/artifact, crates/build,
  crates/pipeline.
- H7a `uta-t7a` cli: fix UTA-B7, UTA-B8 (optimize/transform/measure/discover
  files only). Touch: those four src files.
- H7b `uta-t7b` cli (after H7a lands): add progress bounds, dispatch
  precedence, build_file_collateral, case-matrix, query-render pins; strengthen
  the two assert-nothing tests. Touch: rest of crates/cli.

Gate count per hand stays <= ~10 scoped invocations (testrun distill is the
only per-crate cost; multi-crate hands pay one extra each - justified here
because scope is test-additive and per-crate gate cost is one distill run).

## Bug ledger (updated as hands land)

| id | crate | claim | status |
|----|-------|-------|--------|
| UTA-B1 | transform | covered_old_includes drops unmapped includes | CONFIRMED; H4 BLOCKER - fix needs cross-crate data-shape change (origin field on ExtractedSymbol/GeneratedUnit + cli callers); reviewer-verified blocker, deferred with journal analysis |
| UTA-B2 | index | serde-fail empty fingerprint | CONFIRMED -> H5 fix |
| UTA-B3 | lang-cpp | `#includemy` phantom include | NOT REPRODUCIBLE (H2 locks) |
| UTA-B4 | graph | find_file_id nondeterministic match | CONFIRMED -> H1 fix |
| UTA-B5 | graph | recursive DFS stack risk + SCC undercount | CONFIRMED -> H1 fix |
| UTA-B6 | scheduler | empty-fp checkpoint + swallowed save/first-error | CONFIRMED -> H5 |
| UTA-B7 | cli | save/replan/discovery error swallows | CONFIRMED -> H7 |
| UTA-B8 | cli | residual assumes committed-prefix | CONFIRMED -> H7 fix |
| UTA-P* | various | pin-only adjudications above | PIN |
