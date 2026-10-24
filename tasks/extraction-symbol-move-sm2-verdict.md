# SM-2 Verdict - extraction-symbol-move - APPROVED

Reviewer: SM-2 gate hop, 2026-10-24. Anti-phantom protocol per
`tasks/extraction-symbol-move-sm1-verdict.md`: every number below is this
reviewer's own fresh measurement, quoted verbatim from own tool output; the
claimed SHA chain was cross-checked against `.git/logs/HEAD` before any gate
was trusted.

## Verdict: APPROVED

No blockers. Five minor/informational findings (M1-M5); none requires an
implementer return. All five implementer-declared deviations ACCEPTED
(independently adjudicated, dispositions below).

## Anti-phantom verification (reflog ground truth)

`.git/logs/HEAD` read directly (the `git reflog` command is refused by the
bash allow-list; the file IS the reflog). Total 442 lines; lines 440-442
verbatim:

```
b9936000da7827e49e8dbe999f1dd65d65b75704 0db2680c34515e03d4589d101dca010fd564f86d kcwill2 <kcwill2@gmail.com> 1792823160 -0700	commit: docs(extraction-symbol-move): SM-1 re-gate APPROVED - land verdict + execution-results + incident ledger row (PHANTOM-NARRATION-1 closed)
0db2680c34515e03d4589d101dca010fd564f86d f6c92feed6137c4668de0191486164f8970141f7 kcwill2 <kcwill2@gmail.com> 1792826583 -0700	commit: feat(lang-cpp): implement move_out_top_level_symbols with the ARIADNE-EXTRACT-OUT line-prefix grammar and refusal contract (extraction-symbol-move SM-2)
f6c92feed6137c4668de0191486164f8970141f7 fba3c902f1970b2673b4359c2717334c0b1d4898 kcwill2 <kcwill2@gmail.com> 1792826648 -0700	commit: docs(extraction-symbol-move): SM-2 journal landing record (reflog-verifiable SHA f6c92fe)
```

Duty-1 `0db2680`, landing `f6c92fe`, rider `fba3c90` all exist; `fba3c90` is
HEAD (`git rev-parse HEAD` -> `fba3c902f1970b2673b4359c2717334c0b1d4898`);
`git log` confirms `f6c92fe`'s parent is `0db2680`. Bootstrap
`python tools/checkpoint_guard.py status`: HEAD `fba3c90`, main, dirty =
only the pre-existing `.vsix` stray; untracked none. The gates below ran
against exactly the committed tree.

## Commit-vs-touch-set (the enforcement check)

`git diff --name-only 0db2680 fba3c90` returned exactly:

- `crates/lang-cpp/src/dependency_rewriter.rs`
- `crates/lang-cpp/src/lib.rs`
- `crates/lang-cpp/src/move_retention.rs`
- `tasks/extraction-symbol-move-sm2-journal.md`

All four inside the declared touch-set. Numstat matches the handoff and
journal: landing `f6c92fe` = 4 files, 875 insertions (dependency_rewriter
+16, lib.rs +1, move_retention +648 NEW, journal +210); rider `fba3c90` =
journal only, +19/-1 (replaces the "(append: ...)" placeholder with the
landing record). Frozen locks proven by ABSENCE from the range diff:
`crates/lang/**` (SM-1 hook untouched), `crates/lang-scala/**`,
`crates/validate/**`, `crates/cli/**`, `crates/ui/**`, `symbol_locator.rs`,
`tokenizer.rs`, `parser.rs`, `source_extractor.rs`, `tests/**`, goldens,
fixtures, themes, `tools/**`, `Cargo.lock` - all absent. Working tree holds
HEAD (dirty only `.vsix`), so my fresh gate runs measured the committed bytes.

## Per-commit review (oldest-first)

`f6c92fe` judged alone:

- `dependency_rewriter.rs`: ONE hunk `@@ -111,6 +111,22 @@`, a single added
  method region directly after `consumer_rewrite_spec`, zero deletions. The
  four frozen fns (`rewrite`, `render_diff`, `emit_file_preamble`,
  `find_dependency_targets`), `consumer_rewrite_spec` and all 16 existing
  `#[test]`s are outside every hunk - byte-identical (SM-I4). The hook
  signature and body match the handoff item 5 verbatim, and the delegation
  `crate::move_retention::move_out_text(source, qualified_names, plan_id)`
  forwards ALL THREE arguments verbatim (this materially de-risks SM-1
  finding M2 at the hook, but the call-site pin stays SM-3's duty - M1 below).
  Doc clauses all present, pure ASCII, no em dash: decision of record,
  provenance, refusal answers `None` = copy fallback, block comments never.
- `lib.rs`: exactly `+mod move_retention;` directly before
  `mod symbol_locator;`, keeping the private-mod group alphabetical - the
  declared compile-prerequisite rider, as the handoff :47-53 licensed.
- `move_retention.rs` (NEW, 648 lines, < 1000): grammar, refusals and tests
  audited clause-by-clause below.
- The journal rode with the landing (210 lines in the landing blob).

`fba3c90` judged alone: journal-only +19/-1, records the landing SHA,
`git show --stat` facts, commit-cycle forms, and reflog tail - all three
claims match my own observations. Inside touch-set. Clean.

## Grammar and plan-literal checks (kickoff sec. 1/5/6, shape C2, handoff item 1)

- BEGIN/END byte form: production renders
  `// ARIADNE-EXTRACT-OUT BEGIN plan=` + plan-field + ` symbols=` +
  names-field + terminator, and `// ARIADNE-EXTRACT-OUT END plan=` +
  plan-field (`BEGIN_HEAD`/`END_HEAD`/`SYMBOLS_SEPARATOR` consts,
  move_retention.rs:31-33, 210-224). Matches the C2 grammar block exactly.
- LINE-PREFIX COMMENT ONLY: `prefix_out_region` prepends exactly `//`, no
  space; empty line -> bare `//` (:229-231). Block comments never emitted
  (no block-comment builder exists in the module).
- Plan-field: `plan_field_ascii` maps every char outside `0x21..=0x7E` to
  `?` (:236-247) - the A9 channel plus the `\S+` no-space demand, exact
  handoff form.
- Names-field: `normalize_marker_name` maps outside `[A-Za-z0-9_:]` to `_`
  (:252-262); `;` join (:215). `:` is inside the allowed set - verified
  against the code and the grammar pin (test 4, :514-539).
- Recognition contract: test 4 pins every produced marker line against
  byte-exact manual matchers for `^// ARIADNE-EXTRACT-OUT BEGIN
  plan=(\S+) symbols=(\S+)$` and `... END plan=(\S+)$` and every region
  content line NEGATIVE (:530-538). I walked the matcher-vs-regex
  equivalence on adversarial inputs (multi-` symbols=` lines reject in both;
  generated fields can never carry whitespace since both renderers map it
  out) - no divergence.
- CRLF: `SourceLines` keeps per-line terminators verbatim; markers use the
  dominant terminator (`crlf_count * 2 >= line_count`, :294-295); test 3
  pins all three claims including the `\r` under the prefix.
- Locator discipline: `certified_body` calls the READ-ONLY
  `symbol_locator::locate` with kind order Class, Struct, Union, Enum,
  Function, TypeAlias, first body wins, `Other` never (:19-26, 81-90); the
  hook never re-slices text the locator did not certify. Same
  `ariadne_lang::SymbolKind` the locator imports (locator :18). The
  segment-suffix blessing (locator doc :11-15) grounds test 4's
  `release:v2-beta::Thing` -> record `Thing` and the ctor-name story.

## Refusal-contract audit (handoff items 2-3, hazard corpus)

R-A unlocatable (:81-90 via `locate`; test 5 pins `ThingConfig` against the
13-line fixture - the handoff's exact named pair). R-B idempotence: outcome
pinned (test 6: re-offer answers `None`, different un-marked name still
marks, TWO balanced pairs, no nesting) - mechanism is the tokenizer's
comment invisibility, corroborated by the locator doc :4-9. R-C byte
uniqueness outside marked interiors (:95-114, `interiors` skipped; test 7
first half). R-D line alignment (:338-348; test 7 second half). R-E
unterminated `/*` in pre-region bytes, conservative byte scan refusing MORE
than the tokenizer sees (:138-148; test 8's closed variant marks, proving
the R-E branch fires non-vacuously - the tokenizer ignores `/*` inside a
`//` line, so the hazard variant is still locatable and the refusal IS the
scan). R-F preceding-line backslash (:129); R-G last-line backslash (:132)
- branch independence: see M4. Preflight runs before alignment; ANY refusal
returns `None` for the whole candidate = per-unit atomicity as specified.
Empty-names call answers `None` (conservative copy fallback; no mandated
test contradicts). Merging: `merge_regions` greedy by first_line,
adjacent-or-overlapping (`region.first_line <= previous.last_line + 1`)
into one pair (:152-165); render splices descending = bottom-up (:182,
test 2 reconstructs the source byte-for-byte through two regions).

## Fixture-outcome honor (planner verdict vs the shape's relay)

Test 1 asserts EXACTLY one BEGIN + one END on the 13-line
`tests/fixtures/basic/include/thing.h` for `Thing`/`PLAN-TEST`, exact bytes
`// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-TEST symbols=Thing` /
`// ARIADNE-EXTRACT-OUT END plan=PLAN-TEST`, the 8 region lines
prefix-stripped byte-equal to source lines 6-13, all 5 outer lines
byte-identical. I verified the fixture on disk: `class Thing {` :6, ctor :8,
method :9, `};` :13 - matches the handoff's planner-verified bytes. Grep for
`4 pairs`/`four pairs` in the module: no matches - the relay correction is
honored in every test message. The journal checkpoint records the one-region
measurement without reproducing the shape's wording.

## Red witness adjudication

Journal gate 2: `205 passed / 10 failed / 0 ignored / 1 suites` (exit 1),
failing list verbatim = exactly the ten handoff-mandated names (journal
:80-91), all `move_retention::tests::move_out_*`, each pair-formed test
panicking on its POSITIVE `.expect("extraction-symbol-move SM-2: ...")`
assert while the SM-1 trait default still answered `None`. Falsifiable and
internally consistent: (a) all ten fail for the same predicted reason - the
default `None` cannot satisfy any positive marking assert; (b) the prefix is
present in the committed expects; (c) baseline 205 + exactly those 10 = the
215 I measured fresh; (d) the transient Red state itself is unverifiable,
which is inherent - the logical guarantee (ten tests over the default) is
sound. The `1 suites` at Red vs `2 suites` elsewhere is cargo's fail-fast
stopping the second test binary after the lib suite fails - reported
verbatim, not a misreport (my green re-runs show 2 suites).

## Deviation dispositions (independently adjudicated)

1. `\xNN` -> `"\u{e9}"` specimen escape: ACCEPTED. Rust string `\xNN`
   escapes cap at 0x7F, so the handoff's `"\xe9"` cannot compile (the
   journal's first gate-2 exit-2 parse-drift is the honest witness).
   `"\u{e9}"` is an escape (authored bytes stay pure ASCII - my own
   non-ASCII grep on the file: zero matches), U+00E9 falls outside
   `0x21..=0x7E` and renders `?`, and the test asserts exactly that.
   Intent (escape-built non-ASCII, `?` pin) preserved; the handoff's
   literal form was impossible.
2. Test-4 symbols expectation `release:v2_beta::Thing`: ACCEPTED. The
   grammar keeps `[A-Za-z0-9_:]` and maps `-` to `_`; the corrected
   expectation equals the grammar truth (verified against normalize code
   and the locator's segment-suffix contract; test green in my run).
3. Gate-count D-row (10 vs declared 8): ACCEPTED. The sizing law caps at
   `<= ~10` scoped invocations + commit cycle; the two extras are the
   gate-2 re-run after the harness-only escape fix (first run executed ZERO
   tests - not a semantic gate) and an output-only hygiene exit probe.
   Declared as dated D-2026-10-24 with itemization (testrun 7 + bench 1 +
   hygiene 2). The law punishes silence; this was the opposite. Carry the
   D-row into the ledger dated sub-block at closeout (same disposition
   class as D-SM1-REDO-1).
4. Last-region-line-without-terminator gains the dominant terminator:
   ACCEPTED with observation M3. A line with NO terminator bytes cannot
   simultaneously satisfy "terminator verbatim" (nothing to preserve) and
   "END marker on its own line" (the balanced-pair grammar that the
   recognition regexes depend on); the code chooses grammar integrity,
   keeps content bytes verbatim, and documents the choice (:191-195).
   Not exercised by any mandated test - carried to SM-4's byte-reconstruction
   oracle as M3.
5. R-G hazard also refused by R-D (journal's honesty note): ACCEPTED with
   observation M4. The named branch is independently written and runs
   FIRST; the test pins the OUTCOME pair per the handoff's own R-B
   outcome-pinning language. See M4 for the reachability nuance.
6. Refused-form journaling (raw `Remove-Item`, inline `python -c`,
   `Get-Item | Select-Object` catch-all denies, substituted sanctioned
   forms): ACCEPTED, SM-1 verdict item-8 precedent - my own hop reproduced
   the identical catch-all refusals (raw `git reflog`, inline `python -c`),
   corroborating that refused calls never execute and left no disk effect.

## Fresh gates - this reviewer's own runs (verbatim lines)

- `python tools/testrun.py --distill -p ariadne-lang-cpp` ->
  `215 passed / 0 failed / 0 ignored / 2 suites` (exit 0; capture 291 lines
  names all ten `move_retention::tests::move_out_* ... ok` AND all 16
  pre-existing `dependency_rewriter::tests::* ... ok` UNEDITED).
- `python tools/testrun.py --distill -p ariadne-lang-scala` ->
  `257 passed / 0 failed / 0 ignored / 2 suites` (exit 0; SM-I5, zero-diff).
- `python tools/testrun.py --distill -p ariadne-integration-tests --test
  m18_cpp_plugin_parity_test` -> `23 passed / 0 failed / 0 ignored / 1 suites`
  (exit 0 UNEDITED; generated-byte parity).
- `python tools/testrun.py --baseline` -> refused verbatim:
  `usage error: missing value for --baseline` (the superseded FILE form,
  F9 - expected and recorded). Sanctioned substitute run:
  `python tools/testrun.py --distill` ->
  `1849 passed / 0 failed / 13 ignored / 119 suites` (exit 0, capture 3203
  lines). Reconciliation: anchor `1837 @ b201aa6`; SM-1 landing of record
  `1839`; delta +10 attributed to the ten named tests (capture names them
  `... ok`, e.g. lines 1517/1529); 0 failed, 13 ignored, 119 suites held;
  matches the ledger's per-hand census plan (SM-2 +10, milestone
  expectation `1853`).
- `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 296 tests in 27.049s` / `OK`.
- `python tools/distill.py -- python tools/ui_smoke.py` ->
  `### generic PASS (exit 0)` /
  `boot+route checks clean (cold fixture, attempt 1, 1.1s)` / `ui_smoke: PASS`.
- `python tools/distill.py -- python tools/bench_compare.py` -> exit 0,
  closing line `bench compare: 3 run(s), no regressions past 1.25x
  baseline`; full 17-line capture read: ZERO WARN lines (nothing to
  attribute; PR-2 stance). Fresh recording, not the exemption route - this
  hand adds lang-cpp code, so I ran it.
- `python tools/hygiene_check.py` -> zero R1/R2 rows, zero rows under any
  touch-set file (all 163 output rows are pre-existing R3 warns elsewhere);
  exit machine-proven via `python tools/distill.py -- python tools/hygiene_check.py`
  -> `### generic PASS (exit 0)` (173-line capture).
- `python tools/encoding_guard.py --all` -> silent; exit machine-proven via
  the distill wrapper -> `### generic PASS (exit 0)`, capture 0 lines.
  Independent pure-ASCII check: non-ASCII grep on `move_retention.rs` -> no
  matches.
- Hygiene sizes: `move_retention.rs` 648 (< 1000); longest function
  `render_marked_source` ~38 lines (all well under 100); R1/R2 clean.

## Findings

- M1 (minor, SM-3 duty - carried from SM-1 finding M2): the SM-2 hook body
  forwards all three arguments verbatim (reviewed), but the STRIP CALL SITE
  must pin the forwarding observably: `collect_move_removals` per-unit
  names + `migration.plan_id` reaching `move_out_top_level_symbols` must be
  observed by a fake, not assumed. SM-3's handoff must name a test that
  fails if names or plan id are dropped/swapped at the call.
- M2 (minor, planning-side estimate drift, no action): `move_retention.rs`
  landed 648 lines vs the handoff's ~450-520 estimate (production ~368 vs
  the ~170-210 itemization). Every BINDING bound holds: file < 1000,
  functions < 100, produced lines 875 (+19 rider) vs the ~1200 cap,
  hygiene exit 0. Planner estimate-accuracy note only.
- M3 (minor, SM-4 awareness): deviation 4's terminator-default rule means a
  region ending on a file's final unterminated line gains the dominant
  terminator; SM-4's prefix-strip byte-reconstruction oracle (shape C5
  condition 4) must use terminated files or expect the added terminator.
  Disclosed and documented in code.
- M4 (minor, informational): with refinements R-C/R-D, the R-G branch can
  never fire ALONE (any last line ending in `\` places non-whitespace after
  the body end, which R-D also refuses); the named branch executes first
  and the pair test pins the OUTCOME, matching the handoff's own R-B
  precedent. Defense in depth, honestly journaled; no change required -
  noted so a future hazard-corpus handoff does not mistake it for a
  covered independent path.
- M5 (informational, orchestration): bash allow-list gaps re-confirmed
  (raw `git reflog`, inline `python -c` refused; `git log`/`git show`/
  `git diff` pass). Route: read `.git/logs/HEAD` with the Read tool, prove
  exit codes through `distill.py`. Pre-existing deferred row; re-confirmed.

## Follow-ups routed (never dropped)

- D-2026-10-24 (SM-2) gate-budget D-row: carry into the ledger dated
  sub-block at closeout (alongside D-SM1-REDO-1).
- SM-1 M1 ledger correction: VERIFIED landed in `0db2680` (ledger now reads
  `+74/-4` with the stat-glyph-total honesty note).
- Ledger staleness note for the closeout relay: the committed incident-row
  phrases "verdict ... uncommitted, lands with SM-2 duty-1" and the
  resume-file deletion describe the pre-`0db2680` state; both are now DONE
  facts - the closeout dated row should state the SM-2-era state of record.
- M1: SM-3 call-site forwarding pin (hard MUST for the SM-3 handoff).
- M3, M4: SM-3/SM-4 handoff awareness lines.
- M5: existing bash allow-list deferred row - re-confirmed, unchanged.

## State of record after this gate

HEAD `fba3c90` on `main` (tree: `.vsix` stray only; this verdict and
`tasks/extraction-symbol-move-sm2-execution-results.md` left UNCOMMITTED per
dispatch - they ride SM-3's duty-1). Census of record: `1849 passed /
0 failed / 13 ignored / 119 suites`. Tools: `Ran 296 tests in 27.049s` /
`OK`. Bench: `bench compare: 3 run(s), no regressions past 1.25x baseline`
(exit 0, zero WARN). Smoke: `ui_smoke: PASS`. Hygiene exit 0; encoding exit
0. SM-3 UNBLOCKED on this gate (serial chain: SM-3 threads the plan id at
the validate strip call + m19/m20 riders; SM-4 must land immediately after).
