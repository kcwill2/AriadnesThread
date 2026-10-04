# Shape - Extraction Card Playbook

Companion to `tasks/ui-extraction-card-playbook-kickoff.md`. Shape, not task
list; the planner sizes the three-handoff split at the end. Pure ASCII
throughout, per the file-edit discipline.

## 1. Auto-extract inventory (the honest answer to "did we not implement auto-extract?")

YES - implemented, in the CLI/transform spine (binary `tool`,
`crates/cli/src/bin/tool.rs`). The UI exposes NONE of it today.

Commands (argv uses positional `<repo-root>`, flag `--plan <PLAN-id>`):

| Verb | Implementation | Definition / dispatch |
|---|---|---|
| `tool plan <repo>` (generate + save plan artifacts, print `format_detail`) | `plan_extractions` `crates/cli/src/commands/plan.rs:239` | `Commands::Plan` `crates/cli/src/lib.rs:189-195` |
| `tool plan show <repo> --id PLAN-001` (prints `symbol -> target_header` lines) | `show_optimization_plan` `crates/cli/src/commands/plan.rs:304`; rendering `OptimizationPlan::format_detail` `crates/cluster/src/plan.rs:70-139` | `PlanSubcommands::Show` `crates/cli/src/lib.rs:112-121` |
| `tool transform extract <repo> --plan PLAN-001` (writes generated headers under `generated/`, plugin-routed) | `extract_plan_headers` `crates/cli/src/commands/transform.rs:31` | `TransformSubcommands::Extract` `crates/cli/src/lib.rs:124-133`; dispatch `crates/cli/src/lib.rs:435-436` |
| `tool migrate <repo> --plan PLAN-001 --preview` (consumer include diff, "No files changed", saves `artifacts/migration/preview.json`) | `build_migration_preview` `crates/cli/src/commands/transform.rs:81`; artifact IO `crates/report/src/manager/transform_io.rs:44-76` | `Commands::Migrate` `crates/cli/src/lib.rs:201-211`; dispatch `:301-307` |
| `tool apply <repo> --plan PLAN-001` (transactional: extract + rewrite consumers + 3-tier validate + commit-or-rollback) | `apply_transform_plan` `crates/cli/src/commands/transform.rs:129-192`; `TransactionApplier::apply_with` / `TierValidator` / `ApplyOutcome` / `FileSnapshot` `crates/validate/src/lib.rs:129/336/48/97` | `Commands::Apply` `crates/cli/src/lib.rs:212-219`; dispatch `:308-310` |
| `tool optimize <repo>` (end-to-end) | `crates/cli/src/optimize.rs` (`plan_safe_extractions` :261, extract :348-362, tiers :399-401) | `Commands::Optimize` `crates/cli/src/lib.rs:232-236` |

Supporting spine types: `ExtractionResult { plan_id, headers, errors }`,
`ExtractedSymbol { name, qualified_id, kind, body }`,
`GeneratedUnit { path, symbols, required_deps, primary_source_text }`
`crates/transform/src/lib.rs:27-60`; `ConsumerRewrite { consumer_file,
old_includes, new_includes }` and `MigrationPreview { plan_id, rewrites }`
`crates/transform/src/placement.rs:12-53`; tests
`crates/validate/tests/transaction_apply_test.rs`,
`move_removal_transaction_test.rs`. Intent.md agrees: the flow
Plan -> Preview -> Apply -> Validate (Intent.md ~404-408) and "support both
structural transformation and consumer migration" (~263-266).

What the UI exposes of all this: the hypothetical after-state preview only -
`crates/ui/assets/preview.js` overlays one consumer edge's modelled
after-state from `/api/justification` impact data (`crates/ui/src/server.rs:151`);
the proposed header renders as a *placeholder name* (`preview.js:114,268`),
and the banner says "modelled after-state ... not the current graph"
(`crates/ui/assets/index.html:270`). No target paths, no commands, no
apply/migrate surface. There is no dedicated preview endpoint in `ui.rs`;
`crates/ui/src/server.rs` has ZERO mutation routes.

## 2. Data existence: propose time vs apply time

Propose-time (already in every plan the UI boot path computes):

- `ProposedExtraction { symbol_name, symbol_id, target_header: PathBuf,
  source_file: Option<FileId> }` `crates/cluster/src/plan.rs:35-45` -
  per-symbol destination path IS a plan field. Built at
  `crates/cluster/src/planner.rs:548-566`; the naming template is
  `{sanitized_symbol}_extracted.h` alongside the aggregation file,
  `cpp_target_path_for` `crates/lang-cpp/src/planning.rs:21-27` (injected
  hook; planner stays language-neutral).
- `OptimizationPlan { proposed_extractions, optimization_frontier,
  validation_required_tus, ... }` `crates/cluster/src/plan.rs:47-60`.
- The UI boot path ALREADY runs the same planner
  (`build_extraction_proposals` `crates/cli/src/commands/ui.rs:450-471`,
  DTO mapping `:503-537`) but DISCARDS `target_header`, `source_file`, and
  the TU paths.

Apply/extraction-time only (do NOT fake in UI): verbatim bodies
(`ExtractedSymbol.body`), per-unit `required_deps` (engine + plugin at
extract; the preview-time `plan_rewrite_targets` map is deliberately
conservative, `crates/cli/src/commands/transform.rs:216-235`), consumer
rewrites (`MigrationPreview`, requires TU scan).

Consequence: the recipe the user asks for (which symbols -> which file path,
per file) is ATTACHABLE at propose time with zero new analysis: group
`proposed_extractions` by `target_header`. That is T-B's whole job. The
honest one-line answer for the card/help: "Extraction IS implemented -
`tool apply` does it transactionally with validation and rollback; this
card shows the plan and the command, the UI never applies."

## 3. Boundary decision

Read-only playbook (surface recipe + exact copy-pasteable `tool apply`
invocation), one-click apply EXPLICITLY DEFERRED. Justification: `tool
apply` mutates the user's repository (writes generated units in place or
under `generated/`, rewrites consumer includes, may roll back
automatically - `transform.rs:164-192`, `commit_or_rollback` :341-397).
A UI button needs mutation routes (none exist), a confirmation/preview
step, and outcome reporting - that is its own milestone with its own safety
invariants. "UI-safe and trivial" is FALSE here; the deferral is justified
by the mutation + rollback semantics, not by laziness.

## 4. Invariants (numbered, independently checkable; oracle named)

P1 **Feed truth.** For a constructed `OptimizationPlan`,
`extraction_proposal_dto` emits `target_files` grouped by `target_header`
with symbol lists equal to the plan's mapping (path asc; symbols sorted by
`symbol_id`, mirroring the existing `symbol_names` order at ui.rs:507-514),
capped at 8 files with `target_files_truncated` set iff the group count
exceeds the cap. Oracle: new unit tests in
`crates/cli/src/commands/ui/` (constructed plans, no fixture needed) -
falsifiable, deterministic. No test locks this today.

P2 **Legacy payloads decode.** Payload JSON without `target_files` /
`target_files_truncated` deserializes to `vec![]` / `false`; round-trip of
new payloads byte-stable. Oracle: additions to
`crates/ui/tests/ui_test/proposals_payload.rs` (new tests appended; the one
edited existing test is P3).

P3 **Key-stability amendment.**
`proposals_payload.rs::test_proposal_dto_key_names_are_stable` (line 72) is
updated IN THE T-A COMMIT to old-keys-union `{target_files,
target_files_truncated}` - the amendment IS the de-thaw, recorded here so
review-gate flags nothing. No other pre-existing DTO key changes.

P4 **Collapsed by default.** The builder emits the summary row outside, the
detail inside `<div class="extraction-details" ... hidden>`; with an empty
JS toggle state nothing expands at render time. Oracle: new
`extraction_card_playbook.rs` string locks (`hidden` attribute in the
detail template, `aria-expanded="false"` initial state) + `ui_smoke` cold
boot (markup presence, no crash). No browser-driver oracle exists for
"visually collapsed" - stated plainly; the `hidden` attribute IS the
mechanism, so the string lock is the honest lock.

P5 **Keyboard + pointer semantics.** Toggle is `<button type="button"
class="extraction-disclosure" aria-expanded=... onclick="toggleExtractionCard(event, this)">`;
handler does `event.stopPropagation()` then flips `hidden` +
`aria-expanded` on the `.extraction-details` inside the same
`.extraction-card` (DOM walk, NO element ids - the same card renders in two
surfaces and ids would collide). Enter/Space are native button behavior.
Oracle: string locks (button type, aria-expanded, stopPropagation,
`classList.toggle` or equivalent on `hidden`).

P6 **Nine story tests green UNEDITED.**
`crates/ui/tests/ui_test/extraction_card_story.rs` absent from the T-C
touch set; all 9 pass because every locked substring survives and the
detail keeps beat order `extraction-head` -> `extraction-problem` ->
`extraction-cost` -> `extraction-chips` -> `extraction-tu` (test 8's
strictly-ascending source-index rule). Breakage plan (state before the
fact, since the prior C4 list is NOT binding on this milestone): the only
realistically at-risk lock is story-test 8's beat order; the layout puts
`extraction-head` classes INSIDE the summary button and the other four
beats inside details IN ORDER, so the source template order is unchanged.
If implementation evidence later proves otherwise, amend story-test 8 with
a written justification in the journal + verdict - that is the sanctioned
path, not silent edit.

P7 **Keep-literals survive** (consumers: `static_html.rs::
test_ui_extraction_proposal_cards`, `static_html_sections.rs`,
`dto_builders.rs`, `help_center_readability.rs` - all green UNEDITED):
`function renderExtractionProposalCard(`, `collateral_load_before`,
`collateral_load_after`, `consumer_tu_count`, `extraction-row`, `extracted
symbols need`, `just-more`, `selectHotspotRow('`,
`graphData.extraction_proposals || []`, `proposals.length === 0`,
`Preview drop from selected file`, api.js/index.html/graph.js literals per
the prior C4 list (api.js untouched anyway; graph.js untouched).

P8 **No new analysis in the UI layer.** `hotspots.js` gains no `fetch`/
`XMLHttpRequest`/`EventSource`; renderer reads only `p.*` payload fields.
`api.js` and `preview.js` byte-identical (touch-set + `preview_hypothetical.rs`,
`static_html_sections.rs::test_extraction_impact_preview_overlay` unedited).

P9 **Command honesty.** Card renders exactly
`tool apply <repo-root> --plan <plan_id>` (plan_id interpolated, escaped)
and mentions `tool migrate <repo-root> --plan <plan_id> --preview`;
`<repo-root>` stays a literal placeholder token (live mode could pull
`/api/repo-info` - `server.rs:320-350` - but that is a NEW fetch per card
surface and unavailable in standalone exports: REJECTED as overreach,
recorded). Help copy states the UI never mutates the repo and that `tool
apply` validates and rolls back. Oracle: string locks on the command
literals + reviewer spot-check against `crates/cli/src/lib.rs` command
definitions (verb names, flag names - `--plan` is the flag, repo is
positional).

P10 **Escape discipline + zero hex.** `p.target_files[].path` and every
symbol name pass `escapeHtml`; styling adds only `var(--token)` from the
existing three-theme set. Oracle: playbook test asserts `escapeHtml(t.path`
and `escapeHtml(...symbol` patterns in the new helpers;
`theme_token_check.py --check-added --base <rev>` silent.

P11 **Order + hide behavior unchanged.** No `.sort(` over proposals;
payload order rendered; `renderProposalsSection` cache logic and
`syncExtractionProposalSurfaces` byte-identical (they call the same builder
name; disclosure state is per-DOM, not cached - re-render resets to
collapsed, which is the documented behavior, see C6).

## 5. Contracts

C1 **DTO extension** (`crates/ui/src/dto.rs`, additive, T-A):

```rust
/// Cap for destination-file rows per proposal (mirrors the header cap).
pub const EXTRACTION_PROPOSAL_MAX_TARGET_FILES: usize = 8;

/// One suggested destination file and the symbols it should contain.
/// `path` is repo-relative, lossy-rendered like the plan's PathBuf.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposedTargetFileDto {
    pub path: String,
    pub symbol_names: Vec<String>,   // sorted by symbol_id, cap 12 (reuse EXTRACTION_PROPOSAL_MAX_NAMES)
    pub symbol_count: usize,         // uncapped members in this file
}

// on ExtractionProposalDto (both NEW fields serde-defaulted):
    #[serde(default)]
    pub target_files: Vec<ProposedTargetFileDto>,
    #[serde(default)]
    pub target_files_truncated: bool,
```

Re-export from `crates/ui/src/lib.rs` (one line, `ariadne_ui::
ProposedTargetFileDto` - the cli already consumes `ariadne_ui::
ExtractionProposalDto`). No cluster types cross into ui (the pre-existing
"plain display data only" rule at dto.rs:182-184 is respected: new fields
are plain Strings/usize). Struct-literal constructors of the DTO in tests
gain the two fields: `export_order.rs:163` neighborhood and
`dto_builders.rs`/`proposals_payload.rs` builders are in the T-A touch set
(compelled mechanical fill-ins, `target_files: vec![]` / `false`).

C2 **Feed** (`crates/cli/src/commands/ui.rs`, T-B): inside
`extraction_proposal_dto` (:503), group `plan.proposed_extractions` by
`target_header` into `BTreeMap<PathBuf, Vec<&ProposedExtraction>>` (BTreeMap
gives path-asc determinism), sort each group's members by `symbol_id`
(matches existing :509 sort), cap names per file at
`EXTRACTION_PROPOSAL_MAX_NAMES`, cap file rows at
`EXTRACTION_PROPOSAL_MAX_TARGET_FILES` with the truncated flag. A pure
mapping - no planner, no I/O, no new planner hook.

C3 **Markup skeleton** (T-C; summary always visible, details collapsed):

```
<div class="extraction-card" onclick="selectHotspotRow('<id>')" title="<path>">
  <button type="button" class="extraction-disclosure" aria-expanded="false"
          onclick="toggleExtractionCard(event, this)">
    <span class="extraction-head">
      <span class="extraction-plan-id">PLAN-007</span>
      <span class="extraction-title">graph.h</span>
    </span>
    <span class="extraction-summary-meta">8 symbols</span>
    <span class="extraction-summary-saving">-260 (-87%)</span>   <!-- guarded as today -->
    [renderExtractionRiskTag output]
  </button>
  <div class="extraction-details" hidden>
    [extraction-problem row]      <!-- unchanged markup, moved inside -->
    [extraction-cost row]         <!-- unchanged -->
    [Extract chips row]           <!-- unchanged -->
    [needs row]                   <!-- unchanged, keeps 'extracted symbols need' -->
    [Impact row]                  <!-- unchanged -->
    <div class="extraction-row extraction-files">
      <span class="extraction-label">Files to create (N)</span>
      <div class="extraction-file">
        <code class="extraction-file-path">include/thing_extracted.h</code>
        <span class="extraction-chips">[chips + '+K more' via just-more]</span>
      </div>...  <!-- or <em>no file plan in payload</em> for legacy payloads -->
      <!-- + just-more style '+N more files' when target_files_truncated -->
    </div>
    <div class="extraction-row extraction-apply">
      <span class="extraction-label">How to apply</span>
      <code class="extraction-command">tool apply &lt;repo-root&gt; --plan PLAN-007</code>
      <div class="extraction-apply-note">Validates and rolls back on failure; preview first:
        tool migrate &lt;repo-root&gt; --plan PLAN-007 --preview</div>
    </div>
  </div>
  [preview button row, UNCHANGED markup, OUTSIDE details so it stays discoverable]
</div>
```

Helpers (each < 30 lines, all under 100): `extractionSummaryMeta(p)`,
`renderExtractionFilePlan(targetFiles, truncated)`, `renderExtractionApplyHint(planId)`,
`toggleExtractionCard(event, buttonEl)` - global like its siblings; names
reveal intent; the builder stays one composition function (~70-80 lines).
Legacy-payload fallback (`target_files` empty) renders the honest `<em>`
line, never a fabricated path.

C4 **Serde/versioning policy.** New fields are `#[serde(default)]`; the
`extraction_proposals` outer key keeps `skip_serializing_if` at the payload
level (dto.rs:225-226). Nested rows are always emitted when present.
Standalone exports written by older builds keep loading (P2 test).

C5 **Disclosure mechanics.** No ids, no CSS-animated height (keep `hidden`
- screen-reader correct, testable). Re-render (selection change / payload
swap via `renderProposalsSection` cache) rebuilds innerHTML, so cards reset
to collapsed - documented, acceptable (details are two clicks deep, state
is not precious). `toggleExtractionCard` lives in hotspots.js next to
`selectHotspotRow`; the inspector surface reuses the identical markup
(api.js untouched - it injects the same builder output).

C6 **Token map for new styling** (all tokens exist in all themes; zero
additions): `.extraction-disclosure` transparent button on `--tile-bg`,
hover `--border-light`; `.extraction-summary-meta` `--text-muted`;
`.extraction-summary-saving` `--green` (same guard as today's saving);
`.extraction-details` `--border-light` top separator, `--text-sub`;
`.extraction-file-path` / `.extraction-command` monospace on `--tile-bg` +
`--border-light`; `.extraction-apply-note` `--text-sub`; risk badge classes
unchanged. styles.css growth <= ~70 lines, contiguous inside the existing
`.extraction-*` block.

C7 **Help copy** (`index.html`, Hotspots list; each new `<li>` < 400
stripped chars, wall-of-prose + copy locks stay): (a) cards are collapsed
summaries - click to open the playbook; (b) the playbook lists the
suggested destination files and what goes in each, plus the `tool apply`
command - and the UI NEVER changes files; (c) apply is validated in three
tiers with automatic rollback (matches `commit_or_rollback` truth), run
`tool migrate ... --preview` first. Keep locked phrases `Proposed
extractions section`, `Preview after extraction`.

## 6. Module map (per handoff)

| Handoff | Files | Notes |
|---|---|---|
| T-A (crates/ui, DTO) | `crates/ui/src/dto.rs`, `crates/ui/src/lib.rs` (+1 re-export line), `crates/ui/tests/ui_test/proposals_payload.rs` (P3 amendment + P2 new tests), `crates/ui/tests/ui_test/export_order.rs` + `dto_builders.rs` (mechanical struct-literal fill-ins only) | compile + serde oracles only; no asset edits |
| T-B (crates/cli, feed) | `crates/cli/src/commands/ui.rs`, its `crates/cli/src/commands/ui/` test modules (+1 mod line) | P1 unit tests on constructed plans; `m00_facts_path_parity` unedited |
| T-C (crates/ui, assets+tests) | `crates/ui/assets/hotspots.js`, `crates/ui/assets/styles.css`, `crates/ui/assets/index.html`, NEW `crates/ui/tests/ui_test/extraction_card_playbook.rs`, `crates/ui/tests/ui_test/main.rs` (+1 mod line) | renderer per C3, styles per C6, help per C7 |
| MUST NOT change | `api.js`, `preview.js`, `graph.js`, `main.js`, `themes.css`, `theme.js`, `crates/cluster/**`, `crates/lang-cpp/**`, `crates/transform/**`, `crates/validate/**`, `crates/report/**`, `crates/cli/src/lib.rs`, `crates/cli/src/commands/{transform,plan}.rs`, `optimize.rs`, `tools/**`, `tests/**` (except the three named ui_test files), `crates/ui/src/server.rs` | the apply spine is referenced, never modified |

## 7. Handoff sizing + ordering (planner input; I do not sequence)

Three sequential hands, each <= 1 crate / <= 10 gates / well under 1200
produced lines: T-A (~150-250), T-B (~150-300), T-C (~500-700). Hard
dependencies: T-B needs T-A's struct; T-C's live `ui_smoke` playbook render
needs T-B's feed (string tests do not, but do not split a handoff's
green-gate across the seam). If the planner seeks two hands instead, merging
T-A+T-B crosses exactly two crates by ~60 lines total - permissible ONLY as
a recorded deviation from the crate-count rule (the sizing AXIS - gate
count x output volume - comfortably passes); default recommendation stays
three rule-clean hands.

## 8. Risks (mitigation + oracle per risk)

- **Key-stability test trips (frozen-keys guard).** Expected; T-A amends it
  deliberately (P3). Oracle: the amended test IS the lock; reviewer checks
  the union set equals old + exactly two names.
- **Struct-literal fallout** (`export_order.rs`, `dto_builders.rs`,
  `proposals_payload.rs` builders fail to compile without the fields).
  Mechanical `vec![]`/`false` fill-ins inside T-A; no assertion edited
  beyond P3. Oracle: scoped ui suite green.
- **Story-test order lock (test 8) under the new layout.** Design keeps the
  four detail beats in template order; breakage plan in P6 (amend with
  journal+verdict justification, never silent).
- **Two-surface DOM (hotspot list + inspector) with duplicate disclosure
  state.** No ids, DOM-walk scoping (C5); both surfaces re-inject via the
  one builder (api.js untouched). Oracle: `dto_builders.rs` blob tests +
  reviewer interaction check; no automated DOM-duplication oracle exists -
  stated plainly.
- **Copy overclaiming.** Card copy derives from plan data + real CLI verbs
  only; no body/dep claims; help mirrors `commit_or_rollback` behavior.
  Reviewer spot-checks against `lib.rs` command defs and
  `lang-cpp/src/planning.rs:21-27` naming template ("suggested" wording is
  honest: paths are plan-suggested until `tool apply` writes them).
- **Bench gate for T-B (non-assets Rust).** The feed adds O(symbols)
  grouping to a boot path that already runs the full planner; measured
  phases (planner/justification) are untouched. Reviewer stance: scoped
  `bench_compare` PASS on a quiet machine, or the documented exemption with
  the empty `crates/pipeline/**` diff as proof. Deferred stale-`tool.exe`
  lock row applies to any normal-target run.
- **Concurrency with `cpp-brace-depth-underflow`.** File-disjoint; if it
  lands mid-milestone, payload contents shift (bug fix) - census reconcile
  only (kickoff section). No shared file.
- **styles.css ungated growth** (1731 lines; R2 skips `.css`, deferred
  ledger row owns the split). Mitigation: <= ~70-line contiguous block
  (C6); the extraction block was already consolidated at `ef9d170`.

## 9. Hygiene notes

- `dto.rs` 880 -> ~940 lines: UNDER the 1000 ceiling but the closest file on
  this seam; if any later milestone needs dto.rs again, plan the
  proposal-DTO module extraction (`dto/extraction_proposals.rs`) THEN -
  recorded now so it is not discovered as a blocking R2 failure later.
- `hotspots.js` 395 -> ~520-570: fine; builder ~70-80 lines + four helpers
  under 30 each (100-line ceiling).
- `cli/src/commands/ui.rs` 790 -> ~840: fine; `extraction_proposal_dto`
  stays < 100 with the pure grouping helper extracted.
- styles.css growth bounded (C6); no authored line anywhere near 2000
  chars; help `<li>` blocks under 400 stripped chars each (C7); help text
  uses `&rarr;`-style entities, pure ASCII.
- New names: no `tmp`/`res`/`data` locals; `toggleExtractionCard`,
  `renderExtractionFilePlan`, `renderExtractionApplyHint`,
  `ProposedTargetFileDto` follow the house reveal-intent style.
