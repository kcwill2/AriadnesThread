# Plan - ui-extraction-wizard

Milestone key: `ui-extraction-wizard` (verbatim). Contract of record:
`tasks/ui-extraction-wizard-kickoff.md` and `tasks/ui-extraction-wizard-shape.md`
(this file is the digest; on conflict the two `tasks/` artifacts win).

Authored at HEAD `97dd836` on `main` (clean tree). Follows the post-M28 track
pattern: no guessed M-number, live ledger row until closeout.

## Milestone key and rationale

`ui-extraction-wizard` is kept because it names the deliverable the user
demanded ("This feature requires a wizard or something") without overclaiming
the endgame. The auto-extraction half gets its own key,
`extraction-auto-apply`, because it is backend-first (mutation routes, outcome
reporting) with a UI trigger, and the `ui-` prefix would understate it.

## Capability map (summary)

| Capability | State |
|---|---|
| Proposal payload (plan id, target, before/after, saving, risk, consumer TU count, required headers) | EXISTS (`crates/ui/src/dto.rs:261-286`, feed `crates/cli/src/commands/ui.rs:486-523`) |
| Files-to-create rows (path + per-file symbols) | EXISTS, capped 8 files / 12 names (`dto.rs:244-259`) |
| Full per-file symbol manifest | PARTIAL - in the plan model, not in the API (`crates/cluster/src/plan.rs:35-45`) |
| Per-file required includes of the new file | ABSENT at the API; computable read-only (`crates/transform/src/engine/mod.rs:95-128`) |
| Consumer rewrites / dry-run diff | EXISTS, CLI only (`crates/cli/src/commands/transform.rs:81-127`) |
| Transactional apply with snapshot, three tiers, rollback | EXISTS, CLI only (`crates/validate/src/lib.rs:150-321`) |
| Any UI mutation route | ABSENT - every route is a GET (`crates/ui/src/server.rs:48-79`) |
| Plan-by-id resolution in the server | ABSENT - boot plans are dropped after mapping |
| Stepper / wizard interaction | ABSENT |
| Auto-extraction driven from the UI | ABSENT (follow-on) |

## Wizard flow and stepper contract deltas

Host is a modal (precedent `index.html:423-585`), entered from the collapsed
proposal card; the panel keeps the summary-card-first model.

- S0 Overview - payload as-is, no delta.
- S1 Files to create (PRIMARY) - new route: uncapped per-file manifest.
- S2 Per-file manifest - new route fields `required_includes`,
  `symbol_count`, `includes_unavailable` (closes the backend gap).
- S3 Impact and risk - new fields `cycle_error`, `move_removals`,
  `copy_only`.
- S4 Dry run - new route `dry-run`: new file paths plus consumer rewrites
  (old/new includes, side-effect preservations).
- S5 Execute - exact CLI recipe only; no mutation route in this milestone.
- S6 After apply - describes the three tiers and rollback; no UI surface.

Routes: `GET /api/extraction/{plan_id}/manifest`,
`GET /api/extraction/{plan_id}/dry-run`, served from a new
`crates/ui/src/routes/api_extraction.rs` over a plain-data
`ExtractionManifestReader` the CLI injects into `UiState`. Errors: `503`
loading, `404` unknown plan, `409` stale generation.

## Auto-extract v1 boundary vs deferred

In: stepper, engine-sourced per-file symbols + includes, dry-run from the same
code path `tool apply` uses, exact execution recipe, help copy that names the
boundary. Deferred to `extraction-auto-apply`: `POST .../apply` behind a
dry-run-then-approve gate on the current generation epoch, three-tier outcome
and rollback reporting, apply mutex, post-apply rebuild, and the help-copy flip
from "the UI never applies" to "the UI applies with a dry-run first".

## Clarity disposition

T-A LANDED on main (oracle file present, `main.rs:10` mod line, light `--green`
`#166534` at `themes.css:188`) and is inherited as a frozen lock. T-B is not
dispatched as written: its per-file content shape and token choices transfer
into wizard steps, its card-detail placement and its 4-lock amendment table
transfer to this milestone's C5. Branch `ui-extraction-card-clarity-ta` and
the ledger row stay with the concurrent hand; recommendation is it stops after
T-A and amends the row to "T-B SUPERSEDED by ui-extraction-wizard".

## Handoff split with sizes

| Hand | Crate | Kind | Lines | Gates | Depends on |
|---|---|---|---|---|---|
| W-1 | crates/ui | dto.rs module split (file is at exactly 1000 lines) | ~300-450 | ~6 | - |
| W-2b | crates/ui | reader trait, wizard DTOs, routes, error shapes | ~400-600 | ~9-10 | W-1 |
| W-2a | crates/cli | read-only provider over the isolation engine | ~350-500 | ~10 | W-2b |
| W-3a | crates/ui | new `extraction-wizard.js` asset + stepper shell + steps 0-1 | ~500-700 | ~10-11 | W-1 |
| W-3b | crates/ui | steps 2-4 live, help copy, C5 amendments | ~400-600 | ~10 | W-3a, W-2a |

Sequential on main: every hand adds a `crates/ui/tests/ui_test` module line to
`main.rs`, so no two hands are parallel (standing clarity precedent).

## Top risks

1. TU-scale volume (1312 symbols): capped per-step fetches, boot payload
   provably unchanged.
2. Plan staleness across a watcher generation swap: `409` refusal plus wizard
   reset.
3. Consumer-discovery cost on the dry-run route: per-(plan, epoch) cache.
4. Copy-vs-move overclaim: the C++ backend does not strip moved declarations
   today; `copy_only` flag plus a locked sentence.
5. Card-lock amendment creep: C5 is the complete table, reviewer-enforced.
6. New-asset registration chain (lib.rs const, route, handler, index.html,
   ui_smoke mirror, export inline lock) misses one site.
7. `dto.rs` at exactly 1000 lines makes any DTO addition an R2 failure - W-1
   is a hard predecessor.
