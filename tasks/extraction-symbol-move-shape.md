# Milestone Shape - extraction-symbol-move

Companion to `tasks/extraction-symbol-move-kickoff.md` (decision of record,
scope, amendment policy and the whole-milestone gate live there). Read-only
survey at HEAD on `main` (git state per the CLOSED
`extraction-consumer-rewrite` closeout: census anchor `1837 passed / 0
failed / 13 ignored / 119 suites @ b201aa6`, tree per the closeout record);
the architect role has no bash, so `checkpoint_guard` was not run here - the
first hand bootstraps from it as its duty. Pure ASCII throughout; every line
count and line cite below was measured on disk in the shaping session:
`lang/src/rewriter.rs` 221, `lang-cpp/src/dependency_rewriter.rs` 390,
`lang-cpp/src/symbol_locator.rs` (verbatim locator, untouched),
`validate/src/lib.rs` 847, `cli/src/commands/transform.rs` 581
(`plan_target_consumers` :314), `cli/src/commands/ui/extraction_apply.rs`
295 (true size; the "999/1000" relay is dead per the CW closeout relay
correction), `extraction_apply/tests.rs` 903,
`cli/src/commands/ui/extraction_reader.rs` 899,
`ui/src/dto/extraction.rs` 228, `extraction-wizard.js` (copy sites
:215-219, :341-399, :426-433, :571-575), `m20` 289 lines in its post-CW-1
state, `extraction_wizard_render.rs` (~910; CW-3 era ended ~874 + locks).

## 0. The seam state, and which freezes lift

The move machinery is fully shipped EXCEPT the C++ decision. Today, in order:

- Consumer discovery -> rewrite payload -> rewrite: CW-1/CW-2 territory,
  done, truthful, frozen (see the persist list below).
- Move derivation: `collect_move_removals` (`validate/src/lib.rs:345-370`)
  already derives (original-file, unit-names) candidates rewrite-driven: a
  rewrite's `old_includes` path that exists on disk, is not the consumer,
  and is not a generated target. On the basic fixture the candidate
  `include/thing.h` already flows today - the C++ hook answers `None` and
  the file stays byte-identical. The derivation is NOT touched (SM-I4).
- Snapshot + strip phase: `apply_with` already snapshots every candidate
  source before mutating (`validate/src/lib.rs:173-176`) and
  `strip_moved_symbols` (:382-395) writes back whatever the hook returns
  (`Some`) inside the same transaction, so rollback already restores the
  original file. Scala exercises the whole spine with true deletion
  (`lang-scala/src/dependency_rewriter.rs:109-115`).
- Dry-run: `move_removal_count` (`extraction_reader.rs:232-262`) offers the
  hook read-only over on-disk text and reports `move_removals` /
  `copy_only` - computed, not asserted. For C++ it computes 0 today.
- The only absent piece: a C++ hook implementation, the plan-id context the
  marker needs, and copy truth for the new world.

Freeze adjudication against the CW shape section 0 freezes:

| Seam | Verdict | Role after this milestone |
|---|---|---|
| `IncludeRewriter` (`crates/transform/src/placement.rs`) | PERSISTS frozen, zero diff | path-space rewrite payload + preview builder, exactly as adjudicated; the move never routes through it |
| `plan_target_consumers` (`cli/src/commands/transform.rs:314`) | PERSISTS frozen, zero diff | neutral consumer discovery with the path payload; the move candidate derivation consumes that payload unchanged |
| `consumer_rewrite_spec` (C++ hook, `dependency_rewriter.rs:103-113`) | PERSISTS frozen, byte-identical | consumer-half owner; the strip phase runs AFTER the rewrite phase (`validate/src/lib.rs:246-273`) on different files - composition by ordering, no interaction |
| `remove_top_level_symbols` (trait fn + Scala impl + trait default `None`) | NAME and BEHAVIOR persist; the I5 DOC SENTENCE lifts (amendment 1) | the deletion path stays as Scala's choice; the new hook (C1) delegates to it by default, so Scala, validate's fake-based tests and every existing caller surface keep byte-identical behavior |
| I5 "C++ copy-only" as C++ semantics | LIFTS - this milestone is the named lift the CW kickoff sections 4/5 prepared | C++ opts into the hook (the CW-1 opt-in precedent repeated); the trait DEFAULT stays `None` (pins byte-identical), so copy-only survives as the honest fallback when the hook refuses |
| m20/m19 CW-1 amendments (consumer-side truth) | SURVIVE byte-identical | the second named amendment adds original-file truth only (kickoff section 7 item 2) |
| COPY_VS_MOVE_SENTENCE constant | SURVIVES byte-identical | still true on the `move_removals == 0` branch (hook-refusal fallback); its S5 unconditional use dies (amendment 4) |
| CW-3 no-rot locks (`never runs anything` absent; truncation-note positive pin), help copy claim of record | SURVIVE green UNEDITED | all three statements stay true under move truth |
| Boot payload byte lock, m18 parity bytes, m00, m10/m14/m17/m23, Scala suites, fetch-site `<= 2`, token/mutex/epoch gate proofs | SURVIVE UNEDITED | each is an SM invariant oracle below |

## 1. Capability map (at authoring HEAD)

| # | Capability | State | Where |
|---|---|---|---|
| 1 | Move candidate derivation + snapshot + in-transaction strip + rollback | EXISTS, proven | `validate/src/lib.rs:142-149,173-176,246-273,297-331,334-395`; `validate/tests/move_removal_transaction_test.rs` |
| 2 | Deletion-style hook (Scala) | EXISTS, green | `lang-scala/src/dependency_rewriter.rs:109-115`, `crates/lang/src/rewriter.rs:94-101` default |
| 3 | C++ post-move retention decision | ABSENT (the trait default `None`; I5 doc) | `crates/lang/src/rewriter.rs:94-101`; the doc line naming C++ copy-only |
| 4 | Plan-id context at the strip call (marker binding) | ABSENT (`strip_moved_symbols` sees no plan id) | `validate/src/lib.rs:382-395`; `migration.plan_id` already in scope at the call |
| 5 | Marker grammar (ASCII, plan-id bound, region delimiters, machine-parseable) | ABSENT | new; shape C2 |
| 6 | Dry-run move honesty for C++ | EXISTS machinery, wrong value (0 by honest computation) | `extraction_reader.rs:225-262`; required test 5 pins today's 0 |
| 7 | Apply-side faithfulness oracle (consumer half) | EXISTS | `extraction_apply/tests.rs` required test 1 + helpers (:233-421) |
| 8 | Move truth in the wizard copy | PARTIAL | `extraction-wizard.js`: move branches exist (:359, :392) but state "strips", S5 states the copy sentence unconditionally (:430), :575 conflates staging with move |
| 9 | Purge / cleanup command for marked regions | ABSENT by decision, registered follow-on | kickoff section 6 |

Rows 3-5 are the contract work; 1, 2, 6-8 are extension/reuse/audit.

## 2. Invariants (numbered, falsifiable, oracle named)

- **SM-I1 Marked-retention move truth (the core).** For the basic fixture,
  a committed apply leaves `include/thing.h` containing one
  `ARIADNE-EXTRACT-OUT` BEGIN/END pair per moved unit (four units, four
  pairs), each pair naming the approved plan id, and the region lines minus
  the `//` prefix equal the original declaration-span bytes verbatim; every
  byte OUTSIDE the marked regions of `include/thing.h` is unchanged; no
  live declaration of a moved name remains (re-offering the hook over the
  post-apply text with the same names answers `None`). Oracle: the m20/m19
  riders (kickoff section 7 item 2) + SM-2 unit tests over the hazard corpus
  (class-in-namespace, template, multi-line signature, macro with trailing
  backslashes, CRLF file, already-marked file).
- **SM-I2 Grown faithfulness equality.** Required test 1 grows to:
  `modified == expected_modified` with `expected_modified` defined as C5
  (consumers-with-hits UNION retirable move sources, both read PRE-apply);
  on the fixture `modified == approved_consumers UNION
  {include/thing.h}`; `written == approved_new_files` and the no-deletions
  claim survive UNCHANGED. Oracle: SM-4's grown required test 1 + the
  `move_proof` marker assertions (C5 conditions 4-6).
- **SM-I3 Dry-run/apply move-claim parity.** For the same generation and
  tree state, `dry_run.move_removals` equals the count of symbol names
  bound into markers by the subsequent apply; `copy_only ==
  (move_removals == 0)` keeps its computed meaning. Oracle: SM-4 test
  comparing the dry-run answer against the post-apply marker count;
  amended reader required test 5 (amendment 3).
- **SM-I4 Consumer half and derivation byte-truth.** `collect_move_removals`,
  `strip_moved_symbols` ordering, `IncludeRewriter`,
  `plan_target_consumers`, `consumer_rewrite_spec`, and the C++ fidelity
  pair test are ZERO-DIFF; the m20/m19 consumer-side assertions from CW-1
  stay byte-identical; m18 parity preview bytes UNEDITED. Oracle: reviewer
  per-commit diffs + those suites green UNEDITED.
- **SM-I5 Scala neutrality by delegation.** Every Scala file zero-diff;
  `move_out_top_level_symbols` over the Scala rewriter returns EXACTLY what
  `remove_top_level_symbols` returns (byte-identity pin added in SM-1); the
  validate fake (old hook only) still drives the strip through the default
  delegation - `move_removal_transaction_test.rs` green UNEDITED IS the
  proof; m23 + m17 cone guard UNEDITED. Oracle: SM-1 pin + those suites.
- **SM-I6 Gate machinery untouched.** `apply_plan_with` caller census stays
  3; required tests 2-11 of the apply proof (token/mutex/epoch/honest-
  outcome/ASCII/single-delegation) green UNEDITED; no new POST route, no
  token change, fetch-site bound `<= 2` survives. Oracle: those tests +
  reviewer diff (the one changed test file is required test 1's module tree).
- **SM-I7 In-transaction rollback still restores the original file.** A
  failing tier after a strip restores `include/thing.h` byte-identically
  (the snapshot already covers move paths; nothing may weaken it). Oracle:
  `move_removal_transaction_test.rs` UNEDITED (SM-I5 route) + the A-5
  whole-tree rollback proof (required test 3) UNEDITED.
- **SM-I8 Copy truthfulness.** No shipped asset string claims the apply
  "strips" declarations, claims copy-only for a plan whose dry run reports
  moves, or promises a one-click undo; S5 keys its copy off the computed
  `move_removals`; the CW-3 no-rot locks stay green UNEDITED. Oracle: the
  amended W4 lock + the new move-sentence no-rot lock (SM-5).
- **SM-I9 Discipline.** Every generated/marked byte and every DTO string
  pure ASCII (A9 channel); `encoding_guard --all` exit 0; hygiene R1/R2
  exit 0 (with the SM-4 `tests.rs` split keeping it < 1000); zero added
  hex; census deltas attributed from the anchor.

## 3. Contracts

### C1 The neutral move-text hook (crates/lang)

```rust
// crates/lang/src/rewriter.rs, on DependencyRewriter
/// The post-move source-text contract: return the text that REPLACES
/// `source` once the named top-level symbols have moved into their
/// extracted units, or `None` when any name cannot be safely retired -
/// the caller then proceeds WITHOUT the move (copy fallback). The
/// RETENTION FORM is the language's choice: Scala deletes via the
/// inherited parser spans; C++ comments regions out under
/// `ARIADNE-EXTRACT-OUT` markers bound to `plan_id` (the user's
/// reversible-move decision of record). `plan_id` is marker provenance
/// only - a language whose retention needs no provenance ignores it.
/// The default delegates to `remove_top_level_symbols` (deletion), so
/// every existing implementation keeps byte-identical behavior.
fn move_out_top_level_symbols(
    &self,
    source: &str,
    qualified_names: &[String],
    plan_id: &str,
) -> Option<String> {
    self.remove_top_level_symbols(source, qualified_names)
}
```

No rename, no signature change to anything existing (Scala pins and the
validate fake resolve through the default - SM-I5). `crates/lang` grows
this method + two new pin tests: default-delegates-equals-old-hook, and
the existing `remove_top_level_symbols_default_is_none...` pin stays
byte-identical UNEDITED. The I5 doc sentence on the old hook lifts to
state the truth (amendment 1): the default is still `None`; C++ now opts
into the new hook; deletion-vs-marking is the language's choice.

### C2 The C++ implementation and marker grammar (crates/lang-cpp)

`impl DependencyRewriter for CppDependencyRewriter` gains
`move_out_top_level_symbols`, delegating to a NEW module
`crates/lang-cpp/src/move_retention.rs` (keeps `dependency_rewriter.rs`
390 -> ~420 and the grammar in its own intention-revealing module). The
region source is the existing verbatim declaration locator
(`crates/lang-cpp/src/symbol_locator.rs`, READ ONLY - untouched); the
hook NEVER re-slices text the locator did not certify.

Grammar (pure ASCII by construction; every line well under 2000 chars):

```
// ARIADNE-EXTRACT-OUT BEGIN plan=<plan-id> symbols=<names>
//<original line 1>
//<original line 2>
...
// ARIADNE-EXTRACT-OUT END plan=<plan-id>
```

- Retention is LINE-PREFIX COMMENT ONLY: each region line gets the
  two-byte prefix `//` prepended to the ORIGINAL line bytes (no added
  space - stripping the prefix restores the original exactly; an empty
  original line becomes bare `//`). Block comments are FORBIDDEN as a
  retention form (a `*/` inside moved code would terminate the region
  comment mid-symbol).
- `<plan-id>`: the `migration.plan_id` rendered through the A9
  `ascii_only` channel (`?` per non-ASCII char); plan ids are
  whitespace-free by production (`PLAN-001` form), asserted in tests.
- `<names>`: the requested qualified names joined by `;`, each first
  normalized by mapping any char outside `[A-Za-z0-9_:]` to `_`
  (documented one-way; names are provenance for humans, the byte truth
  lives in the region lines, restore does not depend on them).
- Recognition (machine cleanup / future tooling): lines matching
  `^// ARIADNE-EXTRACT-OUT BEGIN plan=(\S+) symbols=(\S+)$` and
  `^// ARIADNE-EXTRACT-OUT END plan=(\S+)$`. One BEGIN/END pair per
  located declaration span; adjacent/overlapping spans merge to one
  region; regions apply bottom-up so earlier line indices stay valid.
- CRLF: the file's line terminators are preserved verbatim per line, and
  marker lines use the file's dominant terminator.
- Refusal contract (returns `None`, caller proceeds without the move,
  dry run reports it as un-movable): any requested name the locator
  cannot locate; any requested name locatable ONLY inside an existing
  marked region (already moved - idempotence, never nested marking); the
  region start inside an unterminated block comment; the line immediately
  BEFORE the region ending in a backslash (line-splice would pull the
  BEGIN marker into that line's logical continuation); the region's LAST
  line ending in a backslash (the END marker would splice into the
  comment chain and break the balanced-pair grammar). Each refusal is a
  named unit test; a refusal means the whole candidate is skipped whole
  (per-unit atomicity, mirroring the T8b derivation comment).

Cleanup story: the grammar is designed for a future purge command
(kickoff section 6); no expiry, no automatic deletion, this milestone ships no
purger. The region is inert to the tokenizer (line comments), so re-
analysis, the verbatim locator and the M25 tokenizer class treat marked
text as comments; the one named exception (the C++ RAW substring consumer
scan could match a `#include` inside a marked region if moved code ever
contains one) is risk R1 with its registered follow-on.

### C3 Plan-id threading (crates/validate)

`apply_with` already holds `migration` (which carries `plan_id`); thread
`migration.plan_id` through `write_all_targets` into
`strip_moved_symbols`, whose call becomes
`rewriter.move_out_top_level_symbols(&content, names, plan_id)`. No
signature change on the public `apply_with`/`rollback`/`verify_restored`
surface; nothing in validate names a language (I2); the snapshot set is
unchanged (SM-I7 route). ~10 production lines.

### C4 Dry-run honesty (crates/cli reader)

`EngineBackedExtractionReader::move_removal_count` offers
`move_out_top_level_symbols` with `plan.id.0` (same candidates, same
on-disk read-only text - the W4 "computed, not asserted" rule). DTO
bytes, caps and the honest-absent shape do NOT change (no DTO field is
added; X3-class frozen display stays). Required test 5 flips to move truth
(amendment 3): on the fixture it asserts `move_removals > 0` (the count
the locatable fixture names produce is the hand's journal measurement)
and `copy_only == false`; the honest-absent pin's
`move_removals == 0 && !copy_only` shape is untouched.

Dry-run preview must-show (the reversibility contract in the UI): the
move branch sentence states (a) which file's declarations get marked
(the plan target is already displayed in S1/S3), (b) the count, (c) that
the declarations are commented out in place under `ARIADNE-EXTRACT-OUT`
markers naming the plan, and (d) the restore procedure (uncomment the
region AND revert the consumer include rewrites, or git) - never a
one-click claim. Count + file + plan id are already DTO-available; no new
route, no new field.

### C5 The oracle grows - new truth conditions (crates/cli proof)

Required test 1 (`test_a_valid_token_applies_the_plan_end_to_end`) grows
exactly these conditions (helpers into a new `move_proof` submodule of
the tests module to respect the ceiling, section 8):

1. `expected_modified = {approved consumer rows with a pre-apply
   directive hit} UNION {move candidates (reader's own
   `move_removal_candidates` derivation) whose pre-apply text
   `move_out_top_level_symbols` returns `Some` for}` - all reads PRE-
   apply, both sets host-canonical, sorted.
2. Post-apply: `modified == expected_modified` (no silent drop, nothing
   outside the expected set) and on the fixture
   `modified == approved_consumers UNION {"include/thing.h"}` - the CW-2
   equality generalized; the consumer-only equality `expected_modified
   consumers == approved_consumers` survives as a sub-claim.
3. `written == approved_new_files` field-by-field and the no-deletions
   claim: byte-identical UNEDITED claims.
4. Marker proof per marked original file (from the pre/post walks):
   balanced BEGIN/END pairs, count == hook-success calls; every marker
   line carries the approved plan id; prefix-stripping each region's
   lines reconstructs the original span bytes byte-exactly; every marker
   byte ASCII; re-offering the hook over the post-apply text with the
   same names answers `None` (the region is inert).
5. Move-count parity: `dry_run.move_removals ==` the total symbol names
   bound into the written markers (SM-I3).
6. Rollback coverage of the original file is NOT re-proven here - it is
   `move_removal_transaction_test.rs` green UNEDITED through the default
   delegation (SM-I7; the fake implements the old hook and the default
   routes through it - byte-identity of that file is the proof).

### C6 Copy truth audit (crates/ui assets)

Lines that CHANGE (all inside hand SM-5, Red-first, each named here):

1. `extraction-wizard.js:359` (S3 move branch): "the plugin strips N
   moved declaration(s) from the source files" -> the marked-retention
   sentence: declarations are commented out in place under
   `ARIADNE-EXTRACT-OUT` markers naming the plan, reversible by
   uncommenting plus reverting the consumer includes (or git).
2. `extraction-wizard.js:392` (S4 move branch): "strips N moved source
   declaration(s) as it writes" -> the same sentence shape.
3. `extraction-wizard.js:430` (S5 Execute): the unconditional
   `COPY_VS_MOVE_SENTENCE` -> keyed off `wizardCopyOnly(dryRun)` +
   `includes_unavailable` exactly like S4's structure (the sentence is
   false on the move branch; amendment 4 locks the keyed form).
4. `extraction-wizard.js:575` (S6 help): "in place for move languages"
   (staging-vs-move conflation) -> "in place for languages that stage
   units in place"; if a W-3b-era lock pins the string, that lock joins
   the amendment table as clause 5 in the handoff (conditional, closed
   set, no other card-lock edit).

Lines that STAY (audited true under move truth - state them in the
handoff so the implementer does not "fix" them): the help-copy claim of
record "the UI applies with a dry-run first, validation in three tiers
and automatic rollback" (still true); `COPY_VS_MOVE_SENTENCE` constant
bytes (true on the `move_removals == 0` fallback branch); the S4
truncation note + its no-rot lock (consumer truth unchanged, CW-1); the
hotspots.js claim-of-record block + its no-rot lock; the tier/rollback
S6 lines (the original file IS in the snapshot now - truer than before);
"The dry run changes nothing; tool apply performs the same computation
before writing.".

### C7 Amendment table (closed; mirrors kickoff section 7)

| # | Lock | Hand | Change |
|---|---|---|---|
| 1 | `lang/src/rewriter.rs` I5 doc sentence | SM-1 | doc wording only; all pin TESTS byte-identical |
| 2 | m19 + m20 named apply tests (second amendment) | SM-3 rider | additive original-file marker proofs + the stale "not a move" comment; consumer-side CW-1 assertions byte-identical |
| 3 | `extraction_reader.rs` required test 5 | SM-4 | copy-only truth -> move truth; tests 1-4, 6-10 UNEDITED |
| 4 | `test_copy_vs_move_sentence_is_exact_and_keyed_off_the_computed_path` (S5 clause) | SM-5 | unconditional -> keyed; sentence constant and S3/S4 assertions survive |
| 5 | (conditional) any W-3b lock pinning `:575` help text | SM-5 | only if a grep at touch proves a pin; otherwise C6 item 4 is a plain copy edit |

Nothing else pre-existing moves; discovery mid-hand of any other lock
wanting an edit is STOP-and-report (CW-3 discipline precedent).

## 4. Mechanism adjudication (the kickoff's decision of record, in full)

Options evaluated: (a) comment-out-and-mark retention in the original
file; (b) true deletion relying on the shipped transaction rollback;
(c) hybrid (persisted restore journal / undo command).

**Why not (b), stated against the user's worry, not against the wording:**
rollback is real but BOUNDED - `TransactionApplier` snapshots in memory
and the snapshot is dropped once the outcome returns; `commit_or_rollback`
fires rollback only on a tier FAIL within the same transaction; and tier
3 never compiles anything (D-M23-12: "extracted headers are not yet self-
contained", M18.10 honest-skip; include-closure is a registered follow-on,
ledger row 446). So on the flagship corpus a committed move can break a
build that no tier can detect, and by then the only restoration sources
are the verbatim bodies inside the generated units (manual copy-back) or
git. Calling that "easily reversable" would be exactly the claim-behavior
gap D6 was. Deletion-only is REJECTED as the C++ default.

**Why not (c):** a persisted restore journal is a second source of truth
with a lifetime problem (artifact trees are rewritten by re-analysis; the
user's git state is unknowable), invisible to the user at the moment of
the break, and a whole new command surface (restore, staleness rules,
partial restores) - all to deliver what six lines of ASCII comment per
symbol already deliver in the place the user will actually look. REJECTED.

**Why (a), and why it is specified rather than shipped as the user's
raw suggestion:** in-file marked retention satisfies the mandate without
touching the safety stack (one write path, one gate, one transaction -
unchanged). The user's suggestion ("perhaps by") is adopted as the
mechanism with the hazards specified away: LINE-prefix grammar (block
comments cannot be used around arbitrary C++ text), refusal contract at
unsafe boundaries (backslash continuations, open block comments,
unlocatable names - the same "refuse rather than guess" discipline as
`consumer_rewrite_spec`'s honest fallback), idempotence via the
tokenizer's comment invisibility, and an explicit never-expires cleanup
policy with a registered purge follow-on so the dead-text cost is bounded
rather than permanent-by-accident. The file-health objection is real and
is accepted WITH EYES OPEN: marked regions are mechanically deletable
(grammar C2) and the purge command is the first follow-on candidate; the
alternative (silent deletion) trades a hygiene annoyance for a
reversibility failure, which the decision of record ranks worse.

**Plugin split (extensibility intent of record):** the trait owns the
neutral contract ("return the post-move original text, or refuse") - C1;
each language owns its retention form - C2 is C++'s choice (marked
comment-out, per the user), Scala's deletion choice survives untouched
behind the default delegation (zero Scala files in any diff), and a
future Scala-side choice (e.g. marked retention for a Scala project) is
one plugin method away. This milestone is the proof that the move
mechanism is a plugin concern, not a pipeline concern: validate,
transform and cli neutral layers gain only a string parameter.

## 5. Module map (per hand; also the must-not table)

| Hand | Changes | Touched (read/proof) | Must not change |
|---|---|---|---|
| SM-1 (crates/lang) | `src/rewriter.rs` (new default hook, I5 doc lift, 2 pin tests) | every existing pin test as gate | Scala/C++ crates, validate, any other file |
| SM-2 (crates/lang-cpp) | `src/move_retention.rs` (NEW: grammar, region builder, refusals, tests), `src/dependency_rewriter.rs` (the hook method delegating) | `symbol_locator.rs` + tokenizer (READ ONLY), CW-1 fidelity tests green | `symbol_locator.rs`, `parser.rs`, the four frozen rewriter fns + CW-1 hook byte-identical |
| SM-3 (crates/validate + declared riders) | `src/lib.rs` (plan-id threading to the strip call) | `move_removal_transaction_test.rs` green UNEDITED (SM-I5 proof) | riders (declared): `tests/integration/tests/m19_...rs` + `m20_...rs` second amendment only; `transform.rs`, `extraction_reader.rs` (still reports 0 until SM-4 - risk R5), any other test |
| SM-4 (crates/cli) | `commands/ui/extraction_reader.rs` (hook offering + required-test-5 amendment), `commands/ui/extraction_apply/tests.rs` (+1 `mod move_proof;` line) + NEW `commands/ui/extraction_apply/tests/move_proof.rs` (grown oracle per C5) | required tests 2-11, honest-absent pins, W9 boot-payload lock | `transform.rs`, `extraction_apply.rs` (295 stays), DTOs, token/mutex code, the generation-swap test (trigger re-fires, relayed NOT executed, kickoff section 4) |
| SM-5 (crates/ui) | `assets/extraction-wizard.js` (C6 items 1-4), `tests/ui_test/extraction_wizard_render.rs` (amendment 4 + new move-sentence no-rot lock) | `ui_smoke`, hex guard, CW-3 locks UNEDITED, fetch bound | card-family other locks, `hotspots.js`, `index.html`, routes/DTOs/state, `styles.css` |

Hard edges: SM-1 -> SM-2 (the impl needs the hook), SM-2 -> SM-3 (wiring
lands the effect the riders assert), SM-3 -> SM-4 IMMEDIATELY (the dry run
under-reports between them - R5), SM-4 -> SM-5. House default: strictly
serial on main, one scoped landing commit per hand, journal riders per
precedent. No hand touches two crates except SM-3's declared integration-
test riders (A-3 rider precedent).

## 6. Handoff sketch sized to the law (planner input; no sequencing here)

| Hand | Crate dir | Est. produced lines | Est. gates | Kind |
|---|---|---|---|---|
| SM-1 | crates/lang | ~100-160 | ~7 | contract extension + doc lift |
| SM-2 | crates/lang-cpp | ~320-460 | ~9 | the plugin mechanism + hazard corpus |
| SM-3 | crates/validate + 2 declared rider files | ~250-380 | ~9 | threading + frozen-lock second amendment |
| SM-4 | crates/cli | ~420-560 | ~10 | dry-run honesty + grown oracle |
| SM-5 | crates/ui | ~120-220 | ~6-7 | copy truth + locks |

All hands: one crate dir, `<= ~10` distilling-entry-point invocations
(including the commit-cycle trio), `<= ~1200` produced lines priced
itemized FROM the mandated test bodies (M23.5 sizing-arithmetic law).
Every hand's gates include: scoped suite via `testrun --distill`, the
named characterization/parity suites, full census via the `--distill`
substitute, `encoding_guard --all`, `hygiene_check`, fresh `bench_compare`
recording (record WARNs, attribute nothing; PR-2 persists); SM-5 adds
`ui_smoke` + `theme_token_check --check-added --base`. Merge option for
the planner: SM-3's production diff is ~10 lines, so a validated merge of
SM-3+SM-4 (validate change as a declared rider in the cli hand) is legal
IF the merged hand still fits the law - but the m19/m20 rider's Red must
then run BEFORE the reader flip lands, which the serial SM-3-then-SM-4
order gives for free; the planner decides at authoring.

## 7. Risks (named mitigation + oracle per risk)

- **R1 Marked text vs the RAW substring consumer scan.** C++ discovery
  matches `#include` lines inside comments by design (I1 substring
  fidelity). If a moved region ever contains an `#include` of a later
  plan target, the marked file could present as that target's consumer.
  Mitigation: moved regions are top-level DECLARATION spans (the locator
  certifies declarations; includes are not declarations, and the fixture
  corpus proves it); registered marker-aware-exclusion follow-on
  (kickoff section 6). Oracle: none test-locks this beyond the fixture pair test
  (SM-I1 hazard corpus pins "a marked file re-offered answers None");
  stated plainly: general corpora are not provable here, which is
  precisely the include-closure workstream's neighborhood.
- **R2 Span-safety at region boundaries (the M25 hazard class).**
  Mitigation: the refusal contract (C2) turns every unsafe boundary into
  `None` = copy fallback; the hazard corpus is mandated tests (backslash
  continuations both sides, open block comment, nested region, template,
  macro body). Oracle: SM-2 unit tests; no refusal path may weaken the
  locator itself (it is READ ONLY, zero diff).
- **R3 `extraction_apply/tests.rs` ceiling (903).** SM-4's growth goes to
  `tests/move_proof.rs` (submodule), keeping `tests.rs` ~+1 line;
  `move_proof.rs` itself stays well under 1000. Oracle: `hygiene_check`
  exit 0 at SM-4; the planner prices it from C5's bullet list.
- **R4 Second amendment of a twice-touched frozen lock.** m19/m20 were
  amended once (CW-1); a second amendment is the sharpest repeat risk.
  Mitigation: closed policy (kickoff section 7), rider on the named hand only,
  additive-only hunks on the original-file side, CW-1's consumer
  assertions byte-identical, reviewer per-commit diff, the grown oracle
  (never the amended test alone) is the machine witness. Oracle: X-style
  reviewer diffs + SM-I2 green.
- **R5 Transient under-report between SM-3 (disk moves) and SM-4 (dry
  run reports it).** For one inter-hand window the wizard shows the copy
  sentence while apply would mark. Mitigation: SM-4 IMMEDIATELY after
  SM-3, no other landing between, milestone gate forbids shipping a
  release between them; honest per hand at closeout. Oracle: SM-I3 test
  at SM-4; reviewer verifies adjacency of the two landings.
- **R6 Uncomment-redefinition confusion (uncommenting without reverting
  consumer includes yields duplicate declarations).** Mitigated by copy
  C6 (the restore procedure ships in the same sentence, never a
  one-click claim). Oracle: SM-I8 lock wording.
- **R7 Scala regression via the default delegation.** Impossible by
  construction (Scala overrides the OLD hook; the default only forwards);
  byte-identity pin + zero-diff rule. Oracle: SM-I5.
- **R8 Gate drift.** No caller-census, token, mutex, epoch or route
  change; SM-I6 keeps the A-4 proofs green UNEDITED. Oracle: required
  tests 2-11.
- **R9 Re-analysis sees the moved tree.** Post-apply boots differ by
  design (that is the product); the W9 byte lock boots the UNAPPLIED
  fixture and survives; `m00_facts_path_parity` runs on unapplied
  fixtures. Stated, not locked further.

## 8. Hygiene notes

- `extraction_apply/tests.rs` 903: SM-4 adds ~1 line (`mod move_proof;`)
  + the C5 bullets to `tests/move_proof.rs` (~250-330 new). THE WATCH
  THIS MILESTONE OWNS; do not add marker assertions inline.
- `dependency_rewriter.rs` 390 -> ~420 (one delegating method); grammar
  lives in `move_retention.rs` (~250-320, functions well under 100).
- `validate/src/lib.rs` 847 -> ~855. `extraction_reader.rs` 899 -> ~910
  (still < 1000; margin noted, no split owed).
- `lang/src/rewriter.rs` 221 -> ~300. `extraction_wizard_render.rs`
  ~910 -> ~950 (amended test + one lock; under 1000, margin note).
  `extraction-wizard.js` ~645 asset (invisible to R2, ASCII lock holds).
- No line anywhere near 2000 chars; all authored content pure ASCII;
  names: `move_out_top_level_symbols`, `move_retention`,
  `region_marker_open`, `region_marker_close`, `prefix_out_region`,
  `strip_marker_prefix`, `refuse_unsafe_region_boundary`,
  `expected_modified_with_move_sources`, `assert_marked_regions`
  (no `tmp`/`res`/`data`/single-letter locals).

## 9. Freeze set and census

Frozen for the whole milestone (beyond the permanent sets -
`tests/golden/**`, `tests/fixtures/**` byte-locked `UPDATE_GOLDENS=1`
forbidden, `tools/**` operator-law, theme files + baseline): every Scala
file; `crates/cli/src/commands/transform.rs`; `crates/transform/**`;
`crates/lang-cpp/src/symbol_locator.rs` + tokenizer + parser;
`dependency_rewriter.rs` CW-1 regions byte-identical; the
`extraction_apply.rs` 295-line producer; DTO bytes; token/mutex/epoch
machinery; the boot payload bytes (W9); m10/m14/m17/m18/m23 + m00; the
wizard fetch-site bound `<= 2`; the CW-3 no-rot locks; `hotspots.js`;
`index.html`; the bind surface. Census anchor
`1837 passed / 0 failed / 13 ignored / 119 suites @ b201aa6`; hands report
deltas from a clean-tree scoped baseline in their journals; the
`--distill` census substitute governs (the `--baseline` FILE form stays
superseded, M23.5 F9). Expected net new tests: SM-1 +2, SM-2 +8-10, SM-3
+0 (amendments only), SM-4 +3-5, SM-5 +1; exact counts are each hand's
journal duty.
