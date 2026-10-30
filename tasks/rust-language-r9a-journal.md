# rust-language R-9a journal - implementer (mechanism hop)

Date: 2026-10-26.
Authority: `tasks/rust-language-r9-amendment-1.md` (the complete R-9a contract)
and `tasks/rust-language-r9-journal.md` sec. 9 (the STOP hand's resume path).
Dispatch base: `f2387f1` (branch `balsam-basil`; predecessor chain
`c52817c` STOP duty-1, `239df72` STOP journal, `f2387f1` amendment + relay).
Successor to the R-9 STOP hand; the dead session was never resumed.

## Section 1 - duty posture

- R-9a duty 1 is DISCHARGED by `c52817c` per amendment sec. 0 ("do not
  re-land"). Nothing was re-landed: this journal's commit is the hop's only
  landing.
- R-9b duty 1 (this hop's verdict + relay) belongs to the next hand.

## Section 2 - scope executed (amendment-governed)

- F1 mechanism: `mod_tree.rs::module_path_of_src_file` honest module
  semantics (`src/lib.rs`/`src/main.rs` -> `crate`; `mod.rs` -> the
  directory's module; every other src-rooted path byte-identical to the R-8
  rule; the non-`src` `None` stance unchanged) plus the payload-derived
  suffix-retarget leg in `dependency_rewriter.rs::consumer_rewrite_spec`
  (prefix-match leg stays byte-identical and tried FIRST; the derivation
  leg answers flat `crate::X` ids from index-aligned payload pairings;
  unequal lengths and multi-segment/foreign-root ids add nothing).
- F3 hook: `move_out_top_level_symbols` overridden in
  `dependency_rewriter.rs` as ONE delegating method; machinery in the new
  `crates/lang-rust/src/move_retention.rs` (helper-band rule). Retention
  form = deletion + declaration insert (Scala parity); `plan_id` ignored;
  refusal taxonomy exactly the amendment's four clauses (empty name batch
  refused by the completed-move clause's vacuous all-quantifiers, no extra
  invented branch).
- NO caps flip, NO planning enrollment, NO validator, NO suite:
  `plugin.rs`, `planning.rs`, `validator.rs`, `constraints.rs` verified
  ZERO-DIFF (git diff vs `5f87944`); `lib.rs` diff is exactly the one
  `mod move_retention;` line (amendment sec. 1c shape). INV-5 holds: the
  R-7 four-true caps literal is byte-identical (plugin.rs in no hunk).
- Touch-set: exactly `dependency_rewriter.rs`, `mod_tree.rs`,
  `move_retention.rs` (new), `lib.rs` (one mod line), this journal. No
  stash used; no branch change; fixtures/goldens/m18-m23/tools untouched.

## Section 3 - RED (authored before any mechanism edit, per amendment sec. 1b)

Digest line (capture: `target/testrun-capture.txt`, later overwritten by the
green/census runs; failure texts are copied verbatim while on disk):
`86 passed / 8 failed / 0 ignored / 1 suites` (a failing lib run skips the
doc suite - distiller shape unchanged from prior hops).

Verbatim `left:` lines for the named reds (all fail for the predicted
semantic reason; a first green-run attempt exposed three plain compile
errors - lifetime/elision/`ItemFn` field - which were harness fixes, not
mechanism edits, and the red stood unchanged after them):

1. `mod_tree::tests::module_path_pins_the_rust_module_semantics` -
   `left: Some("crate::lib")` (the `src/lib.rs -> "crate"` leg; the
   `src/models/mod.rs` leg predicts `Some("crate::models::mod")`, masked by
   the first assert, per the amendment's "the named failures above").
2. `dependency_rewriter::tests::derived_spec_retargets_the_flat_id_from_the_root_parent` -
   `left: ["lib"]` (exactly the amendment's predicted vector).
3. `dependency_rewriter::tests::derived_spec_retargets_the_flat_id_from_a_mod_rs_parent` -
   `left: ["mod"]`.
4. `dependency_rewriter::tests::derived_spelling_never_touches_a_colliding_flat_name` -
   `left: "use crate::a::m::Thing;\nuse crate::b::m::Thing;\n"` (no rewrite fires).
5. `move_retention::tests::move_out_removes_the_item_and_declares_the_module_once` -
   `expect("a locatable top-level fn with a mod anchor must be retained by
   deletion")` panicked: the trait default delegates to
   `remove_top_level_symbols` and answers `None`.
6. `move_retention::tests::move_out_reinsert_is_idempotent` -
   `expect("the clean variant moves")` panicked (same default-`None` reason).
7. `move_retention::tests::move_out_refuses_unparseable_or_unlocatable` -
   `assert!(move_out(clean, &["marker"]).is_some(), "the paired clean
   variant moves (red non-vacuity)")` failed.
8. `move_retention::tests::move_out_refuses_module_ident_and_already_moved` -
   same paired-clean-variant failure.

Green-at-birth witnesses that passed inside the same red run (justified per
SM-3's clause, amendment sec. 1b item 5 and the two added pins):
`derived_leg_refuses_multisegment_ids_and_foreign_roots`,
`derived_leg_refuses_unequal_payload_pairing`,
`lib_and_mod_stems_are_inert_mod_space_anchors`.

## Section 4 - GREEN

Crate gate: `94 passed / 0 failed / 0 ignored / 2 suites` - baseline `83` +
N = 11 new tests, zero retirements (amendment arithmetic `83 + N`).
N attribution: mod_tree 1 (module-path pin), dependency_rewriter 6 (3 named
derived reds + 3 green-at-birth characterization pins), move_retention 4
(named hook reds).
All 29 R-8 in-crate witnesses (17 rewriter + 12 extractor) ran UNEDITED and
green inside this gate - no test byte of theirs was touched (see the
zero-deletion audit below).

## Section 5 - RE-GATE audit (amendment sec. 1c)

- `git diff 5f87944..worktree -- crates/lang-rust/src/dependency_rewriter.rs`:
  the deleted-line census is 8 content lines, ALL production-side: 4 docblock
  lines (the "retention is its own later hop" hook note, honestly reworded -
  no stale prose), 2 loop-body lines (the prefix-match leg rewrapped into a
  match with the derivation fallback; retarget_id still tried FIRST,
  behavior unchanged - the 17 witnesses are the proof), 2
  `line_terminator` doc/signature lines (widened to `pub(crate)` so the
  hook module rides ONE CRLF discipline). ZERO deleted or modified lines
  inside the existing 17 test functions: their diff is a pure test-module
  APPEND.
- `extractor.rs`, `plugin.rs`, `validator.rs`, `constraints.rs`,
  `planning.rs` (nonexistent, R-9b): ZERO-DIFF; `lib.rs`: exactly one mod
  line (git diff audit above; capture `target/distill-capture.txt`).
- Neutral seam: `crates/lang/src/rewriter.rs` untouched (never opened for
  edit; `git status` shows the crate clean).
- Delegation proof: gate 4 `-p ariadne-validate --test
  move_removal_transaction_test` UNEDITED green `2 passed / 0 failed / 0
  ignored / 1 suites`.
- m23 mirror: gate 5 `-p ariadne-integration-tests --test
  m23_symbol_isolation_test` UNEDITED green `7 passed / 0 failed / 0
  ignored / 1 suites`.

## Section 6 - gates run (verbatim digests, distilling entry points, 600000 ms)

1. `python tools/testrun.py --distill -p ariadne-lang-rust` - baseline:
   `83 passed / 0 failed / 0 ignored / 2 suites` (verbatim, matches the
   amendment's declared baseline).
2. same - RED: `86 passed / 8 failed / 0 ignored / 1 suites`, exactly the 8
   named failures (sec. 3), capture `target/testrun-capture.txt`.
3. same - GREEN: `94 passed / 0 failed / 0 ignored / 2 suites`.
4. `python tools/testrun.py --distill -p ariadne-validate --test
   move_removal_transaction_test` - `2 passed / 0 failed / 0 ignored / 1
   suites` UNEDITED.
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m23_symbol_isolation_test` - `7 passed / 0 failed / 0 ignored / 1
   suites` UNEDITED.
6. `python tools/testrun.py --distill` - census
   `1981 passed / 0 failed / 13 ignored / 125 suites` = anchor
   `1970 / 0 / 13 / 125 @ 5f87944` + 11, every gained test attributed by
   name in sec. 4. Zero failures: the four caps locks stayed GREEN (no flip
   this hop, as the amendment demands).
7. `python tools/hygiene_check.py` - exit 0; output is R3 warn-only and ALL
   entries are preexisting in other crates (zero R1/R2 lines, zero
   lang-rust entries). File ceilings: `dependency_rewriter.rs` 899,
   `mod_tree.rs` 271, `move_retention.rs` 321, `lib.rs` 46 - all < 1000;
   every new function < 100 lines; no source line near 2000 chars.
8. Info-only lint probe: `python tools/distill.py -- cargo clippy -p
   ariadne-lang-rust --all-targets -- -D warnings` - FAILS AT THE BASELINE
   TOO (preexisting lints in untouched files, e.g. `resolver.rs:286`), so
   clippy -D is not this repo's gate. New files show only style suggestions
   (doc-list indentation, the `.then_some(..).unwrap_or_default()` chain
   that mirrors R-8's landed style verbatim); zero substantive findings in
   the new code. Recorded as an observation, nothing chased.
9. Encoding: `python tools/encoding_guard.py --all` - exit 0 (silent). The
   new untracked file is additionally proven pure ASCII by a byte-scan grep
   (no byte outside 0x00-0x7F). After staging via `commit_scoped.py` (the
   tool owns staging; a manual `git add` window is not how this repo
   stages), `--staged` was re-run; see sec. 8 for its shape.
   All authored bytes pure ASCII (the multibyte specimens ride `\u{}`
   escapes - no literal introduced).

## Section 7 - deliberately NOT done

- No caps-literal touch, no hook/validator/planning work: R-9b's per the
  amendment split (sec. 0); the amendment's R-9a touch-set is honored
  literally.
- No F2 `constraints.rs` edit and no F5 suite test: both are assigned to
  R-9b by amendment sec. 2 and sec. 5, and neither file is in R-9a's hard
  touch-set (see deviation D1 for the dispatch-wording conflict).
- No test byte outside my touch-set edited, none superseded (amendment sec.
  9 R-9a row: NOTHING superseded); no rider-band file opened.
- The superseded-witness discipline: nothing turned red unedited - no STOP
  condition arose; `CURRENT_ANALYSIS_VERSION` untouched (plugin.rs
  zero-diff), so 2.2.0 stands.
- Existing clippy lints in untouched files were not "cleaned up" (out of
  scope, would grow hunks outside the amendment's shape).

## Section 8 - deviations with reasons

- D1 (scope adjudication): the dispatch's scope line listed "the
  constraints freeze (F2)" and the F5 named test under R-9a, but the
  amendment - the governing contract the dispatch itself names ("follow
  ONLY its R-9a assignment") - assigns F2 to R-9b's `constraints.rs`
  (sec. 2: "ADDED to R-9b's touch-set") and the F5 test to R-9b's suite
  (sec. 5), and neither file is in R-9a's hard touch-set (sec. 3). The
  amendment's split section was followed; F2/F5 were NOT done here. No
  silent narrowing: this line is the record.
- D2 (named red 5 placement): amendment sec. 1b item 5 says the
  multi-segment refusal is characterized "on a root-parent payload", green
  at birth. At green, the byte-identical prefix-match leg (which sec. 1a(2)
  commands to stay and to be tried FIRST) pairs `crate::inner::Helper` on a
  root-parent payload - so pinning "no Helper pair at all" would fail the
  green and contradict the first-wins rule. The named test therefore pins
  (a) the foreign-root `CoProposed` drop on the root parent (byte-stable
  across the mechanism, UNEDITED clause), (b) that any Helper pairing
  present is the prefix leg's verbatim-id pair (the derivation leg
  contributes no rewritten spelling), and (c) the no-pair refusal where the
  prefix leg cannot fire at all (a `mod.rs` parent) - the only placement
  where the named refusal is observable. Red-5's name, green-at-birth
  status, and refusal semantics are preserved; nothing weakened: the
  derivation leg provably takes flat ids only.
- D3 (two extra green-at-birth pins): `derived_leg_refuses_unequal_payload_
  pairing` (sec. 1a(2)'s "unequal list lengths -> the derivation leg adds
  nothing" clause had no named witness) and
  `lib_and_mod_stems_are_inert_mod_space_anchors` (sec. 1a(1)'s "pinned"
  requirement) were added as green-at-birth characterizations under the
  same SM-3 clause that justifies named red 5. N grew from the amendment's
  implicit 9 witnesses to 11; the green/census deltas above attribute them.
- D4 (harness fixes between red and green): the first green attempt died on
  three plain compile errors (lifetime elision, `ItemFn.sig.ident`,
  `&Item` vs `&ItemMod` binding). The red run itself had compiled and
  failed for the named semantic reasons before those mechanism edits
  existed; no red expectation was altered.
- D5 (red-run suite count): the failing run reports `1 suites` (the doc
  suite does not run when the lib target fails); baseline and green report
  `2 suites`. Preexisting distiller behavior, not a change in shape.
- D6 (encoding coverage honesty): `encoding_guard --all` covers tracked
  content; the new `move_retention.rs` was untracked at check time, so it
  is additionally proven pure ASCII by a direct byte scan; after the
  scoped commit the `--staged` re-run is vacuously clean (empty staging
  area - `commit_scoped.py` owns staging and commits in one step).
- D7 (bootstrap artifact left untracked):
  `tasks/rust-language-r9-handoff-resume.md` (written by the checkpoint
  bootstrap) is outside the amendment's commit scopes and was left
  untracked; the STOP hand's precedent chain did not land its resume
  report either.

## Section 9 - disk-state inventory at landing

- Landed: one scoped commit (see the commit list in the report-back); files:
  the 4 touch-set sources + this journal.
- Working tree after commit: only the untracked bootstrap report (D7).
- No stash; no branch change; nothing outside `crates/lang-rust` and this
  journal modified.

## Section 10 - notes for R-9b (information hand-off, no action taken)

- With the honest module semantics landed, a wired payload whose
  `old_includes` names `src/lib.rs` makes `crate` a source module, so FLAT
  ids on such payloads retarget through the prefix-match leg; the
  derivation leg is the workhorse for `mod.rs` and sub-directory parents.
  Both routes emit the SAME spellings the amendment pins
  (`crate::models::models_marker -> crate::models::models_marker::
  models_marker`), so the suite's apply assertions can pin those bytes.
- The suite's grown-oracle parent-MODIFIED assert for the root-file class
  now has a real producer: this hop's `move_out` hook. The F2 freeze
  (R-9b) must still keep Module proposals out of the plans.
- The m23-mirror apply assertion is satisfiable VERBATIM on this tree
  (option (b) landed); R-9b must not re-word it.
