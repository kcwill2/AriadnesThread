# AR-27 T-a0 Handoff Journal - Inert clone-token pre-land (A2 hop 0/3)

Session: implementer, sole cargo-heavy lane. Contract:
`tasks/ar-27-ta0-handoff.md`. Dispatch HEAD `3c814a277ff52a3dd18945b60f962522e3106302`
(docs(tasks): AR-27 re-ruling hop T-a0 ... RE-RULING R1-R6 landed) - verified
via `python tools/checkpoint_guard.py status` and git log at session start.

Bootstrap (contract Method step 1): journal + baselines BEFORE any mutation.
Authority chain read in full: this contract; `tasks/ar-27-shape-ruling.md`
(RE-RULING R1-R5 + section 2 context); `tasks/ar-27-ta-handoff-journal.md`
(STOP journal: measured ledger, reverted-probe protocol, registry.rs ledger
items 2-6 token forms).

Bootstrap resume line for a successor (never resume a dead session):
`python tools/checkpoint_guard.py resume --spec tasks/ar-27-ta0-handoff.md
--scope ... --cmd ...`

## 1. Pre-mutation baselines (own stdout; capture-race rule - quoted from
own stdout with capture paths; ALL measured BEFORE any mutation at 3c814a2)

- `python tools/testrun.py --distill -p ariadne-build` - FIRST-EVER PIN:
  `23 passed / 0 failed / 0 ignored / 2 suites`
  (capture: target/testrun-capture.txt, 43 lines).
- `python tools/testrun.py --distill -p ariadne-artifact` - FIRST-EVER PIN:
  `40 passed / 0 failed / 0 ignored / 2 suites`
  (capture: target/testrun-capture.txt, 54 lines).
- `python tools/testrun.py --distill -p ariadne-index` - pin match:
  `15 passed / 0 failed / 0 ignored / 2 suites` (ADAPT-8 digest locks included;
  cache.rs untouched this hop - gate 5 re-run is the byte-clean proof).
- `python tools/testrun.py --distill -p ariadne-pipeline` - anchor match:
  `40 passed / 0 failed / 4 ignored / 9 suites`.
- `python tools/testrun.py --distill -p ariadne-pipeline --test for_incremental_parity_test`
  - lock match: `5 passed / 0 failed / 0 ignored / 1 suites`.
- `python tools/testrun.py --distill -p ariadne-cli` - anchor match:
  `118 passed / 0 failed / 8 ignored / 7 suites`.
- `python tools/testrun.py --distill -p ariadne-integration-tests --test m23_symbol_isolation_test --test m22_scala_load_analysis_test`
  combined `13 passed / 0 failed / 0 ignored / 2 suites`; capture per-binary
  split (target/testrun-capture.txt lines 53/65): one binary `6 passed` (m22,
  PINNED `6/0/0/1`), one binary `7 passed` (m23 - matches the standing
  `7/0/0/1` gate-lock pin from T-a journal section 2). m22 suite resolved by
  run = `m22_scala_load_analysis_test` `6/0/0/1`.
- `python tools/hygiene_check.py` - ZERO R1/R2 blocking rows; all reported
  rows R3 (warn) only, identical set expected post-land (token-only diff).
- `python tools/encoding_guard.py --all` - silent (clean).
- Census anchor `1400/0/13/105` - CONTEXT ONLY, NOT re-measured: workspace
  census is the reviewer's acceptance gate per contract + Planning Law.

## 2. Worktree state at dispatch

`python tools/distill.py -- git status --porcelain` (own stdout):

```
 M tasks/ar-closeout-deferred-rows.md
 M tests/fixtures/basic/artifacts/includes/report.json
?? .kilo/plans/1791568620000-audit-remediation.md
?? .kilo/plans/1791690461718-ar20-shape-ruling.md
```

Foreign churn only (closeout-docs lane + known report.json CRLF churn +
untracked plans). `crates/` and `tests/integration/` byte-clean at dispatch.

## 3. Landing ledger (R3 list + landed token set)

Token forms landed are exclusively R2.1 forms: `x` -> `x.clone()`,
`*x` -> `x.clone()`, `x.into()` -> `x.clone().into()`,
`LanguageId::from(x)` -> `LanguageId::from(x.clone())`. Every ledger
expression was read before editing (Read-tool evidence). Error-site vs
fix-site: the R3 list cites compiler ERROR sites (mirroring the T-a journal
section 5 captures); each token attaches at the compiler-required FIX site,
one token per surfaced error - the if-let scrutinee sites (:136/:89/:72/:120
class, `x` -> `x.clone()` on the scrutinee) are the fix sites for the E0382
partial-move errors reported at the later USE sites (:153/:93/:77/:125).
No `&`-rewrites, no signature/pattern surgery, no reordering, no logic.

| # | file | landed token (fix site) | error site it fixes |
|---|---|---|---|
| 1 | `crates/build/src/discovery.rs` :253 | `ctx.language = record.language.clone();` | :253 E0507 |
| 2 | " :272 | `language: record.language.clone(),` | :272 E0507 |
| 3 | `crates/artifact/src/manager/discovery_io.rs` :51 | `.map(\|f\| f.language.clone())` | :51 E0507 |
| 4 | " :130 | `.map(\|c\| c.language.clone())` | :130 E0507 |
| 5 | " :131 | `.map(\|tu\| tu.language.clone())` | :131 E0507 |
| 6 | `crates/artifact/src/manager/m22_manifest_languages_tests.rs` :39 | `*lang` -> `lang.clone()` | :39 E0507 |
| 7 | " :154 | `*lang` -> `lang.clone()` | :154 E0507 |
| 8 | " :169 | `*lang` -> `lang.clone()` | :169 E0507 |
| 9 | `crates/pipeline/src/stage_index.rs` :110 | `self.language_filter.clone(),` | :110 E0507 |
| 10 | " :122 | `if let Some(language) = self.language_filter.clone()` | :122 E0507 |
| 11 | `crates/pipeline/src/stage_load.rs` :183 | `.map(\|record\| record.language.clone())` | :183 E0507 |
| 12 | `crates/pipeline/src/for_incremental.rs` :66 | `LanguageId::from(language.clone())` | :67 bail `{language:?}` E0382 |
| 13 | " :85 | retain closure `LanguageId::from(language.clone())` | :85 E0507 FnMut |
| 14 | " :121 | `ensure_language_registered(&registry, language.clone())` | ruled token (R3) |
| 15 | " :125 | `restrict_and_validate_units(&registry, language.clone(), ..)` | fixes :141/:143 chain; "every use except last clones" |
| 16 | " :141 | `language_filter: language.clone(),` (Self literal) | last use :143 stays a move |
| 17 | " :120 | `if let Some(language) = language.clone()` | PROBE-SURFACED addendum - see section 5 (fixes the :120-:125 E0382 partial-move cluster the T-a journal measured; the R3 5-token set alone does not reach zero-copy-error probe world) |
| 18 | `crates/pipeline/src/lib.rs` :681 | `LanguageId::from(record.language.clone())` | :681 E0507 |
| 19 | `crates/cli/src/registry.rs` :28 | `LanguageId::from(language.clone())` | :29 bail E0382 (item 2) |
| 20 | " :174 | `LanguageId::from(language.clone())` | :175 UnhandledLanguage{language} E0382 (item 3) |
| 21 | " :192 | `distinct.push(unit.language.clone());` | :192 E0507 (item 4) |
| 22 | " :448 | `*language` -> `language.clone()` | :448 E0507 (item 5) |
| 23 | " :520 | `language: language.clone(),` | :520 E0507 (item 6) |
| 24 | `crates/cli/src/commands/analyze.rs` :89 | `if let Some(language) = language.clone()` | :93 E0382 (scrutinee fix site; last use :93 moves) |
| 25 | " :143 | `distinct.push(language.clone());` | :143 E0507 |
| 26 | " :148 | `*language` -> `language.clone()` (slice-pattern arm) | :148 E0507 |
| 27 | " :154 | `*language` -> `language.clone()` (for over &distinct) | :154 E0507 |
| 28 | " :169 | `languages.push(record.language.clone());` | :169 E0507 |
| 29 | " :262 | `.map(\|f\| f.language.clone())` | :262 E0507 |
| 30 | `crates/cli/src/commands/index.rs` :72 | `if let Some(language) = language.clone()` | :77 E0382 (scrutinee fix site) |
| 31 | `crates/cli/src/commands/ui/node_maps.rs` :59 | `.entry(file.language.clone())` | :59 E0507 entry key |
| 32 | " :62 | `*language` -> `language.clone()` (with_key closure) | :62 E0507 |
| 33 | FROZEN `crates/pipeline/tests/for_incremental_parity_test.rs` :95 | `LanguageId::from(language.clone())` | :98 E0382 |
| 34 | " :114 | retain closure `language.clone()` | :114 |
| 35 | " :136 | `if let Some(language) = language.clone()` | :153 E0382 (scrutinee fix site) |
| 36 | " :153 | `restrict_and_validate_units(&registry, language.clone(), ..)` | :163 (last use moves) |
| 37 | FROZEN `tests/integration/tests/m22_scala_load_analysis_test.rs` :280 | `record.language.clone().into()` | :280 E0507 |
| 38 | FROZEN `tests/integration/tests/m23_symbol_isolation_test.rs` :406 | `LanguageId::from(record.language.clone())` | :406 E0507 |
| 39 | PROBE-SURFACED `crates/cli/tests/ar25_seam_error_chain_test.rs` :56 | `*language` -> `language.clone()` | :56 E0507 - same class, same form as registry.rs :448; hidden in T-a's lower-bound world (the cli test binary never compiled while the cli lib errored); file JOINS the commit scope with its own --scope flag |

The ADAPT-8 comment block (`crates/build/src/discovery.rs` :245-252) was NOT
touched; proven by gate 5 index digest suite + the `git show` token-diff.
`clone_on_copy` warnings are EXPECTED while the enum is Copy (R3 sequencing
note; AR-28 owns the sweep) - not silenced, not gated.

## 4. Inert-green check after landing (real world, type still `Copy`)

`python tools/distill.py -- cargo check --workspace --all-targets` ->
`ok (exit 0, 23 warnings emitted)` - capture:
`target\distill-capture.txt`. Warnings are the transient `clone_on_copy`
class (expected per R3 sequencing; not silenced, not gated; AR-28 sweep).
Warning counts across runs are not comparable (cargo re-emits only for
rebuilt crates).

## 5. COMPLETENESS PROBE (throwaway; the R3 method; NEVER committed)

Method: strip `Copy` from the `SourceLanguage` derive list at
`crates/repo/src/types.rs` :141 (single throwaway edit); run
`python tools/distill.py -- cargo check --workspace --all-targets`
per tier; every compiler-surfaced copy-semantics error (E0507/E0382) gets
its token in the real ledger file (committed class) or the throwaway form
in a T-a-owned file (never committed); iterate to ZERO.

Tier enumeration (probe iterations: 4 total, 3 with surfaced errors):

1. TIER 1 -> 1 error: `crates/lang-api/src/identity.rs` :129 E0507
   (`Self::from(*source)` in the kept `&`-From body). Throwaway token
   `Self::from(source.clone())` applied - lang-api is T-a-owned (R3
   throwaway allowance); REVERTED before commit, NEVER committed. This is
   the coherence adaptation the T-a contract already commands for the move.
2. TIER 2 -> 1 error: `crates/pipeline/src/for_incremental.rs` :125 E0382
   "borrow of partially moved value: `language`" (partial move at the :120
   if-let). FIX: ledger #17 - `if let Some(language) = language.clone()`
   scrutinee token at :120. REAL token, committed (file already in ledger;
   the T-a journal `:120-:125` cluster citation).
3. TIER 3 -> 1 error: `crates/cli/tests/ar25_seam_error_chain_test.rs` :56
   E0507 `*language`. FIX: ledger #39 - `language.clone()` token. REAL
   token, committed; file appended to the touch-set per R2.3/R3
   probe-surfaced-remainder clause (one more --scope on the commit).
4. TIER 4 -> `ok (exit 0, 31 warnings emitted)` -> ZERO copy-semantics
   errors. capture: target/distill-capture.txt.

Probe verdict: ledger COMPLETE (not lower-bound). Total landed tokens: 39
in 15 files (the R3 enumerated 14 files + 1 probe-appended file - the R3
"12 files" label was already superseded by the explicit per-path list of
FOURTEEN; the probe adds the fifteenth). Probe additions vs the R3 list =
ledger #17 (`for_incremental.rs` :120 scrutinee) and ledger #39
(`crates/cli/tests/ar25_seam_error_chain_test.rs` :56); everything else the
probe touched was already enumerated. Confirmed NO-OP / zero-error
territories (probe-world, `--all-targets`): `crates/worker-core/src/
tu_store.rs` :210 (ruling claim re-confirmed), `crates/repo/**` internal
code (only the derive line + the lang-api `&`-From body needed anything),
`crates/index/**`, `crates/report/**`, `crates/ui/**`, `crates/graph/**`,
`crates/cluster/**`, `crates/transform/**`, `crates/validate/**`,
`crates/scheduler/**`, `crates/lang/**`, `crates/lang-cpp/**`,
`crates/lang-scala/**` - the other locks (m23-bincode, m00, m24, m24a,
m18, ui) never surfaced a single site.

Throwaway residue (both reverted; attestation in section 6):
`crates/repo/src/types.rs` :141 derive line; `crates/lang-api/src/
identity.rs` :129 `&`-From body line.

## 6. Revert + deleted-before-commit attestation (probe)

Run BEFORE the landing commit, AFTER the revert of both throwaway lines.
Own stdout quoted (capture-race rule); both via the distilling entry point
(raw `git` is allowlist-refused on this lane).

`python tools/distill.py -- git diff -- crates/repo crates/lang-api` ->

```
### generic PASS (exit 0)
capture: target\distill-capture.txt (0 lines)
```

EMPTY diff - both probe edits (derive line + `&`-From body) deleted.

`python tools/distill.py -- git status --porcelain -- crates` -> (own stdout)

```
 M crates/artifact/src/manager/discovery_io.rs
 M crates/artifact/src/manager/m22_manifest_languages_tests.rs
 M crates/build/src/discovery.rs
 M crates/cli/src/commands/analyze.rs
 M crates/cli/src/commands/index.rs
 M crates/cli/src/commands/ui/node_maps.rs
 M crates/cli/src/registry.rs
 M crates/cli/tests/ar25_seam_error_chain_test.rs
 M crates/pipeline/src/for_incremental.rs
 M crates/pipeline/src/lib.rs
 M crates/pipeline/src/stage_index.rs
 M crates/pipeline/src/stage_load.rs
 M crates/pipeline/tests/for_incremental_parity_test.rs
```

LEDGER-ONLY - 13 files under `crates/`, no `crates/repo`, no
`crates/lang-api`, no residue. Full-tree porcelain additionally shows only
the two ledger files under `tests/integration/tests/`, this journal
(untracked), and the pre-existing foreign churn from dispatch
(`tasks/ar-closeout-deferred-rows.md`, `tests/fixtures/basic/artifacts/
includes/report.json`, two `.kilo/plans/` untracked) - identical foreign set
to section 2.

## 7. Gate 9 pre-commit token-diff evidence (R2.4 recipe, working-tree form)

`python tools/distill.py -- git diff -- <file>` per named frozen file, run
BEFORE commit (the reviewer runs the post-commit form `git show
<T-a0-commit> -- <file>`). Full +/- pairs verified from own captures:

1. `crates/pipeline/tests/for_incremental_parity_test.rs` (capture 41
   lines): 4 pairs - `from(language)` -> `from(language.clone())` x2
   (:95, :114 closure), `if let Some(language) = language` ->
   `language.clone()` (:136), `restrict_and_validate_units(&registry,
   language,` -> `language.clone(),` (:153). Deleting inserted `.clone()`s:
   every pair byte-equal. PASS.
2. `tests/integration/tests/m22_scala_load_analysis_test.rs` (13 lines):
   1 pair - `record.language.into()` -> `record.language.clone().into()`
   (:280). Byte-equal after clone deletion. PASS.
3. `tests/integration/tests/m23_symbol_isolation_test.rs` (final capture
   14 lines): 1 pair - `LanguageId::from(record.language)` ->
   `LanguageId::from(record.language.clone())` (:406). PASS.
   PROCESS NOTE (proves the recipe has teeth): the FIRST capture of this
   file showed the `+` line with one extra leading space - a whitespace
   drift I introduced when transcribing the site line (my Read transcript
   included the read-tool prefix column). It was NOT token-normalizable;
   corrected the file to exact original indentation and RE-RAN the diff -
   the quoted PASS pair above is the corrected re-run. No assertion,
   literal, or expectation was ever altered; the fix was whitespace-only.
4. Appended frozen-class file `crates/cli/tests/ar25_seam_error_chain_test.rs`
   (13 lines): 1 pair - `unit_with_language(rel, *language)` ->
   `unit_with_language(rel, language.clone())` (:56). Restoring `*x` for
   `x.clone()` per R2.4: byte-equal. PASS.

Whole-diff sweep of the 11 source-ledger files (`git diff -- <11 paths>`,
capture 305 lines, read in full): every +/- pair is one of the R2.1 token
forms - zero literal/assertion/count/ordering edits anywhere; the ADAPT-8
comment lines appear ONLY as unchanged context lines. The final
`language_filter: language,` at for_incremental.rs :143 stays a MOVE
(last use, untokenized) exactly per the "every use except the last
clones" rule.
Note: git printed `LF will be replaced by CRLF` working-copy notices on
some touched files - pre-existing autocrlf posture of this worktree
(the foreign report.json churn came from the same mechanism); the diffs
contain NO line-ending churn, only the token pairs above.

## 8. Real-world gate record (post-probe-revert; canonical lines verbatim
from own stdout per capture-race rule; capture paths printed by the runs)

1. `python tools/distill.py -- cargo check --workspace --all-targets` ->
   `ok (exit 0, 31 warnings emitted)` - GREEN (real world; warnings are the
   transient `clone_on_copy` class per R3, not silenced, not gated).
2. `python tools/testrun.py --distill -p ariadne-pipeline` ->
   `40 passed / 0 failed / 4 ignored / 9 suites` - IDENTICAL to baseline;
   capture per-binary map shows `for_incremental_parity_test` with its five
   named tests (`unregistered_language_errors_identically`,
   `cpp_filter_on_cpp_repo...`, `scala_filter_on_cpp_repo...`,
   `cpp_filter_on_scala_marker_repo...`,
   `unrestricted_run_on_scala_marker_repo...`) -> `5 passed; 0 failed;
   0 ignored` UNEDITED (5/0/0/1 match).
3. `python tools/testrun.py --distill -p ariadne-cli` ->
   `118 passed / 0 failed / 8 ignored / 7 suites` - IDENTICAL.
4. `python tools/testrun.py --distill -p ariadne-build` ->
   `23 passed / 0 failed / 0 ignored / 2 suites` - IDENTICAL to the
   first-ever pre-mutation pin (section 1).
   `python tools/testrun.py --distill -p ariadne-artifact` ->
   `40 passed / 0 failed / 0 ignored / 2 suites` - IDENTICAL to the
   first-ever pre-mutation pin (section 1).
5. `python tools/testrun.py --distill -p ariadne-index` ->
   `15 passed / 0 failed / 0 ignored / 2 suites` - IDENTICAL; UNEDITED
   digest locks = ADAPT-8 byte-clean proof (cache.rs never touched).
6. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m23_symbol_isolation_test --test m22_scala_load_analysis_test` ->
   `13 passed / 0 failed / 0 ignored / 2 suites`, per-binary `6 passed` +
   `7 passed` - IDENTICAL split to the pre-mutation pair run: m22
   `6/0/0/1` + m23 `7/0/0/1`, assertions UNEDITED except the two ruled
   tokens (section 7 evidence).
7. `python tools/hygiene_check.py` - every reported row R3 (warn), ZERO
   R1/R2 blocking, row-set IDENTICAL to the pre-mutation baseline (section
   1); no touched file gained a row; no function crossed the 100-line
   ceiling from a `.clone()` insert (single-expression inserts only).
   `encoding_guard --staged`: raw `git add` is off this lane's allowlist
   and `commit_scoped.py` stages atomically inside the commit, so
   `--staged` runs trivially AFTER the scoped commit (empty index for the
   scoped paths) - disclosed per the T-a journal section 9 / AR-27 D2 /
   AR-20b D2 precedent; `--all` (gate 8, run BEFORE the commit and again
   after) is the superset covering this commit's content.
8. `python tools/encoding_guard.py --all` - silent (clean), before the
   commit.
9. Token-diff evidence: section 7 (working-tree form, pre-commit; the
   reviewer re-runs the `git show` form post-commit).

## 9. Refused-command log (verbatim; catch-all boilerplate
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}` is
reported regardless of true cause)

- `python -X utf8 tools/encoding_guard.py --help > $null; if ($?) { echo
  "guard-ok" }` -> refused (chained/echo form, known class) -> not needed;
  journal repair done via Edit tool instead.
- All `git` invocations this hop ran ONLY via `python tools/distill.py --`
  (status/diff/show/log) per contract; no raw-git attempt.
- No `python -c` attempted (known refusal). Canonical lines quoted from
  own stdout with capture paths (capture-race rule).

## 10. Deviations

- D1 (planned, per contract): probe additions ledger #17
  (`for_incremental.rs` :120 scrutinee clone) and ledger #39
  (`crates/cli/tests/ar25_seam_error_chain_test.rs` :56) - R2.3/R3
  probe-surfaced-remainder class; same forms; #39's file joins the commit
  scope with its own --scope flag as the contract's commit section provides.
  Both are SAME-class clone-token additions (the D-list escape hatch), no
  new class, no STOP trigger.
- D2: token SITES vs error sites - the R3 list cites compiler error lines;
  four fix tokens physically attach at if-let SCRUTINEES (:89 analyze,
  :72 index, :136 parity, :120 for_incremental), which are the only
  single-token fixes for the corresponding E0382 partial-move errors. All
  are the R2.1 `x` -> `x.clone()` form (scrutinee is an `x`).
- D3: m23 whitespace drift caught and corrected pre-commit (section 7
  process note) - zero effect on any assertion; caught by the contract's
  own R2.4 recipe.
- D4: `--staged` timing as disclosed in gate 7 (environment constraint,
  T-a journal precedent).
