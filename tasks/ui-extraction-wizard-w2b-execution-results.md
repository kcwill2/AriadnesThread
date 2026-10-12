# W-2b Execution Results - ui-extraction-wizard

Hop: `ui-extraction-wizard` W-2b (`crates/ui` reader trait + wizard DTOs +
two GET routes `/api/extraction/{plan_id}/manifest` +
`/api/extraction/{plan_id}/dry-run` with uniform 503 loading / 404 unknown
plan / 409 stale generation).
Range under review: `7b2238c..27cb9de` (landing `f5a7d80`, journal rider
`27cb9de`).
Artifacts: verdict `tasks/ui-extraction-wizard-w2b-verdict.md`, this file.

## Ground truth

`python tools/checkpoint_guard.py status` at review start: HEAD `27cb9de`,
branch `main`, ahead 0 / behind 0, dirty 0, untracked 0. Worktree census: 5
worktrees, none owned by this milestone; only pre-existing foreign-track dirty
entries in `mergetest` (272), `tarjan-s-scc` (2), `shining-enthusiasm` (1).
Working tree re-verified clean at the end of the hop with the read-only
`git_git_status` tool.

## Acceptance criteria, one by one, with the evidence line

| # | Acceptance criterion | Result | Evidence (verbatim) |
|---|---|---|---|
| A1 | Both routes registered with `get(...)` only (W1) | MET | `server.rs:78` `.route("/api/extraction/:plan_id/manifest", get(api_extraction_manifest))`; `server.rs:79` `.route("/api/extraction/:plan_id/dry-run", get(api_extraction_dry_run))`; lock test `test_server_registers_extraction_routes_with_get_only ... ok` (fresh capture line 1980) |
| A2 | New module contains no `fs::write`, `fs::remove_file`, `fs::create_dir`, `ArtifactManager` (W1) | MET | `test_extraction_route_module_has_no_write_calls ... ok` (capture line 1977); the lock sweeps the whole module source via `include_str!` |
| A3 | New module registers no `post(`, `put(`, `delete(`, `patch(` (W1) | MET | `test_extraction_route_module_has_no_mutation_route_registrations ... ok` (capture line 1972) |
| A4 | 503 `analysis_loading` when no reader installed, both routes | MET | `test_manifest_route_returns_503_analysis_loading_while_no_reader ... ok` (1983), `test_dry_run_route_returns_503_analysis_loading_while_no_reader ... ok` (1979); body asserted equal to `{"error":"analysis_loading"}` and ASCII-checked |
| A5 | 200 with the reader's DTO for a live-epoch reader's known plan, both routes | MET | `test_manifest_route_returns_200_with_reader_manifest ... ok` (1982), `test_dry_run_route_returns_200_with_reader_dry_run ... ok` (1973); body asserted equal to the serialized reader DTO |
| A6 | 404 `unknown_plan` with the echoed id, both routes | MET | `test_manifest_route_returns_404_unknown_plan ... ok` (1981), `test_dry_run_route_returns_404_unknown_plan ... ok` (1974) |
| A7 | 409 `stale_generation` with the live epoch, both routes (W6) | MET | `test_manifest_route_returns_409_stale_generation ... ok` (1984), `test_dry_run_route_returns_409_stale_generation ... ok` (1978); `"epoch":1` substring asserted, ASCII-checked. Byte order of the 409 body is verdict Finding 1 (content exact, key order BTreeMap-sorted) |
| A8 | Check order no-reader -> 503; epoch != live -> 409; None -> 404; Some -> 200 | MET | `routes/api_extraction.rs:68-78` `reader_ready_for_request` implements the gate; both handlers call it, so the order is uniform by construction |
| A9 | Error bodies ASCII | MET | `assert_ascii_body` runs on every error body in all four shapes; a non-ASCII sweep over the new module and test file returns zero matches |
| A10 | Trait shape C2 verbatim, exported for the cli provider of W-2a | MET | `routes/api_extraction.rs:28-34`; `lib.rs:14` `pub use routes::api_extraction::ExtractionManifestReader;` |
| A11 | Wizard DTOs additive, `#[serde(default)]` on every field, serde-plain, exact field names | MET | `dto/extraction.rs:70-166`; all four structs with the 18 pinned field names; `dto.rs:10` `pub use extraction::*;` keeps every `ariadne_ui::*` path |
| A12 | Caps 64 / 200 / 200 | MET | `dto/extraction.rs:79` `EXTRACTION_MANIFEST_MAX_FILES = 64`, `:84` `EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE = 200`, `:87` `EXTRACTION_DRY_RUN_MAX_REWRITES = 200` |
| A13 | One `extraction_reader` field with install/accessor; `new_loading`/`new_loaded` signatures unchanged | MET | `state.rs:97-98` field, `:191` `install_extraction_reader`, `:197` accessor; `:102`/`:124` signatures unchanged; `:120`/`:148` initialize to `None` |
| A14 | `main.rs` mod line at the fixed site | MET | `main.rs:11` `mod extraction_wizard_api;` immediately after `mod extraction_contrast;` and before `mod file_view;` |
| A15 | Boot payload does not grow (W9) | MET | Nine-path range diff contains no payload builder, no `export.rs`, no `dto.rs`, no `crates/cli/**`; the payload suites pass unedited inside the 294 |
| A16 | Touch-set conformance, per commit, oldest first | MET | `git diff --name-status "7b2238c" "HEAD"` returns exactly the nine declared paths; `f5a7d80` = 9 paths +808/-0, `27cb9de` = journal only +54/-2 |
| A17 | `crates/cli/**` diff-empty (frozen lock) | MET | `git diff --numstat "7b2238c" "HEAD" -- crates/cli` returns empty (positive control on the same command: `25 0 crates/ui/src/state.rs`) |
| A18 | 18 card-family locks byte-identical (W7); goldens, fixtures, assets, `themes.css`, `theme_token_baseline.json` untouched | MET | None of these paths appears in the nine-path range diff |
| A19 | Zero added hex; `theme_token_baseline.json` diff-empty | MET | `python tools/distill.py -- python tools/theme_token_check.py --check-added --base 7b2238c` -> `### generic PASS (exit 0)`, capture 0 lines |
| A20 | Functions < 100 lines, files < 1000 lines, hygiene exit 0 | MET | `python tools/distill.py -- python tools/hygiene_check.py` -> `### generic PASS (exit 0)`, 179 capture lines all R3 warn-only, zero R3 rows in any of the six touched paths. New file sizes: api_extraction.rs 103, extraction_wizard_api.rs 371, dto/extraction.rs 166, state.rs 241, server.rs 82, lib.rs 28 |
| A21 | Newly authored lines pure ASCII (W-1 lesson) | MET | Zero non-ASCII matches in api_extraction.rs, extraction_wizard_api.rs, the journal, and every added range of the modified files; the only glyphs in the touched files sit in pre-W-2b regions (dto/extraction.rs:30/44, state.rs:16-157) |
| A22 | Encoding guard | MET | `python tools/distill.py -- python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)`, capture 0 lines |
| A23 | F14 reverse dependencies compile | MET | `python tools/distill.py -- cargo check -p ariadne-cli --all-targets` -> `ok (exit 0, 0 warnings emitted)`; `python tools/distill.py -- cargo check -p ariadne-integration-tests --all-targets` -> `ok (exit 0, 1 warnings emitted)` (pre-existing in untouched files) |
| A24 | Red/Green discipline with the predicted failure shape | MET | Journal records the first Red at `287 passed / 7 failed / 0 ignored / 2 suites` (the seventh failure being the module doc quoting the forbidden token), the reworded re-run at `288 passed / 6 failed / 0 ignored / 2 suites`, the six named reader-consulting failures verbatim, and Green at `294 passed / 0 failed / 0 ignored / 3 suites` |

## Workspace gates (this hop owns them)

### W-1 census - canonical line

`python tools/testrun.py --distill` (the unscoped form; see the not-run note
below):

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 2767 lines]
1458 passed / 0 failed / 13 ignored / 112 suites
no failures
1458 passed / 0 failed / 13 ignored / 112 suites
```

Canonical line, quoted verbatim:
`1458 passed / 0 failed / 13 ignored / 112 suites`

Arithmetic, reconciled against the ledger anchor exactly:

- Ledger anchor (`tasks/milestone_ledger.md`, `ui-extraction-wizard` row):
  `1447 passed / 0 failed / 13 ignored / 112 suites` @ `59a99c4` (the
  audit-remediation merge).
- Delta between `59a99c4` and this hop's start rev `7b2238c`: zero. The
  commits in between are `b27aa46` (W-1, a refactor that moved code without
  changing any test), `82e5ffe` (journal rider, docs only) and `7b2238c`
  (W-1 verdict + execution-results, docs only).
- Delta contributed by W-2b: `+11` passed, entirely in the new
  `extraction_wizard_api.rs` - 4 manifest route-shape tests + 4 dry-run
  route-shape tests + 3 string locks.
- `1447 + 11 = 1458` passed. Failed 0 held. Ignored 13 held. Suites 112
  held. Observed line matches on all four numbers with zero residual.

Independent corroboration of the same +11 from the scoped crate:
`python tools/testrun.py --distill -p ariadne-ui` ->
`294 passed / 0 failed / 0 ignored / 3 suites`, and the W-1 start-tree
baseline recorded in the journal is
`283 passed / 0 failed / 0 ignored / 3 suites`; `283 + 11 = 294`. All 11 new
tests appear as `... ok` in the fresh capture (lines 1972-1984), and the
remaining 283 are the pre-existing suite running unedited.

### W-2 tools unit suites

`python -m unittest discover -s tools -p "*_tests.py"` (documented CI form):

```
Ran 219 tests in 23.039s

OK
```

### W-3 ui_smoke

Direct `python tools/ui_smoke.py` was refused by the session allowlist
(verbatim: `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`),
so it ran through the documented distilling fallback
`python tools/distill.py -- python tools/ui_smoke.py`:

```
### generic PASS (exit 0)
boot+route checks clean (cold fixture, attempt 1, 1.0s)
ui_smoke: PASS
capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (2 lines)
```

The two new GET routes do not disturb the cold boot, the route roster or the
`Cache-Control: no-store` assertion.

### W-4 bench_compare

Direct `python tools/bench_compare.py` was refused by the session allowlist
(same verbatim catch-all text), so it ran through
`python tools/distill.py -- python tools/bench_compare.py`:

```
### generic FAIL (exit 2)
ERROR parse format drift: documented phase row(s) missing from bench output (never substituting zeros):
capture: C:\Users\Kurt\Code\AriadnesThread\target\distill-capture.txt (2 lines)
```

The capture's second line is
`  missing documented phase: generate_report+detect_cycles (combined)`.

This is the known deterministic failure from the stale `DOCUMENTED_PHASES`
tuple in `tools/bench_compare.py` (the M23.5 rename means the bench prints
`generate_report+detect_cycle_groups (combined)`). It reproduces at every
hop on this machine, before and after this change, and is independent of any
code in this range. The exemption is recorded only with the zero-diff proof:

```
$ git diff --numstat "7b2238c" "HEAD" -- crates/cli tools/bench_compare.py tools/bench_baseline.json crates/pipeline
(no output)
```

with the positive control on the same command form:

```
$ git diff --numstat "7b2238c" "HEAD" -- crates/ui/src/state.rs crates/cli tools/bench_compare.py tools/bench_baseline.json
25	0	crates/ui/src/state.rs
```

So `crates/cli/**`, `tools/bench_compare.py`, `tools/bench_baseline.json` and
`crates/pipeline/**` are all byte-identical across the range; the bench
producer was not touched and no bench number was substituted, inferred or
silently passed. The gate ran and failed for the documented reason. The
one-line tuple fix remains owed to the orchestrator (`tools/**` is out of
both the implementer's and the reviewer's edit grant), per W-1 verdict
Finding 4.

## What was not run, and why

1. **`python tools/testrun.py --baseline`** - refused at the argument-parser
   level with `usage error: missing value for --baseline` (exit 2).
   `--baseline` is a FILE value flag that needs one `N/M/K/S` line, and no
   baseline census file is committed anywhere in the repo. This is the 8th
   encounter of the standing errata row in `tasks/milestone_ledger.md`
   (`Kickoff G7 literal python tools/testrun.py --baseline is unrunnable`),
   still awaiting the orchestrator's decision to commit a baseline file or
   amend the kickout/AGENTS.md wording. Substituted with the documented
   `--distill` unscoped census above, which reconciles to the ledger anchor
   exactly. Not silently skipped.
2. **`python tools/encoding_guard.py --staged`** - not reproducible
   post-hoc: staging is empty now that `27cb9de` is committed, and
   `commit_scoped.py` owns staging. Standing verdict precedent (W-1 gate 6)
   applies; the post-commit `--all` form is a superset and ran clean.
3. **A byte-level `git cat-file`-inside-Python pure-ASCII proof** - `python
   -c ...` was refused by the session allowlist (verbatim catch-all text
   above). Substituted with a non-ASCII sweep over `crates/ui` and the
   journal plus `encoding_guard --all` exit 0, and with per-hunk
   `git show` diffs of the added ranges.
4. **`git status --porcelain` and `git ls-tree`** - refused by the session
   allowlist. Substituted with the read-only `git_git_status` tool (clean
   tree) and the file listing (35 modules under
   `crates/ui/tests/ui_test`, with `extraction_wizard_api.rs` declared at
   `main.rs:11`).
5. **Browser-level manual smoke** - not in this hop's duty. `ui_smoke` covers
   boot, routes and `no-store`; the wizard UI does not exist until W-3a/W-3b.
6. **Bench regression verdict** - deliberately not issued. The gate fails for
   a producer-independent, pre-existing parse reason and the producer is
   byte-identical, so no regression statement can be drawn from it either
   way. Recorded as the standing exemption, not as a pass.

## Outcome

All acceptance criteria A1-A24 are MET. The workspace gates ran fresh in this
session: census `1458 passed / 0 failed / 13 ignored / 112 suites`, tools
`Ran 219 tests in 23.039s` / `OK`, `ui_smoke: PASS`, bench exempt with
zero-diff proof. Nothing was skipped silently; every gap is itemized above
with its substitution and evidence.
