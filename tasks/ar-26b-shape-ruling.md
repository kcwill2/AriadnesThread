# AR-26b Shape Ruling - Cone Invariant Scope (OPEN ITEM RESOLVED)

Issued 2026-10-11 by the Architect. Binding for AR-26b dispatch. Resolves
the OPEN ITEM box of `tasks/ar-26b-handoff.md` (planner board ASK). This
file + the final report are the entire deliverable: no code, no ledger, no
handoff edits (the handoff's touch-set does not include itself).

## Evidence re-verified against the working tree (2026-10-11)

- `metrics_io.rs:330/370/374/382` - `ariadne_cluster::plan::OptimizationPlan`
  in production `impl ArtifactManager` signatures
  (`save_plan_artifacts`/`load_plan_artifacts`/`load_plan_checked`). CONFIRMED.
- `transform_io.rs:15/47/80/84` - `ariadne_transform::ExtractionResult` /
  `MigrationPreview`. CONFIRMED. Additional fact found at verification:
  `transform_io.rs:37-39` calls `result.write_to_disk(...)` - the DTO carries
  transform-owned behavior, which forecloses relocating it (see Option-B
  assessment).
- `apply_io.rs:16/22/25` - `ariadne_validate::ApplyOutcome`, including
  variant pattern-matching (`Committed`/`RolledBack`). CONFIRMED.
- `crates/scheduler/Cargo.toml:16` - `ariadne-report = { path = "../report" }`
  is today's only artifact-store edge. CONFIRMED.
- `crates/report/Cargo.toml:17-19` - report itself declares
  cluster/transform/validate TODAY. Therefore scheduler's cone already
  contains all three crates via report at HEAD: Option A is a cone-CONTENT
  delta of exactly ZERO, and the audit's "cone loses the stack" claim was
  already false under any reparent that keeps the IO VERBATIM. CONFIRMED.
- `crates/worker-core/Cargo.toml` - clean of report/cluster/transform/
  validate; no back-door into the cone. CONFIRMED.
- `crates/artifact/**` does not exist at ruling time (26a in flight; correct
  for its contract). The cluster/transform/validate local deps arrive at
  26a2 and are MANDATORY, not incidental: no implementer may "clean up" or
  feature-hide them to make a stronger tree assertion pass.

## Why no full-exclusion shape survives this chain's locks (Option B killed)

1. DTO relocation to artifact (or a neutral types crate): `ExtractionResult`
   carries `write_to_disk` behavior living in transform; `OptimizationPlan`
   and `ApplyOutcome` are owned public domain types whose relocation guts
   cluster/validate and rewrites their crate surfaces - non-VERBATIM by
   definition and architecturally wrong (DTOs belong to their domains).
2. Signature inversion (`&impl Serialize` / raw-byte returns): rewrites the
   API of three IO modules and forces deserialization/annotation changes at
   every cli/pipeline call site - violates the F3/F4 zero-consumer-edit
   lock and VERBATIM, and erodes type safety at the persistence boundary.
3. Split artifact again: E0116 - `impl ArtifactManager` blocks must live in
   artifact with the type; `metrics_io.rs` mixes plan-typed IO with
   load-report IO in ONE file; any finer split is an API redesign of
   ArtifactManager, i.e. a rewrite this chain forbids.
4. Feature-gating: m17's `reachable_local_crate_names` walks
   `[dependencies]` lines and `cargo tree -e normal` shows declared edges
   regardless of feature activation - gating hides code, never the cargo
   edge, and default-off features would change the API surface (= behavior).

Option A is the ONLY dispatchable shape under {VERBATIM, bincode byte-lock
authority with m00/m23 UNEDITED, zero consumer edits}. lang-cpp note:
artifact sits in the scheduler/report cone, not lang-cpp's (scheduler's
lang-cpp edge is dev-only; `-e normal` excludes it) - the frozen lang-cpp
cone is unaffected by this ruling either way.

## Ruling 1 - the invariant 26b machine-locks (exact predicate)

Define `Cone(scheduler)` = the set of reachable local crate names per
m17's `reachable_local_crate_names` walk over transitive local
`[dependencies]` edges from `crates/scheduler/Cargo.toml`, and
`Tree(scheduler)` = the set of package names emitted by
`cargo tree -p ariadne-scheduler -e normal`. LOCKED invariant:

    "ariadne-report" NOT-IN Cone(scheduler)  AND
    "ariadne-report" NOT-IN Tree(scheduler)

Mechanisms, both required: (a) m17 test
`scheduler_cone_excludes_report` with
`const SCHEDULER_CONE_FORBIDDEN: [&str; 1] = ["ariadne-report"];` plus the
planted-edge non-vacuity self-test; (b) gate 6 cargo-tree grep of the
exact token `ariadne-report` against the capture from the implementer's
own run (token is unambiguous - no other workspace package name contains
it as a substring). NON-VACUITY lock: the planted self-test proves the
walker detects a planted report edge; the guard-proper RED before the swap
proves the predicate is false at dispatch HEAD - journal verbatim, never
commit the RED.

EXPLICITLY NOT LOCKED (do not write it into any const array, gate, or
amendment): absence of `ariadne-cluster`, `ariadne-transform`,
`ariadne_validate`. Those three are EXPECTED cone members through
`scheduler -> artifact`; a future handoff may add them to
`SCHEDULER_CONE_FORBIDDEN` only via a separate shape ruling once the
API-redesign of Ruling 3 lands.

## Ruling 2 - audit A3 intent: PARTIALLY satisfied, plainly

FULLY delivered by the chain: the AR-26 Change itself (new artifact crate
owning the store IO; report depends DOWN; `scheduler -> report` dropped);
honest crate names; report-as-presentation for the plugin seam; plugin-cone
compounding (m17 `TRANSITIVE_FORBIDDEN_DEPS` extension locks
`ariadne-artifact` out of plugin reach); zero behavior change; byte-locks
UNEDITED. NOT delivered: the Impact line "scheduler's cone loses the
cluster/transform/validate stack". HONEST RESIDUAL: after 26b, scheduler
still transitively compiles cluster/transform/validate via
`artifact -> {cluster, transform, validate}` - cone content byte-identical
to today's HEAD (report dragged them before; artifact drags them after).
The stack loss is unattainable by ANY reparent under VERBATIM - only an
ArtifactManager API redesign buys it.

## Ruling 3 - residual disposition: DEFERRED LEDGER ROW, not a follow-up AR

The residual is a layering taste point, not a defect: a store typed by the
domain DTOs it persists is defensible layering, it loads no gate/test/census
today, and its cone content equals the status quo - scheduling an AR now
would fund a speculative API redesign with no named consumer harm. Disposition
is a dated ledger row (orchestrator's closeout ritual, not this ruling)
registering the residual with this standing promotion charter, verbatim, so
a future planner can promote it without re-deriving the shape:

AR-26c (promotion candidate) - domain-neutral artifact store: split
`ArtifactManager` into a core store (paths, bytes, manifests, fingerprints;
deps stay at repo/index level) and typed facades for plan/extraction/
migration/transaction IO living beside the domain crate that owns each DTO
(cluster/transform/validate), so `ariadne-artifact` links without dragging
those three; m00/m23 bincode wire bytes stay byte-identical with locks
UNEDITED; the report re-export shims keep all 22 external consumers
UNEDITED; scheduler `26` + worker-core `8` conserve with zero test edits;
then `SCHEDULER_CONE_FORBIDDEN` is amended in ONE ruling to
`[ariadne-report, ariadne-cluster, ariadne-transform, ariadne-validate]`
with the matching cargo-tree gate. Out of scope: relocating DTO
definitions out of their domain crates (`ExtractionResult` owns behavior -
`write_to_disk` - and stays transform-owned).

## Ruling 4 - Option-A guard wording: RATIFIED, with three precision notes

Ratified verbatim (no implementer-visible change):
`SCHEDULER_CONE_FORBIDDEN: [&str; 1] = ["ariadne-report"]`, the guard and
planted-self-test test bodies, the panic message, `-e normal` on gate 6,
the m17 8->10 reconciliation, and the RED-first protocol. Precision notes:
(P1) gate 6's "NO occurrence" check is on the exact token `ariadne-report`
in the package-name position of the distilled tree/capture; (P2) cluster/
transform/validate must be described as EXPECTED cone members in the
journal, never asserted absent; (P3) the OPEN ITEM box's closing line
"The 4-crate exclusion becomes a follow-up AR" is SUPERSEDED by Ruling 3
(deferred ledger row with charter).

## Dispatch consequence

This ruling gates ONLY 26b. 26a/26a2 proceed unchanged. 26b is dispatchable
once 26a and 26a2 are APPROVED + landed AND the handoff's OPEN ITEM box is
replaced (by main/planner - implementer may not touch the handoff) with a
resolved-ruling pointer to this file.
