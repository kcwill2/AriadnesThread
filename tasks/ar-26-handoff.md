> SUPERSEDED 2026-10-11: the mandatory SPLIT CHECK fired (journal `tasks/ar-26-handoff-journal.md`: projected move 3101+ lines vs the 500 threshold; commits `e946f57`/`a634ae5`/`2679ebb`). Re-planned by the planner as a 3-hop serial chain: `tasks/ar-26a-handoff.md` (new crate skeleton + `manifests.rs` move + shim) -> `tasks/ar-26a2-handoff.md` (atomic `manager/*` move + report shim; E0116/E0583 forbid finer seams) -> `tasks/ar-26b-handoff.md` (scheduler reparent + m17 cone guard; cone-wording ruling LANDED - `tasks/ar-26b-shape-ruling.md`, Option A binding: machine-locks EXACT `ariadne-report` absence from scheduler's cone + tree; cluster/transform/validate are expected cone members). The "cone loses cluster/transform/validate" impact line below is machine-unsatisfiable under VERBATIM - Rulings 2/3 of `tasks/ar-26b-shape-ruling.md` record A3 as PARTIALLY satisfied with a deferred-ledger-row residual (AR-26c charter). Retained as the original wave-6 contract - do not dispatch.

# AR-26 Handoff - Artifact-store extraction to new `crates/artifact` (A3 MED)

Wave 6. NEW crate + `report` + `scheduler` reparent. Dispatch after AR-20
APPROVED (cone guard compounding) and after AR-25 lands (`report/manager.rs`
serialization). Plan size-check enforced below — split if triggered.

## Objective

New `crates/artifact` crate owning the `manager/*` IO: transform_io,
apply_io, metrics_io, graph/index/discovery io, manifest (and the AR-03
`bincode_options.rs` helper — moves WITH the IO it guards; journal).
`report` keeps presentation/report DTOs and depends DOWN on `artifact`;
`scheduler` depends on `artifact` directly, DROPPING `scheduler -> report`.

## Impact

scheduler's dependency cone loses the cluster/transform/validate stack
dragged in via a "reporting" crate; honest names; compounds AR-20's plugin
cone work. VERBATIM moves + declared paths — no behavior change.

## SPLIT CHECK (hard, before moving anything)

Run `git ls-files crates/report/src/manager | wc -l` + planned re-export
estimate; if projected moved lines > 500, SPLIT into two handoffs and report
back BEFORE implementing: AR-26a (extract artifact crate; report re-exports
manager paths) / AR-26b (scheduler reparent + drop scheduler->report + cone
guard). Journal the numstat evidence either way.

## Touch-set (hard)

- `crates/artifact/**` (new crate: lib + moved manager modules + Cargo.toml)
- `crates/report/**` (delete moved modules, add re-export shims keeping
  `ariadne_report::manager::*` paths compiling — consumers UNTOUCHED)
- `crates/scheduler/**` (dep switch to ariadne-artifact; import paths via
  report shim or direct — journal choice)
- workspace `Cargo.toml`
- cone guard test (AR-20 guard test extended: scheduler no longer pulls
  report/cluster/transform/validate)
- `tasks/ar-26-handoff-journal.md`

## Constraints

- Modules moved VERBATIM (AR-03's bounded-decode swaps included; their
  regression locks run UNEDITED via re-export path).
- bincode wire bytes byte-identical (m00 + m23 artifact locks UNEDITED —
  authority).
- TDD: cone guard RED (scheduler still pulls report) -> GREEN after.
- STRICT COMPILE-GREEN: workspace check before any suite.
- Pure ASCII; commit_scoped artifact + report + scheduler + workspace +
  integration + journal (single cross-crate commit allowed, AR-18 precedent).

## Gate commands

1. `-p ariadne-artifact` (new — moved module suites count here: reconcile
   against report's 47 dropping the moved suites)
2. `-p ariadne-report` (47 minus moved +shim locks — exact reconciliation in
   journal)
3. `-p ariadne-scheduler` (34/0 exact — zero scheduler test edits)
4. m00 + m23 bincode locks UNEDITED
5. cargo tree guard (cone assert) green
6. `encoding_guard --all`; `hygiene_check.py` (zero-blocking baseline)

## Invariants / locks

- `cargo tree -p ariadne-scheduler` contains NO ariadne-report/cluster/
  transform/validate (machine-locked).
- Old `report::manager` paths still resolve (shim) — zero consumer edits
  outside touch-set.
