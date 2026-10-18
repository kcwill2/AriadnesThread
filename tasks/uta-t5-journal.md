# UTA-T5 Journal - scheduler + worker-core + index (UTA-B2, UTA-B6, boundary pins)

Handoff: tasks/uta-t5-handoff.md. Base: main at 7c9805d (after UTA-T4 + docs).
Crate order: index, worker-core, scheduler.

## Status legend
- [fix] red observed, fix shipped
- [pin] test shipped as characterization lock (current behavior already correct)
- [blocker] part of the item not shipped, journal-analyzed

## Item 1 (UTA-B2) index/fingerprint.rs contexts_material

Harness step (not behavior): generalized `contexts_material` to
`&[impl serde::Serialize]` slices so a cache.rs-style injected-unserializable
seam compiles (the AR-05 precedent at cache.rs:92 is generic for exactly this
reason). Concrete callers (fingerprint.rs internal, cli discover.rs:133 pass
`&[CompilationContext]`/`&[TranslationUnit]`) compile unchanged through the
generic; no caller edits needed. Red then targets the `unwrap_or_default()`
bug itself, not the signature.

(RED observed: 3 failures, material == "" / failing fingerprint == empty-string
fingerprint - exactly the predicted semantic reason. Canonical red line:
`15 passed / 3 failed / 0 ignored / 1 suites`.)

[fix] GREEN: `unwrap_or_else(|error| format!("SERIALIZE-FAILED:{error}"))`.
Canonical green line: `22 passed / 0 failed / 0 ignored / 3 suites`
(also carries item 14 pins; capture target/testrun-capture.txt).

## Item 14 (index/types.rs) IndexResult::new pins

[pin] Shared-symbol dedup across two TUs (total_symbols counts ids, not
records), per-TU edge tally with complete/incomplete split, empty-state
all-zero. Green in the same index run as item 1.

## Item 2 worker-core tu_store save tmp-name

CONFIRMED pre-existing fix: tu_store.rs save_shard writes
`.{}.{}.{}.tmp` = file_id + process::id() + process-local AtomicU64 counter
(lines 52-59); the tmp names cannot collide across threads (counter) or
processes (pid). The code arrived verbatim in 0861db0 (AR-20a worker-core
extraction) - git follow-log shows no later change. No bug to red against,
so per T2 no-code-without-red discipline this row ships as a concurrency
LOCK, not a fix; commit message must not claim a tmp-race fix.

[pin] Lock `concurrent_savers_of_one_file_id_never_mix_checkpoint_bytes`:
25 barrier-synchronized rounds, two writers on one FileId; every surviving
shard is one writer's complete, self-consistent envelope; zero tmp residue.

## Item 5 worker-core/task.rs TuTask::cmp pins

[pin] Equal-cost descent table (source_path asc, file_id asc, task_id asc),
PartialEq field set incl. whole `tu`, estimated_cost=0 floor, BinaryHeap pop
order invariant across 6 input permutations.

## Item 10 worker-core/tu_store.rs corrupt-shard pins

[pin] Truncated JSON -> miss; non-UTF-8 bytes -> miss; create_dir_all blocked
by a file -> save_shard Err surfaces.

## Item 12 worker-core/tu_store.rs edge_destinations pins

[pin] Mock resolver (`ScriptedOutcomeResolver`): Wildcard fan-out preserves the
resolver's id order (not lexicographic); External angled header found only via
-I dir (out-of-root temp sdk); Unresolved quoted found via includer dir with
the synthetic (FileId, rel-path) shape; unfindable directive -> no destination.

## Item 13 worker-core/tu_store.rs fingerprint_scoped_context pins

[pin] In-root include_dirs/forced_includes rewritten repo-relative, out-of-root
system dirs verbatim; scoped contexts equal across two copied roots.

Canonical worker-core line (all of the above green):
`19 passed / 0 failed / 0 ignored / 2 suites` (capture target/testrun-capture.txt).

## Items 3-15

## Item 3 (UTA-B6) scheduler engine.rs checkpoint loop [fix]

(RED observed: `foreign_tu_file_id_is_never_checkpointed_with_an_empty_
fingerprint` failed at base - the foreign id's shard WAS written with the
empty fingerprint. Canonical red line: `27 passed / 1 failed / 0 ignored /
1 suites`. The save-failure test passed at base as expected: a swallowed
failure leaves no in-process observable - that is the bug's definition -
so its assertions pin the post-fix contract while the warning ships.)

GREEN shape (journal the chosen shape, per handoff):
- foreign `tu_file_id`: skipped via `match pending_fingerprints.remove(..)`,
  per-incident `eprintln!` WARNING, still pushed into completed_analyses
  (no data loss). Red-proven: no shard JSON on disk for unclaimed ids.
- `let _ = save_shard(..)`: replaced by `if let Err(..)` warn + count; run
  continues (lost checkpoint costs a re-run, never a wrong answer).
- Run-level totals surface as an aggregate WARNING log line.
- Loop extracted to `persist_shard_checkpoints(..)` to keep `analyze()`
  under the 100-line hygiene ceiling after the added branches.
- `checkpoint_save_failure_surfaces_but_does_not_kill_the_run`: pin (passes
  at base and after; asserts run Ok, blocked target untouched, healthy shard
  saved with honest fingerprint).
Canonical green line: `28 passed / 0 failed / 0 ignored / 2 suites`
(capture target/testrun-capture.txt).

[BLOCKER - rippling part, NOT shipped] Surfacing the incident COUNTS in the
returned value: `ProductionAnalysisStats` is struct-literal-constructed by
crates/cli/src/commands/analyze.rs:102 and `ProductionAnalysisResult` at
crates/cli/src/commands/analyze.rs:119; any new field on either breaks the
cli build, and pipeline holds `Option<ProductionAnalysisStats>`
(crates/pipeline/src/lib.rs:184). Adding return-value fields needs cli
caller edits = outside the touch-set, so per instruction the return-field
part is journal-BLOCKED; the in-touch-set part (skip + non-silent WARNING
channels + red/green tests) shipped. Cross-crate handoff needed if the
counts must reach the CLI report.

## Items 4, 6, 7, 8, 9, 11 scheduler pins

- [pin] item 4: `two_permanent_failures_surface_only_the_first_error_while_
  successes_complete` (3 workers, 2 fatal + 3 good: exactly ONE of the two
  fatals is the returned error, all 3 good tasks executed, queue drained) and
  `queue_is_restored_with_residue_when_both_workers_break_on_fatal_tasks`
  (2 workers, 2 fatal: the unpopped good task is restored verbatim). Current
  pool behavior matched the contract at base - both green pre-fix, so NO pool
  code change (handoff: "fix only if current behavior deviates"). UTA-B6's
  "queue-restore is unasserted" gap is now asserted.
- [pin] item 6: format_memory table 0/1023/1024/1MB-1/1MB/1GB-1/1GB;
  format_elapsed 0.00s/59.99s/60s/522s; peak floor via
  build_analysis_stats(peak=0) -> 1MB, 5MB passes through.
- [pin] item 7: HashPartitioner::new(0)/ShardedReducer::new(0) clamp to 1;
  partition_key < num_partitions over 100 keys x {1,2,7,64}; clamped-zero
  pins every key to 0.
- [pin] item 8: empty file -> 1; directory path uses the metadata branch
  (size-or-floor, platform-robust formulation); costs from real temp files
  propagate through from_translation_units_with_context (30/10/5 pop order)
  with context attached to every task.
- [pin] item 9: empty queue -> Ok(empty) + queue empty; num_workers:0 clamps
  to one draining worker.
- [pin] item 11: `pin_body_only_missing_file_analyzes_empty` in worker.rs -
  body-only arm on a missing source returns Ok-with-nothing (report-flagged
  behavior; the store-context arm differs by design: unreadable sentinel).
Canonical scheduler green line (pins + item 3): `40 passed / 0 failed /
0 ignored / 2 suites` (capture target/testrun-capture.txt).

## Item 15 (report only, zero edits)

- scheduler `WorkerPoolConfig::memory_limit_bytes`: dead knob CONFIRMED.
  Write sites only: pool.rs:16 (field decl), pool.rs:24 (Default),
  engine.rs analyze (None), tests/integration/tests/
  m13_production_scale_test.rs:171 (None). Zero read sites: `run_worker_pool`
  and the drain workers never touch `config.memory_limit_bytes`. Deletion is
  an API decision (H5 report-only per the spec row).
- index `cache.rs IncrementalCache` (+ new/get/insert/validate_and_get/len/
  is_empty, cache.rs:190-233): dead public API CONFIRMED. Repo-wide scan:
  zero references outside its own definition (no `IncrementalCache::` call
  anywhere in crates/, tests/, tools/; only historical docs/plans text).

## Refactor notes (R1/R2 forced, no behavior delta)

- `engine.rs::analyze` hit 101 lines with the totals-warning inline -> the
  aggregate WARNING moved into `persist_shard_checkpoints` (returns unit);
  analyze back under 100. Scheduler suite re-run green (same 40).
- `tu_store.rs` hit 1015 lines -> all T5 tests extracted verbatim to
  `crates/worker-core/src/tu_store_t5_pins.rs` (declared
  `#[cfg(test)] #[path = ...] mod t5_pins;`): a child module keeps legal
  access to the private seams (`edge_destinations`,
  `fingerprint_scoped_context`). `make_ctx` duplicated as local `pins_ctx`
  (12 lines) because the original lives in the sibling `tests` module.
  Worker-core suite re-run green (same 19).

## Gate record (commands run, digest lines)

- Gate 1 index red:  `15 passed / 3 failed / 0 ignored / 1 suites`  (exit 1,
  expected red) / green: `22 passed / 0 failed / 0 ignored / 3 suites`
- Gate 2 worker-core: `19 passed / 0 failed / 0 ignored / 2 suites` (pins;
  green at base as characterization, re-run green after module extraction)
- Gate 3 scheduler red: `27 passed / 1 failed / 0 ignored / 1 suites`
  (exit 1, expected red) / green: `40 passed / 0 failed / 0 ignored /
  2 suites`
- Gate 4 hygiene: first run flagged MY R1 (engine analyze 101) + R2
  (tu_store 1015) - both refactored (above); re-run shows only pre-existing
  R3 warns (none authored by this hand; shifted-line-number ones:
  fingerprint.rs a/b = untouched test_inputs_* lines, pool.rs res = untouched
  test_worker_pool_max_retries_exceeded, tu_store tmp = untouched save_shard
  internals).
- Consumer check (out-of-touch-set compile safety, no edits):
  `cargo check -p ariadne-cli -p ariadne-pipeline` via distill -> PASS,
  0 warnings.
- Gate 5 encoding: the handoff-literal `--staged` runs vacuously (no manual
  staging step exists between gate and commit_scoped); the substantive check
  mirrors the T1-T4 precedent with `--all` (recorded after the commit).
- Gate 6 commit: `commit_scoped --scope crates/scheduler/** --scope
  crates/worker-core/** --scope crates/index/** --scope
  tasks/uta-t5-journal.md` per handoff.

## Commit-message truthfulness note

Handoff gate-6 message reads "fingerprint hardening + checkpoint race/error
surfacing". What ACTUALLY shipped: UTA-B2 fingerprint fix (item 1, red->green)
and UTA-B6 checkpoint fix (item 3: foreign-id skip + save-failure surfacing,
red->green). There is NO checkpoint tmp-RACE fix: item 2 verified the
pid+counter tmp naming pre-existed (lock only). Per the T2 truthfulness
precedent and the orchestrator instruction, the committed message names only
what shipped; deviation documented here.

## Final gate record (post-refactor, verbatim digest lines)

- index:       `22 passed / 0 failed / 0 ignored / 3 suites`
- worker-core: `19 passed / 0 failed / 0 ignored / 2 suites`
- scheduler:   `40 passed / 0 failed / 0 ignored / 2 suites`
- capture path: target/testrun-capture.txt (printed per run by the distiller)
- hygiene_check: R1/R2 zero violations (R3 pre-existing warns only)
- encoding_guard: `--all` exit 0 (recorded below)
