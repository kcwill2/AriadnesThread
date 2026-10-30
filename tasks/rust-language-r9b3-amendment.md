# R-9b3 AMENDMENT-3 (dated 2026-10-27, architect) - adjudicates the R-9b2 STOP; rules the nested-module discovery fix, the zero-consumer-commit hazard, the census floor, and the landing hand

Milestone: `rust-language` (unregistered post-M28 track, NO M-number).
HEAD of record: `8c1ac46` (R-9b2 journal landed at the clean boundary; the
symbol-isolation suite, the granted `tests/integration/Cargo.toml` dev-dep
row, and `Cargo.lock` stay UNCOMMITTED in the worktree - journal sec. 11,
"STOP posture" paragraph).
This amendment supersedes `tasks/rust-language-r9b-amendment-2.md` sec. 4
where they conflict (gate-7 census floor, suite-unchanged posture); AMENDMENT-1
(`tasks/rust-language-r9-amendment-1.md`) and AMENDMENT-2 otherwise remain
governing law. Authority: the shape's RISK-MODTREE STOP clause + the
report-don't-fix law; the implementer STOPped correctly (zero `crates/**`
hunk, journal landed, board ASK `board_561d241e` posted - journal sec. 11).
This doc commits nothing; the orchestrator lands it with the sec. 6 relay
drafts (ledger is orchestrator-owned; drafts only).

## 0. Preamble - the STOP and the evidence chain (all re-verified by read 2026-10-27)

R-9b2 (symbol-isolation suite + granted manifest rows) reached gate 2 with
`9 passed / 2 failed / 0 ignored / 1 suites` after harness triage (journal
sec. 11, gate ladder). The two reds -
`apply_moves_models_marker_and_rewrites_the_consumer_import` and
`apply_isolates_both_functions_and_rewrites_the_consumer_verbatim`
(`tests/integration/tests/rust_language_symbol_isolation_test.rs:668` and
`:710`) - are ONE mechanism root cause, and they stay on disk as the named
regression witnesses.

The root cause, chain confirmed by read:

1. Plan 2's parent is the nested module `src/models/mod.rs`. Wired
   `consumer_discovery_targets` probes the moved FLAT ids plus the sanitized
   target stem: `["crate::models_marker", "mod"]`
   (`crates/lang-rust/src/dependency_rewriter.rs:114-123`). The suite's own
   preview test pinned exactly that defective list green
   (`rust_language_symbol_isolation_test.rs:532-537`) - the pin encodes the
   defect.
2. The honest written spelling of the moved fn is
   `use crate::models::models_marker;` (suite :70, CONSUMER_SOURCE). The
   matcher `leaf_affected_by` matches non-star use leaves by FULL STRING
   EQUALITY (`dependency_rewriter.rs:315-323`), and
   `find_dependency_targets` probes use that matcher (:78-108). Neither probe
   equals the leaf spelling, and the `mod` probe cannot hit the consumer
   (no `mod mod;` exists - the inert-anchor pin at :880-891). Zero discovery.
3. Zero discovery means zero rewrite payload. The parent strip and `mod`
   insertion ride the payload through `collect_move_removals` -> the
   `move_out_top_level_symbols` hook (AMENDMENT-1 sec. 3 + the amendment's
   own preamble citing `crates/validate` `collect_move_removals` :347-370).
   No payload, no hook: the commit leaves the fn in the parent AND in the
   extracted file - an orphaned copy, no `pub mod models_marker;` line -
   while tiers answer `[Pass, Pass, Pass]` and the preview printed
   `Migration preview: 0 effective rewrites` (journal sec. 11, THE mechanism
   red paragraph).
4. The F1 FINAL WORD pair (AMENDMENT-1 sec. 1a(2)/sec. 7:
   `crate::models::models_marker` -> `crate::models::models_marker::models_marker`)
   is satisfiable on the SPEC side and already pinned green in-crate
   (`derived_spec_retargets_the_flat_id_from_a_mod_rs_parent`,
   `dependency_rewriter.rs:721-746`), but unsatisfiable through the wired
   route because the consumer is never discovered to receive the rewrite.
   Plan 1 (crate-root parent, honest spelling `crate::fixture_marker` EQUALS
   the flat probe) is fully green through the same route (suite :622-665;
   journal sec. 11) - isolating the gap to the discovery-side SPELLING
   DERIVATION for nested parents, not to discovery generally.
5. The precedent law this breaks is C++'s extraction-consumer-rewrite CW-1
   (`c916a26`, ledger row 59): `consumer_rewrite_spec` uses the SAME bare-file
   rule `consumer_discovery_targets` probes with, "so discovery's predicate
   and the rewrite's matcher agree by construction"
   (`crates/lang-cpp/src/dependency_rewriter.rs:96-113`), and C++ OWNS A
   NAMED FIDELITY WITNESS for it:
   `discovery_hit_iff_rewrite_changes_text_over_the_spelling_classes`
   ("X1(a) fidelity pair (kills the D6 class forever)", lang-cpp :347-372).
   R-9a gave the Rust SPEC side its honest derived spellings but never moved
   the discovery side onto the same grammar - discovery and rewrite now
   disagree by construction, which is exactly the D6 class CW-1 exists to
   kill.
6. Scala parity idiom: Scala probes the moved module paths verbatim
   (`crates/lang-scala/src/dependency_rewriter.rs:138-148`) because Scala ids
   carry their WRITTEN spelling (package clauses) - a shape fact recorded so
   no hand "harmonizes" the languages (AMENDMENT-1 :500-504). Rust's flat
   ids do NOT carry a written spelling; the honest spelling must be DERIVED
   from the module tree. The spec side already derives it
   (`derive_flat_id_pairs`, lang-rust :178-198, grammar pinned by
   `module_path_of_src_file`, `mod_tree.rs:139-173`). Candidate (a) is
   discovery completing the same derivation, per-language, with zero
   Scala/C++ byte (locks L7/L8, INV-3).

## 1. RULING 1 - mechanism fix: option (a). Discovery derives the honest module-path spellings. Option (b) is REJECTED.

### 1a. The ruling

CHOSEN - option (a): `RustDependencyRewriter::consumer_discovery_targets`
(`crates/lang-rust/src/dependency_rewriter.rs:114-123`) derives the honest
written-spelling probes alongside the flat ids. REJECTED - option (b)
(accepting probe-leaf tail matches in the discovery walk).

Rejected with a construction, not a preference:

- (b) must be spelled as segment-tail matching to fire at all on the wired
  shape: the leaf `crate::models::models_marker` is NOT a string-suffix of
  the probe `crate::models_marker` (the `crate::` prefix sits between), so
  any tail rule that fires for plan 2 matches on final-segment/suffix
  segments - and that exact rule makes
  `use crate::models::users::User;` match the probe
  `crate::models::user::User`, falsifying the pinned conservative-collision
  witness `commented_quoted_and_sibling_spellings_are_never_hits`
  ("prefix collisions stay conservative",
  `dependency_rewriter.rs:669-686`, the :683 leg). A rule that avoids THAT
  never fires for plan 2 either. (b) is either inert or witness-breaking.
- Governing law forbids it twice more: AMENDMENT-1 sec. 1a froze the matcher
  - "matches key on the FULL derived written spelling under leaf equality
  (`leaf_affected_by` unchanged)" (`rust-language-r9-amendment-1.md:119-121`)
  - and the collision-safety ruling `derived_spelling_never_touches_a_
  colliding_flat_name` (`dependency_rewriter.rs:752-767`) exists precisely
  to keep a rewrite from reaching across modules. Tail-matching DISCOVERY
  on top of full-equality REWRITE additionally manufactures
  discovered-but-unrewritten consumers - phantom preview entries, the
  inverse D6 violation of the same CW-1 fidelity law.

### 1b. The contract (exact, lang-rust zone only)

Zone: `consumer_discovery_targets` (`crates/lang-rust/src/dependency_rewriter.rs:114-123`)
plus its doc comment (:110-113 and the module-header bullet :31-35). Nothing
else in the file changes byte-wise (the spec leg, both matchers, and every
existing witness stay exactly as landed).

Derivation rule (uses only inputs the hook already has - `target_file` is the
PLAN TARGET, i.e. the aggregation parent `src/lib.rs` / `src/models/mod.rs`,
as today's pinned `lib`/`mod` stem probes prove, suite :527-537):

- For each moved id in order: push it verbatim (unchanged leg).
- Then, for each FLAT id `crate::X` (`flat_id_leaf`, :203-206 - the same
  eligibility grammar the spec leg uses): compute
  `M = module_path_of_src_file(target_file)` (`mod_tree.rs:139-173`). When
  `M == Some(...)` and `M != "crate"`, the honest written spelling is
  `M::X` (AMENDMENT-1 sec. 1a(2)'s own old-spelling grammar - spelled
  `crate::X` when `M == "crate"`, which is then deduped by equality with the
  id leg); push it after its id. Multi-segment ids, foreign-root ids,
  non-src targets: contribute nothing (the conservative "never invent a
  probe" stance, mirroring C++'s fallback pin, lang-cpp :332-343).
- Then push the sanitized target stem (unchanged leg, last).

Expected outputs (machine-pinned):

- plan 1 (`src/lib.rs`, `["crate::fixture_marker"]`):
  `["crate::fixture_marker", "lib"]` - BYTE-IDENTICAL to today (the `crate`
  dedupe), keeping the flat-plan-1 green legs byte-identical in meaning and
  in bytes (suite :527-531 untouched; journal sec. 11's green leg preserved).
- plan 2 (`src/models/mod.rs`, `["crate::models_marker"]`):
  `["crate::models_marker", "crate::models::models_marker", "mod"]` - the new
  middle probe equals the spec leg's pinned old spelling exactly
  (compare `dependency_rewriter.rs:729-737`), so discovery's predicate and
  the rewrite's matcher agree by construction, as CW-1 demands.
- multi-segment (`src/models/user.rs`, `["crate::models::user::User"]`):
  `["crate::models::user::User", "user"]` - BYTE-IDENTICAL, the R-8 witness
  `discovery_targets_are_the_moved_paths_plus_the_source_module_stem`
  (:626-639) survives UNEDITED because the addition is flat-id-only.

Once the consumer is discovered, everything downstream is already shipped and
pinned: the spec pair `crate::models::models_marker` ->
`crate::models::models_marker::models_marker` (:721-746), the rewrite
full-equality match (:285-313), and the parent strip + `pub mod
models_marker;` via `collect_move_removals` -> the move hook (visibility
copied from `pub mod user;`, grown-oracle dry-run already green,
suite :569-575). The two red suite tests are the apply-side proof.

### 1c. Invariants (numbered, falsifiable)

- INV-R9B3-1 (agree-by-construction). For every wired plan whose moved ids
  are flat and whose target-file parent module path is `M`, every probe the
  fix adds is an old spelling the hook-built `consumer_rewrite_spec` can act
  on, and every consumer the discovery walk hits through an added probe is
  rewritten (text changes) by that spec. Lock: W1 + W2 below. C++'s X1(a)
  pair names the shape; this is its Rust analog.
- INV-R9B3-2 (plan-1 byte-identity). Crate-root-parent probe lists and
  multi-segment/foreign-id probe lists are byte-identical before and after.
  Lock: W3 + the UNEDITED :626-639 witness + the UNEDITED suite pin :527-531.
- INV-R9B3-3 (conservative matching unchanged). `leaf_affected_by`
  (:315-323), `find_dependency_targets` (:78-108), `consumer_rewrite_spec`
  (:126-156), `derive_flat_id_pairs` (:178-198), `retarget_id` (:212-235):
  ZERO changed lines. Lock: the gate-C diff audit + the UNEDITED green of
  :669-686 and :752-767 and :721-746.
- INV-R9B3-4 (regression pair green by mechanism). The two named suite reds
  turn green with their bytes UNTOUCHED from the STOP disk state to the
  landing; the ONLY suite text change is the preview re-pin (ruling 4b),
  which is strictly strengthening (the new pinned list is a superset of the
  old, asserted by full-list equality still).

### 1d. The named witnesses (RED first, test module of `dependency_rewriter.rs`)

- W1 `discovery_probes_the_honest_module_spelling_for_a_nested_parent` -
  asserts the plan-2 expected list above verbatim. TRUE RED pre-production
  (`left` lacks the middle probe).
- W2 `discovery_and_rewrite_hit_iff_over_the_honest_spellings` - the Rust
  X1(a) fidelity pair, scoped to the wired grammar (flat ids; sanitized
  leaf equals target stem - identity for real Rust identifiers): over the
  spelling classes {honest `M::X` leaf, colliding flat name in another
  module, foreign root, commented/quoted spelling, glob}, the consumer is
  discovered (some probe hits via `find_dependency_targets`) exactly when
  the hook-built spec's `rewrite` changes the file text. Positive class red
  pre-production; negatives green-at-birth characterizations (justified
  per SM-3's clause as AMENDMENT-2 sec. 2 already used). The existing
  glob asymmetry (discovery hits globs per :643-659; a symbol-level old dep
  never narrows a glob, :523-534) is R-8 landed law - cite it in the
  witness doc, do not re-prove or "fix" it.
- W3 `crate_root_discovery_probes_stay_byte_identical` - positive pin of the
  crate-root dedupe (plan-1 list verbatim). Green-at-birth characterization,
  justified.

### 1e. What the fix must not break (witness inventory, all UNEDITED green)

- 29 R-8 witnesses + 11 R-9a pins + R-9b1's pins: failing is a STOP, not a
  test edit (AMENDMENT-2 sec. 3/4 frozen locks, unchanged).
- The pinned conservative witnesses: `dependency_rewriter.rs:669-686`
  (prefix collisions), `:752-767` (colliding flat name), `:880-891`
  (inert `lib`/`mod` anchors), `:626-639` (multi-segment probes),
  `:721-746` + `:777-872` (spec-derivation refusals).
- F2 planning freeze (`constraints.rs:52-60`): unaffected - probe derivation
  is downstream of planning; the freeze keys on kind/declaration count at
  the planning seam (`rust-language-r9-amendment-1.md` sec. 2). Zero
  cluster/CLI edit; `planner_injection.rs::frozen_symbols_are_skipped_from_
  proposals` + the m18.7 SCC lock run UNEDITED (gate 7).
- R-6/R-9a analyzer + resolver id spellings (`crate::User` et al.): the fix
  touches no analyzer byte; flat ids stay flat (AMENDMENT-1 sec. 1a option
  (a) re-qualification stays REJECTED - the fix qualifies PROBES, never ids).
- Caps literal, INV-5/INV-9, D-laws D-1/D-2/D-3 (kickoff sec. 2): no caps
  byte, no offline/A3 behavior touched (probe derivation spawns nothing);
  `.rs` grep-zero unaffected.
- L7/L8: `crates/lang-cpp`, `crates/lang-scala` zero-diff proofs by name.
  Scala parity stands: Scala's probes ARE written spellings by language
  shape; Rust now reaches the same law by derivation; the languages stay
  different for recorded shape reasons, not drift.

## 2. RULING 2 - the zero-consumer-commit hazard: DATED DEFERRAL, not this amendment's scope

The honesty hazard is real and language-agnostic: any wired extraction whose
discovery finds ZERO consumers still COMMITS with tiers `[Pass, Pass, Pass]`
and leaves a silent orphaned COPY (journal sec. 11, THE mechanism red
paragraph - preview said `0 effective rewrites`, commit said `Committed`).
Refusing it is an ENGINE transaction-policy decision in
`crates/validate` / `crates/cli` / `crates/transform`, and this track's
boundary is explicit: "Zero edits to index/graph/cluster/transform/validate/
... production code" (kickoff sec. 1); R-9b2's own contract froze EVERY
`crates/**` byte beyond lang-rust (AMENDMENT-2 sec. 4). Cracking the engine
inside a language-plugin milestone to add a refusal taxonomy would be its
own amendment with its own freeze audit - the SM-3/AMENDMENT-1 precedent
routes mechanism-policy changes through dedicated hops, not smugglers.
Adjudicated scoping:

- The fix in RULING 1 removes the hazard for the whole nested-module class
  in the Rust wired route: those plans now discover their consumers, so the
  silent-copy shape no longer arises for any plannable Rust fixture case.
- What remains after the fix is the general truth: a GENUINELY unconsumed
  symbol, on any language, commits copy-form with zero rewrites. No shipped
  witness asserts a refusal (C++'s fidelity pair constrains
  discovery-vs-rewrite, not commit-vs-zero). The R-9b2 suite never asserts
  the zero-consumer case, so no red is owed here.
- RULING: record DEFER-ZERO-CONSUMER as a DATED deferral - engine-side
  zero-consumer-commit honesty (refuse vs. documented copy: adjudicate the
  error shape, the tier reporting, and the C++/Scala/Rust parity stance in
  a dedicated engine-capable amendment or the next engine track). R-10 rider
  material (closeout deferral-row pattern, same class as the flat-id-sharing
  honest limit already stated at `constraints.rs:29-33`).
- Report-only note preserved from the implementer's hand (journal sec. 7,
  failure 6): the cli committed-through witness is honest frozen engine
  semantics; one orchestrator glance, no action this hop.

## 3. RULING 3 - the census floor gap: amended to the MEASURED value; padding is forbidden

The contract law is AMENDMENT-2 gate 7 (`rust-language-r9b-amendment-2.md:408`):
">= 2023 / 0 failed / 13 ignored / 126 suites ... floor never below 2023",
and its source is the pre-split handoff's floor arithmetic
`1981 + 42 = 2023` (`rust-language-r9b-handoff.md:68`, "plan sec. 7 R-9
lower bound governs the +42"). 2023 is a PLAN ESTIMATE - a lower bound
computed from an estimated 42-test R-9 increment - not a test-count fact.

The measured record: the pre-R-9 anchor is `1970 / 0 / 13 / 125 @ 5f87944`
(AMENDMENT-1 sec. 1c); R-9a added +11 (`1981 @ fcb078c`, handoff duty-1
clause); R-9b1 added +8 (`1989 / 0 / 13 / 125 @ 588bb86`, the certified
anchor, journal sec. 10); the symbol-isolation suite is 11 named tests
(journal sec. 11; suite file, 11 `#[test]` items). The REAL R-9 increment is
11 + 8 + 11 = 30, not 42, and `1970 + 30 = 2000` - matching the
implementer's honest landing expectation `2000 passed / 0 failed / 13
ignored / 126 suites` (journal sec. 11, hygiene paragraph).

RULING: padding tests to reach a plan estimate is FORBIDDEN - it violates
INV-13's attribution law (kickoff sec. 2, shape :110-112: "every delta
attributed by name ... never by number alone", handoff :68 in terms). The
floor of record becomes the MEASURED value with every delta named. R-9b3's
own landing additionally carries the +3 named discovery witnesses (W1-W3),
so the honest landing census is `2003 / 0 / 13 / 126`, and the R-9
suite-landing floor reads 2000. The dated ledger-relay-eligible sentence
(orchestrator-owned, sec. 6 Draft C) states exactly this; the ledger is the
source of truth and the reviewer certifies with a fresh bare census.

## 4. RULING 4 - hand shape: ONE hand, R-9b3 (fix + suite + manifest rows); the "suite lands UNCHANGED" proposal FAILS on one hunk

### 4a. The hop

Single hand. Splitting a ~150-180-line fix from the suite it unblocks would
leave the named regression reds parked through a second full census hop for
no sizing-law reason (produced lines ~150-180 vs the ~1200 ceiling; 9 gates
+ commit vs the ~10 ceiling; scope one crate file + one already-authored test
file + two already-landed-on-disk manifest lines). Bench: `crates/lang-rust/src`
touched -> `python tools/distill.py -- python tools/bench_compare.py` RUN
fresh on the reviewer's acceptance pass (kickoff sec. 4; WARNs recorded, not
attributed; no baseline/corpus change, D-3). Serial: nothing parallel-safe
(shares the suite substrate and the ledger). The dead-ended R-9b2 hand is
superseded by this amendment, not resumed; the successor bootstraps:

```
python tools/checkpoint_guard.py resume --spec tasks/rust-language-r9b3-amendment.md --scope crates/lang-rust --scope tests/integration --cmd "python tools/testrun.py --distill -p ariadne-lang-rust"
```

Reads first: this amendment (complete), journal secs. 10-11, AMENDMENT-2
secs. 4-5, AMENDMENT-1 secs. 1a/7, the named witnesses above.

### 4b. Touch-set (hard; any edit outside = blocker, report don't fix)

- orchestrator's amendment commit: `tasks/rust-language-r9b3-amendment.md`
  + `tasks/milestone_ledger.md` (sec. 6 Drafts A + C relay).
- `crates/lang-rust/src/dependency_rewriter.rs` - production edit CONFINED to
  `consumer_discovery_targets` (:114-123) + its doc comment (:110-113) + the
  module-header bullet (:31-35) wording truth; test module: exactly the three
  named witnesses W1-W3 appended. Zero other hunk in the file.
- CONDITIONAL hygiene escape, only if the inline growth would push the file
  past 995 lines (it is at 892 now; budget ~+90): `#[cfg(test)] mod
  discovery_fidelity_tests;` one line + new
  `crates/lang-rust/src/discovery_fidelity_tests.rs` carrying W2 (declare in
  the journal; this is the only new-file allowance).
- `tests/integration/tests/rust_language_symbol_isolation_test.rs` - lands
  from disk with EXACTLY ONE dated hunk: re-pin the plan-2 probe list in
  `preview_lists_discovery_targets_and_the_parent_oracle_names_the_mod_line`
  (:532-537) to `["crate::models_marker", "crate::models::models_marker",
  "mod"]` with a dated comment citing this amendment (INV-R9B3-1). Everything
  else BYTE-IDENTICAL - including the two named regression reds (:668, :710)
  and the plan-1 probe pin (:527-531). This is the ruling against the
  proposed "suite lands UNCHANGED": the preview pin (:535) pins the DEFECT
  itself, so landing it unchanged would be landing a false pin. The re-pin
  is strictly strengthening (same full-list equality, superset list).
- `tests/integration/Cargo.toml` + `Cargo.lock` - land UNCHANGED from the
  on-disk disk state (already disk-verified to compile, journal sec. 11;
  machine lock bytes stay machine bytes).
- `tasks/rust-language-r9b-journal.md` - append-only from sec. 12.
- MUST NOT touch: every other `crates/**` byte (lang-rust `plugin.rs`,
  `constraints.rs`, `planning.rs`, `validator.rs`, `mod_tree.rs`,
  `move_retention.rs`, `extractor.rs`, analyzer/resolver: ZERO hunk - the
  fix is one function; lang-cpp/lang-scala/lang/cluster/cli/transform/
  validate: ZERO hunk, L7/L8 proofs); the four caps locks; `m23_*`;
  m18-m22+m24, m00, ar27, goldens, fixtures (read-only L3), `tools/**`;
  UI/report/artifact (INV-14). No `git stash`, no branch change.

### 4c. RGB protocol (per-stage, honestly stated)

- Leg 1 (already on disk, relay + falsifiability): the two suite reds are
  the named regression witnesses; they may NOT be edited to pass - the fix
  turns them green or the hand STOPs (AMENDMENT-2 sec. 4 RGB law unchanged).
- Leg 2 (fresh RED): author W1-W3 FIRST against the unmodified production
  bytes; gate 2 must show exactly W1 + W2 red, W3 green-at-birth (justified,
  SM-3 clause), verbatim `left:` lines journalled. Expected form:
  `103 passed / 2 failed / 0 ignored / 1 suites` (102 baseline + 3 new;
  the `1 suites` shape is the standing distiller observation, journal sec. 4).
- Leg 3 (GREEN): the production edit; `105 / 0 / 0 / 2`; then the suite
  green `11 / 0 / 0 / 1`; refactor re-run same lines. Green-at-birth
  characterizations attributed by name, never silent (AMENDMENT-2 sec. 4).

### 4d. Gates (distilling entry points only, 600000 ms each; 9 + one commit; no designed-red window - the caps locks are already rider-landed and the census is bare)

1. `python tools/testrun.py --distill -p ariadne-lang-rust` - baseline
   `102 passed / 0 failed / 0 ignored / 2 suites` verbatim (R-9b1's green
   line; nothing crate-side moved since - R-9b2 duty 1 was docs-only, its
   STOP touched no crates). Mismatch = contradiction STOP.
2. same - RED (W1-W3 authored, zero production edit): exactly W1 + W2 named
   red, `left:` lines journalled; W3 green-at-birth attributed.
3. same - GREEN: `105 passed / 0 failed / 0 ignored / 2 suites`; the 29+11+
   R-9b1 witnesses green UNEDITED inside the attribution; refactor re-run,
   same line.
4. `python tools/testrun.py --distill -p ariadne-integration-tests --test rust_language_symbol_isolation_test` - `11 passed / 0 failed / 0 ignored / 1 suites` (the two regression reds green BYTE-UNTOUCHED; the preview test green only via the sanctioned re-pin).
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test m23_symbol_isolation_test` - UNEDITED green `7 / 0 / 0 / 1`.
6. `python tools/testrun.py --distill -p ariadne-cli` - UNEDITED green (dispatch consumed, not edited).
7. `python tools/testrun.py --distill -p ariadne-cluster --test planner_injection` - UNEDITED green `6 / 0 / 0 / 1` (the package correction of journal sec. 6 deviation (1) governs; the freeze stays neutrally locked).
8. `python tools/testrun.py --distill` - full census, bare tree: EXACT
   expected line `2003 passed / 0 failed / 13 ignored / 126 suites @ <R-9b3
   landing>`; deltas attributed BY NAME (+3 W1-W3 in-crate, +11 named suite
   tests, suites +1 exactly the one new integration binary, ignored flat 13);
   the floor sentence of record is this doc's ruling 3 / sec. 6 Draft C, not
   2023. Any failure outside the named deltas = STOP.
9. `python tools/hygiene_check.py` - exit 0 (`dependency_rewriter.rs` <= 995
   lines or the conditional test-file split taken; suite <= 1000 - it is at
   990 and the re-pin hunk is net +<=4; functions < 100 -
   `consumer_discovery_targets` stays ~25; banned names clean; authored bytes
   pure ASCII).

Commit (after `python tools/encoding_guard.py --staged`):
`python tools/commit_scoped.py --scope crates/lang-rust --scope tests/integration --scope Cargo.lock --scope tasks/rust-language-r9b-journal.md -m "fix(lang-rust): discovery derives honest module-path spellings; land symbol-isolation suite + granted dev-dep (rust-language R-9b3)"`.
SHA relay per the standing law (duty-less hand: landing SHA + `git show
--stat`; R-10 duty 1 cites it; reflog cross-check countermeasure applies).

### 4e. Machine-checkable acceptance (reviewer gate, skill `review-gate`)

- Gate-C diff audit: `git diff 8c1ac46..<landing> -- crates/lang-rust/
  src/dependency_rewriter.rs` - production hunks intersect ONLY
  `consumer_discovery_targets` + doc comments; ZERO deleted/modified lines in
  `leaf_affected_by`, `find_dependency_targets`, `consumer_rewrite_spec`,
  `derive_flat_id_pairs`, `retarget_id`, `insert_declared_mods`, and every
  existing test fn; test additions are the three named witnesses (or the
  declared split file).
- Suite disk-identity: the landing's suite file vs the STOP disk state shows
  EXACTLY one hunk (the preview re-pin in the named test); `git diff` proves
  the two regression-test regions zero-hunk; `Cargo.toml`/`Cargo.lock` land
  byte-identical to disk.
- Census line verbatim per gate 8; zero-diff proofs by name (lang-cpp,
  lang-scala, cli, cluster, validate, m23, caps locks, fixtures, tools);
  `refs/stash` absent; bench fresh via distiller (WARNs recorded);
  INV-R9B3-1..4 each resolved to its named witness (1c/1d).

## 5. RULING 5 - journal sec. 11 suite-side triage: ACCEPTED without re-authoring

Each triage item is an author error against SHIPPED, pinned behavior, fixed
test-side with a prior source read (journal sec. 11, numbered list) - not a
weakening. Adjudicated one by one:

1. `ends_with` consumer match: the m23 platform-separator idiom; the
   `== CONSUMER_REL` form could never match on this platform's paths.
   ACCEPTED.
2. Per-parent grown-oracle survivor needles: the hardcoded both-parents
   needles made the assert weaker than the SM-4 non-vacuity duty requires;
   parameterization strengthens it (suite :562-614). ACCEPTED.
3. F5 twin fn on a distinct line: the shipped fold dedupes by DeclId
   identity `<id>#L<line>` (AMENDMENT-1 preamble, reduced.rs merge semantics;
   constraints doc `constraints.rs:23-27`), so same-line twins fold to ONE
   DeclId and leg 3's "provably ambiguous" proof would be false. The distinct
   line is the ONLY honest way to author the spec-mandated two-declaration
   proof (suite :80-82, :947-990). ACCEPTED.
4. Moved-unit asserts on the syn-print grammar: the extractor's contract IS
   syn printing (shape :296-298; R-8-pinned `to_token_string` behavior,
   journal sec. 11 item 4); a raw-source-VERBATIM assert demanded bytes the
   engine never promised. The retarget keeps meaningful asserts (generation
   marker + moved fn + literal, spacing left to the printer), and the m23-
   mirror CONSUMER-IMPORT VERBATIM pair - the assertion whose weakening is
   BLOCKER per AMENDMENT-2 sec. 4 reviewer gate - is untouched, NOT-OLD leg
   intact. ACCEPTED.
5. `src/`-scoped rollback/re-apply digests: `artifacts/` manifests carry
   build stamps no byte-convergence claim can honestly re-verify; `src/` is
   the transaction's write surface (suite :326-327 comment). The claim is
   narrowed to what is verifiable and documented in place. ACCEPTED.
6. 1136 -> 990 compression + `assert_twin_fold_is_planning_frozen` extract:
   mandatory hygiene gates (R2 file / R1 function ceilings) with the named
   single-file touch-set; semantics, asserts, and messages unchanged; the
   two mechanism reds byte-stable across the compression reruns (journal
   sec. 11, hygiene hop paragraph). ACCEPTED.

No re-authoring is required or permitted beyond ruling 4b's single re-pin.
The two-gate-rerun-caught regressions (doc-comment fusion, `&Path` vs
`&PathBuf`) prove the triage ran under gate discipline.

## 6. Ledger relay drafts (orchestrator-owned ledger; drafts only - this doc commits nothing)

Draft A (lands WITH this amendment, orchestrator's commit, row 62 live cell):
`R-9b2 STOP 8c1ac46 clean-boundary 2026-10-27 (mechanism red: nested-module parent src/models/mod.rs discovers ZERO consumers yet commits tiers [Pass,Pass,Pass] as a silent orphaned copy - board ASK board_561d241e; suite + Cargo rows stay uncommitted on disk, journal secs. 10-11 landed); AMENDMENT-3 issued 2026-10-27: option (a) RULED - lang-rust consumer_discovery_targets derives the honest parent-qualified spellings (CW-1 agree-by-construction restored; leaf_affected_by + spec legs FROZEN unedited; option (b) rejected by construction: tail-match either never fires on the wired shape or falsifies the pinned conservative-collision witnesses); DEFER-ZERO-CONSUMER ruled a DATED DEFERRAL (engine zero-consumer-commit honesty is language-agnostic engine policy; engine stays frozen; R-10 rider material); census floor 2023 amended to measured values (real R-9 increment 30 not 42; floor 2000 at suite landing, 2003 at R-9b3 landing with +3 named witnesses; padding forbidden per INV-13); journal sec. 11 triage ACCEPTED; R-9b3 dispatched (single hand: fix + suite + granted manifest rows).`

Draft B (lands with R-10 duty 1, alongside the R-9b3 verdict - fill SHAs at
the hop):
`R-9b3 LANDED <sha> APPROVED 2026-10-27 (nested-module consumer-discovery fix option (a) - discovery and rewrite agree by construction; symbol-isolation suite 11 tests green incl. the byte-untouched regression pair; granted lang-rust dev-dep row landed), census anchor 2003 passed / 0 failed / 13 ignored / 126 suites @ <sha>, R-10 dispatched (docs hop + parity suite + dated riders incl. the DEFER-ZERO-CONSUMER row).`

Draft C (the census-floor sentence; lands inside Draft A's hop, dated,
relay-eligible):
`Census floor amended 2026-10-27 (AMENDMENT-3 ruling 3): the ">= 2023" floor (AMENDMENT-2 sec. 4 gate 7; floor arithmetic 1981+42 from the pre-split handoff) was a plan-estimate lower bound, not a test-count fact; the measured R-9 increment is 30 (R-9a +11, R-9b1 +8, symbol-isolation suite +11: 1970 -> 2000); padding to a plan estimate is forbidden by INV-13 attribution law; the floor of record is the measured census - 2000 passed / 0 failed / 13 ignored / 126 suites at the suite landing, 2003 passed / 0 failed / 13 ignored / 126 suites at the R-9b3 landing (delta +3: discovery_probes_the_honest_module_spelling_for_a_nested_parent, discovery_and_rewrite_hit_iff_over_the_honest_spellings, crate_root_discovery_probes_stay_byte_identical), each certified by the reviewer's fresh bare census.`

## 7. What stands unchanged (say it plainly)

STAND: every AMENDMENT-1 and AMENDMENT-2 ruling except the two this doc
amends by name (gate-7 census floor; suite-unchanged posture - superseded
by ruling 4b's single sanctioned re-pin). The F1 FINAL WORD byte pair stands
VERBATIM and is now satisfiable through the wired route by construction; the
caps locks, the rider band history, the frozen locks (m18-m24, m00, ar27,
goldens, fixtures, lang-cpp, lang-scala, cluster/cli/validate), the UTF-8 /
touch-set / no-stash / SHA-relay / distillation laws, the context-wall STOP
law, and INV-1..INV-15 all stand. SUPERSEDED HERE: nothing else. This
amendment adds NO crate-zone scope beyond one function and its witnesses,
adds no new deferral beyond the named DEFER-ZERO-CONSUMER row, and reopens
no id-space, planning-freeze, or caps question.
