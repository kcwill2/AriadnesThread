# AR-20b Execution Results - reviewer acceptance pass (fresh, own stdout)

Adjudication: `tasks/ar-20b-verdict.md` - APPROVED. Landing `dcfc813`,
addendum `f49faeb`, base `4dc7237`, AR-20a anchor `0861db0`, HEAD `f49faeb`.
Worktree state at review: dirty tracked files exclusively `crates/ui`
(AR-15 implementer in flight - foreign to this handoff).

## Acceptance criteria vs measured evidence

| # | Criterion | Evidence (verbatim) | Result |
|---|---|---|---|
| 1 | m17 suite = 8-test enumeration, green-once | `8 passed / 0 failed / 0 ignored / 1 suites` | PASS |
| 2 | Guard never RED in history (invariant 5) | `git log -S plugin_crates_transitive_tree --oneline -- tests/` -> EXACTLY ONE line: `dcfc813 feat(lang-cpp): consume ariadne-worker-core directly; drop scheduler dep; land transitive guard GREEN (AR-20b)`; whole-repo form adds only docs-only `d63c0d0` (numstat `138 0 tasks/ar-20-handoff-journal.md`; m17 stat EMPTY there) and the journal addendum `f49faeb` | PASS |
| 3 | 8-test enumeration re-derived | exactly 8 `#[test]` attrs at :44/:64/:238/:268/:402/:422 + guard :481 + self-test :512 | PASS |
| 4 | Self-test load-bearing, non-vacuous | planted edge walks the same `reachable_local_crate_names`/`collect_reachable_local_deps` descent (:583-588); dev-skip asserted (:541-544); mutation-trail message reproduces the committed format string exactly (`planted scheduler edge not detected through closure descent: ["planted-intermediate"]`) | PASS |
| 5 | lang-cpp suite maintained, zero test edits | `153 passed / 0 failed / 0 ignored / 2 suites` | PASS |
| 6 | m18 byte-parity UNEDITED | `23 passed / 0 failed / 0 ignored / 1 suites`; landing commit touches ONLY m17 under `tests/integration/tests/` | PASS |
| 7 | m18_persistence UNEDITED (shim lock :226) | `7 passed / 0 failed / 0 ignored / 1 suites` | PASS |
| 8 | m00 golden UNEDITED | `4 passed / 0 failed / 0 ignored / 2 suites`; `tests/golden` absent from both commits | PASS |
| 9 | End-state cone free of the six (invariant 6) | `python tools/distill.py -- cargo tree -p ariadne-lang-cpp` -> `ok (exit 0, 0 warnings emitted)`; own-capture grep `ariadne-scheduler|ariadne-report|ariadne-cluster|ariadne-graph|ariadne-transform|ariadne-validate` -> `No files found`; edges exactly build/lang/lang-api/repo/worker-core (:50)/index (:53 under worker-core) | PASS |
| 10 | Scheduler shims conserved | `-p ariadne-scheduler` -> `26 passed / 0 failed / 0 ignored / 2 suites`; rider `-p ariadne-worker-core` -> `8 passed / 0 failed / 0 ignored / 2 suites`; 26 + 8 = 34 unchanged | PASS |
| 11 | Switch completeness: 3 backend.rs sites, dep gone, note rewritten | landing numstat `3 3 crates/lang-cpp/src/backend.rs` (module-doc :19, import :24, module-qualified call :180, `_fingerprint` kept); full Cargo.toml read: no scheduler in any table, dev-deps serde_json/tempfile only; note block byte-matches the handoff's ASCII block line-by-line; `Cargo.lock 1/1` scheduler->worker-core edge only | PASS |
| 12 | No leftover `use ariadne_scheduler` in lang-cpp | grep `ariadne_scheduler|ariadne-scheduler` over `crates/lang-cpp`: only pre-existing module-doc HISTORY prose (backend.rs:3, lib.rs:8); zero code references | PASS |
| 13 | worker-core/scheduler byte-frozen vs 0861db0 | `git diff --stat "0861db0..HEAD" -- crates/worker-core crates/scheduler` -> empty output | PASS |
| 14 | lang-api / ar-20 journal frozen | `git show "dcfc813" -- crates/lang-api ... tasks/ar-20-handoff-journal.md` -> empty; ar-20 journal not in dirty list (porcelain-empty) | PASS |
| 15 | Commit-vs-touch-set | dcfc813 paths: Cargo.lock 1/1, crates/lang-cpp/Cargo.toml 7/6, crates/lang-cpp/src/backend.rs 3/3, tasks/ar-20b-handoff-journal.md 182/0, tests/integration/tests/m17_plugin_seam_test.rs 207/0 - ALL inside the declared touch-set; f49faeb: journal 24/0 only (admitted precedent) | PASS |
| 16 | Hygiene exit 0, zero blocking | `hygiene_check.py` full output R3 warn-only; zero R1/R2/ERROR rows; no AR-20b file appears (D1 `concat!` reshape verified effective) | PASS (exit-code capture: see gap G2) |
| 17 | Encoding guard | `python tools/encoding_guard.py --all` -> silent (no violations across all tracked content) | PASS (exit-code capture: see gap G2) |
| 18 | Tools unit suites (CI-equivalent) | `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 219 tests in 20.081s` / `OK` | PASS |
| 19 | Deviations D1/D2/D3 adjudicated on merits | D1 accepted (scanner misparse real; semantics unchanged; tooling row = verdict F2); D2 accepted (my fresh `--all` independently verifies the committed bytes - the substantive oracle); D3 accepted (commit header matches contract `-m` verbatim) | PASS |

## Explicitly NOT run (and why)

- G1. `python tools/testrun.py --baseline` (workspace-wide) and the handoff's
  gate 1 `cargo check --workspace --all-targets`, plus `ui_smoke`/
  `bench_compare`: NOT run. The tracked tree carries AR-15's in-flight
  `crates/ui` edits (dirty `state.rs`, `api_graph.rs`, `api_payloads.rs`,
  `ui_test/main.rs` + untracked `payload_etag.rs`); a workspace-wide run would
  attribute foreign WIP to this milestone. The review-gate skill assigns the
  workspace final gate to the orchestrator; the dispatch states ui-class
  gates are not in this contract. This lane's compile-green is established by
  gates 1-5 building every touched crate and suite.
- G2. Verbatim exit codes for `hygiene_check.py` / `encoding_guard.py --all`:
  not capturable in this reviewer sandbox - `$LASTEXITCODE`, `cmd /c`, and
  `python -c` forms are refused with the catch-all deny boilerplate
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`
  (a non-allowlist configuration fact, not a missing gate: both tools RAN,
  outputs quoted above; pass signatures - zero blocking rows / silence - are
  the same evidence the tools' CI contracts consume).
- No mutation re-run of the self-test: reviewer cannot modify sources; the
  committed restored state was audited line-by-line (descent present at
  :583-588) and the journal's mutation message is byte-consistent with the
  committed assert format string.

## Chain effects on APPROVAL

A4 CLOSED machine-locked; AR-26 reparent hold LIFTED; lang queue drains AR-24
first. Verdict finding F2 (hygiene scanner misparse) to be banked by the
orchestrator in `tasks/milestone_ledger.md` deferred-findings.
