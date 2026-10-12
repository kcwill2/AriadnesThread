# Milestone Kickoff - ui-extraction-wizard

Milestone key (verbatim for the planner/ledger): `ui-extraction-wizard`.
Follows the post-M28 track pattern used by `ui-hotspot-canvas-direction`,
`ui-traversal-back-button`, `ui-degree-opacity-ramp` and `branding-assets`:
no M-number is guessed, the `milestone-lifecycle` kickoff assigns one if the
ledger numbering demands it, and a live ledger row is registered until
closeout. Authored by the architect at HEAD `97dd836` on `main` (working tree
clean, read-only survey). Prior closed artifacts (`ui-extraction-card-story`,
`ui-extraction-card-playbook`) and the in-flight `ui-extraction-card-clarity`
row are background, not binding - see the clarity disposition below.

## 1. Intent

The user's verdict on the current "Proposed Extractions" view, verbatim:

- "This view is unusable. It's too small. The useful information is the
  'files-to-create', the symbols that should go in them and the imports that
  should be used in the new file. None of that information is available."
- "It is also a lot of information. This feature requires a wizard or
  something. Ideally with an automatic extractor that creates the new files
  and moves the symbols."

The observed case that produced the verdict: PLAN-002, target `pcre_internal.h`,
1312 symbols, 8 files to create, 22 consumer TUs.

**Behavior change.**

1. Files-to-create becomes the PRIMARY surface of the extraction feature.
   Today it is one collapsed detail row inside a summary card
   (`crates/ui/assets/hotspots.js:505-517` `renderExtractionFilePlan`, markup
   at `:571-582`), reachable only after a disclosure click, inside a 520px
   side panel.
2. The per-file manifest must carry the symbols destined for each new file AND
   the includes/imports each new file needs. Ground truth: the symbol half
   exists behind a display cap (`EXTRACTION_PROPOSAL_MAX_NAMES` = 12 names,
   `EXTRACTION_PROPOSAL_MAX_TARGET_FILES` = 8, `crates/ui/src/dto.rs:244-259`).
   The includes half does NOT exist at any API boundary - it is computed only
   by the isolation engine at extraction time
   (`GeneratedUnit.required_deps`, built in
   `crates/transform/src/engine/mod.rs:95-128`), and the CLI's preview path
   deliberately passes empty deps (`crates/cli/src/commands/transform.rs:216-235`
   `plan_rewrite_targets`). That is a backend contract gap, not a CSS problem.
3. The interaction becomes a stepped wizard (stepper). At 1312 symbols / 8
   files / 22 consumer TUs, a stacked card cannot carry the volume; the
   stepper is what makes the volume navigable.
4. The endgame is automated extraction driven from the UI. This milestone
   builds the wizard and the read-only contract that makes every step honest;
   UI-triggered mutation is a separate follow-on milestone (boundary line in
   shape section 4).

**Underlying why.** The extraction planner is the only actionable output the
UI has, and the analyze -> apply spine documented in `Intent.md` (~392-413,
Plan -> Preview -> Apply -> Validate -> Measure -> Commit or Roll Back) is real
but only reachable through the CLI. The user is asking for the spine to be
drivable from the surface where the decision is made.

**Blast radius.** `crates/ui/src/{dto.rs, state.rs, server.rs, lib.rs,
routes/}`, `crates/ui/assets/{index.html, hotspots.js, styles.css}` plus a NEW
asset `crates/ui/assets/extraction-wizard.js`, `crates/cli/src/commands/ui.rs`
plus a new sibling module, `crates/ui/tests/ui_test/**` (new locks plus a
bounded amendment set), and `tools/ui_smoke.py` (its route table is a hand
mirror of `build_router`, `tools/ui_smoke.py:58-60`). The mutation spine
(`crates/transform/**`, `crates/validate/**`, `crates/cluster/**`,
`crates/lang*/**`, `crates/artifact/**`) is READ by the new provider and never
modified.

## 2. Capability map (summary)

Full table with file:line citations is shape section 1. Short form:

| Capability | State today |
|---|---|
| Proposal payload (plan id, target, before/after, saving, risk, consumer TU count, required headers) | EXISTS |
| Files-to-create rows (path + per-file symbol names) | EXISTS, capped at 8 files / 12 names |
| Full per-file symbol manifest (uncapped) | EXISTS in the plan model, NOT in the API |
| Per-file required includes/imports for the new file | ABSENT at the API boundary; computable read-only by the isolation engine |
| Verbatim symbol bodies / generated-file text | ABSENT at the API boundary; computable at extract time |
| Consumer include rewrites and dry-run diff | EXISTS in the CLI only (`tool migrate --preview`, `tool apply`) |
| Transactional apply (write files, rewrite consumers, snapshot, 3 tiers, rollback) | EXISTS in the CLI only |
| UI route that mutates anything | ABSENT - every route in `crates/ui/src/server.rs:48-79` is a GET |
| Plan-by-id resolution in the server | ABSENT - the UI boot path plans in memory and drops the plans |
| Wizard / stepper interaction | ABSENT |
| Auto-extraction executed from the UI | ABSENT (follow-on milestone) |

The shape must not promise an auto-extractor that is not there. What IS there
is a transactional CLI apply; what this milestone adds is a stepper plus the
read-only contract that lets the stepper tell the truth.

## 3. Scope in (each with its reason)

1. Read-only wizard backend contract: `GET /api/extraction/{plan_id}/manifest`
   and `GET /api/extraction/{plan_id}/dry-run`, served from
   `crates/ui/src/routes/api_extraction.rs`, computed by a plain-data reader
   the CLI injects into `UiState` (shape contract C2). This is the gap closure
   for per-file required includes.
2. Wizard UI as a modal stepper in a new asset `extraction-wizard.js`
   (precedent: `preview.js` was added as a new asset rather than growing
   `graph.js` past its budget), entered from the existing proposal cards in
   the Hotspots panel.
3. Additive wizard DTOs in `crates/ui/src/dto.rs` after a hygienic module
   split (see section 6).
4. The bounded amendment set for the existing extraction-card string locks
   (shape section 5) - inherits clarity T-B's amendment table because
   clarity T-B is superseded.
5. Help copy for the wizard in `crates/ui/assets/index.html` (Hotspots list),
   including the honest statement that the UI never mutates the repository in
   this version.
6. `tools/ui_smoke.py` route-table mirror rows for the two new routes.

## 4. Scope out (each with its reason)

- **UI-triggered execution of the apply transaction.** Zero mutation routes
  exist today and `tool apply` performs file writes, consumer rewrites, a
  three-tier validation and an automatic rollback
  (`crates/validate/src/lib.rs:150-274`, `:297-321`). A UI execution path needs
  POST routes, a dry-run-then-approve gate, outcome reporting and its own
  safety invariants. Deferred to follow-on key `extraction-auto-apply`, boundary
  line in shape section 4.
- **Editing or re-planning.** The wizard never edits the plan: no symbol
  re-assignment between destination files, no path renaming, no planner
  reconfiguration. v1 selects whole plans and shows whole files.
- **Generated-file bodies in the UI.** Bodies can be large and are the
  apply-time artifact; the wizard reports body presence and counts, not text.
- **`crates/cluster/**` planner or naming changes.** Suggested paths already
  exist (`ProposedExtraction.target_header`, `crates/cluster/src/plan.rs:38`);
  the `{sanitized_symbol}_extracted.h` template is
  `crates/lang-cpp/src/planning.rs:21-27` and changing it re-gates goldens and
  the characterization suites.
- **`styles.css` split / `dto.rs` split beyond what this milestone needs.**
  Both files sit at or over the 1000-line ceiling; the split is a hygiene
  necessity for this milestone's own additions (section 6), the general
  styles.css split decision stays with its dated deferred ledger row.
- **`preview.js`, `api.js`, `graph.js`, `traversal.js`, `main.js`,
  `themes.css`, `theme.js`, `tools/theme_token_baseline.json`,
  `tests/fixtures/**`, `tests/golden/**`.** Standard frozen sets.

## 5. Invariants (checkable forms in the shape)

1. No mutation from the UI in this milestone: every new route is a GET, and a
   lock asserts no non-GET handler and no filesystem write in the new route
   module.
2. The wizard's read-only provider writes nothing: running a manifest or a
   dry-run against a fixture leaves the tree byte-identical.
3. Every per-file includes list is engine-sourced or honestly absent; the
   renderer never fabricates an include.
4. Copy-vs-move honesty: the wizard states the current backend semantics (the
   C++ backend copies declarations into the new file and leaves the source
   byte-identical unless the plugin strips).
5. Command honesty: only real CLI verbs are named, with the real flag surface.
6. Staleness: a manifest or dry-run computed against a superseded analysis
   generation is refused, and the wizard resets.
7. Existing extraction-card locks survive or are amended per the named table
   only; no lock is deleted.
8. Zero added hex; hygiene R1/R2 exit 0; encoding guard exit 0 at every hop;
   authored content is pure ASCII.

## 6. Hygiene facts that force ordering (measured at HEAD)

- `crates/ui/src/dto.rs` is exactly 1000 lines (read to EOF at HEAD).
  `tools/hygiene_check.py:606` fails any file over 1000, so ANY DTO addition
  goes red. W-1 (a module split of the extraction-proposal family into
  `dto/extraction.rs`) is therefore a hard predecessor of the DTO work.
- `crates/cli/src/commands/ui.rs` is 885 lines. The provider must land in a
  new sibling module (`commands/ui/extraction_reader.rs`); the fee for
  `ui.rs` is call sites only.
- `crates/ui/assets/hotspots.js` is 680 lines. The wizard does NOT grow it;
  a new asset keeps both files under the ceiling.
- `crates/ui/assets/index.html` is 603 lines (one script tag plus the wizard
  modal shell).
- `crates/ui/assets/styles.css` is over the 1000-line ceiling and invisible to
  R2 (`R2_SCANNED_SUFFIXES` excludes `.css`, `hygiene_check.py:19`); the
  wizard carries a net-capped contiguous block and leaves the split decision
  to its dated deferred ledger row.
- Asset registration chain for a new asset, all four sites required:
  `pub const ... = include_str!` (`crates/ui/src/lib.rs:12-24`), a route plus
  handler (`server.rs` + `routes/api_payloads.rs`), the `index.html` script
  tag, the standalone-export inline lock (`crates/ui/tests/ui_test/
  static_html.rs`) and the `tools/ui_smoke.py` probe list.

## 7. Clarity milestone disposition (detail in shape section 6)

- **KEEP:** clarity T-A landed on `main` (verified on disk:
  `crates/ui/tests/ui_test/extraction_contrast.rs` exists, `main.rs:10` has
  `mod extraction_contrast;`, and the light `--green` value is `#166534` at
  `themes.css:188`). Its four-theme contrast oracle and the value fix are the
  legibility foundation the wizard inherits.
- **ABSORB:** clarity T-B's per-file block content shape (its C2/C3) becomes
  wizard step content. Its card-detail placement is superseded.
- **SUPERSEDED AS WRITTEN:** clarity T-B's contract (renderer restructure
  inside the card, 4 test amendments in the card family) should NOT be
  dispatched on main; its amendment table transfers to this milestone's
  renderer hands (shape section 5).
- **NOT MINE TO CLOSE:** branch `ui-extraction-card-clarity-ta` and the
  clarity ledger row belong to the concurrent isolated hand. Recommendation:
  it lands T-A on main (already the on-disk state), stops before T-B, and the
  ledger row is amended to record "T-A LANDED, T-B SUPERSEDED by
  ui-extraction-wizard" rather than dispatched.

## 8. Handoff split summary (sizing detail in shape section 7)

Five sequential hands, each one crate/module dir, each well inside the gate
and line budgets:

| Hand | Crate | Kind | Depends on |
|---|---|---|---|
| W-1 | crates/ui | refactor (dto split) | - |
| W-2b | crates/ui | backend contract (reader trait, DTOs, routes, 503/404/409 errors) | W-1 |
| W-2a | crates/cli | backend provider (engine + migration preview, read-only, cached) | W-2b |
| W-3a | crates/ui | UI-only (wizard asset registration + stepper shell + step 0/1) | W-1 |
| W-3b | crates/ui | UI-only (steps 2-4 against the live provider, help copy) | W-3a, W-2a |

Every hand that adds a `crates/ui/tests/ui_test` module touches `main.rs`, so
no two hands are parallel (standing precedent in the clarity ledger row).

## 9. Frozen locks inherited from the ledger (with proving command)

| Lock | Proof |
|---|---|
| `tests/golden/**` byte-locked, `UPDATE_GOLDENS=1` FORBIDDEN | touch-set + reviewer hop; this milestone changes no producer serialization |
| `tests/fixtures/**` byte-frozen | touch-set check + status census at review |
| `m00_facts_path_parity` (payload planner == CLI plan output) | the wizard adds no planner run on the boot path; the provider is plan-by-id and read-only |
| Themes are the single color-definition site; zero added hex | `python tools/theme_token_check.py --check-added --base <start-rev>` silent (combined form; the bare form is a usage error per the ledger) |
| `tools/theme_token_baseline.json` byte-locked counts | expected to appear in no diff of this milestone |
| The UI never mutates the repository (help copy + `test_help_playbook_copy`) | amended deliberately in W-3b with the follow-on boundary named |
| Hygiene R1/R2 blocking | `python tools/hygiene_check.py` exit 0 |
| Mojibake/UTF-8 gate | `python tools/encoding_guard.py --staged` pre-commit, `--all` at hop close |
| UI runtime smoke | `python tools/ui_smoke.py` PASS (distill route if the session allowlist refuses the direct run) |
| Census anchor rule | reconcile from the newest anchor (the audit-remediation merge `59a99c4` records `1447 passed / 0 failed / 13 ignored / 112 suites`); hands report deltas, never hardcoded totals |
| `--baseline` FILE form errata | workspace census via the `--distill` substitute (8th encounter) |
| Bench gate | W-2a/W-2b touch `crates/*/src`, so the assets-only exemption does not apply; scoped `bench_compare` or the documented operator-tail stance |

## 10. Acceptance gate (milestone as a whole)

1. W-1: `crates/ui/src/dto.rs` under 1000 lines with `dto/extraction.rs`
   holding the proposal family; scoped ui suite green UNEDITED; hygiene exit 0.
2. W-2b: the two routes exist, return the contract shapes, and carry the
   documented error shapes (`503` while loading, `404` unknown plan, `409`
   stale generation); scoped ui suite green with the new locks; no non-GET
   handler; no write call in the new route module.
3. W-2a: the provider answers for a real fixture plan; a byte-identity proof
   shows zero filesystem mutation; the boot path and the watcher swap install
   the reader with the generation epoch; scoped cli suite green; scoped ui
   still green.
4. W-3a: the wizard modal opens from a proposal card, renders steps 0 and 1
   from the existing payload, and the card-family locks survive or are amended
   per the shape table.
5. W-3b: steps 2-4 render live data from the provider, the dry-run diff view
   matches `tool migrate --preview` semantics, the apply recipe is exact, help
   copy states the execution boundary; `ui_smoke: PASS`; zero added hex;
   hygiene and encoding exit 0.
6. Census at closeout reconciles from the newest anchor with every delta
   attributed; `tools/ui_smoke_tests.py` discover unchanged unless route rows
   were added.
7. Review-gate commit-vs-touch-set enforcement per hand; touch sets equal the
   scope lists exactly.
