# AR-23 Journal - analyze incremental path consumes Pipeline::for_incremental (A6 part 2)

## Scope / outcome

`crates/cli/src/commands/analyze.rs::run_incremental_analysis_filtered` now
builds its input state through `Pipeline::for_incremental(repo_path, language)`
(landed `bcc08fc`) instead of the hand-rolled locate/discover/shaper/restrict
mirror. The mirrored helpers `restrict_and_validate_units` and
`artifacts_are_authoritative` are deleted from the cli together with the
self-admitted mirror comment; the artifact-authority decision is read from
`pipeline.incremental_view` (the ctor records it). The `incremental_backend`
pairing stays cli-side over `pipeline.build` exactly as the AR-22 journal
prescribed (backend plugin selection is the caller's registry dispatch). The
pipeline crate is UNEDITED (consumption only).

## Pre-change observable capture (PARITY-FIRST, before any edit)

Workspace HEAD `0d7ecd3` (includes AR-19 `5de3884` and AR-22 `bcc08fc`);
`crates/cli` and `crates/pipeline` clean; no analyze.rs re-anchor needed (the
handoff's ~:32-56 sequence sits at :32-56 and the mirror comment at :110-114
as-is). Binary built pre-change:
`python tools/distill.py -- cargo build --bin tool` ->
`ok (exit 0, 0 warnings emitted)`.

Capture method: real `target\debug\tool.exe analyze <repo>` runs on a FRESH
copy of `tests/fixtures/basic` (robocopy `/E /XD artifacts`, exit 1 = files
copied) under
`C:\Users\Kurt\AppData\Local\Temp\kilo\ar23-pre\basic`; every stdout digest
below is verbatim; the distiller's full capture path for each run is
`target\distill-capture.txt` (rewritten per run; the quoted text IS the
run's complete stdout).

### C1 - cold full run (authoritative)

```
Analyzing...

Reused:
    0 translation units

Reanalyzed:
    2 translation units

Cache hit rate:
    0.0%
```
(exit 0; capture: `target\distill-capture.txt`, 10 lines)

Artifact file set written under the repo copy (paths relative to
`artifacts/`, sorted):

```
artifacts/index/index.json
artifacts/index/manifest.json
artifacts/index/tu_analyses.bin
artifacts/reduced/edges.bin
artifacts/reduced/extends-edges.bin
artifacts/reduced/file-symbols.bin
artifacts/reduced/manifest.json
artifacts/reduced/sealed-symbols.bin
artifacts/reduced/symbols.bin
artifacts/reduced/tu-usage.bin
artifacts/repository/files.bin
artifacts/repository/files.json
artifacts/repository/manifest.json
artifacts/shards/12388715314208539073.json
artifacts/shards/5683510519367502559.json
```

(15 files; the two shard ids are the same deterministic FileId hashes the
fixture itself ships.)

### C2 - warm full run (second run, same copy)

```
Analyzing...

Reused:
    2 translation units

Reanalyzed:
    0 translation units

Cache hit rate:
    100.0%
```
(exit 0; capture path as above; artifact file set unchanged - same 15 files.)

### C3 - filtered full run (`--lang cpp`, same copy, warm shard store)

```
Restricted run: skipping authoritative index/reduced artifact persistence (a filtered view is never the full-repo artifact).
Analyzing...

Reused:
    2 translation units

Reanalyzed:
    0 translation units

Cache hit rate:
    100.0%
```
(exit 0; the restricted branch prints first, then the stats block; artifact
file set still the same 15 files - the filtered run persisted nothing new.)

### C4 - pristine filtered run (fresh copy, single `--lang cpp` run)

Fresh second copy at `...\kilo\ar23-filteronly-pre\basic`:

```
Restricted run: skipping authoritative index/reduced artifact persistence (a filtered view is never the full-repo artifact).
Analyzing...

Reused:
    0 translation units

Reanalyzed:
    2 translation units

Cache hit rate:
    0.0%
```
(exit 0; artifact file set after this run - exactly the shard cache, NO
authority artifacts:)

```
artifacts/shards/12388715314208539073.json
artifacts/shards/5683510519367502559.json
```

### AR-22 Finding 2 disposition - the eager `create_dir_all` is NOT a behavior delta (how I distinguished it)

The ctor eagerly `create_dir_all`s `artifacts/` while the mirror only created
it during the run. I distinguished this by capturing the observable as the
artifact FILE SET per scenario, never directory existence:

- The mirror's own `analyze()` already writes shard-cache files unconditionally
  during the run (C4 proves it: a filtered run that persists NO authority
  artifacts still ends with `artifacts/shards/*.json` present, created by the
  shard store, not by `save_artifacts`). So for every successful run of the
  basic fixture, `artifacts/` exists at end-state on BOTH sides; there is no
  scenario on this path where the pre-change run ends without the directory.
- Remaining theoretical delta: a run that errors before any write (e.g. the
  `incremental_backend` sole-plugin rejection over zero units) would leave an
  empty `artifacts/` behind post-change where the mirror wrote nothing. That
  delta is invisible to every file-set observation (an empty directory
  contributes zero files to the set) and to every stdout capture; it is a
  directory-creation side effect AR-22 already accepted as harmless. The
  parity lock therefore asserts FILE SETS and stdout, not directory
  existence, and the post-change re-runs below reproduce the identical file
  sets.

### AR-22 Finding 3 disposition - no fps-value parity claim

This switch never consumes `pipeline.fps` on the analyze path (the mirror
computed none; the analyzer's `IncrementalAnalysisResult::fingerprints` come
from the scheduler's own stage, untouched). I claim FILE-SET and stdout
parity only - not fps-value parity (the pipeline parity lock is
shared-formula shape only, per the AR-22 verdict).

## Baseline (pre-change, own stdout)

- `python tools/testrun.py --distill -- -p ariadne-cli` ->
  `117 passed / 0 failed / 8 ignored / 6 suites` (capture
  `target/testrun-capture.txt`, 194 lines). Note: the handoff text says
  baseline 118; the tree at HEAD measures 117 - the 117 is the reconciled
  truth this lane carries forward.

## The switch

- `run_incremental_analysis_filtered` now: `Pipeline::for_incremental(
  repo_path, language)?` -> authority from
  `pipeline.incremental_view.artifacts_are_authoritative()` ->
  `registry::incremental_backend` over `&pipeline.build` (the AR-22
  journaled cli-side pairing surface) -> `IncrementalAnalyzer::new(
  &pipeline.repo, &pipeline.discovery, &pipeline.build, plugin)` ->
  unchanged persist/print blocks (stdout text untouched, block order kept).
- DELETED: the mirrored locate/discover/shaper/restrict sequence, the
  `restrict_and_validate_units` and `artifacts_are_authoritative` helpers
  with the self-admitted mirror comment, the now-unneeded imports
  (`ariadne_build::BuildDiscoverer`, `Repository`), and the cli-side
  `ensure_language_registered` call (the ctor runs the guard first, error
  text locked verbatim by the pipeline parity suite).
- The three cli unit tests named after the deleted helper
  (`unrestricted_full_unit_set_is_authoritative`,
  `active_language_filter_is_never_authoritative`,
  `unit_set_narrowed_below_discovery_is_never_authoritative`) are deleted
  with it: the rule they locked is the ctor's (already locked by
  `crates/pipeline/tests/for_incremental_parity_test.rs`), and its
  analyze-path CONSEQUENCE is now locked behaviorally by the new
  integration suite (filtered run never persists authority artifacts).
- Pipeline crate UNEDITED - no ctor knob was missing (the AR-22
  consumption-only scope sufficed: repo/discovery/build + the recorded
  view are all public), so no gap finding; no STOP.

## New lock suite

`tests/integration/tests/ar23_analyze_incremental_parity_test.rs` (the
handoff's "cli tests or the existing cli test module covering analyze"
choice: the existing analyze coverage lives in `crates/cli/tests/` only as
benches and in this integration m-series tradition - the new m12-adjacent
integration lock is where a fixture-run observable belongs): 4 tests
encoding C1-C4 - reuse-line DATA (stats values the lines print), the
captured artifact FILE SETs, and the shard-only / persist-nothing shapes
of the restricted branch.

## TDD phases (digest lines, verbatim own stdout)

This is a byte-identity refactor: the handoff's PARITY-FIRST discipline
(order: capture -> lock green PRE-change -> switch -> identical expectations
post-change) governs over Red-first, disclosed here as a deliberate
deviation from the classic stub-red phase; non-vacuity is proven by
mutation instead.

- Lock vs the UNMODIFIED mirror (capture authenticity):
  `python tools/testrun.py --distill -- --test ar23_analyze_incremental_parity_test` ->
  `4 passed / 0 failed / 0 ignored / 1 suites`.
- `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`
  (after the switch) -> `ok (exit 0, 0 warnings emitted)`.
- Lock post-switch, expectations UNCHANGED: same scoped run ->
  `4 passed / 0 failed / 0 ignored / 1 suites`.
- Mutation proof (uncommitted then reverted): authority read replaced by a
  hardcoded `true` -> `2 passed / 2 failed / 0 ignored / 1 suites`, failing
  on exactly the C4 shard-only and C3 persist-nothing assertions (filtered
  runs wrote the full 15-file authority set - the semantic break the locks
  exist to catch). Reverted -> final green `4 passed / 0 failed / 0 ignored
  / 1 suites`.

## Gate commands (own stdout, verbatim digest lines)

1. `-p ariadne-cli` post-change ->
   `114 passed / 0 failed / 8 ignored / 6 suites`
   (reconciliation: 117 baseline - 3 deleted helper-named tests + 0 new in-crate;
   the 4 new locks live in the integration package).
2. `-p ariadne-pipeline` (UNEDITED - `crates/pipeline` absent from the
   commit's file list) -> `40 passed / 0 failed / 4 ignored / 9 suites`
   (exact match).
3. `--test m00_golden_baseline_test` -> `2 passed / 0 failed / 0 ignored / 1 suites`;
   `--test m00_facts_path_parity_test` -> `2 passed / 0 failed / 0 ignored / 1 suites`
   (both files UNEDITED; facts-path stability authority held).
4. New lock -> `4 passed / 0 failed / 0 ignored / 1 suites` (final green
   after mutation revert). Double-run stability on the real binary: fresh
   copy `...\kilo\ar23-post\basic` (robocopy sans artifacts), rebuilt bin
   (`cargo build --bin tool` -> `ok (exit 0, 0 warnings emitted)`):
   cold run printed the C1 block verbatim (capture
   `target\distill-capture.txt`, 10 lines), second run printed the C2
   block verbatim - reuse lines `Reused: 2 / Reanalyzed: 0 / 100.0%` prove
   the warm path unchanged; filtered run on it printed the C3 block
   verbatim (11 lines); pristine filtered copy
   `...\kilo\ar23-filteronly-post\basic` printed the C4 block verbatim.
   Post-change artifact file sets enumerated: 15-file set identical to C1
   on the double-run copy (filtered run added no files), shard-only set
   identical to C4 on the pristine filtered copy - byte-for-byte match of
   every capture scenario.
5. `python tools/encoding_guard.py --all` -> silent (exit 0).
   `python tools/hygiene_check.py` -> the only blocking rows are the two
   known `crates/ui/src/server.rs` ones (R2 1172 > 1000; R1
   `api_graph_traversal` 161 lines); all other rows warn-only R3 in foreign
   pre-existing files; zero rows from `crates/cli/src/commands/analyze.rs`,
   the new lock file, or this journal.

## Deliberate non-goals / deviations (with reasons)

- No Red-stub phase (byte-identity refactor under the handoff's
  PARITY-FIRST ordering): authenticity via pre-change green lock +
  verbatim binary captures; non-vacuity via the authority mutation above.
- Capture tooling detours (no semantic impact): `shutil` one-liner and raw
  `xcopy`/`Get-ChildItem` off the command allowlist (catch-all deny
  boilerplate each time) - fixture copies made through
  `python tools/distill.py -- robocopy ... /XD artifacts` (exit 1 = files
  copied). All captures quoted from own stdout per the shared-worktree
  capture-race ruling on the board; `target/*-capture.txt` paths cited for
  completeness only.
- The journal's em dash characters existed in tracked prose; all content
  authored here is pure ASCII (encoding_guard above).
- `ensure_language_registered` stays live in cli/registry.rs (production
  path and index.rs still call it; its unit tests stay green) - only the
  incremental path's duplicate guard call was removed.
- Touch-set honored: `crates/cli/src/commands/analyze.rs`,
  `tests/integration/tests/ar23_analyze_incremental_parity_test.rs` (new),
  this journal. No fixture/golden/theme files touched; the pre-existing
  foreign `tests/fixtures/basic/artifacts/includes/report.json` dirt left
  untouched.
