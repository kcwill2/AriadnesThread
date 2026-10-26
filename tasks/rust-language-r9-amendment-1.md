# R-9 AMENDMENT-1 (dated 2026-10-26, architect) - adjudicates the STOP; supersedes the handoff's apply-stage mechanics; splits R-9 into R-9a -> R-9b

Milestone: `rust-language` (unregistered post-M28 track, NO M-number).
Authority: the shape's RISK-MODTREE STOP clause + the dispatch context-wall law
(the implementer correctly STOPped at a clean boundary - zero src touched,
journal landed at `c52817c`, textbook CW-3 discipline, zero improvisation).
Artifact precedent: `tasks/extraction-symbol-move-sm3-handoff.md` AMENDMENT-1
(2026-10-24): STOP -> dated amendment -> re-dispatch under the amended contract.
Trigger: `tasks/rust-language-r9-journal.md` sections 2 (F1-F5) and 3, each
finding re-verified by the architect against the named files on 2026-10-26
before issuance (parser_items.rs flat-id walker; dependency_rewriter.rs
`retarget_id`/`consumer_rewrite_spec`; mod_tree.rs `module_path_of_src_file`;
cluster/planner.rs `propose_batch` + the T7b filter + `frozen_symbol_set`;
cli/registry.rs `planning_hooks` (:280-300 - the `target` closure passes NO
repo_root, and a `None` hook answer unwraps to `candidate.path`);
cli/commands/plan.rs `LanguageRoutingConstraints::frozen_symbols` (:140-163,
per-plugin language routing already wired); lang/rewriter.rs
`move_out_top_level_symbols` default; lang-scala dependency_rewriter/planning
(Scala ids carry package clauses WRITTEN in source - Rust's flat ids do not,
and option (a) would contradict R-5 D5's accepted judgment);
lang-cpp/move_retention.rs (marked form + refusal taxonomy); validate lib.rs
`collect_move_removals` (:347-370 - hook names are the units' SIMPLE
`SymbolRecord::name`s) and `strip_moved_symbols` (:386-399);
reduced.rs `merge_tu_symbols` (fold keyed by `owning_file` per record;
declarations deduped by DeclId identity; shared ids merge declarations);
graph/file_symbols semantics; m23 apply assertions
(`assert_consumer_narrowed_and_untouched_bytes` - consumer file contains the
new import line, not the old one).

Everything in `tasks/rust-language-r9-handoff.md` STANDS except as amended
here. Reviewer: adjudicate each landing against THIS section where it
conflicts with the original text. No silent narrowing anywhere in this
amendment; every weakening-shaped choice below is either rejected or named
ESCALATE-TO-USER and not chosen.

## 0. The split (ruling first, because it renumbers the touch-sets)

R-9 splits into two serial hands. The honest fix exceeds R-9's single-hand
budget on BOTH axes of the sizing law (produced lines ~1500+ against ~1200;
scoped gate invocations 12+ against ~10 once the R-7 per-test-invocation
rider-law is honored), and it would additionally carry a RE-GATE of
R-8's APPROVED-gated mechanism inside a hop already carrying the caps flip -
the SM-3 amendment rejected a merge on weaker sizing grounds.

- **R-9a - mechanism hop** (rewriter + mod-path semantics + move-retention
  hook; NO caps flip, NO planning enrollment, NO suite). The F1 mechanism,
  the F3 hook, their red-first in-crate witnesses, and the R-8 re-gate.
- **R-9b - enrollment hop** (planning hooks, probe validator, the final
  all-true flip, the F2 freeze, the symbol-isolation suite) = the original
  R-9 hand, with the touch-set and suite amendments below, riding a green
  mechanism.

Caps posture: the all-true literal flips ONLY at R-9b. INV-5 holds at every
R-9a commit (rewriting bit stays `false` while its mechanism is fixed - the
bit never precedes its component, and R-7's four-true literal byte-identical
throughout R-9a). F14: the ledger relay for this amendment lands with the
orchestrator's amendment commit (sec. 8); R-9a duty 1 is DISCHARGED by
`c52817c` (R-8 verdict + relay already landed - do not re-land); R-9b duty 1
lands R-9a's verdict + relay.

## 1. F1 ruling - CORE BLOCKER: option (b), chosen. Payload-derived retarget in the Rust rewriter.

### Options adjudicated

- **(a) Analyzer id re-qualification - REJECTED.** The analyzer seam
  receives only hashed `FileId` + context name; a file-position prefix is
  fabricated input - the exact judgment the R-5 verdict accepted as D5, with
  the R-6 resolver as the one path-aware seam. Re-qualification would reopen
  every APPROVED id witness (R-5/6/7 suites' flat-id asserts: `crate::User`,
  `crate::default_user`, the resolver edge grammar, the merged-closure
  envelope-count witnesses) across the whole landed arc. Cost violates the
  frozen-lock discipline; there is no honest version of it inside this
  milestone.
- **(c) Re-word the handoff's apply assertion down to no consumer rewrite -
  NOT CHOSEN; would be ESCALATE-TO-USER.** The decision of record is FULL
  PARITY with Scala, where the consumer rewrite DOES fire (m23's apply
  asserts a real rewritten import line: consumer contains the new import,
  not the old). A re-wording that dropped the requirement is a scope cut on
  a user decision and may not be chosen silently by this role. It is not
  chosen because the requirement is satisfiable without reopening the id
  space: option (b) keeps the m23-mirror assertion VERBATIM.
- **(b) Payload-derived retarget (suffix-retarget) in
  `crates/lang-rust/src/dependency_rewriter.rs` - CHOSEN.** The consumer
  rewrite is derived from the plan payload the hook already receives
  (`old_includes`/`new_includes` file paths + moved ids) - the same data
  discipline that makes Scala's `old_deps = [package]` honest. It amends the
  R-8 APPROVED mechanism, so it carries the RE-GATE clause below (sec. 2);
  the collision question is answered constructively (a rewrite only ever
  fires on a FULL path-qualified written spelling, and written paths are
  location-unambiguous - flat-name collisions never key a match).

### 1a. The amended mechanism - exact contract (R-9a implements this)

(1) `mod_tree.rs::module_path_of_src_file` - Rust module semantics, replacing
the naive stem rule for the two special forms:
- `src/lib.rs` and `src/main.rs` (crate-root files) -> `Some("crate")`
  (they ARE the crate-root module; the current `crate::lib` is wrong).
- a file named `mod.rs` -> `crate::<dirs>` (the directory's module; the
  current `crate::models::mod` is wrong).
- every other src-rooted path -> `crate::<dirs>::<stem>` (byte-identical to
  today); outside a `src` dir -> `None` (unchanged conservative stance).

(2) `consumer_rewrite_spec` - after the EXISTING prefix-match `retarget_id`
path (first-wins, byte-identical behavior, still tried FIRST so every R-8
witness keeps its exact old_deps/new_deps bytes), add the DERIVATION leg:
- Eligible moved id: starts with `crate::` and its remainder after
  `crate::` is a SINGLE segment `X` (flat id - the only shape the wired
  route ever produces). Multi-segment ids take the prefix-match leg ONLY.
- For each index `i` pairing `old_includes[i]` with `new_includes[i]`
  (unequal list lengths -> the derivation leg adds nothing; the payload
  cannot name an honest pairing): with `M_old = module_path_of_src_file
  (old_includes[i])` and `M_new = module_path_of_src_file(new_includes[i])`,
  emit the use-space pair:
  - old spelling: `M_old::X`, spelled `crate::X` when `M_old == "crate"`;
  - new spelling: `M_new::X`, spelled `crate::X::X` is NOT special-cased -
    when the new file's sanitized stem equals `X`, the honest Rust spelling
    repeats the segment (`use crate::models::models_marker::models_marker;`)
    and the suite pins exactly that byte.
- Collision safety (the R-8 amendment question, answered): matches key on
  the FULL derived written spelling under leaf equality (`leaf_affected_by`
  unchanged). Two flat ids `crate::Thing` in different files derive
  different old spellings (each keyed by its own payload file); a consumer
  written path resolves to exactly one module-tree position (R-6 resolver
  truth), so a rewrite can never touch the other module's consumer. The
  "never invent an edge the spec does not name" discipline holds: every
  emitted pair is grammatically derived from a payload pairing naming this
  moved id - never observed-edge-fabricated - and a pair that matches no
  consumer rewrites nothing.
- Existing refusals stand byte-identically: ids without a `crate::` root are
  dropped (`consumer_rewrite_spec_drops_ids_outside_the_source_module`
  survives UNEDITED); mod-space half unchanged (bare stems still anchor the
  sub-directory grafts; `lib`/`mod` stems are inert anchors - no file
  declares `mod lib;`/`mod mod;` - pinned).

### 1b. RED protocol for the new mechanism (R-9a true Red; author tests BEFORE any mechanism edit)

Named reds against the `c52817c`-tree blobs (all in `dependency_rewriter.rs`
/ `mod_tree`-exercising pins; journal the verbatim `left:` lines):
1. `module_path_pins_the_rust_module_semantics` -
   `src/lib.rs -> Some("crate")` fails `left: Some("crate::lib")`;
   `src/models/mod.rs -> Some("crate::models")` fails `left: Some
   ("crate::models::mod")`; plus the unchanged forms (positive legs green-at-
   birth are fine inside a red-captured test file - the named failures
   above are the red).
2. `derived_spec_retargets_the_flat_id_from_the_root_parent` -
   spec from old `[src/lib.rs]`, new `[src/fixture_marker.rs]`, moved
   `["crate::fixture_marker"]` must contain old spelling
   `crate::fixture_marker` and new spelling
   `crate::fixture_marker::fixture_marker` - fails pre-mechanism (`left:
   ["lib"]`).
3. `derived_spec_retargets_the_flat_id_from_a_mod_rs_parent` -
   old `[src/models/mod.rs]`, new `[src/models/models_marker.rs]`, moved
   `["crate::models_marker"]` -> `crate::models::models_marker` ->
   `crate::models::models_marker::models_marker` - fails pre-mechanism.
4. `derived_spelling_never_touches_a_colliding_flat_name` - two modules
   `src/a/m.rs`/`src/b/m.rs` each declaring `Thing`; spec derived from the
   `src/a/m.rs` payload rewrites `use crate::a::m::Thing;` and leaves
   `use crate::b::m::Thing;` BYTE-IDENTICAL - fails pre-mechanism.
5. `derived_leg_refuses_multisegment_ids_and_foreign_roots` -
   `crate::inner::Helper` and `other::place::CoProposed` on a root-parent
   payload yield NO derived pair (GREEN-at-birth characterization of the
   refusal - justified-green-at-birth per SM-3's clause; the refusal is
   today's behavior and stays).
6. Move-hook reds (sec. 3): `move_out_removes_the_item_and_declares_the_
   module_once`, `move_out_reinsert_is_idempotent`, `move_out_refuses_
   unparseable_or_unlocatable`, `move_out_refuses_module_ident_and_
   already_moved` - fail pre-hook (the default delegation answers `None`).

Green: implement 1a + sec. 3; re-run. The R-8 witnesses below must go green
without a single edited byte - a witness failing for the documented
mechanism reason is a STOP, not a test edit.

### 1c. RE-GATE clause (R-8's mechanism is APPROVED-gated; this amends it)

R-9a's landing is re-gated by the R-9a reviewer on top of the ordinary
acceptance pass:
- `git diff 5f87944..<R-9a landing> -- crates/lang-rust/src/
  dependency_rewriter.rs` shows ZERO deleted or modified lines inside the
  existing 17 test functions (their bytes identical; all hunks are
  production-fn edits plus test-module APPENDS); `extractor.rs` and
  `lib.rs`-aside-from-one-mod-line are ZERO-diff (extractor.rs in no hunk).
- All 29 R-8 in-crate tests run UNEDITED and green inside the crate gate
  (`-p ariadne-lang-rust`), arithmetic: `83 passed` baseline + N new tests,
  zero retirements in this hop (none of R-8's pins is superseded - the
  derivation leg is strictly additive because the prefix-match leg runs
  first and keeps the `crate::models::user::User` specimen bytes).
- The neutral-seam proofs run UNEDITED: `crates/lang/src/rewriter.rs`
  zero-diff, `crates/validate/tests/move_removal_transaction_test.rs` green
  (delegation proof), `m23_symbol_isolation_test.rs` green zero-diff (gate
  commands 4/5 below).
- Census: `1970 + N passed / 0 failed / 13 ignored / 125 suites` (anchor of
  record `1970 / 0 / 13 / 125 @ 5f87944`), deltas attributed by name.

## 2. F2 ruling - mod declarations stay records, become planning-frozen at the plugin's constraints seam; zero cluster/CLI edits

Module-kind symbols belong in the INDEX and GRAPH (they are R-6's APPROVED
resolution witnesses and the include-graph edges) but are NOT extraction
units: a `mod x;` declaration is structure; its "body" IS the child file at
its declared path, so any "extraction" targeting it is either a frozen-child
clobber (`src/lib.rs -> src/greeting.rs`, `src/models/mod.rs ->
src/models/user.rs`) or a self-referential stub (`src/models.rs` beside
`src/models/`). Deciding it that way is language GRAMMAR, so the filter sits
in the plugin, not the neutral engine - `crates/cluster` and `crates/cli`
stay in NO hunk.

Contract (R-9b, `crates/lang-rust/src/constraints.rs` - ADDED to R-9b's
touch-set; the file's R-3 header line "This file does not grow" is
superseded by this amendment, honestly reworded in the same commit):
`frozen_symbols` answers the union of
- records with `kind == SymbolKind::Module` (structure, never extractable);
  and
- records whose `declarations.len() >= 2` (the flat-id sharing class - a
  record folded from two written items cannot honestly be extracted at all;
  the analyzer gives every written item exactly one `<qualified-id>#L<line>`
  DeclId and the fold dedupes by DeclId identity, so >= 2 declarations is
  proof of a shared fold, deterministic regardless of fold order or folded
  kind).
`colocated_groups` stays empty; `colocated_groups_with_edges` inherits
emptiness. The existing planner semantics do the rest: "a batch holding a
frozen symbol is skipped whole" - the mechanism is ALREADY neutrally locked
(`crates/cluster/tests/planner_injection.rs::frozen_symbols_are_skipped_
from_proposals` + the m18.7 SCC lock run UNEDITED as gates;
`LanguageRoutingConstraints::frozen_symbols` (cli/commands/plan.rs:140-163)
already routes Rust subsets to the Rust plugin with zero CLI hunk; the CLI
dispatcher's own empty-frozen test feeds Scala/C++ records only - stays
green UNEDITED, cli gate covers it).

IV3 stance, recorded (architect amendment to the shape's decision-table row):
IV3 says the POLICY layer owns freezing symbol instances; this rule freezes
a symbol KIND plus provably-ambiguous folds at the language layer - the
M18.7 constraints seam exists precisely for language grammar answering "what
cannot move". The shape table row and the constraints docblock are updated
with this sentence in the R-9b landing (no stale prose survives).
The `no_colocated_groups_and_no_frozen_symbols` pin (in-touch-set file)
EVOLVES: its fn/struct batch legs survive byte-identical (still empty), a
Module batch and a two-declaration batch gain named positive pins.

Witnesses (R-9b suite): a plan-stage assert that the fixture-copy plans are
EXACTLY the two legitimate fn proposals (`fixture_marker ->
src/fixture_marker.rs`, `models_marker -> src/models/models_marker.rs`) and
ZERO proposals name a Module-kind symbol or target a frozen child file.
Apply-side protection: none is needed - a frozen batch never reaches apply;
if a future engine change bypasses the filter, the extractor's honest
absence for a `mod` body is the second line of defense (report, don't fix).

## 3. F3 ruling - the move-retention hook lives in the mechanism hop; touch-set expanded, not reassigned

Granting R-9a the touch-set the hook physically requires is the honest fix
(SM-3's declared-rider shape: the mechanism and its witnesses ship in one
hand); routing it to R-10 would strand R-9b's suite on a copy-fallback tree
and make the grown-oracle assert un-satisfiable. R-10 stays the docs hop.

R-9a touch-set (hard): `tasks/rust-language-r9a-journal.md` (create; opens by
citing this amendment + `tasks/rust-language-r9-journal.md` sec. 9);
`crates/lang-rust/src/dependency_rewriter.rs`; `crates/lang-rust/src/
mod_tree.rs`; NEW `crates/lang-rust/src/move_retention.rs` (helper-band rule
- the R-8 verdict's own precedent for `mod_tree.rs`; use it when the hook +
deletion machinery would push `dependency_rewriter.rs` past ~700 lines, the
override itself stays in `dependency_rewriter.rs` as one delegating method);
`crates/lang-rust/src/lib.rs` (one `mod move_retention;` line if created).
MUST NOT touch: `plugin.rs` (the caps literal and every pin stay
byte-identical - INV-5), `planning.rs`/`validator.rs` (R-9b), `extractor.rs`,
resolver/parser/analyzer, `crates/cluster`, `crates/cli`, `crates/lang`,
`crates/validate`, fixtures, m23, m18-m22, tools, manifests, goldens. No
stash, no branch change.

The hook contract - `move_out_top_level_symbols(source, names, plan_id)`,
where names arrive as the units' SIMPLE `SymbolRecord::name`s
(`collect_move_removals` :365-367):
- RETENTION FORM: deletion + declaration insert (Scala parity - the user
  decision of record is full parity with Scala, whose retention deletes;
  C++'s marked `ARIADNE-EXTRACT-OUT` form belongs to the
  extraction-symbol-move decision for C++ headers and is NOT copied here).
  `plan_id` is ignored (a retention that needs no provenance ignores it -
  the trait doc's own allowance).
- Refusals (`None` -> engine copy-fallback leaves bytes untouched): source
  unparseable; any requested name locates no top-level item AND its
  sanitized `mod <name>;` sibling declaration is absent (nothing moved and
  nothing declared - honest refusal); any name locates only a `mod`-ident
  item (structure never moves); re-offer after a completed move (all names
  absent, all declarations present) answers `None` - the byte-preserving
  idempotence outcome, C++'s R-B outcome-shape.
- Work: remove each located top-level item by syn byte-span (attrs included,
  the existing `apply_splices` + full-line widening discipline; comments
  adjacent to an item are not item bytes - the R-8 printing caveat,
  documented); then insert `mod <sanitize_module_stem(name)>;` for every
  moved name not yet declared - the stem grammar is EXACTLY the
  `target_path_for` grammar (same function, one source of truth:
  `fixture_marker` -> `mod fixture_marker;`), anchored after the LAST
  existing out-of-line `mod` declaration (visibility/indent copied from it;
  file-private default when none), CRLF-aware like the R-8 grafts; already-
  declared -> no second insert (byte-identical pin).
- End-to-end routing (the R-8 verdict's honest-scope note discharged): the
  root-file-to-sibling class (`src/lib.rs -> src/fixture_marker.rs`,
  `src/models/mod.rs -> src/models/models_marker.rs`) inserts via THIS hook
  because no `mod <source-stem>;` consumer anchor exists; the sub-directory
  class keeps R-8's consumer-side anchored graft; `move_out`-driven parent
  modification must appear in the apply/preview bytes the suite reads (the
  handoff's reviewer MUST: the assert names the parent path AND the inserted
  line - non-vacuous or the hand STOPs; the engine is frozen either way).

## 4. F4 confirmation - the exact caps-lock rider band (orchestrator's commit, not the implementer's)

The all-true flip supersedes exactly these FOUR integration locks, all
outside R-9b's touch-set. The implementer NEVER touches them (report path:
board ASK + journal, R-5 D3 / R-6 D1 / R-7 precedent); the architect lands
one rider commit after R-9b's landing and before the reviewer's fresh census:

| file | test fn | rider hunk |
|---|---|---|
| `tests/integration/tests/rust_language_discovery_test.rs` | `capabilities_lock_the_registration_flip_literal` | four bits `false -> true` (planning, extraction, rewriting, validation) + lock-narrative comment evolution |
| `tests/integration/tests/rust_language_semantic_index_test.rs` | `capabilities_lock_the_semantic_index_flip_literal` | same shape |
| `tests/integration/tests/rust_language_dependency_graph_test.rs` | `capabilities_lock_the_dependency_graph_flip_literal` | same shape |
| `tests/integration/tests/rust_language_load_analysis_test.rs` | `capabilities_lock_the_load_metric_flip_literal` | same shape PLUS the explicit true-count `4 -> 8` |

Full-struct byte-exact `assert_eq!` discipline intact in all four; no
assertion removed, no name changed (the m23/R-5/R-6/R-7 evolution idiom).
In-src witnesses are touch-set-owned by R-9b, not rider: the impl literal +
pin rename to `capabilities_match_the_parity_flip_literal` (handoff item 3)
and `plugin_default_seams_stay_at_trait_defaults` evolving its two hook legs
(hook-default red is the handoff's own named red).
Designed-red window: R-9b lands carrying EXACTLY the four named reds
disclosed here (SM-3 hard-chain / R-7 flow); the implementer's census runs in
the no-fail-fast form with the four failures attributed by name; the rider
turns them; the R-9b reviewer's fresh bare census proves both (`126 suites`).

## 5. F5 ruling - IN SCOPE, named test, no re-park

The R-5 D5 parked class is adjudicated HERE, in R-9b's suite
(`tests/integration/tests/rust_language_symbol_isolation_test.rs`), exact
test: `flat_id_shared_mod_and_fn_record_is_planning_frozen`. On the fixture
copy it asserts machine-checkable:
1. the reduced model answers EXACTLY ONE record under id `crate::greeting`
   with `declarations.len() == 2` (the `#L4` fn decl from `src/greeting.rs`
   and the `#L6` mod decl from `src/lib.rs`), `declaring_files ==` the
   sorted pair - and it NEVER pins the folded `kind` or `defining_file`
   (fold-order-dependent bytes are not pinned, honesty over completeness);
2. the wired plans contain NO proposal naming `crate::greeting` and NO
   proposal targeting `src/greeting.rs` anywhere;
3. the same >=2-declaration freeze witnesses on a suite-authored temp pair
   (two temp files each declaring `fn twin();` -> ids `crate::twin` -> the
   fold is ambiguous -> frozen; any candidate listing it proposes nothing).
RULING text for the record: flat-id sharing is the known, ACCEPTED cost of
the flat-id analyzer (R-5 D5's accepted design + option (a) rejected); it is
neutralized at the planning seam by the F2 freeze, not by re-qualification.
The honest limit (shared-id symbols are never plannable; safe support would
require path-aware ids) is a DATED deferral row for the closeout ledger +
R-10 rider material (`constraints.rs` docblock states it at R-9b landing).

## 6. Red/green protocols and gates (per-test invocations per the R-7-accepted law; distilling entry points; 600000 ms)

R-9a - declared budget 7 + commit trio:
1. `python tools/testrun.py --distill -p ariadne-lang-rust` - baseline `83 / 0 / 0 / 2` verbatim.
2. same - Red: exactly sec. 1b's named failures, `left:` lines journalled.
3. same - Green: `83 + N / 0 / 0 / 2`, R-8 witnesses green UNEDITED (sec. 1c).
4. `python tools/testrun.py --distill -p ariadne-validate --test move_removal_transaction_test` - UNEDITED green.
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test m23_symbol_isolation_test` - UNEDITED green.
6. `python tools/testrun.py --distill` - census `1970 + N / 0 / 13 / 125`, deltas by name.
7. `python tools/hygiene_check.py` - exit 0.
Commit: `python tools/encoding_guard.py --staged` then `python tools/commit_scoped.py --scope crates/lang-rust --scope tasks/rust-language-r9a-journal.md -m "fix(lang-rust): payload-derived consumer retarget + move-retention hook (rust-language R-9a)"`. Reviewer's acceptance pass: `bench_compare` fresh (src-touching hop, kickoff sec. 4) + the sec. 1c re-gate.

R-9b - declared budget 9 + commit trio (the four rider locks are NOT in its gates - they ride the rider commit; the census names them):
1. `-p ariadne-lang-rust` - baseline (`83 + N`).
2. same - Red: the renamed parity pin `left: planning: false, right: true`, the hook-default `None` seam legs, the validator probe tests on the stub, the `constraints` Module/two-decl pins.
3. same - Green.
4. `--test rust_language_symbol_isolation_test` - Red (plan-stage empty: hooks None -> honestly no aggregation unit).
5. same - Green (stages per sec. 7).
6. `--test m23_symbol_isolation_test` - UNEDITED green.
7. `-p ariadne-cli` - UNEDITED green.
8. full census, no-fail-fast form: `1962/1966-shape` pre-rider truth is NOT stated as a number here - journal the observed line; expected failure set EXACTLY the four sec. 4 locks, each `left:` showing the flipped struct.
9. `python tools/hygiene_check.py` - exit 0.
Commit scopes: `--scope crates/lang-rust --scope tests/integration/tests/rust_language_symbol_isolation_test.rs --scope tasks/rust-language-r9b-journal.md`, message `"feat(lang-rust): planning hooks, probe-only validator, F2 freeze, final all-true capabilities, isolation suite (rust-language R-9b)"`. Reviewer acceptance: bench fresh; caps hunk bit-count audit (exactly four bits gained; `load_metric` must NOT re-appear); INV-10 a/b/c UNEDITED-form audit; every spawn in probe tests `--version`-shaped; grown-oracle assert non-vacuity (names the parent path AND the inserted line); F2/F5 tests meaningful (not vacuously true); census attribution across `1970 + N`.

## 7. R-9b suite spec (amended apply-stage - the m23 mirror now satisfiable VERBATIM)

Fixture-driven, temp-copy idiom (fixture read-only, L3), tier-3 `Skipped`
via the scripted all-None runner through
`RustToolchainValidator::with_runner` consumed by
`TierValidator::validate_affected_tus` (journal F6's consumption mapping -
cargo/rustc ON PATH here, so a wired apply would report `Pass`; the suite
never asserts `Skipped` against the real environment).
- plan: aggregation candidates = `src/lib.rs` + `src/models/mod.rs` (hook
  dispatch); proposals exactly the two fn extractions (F2); no Module
  proposal, no child target, no `crate::greeting` (F5).
- preview: consumer discovery targets listed; the parent-MODIFIED grown-
  oracle assert per the handoff's reviewer MUST, non-vacuous on the engine's
  actual dry-run bytes (STOP-and-report if the frozen engine cannot honestly
  report it - engine stays frozen).
- apply (temp consumer files authored IN the temp copy only, R-6
  suite-authored-temp precedent, added to lib.rs's copy with `mod
  consumer;`): consumer bytes contain `use
  crate::fixture_marker::fixture_marker;` and NOT the old one-line import;
  same pair for `models_marker` (`use
  crate::models::models_marker::models_marker;`); parents lose the fn bytes
  and gain their `mod` line exactly once; re-apply discipline =
  rollback-then-re-apply byte convergence + the hook-level idempotence pins
  (sec. 3); no frozen fixture child modified anywhere in the copy (byte
  census of `src/greeting.rs`/`src/models/user.rs`).
- rollback: bytes back (`tree_digest` m23 idiom); re-plan converges to the
  pre-apply plan.
- tier-3: `Skipped` via the scripted runner; NEVER a spawn beyond
  `--version` probes (INV-10).
Caps: the new suite's own all-true byte-lock is authored GREEN-SIDE with the
flip commit (R-6 D5 accepted precedent; the red evidence is the renamed
in-src pin + the seam-legs).

## 8. Bootstrap lines and the orchestrator's own obligations

R-9a dispatch (fresh session; this STOP journal's successor - the dead
session is never resumed):
```
python tools/checkpoint_guard.py resume --spec tasks/rust-language-r9-handoff.md --scope crates/lang-rust --cmd "python tools/testrun.py --distill -p ariadne-lang-rust"
```
Reads first: this amendment (complete), R-9 handoff (context), journal
sec. 2/3/9, `tasks/rust-language-r8-verdict.md` sec. 4/9, then the code
files named in sec. 1a/1b/3. Journal: create
`tasks/rust-language-r9a-journal.md`.

R-9b dispatch (after R-9a gates green; fresh handoff doc authored by the
planner from this amendment - same content, R-9a landing as duty-1 target):
```
python tools/checkpoint_guard.py resume --spec tasks/rust-language-r9-handoff.md --scope crates/lang-rust --scope tests/integration/tests --cmd "python tools/testrun.py --distill -p ariadne-lang-rust"
```

Orchestrator lands WITH this amendment (pre-authorized rider band, dated
2026-10-26, ledger is orchestrator-owned):
- ledger live-row append (verbatim draft): `R-9 STOP c52817c clean-boundary
  2026-10-26 (zero src, journal landed); AMENDMENT-1 issued 2026-10-26
  (F1 payload-derived retarget chosen / F2 Module+ambiguous-fold freeze at
  the constraints seam / F3 touch-set expanded - hook in R-9a / F4 four-lock
  rider band confirmed / F5 in-scope named test); R-9 split R-9a (mechanism,
  no caps flip) -> R-9b (hooks + validator + all-true flip + suite + rider);
  R-9a dispatched.`
- R-9b rider commit (sec. 4's four locks) - the ONLY test-byte edits this
  amendment supersedes outside touch-sets.
- R-10 unchanged: docs hop + the dated `quote`/`syn::__private` deferral row
  (applied only if R-10 gets a manifest-capable scope) + the flat-id-sharing
  honest-limit rider material (sec. 5).

## 9. Superseded-witness register (explicit - who rides what)

RIDER BAND (architect's commits; the implementer must never touch these):
1. `rust_language_discovery_test.rs::capabilities_lock_the_registration_flip_literal`
2. `rust_language_semantic_index_test.rs::capabilities_lock_the_semantic_index_flip_literal`
3. `rust_language_dependency_graph_test.rs::capabilities_lock_the_dependency_graph_flip_literal`
4. `rust_language_load_analysis_test.rs::capabilities_lock_the_load_metric_flip_literal`
Nothing else. No UNEDITED-witness in any frozen lock (m18-m23, m00, ar27,
goldens, fixtures, lang-cpp, lang-scala, cluster suites) is superseded by
this amendment.

IMPLEMENTER BAND (inside touch-sets, evolve in-landing with positive
replacement - retirement-of-SUPERSEDED, never weakening):
- R-9b `plugin.rs`: `capabilities_match_the_load_metric_flip_literal` ->
  `capabilities_match_the_parity_flip_literal` (handoff item 3);
  `plugin_default_seams_stay_at_trait_defaults` hook legs -> Some-dispatch
  pins (non-hook legs byte-identical).
- R-9b `validator.rs`: the two stub pins
  (`toolchain_status_is_missing_never_unknown`,
  `validate_compile_errors_naming_the_deferral`) are superseded by the
  scripted-runner probe tests (a real probe answers `Available` on this
  machine - the old unconditional-`Missing` pin cannot survive the flip;
  the deferral-message truth survives in the offline-scripted test).
- R-9b `constraints.rs`: `no_colocated_groups_and_no_frozen_symbols`
  evolves per sec. 2; its R-3 header line reworded.
- R-9a: NOTHING superseded (sec. 1c's byte-identity clause).

## 10. Risks and oracles (delta)

- RISK-R9-MECHANISM-DRIFT: the derivation leg could perturb the
  prefix-match leg's bytes. Oracle: the sec. 1c zero-deletion diff + the 29
  UNEDITED witnesses + the `split_spec` specimen's exact vectors.
- RISK-R9-HOOK-SPICE: deletion by syn span could orphan comments or
  mis-splice CRLF. Oracle: named in-crate pins (byte-exact before/after,
  CRLF specimen, attr-bearing struct specimen), all-occurrence discipline
  inherited from R-8's splices.
- RISK-R9-FREEZE-BLAST: the freeze could accidentally hit fn batches.
  Oracle: the fixture plans pin EXACTLY two proposals;
  `frozen_symbols_are_skipped_from_proposals` UNEDITED; the CLI dispatcher
  test (Scala/C++ records) UNEDITED green.
- RISK-R9-DESIGNED-RED: R-9b's window (four red locks pre-rider) could be
  mistaken for a broken tree. Oracle: this amendment's disclosure, the
  journal's named-failure census line, the rider's four-line diff, the
  reviewer's fresh bare census `126 suites / 0 failed`.
- Hygiene: `dependency_rewriter.rs` 617 -> ~700 (helper module absorbs the
  rest; new files: `move_retention.rs` ~180, `planning.rs` ~280,
  `validator.rs` ~200, suite ~650 - every file < 1000, functions < 100, no
  banned names; authored bytes pure ASCII; ledger row growth continues its
  closeout-consolidation note - not a blocker class).

## 11. What stands unchanged (say it plainly)

The handoff's hooks contract (aggregation_filter / target_path_for pair, the
sanitize discipline mirroring `scala_target_path_for`, registry-dispatched
enrollment with zero CLI edit), the probe-only validator contract (probe
order `["cargo", "rustc"]`, CommandRunner seam mirroring the Scala file
1-for-1, INV-10 a/b/c as in-src tests, single-input honesty doc, tier-3
`Skipped` consumed from `crates/validate`), the caps literal and its
monotonicity audit, the m23 zero-diff and grown-oracle disciplines, every
frozen lock, the UTF-8/encoding/touch-set laws, and the STOP clause itself -
all stand. The apply-stage consumer-rewrite assertion stands VERBATIM -
option (b) makes it true instead of weakening it. C++/Scala remain zero-diff
(Scala needs none of this: its ids carry written package clauses - a shape
fact, re-verified this session, recorded so no future hand "harmonizes" the
two languages' specs).
