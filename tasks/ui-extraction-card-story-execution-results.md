# Execution results - ui-extraction-card-story (reviewer-hop workspace gates)

Milestone acceptance run fresh at HEAD `ef9d170` (T1 landing commit,
parent/start rev `4900e04`). All lines quoted verbatim from this session's
runs unless marked journal-attested.

## Kickoff/shape acceptance gate

| Criterion | Result | Evidence line (verbatim) |
|---|---|---|
| Card tells the five-beat story, locked by new tests | MET | scoped run `197 passed / 0 failed / 0 ignored / 3 suites`; the 9 locks in `extraction_card_story.rs` are in that count and match the handoff's required-test list one-for-one |
| All pre-existing card/section/help/payload tests green UNEDITED | MET | name-only diff shows zero test-file edits besides the new module + 1 `mod` line; scoped suite exit 0 |
| Display-layer only; DTO/producers/assets-locked byte-identical | MET | `git diff --stat 4900e04..HEAD -- crates/pipeline crates/cli crates/cluster crates/graph crates/ui/src tools tests/fixtures` -> empty; `api.js`/`preview.js`/`graph.js`/`main.js`/`themes.css`/`theme.js` absent from diff |
| Zero added hex; tokens from declared set only | MET | `python tools/theme_token_check.py --check-added --base 4900e04` silent exit 0 |
| Copy honest vs producers | MET | `planner.rs:659`/`:664`, `ui.rs:534` spot-read; help `<li>` claims match |
| Hygiene + encoding | MET | `hygiene_check.py` exit 0 (R3 warn-only legacy rows, none in touch-set files); `encoding_guard.py --all` silent exit 0 |
| UI cold-boot visual sanity | MET | `boot+route checks clean (cold fixture, attempt 1, 1.0s)` + `ui_smoke: PASS`; card height inside the 240px scroller remains the human eyeball step (no height-locked test exists, per shape risks) |

## Reviewer-hop gate lines (verbatim)

1. Scoped ui: `python tools/testrun.py --distill -p ariadne-ui` ->
   `### cargo test PASS (exit 0)` / `197 passed / 0 failed / 0 ignored / 3 suites`
   (capture `target/testrun-capture.txt`) -- reproduces the implementer's
   reported Green digest exactly.
2. Census: `python tools/testrun.py --distill` (unscoped; G7 `--baseline`-file
   errata stands -- no baseline committed, handoff's prescribed form used) ->
   `### cargo test PASS (exit 0)` /
   `1181 passed / 0 failed / 4 ignored / 85 suites`
   Reconciliation: 1168 prior census @ `1bc50b8` + 4 `theme_dark_grey_palette`
   tests (4 `#[test]`, landed `41ec354`) + 9 new = 1181; suites unchanged at
   85; ignored unchanged at 4. Environment note: the first two attempts against
   the normal target died as `### cargo test DRIFT (exit 2)` from
   `error: failed to remove file ...target\debug\tool.exe ... Access is denied.
   (os error 5)` (a live process holds the binary); the clean census was run
   with `CARGO_TARGET_DIR` redirected to a temp dir -- same workspace code,
   isolated build dir. See verdict finding 2.
3. Tools suites: `python -m unittest discover -s tools -p "*_tests.py"` ->
   `Ran 188 tests in 20.991s` + `OK`.
4. Smoke: `python tools/distill.py -- python tools/ui_smoke.py` ->
   `### generic PASS (exit 0)` / `boot+route checks clean (cold fixture,
   attempt 1, 1.0s)` / `ui_smoke: PASS`
   (direct `python tools/ui_smoke.py` was refused by the permission allowlist:
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`;
   distilling wrapper is the sanctioned route -- verdict finding 1).
 5. Bench [SUPERSEDED 2026-10-04 by the post-lock clean PASS -- see Addendum;
    text kept below as audit trail]: NOT RUN as a PASS -- sanctioned exemption applied per handoff gate 5
   and the ui-progress-parity precedent: bench_compare not run, assets-only
   change, pipeline byte-identical (empty diff over all producer crates,
   quoted above). Attempt on record: direct invocation refused (same allowlist
   boilerplate); via distill it died mid-run with
   `ERROR cargo bench run 2 of 3 failed (exit 101): error: failed to remove
   file C:\Users\Kurt\Code\AriadnesThread\target\debug\tool.exe` -- the same
   environment lock, not a performance signal.
6. Token drift: `python tools/theme_token_check.py --check-added --base 4900e04`
   -> no output, exit 0; `themes.css` absent from the diff.
7. Encoding: `python tools/encoding_guard.py --all` -> no output, exit 0.
8. Hygiene: `python tools/hygiene_check.py` -> R3 warn-only legacy listing
   (no R1/R2 lines, none in touch-set files), exit 0 (no nonzero exit reported
   by the harness; exit-code echo attempt was itself allowlist-refused).
9. Review-gate structural: single commit `ef9d170` reviewed whole via
   byte-safe per-file `git diff` + working-tree reads; touch-set enforcement
   via `git diff --name-only 4900e04..HEAD` -- every path in-set or sanctioned
   rider; worktree census clean of fixture/test-src dirt.

## Implementer-hop evidence re-verified, not trusted

- Green digest `197 passed / 0 failed / 0 ignored / 3 suites` -- reproduced
  verbatim above (gate 1).
- Red `189 passed / 8 failed / 0 ignored / 2 suites` -- journal-attested only;
  Red is a historical state unreproducible after Green. Consistency checks
  pass: totals 197 at both Red and Green; 188 pre-existing + 9 new; the 8
  failing names equal the handoff's predicted set; 2-vs-3 suite difference
  explained (cargo fail-fast skips the 0-test doc-test suite when the
  integration suite fails). Deviation D4 disposition: cosmetic, accepted.
- `theme_token_check`/`hygiene`/`encoding --staged` exits 0 -- independent
  equivalents (`--base 4900e04`, workspace-level forms) all exit 0 above.

## Explicitly NOT run

- `python tools/testrun.py --baseline` -- unrunnable by design (G7 errata: no
  baseline file committed; open ledger row). Prescribed `--distill` unscoped
  form used instead and recorded verbatim.
- `bench_compare.py` PASS line -- not obtained (SUPERSEDED 2026-10-04: the
  line has since been obtained on a quiet machine; see Addendum; original
  trail kept: allowlist refusal for direct run, environment lock for the
  wrapped run, then the documented assets-only exemption, whose precondition
  is machine-proven above).
- The Red-gate cargo run -- historical state; journal evidence accepted per the
  consistency checks above.

## Post-approval obligations (not blockers)

- `ui-dark-grey-theme` ledger row status flip remains that milestone's
  reviewer-hop duty (deviation D2 disposition).
- Verdict findings 1 (allowlist gap for `ui_smoke`/`bench_compare` under
  direct invocation) and 2 (stale process holding `target\debug\tool.exe`)
  route to the orchestrator's ledger/deferred-findings hop.

## Addendum 2026-10-04 (late hop, post-lock): bench_compare real PASS

After the operator killed the stale process holding `target\debug\tool.exe`
and quieted the machine, the load-perf gate was re-run on the NORMAL target
dir via the sanctioned wrapper `python tools/distill.py -- python
tools/bench_compare.py` (no temp `CARGO_TARGET_DIR`). Result, verbatim:

```
### generic PASS (exit 0)
bench compare: 3 run(s), no regressions past 1.25x baseline
capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (17 lines)
```

This PASS SUPERSEDES the assets-only exemption recorded at gate 5 (whose
precondition had been machine-proven; the exemption text stands above,
marked superseded, as audit trail).

History for attribution, all in-session: attempt with the stale lock died as
`ERROR cargo bench run 2 of 3 failed (exit 101)` (lock, not perf); on a
loaded machine two normal-target runs failed with one 0.25-ms-past-gate
breach (run 1: `WARN build_include_elements: 68.8 ms vs baseline 54.9 ms
(+25%)`) and six uniform breaches +29..+67% (run 2, pure-Rust phases
included -- impossible for an assets-only commit); the quiet-machine run
shows every phase inside the 1.25x gate (`build_include_elements` back to
59.4 ms, clone warm 1.3 ms, planner 1730.3 ms, justification 9530.5 ms),
confirming load vs a per-machine baseline, not a regression from `ef9d170`.
