# rust-language R-5 journal - syn analyzer, fingerprint "rust-1", semantic_index flip, span truth

Hand 5 of 10, serial. Contract: `tasks/rust-language-r5-handoff.md`.
Start rev: `c54035d` (dispatch head; R-4 verdict uncommitted awaiting duty 1).

## Duty 1 (before any R-5 mutation)

Landed the R-4 verdict + ledger flip in ONE commit: `80a2e28`
(`tasks/rust-language-r4-verdict.md` +237, `tasks/milestone_ledger.md` 1-line
flip carrying BOTH clauses verbatim from the R-4 verdict relay section):
- `R-4 LANDED ec552bd APPROVED 2026-10-26, census anchor 1886 passed / 0
  failed / 13 ignored / 122 suites @ a9b6864, R-5 dispatched (analyzer hop).`
- `fixture rust-basic APPROVED 2026-10-26 at ec552bd via R-4 verdict - L3
  approval path discharged`

Ledger row 62 was already above the 2000-char read-truncation line before this
append (pre-existing per-hand growth from R-1..R-4 duty-1s); the append is the
track's established duty-1 shape. kilo-memory key `ledger.milestone.rust-language`
mirrored same-commit (AGENTS.md ledger law).

## Gate 1 - clean baseline BEFORE red edits

`python tools/testrun.py --distill -p ariadne-lang-rust` (exit 0):
`24 passed / 0 failed / 0 ignored / 2 suites` (capture
`target/testrun-capture.txt`, 37 lines). Matches the R-4 verdict's full-package
figure exactly; the working tree equals the duty-1 landing for this crate.

## Declared deviations (journal-first, before any mutation)

- D1 (stale handoff literal, honored anyway): the handoff names
  `CURRENT_ANALYSIS_VERSION` as "frozen `2.1.0`"; the live tree reads
  **`2.2.0`** - raised by the M28 T4 DP-2 bump, and m21's own UNEDITED
  assertion (`scala_fingerprint_moves_only_the_scala_digest`) pins `"2.2.0"`.
  The operative instruction - ZERO bytes under `crates/index/**`, never the
  vehicle - is what I execute. Read-check recorded: `crates/index/src/cache.rs`
  reads `2.2.0` (grep at gate 7; file value quoted there).
- D2 (gate budget +1): the handoff's Red protocol requires journaling the
  SUITE failing list, which needs one scoped run of the new suite in red
  (`--test rust_language_semantic_index_test`). That invocation sits outside
  the seven named gates; declared here. Green re-runs of the same gates are
  the named invocations.
- D3 (external stale lock the mandatory flip breaks - REPORT, do NOT fix):
  `tests/integration/tests/rust_language_discovery_test.rs`
  `capabilities_lock_the_registration_flip_literal` (:294-315) locks the
  R-4-side caps literal incl. `semantic_index: false`. Item 6 (`semantic_index:
  true`) is mandatory and census-gate-6 demands failed 0, yet the file is
  outside the touch-set. Per the touch-set discipline (report, never fix) I do
  NOT edit it; census evidence is journalled at gate 6 and the coordinator is
  notified. Precedent for the fix shape exists: m19's Scala caps lock was
  "EVOLVED at M23 T7 (D-M23-6 flip)" at the flip hop (integration-side lock
  moves with the flip). The new integration-side lock for the flipped literal
  lands in MY suite (`capabilities_lock_the_semantic_index_flip_literal`), per
  shape INV-5 ("that test + m19-style capability lock in the new integration
  suite"), arriving with the flip commit (amendment-style byte-pin, authored
  green-side, never red-in-isolation).
- D4 (forced in-file amendment inside plugin.rs's named edits): the touch-set
  grants "the `analysis_fingerprint` override + caps literal flip + pin-test
  rename; nothing else in the file". The override makes
  `plugin_default_seams_stay_at_trait_defaults`'s `analysis_fingerprint() ==
  "v1"` assertion false by construction; the minimal forced fix is deleting
  that one stale assertion (its sibling aggregation_filter/target_path_for
  assertions stay untouched). This is a direct consequence of the named
  override, not an additional edit of intent.
- D5 (qualified-id scheme, design judgment): `crate::<path>::<Name>` cannot
  carry the TU's file position at the `SemanticAnalyzer` seam - read-verified:
  `FileId` is a SHA-256-prefix hash (irreversible), `context` is the build-
  context name (`worker.rs:95` `&task.tu.context_name`;
  `worker-core/src/tu_store.rs:208` same), `proc_macro2` spans carry only
  line/col. Inventing a file-position prefix would fabricate an input the
  analyzer does not receive - the exact class D-2 forbids. `crate::<path>::`
  is therefore the `mod`-declaration chain VISIBLE IN THE FILE (inline
  `mod x { ... }` nesting), rooted at `crate::`; out-of-line `mod x;` records
  the declared module name. Cross-file module-tree truth rides R-6's resolver
  (which does receive `rel_path` on `resolve_edges`). Consequence: the lib.rs
  `mod greeting;` record id equals the greeting.rs `fn greeting` id
  (`crate::greeting`, differing `kind`/`owning_file`) - the same id-sharing
  class `ReducedSemanticModel::reduce` already FOLDS by design (pinned for
  Scala's cross-file `sub` fold, `crates/graph/src/reduced.rs` test module),
  and it is R-9's named symbol-isolation suite that owns this hazard's
  adjudication (plan line R-9), not this hop.

## Span convention (INV-7 discipline, locked before authoring)

- Location = the item's NAME token position (`proc_macro2::Span::start()` on
  the ident), matching Scala's name-token `SymbolDecl` locations
  (`lang-scala/src/parser/declarations.rs` `name_line`/`name_column`).
  Name-token choice is also attribute-immune (fixture `User` carries
  `#[derive(Debug)]` on its own line; item-span starts would fork on syn
  internals, name spans do not).
- 1-based line and 1-based column: `Span::start().column` is 0-based, so the
  analyzer adds 1 - the pinned convention of both shipped tokenizers
  ("1-based source position ... matching the `lang-cpp` tokenizer convention",
  `lang-scala/src/scanner.rs:31-32`, pinned at scanner.rs:394).
- `DeclId` form mirrors Scala at the same seam: `<qualified-id>#L<line>`
  (`declarations.rs:324`), not C++'s `decl_<counter>`.

## Red (gate 2 + protocol suite run)

Gate 2 `python tools/testrun.py --distill -p ariadne-lang-rust` (exit 1):
`24 passed / 2 failed / 0 ignored / 1 suites` - the two named in-src pins
fail exactly as the handoff predicted:
- `plugin::tests::analysis_fingerprint_is_rust_1` - `left: "v1"` (vs
  `"rust-1"`), the handoff's verbatim expectation;
- `analyzer::tests::analyze_carries_exact_name_spans_on_written_items` -
  `left: []` (the stub's empty surface: missing coordinates).
A first suite-red attempt hit `DRIFT (exit 2)` on three COMPILE errors in the
new suite (type-mismatch in a chain, borrow conflict, 2 unused imports) -
harness defects fixed before any Red was claimed; the second run is the real
Red below.

Protocol suite run `python tools/testrun.py --distill -p
ariadne-integration-tests --test rust_language_semantic_index_test` (exit 1,
capture 156 lines): `1 passed / 9 failed / 0 ignored / 1 suites`.
Named failures, all on the stub's empty surface (semantic reason, not
harness): `registered_plugin_fingerprint_is_rust_1` (`left: "v1"`),
`lib_rs_locks_the_crate_root_surface` (stub stamps no context),
`sibling_module_file_locks_its_written_surface`,
`mod_rs_sibling_locks_its_written_surface`,
`nested_user_module_locks_its_written_surface`,
`fixture_spans_carry_exact_line_and_column` (`left: []` - missing
coordinates), `macro_specimen_surfaces_only_written_items` (`left: []`),
`cold_index_writes_one_envelope_per_unit` (production index carries no
written symbol), `warm_reindex_of_unchanged_bytes_is_cache_served` (cold
envelope has no written symbols for the sentinel to ride). The one passing
test at Red, `warm_reindex_never_rewrites_envelope_bytes`, is the reuse
witness - vacuously green against the stub, meaningful at Green; noted, not
hidden. The integration-side caps lock was NOT authored red (D3:
amendment-style byte-pin, added with the flip commit).

## Green (gate 4 checkpoint)

`python tools/testrun.py --distill -p ariadne-integration-tests --test
rust_language_semantic_index_test` (exit 0, first run after gate 3):
`11 passed / 0 failed / 0 ignored / 1 suites` (capture
`target/testrun-capture.txt`, 39 lines). All ten red failures now green on
their first re-run - no span pin needed moving from its authored literal
(the name-token + column+1 convention held against every fixture file,
including the `pub(crate)` attribute case and the glob/rename cases), and
the new integration-side `capabilities_lock_the_semantic_index_flip_literal`
passes with the flip. The reuse witness `warm_reindex_never_rewrites_
envelope_bytes` now runs against a populated written surface (no longer
vacuous).

## Green (gate 5 checkpoint)

`python tools/testrun.py --distill -p ariadne-integration-tests --test
m21_scala_semantic_index_test` (exit 0): `5 passed / 0 failed / 0 ignored /
1 suites`, UNEDITED. Warm/cold + cache-digest witness holds: the `rust-1`
fingerprint is a per-plugin digest input and does not move the C++/Scala
envelope bytes (m18/ar27 suites likewise green inside the gate-6 sweep).

## Gate 6 - census: partial (failed 0 NOT achievable, D3 confirmed)

First bare run `python tools/testrun.py --distill` (exit 1): the
fail-fast sweep died early on a TRANSIENT in-scope-external flake -
`commands::ui::extraction_apply::tests::test_a_valid_token_applies_the_
plan_end_to_end` ("copy fixture: Os { code: 2, NotFound }", a Windows file
race copying the untouched `basic` fixture). Classification: fixture tree
verified clean (`git status --porcelain tests/fixtures` empty, exit 0) and
the test PASSED standalone (`1 passed / 0 failed / 0 ignored / 1 suites`).
Bare re-run then died exactly where D3 predicted, at suite 67:
`capabilities_lock_the_registration_flip_literal` (OUT-of-touch-set
`rust_language_discovery_test.rs:299`, stale R-4 literal).

Census amendment (journal-logged): the handoff's bare command cannot deliver
a full 123-suite sweep while a known external suite is red (fail-fast), and
`testrun.py`'s allowlist carries no `--no-fail-fast`. Full sweep run as
`python tools/distill.py --timeout 3500 --capture target/census-
no-fail-fast.txt -- cargo test --workspace --no-fail-fast`, canonical line
replayed through the documented seam
`python tools/testrun.py --distill --replay target/census-no-fail-fast.txt`
(exit 1):

`1915 passed / 1 failed / 13 ignored / 123 suites`

Attribution against the duty-1 anchor `1886 passed / 0 failed / 13 ignored /
122 suites @ a9b6864`: suites +1 = the new `rust_language_semantic_index_
test` binary; ignored 13 unchanged; passed delta = +19 lang-rust net (24 to
43, gate-3 attribution) +11 new-suite tests -1 the registration flip-lock
flipping passed-to-failed with the mandated flip; 1886 + 19 + 11 - 1 = 1915
by name. The ONE failure is that stale registration lock and nothing else:
the transient cli flake did not recur (cli lib 173/173 green in the sweep),
no other collateral.

Gate-6 verdict clause: `failed 0` is NOT achievable by any in-touch-set
edit - the handoff touch-set omitted the flip-time amendment target (Scala
m23 precedent: the superseded registration lock is amended IN the flip
commit). Per handoff rule "any edit outside = blocker, report don't fix":
NOT touched; board ASK posted to the orchestrator with the one-line
amendment option; in-scope work landed with this evidence attached.

## Green (gate 3 checkpoint)

`python tools/testrun.py --distill -p ariadne-lang-rust` (exit 0):
`43 passed / 0 failed / 0 ignored / 2 suites` (capture
`target/testrun-capture.txt`, 57 lines).
Attribution vs the 24-pass baseline: -1 (`analyze_returns_the_neutral_empty_analysis`
pinned the stub contract - superseded by dispatch, deleted with the stub,
never weakened), +4 `analyzer.rs` (stamps / unparseable-neutral / the red
span pin now exact / Send+Sync), +1 `plugin.rs`
(`analysis_fingerprint_is_rust_1`; caps pin renamed 1-for-1, seam pin
amended 1-for-1 per D4), +8 `parser_items.rs`
(kinds x2 / out-of-line mod span / inline nesting / impl forms / macro
absence / union+use boundary / record-decl wiring), +7 `parser_uses.rs`
(plain / grouped / nested-group / glob / rename-lock / raw-relative-self-
absolute / source order). 24 - 1 + 4 + 1 + 8 + 7 = 43, exact by name and count.

Implementation deviations surfaced during Green (all in-design, none an
assertion weakening): syn 2.0.119's real shapes (`trait_` carries the
negative-impl `Not`, `UsePath` is one-ident + tree, `UseGlob` carries only
`star_token`, qself does not exist in use trees) pinned the final walker
shape; the glob fact sits at the STAR span.

(Section order note: gate-5/6 entries were appended above the gate-4/3
checkpoints by insert-anchoring; chronology is Red -> gate 3 -> gate 4 ->
gate 5 -> gate 6, content unchanged.)

## Gate 7 - hygiene + lock proofs

- `python tools/hygiene_check.py` direct run clean, then
  `python tools/distill.py --capture target/hygiene-r5.txt -- python tools/
  hygiene_check.py` for an explicit verdict: `### generic PASS (exit 0)`
  (capture `target/hygiene-r5.txt`, 173 lines). R1/R2 violations: ZERO
  (full-capture grep for R1/R2/BLOCK found none; every printed row is R3
  warn-only and none names a file of this hand).
- Cache epoch read-check (D1): `CURRENT_ANALYSIS_VERSION` reads `"2.2.0"`
  at `crates/index/src/cache.rs:30` - NOT the `2.1.0` the handoff text
  states (M27 T5 moved it; the m21 suite pins 2.2.0). Zero bytes touched
  under `crates/index` either way; the handoff's zero-bytes rule is the
  binding one.
- Lock zero-diffs `git diff --name-only c54035d..HEAD -- tests/fixtures
  tests/golden crates/index crates/lang-cpp crates/lang-scala tools
  crates/repo` -> EMPTY (exit 0). L4 m00 witnessed green UNEDITED inside
  the sweep; m18-m23/ar27 likewise. L11/L1 repo absent from every diff.
- `git status --porcelain` at this point: exactly the seven touch-set paths
  (3 modified lang-rust src, 2 new parser modules, 1 new suite, 1 journal).
  `Cargo.toml`/`Cargo.lock` ABSENT - the hop added no dependency
  (syn + proc-macro2 were pre-declared and unchanged).

## Landing

Lands as gate-8's single scoped commit
(`feat(lang-rust): syn analyzer, fingerprint rust-1, semantic_index flip,
span truth (rust-language R-5)`; scopes: crates/lang-rust, the new suite
file, this journal), on top of duty-1 `80a2e28`. Its SHA is reported in the
hand's final report and is the R-6 duty-1 ledger clause's quote source.

## Deliberately NOT done

- `rust_language_discovery_test.rs:299` stale registration lock: NOT
  edited (out of touch-set). The flip-time amendment is one literal
  (`semantic_index: true`) - flagged to the orchestrator as the handoff's
  omitted amendment target (m23 Scala precedent amends the superseded flip
  lock IN the flip commit; the R-6 dispatch should carry that one-liner or
  the orchestrator riders it).
- plugin.rs/lib.rs stale header docs still calling the analyzer a stub:
  left untouched per touch-set discipline; R-10 rider material.
- `testrun.py` allowlist unchanged (no `--no-fail-fast` added) - tool
  files are out of touch-set; the distill+replay path is the documented
  seam.
