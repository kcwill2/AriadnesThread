# R-10 Handoff - rust-language: full-parity acceptance suite, the docs riders, closeout-facing evidence packaging

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). Hand
10 of 10, serial. The last hand: `rust_language_parity_test.rs` binds the
whole arc, `docs/adding-a-language.md` gains its dated riders (the ONLY
docs file the shape permits), and the closeout evidence is packaged for the
`milestone-lifecycle` skill's closeout hop. Start rev: your dispatch head
(R-9's landing + verdict via duty 1).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/rust-language-shape.md` sec. 8 (riders 1-5 - YOUR doc deltas,
   dated), sec. 2 INV-1..INV-15 (the suite binds them), sec. 6 RISK-SELFSCAN
   (fixture-path grep evidence lands in YOUR suite), RISK-HONEST-LIMITS-STALE
   (rider 3 text duty). Kickoff sec. 4 acceptance commands (you run their
   per-hand scoped forms; the closeout hop runs the workspace forms).
3. Models: `tests/integration/tests/m18_cpp_plugin_parity_test.rs` and
   `m24_language_identity_test.rs` (parity + identity idioms, zero-diff
   models); `docs/adding-a-language.md` (229 lines - YOUR file; its stale
   refs: the dispatch paragraph at :21-26 cites `optimize.rs` 247/272-281 and
   `ui.rs` 359-368 - current sites are `commands/plan.rs:203`,
   `optimize.rs:305`, `commands/ui.rs:419`, hooks registry-dispatched since
   AR-19; the manifest snippet at :28-30 "three deps and no more"; the
   "two-element list" guard wording at :162; the honest-limits bullets at
   :201-220; the Step-5 naming rule at :141-155).
4. Your duty-1 targets: `tasks/rust-language-r9-verdict.md` + ledger.

## Duty 1
Land R-9's verdict + ledger flip (`R-9 LANDED <SHA> APPROVED <date>, census
anchor <line> @ <SHA>, R-10 dispatched (final hand).`).

## Touch-set (hard)
- tasks/rust-language-r10-journal.md (create)
- tasks/rust-language-r9-verdict.md + tasks/milestone_ledger.md (duty 1 only)
- tests/integration/tests/rust_language_parity_test.rs (NEW suite)
- docs/adding-a-language.md (dated riders - the ONLY docs file; `docs/adr/**`
  and every other doc in NO hunk)

You MUST NOT touch: ANY `crates/**` path (zero src bytes this hop - your
bench exemption depends on this), `tests/fixtures/**`, goldens, `tools/**`,
`tests/Cargo.toml`, every existing suite. No `git stash`/branch change.

## What to build

1. Suite `tests/integration/tests/rust_language_parity_test.rs`
   (milestone-named - the mNN trap named explicitly in its module doc;
   mirrors m18's parity shape at Scala-parity breadth), asserting per
   capability against the shipped plugin (all-eight-true dispatch):
   - enumeration + identity bytes final-state witness (INV-1/INV-4 re-bind
     through the real registry - cheap re-pin, cites m24, edits nothing);
   - discovery/graph/index/load data re-bindings (fixture `rust-basic`,
     cheap re-drive, not re-litigation of the per-cap suites - each
     assertion names the hand that owns its truth);
   - capability monotonicity final witness (INV-5): the all-true +
     toolchain-false struct, byte-exact, plus m19-style in-suite pin;
   - RISK-SELFSCAN evidence: the suite + every rust_language_* suite operate
     on `tests/fixtures/rust-basic` copies - journal-quote the grep that the
     milestone suites never point at the repo's own tree (a fixture-path grep
     of `tests/integration/tests/rust_language_*.rs` for `tests/fixtures/`
     and no `env::current_dir`-style own-tree roots);
   - mixed-language honesty (RISK-EMPTY-CLAIMANT): cpp+rust in one repo
     keeps the honest existing multi-language error - assert the CURRENT
     behavior unchanged (do NOT fake end-to-end mixed planning - the tree's
     behavior is the contract; shape sec. 6).
2. Docs riders into `docs/adding-a-language.md` (shape sec. 8, each DATED
   2026-10-26, plain-ASCII, sentence-per-line where prose):
   - rider 1 (manifest snippet): the lang-rust deliberate parser-deps
     extension (`syn`/`proc-macro2`[span-locations]/`unicode-ident`,
     workspace-pinned), why (D-2 correctness guarantee), what the m17 guards
     STILL forbid (the four-name manifest list + the transitive list) - the
     trio-for-three rule stays the lang-scala baseline, not a universal cap.
   - rider 2 (dispatch paragraph): refresh the stale :21-26 line refs to
     203/305/419, state the AR-19 hook-dispatch fact and the informational
     CyclePolicy stance ("a language enrolls by overriding the trait hooks;
     these literals stayed").
   - rider 3 (honest limits): dated third-plugin update - a third plugin now
     exists, still this team, R1 exercised on a foreign-to-the-spine language
     for the first time; dynamic loading stays parked (the bullet's own
     claim kept TRUE, not deleted).
   - rider 4 (guard wording): "two-element list" -> three (append discipline
     from INV-11 named as the pattern).
   - rider 5 (Step 5): name `rust_language_*` as the milestone-named example
     of the mNN-trap rule.
3. Closeout packaging (journal duty, no extra files): assemble for the
   closeout hop the ledger-ready evidence digest - per-hand landing SHAs
   relay (from the ledger clauses your duty 1 extended), the final census
   line, the bench exemption proof (below), and the closeout checklist
   pointer: skill `milestone-lifecycle` (execution-results addendum +
   walkthrough committed + memory mirror same-commit law). R-10's OWN verdict
   + execution-results land in the closeout commit itself (extraction-
   symbol-move precedent) - do NOT author them here.

## RGB protocol
1. Red - JUSTIFIED RED-SKIP (SM-1 / W-1 / CW-2 trio, cited): every assertion
   the parity suite binds was already RED at its owning hand and journal-
   recorded there (the suite codifies shipped truth; a suite that invents no
   new claim cannot manufacture a fresh red without re-breaking a green
   hand). Falsifiability statement (journal it verbatim): the suite's
   all-true caps bind and mixed-language-current-behavior bind fail on the
   R-9 tree MINUS any single flip - the reviewer may sanity-audit by
   reasoning from the R-9 journal, no re-red is run (per-hand reds are the
   milestone's red record; INV-13 attribution chain closes here).
2. Green: suite + riders land green.
3. Refactor: rider prose plain; suite dedup helpers; re-run.

## Hygiene
Suite ~400 (< 1000); docs prose wraps (source-line rule applies to
Markdown: no line > 2000 chars, sentence-per-line body - AGENTS.md);
`hygiene_check.py` exit 0 (it walks .rs/.py/.html/.js - the .md is guarded
by the line rule + the encoding gate); pure ASCII.

## Frozen locks
- YOUR BENCH EXEMPTION PROOF (kickoff sec. 4 assets/docs-only exemption):
  `git diff --name-only <start>..HEAD -- crates` EMPTY (stronger than a
  src-only filter - you touch ONLY tests/integration + docs). Journal this
  proof verbatim; the reviewer re-runs it.
- L5/L6: m18-m23, m24, m17 absent from any hunk; L7/L8/L9 lang-cpp/
  lang-scala/tools absent; L2/L3 goldens/fixtures empty-diff; L4 m00 green
  UNEDITED (census); L1 repo absent; L12 `encoding_guard --all` exit 0
  (docs included - no mojibake in the riders).
- `docs/adr/**` in NO hunk (shape MUST NOT EDIT list).

## Gates (distilling entry points only, 600000 ms)
1. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   rust_language_parity_test` - Green on arrival (declared red-skip).
2. `python tools/testrun.py --distill` - FULL milestone census: failed 0,
   ignored 13, suites +1 vs R-9 (parity file), every passed delta
   attributed; this line is the milestone ANCHOR candidate quoted in the
   closeout.
3. `python -m unittest discover -s tools` - expect `Ran 296 ... OK` (the
   tools suite is zero-diff; this re-binds kickoff sec. 4's CI form at the
   final hop).
4. `python tools/distill.py -- python tools/ui_smoke.py` - expect
   `ui_smoke: PASS`.
5. `python tools/hygiene_check.py` - exit 0.
6. Commit cycle: `python tools/encoding_guard.py --staged` (or `--all`) then
   `python tools/commit_scoped.py --scope
   tests/integration/tests/rust_language_parity_test.rs --scope
   docs/adding-a-language.md --scope tasks/rust-language-r10-journal.md -m
   "test(rust-language): full-parity acceptance suite + dated guide riders
   (R-10)"`. Journal inside.

Declared gate budget: 5 + commit trio. BENCH: EXEMPT this hop with the
zero-`crates`-diff proof above (kickoff sec. 4) - no bench run belongs to
this hand or its reviewer pass.

## Discipline clauses
UTF-8 Write/Edit-only (the riders are the riskiest bytes in the milestone
for mojibake - the guide file already contains legitimate UTF-8 section
signs and arrow glyphs
histories; you ADD pure ASCII only, and never rewrite a pre-existing
non-ASCII line's bytes); touch-set hard; journal per Green gate; STOP +
`checkpoint_guard.py resume --spec tasks/rust-language-r10-handoff.md --scope
tests --scope docs --cmd ...`; SHA relay (the closeout hop cites it); F14.

## Report back
Duty-1 + landing SHAs + `git show --stat`; the green canonical lines
(gates 1-4) + capture path; the bench-exemption proof line; the closeout
evidence digest (per-hand SHA relay table + final census + anchor math);
deviations. No raw output.

## Reviewer acceptance pass
NO bench run (exemption proof re-run instead); per-commit vs touch-set;
rider-by-rider adjudication against shape sec. 8 (each DATED); guide's
pre-existing non-ASCII bytes unchanged (`encoding_guard --all` + a
`git show` eyeball on the touched hunks); mixed-language honesty assertion
adjudicated as CURRENT-behavior pin (not a feature claim); census anchor
declaration recommended in the verdict ("NEW ANCHOR `<line> @ <SHA>`" wording
for the closeout ledger).

Reviewer-gate reference: `review-gate` skill. Extra MUSTs: no `crates/**`
hunk anywhere in the landing range (the exemption depends on it); the closeout
checklist (skill `milestone-lifecycle`: execution-results addendum, ledger
closeout flip, walkthrough committed via `walkthrough-commit`, memory mirror
same-commit) is the NEXT hop's - R-10 must not pre-run it.

Sizing (planner, itemized): suite ~400, riders ~120, journal ~20 - total
~450-560 produced lines; 5 gates + trio; tests-dir + docs file declared
vertical (no src at all). No split: the parity suite's claims and the guide's
truth are the same statement about the same shipped state.
