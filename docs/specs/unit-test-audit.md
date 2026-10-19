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
  drops serde and write errors, returns Ok), `commands/transform.rs:230-233`
  (`load_plan_for_action` erases the replan error), `commands/measure.rs:11` +
  `commands/discover.rs:87` (`discover().ok()` turns discovery failure into an
  *empty-file* fingerprint). Hand H7a fixed all three (root-cause Errs).
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
- `crates/worker-core` shard tmp-name race: DOWNGRADED by H5 - pid+counter
  tmp naming verified byte-identical since 0861db0 (pre-existing fix); lock
  shipped, no code change.
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
- H5 `uta-t5` scheduler + worker-core + index: fix UTA-B2 (+B6 surfacing);
  tmp-race row shipped as lock (downgraded); test
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
| UTA-B2 | index | serde-fail empty fingerprint | FIXED by H5 3e48c51 (fail-closed material) |
| UTA-B3 | lang-cpp | `#includemy` phantom include | NOT REPRODUCIBLE (H2 locks) |
| UTA-B4 | graph | find_file_id nondeterministic match | FIXED by H1 994e5c3 |
| UTA-B5 | graph | recursive DFS stack risk + SCC undercount | FIXED by H1 994e5c3 (iterative frames); fed-diamond undercount residual deferred |
| UTA-B6 | scheduler | empty-fp checkpoint + swallowed save/first-error | FIXED (surfacing) by H5 3e48c51; counts-to-CLI-report half deferred (cli struct-literal coupling) |
| UTA-B7 | cli | save/replan/discovery error swallows | FIXED by H7a 60b76f3 (root-cause Errs; files live under crates/cli/src/commands/) |
| UTA-B8 | cli | residual assumes committed-prefix | FIXED by H7a 60b76f3 (red 730 vs 640) |
| UTA-P* | various | pin-only adjudications above | PIN |

## Deferred findings (closeout ledger sync, 2026-10-19)

Fixes SHIPPED: UTA-B2, B4, B5 (iterative core), B6 (surfacing), B7, B8;
plus write-path containment (transform). FALSIFIED: UTA-B3, worker-core
tmp-race (pre-existing). DEFERRED with evidence:

- UTA-B1 include-provenance fix: needs origin field on `ExtractedSymbol`/
  `GeneratedUnit` + cli caller changes (tasks/uta-t4-journal.md BLOCKER).
- UTA-B5 residual: fed-diamond cycle undercount preserved (changes
  `CallGraphManifest.recursion_cycles` on real repos).
- UTA-B6 residual: checkpoint-count fields to CLI report (cli struct-literal
  coupling; scheduler+cli+pipeline D-shape hand).
- graph builder.rs:89-92: per-file index failure swallowed (`Err(_) =>
  continue`), all-fail repo indistinguishable from empty (diagnostic policy).
- repo discovery.rs:43-45: file-root/unreadable root walk yields empty Ok -
  upstream class of the empty-fingerprint bug (fix here would neutralize the
  class H7a pinned downstream).
- cli progress overshoot unclamped (>100%, oversized bar) - pinned
  `pin_progress_overshoot_unclamped`.
- cli `parse_language_arg` case-strict; legacy test name overclaims
  "case_insensitively" (rename owed).
- cli dispatch: subrepo silently wins over positional in analyze;
  includes/plan drop the positional via an uncomparing wildcard arm.
- scheduler: `memory_limit_bytes` dead knob; index `IncrementalCache` dead
  public API (deletion = API decision).
- lang-cpp: `.H`/`.CPP` case divergence across detector/planning/shaper;
  backend missing-source analyzes empty; `#else` of disabled `#if 0`
  double-masked; ParserCtx self-edge; `A a();` phantom double-record (all
  pinned with owner names).
- lang-scala: scanner stops unterminated strings at newline while
  directive_parser runs to EOF (docs corrected; unification owed);
  two-deps-one-statement emits duplicate new imports; truncated `import
  {` absorbs identifiers to EOF (tolerance policy owed).
- cluster frontier: breaks at first frozen (nearer-collateral optimizable
  missed); ownership arg ignored; planner.rs:477 per-candidate SCC rebuild
  (perf); second digit-grouping impl in plan.rs (divergence pinned).
- artifact: manifest without root-key skips root check; corrupt .bin silently
  falls back to older .json; rename-fail leaves staged tmp; verbatim-root
  breaks separator-cosmetic same_path equality; seq-side bounded decode
  rejects via element-EOF, not the limit path (blob design owed).
- build `normalize` dedup drops a whole duplicate-named context.
- graph: `include_load == total_load` degeneracy; parallel-directive degree
  inflation; call edges with caller-less nodes (pinned).
- closeout rider shipped: bench_compare phase label matched to the M23.5
  `detect_cycle_groups` producer rename (pre-existing gate drift).

## Round 2 (added 2026-10-19)

Angles round 1 couldn't reach: the ui crate (its round-1 audit died), the
Python tooling suite, the integration-seam layer, and below-cutoff residuals.
Four read-only audits returned; suspects verified by orchestrator code-read
where marked ORCH.

New confirmed bugs (round-2 tag `UTB-`):
- UTB-1 (ORCH) ui `api_graph.rs:811-830` vs `:749-756` -
  `build_subtree_edges_from_closure` iterates the FULL closure while nodes
  exclude the traversal dep: emitted `actual_subtree` edges can reference a
  node absent from their own node list (orphan edges for cytoscape). Hand R1.
- UTB-2 (ORCH) ui `api_graph.rs:754,817` - subtree nodes/edges built by
  HashSet iteration, unsorted: JSON array order varies per process - the
  UTA-B4 nondeterminism class recurring ui-side, uncatchable today because
  the endpoint has zero tests. Hand R1 fixes (sort by FileId).
- UTB-3 (ORCH) ui `api_source.rs:72-81` - `/api/repo-info` `file_paths`
  documented "relative paths" returns numeric FileId strings
  (`id.0.to_string()`), defeating the endpoint's stated cross-environment
  purpose; zero tests existed. Hand R1 fixes (map keys() -> values()).
- UTB-4 ui `api_graph.rs:273-289` - `top_hubs` degree-rank order from
  `get_entrypoints` silently replaced by payload emission order (in-code
  comment admits the skipped re-sort). Hand R1: fix iff JS consumers allow,
  else loud pin + report.
- UTB-5 tools `bench_compare.py:519` - non-numeric baseline phase value ->
  uncaught ValueError traceback where module contract promises exit 2
  internal error. Hand R2 fixes (validate -> exit 2).
- UTB-6 tools `theme_token_check.py:143,160` + `:211` except-tuple -
  non-dict baseline JSON -> uncaught AttributeError, contract broken. Hand
  R2 fixes.
- UTB-7 tools NaN-timeout holes - `ui_smoke.parse_arguments:101-110` +
  `poll_until_ready:246` (NaN deadline never reached -> infinite hang) and
  `bench_compare._positive_float:556-563` (NaN/inf accepted). Hand R2
  fixes (reject nan/inf).
- UTB-8 (ORCH) repo `discovery.rs:40-63` - stack walk follows symlinked
  directories with no visited set: a directory loop is an INFINITE walk
  (hang). Hand R3 fixes (canonical-path visited set) + platform-conditional
  loop test (junction on Windows, symlink on unix, graceful skip when the
  environment forbids creation, per T7a share_mode precedent).

Pin-and-report class (behavior current, wrong-ish, or owner-decision):
worker-core `tu_store.rs:295-305` silent drop of Resolved/Wildcard ids
missing from `path_by_id` (R4 pin + surface-count report); lang-api
`catalog.rs:98-101` by_basename first-insert-wins (R3 pin); `is_toolchain`
dead arm (R3 report); repo walk error-polarity contradiction at :43-45 vs
:49/:75 (deferred sibling, R3 pins both sides + journal proposal);
entrypoints rank-order fallback covered under UTB-4.

Round-2 hands (same gated pipeline, sequential):
- R1 `uta2-r1` ui: fixes UTB-1/2/3 (+UTB-4 decision) + traversal endpoint /
  repo-info / symbol-call-arm / etag-refresh tests (audit rows 1-13).
  Gate: `-p ariadne-ui`.
- R2 `uta2-r2` tools: fixes UTB-5/6/7 + package_offline fake-subprocess
  suite + error-path pins (rows 5-20 selectively). Gates: unittest
  discover, no cargo.
- R3 `uta2-r3` repo + lang-api: fix UTB-8; pin by_basename/by_module_path_
  suffix/by_path; policy keyword + FileKind extension matrices; locate Err
  text; walk-polarity characterization pins. Gates: `-p ariadne-repo`,
  `-p ariadne-lang-api`.
- R4 `uta2-r4` sweep-B remainder: worker-core drop pin; scheduler
  incremental IO; build candidate roots/malformed-JSON/measure-fallback;
  cluster format_summary; artifact checked-variants; transform is_noop
  completion; lang render_unit; WorkerBackend spy; validate observations.
  Gates: one distill per touched crate (<=10).
- R5 `uta2-r5` sweep-A remainder: graph `map_aggregation_consumers`
  (zero-coverage function); lang-cpp raw_string_open multibyte + probe
  Available arm; lang-scala boundary.rs marker pin; cli transform action-
  arm verification. Gates: `-p ariadne-graph`, `-p ariadne-lang-cpp`,
  `-p ariadne-lang-scala`, `-p ariadne-cli`.
