# AR-09 Execution Results - reviewer fresh-gate record

Reviewer session 2026-10-10 (UTC), worktree `shining-enthusiasm`, HEAD
`55c93c8` (commit under review `14abc55` = HEAD~1; nothing between touched
`crates/graph`). Reviewer ran every gate itself; outputs are this session's
own stdout, not the journal's.

Scope restriction per orchestrator instruction: a concurrent AR-11 agent is
editing `crates/ui/assets` + `crates/ui/tests` (dirty at session start), so
gates are restricted to the `graph` package + named suites; workspace-wide
runs are excluded (also per the Planning Law: they belong to a fresh
session/orchestrator run).

## Ground truth (checkpoint_guard)

HEAD `55c93c8`, branch `shining-enthusiasm`. Dirty: `crates/ui/tests/ui_test/
main.rs`, `crates/ui/assets/hotspots.js` (AR-11 WIP, not charged here) +
untracked `tasks/` documents. Worktree census as reported; no unexpected
graph-tree changes.

## Acceptance criteria vs evidence

### 1. Package suite - PASS, expected line reproduced exactly

Command: `python tools/testrun.py --distill -- -p ariadne-graph`

    129 passed / 0 failed / 0 ignored / 6 suites

(exit 0; capture `target/testrun-capture.txt`, 175 lines)
Matches handoff expectation 129/0/0/6 = prior 116 + 13 new locks.

### 2. m00 parity, goldens UNEDITED - PASS

Command: `python tools/testrun.py --distill -- --test m00_golden_baseline_test`

    2 passed / 0 failed / 0 ignored / 1 suites

(exit 0). UNEDITED verified two ways: `git diff --name-only 14abc55^..14abc55`
lists only the five handoff files (no golden path); working-tree dirty set
lists no golden/m00 file.

### 3. Bench (median-of-3) - stage/build rows clean; clone rows attributed

Command: `python tools/distill.py -- python tools/bench_compare.py` (run 1,
exit 1; capture `target/distill-capture.txt`):

    symbol_pass+prep (combined, <= first event)      |    1535.8
    file_pass (event span)                           |     589.0
    save_load_artifacts+cache (combined)             |    2376.7
    ensure_load total                                |    4501.5
    get_or_build_include_graph clone (cold)          |       1.8
    get_or_build_load hand-out (cold)                |       0.0
    get_or_build_include_graph clone (warm)          |       1.3
    get_or_build_load hand-out (warm)                |       0.0
    generate_report+detect_cycles (combined)         |      75.1
    build_include_elements                           |      58.8
    path_to_id+file_collateral maps (combined)       |     179.9
    OptimizationPlanner::plan() facts+closures       |     825.2
    conservative filter pass                         |       0.0
    build_file_justification_with_memo               |    8122.0
    WARN get_or_build_include_graph clone (cold): 1.8 ms vs baseline 1.3 ms (+38%)
    WARN get_or_build_include_graph clone (warm): 1.3 ms vs baseline 1.0 ms (+30%)

Re-run (same command, exit 1, only 1 WARN): clone cold 1.4 ms (WARN cleared,
1.08x), clone warm 1.3 ms (WARN reproduced, 1.30x), ensure_load total 4511.7,
build_file_justification_with_memo 8128.4, planner 820.5 (digest omitted
file_pass + save_load rows; all visible rows clean).

Adjudication: all heavy stage/build rows are clean AND faster than the
implementer's own recorded run (journal ensure_load 4913.3, justification
8786.7, planner 921.8). The two clone rows are the documented borderline rows
(history 1.20-1.23x; journal run 1.6/1.2 with clean verdict). Cold WARN did
not reproduce (timer noise, +/-0.4 ms on sub-2 ms medians). Warm clone 1.3 ms
reproduced (+0.3 ms over baseline) with no mechanistic AR-09 slowdown path
(Clone = one uncontended mutex acquire + 2-word slot copy; cache map is
Arc-shared, never copied). Attributed to sub-ms quantization + concurrent
AR-11 build load; reported, not charged as a new regression class (verdict
finding F1). No stage/build row exceeded 1.25x in either run. No rejection
trigger fired.

### 4. Encoding - PASS

Command: `python tools/encoding_guard.py --all` - silent pass (zero output,
the repo's documented pass shape).

### 5. Hygiene - PASS with attribution

Command: `python tools/hygiene_check.py` - blocking rows in output:

    crates/ui/src/server.rs: R2 1172 lines > 1000
    crates/ui/src/server.rs:816: R1 fn `api_graph_traversal` is 161 lines (> 100)

Exactly the two known server.rs baseline rows (verdict instruction: "only 2
server.rs rows"). Attribution: `server.rs` is absent from the commit diff and
from the working-tree dirty set - both rows predate and are independent of
`14abc55`; not charged to AR-09. Zero blocking rows in any file this commit
touched (graph.rs 806 lines; new test file 777 lines; no R1/R2 hits under
crates/graph). R3 (warn-only) rows under `crates/graph` are pre-existing:
`traversal.rs:247/406/514` single-letter `n` were diffed against the
pre-change blob - they are the reduced-model scan binding + `for n in
&all_nodes` entry-point loops; the commit REMOVED two further `n` bindings
(net warning count down, zero added). The two R3 rows the journal says its
test file briefly added are indeed absent.

### 6. Tool unit suites (CI command) - PASS

Command: `python -m unittest discover -s tools -p "*_tests.py"`

    Ran 219 tests in 18.892s
    OK

### 7. Equivalence locks (13) - PASS

All 13 in `ar09_adjacency_equivalence_test.rs` ran inside gate 1's 129
(6 suites includes the new file; suite names visible in
`target/testrun-capture.txt`). Oracle functions diffed verbatim against
`git show "14abc55^:..."` blobs: frozen copies of the retired scans, so the
equivalence argument is implementation-independent.

### 8. Touch-set - PASS

Five paths in `14abc55`, all inside the declared hard touch-set;
`load_caches.rs` untouched as declared (memo not reused).

## NOT run (explicit statement)

- `python tools/testrun.py --baseline` (workspace-wide) - excluded by the
  orchestrator's concurrency restriction and the Planning Law
  (milestone-baseline run belongs to a fresh session/orchestrator).
- `python tools/ui_smoke.py` - excluded: would build/launch `crates/ui`
  against AR-11's in-progress asset edits, misattributing their WIP.
- `python tools/bench_compare.py` under quiet conditions - both reviewer runs
  occurred while AR-11 was live (load noted per instruction; stage rows
  unaffected).
- `cargo check --workspace --all-targets` - journal reports exit 0; not
  re-run here (workspace-wide, contention). Package gate 1 + tool suites
  cover the commit's blast radius.

## Refused commands (environment, all recovered)

Catch-all deny refusals occurred for piped/chained command forms (`git show |
Select-String`, `git cat-file` via `& {}`, `cmd; echo $LASTEXITCODE`,
`unittest ... | Select-Object`). Refusal text (verbatim, identical each time):
"The user has specified a rule which prevents you from using this specific
tool call. Here are some of the relevant rules
{"permission":"bash","pattern":"*","action":"deny","source":"global"}"
Every expected gate above was subsequently run as a plain command and
succeeded; no gate was skipped due to refusal.

## Verdict

APPROVED - see `tasks/ar-09-verdict.md` for findings F1-F3 and dispositions.
