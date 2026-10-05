# T1a Handoff - M27 tu-index-hang-throughput: RELEASE-profile `boot_stages` bench + sufficiency verdict

Milestone: `tu-index-hang-throughput` (M27, ledger row 2026-10-05). Single
scope: `crates/pipeline/tests/` (+ one `tasks/` evidence doc). Sizing law
held: 1 module dir, 6 implementer gate invocations (<= ~10), ~350-550
expected produced lines (<= ~1200). Workspace-wide gates belong to the
reviewer hop. The bench REGRESSION GATE (`bench_compare --bench boot-stages`)
is T1b's file; PR-2 (operator `--update-baseline` on a VERIFIED-IDLE machine)
gates the GATE and the landing, not this measurement hop.

DISPATCH PRECONDITION: `tasks/tu-index-hang-throughput-t0-evidence.md` landed
(T0-report hop). Parallel-safe with T0b and T2 (disjoint trees).

Why: E3 proved the user's 90-minute, 3.1 s/TU run was a DEBUG build - part
of "indexing takes forever" is profile, not algorithm. Before ANY
throughput machinery (T3), the release+single-thread SUFFICIENCY GATE must
be answered with a RELEASE-profile table (shape sec. 3.3; measurement-before-
prescription).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md`
   sec. 3.3 (labels, gate plan, E3 obligations a/b/c), sec. 0 (E1/E3),
   sec. 9 (`load_perf_bench.rs` 866 lines - do NOT grow it).
3. `crates/pipeline/tests/load_perf_bench.rs` lines 40-141 (the pure
   generators to REUSE: `fanout_targets`, `header_source`, `write_headers`,
   `tu_source`, `write_compile_commands`, `generate_fixture`, `millis`) and
   its `#[ignore]` idiom + `label | ms` output grammar.
4. `crates/pipeline/src/lib.rs` public surface the bench replicates
   (`Pipeline::new(path, workers)`, `ensure_include_graph`, `ensure_index`,
   `rehash_inputs`) + `crates/cli/src/commands/ui.rs:312-316` (what UI boot
   actually runs).
5. `AGENTS.md` build/test sections, `tools/ui_smoke.py` build step, any
   repo run scripts/aliases (launch-profile record - obligation (b)).
6. `docs/specs/tool-output-distillation.md`.

## Objective and invariants

1. `crates/pipeline/tests/common/mod.rs` (NEW): COPY (duplicate - shape
   allows; `load_perf_bench.rs` stays byte-identical) the pure generators,
   parameterized: `generate_fixture_sized(root, header_count, tu_count)`
   (the load_perf_bench setting is 520/40; this bench's consts, ratifiable
   at the evidence review: `const HEADER_COUNT: usize = 2000;`
   `const TU_COUNT: usize = 400;` - "start" values per shape sec. 3.3).
2. `crates/pipeline/tests/boot_stages_bench.rs` (NEW), two `#[test]
   #[ignore]` fns printing the `label | ms` grammar:
   `boot_stages_release_timing` - emits EXACTLY these eight labels (shape
   sec. 3.3 verbatim; they become T1b's `DOCUMENTED_PHASES_BOOT_STAGES`):
   ```
   discover+hash total (release)
   build discovery (contexts) total (release)
   stage fingerprints total (release)
   include-graph cold total (release)
   include-graph warm load total (release)
   tu-index cold total (W=1, release)
   tu-index cold total (W=N, release)
   tu-index warm reuse total (release)
   ```
   Method (replicate the boot path like `load_perf_bench` does):
   discover+hash via `ariadne_repo::Repository::locate` + `discover()`;
   build discovery via `BuildDiscoverer` with the pipeline registry shaper
   selection (`ariadne_pipeline::default_registry()`); fingerprints via
   `ariadne_index::StageFingerprints::from_inputs`; stage rows via fresh
   `Pipeline` instances over a temp fixture (cold = fresh artifacts tree;
   warm rows reuse the artifacts written by the cold run, measuring the
   load path with construction time subtracted by re-running
   `rehash_inputs`-equivalent construction cost - state the subtraction
   method in the doc comment); W=N uses `Pipeline::new(&path, Some(n))` with
   `n = usize::min(8, std::thread::available_parallelism)` (a second N is
   exactly the data the sufficiency gate needs; `W=N` is NOT T3 dispatch -
   `Some(workers)` is already today's library capability).
   The commit's doc comment carries the MEASUREMENT TABLE (date, machine,
   profile) beside the consts - the A4 calibration-evidence precedent.
   `boot_stages_debug_context_timing` - ONE row:
   `tu-index non-ASCII repro TU (debug, panic-pre-T2 / done-post-T2) | ms`.
   Builds a ONE-TU temp corpus whose source contains, via BYTE ESCAPES only
   (authored source stays pure ASCII, e.g. a `String` from
   `[0x63, 0x61, 0x66, 0xC3, 0xA9]`-style construction), the E3 shape
   (non-ASCII word run + non-ASCII punct run); runs `Pipeline::new` +
   `ensure_index` cold and prints the row. PRE-T2 under the DEBUG profile
   this run PANICS (`attempt to subtract with overflow`) - that captured
   panic is obligation (a)'s debug-context evidence and is EXPECTED (exit 1,
   ignored test, never gates the suite). Post-T2 (or under release where it
   silently wraps) it prints ms. Never add it to any gate tuple.
3. `tasks/tu-index-hang-throughput-t1-measurement.md` (NEW, tracked) with:
   (a) the RELEASE median table (paste the eight rows + medians over the
   runs you captured; run count >= 3, machine + date); (b) the DEBUG context
   row + captured panic line (path-cited, no raw dumps); (c) LAUNCH-PROFILE
   RECORD (obligation (b)): what AGENTS.md build lines / `ui_smoke.py`'s
   build step / repo run scripts yield, and the de-facto default profile an
   actual user gets - if debug is the de facto norm, record it AS A
   FINDING ("cheap throughput before any machinery"); (d) SUFFICIENCY
   VERDICT in this exact format:
   ```
   release ms/TU (W=1) = <tu-index cold total (W=1)> / TU_COUNT
   projected stage-2 cold wall at corpus scale = ms/TU * 5744
   W=N speedup factor = W=1 median / W=N median
   RECOMMENDATION: <ACCEPT|FAIL> - <one line>
   SUFFICIENCY-GATE: ACCEPT - release+single-thread meets the ratified
     wall-time bar at corpus scale (T3 NOT dispatched)
   ```
   or
   ```
   SUFFICIENCY-GATE: FAIL - release+single-thread does not meet the ratified
     wall-time bar at corpus scale (T3 dispatch required)
   ```
   The wall-time BAR is ratified at the evidence review by the
   orchestrator+user (shape sec. 11 Q2/Q3) - you record projection +
   recommendation; the ACCEPT/FAIL line is finalized by the review note
   appended under your section. Also propose K (improvement target) and the
   auto-workers cap N from the W=N row (Q3, Q7 - ratified at review).

Invariants: ZERO src change (pipeline `src/` untouched; bench replicates the
public path); `load_perf_bench.rs` byte-identical (866-line ceiling); goldens/
fixtures frozen; existing pipeline suite green UNEDITED; the bench is the
SYNTHETIC-scale measurement (real-corpus numbers live only in field-evidence
docs - shape sec. 4 "Out").

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/tu-index-hang-throughput-t1a-journal.md` (journal; always in-set)
- `crates/pipeline/tests/common/mod.rs` (NEW)
- `crates/pipeline/tests/boot_stages_bench.rs` (NEW)
- `tasks/tu-index-hang-throughput-t1-measurement.md` (NEW - evidence)

You MUST NOT touch: `crates/pipeline/src/**`;
`crates/pipeline/tests/load_perf_bench.rs` (byte-identical);
`crates/pipeline/tests/pipeline_test.rs`; `crates/cli/**`; `tools/**` incl.
`bench_baseline.json` (T1b + operator own it); `tests/golden/**`,
`tests/fixtures/**` (the debug-row corpus is constructed in a temp dir from
byte escapes, NEVER committed); any other file. No `git stash`, no branch
change.

## RGB protocol
Red: author the bench file with the debug-context test wired but the release
timing test printing only a partial label set (e.g. missing the two warm
rows) - the bench-run gate (Gate 1) then FAILS the label completeness check
you write as a doc-comment TODO list AND the measurement doc's checklist
cannot be ticked; capture. (This hop's true "test" is the 8-label gate tuple
T1b enforces; until then your Red evidence = first run missing rows + the
debug panic capture.) Green: complete rows; re-run. Refactor: shared helper
for the timed-section print; re-run.

## Hygiene (review-enforced)
Functions < 100 lines (each timed section a helper); files: `common/mod.rs`
<= ~180, `boot_stages_bench.rs` <= ~450; no growth of `load_perf_bench.rs`;
pure ASCII; `python tools/hygiene_check.py` exit 0.

## Frozen locks (each with its proving command)

| Lock | Proof |
|---|---|
| pipeline `src/` + `load_perf_bench.rs` byte-identical | `git diff --stat <start-rev>..HEAD` lists touch-set only |
| pipeline suite green UNEDITED | `python tools/testrun.py --distill -p ariadne-pipeline` -> `0 failed` |
| goldens/fixtures frozen; bench adds no committed corpus | diff-scope proof |
| mojibake + hygiene | `encoding_guard --staged` / `hygiene_check.py` exit 0 |

## Gates (exact commands, timeout 600000 ms each; scoped only)
Distilling entry points only (ledger row 58). Workspace-wide gates belong to
the reviewer hop.

1. Release bench: `python tools/distill.py -- cargo test --release -p
   ariadne-pipeline --test boot_stages_bench -- boot_stages_release_timing
   --ignored --nocapture --test-threads=1` - all EIGHT labels present with
   finite ms; capture path cited.
2. Debug context row: `python tools/distill.py -- cargo test -p
   ariadne-pipeline --test boot_stages_bench -- boot_stages_debug_context_timing
   --ignored --nocapture` - PRE-T2 the captured panic line is the expected
   evidence (exit 1 correct; journal records it).
3. Scoped suite: `python tools/testrun.py --distill -p ariadne-pipeline` ->
   `0 failed` exit 0 (ignored tests excluded; census +2 ignored).
4. `python tools/hygiene_check.py` - exit 0.
5. `python tools/encoding_guard.py --staged` - exit 0.
6. Commit: `python tools/distill.py -- python tools/commit_scoped.py --scope
   crates/pipeline/tests/common --scope
   crates/pipeline/tests/boot_stages_bench.rs --scope
   tasks/tu-index-hang-throughput-t1-measurement.md --scope
   tasks/tu-index-hang-throughput-t1a-journal.md -m "test(pipeline): boot
   stages RELEASE bench + launch-profile record + sufficiency verdict
   (M27 T1a)"`.

Total implementer gate invocations: 6 (<= 10).

## Acceptance (machine-checkable; implementer hop)
1. Gate 1 prints exactly the eight labels; medians table in the measurement
   doc matches the run (>= 3 runs, medians; run the release bench 3x - the
   same distill command, 3 invocations counted inside the 6).
2. Measurement doc sections (a)-(d) complete incl. the verbatim
   `SUFFICIENCY-GATE:` line with RECOMMENDATION.
3. Suite `0 failed`; hygiene + encoding exit 0; `git show --stat` = touch-set.
Reviewer hop: scoped re-runs; census `+2 ignored`; `encoding_guard --all`;
verdict: table credible (medians, >= 3 runs, idle-ish notes), labels exactly
the eight, debug row NOT in any gate tuple, launch-profile record names the
de-facto profile. The orchestrator appends the ratified wall-time bar +
final ACCEPT/FAIL to the measurement doc at evidence review; T1b dispatch
requires this landed.

## Discipline clauses
- UTF-8: Write/Edit only; pure ASCII (bench fixture bytes via numeric arrays
  / escapes).
- Bench scale bounded (R9): 2000/400 start; if a release run exceeds the
  600 s gate timeout, HALVE the scale, record the halving in the doc +
  journal, report (never silently scale down).
- Touch only the touch-set; out-of-scope: report, don't fix.
- Journal checkpoint after each gate.
- Context wall: STOP + disk-state inventory; successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/tu-index-hang-throughput-t1a-handoff.md --scope ... --cmd ...`.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence (incomplete-labels run +
debug panic capture, path-cited). 3. Green: eight-label digest + canonical
suite line + capture paths. 4. The four measurement-doc sections complete +
`SUFFICIENCY-GATE: RECOMMENDATION` line. 5. Deviations. Never paste raw
output.

## Reviewer-gate reference
Adjudicated via the `review-gate` skill. Extra checks: labels byte-exact
against shape sec. 3.3 (they are T1b's contract); medians >= 3 runs;
`load_perf_bench.rs` diff-empty; W=N row uses `Some(n)` not new machinery;
debug row excluded from gate labels; measurement doc ASCII.

## Sequencing note
After T0-report lands. Parallel-safe with T0b and T2. T1b dispatches after
this lands (its live gate run needs the bench binary) - and T1b's BASELINE
ROWS additionally need PR-2 done (operator `--update-baseline` on a
VERIFIED-IDLE machine; never run this yourself).
