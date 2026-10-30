# rust-language R-9a verdict - reviewer (gate hop R-9a, mechanism)

Date: 2026-10-26. Reviewer: fresh session, worktree `balsam-basil`.
Governing contract: `tasks/rust-language-r9-amendment-1.md` (sec. 1 mechanism
+ sec. 1c re-gate are law for this hop). Context: `tasks/rust-language-r9-
handoff.md`, `tasks/rust-language-r9-journal.md` (STOP record),
`tasks/rust-language-r9a-journal.md` (this hop's journal).

## Verdict: APPROVED

Chain check: base `f2387f1` (AMENDMENT-1 + relay), single landing `fcb078c`
directly on top. Landing files match the touch-set exactly (stat:
`dependency_rewriter.rs` +283/-8, `mod_tree.rs` +74/-8,
`move_retention.rs` NEW 321, `lib.rs` +1 `mod move_retention;`,
r9a-journal NEW 244). The dispatch relay's "+291/-9, +82/-8" were stat-total
phrasings; the actual numstat is above and satisfies every clause measured
against it.

All claims verified personally; every gate re-run fresh. Canonical lines:

- Crate gate (amendment gate 3, green): `94 passed / 0 failed / 0 ignored / 2 suites`
- R-8 witnesses UNEDITED, run by name: rewriter module `23 passed / 0 failed / 0 ignored / 1 suites` (17 R-8 witnesses green, zero bytes changed - pure-append proof below); extractor module `12 passed / 0 failed / 0 ignored / 1 suites` (file zero-diff). 17 + 12 = 29 witnesses, green, unedited.
- Delegation proof (gate 4): `2 passed / 0 failed / 0 ignored / 1 suites`
- m23 mirror UNEDITED (gate 5): `7 passed / 0 failed / 0 ignored / 1 suites`
- FRESH CENSUS (gate 6, NEW ANCHOR): `1981 passed / 0 failed / 13 ignored / 125 suites` = anchor `1970 / 0 / 13 / 125 @ 5f87944` + 11, suites flat 125, ignored flat 13, every gained test attributed by name (below). Verbatim expected-vs-actual match.
- Hygiene: exit 0, output R3 warn-only, zero lang-rust entries.
- Encoding `--all`: exit 0 (silent) with `move_retention.rs` now tracked - the new file's ASCII purity is directly gate-proven at HEAD (journal D6 honesty accepted and now superseded by the tracked-state proof).
- Tools: `Ran 296 tests in 31.215s` / `OK`.
- Bench (kickoff sec. 4 posture, via distiller): `### generic PASS (exit 0)`; `bench compare: 3 run(s), no regressions past 1.25x baseline`.

11-test attribution (by name, at source): mod_tree 1
(`module_path_pins_the_rust_module_semantics`); dependency_rewriter 6
(`derived_spec_retargets_the_flat_id_from_the_root_parent`,
`derived_spec_retargets_the_flat_id_from_a_mod_rs_parent`,
`derived_spelling_never_touches_a_colliding_flat_name`,
`derived_leg_refuses_multisegment_ids_and_foreign_roots`,
`derived_leg_refuses_unequal_payload_pairing`,
`lib_and_mod_stems_are_inert_mod_space_anchors`); move_retention 4
(`move_out_removes_the_item_and_declares_the_module_once`,
`move_out_reinsert_is_idempotent`, `move_out_refuses_unparseable_or_unlocatable`,
`move_out_refuses_module_ident_and_already_moved`). Green arithmetic
83 + 11 = 94 matches; census 1970 + 11 = 1981 matches.

## Re-gate clause (amendment sec. 1c) - PASSES, verified personally

- `git diff 5f87944..fcb078c -- crates/lang-rust/src/dependency_rewriter.rs`:
  deleted-line census = exactly 8 content lines, ALL production-side:
  4 docblock lines (the stale "retention is its own later hop" hook note,
  honestly reworded - no stale prose), 2 loop-body lines (prefix-match leg
  rewrapped into a match arm with the derivation fallback; `retarget_id`
  still tried FIRST), 2 `line_terminator` doc/signature lines (widened to
  `pub(crate)`). ZERO deleted or modified lines inside the existing 17 test
  functions: the final hunk (`@@ -614,4 +684,209 @@`) is a pure test-module
  APPEND - no `-` lines past `#[cfg(test)]`.
- `extractor.rs`, `plugin.rs`, `validator.rs`, `constraints.rs` ZERO-DIFF from
  `5f87944`; `lib.rs` diff is exactly one `mod` line. `extractor.rs` in no
  hunk. Caps posture: plugin.rs byte-identical => the R-7 four-true literal
  is byte-identical (INV-5) and the four integration caps locks stayed GREEN
  (census 0 failed) - no flip this hop, as the amendment demands.
- Neutral seam: `crates/lang/src/rewriter.rs` zero-diff (range name-diff
  empty across lang, lang-cpp, lang-scala, index, pipeline, cli, repo,
  cluster, validate, graph, tests, tools); `move_removal_transaction_test`
  and `m23_symbol_isolation_test` green UNEDITED (lines above).
- `Cargo.toml`/`Cargo.lock` untouched; `refs/stash` absent
  (`git rev-parse --verify refs/stash` -> `fatal: Needed a single revision`).

## Mechanism audit (plan-literal, amendment sec. 1a/3)

- `module_path_of_src_file`: `src/lib.rs`/`src/main.rs` -> `Some("crate")`;
  `mod.rs` -> the directory's module; every other src-rooted path byte-
  identical to the R-8 rule; non-src -> `None` unchanged. Landed pin's red
  witness matched the amendment's predicted `left: Some("crate::lib")`;
  the other named reds matched their predicted vectors verbatim
  (`left: ["lib"]`, `left: ["mod"]`, collision no-rewrite text, the four
  hook `None`-default panics).
- `consumer_rewrite_spec`: prefix-match leg runs first and keeps single-pair
  push behavior byte-identically; the derivation leg answers ONLY flat
  `crate::X` ids (single-segment remainder; multi-segment/foreign roots
  structurally cannot reach it), refuses unequal `old_includes`/
  `new_includes` lengths, and emits `M_old::X` -> `M_new::X` with the
  crate-root spelling `crate::X`. Matches key on the FULL derived spelling
  under unchanged `leaf_affected_by` leaf equality - the collision pin
  demonstrates a `crate::Thing` in `src/b/m.rs` staying byte-identical while
  the `src/a/m.rs` payload rewrites. Refusals (foreign-root drop, inert
  mod-space anchors) unchanged; `consumer_rewrite_spec_drops_ids_outside_
  the_source_module` survives UNEDITED.
- Move-retention hook (sec. 3): exactly ONE delegating method in
  `dependency_rewriter.rs`; machinery in `move_retention.rs` (helper-band
  rule, R-8 mod_tree precedent). Retention = deletion + declaration insert
  (Scala parity); `plan_id` ignored; the four named refusals land exactly
  (unparseable, unlocatable-without-declaration, mod-ident structure,
  completed-move re-offer -> `None` = byte-preserving idempotence); empty
  batch refused by the vacuous all-quantifiers, no invented branch. Stem
  grammar rides the single-source `sanitize_module_stem`; insert anchored
  after the LAST out-of-line `mod`, visibility/indent copied, file-private
  at EOF when none, CRLF-aware via the shared `line_terminator`; already-
  declared -> no second insert. Attr-bearing and CRLF specimens pinned
  (RISK-R9-HOOK-SPICE oracle). Deletions skip `Item::Mod` (double defense).
  No retained dual representation: the old naive stem rule is gone, one
  `module_spelling` source of truth.

## F1 FINAL WORD (for R-9b consumption)

YES - the landed mechanism makes the handoff's verbatim m23-mirror
consumer-rewrite apply assertion satisfiable. The amendment sec. 7 route is
producible and byte-pinned at the mechanism level:
`[src/lib.rs] -> [src/fixture_marker.rs]` with `crate::fixture_marker`
emits `crate::fixture_marker -> crate::fixture_marker::fixture_marker`;
`[src/models/mod.rs] -> [src/models/models_marker.rs]` with
`crate::models_marker` emits `crate::models::models_marker ->
crate::models::models_marker::models_marker` - exactly the bytes the suite's
apply stage must assert ("consumer contains the new import, not the old";
the segment-repeat spelling is pinned, not special-cased away). The collision
pin proves the rewrite reaches the consumer line and nothing else, and both
legs (prefix-first on root-parent payloads, derivation on `mod.rs` parents)
emit the SAME spellings (journal sec. 10 cross-checked against the code).
R-9b must consume these bytes and must NOT re-word the assertion.

## Deviation dispositions

- D1 (F2/F5 deferred to R-9b): ACCEPTED - CORRECT-BY-AMENDMENT. The
  amendment is law for this hop and says it three times: sec. 0 gives R-9b
  "the F2 freeze, the symbol-isolation suite"; sec. 2 marks the
  `constraints.rs` contract "ADDED to R-9b's touch-set"; sec. 5 names
  `flat_id_shared_mod_and_fn_record_is_planning_frozen` "in R-9b's suite".
  Neither `constraints.rs` nor any integration suite path is in R-9a's hard
  touch-set (sec. 3), so doing F2/F5 here would have BROKEN the touch-set.
  The dispatch brief's scope line was wrong against the amendment; the
  implementer followed the governing document. Recorded, no action.
- D2 (named red-5 placement): ACCEPTED. The literal pin "no Helper pair at
  all on a root-parent payload" is unsatisfiable at green by the amendment's
  OWN clauses: sec. 1a(2) commands the byte-identical prefix-match leg to
  stay and be tried FIRST, and with honest `crate` module paths that leg
  pairs `crate::inner::Helper` on a root-parent payload (first-wins). The
  landed test preserves the named refusal's semantics without weakening:
  (a) foreign-root `CoProposed` dropped (the UNEDITED clause), (b) any
  Helper pairing present is the prefix leg's verbatim-id pair - the
  derivation leg provably contributes no rewritten spelling (code: the
  single-segment gate makes any other contribution impossible), (c) total
  no-pair refusal on a `mod.rs` parent, the only placement where the
  derivation leg is the sole candidate. Name, green-at-birth status, and
  refusal meaning all preserved.
- D3 (two green-at-birth pins): ACCEPTED. Each pins a clause the amendment
  itself demanded witnesses for but named none: the unequal-lengths refusal
  (sec. 1a(2)) and the "inert `lib`/`mod` anchors - pinned" requirement
  (sec. 1a(1)). Honest characterization of always-true structural
  invariants under the same SM-3 clause that justifies named red 5 - not
  slop. N grew 9 -> 11 and the green/census arithmetic attributes both.
- D4 (three compile-harness fixes between red and green): ACCEPTED. The red
  run compiled and failed for the named semantic reasons before any
  mechanism edit existed; the journal's verbatim `left:` lines match the
  amendment's predicted vectors; the fixes (lifetime elision,
  `ItemFn.sig.ident`, `&Item` binding) altered no red expectation, and the
  landing's pure-append test-region diff shows no red-to-green test rewrite.
- D5 (red run reports `1 suites`): ACCEPTED as recorded - preexisting
  distiller shape (a failing lib target skips the doc suite); baseline and
  green both report `2 suites` (verified in my runs).
- D6 (byte-scan for the untracked new file): ACCEPTED; now moot - the file
  is tracked at HEAD and `encoding_guard --all` exit 0 gate-proves it.
- D7 (resume artifact left untracked): ACCEPTED; disposition below.

## Findings

1. (minor, evidence) Journal sec. 6.7 overstates two file ceilings:
   `dependency_rewriter.rs` is 892 lines (journal: 899) and `mod_tree.rs` is
   261 (journal: 271); `move_retention.rs` 321 and `lib.rs` 46 are exact.
   The < 1000 ceiling holds on actual bytes and hygiene_check exit 0 is the
   operative gate; no action - recorded so the number in circulation is the
   measured one.
2. (informational, recorded here per instruction - out-of-scope observation)
   `cargo clippy -p ariadne-lang-rust --all-targets -- -D warnings` FAILS at
   BASELINE: 7 errors land in the untouched `crates/lang-rust/src/
   resolver.rs` (lines 6-12, 286), so the `-D warnings` posture - not this
   hop - is what fails; `dependency_rewriter.rs:308`/`:393` sit in unedited
   R-8-era code (preexisting at baseline). Style-level lints DO point at new
   R-9a bytes (doc-list indentation in the new docblocks; the
   `.then_some(..).unwrap_or_default()` chain verbatim-copied from the
   landed R-8 idiom; new-test style) - zero substantive findings, journal
   disclosed them accurately. Warn-only posture per the ledger graduation
   rows; nothing chased; NOT a blocker.

## Resume-artifact disposition (SM resume-artifact precedent)

`tasks/rust-language-r9-handoff-resume.md` (untracked, 25 lines) is the
checkpoint_guard bootstrap report from the `f2387f1` dispatch boot (verified
by content). DISPOSITION: DELETE at verdict-landing - it is outside every
commit scope and must never be committed. R-9b duty 1 owns the deletion
(this reviewer cannot and does not touch it).

## Relay for R-9b duty 1 (landing this verdict + ledger row)

- Ledger relay row (draft, verbatim): `R-9a LANDED fcb078c APPROVED
  2026-10-26, census anchor 1981 passed / 0 failed / 13 ignored / 125 suites
  @ fcb078c, R-9b dispatched (hooks + validator + F2 freeze + all-true flip
  + suite hop; four caps locks ride the architect rider).`
- R-9b must NOT re-land: duty 1 `c52817c` (R-8 verdict + relay), the
  amendment commit `f2387f1`, or the R-9a mechanism `fcb078c`
  (`dependency_rewriter.rs` derivation leg, `mod_tree.rs` semantics,
  `move_retention.rs`, the one `lib.rs` mod line). Do not re-edit the 29
  R-8 witnesses or the 11 R-9a pins; consume their pinned spellings.
- R-9b suite: apply bytes are pinned by the mechanism (F1 word above);
  grown-oracle parent-MODIFIED assert now has a real producer (the
  `move_out` hook) - keep it non-vacuous (names the parent path AND the
  inserted line); do not re-word the m23-mirror assertion.
- Delete `tasks/rust-language-r9-handoff-resume.md` at landing.
- FOUR-LOCK RIDER REMINDER (orchestrator's commit, amendment sec. 4): the
  all-true flip supersedes exactly four integration locks
  (`rust_language_discovery_test.rs::capabilities_lock_the_registration_
  flip_literal`, `rust_language_semantic_index_test.rs::
  capabilities_lock_the_semantic_index_flip_literal`,
  `rust_language_dependency_graph_test.rs::
  capabilities_lock_the_dependency_graph_flip_literal`, and
  `rust_language_load_analysis_test.rs::capabilities_lock_the_load_metric_
  flip_literal` whose hunk also changes the true-count `4 -> 8`). Rider lands AFTER R-9b's
  landing and BEFORE the R-9b reviewer's fresh census. R-9b's own census
  runs no-fail-fast and names EXACTLY those four designed reds; the fresh
  bare census after the rider must show `126 suites / 0 failed`. The
  `load_metric` bit must NOT re-appear in the plugin.rs caps hunk (R-7
  owns it; bit-count audit: exactly four bits gained).

## Refused / not-run (explicit)

- `python tools/testrun.py --baseline` (bare): refused - `usage error:
  missing value for --baseline` (the flag takes a FILE; no baseline file
  exists for this track, same disposition as the R-8 addendum). The
  governing fresh census ran in the amendment gate-6 form
  `python tools/testrun.py --distill` and its line matches the expected
  verbatim.
- Catch-all-deny refusals (verbatim `{"permission":"bash","pattern":"*",
  "action":"deny","source":"global"}`): `git stash list` (substituted by the
  strictly-stronger `refs/stash` rev-parse check), `git grep -c` on blobs
  (substituted by working-tree test inventory + the pure-append diff proof),
  inline `python -c` helper probes, a temp-dir script Write (agent edit
  grant: `{"permission":"edit","pattern":"*","action":"deny",
  "source":"agent"}`; substituted by direct `git diff` reading), and a
  `Get-Content` line-count probe (substituted by Read-tool line totals).
  Multi-filter `testrun -- mod_a::tests mod_b::tests` hit parse DRIFT
  (exit 2, tool-shape note); the single-filter form ran the witness proofs.
- `python tools/ui_smoke.py`: not required by this hop (no UI/runtime
  bytes); NOT run. `tools/golden_guard.py`: not landed; the handoff's prose
  byte-checks were used (name-diff + diff-stat proofs above).
