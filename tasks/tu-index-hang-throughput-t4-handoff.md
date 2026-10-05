# T4 Handoff - M27 tu-index-hang-throughput: conditional DP-4 scope (BLOCKED STUB - no spec)

*** STATUS: DISPATCH-BLOCKED, UNAUTHORED ON PURPOSE *** This file is an
evidence-door placeholder, NOT an implementer contract. It contains no
touch-set and no gates because authoring either would pre-judge the DP-4
evidence review. Do not "complete" this handoff from chat memory: per the
kickoff (sec. 4) and shape sec. 3.5/4, the conditional scope "stays OPEN,
evidence-gated" and is "authored ONLY on T0/T1 evidence".

Milestone: `tu-index-hang-throughput` (M27, ledger row 2026-10-05).

## The evidence door (what must exist before any T4 spec is written)

A written T1 evidence review (appended to
`tasks/tu-index-hang-throughput-t1-measurement.md` by the orchestrator)
ranking the conditional candidates by MEASURED phase cost, from the
RELEASE-profile boot table:

- H1' shard serde cost (`tu_store` save path) - only if `tu-index cold`
  medians show serde-dominated time; envelope bytes stay FROZEN by I7, so
  any speed work must be format-preserving (or a fresh user DP).
- H3' per-TU `fs::canonicalize` hoist in `fingerprint_scoped_context` - only
  if the fingerprint/build-discovery rows are hot; fingerprint bytes
  asserted identical (oracle: fingerprints byte-equal pre/post).
- DP-4a `crates/repo` hash parallelism (order-preserving `par_iter`) - only
  if `discover+hash total` dominates the table; order-preservation oracle:
  the T0b-style hook-count + result identity locks (M26 V2 tests) plus
  record-order byte-equality.
- DP-4b `crates/lang-cpp` per-file parse work - only if `tu-index cold`
  remains dominant AFTER T2 and (if dispatched) T3; must not weaken the T2
  locks (both-profile boundedness tests stay green UNEDITED).

If the review ranks a candidate as material (i.e., it, not parallelism,
owns the ratified wall-time gap), the PLANNER authors a fresh handoff
`tasks/tu-index-hang-throughput-t4a-handoff.md` (or -t4b/-t4c) sized to the
law: <= 1 crate (repo, lang-cpp, or scheduler - chosen by the candidate),
<= ~10 distilling gates, <= ~1200 diff lines, workspace-wide gates reserved
to the reviewer hop. Each candidate is its own handoff; two candidates never
share one (disjoint crates, independent evidence, independent revert).

## Standing locks any future T4 inherits (from the M27 ledger row)

- All M26 locks (DP-1 floor, append-only boot tests, goldens/fixtures,
  m00/m18 parity UNEDITED).
- Shard envelope FIELDS + `CURRENT_ANALYSIS_VERSION` (already bumped to
  2.1.0 by T2; frozen again after that commit).
- M27 T2 locks: both-profile tokenizer bounds green UNEDITED.
- M26-appended tests green under whatever arrival pattern ships (I13 shape).
- Distilling entry points for every gate (ledger row 58); PR-2 bench
  precondition for any bench-gated landing.

## What this file changes when

The planner rewrites this stub into the dispatch-ready handoff ONLY after
the written evidence review exists; until then the correct action for every
role is: do nothing here. (If the evidence review dismisses ALL candidates,
record that sentence in the evidence doc and this stub retires at M27
closeout with the ledger note `DP-4 dismissed on T1 evidence - no T4`.)
