# AR-27 T-a0 Verdict - APPROVED

Adjudicated by: reviewer (fresh full verification per `review-gate` + R2.4
token-diff class check). Landing commit `43e7de1` (parent `3c814a2`, the
dispatch HEAD); addendum `046e38c` (journal hash-record, journal-only —
AR-16 `a4c018` / AR-08 `73f2160` / AR-07 `602c176` precedent). Evidence
addendum: `tasks/ar-27-ta0-execution-results.md`.

## Ground truth (checkpoint_guard + git, own runs)

HEAD `046e38c`, branch `shining-enthusiasm`; dirty = foreign closeout-docs
only; worktree census captured pre-review. `git log --oneline 3c814a2..HEAD`:
exactly two commits (`43e7de1`, `046e38c`). Per-commit review done oldest
first; `046e38c` touches ONLY `tasks/ar-27-ta0-handoff-journal.md`
(`git show --name-only`; `git diff 43e7de1 HEAD --stat` = journal +21).

## Commit-vs-touch-set - EXACT

`git diff --name-only 3c814a2..HEAD` = exactly 16 paths = the R3 explicit
per-path ledger (14 files) + probe-appended
`crates/cli/tests/ar25_seam_error_chain_test.rs` (15th, sanctioned by the
contract's commit section "plus one --scope per probe-appended ledger file,
journal-listed") + the journal. Zero paths outside. `git show --stat` =
`377 insertions(+), 39 deletions(-)` (39 token +/- pairs + journal 338 lines).
No git mv; no goldens; `377+/39-` exactly as claimed.

## Claim 1 - TOKEN-ONLY PURITY: CONFIRMED

R2.4 recipe run myself post-commit on `git show 43e7de1 -- <file>` for every
one of the 15 code paths (unified form read pair-by-pair, plus a mechanical
`git show --word-diff=plain --unified=0` second pass over all 15 files in
one invocation). All 39 pairs fall in exactly two classes:

- class A (`x` -> `x.clone()`, incl. `x.into()` -> `x.clone().into()` and
  `From::from(x)` -> `From::from(x.clone())`): 31 pairs
- class B (`*x` -> `x.clone()`): 8 pairs (m22_manifest x3, analyze :148/:154,
  node_maps :62, registry :448/:520, ar25 :56)

Word-diff shows every changed span confined to the token expression —
no whitespace, no literal, no assertion, no count, no ordering edit anywhere.
Per-file pair counts: discovery_io 3, m22_manifest 3, build/discovery 2,
analyze 6, index 1, node_maps 2, registry 5, ar25 1, for_incremental 6,
lib 1, stage_index 2, stage_load 1, parity 4, m22-it 1, m23-it 1 = 39.

Frozen-file token-normalizability (R2.4, quoted):

- `crates/pipeline/tests/for_incremental_parity_test.rs` (4 pairs, parity x4):
  `[-.plugin_for_id(ariadne_lang::LanguageId::from(language))-]{+...from(language.clone()))+}`
  (:95), `== [-Some(...from(language))-]{+Some(...from(language.clone()))+}`
  (:114), `if let Some(language) = [-language-]{+language.clone()+}` (:136),
  `restrict_and_validate_units(&registry, [-language,-]{+language.clone(),+}`
  (:153). Byte-equal after clone deletion. PASS.
- `tests/integration/tests/m22_scala_load_analysis_test.rs` (1 pair):
  `[-registry.plugin_for_id(record.language.into())-]{+registry.plugin_for_id(record.language.clone().into())+}`
  (:280). Byte-equal after clone deletion. PASS.
- `tests/integration/tests/m23_symbol_isolation_test.rs` (1 pair):
  `[-registry.plugin_for_id(ariadne_lang::LanguageId::from(record.language))-]{+...from(record.language.clone()))+}`
  (:406). Byte-equal after clone deletion — the D3 whitespace drift is
  byte-exact at the blob (changed span confined to the token; indentation
  identical). PASS.
- `crates/cli/tests/ar25_seam_error_chain_test.rs` (1 pair, appended
  frozen-class): `unit_with_language(rel, [-*language))-]{+language.clone()))+}`
  (:56). `*x` restored for `x.clone()` -> byte-equal. PASS.

ADAPT-8: the `crates/build/src/discovery.rs` :245-252 comment block appears
in the diff as UNCHANGED CONTEXT only (both :253/:272 hunks show it
untouched); `crates/index/**` diff vs parent is EMPTY and the index digest
suite is green UNEDITED (`15/0/0/2`, incl. `honest_context_digest` locks).
`crates/pipeline/src/for_incremental.rs:143` `language_filter: language,`
stays the last-use MOVE (verified in the :138 hunk: only :141 carries the
token; :143 is unchanged context).

## Claim 2 - ESCAPE-HATCH USE (D1/D2): ACCEPTED

- `for_incremental.rs:120` `if let Some(language) = language.clone()`:
  R3 Sequencing hatch = "newly surfaced SAME-class copy site = clone-token
  addition + journal, no re-ruling". Form is R2.1 class A on the scrutinee;
  journaled §3 #17 + §5 tier 2 BEFORE commit. Without it the R3 five-token
  set cannot reach zero-copy-error probe world (E0382 partial-move cluster
  :120-:125 — the T-a journal's own measured citation). Merits: accepted.
  Consequence accepted too: "every use except the last clones" then forces
  clones at BOTH :125 and :141 (R3's "one clone among :125/:141/:143" was
  written under the no-:120 premise); :143 stays the move — the governing
  rule is still satisfied exactly.
- `ar25_seam_error_chain_test.rs:56` `*language` -> `language.clone()`:
  same class, same form as registry.rs:448; was structurally invisible to
  T-a's lower bound (cli test binary never compiled while the cli lib
  errored); journaled §3 #39/§5 tier 3 BEFORE commit; file joined scope via
  its own `--scope` exactly as the contract's commit section provides.
- The four if-let-scrutinee attachments (:89 analyze, :72 index, :136
  parity, :120 for_incremental) — ACCEPTED: each is the unique single-token
  fix for the compiler-surfaced E0382 at the later use; all are the R2.1
  `x` -> `x.clone()` form; error-site vs fix-site mapping is journaled
  (§3 header) and consistent with R3 citing compiler error lines.

## Claim 3 - PROBE DISCIPLINE: CONFIRMED (reproduced)

Attestations reproduced STRONGER than the worktree form:
`git diff 3c814a2 43e7de1 -- crates/repo crates/lang-api crates/worker-core crates/index`
is EMPTY — the probe revert is IN the commit, not merely in the worktree.
`crates/repo/src/types.rs:141` derive list still carries `Copy` (read at
HEAD; blob identical to parent); `crates/lang-api/src/identity.rs:129` still
`Self::from(*source)` — the probe-world `source.clone()` token is NOT in the
landing commit or anywhere tracked. Tier enumeration 4->0 errors to
`ok (exit 0, ...)` matches journal §5; complete count 39 tokens / 15 files
is inside planning's ~28-45 (R1 "~35-45 single-token edits").

## Claim 4 - IDENTITY: CONFIRMED (own runs, this review)

Canonical lines, verbatim from own stdout (capture-race rule):

- `-p ariadne-pipeline`: `40 passed / 0 failed / 4 ignored / 9 suites` —
  parity suite's five named tests all green in-map (`5 passed; 0 failed;
  0 ignored` = 5/0/0/1)
- `-p ariadne-cli`: `118 passed / 0 failed / 8 ignored / 7 suites`
- `-p ariadne-build`: `23 passed / 0 failed / 0 ignored / 2 suites`
- `-p ariadne-artifact`: `40 passed / 0 failed / 0 ignored / 2 suites`
- `-p ariadne-index`: `15 passed / 0 failed / 0 ignored / 2 suites`
- integration `--test m23_symbol_isolation_test --test
  m22_scala_load_analysis_test`: `13 passed / 0 failed / 0 ignored / 2
  suites`, per-binary capture split `6 passed` (m22, incl. the token-site
  test `plan_evolved_from_capability_rejection_to_planned_nothing`) +
  `7 passed` (m23, incl. `wired_plan_proposes_scala_extraction`) —
  m22 `6/0/0/1` + m23 `7/0/0/1` IDENTICAL.

All IDENTICAL pre/post. Hygiene: own run prints ONLY R3 (warn) rows, zero
R1/R2 blocking; of the 15 touched code files exactly one carries a row
(`crates/build/src/discovery.rs:320` single-letter `c`) and that line is
not a touched line — no touched file gained a row. Encoding:
`encoding_guard.py --all` silent.

Transient clone warnings: NOT silenced (no allow-attribute anywhere in the
diff — the entire commit is 39 token pairs). My own `cargo clippy -p
ariadne-build --all-targets` capture shows `clippy::clone_on_copy` firing
live, unsuppressed, at `discovery.rs:253` and `:272`. CI gates clippy
without `-D warnings` (ci.yml:26-30, legacy-baseline flip tracked), so the
R3 clause "not silenced, not gated, AR-28 owns the sweep" holds. See
finding 1 for an attribution correction.

## Claim 5 - TOUCH-SET + D3/D4: CONFIRMED

16 scopes = ledger 15 + journal, exactly (see Commit-vs-touch-set). D3:
verified byte-exact at the blob (Claim 1). D4 `--staged` timing: benign —
`commit_scoped.py` stages atomically so no index window exists; `--all`
(gate 8, pre-commit AND my own post-landing run) is the superset covering
the commit content, both silent. Accepted.

## Ledger-count note

R3's "12 files" label vs the explicit per-path list of fourteen is disclosed
in the contract itself ("The reviewer enforces the EXPLICIT PER-PATH list,
not the count"); the landing matches the explicit list + the probe-appended
path. No finding.

## Findings

1. [minor, journal-wording] `tasks/ar-27-ta0-handoff-journal.md` §4/§8-gate-1
   attribute the cargo-check warnings ("23"/"31 warnings emitted") to "the
   transient `clone_on_copy` class". They are not: `cargo check` does not run
   clippy lints; my clean-rebuild reproduction of gate 1 (`cargo clean -p` of
   the four token crates, then `cargo check --workspace --all-targets` ->
   exit 0, 31 warnings) shows those 31 are the pre-existing legacy rustc
   class (deprecated `IncludeResolver` oracle, goldens unused-helpers, unused
   vars) — zero clone-class lines. The clone-on-Copy warnings exist only
   under clippy, where I independently confirmed them present and unsuppressed
   (capture lines 5-21: `discovery.rs:253`/`:272`). Substance of the R3
   sequencing clause is unaffected; correct the attribution wording on the
   journal's next touch so no downstream session hunts for clone warnings in
   `cargo check` output.
2. [info, external] The dispatch-time foreign churn
   `tests/fixtures/basic/artifacts/includes/report.json` (journal §2) no
   longer appears dirty at review time; no path of this hop touched it and
   the census is EXACT. No action; recorded so the §2 snapshot is not later
   misread as lost churn.

No blockers. No rework on `43e7de1`.

## Deviation dispositions

- D1 (probe additions :120 + ar25:56): ACCEPTED (Claim 2).
- D2 (fix-site-at-scrutinee class + both :125 and :141 cloned): ACCEPTED
  (Claim 2).
- D3 (m23 whitespace drift self-caught, corrected pre-commit): ACCEPTED —
  byte-exactness verified independently at the blob.
- D4 (`--staged` after-commit timing): ACCEPTED as disclosed environment
  shape; `--all` superset clean.

## Adopted pins for downstream contracts

Reviewer re-pin at T-a0 per R2.5 (first-ever measured, now ADOPTED after my
own independent runs — downstream contracts (T-a, T-b) must baseline against
these): `-p ariadne-build` = `23/0/0/2`; `-p ariadne-artifact` = `40/0/0/2`.
Standing pins re-confirmed unchanged at landing: pipeline `40/0/4/9` (par
ity `5/0/0/1`), cli `118/0/8/7`, index `15/0/0/2`, m22 `6/0/0/1`,
m23-symbol `7/0/0/1`, census anchor `1400/0/13/105`.

## Sequencing

APPROVED -> T-a dispatches (amended zero-rider contract). T-a must
compile-green with ZERO rider edits; newly surfaced same-class copy site =
clone-token addition + journal; non-token adaptation = STOP-AND-REPORT.
T-a0's clone tokens in `crates/cli/src/registry.rs` must survive T-a's parse
rework untouched.
