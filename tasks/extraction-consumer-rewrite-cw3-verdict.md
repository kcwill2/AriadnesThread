# CW-3 Verdict - extraction-consumer-rewrite: APPROVED

Reviewer hop 2026-10-23 (review-gate). Adjudicates
`tasks/extraction-consumer-rewrite-cw3-handoff.md` INCLUDING the DATED
AMENDMENT (planner erratum), chain `701d7d7` (duty-1) -> `90beaef` (erratum
docs) -> `83f0b7c` (landing) -> `b201aa6` (journal rider, HEAD) on `main`,
start rev `d3f6dbc`. Worktree at review: `ahead=0 behind=0 dirty=1` (the
tolerated `ariadne-extension-0.1.0.vsix` only; untouched, recorded).

## Verdict: APPROVED - zero blockers, three minor/record findings

## 1. Fresh workspace gates (reviewer's own runs at `b201aa6`, verbatim)

- Scoped: `python tools/testrun.py --distill -p ariadne-ui` ->
  `371 passed / 0 failed / 0 ignored / 3 suites`, exit 0 (capture
  `target/testrun-capture.txt`, 408 lines) - the erratum's mechanical
  expectation exactly (baseline 369 + the 2 locks).
- Full census: `python tools/testrun.py --distill` ->
  `1837 passed / 0 failed / 13 ignored / 119 suites`, exit 0 (capture, 3194
  lines) - anchor `1835/0/13/119` +2 (the two locks), nothing else moved.
- Tools: `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 296 tests in 26.568s` + `OK` (expected Ran 296, matches).
- `python tools/distill.py -- python tools/ui_smoke.py` -> `ui_smoke: PASS`
  / `boot+route checks clean (cold fixture, attempt 1, 1.1s)`, exit 0 -
  served-asset byte change verified through boot/routes/no-store.
- `python tools/distill.py -- python tools/bench_compare.py` -> exit 0,
  `bench compare: 3 run(s), no regressions past 1.25x baseline` (capture 17
  lines; NOT deemed exempt - shape section 6 mandates fresh recording per
  hand; zero-src proof also recorded: `git diff --name-only "d3f6dbc..HEAD"
  -- crates/cli/tests crates/lang-cpp tests/integration Cargo.lock
  crates/ui/assets/styles.css crates/ui/src crates/ui/tests/ui_test/main.rs`
  -> empty).
- `python tools/hygiene_check.py` -> EXIT=0 (R1/R2 silent; all printed rows
  warn-only R3, zero rows in the three touched files).
- `python tools/encoding_guard.py --all` -> silent, exit 0.
- `python tools/theme_token_check.py --check-added --base 701d7d7` ->
  silent, exit 0 (zero added hex).
- `python tools/testrun.py --baseline` -> tool refuses: `usage error:
  missing value for --baseline` (M23.5 F9; milestone-governing substitute is
  the `--distill` census above per shape section 9 - identical count
  semantics, no weaker evidence substituted). Finding 2.

## 2. Per-commit review (oldest-first)

- `701d7d7` (duty-1): exactly the two CW-2 reviewer artifact paths, verdict
  +342/-0, execution-results +73/-3 (415 insertions / 3 deletions total),
  message verbatim from the CW-2 verdict's duty-1 block. `git diff 701d7d7 --
  <both paths>` empty: the committed blobs equal the reviewer artifacts with
  zero drift to HEAD. Correct.
- `90beaef` (erratum docs): single handoff path, +88/-0 - PURE insertion
  (top pointer +5 at :8-11, DATED AMENDMENT footer +83). No pre-existing
  contract text altered. The erratum's own scope claim ("supersedes ONLY the
  build-item-1 block text; the lock spec is UNCHANGED") is TRUE as landed:
  the pin and all three absence strings in the footer are byte-identical to
  the original mandates at lines 75-93.
- `83f0b7c` (landing): exactly journal + the three code files, message
  verbatim from gate 9. Content adjudicated in sections 3-4.
- `b201aa6` (journal rider): journal only, +63/-4; established
  journal-rider precedent (shape section 5 "plus the journal-rider
  precedent").

## 3. Commit-vs-touch-set

`git diff --name-only d3f6dbc..HEAD` -> exactly 7 paths: the three CW-3 code
files + journal (all in the hard touch-set), `cw3-handoff.md` (authorized by
the erratum's own landing procedure step 1), `cw2-verdict.md` +
`execution-results.md` (authorized duty-1 from the CW-2 verdict). `Cargo.lock`
absent; `main.rs`, `styles.css`, `index.html`, `crates/ui/src/**`,
`crates/cli/**`, `crates/lang-cpp/**`, `tests/integration/**` in no diff
(reviewer-run empty proof above). **Touch-set verdict: PASS - zero foreign
paths, zero undeclared files.**

## 4. Plan-literal + copy byte-identity (headline risks)

- `hotspots.js`: diff vs pre-hand is exactly +2/-2 at :521-522; :519-520 are
  unchanged hunk-context (byte-identical); landed block byte-for-byte equal
  to the AMENDED four-line block (erratum :235-238); full claim contiguous at
  :522 (read + green positive pin); file 691 lines; zero code change;
  `renderExtractionApplyHint` body (P9/P10-locked) untouched.
- `extraction-wizard.js`: exactly +1/-1 at :386; new note line byte-equal to
  the handoff item-2 mandate; `COPY_VS_MOVE_SENTENCE` declaration at :219 in
  no hunk (X5 - the `copyMoveLine` reference appears below as unchanged
  hunk-context only; constant byte-unchanged).
- `extraction_wizard_render.rs`: pure append +39/-0 (single hunk
  @@ -849,3 +849,42), file 851 -> 890 (< 1000); both test names and every
  assertion message byte-equal to the mandates; all pre-existing bytes (incl.
  the `<= 2` fetch bound at :349-352, ASCII locks, P9/P10 `function_body`
  locks) byte-identical and green UNEDITED in my fresh run.

## 5. Lock strength (the no-rot intent, judged on merits)

- Positive pin = the erratum's verbatim full-claim string; verified against
  the raw blob: passes at :522, and rejects any head-negation ("the UI
  NEVER applies...") or paraphrase - strictly stronger than the rejected
  option (a) tail pin, exactly as the erratum argues. Adjudication of option
  (b): **ACCEPTED on merits.**
- Absence locks pin the OLD FULL clauses, not shared prefixes:
  `"never runs anything"` is absent from both assets (grep: zero asset
  occurrences at HEAD); `"every consumer.</div>"` is contained in the old
  clause but NOT in the new claim (new copy continues "consumer the dry run
  lists") - the substring trap is avoided and the absence string is exactly
  the mandated one, handoff line 88 and erratum line 255.
- Wizard positive `"the apply rewrites every consumer the dry run lists"`
  green. The claim TEXT is byte-identical to the shipped A-5 help copy at
  `index.html:553`/`:556` (reviewer grep) - the erratum's copy-honesty claim
  holds.

## 6. The blocked-then-resumed phase and the deviation table

Chain verified end-to-end: unsatisfiable mandate (build-item-1 wrap vs
contiguous pin) -> implementer STOP with mechanical witness (journal Blocker
+ gate-3 line `370 passed / 1 failed / 0 ignored / 2 suites`, panic
:874:5) -> planner adjudication option (b), planner-attributed (F14 class) ->
docs-first landing (`90beaef` before `83f0b7c`) -> resumed Green matches the
erratum's expectation exactly. This is the clarity-track pattern executed by
the book - **not drift.**

- Deviation 1 (the stop under the Blocker): **ACCEPTED** - ratified by the
  erratum ("The implementer's stop was correct"). Worth the ledger's
  deviations summary as the discipline precedent: refuse to weaken an
  assertion, drift copy, or touch out-of-set; stop with a mechanical witness
  and name both one-line candidates without applying either.
- Deviations 2 & 7 (gates 4/6/7 producer-unrun): **ACCEPTED** - dispatch
  assigns workspace-wide gates to the reviewer hop; I ran all three fresh
  (section 1).
- Deviation 3 (refactor not reached): **ACCEPTED** - handoff itself: "none
  expected".
- Deviation 4 (boot `git log -1` refusal): **RECORD, not a deviation** -
  boilerplate catch-all reproduced independently this hop (finding 1).
- Deviation 5 (dispatch "vsix only" vs duty-1 artifacts on disk):
  **ACCEPTED record** - the CW-2 verdict's duty-1 block is the design; the
  boot state was read correctly.
- Deviation 6 (erratum option (b) applied byte-exact, no self-amendment):
  **ACCEPTED** - section 5 merits adjudication; journal's "Copy edits applied"
  item 1 and the ledger:325 discharge line cite the AMENDED block as their
  byte-exact source, per the erratum step 2; ledger:326 line stands unchanged.
- Deviation 8 (two commit cycles + rider instead of one): **ACCEPTED** - the
  extra docs cycle is mandated by the erratum's landing procedure, and the
  rider is house precedent; both declared per the 2026-10-06 M28 itemization
  law, and the journal's final itemization (8 blocked-phase scoped +
  duty-1 pair + 11 resume-phase routed) is complete: CW-3's own scoped gate
  class stayed inside the declared 8 + commit-cycle budget; the excess is
  exactly the erratum's docs cycle, the RGB re-run mandate, and post-landing
  proofs. No gate skipped, none retried.

Report-evidence (distillation contract): journal is digest-shaped throughout -
canonical lines, exit codes, capture paths, verbatim panics, zero raw
dumps, routed entry points only. PASS.

## 7. Findings (numbered; no blockers)

1. (minor, config, RECURRENCE) The direct `python tools/ui_smoke.py` form is
   still refused at this hop, verbatim boilerplate
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`;
   several compound/echo forms likewise (the sanctioned distilling entry
   points all run clean). The allowlist row requested by the CW-1 verdict
   (carried by CW-2 finding 3, M23.5 F10 class) still is not landed. Owed to
   the CLOSED row: land the ledger row at the next ledger touch / operator
   hop, or declare the distill form the canonical reviewer form in
   AGENTS.md.
2. (minor, record-only) `python tools/testrun.py --baseline` refuses
   `usage error: missing value for --baseline` (M23.5 F9 defect row); the
   milestone's documented `--distill` substitute governs and was run fresh -
   no weaker evidence used. Closeout should relight the F9 fix-or-baseline-
   file row at milestone close.
3. (record-only, testrun observation) Under failing runs the canonical line
   reports `2 suites` where the green run reports `3 suites` for the same
   371-test body; pass/failed counts reconcile exactly (369+2, 370+1, 371+0)
   across the journal's gate 2/3, the erratum witness, and my green re-run,
   so no evidence gap exists - the suite-count semantics under failure is
   testrun's own backlog, not a CW-3 defect.

## 8. Closeout-facts (for the planner closeout hop)

- **Four INGRESSED deferred rows - final dispositions:**
  - ledger:321 (D6 latent vacuous-rewrite): DISCHARGED by CW-1 (landing
    `c916a26`, verdict APPROVED; hook + fidelity tests + m19/m20 riders).
  - ledger:322 (expected-modified oracle), 323 (`plan_id` half of the A9
    carve-out; the `new_files` lossy half stays frozen-unsanitized by design),
    324 (cosmetic, discharged on zero-diff re-verification), 328 NO-DELETIONS
    half: DISCHARGED by CW-2 (landing `fddb200`, verdict APPROVED).
  - ledger:325 (stale apply-hint relay), 326 (S4 truncation-note relay):
    DISCHARGED by CW-3 landing `83f0b7c` - the journal's closeout-discharge
    lines are ready verbatim; ledger:325 cites the AMENDED four-line block as
    its byte-exact source (erratum step 2), ledger:326 stands unchanged.
  - ledger:327 (wizard-era fetch-site invariant): **PRESERVED, NOT
    discharged** - bound `<= 2` at :349-352 byte-identical, green UNEDITED in
    every phase; the row's "every future touch" trigger stands.
  - ledger:328 GENERATION-SWAP half: **stays OPEN** - "next
    `extraction_apply.rs` test touch".
- **Milestone census progression:** `1830/0/13/119` @ `765df7b` (pre-milestone
  anchor) -> `1834/0/13/119` (CW-1, +4) -> `1835/0/13/119` (CW-2, +1) ->
  `1837/0/13/119` (CW-3, +2, reviewer fresh-measured @ `b201aa6`). Failed /
  ignored / suites held at 0/13/119 across all four points; every delta fully
  attributed; zero foreign residue.
- **Owed to the CLOSED row (record these, do not drop):**
  - Mid-hand erratum event (this hand): build-item-1 unsatisfiable vs its own
    pin; implementer's stop ratified; option (b) rewrap adjudicated
    planner-side (F14 class, planner side), landed docs-first `90beaef`.
    Suggested ledger phrasing: clarity-track precedent + deviation-1
    discipline precedent for the deviations summary.
  - Allowlist-gap recurrence (finding 1): CW-1 finding 1 / CW-2 finding 3 /
    this hop - the direct `ui_smoke` form is still un-allowlisted; the ledger
    row has still not landed.
  - F14-class kickoff ledger-pointer offset (CW-2 finding 4): kickoff
    section 2 pointers run ~one row low; closeout must cite the actual ledger
    lines 321-328, never the "approximately" ones.
  - 999->295 planner relay (CW-2 finding 2): `extraction_apply.rs` is 295
    lines at every rev of the milestone ("999/1000" ended with the UTA split;
    `tests.rs` at 903 post-CW-2) - correct the fact for the archive.
  - M23.5 F9 (finding 2): the `--baseline` FILE form is still broken; the
    `--distill` substitute was the census of record for all three hands.
- **Milestone final state of record:** HEAD `b201aa6` (`main`, ahead 0, only
  the tolerated `.vsix` dirty); census `1837 passed / 0 failed / 13 ignored /
  119 suites`; tools `Ran 296 tests in 26.568s` + `OK`; bench: fresh
  recording this hop, `bench compare: 3 run(s), no regressions past 1.25x
  baseline`; `ui_smoke: PASS`. Milestone scope complete: all three hands
  landed and APPROVED; closeout may proceed.
