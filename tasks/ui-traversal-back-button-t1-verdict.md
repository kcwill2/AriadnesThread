# Verdict - ui-traversal-back-button T1: traversal panel Back control

Reviewer hop, fresh session, 2026-10-09. Adjudicated via `review-gate`.
Track: TRACK_BASE `20e3fb832263e0459ec05510c142dfc9129d74fc` ("fixed
traversal view", operator WIP landing) .. HEAD `5b4da58`.
Commits under review: `9c0cd0c` (feat), `5b4da58` (docs journal rider).

## VERDICT: APPROVED

No blockers. All numbered findings below are minor dispositions requiring an
operator/orchestrator ledger row, none requiring rework by this track.

## Gate results (reviewer-fresh; canonical lines verbatim)

- Scoped ui suite `python tools/testrun.py --distill -- -p ariadne-ui`:
  `218 passed / 2 failed / 0 ignored / 2 suites` (exit 1 — both failures
  pre-existing by name identity with the journal's BASELINE FAILING SET;
  capture `target/testrun-capture.txt`). The three `traversal_back` tests
  green in the capture (lines 212/214/216: `test ... ok`).
  Passed = 215 baseline + 3, per contract. Note: `--baseline` FILE form not
  used per the handoff's standing 5-encounter errata row (live-tree
  reconciliation via `--distill`); this supersedes the default suggestion in
  the workspace-wide-gate checklist.
- Workspace census `python tools/testrun.py --distill -- --workspace`:
  `1250 passed / 2 failed / 12 ignored / 70 suites` (exit 1, same two
  pre-existing reds). Reconciled from live tree: this track's whole test
  delta is the 3 new tests in the one new module, so TRACK_BASE live baseline
  = 1247 passed / 2 failed; 1250 = baseline + 3 ✓.
- Tool suites `python -m unittest discover -s tools -p "*_tests.py"`:
  `Ran 219 tests in 18.966s` / `OK` (exit 0).
- `python tools/distill.py -- python tools/ui_smoke.py`:
  `ui_smoke: PASS`, `### generic PASS (exit 0)`.
- `python tools/distill.py -- python tools/bench_compare.py`:
  `bench compare: 3 run(s), no regressions past 1.25x baseline`,
  `### generic PASS (exit 0)`.
- `python tools/encoding_guard.py --all`: exit 0.
- `python tools/hygiene_check.py`: exit 1 — 3 R1/R2 findings, all on
  zero-diff carriers; see Finding 2. R3 warn-only (non-blocking).
- `python tools/theme_token_check.py --check-added --base 20e3fb8`: exit 1 —
  three violation lines; see Finding 3.

## Commit-vs-touch-set (G6/I3/I4/I7) — HOLDS

- `git diff --name-only 20e3fb8..HEAD` = exactly the 7 touch-set paths
  (index.html, traversal.js, hotspots.js, styles.css, ui_test/main.rs,
  traversal_back.rs NEW, journal). Nothing outside the touch-set.
- `git diff --stat 20e3fb8..HEAD -- crates/ui/assets/graph.js
  crates/ui/tests/ui_test/static_html_sections.rs crates/ui/src`: EMPTY
  (graph.js budget red carrier + frozen lock + all Rust source byte-
  identical; degree code path in graph.js zero-diff, I4 ✓).
- I3 list (api.js, main.js, preview.js, provenance.js, file-view.js,
  theme.js, themes.css, tools/**, tests/**, docs/**): diff EMPTY.
- Manifests/lockfiles: diff EMPTY.
- I7: `git diff --name-only` filtered to `crates/ui/tests` == exactly
  `main.rs` + `traversal_back.rs`. `mod traversal_back;` is the last line of
  the roster (alphabetically correct site).
- Rider `5b4da58` touches the journal only (22+/5-).

## Per-commit review (oldest-first)

- `9c0cd0c feat(ui): traversal Back control returns to origin tab`: C1/C2/
  C3/C4 applied verbatim per contract (see plan-literal section). New lock
  module implements exactly the three required tests with every mandated
  assertion and message; no weakened assertion; doc-comment style mirrors
  `hotspot_direction.rs` as specified. `traversal.js` missing trailing
  newline at EOF is pre-existing form (present on the before side too).
- `5b4da58 docs(...)`: journal rider only; corrects gate itemization after
  Gate-7 count finalization; no code.

## Plan-literal checks — PASS

- C1: `index.html:388` byte-exact:
  `<button class="traversal-back" onclick="backFromTraversal()" title="Back to the view you came from">&#8592; Back</button>`
  — numeric entity, pure-ASCII source. The three unrelated `title="Close"`
  buttons (2x modal-close :132/:145, hotspot-close :234) preserved (I1
  parenthetical).
- C2: `traversal.js:398-416` matches the contract block including doc
  comments, guard `currentTab !== 'traversal'`, and the two-line export
  swap; `closeTraversal` and `traversal-close` appear nowhere in
  `crates/ui/assets/**` (rename total).
- C3: `index.html:72` `onclick="openTraversalTab()"` (class/id/svg/label
  unchanged); `hotspots.js::openTraversalForRoot` first statement
  `openTraversalTab();`. Exactly one `switchTab(['"`]traversal` call site
  remains workspace-wide in assets — inside `openTraversalTab` itself
  (traversal.js:404), contract-internal. Entry-point survey exhaustive: no
  other traversal entry exists.
- C4: `.traversal-close`/`:hover` renamed in place; only additions
  `gap: 4px; align-items: center;`; no hex in the diff.
- Journal carries BASELINE/Red/Green digest headers, canonical lines,
  capture paths, exit codes — digest-shaped report, zero raw dumps (step 8
  evidence check PASS; all gate commands via distilling entry points).

## Numbered findings (all minor; disposition per finding)

1. **F1 — TWO pre-existing reds at TRACK_BASE (operator disposition row
   required).** `static_html_sections::test_extraction_impact_preview_overlay`
   (graph.js 1051 > 998, "(cap 990)") AND
   `static_html_sections::test_inspector_panel_is_wider_and_collapsible`
   (assert at `static_html_sections.rs:411`-413: `INDEX_HTML.contains
   ("onclick=\"toggleInspector()\"")`). Both independently confirmed
   not-track-attributable: failing-set name identity baseline↔HEAD (fresh
   run), both carriers of the graph red proven zero-diff, and the inspector
   red's predicate is invariant under this track's 2-line index.html diff
   (neither side contains `toggleInspector`). Independent attribution of the
   inspector red: the assertion is old (introduced `b78ec19`, last file edit
   `c17d182` = green ledger anchor, 0 failed); the ONLY commit touching
   `index.html` in `c17d182..20e3fb8` is the operator's WIP `20e3fb8` —
   implementer's attribution CLAIM CONFIRMED. `toggleInspector` now exists
   only as the orphaned handler at `main.js:388`.
   DISPOSITION: accepted as pre-existing; needs a separate disposition row
   in `tasks/milestone_ledger.md` (operator: restore the collapse control in
   the WIP's own follow-up, or amend the frozen lock deliberately).
2. **F2 — `hygiene_check.py` exit 1 at TRACK_BASE already.** Three R1/R2
   findings: `graph.js R2 1051>1000`, `server.rs R2 1120>1000`,
   `server.rs:764 R1 fn api_graph_traversal 161>100`. All carriers proven
   byte-identical to TRACK_BASE (content-identity ⇒ findings pre-date the
   track; the handoff's "stays exit 0" precondition was false at baseline —
   planner error same root cause as F1, not an implementer fault). Zero
   findings in touch-set files confirmed in my fresh run (traversal.js 416
   lines; `.css` not in R2_SCANNED_SUFFIXES; traversal_back.rs clean;
   index.html far below 1000). Pre-commit attribution of server.rs size to
   `20e3fb8` could NOT be independently run (see gate-execution ledger);
   server.rs is on the kickoff WIP dirty list, making WIP attribution highly
   likely. DISPOSITION: accepted; the graph.js R2 row maps to the existing
   deferred heat/legend-extraction ledger row; server.rs R1/R2 needs a NEW
   disposition row.
3. **F3 — theme_token_check exit 1; contract-internal collision ADJUDICATED
   in favor of keeping the contract markup.** Mechanism independently
   verified: `tools/theme_token_check.py:42-47` `HEX_LITERAL` matches `#`
   plus hex digits with no exemption for `&#...;` numeric character
   references, and `&#8592;`'s digits are all valid hex → phantom `#8592`.
   The tool's own committed baseline (`tools/theme_token_baseline.json`)
   records four such entity misreads as sanctioned "locations" — proving the
   misread predates the track. The only track-attributable violation is
   exactly the C1-mandated entity; C1 "after (exact)" and Gate-6 silence are
   unsatisfiable together, and the two forbidden escapes (alter contract
   markup / update frozen baseline) were correctly refused. Substantive I6
   preserved and locked: `test_back_css_token_only` green (rule body
   `var(--`, no `#`), and styles.css frozen count clean (0>0 — no styles.css
   violation in my fresh run). REQUIRED ACTION (not for this track): ledger
   row for a tool fix (skip `&#\d+;` numeric refs in `HEX_LITERAL` /
   scan). SEPARATE sub-finding: the frozen-COUNT violations
   (`index.html 6>4`, `traversal.js 1>0`) are WIP-introduced emoji-entity
   inflation (base 5>4 before this track added any line) — operator
   disposition row (deliberate `--update-baseline` in a dedicated commit, or
   entity removal).
4. **F4 — gate-count overshoot 13 > ~10 (D-row 2026-10-09): ACCEPTED.**
   Declared as the handoff's journal law requires; itemization internally
   consistent (12 gate invocations + 1 rider commit); the 3 extra
   invocations exist solely to surface exit codes in digest headers — a
   distillation-contract artifact, not scope creep. Supporting probes are
   correctly classified as evidence captures.
5. **F5 — `encoding_guard --staged` seam genuinely absent: ACCEPTED.**
   `tools/commit_scoped.py:7-8` confirms the atomic `git add` +
   `git commit --only` mechanism leaves no pre-commit staged window.
   Substitute evidence is evidence-equivalent at the byte level and
   non-vacuous; I re-ran `--all` fresh: exit 0. Encoding hygiene of the
   track independently clean.
6. **F6 — G7 manual browser smoke NOT executed; contract-compliant form,
   runtime behavior UNVERIFIED pending operator.** The journal correctly
   says "manual smoke pending operator" instead of checking off the matrix,
   as the handoff's G7 fallback demands. Explicitly unverified at runtime:
   (a) enter via tab button from symbol tab → Back lands on symbol;
   (b) from call tab → Back lands on call; (c) enter via hotspot card →
   Back returns to include with hotspot panel restored; (d) degree behavior
   unchanged from baseline (still the known-open
   `ui.traversal_view_degree_setting_bug`, not worsened — I4 diff-zero is
   static evidence only). `ui_smoke` PASS covers boot/routes/no-store only,
   not the Back interaction. Static string locks pin the wiring; the
   functions are trivial (`switchTab` with a captured variable).
   DISPOSITION: operator must run the 4-row matrix before the track is
   considered fully consumed; record in the ledger row.
7. **Cosmetic (no action):** journal lines 61/63 duplicate the heading
   `### Green - C1-C4 asset edits`. Audit trail — noted, not fixed.

## Out-of-scope observations (report-only)

- Workspace census vs ledger anchor drift: anchor `1263 passed / 0 failed /
  13 ignored / 91 suites` @ `c17d182` vs live `1250/2/12/70`. The suite
  delta (−21) and passed delta are WIP-era drift, NOT this track's (track
  adds 0 suites); orchestrator may want an anchor-refresh note per the
  standing errata.
- Degree bug stays OPEN, untouched as mandated; I encountered nothing during
  review to change its status or severity.

## Gate-execution ledger (reviewer hop; refusals verbatim where relevant)

Ran fresh: checkpoint_guard status; scoped testrun; workspace census;
unittest discover; ui_smoke; bench_compare; hygiene_check (output + exit);
theme_token_check --check-added --base 20e3fb8 (output + exit);
encoding_guard --all; per-commit git show both commits; all zero-diff
proofs; pickaxe + log-range attribution probes; all plan-literal reads;
entry-point and dead-contract greps; `title="Close"` count.
REFUSED by catch-all rule
(`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`),
each retried in an alternative permitted form unless noted: unittest with
pipe (plain form ran OK); zero-diff batch with echo strings (split: ran OK);
inline multi-line `python -c` blob probe; Write to temp dir (outside
`tasks/**` grant — boundary respected); `git grep` in every form tried
(substituted `git log -S` pickaxe + path-scoped `git log` range —
equivalent outcome); `git worktree add` for an isolated hygiene checkout
(write-git, correctly denied; substituted content-identity argument);
`git show | Measure-Object -Line` for server.rs/graph.js pre-WIP line counts
(NOT run — sole evidence gap, noted under Finding 2); raw `bench_compare`
(distill form ran OK).
Not run for the track: `--baseline` FILE form (superseded by handoff errata
row; live-tree reconciliation performed instead).
