# W-2a Execution Results - ui-extraction-wizard: read-only engine-backed provider (cli)

Companion to `tasks/ui-extraction-wizard-w2a-verdict.md` (verdict: APPROVED).
Everything below was executed fresh by the reviewer in this session against
HEAD `90f4298` (landing `cb51bf8` + docs rider `90f4298`, start rev
`a9d8655`), working tree clean throughout.

## 1. Scoped verification (handoff gates 1-8 equivalents, re-run)

### 1.1 `python tools/testrun.py --distill -p ariadne-cli`

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 200 lines]
127 passed / 0 failed / 8 ignored / 7 suites
no failures
```

Matches the journal's Green canonical line exactly (baseline
`120 passed / 0 failed / 8 ignored / 7 suites` + the 7 new tests).

### 1.2 `python tools/testrun.py --distill -p ariadne-cli --lib` (per-test evidence)

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 150 lines]
121 passed / 0 failed / 0 ignored / 1 suites
```

The 7 required tests, individually green in the capture:

```
test commands::ui::extraction_reader::tests::test_reader_reports_the_epoch_of_its_generation ... ok
test commands::ui::extraction_reader::tests::test_manifest_lists_engine_sourced_includes_for_the_fixture_plan ... ok
test commands::ui::extraction_reader::tests::test_boot_payload_extraction_proposals_bytes_are_unchanged ... ok
test commands::ui::extraction_reader::tests::test_dry_run_caches_consumer_discovery_per_plan_epoch ... ok
test commands::ui::extraction_reader::tests::test_manifest_and_dry_run_leave_the_fixture_tree_identical ... ok
test commands::ui::extraction_reader::tests::test_dry_run_reports_copy_only_honesty_on_the_cpp_fixture ... ok
test commands::ui::extraction_reader::tests::test_dry_run_rewrites_match_the_cli_migration_preview ... ok
```

### 1.3 `python tools/testrun.py --distill -p ariadne-ui`

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 323 lines]
294 passed / 0 failed / 0 ignored / 3 suites
no failures
```

Unchanged from the W-2b landing line: the install call sites change nothing
for the ui crate.

### 1.4 `python tools/distill.py -- cargo check --workspace --all-targets`

```
ok (exit 0, 23 warnings emitted) -> capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt
```

A grep of the capture for the touched files returned no matches: zero
warnings in `ui.rs` or `extraction_reader.rs`; the enumerated warnings all
sit in files this hand does not touch.

### 1.5 `python tools/hygiene_check.py`

Exit 0 (`HYGIENE_OK`). Rows are R3 warn-only, all pre-existing patterns; a
grep of the run for `R1|R2` returned zero matches. Zero hygiene rows in
`extraction_reader.rs`; the ui.rs rows (`p` at :627/:762/:899, `bar` at
:754/:770) are the pre-existing swap/progress locals. Line ceilings:
`ui.rs` 986 lines, `extraction_reader.rs` 839 lines (Read-tool counts).

### 1.6 `python tools/encoding_guard.py --all`

Exit 0, silent (`ENCODING_OK`).

### 1.7 `python tools/distill.py -- python tools/theme_token_check.py --check-added --base a9d8655`

```
### generic PASS (exit 0)
capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (0 lines)
```

Zero added hex; `tools/theme_token_baseline.json` untouched.

## 2. Frozen locks (independent proofs, not taken from the journal)

- `git diff --name-only a9d8655 90f4298` = exactly the three touch-set paths
  (`crates/cli/src/commands/ui.rs`,
  `crates/cli/src/commands/ui/extraction_reader.rs`,
  `tasks/ui-extraction-wizard-w2a-journal.md`).
- `git diff --numstat a9d8655 90f4298 -- crates/ui/src/routes/api_extraction.rs
  tools/theme_token_baseline.json themes.css` = EMPTY.
- `git diff --numstat a9d8655 90f4298 -- crates/ui tests tools assets
  walkthroughs docs` = only the two cli rows.
- `tests/golden/basic/plans_plans.json` read at `27cb9de`: PLAN-001 with the
  four `*_extracted.h` target headers - the shape the W9 capture constant
  locks against.
- `tools/golden_guard.py` does not exist yet; the prose byte-check above is
  the substitute the review-gate skill names for this case.

## 3. Reverse-dependency package: `python tools/testrun.py --distill -p ariadne-integration-tests`

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 926 lines]
154 passed / 0 failed / 0 ignored / 38 suites
no failures
```

The two UNEDITED m00 locks, from the capture:

```
test facts_path_reproduces_production_plans_expensive ... ok
test facts_path_reproduces_production_plans_basic ... ok
```

`m00_facts_path_parity` stays green and its host file is diff-empty.

## 4. Workspace gates (reviewer hop)

### 4.1 Census: `python tools/testrun.py --distill`

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 2773 lines]
1465 passed / 0 failed / 13 ignored / 112 suites
no failures
```

Arithmetic: W-2b anchor `1458 passed / 0 failed / 13 ignored / 112 suites`
(`tasks/ui-extraction-wizard-w2b-execution-results.md:58`) + exactly 7 new
cli lib tests (individually verified above) = `1465 passed`. Failed 0 held;
ignored 13 held; suites 112 held. No unexplained delta.

### 4.2 Tools unit suites: `python -m unittest discover -s tools -p "*_tests.py"`

```
...........................................................................................................................................................................................................................
----------------------------------------------------------------------
Ran 219 tests in 22.313s

OK
```

Documented CI form, 219 tests OK as expected.

### 4.3 UI smoke (distiller reroute): `python tools/distill.py -- python tools/ui_smoke.py`

```
### generic PASS (exit 0)
boot+route checks clean (cold fixture, attempt 1, 1.0s)
ui_smoke: PASS
capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (2 lines)
```

### 4.4 Bench gate (OWED to this hand per the kickoff lock table)

Timing mode: `python tools/distill.py -- python tools/bench_compare.py`

```
### generic FAIL (exit 2)
ERROR parse format drift: documented phase row(s) missing from bench output (never substituting zeros):
  missing documented phase: generate_report+detect_cycles (combined)
```

Adjudication (deterministic, pre-existing, operator row):

- `tools/bench_compare.py:84` still carries the stale label
  `generate_report+detect_cycles (combined)`; the producer prints
  `generate_report+detect_cycle_groups (combined)`
  (`crates/cli/tests/payload_tail_bench.rs:389`; renamed at the M23.5 T3.2
  re-pin when the public `detect_cycles()` was deleted).
- Zero-diff proof on the producer set vs `a9d8655`: `git diff --numstat
  a9d8655 90f4298 -- crates/cli/tests/payload_tail_bench.rs
  crates/cli/tests/stage5_memory_bench.rs tools/bench_compare.py
  crates/pipeline crates/graph crates/transform crates/validate` = EMPTY.
- Compensating gate evidence, memory mode:
  `python tools/distill.py -- python tools/bench_compare.py --bench stage5-memory --runs 6`

```
### generic PASS (exit 0)
stage5-memory sample accounting: accepted 6 of 6 invocations, rejected 0 (> 8% peak-operand divergence from per-slot median)
stage5-memory gate table -- RSS/size values in MiB, wall rows seconds, memo rows counts
...
  "recorded": "2026-10-12",
  "runs": 6
}
capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (107 lines)
```

(Owed operator fix: refresh the `DOCUMENTED_PHASES` label, then re-run the
timing gate on a verified-idle machine; the stale per-machine baseline is the
separate escalated PR-2 row.)

## 5. What was NOT run, and why

- **Handoff gate 2 (the Red run):** not re-executed. Producing Red requires
  reverting the provider to the honest-absent scaffold - a tree mutation a
  reviewer must not make. The Red evidence (exit 1; the 4 named failing tests
  and their verbatim panic lines) was adjudicated from the journal together
  with the live Green state; the failures are the semantically predicted ones
  for an honest-absent scaffold (empty manifest rows, empty rewrites, unmoved
  discovery counter, copy_only false), and the three pre-mutation oracles
  (W2 tree hash, W9 byte lock, epoch/install) are consistent with passing at
  Red.
- **Handoff gate 1's baseline worktree (D1):** not re-created. The +7
  reconciliation is machine-proven instead via the W-2b census anchor and the
  per-test capture (1458 + 7 = 1465), which is the kickoff's census anchor
  rule.
- **`tools/golden_guard.py`:** does not exist yet; frozen locks were proven
  with the prose byte-check (`git diff --numstat` emptiness over the lock
  paths) that the review-gate skill names for this case.
- **Allowlist substitutions:** `python -c`, `git grep -P`, PowerShell pipes
  (`Select-Object`, `Out-Null`, `>` redirects) and bash `git status
  --porcelain` were all refused by the catch-all deny rule; each was replaced
  by an allowed equivalent (Read-tool counts, the Grep tool, the plain CI
  forms, the read-only git tool). No gate was silently skipped.

## 6. Bottom line

Every acceptance criterion of the handoff is MET with fresh evidence; the
touch-set is exact; the frozen locks are byte-identical; the engine ->
provider -> DTO includes path is real and non-fabricating; the dry-run parity
comparison is meaningful; the resume scaffold was legitimate and the landing
is coherent as one hand. Verdict: APPROVED (see
`tasks/ui-extraction-wizard-w2a-verdict.md` for the numbered findings F1-F3
and the D1-D8 adjudications).
