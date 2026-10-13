# A-3 Handoff - extraction-auto-apply: the applier trait, apply DTOs, the POST apply route, the state seam

Depends on nothing in the code (A-1 and A-2 may still be in flight; this
hand touches a different crate). A-4 cannot start until this hand lands.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/extraction-auto-apply-shape.md` sections 2 (invariants A1-A5, A9
   and their oracles), 3 (C2 trait + DTOs verbatim, C3 route table verbatim,
   C4 token semantics, C5 mutex + epoch model), 5 (module map, A-3 row and
   the declared rider), 8 (hygiene notes).
   `tasks/extraction-auto-apply-kickoff.md` sections 3 (items 1-3), 6.
3. Model files for structure and doc idiom:
   `crates/ui/src/routes/api_extraction.rs` (all 103 lines - the read-only
   twin this module mirrors), `crates/ui/src/routes/mod.rs`,
   `crates/ui/src/state.rs` (the reader field at :97-98, install at
   :191-193, accessor at :197-199, `new_loading` :102-122, `new_loaded`
   :124-155), `crates/ui/src/server.rs` (the route table :50-84), 
   `crates/ui/src/lib.rs` (:11-14), `crates/ui/src/dto/extraction.rs`
   (the wizard DTO section :70-166).
4. Model test file: `crates/ui/tests/ui_test/extraction_wizard_api.rs`
   (route-test idiom, `ScriptedReader`, the `wizard_stale_reader_state`
   injection recipe :130-139, `assert_ascii_body` :141-143,
   `request_json_body` :145-155) and `crates/ui/tests/ui_test/main.rs`
   (the mod list).
5. `crates/cli/src/commands/ui/extraction_reader.rs` :182 and :417 (the two
   declared rider sites).

## Objective and preserved invariants

Land the apply backend contract in `crates/ui`, deliberately mirroring the
read-only wizard seam so the ui crate never names a write: the
`ExtractionPlanApplier` trait, the refusal enum, the apply DTOs, one new
`extraction_applier` field on `UiState` with install/accessor, the
`POST /api/extraction/:plan_id/apply` route with its full refusal table,
and the dry-run route's token mint. The ui crate gains no filesystem write:
every mutation is delegated to the host through the trait, and the trait's
only implementation lands in A-4 (`crates/cli`).

Invariants this hand must preserve:

- **Single write path (A1).** The new module contains no `fs::write`,
  `fs::remove_file`, `fs::create_dir` or `ArtifactManager` token, and no
  `post(`/`put(`/`delete(`/`patch(` registration (the registration lives in
  `server.rs`). Locked by string assertions in the new test file.
- **Wizard locks survive UNEDITED.** The W1 GET-only lock and the two
  no-write string locks in `extraction_wizard_api.rs:325-371` stay green:
  the GET module is only appended to (the mint), and the apply handler plus
  the POST registration live in NEW files. The server lock's helper
  (`:317-323`) keeps only lines containing the manifest or dry-run path,
  and the apply line contains neither.
- **W2b error family uniform.** Check order in the handler: no applier
  installed -> 503 `analysis_loading`; applier epoch != live epoch -> 409
  `stale_generation` with the live epoch; then the trait's answer. A
  missing, absent or unparseable request body is the `dry_run_required`
  refusal, never an axum-extractor 4xx.
- **ASCII bodies (A9).** Every response body of this route - the 200 and
  every refusal - is pure ASCII, asserted in the new test file.
- **State seam discipline.** `UiState::new_loading` / `new_loaded` keep
  their signatures and start the field at `None`, so no existing call site
  changes (the W-2b discipline).
- **Additive DTO rule.** Every new field carries `#[serde(default)]` and is
  serde-plain: no `ariadne_validate`, `ariadne_transform` or
  `ariadne_cluster` type crosses into the ui crate.

### The contract, verbatim from shape C2/C3 (binding)

```rust
pub trait ExtractionPlanApplier: Send + Sync + 'static {
    fn epoch(&self) -> u64;
    /// Single-use approval token bound to (plan_id, epoch); None when the
    /// plan does not resolve in this generation.
    fn mint_apply_token(&self, plan_id: &str) -> Option<String>;
    fn apply(
        &self,
        plan_id: &str,
        token: &str,
    ) -> Result<ExtractionApplyOutcomeDto, ExtractionApplyRefusal>;
}

pub enum ExtractionApplyRefusal {
    UnknownPlan,
    StaleGeneration,
    NotApproved,    // missing, unknown, replayed or foreign-plan token
    InProgress,     // another apply holds this generation's lock
    Failed(String), // transaction error; the tree was rolled back (AR-04)
}
```

| Condition | Status | Body |
|---|---|---|
| No applier installed (mid-swap window) | `503` | `{"error":"analysis_loading"}` |
| Applier epoch != live epoch | `409` | `{"error":"stale_generation","epoch":<live>}` |
| Plan id unknown to the applier | `404` | `{"error":"unknown_plan","plan_id":"<id>"}` |
| No valid single-use approval token | `428` | `{"error":"dry_run_required","plan_id":"<id>"}` |
| Another apply holds the generation's lock | `409` | `{"error":"apply_in_progress","plan_id":"<id>"}` |
| Transaction error (tree already rolled back) | `500` | `{"error":"apply_failed","plan_id":"<id>","detail":"<ascii>"}` |
| Committed or rolled back (both complete) | `200` | `ExtractionApplyOutcomeDto` |

DTOs (additive, in `dto/extraction.rs`, every field `#[serde(default)]`):
`ExtractionApplyRequestDto { apply_token: String }`,
`ExtractionTierResultDto { tier: String, status: String, detail: String }`,
`ExtractionApplyOutcomeDto { plan_id: String, generation_epoch: u64,
committed: bool, tiers: Vec<ExtractionTierResultDto>, new_files:
Vec<String>, output_text: String }`, and exactly one new field on the
existing `ExtractionDryRunDto`: `apply_token: String` ("minted by the
dry-run route; empty without an applier"). Tier names mirror
`status` is "pass"/"fail"/"skipped". The tier strings are
`ValidationTier`'s serde names verbatim - "Structural", "GeneratedHeaders",
"AffectedTUs" (`crates/validate/src/lib.rs:16-21`, stability locked by
`status_serde_names_stable_and_skipped_is_additive` at :840-857). The cli
applier (A-4) owns the `ValidationStatus` -> `status` + `detail` mapping and
the ASCII sanitization; this hand owns only the DTO shape.

`428 Precondition Required` is chosen over folding the approval gate into
409 so 409 keeps its single meaning (stale generation) across the whole
route family.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- tasks/extraction-auto-apply-a3-journal.md (checkpoint journal)
- crates/ui/src/routes/api_extraction_apply.rs (NEW: trait, refusal enum,
  handler, error responses - the deliberate twin of `api_extraction.rs`)
- crates/ui/src/routes/mod.rs (ONE line: `pub(crate) mod api_extraction_apply;`
  after the `api_extraction;` line)
- crates/ui/src/dto/extraction.rs (the three apply DTOs + the one
  `apply_token` field)
- crates/ui/src/state.rs (the applier field, `install_extraction_applier`,
  the accessor - the twin of the reader seam)
- crates/ui/src/server.rs (ONE route line + the `post` import + the handler
  in the existing `use crate::routes::api_extraction::{...}` line becomes
  two `use` lines or one widened line)
- crates/ui/src/routes/api_extraction.rs (the dry-run mint: a few lines)
- crates/ui/src/lib.rs (ONE line:
  `pub use routes::api_extraction_apply::{ExtractionApplyRefusal, ExtractionPlanApplier};`)
- crates/ui/tests/ui_test/extraction_apply_api.rs (NEW: the route contract
  locks)
- crates/ui/tests/ui_test/main.rs (ONE mod line: insert
  `mod extraction_apply_api;` immediately after `mod extraction_wizard_api;`
  and before `mod extraction_wizard_render;` - a fixed site for
  review-diff stability)
- crates/ui/tests/ui_test/extraction_wizard_api.rs (C8 amendment 4 ONLY:
  `apply_token: String::new(),` inside `sample_dry_run_dto` at :99-115.
  Nothing else in that file changes)
- crates/cli/src/commands/ui/extraction_reader.rs (the DECLARED 2-line
  rider: `apply_token: String::new(),` in the two struct literals at :182
  and :417)

You MUST NOT touch: any card test file, `extraction_wizard_render.rs`,
`crates/ui/assets/**`, `crates/cli/src/commands/transform.rs`,
`crates/cli/src/commands/ui.rs`, any other `crates/cli/**` file,
`tools/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`. No `git stash`, no
branch change.

### Declared cross-crate rider (shape section 5, the F14 rule)

The additive `apply_token` field breaks exactly two struct literals outside
this hand's home crate, in `crates/cli/src/commands/ui/extraction_reader.rs`
at `:182` (`compute_dry_run`) and `:417` (`honest_absent_dry_run`). Both are
construction sites with no `..Default::default()`, so both must gain the
field. F14 caller census at HEAD `023558c`, verified by the planner:
`grep -rn "ExtractionDryRunDto {" ` finds exactly FOUR sites - the struct
definition (`dto/extraction.rs:131`), the two cli literals above, and the
test sample (`extraction_wizard_api.rs:100`). Re-run that grep at preflight;
a fifth site is a STOP-and-report, not a silent fix. The rider is proved by
gate 4 (`cargo check -p ariadne-cli --all-targets`).

## Frozen locks

- The 18 card-family locks stay byte-identical: this hand's diff must not
  touch `extraction_card_story.rs` or `extraction_card_playbook.rs` (their
  two C8 amendments are A-5's).
- `extraction_wizard_api.rs` stays byte-identical APART FROM the single
  C8 amendment 4 line. The reviewer diffs that file hunk by hunk against the
  table below.
- `extraction_wizard_render.rs`, `static_html.rs`, `static_html_sections.rs`,
  `proposals_payload.rs`, `export_order.rs`, `dto_builders.rs`,
  `theme_contract.rs`, `theme_js_zero_hex.rs`, `theme_dark_grey_palette.rs`,
  `extraction_contrast.rs`, `preview_hypothetical.rs`, `icon_asset.rs`,
  `asset_injection_hardening.rs`, `help_center_readability.rs`: all
  BYTE-IDENTICAL.
- `tests/golden/**` byte-locked (`UPDATE_GOLDENS=1` FORBIDDEN);
  `tests/fixtures/**` byte-frozen; the boot payload bytes (the W9 capture in
  `extraction_reader.rs` tests) unchanged - the apply DTOs are route
  payloads, never boot payload fields.
- `tools/theme_token_baseline.json` diff-empty; zero added hex (no asset is
  touched).
- The localhost bind default: `resolve_bind_addr` and `--expose` appear in
  no diff of this hand, and `crates/cli/src/commands/ui.rs` is not touched
  (A8).

## What to build

1. `routes/api_extraction_apply.rs`: the module doc (naming the read-only
   twin and the mutation-delegation rule), the trait, the refusal enum, the
   handler, the tolerant body parse (`axum::body::Bytes` plus
   `serde_json::from_slice::<ExtractionApplyRequestDto>`; a missing or
   unparseable body yields the `dry_run_required` refusal), the applier
   accessor and the live-epoch check, and one error-response helper per
   refusal shape. Map EVERY refusal variant, including `StaleGeneration ->
   409 stale_generation` (defensive second line behind the route's own
   epoch check; it also keeps the variant constructible so no dead-code
   warning fires).
2. The apply runs inside `tokio::task::spawn_blocking` (the `spawn_watcher`
   precedent at `ui.rs:663`), so a multi-second transaction never blocks
   the async runtime; the `Send + Sync` trait bound makes the `Arc` movable.
   The handler awaits the blocking join and maps the outcome to the
   response.
3. `dto/extraction.rs`: the three DTOs and the one new field, each with a
   doc comment naming the milestone key and the plain-display-data rule.
4. `state.rs`: the field, `install_extraction_applier`, and the accessor,
   with doc comments mirroring the reader seam's; both constructors start
   the field at `None`.
5. `server.rs`: the ONE route line
   `.route("/api/extraction/:plan_id/apply", post(api_extraction_apply))`
   placed immediately after the dry-run line, `post` added to the
   `use axum::routing::{...}` line, and the handler named in the
   api_extraction `use` line.
6. `routes/api_extraction.rs`: after a `Some` dry-run answer, fetch the
   installed applier; when it exists AND its epoch equals the live epoch,
   call `mint_apply_token(&plan_id)` and place the value in
   `dry_run.apply_token`. Without an applier, or on an epoch mismatch, the
   field stays empty. Do NOT change the manifest route, the check order, or
   any error body.
7. The rider and the C8 amendment 4 line, as listed in the touch-set.

## RGB protocol (per module, in this file order)

Red: land the compile-complete skeleton FIRST, then the tests. The
skeleton is: the trait, the refusal enum, all DTOs, the state field +
install + accessor, the POST registration, and a handler whose ONLY
implemented branch is the no-applier 503 (every other request gets the
`dry_run_required` refusal). The `main.rs` mod line, the C8 amendment 4
line and the two rider lines are part of this compile-complete state - they
are prerequisites, NOT part of the Red (without them the whole ui suite
fails to compile and there is no Red at all).

Then author `extraction_apply_api.rs` and run gate 2. Expected failures,
verbatim shape (every response is `503 {"error":"analysis_loading"}` or the
placeholder `428`):

- `test_apply_route_returns_200_with_a_valid_token`: `expected status 200
  for an approved apply, got 428`
- `test_apply_route_refuses_an_unknown_plan`: `expected 404 for a plan the
  applier does not know, got 428`
- `test_apply_route_refuses_a_stale_generation_applier`:
  `expected 409 for an applier whose epoch differs from the live epoch,
  got 428`
- `test_apply_route_refuses_while_another_apply_holds_the_lock`:
  `expected 409 apply_in_progress, got 428`
- `test_apply_route_reports_a_failed_transaction`:
  `expected 500 for a transaction error, got 428`
- `test_dry_run_route_mints_an_apply_token_with_a_live_applier`:
  `the dry-run 200 body must carry the minted apply_token`

The 503 test and the five token-refusal tests (required tests 1-6) pass at
Red by construction - the placeholder IS the 428 refusal - and that is the
documented order, not a weak Red. The `extraction_wizard_api.rs` suite is
untouched and stays green throughout: its dry-run 200 test installs no
applier, so the field stays empty and its byte-equality assertion holds.

Green: implement the handler branches in the C3 check order, the mint, and
the `spawn_blocking` wrap. Re-run until exit 0.

Refactor: one helper per error body; the handler stays under 100 lines;
`routes/api_extraction_apply.rs` stays small (~150-200 lines).

## Required tests (in `crates/ui/tests/ui_test/extraction_apply_api.rs`)

The new module defines its OWN `assert_ascii_body` and request helpers (the
ones in `extraction_wizard_api.rs` are private to that module, and editing a
frozen wizard test file to share them is not allowed). Use
`common::ready_state` / `common::loading_state` from `crate::common`
exactly as the wizard route tests do.

1. `test_apply_route_returns_503_analysis_loading_with_no_applier` -
   loading state, no applier: 503 + exact body + ASCII.
2. `test_apply_route_returns_428_when_the_body_is_absent` - live applier,
   no body at all.
3. `test_apply_route_returns_428_when_the_body_is_unparseable` - live
   applier, a non-JSON body (proves the tolerant parse, not an axum 4xx).
4. `test_apply_route_returns_428_when_the_token_is_empty` - body
   `{"apply_token":""}`.
5. `test_apply_route_returns_428_for_an_unknown_token` - a scripted applier
   whose `apply` answers `NotApproved`.
6. `test_apply_route_returns_428_for_a_foreign_plan_token` - the token was
   minted for a different plan id (the scripted applier answers
   `NotApproved`); the route must not consult the applier twice.
7. `test_apply_route_returns_200_with_a_valid_token` - the scripted
   applier answers `Ok(scripted_outcome)`; assert the full DTO body: the
   three tier rows, `committed`, `new_files`, `generation_epoch` and the
   verbatim `output_text`.
8. `test_apply_route_returns_404_for_a_plan_the_applier_does_not_know` -
   `Err(UnknownPlan)` -> 404 with the plan id echoed.
9. `test_apply_route_returns_409_stale_generation_for_a_stale_applier` -
   the W6 injection recipe: `UiState::new_loading` + one `install_analysis`
   (live epoch 1) + an applier with `epoch() == 0`; assert 409 and
   `"epoch":1` in the body (shape A3).
10. `test_apply_route_returns_409_apply_in_progress_while_the_lock_is_held` -
    `Err(InProgress)` -> 409 `apply_in_progress` with the plan id.
11. `test_apply_route_returns_500_apply_failed_on_a_transaction_error` -
    `Err(Failed("rollback restored the tree"))` -> 500 with the plan id and
    the ASCII detail.
12. `test_dry_run_route_mints_an_apply_token_with_a_live_applier` - install
    an applier whose epoch equals the live epoch alongside the reader; the
    dry-run 200 body's `apply_token` must equal the scripted token.
13. Lock (A1): `routes/api_extraction_apply.rs` contains none of
    `fs::write`, `fs::remove_file`, `fs::create_dir`, `ArtifactManager`.
14. Lock (A1/C3): `server.rs` registers the apply path exactly once and
    that line contains `post(`, and no line mentioning the manifest or
    dry-run path contains any mutation verb.
15. Lock (W1 survival): `routes/api_extraction.rs` still contains no write
    token and no mutation-route registration.
16. Lock (A9): every response body exercised above (200 plus all six
    refusal shapes) is asserted pure ASCII.

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned identifier names, no
single-letter locals, intention-revealing names (shape section 9's name
list is binding for every new item). Measured at HEAD `023558c`:
`routes/api_extraction_apply.rs` NEW ~150-200; `dto/extraction.rs` 166 ->
~250; `state.rs` 241 -> ~270; `server.rs` 84 -> ~87; `lib.rs` 29 -> 30;
`routes/mod.rs` 7 -> 8; `routes/api_extraction.rs` 103 -> ~112;
`extraction_apply_api.rs` NEW ~300-360; `main.rs` 34 -> 35. No file
approaches 1000. `python tools/hygiene_check.py` exit 0.

## Frozen locks (proof commands)

| Lock | Proof |
|---|---|
| Card test files byte-identical | reviewer per-commit diff; gate 3's suite green with them unedited |
| `extraction_wizard_api.rs` one-line amendment only | reviewer hunk-by-hunk diff against the C8 table |
| GET-only + no-write wizard locks | required tests 14-15 |
| Goldens / fixtures / boot payload | touch-set check + no commit touches them |
| Bind default unchanged (A8) | `crates/cli/src/commands/ui.rs` is outside the touch-set |

## Gates (exact commands, timeout 600000 ms each; scoped only)

1. `python tools/testrun.py --distill -p ariadne-ui` (baseline canonical
   line recorded verbatim in the journal)
2. `python tools/testrun.py --distill -p ariadne-ui` (Red: exit 1; exactly
   the six named failing tests, verbatim panic lines, in the journal)
3. `python tools/testrun.py --distill -p ariadne-ui` (Green: exit 0;
   canonical line = baseline + the new tests, no other movement)
4. `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`
   (exit 0: the declared 2-line rider; F14 standing rule 2, reverse
   dependency 1 of 2)
5. `python tools/distill.py -- cargo check -p ariadne-integration-tests
   --all-targets` (exit 0: F14 standing rule 2, reverse dependency 2 of 2 -
   `tests/integration/Cargo.toml:25` declares `ariadne-ui`; the census shows
   zero textual references to the apply DTOs today, so the check is the
   proof)
6. `python tools/hygiene_check.py` (exit 0)
7. `python tools/encoding_guard.py --staged` (exit 0)
8. `python tools/commit_scoped.py --scope crates/ui/src/routes/api_extraction_apply.rs
   --scope crates/ui/src/routes/mod.rs --scope crates/ui/src/dto/extraction.rs
   --scope crates/ui/src/state.rs --scope crates/ui/src/server.rs --scope
   crates/ui/src/routes/api_extraction.rs --scope crates/ui/src/lib.rs
   --scope crates/ui/tests/ui_test/extraction_apply_api.rs --scope
   crates/ui/tests/ui_test/main.rs --scope
   crates/ui/tests/ui_test/extraction_wizard_api.rs --scope
   crates/cli/src/commands/ui/extraction_reader.rs --scope
   tasks/extraction-auto-apply-a3-journal.md -m "feat(ui): POST
   /api/extraction/{plan_id}/apply behind the dry-run-then-approve gate
   (trait seam, apply DTOs, 428/409/404/500/200 table)"` (never `git add .`)
9. `python tools/encoding_guard.py --all` (post-commit, exit 0)

Gate count: 9 (the commit-cycle trio is gates 7-9; gates 4-5 are the F14
reverse-dependency pair). Bench stance: this hand touches `crates/ui/src`
and (by rider) `crates/cli/src`, so the DOCUMENTED_PHASES dated row
applies - the producer set is byte-identical across this hand's range
(zero-diff producer proof in the journal), the timing gate is
deterministically blocked at `tools/bench_compare.py:84`, and NO timing
number is attributed to this hand. Do not run `bench_compare.py`; the
reviewer hop owns it.

## Acceptance (machine-checkable)

1. The POST route answers every row of the C3 table with the exact body,
   and every body is pure ASCII (required tests 1-12, 16).
2. The dry-run route mints a token only with a live-epoch applier, and the
   minted value is an ASCII string of the shape
   `format!("apply-{epoch}-{plan_id}-{counter}")` (the counter is A-4's
   implementation detail; the route contract here is the field's presence
   and emptiness).
3. `routes/api_extraction_apply.rs` contains no write token and no mutation
   route registration (required test 13).
4. The wizard GET-only and no-write locks are green with
   `extraction_wizard_api.rs` and `routes/api_extraction.rs`'s existing
   assertions UNEDITED (required tests 14-15 plus the untouched suite).
5. Gates 4 and 5 exit 0 (the declared rider compiles both reverse
   dependencies).
6. `python tools/hygiene_check.py` exit 0; `python
   tools/encoding_guard.py --all` exit 0.
7. The commit's touch-set is exactly the twelve files in gate 8 (reviewer
   commit-vs-touch-set enforcement).

## Discipline clauses

- UTF-8: Write/Edit tools only; never `Set-Content`/`Out-File`/`>`/
  `Get-Content | -replace`; pure-ASCII authored content; no em dashes; in
  Rust source use `&rarr;` only in doc text if a glyph is ever needed
  (none should be).
- Touch only the touch-set; out-of-scope breakage: report, do not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`) are a reviewer
  blocker: distilling entry points only. This hand edits no asset and no
  tool, so it carries no `ui_smoke` duty - the reviewer hop owns the
  runtime gate.
- Do NOT install an applier in the existing dry-run 200 test of
  `extraction_wizard_api.rs` (it is frozen): a minted token there would
  break its byte-equality assertion. The mint's own test is required test
  12 in the NEW file.
- Checkpoint the journal after each Green gate (module, commit hash,
  deviation, lock finding).
- Context exhaustion: STOP with a disk-state inventory; successor resumes
  via `python tools/checkpoint_guard.py resume --spec
  tasks/extraction-auto-apply-a3-handoff.md --scope ... --cmd ...`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence (the six named failing
   tests with verbatim panic lines). 3. Green evidence (distill digest
   header line + canonical line + capture path; gates 4 and 5 exit codes).
   4. Deviations + reasons. 5. Out-of-scope observations. No raw tool
   output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill; its commit-vs-touch-set check enforces the touch-set above exactly.
