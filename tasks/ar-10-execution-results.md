# AR-10 Execution Results — reviewer acceptance pass

Reviewer: review-gate adjudication of handoff `tasks/ar-10-handoff.md`,
commit `3a26a9a`, run 2026-10-10 against HEAD `3a26a9a` (branch
shining-enthusiasm; foreign lanes dirty in tests/integration +
crates/pipeline — attributed, see verdict).

## Against the handoff's acceptance criteria

| Criterion | Result | Evidence (verbatim) |
|---|---|---|
| Gate 1 `testrun.py --distill -- -p ariadne-ui` (245 baseline + 11 locks) | MET | `256 passed / 0 failed / 0 ignored / 3 suites` |
| Gate 2 `distill.py -- python tools/ui_smoke.py` | MET | `ui_smoke: PASS` |
| Gate 3 `encoding_guard.py --all` | MET | clean (no output, exit 0) |
| Gate 4 `hygiene_check.py` zero NEW violations | MET | server.rs rows exactly: `crates/ui/src/server.rs: R2 1172 lines > 1000` and `crates/ui/src/server.rs:816: R1 fn \`api_graph_traversal\` is 161 lines (> 100)`; `api_graph_search` measured 91 lines (server.rs:654-744), no R1 row; file line count 1172 on disk, base 1155 confirmed via `git diff --numstat 70e1699 3a26a9a -- crates/ui/src/server.rs` = `26	9` (net +17) |
| Characterization locks identical pre/post | MET | All 11 `search_characterization::test_*` lines `ok` in target/testrun-capture.txt at final HEAD; each wildcard expectation re-derived independently from OLD code semantics |
| One pre-change expectation correction f11->f10 | MET | Committed lock = `f10`, independently derived from empty-query substring semantics (node 9 included); substring fallback path byte-identical pre/post, so pre-change green is implied by post-change green for that lock |
| Touch-set exact (4 paths) | MET | `git show --stat 3a26a9a`: server.rs 35 (26+/9-), main.rs +1, search_characterization.rs +279, journal +137; main.rs registration = sanctioned new-mod mechanics, AR-02 `b49ed06` precedent verified |
| Branch-order + case-sensitive regex path untouched | MET | Diff read: closure order regex->glob->substring preserved; server.rs:672-677 (`len()>1` guard, raw compile/match) outside diff |
| `build_wildcard_glob` private, compiled once per request | MET | server.rs:641 `fn` (no `pub`), single call site 683; no `Regex::new` inside `is_match` closure; residual per-element `to_lowercase` only in substring fallback (line 692), out of handoff scope and disclosed |
| Report evidence digest-shaped | MET | Journal gate lines all from distilling entry points with capture paths; zero raw dumps |

## Workspace-wide gates run by the reviewer

- Tools suites (CI-equivalent): `python -m unittest discover -s tools -p
  "*_tests.py"` -> `Ran 219 tests in 18.423s` / `OK`
- `python tools/hygiene_check.py`, `python tools/encoding_guard.py --all`,
  `python tools/ui_smoke.py` (via distill wrapper): lines above.

## Explicitly NOT run

- `python tools/testrun.py --baseline` (workspace-wide): dispatch restricted
  this review to the ui package because concurrent lanes hold
  tests/integration and crates/graph dirty mid-edit; per the Planning Law
  the workspace `--baseline` is the orchestrator's gate. The scoped
  `-p ariadne-ui` run stands as this review's package gate and is quoted
  verbatim above.
- `python tools/bench_compare.py`: not required by the AR-10 handoff
  (explicitly out of contract — UI handler path, not pipeline stage);
  journal substitutes an analytic cost note.

## Verdict

APPROVED — see `tasks/ar-10-verdict.md` for numbered findings/dispositions.
No blockers; follow-up routing: the documented (?i) simple-fold vs
full-lowercase divergence class and the pre-existing `crates/build`
`type_complexity` clippy row go to `tasks/milestone_ledger.md`
deferred-findings (orchestrator action; reviewer grant excludes the ledger).
