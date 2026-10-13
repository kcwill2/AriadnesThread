# Milestone Shape - extraction-auto-apply

Companion to `tasks/extraction-auto-apply-kickoff.md` (key verbatim
there). Read-only survey at HEAD `023558c` on `main`, working tree clean.
Pure ASCII throughout, per the file-edit discipline. Every line count,
route list and claim below was measured on disk in the shaping session;
every claim that a capability is absent cites the file and line that
proves it. Contract of record inherited from `ui-extraction-wizard`:
wizard shape section 4 and the wizard plan's "Auto-extract v1 boundary
vs deferred" section, both quoted where they bind.

## 1. Capability map (the honest inventory)

Legend: EXISTS = shipped and reachable; PARTIAL = shipped but capped,
wrong owner, or wrong process; ABSENT = no code path today.

| # | Capability the apply flow needs | State | Where (file:line) |
|---|---|---|---|
| 1 | Transactional apply: snapshot, write generated units, rewrite consumers, strip moves, return created paths | EXISTS, CLI only | `apply_transform_plan` `crates/cli/src/commands/transform.rs:129-192`; `TransactionApplier::apply_with` `crates/validate/src/lib.rs:150-201` (writes `:208-274`, strip `:382-395`) |
| 2 | Three-tier validation + rollback | EXISTS, CLI only | `commit_or_rollback` `transform.rs:341-397`; tiers `crates/validate/src/lib.rs:400-489`; rollback `:297-321` |
| 3 | Apply outcome as structured data | PARTIAL | `ApplyOutcome` is `Serialize` (`crates/validate/src/lib.rs:47-56`) with `format_output` (`:58-81`), but the CLI both builds and prints it (`transform.rs:392-393`, `:411`); no in-process return path exists |
| 4 | A UI route that mutates anything | ABSENT | every registration in `crates/ui/src/server.rs:50-84` is `get(...)` |
| 5 | Host-implemented mutation seam in the ui crate | ABSENT | the shipped seam is the read-only `ExtractionManifestReader` (`crates/ui/src/routes/api_extraction.rs:30-34`) |
| 6 | Plan-by-id resolution against the live generation | EXISTS, read-only | the cli reader holds the generation's plans (`crates/cli/src/commands/ui/extraction_reader.rs:44-51`), `plan_for` `:79-81`; installed in `swap_into_state` (`crates/cli/src/commands/ui.rs:606-623`) |
| 7 | Dry-run computed by the apply's own code path | EXISTS | provider `compute_dry_run` `extraction_reader.rs:159-192` shares `plan_rewrite_targets` + consumer discovery with the CLI preview (C4 of the wizard shape) |
| 8 | Generation epoch + staleness refusal | EXISTS, GET surface only | `analysis_epoch` `crates/ui/src/state.rs:84-85`, `:162-177`; 409 refusal `api_extraction.rs:67-78` |
| 9 | Per-generation apply serialization | ABSENT | no lock exists anywhere near the apply path |
| 10 | Approval gate (proof of a prior dry-run fetch) | ABSENT | the GET dry-run returns data with no server-side state tying a later request to it |
| 11 | Post-apply rebuild | PARTIAL | the fingerprint watcher rebuilds on input change (`crates/cli/src/commands/ui.rs:638-708`; inputs per `crates/pipeline/src/lib.rs:366-371,380-382`); nothing rebuilds on an explicit post-apply request; interval defaults to 2s (`crates/cli/src/lib.rs:250-252`) |
| 12 | UI outcome reporting (tiers, verdict, rollback) | ABSENT | S6 is static copy (`crates/ui/assets/extraction-wizard.js:407-416`) |
| 13 | Help copy stating the apply claim | ABSENT | the copy states the opposite (`crates/ui/assets/index.html:548`, `:551`) and three locks pin that wording (`extraction_card_playbook.rs:214-217`, `:303-306`; `extraction_wizard_render.rs:588-591`) |
| 14 | `ui.rs` headroom for the install wiring | ABSENT | 986/1000 lines (`crates/cli/src/commands/ui.rs`; the deferred W-2a F2 row) |

Consequence: rows 3, 4, 5, 9, 10 and 14 are the contract work; rows 1,
2, 6, 7 and 8 are the reuse. The shape therefore promises one
transactional core shared by CLI and UI, a token-gated POST route,
serialized applies, honest outcome reporting, and an explicit
extraction-first discharge of the `ui.rs` ceiling.

## 2. Invariants (each independently checkable; oracle named)

- **A1 Single write path.** Every UI-triggered mutation executes through
  `TransactionApplier::apply_with` via the extracted `apply_plan_with`
  core, and the ui crate's apply route module contains no `fs::write`,
  `fs::remove_file`, `fs::create_dir` or `ArtifactManager` token.
  Oracle: string locks over `crates/ui/src/routes/api_extraction_apply.rs`
  in `crates/ui/tests/ui_test/extraction_apply_api.rs` (the same idiom
  the W1 locks use over the GET module), plus a caller census proving the
  cli applier has exactly one delegation site into the core
  (`grep -c "apply_plan_with(" crates/cli/src` at the dispatch base,
  expected 2: the definition plus the CLI wrapper), plus the m10/m20/m23
  integration apply suites green UNEDITED.
- **A2 Dry-run-then-approve gate.** An apply is accepted only with a
  single-use token minted by the GET dry-run route for the same plan id
  and the current generation epoch. A missing, unknown, replayed or
  foreign-plan token is refused; a replayed token cannot apply twice.
  Oracle: `extraction_apply_api.rs` route tests for each refusal shape,
  plus a provider test that a consumed token refuses a second apply.
- **A3 Epoch binding.** The applier's epoch must equal the live
  `analysis_epoch` (409 `stale_generation`), and the token carries that
  epoch, so a generation swap discards every outstanding approval.
  Oracle: a route test with a stale-epoch applier injected (the wizard
  W6 injection recipe), plus a provider test that a token minted for one
  epoch fails against another.
- **A4 One apply at a time.** A second apply while the generation's apply
  lock is held is refused with `apply_in_progress` and does not consume
  the approval token, so an honest retry is possible. Oracle: a provider
  test that pre-holds the lock, asserts the refusal, then applies
  successfully with the same token.
- **A5 Honest outcome plus rollback safety.** The outcome DTO carries the
  three tier results, the committed/rolled-back verdict and the verbatim
  `ApplyOutcome::format_output` text; a rolled-back apply leaves the tree
  byte-identical to its pre-apply fingerprint. Oracle: a route test
  asserting the 200 body against a scripted outcome, plus a cli-side
  provider test with a fabricated extraction that fails tier 2, asserting
  the rollback path through `TransactionApplier::verify_restored`.
- **A6 Dry-run/apply parity.** On the same generation, the files an
  apply writes are exactly the approved dry-run's `new_files` under the
  same plugin staging decision, and the consumer rewrites match. Oracle:
  the fixture end-to-end test (apply on an isolated fixture copy; compare
  the outcome's file set against the dry-run response field by field).
- **A7 Approval surface honesty.** The Execute step's apply control is
  enabled only when a dry-run approval token is present in the wizard
  state; without one the step renders the dry-run-first instruction and
  no apply request is possible. Oracle: a renderer lock on the enable
  condition plus the refusal copy in `extraction_wizard_render.rs`.
- **A8 Bind default unchanged.** No route, handler or flag widens the
  exposure surface: `resolve_bind_addr` and `--expose` appear in no diff
  of this milestone. Oracle: touch-set inspection per hand plus the
  existing binding behavior tests green unedited.
- **A9 ASCII discipline.** Every apply response body (token, tier rows,
  output text, refusal details) is pure ASCII. Oracle:
  `assert_ascii_body` over the 200 and refusal bodies in
  `extraction_apply_api.rs`, plus `encoding_guard` exit 0 at every hop.
- **A10 Lock survival or named amendment.** 16 of the 18 card-family
  locks survive byte-identical and exactly 2 are amended per C8; the
  wizard GET-only and no-write locks in `extraction_wizard_api.rs`
  survive UNEDITED. Oracle: reviewer per-commit diff of
  `extraction_card_story.rs` and `extraction_card_playbook.rs` against
  the start rev, plus the string locks staying green without edits.

## 3. Contracts

### C1 The apply core (crates/cli/src/commands/transform.rs)

One transactional core, two thin callers. Extraction of the existing
`apply_transform_plan` body (`transform.rs:129-192`); no behavior change
to the CLI verb.

```rust
/// One apply transaction's report: the tier outcome plus the files the
/// transaction created. Returned, never printed - the CLI wrapper prints
/// it, the UI route serializes it.
pub struct ApplyTransactionReport {
    pub outcome: ariadne_validate::ApplyOutcome,
    pub new_files: Vec<PathBuf>,
}

/// The transactional core every caller shares: engine extraction over
/// `reduced_model`, the apply-identical rewrite targets, consumer
/// discovery, `TransactionApplier::apply_with`, the three tiers, and
/// commit-or-rollback with artifact records. `reduced_model` is a
/// parameter on purpose: the CLI wrapper loads the artifact model
/// (today's behavior) while the UI applier passes the generation's
/// in-memory model - the exact model the approved dry-run was computed
/// from (A6).
pub fn apply_plan_with(
    repo_path: &PathBuf,
    plan: &ariadne_cluster::plan::OptimizationPlan,
    reduced_model: &ariadne_graph::ReducedSemanticModel,
) -> Result<ApplyTransactionReport>
```

`apply_transform_plan` keeps its exact signature
(`repo_path: &PathBuf, plan_id: &str`) and becomes: locate repository,
prepare artifacts, `load_plan_for_action` (the unknown-plan error text
"Plan {} not found" is preserved for the m20 lock), load the reduced
model, print `Applying {}...`, call the core, print
`outcome.format_output()`. Success-path stdout is byte-identical; on an
error path the "Applying" line now precedes the error text (the m10/
m20/m23 suites assert error message content and stdout of the success
path only - the implementer verifies at the gate).

### C2 The applier trait and state seam (crates/ui)

New module `crates/ui/src/routes/api_extraction_apply.rs`, mirroring the
read-only seam so the ui crate never names a write:

```rust
/// The mutation seam, the deliberate twin of the read-only
/// `ExtractionManifestReader`: the host (cli) implements it; the ui crate
/// names no write. Ledger invariants encoded: every mutation goes through
/// the existing TransactionApplier path (the implementation delegates to
/// `apply_plan_with`), one apply at a time, token-gated, epoch-bound.
pub trait ExtractionPlanApplier: Send + Sync + 'static {
    fn epoch(&self) -> u64;
    /// Single-use approval token bound to (plan_id, epoch); None when the
    /// plan does not resolve in this generation.
    fn mint_apply_token(&self, plan_id: &str) -> Option<String>;
    /// The gated apply. Every refusal is a value, never a panic.
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

State seam (`crates/ui/src/state.rs`), the twin of the reader field
(`state.rs:97-98`) with install and accessor twins (`:191-199`):

```rust
extraction_applier:
    std::sync::RwLock<Option<Arc<dyn ExtractionPlanApplier>>>,
```

`install_extraction_applier` is called by the host in `swap_into_state`
immediately after `install_extraction_reader`, inside the same bounded
window the payload write documents (`crates/cli/src/commands/ui.rs:610-623`).
`UiState::new_loading`/`new_loaded` keep their signatures (the field
starts `None`), so no existing test call site changes - the same
discipline W-2b used for the reader.

### C3 The apply route (server.rs + the new module)

`POST /api/extraction/:plan_id/apply`, registered on its own line in
`crates/ui/src/server.rs`. The GET-only server lock survives unedited:
its helper keeps only lines containing the manifest or dry-run path
(`crates/ui/tests/ui_test/extraction_wizard_api.rs:317-323`), and the
apply line contains neither.

Request body, parsed tolerantly - an absent or unparseable body is the
`dry_run_required` refusal, never an axum-extractor 4xx, so the error
family stays uniform:

```json
{"apply_token": "<token minted by the GET dry-run route>"}
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

`428 Precondition Required` is chosen over folding this into 409 so 409
keeps its single meaning (stale generation) across the whole route
family; the front-end maps each code to its own copy. The apply runs
inside `tokio::task::spawn_blocking` (the `spawn_watcher` build
precedent, `ui.rs:663`), so a multi-second transaction never blocks the
async runtime; the trait bound `Send + Sync` makes the `Arc` movable.

Response DTOs (`crates/ui/src/dto/extraction.rs`, additive, serde-plain,
`#[serde(default)]` on every field - the module's plain-display-data rule
keeps validate/transform types out of the ui crate):

```rust
pub struct ExtractionApplyRequestDto {
    #[serde(default)]
    pub apply_token: String,
}

pub struct ExtractionTierResultDto {
    #[serde(default)]
    pub tier: String,   // "Structural" | "GeneratedHeaders" | "AffectedTUs"
    #[serde(default)]
    pub status: String, // "pass" | "fail" | "skipped"
    #[serde(default)]
    pub detail: String, // fail/skip reason; empty on pass
}

pub struct ExtractionApplyOutcomeDto {
    #[serde(default)]
    pub plan_id: String,
    #[serde(default)]
    pub generation_epoch: u64, // the epoch the approval was minted on
    #[serde(default)]
    pub committed: bool,
    #[serde(default)]
    pub tiers: Vec<ExtractionTierResultDto>,
    #[serde(default)]
    pub new_files: Vec<String>, // files the transaction created, lossy
    #[serde(default)]
    pub output_text: String,    // ApplyOutcome::format_output(), verbatim
}
```

Tier names mirror `ValidationTier`'s serde names (stability locked by
`status_serde_names_stable_and_skipped_is_additive`,
`crates/validate/src/lib.rs:840-857`); the cli applier maps
`ValidationStatus` into `status` + `detail` and sanitizes every string to
ASCII (A9).

`ExtractionDryRunDto` gains exactly one additive field:

```rust
#[serde(default)]
pub apply_token: String, // minted by the dry-run route; empty without an applier
```

### C4 Token semantics (the gate of record)

- Mint point: the GET dry-run route, after a `Some` answer, calls
  `applier.mint_apply_token(&plan_id)` and places the value in the
  response's `apply_token` field. `routes/api_extraction.rs` grows by a
  few lines (state accessor plus the field set); none of its four
  forbidden tokens appears, so the W1 string locks stay green unedited.
- Value: deterministic ASCII, epoch-bound and registry-keyed:
  `format!("apply-{epoch}-{plan_id}-{counter}")` with a per-applier
  `AtomicU64` counter. It is a gate, not a credential: the server has no
  authentication by design and stays on the localhost bind default (A8).
- Registry: `Mutex<HashMap<String, (String, u64)>>` inside the applier,
  keyed token to (plan_id, epoch). An epoch swap installs a new applier
  and therefore discards every outstanding token.
- Single use: consumption happens atomically when the apply transaction
  starts. A lock refusal (`InProgress`) does not consume the token, so an
  honest retry works; a successful apply consumes it, so a second apply
  requires a fresh dry-run fetch.

### C5 Mutex and epoch model

- The applier holds `apply_lock: std::sync::Mutex<()>`; the blocking
  closure `try_lock`s it. A held lock yields `InProgress` immediately -
  applies are never queued, and a queued apply would be an unapproved
  mutation waiting to happen.
- Per-generation by construction: each generation installs its own
  applier instance, and only the live epoch can apply (409 otherwise), so
  within the live generation applies are strictly serial.
- Epoch swap mid-apply: the in-flight transaction is not cancelled -
  `TransactionApplier` owns the tree through its snapshot (AR-04) - and
  the response still reaches the client carrying the approved epoch. Any
  later apply hits 409 `stale_generation`, because the new applier's
  registry has no token from the old generation.

### C6 Post-apply rebuild

- No new route and no rebuild channel. The shipped fingerprint watcher
  (`crates/cli/src/commands/ui.rs:638-708`) re-hashes discovery, build
  contexts and TUs (`crates/pipeline/src/lib.rs:366-371,380-382`), which
  cover the files an apply rewrites; the next tick (default 2s,
  `crates/cli/src/lib.rs:250-252`) rebuilds and swaps a new generation
  epoch, which in turn invalidates every outstanding token.
- The apply response therefore needs no rebuild flag; the S6 copy states
  the truth: the background watcher re-analyzes and the wizard resets
  when the new analysis lands (the existing payload-swap reset,
  `extraction-wizard.js:418-446`). When the server runs with watching
  disabled the copy says the analysis refreshes on restart - the
  front-end does not claim a rebuild it cannot know happened.
- The outcome record is also written under `artifacts/transaction` by
  the core (`transform.rs:394`, `:413`), so the verdict outlives the
  wizard reset.

### C7 Front-end steps S5/S6 and the help-copy flip

S5 Execute (`renderWizardStepExecute`, `extraction-wizard.js:395-405`):
keeps the three-verb recipe (the W5 verb-count lock,
`extraction_wizard_render.rs:543-581`, stays satisfiable - no new `tool `
string is introduced), drops "The UI shows this recipe and never runs
it", and gains:

- an apply control enabled iff `wizardState.dryRun.apply_token` is
  non-empty (A7); disabled state carries the dry-run-first instruction;
- an in-flight state that disables the control and shows progress copy;
- response mapping: 200 stores the outcome and advances to S6; 409 rides
  the existing stale-generation reset (`:231-237`); 428 shows
  "re-open the dry run to refresh the approval"; 409 `apply_in_progress`
  shows "another apply is running; try again in a moment".

S6 After apply (`renderWizardStepAfterApply`, `:407-416`): when
`wizardState.outcome` exists, render the tier rows (name, PASS/FAIL/
SKIPPED plus reason), the verdict line, the created files, and the
verbatim `output_text` through the shared escaper; keep the descriptive
copy when no outcome exists (the step stays enterable before an apply).
Every interpolated value passes `escapeHtml` (the raw-interpolation ban
at `extraction_wizard_render.rs:596-608`).

Help copy flip (`crates/ui/assets/index.html` legend tab, lines 548 and
551) to the claim of record: "the UI applies with a dry-run first,
validation in three tiers and automatic rollback". The flipped sentences
keep the milestone key `extraction-auto-apply` as provenance, so the two
key-presence assertions (`extraction_card_playbook.rs:218-221`,
`extraction_wizard_render.rs:592-595`) stay byte-identical.

Wizard state (C7 of the wizard shape) gains `applying` and `outcome`,
cleared by every existing reset path (different plan, 409 stale, payload
swap) because an outcome belongs to the generation that produced it.

### C8 Lock amendment table (complete; no other test file changes)

Card family (the 18 locks: 9 in `extraction_card_story.rs`, 9 in
`extraction_card_playbook.rs`):

Amended (2, each with a rationale comment in its diff):

1. `extraction_card_playbook::test_apply_recipe_is_honest` - the legend
   assertion `legend.contains("never applies")` (`:214-217`) flips to the
   new claim; the verb-count and `extraction-auto-apply` assertions in
   the same test survive.
2. `extraction_card_playbook::test_help_playbook_copy` - the second
   `legend.contains("never applies")` (`:303-306`) flips the same way;
   the "three tiers"/"rollback"/"collapsed summary" assertions survive.

Survive byte-identical: the remaining 16 card-family locks (including
all 9 in `extraction_card_story.rs`), plus `proposals_payload.rs`,
`static_html.rs`, `static_html_sections.rs`, `help_center_readability.rs`,
`export_order.rs`, `dto_builders.rs`, `theme_contract.rs`,
`theme_dark_grey_palette.rs`, `theme_js_zero_hex.rs`,
`extraction_contrast.rs`, `preview_hypothetical.rs`, `icon_asset.rs`,
`asset_injection_hardening.rs`, and the wizard-era locks in
`extraction_wizard_api.rs` (the GET-only and no-write string locks stay
unedited by design - the apply handler and POST registration live in new
files).

Wizard-era amendments (2, in the wizard's own files, recorded here
because the freeze set names them):

3. `extraction_wizard_render::test_wizard_help_copy_names_the_execution_boundary`
   - the `never mutates the repository` assertion (`:588-591`) flips to
   the new claim; the `extraction-auto-apply` assertion survives.
4. `extraction_wizard_api.rs` sample constructors (`:99-115`) gain the
   one-line `apply_token: String::new()` field default; the assertions
   and route tests are otherwise untouched.

## 4. Auto-extract scope boundary

**In this milestone (the deferred scope of record, discharged):**

- `POST /api/extraction/{plan_id}/apply` behind the dry-run-then-approve
  gate on the current generation epoch.
- Three-tier outcome and rollback reporting carrying
  `ApplyOutcome::format_output`.
- The per-generation apply mutex, and generation-epoch staleness
  enforcement beyond the GET 409 surface (applier epoch check plus token
  epoch binding plus post-swap token invalidation).
- Post-apply rebuild through the existing watcher, with honest UI copy.
- The help-copy flip from "the UI never applies" to "the UI applies with
  a dry-run first, validation in three tiers and automatic rollback".

**Out (each with its reason):**

- Remote apply or any bind widening (A8; the server has no auth and
  stays localhost).
- Background, scheduled, batch or multi-plan apply: every mutation needs
  its own fresh single-use approval.
- Any second transactional implementation: the CLI and the UI share
  `apply_plan_with`; `crates/validate/**`, `crates/transform/**` and
  `crates/artifact/**` are not edited.
- New rebuild routes, plan editing, generated-body display, planner or
  naming changes (same exclusions the wizard carried).
- The `styles.css` split and any theme-token change (zero added hex).

## 5. Module map (per hand)

| Hand | Changes | Must not change |
|---|---|---|
| A-1 (crates/cli) | `commands/ui.rs` (family moves out, re-export surface added), `commands/ui/boot_progress.rs` (receives the boot-progress family) | every other cli file; all six frozen test modules compile unedited |
| A-2 (crates/cli) | `commands/transform.rs` (`ApplyTransactionReport` + `apply_plan_with` core; wrapper keeps its signature and stdout) | `commands/ui.rs`, crates/ui, tests/ |
| A-3 (crates/ui) | NEW `routes/api_extraction_apply.rs`, `dto/extraction.rs` (apply DTOs + `apply_token` field), `state.rs` (applier seam), `server.rs` (+1 POST line), `routes/api_extraction.rs` (dry-run mints the token), `lib.rs` (+1 trait re-export), NEW `tests/ui_test/extraction_apply_api.rs`, `main.rs` (+1 mod); plus the declared cli rider below | card test files, assets, `transform.rs`, `commands/ui.rs` |
| A-4 (crates/cli) | NEW `commands/ui/extraction_apply.rs` (applier + token registry + mutex + tests), `commands/ui.rs` (`swap_into_state` install, +~14 lines) | `transform.rs`, crates/ui, `extraction_reader.rs` |
| A-5 (crates/ui) | `assets/extraction-wizard.js` (S5/S6 + state), `assets/index.html` (copy flip), `assets/styles.css` (net-capped block), the C8 amendments (2 card tests + 1 wizard render test + 2 sample lines) | routes, DTOs, provider, card test files beyond C8 |

A-3's declared rider: the new `apply_token` field on `ExtractionDryRunDto`
breaks exactly two struct literals in
`crates/cli/src/commands/ui/extraction_reader.rs` (`:182`, `:417`), so
A-3 adds `apply_token: String::new()` to both as a two-line rider outside
its home crate and proves it with `cargo check -p ariadne-cli
--all-targets` (the F14 reverse-dependency rule; the census count is in
section 8 R5).

## 6. Handoff sizing and ordering (planner input; no sequencing here)

| Hand | Crate dir | Est. produced lines | Est. gates | Kind |
|---|---|---|---|---|
| A-1 | crates/cli | ~250-350 (move + module docs + re-exports) | ~8 | refactor, hygiene-forced |
| A-2 | crates/cli | ~130-220 | ~8 | refactor, reuse seam |
| A-3 | crates/ui | ~450-600 | ~9-10 | backend contract |
| A-4 | crates/cli | ~400-550 | ~10 | apply provider |
| A-5 | crates/ui | ~350-500 | ~10 | front-end + copy flip |

Lines are estimated implementation-plus-test diff against the ~1200
bound, priced from the mandated test bodies per the F14 rule; gates are
implementer distilling-entry-point invocations including the
commit-cycle trio.

Hard edges: A-1 before A-4 (the ui.rs extraction is the headroom the
install needs; the deferred F2 row forbids spending the last 14 lines);
A-2 before A-4 (the provider delegates to the core); A-3 before A-4
(the provider compiles against the trait, DTOs and state seam); A-3
before A-5 (the front end implements the route contract); A-4 before A-5
(the execute step needs a live applier). A-1, A-2 and A-3 are
independent of each other: A-1 and A-2 are file-disjoint inside
crates/cli and A-3 is a different crate, so parallel dispatch is
possible, but the house default is strictly serial on main with one
scoped landing commit per hand (the wizard's "never parallel"
precedent). A-3 and A-5 each add a `tests/ui_test` mod line to
`main.rs`.

If A-4 exceeds its budget, the split line is inside the provider: the
token registry and mutex first, the `apply_plan_with` delegation second,
because the gate is what makes the milestone safe rather than merely
functional.

## 7. Risks (mitigation plus oracle per risk)

- **R1 Browser-side mutation of the user's repository.** This is the
  highest-blast-radius change the UI has ever carried. Mitigation: the
  ledger's invariants encoded as A1-A5, the localhost bind default
  untouched (A8), and the fixture-level end-to-end proof. Oracle: A1/A2/
  A4/A6 locks plus the apply-on-a-fixture-copy test.
- **R2 Gate bypass or replay.** Mitigation: single-use, epoch-bound,
  plan-bound tokens with consumption at transaction start; a missing or
  unparseable body is a refusal, not a crash. Oracle: A2 route tests per
  refusal plus the provider replay test.
- **R3 A long apply blocks the async runtime.** Mitigation:
  `spawn_blocking` plus `try_lock` (refuse, never queue). Oracle: the A4
  pre-held-lock provider test.
- **R4 `ui.rs` ceiling.** 986/1000 with a mandated +~14 install.
  Mitigation: A-1 is a hard predecessor and A-1's journal records the
  measured post-extraction count (priced, not predicted). Oracle:
  `hygiene_check.py` exit 0 at A-1 and A-4.
- **R5 Cross-crate compile coupling.** The `apply_token` field breaks
  struct literals outside the ui crate. Mitigation: the F14 caller
  census (exactly 3 construction sites at HEAD:
  `extraction_reader.rs:182`, `:417`, `extraction_wizard_api.rs:100`)
  plus the declared 2-line rider in A-3 gated by
  `cargo check -p ariadne-cli --all-targets`. Oracle: green cli check
  inside A-3.
- **R6 The watcher races an in-flight apply** (the apply's own mutations
  can trip the fingerprint check mid-transaction). Mitigation: the
  transaction completes on its own snapshot; the outcome carries the
  approved epoch; subsequent applies are refused 409. No test can force
  a mid-apply swap deterministically - stated plainly: this invariant is
  reasoning-locked, not test-locked.
- **R7 Amendment creep on the frozen locks.** Mitigation: C8 is the
  complete table; any further card-lock edit discovered mid-hand is a
  STOP-and-report (the wizard R6 precedent). Oracle: A10, reviewer
  per-commit diffs.
- **R8 Rollback overclaim.** A rolled-back apply must restore the tree
  byte-identically. Mitigation: the applier test drives a fabricated
  tier-2 failure through the real rollback and asserts
  `verify_restored`. Oracle: A5.
- **R9 Bench gate blocked.** The `DOCUMENTED_PHASES` tuple defect
  (`tools/bench_compare.py:84`) makes the timing gate exit 2 on every
  run. Mitigation: the dated deferred row's stance - per-hand
  zero-diff producer proofs; the optional compensating
  `--bench stage5-memory --runs 6` at A-4 (the W-2a precedent). No
  timing number is attributed to any hand.
- **R10 Model drift between dry-run and apply.** The CLI apply loads the
  reduced model from artifacts while the dry-run used the generation's
  in-memory model. Mitigation: the C1 parameterization - the UI path
  passes the generation's own model, so dry-run/apply model identity
  holds by construction for the UI. Oracle: A6.
- **R11 Raw outcome text breaks the ASCII or escape discipline.**
  Mitigation: the applier sanitizes every string to ASCII; the renderer
  escapes every interpolation. Oracle: A9 and the renderer
  raw-interpolation ban.
- **R12 Plugin-isolation guard.** The milestone adds no dependency edges
  from a plugin crate, so `m17_plugin_seam_test`'s transitive cone guard
  cannot regress. Oracle: the guard stays green UNEDITED (it is outside
  every touch set).

## 8. Hygiene notes

- `commands/ui.rs` 986 -> ~830-850 at A-1 (measured in the A-1 journal);
  `commands/ui/boot_progress.rs` 115 -> ~265. A-4's install is +~14
  lines, leaving the file well under the ceiling.
- NEW `routes/api_extraction_apply.rs` stays small (~120-160 lines):
  trait, refusal enum, handler, error responses.
- `dto/extraction.rs` 166 -> ~250; `state.rs` 241 -> ~270; `server.rs`
  84 -> ~85; `transform.rs` 415 -> ~470.
- NEW `commands/ui/extraction_apply.rs` ~350-450 lines (the
  `extraction_reader.rs` idiom, which is 839 lines with its tests).
- `extraction-wizard.js` 446 -> ~560; `index.html` 640 -> ~645; the
  styles.css block is contiguous and net-capped.
- `crates/validate/src/lib.rs` (929) is not touched; no file this
  milestone grows approaches 1000.
- New Rust test files stay under 1000 lines each; every helper under 100
  lines; no source line near 2000 characters; all authored content is
  pure ASCII (`&rarr;` entities where a glyph is wanted, per house
  idiom).
- New names reveal intent and avoid the R3 banned set:
  `ExtractionPlanApplier`, `ExtractionApplyRefusal`,
  `ExtractionApplyOutcomeDto`, `ExtractionTierResultDto`,
  `ExtractionApplyRequestDto`, `ApplyTransactionReport`, `apply_plan_with`,
  `install_extraction_applier`, `EngineBackedExtractionApplier`,
  `mint_apply_token`, `renderWizardStepOutcome`, `applyExtractionPlan`.

## 9. Freeze set and census

Frozen for the whole milestone: `tests/golden/**` (byte-locked,
`UPDATE_GOLDENS=1` forbidden), `tests/fixtures/**`, the boot payload
bytes (the W9 capture in `extraction_reader.rs` tests), the theme assets
(`themes.css`, `theme.js`, `tools/theme_token_baseline.json`, zero added
hex), `preview.js`, `api.js`, `graph.js`, `hotspots.js`, `main.js`,
`traversal.js`, `file-view.js`, `provenance.js`, and the card-family
locks except the two C8 amendments. Census anchor:
`1483 passed / 0 failed / 13 ignored / 112 suites` @ `24c2f7e`; hands
report deltas from a clean-tree scoped baseline; the `--baseline` FILE
form stays superseded by the `--distill` substitute (9th encounter).
