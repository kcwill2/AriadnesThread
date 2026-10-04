# Verdict - ui-extraction-card-story T1: APPROVED

Commit under review: `ef9d170` (parent `4900e04`, the journal's recorded start rev).
Adjudicated via `review-gate`; reviewer-hop workspace gates run fresh in this
session. Full evidence lines: `tasks/ui-extraction-card-story-execution-results.md`.

## Gate results (reviewer hop, fresh)

| # | Gate | Result |
|---|---|---|
| 1 | `testrun.py --distill -p ariadne-ui` | `197 passed / 0 failed / 0 ignored / 3 suites` exit 0 -- reproduces implementer Green verbatim |
| 2 | `testrun.py --distill` (census, `--baseline`-file errata form) | `1181 passed / 0 failed / 4 ignored / 85 suites` exit 0; reconciles 1168 @ `1bc50b8` + 4 dark-grey `@41ec354` + 9 new; suites held at 85 |
| 3 | `unittest discover -s tools` | `Ran 188 tests in 20.991s` + `OK` |
| 4 | `ui_smoke.py` | `boot+route checks clean (cold fixture, attempt 1, 1.0s)` + `ui_smoke: PASS` |
| 5 | `bench_compare.py` | Sanctioned assets-only exemption applied (see finding 3 for the blocked attempt) |
| 6 | `theme_token_check.py --check-added --base 4900e04` | silent, exit 0; `themes.css` absent from diff |
| 7 | `encoding_guard.py --all` | silent, exit 0 |
| 8 | `hygiene_check.py` | exit 0; R3 warn-only legacy rows, none in touch-set files |
| 9 | per-commit `git show` + touch-set + census | single commit; all paths inside touch-set + riders; locks held (below) |

## Commit-vs-touch-set

`git diff --name-only 4900e04..HEAD` = `hotspots.js`, `index.html`, `styles.css`,
`extraction_card_story.rs` (new), `main.rs` (+1 mod line, alphabetical),
plus the sanctioned riders `tasks/milestone_ledger.md`, the handoff, the journal.
Nothing else. `git diff --stat 4900e04..HEAD -- crates/pipeline crates/cli
crates/cluster crates/graph crates/ui/src tools tests/fixtures` -> EMPTY:
DTO/producers/tools/fixtures byte-identical; `api.js`, `preview.js`, `graph.js`,
`main.js`, `themes.css`, `theme.js` absent from the diff (byte-identical).

## Plan-literal audit (spot-verified, not averaged)

- C4 keep-literals present verbatim in new `hotspots.js`: `function
  renderExtractionProposalCard(` (:309), `collateral_load_before`/`_after`
  (:311-312), `consumer_tu_count` (:341), `extraction-row` (:339-341),
  `extracted symbols need` (:340), `just-more` (:315), `selectHotspotRow('`
  (:329), `graphData.extraction_proposals || []` (:355), `proposals.length ===
  0` (:356), `Preview drop from selected file` (:326). All existing card/
  section/help/payload tests green UNEDITED (edited-file census proves zero test
  edits; scoped suite green).
- S6: no `.sort(` over proposals (the three `.sort(` hits at :39/:50/:223 are the
  pre-existing ranker, outside the diff). S8: `.just-preview-btn` only at :326,
  markup byte-faithful. S4: `.join(', ')` eliminated file-wide; `.join('')`
  assembly only.
- S3 zero-guard: `!(before > 0) || !(after < before)` early-return;
  `Math.round(drop * 100 / before)`; no raw-string class injection (:300
  `[^a-z]` strip).
- C6 copy honest against producers: `planner.rs:659` templated problem,
  `planner.rs:664` always `Conservative` ("every plan the planner emits today
  is Conservative" -- true, no overclaim), `ui.rs:534` `consumer_tu_count =
  validation_required_tus.len()`. Locked phrases `Proposed extractions section`
  (index.html:511) and `Preview after extraction` (:499) intact; exactly 3 new
  `<li>`, each well under 400 stripped chars, pure ASCII (`&rarr;` entity).
- CSS: growth entirely inside the extraction block (~+75 lines, budget ~90),
  new classes only `.extraction-*`, `var(--token)` only, `.extraction-list`
  kept (:989), `.extraction-names` dropped from BOTH CSS and JS (no retained
  dual representation), `.just-*` rules and `.extraction-card` spine untouched.
- Hygiene targets: card builder :309-344 (36 lines <= 70 target), helpers
  4-6 lines, `hotspots.js` 395 of 1000.
- Report evidence (distillation contract): journal is digest-shaped -- canonical
  lines, exit codes, capture path `target/testrun-capture.txt`, zero raw dumps,
  commands via distilling entry points. Red `189 passed / 8 failed / 0 ignored /
  2 suites` exit 1 is journal-attested (Red is a historical state, unreproducible
  post-Green); totals reconcile (188 pre-existing + 9 new = 197 at Red and Green).

## Deviation dispositions

1. `--text` delta emphasis dropped (font-weight: 700, inherited color) --
   ACCEPTED. Independently verified: `themes.css` declares `--text-muted`/
   `--text-sub`/`--text-main` but NO `--text`;
   `theme_js_zero_hex.rs::test_every_var_reference_in_static_surfaces_is_
   declared_in_default_theme` fails on undeclared `var(--x)` in styles.css, and
   themes.css is frozen (S9). The shape C5 wording is unsatisfiable against the
   frozen contract; the stricter declared-token oracle wins.
2. Ledger riding with BOTH planner PLANNED rows -- ACCEPTED as sanctioned
   (orchestrator instruction; ledger is one file riding as one unit).
   Follow-up disposition: the `ui-dark-grey-theme` PLANNED->status flip remains
   THAT milestone's reviewer-hop duty; not discharged by this commit.
3. Start rev `4900e04` not planning-time `d76a559` -- ACCEPTED. dark-grey T1
   landed first (`41ec354` + journal `4900e04`); "second-to-land uses first
   landing head" satisfied; index.html edits content-anchored (verified: the
   3 `<li>` sit in the Hotspots help list).
4. Suite count 2->3 (doc-test suite appears only once fail-fast no longer
   short-circuits it) -- ACCEPTED as cosmetic; totals reconcile exactly.

## Findings

1. (minor, config) Reviewer allowlist refuses `python tools/ui_smoke.py` and
   `python tools/bench_compare.py` directly -- refusal verbatim:
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
   `ui_smoke` was still run via `python tools/distill.py -- python tools/ui_smoke.py`
   (PASS). Add both scripts to the reviewer allowlist or sanction the distill
   route in the handoff template. Not the commit's fault.
2. (minor, environment) `target\debug\tool.exe` is held by a live process
   (`error: failed to remove file ... Access is denied. (os error 5)`); the
   first two unscoped census runs died as testrun DRIFT (exit 2) purely from
   this build failure. The clean census was obtained with
   `CARGO_TARGET_DIR=<temp>`; the census numbers are code-equivalent. Someone
   must close the stale UI/tool process before the next normal-target build,
   or `--baseline`/bench runs keep dying. Orchestrator follow-up, not a code
   defect.
3. (disposition) `bench_compare`: attempted directly (refused, finding 1) and
   via distill (blocked by finding 2's lock mid-run: `ERROR cargo bench run 2
   of 3 failed (exit 101)`). Exemption sentence, per handoff gate 5 / ui-progress-parity
   precedent: bench_compare not run -- assets-only change, pipeline byte-identical
   (proven: empty diff over `crates/pipeline` and all producer crates).

APPROVED. No blockers. Deferred-record routing for findings 1-2: orchestrator's
`tasks/milestone_ledger.md` row or deferred-findings hop.
