# extraction-consumer-rewrite - Execution Results (through CW-3 review hop)

Written by the CW-1 reviewer hop, 2026-10-23, against
`tasks/extraction-consumer-rewrite-cw1-handoff.md` acceptance and the
workspace-wide gates the reviewer owns. HEAD at gate time: `f73a9a1` on `main`
(landing `c916a26` + journal rider), start rev `2e5b86f`. All commands run
fresh in the reviewer session; nothing is inherited from the implementer's
journal without a re-run.

## Handoff acceptance criteria (CW-1)

| Criterion | Result | Evidence (verbatim) |
|---|---|---|
| lang-cpp green with 4 new tests (gate 4) | PASS | `python tools/testrun.py --distill -p ariadne-lang-cpp` -> `205 passed / 0 failed / 0 ignored / 2 suites`, exit 0 (201 baseline + 4 new) |
| Amended riders green (gate 5) | PASS | `python tools/testrun.py --distill -p ariadne-integration-tests --test m20_apply_migrate_characterization_test --test m19_optimize_characterization_test` -> `11 passed / 0 failed / 0 ignored / 2 suites`, exit 0 |
| m10/m14/m17/m18/m23 green UNEDITED (gate 6) | PASS | same-form command -> `48 passed / 0 failed / 0 ignored / 5 suites`, exit 0; none of the five files appears in `git diff --name-only 2e5b86f..HEAD` |
| Scala green UNEDITED (gate 7) | PASS | `python tools/testrun.py --distill -p ariadne-lang-scala` -> `257 passed / 0 failed / 0 ignored / 2 suites`, exit 0; `crates/lang-scala/**` in no diff |
| Full census +4 vs anchor (gate 8, `--distill` substitute per handoff) | PASS | `python tools/testrun.py --distill` -> `1834 passed / 0 failed / 13 ignored / 119 suites`, exit 0; anchor `1830 passed / 0 failed / 13 ignored / 119 suites @ 765df7b` -> exactly +4 passed, failed/ignored/suites held |
| Frozen lock pins green in census capture | PASS | capture (`target/testrun-capture.txt`, reviewer's own census run): line 1388 `test rewriter::tests::pin_neutral_consumer_rewrite_spec_default_is_none ... ok`; line 2639 `test test_transaction_apply_and_rollback ... ok` |
| `crates/cli/tests` zero-diff (producer proof for CW-2 attribution) | PASS | `git diff --numstat 2e5b86f..HEAD -- crates/cli/tests` -> empty |
| Touch-set containment | PASS | `git diff --name-only 2e5b86f..HEAD` -> exactly the 4 declared paths; Cargo.lock and all MUST-NOT-touch crates absent |
| Rider hunk containment (handoff reviewer-gate reference) | PASS | m19: hunk on the sanctioned `fn` rename line + one body hunk inside the named test; m20: single hunk inside the named test (base blobs read via git MCP, byte-safe) |
| Hygiene (gate 10) | PASS | `python tools/hygiene_check.py` -> R1/R2 silent; only pre-existing R3 warn-only rows, none in touched files |
| Encoding | PASS | `python tools/encoding_guard.py --all` -> silent (tool prints every violation then exits nonzero; pass form per its design) |

## Workspace-wide gates (reviewer-owned)

Canonical lines, quoted verbatim from the reviewer's own fresh runs:

- Full census: `1834 passed / 0 failed / 13 ignored / 119 suites`
  (`python tools/testrun.py --distill`, exit 0, capture
  `target/testrun-capture.txt`, 3179 lines)
- Tools suite: `Ran 296 tests in 26.475s` + `OK`
  (`python -m unittest discover -s tools -p "*_tests.py"`) - matches the
  ROUND-2 anchor `Ran 296` OK
- UI smoke: `ui_smoke: PASS` / `boot+route checks clean (cold fixture,
  attempt 1, 1.1s)` via `python tools/distill.py -- python tools/ui_smoke.py`
  (exit 0). The direct form `python tools/ui_smoke.py` was REFUSED by the
  sandbox with the boilerplate catch-all deny
  (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`);
  the distilling wrapper form ran unmodified.
- Timing gate (NOT exempt - this hand touched `crates/lang-cpp`):
  `bench compare: 3 run(s), no regressions past 1.25x baseline` via
  `python tools/distill.py -- python tools/bench_compare.py` (exit 0, capture
  `target/distill-capture.txt`, 17 lines). The `f20ad35` label fix holds: the
  gate ran to completion, 3-run median table emitted, no machine-class WARN
  rows surfaced this run beyond the reported no-regression verdict. Attribution
  for CW-2's oracle is clean: `crates/cli/tests` in no diff.

## Explicitly NOT run

- `python tools/testrun.py --baseline` (FILE form): NOT run. The CW-1 handoff
  gate 8 declares it superseded for this milestone ("the `--baseline` FILE
  form stays superseded by this `--distill` substitute"), and the dispatch for
  this hop assigned the `--distill` full census as the census of record. The
  `--distill` census above is the substituted evidence; no weaker evidence was
  used (identical count semantics, exit codes + capture proven).
- Exit-code echo gates (`python -c`, `cmd /c`, `echo`, `; Write-Output`
  chains) were refused by the sandbox allowlist; where a canonical line or
  digest header with `PASS (exit 0)` came from the tool itself, that is the
  exit evidence (see verdict finding 1 for the config item).

## Red evidence status

Red (4 unit + 2 integration failures) is evidenced by the journal's verbatim
capture lines, which match the committed assert messages word-for-word, and is
mechanically corroborated at HEAD (frozen trait default returns `None`;
reviewer could not re-run Red without mutating the tree, which a reviewer must
not do). Disposition: accepted - see verdict "Red capture adjudication".

## Handoff acceptance criteria (CW-2)

Reviewer hop 2026-10-23 against `tasks/extraction-consumer-rewrite-cw2-handoff.md`.
HEAD at gate time: `d3f6dbc` on `main` (duty-1 `ece3513` + landing `fddb200` +
journal rider), start rev `f73a9a1`. All commands re-run fresh in this session.

| Criterion | Result | Evidence (verbatim) |
|---|---|---|
| Red: the one named test fails pre-fix | PASS (mechanically corroborated; journal-only Red, a reviewer must not re-derive it) | journal verbatim panic at `tests.rs:229:13` = `the non-ASCII plan_id mapping: DTO string must be ASCII, got "PLAN-<U+00E9>N-001"` - matches the existing ASCII-guard helper's format string at HEAD character-for-character; base blob passed `plan_id.to_string()` unsanitized |
| C5 sanitizer line is the ONLY production hunk; `new_files` stays unsanitized | PASS | `git show "fddb200" -- crates/cli/src/commands/ui/extraction_apply.rs` = one hunk, `plan_id: plan_id.to_string(),` -> `plan_id: ascii_only(plan_id),`; `new_files` render untouched |
| C4 oracle: plugin-spelling probe + both equalities verbatim + all surviving clauses kept | PASS | per-commit diff: the 12 deleted tests.rs lines are ONLY the `_plans` binding + the 11-line D6 block - every surviving clause byte-kept; helper realizes the DATED note literally (spellings from `plan_consumer_discovery_targets`, probe via `plan_backend` with the `caps.rewriting` selector and the `"rewriting"` label, PRE-APPLY read, sorted); both `assert_eq!` messages reconcile verbatim with the handoff literals under `\`-continuation; both sides sorted |
| D6 block replaced exactly per the handoff; fixed-claim comment names `extraction-consumer-rewrite` | PASS | diff hunk :280-290 -> :284-294 (11 lines -> 11); UNSATISFIABLE text gone |
| A-4 no-deletions assertion (observations row) | PASS | verbatim in `assert_modified_faithfully_applied` claim 3, `is_artifact_tree`-restricted, called from required test 1 |
| Required tests 2-11 + UTA pin + helpers byte-identical UNEDITED | PASS | hunks confined to test 1 region + appended helpers + end-of-file test (per-commit diff); all 13 module tests green in the reviewer's own census capture (`test_a_valid_token_applies_the_plan_end_to_end ... ok` line 271; caller-census test 11 `... ok` line 199) |
| Scoped cli green (+1 over 178 baseline) | PASS | `python tools/testrun.py --distill -p ariadne-cli` -> `179 passed / 0 failed / 8 ignored / 8 suites`, exit 0 (fresh reviewer run) |
| m20/m19 green UNEDITED/unweakened | PASS | `python tools/testrun.py --distill -p ariadne-integration-tests --test m20_apply_migrate_characterization_test --test m19_optimize_characterization_test` -> `11 passed / 0 failed / 0 ignored / 2 suites`, exit 0; neither file in `git diff --name-only f73a9a1..HEAD` |
| lang-cpp green UNEDITED (CW-1 fix stands under the oracle) | PASS | `python tools/testrun.py --distill -p ariadne-lang-cpp` -> `205 passed / 0 failed / 0 ignored / 2 suites`, exit 0 |
| Census +1 attributed (anchor 1834 @ CW-1) | PASS | `python tools/testrun.py --distill` -> `1835 passed / 0 failed / 13 ignored / 119 suites`, exit 0 (capture `target/testrun-capture.txt`, 3191 lines): +1 passed = the new plan_id test; failed/ignored/suites held |
| Bench attribution producer proof | PASS | `git diff --numstat f73a9a1..HEAD -- crates/cli/tests` -> empty (reviewer-run) |
| Touch-set containment + duty-1 byte-identity | PASS | `git diff --name-only f73a9a1..HEAD` -> exactly the 5 declared paths; `git diff ece3513..HEAD -- <both CW-1 artifacts>` empty and `git diff HEAD --numstat --` same, so the duty-1 blobs equal the reviewer artifacts byte-for-byte; Cargo.lock absent |
| Cosmetic erratum row discharged on zero-diff | PASS | journal carries the line-by-line verification; reviewer re-verified at HEAD: doc comment `extraction_apply.rs:243` at column 0, fn :244 column 0, chain indent 4/8 standard - no drift anywhere the comment sits |
| File metrics | PASS | `extraction_apply.rs` = 295 lines at HEAD (corrected fact: the hand is a 1/1 replacement, net 0 - the "999" dispatch claim and the "+1 line" phrasings are wrong; see verdict finding 2); `tests.rs` = 903 lines (< 1000, margin 97); test 1 = 95 fn lines |
| Hygiene | PASS | `python tools/hygiene_check.py` -> R1/R2 silent; warn-only R3 rows only, none in touched files |
| Encoding | PASS | `python tools/encoding_guard.py --all` -> silent (violations print before nonzero exit by design) |
| Report-evidence (distillation contract) | PASS | journal digest-shaped: distilling entry points, canonical lines, capture paths, verbatim Red, zero raw dumps |

## Workspace-wide gates (CW-2 reviewer hop, fresh)

This hand touched `crates/cli/src/commands/ui/extraction_apply.rs` production
source (one line), so the timing gate is NOT exempt. Canonical lines, verbatim:

- Full census: `1835 passed / 0 failed / 13 ignored / 119 suites`
  (`python tools/testrun.py --distill`, exit 0, capture
  `target/testrun-capture.txt`, 3191 lines)
- Tools suite: `Ran 296 tests in 26.663s` + `OK`
  (`python -m unittest discover -s tools -p "*_tests.py"`) - matches the
  ROUND-2 `Ran 296` OK anchor
- UI smoke: `ui_smoke: PASS` / `boot+route checks clean (cold fixture,
  attempt 1, 1.0s)` via `python tools/distill.py -- python tools/ui_smoke.py`
  (exit 0). Direct form refused again by the sandbox (allowlist row still
  missing - verdict finding 3).
- Timing gate: `bench compare: 3 run(s), no regressions past 1.25x baseline`
  via `python tools/distill.py -- python tools/bench_compare.py` (exit 0,
  capture `target/distill-capture.txt`, 17 lines; no WARN rows beyond the
  recorded no-regression verdict; attribution clean on the verified
  `crates/cli/tests` zero-diff)
- Hygiene: `python tools/hygiene_check.py` - R1/R2 silent
- Encoding: `python tools/encoding_guard.py --all` - silent

## Explicitly NOT run (CW-2 hop)

- `python tools/testrun.py --baseline` (bare, FILE form): refused by the tool
  itself with the verbatim usage error `usage error: missing value for
  --baseline` (the known M23.5 F9 defect, ledger row: no baseline file exists
  in the repo; creating one in the temp dir was denied to the reviewer's edit
  grant). The handoff's gate 6 governs instead: the `--distill` census
  substitute above is the census of record ("the `--distill` substitute
  governs, FILE-form errata stands"; shape section 9: "the `--baseline` FILE
  form stays superseded"). Identical count semantics; no weaker evidence used.
- The direct `python tools/ui_smoke.py` form and exit-code-echo forms
  (`python -c`, `echo`, pipe-`Select-Object` chaining): refused by the sandbox
  allowlist with the boilerplate catch-all deny (refusal text quoted in the
  verdict finding 3). Each gate ran to PASS through its sanctioned distilling
  entry point; where a digest header carries `PASS (exit 0)`, that is the exit
  evidence.

## Milestone status

CW-1: gates green, ACCEPTED (see `tasks/extraction-consumer-rewrite-cw1-verdict.md`).
CW-2 (cli faithfulness oracle): gates green, ACCEPTED (see
`tasks/extraction-consumer-rewrite-cw2-verdict.md`); closeout discharges
ledger rows 321 (CW-1), 322, 323 (`plan_id` half), 324 (zero-diff), 328
(no-deletions observation) there; the two copy relays (325, 326) remain for
CW-3 (shape C6). CW-3 (copy relay) pending; this file gains its section at
that hop.

## Handoff acceptance criteria (CW-3)

Reviewer hop 2026-10-23 against
`tasks/extraction-consumer-rewrite-cw3-handoff.md` INCLUDING the DATED
AMENDMENT. HEAD at gate time: `b201aa6` on `main` (duty-1 `701d7d7` + erratum
docs `90beaef` + landing `83f0b7c` + journal rider), start rev `d3f6dbc`.
All commands re-run fresh in this session.

| Criterion | Result | Evidence (verbatim) |
|---|---|---|
| Erratum chain is the sanctioned clarity-track pattern, not drift | PASS | `90beaef` is +88/-0 PURE insertion (top pointer +5, footer +83) declaring "supersedes ONLY the build-item-1 block text"; pin + all three absence strings in the footer byte-identical to the original mandates; docs-first landing followed (`90beaef` parent of `83f0b7c`); implementer stopped with mechanical witness (`370 passed / 1 failed / 0 ignored / 2 suites`, panic `:874:5`) and applied NOTHING of the candidates - the planner amended, then the hand resumed |
| hotspots.js flip byte-exact vs the AMENDED block, +2/-2 at :521-522 only | PASS | `git show "83f0b7c" -- crates/ui/assets/hotspots.js`: one hunk, two `-`/two `+`; :519-520 unchanged hunk-context; landed lines 519-522 byte-for-byte the erratum block (direct read); claim contiguous at :522; file 691 lines; zero code change |
| extraction-wizard.js note line +1/-1 at :386; COPY_VS_MOVE_SENTENCE in no hunk (X5) | PASS | one hunk; new line byte-equal the handoff item-2 mandate; constant at :219 absent from all diffs of the hand (`copyMoveLine` reference appears only as unchanged context) |
| Lock tests appended byte-exact; pure append; file < 1000 | PASS | single hunk `@@ -849,3 +849,42 @@` (+39/-0); names, pin, absence strings, messages all byte-equal the mandates (handoff :75-93 + erratum :245-255); file 851 -> 890 (Read: "total 890 lines"); `main.rs` mod line untouched (module-map ruling) |
| No-rot locks STRONG as the erratum intends | PASS | positive full-claim pin green against the raw blob and rejects head-negation/paraphrase (option (a) tail-pin weakness argument verified); absence locks pin the OLD FULL clauses - `"every consumer.</div>"` is not a substring of the new claim (trap avoided), `"never runs anything"` has zero asset occurrences at HEAD (grep); wizard positive green; claim TEXT byte-identical to the shipped A-5 help copy `index.html:553`/`:556` |
| Frozen locks green UNEDITED | PASS | fetch-site bound `<= 2` (:349-352), ASCII locks, P9/P10 `function_body(HOTSPOTS_JS, "renderExtractionApplyHint")` locks, theme contract, static_html/standalone-export presence - all in the byte-identical pre-append region and green in the reviewer's fresh `371 passed / 0 failed` run |
| Scoped suite = baseline 369 + the 2 locks (erratum mechanical expectation) | PASS | `python tools/testrun.py --distill -p ariadne-ui` -> `371 passed / 0 failed / 0 ignored / 3 suites`, exit 0 |
| Touch-set containment incl. duty-1 + authorized erratum path | PASS | `git diff --name-only d3f6dbc..HEAD` -> exactly 7 paths (3 code + journal + handoff-erratum + the two CW-2 reviewer artifacts); `git diff 701d7d7 -- <both CW-2 artifacts>` empty (duty-1 blobs byte-equal the reviewer artifacts, zero drift); Cargo.lock absent |
| Continuity zero-diffs | PASS | `git diff --name-only "d3f6dbc..HEAD" -- crates/cli/tests crates/lang-cpp tests/integration Cargo.lock crates/ui/assets/styles.css crates/ui/src crates/ui/tests/ui_test/main.rs` -> EMPTY (lang-cpp hook, m19/m20 files, CSS/server/DTO, fetch sites - no hand in this range touches any) |
| Duty-1 stat matches CW-2 verdict prescription | PASS | `701d7d7`: 415 insertions / 3 deletions = verdict +342/-0 + execution-results +73/-3; exactly the two declared paths; message verbatim |
| Deviations 1-8 dispositioned | PASS | verdict section 6: 1 ACCEPTED (the stop - ledger deviations-summary precedent), 2/7 ACCEPTED (workspace gates assigned to reviewer hop, all run fresh here), 3/4/5 ACCEPTED/record, 6 ACCEPTED (erratum judged on merits), 8 ACCEPTED (two commit cycles + rider mandated/precedent, fully itemized per the M28 law) |
| Hygiene | PASS | `python tools/hygiene_check.py` -> EXIT=0; warn-only R3 rows only, zero rows in the touched files |
| Encoding | PASS | `python tools/encoding_guard.py --all` -> silent, exit 0 (commit-cycle form `--all` superset, form journalled per gate 9) |
| Theme hex | PASS | `python tools/theme_token_check.py --check-added --base 701d7d7` -> silent, exit 0 |
| Report-evidence (distillation contract) | PASS | journal digest-shaped: canonical lines, exit codes, capture paths, verbatim Red/Blocker panics, routed entry points, zero raw dumps |

## Workspace-wide gates (CW-3 reviewer hop, fresh)

This hand is assets + tests only, but shape section 6 mandates the bench
recording per hand, so it was RUN, not exempted. Canonical lines, verbatim:

- Full census: `1837 passed / 0 failed / 13 ignored / 119 suites`
  (`python tools/testrun.py --distill`, exit 0, capture
  `target/testrun-capture.txt`, 3194 lines) - CW-2 anchor `1835/0/13/119` +2
  (the two locks), nothing else moved; the handoff gate-6 arithmetic
  (`1830 @ 765df7b` +4 +1 +2) reconciles with zero foreign residue
- Tools suite: `Ran 296 tests in 26.568s` + `OK`
  (`python -m unittest discover -s tools -p "*_tests.py"`) - matches the
  ROUND-2 `Ran 296` OK anchor
- UI smoke: `ui_smoke: PASS` / `boot+route checks clean (cold fixture,
  attempt 1, 1.1s)` via `python tools/distill.py -- python tools/ui_smoke.py`
  (exit 0, capture 2 lines) - gate 4 satisfied at the reviewer hop per
  dispatch; served-asset bytes changed and boot/routes/no-store hold
- Timing gate (RUN, not exempt): `bench compare: 3 run(s), no regressions
  past 1.25x baseline` via
  `python tools/distill.py -- python tools/bench_compare.py` (exit 0, capture
  17 lines; no WARN rows beyond the recorded no-regression verdict;
  producer attribution additionally clean on the empty
  `crates/cli/tests` zero-diff proof above)

## Explicitly NOT run (CW-3 hop)

- `python tools/testrun.py --baseline` (bare, FILE form): refused by the tool
  itself with the verbatim `usage error: missing value for --baseline`
  (M23.5 F9, unchanged); the `--distill` census above is the census of
  record per shape section 9 - identical count semantics, exit code + capture
  proven; no weaker evidence used.
- The direct `python tools/ui_smoke.py` form was probed this hop and refused
  again with the boilerplate catch-all deny
  (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`) -
  the CW-1/CW-2 allowlist row still un-landed (CW-3 verdict finding 1); the
  distill-routed form ran to PASS. Compound echo/pipe/`python -c` forms likewise
  refused; the `; $LASTEXITCODE` form runs and supplied EXIT=0/exit 0 for the
  static gates.

## Milestone status (final)

CW-3: gates green, ACCEPTED (see
`tasks/extraction-consumer-rewrite-cw3-verdict.md`); closeout discharges
ledger rows 325 + 326 against `83f0b7c` (journal discharge lines ready;
ledger:325 cites the AMENDED block), PRESERVES 327 (bound green UNEDITED),
leaves the 328 generation-swap half OPEN. All three hands landed and
approved. State of record @ `b201aa6`: census `1837/0/13/119`, tools `Ran 296`
OK, bench fresh no-regressions, `ui_smoke: PASS`.

