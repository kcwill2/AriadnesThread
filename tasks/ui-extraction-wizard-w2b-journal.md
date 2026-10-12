# W-2b Journal - ui-extraction-wizard: reader trait, wizard DTOs, two GET routes

Handoff: `tasks/ui-extraction-wizard-w2b-handoff.md` (milestone
`ui-extraction-wizard`). Dependent on W-1 (landed `b27aa46`, journal rider
`82e5ffe`). This journal is the checkpoint record; gates are quoted verbatim
from the distilling entry points.

## Boot (session start, before any edit)

- Duty 1 first (reviewer's exact command): landing `7b2238c` (parent
  `82e5ffe`) with exactly the two scoped paths
  (`tasks/ui-extraction-wizard-w1-verdict.md`,
  `tasks/ui-extraction-wizard-w1-execution-results.md`); staged set verified
  via `python tools/distill.py -- git diff --cached --name-only` (exactly the
  two paths, nothing else); `encoding_guard.py --staged` exit 0 pre-commit.
- `python tools/checkpoint_guard.py status` at boot: HEAD `82e5ffe`, branch
  `main`, dirty tracked files none, untracked only the two W-1 review
  artifacts Duty 1 tracks. No foreign WIP under `crates/**` - boot rule met,
  no STOP condition.
- Post Duty-1 HEAD: `7b2238c`.

## Gate 1 - baseline scoped ui suite (start rev `7b2238c`)

Command: `python tools/testrun.py --distill -p ariadne-ui`

Verbatim digest:

```
### cargo test PASS (exit 0) [capture: ...testrun-capture.txt, 312 lines]
283 passed / 0 failed / 0 ignored / 3 suites
no failures
283 passed / 0 failed / 0 ignored / 3 suites
```

Baseline canonical line: `283 passed / 0 failed / 0 ignored / 3 suites`
(identical to W-1's post-move line, as expected: no test change between
`82e5ffe` and `7b2238c`).

## Plan of record

Per handoff RGB section, file order: test file first, then compile-complete
skeleton (trait, DTOs, state field, routes, 503-only handlers); Red gate
expected exit 1 with the reader-consulting tests failing semantically
(200/404/409 asserted, 503 served, no-write locks already green); Green wires
the reader accessor and live epoch in the C3 check order.

## Gate 2 - Red

Command: `python tools/testrun.py --distill -p ariadne-ui` (after authoring
`extraction_wizard_api.rs` plus the compile-complete skeleton: trait, DTOs,
caps, state field + install/accessor, both routes registered, handlers
serving only the loading branch).

First attempt (deviation D1, declared below): `287 passed / 7 failed /
0 ignored / 2 suites` - the seventh failure was
`extraction_wizard_api::test_extraction_route_module_has_no_write_calls`
panicking "routes/api_extraction.rs must not contain ArtifactManager (W1)"
because the new module's OWN doc comment quoted that forbidden token. The
module doc was reworded to state the rule without quoting the lock's
literals; the lock itself was not touched.

Verbatim digest of the corrected Red run (exit 1):

```
### cargo test FAIL (exit 1) [capture: ...testrun-capture.txt, 372 lines]
288 passed / 6 failed / 0 ignored / 2 suites
```

The six named failing tests (verbatim section headers from the digest and
capture; the digest truncated the sixth as "... and 1 more failures", the
capture holds it):

- `extraction_wizard_api::test_manifest_route_returns_200_with_reader_manifest`
  - `assertion 'left == right' failed: expected status 200 for a live-epoch
  reader's known plan` / `left: 503`
- `extraction_wizard_api::test_manifest_route_returns_404_unknown_plan` -
  `expected 404 for a plan the reader does not know` / `left: 503`
- `extraction_wizard_api::test_manifest_route_returns_409_stale_generation`
  - `expected 409 for a reader whose epoch differs from the live epoch` /
  `left: 503` (first Red attempt output, same shape)
- `extraction_wizard_api::test_dry_run_route_returns_200_with_reader_dry_run`
  - `expected status 200 for a live-epoch reader's known plan` / `left: 503`
- `extraction_wizard_api::test_dry_run_route_returns_404_unknown_plan` -
  `expected 404 for a plan the reader does not know` / `left: 503`
- `extraction_wizard_api::test_dry_run_route_returns_409_stale_generation`
  - `expected 409 for a reader whose epoch differs from the live epoch` /
  `left: 503`

This is exactly the handoff's predicted Red shape: the reader-consulting
tests (200/404/409) fail while every response is
`503 {"error":"analysis_loading"}`, and the no-write/GET-only locks pass
already. Total accounting: `288 + 6 = 294 = 283 baseline + 11 new tests`
(8 route-shape tests + 3 string locks). The suite counter reads 2 because
cargo stops after the first failing target, skipping the empty doc-test
target that the passing baseline counts (capture lines 370-371:
`unittests src\lib.rs` + `tests\ui_test\main.rs` ran).

## Gate 3 - Green

Command: `python tools/testrun.py --distill -p ariadne-ui`

Verbatim digest:

```
### cargo test PASS (exit 0) [capture: ...testrun-capture.txt, 324 lines]
294 passed / 0 failed / 0 ignored / 3 suites
no failures
294 passed / 0 failed / 0 ignored / 3 suites
```

Canonical line = baseline + 11 new tests, exactly (`294 = 283 + 11`). The
suite counter is back to 3 (the doc-test target runs again once no target
fails). Green wiring: both handlers resolve the reader through the shared
`reader_ready_for_request` helper in the C3 check order (no reader -> 503;
reader epoch != `analysis_epoch()` -> 409 with the live epoch) and then the
plan lookup (None -> 404 with the echoed id; Some -> 200 with the DTO).
Refactor folded into this run: the error bodies go through one
`error_response` helper and the shared reader gate carries the duplication;
every function is far under 100 lines. Re-run after the refactor naming
pass (`Err(refused)`) reproduces the same canonical line (this digest IS
that run).

## Gate 4 - F14 cli reverse dependency

Command: `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`

Verbatim digest: `ok (exit 0, 0 warnings emitted) -> capture:
...distill-capture.txt`. The live reverse dependency compiles unchanged: the
`state.rs` seam moved no cli call site (W9: no builder change, no payload
field, no cli change).

## Gate 5 - hygiene

Run of record (distiller, for the exit code):
`python tools/distill.py -- python tools/hygiene_check.py` ->
`### generic PASS (exit 0)`, capture 179 lines, every line an R3 warn-only
row pre-existing in untouched files. Zero R1/R2 findings; zero R3 rows in
any file this hand touched. (Deviation D4: the first invocation was direct
`python tools/hygiene_check.py`, which leaked the raw warn list; the run of
record above is the distilling-form one, identical result, exit 0.)

## Frozen-lock checks (handoff "Frozen locks" section)

- `python tools/distill.py -- python tools/theme_token_check.py
  --check-added --base 7b2238c` -> `### generic PASS (exit 0)`, capture 0
  lines (silent; zero added hex, no asset touched;
  `tools/theme_token_baseline.json` appears in no diff of this hand).
- Card-family locks (`extraction_card_story.rs`,
  `extraction_card_playbook.rs` and the rest of the 18): neither appears in
  this hand's diff; the commit-vs-touch-set proof below (exactly nine paths)
  is the machine check.
- `crates/cli/**` diff-empty: proven post-commit in the rider record.

## Gate 6 - encoding guard, staged

Staging: `python tools/distill.py -- git add -- <the nine touch-set paths>`
(sanctioned explicit-path distill route; raw `git add` is allowlist-denied;
never `git add .`). Staged set verified via
`python tools/distill.py -- git diff --cached --name-only`: exactly the nine
touch-set paths, nothing else.

Command: `python tools/encoding_guard.py --staged` -> exit 0, silent.

## Gate 7 - landing commit

Command (exact handoff form, nine explicit scopes, never `git add .`):

```
python tools/commit_scoped.py --scope crates/ui/src/dto/extraction.rs
  --scope crates/ui/src/state.rs --scope crates/ui/src/routes/api_extraction.rs
  --scope crates/ui/src/routes/mod.rs --scope crates/ui/src/server.rs
  --scope crates/ui/src/lib.rs --scope
  crates/ui/tests/ui_test/extraction_wizard_api.rs --scope
  crates/ui/tests/ui_test/main.rs --scope
  tasks/ui-extraction-wizard-w2b-journal.md -m "feat(ui): extraction
  manifest/dry-run routes behind a read-only reader trait (503/404/409)"
```

Result (rider record, deviation D5 as pre-declared): landing `f5a7d80`,
parent `7b2238c`. `git show --stat`: exactly the nine touch-set paths,
808 insertions(+), zero deletions across tracked source
(dto/extraction.rs 98, lib.rs 4, routes/api_extraction.rs 103 [NEW],
routes/mod.rs 1, server.rs 3, state.rs 25, extraction_wizard_api.rs 371
[NEW], main.rs 1, journal 202). Touch-set conformance:
`git diff --name-only 7b2238c..HEAD` -> exactly those nine paths, nothing
else. This rider commit is the docs-only second commit scoped to the
journal path alone.

## Gate 8 - encoding guard, all

Command: `python tools/encoding_guard.py --all` -> exit 0, silent.

## Rider post-commit proofs (all via the distilling entry point)

- `git status --porcelain` -> empty capture; clean tree at landing.
- `git diff 7b2238c..HEAD -- crates/cli
  crates/ui/tests/ui_test/extraction_card_story.rs
  crates/ui/tests/ui_test/extraction_card_playbook.rs` -> 0-line capture
  (crates/cli diff-empty per the handoff frozen locks; the two named
  card-family test files untouched; the 9-path range listing above proves
  the same for all 18).
- Frozen regions: `tools/theme_token_baseline.json`, `tests/golden/**`,
  `tests/fixtures/**`, assets, themes.css appear in no diff of this hand
  (the 9-path listing is the machine proof).

## Completion record

- Gates 1-8: 1 `283 passed / 0 failed / 0 ignored / 3 suites` exit 0;
  2 exit 1 with the six named Red failures; 3 `294 passed / 0 failed /
  0 ignored / 3 suites` exit 0; 4 `ok (exit 0, 0 warnings emitted)`;
  5 `### generic PASS (exit 0)`; 6 exit 0 silent (staged set verified as
  exactly the nine paths first); 7 landing `f5a7d80`; 8 exit 0 silent.
- W-2a handoff note: the seam is live - `ariadne_ui::ExtractionManifestReader`
  (exported from `lib.rs`), the four wizard DTOs and the three caps
  (exported through `dto/extraction.rs` -> `dto.rs` -> `lib.rs`), and
  `UiState::install_extraction_reader` / `UiState::extraction_reader`.
  `new_loading`/`new_loaded` signatures unchanged; the boot payload grew by
  nothing (W9).

## Numbered deviations

- **D1 (Red hygiene of my own doc text).** The first Red run was
  `287 passed / 7 failed` - the seventh failure was my NEW module doc
  quoting the literal token `ArtifactManager` to state the W1 rule, tripping
  the very string lock that required it to be absent. The module doc was
  reworded to state the rule without quoting the forbidden tokens; the lock
  file was not touched; Red reproduced with exactly the six predicted
  failures. No scope impact; declared because the first Red digest differs
  from the run of record.
- **D2 (path-parameter syntax).** The handoff documents the routes as
  `/api/extraction/{plan_id}/manifest` and `.../dry-run`; axum 0.7 (the
  workspace pin) registers path parameters with `:name` syntax, so
  `server.rs`, the route table and the test lock use `:plan_id`. The
  requested URLs are identical and the trait/DTO/state seams W-2a plugs
  into are unaffected. Documentation form vs compile-forced syntax, not a
  contract change.
- **D3 (Red skeleton handler shape).** The Red handlers took underscored
  extractors (`State(_state)`, `Path(_plan_id)`) and returned the loading
  503 on every path rather than writing an explicit `is_none()` branch that
  is immediately deleted at Green. The handoff's observable Red prediction
  ("every response is `503 {\"error\":\"analysis_loading\"}`") is what the
  six named failures recorded; the Green handlers carry the full C3 order.
- **D4 (hygiene first-invocation form).** `python tools/hygiene_check.py`
  was first run direct (it was allowed and exited 0 but printed the raw
  warn list into context, violating the distilling-only discipline); the
  run of record quoted in Gate 5 is the `distill.py` wrapper re-invocation.
  Same exit 0, same result; declared as a process deviation.
- **D5 (journal rider, pre-declared).** The landing commit (gate 7) can
  only record gates up to 6; the landing hash and the gate-8 result are
  appended by a docs-only journal rider commit immediately after, per the
  W-1 precedent (`b27aa46` + `82e5ffe`). Numbered here up front per W-1
  verdict Finding 1 (rider-style deviations get numbers, not silence). The
  rider is inside the touch-set (the journal path is scope 1) and carries
  zero source change.
