# Milestone Kickoff - extraction-consumer-rewrite

Key: `extraction-consumer-rewrite`. UNREGISTERED post-M28 track per this
kickoff - NO M-number; `tasks/milestone_ledger.md` stays the numbering
authority (precedent: the `extraction-auto-apply` row carries the same
"UNREGISTERED post-M28 track" marker, and before it
`ui-extraction-wizard` and `ui-hotspot-canvas-direction`).
Authored by the architect hop 2026-10-23; the user authorized the milestone
as the follow-on to `extraction-auto-apply` (CLOSED 2026-10-13).

Companion shape: `tasks/extraction-consumer-rewrite-shape.md`. The planner
performs the ledger kickoff flip (see "Ledger flip" below); this artifact
and the shape are committed by a later implementer errand, per the
precedent in the `extraction-auto-apply` closeout record.

## 1. Intent

**Behavior change.** Make an apply's consumer rewrites actually rewrite.
Today `tool apply`, the `tool optimize` safe-plan path, and the UI apply
route all discover consumers correctly (C++ sources that include the plan
target) but then search the consumer text for the plan target's full path
spelling (`#include "include/thing.h"`) while the sources hold the bare
spelling (`#include "thing.h"`). The rewrite matches nothing, the tree
"changes" only by writing new files, and the whole consumer half of apply
is a silent no-op on the current fixture family. Two frozen
characterization tests lock this today, each saying so in its assertion
message: `m20_apply_migrate_characterization_test.rs:179` and
`m19_optimize_characterization_test.rs:110` (the m19 twin is found in the
shaping survey and the ledger headline row does not name it; both are
inside the boundary of record because both quote the same defect).

**The fix seam (one line here, adjudicated in the shape).** The directive
spelling is owned by the language plugin - the `DependencyRewriter` trait
says so and already reserves the exact hook (`consumer_rewrite_spec`).
The fix implements that hook for C++ with the same spelling rule its
consumer discovery already uses. No neutral code, no language-name leak,
no payload change.

**Underlying why.** The product now claims (help copy of record, flipped
by A-5) that "the UI applies with a dry-run first, validation in three
tiers and automatic rollback". That claim is only honest when the apply
does what its dry run promises. The headline deferred row (D6 below) is
the gap between the claim and the behavior, and two shipped sentences
still describe the pre-fix world (`hotspots.js:521-522`, the S4
truncation note). This milestone closes the behavior and restores the
copy in the same stroke so honesty returns when the fix lands, not after.

**Blast radius.** The C++ text-mutation step at apply time only
(`crates/lang-cpp` rewriter hook + its two characterization riders, the
cli-side faithfulness oracle, and two copy/comment edits in `crates/ui`
assets). Discovery sets, preview artifacts, dry-run DTO bytes, the
transaction/gate machinery, and Scala are all byte-untouched (proofs in
the shape's invariants X2-X6).

## 2. Deferred rows inherited verbatim (the boundary of record)

Quoted verbatim from `tasks/milestone_ledger.md` dated rows (2026-10-13).
Line citations inside the rows are as-of-dating; at HEAD the discovery push
sits at `crates/cli/src/commands/transform.rs:338-342` inside
`plan_target_consumers` (UTA-era drift; the rows stand unedited).

Headline row (ledger ~line 320), INGRESSES as this milestone's core:

> extraction-auto-apply D6 latent vacuous-rewrite bug (dated 2026-10-13;
> the headline deferred row; A-4 verdict finding 2 + sec. 7, A-5 verdict
> closeout item 1): an apply's consumer rewrites are a no-op on the basic
> fixture because consumer discovery reports the plan target's spelling
> `include/thing.h` as the rewrite's `old_includes`
> (`crates/cli/src/commands/transform.rs:330-334`) while the sources hold
> the bare `#include "thing.h"`; locked as characterization only by
> `tests/integration/tests/m20_apply_migrate_characterization_test.rs:179`
> `apply_thing_plan_locks_transaction_outcome_and_disk_effects`. Shape
> A6's consumer half is not satisfiable today; A-4 required test 1 asserts
> subset parity plus a non-vacuity guard instead (journal D6). Fix owner:
> `crates/transform` (`IncludeRewriter`) plus the consumer-discovery
> `old_includes` spelling; requires amending the m20 characterization

Follow-up oracle row (ledger ~line 321), rides the fix:

> extraction-auto-apply D6 follow-up oracle (dated 2026-10-13; A-4 verdict
> finding 2, A-5 verdict closeout item 2): expected-modified = the
> approved consumers whose on-disk text actually contains an
> `old_includes` spelling (the dry-run DTO exposes `old_includes`,
> `crates/ui/src/dto/extraction.rs:167`); a positive faithfulness oracle
> that proves the rewrite was faithfully attempted rather than silently
> dropped, and restores equality automatically when the D6 bug is fixed

Stale-hint relay row (ledger ~line 324), ingressed into the copy hand:

> extraction-auto-apply stale apply-hint relay (dated 2026-10-13; A-5
> verdict D5/sec. 3.4 + closeout item 5 - NOW LOAD-BEARING):
> `crates/ui/assets/hotspots.js:521-522` (`renderExtractionApplyHint`)
> still states "the UI states the command, it never runs anything" - as of
> A-5 the help copy says the opposite, so the card hint contradicts
> shipped behavior; the next card-touching hop must flip it to the claim
> of record ("the UI applies with a dry-run first, validation in three
> tiers and automatic rollback")

S4 truncation-note relay row (ledger ~line 325), ingressed into the copy
hand ("with the D6 fix" is this milestone):

> extraction-auto-apply S4 truncation-note relay (dated 2026-10-13; A-5
> verdict closeout item 6): `extraction-wizard.js:386` "the apply rewrites
> every consumer" is aspirational against the D6 latent bug (consumers are
> not rewritten on the current fixture family); the note renders only when
> the rewrite list truncates at 200; report-only, reword when the D6 row
> is worked

Also arriving at this milestone's touch sites (their rows say "next touch"
of exactly the files this milestone touches; the planner may ingest or
leave them):

- A9 path carve-out (ledger ~line 322): `outcome_dto` leaves `plan_id`
  unsanitized; the one-line `ascii_only(&plan_id)` rides the oracle hand
  (shape C5). The `new_files` half stays frozen by the parity rule and is
  OUT.
- Cosmetic (ledger ~line 323): the `ascii_only` doc-comment indentation in
  `extraction_apply.rs` (:243-248 at the dating; re-verify at touch), same
  hand.
- A-4 non-numbered observations (ledger ~line 327): the deleted-files
  absence check belongs to the same test 1 the oracle hand amends.
- The wizard-era fetch-site invariant (ledger ~line 326) is a BINDING lock
  on the copy hand: fetch handling stays the GET fetcher plus the one
  sanctioned apply POST site (`extraction_wizard_render.rs:349-352`,
  bound `<= 2`); never restore `count == 1`.

The `extraction-auto-apply` closeout record's statement that its deferred
scope of record is UNCHANGED stays true: this milestone ingests rows
recorded inside that closed milestone, and touches none of its boundary
text.

## 3. Scope

**In:**

- The C++ plugin implements `DependencyRewriter::consumer_rewrite_spec`
  with the file-name directive rule its `consumer_discovery_targets`
  already uses (shape C1) - this is the adjudicated reading of the
  headline row's "consumer-discovery `old_includes` spelling": the row is
  the boundary of record; the shape names the exact seam and why the
  alternatives were rejected.
- The two characterization amendments it forces: m20
  `apply_thing_plan_locks_transaction_outcome_and_disk_effects` and m19
  `optimize_writes_all_extracted_headers_and_leaves_consumer_sources_unchanged`
  (shape C2, C3), under the amendment policy in section 6.
- The positive faithfulness oracle in the cli apply proof (shape C4),
  which turns the A-4 subset-parity clause into the deferred row's
  expected-modified equality on the fixture.
- The two copy relays (shape C6) plus the arriving one-liners (shape C5).
- A dated lock that no shipped asset string still claims "never runs
  anything" (shape C6 acceptance).

**Out (each with its reason):**

- Symbol MOVE (removing moved declarations from sources through the
  transaction). Registered follow-on, section 5. The verdict and its
  evidence are the shape's adjudicated decision; the short form: the move
  machinery is shipped and honestly reported for move-capable plugins,
  the C++ endgame work is a plugin-parser workstream plus a product
  decision the user owns, and D6 is its hard prerequisite anyway.
- Anything that makes the rewritten consumers COMPILE (include-closure:
  where the generated headers live vs the project's include dirs). This
  has never been promised - kickoff D-M23-12 deferred real compile
  invocation and named extraction include-closure its own workstream;
  tier 3 is an existence check plus an honest toolchain skip. Registered
  as a follow-on candidate, section 5.
- Changing the dry-run DTO to display the directive-level spelling (the
  DTO rows are path data by contract; the wizard relays keep the copy
  honest). Display-spelling surfacing is a registered follow-on candidate.
- Scala behavior changes of any kind (its rewrite spec and move hooks are
  implemented and green; its suites are frozen-unedited locks here).
- Side-effect-preservation semantics (`compute_rewrite`'s empty-symbols
  path): unchanged by the fix and separately pinned; touching it here
  would widen the characterization amendment set for no product gain.
- The `tool optimize` path's own payload quirks (its
  `generated/`-prefixed `new_includes` spelling, tier-3 p2 rescue): it
  benefits from the fix through the shared discovery, but its payload
  shapes are m18-parity-locked and unchanged.
- `tools/**` (operator-law territory), `tests/golden/**`,
  `tests/fixtures/**`, theme tokens, the bind surface.

## 4. Inherited locks (provenance, not re-derived here)

- Wizard shape section 4 (the v1 boundary) and the safety invariants that
  bound both halves: every mutation through the existing
  `TransactionApplier` snapshot/rollback path, never a new write path;
  dry-run-then-approve on the current generation; localhost bind default;
  every generated byte pure ASCII through the plugin's extractor.
- `extraction-auto-apply` shape invariants A1 (single write path, caller
  census 3), A2/A3/A4 (token gate, epoch binding, one apply at a time),
  A6 (dry-run/apply parity), A9 (ASCII), and the A-4 required-test
  non-negotiables the oracle hand preserves or strengthens byte-clausibly.
- Characterization suites inherited as frozen UNLESS named for amendment:
  m10/m14/m19/m20/m23 consumer-set equality (I1), the m18 parity suite
  (preview JSON and diff bytes), the m17 plugin-seam transitive-cone
  guard, the `#[cfg(test)]` trait-default pins in `crates/lang`.
- The wizard-era fetch-site bound `<= 2` (dated row ~326), binding on the
  copy hand.
- Census anchor of record: `1830 passed / 0 failed / 13 ignored / 119
  suites @ 765df7b` (unit-test-audit ROUND 2, CLOSED 2026-10-19). Live
  tree at this authoring (git MCP, fresh): branch `main` at `a38c7aaa`
  (the round-2 closeout commit, child of the anchor), dirty only the
  pre-existing `extensions/ariadne-extension/ariadne-extension-0.1.0.vsix`
  stray. The architect role has no bash, so
  `python tools/checkpoint_guard.py status` was NOT run here; the planner
  or first hand re-runs it as its bootstrap duty.
- Unit-test-audit closeout intake (dated row ~328): this milestone touches
  the lang front-ends, so the handoff author must read the "Deferred
  findings" section of `docs/specs/unit-test-audit.md` first and carry
  anything that names the touched seams.

## 5. Registered follow-on key

**`extraction-symbol-move`** (registered here the way `ui-extraction-wizard`
registered `extraction-auto-apply`; proposed key, no M-number until a
kickoff takes it). The endgame the wizard named - "the automatic extractor
creates the new files AND moves the symbols" - is half-shipped: the
transaction strip, the dry-run `move_removals` count, and the
strips-as-it-writes copy all exist and work for move-capable plugins
(Scala), and copy-only is the honest C++ answer today with the copy that
says exactly that. What the endgame still lacks is C++-side: a real
`CppDependencyRewriter::remove_top_level_symbols` (parser-span removal;
the M25 brace-depth class of hazards lives there), plus the product
decision the user owns at that kickoff (move as default vs opt-in, given
I5 pins copy-only as C++ semantics and several shipped sentences and
characterization clauses say so), plus the amendment table that flips
those locks deliberately. Doing the D6 rewrite fix first is required for
that milestone to make sense on C++ (a move that leaves consumers
pointing at the old monolith is incoherent), which is the sequencing
argument the shape's decision section makes in full.

Follow-on candidates to date at this milestone's closeout (not yet keys):

- Include-closure / compilability of rewritten consumers (successor to
  D-M23-12's deferral): making the new-include spelling resolve under a
  project's real include dirs.
- Surfacing the directive-level spelling in the dry-run rewrite rows (the
  DTO stays path data today; the wizard shows paths by contract).
- Operator tails that persist across all of this: the PR-2 per-machine
  bench baseline refresh.

## 6. Frozen-lock amendment policy (this milestone's sharpest edge)

The m19/m20 characterization amendments amend frozen locks. Who may amend,
and under what proof:

- ONLY the CW-1 implementer hand, only the two named tests, only under an
  explicit amendment clause in its handoff, and only after capturing the
  Red: the amended assertions must first FAIL on the pre-fix tree for the
  documented reason (or, post-fix, pass with the pre-fix assertion
  demonstrably false - the handoff picks the ordering and journals it).
- The proof that rides the handoff: every other assertion in both files
  byte-identical (consumer-set counts, preview JSON contents, generated
  file lists, report fields, the whole migrate test); the reviewer hop
  diffs per commit against the start rev and rejects any hunk outside the
  two tests; the I1/m17/m18 suites green UNEDITED; and the faithfulness
  oracle (CW-2) green so the new truth is machine-witnessed, not asserted.
- No other hand touches m19/m20; no other pre-existing test is amended
  except the declared CW-2 test-1 upgrade (same hand, same reviewer
  diff) and the CW-3 wizard-note copy string, which nothing locks today
  (grep-proven at authoring) and which CW-3 then locks.

## 7. Milestone acceptance gate (whole-milestone)

Every item machine-checkable at the closeout hop, reconciled from the
anchor `1830 passed / 0 failed / 13 ignored / 119 suites @ 765df7b` with
every delta attributed (the `--distill` census substitute governs; the
`--baseline` FILE form stays superseded):

1. An apply on the basic fixture rewrites `tests/consumer.cpp` and
   `src/thing.cpp` (bare old include gone, the four bare extracted
   includes in place, declarations intact) - m20 and m19 green with the
   byte-identical remainder per section 6.
2. The faithfulness oracle is green end-to-end through the UI applier:
   modified set == expected-modified (spelling-matching set) == the
   approved dry-run's consumer set, on the fixture.
3. No shipped string in the wizard/card assets claims the UI never runs
   anything; the S4 truncation note states the shipped truth; a lock
   test pins the absence so it cannot rot back.
4. Consumer-set equality (I1), preview bytes (m18 parity + m20 migrate),
   plugin neutrality (m17 guard), the Scala suites, and the apply
   gate/mutex/token proofs are green UNEDITED.
5. Workspace gates exit 0 at every hand and at the close: tools discover,
   `ui_smoke: PASS`, hygiene R1/R2, `encoding_guard --all`,
   `theme_token_check --check-added --base` silent. Bench: the timing
   gate is unblocked (the `f20ad35` label fix landed), so hands record a
   fresh `bench_compare` run through the distiller; machine-class WARNs
   are recorded, not attributed (PR-2 persists).
6. ASCII throughout; census deltas attributed; the milestone's dated rows
   written at closeout discharging the four inherited rows quoted above.

## 8. Ledger flip (the planner's duty, not this hop's)

The planner hop touches exactly one file for the kickoff flip:
`tasks/milestone_ledger.md` - add the `extraction-consumer-rewrite`
registration row (UNREGISTERED post-M28 track, NO M-number; kickoff and
shape paths; the four inherited deferred rows named as INGRESSED with a
one-line pointer each; census anchor; the `extraction-symbol-move`
registration note). The deferred rows themselves stay verbatim-unchanged
until the closeout discharges them (append pointers, never rewrite).
This kickoff, the shape, and the plan digest are committed by a later
implementer errand, per the `extraction-auto-apply` precedent; this hop
commits nothing.
