# Execution Results - M26 boot-progress-visibility (V1 reviewer hop)

Written by the reviewer gate on the committed tree at HEAD `f325bee`
(range `7aeee44..f325bee`, commits `79f3403` + `f325bee`). Verdict:
`tasks/boot-progress-visibility-v1-verdict.md` -> APPROVED. Every gate
below was re-run fresh by the reviewer; canonical lines are quoted
verbatim.

## Acceptance criteria - machine checks

| # | Criterion | Result | Evidence (verbatim) |
|---|---|---|---|
| A1 | Scoped `-p ariadne-cli` re-run on committed tree | PASS, exact MUST-line match | `python tools/testrun.py --distill -p ariadne-cli` -> `108 passed / 0 failed / 1 ignored / 5 suites`, exit 0 |
| A2 | Workspace census reconciles to anchor + added tests | PASS | `python tools/testrun.py --distill` -> `1226 passed / 0 failed / 4 ignored / 86 suites`, exit 0; ledger anchor `1219 passed / 0 failed / 4 ignored / 86 suites` + 7 new tests (4 `first_frame_tests` + 3 `boot_panic_tests`), suites unchanged |
| A3 | `python -m unittest discover -s tools -p "*_tests.py"` | PASS | via distill wrapper: `Ran 188 tests in 20.655s` / `OK`, exit 0 |
| A4 | `python tools/hygiene_check.py` exit 0 | PASS | distill verdict `### generic PASS (exit 0)`; output R3 warn-only, zero lines from the hop's three new files |
| A5 | `python tools/encoding_guard.py --all` exit 0 | PASS | distill verdict `### generic PASS (exit 0)`, tool printed nothing (capture 0 lines) |
| A6 | `python tools/ui_smoke.py` via distill route | PASS | `boot+route checks clean (cold fixture, attempt 1, 0.9s)` / `ui_smoke: PASS`, exit 0 |
| A7 | `git show --stat` of landing commits lists exactly the touch-set | PASS | 79f3403: ui.rs / ui/stage_spans.rs / journal; f325bee: ui.rs / ui/boot_panic_tests.rs / ui/first_frame_tests.rs / ui/stage_spans.rs / journal. `git diff --name-only 7aeee44..f325bee` = same 5 paths |

## Frozen locks

- `crates/cli/src/commands/progress.rs` zero-diff across the range
  (`git diff --stat 7aeee44..f325bee -- .../progress.rs` -> empty).
- `progress_contract_tests.rs`, `boot_progress_tests.rs`,
  `builder_tests.rs`, `target_files_tests.rs` zero-diff (same command,
  empty output).
- Oracles green UNEDITED in the workspace capture:
  `test_stage_five_milestone_sequence_is_monotonic_and_contained ... ok`,
  `test_planning_milestone_precedes_justifying_in_source ... ok`,
  `test_i2_terminal_parity_mid_boot_prefix ... ok`,
  `test_i2_terminal_parity_full_boot_run ... ok`, plus all remaining
  `boot_progress_tests` and all 7 new tests `... ok`.
- Verbatim move independently proven: git blob-vs-blob diff
  (7aeee44 ui.rs vs 79f3403 ui/stage_spans.rs, byte-safe distill capture
  `target/blobdiff-verbatim-proof.txt`) shows inside old lines 90-189 only
  `pub `-prefix pairs, the module doc header, and the authorized
  `DISCOVERY_HASHED_SPAN_END` block; everything else context = byte-equal.

## Hot-path / bench exemption

`bench_compare.py` NOT RUN under the handoff's written behavioral no-op
exemption. The exemption's premise was verified, not assumed: the range
diff touches only `crates/cli/src/commands/ui.rs`, the three new cli
files, and the journal - no stage-5 producer path, no pipeline/repo/graph
diff, `run_pipeline_stages` frames byte-identical minus the relocated
start emission, `catch_unwind` wraps only the one-shot boot call.

## Not run / deviations in gate forms (explicit)

- `python tools/testrun.py --baseline FILE`: NOT RUN - `--baseline`
  requires a baseline FILE and none exists in this repo; the reviewer
  reconciled the canonical line against the ledger's pinned anchor
  `1219/0/4/86` instead (A2). Plain non-distill scoped form attempted first
  and crashed on the known cp1252 echo defect
  (`UnicodeEncodeError: 'charmap' codec can't encode characters in
  position 2017-2036` in `_echo`) - deferred-row family, not implementer
  fault; the `--distill` form produced the canonical line.
- `bench_compare.py`: not run (exemption above).
- Refused reviewer conveniences (boilerplate
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
  multi-line `python -c`, inline python with nested `git` subprocess,
  inline python `open()` file I/O, PowerShell compound forms
  (`Measure-Object`, `if ($?)` exit-code checks). None blocked a required
  gate; exit codes were proven via distill verdict lines instead. Verdict
  finding 1.
- Implementer-stage gate `encoding_guard --staged` is superseded here by
  the stronger `--all` at HEAD (A5).

---

# Execution Results - M26 boot-progress-visibility (V2 reviewer hop)

Written by the reviewer gate on the committed tree at HEAD `5c81881`
(range `f325bee..5c81881`, commits `3f8be39` + `5c81881`). Verdict:
`tasks/boot-progress-visibility-v2-verdict.md` -> APPROVED. Every gate
below re-run fresh by the reviewer; canonical lines quoted verbatim.

## Acceptance criteria - machine checks

| # | Criterion | Result | Evidence (verbatim) |
|---|---|---|---|
| B1 | Scoped `-p ariadne-repo` re-run on committed tree | PASS, exact MUST-line match | `python tools/testrun.py --distill -p ariadne-repo` -> `15 passed / 0 failed / 0 ignored / 2 suites`, exit 0 |
| B2 | Workspace census reconciles to V1 anchor + added tests | PASS | instructed form `python tools/testrun.py --workspace --no-fail-fast` REFUSED (`usage error: unknown flag: --no-fail-fast` - testrun.py flag surface does not carry it); V1-sanctioned form `python tools/testrun.py --distill` -> `1230 passed / 0 failed / 4 ignored / 86 suites`, exit 0 = V1 anchor `1226/0/4/86` @ f325bee **+ 4** new tests (identity, monotone-counts, mixed-tree, sort-independent), suites/ignored unchanged. Dispatch's expected `1229 (1226+3)` undercounted the shipped tests by one (verdict finding 2); workspace number is right |
| B3 | `python -m unittest discover -s tools -p "*_tests.py"` | PASS | `Ran 188 tests in 20.463s` / `OK` (unpiped; piped form was allowlist-refused - compound family) |
| B4 | `python tools/hygiene_check.py` exit 0 | PASS | exit 0; output entirely pre-existing R3 warn-only rows; zero rows in `crates/repo/src/discovery.rs`; lib.rs:92-96 `res` rows predate the hop (diff hunk covers lines 29-44 only) |
| B5 | `python tools/encoding_guard.py --all` exit 0 | PASS | exit 0, no output |
| B6 | `python tools/ui_smoke.py` via distill | PASS | `### generic PASS (exit 0)` / `boot+route checks clean (cold fixture, attempt 1, 0.9s)` / `ui_smoke: PASS`; capture `target/distill-capture.txt` |
| B7 | Commit-vs-touch-set | PASS | `git diff --name-only f325bee..HEAD` = exactly `crates/repo/src/discovery.rs`, `crates/repo/src/lib.rs`, `tasks/boot-progress-visibility-v2-journal.md`; per-commit stats: 3f8be39 (+182, +10/-1, +66), 5c81881 (journal +16) |
| B8 | `discover()` behavior byte-identical (delegation) | PASS | code-level: `discover` body = no-op delegation; skip/sort rules zero-deletion in diff; hook fires only post-`compute_file_hash` success inside the walk loop before the untouched path sort; identity test 1 locks record+summary equality; pre-existing repo tests unedited (census +4 only) |
| B9 | Public API additive-only | PASS | `lib.rs:7` re-export line unchanged; new fn is `pub(crate)` at discovery.rs:34, not re-exported; `discover_with_progress` is the only new pub item, signature verbatim per contract |
| B10 | Call-site census | PASS | grep: `discover_with_progress` referenced only inside `crates/repo` (lib.rs delegation + repo tests); no external consumer switched - boot path unchanged, grounding the bench exemption |

## Red protocol

Journal: `12 passed / 3 failed / 0 ignored / 1 suites`, exit 1; failing
list = exactly tests 2-4 (hook count `left: 0` on the discarding stub),
test 1 green - matches handoff prediction; arithmetic self-consistent
(12+3=15=green). Suite 1->2 is cargo fail-fast skipping the doctest target
after the lib target fails, not drift. Red capture overwritten by later
runs; failing list is a named claim (verdict finding 3, minor).

## Frozen locks

- `git diff --name-only f325bee..HEAD` outside touch-set: none. Goldens,
  fixtures, `crates/repo/src/types.rs`, `crates/pipeline/**`, `tools/**`
  zero-diff across the range.
- Mojibake gate at HEAD via `--all` (B5) supersedes the implementer-stage
  `--staged`.

## Bench exemption (verified premise)

`bench_compare.py` NOT RUN under the ledger M26 row's written
behavioral-no-op exemption ("NO bench gate ... PR-2 bench refresh belongs
to M27 only"). Premise verified independently (B10): `discover_with_progress`
has zero callers outside `crates/repo`; the boot/discovery path executes the
same code as at f325bee, hook closure no-op aside.

## Not run / refused (explicit)

- `python tools/testrun.py --baseline FILE`: NOT RUN - no test-count
  baseline FILE exists in the workspace (only `theme_token_baseline.json` /
  `bench_baseline.json`); census reconciled against the V1 verdict anchor
  instead (B2), same as the V1 hop.
- `bench_compare.py`: not run (written exemption, premise verified).
- Refused reviewer forms (boilerplate
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
  piped `unittest discover ... | Select-Object` and `hygiene_check.py;
  echo $LASTEXITCODE` compound forms - both succeeded run plain; the
  instructed `--no-fail-fast` census flag (tool-level usage error, not a
  permission). None blocked a required gate; exit codes proven via distill
  verdict lines and clean tool runs.

---

# Execution Results - M26 boot-progress-visibility (V3 reviewer hop, milestone-closing)

Written by the reviewer gate on the committed tree at HEAD `1e81d7e`
(range `5c81881..1e81d7e`, commits `1819b8f` + `8eedb05` + `1e81d7e`).
Verdict: `tasks/boot-progress-visibility-v3-verdict.md` -> APPROVED.
Every gate below re-run fresh by this reviewer; canonical lines verbatim.

## Acceptance criteria - machine checks

| # | Criterion | Result | Evidence (verbatim) |
|---|---|---|---|
| C1 | Scoped `-p ariadne-pipeline` re-run | PASS | `python tools/testrun.py --distill -p ariadne-pipeline` -> `### cargo test PASS (exit 0)` / `25 passed / 0 failed / 2 ignored / 5 suites`; = commit-1 post-move baseline `22/0/2/4` + 3 new `boot_events_test` locks |
| C2 | Scoped `-p ariadne-cli` re-run | PASS | `python tools/testrun.py --distill -p ariadne-cli` -> `### cargo test PASS (exit 0)` / `110 passed / 0 failed / 1 ignored / 5 suites`; V2-era 108 + 2 appended contract tests, frozen 108 green UNEDITED |
| C3 | Commit-1 pre/post identical-count proof | ACCEPTED | Implementer captured pre/post `22 passed / 0 failed / 2 ignored / 4 suites` each; post-state is today's `25/0/2/5` minus the 3 boot locks (arithmetic exact); pre-move state is unrecoverable at HEAD, so this leg rests on the journal's paired canonical lines + byte-level verbatim proof (below), which is strictly stronger than count equality |
| C4 | Commit-vs-touch-set | PASS | `git diff --name-only 5c81881..HEAD` = exactly 9 paths, all in the declared touch-set: index.rs(6/1), ui.rs(26/5), progress_contract_tests.rs(137/0), stage_spans.rs(28/1), lib.rs(130/455), stage_index.rs(288/0), stage_load.rs(210/0), boot_events_test.rs(244/0), journal(268/0) |
| C5 | m00 parity UNEDITED | PASS | `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_facts_path_parity_test` -> `2 passed / 0 failed / 0 ignored / 1 suites`; test files absent from range diff (`tests/**` zero paths) |
| C6 | m18 parity UNEDITED | PASS | same form, `--test m18_cpp_plugin_parity_test` -> `23 passed / 0 failed / 0 ignored / 1 suites`; absent from range diff |
| C7 | Full workspace census | PASS | `python tools/testrun.py --distill` (unscoped; `--no-fail-fast` does not exist; literal `--baseline` unrunnable - ledger errata row, 4th encounter) -> `1235 passed / 0 failed / 4 ignored / 87 suites`, exit 0; reconciles exactly to V2 anchor `1230/0/4/86` + 3 pipeline + 2 cli = `1235`, suites 86 + 1 new test binary = 87, ignored 4 held |
| C8 | tools unit suites | PASS | `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 188 tests in 24.728s` / `OK` |
| C9 | `ui_smoke` via distill | PASS | `python tools/distill.py -- python tools/ui_smoke.py` -> `### generic PASS (exit 0)` / `boot+route checks clean (cold fixture, attempt 1, 0.7s)` / `ui_smoke: PASS` |
| C10 | `hygiene_check.py` exit 0 | PASS | run plain, exit 0; R3 warn-only, every repo row pre-existing; `ui.rs` warn rows at lines 535/657/665/673/771 are outside every changed hunk (5, 18-23, 210-230, 244-246); zero R3 rows in any file the hop authored (the only rows belonging to this hop's session were reviewer scratch scripts, queued for purge) |
| C11 | `encoding_guard.py --all` exit 0 | PASS | silent, exit 0 (supersedes the implementer-stage `--staged` rows) |
| C12 | bench | NOT RUN - exemption upheld | premise re-verified at this hop, not inherited: (a) `Pipeline::new` no-op arm green (C1, test 1); (b) the ENTIRE stage-5 producer module `stage_load.rs` is byte-identical from start-rev code to HEAD (single sanctioned `pub(crate)` line + module header; zero change in commits 2-3), planner/justification code untouched by the ui.rs hunks; (c) all boot-path changes are callback-gated or event-count-preserving (R7 replace). Running against the dated stale baseline (ledger escalated row: flaps on zero-Rust-diff trees) would produce gate-noise, not truth; refresh is M27 T1b / operator duty on VERIFIED-IDLE |

## Verbatim-extraction proof (reviewer-owned, byte-level)

Independent script (git cat-file blobs, no PowerShell text pipes; scratch
deleted after run): all five declared chunks - `382-537`, `779-798`,
`822-891` -> `stage_index.rs`; `617-741`, `893-952` -> `stage_load.rs` -
byte-identical contiguous substrings of both the start-rev `lib.rs` and the
landing modules. The `collect_file_metrics` region differs from the start
rev by EXACTLY ONE line: `fn` -> `pub(crate) fn` (old:920 vs
stage_load:178) - the sanctioned visibility raise. Opcode-level commit-1
`lib.rs` diff: 430 deletions inside the declared chunks; 22 stray deletions
ALL inside the start-rev import block (imports travelling with moved items)
plus 5 boundary blanks/section-banner; 30 additions = re-pruned import
subsets + the two `mod` declarations + `#[cfg(test)]` companion imports /
`collect_file_metrics` re-export. Zero moved-code change. Line counts:
`lib.rs` 959 -> 537 (commit 1) -> 634 (HEAD); `stage_index.rs` 276 -> 288;
`stage_load.rs` 210; `stage_spans.rs` 117 -> 144; `ui.rs` 853; contract
tests 305 -> 442.

## Frozen locks (byte-level verdicts)

- `progress.rs`: byte-identical `5c81881 -> HEAD` (blob equality).
- `progress_contract_tests.rs`: APPEND-ONLY proven at byte level - HEAD
  blob starts with the complete start-rev blob; difflib opcodes: exactly
  one `insert old[305:305] new[305:442]`, zero delete/replace ops; existing
  305 lines incl. top `use super::{...}` block and `emitted_milestones()`
  byte-identical; appended fns carry own `use` blocks; stage-5 13-frame +
  I2 oracles green UNEDITED inside C2.
- Enum append-only: start-rev `ProgressEvent`..`type ProgressCb` block
  (21 lines) all byte-identical, in order, at HEAD; the only ops are
  inserts - `BootMilestone`/`TuStarted` after the three existing variants,
  `BootMilestone` enum + derive `Debug, Clone, PartialEq, Eq` before
  `type ProgressCb`.
- `stage_spans.rs` single deletion across the hop: exactly
  `#[allow(dead_code)] // first consumer is the V3 handoff, by design`
  (allow lifted when its consumer landed); all else inserts.
- `stage_load.rs` byte-identical `1819b8f -> HEAD` (untouched by commits
  2-3).
- Zero-diff ranges confirmed absent from `git diff --name-only`:
  `crates/{ui,graph,repo,scheduler}/**`, `tests/golden/**`,
  `tests/fixtures/**`, `tools/**`, and the five frozen V1 test modules
  (`first_frame_tests` - `Pipeline::new` needle green via the
  `new_with_progress` substring, `boot_panic_tests`, `boot_progress_tests`,
  `builder_tests`, `target_files_tests`) plus `ui/tests.rs`.
- R7/message literals: per-TU `TuStarted` REPLACES the per-TU `TuIndex`
  notify (single call site, `strip_prefix(&repo_root)` fallback as-is);
  terminal `TuIndex { done: total, total }` stays; ui.rs message
  `format!("{done}/{total} TUs  {}", source.display())` (double space,
  contract test 5 locks `"3/9 TUs  src/some TU.cpp"` byte-exactly);
  legacy count-only `TuIndex` arm intact for the terminal frame;
  index.rs rider keeps both locked literals.

## Red-evidence adjudication (transport-loss resume)

Journal states both Red gates were RE-RUN on the identical working tree at
the `checkpoint_guard.py status` resume, before any Green edit. Capture
strings match the committed assert-message templates exactly - "got 0 for 2
TUs" vs `"got {} for {tu_count} TUs"`, `left: "3/9 TUs"` vs the committed
count-only message shape and test-5 assertion, "bounded hashing ends at
the D split point [0, 10]" verbatim at contract test line 328, the
"0 DiscoveryHashed" zero-count arms - and cli Red arithmetic reconciles
(108+2 = 110 green). Residual limitation stated openly: the pipeline Red
fail-mode digest (`12/3/0/2`) cannot be recomputed post-Green (shared
capture overwritten; source not re-runnable without forbidden edits);
name-level capture-grounding stands.

## Refused gate forms (verbatim boilerplate
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`)

Chained `git log; echo; git diff; echo; git show` and chained
`git show; echo; git show` (per-commit stats then ran fine as three
individual `git show` calls); direct `Write` to the temp dir (agent edit
grant is `tasks/**` + `.kilo/plans/**`; scratch relocated);
`git ls-files` / `git status --porcelain` (census taken from
`checkpoint_guard.py status` instead); `Remove-Item` of scratch scripts
(add to operator purge queue, M25 precedent). Script EXECUTION was routed
through `python tools/distill.py -- python tasks/...` without issue.
`python tools/testrun.py
--baseline` (FILE flag) NOT RUN per the standing ledger errata row;
`--no-fail-fast` confirmed nonexistent (not attempted per errata). None
blocked a required gate; every gate above ran through a permitted form.
