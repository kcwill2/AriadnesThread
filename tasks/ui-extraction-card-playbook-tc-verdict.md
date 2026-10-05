# Verdict - ui-extraction-card-playbook T-C: APPROVED

Commit under review: `816d09d` (full `816d09d7c9798a3985002fbd7190971cdedb9fc5`),
parent `6a48a85` (= journal start rev). Final hand of the HARD CHAIN
T-A -> T-B -> T-C. With this verdict the milestone is fully gated end-to-end
(see completeness statement at the end).

## Gate results (fresh, this hop, verbatim)

| # | Gate | Result |
|---|---|---|
| 1 | `testrun.py --distill -p ariadne-ui` | `209 passed / 0 failed / 0 ignored / 3 suites` exit 0 -- reproduces implementer Green verbatim |
| 2 | capture inspection | all 9 `extraction_card_story::test_* ... ok` green UNEDITED (incl. test 8 survival) + all 9 `extraction_card_playbook::test_* ... ok` |
| 3 | census `testrun.py --distill` | `1213 passed / 0 failed / 4 ignored / 85 suites` exit 0 |
| 4 | `distill.py -- python tools/ui_smoke.py` | `boot+route checks clean (cold fixture, attempt 1, 0.7s)` / `ui_smoke: PASS` exit 0 |
| 5 | `theme_token_check.py --check-added --base 6a48a85` | silent, exit 0 (zero added hex) |
| 6 | `hygiene_check.py` | R3 warn-only legacy rows; zero in T-C files; exit 0 |
| 7 | `encoding_guard.py --all` | no output, exit 0 |
| 8 | `distill.py -- python tools/bench_compare.py` | recorded EXEMPTION for this hand (see bench adjudication) |

## Census arithmetic

T-B's clean census `1204 passed / 0 failed / 4 ignored / 85 suites` + T-C's 9
new playbook tests = **`1213 passed / 0 failed / 4 ignored / 85 suites`** --
hit exactly. Ignored held at 4; suites held at 85: the new test file is a
`mod` inside the existing `ui_test` binary, not a new test target (the
2-vs-3 suite count between Red and Green is the same canonical-line parser
artifact recorded at the story, brace, and T-B hops; totals reconcile
201+8 = 209).

## Bench adjudication (recorded: exemption, attempts on file)

T-C is an assets+tests hand: `git diff --name-only 6a48a85..816d09d` shows
six paths, ZERO under `crates/*/src` -- every measured phase's code is
byte-identical across this range (machine-proved, re-verified this hop, not
carried over). Live attempts anyway (machine was idle, smoke booted in 0.7s):
attempt 1 exit 1 `WARN save_load_artifacts+cache ... 16386.6 ms vs baseline
12825.1 ms (+28%)`, `WARN ensure_load total: 18235.5 ms vs baseline 14413.0
ms (+27%)`, `WARN OptimizationPlanner::plan() facts+closures: 1937.5 ms vs
baseline 1451.0 ms (+34%)`; protocol retry (max one) exit 1 with four
breaches +25..+39% and medians WORSE than attempt 1 (e.g. `plan()`
2015.8 ms). Medians rose monotonically across today's hops
(`plan()` 1667 -> 1918/1937 -> 2016; `ensure_load` 15648 -> 18235 -> 18631),
now including disk-bound phases embedded assets cannot touch: ongoing
machine-state drift against a stale per-machine baseline, not a T-C
regression. Disposition: the documented assets-only exemption applies to
this hand (ui-progress-parity precedent + the empty Rust-diff proof above);
T-B's retry PASS (`bench compare: 3 run(s), no regressions past 1.25x
baseline` @ `6a48a85`) stands as this milestone's bench truth. OPERATOR:
the machine's baseline is now demonstrably stale (4th consecutive hop with
run-1 noise, worsening) -- a verified-idle `--update-baseline` must happen
before any closeout bench run.

## Disclosure semantics audit (duty 3, verified in the code)

- P4: summary button `aria-expanded="false"` initial; detail template ships
  `class="extraction-details" hidden>` -- collapsed by default.
- P5: real `<button type="button" class="extraction-disclosure">` dispatching
  `toggleExtractionCard(event, this)`; the toggle does
  `event.stopPropagation()` (card click still hits `selectHotspotRow`),
  locates details via `closest('.extraction-card')` + `querySelector`
  (works on both surfaces), flips `details.hidden` and mirrors
  `aria-expanded`; NO `getElementById`, no element ids. Keyboard access is
  native (real button). Reset-to-collapsed on re-render is the documented C5
  contract (cache rebuilds innerHTML).
- Playbook rows: `Files to create (N)` + one `.extraction-file` per feed row
  with `${escapeHtml(t.path)}` (no invented paths -- data is exactly the
  T-B feed), symbols only via `extractionChips` (escaped), `+N more` symbol
  overflow via reused `.just-more`, `more files not listed` iff
  `target_files_truncated`, legacy fallback `<em>no file plan in payload</em>`
  -- never a fabricated path.
- P9 command honesty: exactly `tool apply &lt;repo-root&gt; --plan ${id}` and
  `tool migrate &lt;repo-root&gt; --plan ${id} --preview`; greps prove those
  are the ONLY `tool ` occurrences in hotspots.js; spot-diffed against
  `crates/cli/src/lib.rs` 201-219 (`Migrate { repo positional, --plan,
  --preview }`, `Apply { repo positional, --plan }`, "transactionally") and
  `crates/validate/src/lib.rs:17-21` (`ValidationTier` = exactly
  Structural/GeneratedHeaders/AffectedTUs, rollback fires on Fail) -- the
  three-tiers/rollback help copy is true. Plan id escaped.
- P8: no `fetch(`/`XMLHttpRequest`/`EventSource` (grep-verified absent).
- P10/P11/C6: cache + `syncExtractionProposalSurfaces` outside every diff
  hunk (byte-identical); keep-literals intact (`extracted symbols need`
  :407, `just-preview-btn` only :386, `join(', ')` absent); CSS +64 lines
  contiguous inside the `.extraction-*` block (budget ~70), all colors
  `var(--token)`, zero hex; `.just-*`/risk tiers/`.extraction-list`/spine
  unchanged; help = exactly 3 new `<li>`, ASCII entities (`&mdash;`,
  `&lt;`), <400 chars each, locked phrases untouched.
- Survival rules: none of the five beat substrings appear in the four new
  pre-builder helpers (first plural `.extraction-chips` literal is the
  builder's Extract row); story test 8 green UNEDITED. Story file absent
  from the diff (byte-identical, machine-proved).

## Deviation dispositions

1. D1 (per-file chip wrapper placement): ACCEPTED. The handoff's survival
   rule 2 -- binding, derived from the UNEDITABLE story test 8 --
   prescribes exactly this resolution: the helper emits only singular
   `.extraction-chip` (via the existing escape path) and the plural wrapper
   "is composed by the BUILDER below the `extraction-head` line". Where the
   shape C3 sketch showed a wrapper inside file rows, rule 2's own text
   overrides it; the implementer followed the binding rule literally and
   test 8 confirms the analysis. No story test amended.
2. D2 (staging via `distill.py -- git add -- <six paths>`): ACCEPTED. This
   is the handoff's own sanctioned allowlist-refusal route; `encoding_guard
   --staged` then checked real staged blobs, and the reviewer's independent
   `--all` (exit 0) plus exact set-equality of the committed paths close the
   evidence chain.

## Findings

1. (minor, handoff erratum) The handoff's mod-line instruction ("alphabetical:
   after `mod edge_verdict_story;`, before `mod export_order;`") is itself
   wrong: alphabetically `export_order` precedes `extraction_card_playbook`.
   The implementer complied plan-literally, so `main.rs` now lists
   `extraction_card_playbook` before `export_order` (zero functional effect;
   the real alphabetical site is after `export_order`, before
   `extraction_card_story`). Record in the closeout ledger row as a
   handoff erratum; fix opportunistically at next touch, NOT as a rework.
2. (operational, carried from T-B finding 1, escalated) Bench baseline
   staleness is now worsening across four hops and includes disk-bound
   phases; operator `--update-baseline` on a verified-idle machine is a
   prerequisite for a meaningful closeout bench. No baseline touched by
   this hop.

## Milestone gate completeness + closeout obligations

Chain complete and each hand gated APPROVED: T-A `df1e21e` (DTO, verbatim
key-union lock), T-B `6a48a85` (feed, P1 oracles, bench PASS via retry),
T-C `816d09d` (renderer, disclosure locks, smoke PASS). Every hand ran its
scoped proofs; workspace gates discharged at the first hops that could take
them: census progression 1195 (brace anchor) -> 1204 (T-B) -> 1213 (T-C),
all `0 failed / 4 ignored / 85 suites`; ui_smoke PASS; bench truth = T-B's
PASS, T-C under the recorded exemption.

The CLOSEOUT (planner ledger flip PLANNED -> CLOSED + memory mirror, its own
commit) must record:
- three landing hashes + three APPROVED verdict paths (ta/tb/tc verdicts are
  still UNTRACKED -- track them, kickoff/shape files, and journals' final
  state in the closeout docs commit, precedent `e3d8700`);
- final census line `1213 passed / 0 failed / 4 ignored / 85 suites` @
  `816d09d` (+ progression arithmetic);
- scoped digests per hand (verbatim in the three verdicts);
- ui_smoke PASS lines; bench: T-B PASS (retry) + T-C exemption sentence +
  the operator baseline-refresh prerequisite (finding 2) -- if the closeout
  runs bench again it must be after the baseline refresh on idle hardware;
- zero story-test amendments (P6 honored; `extraction_card_story.rs` never
  touched across all three hands);
- deferred-findings routing: bench baseline row, reviewer allowlist row
  (direct `ui_smoke`/`bench_compare` invocations), handoff erratum (finding
  1), and the styles.css split decision that stays with its existing ledger
  row;
- worktree census clean + ledger 3/2 dirty edit belongs to the closeout
  itself (orchestrator's own pending edit, not part of any handoff).

APPROVED. No blockers. Milestone `ui-extraction-card-playbook` is ready for
closeout.
