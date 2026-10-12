# AR-11 Execution Results - reviewer fresh-gate record

Workspace: `shining-enthusiasm` worktree, HEAD `adb54ea`, tracked files
clean (checkpoint_guard: dirty none; 62 untracked = other lanes' ar-*
docs). All lines below quoted verbatim from the reviewer's own command
stdout. Reviewer session: 2026-10-11 ~02:10Z.

## Ground truth

- `python tools/checkpoint_guard.py status` -> HEAD `adb54ea`,
  branch `shining-enthusiasm`, no tracked dirty files.
- Single commit under review; actual parent `0741074` (docs-only AR-30
  landing; journal finish base `0d7ecd3` two commits back; delta carries
  zero crates/ui content).

## Gate 1 - scoped package suite (handoff gate 1; reviewer ran raw
`testrun.py` for own stdout per AR-02/AR-08 capture-race ruling)

    python tools/testrun.py -- -p ariadne-ui

Canonical line (verbatim):

    265 passed / 0 failed / 0 ignored / 3 suites

= 256 baseline + the 9 oracle tests. Raw output in the reviewer run names
ALL 9 `asset_injection_hardening::*` tests passing, including both
previously-swallowed ones:

    test asset_injection_hardening::test_apijs_declared_sites_escaped_and_delegated ... ok
    test asset_injection_hardening::test_traversaljs_declared_option_sites_escaped_without_degree_regression ... ok

(plus test_mainjs_delegation_wiring_calls_the_legacy_handlers,
test_no_repo_interpolation_survives_in_inline_onclick,
test_graphjs_inspector_sites_delegated_and_escaped,
test_hotspotsjs_rows_and_cards_delegated_behaviour_preserved,
test_shared_escapers_are_quote_safe_and_shipped_exactly_once,
test_no_nested_escaper_calls,
test_served_and_exported_surfaces_carry_hardened_markup_for_hostile_payloads)

Story/characterization locks green UNEDITED in the same run:
`test_graph_js_line_budget`, `extraction_card_playbook::*` (incl.
`test_playbook_escape_discipline`), `extraction_card_story::*`,
`hotspot_direction::*`, `source_containment::*`, `static_html::*`,
`theme_contract::*`.

## Gate 2 - UI runtime smoke

    python tools/distill.py -- python tools/ui_smoke.py

Verbatim:

    ### generic PASS (exit 0)
    boot+route checks clean (cold fixture, attempt 1, 0.2s)
    ui_smoke: PASS
    capture: ...\target\distill-capture.txt (2 lines)

Attempt 1; no build contention encountered (worktree clean at run time).

## Gate 3 - encoding

    python tools/encoding_guard.py --all

Verbatim output: silent (no findings; exit-0 contract per tool design).

## Gate 4 - hygiene

    python tools/hygiene_check.py

Blocking rows, verbatim (exactly 2, both AR-13-owned server.rs pair):

    crates/ui/src/server.rs: R2 1172 lines > 1000
    crates/ui/src/server.rs:816: R1 fn `api_graph_traversal` is 161 lines (> 100)

Zero NEW R1/R2. No R3 warn in any AR-11-touched file
(`asset_injection_hardening.rs` and all 6 assets absent from the warn
list; all warns pre-existing elsewhere). Exit code not echoable
(allowlist); adjudicated on printed contract per repo precedent.

## Gate 5 - theme hex drift

    python tools/theme_token_check.py --check-added --base 0741074

Verbatim output (exit 1 by tool contract; all three from the base-
independent `scan_frozen_counts`, ZERO commit-scoped
`new hex literal added since` violations):

    crates/ui/assets/index.html: 6 hex literals exceeds frozen baseline count 4 (update tools/theme_token_baseline.json deliberately in this commit)
    crates/ui/assets/main.js: 1 hex literals exceeds frozen baseline count 0 (update tools/theme_token_baseline.json deliberately in this commit)
    crates/ui/assets/traversal.js: 1 hex literals exceeds frozen baseline count 0 (update tools/theme_token_baseline.json deliberately in this commit)

Attribution (reviewer-verified): flagged matches are HTML entities
(`main.js:308` `&#8250;`; `traversal.js:390` `&#128161;` — both outside
every diff hunk; `index.html` blob unchanged in adb54ea).
`tools/theme_token_baseline.json` (frozen `d76a559`: index.html=4,
main.js=0, traversal.js=0) byte-untouched by the commit. Disposition:
informational for AR-11; baseline fix = separate approval-gated commit
(see verdict finding 3).

## Acceptance-criterion ledger (handoff contract)

| Criterion | Result | Evidence |
|---|---|---|
| Objective 1: escaper at every repo interpolation, quote extension | MET | per-asset diff review; oracle escaper lock green |
| Objective 2: data-id + delegated listeners, behavior preserved | MET (delta verdict-1 accepted) | mapping table = diff; button branch first + early return |
| Objective 3: NEW ui_test mod, hostile fixture, single-pass string locks | MET | 9 tests, 442-line new mod, +1 main.rs, no server_api.rs growth |
| Touch-set hard | EXACT | 9 paths, zero crates/ui/src |
| graph.js budget cap 990 green, cap not raised | MET | 961 lines; `test_graph_js_line_budget ... ok` in own run |
| traversal.js zero-diff except :44/:131; degree bug untouched | MET | diff = 2 hunks of 1 line each; byte-locks green |
| TDD red-first | MET (journal trail) | 1/6 -> 2/7 -> 6/3 -> 9/0 module-scoped; red arithmetic reconciles with the delimiter-repair story exactly (verdict section 2) |
| Story tests unedited and green | MET | not in touch-set; green in 265 run |
| Pure ASCII + encoding gate | MET | encoding_guard --all silent |
| Commit scoped | MET | 9 paths; foreign untracked files left alone |
| Gate 5 silent | NOT MET at launch - inherited drift, unsatisfiable at base; disposition verdict section 5 | verbatim output above |

## Explicitly NOT run

- `python tools/testrun.py --baseline` / `--workspace` census: skipped per
  orchestrator dispatch restriction (AR-20/AR-23 active on cli/lang;
  ui-external compilation out of bounds; precedent AR-10/AR-18 "NOT run
  per dispatch restriction"; AR-03 precedent: no repo baseline file exists
  for `--baseline`).
- `python tools/bench_compare.py`: not required by this milestone's
  contract (no perf surface); out of scope.
- Full integration/other-crate suites: dispatch restricts to ui-package
  scope; ui package ran fully.

## Supplementary workspace gate run (no cargo contention; python-only)

    python -m unittest discover -s tools -p "*_tests.py"

Verbatim:

    Ran 219 tests in 20.429s

    OK

## Refused commands (verbatim refusal)

- `python -c "import re; ..."` (testattr/delimiter counting): refused —
  `The user has specified a rule which prevents you from using this
  specific tool call. Here are some of the relevant rules
  {"permission":"bash","pattern":"*","action":"deny","source":"global"}`
  (catch-all boilerplate; pattern-level denial of ad-hoc `-c` form;
  script-path execution works — checkpoint_guard/testrun/encoding/hygiene/
  theme/distill all ran). Substituted evidence: Grep attribute/delimiter
  counts + full Read of the oracle + raw testrun output naming all 9
  tests (stronger than the intended probe).
- Exit-code echo (`echo $LASTEXITCODE` chaining, `python -c`): same class
  of refusal; gates adjudicated on printed contracts + exact output shape
  (AR-01/AR-02/AR-03/AR-08 precedent).
