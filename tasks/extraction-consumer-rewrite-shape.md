# Milestone Shape - extraction-consumer-rewrite

Companion to `tasks/extraction-consumer-rewrite-kickoff.md` (key verbatim
there; the four inherited deferred rows are quoted there verbatim and are
the boundary of record). Read-only survey at `a38c7aaa` on `main` (git
MCP; the architect role has no bash, so `checkpoint_guard` was not run -
the first hand bootstraps from it). Only the live dirty stray at authoring
is the pre-existing `extensions/.../ariadne-extension-0.1.0.vsix`. Pure
ASCII throughout; every line number and claim below was measured on disk
in the shaping session.

## 0. The defect, precisely, and the seam where it dies

Consumer discovery and consumer rewriting disagree about spelling.

- Discovery (`crates/cli/src/commands/transform.rs:285-346`, shared by the
  apply path, the `tool optimize` path (`optimize.rs:336-364`), and the
  wizard dry-run reader (`extraction_reader.rs:199-223`)) probes each TU
  with the plugin's dependency-space targets - for C++ the plan target's
  BARE file name (`thing.h`) via the trait default
  `consumer_discovery_targets` (`crates/lang/src/rewriter.rs:66-75`). A hit
  makes the TU a consumer.
- The consumer payload then reports `old_includes` as the plan target's
  PATH (`vec![target_file.clone()]`, `transform.rs:340`), i.e.
  `include/thing.h`. That path payload is correct as identity data: the
  dry-run DTO mirrors it (`crates/ui/src/dto/extraction.rs:163-172`), the
  migrate preview persists it (m20 locks the value), and the transaction's
  move-removal derivation needs on-disk paths
  (`crates/validate/src/lib.rs:345-370`, load-bearing for Scala's move).
- At apply time the text mutation is the plugin's:
  `consumer_rewrite_spec` is "the caller keeps the path strings - today's
  C++ behavior byte-identical" default for C++
  (`crates/lang/src/rewriter.rs:85-92`), so `RewriteSpec.old_deps` carries
  `include/thing.h` and `CppDependencyRewriter::rewrite`
  (`crates/lang-cpp/src/dependency_rewriter.rs:25-38`) replaces
  `#include "include/thing.h"` - a directive nobody writes. The rewrite is
  vacuous; m19 and m20 characterize exactly this ("latent vacuous-rewrite
  bug, characterization only", `m19...:139-141`, `m20...:226-228`).

The trait already reserves the seam: directive spellings belong to the
plugin, `consumer_rewrite_spec` is the named hook, and Scala already
implements it (`crates/lang-scala/src/dependency_rewriter.rs:160`). The
fix is C++ opting into the hook with the SAME file-name rule its
discovery uses, so discovery's predicate and the rewrite's matcher agree
by construction.

Ledger-row adjudication (the D6 headline row names the fix owner as
"crates/transform (`IncludeRewriter`) plus the consumer-discovery
`old_includes` spelling"; the row stands as the boundary of record - this
is the named seam inside it, not a deviation from it):

- Changing `plan_target_consumers` to push the bare spelling would fix
  C++ by leaking one language's spelling into the shared neutral function
  (I2 violation), break the move-removal derivation for move-capable
  plugins (it needs on-disk paths), and falsify the DTO/preview path data
  that m20 and the m18 parity suite byte-lock. REJECTED.
- Changing `IncludeRewriter::compute_rewrite` would rewrite path data the
  preview locks and still could not know the directive grammar. REJECTED;
  `crates/transform` stays untouched.
- The alternative that survives on the C++ side of the plugin boundary is
  exactly this shape. The apply/gate/transaction layers need no change at
  all, which is also why the blast radius is small.

## 1. Capability map (the honest inventory, at `a38c7aaa`)

Legend: EXISTS / PARTIAL / ABSENT as in the predecessor shape.

| # | Capability | State | Where (file:line) |
|---|---|---|---|
| 1 | Consumer discovery via plugin targets (bare name for C++) | EXISTS, correct | `transform.rs:285-346`; trait default `lang/src/rewriter.rs:66-75` |
| 2 | Path-space rewrite payload (`old_includes` = plan target path) | EXISTS, correct as data | `transform.rs:340`; DTO `ui/src/dto/extraction.rs:163-172`; preview locked by `m20:113-116` |
| 3 | Path -> directive-identifier mapping at the rewrite boundary | PARTIAL - Scala yes, C++ falls back to raw paths (THE defect) | hook `lang/src/rewriter.rs:85-92`; Scala impl `lang-scala/.../dependency_rewriter.rs:160`; call site `validate/src/lib.rs:255-268` |
| 4 | C++ directive text ownership (rewrite of a correctly spelled spec) | EXISTS, correct | `lang-cpp/src/dependency_rewriter.rs:25-38` (replace-all both spellings, all occurrences) |
| 5 | Discovery/rewrite agreement (a rewrite fires iff discovery hit) | ABSENT for C++ (vacuous); HOLDS for Scala | m19:137-142, m20:224-229 lock the disagreement |
| 6 | Positive faithfulness oracle (expected-modified equality) | ABSENT (A-4 shipped subset parity + non-vacuity) | `cli/src/commands/ui/extraction_apply/tests.rs:233-300` |
| 7 | Move strip through the same transaction | EXISTS for move-capable plugins; C++ copy-only by design (I5) | `validate/src/lib.rs:334-395`; hook default `lang/src/rewriter.rs:99-101`; dry-run count `extraction_reader.rs:232-262` |
| 8 | Honest copy about what apply does | PARTIAL - help copy flipped (A-5), two strays remain | `hotspots.js:519-522` comment; `extraction-wizard.js:385-387` note (unlocked today - grep-proven) |
| 9 | `A9` plan_id sanitization + cosmetic indentation + deleted-files absence check in the apply proof | ABSENT (dated "next touch" rows; this milestone is that touch) | `cli/src/commands/ui/extraction_apply.rs:146-170` region; `extraction_apply/tests.rs` test 1 |

Consequence: one plugin-hook hand, one cli-proof hand, one ui-copy hand.
Rows 1, 2, 4, 7 are reuse; rows 3, 5, 6, 8, 9 are the contract work.

## 2. Invariants (numbered, each independently falsifiable, oracle named)

- **X1 Discovery/rewrite fidelity (the core).** For every consumer text
  and every plan target: the consumer rewrite CHANGES that file's text iff
  discovery reported a target hit in it (non-empty `new_deps` assumed;
  vacuity-guarded). Two oracles: (a) lang-cpp unit tests over the spelling
  classes - bare quoted, angled, CRLF, comment-embedded (C++ discovery
  matches these; replace-all matches them too - the pair is symmetric),
  unrelated path spellings (`sub/thing.h` must neither hit nor change),
  no-match byte-identity; (b) end-to-end, the CW-2 oracle equality
  modified == expected-modified == approved consumers on the fixture.
- **X2 Consumer-set equality (I1) survives.** The set/count of files
  reported as consumers is identical before and after: m10/m14/m19/m20/m23
  set assertions (`affected_consumer_files == 2`, `preview.rewrites.len()
  == 2`, `effective_rewrites() == 2`) and the m18 parity fixture outputs
  stay green with those clauses byte-identical UNEDITED (only the named
  amendment hunks move inside m19/m20, and none of them is a set
  assertion).
- **X3 Preview and dry-run bytes unchanged.** `tool migrate`
  `preview.json`/`preview.bin` content assertions (m20 migrate test),
  m18 parity `preview_json` + `diff_text` (note: the diff text itself
  renders `- #include "include/thing.h"` - it is path-space display and
  STAYS byte-identical; the fix is below the payload layer), and the
  wizard dry-run DTO bytes all survive UNEDITED.
- **X4 Plugin neutrality (I2).** No language name or C++ spelling string
  enters `crates/cli`, `crates/transform`, `crates/validate`, or
  `crates/lang` sources in any commit; the m17 plugin-seam transitive-cone
  guard and every Scala suite are green UNEDITED; `cargo check` adds no
  dependency edges.
- **X5 Copy-only semantics preserved.** `CppDependencyRewriter` gains NO
  `remove_top_level_symbols`; after a fixed apply, `src/thing.cpp` still
  declares Thing's members (the m20 amendment asserts declaration
  survival next to the include flip); `COPY_VS_MOVE_SENTENCE`
  (`extraction-wizard.js:219`) remains true and byte-unchanged; the
  trait-default pins in `crates/lang/src/rewriter.rs` survive UNEDITED.
- **X6 The apply gate is untouched.** Caller census of `apply_plan_with`
  stays 3; the A-4 token/mutex/epoch/e2e proofs (required tests 2-11)
  stay green UNEDITED; the wizard fetch-site bound (`<= 2`) survives the
  copy hand with zero new fetch sites.
- **X7 ASCII and hygiene discipline.** Every byte written by a rewrite on
  an ASCII tree stays ASCII (spelling rule maps paths through
  `to_string_lossy`, same channel as today's fallback);
  `encoding_guard --all` exit 0; hygiene R1/R2 exit 0; zero added hex.
- **X8 Amendment scope.** Exactly two characterization tests are amended
  (m19:110, m20:179, plus the m19 test's name), one pre-existing cli test
  is upgraded (A-4 required test 1, CW-2), one asset sentence is reworded
  (wizard S4 note), one code comment is flipped (`hotspots.js:521-522`),
  and NOTHING else pre-existing changes, proven by reviewer per-commit
  diffs against each hand's start rev.

## 3. Contracts (numbered, machine-checkable acceptance)

### C1 The C++ rewrite-spec hook (crates/lang-cpp)

```rust
// crates/lang-cpp/src/dependency_rewriter.rs, inside
// impl DependencyRewriter for CppDependencyRewriter

/// Path payload -> directive identifiers with the SAME bare-file-name
/// rule `consumer_discovery_targets` probes with. A path without a file
/// name keeps its display string (honest fallback, mirrors the trait
/// default's refusal to invent a probe).
fn consumer_rewrite_spec(
    &self,
    old_includes: &[std::path::PathBuf],
    new_includes: &[std::path::PathBuf],
    _moved_module_ids: &[String],
) -> Option<ariadne_lang::RewriteSpec> { ... }

fn dependency_spelling(path: &std::path::Path) -> String // file_name rule
```

Always `Some` (C++ always has directive grammar). `moved_module_ids` is
unused by C++ (its directive space names files, not modules).

Acceptance (lang-cpp unit tests, all new, all green at hand close):

1. Identity rows: `["old.h"] -> ["new.h"]` maps to identical
   `RewriteSpec` as the paths (this is why the relocated
   `crates/validate/tests/transaction_apply_test.rs` byte-parity test
   survives UNEDITED - it already uses bare names).
2. Path rows: `["include/thing.h"] -> old_deps ["thing.h"]`;
   `["generated/include/Thing_extracted.h"] -> new_deps
   ["Thing_extracted.h"]`.
3. Fallback row: a path with no file name keeps its display string in the
   spec.
4. Fidelity pair (X1(a)): for the fixture-shaped text, the same spelling
   classes listed in X1 - discovery hit iff `rewrite` changes the text;
   the comment-embedded pair is pinned explicitly on BOTH sides.
5. Scala's `consumer_rewrite_spec` and the trait defaults: files in no
   diff (X4) - reviewer diff proof, not a test.

### C2 m20 characterization amendment (rider on CW-1)

`tests/integration/tests/m20_apply_migrate_characterization_test.rs`,
test `apply_thing_plan_locks_transaction_outcome_and_disk_effects`
(:179) ONLY. New truth after `apply_transform_plan` on the basic fixture:

- `tests/consumer.cpp` and `src/thing.cpp`: the quoted old directive
  `#include "thing.h"` is gone; the four extracted includes appear in
  place as bare quoted directives (`ThingConfig_extracted.h`,
  `Thing_Thing_extracted.h`, `Thing_extracted.h`,
  `Thing_get_id_extracted.h` - the same BTreeSet order the generated-file
  list already asserts); all other bytes of both files are unchanged.
- `src/thing.cpp` still contains Thing's declarations (X5: the flip is an
  include rewrite, not a move).
- The assertion messages that recorded "latent vacuous-rewrite bug,
  characterization only" are replaced with the fixed claim naming the
  milestone key `extraction-consumer-rewrite` (provenance, house idiom).

Acceptance: the test is green at hand close; the reviewer's diff shows
this test's hunks ONLY (manifest/outcome/tier/generated-list/migrate
assertions byte-identical; the other five m20 tests appear in no hunk);
the m18 parity and m10/m14/m23 suites are green UNEDITED (X2/X3).

### C3 m19 characterization amendment (rider on CW-1)

`tests/integration/tests/m19_optimize_characterization_test.rs`, test
`optimize_writes_all_extracted_headers_and_leaves_consumer_sources_unchanged`
(:110) ONLY. Its name states the bug, so it renames to
`optimize_writes_all_extracted_headers_and_rewrites_consumer_includes`
(name change is a declared amendment; test count and suite count are
unchanged so census-by-name reconciliation stays mechanical). New truth:
the same include flip as C2 (via `EndToEndOptimizer::optimize_repository`
- the shared discovery + shared `apply_with` mean the same one hook fixes
both verbs); `report.committed_transformations == 1`,
`report.affected_consumer_files == 2`, the generated-file list, and every
other test in the file stay byte-identical (X2).

### C4 The faithfulness oracle (crates/cli proof, CW-2)

In `crates/cli/src/commands/ui/extraction_apply/tests.rs`, upgrade
required test 1 (`test_a_valid_token_applies_the_plan_end_to_end`, the
D6 subset-parity clause at :280-300) to the deferred row's equality:

```rust
// expected-modified(consumers, dry_run) :=
//   the approved consumer_files whose on-disk (pre-apply) text contains
//   a directive hit for at least one discovery spelling of that rewrite
//   row's old_includes - computed with the public pieces:
//   plan_consumer_discovery_targets(&registry, plan) probed by the plan
//   backend's rewriter via find_dependency_targets over the consumer text
```

Assertions after the real apply: `modified == expected_modified`, and on
the fixture `expected_modified == approved_consumers` (the deferred row's
"restores equality automatically when the bug is fixed", machine-witnessed
here), keeping every surviving clause: `written == approved_new_files`
field-by-field, `!written.is_empty()`, no-modification-outside-approved,
`committed`, `output_text.ends_with("\nCommitted.")`, epoch/plan round
trip. NEW one-liner from the A-4 observations row: no file present before
is absent after (no deletions). The stale "UNSATISFIABLE / journal D6"
comment block (:280-290) is replaced by the fixed-claim comment naming the
milestone key.

Acceptance: the upgraded test + the no-deletions assertion green at hand
close; required tests 2-11 byte-identical UNEDITED; caller census still 3
(test 11 self-locks); scoped cli suite green; census delta attributed.

### C5 The dated one-liners that arrive at this touch (crates/cli, CW-2)

- `ascii_only(&plan_id)` on `plan_id` in `outcome_dto`
  (`extraction_apply.rs:146` region; the A9 carve-out row's named half;
  the `new_files` lossy-rendering half stays frozen by the field-by-field
  parity with the reader and is OUT).
- The cosmetic doc-comment indentation on `ascii_only` (the cosmetic row;
  line numbers drifted post-UTA split - re-verify at touch, file is now
  295 lines plus `extraction_apply/tests.rs`).
- A small direct unit test: `outcome_dto` with a crafted non-ASCII
  plan_id yields an ASCII `plan_id` field (locks the sanitizer; the e2e
  fixture is ASCII-only so route test 10 alone cannot prove it).

Acceptance: the new test green; scoped cli green; `encoding_guard --all`
exit 0; the two dated rows discharged at closeout with the landing commit.

### C6 The copy-relay hand (crates/ui assets, CW-3)

- `hotspots.js:521-522`: the `renderExtractionApplyHint` comment "the UI
  states the command, it never runs anything" flips to the claim of record
  (comment text only; zero code change).
- `extraction-wizard.js:385-387`: the S4 truncation note's clause "the
  apply rewrites every consumer" becomes the shipped truth, e.g. "the
  apply rewrites every consumer the dry run lists" - it renders only when
  the list truncates at 200, and post-fix that claim holds
  (X1/expected-modified). Copy string only.
- A lock so honesty cannot rot back: a test (the handoff's call between
  `extraction_wizard_render.rs` - which already reads these assets - and
  a sibling) asserting neither asset contains "never runs anything" and
  the truncation note carries the new clause. Nothing today locks either
  sentence (grep-proven at authoring: the only hits are the assets and
  `tasks/**` history), so this is additive locking, no amendment.

Acceptance: scoped ui suite green; the new lock green; every existing
wizard/card lock byte-identical UNEDITED; fetch-site bound survives with
zero new `fetch(` occurrences (X6); `ui_smoke: PASS`;
`theme_token_check --check-added --base` silent; `encoding_guard --all`
exit 0 (the wizard asset carries an ASCII-only lock).

## 4. Adjudicated decision: does symbol-move ingress this milestone?

**No. Defer; the follow-on key `extraction-symbol-move` is registered in
the kickoff (section 5).** The endgame line the wizard wrote - "the
automatic extractor creates the new files AND moves the symbols" - reads
today as: create (shipped), move (shipped for move-capable plugins,
copy-only for C++ BY DESIGN I5, with the dry run honestly reporting
`move_removals == 0` / `copy_only: true` computed via the plugin hook, and
the strips-as-it-writes sentence already shipped for plugins that strip).

Evidence for deferral, not taste:

1. Blast radius. The remaining endgame work on the flagship language is
   `CppDependencyRewriter::remove_top_level_symbols`: parser-span
   removal of top-level declarations across the namespace/class/template/
   macro/multi-line-signature hazard field - the same surface class that
   produced the M25 brace-depth incident. That is a lang-cpp parser
   workstream (with its own fixture set), plus the amendment table for
   every place copy-only is stated or pinned: the trait-default pin,
   the engine's I5 "never removes" semantics, the `COPY_VS_MOVE`
   sentence, the m19/m20 copy assertions (amended a second time), and
   m23's expectations. None of that fits alongside a rewrite fix without
   breaking the sizing law or sneaking a product change into a bug
   milestone.
2. Product decision the user owns. Move-by-default vs opt-in move for a
   language whose whole historical semantics is copy is exactly the kind
   of decision a kickoff puts in front of the user with options; it
   cannot be silently decided inside D6's shape. The dry-run already has
   the reporting spine (`move_removals`, `copy_only`) that the move
   milestone extends, which is why deferral costs nothing later.
3. Sequencing. D6 is a hard prerequisite for C++ move: moving
   declarations out of sources while the consumers' redirect is still a
   vacuous no-op leaves the tree referencing removed symbols with the
   rewrite powerless to help. Landing the rewrite fix first is the
   ordering physics, and this milestone IS that prerequisite.

What apply UX says about copy-only meanwhile (the kickoff's promise):
copy-only remains available and is the C++ behavior; the UI states it
truthfully (`COPY_VS_MOVE_SENTENCE` byte-unchanged, the dry-run's
computed `copy_only` flag drives the line); nothing in this milestone
buries or fudges it.

## 5. Module map (per hand; also the "must not" table)

| Hand | Changes | Touched (read/proof) | Must not change |
|---|---|---|---|
| CW-1 (crates/lang-cpp + declared riders) | `crates/lang-cpp/src/dependency_rewriter.rs` (hook + `dependency_spelling` + unit tests); RIDER 1: m20 test C2 hunks; RIDER 2: m19 test C3 hunks (both inside `tests/integration/tests/`, declared per the A-3 rider precedent) | `crates/validate` (call site unchanged), m10/m14/m18/m23 suites as gates | every other test in m19/m20; `crates/cli/**`, `crates/transform/**`, `crates/validate/**`, `crates/lang/**`, `crates/lang-scala/**`, `tests/fixtures/**` (fixture byte-frozen), `tools/**` |
| CW-2 (crates/cli) | `crates/cli/src/commands/ui/extraction_apply.rs` (C5 one-liners), `crates/cli/src/commands/ui/extraction_apply/tests.rs` (C4 upgrade + no-deletions + C5 test) | the apply route/DTO (proof only) | `transform.rs`, `optimize.rs`, `extraction_reader.rs`, `crates/ui/**`, required tests 2-11, the token/mutex code |
| CW-3 (crates/ui) | `crates/ui/assets/hotspots.js` (comment), `crates/ui/assets/extraction-wizard.js` (note copy), the new lock (one `tests/ui_test` file + its `main.rs` mod line iff the handoff picks a new file - planner/handoff decides; a lock appended to `extraction_wizard_render.rs` avoids the mod line) | assets served proof via `ui_smoke` | every existing lock, `styles.css`, `index.html`, routes/DTOs/state, fetch sites |

Hard edges: CW-1 before CW-2 (the equality asserts fixed behavior) and
CW-1 before CW-3 (the note's truth depends on the fix). House default
strictly serial on main, one scoped landing commit per hand (plus the
journal-rider precedent). CW-2 and CW-3 are file-disjoint but still
serialize per house default.

## 6. Handoff sketch sized to the law (planner input; no sequencing here)

| Hand | Crate dir | Est. produced lines | Est. gates | Kind |
|---|---|---|---|---|
| CW-1 | crates/lang-cpp + 2 declared rider files | ~250-400 (hook + fidelity/unit tests + the two amendment hunks) | ~9 | fix + frozen-lock amendment |
| CW-2 | crates/cli | ~200-330 | ~9 | proof upgrade + dated one-liners |
| CW-3 | crates/ui | ~50-110 | ~6-7 | copy honesty + one lock |

All hands: one crate dir, `<= ~10` distilling-entry-point gate invocations
(including the commit-cycle trio per the standing template row), and
`<= ~1200` produced lines, priced itemized FROM the mandated test bodies
(the M23.5 sizing-arithmetic rule). Every hand's gate list includes:
scoped suite via `testrun --distill`, the named characterization/parity
suites, full census `testrun --distill`, `encoding_guard --all`,
`hygiene_check`, plus per-hand extras (`ui_smoke` for CW-3,
`theme_token_check` for CW-3, `bench_compare` fresh recording for all
three now that the label fix `f20ad35` unblocked it - record WARNs,
attribute nothing, PR-2 persists).

## 7. Risks (named mitigation + oracle per risk)

- **R1 Amending a frozen lock is itself the risk.** The m19/m20 suites
  have survived as byte-identical locks through milestone after milestone;
  loosening them wrongly would hide a real regression behind a rewritten
  expectation. Mitigation: the kickoff section-6 policy - only CW-1's
  implementer, only the two named tests, handoff amendment clause +
  captured Red, reviewer per-commit diff, all set/preview/report
  assertions byte-identical UNEDITED, and the fix's truth independently
  witnessed by the C4 oracle and the C1 fidelity tests (the amendments
  are never the sole evidence). Oracle: reviewer diffs + X1/X2/X3 green.
  Who may amend: the CW-1 hand under its handoff clause, gated by the
  reviewer hop; nobody else, including later hands.
- **R2 Rewritten consumers may not compile** (the generated headers land
  under `generated/` and the new spelling is bare). This is NOT new
  behavior promised today - compile invocation was never tier 3 and
  include-closure is a named separate workstream (D-M23-12) - but the
  fix moves the tree from "consumers untouched" to "consumers rewritten
  to a spelling that may not resolve". Mitigation: the whole apply path
  is dry-run-first, transactional, and rollback-covered; the contract
  states the spelling rule explicitly so a future include-closure
  milestone changes one plugin function, not the pipeline; the kickoff
  registers the follow-on candidate in plain words. Oracle: none -
  stated plainly: compilability is not test-locked here because the repo
  never runs a compiler; the honest-scope claim is the contract.
- **R3 Over-match.** replace-all could touch comment-embedded directives.
  It already does (pre-M18.8b byte parity), and so does C++ discovery -
  the symmetry IS the invariant; C1 test 4 pins the pair on both sides.
  Oracle: X1(a).
- **R4 CRLF / angled spellings drift.** Both sides already handle them
  (`find_dependency_targets` survives CRLF; rewrite handles angled);
  C1 test 4 pins the pair on both. Oracle: X1(a).
- **R5 Collateral to `tool optimize`.** The fix flips optimize's disk
  effect too (same shared machinery) - that is intended symmetry, one
  hook fixes all verbs; the report fields and tier outcomes are locked
  unchanged (m19 report assertions byte-identical; tier 3 still sees the
  path-space `new_includes` it can resolve). Oracle: C3 + m19 green.
- **R6 Scala regression.** Impossible by containment: its spec hook is
  implemented and untouched; every Scala suite and the m17 cone guard are
  UNEDITED gates. Oracle: X4.
- **R7 Wizard honesty gap after the fix.** The dry-run rows keep showing
  path-space spellings while the text change is bare-file-name; the copy
  hand keeps the visible claims true and the display-spelling surfacing
  is registered as a follow-on candidate. Residual risk accepted, stated,
  and cheap to revisit in one DTO row later.
- **R8 Amendment creep beyond the named hunks** (the R7 precedent from
  the predecessor milestone). Mitigation: X8's closed list; any further
  pre-existing-test edit discovered mid-hand is STOP-and-report. Oracle:
  reviewer per-commit diffs.
- **R9 Bench attribution.** `bench_compare` is runnable now; the two
  1.x-ms clone phases WARN under load and vanish idle (machine-class
  sensitivity, PR-2 open). Mitigation: every hand records the fresh run,
  attributes timing only if clean; producer files (`crates/cli/tests`)
  are byte-untouched by all three hands, the standing proof idiom.
  Oracle: the recorded runs + producer zero-diff proofs.

## 8. Hygiene notes

- `crates/lang-cpp/src/dependency_rewriter.rs` 244 -> ~300-330: fine.
- `crates/cli/src/commands/ui/extraction_apply.rs` is 295 lines (UTA split
  the tests into `extraction_apply/tests.rs`, 780 lines): CW-2's growth
  lands in `tests.rs` to ~830-870, under 1000 with headroom but worth a
  margin note; no split needed now, and no hand starts from growth in a
  near-ceiling file. (The predecessor's "999/1000" era ended with the
  UTA split; this shape measured 295/780 fresh.)
- m19 215 -> ~230, m20 268 -> ~290: fine.
- `extraction-wizard.js` 606 (asset, invisible to R2), `hotspots.js` 691:
  both fine; the new ui lock grows `extraction_wizard_render.rs` 851 ->
  ~880-900 or a new file - keep it a single focused module either way.
- No source line anywhere near 2000 chars; all authored content pure
  ASCII; names from the intent-revealing set:
  `consumer_rewrite_spec` (trait hook, inherited), `dependency_spelling`,
  `expected_modified_consumers` (test helper), the renamed m19 test; no
  `tmp`/`res`/`data`/single-letter locals in new code.

## 9. Freeze set and census

Frozen for the whole milestone (beyond the permanent sets:
`tests/golden/**`, `tests/fixtures/**` byte-locked, `UPDATE_GOLDENS=1`
forbidden; `tools/**` operator-law): `crates/cli/src/**` except the two
CW-2 files; all of `crates/transform/**`, `crates/validate/**`,
`crates/lang/**`, `crates/lang-scala/**`, `crates/ui/src/**`; every
wizard/card lock; the m10/m14/m17/m18/m23 suites; the other five m20
tests and the rest of m19; themes/assets beyond the two CW-3 strings;
the bind surface; the boot payload bytes. Census anchor:
`1830 passed / 0 failed / 13 ignored / 119 suites @ 765df7b`; hands
report deltas from a clean-tree scoped baseline in their journals; the
`--distill` census substitute carries the anchor (the `--baseline` FILE
form stays superseded). Expected: CW-1 net zero new tests (two amendments,
two renames of tests, N new unit tests raise the lang-cpp crate suite -
exact counts are the hand's journal duty); CW-2 +1 or +2; CW-3 +1.
