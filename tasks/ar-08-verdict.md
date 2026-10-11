# AR-08 Verdict - APPROVED

Handoff: `tasks/ar-08-handoff.md`. Commits: `7f7df89` (implementation +
locks + journal), `73f2160` (journal addendum). Base of range: `3a26a9a`.
Head at review: `73f2160`, branch `shining-enthusiasm`.

## Gate summary (fresh, own stdout; verbatim lines in `tasks/ar-08-execution-results.md`)

- Scoped pipeline: `35 passed / 0 failed / 4 ignored / 8 suites` - exactly
  the expected line.
- m00 parity + golden suites UNEDITED green: `4 passed / 0 failed / 0 ignored / 2 suites`.
- bench_compare: journal's three runs all PASS; my two clean reruns show the
  targeted `symbol_pass+prep` row BELOW baseline (1565.3 / 1576.0 vs baseline
  1585.6); see Finding 1 for the exit-1 micro-row WARN attribution.
- encoding_guard --all: silent (clean, whole tracked tree).
- hygiene_check: zero new rows from this hop; all blocking rows foreign
  (server.rs AR-13 pair; m15 = AR-17 WIP, dirty per checkpoint_guard).
- Tools unit suites: `Ran 219 tests in 19.349s` / `OK`.
- Commit-vs-touch-set: `git diff --name-only 3a26a9a..73f2160` =
  `crates/pipeline/src/stage_load.rs`,
  `crates/pipeline/tests/load_symbol_metrics_lock_test.rs`,
  `tasks/ar-08-handoff-journal.md` - all three inside the declared globs.
  NO `crates/graph` path in either commit (verified per-commit with
  `git show --stat`): AR-07's glob-leak does not repeat.

## The central claim (resolution table) - VERIFIED SOUND

Independent verification, not the journal's say-so:

1. Legacy semantics, read from `crates/graph/src/load.rs:226-232` (file NOT
   in the dirty set, so working-tree bytes == reviewed blob):
   `.find(|(k, v)| k.0 == query || v.name == query)` over `model.symbols`,
   which is `BTreeMap<SymbolId, ReducedSymbolRecord>`
   (`reduced.rs:86`) - ascending key order, first record matching EITHER arm
   wins. The name entry (load.rs:563-568) delegates to
   `analyze_symbol_load_by_id` after that resolution, so a table reproducing
   the resolution is byte-identical by construction.
2. `symbol_query_resolution` walks `&model.symbols` (same BTreeMap, same
   ascending order) and offers `entry(record.name).or_insert(id)` then
   `entry(symbol_id.0).or_insert(id)` - both match arms of the scan are
   covered, and vacant-only insertion makes the survivor exactly "first
   record in key order matching either arm", identical to the scan. Within a
   record, arm order is irrelevant (both arms map to the same `symbol_id`).
   The HashMap is only ever built (in BTreeMap iteration order) and read via
   `get` - no HashMap iteration determines a value; deterministic across runs.
3. The pinning test literally contains the pre-captured shared-name fixture:
   `setup_scala_repo` builds `a.Widget`/`b.Widget` sharing `name=Widget`
   (test lines 76-98), and the literal (lines 226-227) asserts the FIRST
   record's metric (`file=src/main/scala/a/widget.scala`,
   `sem_syms=[a.Widget]`) emitted TWICE - the legacy quirk, NOT the
   naive-by-id output (`b/widget.scala`/`b.Widget`). My own derivation of the
   legacy scan on this fixture produces exactly that rendering.
4. Non-vacuity is evidenced in the journal: capture `1 passed / 0 failed / 0
   ignored / 1 suites`, pinned locks GREEN pre-change `3 passed / 0 failed /
   0 ignored / 1 suites`, mutation to naive-by-id values
   `2 passed / 1 failed / 0 ignored / 1 suites` (expected red, restored).
   A characterization lock is legitimately green pre-change (same stance
   AR-06's byte-lock set); the mutation run is what plays the red role.
5. Chunk width, output order, skip contract unchanged (full diff read):
   `LOAD_PASS_CHUNK` untouched, same walk of `model.symbols.values()` (same
   BTreeMap order as the old `reduced.symbols.values()` - `analyzer.model`
   IS the same `&reduced`, `build_stage5_analyzer` ends with
   `LoadAnalyzer::with_memo(graph, reduced, ...)`), same
   `par_iter().filter_map().collect()` per chunk with sequential append, same
   error-skip (by-id entry errors with the same "No defining file for
   symbol" message; the new `else { continue }` is unreachable by
   construction - every walked record's name is a table key - and journal-
   acknowledged as a shape guard, harmless).

## Deviation dispositions

- D1 (resolution table instead of map/naive bypass): ACCEPTED. The handoff
  invariant "Output byte-identical (same metrics computed)" is binding and
  the naive by-id bypass verifiably violates it on shared-name models (Scala
  lane, `name = decl.simple_name`). The handoff's parenthetical "(or
  resolves via the AR-07 map)" names the map branch; AR-07 shipped no map,
  and building it locally in `stage_load.rs` is inside the touch-set. The
  re-anchoring of "line 347" to `collect_symbol_metrics` + chunk helper is
  factually correct per the diff.
- D2 (dropped redundant `reduced` parameter): ACCEPTED. Verified sound: the
  analyzer stores the very `&reduced` passed to `with_memo`, so the two
  handles were one map; the single-handle form additionally cannot index one
  model while resolving against another. Call site updated in the same file.
- D3 (new test file): ACCEPTED. `crates/pipeline/tests/**` is a declared
  touch-set glob; `load_stage_lifecycle_test.rs` and the
  `load_perf_bench.rs` sequential-reference oracles are proven UNEDITED
  (empty range-diff for those paths, and they are green in my run).
- Characterization-first instead of literal "red-first" wording: ACCEPTED.
  For a refactor whose pinned invariant is unchanged output, a red test is
  impossible without a behavior change; the mutation run demonstrates the
  suite discriminates the exact risk. Declared and argued in the journal.

## Findings (numbered)

1. MINOR / recorded, no action on this hop: my acceptance bench runs exited 1
   with WARN rows that are not attributable to AR-08. Contended run #1: four
   WARNs, all non-stage-5 rows (+30/+36/+37/+72% on warm-clone,
   generate_report, path_to_id maps, build_include_elements). Clean reruns:
   exactly one WARN each, flapping between `clone (cold)` (1.7 vs 1.3 ms)
   and `clone (warm)` (1.4 vs 1.0 ms) - a 0.4 ms jitter on 1 ms baselines
   whose 1.25x gate sits below the timer's noise floor, in code paths this
   diff never touches. Journal's own three runs:
   `bench compare: 3 run(s), no regressions past 1.25x baseline` x3, targeted
   row 0.971x/0.965x. My clean reruns put the targeted row below baseline
   (0.987x/0.994x vs the recorded 1585.6). Disposition: pass; suggest the
   1 ms clone-row noise floor become a ledger observation, not an AR-08 debt.
   Note my bench compiled the dirty shared worktree (AR-09's WIP
   `crates/graph`), so my numbers measure HEAD+WIP, not AR-08 alone.
2. Recorded: isolated-probe quadratic-removal table (93.9 vs 1.4 ms at
   S=2160 up to 13558.7 vs 16.3 ms at S=24000) is in the journal, and the
   probe file was never committed (final suite line `35/0/4/8` after probe
   deletion matches the post-change gate line). Not re-run, per dispatch.
3. Recorded gate-tooling refusals during this review (boilerplate
   catch-all-deny text `{"permission":"bash","pattern":"*","action":"deny",
   "source":"global"}` each time): `git grep` at a revision, `python -c`,
   and any piped/redirected command. No expected gate was skipped because of
   them - blob checks were done via the Read/grep tools on files proven
   untouched by the dirty-file census, and the tools suite ran bare.

## Follow-up dispositions

- The legacy first-wins shared-name resolution (a shared `name` re-emitting
  the first record's metric is arguably wrong output): explicitly out of
  scope here, correctly deferred by the journal to a future behavior-change
  handoff with golden-artifact review. This verdict records it as a known
  deferred-behavior item for the ledger, matching the journal's reviewer
  note.
- Finding 1's clone-row noise-floor note: ledger-bound observation.
