# Milestone Shape - ui-extraction-wizard

Companion to `tasks/ui-extraction-wizard-kickoff.md` (key verbatim there).
Read-only survey at HEAD `97dd836` on `main`, working tree clean. Pure ASCII
throughout, per the file-edit discipline. The ratios, line counts and route
lists below were measured on disk in the shaping session; every claim that a
capability is absent cites the file and line that proves it.

## 1. Capability map (the honest inventory)

Legend: EXISTS = shipped and reachable; PARTIAL = shipped but capped, wrong
owner, or wrong process; ABSENT = no code path today.

| # | Capability the wizard needs | State | Where (file:line) |
|---|---|---|---|
| 1 | Proposal payload: plan id, target, before/after collateral, saving, risk, consumer TU count, required headers | EXISTS | `crates/ui/src/dto.rs:261-286` (`ExtractionProposalDto`); feed `crates/cli/src/commands/ui.rs:486-523` |
| 2 | Files-to-create rows with per-file symbol manifest | EXISTS, capped | `dto.rs:244-259` (`EXTRACTION_PROPOSAL_MAX_TARGET_FILES = 8`, names capped at `EXTRACTION_PROPOSAL_MAX_NAMES`); feed `crates/cli/src/commands/ui.rs:451-484` (`target_file_groups`) |
| 3 | Full symbol -> destination-file mapping for a plan | PARTIAL | present in the plan model (`crates/cluster/src/plan.rs:35-45`) but never leaves the boot path; the UI boot computes plans and drops them (`crates/cli/src/commands/ui.rs:383-436`) |
| 4 | Per-file required includes/imports of the NEW file | ABSENT at the API boundary, COMPUTABLE read-only | computed at extract time by the isolation engine from the reduced model (`crates/transform/src/engine/mod.rs:95-128` `required_deps`); the CLI preview path passes `required_deps: vec![]` (`crates/cli/src/commands/transform.rs:216-235`); no DTO field exists |
| 5 | Verbatim symbol bodies | ABSENT at the API, computable | `ExtractedSymbol.body` `crates/transform/src/lib.rs:29-38`, resolved through `crates/transform/src/engine/bodies.rs`; system header bodies are outside the model, so bodies are partial by nature |
| 6 | New-file content preview | PARTIAL | `tool transform extract` writes generated units under `artifacts/generated` (`crates/cli/src/commands/transform.rs:31-79`); no read-only preview surface |
| 7 | Consumer include rewrites (old -> new) | EXISTS, CLI only | `crates/cli/src/commands/transform.rs:81-127` (`build_migration_preview`); `ConsumerRewrite` / `MigrationPreview` `crates/transform/src/placement.rs:12-71` |
| 8 | Dry-run diff text | EXISTS, CLI only | `MigrationPreview::format_diff_with` `placement.rs:58`; printed by `transform.rs:99-116`; artifact `artifacts/migration/preview.json` |
| 9 | Transactional apply: write new files, rewrite consumers, snapshot, three tiers, rollback | EXISTS, CLI only | `apply_transform_plan` `crates/cli/src/commands/transform.rs:129-192`; `TransactionApplier::apply_with` `crates/validate/src/lib.rs:150-274`; rollback `:297-321`; tiers `transform.rs:341-397` |
| 10 | End-to-end plan + apply | EXISTS | `crates/cli/src/optimize.rs:365-429` (`apply_safe_plan`) |
| 11 | A UI route that mutates anything | ABSENT | every route in `crates/ui/src/server.rs:48-79` is a GET; the reader trait in C2 keeps it that way |
| 12 | Plan-by-id resolution in the server | ABSENT | plan artifacts exist (`crates/artifact/src/manager/metrics_io.rs:327,370`) but the UI never loads them; boot plans in memory and discards them |
| 13 | Stale-plan detection | PARTIAL | generation swapping exists (`crates/ui/src/state.rs:152-168` `install_analysis`, epoch bump `:160`); artifact freshness uses fingerprints (`crates/artifact/src/manager/graph_io.rs:239` `load_reduced_checked`); nothing compares a PLAN against current repo state |
| 14 | Stepper / wizard interaction | ABSENT | today's surface is a summary card with one disclosure level (`crates/ui/assets/hotspots.js:447-455`, `:540-585`); modal infrastructure exists (`index.html:423-585` `#modalOverlay` + `.modal`) |
| 15 | Auto-extraction executed from the UI | ABSENT | the endgame; follow-on `extraction-auto-apply` |

Consequence: rows 4, 6, 7, 8 and 12 are contract work, not styling work, and
row 11 is the boundary that makes this milestone honest. The shape therefore
promises a stepper over plan data plus a read-only engine-backed manifest and
dry-run, and an exact recipe for execution - never an execution button.

## 2. Invariants (each independently checkable; oracle named)

- **W1 No UI mutation.** Every route this milestone adds is registered with
  `get(...)` only, and the new route module contains no `fs::write`,
  `fs::remove_file`, `fs::create_dir` or `ArtifactManager` call. Oracle:
  `crates/ui/tests/ui_test/extraction_wizard_api.rs` string/import locks over
  `routes/api_extraction.rs` plus the server.rs route table diff in review.
- **W2 Read-only provider.** Running a manifest or a dry-run over a fixture
  leaves the fixture byte-identical (a directory hash before and after).
  Oracle: a cli provider unit test that snapshots the fixture tree.
- **W3 Engine-sourced includes.** Every `required_includes` entry in a
  response comes from `GeneratedUnit.required_deps` or the list is empty with
  an explicit `includes_unavailable` flag; the renderer emits the honest
  fallback line, never a fabricated path. Oracle: a provider test on a fixture
  with known inter-unit dependencies plus a renderer string lock.
- **W4 Copy-vs-move honesty.** The wizard's step 3 and step 5 copy state that
  the C++ backend writes the new file and leaves the source declarations in
  place (`remove_top_level_symbols` returning `None` writes nothing back,
  `crates/validate/src/lib.rs:142-150`, `:273`). Oracle: a renderer lock on
  the exact sentence, and a provider test asserting `move_removals == 0` on a
  C++ fixture with `copy_only == true`.
- **W5 Command honesty.** Only `tool apply <repo-root> --plan <id>`,
  `tool migrate <repo-root> --plan <id> --preview` and
  `tool transform extract <repo-root> --plan <id>` are named, matching the
  verbs in `crates/cli/src/lib.rs` (plan is a positional repo root, `--plan`
  is the flag). Oracle: the amended `test_apply_recipe_is_honest` plus a new
  verb-count lock.
- **W6 Staleness refusal.** A manifest or dry-run whose reader epoch does not
  equal the live `analysis_epoch` is refused with `409` and the documented
  error body; the wizard resets its step state on that response. Oracle:
  `extraction_wizard_api.rs` route tests with a stale reader injected.
- **W7 Lock survival or named amendment.** The 18 card-family locks
  (`extraction_card_story.rs`, `extraction_card_playbook.rs`) survive or are
  amended strictly per the C5 table; zero deletions. Oracle: reviewer per-commit
  diff of both test files against the start rev.
- **W8 Gauge discipline.** Zero added hex (`theme_token_check --check-added`
  silent), `theme_token_baseline.json` diff-empty, `hygiene_check.py` exit 0,
  `encoding_guard` exit 0, all four themes covered by the contrast oracle
  (`extraction_contrast.rs`, which stays UNEDITED).
- **W9 Volume bound.** The boot payload grows by nothing for the wizard; every
  per-file symbol list is fetched per step under a documented cap
  (`EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE`). Oracle: a payload test that
  the serialized `extraction_proposals` bytes are unchanged, plus a manifest
  cap test.
- **W10 Payload order and disclosure defaults.** No sort is introduced; the
  panel cards render in payload order and stay collapsed by default. Oracle:
  `test_order_and_preview_locks` and `test_details_collapsed_by_default` stay
  green.

## 3. Contracts

### C1 Wizard flow: stepper states

Host: a modal (`#modalOverlay` + `.modal` precedent, `index.html:423-585`),
not the 520px panel - the volume the user reported cannot live in the panel.
Entry: an "Open wizard" button on each proposal card (the panel keeps the
collapsed summary-first model `ui.extraction_card_interaction_model`).

| Step | Shows | Defers to | Contract delta |
|---|---|---|---|
| S0 Overview | plan id, target file, collateral before -> after, saving, risk tier, consumer TU count, total symbols | detailed lists | none - payload as-is (row 1) |
| S1 Files to create (PRIMARY) | one row per destination file: real path, symbol count, per-file include count, selectable | per-symbol detail | needs uncapped per-file manifest -> new route |
| S2 Per-file manifest | full symbol list for the selected file plus the includes that file needs | bodies | new route fields `required_includes`, `symbol_count` (row 4 gap closed) |
| S3 Impact and risk | consumer TU count, required headers, engine cycle error, copy-vs-move statement | the diff itself | new route fields `cycle_error`, `move_removals`, `copy_only` |
| S4 Dry run | the new file paths that would be written and every consumer rewrite (old -> new includes, preserved side-effect includes) | execution | new dry-run route (rows 7-8 surfaced) |
| S5 Execute | the exact CLI recipe with the real plan id, and the `--preview` first step | the action itself | copy only; no mutation route in this milestone |
| S6 After apply | what the three tiers validate, what rollback restores, where the artifacts land | UI-side reporting | CLI behavior described; no UI surface |

Stepper chrome: ordered step list with `aria-current="step"`, Back/Next as real
`<button>`s, `Esc` closes via the existing `closeModals()` path, no animation
(keeps `hidden` and string locks honest), ASCII-only labels.

### C2 Read-only provider and routes

New module `crates/ui/src/routes/api_extraction.rs`. It owns the two handlers
and the plain-data reader trait; the CLI owns the implementation. This is what
keeps `ariadne-cluster`, `ariadne-transform` and `ariadne-validate` types out
of the ui crate (the "plain display data only" rule at `dto.rs:182-184` of the
pre-split file, which the split preserves as a module doc).

```rust
/// Read-only extraction computation the wizard routes need. The host (cli)
/// implements it against the isolation engine and the include rewriter; no
/// implementation may write to disk (W1, W2).
pub trait ExtractionManifestReader: Send + Sync + 'static {
    /// The analysis generation this reader was built for.
    fn epoch(&self) -> u64;
    fn manifest(&self, plan_id: &str) -> Option<ExtractionManifestDto>;
    fn dry_run(&self, plan_id: &str) -> Option<ExtractionDryRunDto>;
}
```

State seam (`crates/ui/src/state.rs`): one new field
`extraction_reader: RwLock<Option<Arc<dyn ExtractionManifestReader>>>` with
`install_extraction_reader`, plus `extraction_reader()` accessor. The CLI calls
`install_analysis` then `install_extraction_reader` inside the same
`swap_into_state` sequence (documenting the same bounded window the payload
write already has, `state.rs:12-25`). `UiState::new_loading` and `new_loaded`
keep their signatures, so no existing test call site changes.

Routes (registered in `server.rs`, handlers in the new module):

- `GET /api/extraction/{plan_id}/manifest`
- `GET /api/extraction/{plan_id}/dry-run`

DTOs (additive, in the new `crates/ui/src/dto/extraction.rs` created by W-1;
all `serde`-plain types, `#[serde(default)]` on every field):

```rust
pub const EXTRACTION_MANIFEST_MAX_FILES: usize = 64;
pub const EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE: usize = 200;
pub const EXTRACTION_DRY_RUN_MAX_REWRITES: usize = 200;

pub struct ExtractionUnitManifestDto {
    pub path: String,                  // repo-relative, lossy-rendered
    pub symbol_names: Vec<String>,     // capped, symbol_id order
    pub symbol_count: usize,           // uncapped members
    pub required_includes: Vec<String>,// engine required_deps, lossy-rendered
    pub includes_unavailable: bool,    // true iff the engine could not resolve deps
}

pub struct ExtractionManifestDto {
    pub plan_id: String,
    pub generation_epoch: u64,
    pub files: Vec<ExtractionUnitManifestDto>,
    pub files_truncated: bool,
    pub cycle_error: Option<String>,   // engine unit-graph cycle, when any
}

pub struct ExtractionDryRunDto {
    pub plan_id: String,
    pub generation_epoch: u64,
    pub new_files: Vec<String>,        // paths the apply would write
    pub rewrites: Vec<ExtractionRewriteDto>,
    pub rewrites_truncated: bool,
    pub move_removals: usize,          // symbols the plugin will strip; 0 for C++
    pub copy_only: bool,               // true when no source symbol is removed
    pub includes_unavailable: bool,
}

pub struct ExtractionRewriteDto {
    pub consumer_file: String,
    pub old_includes: Vec<String>,
    pub new_includes: Vec<String>,
    pub side_effect_preservations: Vec<String>,
}
```

Field names mirror `ConsumerRewrite` (`crates/transform/src/placement.rs:14-23`)
so the mapping is mechanical and reviewable.

### C3 Error shapes (new, uniform, ASCII bodies)

| Condition | Status | Body |
|---|---|---|
| No analysis generation yet | `503` | `{"error":"analysis_loading"}` |
| Reader epoch != live epoch (stale plan) | `409` | `{"error":"stale_generation","epoch":<live>}` |
| Unknown plan id | `404` | `{"error":"unknown_plan","plan_id":"<id>"}` |

The `503` while-loading shape follows the existing convention
(`crates/ui/tests/ui_test/graph_api.rs:269-289`).

### C4 Dry-run semantics the UI may claim

The dry-run route must reproduce `build_migration_preview` exactly: the same
`plan_rewrite_targets`, the same consumer discovery, the same
`IncludeRewriter::build_preview`. The route renders directive spelling through
the plan-language rewriter (`transform.rs:110-113`) or returns the neutral
path lists; either is acceptable, but the choice is locked by a test comparing
the route output against `MigrationPreview` on a fixture. The UI states that
the dry-run changes nothing and that `tool apply` performs the same
computation before writing.

### C5 Amendment table for existing card locks (inherited from clarity T-B)

Amended (each with a rationale comment in its diff):

1. `extraction_card_story::test_beats_appear_in_story_order` - beat list
   becomes the wizard-step order; survives as an order lock in the new shape.
2. `extraction_card_story::test_risk_tier_badge_sanitized` - base-rule token
   `var(--text-sub)` -> `var(--text-muted)` where it paints the card surface
   (measured 2.97:1 in dark).
3. `extraction_card_playbook::test_file_plan_rows_markup` - the literal
   `class="extraction-file"` cannot coexist with the wizard's
   `.extraction-file-block`; the selector moves with the block.
4. `extraction_card_playbook::test_playbook_styles_declared_tokens` - selector
   list extended with the wizard rules.
5. `extraction_card_playbook::test_apply_recipe_is_honest` - verb list grows
   by `tool transform extract` only, and the "UI never applies" sentence is
   reworded to name the follow-on boundary.

Survive byte-identical: the remaining 13 of the 18 card-family locks, plus
`proposals_payload.rs`, `static_html.rs`, `static_html_sections.rs`,
`help_center_readability.rs`, `export_order.rs`, `dto_builders.rs`,
`theme_contract.rs`, `theme_dark_grey_palette.rs`, `theme_js_zero_hex.rs`,
`extraction_contrast.rs`, `preview_hypothetical.rs`, `icon_asset.rs`,
`asset_injection_hardening.rs`. `preview.js`, `api.js`, `graph.js` stay
byte-identical.

New locks: `crates/ui/tests/ui_test/extraction_wizard_api.rs` (route contract,
error shapes, stale-generation refusal, no-write lock) and
`crates/ui/tests/ui_test/extraction_wizard_render.rs` (stepper markup, step
order, caps and truncation flags, command honesty, ASCII/escape discipline).

### C6 Volume rules (the 1312-symbol case)

- The boot payload is untouched (W9). Per-file lists are fetched per step
  under `EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE = 200` with `symbol_count`
  carrying the truth; at 1312 symbols over 8 files a single fetch is bounded by
  8 x 200 entries.
- Rewrites are capped at 200 with `rewrites_truncated`; the user's 22-TU case
  is far below it.
- No symbol body is ever returned.
- List rendering uses the existing `+K more` chip idiom
  (`just-more`, `hotspots.js:509`) rather than rendering unbounded rows.

### C7 Wizard state

One module-level wizard state object in `extraction-wizard.js`
(`{planId, step, selectedFile, manifest, dryRun, error}`), reset on: modal
open for a different plan, `409` stale generation, payload swap (the existing
re-render path). No plan state is cached in the DOM; step transitions use
`hidden` on step sections (string-lockable, screen-reader correct). Element
ids are wizard-scoped (`wizardStep1` etc.) because the wizard is a singleton,
unlike the two-surface cards that must avoid ids.

## 4. Auto-extract scope boundary

**In this milestone (the honest v1 of the endgame, minus the mutation):**

- The stepper that makes the plan navigable at TU scale.
- The per-file manifest with symbols AND the includes each new file needs,
  computed read-only by the existing isolation engine.
- The dry-run: exactly which new files would be written and exactly which
  consumer includes would be rewritten, computed by the same code path
  `tool apply` uses.
- The exact recipe for execution, with the validation and rollback guarantees
  the CLI actually provides.

**Deferred to follow-on milestone key `extraction-auto-apply`** (proposed key;
it is backend-first with a UI trigger, so it does not carry the `ui-` prefix):

- `POST /api/extraction/{plan_id}/apply` behind an explicit
  dry-run-then-approve gate (the client must have fetched a dry-run for the
  current generation epoch before the apply is accepted).
- Outcome reporting: the three tier results, the rollback case, the artifact
  paths (`ApplyOutcome::format_output`, `crates/validate/src/lib.rs:48-81`).
- Per-generation apply mutex (one apply at a time), and a post-apply rebuild
  request so the UI re-plans against the mutated tree.
- The help-copy flip from "the UI never applies" to "the UI applies with a
  dry-run first, validation in three tiers and automatic rollback".

**Safety invariants that bound both halves** (stated here so the follow-on
inherits them): never mutate the user's repository without an explicit
dry-run-then-approve gate on the current generation; every mutation goes
through the existing `TransactionApplier` snapshot/rollback path and never a
new write path; every write is pure ASCII through the plugin's extractor (the
plugin owns every generated byte, `crates/validate/src/lib.rs:131-136`);
encoding and budget gates must stay green; the mutation surface stays behind
the localhost bind default (`resolve_bind_addr`, `crates/cli/src/lib.rs`) with
no new `--expose` widening.

## 5. Module map (per hand)

| Hand | Changes | Must not change |
|---|---|---|
| W-1 (crates/ui) | `crates/ui/src/dto.rs` (move out), NEW `crates/ui/src/dto/extraction.rs`, no test change | anything else; behavior byte-identical |
| W-2b (crates/ui) | `crates/ui/src/dto/extraction.rs` (add wizard DTOs), `state.rs` (reader field + install), `routes/api_extraction.rs` (NEW), `server.rs` (+2 routes), `lib.rs` (module visibility only), NEW `crates/ui/tests/ui_test/extraction_wizard_api.rs`, `main.rs` (+1 mod line) | `crates/cli/**`, assets, the card test files |
| W-2a (crates/cli) | `crates/cli/src/commands/ui/extraction_reader.rs` (NEW), `commands/ui.rs` (install call + tests module), `commands/ui.rs` `swap_into_state` | `crates/ui/**` beyond what W-2b already shipped, `crates/transform/**`, `crates/validate/**` |
| W-3a (crates/ui) | NEW `crates/ui/assets/extraction-wizard.js`, `lib.rs` (asset const), `server.rs` (asset route), `routes/api_payloads.rs` (asset handler), `index.html` (script tag + modal shell + card button), `hotspots.js` (entry button + section visibility, bounded), `styles.css` (wizard block, net-capped), `tools/ui_smoke.py` (route rows), NEW `extraction_wizard_render.rs`, `main.rs` (+1 mod) | DTOs, provider, other assets |
| W-3b (crates/ui) | `extraction-wizard.js` (steps 2-4, fetching, stale reset), `index.html` (help copy), `styles.css` (dry-run diff rules), amendments per C5 | everything else |

## 6. Clarity milestone disposition (merge / closeout implications)

- **T-A is already on `main`.** Disk proof: `extraction_contrast.rs` exists,
  `main.rs:10` declares `mod extraction_contrast;`, and the light `--green` is
  `#166534` (`themes.css:188`). Its oracle and its measured floor table are
  inherited as frozen locks for the wizard surfaces (W8).
- **T-B should not be dispatched as written.** Its renderer restructure moves
  the per-file blocks inside the card detail; the wizard moves the same content
  into modal steps. Its C2 content shape (per-file heading, provides line,
  chips, footnote strip, commands last) and its C3 token choices transfer
  verbatim as the wizard's visual language; its placement does not.
- **Its amendment table transfers.** The 4 amendments clarity T-B carried
  become C5 here, re-justified by the wizard restructure.
- **The concurrent hand keeps ownership.** Branch
  `ui-extraction-card-clarity-ta` and the ledger row at
  `tasks/milestone_ledger.md:46` stay with the isolated session. Recommended
  landing path: it lands T-A on main (already the on-disk state), stops before
  T-B, and its closeout amends the row to
  "T-A LANDED `8a254ef` + review rider; T-B SUPERSEDED by `ui-extraction-wizard`
  (amendment table transferred to that milestone's C5)". Nothing in this
  milestone lands on that branch, and nothing in it is blocked by its state.
- **Ledger sync note (not this milestone's duty):** the ledger carries no row
  for the audit-remediation workstream that merged at `59a99c4`. Recorded here
  so the orchestrator can add one; this milestone reconciles its census from
  that merge's recorded anchor instead.

## 7. Handoff sizing and ordering (planner input; no sequencing here)

| Hand | Crate dir | Est. produced lines | Est. gates | Kind |
|---|---|---|---|---|
| W-1 | crates/ui | ~300-450 (a move plus module doc) | ~6 | refactor, hygiene-forced |
| W-2b | crates/ui | ~400-600 | ~9-10 | backend contract |
| W-2a | crates/cli | ~350-500 | ~10 | backend provider |
| W-3a | crates/ui | ~500-700 | ~10-11 | UI-only |
| W-3b | crates/ui | ~400-600 | ~10 | UI-only |

Hard edges: W-1 before W-2b (the DTO additions need the headroom);
W-2b before W-2a (the provider compiles against the trait and DTOs);
W-2a before W-3b (step 2-4 render live provider data); W-1 before W-3a (the
asset registration chain does not need the DTOs, but the shared `dto/extraction.rs`
module and `main.rs` contention make serialization the safe default).
W-3a could theoretically start after W-1 in parallel with W-2b/W-2a, but every
hand adds a `crates/ui/tests/ui_test` module and therefore a `main.rs` line
(standing precedent: the clarity row's "never parallel" rule), so the default
is sequential on main with one scoped landing commit per hand.

If W-2a exceeds its budget, the split line is inside the provider:
`extraction_reader.rs` (engine manifest) first, dry-run second, because the
manifest is the gap the user actually complained about.

## 8. Risks (mitigation plus oracle per risk)

- **R1 Volume at TU scale.** 1312 symbols over 8 files. Mitigation: C6 caps
  and per-step fetching; the boot payload is provably unchanged. Oracle: W9
  payload byte-equality lock plus a manifest cap test.
- **R2 Plan identity and staleness.** A plan id is generation-local; the
  watcher can swap generations mid-wizard. Mitigation: C3 `409` on epoch
  mismatch, wizard reset, reader installed with the generation. Oracle: W6
  route tests with an injected stale reader.
- **R3 Consumer discovery cost.** `find_plan_consumers` runs a build
  discovery and scans TU contents (`crates/cli/src/commands/transform.rs:309-339`);
  at 22 TUs it is cheap, at corpus scale it is not. Mitigation: the provider
  caches the dry-run per (plan id, epoch) behind a mutex. Oracle: a provider
  test that calls the dry-run twice and asserts the discovery ran once.
- **R4 Copy-vs-move overclaim.** The C++ backend does not remove moved
  declarations today. Mitigation: W4, the `copy_only` flag, and the locked
  sentence. Oracle: a fixture test asserting `move_removals == 0` for C++.
- **R5 Wizard state leaks across surfaces.** Mitigation: C7 singleton ids and
  explicit resets. Oracle: a renderer lock on the reset calls plus `ui_smoke`
  cold boot.
- **R6 Amendment creep on the card locks.** Mitigation: C5 is the complete
  list; reviewer diffs both test files per commit. Oracle: W7.
- **R7 Asset registration chain missed.** A new asset needs four sites plus
  the ui_smoke mirror; a missing one fails `ui_smoke` or the export lock.
  Mitigation: W-3a's touch-set lists all five; `ui_smoke: PASS` is the gate.
- **R8 dto.rs ceiling.** Already at exactly 1000 lines; any addition fails
  R2. Mitigation: W-1 is a hard predecessor and W-2b adds only into the new
  submodule. Oracle: `hygiene_check.py` exit 0.
- **R9 Live-milestone interference.** Concurrency at kickoff: the clarity
  isolated hand owns `crates/ui/tests/ui_test/main.rs` and `themes.css`; the
  audit-remediation merge just landed; `branding-assets` leaves an operator
  tail; the bench baseline is stale (PR-2 open). Mitigation: this milestone
  starts from clean `main`, touches no `themes.css` value, and does not rely
  on a bench pass without the documented stance. Oracle: status census at each
  hop.
- **R10 Plugin-isolation guard.** The wizard adds no dependency edges from a
  plugin crate, so `m17_plugin_seam_test`'s transitive cone guard cannot
  regress. Oracle: the guard stays green UNEDITED (it is outside every touch
  set).

## 9. Hygiene notes

- `dto.rs` at exactly 1000 lines: W-1 extracts the extraction-proposal family
  into `dto/extraction.rs`; `dto.rs` keeps the payload root and re-exports via
  `pub use extraction::*;`, so `ariadne_ui::ExtractionProposalDto` and every
  other public path survives unchanged (`lib.rs` is `pub use dto::*;`).
- `cli/src/commands/ui.rs` at 885 lines: the provider lives in a new sibling
  module; `ui.rs` gains call sites only (target <= ~30 lines).
- `hotspots.js` at 680 lines: entry button and section visibility only; the
  wizard lives in the new `extraction-wizard.js`, keeping both files far under
  the ceiling.
- `styles.css` over the ceiling and invisible to R2: wizard styles are one
  contiguous block with a net cap; the split stays with its deferred row.
- New Rust test files stay under 1000 lines each; every helper under 100 lines;
  no source line near 2000 characters; all authored content is pure ASCII
  (use `&rarr;` / `&#8592;` entities where a glyph is wanted, per house idiom).
- New names reveal intent and avoid the R3 banned set: `ExtractionManifestDto`,
  `ExtractionUnitManifestDto`, `ExtractionDryRunDto`, `ExtractionRewriteDto`,
  `ExtractionManifestReader`, `install_extraction_reader`,
  `renderWizardStepFiles`, `renderWizardStepManifest`.
