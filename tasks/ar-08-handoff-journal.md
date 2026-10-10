# AR-08 Journal - Stage-5 by-id symbol metrics (P1 HIGH, part 2)

Handoff: `tasks/ar-08-handoff.md`. Depends on AR-07 (`8ea8d0d`, LANDED).
Worktree: `.kilo/worktrees/shining-enthusiasm`. Host local time in this
journal is UTC-4 (calibration: the board turn opened at 2026-10-10T00:13:34Z,
host clock read Fri 10/09/2026 20:13).

## What was done

- `crates/pipeline/src/stage_load.rs` (the only implementation file touched):
  - New private `symbol_query_resolution(&ReducedSemanticModel) ->
    HashMap<&str, &SymbolId>`: one O(S) pass building the query -> id table
    with the SAME first-match rule `LoadAnalyzer::symbol_id_by_name` applies
    (key-ascending `BTreeMap` walk - the order that scan uses - and both of
    its match arms, the key string `id.0` and the record `name`, offered only
    while the slot is vacant, so the surviving value is the first record the
    scan would have returned). Keys and values borrow the model.
  - `collect_symbol_metrics` now walks `analyzer.model.symbols.values()`,
    resolves each record ONCE through that table, and hands `&SymbolId`
    chunks to `analyze_symbol_chunk`, which calls the AR-07 by-id entry
    `analyze_symbol_load_by_id`. `LOAD_PASS_CHUNK`, the sequential chunk
    loop, `par_iter().filter_map().collect()` and the append order are
    untouched, so element order and the skip contract are unchanged.
  - The `reduced` parameter of `collect_symbol_metrics` was dropped: the pass
    now reads `analyzer.model` for BOTH the walk and the resolution, so it
    cannot index one model while resolving against another (the analyzer's
    model is the same `&reduced` `ensure_load` handed to
    `build_stage5_analyzer`). Call site updated in the same file. The name
    chunk (`Vec<String>` with one `clone()` per symbol) became a
    `Vec<&SymbolId>` chunk - zero clones.
  - One import added: `use ariadne_index::types::SymbolId;`.
- New `crates/pipeline/tests/load_symbol_metrics_lock_test.rs` - the
  characterization lock (3 tests, see TDD sequence).

## Complexity rationale (the term removed)

Old shape, per symbol pass: for every one of the S reduced symbols the pass
called `analyze_symbol_load(name)`, and that entry resolves its query with
`symbol_id_by_name` - a linear `find` over the whole symbol map doing two
string comparisons per candidate. The pass therefore ran O(S^2) string
comparisons and S `String` clones, over `BTreeMap` nodes that do not sit in
cache next to each other. The by-id entry needs no resolution at all (one
O(log S) `BTreeMap::get`).

New shape: one O(S) hash-table build (S inserts x 2 arms) + S O(1) lookups +
S by-id calls, zero name clones. The quadratic comparison term is gone; what
removes it is the precomputed table, so the pass stays byte-identical instead
of merely resolving differently (see the finding below).

Measured term isolation (temporary ignored probe, `LoadAnalyzer` public API,
S synthetic symbols with id == name - the C++ lane shape; both arms timed
with no metric bodies involved; the probe file was deleted after the capture,
so these numbers are its record):

| S      | old: S x `symbol_id_by_name` | new: table build + S lookups | ratio |
| ------ | --------------------------- | ---------------------------- | ----- |
| 2160   | 93.9 ms                     | 1.4 ms                       | 67x   |
| 6000   | 708.5 ms                    | 3.6 ms                       | 194x  |
| 12000  | 3134.9 ms                   | 8.3 ms                       | 379x  |
| 24000  | 13558.7 ms                  | 16.3 ms                      | 832x  |

The old arm grows quadratically (x4.3 for a x2 scale), the new arm is flat.
At the bench fixture's S=2160 the removed term is ~92 ms of work, spread
across the rayon workers - which is why the WALL delta on the bench row is
tens of ms, not ~90 ms (see bench section). At corpus scale (the M28 corpus
that motivated the ceiling work carries ~2.6x the files of that fixture and
proportionally more symbols) the same term is seconds even divided by cores.

## Correctness finding: the naive "iterate ids" switch is NOT byte-identical

AR-07's journal anticipated this hop would "iterate ids and bypass name
resolution entirely". That would have changed stage-5 output on any model
where two records share one `name`:

- C++ cannot show it - `lang-cpp/src/parser.rs` builds `SymbolId::new(name)`,
  so id == name and every name resolves to its own record.
- Scala does: `lang-scala/src/parser/declarations.rs` sets `name =
  decl.simple_name` while the id is package-qualified. `a.Widget` and
  `b.Widget` are two records with the name `Widget`.
- There the legacy pass emits the FIRST key-ascending match twice: the
  `b.Widget` entry carries `a.Widget`'s metric (target file, loads, symbol
  sets). Captured pre-change from the Scala fixture, this is line 2 of the
  `scala_lane_name_collision_keeps_first_wins_metric` literal.

So the switch went through a resolution table that replicates the scan's
first-match rule rather than through the record's own id. Multi-language
output (the project's core extensibility intent) is unchanged either way; the
legacy resolution quirk is preserved, not silently "fixed" - fixing it is a
behavior change that belongs in its own handoff, not inside a perf refactor.

## TDD sequence (characterization-first, per the handoff's byte-lock)

1. Capture: one test cold-built stage 5 on each fixture and wrote the
   rendered symbol pass to `target/ar08-capture-{cpp,scala}.txt` -
   `1 passed / 0 failed / 0 ignored / 1 suites`. This is the pre-change tree
   capture the literals come from. The capture itself is what surfaced the
   Scala first-wins behavior above.
2. Pin: literals moved into three tests, capture helper deleted -
   `3 passed / 0 failed / 0 ignored / 1 suites` GREEN PRE-CHANGE (a
   characterization lock cannot be red against the tree it characterizes;
   same stance as AR-06's byte-lock, journal sec. "Gate results").
   - `cpp_lane_symbol_metrics_match_recorded_literal` - 5 symbol metrics,
     every `LoadMetric` field (target name/file, four loads, both symbol
     sets, both source groupings, largest-collateral pick) in report order.
   - `scala_lane_name_collision_keeps_first_wins_metric` - the 2-metric
     literal above.
   - `two_independent_tempdirs_render_identically` - the rendering carries
     fixture content, never the temp root.
3. Non-vacuity: the Scala literal was mutated to what a naive by-id switch
   produces (line 2 = `b/widget.scala` / `b.Widget`) -
   `2 passed / 1 failed / 0 ignored / 1 suites`, the expected failure,
   `left` showing the real first-wins rendering. Literal restored, re-run
   green. This run is the evidence that the lock discriminates exactly the
   risk this refactor carries.
4. Green: implementation as above -
   `35 passed / 0 failed / 4 ignored / 8 suites`.
   Pre-change scoped baseline was `32 passed / 0 failed / 4 ignored / 7 suites`
   (35 = 32 + the 3 new locks). Every pre-existing stage-5 lock stayed green
   UNEDITED: the I6 event-stream literal and the sequential-reference
   oracles in `load_perf_bench.rs`.

## Bench: before/after (median-of-3, `python tools/distill.py -- python tools/bench_compare.py`)

Fixture per the bench source: 520 headers / 40 TUs, 2160 reduced symbols, 560
file metrics. The `symbol_pass+prep` row is `ensure_load` start -> first
`CollateralFile` event (the symbol loop runs entirely before any file event),
so it is the row this hop targets.

| run (host local, UTC-4)                     | symbol_pass+prep | ensure_load total | verdict |
| ------------------------------------------- | ---------------- | ----------------- | ------- |
| BEFORE, ended ~20:33                        | 1586.0 ms        | 4559.5 ms         | PASS    |
| AFTER #1, ended just before 20:37           | 1539.6 ms        | 4458.0 ms         | PASS    |
| AFTER #2, 20:40:05 -> 20:41:43 (re-run)     | 1529.9 ms        | 4552.2 ms         | PASS    |

Observed ratio on the targeted row: 0.971x / 0.965x, i.e. a consistent
-3.2% median drop (two independent median-of-3 samples both below the
pre-change median). `ensure_load total` is noise-dominated at this fixture
(4559.5 -> 4458.0 -> 4552.2, spread ~2%) - the save tail (~2.4 s) dominates
that total and this hop does not touch it. Each run's verdict line was
verbatim: `bench compare: 3 run(s), no regressions past 1.25x baseline`.
No row regressed; the 1.25x threshold was never approached in either
direction on any row.

Honest reading of the magnitude, per the handoff's "declare observed ratio;
REJECT if none": the drop is real and reproducible but small at THIS fixture,
because the quadratic term it kills is ~92 ms of parallelizable work at
S=2160 (probe table above) inside a ~1.5 s row. The handoff's expectation of
a "measurable" drop holds at corpus symbol counts (67x/194x/379x/832x by
scale), not at the bench's 2160. Reviewer: if the ratio alone is the accept
criterion, the probe table is the load-bearing evidence, and it says the term
is O(S^2) removed.

## Deliberately NOT done

- No edits to `crates/graph/**` (AR-07 landed; another agent is mid-flight
  there), no edits to `crates/ui/**`, `tests/integration/**`, or
  `crates/cli/tests/payload_tail_bench.rs` (the bench fixture is read-only
  material here).
- No new public API: `collect_symbol_metrics` and
  `symbol_query_resolution` stay private (the locks run through the public
  `Pipeline::ensure_load` / `get_or_build_load` surface, which is what a
  stage-5 output lock should ride anyway).
- No name->id map inside the graph crate (AR-07 declined it with reasons; the
  pipeline builds its own, from the model it is already iterating).
- No change to the legacy first-wins resolution semantics (see finding).
- No `LOAD_PASS_CHUNK` retune, no memo/cap changes, no rayon topology change.
- `bench_compare --bench stage5-memory` (RSS mode) not run: it is not in this
  handoff's gate list and its 5-invocation RATIFIED protocol is a separate
  expensive gate. The added structure is bounded and stated here: the table
  holds <= 2 entries per symbol of borrowed `&str`/`&SymbolId` (no string or
  id clones, ~2 pointers per entry) and is dropped when the symbol pass
  returns - before the file pass and before the artifact-save span that the
  M28 T2 memory work bounds. The old pass allocated one `String` per symbol
  for the chunk (chunk-bounded), so the change trades a chunk-bounded
  allocation for a symbol-count-bounded table of pointers.

## Deviations from the handoff

- D1: the handoff's line "`stage_load.rs:347` collects `(symbol_id, name)`
  once (or resolves via the AR-07 map)" assumed either an id+name pair or an
  AR-07-provided map. AR-07 shipped no map (documented in its journal), and
  collecting pairs while calling by-id with the record's own id is the
  non-byte-identical switch above. Taken: the "resolves via ... map" branch,
  with the map built locally in `stage_load.rs` under the exact scan rule.
  Re-anchoring note: line 347 in the handoff is the `filter_map` inside
  `analyze_symbol_chunk`; the loop the change targets is
  `collect_symbol_metrics` (the collection point) plus that chunk helper.
- D2: `collect_symbol_metrics` lost its now-redundant `reduced` parameter
  (single-file change, call site updated in the same edit).
- D3: the byte-lock lives in a new integration test file rather than in
  `load_stage_lifecycle_test.rs` (touch-set allows `crates/pipeline/tests/**`);
  the existing lifecycle/I6 file is untouched and still green.

## Gate results (verbatim digest lines, own run stdout per the capture-race ruling)

- Pre-change scoped baseline: `32 passed / 0 failed / 4 ignored / 7 suites`
- Capture run: `1 passed / 0 failed / 0 ignored / 1 suites`
- Pinned locks, pre-change: `3 passed / 0 failed / 0 ignored / 1 suites`
- Non-vacuity mutation: `2 passed / 1 failed / 0 ignored / 1 suites` (expected red, restored after)
- Post-change scoped gate: `python tools/testrun.py --distill -- -p ariadne-pipeline` -> `35 passed / 0 failed / 4 ignored / 8 suites`
- `python tools/bench_compare.py` (via distill) x 3 -> `bench compare: 3 run(s), no regressions past 1.25x baseline` (table above)
- `python tools/encoding_guard.py --all` - see below
- `python tools/hygiene_check.py` - see below
