# Kickoff - Extraction Card Playbook (actionable story + progressive disclosure)

Milestone key: `ui-extraction-card-playbook` (planner: use verbatim; the
ledger row's M-number and dates are assigned by the `milestone-lifecycle`
kickoff - do not guess one). Artifacts: this kickoff and
`tasks/ui-extraction-card-playbook-shape.md` (the shape the planner sizes
from). Successor to the CLOSED `ui-extraction-card-story` milestone (cards
landed at `ef9d170`, ledger row CLOSED 2026-10-04).

## Intent

**Behavior change.** The "Proposed Extractions" card must tell the extraction
story as *actionable instructions*, not just narrative:

1. WHAT to extract - which symbols leave which source file.
2. SUGGESTED new file(s) WITH SUGGESTED LOCATIONS - real paths, not the
   preview overlay's placeholder header node.
3. WHAT SHOULD GO IN EACH FILE - per-file content manifest when a proposal
   implies multiple destination files.
4. Progressive disclosure - a collapsed SUMMARY card per proposal (target
   file, N symbols, saving, risk); CLICK to reveal the detail. Today's card
   renders all five story beats expanded.
5. Answer "did we not implement auto-extract?" honestly: it IS implemented -
   in the CLI/transform spine (`tool plan` / `tool transform extract` /
   `tool migrate` / `tool apply` / `tool optimize`; full file:line inventory
   in the shape, section 1). The card surfaces that spine's recipe and an
   exact `tool apply ...` invocation; the UI itself stays read-only.

**Underlying why.** The user wants to act on a proposal without leaving the
UI cold: the card should read like a playbook (do this, into these files,
then run this command), while the tool's plan->preview->apply->validate
pipeline (Intent.md's documented flow, lines ~404-408) keeps mutation in the
CLI where the tier validators and snapshot/rollback live.

**Blast radius.** Three hand-sized seams: `crates/ui/src/dto.rs` (additive
DTO fields), `crates/cli/src/commands/ui.rs` (proposal->DTO feed),
`crates/ui/assets/{hotspots.js,styles.css,index.html}` +
`crates/ui/tests/ui_test/` (renderer, styling, help, locks). Producers of
analysis (`crates/cluster/**`, `crates/lang-cpp/**`, `crates/transform/**`,
`crates/validate/**`, `crates/report/**`) and the CLI command surface are
NOT touched - the playbook reuses data that already exists at propose time.

## Scope

In:

- `crates/ui/src/dto.rs`: additive `ExtractionProposalDto` fields carrying
  the per-symbol destination plan (grouped target file + symbol manifest +
  truncation flag), serde-defaulted so old payloads still decode.
- `crates/cli/src/commands/ui.rs`: fill those fields from
  `OptimizationPlan::proposed_extractions` (already loaded at boot; no new
  planner run, no new analysis).
- `crates/ui/assets/hotspots.js`: summary/detail split (button +
  `aria-expanded` + `hidden` detail), new PLAYBOOK beats (per-file manifest,
  apply invocation), all data from the payload only.
- `crates/ui/assets/styles.css`: `.extraction-*` summary/detail styles,
  token-only, contiguous growth budget <= ~90 added lines.
- `crates/ui/assets/index.html`: Hotspots help `<li>` additions (playbook
  semantics; "the UI never applies - apply is the CLI's transactional
  command").
- Tests: amended `proposals_payload.rs` key-stability lock (T-A), new cli
  feed unit tests (T-B), new `crates/ui/tests/ui_test/
  extraction_card_playbook.rs` (T-C).

Out (with reasons):

- "One-click apply from the UI". No UI route mutates the repo today; apply
  performs file writes, consumer-include rewrites, 3-tier validation and
  rollback (`crates/cli/src/commands/transform.rs:129-192`). Exposing that
  needs an auth-adjacent mutation endpoint, confirmation UI, and outcome
  reporting - a separate milestone. Deferred explicitly, not implicitly.
- Per-file REQUIRED-DEP lists and verbatim bodies (`GeneratedUnit.
  required_deps`, `ExtractedSymbol.body`): computed at extraction time by
  the engine + language plugin, not present in the plan; inventing them in
  the UI would be analysis fabrication. The card says what the plan knows
  (which symbols move into which path) and points at the apply command for
  the real artifact.
- Consumer TU paths / migration rewrite diffs (`validation_required_tus`
  paths, `MigrationPreview`): consumer discovery is expensive and only runs
  in `tool migrate`/`tool apply`; the card keeps the TU *count* it already
  has. (Optional capped TU-path sample: not requested; out.)
- `crates/cluster/**`, `crates/lang-cpp/**` planner/naming behavior: the
  suggested paths already exist (`ProposedExtraction.target_header`);
  changing naming would re-gate goldens and characterization suites.
- `preview.js`, `api.js`, `graph.js`, `preview` overlay semantics:
  byte-identical; the hypothetical-overlay lock stays. The playbook *adds*
  the honest path next to the placeholder-name overlay, correcting the
  story without touching the overlay.
- `themes.css`, `theme.js`, `tools/theme_token_baseline.json`,
  `tests/fixtures/**`, `tests/golden/**`: standard frozen sets.

## Invariants (must hold; checkable forms in the shape)

1. Recipe truth: for every fixture plan, the payload's
   `target_files[].path` set equals the set of `target_header` values in
   that plan's `proposed_extractions`, with the symbol->file assignment
   preserved (capped, truncation flag correct).
2. Old payloads decode: JSON without the new keys round-trips to empty/false
   (`serde(default)`); the key-stability test is amended in the same commit
   as the DTO change and lists exactly old-keys-union-new.
3. Collapsed by default: every `.extraction-details` renders `hidden`; the
   summary row alone (plan id, target, N symbols, saving, risk) shows.
4. Keyboard-accessible disclosure: the toggle is a real `<button>` with
   `aria-expanded`; Enter/Space work natively; card body still selects the
   hotspot row (toggle stops propagation).
5. The 9 story tests in `extraction_card_story.rs` stay GREEN UNEDITED
   (they are source-string tests; the shape's layout keeps every locked
   substring and the locked beat order inside the detail section).
6. No new analysis in the UI layer: the renderer adds no fetch/XHR and
   reads only `graphData.extraction_proposals` fields.
7. Command honesty: the card names only commands that exist
   (`tool apply <repo-root> --plan <plan_id>`, `tool migrate <repo-root>
   --plan <plan_id> --preview`) with the proposal's real `plan_id`; help
   copy states the UI never mutates the repo.
8. Preview contract byte-faithful: `just-preview-btn`, label,
   `data-preview-*`, `bindPreviewControls` wiring, visibility condition -
   unchanged; `preview.js`/`api.js` byte-identical.
9. Payload order rendered as-is; empty/absent payload hides the section
   (`graphData.extraction_proposals || []`, `proposals.length === 0` kept).
10. Escape discipline for every new interpolation (paths, symbol names);
    zero added hex; authored content pure ASCII.

## Frozen locks inherited from the ledger

| Lock | Proof command |
|---|---|
| `tests/golden/**` byte-locked (`UPDATE_GOLDENS=1` FORBIDDEN) | touch-set + reviewer hop; this milestone changes no producer serialization (plans_plans.json stays byte-equal because `OptimizationPlan` serialization is untouched) |
| `tests/fixtures/**` byte-frozen (all branches) | touch-set check + status census at review |
| `m00_facts_path_parity` (payload planner == CLI plan output) | feed change maps existing plans only; scoped integration suite green unedited |
| Mojibake/UTF-8 gate | `python tools/encoding_guard.py --staged` pre-commit; `--all` CI |
| Hygiene (R1/R2 blocking) | `python tools/hygiene_check.py` exit 0 |
| Canonical census | `python tools/testrun.py --distill -p ariadne-ui` / `-p ariadne-cli` scoped; unscoped census at closeout (`--baseline`-FILE errata still open) |
| UI runtime smoke | `python tools/distill.py -- python tools/ui_smoke.py` -> `ui_smoke: PASS` (allowlist row still owed; distill route sanctioned per deferred findings row) |
| Zero added hex | `python tools/theme_token_check.py --check-added --base <rev>` silent |
| graph.js E-I6 cap | `static_html_sections.rs::test_extraction_impact_preview_overlay` (file untouched) |
| Stale `target\debug\tool.exe` lock | deferred ledger row: close the stale process before normal-target census/bench |

## Concurrency / serialization (checked against ledger 2026-10-04)

- `ui-dark-grey-theme`: T1 LANDED (`41ec354`); only its ledger-row flip is
  outstanding - an orchestrator docs duty, no asset conflict. No open
  milestone currently plans to touch `index.html`/`styles.css`/`hotspots.js`
  besides this one; the prior milestone's index.html serialization is spent.
- `cpp-brace-depth-underflow` (PLANNED): touches only
  `crates/lang-cpp/{tokenizer,parser}` - file-disjoint from all three
  seams. Benign interaction only: if it lands between T-B and review,
  plan/payload *contents* may shift (bug fix); no test this milestone owns
  pins payload bytes. Census totals may drift by its test count - reconcile,
  do not chase.
- `ui-dark-grey-theme` closed out `bench_compare` precedent; T-B is a
  non-assets Rust change, so the assets-only exemption does NOT auto-carry
  - see shape risks for the bench stance.

## Acceptance gate (milestone as a whole)

1. T-A: `python tools/testrun.py --distill -p ariadne-ui` -> `0 failed`
   with the amended key-stability test + round-trip/legacy-decode tests
   green; dto diff is purely additive (`#[serde(default)]` on both new
   fields).
2. T-B: `python tools/testrun.py --distill -p ariadne-cli` -> `0 failed`
   (new feed unit tests assert invariant 1 on constructed plans); scoped ui
   suite still green; `m00_facts_path_parity` unedited.
3. T-C: `python tools/testrun.py --distill -p ariadne-ui` -> `0 failed`
   including the 9 story tests and all C4-keep-literal consumers UNEDITED;
   new `extraction_card_playbook.rs` locks invariants 3, 4, 6, 7.
4. `ui_smoke: PASS` (live card renders playbook from the real payload);
   zero added hex; hygiene + encoding exit 0 at every hop.
5. Census at closeout reconciles: 1181 prior + N new tests (T-A + T-B + T-C),
   suites unchanged unless a new cli test file adds a suite.
6. Review-gate commit-vs-touch-set enforcement per handoff; touch sets equal
   the scope lists exactly.
7. Sizing: three sequential handoffs, each <= 1 crate, <= 10 gate
   invocations, <= ~1200 produced lines (expected ~150-250 / ~150-300 /
   ~500-700). Ordering T-A -> T-B -> T-C is a hard dependency chain.
