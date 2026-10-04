# T1 Journal - ui-extraction-card-story (extraction card five-beat story)

Handoff: `tasks/ui-extraction-card-story-t1-handoff.md`.
Milestone key `ui-extraction-card-story`; no M-number guessed (ledger row owns it).

## Bootstrap (2026-10-04)

- Start rev (HEAD before my first commit): `4900e04`
  (`docs(ui): record dark-grey T1 landing (gates 4-7, hash 41ec354)`).
  All `--base` gate arguments use it.
- Sibling race observed and resolved at bootstrap: `ui-dark-grey-theme` T1
  landed DURING this session's bootstrap (`41ec354` + journal follow-up
  `4900e04`), its staged files going clean under a concurrent commit. The
  handoff sequencing precondition is therefore MET: dark-grey landed FIRST,
  this milestone starts from its landing head, exactly as the ledger row
  demands ("second-to-land uses first landing commit as --base rev").
- Consequence for line refs: the handoff's index.html line anchors (506-511)
  drifted with the dark-grey additions; the Hotspots help `<ul>` is located by
  CONTENT anchor (the `Hotspots` `<h5>` list), as the ledger row instructs.
- Deliberately riding the landing commit (orchestrator instruction + planner
  ledger row "handoff + ledger ride with the T1 landing commit"):
  `tasks/ui-extraction-card-story-t1-handoff.md`, `tasks/milestone_ledger.md`.
  The ledger file's pending diff adds BOTH planner rows
  (`ui-dark-grey-theme` PLANNED + this milestone PLANNED); the file rides as
  one unit, so the dark-grey PLANNED row lands with my commit. Its status
  flip stays the dark-grey reviewer hop's own ledger duty (recorded as
  deviation D2 in the final report).
- Left untouched (report-only): `.kilo/plans/**` untracked plans (incl. this
  milestone's plan file and the still-untracked dark-grey shape),
  `tasks/ui-extraction-card-story-kickoff.md` + `-shape.md` (untracked;
  precedent: dark-grey shape still untracked at `4900e04`; the orchestrator
  tracks docs in a separate docs commit like `e3d8700`), and the unrelated
  unstaged `.kilo/plans/1790697359283-systems-layer-engine-room.md` edits.
- C6 producer facts verified against tree: `planner.rs:659` templated
  `"{N} symbols of collateral load"`, `planner.rs:664` risk always
  `Conservative`, `cli/src/commands/ui.rs:534`
  `consumer_tu_count = validation_required_tus.len()`.

## Green checkpoint (2026-10-04)

- Red digest: `189 passed / 8 failed / 0 ignored / 2 suites`, exit 1; the
  failing list was exactly the predicted eight (beats 1-6, 8, 9);
  `test_order_and_preview_locks` green from day one, as the handoff expects.
- Green digest: `197 passed / 0 failed / 0 ignored / 3 suites`, exit 0
  (capture `target/testrun-capture.txt`). The 3rd suite is the 0-test
  doc-test suite; totals reconcile (188 pre-existing + 9 new).
- Green-loop corrections (inside Green, not refactor): (a) the chip wrapper
  span moved into the return template because a hoisted `const` placed the
  `extraction-chips` beat before `extraction-cost` (story-order lock);
  (b) `var(--text)` dropped from `.extraction-problem`/`.extraction-delta` -
  `--text` is NOT a declared token; `theme_js_zero_hex` caught it. Delta
  emphasis is font-weight, color inherited. DEVIATION D1: shape C5 names
  `--text` for delta emphasis, but the theme contract's declared-token test
  is the stricter oracle and themes.css is untouchable (S9), so the shape
  wording loses.
- Gates so far: `theme_token_check.py --check-added --base 4900e04` silent,
  exit 0 (zero added hex); `hygiene_check.py` exit 0 (R3 legacy warns only,
  none in touch-set files). Refactor re-run gate not triggered: no
  behavior beyond naming/comments changed after Green.

