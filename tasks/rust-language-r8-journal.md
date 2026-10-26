# R-8 Journal - rust-language: real extractor + real dependency rewriter + the mod-tree mechanism

Milestone: `rust-language` (unregistered post-M28 track, NO M-number). Hand 8
of 10, serial. Handoff of record: `tasks/rust-language-r8-handoff.md` (read in
full first). Start rev: `9018f40` (dispatch head).

## Duty 1

- Landed `53afb19` - `tasks/rust-language-r7-verdict.md` (+223, the R-7
  verdict APPROVED verbatim as authored by the reviewer) + the ledger relay
  clause from verdict sec. 9 landed verbatim into the `rust-language` row:
  "R-7 LANDED 4fd40ca (+rider a659465) APPROVED 2026-10-26, census anchor
  1943 passed / 0 failed / 13 ignored / 125 suites @ a659465, R-8 dispatched."
- Resume-artifact disposition (verdict sec. 2): untracked
  `tasks/rust-language-r7-handoff-resume.md` DELETED (not landed), per the
  SM-precedent disposition routed to this duty-1 band. Disposal verified
  ("removed tasks/rust-language-r7-handoff-resume.md").
- Memory mirror `ledger.milestone.rust-language` updated same session.

## Gates (canonical lines verbatim; capture = target/testrun-capture.txt unless noted)

1. Baseline `-p ariadne-lang-rust`: `56 passed / 0 failed / 0 ignored / 2 suites`
   (capture 69 lines) @ `53afb19`.
2. Red `-p ariadne-lang-rust`: `62 passed / 21 failed / 0 ignored / 1 suites`
   (capture 250 lines). The single-suite count is the doctest target not yet
   run while unit tests fail; it returns to the baseline's 2 at green.
   Red list (21 named, all for the predicted semantic reasons):
   - extractor (8, against the header-only stub / trait defaults):
     extract_renders_header_use_block_and_item_text_byte_exact,
     empty_deps_omit_the_use_block_and_its_separator,
     deps_render_sorted_deduped_and_verbatim,
     generated_source_reparses_with_syn_and_carries_the_item,
     locate_symbol_bodies_prints_written_items_that_reparse,
     locate_of_the_macro_specimen_yields_written_items_only,
     generated_units_place_in_place_for_the_mod_tree,
     extractor_emits_real_rust_bytes_not_the_deferral_stub.
   - rewriter (13, against the identity stub / trait defaults):
     consumer_rewrite_spec_maps_the_use_space_and_the_mod_space,
     consumer_rewrite_spec_drops_ids_outside_the_source_module,
     use_path_maps_before_after_literals,
     every_occurrence_of_the_old_path_is_rewritten,
     grouped_survivors_keep_their_renames_and_the_new_dep_lands_at_the_slot,
     glob_narrows_on_the_aggregate_old_dep_never_on_a_symbol_dep,
     old_dep_with_empty_new_deps_deletes_the_statement_line,
     parent_gets_the_sibling_mod_declared_once_in_the_anchor_style,
     parent_mod_insertion_is_idempotent_noop,
     use_consumer_without_the_mod_anchor_never_receives_an_insertion,
     discovery_targets_are_the_moved_paths_plus_the_source_module_stem,
     find_dependency_targets_hits_use_leaves_globs_and_mod_declarations,
     commented_quoted_and_sibling_spellings_are_never_hits.
3. Green `-p ariadne-lang-rust`: `83 passed / 0 failed / 0 ignored / 2 suites`
   (capture 97 lines). Arithmetic: 56 baseline - 2 retired stub pins
   (`extract_emits_only_the_deferral_header`, `rewrite_is_the_honest_identity`
   - the stubs' own docs named R-8 as their retirement hand; Scala's
   `plugin_extractor_emits_real_scala_bytes_not_the_stub` precedent replaced
   them with real-bytes pins) + 29 new in-src tests = 83. The caps pin
   `capabilities_match_the_load_metric_flip_literal` ran green in this suite
   UNEDITED. Two pre-green runs were compiler DRIFT (exit 2, capture 58 and
   30 lines): compile-level harness fixes only (see D2/D3), no semantic
   change, no assertion weakened.
4. Delegation proof `-p ariadne-validate --test move_removal_transaction_test`:
   `2 passed / 0 failed / 0 ignored / 1 suites` (capture 11 lines) - UNEDITED
   green; the overrides resolve through the existing seam.
5. Full census `python tools/testrun.py --distill`:
   `1970 passed / 0 failed / 13 ignored / 125 suites` (capture 3394 lines).
   Anchor `1943 / 0 / 13 / 125 @ a659465`: +27 passed = the 29 new in-src
   tests minus the 2 retired stub pins (all in `ariadne-lang-rust`'s
   in-crate suite); ignored flat 13; SUITES FLAT at 125 explicitly - no new
   integration file (the apply-suite is R-9's; this hop proves components
   in-crate only).
6. Hygiene `python tools/hygiene_check.py`: PASS exit 0 (distiller witness
   `### generic PASS (exit 0)`, capture 173 lines; every printed row is a
   pre-existing warn-only R3 in crates this hand did not touch; zero R1/R2).

## RISK-MODTREE mechanism - seam analysis (STOP-check FIRST, then build)

Read the drivers before designing: `crates/cli/src/commands/transform.rs`
`plan_consumer_discovery_targets`/`plan_target_consumers` (discovery =
`consumer_discovery_targets` spellings probed through
`find_dependency_targets` per file) and `crates/validate/src/lib.rs`
`write_all_targets` (rewrite = `consumer_rewrite_spec` payload applied by
`rewrite` per consumer file). Both consume ONLY existing trait hooks - the
planner ruling holds and the STOP clause does NOT fire:

- `consumer_discovery_targets` override: probes = the moved units' module
  ids (use-space) + the source file's sanitized module stem (mod-space - the
  parent that declares `mod <source-stem>;` is exactly the file that must
  gain the sibling declaration; the probe makes it a consumer-side target).
- `find_dependency_targets` override: real syn-scanned directive hits only -
  `use` leaves (plain/grouped/glob-under-base, never comments or strings)
  and out-of-line `mod <name>;` declarations (the mod-space probe).
- `consumer_rewrite_spec` override: maps old_includes/new_includes/
  moved_module_ids into a dual-space RewriteSpec - use-space old/new module
  paths for `use x::y::Z;` consumers + the bare (mod-space) old source stem
  and new target stems. Ids under no source module are dropped (never
  invent an edge the spec does not name).
- `rewrite` override: (a) maps affected `use` statements' leaves -
  survivors re-render one leaf per line with renames kept, new deps land at
  the first matched slot (Scala slot discipline; each new dep at most once
  per statement), empty new list deletes the statement line; (b) MOD-TREE
  insert - for every bare new spelling NOT yet declared in the file, grafts
  `mod <sanitized>;` (visibility + indent copied from the anchored
  `mod <old-stem>;` declaration) - anchored ONLY at a declared old spelling,
  so `use`-consumers without the anchor never receive an insertion (never
  corrupt); IDEMPOTENT: already-declared => the whole mod-space pass is a
  no-op, bytes unchanged (pinned).
- No trait edit needed: all four hooks are existing seam members; the five
  trait-default pins and the `move_out_top_level_symbols` default stay
  UNEDITED (not overridden - retention stays the trait default until its
  own hand; `render_diff`/`emit_file_preamble` stay at neutral defaults).

Honest scope note for the R-9 planner (NOT a silent narrowing, reported):
the consumer-hook mechanism reaches the parent that DECLARES the source
module (the sub-directory/`mod.rs` sibling class). When the extraction
source IS the module-tree parent (root file keeps its items, e.g.
`src/lib.rs` -> new `src/widgets.rs` beside it), no `mod <source-stem>;`
anchor exists for the probe, and that insert rides the move-retention hook
(`move_out_top_level_symbols`), which is R-9's hooks hop - the RISK-MODTREE
honest-fallback wording ("items whose parent already declares the module")
matches the class the consumer mechanism covers. R-9's apply suite decides
the end-to-end routing.

## Deviations (all reported, none silently absorbed)

- D1 - printing seam via `syn::__private::ToTokens`: the handoff names
  "proc_macro2::TokenStream to-string via syn printing"; `quote` is NOT a
  direct dependency of `ariadne-lang-rust` and `crates/lang-rust/Cargo.toml`
  is outside the touch-set (Cargo.lock-unchanged law). The printing trait is
  reached through syn's `#[doc(hidden)] pub` alias module (the same path
  syn's own macros re-export under its default `printing` feature) - no
  manifest change, mechanism as named. Planner/reviewer may prefer a future
  direct `quote` dep rider; reported for the record.
- D2 - two pre-green compile fixes (compiler DRIFT, not semantic red):
  `syn::Visibility` field shape (public-token `is_some` is syn-internal ->
  `matches!(vis, Visibility::Public(_))`) and `UseRename.rename` (field name).
  The first Red-to-green assertion change: the find-test's exact-name probe
  legitimately hits BOTH the named leaf statement and the wildcard under its
  base (Scala `statement_mentions` parity) - the test was authored
  under-specified in Red; the green assertion STRENGTHENED to expect both
  lines. No assertion was ever weakened.
- D3 - hygiene R1 false-scan on embedded Rust fixtures: multi-physical-line
  string constants holding specimen Rust code tripped the naive function-
  length scan (`fn default_user` "145 lines"); the repo's own analyzer-test
  idiom (brace-balanced single-line fixtures) fixes it without touching the
  scanner (out of touch-set). Longest authored line 151 chars, no line
  near the 2000 ceiling.
- D4 - file sizes vs the planner's itemized estimates: extractor 350
  (~330 estimated), rewriter 617 (~450 estimated; Scala's is 620 and the
  hand's ceiling is "less than Scala" - met), helper `mod_tree.rs` 195
  (< 200 cap - met). All far below the hard 1000. Delta attributed to
  dense test modules + the mechanism documentation the shape's risk register
  demands.
- D5 - SHA relay via report-back (self-referential SHA impossible, F14):
  this journal is committed WITH the landing commit; the landing SHA is
  reported in the report-back for R-9's duty 1 alongside `53afb19`.

## Caps stance (verified against the handoff text, not the briefing)

Caps literal UNCHANGED: the handoff's item 6 and flip posture keep ALL FOUR
stage bits at the R-7 four-true literal this hop (components land; bits
flip at R-9 per INV-5's waiting direction). No flip table in this handoff
moves extraction/rewriting here - the dispatch briefing's "caps flips this
hand" was superseded by the handoff's own text (the handoff is the contract).
plugin.rs change is doc-header only (naming the components landed); the
caps literal and the byte-pin test are untouched.

## Frozen locks (checked)

- Caps pin byte-identical UNEDITED (in gate 3's green run; reviewer
  diff-checks the literal).
- `git diff --name-only 9018f40` over the whole hand names only:
  `crates/lang-rust/{extractor.rs, dependency_rewriter.rs, mod_tree.rs,
  plugin.rs, lib.rs}` + `tasks/{milestone_ledger.md,
  rust-language-r7-verdict.md, rust-language-r8-journal.md}` - no
  `crates/lang/**`, no transform/validate/cli/repo/index/pipeline, no
  lang-cpp/lang-scala/tools, no fixture/golden path, no `tests/**`,
  Cargo.toml/Cargo.lock absent (no new deps; `syn::__private` seam D1).
- CURRENT_ANALYSIS_VERSION untouched (zero bytes under `crates/index`);
  fingerprint `"rust-1"` unchanged (the syn printing seam is not a parser
  change; no analyzer behavior moved - reported here explicitly).
- No `git stash`, no branch change (reflog shows duty-1 + landing only).
