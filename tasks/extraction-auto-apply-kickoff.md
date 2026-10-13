# Milestone Kickoff - extraction-auto-apply

Milestone key (verbatim for the planner/ledger): `extraction-auto-apply`.
Follows the post-M28 track pattern used by `ui-extraction-wizard`: no
M-number is guessed, the `milestone-lifecycle` kickoff assigns one only if
the ledger numbering demands it, and the live ledger row registered
2026-10-12 stays until closeout. Authored by the architect at HEAD
`023558c` on `main` (working tree clean, read-only survey).

The row of record in `tasks/milestone_ledger.md` reads (abbreviated to its
operative clauses): "PLANNED / DEFERRED - DISPATCH GATE NOW OPEN (the
wizard CLOSED 2026-10-12; no handoffs authored - a kickout is owed before
dispatch; boundary of record unchanged: wizard shape section 4 + the
wizard plan's 'Auto-extract v1 boundary vs deferred' section)". This
document is that kickout. The boundary of record is restated in shape
section 4; the deferred scope of record is quoted verbatim in section 7.

## 1. Intent

The user accepted the live wizard build, verbatim: "I see it now. it
looks good." That verdict closed `ui-extraction-wizard` at `023558c` and
authorized this follow-on. The demand this milestone discharges is the
user's original one, verbatim from the wizard kickoff: "Ideally with an
automatic extractor that creates the new files and moves the symbols."

**Behavior change.**

1. The wizard's Execute step (S5) becomes a real action: a
   `POST /api/extraction/{plan_id}/apply` behind an explicit
   dry-run-then-approve gate on the current analysis generation epoch.
   Today S5 renders the CLI recipe only and states the boundary itself
   (`crates/ui/assets/extraction-wizard.js:395-405`
   `renderWizardStepExecute`: "The UI shows this recipe and never runs
   it").
2. The apply reuses the shipped transactional code path - the CLI
   `apply_transform_plan` core over `TransactionApplier::apply_with`
   (`crates/cli/src/commands/transform.rs:129-192`,
   `crates/validate/src/lib.rs:150-201`, rollback `:297-321`). No new
   write path is created anywhere; `crates/validate/**` is not edited.
3. The After-apply step (S6) becomes real outcome reporting: the three
   tier results, the committed-vs-rolled-back verdict, and the verbatim
   `ApplyOutcome::format_output` text
   (`crates/validate/src/lib.rs:58-81`). Today S6 is static description
   copy (`extraction-wizard.js:407-416`
   `renderWizardStepAfterApply`).
4. The help copy flips from "the UI never applies" to the claim of
   record: "the UI applies with a dry-run first, validation in three
   tiers and automatic rollback" (today: `index.html:548`, `:551`).

**Underlying why.** The analyze -> apply spine in `Intent.md`
(~391-413: Plan transformation -> Preview -> Apply -> Validate -> Measure
-> Commit or Roll Back) is documented product intent, and
`Spec.md`/`Intent.md` section 13 add "the system should never treat a
successful graph transformation as sufficient evidence of correctness".
The wizard made every step of that spine honest to look at; this
milestone makes the Apply half drivable from the surface where the user
makes the decision, without weakening a single safety property the CLI
already provides. The mutation machinery is reused, not rebuilt, so the
spine's guarantees (snapshot, three tiers, automatic rollback) are the
guarantees the UI inherits.

**Blast radius.** `crates/ui/src/{routes/, dto/extraction.rs, state.rs,
server.rs, lib.rs}` plus one new route module and one new ui_test module;
`crates/cli/src/commands/ui.rs` plus two sibling modules (one new,
`extraction_apply.rs`; one receiving an extraction, `boot_progress.rs`)
and `crates/cli/src/commands/transform.rs`; the front-end assets
`extraction-wizard.js`, `index.html`, `styles.css`; and a named
two-lock amendment in the card-family test files. `crates/validate/**`,
`crates/transform/**`, `crates/artifact/**`, `crates/cluster/**` and
`crates/lang*/**` are READ and never modified.

## 2. Capability map (summary)

Full table with file:line citations is shape section 1. Short form:

| Capability | State today |
|---|---|
| Transactional apply (write files, rewrite consumers, snapshot, 3 tiers, rollback) | EXISTS, CLI only (`tool apply`, `crates/cli/src/commands/transform.rs:129`) |
| Apply outcome as structured data (tiers, verdict, rollback) | PARTIAL - `ApplyOutcome` exists and is `Serialize`, but the CLI prints it (`crates/validate/src/lib.rs:48-81`, `transform.rs:392-393`) |
| UI route that mutates anything | ABSENT - every route in `crates/ui/src/server.rs:50-84` is a GET |
| Dry-run-then-approve gate | ABSENT - the gate is this milestone's core new contract |
| Per-generation apply serialization | ABSENT |
| Post-apply rebuild | PARTIAL - the fingerprint watcher rebuilds on change (`crates/cli/src/commands/ui.rs:638-708`) but nothing requests it after an apply |
| Apply outcome reporting in the UI | ABSENT - S6 is static copy |
| Help copy stating the apply claim | ABSENT - the copy states the opposite (never applies / never mutates) |

What this milestone adds is a mutation route that reuses the CLI
transaction verbatim, a gate that binds an approval to one plan on one
generation epoch, one-apply-at-a-time serialization, and honest outcome
reporting. It does not add a second transactional implementation.

## 3. Scope in (each with its reason)

1. `POST /api/extraction/{plan_id}/apply` in a NEW route module
   `crates/ui/src/routes/api_extraction_apply.rs`, with the host-implemented
   `ExtractionPlanApplier` trait mirroring the shipped
   `ExtractionManifestReader` seam (`crates/ui/src/routes/api_extraction.rs:30-34`).
   The apply handler lives in its own module so the wizard's W1 no-write
   string locks over `routes/api_extraction.rs` survive UNEDITED (they
   forbid `post(`/`fs::write`/`ArtifactManager` tokens in that file only,
   `crates/ui/tests/ui_test/extraction_wizard_api.rs:346-371`).
2. The dry-run-then-approve gate: the GET dry-run route mints a
   single-use, epoch-bound approval token (new additive field on
   `ExtractionDryRunDto`); the apply route accepts only a live token.
   This is the explicit gate the ledger's safety invariants demand.
3. Apply outcome DTOs (three tiers, verdict, new files, verbatim
   `ApplyOutcome::format_output` text) in `crates/ui/src/dto/extraction.rs`.
4. The CLI applier `EngineBackedExtractionApplier` in a new sibling
   module `crates/cli/src/commands/ui/extraction_apply.rs`, installed with
   the generation in `swap_into_state`, delegating every mutation to the
   extracted `apply_plan_with` core in `commands/transform.rs` (no
   duplicate transactional logic).
5. The per-generation apply mutex (one apply at a time; concurrent
   applies refused, never queued) and the epoch-staleness enforcement
   beyond the GET 409 surface (applier epoch check plus token epoch
   binding).
6. Post-apply rebuild: no new route - the shipped fingerprint watcher
   already detects the mutated inputs and rebuilds; the apply flow tells
   the user what to expect and the existing payload-swap reset clears the
   wizard (shape C6).
7. The front-end Execute step (S5) becomes the approval surface (apply
   control + in-flight state + refusal copy) and the After-apply step
   (S6) renders the live outcome.
8. The help-copy flip in `crates/ui/assets/index.html` with the named
   lock amendments (shape C8).
9. The `ui.rs` ceiling discharge: the first cli hand extracts the
   boot-progress family into the existing `commands/ui/boot_progress.rs`
   sibling BEFORE any `ui.rs` growth (the deferred W-2a F2 row demands
   extraction first, not growth).
10. `apply_plan_with` extraction in `commands/transform.rs` so the CLI
    and the UI share one transactional core (the CLI wrapper keeps its
    stdout byte-identical on the success path).

## 4. Scope out (each with its reason)

- **Remote apply / bind widening.** No new `--expose` surface, no change
  to `resolve_bind_addr` (`crates/cli/src/commands/ui.rs:834-839`); the
  UI keeps the localhost default with its startup warning (`:843-850`).
  The mutation surface stays behind the same-origin CORS predicate
  (`server.rs:36-48`).
- **A new rebuild route or rebuild channel.** The watcher is the rebuild
  mechanism; a second trigger would duplicate the swap machinery.
- **Background or scheduled apply.** No apply runs without an approval
  minted by a dry-run fetch in the same generation; no batch apply, no
  multi-plan apply, no re-planning from the UI.
- **Editing `crates/validate/**`, `crates/transform/**`,
  `crates/artifact/**`, `crates/lang*/**`.** The milestone reuses the
  transactional machinery as shipped; `crates/validate/src/lib.rs` at
  929/1000 lines is deliberately left untouched.
- **New generated-file text, symbol bodies, or planner changes.** Same
  exclusions the wizard carried; the apply writes exactly what the
  engine and the plugin produce.
- **The styles.css split.** Over the 1000-line ceiling and invisible to
  R2; the apply styles are one contiguous net-capped block, the split
  stays with its dated deferred row.
- **Frozen sets:** `tests/golden/**`, `tests/fixtures/**`, boot payload
  bytes, theme assets (`themes.css`, `theme.js`,
  `tools/theme_token_baseline.json`), `preview.js`, `api.js`, `graph.js`,
  `hotspots.js`, `main.js`, `traversal.js`, `file-view.js`,
  `provenance.js` (wizard asset excluded - it is this milestone's
  front-end hand), and the 18 card-family locks except the two named
  amendments (shape C8).

## 5. Invariants (checkable forms in the shape)

1. Single write path: every UI-triggered mutation executes through
   `TransactionApplier::apply_with` via the extracted `apply_plan_with`
   core; the ui crate's apply route module contains no write call.
2. Dry-run-then-approve gate: an apply is accepted only with a
   single-use token minted by the dry-run route for the same plan id and
   the current epoch; missing, replayed, or foreign tokens are refused.
3. Epoch binding: applier epoch must equal the live epoch, and the token
   carries that epoch; an epoch swap invalidates outstanding approvals.
4. One apply at a time: a second apply while the generation's lock is
   held is refused and does not consume the approval.
5. Honest outcome: the outcome DTO carries the three tier results, the
   verdict, and the verbatim `format_output` text; a rolled-back apply
   leaves the tree byte-identical.
6. Dry-run/apply parity: the files an apply writes are exactly the
   approved dry-run's new files under the same staging decision.
7. Approval surface: the Execute step's apply control is enabled only
   with a token present; without one the UI shows the dry-run-first
   instruction.
8. Bind default unchanged: no route or flag widens exposure.
9. ASCII everywhere: every apply response body is pure ASCII.
10. Lock survival: 16 of the 18 card-family locks survive byte-identical,
    exactly 2 are amended per the named table; the wizard GET-only and
    no-write locks survive UNEDITED.

## 6. Hygiene facts that force ordering (measured at HEAD)

- `crates/cli/src/commands/ui.rs` is 986 lines, ceiling 1000: margin 14.
  The deferred W-2a F2 row (`tasks/milestone_ledger.md`, "W-2a verdict
  F2") rules that any `extraction-auto-apply` hand needing ui.rs growth
  "must extract a sibling module rather than spend the ceiling". A-1
  therefore moves the boot-progress family (`make_bar_progress`,
  `make_bar_progress_with_sink`, `make_boot_progress`,
  `make_boot_progress_with_sink`, `panic_payload_text`,
  `surface_boot_failure`, `drive_boot_build`, `build_boot_generation`;
  `ui.rs:710-860` minus the bind helpers) into the existing
  `commands/ui/boot_progress.rs` (115 lines today), taking ui.rs to
  roughly 830-850. A-4's `swap_into_state` install (+~14 lines) then fits
  with room to spare.
- The extraction's frozen test-import surface is enumerated and small:
  `boot_progress_tests.rs:1-8` and `boot_panic_tests.rs:7-10` import the
  moved names via `use super::{...}`; the precedent is the `stage_spans`
  move's re-export comment (`ui.rs:13-23`), which kept every frozen import
  line unedited.
- `crates/cli/src/commands/transform.rs` is 415 lines: the `apply_plan_with`
  core extraction lands with headroom (`commit_or_rollback` and the tier
  sequence move into it).
- `crates/ui/src/dto/extraction.rs` is 166 lines after W-1; the apply
  DTOs and the `apply_token` field land well under the ceiling.
- `crates/ui/src/routes/api_extraction.rs` is 103 lines and stays W1-locked;
  the apply module is new, so the no-write string locks keep scanning a
  file this milestone does not grow.
- `crates/ui/assets/extraction-wizard.js` is 446 lines: S5/S6 growth
  stays far under 1000.
- `crates/ui/assets/index.html` is 640 lines: two sentences flip, no new
  script tags, so the asset-count locks are untouched.
- `crates/ui/assets/styles.css` is over the 1000-line ceiling and
  invisible to R2: the apply-button and outcome rules are one contiguous
  net-capped block using existing theme tokens (zero added hex).
- `crates/validate/src/lib.rs` is 929/1000: untouched by design.
- `ExtractionDryRunDto` gains one field; the caller census at HEAD finds
  exactly three construction sites (`extraction_reader.rs:182`,
  `extraction_reader.rs:417`, `extraction_wizard_api.rs:100`), so A-3
  carries a declared two-line rider in the cli reader plus the test
  sample, gated by `cargo check -p ariadne-cli --all-targets`.

## 7. Inherited deferred findings and ledger duties

- **W-2a verdict F2 (ui.rs 986/1000 ceiling watch)** targets this
  milestone by its own text: "any `extraction-auto-apply` hand needing
  ui.rs growth must extract a sibling `commands/ui/` module rather than
  spend the ceiling". Discharged by A-1.
- **The bench `DOCUMENTED_PHASES` blocker** (`tools/bench_compare.py:84`,
  the tuple label predates the M23.5 rename) stays an OPERATOR row; this
  milestone inherits it as the documented bench stance for every hand
  that touches `crates/*/src` (zero-diff producer proofs; the optional
  `--bench stage5-memory --runs 6` compensating run at A-4, the W-2a
  precedent).
- **The `--baseline` FILE-form errata** stands: workspace census runs via
  the `--distill` substitute; this kickoff is the 9th recorded encounter.
  The census anchor of record is `1483 passed / 0 failed / 13 ignored /
  112 suites` @ `24c2f7e`; every hand reports deltas from a clean-tree
  scoped baseline, never hardcoded totals.
- **Deferred scope of record (verbatim from the ledger row):**
  "`POST /api/extraction/{plan_id}/apply` behind the dry-run-then-approve
  gate; three-tier outcome + rollback reporting
  (`ApplyOutcome::format_output`); per-generation apply mutex; post-apply
  rebuild; generation-epoch staleness enforcement beyond the 409 surface;
  the help-copy flip from 'the UI never applies' to 'the UI applies with
  a dry-run first, validation in three tiers and automatic rollback'."
- **Graduation backlog audit (D4):** all four warn-only rows stay
  warn-only this milestone. R3 naming: no new legacy names are authored,
  but the flip requires a full milestone with zero new findings and this
  one does not close it. Clippy `-D warnings`: baseline unchanged, no new
  warnings expected. Bench CI: runner-class incomparability unchanged.
  Theme Phase B: belongs to the theming plan, not this track.

## 8. Handoff split summary (sizing detail in shape section 6)

Five hands, each one crate or module dir, each well inside the gate and
line budgets:

| Hand | Crate | Kind | Depends on |
|---|---|---|---|
| A-1 | crates/cli | refactor (ui.rs boot-progress family extraction) | - |
| A-2 | crates/cli | refactor (apply core extraction in transform.rs) | - |
| A-3 | crates/ui | backend contract (applier trait, apply DTOs, POST route, state seam, route tests) | - |
| A-4 | crates/cli | apply provider (applier implementation, swap install, provider tests) | A-1, A-2, A-3 |
| A-5 | crates/ui | front-end (S5 approval surface, S6 outcome, help-copy flip, C8 amendments) | A-3, A-4 |

A-1 and A-2 are file-disjoint inside crates/cli and A-3 is a different
crate, so those three could theoretically run in parallel; the house
default is strictly serial on main with one scoped landing commit per
hand (the wizard's "never parallel" precedent), and A-4/A-5 are serial by
contract. Every hand that adds a `crates/ui/tests/ui_test` module touches
`main.rs` (+1 mod line each: A-3 and A-5).

## 9. Frozen locks inherited from the ledger (with proving command)

| Lock | Proof |
|---|---|
| `tests/golden/**` byte-locked, `UPDATE_GOLDENS=1` FORBIDDEN | touch-set + reviewer hop; this milestone changes no producer serialization |
| `tests/fixtures/**` byte-frozen | touch-set check + status census at review |
| `m00_facts_path_parity` (payload planner == CLI plan output) | the apply adds no planner run; the applier resolves plan ids from the generation's own plans |
| Boot payload byte-parity (W9 capture) | the apply DTOs are route payloads, never boot payload fields; the W9 lock in `extraction_reader.rs` tests stays unedited |
| Themes are the single color-definition site; zero added hex | `python tools/theme_token_check.py --check-added --base <start-rev>` silent (combined form; the bare form is a usage error per the ledger) |
| `tools/theme_token_baseline.json` byte-locked counts | expected to appear in no diff of this milestone |
| Hygiene R1/R2 blocking | `python tools/hygiene_check.py` exit 0 |
| Mojibake/UTF-8 gate | `python tools/encoding_guard.py --staged` pre-commit, `--all` at hop close |
| UI runtime smoke | `python tools/ui_smoke.py` PASS (distill route if the session allowlist refuses the direct run) |
| Census anchor rule | reconcile from `1483 passed / 0 failed / 13 ignored / 112 suites` @ `24c2f7e`; deltas attributed per hand |
| `--baseline` FILE form errata | workspace census via the `--distill` substitute (9th encounter) |
| Bench gate | hands touching `crates/*/src` carry the DOCUMENTED_PHASES stance: zero-diff producer proofs, timing gate exit 2 recorded, no timing number attributed |
| Localhost bind default | `resolve_bind_addr` and `--expose` appear in no diff; existing binding behavior tests unedited |
| The 18 card-family locks | exactly 2 amended per shape C8, 16 byte-identical, zero deletions |
| Wizard GET-only / no-write locks | `extraction_wizard_api.rs:325-371` stays UNEDITED (the apply handler and POST registration live in new files) |

## 10. Acceptance gate (milestone as a whole)

1. A-1: `commands/ui.rs` drops to roughly 830-850 lines with the
   boot-progress family in `boot_progress.rs`; the scoped cli suite is
   green UNEDITED; `cargo check -p ariadne-cli --all-targets` exit 0;
   hygiene exit 0.
2. A-2: `apply_plan_with` exists as the single transactional core;
   `apply_transform_plan` keeps its signature and its success-path
   stdout; the m10/m20/m23 integration apply suites are green UNEDITED.
3. A-3: the POST route exists and answers the full refusal table; the
   dry-run route mints the token; the state seam installs with the
   generation; scoped ui suite green with the new locks; cli check exit 0
   including the two-line rider.
4. A-4: the applier answers a real fixture apply end-to-end; the written
   file set equals the approved dry-run's new files; a consumed token
   cannot apply twice; a held lock refuses without consuming the token;
   the scoped cli suite and the consumer ui suite are green.
5. A-5: the Execute step applies only with a token, shows in-flight
   state, and renders the outcome (tiers, verdict, verbatim output text)
   in S6; the help copy carries the flipped claim;
   `ui_smoke: PASS`; zero added hex; hygiene and encoding exit 0; the C8
   amendments are the only card-lock edits.
6. Every response body of the apply route is pure ASCII (route test
   assertion), and the outcome text equals
   `ApplyOutcome::format_output` for the same outcome.
7. Census at closeout reconciles from the anchor with every delta
   attributed; `tools/ui_smoke_tests.py` discover unchanged (no route
   rows are added; the POST route is not a static-asset probe).
8. Review-gate commit-vs-touch-set enforcement per hand; touch sets
   equal the scope lists exactly; no bench timing number is attributed to
   any hand.
9. The rollback story is proven, not asserted: a fixture-level test
   shows a failed tier restores the tree byte-identically (validate's
   `verify_restored` machinery, exercised through the applier).
10. Closeout: ledger row flip, walkthrough, memory mirror, worktree
    census, and the A-1 headroom measurement recorded (measured, not
    predicted).
