# R-2 Verdict - rust-language spine, part lang-api (AggregationKind::ModDeclaration + advisory row A6)

Verdict: APPROVED

Handoff of record: `tasks/rust-language-r2-handoff.md`. Chain under review from
start rev `e862acf`: duty-1 landing `f7c2f93`, sanctioned rider `1cf6786`,
landing `3d4dfae`, orchestrator rider `1dd042d`. Reviewed fresh on
2026-10-26; the implementer relay was treated as claims, every measurable item
re-run.

## Ground truth (before judging)

- `python tools/checkpoint_guard.py status`: HEAD `1dd042d`, branch
  `balsam-basil`, clean tree (no dirty, no untracked); this worktree dirty=0.
- `git log --oneline e862acf..HEAD`: exactly the four commits above, subjects
  matching the dispatch brief's chain description.

## Per-commit review (oldest-first)

### f7c2f93 - duty 1

`git show --stat`: `tasks/milestone_ledger.md | 2 +-` and
`tasks/rust-language-r1-verdict.md | 201 +++` - exactly the two paths the
duty-1 command trio names. Ledger hunk (verified line by line): one table row
rewritten solely to append the dated clause `R-1 LANDED e862acf APPROVED
2026-10-26, census anchor 1854 passed / 0 failed / 13 ignored / 119 suites @
e862acf, R-2 dispatched.` Cross-checked against the landed R-1 verdict:
verdict word APPROVED, landing `e862acf`, canonical census
`1854 passed / 0 failed / 13 ignored / 119 suites`, review date 2026-10-26 -
every element of the clause matches the adjudicated state. Verdict blob is
byte-stable from `f7c2f93` to HEAD (no later commit touches it; name-only
diffs below).

Duty-1 completeness adjudication: the handoff duty-1 text stages ONLY
`tasks/rust-language-r1-verdict.md` + the one-line ledger edit, and the string
`execution-results` appears NOWHERE in the handoff (grep: no match). Duty 1 is
therefore COMPLETE as written without an execution-results file. The
implementer correctly recorded the untracked
`tasks/rust-language-execution-results.md` as an out-of-scope observation
(journal:31-33) and left it untouched; the orchestrator later landed it as
separate rider `1dd042d` (below). No finding against the implementer.

### 1cf6786 - sanctioned rider

`git show --stat`: `tasks/rust-language-r1-journal.md | 6 ------` - deletions
only, exactly the stray placeholder `(to be appended per Green gate)`, the
truncated duplicate header `## Gate inventory (itemized per-hand counts,
M28 law)`, and the orphan line `- Gate 1 baseline: 1 invocation.` (plus their
blank separators). This is precisely the R-1 verdict finding-1 scope; the
verdict's disposition pre-authorizes "clear via a docs-only rider". Deviation
(a) disposition below.

### 3d4dfae - the landing

`git show --stat`: `crates/lang-api/src/identity.rs | 1 +`,
`crates/lang-api/src/modeling.rs | 28 +`, `tasks/rust-language-r2-journal.md |
159 +` - exactly the two lib files + journal, 188 insertions, ZERO deletions.

- `modeling.rs` hunk 1 (`@@ -17,6 +17,13 @@`): adds 7 lines between `None,`
  and the enum's closing brace - 6 doc lines + `ModDeclaration,`. Zero
  deletions, zero reordered lines. Final enum `Header, PackageObject, None,
  ModDeclaration` matches the shape sec. 3.1 contract literal verbatim. THE
  LOCK of this hand: L11 pure append HOLDS - pre-existing variants occupy
  indices 0/1/2 unmoved; `ModDeclaration` takes index 3 at the END (INV-12).
  Doc mirrors the shape sec. 3.3 reason column (PackageObject would lie, None
  would understate) and carries the frozen-indices note.
- `modeling.rs` hunk 2 (`@@ -201,4 +208,25 @@`): append inside the tests
  module after the last existing test's closing brace - blank + `#[test]` +
  `aggregation_kind_append_is_serde_index_stable` (21 added lines, zero
  deleted). Name verbatim per handoff item 2; four unit variants asserted to
  serialize to `"Header"`, `"PackageObject"`, `"None"`, `"ModDeclaration"`;
  message names INV-12/L11. See finding 1 on what this pin does and does not
  catch.
- `identity.rs` single hunk (`@@ -40,6 +40,7 @@`): exactly one added line
  `(FileKind::RustSource, LanguageId::Rust),` at the tail of the `cases`
  array of `language_of_file_kind_maps_every_file_kind_exhaustively` - an
  APPEND-only change to the tests module; verbatim per amendment A6
  (kickoff A-list). The :15 re-export
  `pub use ariadne_repo::identity::{LanguageId, CPP, OTHER, SCALA};` is
  outside every hunk - byte-identical (the commit's identity.rs diff is the
  single tuple). No `RUST` symbol added, per the handoff's
  `LanguageId::Cpp`-mirroring note.
- RISK-AGGKIND: fresh full census compiled the whole workspace exit 0; grep
  shows every `AggregationKind` occurrence is an import, a construction site,
  or a `#[cfg(test)]` module - no exhaustive `match` surfaced anywhere. No
  STOP condition; no planner amendment pending.

### 1dd042d - orchestrator rider

`git show --stat`: `tasks/rust-language-execution-results.md | 42 +` - a
single `tasks/` path, docs-only, landing the R-1 reviewer's per-hand
acceptance table (content verified). Landed by the orchestrator, not the
implementer, after the implementer correctly refused to touch it. Adjudicated
docs-only; no finding.

## Commit-vs-touch-set

- Whole chain `git diff --name-only e862acf..HEAD`: `crates/lang-api/src/
identity.rs`, `crates/lang-api/src/modeling.rs`, `tasks/milestone_ledger.md`,
`tasks/rust-language-execution-results.md`, `tasks/rust-language-r1-journal.md`,
`tasks/rust-language-r1-verdict.md`, `tasks/rust-language-r2-journal.md`.
Inside the touch-set, plus the sanctioned rider path (`r1-journal`, deviation
a) and the orchestrator's own rider path (`execution-results`, not the
implementer's commit).
- Post-duty-1 `git diff --name-only f7c2f93..HEAD` (handoff L1-L9 form):
  exactly the two lib files + `r1-journal` (rider) + `execution-results`
  (orchestrator rider) + `r2-journal`. Nothing under `crates/repo/`,
  `crates/lang/`, `crates/lang-rust/`, `crates/cli/`, `tests/`, `tools/`; no
  `Cargo.lock` delta.

## Fresh verification (reviewer's own runs, distilling entry points)

- Scoped lang-api (gate 2 form, at landing HEAD):
  `python tools/testrun.py --distill -p ariadne-lang-api` exit 0 - canonical
  line verbatim:
  `48 passed / 0 failed / 0 ignored / 5 suites`
  Consistent with the journalled pre-mutation baseline
  `47 passed / 0 failed / 0 ignored / 5 suites` (gate 1; unreproducible by a
  reviewer post-landing, see Not-run note) + the one new pin. The advisory
  exhaustive test is green WITH its +1 row (it is among the 48; the hunk
  review proves UNEDITED otherwise).
- Full census (gate 3): `python tools/testrun.py --distill` exit 0 -
  canonical line verbatim:
  `1855 passed / 0 failed / 13 ignored / 119 suites`
  (capture `target/testrun-capture.txt`, 3215 lines). Reconcile vs the duty-1
  ledger anchor `1854 passed / 0 failed / 13 ignored / 119 suites @ e862acf`:
  delta +1, fully attributed to `aggregation_kind_append_is_serde_index_stable`
  (named in the scoped run); failed 0; ignored 13 unchanged; suites flat at
  119 - goldens, m00, m17, lang-cpp, lang-scala, tools green UNEDITED.
- Hygiene (gate 4): `python tools/distill.py -- python tools/hygiene_check.py`
  -> `### generic PASS (exit 0)` (capture 173 lines). Every printed line is
  warn-only R3; grep of the capture for `lang-api` finds NO line naming
  either touched file. modeling.rs final 232 lines, identity.rs 53 - both <<
  1000; functions well under 100.
- L12 encoding: `python tools/distill.py -- python tools/encoding_guard.py
  --all` -> `### generic PASS (exit 0)`.
- Reviewer acceptance bench (kickoff fully-run stance; `crates/lang-api/src`
  touched): `python tools/distill.py -- python tools/bench_compare.py` RUN
  fresh -> `### generic PASS (exit 0)`; digest carries
  `bench compare: 3 run(s), no regressions past 1.25x baseline`. Zero WARN
  lines in the capture (grep: no match); PR-2 stance moot.
- Tool unit suites (CI form): `python -m unittest discover -s tools -p
  "*_tests.py"` -> `Ran 296 tests in 29.127s` / `OK`.

## Refusals encountered (verbatim)

1. One compound bash call (chained `git diff` + `Select-Object`) was refused
   with the catch-all boilerplate
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
   Substituted: the two `git diff --name-only` forms run as separate
   invocations - both executed, both reported above.
2. `python tools/testrun.py --baseline` -> `usage error: missing value for
   --baseline`. Same F9 errata as every prior hop of this track: the FILE
   form is superseded, `--distill` (identical count semantics) governs and was
   run fresh above. Not a finding.
No expected gate was skipped on account of a refusal.

## Red evidence - the justified RED-SKIP

Disposition: ACCEPTED.

- The handoff's RGB protocol item 1 PRE-AUTHORIZES the skip with an explicit
  precedent trio; I verified each cited journal actually says what is
  claimed:
  - `tasks/extraction-symbol-move-sm1-journal.md:21-26` - "JUSTIFIED
    RED-SKIP declared ... No assertion can fail against the pre-hand tree
    WITHOUT the new method existing" (isomorphic argument: naming a variant
    that does not exist is a compile error, not a failing test).
  - `tasks/ui-extraction-wizard-w1-journal.md:14-17` - "D1 - Red-skip
    declaration": no failing assertion authorable under the hand's bound.
  - `tasks/extraction-consumer-rewrite-cw2-journal.md:153-160` - "justified
    Red-skip (precedent: ui-extraction-wizard W-1's justified Red-skip)" with
    its falsifiability paragraph - the declared-falsifiability idiom the R-2
    journal mirrors.
- The pin-of-shipped-truth leg is verified independently, not taken on the
  relay's word: `crates/repo/src/identity.rs:152` carries
  `FileKind::RustSource => Self::Rust,` (shipped at R-1 `e862acf`, in that
  verdict's hunk list), so the A6 row is green-on-landing BY CONSTRUCTION and
  falsifiable only against a future regression of the arm - exactly the
  amendment discipline (CW-1 precedent).
- Journal carries the required falsifiability statement. One leg of it is
  overstated - see finding 1 (does not reopen the skip: the hand itself
  prescribed the test verbatim, so the implementation is plan-literal).

## Findings

1. MINOR (informational; follow-up, not a blocker) - `crates/lang-api/src/
modeling.rs:214-232` with `tasks/rust-language-r2-journal.md:57-61`: the
falsifiability claim "fails if the variant is ... inserted not-last (the
historical names/order pins catch the reorder through the positional
contract)" is only half enforced by the prescribed test. serde_json
serialization of UNIT variants is name-based and position-INDEPENDENT: a
future insertion of `ModDeclaration` BEFORE `None` keeps all four JSON names,
so every assertion in `aggregation_kind_append_is_serde_index_stable` still
passes. The rename leg is genuinely pinned; the reorder leg is caught at this
hop only by the L11 diff proof (which I re-verified as pure append) and by
future per-commit review, NOT by this test. The implementation matches
handoff item 2 verbatim, so this is a plan-level weakness, not an implementer
deviation; per plan-literal compliance it cannot be held against this hand.
Follow-up disposition: route to the planner at R-4 or the closeout - a true
positional pin needs a bincode round-trip against a frozen historical blob
or explicit index asserts; the ar27/golden/m00 UNEDITED oracle (green every
hop) remains the effective warm-byte defense meanwhile.

No blockers. No undeclared deviations: every commit's diff equals its
command-frozen or explicitly sanctioned scope, plan literals verified verbatim
(enum line, A6 tuple, pin name, messages), and nothing outside the touch-set
carries an implementer hunk.

## Deviation dispositions

- Deviation (a) - sanctioned docs-only rider `1cf6786` touching
  `tasks/rust-language-r1-journal.md`, outside the literal touch-set:
  ACCEPTED. Authority verified on its merits: the R-1 verdict finding-1
  disposition explicitly sanctions "clear via a docs-only rider"; the clear is
  byte-exact to the finding's quoted scope (6 deletions, placeholder +
  truncated header + orphan line only); landing it as its OWN commit preserved
  the command-frozen scopes of duty 1 (two paths) and the landing (three
  paths) - the alternative the verdict offered (riding the landing commit)
  would have BROKEN the landing's frozen scope. Correct call.
- Deviation (b) - refactor step consumed zero re-runs: ACCEPTED, reclassified
  as NOT a deviation. The hand's refactor step ("doc wording plain; re-run")
  presupposes a change; with none authored there was nothing to re-run, and
  the doc wording is plain (verified in the diff).
- Deviation (c) - landing SHA not self-journalled (journal:156-159):
  ACCEPTED. A commit cannot contain its own SHA; relaying it in the report
  with R-3 duty 1 citing this journal + the relayed SHA is exactly the
  handoff's Landing SHA relay clause, and avoids re-creating the R-1
  finding-1 placeholder class.
- Not deviations (recorded so the ledger inherits none): the hygiene distill
  form (matches R-1 recorded practice); gate-1 baseline being journalled-only
  (a reviewer cannot un-land the mutation; the 48 = 47 + 1 arithmetic is
  verified at landing HEAD).

## Follow-up dispositions

- Finding 1 goes to the planner (R-4 vocabulary hop or closeout): strengthen
  the positional pin per the note above. Recorded here; not dropped.
- `tasks/rust-language-execution-results.md` accumulates this hand's
  reviewer-run acceptance evidence - appended in the same writing session as
  this verdict (uncommitted, for R-3 duty 1 / the closeout to land per the
  track's pattern).
- No out-of-scope breakage was reported or found; nothing for
  `tasks/milestone_ledger.md` deferred-findings beyond finding 1.
