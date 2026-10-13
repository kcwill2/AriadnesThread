# Plan - extraction-auto-apply

Milestone key: `extraction-auto-apply` (verbatim). Contract of record:
`tasks/extraction-auto-apply-kickoff.md` and
`tasks/extraction-auto-apply-shape.md` (this file is the digest; on conflict
the two `tasks/` artifacts win).

Authored at HEAD `023558c` on `main` (clean tree; the two contract artifacts
are untracked until this plan's commit). Follows the post-M28 track pattern:
no guessed M-number, live ledger row until closeout.

## Intent restatement

The user's demand, verbatim from the wizard kickoff: "Ideally with an
automatic extractor that creates the new files and moves the symbols." The
wizard made every step of the analyze -> apply spine honest to LOOK at; this
milestone makes the Apply half drivable from the surface where the user
decides, without weakening a single safety property the CLI already
provides.

Four behavior changes:

1. `POST /api/extraction/{plan_id}/apply` behind a single-use,
   epoch-bound, plan-bound approval token minted by the GET dry-run route.
2. The apply reuses the shipped transactional code path - one extracted
   `apply_plan_with` core shared by the CLI verb and the UI applier, over
   `TransactionApplier::apply_with` (snapshot, three tiers, rollback). No
   second transactional implementation; `crates/validate/**` is not edited.
3. Honest outcome reporting: the three tier results, the
   committed-vs-rolled-back verdict, and the verbatim
   `ApplyOutcome::format_output` text, carried as structured data.
4. The help-copy flip from "the UI never applies" to "the UI applies with a
   dry-run first, validation in three tiers and automatic rollback".

Safety frame carried unchanged from the ledger row of record: every
mutation goes through the existing `TransactionApplier` snapshot/rollback
path (never a new write path); an explicit dry-run-then-approve gate on the
current generation epoch before any apply is accepted; one apply at a time
(per-generation mutex, never queued); post-apply rebuild through the
existing fingerprint watcher; the localhost bind default stays
(`resolve_bind_addr`, no new `--expose` widening); every response body is
pure ASCII.

## Hand ordering and dependency graph

```
A-1 (cli: boot-progress family extraction, ui.rs ceiling)
A-2 (cli: apply_plan_with core extraction)
A-3 (ui: applier trait + apply DTOs + POST route + state seam + route tests)
      \
A-4 (cli: EngineBackedExtractionApplier + swap install + provider tests)  <- needs A-1, A-2, A-3
      \
A-5 (ui: S5 approval surface, S6 outcome, help-copy flip, C8 amendments) <- needs A-3, A-4
```

Hard edges (shape section 6):

- A-1 before A-4: the ui.rs extraction is the headroom the install needs;
  the deferred W-2a F2 row forbids spending the last 14 lines.
- A-2 before A-4: the provider delegates to the core.
- A-3 before A-4: the provider compiles against the trait, DTOs and state
  seam.
- A-3 before A-5: the front end implements the route contract.
- A-4 before A-5: the execute step needs a live applier.

A-1, A-2 and A-3 are mutually independent (A-1 and A-2 are file-disjoint
inside crates/cli; A-3 is a different crate). The HOUSE DEFAULT IS STRICTLY
SERIAL on main with one scoped landing commit per hand - the wizard's
"never parallel" precedent, followed by every post-M28 track. A-4 and A-5
are serial by contract. Only A-3 adds a `crates/ui/tests/ui_test` module
line to `main.rs`; A-5 adds none (its locks land in the existing
`extraction_wizard_render.rs`), so no two hands contend for `main.rs`.

De-risking order rationale: the central bet of this milestone is that a
browser-triggered mutation can be made safe by construction (reuse the
transaction verbatim, gate it, serialize it, report it honestly). A-3
therefore ships the contract and its refusal oracles first, and A-4's
provider is the first hand allowed to write a real file.

## Ownership table

| Artifact | Written by | Where |
|---|---|---|
| Handoff contract | planner (this hop) | `tasks/extraction-auto-apply-a{1..5}-handoff.md` |
| Milestone plan | planner (this hop) | `.kilo/plans/1791863217000-extraction-auto-apply.md` |
| Ledger row + sub-block | planner (this hop; the ledger changes in the same commit as the state it records) | `tasks/milestone_ledger.md` |
| Checkpoint journal | implementer, after each Green gate | `tasks/extraction-auto-apply-a{1..5}-journal.md` |
| Reviewer verdict | reviewer hop (review-gate skill) | `tasks/extraction-auto-apply-a{1..5}-verdict.md` |
| Workspace gate evidence (census, ui_smoke, bench stance, theme scan) | reviewer hop | `tasks/extraction-auto-apply-a{1..5}-execution-results.md` |
| Closeout record, walkthrough, memory mirror | milestone-lifecycle closeout | ledger row + `walkthroughs/` |

Implementers never author verdicts or execution-results; reviewers never
edit source. The planner hop owns no source file.

## Sizing table check per hand

| Hand | Crate dir | Gates | Est. produced lines | Bound check |
|---|---|---|---|---|
| A-1 | crates/cli | 8 | 260-330 | 1 crate, 8 <= 10, 330 <= 1200 |
| A-2 | crates/cli | 9 | 140-230 | 1 crate, 9 <= 10, 230 <= 1200 |
| A-3 | crates/ui | 9 | 600-700 | 1 crate, 9 <= 10, 700 <= 1200 |
| A-4 | crates/cli | 9 | 420-560 | 1 crate, 9 <= 10, 560 <= 1200 |
| A-5 | crates/ui | 9 | 250-310 | 1 crate, 9 <= 10, 310 <= 1200 |

Line estimates are implementation-plus-test diff priced FROM the mandated
test bodies (the F14 sizing law: never re-derive a count from a ledger
line). Gates are implementer distilling-entry-point invocations including
the commit-cycle trio (encoding_guard --staged, commit_scoped,
encoding_guard --all).

Planner authoring-time corrections against the architect's shape (recorded
so the next kickoff does not re-open them):

1. A-3's module map omits two compile-required edit sites. The new route
   module needs a `pub(crate) mod api_extraction_apply;` line in
   `crates/ui/src/routes/mod.rs`, and the additive `apply_token` field
   breaks the sample constructor at
   `crates/ui/tests/ui_test/extraction_wizard_api.rs:99-115`, which must
   gain the one-line field default or the whole ui suite fails to compile
   (an invalid Red). Both files join A-3's touch-set; this is shape C8
   amendment 4, which the shape placed in A-5 but which A-3 cannot avoid.
   A-5 therefore carries C8 amendments 1-3 only.
2. A-5 adds no `crates/ui/tests/ui_test` module, so it does not touch
   `main.rs`; the kickoff's "A-3 and A-5 each add a mod line" is superseded
   by the shape's module map. A-5's A7 oracle lands in the existing
   `extraction_wizard_render.rs`.
3. A-1 and A-2 are behavior-preserving refactors, so their Red is a
   source-text lock (the house oracle idiom) rather than a behavioral
   failure: A-1 locks the extraction and the ceiling; A-2 locks the C1 core
   signature and the returned-report contract. Both needles are built with
   `format!` so a test's own source cannot match itself (the
   `progress_contract_tests.rs:73` idiom).

## Census and anchor baseline

Census anchor of record: `1483 passed / 0 failed / 13 ignored / 112 suites`
@ `24c2f7e` (the wizard closeout, NEW ANCHOR). Every hand reports deltas
from a clean-tree SCOPED baseline captured in its own journal - never a
hardcoded workspace total. The workspace census itself is the reviewer
hop's duty, reconciled from the anchor with every delta attributed.

Standing errata inherited by this milestone:

- `python tools/testrun.py --baseline` (the bare FILE form) is unrunnable;
  the workspace census runs via the `--distill` substitute. This is the 10th
  recorded encounter; the decision is still owed.
- The bench gate is deterministically blocked by the `DOCUMENTED_PHASES`
  tuple label at `tools/bench_compare.py:84`, so no timing verdict is
  attributable to this milestone. Per-hand stance: zero-diff producer
  proofs (the producer set under `crates/cli/tests/` and
  `crates/graph/tests/` is byte-identical across every hand range), plus
  the optional compensating `--bench stage5-memory --runs 6` at A-4 (the
  W-2a precedent), declared in the journal if run.

## Gate policy

Workspace-wide gates belong to the REVIEWER hop's acceptance pass, never
to an implementer hand:

- `python tools/testrun.py --distill` (unscoped / workspace census)
- full `python -m unittest discover -s tools -p "*_tests.py"`
- `python tools/ui_smoke.py` (except where a hand edits the tool's own
  route table or an asset the runtime serves, in which case the hand runs
  it through `python tools/distill.py -- python tools/ui_smoke.py`)
- `python tools/bench_compare.py` and the `--bench stage5-memory` form

Implementer hands use SCOPED gates through the distilling entry points:
`python tools/testrun.py --distill -p <crate> [--test <target>]` for
suites and `python tools/distill.py -- cargo check -p <crate> --all-targets`
for reverse-dependency checks. Bare `cargo test` / `cargo build` /
`cargo clippy`, full `unittest discover`, and direct `ui_smoke.py` /
`bench_compare.py` invocations in an implementer session are a reviewer
blocker (the global bash catch-all denies them anyway; the distill route is
the sanctioned path).

Per-hand hygiene, encoding and theme gates are implementer duties:
`python tools/hygiene_check.py` exit 0, `python tools/encoding_guard.py
--staged` before each commit and `--all` after it, and
`python tools/theme_token_check.py --check-added --base <rev>` silent for
any hand touching assets.
