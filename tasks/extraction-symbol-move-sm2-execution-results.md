# SM-2 Execution Results - extraction-symbol-move (reviewer gate, 2026-10-24)

Every line below is this reviewer's own fresh run against HEAD `fba3c90`
(tree clean except the pre-existing `.vsix` stray, per
`python tools/checkpoint_guard.py status`). Canonical lines quoted verbatim.
Handoff: `tasks/extraction-symbol-move-sm2-handoff.md`; verdict:
`tasks/extraction-symbol-move-sm2-verdict.md` (APPROVED).

## Reflog ground truth (anti-phantom duty)

`.git/logs/HEAD` (direct Read; the `git reflog` command itself is refused by
the bash allow-list catch-all): 442 total lines; lines 440/441/442 carry
`b993600 -> 0db2680`, `0db2680 -> f6c92fe`, `f6c92fe -> fba3c90` - duty-1,
landing, rider all exist, rider = HEAD. Full verbatim lines in the verdict.

## Handoff gate table, re-run fresh

| # | Handoff gate | Result (verbatim) | Exit |
|---|---|---|---|
| 1/2 | Red witness | Transient pre-commit state, not re-runnable; journal's ten named failures + `extraction-symbol-move SM-2:` expect prefix adjudicated falsifiable (verdict section "Red witness adjudication"); baseline 205 + 10 = my 215 | n/a |
| 3 | `python tools/testrun.py --distill -p ariadne-lang-cpp` | `215 passed / 0 failed / 0 ignored / 2 suites` | 0 |
| 4 | `python tools/testrun.py --distill -p ariadne-lang-scala` | `257 passed / 0 failed / 0 ignored / 2 suites` | 0 |
| 5 | `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` | `23 passed / 0 failed / 0 ignored / 1 suites` | 0 |
| 6 | `python tools/testrun.py --distill` (substitute for gate 6) | `1849 passed / 0 failed / 13 ignored / 119 suites` | 0 |
| 7 | `python tools/distill.py -- python tools/bench_compare.py` | `bench compare: 3 run(s), no regressions past 1.25x baseline` (fresh recording, full 17-line capture read, ZERO WARN lines) | 0 |
| 8 | `python tools/hygiene_check.py` | zero touch-set rows, zero R1/R2; exit via `python tools/distill.py -- python tools/hygiene_check.py` -> `### generic PASS (exit 0)` (173-line capture) | 0 |
| 9 | `python tools/encoding_guard.py --all` | silent; exit via `python tools/distill.py -- python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)`, capture 0 lines | 0 |

- Gate 3 capture (291 lines) names all ten `move_retention::tests::move_out_*
  ... ok` (lines 44-54) and all 16 `dependency_rewriter::tests::* ... ok`
  (lines 14-30) - the 215 = 205 baseline + the ten named tests, pre-existing
  tests green UNEDITED.
- Gate 6 reconciliation: anchor `1837 passed / 0 failed / 13 ignored /
  119 suites @ b201aa6`; SM-1 landing of record `1839`; +10 attributed to
  the ten SM-2 tests (census capture names them `... ok`, lines 1517/1529
  sampled); 0 failed / 13 ignored / 119 suites held; matches the ledger
  census plan (SM-2 +10; milestone expectation `1853`).

## Baseline form (F9)

`python tools/testrun.py --baseline` -> refused verbatim:
`usage error: missing value for --baseline` (the superseded FILE form; the
`--distill` substitute above governs).

## Workspace acceptance gates (reviewer duty, not carried by the handoff)

- Tools suites: `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 296 tests in 27.049s` / `OK`.
- UI runtime smoke (sanctioned wrapper; direct invocation refused by the
  allow-list in prior hops): `python tools/distill.py -- python tools/ui_smoke.py`
  -> `### generic PASS (exit 0)` /
  `boot+route checks clean (cold fixture, attempt 1, 1.1s)` / `ui_smoke: PASS`.
- Bench: fresh recording run (no exemption route taken) - this hand adds
  lang-cpp runtime code, so I judged the exemption unavailable and recorded
  fresh. Result line above.

## Touch-set and frozen locks

- `git diff --name-only 0db2680 fba3c90` = exactly the four declared paths
  (dependency_rewriter.rs, lib.rs, move_retention.rs, the journal).
- Landing numstat: `16`, `1`, `648`, `210` insertions; rider journal `+19/-1`.
- Absent from the range diff (= byte-identical): `crates/lang/**`,
  `crates/lang-scala/**`, `symbol_locator.rs`, `tokenizer.rs`, `parser.rs`,
  `source_extractor.rs`, `crates/validate/**`, `crates/cli/**`,
  `crates/ui/**`, `tests/**`, goldens, fixtures, themes, `tools/**`,
  `Cargo.lock`.
- Frozen rewriter fns + `consumer_rewrite_spec` + 16 existing tests: single
  hunk `@@ -111,6 +111,22 @@` in `dependency_rewriter.rs`, zero deletions.

## Hygiene detail

- `move_retention.rs` 648 lines (< 1000); longest function
  `render_marked_source` ~38 lines (< 100); no banned names or
  single-letter locals in any touch-set file (hygiene output rows: zero).
- Pure-ASCII independent check: regex `[^\x00-\x7F]` over
  `move_retention.rs` -> no matches.

## Explicitly NOT run / not re-verifiable (declared)

- The gate-2 Red state: transient pre-commit state; the capture file was
  overwritten by later runs. Adjudicated by falsifiability logic (verdict),
  not re-run.
- `python tools/theme_token_check.py --check-added --base`: not in the SM-2
  gate table (no UI asset touched; zero hex added per hygiene + encoding
  gates) - owed at the milestone closeout, not this hand.
- `python tools/golden_guard.py`: not present (prose byte-check per the
  handoff's lock section was used: range-diff absence proofs above).
- Red-era first-assert panic capture: distiller retained five of ten panic
  lines at journal time; the ten FAILURES names are the pin, not the panic
  text.
- Refused during this hop (catch-all boilerplate, zero effect, sanctioned
  substitutes used): raw `git reflog ...`, compound `git log ... ; Write-Output`
  probe, inline `python -c` reflog read. All exit-code proofs went through
  `distill.py`; the reflog went through a direct Read of `.git/logs/HEAD`.

## State of record

HEAD `fba3c90`; census `1849 passed / 0 failed / 13 ignored / 119 suites`;
tools `Ran 296 tests in 27.049s` / `OK`; bench `bench compare: 3 run(s), no
regressions past 1.25x baseline`; smoke `ui_smoke: PASS`; hygiene exit 0;
encoding exit 0. Verdict: APPROVED (M1-M5, no blockers). This verdict and
this file ride SM-3's duty-1 (uncommitted by design).
