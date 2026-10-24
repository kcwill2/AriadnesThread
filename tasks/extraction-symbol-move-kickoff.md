# Milestone Kickoff - extraction-symbol-move

Key: `extraction-symbol-move` (already registered in
`tasks/milestone_ledger.md`). UNREGISTERED post-M28 track, NO M-number;
`tasks/milestone_ledger.md` stays the numbering authority (precedent: the
CLOSED `extraction-consumer-rewrite` row at ledger table line 59 and its
closeout sub-block at lines 246-361, and before it
`extraction-auto-apply` and `ui-extraction-wizard`). Authored by the
architect hop 2026-10-24; the kickoff/shape pair is landed by an
implementer errand because the architect role has no bash (the
`extraction-auto-apply` / `extraction-consumer-rewrite` precedent).

Companion shape: `tasks/extraction-symbol-move-shape.md`.

## 1. Decision of record (the user's, verbatim)

> "Move by default but easily reversable when something goes wrong.
> perhaps by commenting out the symbols with clear markings that indicate
> the extraction."

This milestone exists to implement that decision. The first clause
("Move by default") and the goal ("easily reversable when something goes
wrong") are the mandate. "perhaps by commenting out the symbols with
clear markings" is the user's SUGGESTED mechanism ("perhaps by"), not a
mandate: this kickoff adjudicates the mechanism and adopts it, with the
hazards below specified away (line-prefix grammar, never block comments;
refusal contract; bounded cleanup story).

**Adjudicated mechanism: (a) comment-out-and-mark retention in the
original file, held behind a plugin-owned contract so a language may
still choose true deletion (Scala does, untouched).** The full
adjudication, including the rejected deletion-only and undo-journal
alternatives, is shape section 4. The two-sentence rationale: the shipped
transaction rollback cannot reach the user's worry, because it fires only
inside the transaction and tier 3 never compiles anything (D-M23-12
deferral; include-closure is a registered follow-on), so a committed move
can break a build with no restore path once the in-memory snapshot is
dropped at commit; marked in-file retention is the only mechanism that
makes post-commit reversal something the user can do by eye in any
editor without adding a new write path, a new command surface, or a
second source of truth.

**Reversibility semantics of record (precise, three layers):**

1. *In-transaction rollback (already shipped, unchanged, re-locked):* a
   tier failure or an internal error restores every touched file -
   including the original file about to be marked, which
   `TransactionApplier::apply_with` already snapshots via the
   move-removal paths (`crates/validate/src/lib.rs:173-176`) - byte for
   byte (`verify_restored` machinery).
2. *Post-commit declaration reversibility (this milestone's mechanism):*
   the original file keeps every moved declaration's bytes verbatim
   behind a `//` prefix inside a machine-recognizable
   `ARIADNE-EXTRACT-OUT` region bound to the plan id (grammar in shape
   C2); restoring the declarations is a mechanical uncomment, doable by
   a human in any editor and by a future tool from the grammar alone.
3. *Post-commit full reversibility: explicitly NOT promised.* Uncomment
   does not revert the consumers' include rewrites (those are CW-1's
   real rewrites and stay); a complete restore is uncomment + revert the
   consumer include flips, or git. The shipped copy must never claim a
   one-click undo, and the copy truth contract (shape C6) says the
   restore procedure honestly.

**Cleanup story (explicit, since marked text is retained):** marked
regions NEVER auto-expire. This milestone ships no purge command; the
grammar is designed so a registered follow-on command can delete regions
by marker match (registered in section 6, scope-out reason there).
"Reversable" is the mandate; auto-expiry was rejected because silent
deletion of the user's declarations would be the exact surprise the
decision guards against.

## 2. Intent

**Behavior change.** Extraction apply moves the plan's symbols out of
the ORIGINAL file by default: today `DependencyRewriter::
remove_top_level_symbols` (`crates/lang/src/rewriter.rs:94-101`) is the
trait default `None` for C++ (I5 copy-only), so the shipped move
machinery (`crates/validate/src/lib.rs` snapshot + strip phases, the
dry-run `move_removals` / `copy_only` computed reporting, the
strips-as-it-writes copy sentence) is honest for Scala and honestly
reports copy-only for C++. This milestone gives C++ a real
implementation: the apply comments the moved regions out in place under
marked delimiters, the dry run reports it before approval, and the
wizard copy tells the move truth.

**Composition, not disturbance.** Consumer includes are already truly
rewritten (CW-1 `c916a26`): the strip phase runs after the consumer
rewrite phase inside the same transaction
(`crates/validate/src/lib.rs:246-273`), touches different files on the
fixture, and does not touch `consumer_rewrite_spec`,
`plan_target_consumers`, or `IncludeRewriter` at all. The CW-2
faithfulness-oracle equality grows new truth conditions - the original
file becomes a modified file - enumerated in shape C5; the consumer half
of the equality is byte-identical to the CW-1/CW-2 claims.

**Underlying why.** The wizard's endgame line ("the automatic extractor
creates the new files AND moves the symbols") is half-shipped; the user
made the product decision the consumer-rewrite kickoff handed forward
("its kickoff owns the user-owned copy-vs-move default decision", ledger
row 446). Move-by-default with marked, human-reversible retention is that
decision, executed.

**Blast radius.** `crates/lang` (one neutral default-method hook + doc
lift), `crates/lang-cpp` (the C++ retention implementation + new module),
`crates/validate` (thread the plan id into the strip call - the marker
binds it), `crates/cli` (reader dry-run honesty + the apply-side oracle
growth), `crates/ui` assets + one test file (copy truth), and the named
integration amendments (m19/m20 second amendment, rider). Scala, goldens,
fixtures, themes, tools, the gate/mutex/token machinery, and the boot
payload are byte-untouched (proofs in the shape invariants).

## 3. Read of record (cited, not restated)

- `tasks/milestone_ledger.md`: the CLOSED `extraction-consumer-rewrite`
  row (table line 59) and closeout sub-block (lines 246-361); the
  deferred rows at table rows 438-446 - row 445 carries the GENERATION-
  SWAP HALF STILL OPEN (the prior artifacts' "ledger:328" pointer runs
  ~one row low; the closeout relay correction at lines 323-332 says to
  cite the actual rows), row 446 is the follow-on + open-tails intake
  this kickoff inherits (include-closure/compilability, directive-
  spelling display, "its kickoff owns the user-owned copy-vs-move default
  decision").
- `tasks/extraction-consumer-rewrite-kickoff.md` (section 5 registers
  this key) and `-shape.md` (section 0 seam freezes, section 4 deferral
  evidence, section 6 amendment policy precedent - the freeze lift/persist
  adjudication is shape section 0 here).
- `tasks/extraction-auto-apply-kickoff.md` + `-shape.md`: the boundary of
  record (wizard shape section 4 safety invariants; every mutation via
  `TransactionApplier`; localhost bind; ASCII-generated-bytes invariant;
  A1-A10).
- Code read at authoring (all measured, not relayed):
  `crates/lang-cpp/src/dependency_rewriter.rs` (390 lines, CW-1 hook),
  `crates/cli/src/commands/transform.rs` (581 lines;
  `plan_target_consumers` at :314),
  `crates/cli/src/commands/ui/extraction_apply.rs` (295 lines - the true
  size, the "999/1000" era ended with the UTA split),
  `extraction_apply/tests.rs` (903 lines - a hygiene ceiling watch this
  milestone must plan around), `extraction_reader.rs` (899 lines),
  `crates/validate/src/lib.rs` (847 lines; strip machinery :142-149,
  :173-176, :246-273, :334-395), `crates/lang/src/rewriter.rs` (221),
  `crates/ui/assets/extraction-wizard.js` (copy at :215-219, :341-399,
  :426-433), `crates/ui/src/dto/extraction.rs` (228), the m20
  characterization in its post-CW-1 state, and the CW-1 verdict
  (`tasks/extraction-consumer-rewrite-cw1-verdict.md`) as the
  amendment-policy precedent.

## 4. Scope

**In:**

- The neutral trait-level move contract: a plan-id-aware default-method
  hook on `DependencyRewriter` whose result IS the post-move original
  text, retention form chosen by the language; default delegates to the
  existing `remove_top_level_symbols` so Scala and every existing fake
  keep their behavior byte-identically (shape C1).
- The C++ implementation: verbatim-span marking with the marker grammar
  of shape C2 (pure ASCII, plan-id bound, line-prefix regions, refusal
  contract for unsafe boundaries), reusing the existing verbatim locator
  without touching it.
- The plan-id threading: the strip phase passes `migration.plan_id` to
  the hook (`crates/validate/src/lib.rs`, one call site; the marker
  binding requires it and the data is already in scope).
- Dry-run honesty: `extraction_reader.rs::move_removal_count` offers the
  SAME hook with the SAME plan id (dry-run/apply parity of the move
  claim), and the reader's W4 fixture test (required test 5,
  `test_dry_run_reports_copy_only_honesty_on_the_cpp_fixture`) is
  amended from copy-only truth to move truth.
- The faithfulness-oracle growth in `extraction_apply/tests.rs` required
  test 1: the new truth conditions of shape C5 (original file marked,
  equality grown, written set and no-deletions claims surviving).
- Copy truth re-audit, since apply now touches the ORIGINAL file: the
  named lines that change and the lines that stay are shape C6/C7
  (closed table; every change rides the named hand).
- The m19/m20 second amendment (move truth on the original file), rider
  on the validate hand, under the closed policy in section 7.

**Out (each with its reason):**

- Compilability / include-closure of rewritten or moved files: stays the
  registered follow-on (ledger row 446 item ii; D-M23-12 deferral
  unchanged). No tier compiles, and this milestone does not change that.
- Directive-spelling display in dry-run rows: registered follow-on
  (row 446 item ii); the DTO rows stay honest path data.
- The generation-swap half of the A-4 observations row (row 445): the
  touch trigger ("next `extraction_apply.rs` test touch") ARRIVES at
  this milestone and is DELIBERATELY NOT EXECUTED - the oracle growth is
  the milestone's own claim; the generation-swap test stays registered
  and the closeout appends a dated relay that the trigger re-fired.
- The purge command for marked regions: registered follow-on (shape C2
  cleanup story); shipping it here doubles the ui/cli surface for no
  reversibility gain the marker does not already give.
- Any analyzer treatment of marked regions beyond the tokenizer's
  natural comment invisibility (e.g., marker-aware exclusion of the C++
  raw substring consumer scan): registered follow-on candidate; the
  hazard analysis is shape R1.
- Scala changes of ANY kind: the default delegation exists precisely so
  zero Scala files appear in any diff (plugin-extensibility intent: the
  mechanism lives behind the trait, C++ first, Scala later untouched).
- Move for plans with zero consumers: `collect_move_removals` stays
  rewrite-driven (a move triggers only when a consumer rewrite names the
  original); broadening the derivation is a pipeline change with its own
  amendment surface, out with a reason (invariant SM-I4 locks the
  derivation byte-identical).
- One-click undo, persisted undo journals, restore commands: rejected in
  shape section 4 with reasons; nothing here persists the snapshot.
- `tools/**`, `tests/golden/**`, `tests/fixtures/**`, theme files,
  `tools/theme_token_baseline.json`, the bind surface.

## 5. Inherited locks (provenance; proofs in the shape's gate table)

- Wizard shape section 4 + `extraction-auto-apply` A1-A10: single write
  path (`apply_plan_with`, caller census 3 - unchanged by this milestone,
  which adds no caller), token gate, epoch binding, one apply at a time,
  honest outcome, dry-run/apply parity, localhost bind (`resolve_bind_addr`
  in no diff), ASCII (A9) - every marker byte and DTO string pure ASCII.
- CW locks: `consumer_rewrite_spec` C++ hook byte-identical (its fidelity
  test green UNEDITED), `IncludeRewriter` and `plan_target_consumers`
  frozen (shape section 0 adjudicates their lifted/persisted roles),
  m18 parity preview bytes path-space display UNEDITED, m10/m14/m17/m23
  and every Scala suite UNEDITED, m00 parity, the fetch-site bound
  `<= 2`, the CW-3 no-rot locks (`never runs anything` absent; the
  truncation note positive pin) - both stay TRUE under move truth and
  survive UNEDITED.
- Boot payload byte lock (`CAPTURED_PROPOSALS_JSON`): boots the UNAPPLIED
  fixture; survives UNEDITED.
- Hygiene R1/R2, `encoding_guard`, `theme_token_check --check-added
  --base` silent, census anchor `1837 passed / 0 failed / 13 ignored /
  119 suites @ b201aa6` (the `--distill` census substitute governs; the
  `--baseline` FILE form stays superseded - M23.5 F9).
- Bench stance: the timing gate is unblocked (`f20ad35`); every hand
  records a fresh `bench_compare` through the distiller, records WARNs,
  attributes nothing (PR-2 persists).
- Unit-test-audit intake: this milestone touches the lang front-ends, so
  the handoff author reads the "Deferred findings" section of
  `docs/specs/unit-test-audit.md` first and carries anything naming the
  touched seams (the consumer-rewrite kickoff's rule, inherited).

## 6. Follow-on registrations (made at this kickoff's closeout)

- `extraction-marked-region-purge`: a command that deletes
  `ARIADNE-EXTRACT-OUT` regions by marker match (the cleanup story's
  second command; grammar designed for it from day one).
- Include-closure / compilability (inherited, unchanged).
- Directive-spelling display in dry-run rows (inherited, unchanged).
- Marker-aware exclusion for the C++ raw substring consumer scan
  (shape R1 hazard analysis).
- The generation-swap half of ledger row 445 (relay re-fired here,
  unexecuted by choice, section 4).

## 7. Frozen-lock amendment policy (closed policy, CW-1 sec. 6 precedent)

Only these pre-existing locks move, only on these named hands, each under
its handoff's explicit amendment clause with a captured Red first, each
proven by the reviewer's per-commit diff against the hand's start rev
(everything else in the same file byte-identical):

1. `crates/lang/src/rewriter.rs` doc comment naming C++ copy-only (I5
   wording) - the lang hand, DOC line only; every trait-default PIN TEST
   stays byte-identical UNEDITED (the defaults' behavior does not change).
2. m19 `optimize_writes_all_extracted_headers_and_rewrites_consumer_
   includes` and m20
   `apply_thing_plan_locks_transaction_outcome_and_disk_effects`: SECOND
   named amendment, additive original-file proofs + the stale "not a
   move" comment wording; the validate hand carries the rider; every
   consumer-side assertion from CW-1's amendment survives byte-identical.
3. `extraction_reader.rs` required test 5 (W4 fixture copy-only honesty):
   flipped to move truth by the cli hand; tests 1-4 and 6-10 UNEDITED.
4. `extraction_wizard_render.rs`
   `test_copy_vs_move_sentence_is_exact_and_keyed_off_the_computed_path`:
   the S5 unconditional-sentence assertion flips to the keyed branch
   (matching S4's existing pattern); the sentence constant itself stays
   byte-identical (it remains true on the copy-only branch, which still
   exists for un-locatable symbols); the ui hand carries it.

Any other pre-existing test discovered mid-hand wanting an edit is
STOP-and-report (the CW-3 discipline precedent, ratified at the last
closeout).

## 8. Milestone acceptance gate (whole-milestone)

Machine-checkable at the closeout hop, reconciled from the anchor
`1837 passed / 0 failed / 13 ignored / 119 suites @ b201aa6` with every
delta attributed (`--distill` substitute governs):

1. A fixture apply marks `include/thing.h` per the grammar (shape C2):
   one BEGIN/END pair per moved unit bound to the plan id, region bytes
   verbatim-under-prefix, un-marked bytes of the file untouched; m19/m20
   green with the byte-identical remainder per section 7.
2. The grown faithfulness oracle is green through the UI applier: the
   shape C5 truth conditions (equality, written set, no deletions,
   marker proof, dry-run move-count parity) all hold on the fixture.
3. Copy truth: no shipped string claims "strips" for the C++ fixture
   plan or claims the UI never mentions the marked region; the amended
   W4 lock green; the two CW-3 no-rot locks and the help copy claim of
   record ("the UI applies with a dry-run first, validation in three
   tiers and automatic rollback") green UNEDITED.
4. Consumer-side zero regression: `consumer_rewrite_spec` hook file,
   `IncludeRewriter`, `plan_target_consumers`, the m18 parity bytes,
   m10/m14/m17/m23, every Scala file, the m00 parity, and the boot
   payload byte lock all UNEDITED and green; Scala reaches identical
   deletion behavior through the default delegation (byte-identity pin).
5. Rollback still covers the original file: the validate move-transaction
   proof green UNEDITED (its fake reaches the strip through the default
   delegation - that IS the proof).
6. Workspace gates exit 0 at every hand and at the close: scoped suites,
   tools discover, `ui_smoke: PASS`, hygiene R1/R2, `encoding_guard
   --all`, `theme_token_check --check-added --base` silent, fresh bench
   recorded per hand; commit-vs-touch-set exact per hand.
7. ASCII throughout (marker grammar is pure ASCII by construction);
   closeout writes the dated rows discharging this milestone's
   contributions and re-registering the section 6 follow-ons.

## 9. Ledger flip (the planner's duty, not this hop's)

The planner flips `tasks/milestone_ledger.md` once: the
`extraction-symbol-move` registration row (already present as intake) to
PLANNED with kickoff/shape/plan paths, the decision-of-record quote, the
census anchor, and pointers (append-only) to rows 445/446 relays. The
inherited deferred rows stay verbatim until closeout discharges them.
