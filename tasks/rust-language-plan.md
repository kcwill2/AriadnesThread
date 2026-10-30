# rust-language - Planner's Sized Handoff Plan (2026-10-26)

Companion to `tasks/rust-language-shape.md` (shape binding; the arc SKETCH is
the planner's to resize - shape sec. 5 opens with exactly those words) and
`tasks/rust-language-kickoff.md` (locks L1-L12, amendments A1-A8, acceptance
commands). Branch `balsam-basil`; authored above HEAD `5aa191e` (docs hops
above the anchor keep the census at `1853 passed / 0 failed / 13 ignored /
119 suites @ 522a770` - the docs-only kickoff/shape/planning commits move
zero test bytes).

## 1. Arc resize record (shape sketch R-1..R-6 -> hands R-1..R-10)

The shape's R-1 sketch spans `crates/repo` + `crates/lang-rust` +
`crates/pipeline` + `crates/cli` + `tests/fixtures` - multiple crates, over
the Planning Law's scope axis; it is split as the mission directs. Every
shape decision of record (syn parser, full-parity arc, fixtures-only, no
M-number, the modeling table, the frozen trio) is UNTOUCHED by the resize.

| hand | file | content |
|---|---|---|
| R-1 | `tasks/rust-language-r1-handoff.md` | spine: `FileKind::RustSource` + `LanguageId::Rust` (crates/repo), amendment A5 |
| R-2 | `tasks/rust-language-r2-handoff.md` | `AggregationKind::ModDeclaration` + A6 (crates/lang-api) |
| R-3 | `tasks/rust-language-r3-handoff.md` | lang-rust crate skeleton, unregistered, caps ALL-false; workspace deps + A8 (m17 appends, same commit as manifest - INV-11) |
| R-4 | `tasks/rust-language-r4-handoff.md` | registration line + `.rs` flip + A1/A2/A3/A4/A7 + fixture `rust-basic` (approval path) + discovery suite (INV-4/6/9) |
| R-5 | `tasks/rust-language-r5-handoff.md` | syn analyzer (pre-split `parser_items`/`parser_uses`), fingerprint `"rust-1"`, `semantic_index` flip, span truth (INV-7), semantic-index suite |
| R-6 | `tasks/rust-language-r6-handoff.md` | resolver (mod tree + use paths), `dependency_graph` flip, graph suite (RISK-CYCLE data assert) |
| R-7 | `tasks/rust-language-r7-handoff.md` | `load_metric` flip + INV-14 data-path proofs (manifest/CLI vocabulary/UI label; ui/report/artifact zero-diff), load suite |
| R-8 | `tasks/rust-language-r8-handoff.md` | real extractor + real rewriter incl. the RISK-MODTREE parent-`mod` consumer-hook mechanism (shape 5a); caps WAIT (all-8-still-partial) |
| R-9 | `tasks/rust-language-r9-handoff.md` | planning hooks + probe-only validator (INV-10 a/b/c) + FINAL all-true caps byte-lock + symbol-isolation suite (shape 5b) |
| R-10 | `tasks/rust-language-r10-handoff.md` | `rust_language_parity_test.rs` full-parity acceptance + the five dated guide riders (shape sec. 8) + closeout evidence packaging |

Shape-sketch mapping: sketch R-1 = my R-1..R-4; sketch R-2 = R-5; sketch R-3 =
R-6; sketch R-4 = R-7; sketch R-5 = R-8 + R-9 (the shape's own pre-authorized
5a/5b cut); sketch R-6 = R-10. The capability schedule relocates one notch
(capability pins: R-3 all-false skeleton -> R-4 `discovery` -> R-5
`+semantic_index` -> R-6 `+dependency_graph` -> R-7 `+load_metric` -> R-9
`+planning/extraction/rewriting/validation` + the final literal) - shape
sec. 3.4 assigns the per-hand schedule to the planner ("planner owns").

## 2. Planner verification findings against the shape (all other shape claims re-verified true)

- FALSE-IN-COMPLETENESS (shape sec. 3.1 A5 inventory): flipping `.rs` also
  flips the matrix row `("a.h.rs", FileKind::Other)` at
  `crates/repo/src/types.rs:114` (`Path::extension()` of `a.h.rs` is `rs`).
  A5 must amend :103 AND :114 - encoded in R-1 (reviewer blocker on a miss).
- CONFIRMED-with-choice: `from_file_kind` (`repo/src/identity.rs:133`) is the
  ONLY exhaustive `FileKind` match in the workspace (grep-verified 2026-10-26)
  -> the spine enum + arm fit one hand with no compile breakage outside it.
  No production `match` on `AggregationKind` exists (construction/test sites
  only) -> the R-2 append is compile-neutral as the shape claims.
- CONFIRMED: `.rs` claimed by neither shipped detector; registration order
  appends last; `planning_hooks` dispatch (:276) enrolls Rust via trait
  overrides with ZERO cli edit; `tests/integration` package name
  `ariadne-integration-tests` (scoped gate form); Scala lock-test pattern
  `capabilities_match_the_m23_flip_literal` (lang-scala plugin.rs:431);
  Scala plugin src = 4819 lines (calibration for the estimates below).

## 3. Hard serialization edges (each with its because)

R-1 -> R-2: A6's advisory row needs `FileKind::RustSource` + `LanguageId::Rust`
(one hand earlier; both live in repo). R-2 -> R-3: `plugin.rs`'s modeling
literal references `AggregationKind::ModDeclaration`. R-3 -> R-4: the
registration line names `RustPlugin` - the crate + manifest must exist
(plus A8's INV-11 same-commit law pins the guards to the manifest hop). R-4
-> R-5..R-9: discovery/identity truth (`detect_language` answers) is the
fixture-suite substrate; the caps-literal chain is strictly monotone, so no
later flip can land before an earlier one. R-5 -> R-6: resolver edges carry
analyzer spans (INV-7 end-to-end). R-8 -> R-9: the all-true literal may not
precede extractor/rewriter (INV-5); the shape's own 5a->5b order. R-9 ->
R-10: the parity suite binds all-eight-true. Each hand N+1's duty 1 lands
hand N's APPROVED verdict + ledger flip - the chain is reflog-verifiable at
every link (per-hand landing SHA relay + `git reflog` tip check; phantom-
narration countermeasure per the reflog-SHA cross-check correction).
NO hand is parallel-safe with another: every adjacent pair shares either a
file (`plugin.rs` caps chain), a suite substrate (fixture), or the ledger.
Two handoffs touching the same file are never parallel - there is no
disjoint-file adjacency in this arc.

## 4. Amendment routing (A1-A8 -> landing hand)

| amendment | landing hand | because |
|---|---|---|
| A5 (types.rs :103 + :114 + new case row) | R-1 | the `.rs`->RustSource flip is the commit that turns :103 AND :114 red; same hand greens them |
| A6 (lang-api advisory row) | R-2 | additive amendment to the shim; needs R-1's two spine symbols |
| A8 (m17 :271 + :486 appends) | R-3 | INV-11: both appends land the SAME commit as `crates/lang-rust/Cargo.toml` (guards self-skip a missing manifest - append-late is the violation) |
| A1, A2 (cli enum vec, parse set, closed set, error text) | R-4 | registration + closed-set arm flip them together |
| A3, A4 (unclaimed.rs witnesses -> `.f90`) | R-4 | they go red-exact-at-registration (`pin_plan_backend_honest_errors...` / `planning_hooks_answer_honestly...` claim `.rs` is unclaimed); pre-flip edits land in the flip commit (CW-1 amendment discipline, inverted-falsification note in R-4) |
| A7 (m19 :217 vec) | R-4 | same registration flip (`[Cpp, Scala]` -> `[Cpp, Scala, Rust]`) |

No hand lands its own red: every red-turning edit rides in the commit that
greens it again (mission rule honored; the R-4 journal captures red BEFORE
the flip edit lands).

## 5. Per-hand produced-line estimates vs the ~1200 ceiling

Estimates itemized from the Scala calibration (4819 src lines + suite sizes
m19 283 / m20 258 / m21 575 / m22 457 / m23 831) minus the Rust-specific
savings (`syn` replaces Scala's scanner/imports/directive_parser trio = ~1457
of its lines; no extends channel; constraints final-empty). Milestone total
~6.6k-8.3k lines incl. fixtures/journals - consistent with the mission's
~8.4k Scala-grade expectation at 10 hands.

| hand | est. produced lines | margin note |
|---|---|---|
| R-1 | ~60-110 | huge |
| R-2 | ~35-60 | huge |
| R-3 | ~1150-1350 | AT PRESSURE - declared; split trigger at >~1350 moves shaper.rs+tests (~280) to R-4 (clean interim: unregistered crate, `shaper()` None; R-4 absorbs +330) |
| R-4 | ~560-750 | ok |
| R-5 | ~1050-1250 | at the ceiling's edge - analyzer pre-split keeps any single file bounded; no further cut exists (fingerprint+analyzer+INV-7 oracle are one hop) |
| R-6 | ~700-900 | ok |
| R-7 | ~270-330 | huge |
| R-8 | ~950-1150 | ok (helper module contingency named in touch-set) |
| R-9 | ~1000-1200 | at the edge - no cut avoids a false literal or an unaccepted flip |
| R-10 | ~450-560 | ok |
Cargo.lock regenerations (R-3) are machine bytes, declared excluded from the
hand-written budget; reviewer rule keeps the file in the touch-set.

## 6. Per-hand scoped gate lists vs the ~10 ceiling (all distilling entry points)

Counts are scoped invocations + the commit trio (encoding_guard,
commit_scoped; the journal rides inside per the 2026-10-06 M28 law):
R-1: 6+trio | R-2: 4+trio | R-3: 5+trio | R-4: 10+trio (AT CEILING, declared)
| R-5: 7+trio | R-6: 7+trio | R-7: 7+trio | R-8: 6+trio | R-9: 8+trio |
R-10: 5+trio. Gate forms used (kickoff sec. 4 canonical set):
`python tools/testrun.py --distill -p <crate>` and `--test <suite>` scoped
forms (`-p ariadne-lang-rust`, `-p ariadne-integration-tests`, `-p
ariadne-cli`, `-p ariadne-pipeline`, `-p ariadne-repo`, `-p
ariadne-lang-api`, `-p ariadne-validate`), `python tools/hygiene_check.py`,
`python tools/encoding_guard.py --staged/--all`, `python tools/distill.py --
python tools/ui_smoke.py` (R-7, R-10), `python -m unittest discover -s tools`
(R-10 only, CI-form re-bind), the INV-9 grep-zero witness (R-4), and
`python tools/distill.py -- python tools/bench_compare.py` (REVIEWER pass -
see stance). The `--baseline` FILE form stays superseded everywhere (M23.5 F9
errata; `--distill` substitute governs, identical count semantics).

Bench stance (kickoff sec. 4 encoded verbatim, no invented policy): RUN fresh
at EVERY reviewer hop that touches any `crates/*/src` byte - hands R-1..R-9;
fully-run stance per the f20ad35-onward precedent; WARNs recorded, NOT
attributed (PR-2); `bench_baseline.json` operator-only, NO baseline and NO
corpus change (D-3: benches stay on existing fixture corpora; the `.rs` flip
re-baselines nothing). R-10 (and only R-10) is the assets/docs-only exempt
hop with the `git diff --name-only <start>..HEAD -- crates` EMPTY proof
(journal + reviewer re-run).

## 7. Census expectation math chain (from `1853 / 0 / 13 / 119 @ 522a770`)

Rules (INV-13): every hop quotes the canonical `testrun.py --distill` line;
passed >= previous anchor; failed 0; ignored 13 held (no hand adds an
ignored test); suites rise ONLY by named new test binaries (+1 per new
integration file; +1 per new crate lib binary, and a Doc-tests group if
cargo prints one for lang-rust - attributed as it appears). Milestone
expectation (kickoff sec. 4): ">= 1853 passed, 0 failed at every hop; every
delta attributed per hand; zero foreign residue."

Predicted per-hand deltas n_i (attribution names, not just numbers - the
journal + verdict carry the new-test list):
- R-1: +2..+6 tests (the item-7 pin + matrix-row edits are not new tests);
  suites 119 -> 119. Running expectation `1855..1859 / 0 / 13 / 119`.
- R-2: +1..+3 (serde-index pin); suites flat.
- R-3: +35..+50 crate in-src tests; suites 119 -> 120 (or 121 if a
  `Doc-tests ariadne-lang-rust` header appears - attribute, don't assume).
- R-4: +18..+24 (discovery suite ~14-18 incl. the two m24-mirror identity
  tests; amended cli/m19 tests are NOT new counts); suites +1.
- R-5: +40..+55 (analyzer in-src pins ~25-35 incl. fingerprint + span
  literals; semantic-index suite ~12-16); suites +1.
- R-6: +28..+35 (resolver in-src ~15-20; graph suite ~10-12); suites +1.
- R-7: +14..+20 (load suite ~10-12 + the three INV-14 data-path proofs);
  suites +1.
- R-8: +25..+35 (extractor/rewriter/mod-tree in-src); suites FLAT (declared
  explicitly in R-8's gate 5 - no new file this hop).
- R-9: +40..+50 (hooks/validator in-src incl. INV-10 a/b/c + the final caps
  lock ~22-28; isolation suite ~14-18); suites +1.
- R-10: +10..+15 (parity suite); suites +1.

Chain (running lower..upper bound): 1853 -> R-1 `1855..1859` -> R-2
`1856..1862` -> R-3 `1891..1912 / 120..121` -> R-4 `1909..1936 / 121..122`
-> R-5 `1949..1991 / 122..123` -> R-6 `1977..2026 / 123..124` -> R-7
`1991..2046 / 124..125` -> R-8 `2016..2081 / 124..125` -> R-9
`2056..2131 / 125..126` -> R-10 `2066..2146 / 126..127`.
Milestone-expectation summary the kickoff placeholders asked for:
`+<n1> R-1 .. +<n10> R-10` = the per-hand ranges above; terminal ANCHOR
candidate `~2066..~2146 passed / 0 failed / 13 ignored / 126..127 suites @
<R-10 SHA>` (R-10's verdict recommends the verbatim NEW ANCHOR wording).
Any hop whose canonical line exceeds the upper bound, or moves ignored, or
adds an unnamed suite, is a HOLD-and-explain - not a silent carry.

## 8. Ledger / memory duty assignments (per-hand, extraction-symbol-move precedent)

- Landing of THESE docs + the ledger planning flip: the orchestrator's
  commit_scoped hop (kickoff sec. 8 form, scopes += the plan + 10 handoffs),
  memory mirror `ledger.milestone.rust-language` same-commit law.
- R-2..R-10 duty 1: land the prior hand's APPROVED verdict + the one-line
  ledger flip (landing SHA + verdict + census anchor + "R-<n> dispatched");
  R-5's duty 1 additionally carries the dated fixture-approval note (L3 /
  M27-T0b path discharged via R-4's verdict).
- R-10's own verdict + execution-results + the ledger CLOSEOUT flip land in
  the closeout commit itself (skill `milestone-lifecycle`; walkthrough
  committed via `walkthrough-commit`) - R-10 must not pre-run it.

## 9. Scope guards restated for implementers (all hands)

No M-number anywhere in names/paths (unregistered track; ledger is numbering
authority). Integration suite names: `rust_language_*` - the mNN prefix is
NOT the milestone (guide trap, named in R-4/R-10 handoffs). All authored
bytes pure ASCII; Write/Edit tools only (PowerShell file writes banned);
`commit_scoped.py` per hand (never `git add .`); `CURRENT_ANALYSIS_VERSION`
stays `2.1.0` (M27 freeze; R-5 lock); `WireLanguageForm`/`legacy_rank`/
`language_transparent_frame` zero hunk (Rust rides the tag-3 escape);
`analysis_fingerprint() == "rust-1"` lands in the first analyzer-shipping
commit (R-5) and `CURRENT_ANALYSIS_VERSION` is never the vehicle.
