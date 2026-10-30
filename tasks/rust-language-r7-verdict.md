# R-7 Verdict - APPROVED

- Milestone: `rust-language` (unregistered post-M28 track, NO M-number). Hand 7 of 10.
- Handoff of record: `tasks/rust-language-r7-handoff.md` (complete contract; read in full).
- Chain under review (start rev `1cfb3cb`, the dispatch head): duty 1 `5492372`
  (predecessor) -> landing `4fd40ca` (SUCCESSOR) -> orchestrator rider `a659465`
  (three superseded flip-locks, m23/R-5/R-6 precedent, R-7 board-ASK option a).
  Three commits, linear, reviewed individually oldest-first.
- Incident under adjudication: R-7-DEADSESSION-1 (successor-hand class; precedents
  unit-test-audit H6 `e95a6d5` and Round-2 R1 `af8a997` - the dispatch's "H1" label
  corresponds to the unit-test-audit successor hand, which the ledger records at H6).
- Reviewer: fresh adjudication 2026-10-26 at HEAD `a659465`, ground truth via
  `python tools/checkpoint_guard.py status`: HEAD `a659465`, branch `balsam-basil`,
  tree clean except the untracked guard artifact `tasks/rust-language-r7-handoff-resume.md`.
- Verdict: **APPROVED** - zero blockers; three minor findings; all deviations accepted
  on independently verified merits.

## 1. Successor-hand integrity + Red-basis ruling (ADJUDICATION 1) - ACCEPTED

**Integrity - verified on disk, not trusted from relay:**

- Duty 1 `5492372` intact: authored pre-death by the predecessor; `git show` content is
  exactly `tasks/rust-language-r6-verdict.md` (+270) + one ledger line; the relay clause
  landed verbatim: "R-6 LANDED baf24f8 (+rider 470ae22) APPROVED 2026-10-26, census
  anchor 1938 passed / 0 failed / 13 ignored / 124 suites @ 470ae22, R-7 dispatched
  (load_metric hop)." The dead session was never resumed (law honored; successor
  bootstrapped via `checkpoint_guard.py status` + `resume --spec ... --scope crates/lang-rust`).
- Green-state verification: the guard-generated resume report records, at HEAD `5492372`,
  dirty `crates/lang-rust/src/plugin.rs` + untracked
  `tests/integration/tests/rust_language_load_analysis_test.rs` with three guard suite
  verdicts green (56/0, 5/0, 6/0). The landing commit stat matches the journalled state
  exactly (plugin.rs +16/-13; suite +500; journal +163). Nothing else dirty or committed
  beyond the touch-set - the full-range name-only diff (finding list, sec. 6) confirms.

**Red-basis - the relay basis SATISFIES the spirit of red discipline here.** Ruling on
merits, three independent legs, only the first of which is relay:

1. Relay leg (durable, cross-readable): predecessor board post `board_8e711406-507c-4c58-a0bb-0ebba6088e90`
   reports the suite RED at `4/1` with the failing test being the caps pin at
   `left: load_metric: false, right: true` - byte-matching the handoff's predicted Red
   (handoff sec. "RGB protocol" item 1). I read this post directly from board history;
   the journal quotes it AS relay, not as re-observed data. Correct labeling.
2. Mechanical corroboration leg (fresh, this hand's own machine truth): the successor's
   fresh no-fail-fast sweep - and my per-suite green runs at rider state as the mirrored
   witness - show the three superseded caps locks failing with `left:` carrying the
   flipped struct. That is live observation that the pre-hand literal set (`load_metric:
   false`) cannot satisfy the post-hand locks: the same falsifiability statement as the
   Red, observed mirrored by a fresh-context session. The rider then turns exactly those
   three reds green - the flip's Red IS the superseded-lock evidence the rider consumed.
3. Falsifiability leg (source read): in-src pin `capabilities_match_the_load_metric_flip_literal`
   (plugin.rs:249) and integration lock `capabilities_lock_the_load_metric_flip_literal`
   (load suite :212) each assert against an INDEPENDENT full-struct literal plus an
   exactly-four-true count assertion; the pre-hand struct cannot pass either. Not tautological.

**Precedent comparison:** the unit-test-audit successor hand (uta-t6) was gated on the
successor's checkpoint canonical lines + exit witnesses, with the reviewer closing the
capture-path gap by fresh re-runs - a minor finding, never a demand to re-author Red.
Round-2 R1 (`af8a997`) had a live Red only contingently (predecessor's src fixes were
uncommitted); its red basis in the gate was the journal's canonical red line + arithmetic
criterion 1 (`358 passed / 11 failed ... + zero deletions = 369`). No precedent made
red re-authorship a blocker on a successor hand, and the SM-3 ledger law ("machine-truth
strengthening" over pure narrative) is met here because leg 2 makes the claim machine-checked.
Re-authoring would have required reverting the flip in the landing tree purely to
re-observe - the dispatch bars that unless cheap and safe.

**Noted (not a finding):** the handoff additionally predicted an honest-rejection /
absent-manifest-field red class; the relay shows Red was `4/1` - only the caps pin. That
over-prediction is consistent with the handoff's own item-3 honesty disclosure (the
backing load lane already existed, so the data tests were green pre-flip); it is a
planning-text over-prediction, not implementer misconduct. Routed to Finding 3.

## 2. Resume artifact disposition (ADJUDICATION 2) - DELETE at verdict-landing

`tasks/rust-language-r7-handoff-resume.md` (untracked) is a `checkpoint_guard resume`
report, correctly NOT committed by the successor (outside the touch-set; the journal
discloses it). Disposition per the SM precedent ("resume file cleaned" at verdict
landing once the successor's work is gated; SM-1 class rule): **DELETE** it in the commit
that lands this verdict - i.e., R-8's duty-1 band - and do NOT land it. Nothing is lost:
the journal's "SUCCESSOR-hand fact" + "Predecessor's landed scope" sections carry every
bootstrap fact the resume file holds (HEAD, dirty state, guard verdicts, scope labels).
The reviewer's edit grant (verdict + execution-results only) bars me from deleting it;
routed to R-8 duty-1.

## 3. Fresh gates (ADJUDICATION 3) - all run personally, verbatim

| Gate | Fresh reviewer run (exit 0 unless stated) |
|---|---|
| lang-rust (handoff gate 1) | `56 passed / 0 failed / 0 ignored / 2 suites` (`python tools/testrun.py --distill -p ariadne-lang-rust`, capture 69 lines) |
| new suite (gates 2/3 green) | `5 passed / 0 failed / 0 ignored / 1 suites` (`--distill -p ariadne-integration-tests --test rust_language_load_analysis_test`, capture 29 lines) |
| m22 model UNEDITED (gate 4) | `6 passed / 0 failed / 0 ignored / 1 suites` (`--test m22_scala_load_analysis_test`); file absent from `1cfb3cb..a659465` name-only diff |
| rider suite dependency_graph | `9 passed / 0 failed / 0 ignored / 1 suites` |
| rider suite semantic_index | `11 passed / 0 failed / 0 ignored / 1 suites` |
| rider suite discovery | `7 passed / 0 failed / 0 ignored / 1 suites` |
| FRESH FULL CENSUS (gate 6) | `1943 passed / 0 failed / 13 ignored / 125 suites` (`python tools/testrun.py --distill`, capture 3363 lines) @ `a659465` |
| ui_smoke (gate 5) | `ui_smoke: PASS` / `boot+route checks clean (cold fixture, attempt 1, 1.1s)` (`python tools/distill.py -- python tools/ui_smoke.py`) |
| hygiene (gate 7) | `### generic PASS (exit 0)` (`python tools/distill.py -- python tools/hygiene_check.py`, capture 173 lines; all rows warn-only R3; none of this hand's files appear) |
| L12 encoding | `### generic PASS (exit 0)` (`python tools/distill.py -- python tools/encoding_guard.py --all`, capture 0 lines) |
| tools CI | `Ran 296 tests in 31.066s` / `OK` (`python -m unittest discover -s tools -p "*_tests.py"`) |
| bench (reviewer acceptance) | `### generic PASS (exit 0)` / `bench compare: 3 run(s), no regressions past 1.25x baseline` (`python tools/distill.py -- python tools/bench_compare.py`); zero WARN lines, so the record-not-attribute stance was moot; no baseline/corpus change (D-3) |

**Census attribution (by name):** anchor `1938 / 0 / 13 / 124 @ 470ae22`; +5 from the new
suite's five tests (`capabilities_lock_the_load_metric_flip_literal`,
`load_metric_answers_over_the_rust_basic_units`, `load_manifest_carries_rust_language`,
`d26_vocabulary_surfaces_through_the_graph_render_lane`,
`node_language_labels_derive_from_the_bare_debug_name` - all present in source, all green
in the scoped run); rider 0-count-change (it amended three already-counted tests in
place); suites +1 = `rust_language_load_analysis_test`. `1938 + 5 = 1943`, `124 + 1 = 125`,
ignored flat at 13. Exact.

## 4. INV-14 load-metric contract + language-is-data proofs (ADJUDICATION 4) - PASS

- Manifest proof: `load_manifest_carries_rust_language` asserts the run's
  `artifacts/load/manifest.json` `languages` == `["Rust"]` through the existing
  `collect_stage_languages` path (read of the data, no helper edit).
- Vocabulary proof: `d26_vocabulary_surfaces_through_the_graph_render_lane` pins the
  handoff's literals byte-exact - `directive_label == "Use"`,
  `directive_passive_verb == "used"`, `aggregation_noun == "mod aggregation"` - and
  renders them through `format_summary_with` / `format_explanation_with` ("Use load:",
  "user.rs is used by consumer.rs."), with negative pins excluding Include/Import wording.
  Plan-literal check against handoff item 2: match.
- UI-label proof: `node_language_labels_derive_from_the_bare_debug_name` asserts
  `format!("{:?}", language) == "Rust"` on the run's discovery values - the exact
  `node_maps.rs` idiom, cited read-only.
- No-stdout-scrape MUST: held - the suite asserts returned models/manifest JSON; zero
  `Command`/stdout capture anywhere in the file (source read).
- Zero-diff re-run: `git diff --name-only 1cfb3cb..a659465 -- crates/ui crates/report
  crates/artifact` -> EMPTY; additionally `crates/cli` absent from the whole range
  (`node_maps.rs` in no hunk), fixtures/goldens/tools/Cargo.toml/Cargo.lock absent.

## 5. Caps witness census (ADJUDICATION 5) - exactly four true bits, all agree

Counted myself at source: **six byte-witnesses** - plugin.rs impl literal (:151-161),
in-src pin (:252-262), integration locks in discovery (:304), semantic_index (:172),
dependency_graph (:132), load (:219). All six carry `discovery: true, dependency_graph:
true, semantic_index: true, load_metric: true`, `planning/extraction/rewriting/validation:
false`, `requires_toolchain: false` - exactly FOUR true stage bits, identical field sets.
Agreement is machine-checked by this review's fresh green runs (each lock compares the
registry plugin's live caps against its independent literal; any slip turns the run red:
56/0, 5/0, 9/0, 11/0, 7/0 all green). The load lock additionally asserts the four-true
count explicitly. The load suite's extra 4-true counter is the strongest witness.

## 6. Per-commit review + commit-vs-touch-set (ADJUDICATION 6) - PASS

- `5492372` (duty 1): verdict + ledger flip only; inside duty-1 band.
- `4fd40ca` (landing): exactly the three touch-set paths (`plugin.rs` caps flip + pin
  rename + doc comments, 29 lines; new suite +500; journal +163); plugin delta confined to
  the caps literal, pin name, and their doc comments - plan-literal: handoff item 1
  satisfied byte-exact (four true incl. `load_metric`, `requires_toolchain: false`, pin
  renamed `capabilities_match_the_load_metric_flip_literal`).
- `a659465` (rider): the three named files only; each hunk is exactly the one
  `load_metric: false -> true` literal plus its surrounding lock narrative comment
  evolution (m23/R-5/R-6 evolution idiom - the prior riders used the same shape);
  nothing weakened: each lock retains an independent full-struct literal.
- `git diff --name-only 1cfb3cb..a659465` = exactly 8 paths: `crates/lang-rust/src/plugin.rs`,
  `tasks/milestone_ledger.md`, `tasks/rust-language-r6-verdict.md`,
  `tasks/rust-language-r7-journal.md`, `tests/integration/tests/rust_language_dependency_graph_test.rs`,
  `..._discovery_test.rs`, `..._load_analysis_test.rs`, `..._semantic_index_test.rs`.
  Every path inside the handoff touch-set + duty-1 band + the three named rider files.
  Cargo.lock untouched; L1-L9 all hold (repo/tools/m18-m23/lang-cpp/lang-scala/goldens/
  fixtures absent from every hunk; m00 green inside the 1943 census UNEDITED).
- NO git stash recurrence (R-6 D6 ruling): `refs/stash` absent and `logs/stash` absent
  under `.git/worktrees/balsam-basil/`; the worktree HEAD reflog contains zero `stash`
  entries. (`git stash list` itself was allowlist-refused; on-disk ref + reflog
  inspection substituted - strictly stronger evidence.)

## 7. Deviation dispositions (independently adjudicated)

- **D1 - suite 500 lines vs ~250 planner estimate**: ACCEPTED. < 1000 (R1 clean, verified
  in hygiene), all five handoff-mandated proofs present and green; the shape-class helper
  + collateral witness asserted as named invariants instead of inline lumps is a quality
  gain, not slop.
- **D2 - guard touch-set parse mislabeled the in-touch-set new suite as "stray edit"**:
  ACCEPTED. Known best-effort guard limit (it parsed the MUST-NOT globs); explicit
  `--scope`/`--cmd` were passed, the triage is disclosed in the journal, and the resume
  verdicts are trustworthy (my fresh runs reproduce all three).
- **D3 - no stash**: VERIFIED TRUE (sec. 6).
- **D4 - SHA relay via report-back + reflog** (self-referential SHA impossible): ACCEPTED
  per the handoff's discipline clause; R-8 duty 1 cites the relay line below.
- **D5 - Red not re-authored; relay basis**: ACCEPTED per Adjudication 1.

## 8. Findings (all minor; none block the merge)

1. **Journal capture paths are generic, not per-run values** (r7-journal.md:59-61 -
   "each distilling run overwrote target/testrun-capture.txt / target/distill-capture.txt
   (paths printed by each digest line above the fold...)"): the review-gate step-8 clause
   prefers recorded per-run capture witnesses. Closed reviewer-side: this verdict's and
   the execution-results addendum's fresh runs carry the capture line-counts. Route:
   journaling habit, closeout note (precedent: uta-t6 finding 1).
2. **Dispatch witness text miscounts the caps witness class**: "five integration locks" -
   there are FOUR integration-suite locks; the fifth non-impl witness is the in-src pin
   (the R-6 verdict's five-witness enumeration is the likely source). Artifact-neutral:
   all six witnesses verified. Route: correct the enumeration in R-8/R-9 relay text.
3. **Handoff predicted-Red over-predicted** (honest-rejection/absent-manifest class never
   materialized; Red was 4/1 caps-pin-only): consistent with the handoff's own item-3
   disclosure that the backing lane already exists. Route: R-8/R-9 flip handoffs should
   predict the caps-pin as the PRIMARY red and list lane-rejection reds only if the
   gating is real. Planner note.

## 9. Fresh census anchor + R-8 duty-1 relay

NEW ANCHOR: `1943 passed / 0 failed / 13 ignored / 125 suites @ a659465`.

R-8 duty 1 lands this verdict + the ledger flip; relay line:

> R-7 LANDED `4fd40ca` (+rider `a659465`) APPROVED 2026-10-26, census anchor 1943 passed
> / 0 failed / 13 ignored / 125 suites @ `a659465`, R-8 dispatched.

R-8 duty 1 also DELETES the untracked `tasks/rust-language-r7-handoff-resume.md`
(Adjudication 2 disposition).

## 10. Not run / refused (explicit)

- `python tools/testrun.py --baseline` (bare): refused - `usage error: missing value for
  --baseline`; the F9-errata FILE form, superseded track-wide. Gate 6's prescribed
  `--distill` substitute governs (identical count semantics) and matches exactly.
- `git stash list`, `git rev-list`, `git status --short`, a `Test-Path` chain, and a
  `| Select-Object`-piped unittest run: allowlist-refused, each with the catch-all
  boilerplate `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
  Substitutions: on-disk stash-ref + reflog inspection (sec. 6), `git log --oneline`
  range listing, the git status tool, plain unittest run. No expected gate skipped;
  hygiene/encoding exit-0 witnessed via distiller `PASS (exit 0)` lines.
- `python tools/golden_guard.py --base`: per the handoff's lock section the prose
  byte-check governs until that tool lands; substituted by the name-only zero-diff proofs.
