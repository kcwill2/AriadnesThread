# R-2 Journal - rust-language: spine, part lang-api (AggregationKind::ModDeclaration + advisory row A6)

Handoff of record: `tasks/rust-language-r2-handoff.md`. Milestone track
`rust-language` (UNREGISTERED post-M28, NO M-number), hand 2 of 10, serial.
Dispatch head (start rev): `e862acf` (R-1 landing). Author rev of the shape:
the planning commit above `5aa191e`. Session date 2026-10-26.

## Duty 1 (landed BEFORE any R-2 mutation)

- R-1 landing SHA: `e862acf` - the spine hop (FileKind::RustSource +
  LanguageId::Rust), gated APPROVED per `tasks/rust-language-r1-verdict.md`.
- Duty-1 commit SHA: `f7c2f93` - `review(rust-language): R-1 APPROVED
  verdict + ledger flip (duty 1)`; staged ONLY
  `tasks/rust-language-r1-verdict.md` + the one-line ledger edit, per the
  duty-1 command trio (`distill.py -- git add`, `encoding_guard --staged`
  silent exit 0, `commit_scoped.py`).
- Ledger edit: dated clause appended to the `Rust language plugin` row:
  `R-1 LANDED e862acf APPROVED 2026-10-26, census anchor 1854 passed /
  0 failed / 13 ignored / 119 suites @ e862acf, R-2 dispatched.`
- R-1 finding 1 (journal stray placeholder at
  `tasks/rust-language-r1-journal.md:93-97`) - CLEARED in the sanctioned
  docs-only rider `1cf6786` (`docs(rust-language): clear R-1 verdict
  finding 1 stray placeholder section (sanctioned rider)`; 0 insertions,
  6 deletions - exactly the placeholder line plus the truncated duplicate
  section header and orphan line). Authority: the verdict's own finding-1
  disposition sanctions "a docs-only rider"; the dispatch brief directed the
  clear. Declared deviation: `tasks/rust-language-r1-journal.md` is outside
  the hand's touch-set, so the clear rides its OWN commit, never the duty-1
  commit (which the handoff command-freezes to two paths) and never the
  landing commit (whose scope is three paths).
- Out-of-scope observation recorded here:
  `tasks/rust-language-execution-results.md` is untracked and listed by no
  duty-1 command of this hand; left in the worktree untouched.

## Gate 1 - clean baseline before any R-2 mutation

- `python tools/testrun.py --distill -p ariadne-lang-api` -> exit 0,
  canonical: `47 passed / 0 failed / 0 ignored / 5 suites`.

## Red - JUSTIFIED RED-SKIP (declared)

Precedent trio, cited per the handoff's RGB protocol item 1:

1. `tasks/extraction-symbol-move-sm1-journal.md` (SM-1 red-skip precedent).
2. `tasks/ui-extraction-wizard-w1-journal.md` (W-1 red-skip precedent).
3. `tasks/extraction-consumer-rewrite-cw2-journal.md` (CW-2 red-skip
   precedent).

Justification: no assertion can fail against the pre-hand tree without the
new variant existing - naming `AggregationKind::ModDeclaration` is a
compile error, not a failing test. The A6 row is green-on-landing by
construction: the advisory arm already maps `RustSource -> Rust` since R-1
(`crates/repo/src/identity.rs` `from_file_kind`, verified this session), so
the row PINS a truth the spine already shipped - exactly what an amendment
is for, per the CW-1 amendment-discipline precedent.

Falsifiability statement (per the handoff):
`aggregation_kind_append_is_serde_index_stable` fails if the variant is
renamed (the JSON name changes) or inserted not-last (the historical
names/order pins catch the reorder through the positional contract); the A6
row fails if `from_file_kind`'s arm were later regressed.

## Green - checkpoint after gate 2

- `modeling.rs`: `ModDeclaration` appended LAST after `None`, with the
  mechanism doc (shape sec. 3.3 reason column) and the frozen-indices note
  (pre-existing variants keep positional (bincode) indices 0/1/2; new
  variant takes index 3 at the END).
- `modeling.rs` tests: new pin
  `aggregation_kind_append_is_serde_index_stable` - four unit variants
  serialize to their names; first three keep their historical names;
  message names INV-12/L11.
- `identity.rs`: the A6 tuple `(FileKind::RustSource, LanguageId::Rust),`
  appended to the `cases` array of
  `language_of_file_kind_maps_every_file_kind_exhaustively`; the :15
  re-export list byte-identical (verified in the diff: the hunk touches
  only the cases array).
- RISK-AGGKIND watch: gate 3 compiles the whole workspace; no surprise
  exhaustive `AggregationKind` site surfaced (no STOP condition).
- Gate 2: `python tools/testrun.py --distill -p ariadne-lang-api` ->
  exit 0, canonical `48 passed / 0 failed / 0 ignored / 5 suites` =
  baseline 47 + 1 (the new pin); the advisory exhaustive test green with
  its +1 row, UNEDITED otherwise.

## Green - checkpoint after gate 3 (census) and gate 4 (hygiene)

- Gate 3: `python tools/testrun.py --distill` -> exit 0, canonical
  `1855 passed / 0 failed / 13 ignored / 119 suites`; capture
  `target/testrun-capture.txt` (3234 lines). Reconcile from the duty-1
  ledger anchor `1854 passed / 0 failed / 13 ignored / 119 suites @
  e862acf`: delta +1, fully attributed by name to
  `aggregation_kind_append_is_serde_index_stable`; failed 0; ignored 13
  unchanged; suites flat at 119. Goldens, m00, m17, lang-cpp, lang-scala,
  tools green UNEDITED via this census (their bytes absent from every
  diff of this hand - see Lock proofs).
- Gate 4: `python tools/hygiene_check.py` exit 0; every printed line is
  warn-only R3, pre-existing; NO line names `crates/lang-api/src/
  modeling.rs` or `crates/lang-api/src/identity.rs`. Form recorded:
  `python tools/distill.py -- python tools/hygiene_check.py` ->
  `### generic PASS (exit 0)`, capture 173 lines (the direct invocation
  ran identically; the distill form supplies the explicit exit line).
- Refactor pass: doc wording already plain; no renaming or restructuring
  needed; no re-run consumed beyond the gate sequence above.

## Lock proofs

- L11 (pure append): the `modeling.rs` enum hunk adds 7 lines after
  `None` (doc + variant), deletes 0, reorders none (distill capture of
  `git diff -U2 -- crates/lang-api/src/modeling.rs` reviewed line by
  line); the `identity.rs` diff is exactly one added array tuple.
- L1-L9: at the landing HEAD, `git diff --name-only e862acf..HEAD` shows
  exactly: `tasks/rust-language-r1-verdict.md`, `tasks/milestone_ledger.md`
  (duty 1), `tasks/rust-language-r1-journal.md` (sanctioned rider),
  `crates/lang-api/src/modeling.rs`, `crates/lang-api/src/identity.rs`,
  `tasks/rust-language-r2-journal.md` (landing). Nothing outside the
  touch-set plus the sanctioned rider; no `Cargo.lock` delta (std-only,
  cargo rewrote nothing); no `crates/repo/**`, `crates/lang/**`,
  `crates/lang-rust/**`, `crates/cli/**`, `tests/**`, `tools/**` hunk.
- L12: `python tools/encoding_guard.py --staged` at the commit cycle
  (journal the form - see Commit cycle below).

## Commit cycle

- Staged via `python tools/distill.py -- git add --
  crates/lang-api/src/modeling.rs crates/lang-api/src/identity.rs
  tasks/rust-language-r2-journal.md`.
- `python tools/encoding_guard.py --staged` - form: STAGED.
- `python tools/commit_scoped.py --scope crates/lang-api/src/modeling.rs
  --scope crates/lang-api/src/identity.rs
  --scope tasks/rust-language-r2-journal.md -m "feat(lang-api): append
  ModDeclaration to AggregationKind and the A6 advisory row (rust-language
  R-2 spine)"` - journal rides inside; never `git add .`.

## Gate inventory (itemized, per the declared budget)

- Gate 1 baseline: 1 invocation of `testrun.py --distill -p
  ariadne-lang-api`.
- Gate 2 Green: 1 invocation of `testrun.py --distill -p ariadne-lang-api`.
- Gate 3 census: 1 invocation of `testrun.py --distill`.
- Gate 4 hygiene: 1 invocation `hygiene_check.py` direct + 1 distill form
  (explicit exit line; declared as 4 scoped + commit cycle - the distill
  form is the same gate's evidence, matching the R-1 recorded practice).
- Commit-cycle trio: `distill.py -- git add` + `encoding_guard --staged` +
  `commit_scoped` (L12 evidence inside).
- Refactor re-runs consumed: 0.

## Deviations (declared)

- (a) Sanctioned docs-only rider `1cf6786` touched
  `tasks/rust-language-r1-journal.md` - outside the literal touch-set,
  authorized by R-1 verdict finding-1 disposition ("clear via a docs-only
  rider") and by the dispatch brief; landed as its own commit so the
  duty-1 and landing commits keep their command-frozen scopes.
- (b) Refactor step consumed no re-run: the authored docs/tests needed no
  change after the Green pass.
- Landing SHA relay: the landing commit's SHA is NOT journalled inside
  itself (self-reference impossible; placeholders banned by R-1 finding 1
  class); it is relayed in the handoff report, and R-3's duty 1 cites this
  file + that SHA per the handoff's Landing SHA relay clause.
