# V3 Handoff - M26 boot-progress-visibility: BootMilestone + TuStarted events, boot wiring

Milestone: `boot-progress-visibility` (M26, ledger row 2026-10-05). Two-crate
handoff (`pipeline` + `cli`) - a declared sizing exception: the enum is
matched exhaustively in `ui.rs`'s `stage_progress_callback`, so the pipeline
event additions and the cli mapping MUST land as one compile unit (shape sec.
4: "compile-coupled by exhaustive `match`"). Written pipeline-touch
justification (ledger M26 row): boot events + `new_with_progress` + emission
points + the 959-line-ceiling stage-module extraction. Sizing law otherwise
held: 9 implementer gate invocations (<= ~10), ~950-1150 expected produced
lines (<= ~1200 - the verbatim-move dominates; if your REAL diff approaches
~1200, split at the commit boundary defined below and report - the two
commits are already an allowed split). Workspace-wide gates belong to the
reviewer hop.

DISPATCH PRECONDITIONS (hard): V1 landed (`ui/stage_spans.rs` exists, frame-1
first, panic bridge) AND V2 landed (`Repository::discover_with_progress`).
V3-before-T3 ordering stands: M27 T3b may not start before this lands
(`crates/pipeline/src/lib.rs` + extracted stage modules are its touch-set).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md`
   sec. 3.1 (verbatim event enums, `new_with_progress`, mapping contract,
   append-only rule), sec. 4 (V3 row), sec. 9 (extraction hygiene), R7.
3. `crates/pipeline/src/lib.rs`: 126-146 (`ProgressEvent`, `LoadMilestone`,
   `ProgressCb`), 177-257 (`new`/`build`/`with_progress`), 264-286
   (`compute_inputs`), 320-352 (`ensure_include_graph`), 488-532
   (`rebuild_index_single_threaded`), 560-959 (reduced/callgraph/load - the
   extraction source).
4. `crates/cli/src/commands/ui.rs` post-V1 (span block now via
   `ui/stage_spans.rs`; `stage_progress_callback`; `build_ui_payload` head;
   `run_pipeline_stages`), `crates/cli/src/commands/ui/stage_spans.rs`,
   `crates/cli/src/commands/index.rs` lines 33-37 (`index_progress_cb`).
5. `crates/repo/src/lib.rs` lines 15-35 (V2 hook you consume).
6. `crates/cli/src/commands/ui/progress_contract_tests.rs` - append-only
   floor; and V1's `ui/first_frame_tests.rs` (your changes must keep its
   `Pipeline::new` needle green - `new_with_progress` shares the prefix).
7. `docs/specs/tool-output-distillation.md`.

## Objective and invariants

Commit 1 (VERBATIM EXTRACTION, I1-locked) - before touching behavior, split
`crates/pipeline/src/lib.rs` (959 lines, AT the 1000 ceiling - hygiene sec.
9): move the index-stage code (sec. `Index stage`, ~382-537) verbatim to
`crates/pipeline/src/stage_index.rs` and the load-stage code (~560-959
region: the load stage + `collect_file_metrics`/pool helpers) verbatim to
`crates/pipeline/src/stage_load.rs` as `impl Pipeline` blocks in those files;
declare `mod stage_index; mod stage_load;` in `lib.rs`; move private items
with their code (raise visibility to `pub(crate)` ONLY where a moved item is
shared; never change signatures or bodies). Proof of the verbatim rule:
`lib.rs` lands at ~450-550 lines; behavior zero-change - the scoped suite is
identical pre/post, goldens UNEDITED. M27 T3/T4 then add INTO these modules,
not `lib.rs`.

Commit 2 (events + wiring):

Events (append-only enum, shape sec. 3.1 verbatim):
```rust
pub enum ProgressEvent {
    TuIndex { done: usize, total: usize },        // unchanged
    CollateralFile { done: usize, total: usize }, // unchanged
    LoadMilestone(LoadMilestone),                 // unchanged
    BootMilestone(BootMilestone),                 // NEW (stage 1)
    TuStarted { done: usize, total: usize, source: std::path::PathBuf }, // NEW (stage 2)
}

pub enum BootMilestone {
    /// Input-computation hashing progressed; total None while the
    /// single-pass walk has not bounded the file set.
    DiscoveryHashed { done: usize, total: Option<usize> },
    /// Include-scan progress of the [1/5] cold rebuild.
    IncludeScan { done: usize, total: usize },
}
```
(derive `Debug, Clone, PartialEq, Eq` like the parent enum requires).

Construction seam: `Pipeline::new_with_progress(repo_path: &Path,
workers: Option<usize>, progress: Box<dyn FnMut(&ProgressEvent) + Send>)
-> Result<Self>` - threads the callback into `build`/`compute_inputs` so the
repo hook emits `BootMilestone::DiscoveryHashed { done, total: None }` per
hashed file via `repo.discover_with_progress` (the single-pass walk has no
bounded total - always `None` on this path, per the enum doc).
`Pipeline::new`/`with_registry`/`compute_inputs` (and thus `rehash_inputs`)
delegate with no callback - BEHAVIORAL NO-OP (G1-5 exemption rests on this);
`with_progress` (builder) stays for the other callers unchanged.

Include-scan emission (`ensure_include_graph`, cold rebuild branch only; the
warm-load branch emits nothing - warm runs already jump honestly): the
crate-internal loop is the substep sequence, and per the honesty rule each
milestone fires where work really completed. Emit
`IncludeScan { done: k, total: 4 }` after each of the four real substeps:
(1) `build_dependency_graph` returned, (2) `compute_closures` returned,
(3) `generate_report_with_cycle_policies` returned, (4)
`save_include_artifacts` returned. (Planning decision, stated: there is no
per-file loop inside the pipeline crate for the scan - the BFS lives in
`crates/graph`, which V3 MUST NOT touch; substep milestones are the honest
granularity available at this seam, contract-shape preserved.)

Stage-2 emission (`rebuild_index_single_threaded`): REPLACE the per-TU
`notify(&ProgressEvent::TuIndex { done: i, total })` (line ~508) with
`notify(&ProgressEvent::TuStarted { done: i, total, source: <repo-relative
path of tu.source_path> })` - R7: exactly ONE event per TU (replace, not
add). The terminal `TuIndex { done: total, total }` at ~524 STAYS.
`source` = the TU path made repo-relative (`strip_prefix(repo.root)`,
fallback to the path as-is), verbatim - identifiability wins over line length
(shape: the message keeps the raw relative path).

Cli mapping (`ui/stage_spans.rs` + `ui.rs::stage_progress_callback`):
- new fns in `stage_spans.rs`: `discovery_hashed_percent(done: usize,
  total: Option<usize>) -> usize` = `total.map_or(stage_start_percent(1),
  |t| mapped_span_percent(stage_start_percent(1),
  DISCOVERY_HASHED_SPAN_END, done, t))` (total None => percent held at the
  stage start while the MESSAGE advances - honest, I9);
  `include_scan_percent(done, total) = mapped_span_percent(
  DISCOVERY_HASHED_SPAN_END, 19, done, total)` (stage-end frame `(1, 20)`
  stays with `run_pipeline_stages`, untouched).
- `stage_progress_callback` arms (compiler-enforced exhaustive):
  `BootMilestone(DiscoveryHashed { done, total })` -> stage 1,
  `discovery_hashed_percent`, `ARTIFACTS_PHASE`, message
  `format!("Hashing {done} files")`; `BootMilestone(IncludeScan { done,
  total })` -> stage 1, `include_scan_percent`, `ARTIFACTS_PHASE`, message
  `format!("{done}/{total} include scan")`; `TuStarted { done, total,
  source }` -> stage 2, `tu_index_percent(done, total)`, `TU_INDEX_PHASE`,
  message EXACTLY `format!("{done}/{total} TUs  {}", source.display())` -
  two spaces before the path (shape sec. 3.1 verbatim literal).
- `build_ui_payload`: `Pipeline::new(repo_path, None)?` becomes
  `Pipeline::new_with_progress(repo_path, None, Box::new(move |event|
  progress_cb_clone(&event)))`-style wiring keeping the frame-1-first
  statement (V1) intact - the V1 `first_frame_tests` needle is
  `"Pipeline::new"` WITHOUT a paren precisely so it stays green through this
  rename; do not edit that test.
- Rider (shape sec. 3.1 + open Q8 default): `index.rs::index_progress_cb`
  also handles `TuStarted { done, total, .. }` with the SAME
  `print_progress(done, total, "Indexing translation units", "TUs")` call as
  `TuIndex` (terminal bar keeps moving; path not shown in `tool index` -
  behavior-neutral, drop-without-note allowed by reviewer).

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/boot-progress-visibility-v3-journal.md` (journal; always in-set;
  create alongside this handoff)
- `crates/pipeline/src/lib.rs` (commit 1 extraction declarations + commit 2
  enum/`new_with_progress`/emission edits)
- `crates/pipeline/src/stage_index.rs`, `crates/pipeline/src/stage_load.rs`
  (NEW, commit 1 verbatim move)
- `crates/pipeline/tests/boot_events_test.rs` (NEW - pipeline-side locks)
- `crates/cli/src/commands/ui.rs` (mapping arms + `new_with_progress` call)
- `crates/cli/src/commands/ui/stage_spans.rs` (V1's file: two new mapping fns
  + `include_scan` upper const if needed)
- `crates/cli/src/commands/ui/progress_contract_tests.rs` - APPEND-ONLY:
  existing lines (incl. the top `use super::{...}` block and
  `emitted_milestones()`) byte-identical; new tests arrive as appended fns
  carrying their OWN `use` statements
- `crates/cli/src/commands/index.rs` (the one `match`-arm rider)

You MUST NOT touch: `crates/cli/src/commands/progress.rs` (DP-1/I3 frozen);
`crates/ui/**`; `crates/graph/**`; `crates/repo/**`; `crates/scheduler/**`;
`tests/golden/**`, `tests/fixtures/**`; `crates/cli/src/commands/ui/
{boot_progress_tests.rs, first_frame_tests.rs, boot_panic_tests.rs,
builder_tests.rs, target_files_tests.rs}` (green UNEDITED); `tools/**`. No
`git stash`, no branch change.

## Required tests

Pipeline side (`crates/pipeline/tests/boot_events_test.rs`; `pipeline_test.rs`
has NO event assertions - verified - but re-grep before you trust that):
1. `test_discovery_hashed_events_fire_during_construction` - tiny temp repo;
  record events through `new_with_progress`; assert >= 1 `DiscoveryHashed`,
  `done` strictly increases, last `done == discovery.files.len()`,
  `total == None`; and `Pipeline::new` (no-callback) path emits nothing
  (behavioral no-op lock).
2. `test_include_scan_events_cold_only_and_monotone` - cold: `IncludeScan`
  sequence `done` 1..=4, `total` 4, monotone, emitted only on the rebuild
  branch; a second pipeline over the same repo (warm) emits zero
  `IncludeScan`.
3. `test_tu_started_fires_exactly_once_per_tu_with_relative_path` - tiny
  2-TU temp corpus (model the minimal repo shape used by
  `crates/pipeline/tests/pipeline_test.rs`); cold index; assert per-TU event
  count == 1 (R7), each is `TuStarted` whose `source` equals the TU's
  repo-relative path, `done` runs 0.., terminal `TuIndex { done: total,
  total }` present.

Cli side (appended to `progress_contract_tests.rs`, own `use` blocks):
4. `test_discovery_and_scan_events_stay_in_stage_one_span` - mirror of
  `test_tu_index_events_stay_in_stage_two_span`: mapped percents within
  `[0, 19]`, `IncludeScan` monotone over `done`, `DiscoveryHashed` with
  `total: None` holds percent at the stage start, stage-end `(1, 20)` still
  produced by the stage-end frame (I9).
5. `test_tu_started_message_names_verbatim_relative_path` - I10: map a real
  `TuStarted { done: 3, total: 9, source: PathBuf::from("src/some TU.cpp")
  }` through `stage_progress_callback`; the produced message ==
  `"3/9 TUs  src/some TU.cpp"` (double space, verbatim relative path);
  percent == `tu_index_percent(3, 9)`.

Red protocol (valid Red, no todo!()): in commit 2's Red step, add the enum
variants + a `new_with_progress` that stores the callback but wires NOTHING
(compiles, emits nothing), stub mapping fns returning the stage start
percent; pipeline tests 1-3 fail ("0 DiscoveryHashed events"), cli test 5
fails on the message (no path in count-only message), cli test 4 passes the
`None`-holds case but fails the scan span walk - capture the failing list.
Commit 1 has NO red (verbatim move): evidence = scoped suite green BEFORE and
AFTER the move, identical test counts (capture both canonical lines).

## Hygiene (review-enforced)
Functions < 100, files < 1000 - this hop EXISTS for the ceiling: `lib.rs`
lands <= ~550, `stage_load.rs` <= ~450, `stage_index.rs` <= ~220; every
extracted fn stays < 100; pure ASCII; `python tools/hygiene_check.py` exit 0.

## Frozen locks (each with its proving command)

| Lock | Proof |
|---|---|
| stage-5 13-frame replay UNEDITED (I2) | `progress_contract_tests.rs` diff = appended-only (top block + existing fns byte-identical); replay test green in scoped cli run |
| `progress.rs` byte-identical (I3) | absent from `git diff --stat <start-rev>..HEAD` |
| m00/m18 parity UNEDITED (I1) | scoped runs `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_facts_path_parity_test` + `--test m18_cpp_plugin_parity_test` green; files absent from diff (reviewer hop runs them; implementer reports them skipped if the reviewer hop owns them) |
| goldens/fixtures frozen | diff lists touch-set only |
| V1 locks green UNEDITED (frame-1-first, panic bridge, parity) | `first_frame_tests.rs`/`boot_panic_tests.rs`/`boot_progress_tests.rs` absent from diff, green in scoped run |
| behavioral no-op of `Pipeline::new` (G1-5 exemption basis) | test 1's no-callback arm + written proof line in the journal |
| mojibake + hygiene | `encoding_guard --staged` / `hygiene_check.py` exit 0 |

Record start rev in the journal at bootstrap.

## Gates (exact commands, timeout 600000 ms each; scoped only)
Distilling entry points only (ledger row 58). Workspace-wide gates belong to
the reviewer hop.

1. Commit-1 pre/post proof: `python tools/testrun.py --distill -p
   ariadne-pipeline` (pre-move green) ... verbatim move ... same command
   (post-move green, identical counts). [1-2 invocations]
2. Commit-1 commit: `python tools/distill.py -- python tools/commit_scoped.py
   --scope crates/pipeline/src/lib.rs --scope crates/pipeline/src/stage_index.rs
   --scope crates/pipeline/src/stage_load.rs --scope
   tasks/boot-progress-visibility-v3-journal.md -m "refactor(pipeline):
   verbatim stage_index/stage_load extraction from lib.rs
   (boot-progress-visibility V3a)"`.
3. Red (commit 2): `python tools/testrun.py --distill -p ariadne-pipeline`
   AND `python tools/testrun.py --distill -p ariadne-cli` - failing lists =
   tests named above; exit 1 each. [2 invocations]
4. Green: same two commands - `0 failed`, exit 0. [2 invocations]
5. `python tools/hygiene_check.py` - exit 0.
6. `python tools/encoding_guard.py --staged` - exit 0 before committing.
7. Commit 2: `python tools/distill.py -- python tools/commit_scoped.py
   --scope crates/pipeline/src/lib.rs --scope crates/pipeline/tests/boot_events_test.rs
   --scope crates/cli/src/commands/ui.rs --scope
   crates/cli/src/commands/ui/stage_spans.rs --scope
   crates/cli/src/commands/ui/progress_contract_tests.rs --scope
   crates/cli/src/commands/index.rs --scope
   tasks/boot-progress-visibility-v3-journal.md -m "feat(pipeline,cli):
   BootMilestone + TuStarted boot events, new_with_progress, honest stage-1
   mapping (boot-progress-visibility V3b)"`.

Total implementer gate invocations: 9 (<= 10).

## Refactor and hygiene targets (named ceilings)
- `lib.rs` 959-line ceiling is the REASON for commit 1; `ui.rs` must stay
  ~700 (V1 left headroom for exactly these two match arms).
- `mapped_span_percent` reuse keeps `stage_spans.rs` <= ~180 lines.
- Function ceiling: none of the moved fns may grow; `new_with_progress`
  target <= ~30 lines (it delegates `build`).

## Acceptance (machine-checkable; implementer hop)
1. `-p ariadne-pipeline` and `-p ariadne-cli` distill runs -> `0 failed`.
2. `hygiene_check.py` exit 0; `encoding_guard.py --staged` exit 0.
3. `git show --stat` per commit lists exactly its scoped files; commit 1
   diff is a pure move (reviewer eyeballs byte-equality; `lib.rs` line count
   DOWN, no logic lines changed).
Reviewer hop (NOT yours): both scoped re-runs; m00/m18 parity + golden
diff-EMPTY (G1-2); full census via `python tools/testrun.py --distill`
(unscoped, standing errata); `python tools/distill.py -- python tools
/ui_smoke.py` PASS (boot frames from the first second are the smoke truth);
bench NOT RUN - written G1-5 exemption: `Pipeline::new` delegates a
behavioral no-op (test 1), zero diff on any stage-5 producer path, journal
carries the proof line; `encoding_guard --all`.

## Discipline clauses
- UTF-8: Write/Edit tools only; pure-ASCII content (AGENTS.md).
- Touch only the touch-set; out-of-scope breakage: report, do not fix -
  ESPECIALLY any temptation to touch `crates/graph` for finer scan
  granularity or `crates/scheduler` for the parallel path (T3 owns that).
- Bare high-volume commands are a reviewer blocker; distilling entry points
  only.
- Journal checkpoint after EACH Green gate.
- If context exhaustion looms: STOP, disk-state inventory; successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/boot-progress-visibility-v3-handoff.md --scope ... --cmd ...`.

## Report back (final message)
1. Both commit hashes + `git show --stat` each. 2. Red evidence (failing
lists). 3. Green evidence (distill headers + canonical lines + capture paths).
4. Commit-1 pre/post canonical lines (identical-count proof). 5. Deviations +
reasons. 6. Out-of-scope observations (report-only). Never paste raw output.

## Reviewer-gate reference
Adjudicated via the `review-gate` skill (commit-vs-touch-set enforced; the
two-crate scope is the declared compile-coupling exception - verify the
coupling is real: `stage_progress_callback`'s exhaustive match). Extra
checks: enum additions appended not inserted (existing variant order/lines
byte-identical); R7 event-per-TU == 1; message literal double-space; V1 test
files diff-empty and green; contract test file appended-only (byte diff of
the existing prefix); extraction commits verbatim; G1-5 exemption sentence
in the verdict.

## Sequencing note
After V1 AND V2 (compile-coupled to both). BEFORE M27 T3b - hard, on
`crates/pipeline/src/lib.rs` + the extracted stage modules (shape R6). NOT
parallel-safe with any hop touching `crates/cli/src/commands/{ui.rs,
index.rs}` or `crates/pipeline/src` (T2/T0b/T1a touch neither; V1/V2 are its
prerequisites).
