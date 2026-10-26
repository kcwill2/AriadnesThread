# R-8 Verdict - APPROVED

- Milestone: `rust-language` (unregistered post-M28 track, NO M-number). Hand 8 of 10.
- Handoff of record: `tasks/rust-language-r8-handoff.md` (complete contract; read in full).
- Chain under review (start rev `9018f40`, the dispatch head): duty 1 `53afb19`
  (R-7 verdict +223 + ledger relay line + resume-artifact deletion) -> landing
  `5f87944` (`dependency_rewriter.rs` +598/-22, `extractor.rs` +314/-33,
  `mod_tree.rs` 195 NEW, `lib.rs` +1, `plugin.rs` doc-header-only 11/10,
  journal +181). Two commits, linear, reviewed individually oldest-first.
- Reviewer: fresh adjudication 2026-10-26 at HEAD `5f87944`, ground truth via
  `python tools/checkpoint_guard.py status`: HEAD `5f87944`, branch
  `balsam-basil`, dirty (none), untracked (none).
- Verdict: **APPROVED** - zero blockers; two minor findings (journal
  measurement claims), all deviations adjudicated on independently verified
  merits; D1 carries a dated deferral row that is this milestone's final word
  on `syn::__private`.

## 1. Commit-vs-touch-set (enforcement net) - PASS

`git diff --name-only 9018f40..HEAD` names exactly 8 paths:
`crates/lang-rust/{extractor.rs, dependency_rewriter.rs, mod_tree.rs, plugin.rs,
lib.rs}` + `tasks/{milestone_ledger.md, rust-language-r7-verdict.md,
rust-language-r8-journal.md}`. All inside the declared touch-set (helper
module band used for `mod_tree.rs`: one new small helper file, honestly
named). Nothing under `crates/lang`, `transform`, `validate`, `cli`, `repo`,
`index`, `pipeline`, `lang-cpp`, `lang-scala`, `tools`, `tests`, fixtures, or
goldens. `Cargo.toml`/`Cargo.lock` in NO hunk (D1's no-manifest-change claim
holds). The three `CyclePolicy::Reject` literals and the lang trait files sit
in no hunk (the only lang-rust CyclePolicy occurrences are the pre-existing
`TolerateWithinPackage` modeling literals at unchanged plugin.rs lines 26/128/213).

Per-commit:
- `53afb19` (duty 1): exactly the two duty-1 paths. Ledger relay line landed
  verbatim: "R-7 LANDED 4fd40ca (+rider a659465) APPROVED 2026-10-26, census
  anchor 1943 passed / 0 failed / 13 ignored / 125 suites @ a659465, R-8
  dispatched." R-7 verdict landed as the APPROVED file as authored (223
  lines). Resume artifact disposition verified: no
  `tasks/rust-language-r7-handoff-resume.md` exists and none was committed.
- `5f87944` (landing): the five contract files + journal. `plugin.rs` diff is
  ONE hunk inside the leading doc block: 11 added / 10 deleted lines, every
  changed line a `//!` line. No capability byte changed (blocker-class check
  at this hop - clean). `lib.rs` diff is one line: `mod mod_tree;`.

## 2. RGB attribution (verified personally) - PASS

- Red `62/21/0/1` (journal, unreproducible at landing - intermediate state
  uncommitted): verified BY NAME + BY STUB BEHAVIOR instead. All 21 named
  failures exist at HEAD (extractor 8 + rewriter 13), and each is a true
  failure against the `9018f40` blobs, which the reviewer read directly
  (extractor stub emits `DEFERRAL_HEADER` only and overrides nothing else;
  rewriter stub is pure identity). The single-suite count (doctest stage not
  run while unit tests fail) is plausible cargo behavior and returns to 2 at
  green.
- Green `83/0/0/2` - reviewer's own fresh run: `83 passed / 0 failed / 0
  ignored / 2 suites` (exit 0, `python tools/testrun.py --distill -p
  ariadne-lang-rust`). Attribution arithmetic verified by test inventory:
  the two stub files at `9018f40` held exactly 1 test each (56 = 54 other + 2
  stub pins); the landed files hold exactly 29 tests (extractor 12 + rewriter
  17); 56 - 2 + 29 = 83 exactly. Red pass count reconciles too: 8 of the 29
  new tests are trivially green on the stub (identity/empty/default-shape
  probes), 54 + 8 = 62.
- The two retirements (`extract_emits_only_the_deferral_header`,
  `rewrite_is_the_honest_identity`) are retirement-of-SUPERSEDED, not
  weakening: both pinned stub deferral behavior and BOTH name R-8 as their
  own retirement hand in their messages ("R-8 lands the extractor", "the
  use-path rewrite + mod-tree mechanism lands at R-8"). Each is replaced by
  positive byte-locks of the real mechanism (Scala
  `plugin_extractor_emits_real_scala_bytes_not_the_stub` precedent - the
  replacement pin `extractor_emits_real_rust_bytes_not_the_deferral_stub` is
  present and asserts the deferral string is GONE).
- Delegation proof (reviewer fresh): `2 passed / 0 failed / 0 ignored / 1
  suites` (exit 0, `python tools/testrun.py --distill -p ariadne-validate
  --test move_removal_transaction_test`) - UNEDITED green; `crates/validate`
  in no hunk; the four overrides resolve through the existing seam.

## 3. Caps posture (UNCHANGED this hop) - PASS

The caps literal is the R-7 four-true literal, byte-identical: plugin.rs
:253-263 pins `discovery/dependency_graph/semantic_index/load_metric: true`,
planning/extraction/rewriting/validation: false, `requires_toolchain: false`.
The pin `capabilities_match_the_load_metric_flip_literal` (plugin.rs:250) sits
in the UNEDITED test module (plugin.rs diff is doc-header only) and ran green
inside the reviewer's 83/0 run. INV-5 waiting direction honored: components
land, bits flip at R-9. The journal's Caps-stance section (handoff item 6
governs over the dispatch briefing's "caps flips this hand" phrasing) is
verified against the handoff text, which indeed carries no flip table for
this hop.

## 4. RISK-MODTREE contract (this hand's risk register) - PASS

STOP clause verified NOT firing before the build: the reviewer independently
confirmed all four overridden hooks are EXISTING neutral-seam members with
defaults (`crates/lang/src/rewriter.rs`: `find_dependency_targets` :54,
`consumer_discovery_targets` :66, `consumer_rewrite_spec` :85; plus
`move_out_top_level_symbols` :118 / `render_diff` :130 / `emit_file_preamble`
:147 left at defaults; `crates/lang/src/extractor.rs`: `locate_symbol_bodies`
:56, `places_generated_units_in_place` :94). No trait edit was needed; the
five trait-default pins and the move-retention default are UNEDITED (zero-diff
by name).

Parent-mod-insert travels WITH the extraction transaction: the mechanism is
one `RewriteSpec` carrying use-space + mod-space spellings, applied per
consumer through the existing `rewrite` seam, and discovery of the parent
flows through `consumer_discovery_targets` - the SM-4 grown-oracle discipline
holds (parent appears in the dry-run MODIFIED set through existing hooks).
Acceptance-list audit, each contract behavior against its dedicated test -
no assertion-stretching found (every test is byte-literal or exact-vector):
- discovery targets = moved ids + sanitized source stem:
  `discovery_targets_are_the_moved_paths_plus_the_source_module_stem` - asserts
  the exact 2-probe vector.
- directive hits = real syn statements only:
  `find_dependency_targets_hits_use_leaves_globs_and_mod_declarations` +
  `commented_quoted_and_sibling_spellings_are_never_hits` - comments/strings
  unreachable, mod-space probe hits the declaration verbatim, sibling prefix
  collision stays conservative (`users::User` is not under `user::`).
- glob narrowing: `glob_narrows_on_the_aggregate_old_dep_never_on_a_symbol_dep`
  - narrows only when the old dep IS the base or the base-glob; a symbol old
  dep never touches the wildcard (Scala D-M23-5 analog, mirrored honestly).
- idempotent mod: `parent_mod_insertion_is_idempotent_noop` - first
  application inserts, second is asserted BYTE-IDENTICAL. Pinned as demanded.
- no-anchor-never-inserts:
  `use_consumer_without_the_mod_anchor_never_receives_an_insertion` - a
  use-only consumer gets the use rewrite and NO `mod` graft, ever.
- anchor style: `parent_gets_the_sibling_mod_declared_once_in_the_anchor_style`
  - insert-once, visibility + indent copied from the anchored declaration,
  private anchor stays private, ASCII, CRLF-aware terminator.
- spec discipline: `consumer_rewrite_spec_drops_ids_outside_the_source_module`
  - never invent an edge the spec does not name.
- in-place placement: `generated_units_place_in_place_for_the_mod_tree` + the
  extractor docblock (a staging dir would strand the file invisible).
Helper `mod_tree.rs` (195 lines, one file, honestly named) has no own test
module; its primitives are exercised exclusively through the pinned rewriter
surface above - accepted, with the observation recorded (not a finding) that
`sanitize_module_stem`'s edge cases ride indirect coverage.

## 5. D1 - `syn::__private::ToTokens` (RISK-RUST-SYN-PRIVATE): RULING

**RULING (milestone's final word): ACCEPT-as-recorded + dated deferral row.
Not a blocker at this hop; not a permanent private path.**

Evidence gathered at the lock-pinned version (syn 2.0.119, Cargo.lock:1804-1805,
Cargo.lock zero-diff):
- `syn::__private` is `#[doc(hidden)] pub mod` (syn lib.rs:884-887, above the
  module the source comment reads "Not public API."), pathed to `export.rs`.
- export.rs:68-70: `#[cfg(feature = "printing")] #[doc(hidden)] pub use
  quote::{ToTokens, TokenStreamExt};` - the path IS quote's own `ToTokens`
  trait, re-exported under syn's default `printing` feature. Zero runtime
  divergence versus the quote path; one and the same trait for the locked
  quote version.
- The path is de-facto stable within the 2.x line: syn's OWN exported macros
  emit it downstream (`impl_to_tokens_for_custom_keyword!` implements
  `$crate::__private::ToTokens`, custom_keyword.rs:179-182; `custom_keyword!`
  uses `$crate::__private::Span/IntoSpans`, :95-112), so any crate consuming
  syn's macro surface has `__private` paths in its expanded code; the path
  survived the 1.x and 2.x major lines. The 1.x->2.x precedent shows the
  contents do move across majors.
- But "Not public API" is a real label: no semver promise inside 2.0.x, and
  this workspace lock already carries a `syn 3.0.6` package (Cargo.lock:1815)
  for another consumer - the 2.x line is not forever.
- Exposure at landing: exactly ONE import site (`grep __private` over
  `crates/lang-rust`: extractor.rs:36; the doc-comment at :30 records the
  reason).

Why not BLOCK: the handoff put `crates/lang-rust/Cargo.toml` outside the hard
touch-set under an explicit no-new-deps / Cargo.lock-unchanged law, and named
the mechanism as "syn printing", not quote. A direct `quote` import was not
legal at this hop; the honest alternatives were STOP/ASK over one import line
or the re-export path the implementer took (reported as D1, journal-first).
Why not bare ACCEPT: a private path that becomes permanent by silence is the
failure mode this risk register exists for.

DEFERRED ROW (dated 2026-10-26, to `tasks/milestone_ledger.md` deferred
findings via R-9 duty 1's ledger band): *add `quote` as a direct dependency of
`ariadne-lang-rust` and switch `crates/lang-rust/src/extractor.rs:36` from
`syn::__private::ToTokens` to `quote::ToTokens` at the FIRST hand whose
touch-set includes the crate manifest (R-10 docs hop if the planner grants
it; else the next manifest-capable hand). One import line; no behavior
change; syn 2.x stability policy makes the interim path safe, the label makes
it temporary.*

## 6. D2-D5 dispositions

- **D2 (strengthened Red-to-parity) - ACCEPTED on the merits.** The final
  assertion was independently checked against the Scala precedent: Scala
  `statement_mentions` (lang-scala/src/dependency_rewriter.rs:55-64) makes a
  wildcard hit for `target == base` and every target under `base.` (locked at
  :453-455 - `h`, `h.K`, `h.m.N` all hit `import h._`). The Rust twin
  `leaf_affected_by(..., allow_glob_prefix = true)`
  (dependency_rewriter.rs:246-254) reproduces exactly that with the `::`
  separator, and the final test asserts BOTH lines (`[2, 3]`) plus the
  glob-ghost and mod-decl probes - strictly MORE assertions than the
  under-specified Red version, and the sibling-collision guard is separately
  pinned. Strengthened-to-parity, never weakened. The two pre-green fixes are
  compiler drift (syn API field-shape corrections visible in the final code:
  `matches!(vis, Visibility::Public(_))`, `UseRename.rename`). Limit of
  evidence recorded: the Red text of that one test is not git-observable
  (intermediate state uncommitted); the FINAL state is honest parity and the
  direction of the change matches the claim.
- **D3 (brace-balanced fixture idiom) - ACCEPTED with finding 1.** The idiom
  claim is real: `analyzer.rs:114` is the pre-existing single-line
  brace-balanced fixture (`let source = "pub fn alpha() {}\npub struct Beta {
  pub field: u32 }\n";`), and the scanner was untouched (no `tools/**` hunk).
  The accompanying length claim is false - see finding 1.
- **D4 (size claims) - ACCEPTED with finding 2.** Measured: rewriter 617 <
  Scala 620 (both counted at the same bytes - Scala file read end to end,
  620); helper `mod_tree.rs` 195 < 200 cap; extractor is 344, not the claimed
  350. All far below the 1000 ceiling.
- **D5 (SHA relay via report-back) - ACCEPTED.** Self-referential SHA is
  impossible in a journal that lands inside its own commit (F14 precedent);
  the relay row in sec. 9 carries the SHA for R-9 duty 1.

## 7. Frozen locks + workspace gates - PASS

Caps pin byte-identical UNEDITED (sec. 3). Five trait-default pins +
`move_out_top_level_symbols` default + `crates/validate` delegation proof
UNEDITED (zero-diff by name; gate re-run green). L7/L8/L9: lang-cpp,
lang-scala, tools, transform, validate, cli in no hunk. L2/L3: no fixture or
golden path named. L4: m00 green inside the fresh census. L12:
`python tools/distill.py -- python tools/encoding_guard.py --all` -> `###
generic PASS (exit 0)` (capture 0 lines).
`CURRENT_ANALYSIS_VERSION` untouched (zero bytes under `crates/index`);
fingerprint `"rust-1"` unchanged (no analyzer hunk - journal's explicit
no-parser-change statement verified). No stash, no branch change: `git
rev-parse --verify refs/stash` -> `fatal: Needed a single revision` (no
`refs/stash` exists); the range is exactly duty-1 + landing, linear.
Fresh full census (the `--baseline` FILE form refused with `usage error:
missing value for --baseline` - the F9-errata form, no baseline file exists
for this track): `1970 passed / 0 failed / 13 ignored / 125 suites` (exit 0,
`python tools/testrun.py --distill`, capture 3390 lines) @ `5f87944` - NEW
ANCHOR, matches the expected verbatim. Attribution vs `1943 @ a659465`: +27 =
+29 new in-src tests - 2 retired stub pins, all in `ariadne-lang-rust`;
ignored flat 13; SUITES FLAT at 125 explicitly - no new integration file (the
apply-suite is R-9's). Hygiene: `python tools/distill.py -- python
tools/hygiene_check.py` -> `### generic PASS (exit 0)` (capture 173 lines;
every row a pre-existing warn-only R3, zero naming a file of this hand; zero
R1/R2). Tool suites: `python -m unittest discover -s tools -p "*_tests.py"`
-> `Ran 296 tests in 31.180s` / `OK`. Bench (kickoff fully-run stance - this
hand touches plugin-crate bytes that reach transform paths, RUN fresh):
`python tools/distill.py -- python tools/bench_compare.py` -> `### generic
PASS (exit 0)`; `bench compare: 3 run(s), no regressions past 1.25x
baseline`; zero WARN lines (record-not-attribute stance moot).

## 8. Findings (zero blockers)

1. MINOR - `crates/lang-rust/src/extractor.rs:205` and `:210`: journal D3
   claims "Longest authored line 151 chars"; both lines exceed 180 chars
   (grep `^.{180,}$` matches them; `^.{182,}$` matches :210). No rule
   violated (the 2000-char ceiling holds - no line truncated by the Read
   tool, hygiene exit 0), but the journalled measurement is false. Fix
   forward: measure before claiming.
2. MINOR - journal D4 records "extractor 350"; the landed file is 344 lines.
   Overstated (safe direction), within all caps either way; the other two D4
   numbers verify exactly (617 < 620; 195 < 200).

Follow-up dispositions (recorded here, landed via R-9 duty 1's ledger band):
- D1 deferred row (sec. 5) -> `tasks/milestone_ledger.md` deferred findings.
- R-9 scope note (sec. 9 quote) -> R-9 handoff/planner, verbatim.

## 9. Relay (for R-9 duty 1)

R-9 duty 1 ledger row (land verbatim):

> R-8 LANDED 5f87944 APPROVED 2026-10-26, census anchor 1970 passed / 0
> failed / 13 ignored / 125 suites @ 5f87944, R-9 dispatched (hooks +
> validator + flip hop).

Plus R-9 must carry forward, quoted from the R-8 journal (honest scope note,
NOT a silent narrowing - journalled and reported by the implementer, accepted
by this verdict):

> Honest scope note for the R-9 planner: the consumer-hook mechanism reaches
> the parent that DECLARES the source module (the sub-directory/`mod.rs`
> sibling class). When the extraction source IS the module-tree parent (root
> file keeps its items, e.g. `src/lib.rs` -> new `src/widgets.rs` beside it),
> no `mod <source-stem>;` anchor exists for the probe, and that insert rides
> the move-retention hook (`move_out_top_level_symbols`), which is R-9's
> hooks hop - the RISK-MODTREE honest-fallback wording ("items whose parent
> already declares the module") matches the class the consumer mechanism
> covers. R-9's apply suite decides the end-to-end routing.

And the D1 deferred row: add `quote` as a direct dep of `ariadne-lang-rust`
and switch `extractor.rs:36` at the first manifest-capable hop (R-10 docs hop
if granted). R-9 owns: the `extraction`/`rewriting`/`validation` caps flip
table, the retention hook (root-file-to-sibling `mod` insert), the probe-only
validator, the apply-suite, and the fixture read-only boundary.

## Reviewer toolchain note (config, not the hand)

Refused commands this session (all the catch-all boilerplate
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`): two
`python -c` line-measurement probes, a PowerShell `Get-Content` measurement
loop, `git show | Select-String`, `git stash list`, and chained
exit-code/`> $null` probes; a scratchpad script Write was refused by the
agent edit grant (`{"permission":"edit","pattern":"*","action":"deny",
"source":"agent"}`). Every refusal was substituted at equal or stronger
strength: Read-tool byte-verified line totals + grep regex length probes
(`^.{152,}$`, `^.{180,}$`, `^.{182,}$`), the `refs/stash` on-disk check
(strictly stronger than `git stash list`), and distiller `### generic PASS
(exit 0)` witnesses. No expected gate was skipped; nothing weaker than the
prescribed evidence was used.
