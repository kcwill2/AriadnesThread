# T1 Handoff - ui-traversal-back-button: traversal panel Back control (return-to-origin)

Authored 2026-10-09 by the planner role. Unregistered post-M28 track per
kickoff: NO M-number; `tasks/milestone_ledger.md` stays numbering authority.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract; zero verbal briefing needed).
2. `tasks/ui-traversal-back-button-kickoff.md` (intent, scope in/out, gate set G1-G8).
3. `tasks/ui-traversal-back-button-shape.md` (chosen semantics, invariants I1-I7,
   contracts C1-C5, risks R1-R6).
4. Content anchors (verbatim, verified against the working tree 2026-10-09):
   `crates/ui/assets/index.html` tab button (id `tabTraversal`) and traversal
   head button (class `traversal-close`); `crates/ui/assets/traversal.js` tail
   `closeTraversal` block + `window.closeTraversal` export line (file 406 lines);
   `crates/ui/assets/hotspots.js::openTraversalForRoot` first statement;
   `crates/ui/assets/styles.css` `.traversal-close` rules; the `mod` roster in
   `crates/ui/tests/ui_test/main.rs`; asset-string lock style to mirror:
   `crates/ui/tests/ui_test/hotspot_direction.rs` and
   `crates/ui/tests/ui_test/static_html_sections.rs`.
5. Field/literal sources: `crates/ui/src/lib.rs` constants `INDEX_HTML`,
   `TRAVERSAL_JS`, `HOTSPOTS_JS`, `STYLES_CSS` (read-only via `include_str!`).

## Dispatch precondition (blocking; carried verbatim from the kickoff)

The working tree at kickoff time is NOT gate-clean and carries an uncommitted
traversal-view work-in-progress:
`crates/ui/assets/traversal.js` is untracked (new); `api.js`, `graph.js`,
`hotspots.js`, `index.html`, `main.js`, `styles.css`, `dto.rs`, `export.rs`,
`lib.rs`, `server.rs`, `theme_contract.rs` are modified;
`docs/specs/collateral-traversal-view.md` is untracked.
Saved project memory records a failed degree-setting fix attempt in this same
area, and the tree shows working-tree `graph.js` at 1051 lines against the
committed budget lock `GRAPH_JS.lines().count() <= 998`
(`crates/ui/tests/ui_test/static_html_sections.rs:215`), i.e. the scoped ui
suite is expected red in the working tree today.

Consequences, binding on this handoff:
- This track's baseline is the tree AFTER that WIP lands or is otherwise
  dispositioned by the operator/orchestrator (their commit duty; this track
  neither absorbs, reverts, nor rebuilds it). Do NOT dispatch this handoff
  into a dirty tree.
- This handoff must NOT touch `graph.js` and must NOT attempt the budget-lock
  fix. The `<= 998` relocation + stale "(cap 990)" message fix belong to the
  deferred heat/legend-extraction row in the ledger; leave them alone.
- The reviewer must treat that specific red (graph.js line-budget test) as
  PRE-EXISTING, not attributable to this track. The machine-checkable
  distinction is Step 0 below: the pre-implementation failing-test-name set is
  recorded verbatim in the journal, and acceptance is failing-set identity
  before/after PLUS an empty diff on the two files that produce that red
  (`crates/ui/assets/graph.js` and `crates/ui/tests/ui_test/static_html_sections.rs`).
- If at session start the working tree is dirty on any touch-set file, or any
  verbatim before-string below does not match the current file exactly, STOP
  and report; do not silently rebase onto moved content (shape R1).

## Objective and invariants

Replace the traversal panel's `Close` (X) control with a `Back` control that
returns the user to the tab active at the most recent traversal entry
(default `'include'`), routing BOTH traversal entry points through a capturing
entry function. Preserve, falsifiably:

- I1 Back control exists, close contract dead: `INDEX_HTML` contains
  `class="traversal-back"`, `onclick="backFromTraversal()"`, a `title` starting
  `Back to`; `INDEX_HTML` and `TRAVERSAL_JS` contain no `traversal-close` and
  no `closeTraversal`; `STYLES_CSS` has no `.traversal-close` rule. (The other
  three `title="Close"` buttons in `index.html` - two `modal-close`, one
  `hotspot-close` - are unrelated and stay.)
- I2 Return-to-origin wired at every entry (exactly two exist, survey-verified):
  the `#tabTraversal` tab button and `hotspots.js::openTraversalForRoot`.
- I3 Zero collateral diff: `graph.js`, `api.js`, `main.js`, `preview.js`,
  `provenance.js`, `file-view.js`, `theme.js`, `themes.css`, all of
  `crates/ui/src/**`, all of `tools/**` byte-identical to the track baseline.
- I4 Degree code path untouched (protection for the adjacent open bug).
- I5 Asset transport and script roster unchanged (9 classic scripts;
  `/app/traversal.js` `no-store` route; export inline roster). Relay fact, do
  NOT fix here: the byte-equality route roster in `static_html.rs` omits
  `/app/traversal.js` (WIP-side artifact); neither extend nor amend it.
- I6 Theme safety: zero added hex; the renamed rule declares only existing
  `var()` tokens.
- I7 Existing-test freeze: the only test diffs are the new `traversal_back.rs`
  module and one `mod` line in `ui_test/main.rs`.

## Scope boundaries (explicit out-of-scope)

- The traversal degree-setting-not-respected-on-activation bug
  (open correction `ui.traversal_view_degree_setting_bug`; a previous fix
  attempt FAILED) is OUT OF SCOPE. Its code lives in `graph.js::switchTab`
  traversal branch, `parseAndRenderElements`, `filterAndExpandSubgraph` and in
  the WIP. This track's only obligation is I4: zero diff there. It must not
  regress, absorb, or silently re-fix the bug, must not amend the failed WIP,
  and must not claim the bug fixed anywhere. It stays OPEN after this milestone.
- Multi-level back / navigation history stack: out (one level covers every
  entry; 4-tab app with a visible tab bar).
- Esc-key or global shortcut: out (none exists today for this panel).
- `panel-restore-tab` left-edge affordance: out (the Traversal tab button
  stays visible; hotspots need theirs because they have no tab).
- `themes.css` / `theme.js` / token baseline: out (token-only restyle, I6).
- graph.js 998 budget-lock relocation and "(cap 990)" message fix: out
  (deferred ledger row; this track keeps `graph.js` byte-identical).

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/ui-traversal-back-button-t1-journal.md` (checkpoint journal; create
  it; always in-set)
- `crates/ui/assets/index.html` (exactly 2 lines: the `#tabTraversal` onclick;
  the traversal-head button)
- `crates/ui/assets/traversal.js` (tail `closeTraversal` block replaced; the
  `window.closeTraversal` export line replaced by two export lines)
- `crates/ui/assets/hotspots.js` (1 line in `openTraversalForRoot`)
- `crates/ui/assets/styles.css` (2 selector renames + 2 declarations, in place)
- `crates/ui/tests/ui_test/traversal_back.rs` (NEW lock module)
- `crates/ui/tests/ui_test/main.rs` (+1 line: `mod traversal_back;` appended
  as the last line of the mod roster; roster-tail is the alphabetical site)

You MUST NOT touch: `crates/ui/assets/graph.js`, `main.js`, `api.js`,
`themes.css`, `theme.js`, `tools/theme_token_baseline.json`, any file under
`crates/ui/src/**` (asset changes ride `include_str!` - `lib.rs`, `server.rs`,
`export.rs` are touched-by-embedding-only, zero edits), anything under
`tools/**`, any pre-existing test file (including `static_html.rs`,
`static_html_sections.rs` - it carries the 998 lock - and `theme_contract.rs`),
`docs/specs/collateral-traversal-view.md` (the WIP's own doc), any path under
`tests/**` outside `crates/ui/tests`, and any `Cargo.toml`/`Cargo.lock`.
No `git stash`, no branch change, no rebase, no amending the WIP's commits.

## Step 0 - baseline red capture (BEFORE any edit; this is the red baseline)

1. Record the track start rev (`git rev-parse HEAD`, or the orchestrator
   reports it if your session's permissions deny git) in the journal as
   `TRACK_BASE <rev>`. If the working tree is dirty on touch-set files, STOP
   (dispatch precondition above).
2. Run `python tools/testrun.py --distill -- -p ariadne-ui` (timeout 600000 ms).
3. In the journal, record VERBATIM: the canonical summary line, the digest
   header, the capture path, and the FULL failing-test-name set read from
   `target/testrun-capture.txt`. Label it `BASELINE FAILING SET (pre-existing)`.
   Expected at the known baseline: it contains at minimum
   `ui_test::static_html_sections::test_extraction_impact_preview_overlay`
   (assert `GRAPH_JS.lines().count() <= 998` fails; working-tree graph.js is
   1051 lines). Record the actual set whatever it is - you reconcile against
   what you MEASURE, never against guessed numbers (standing ledger errata).
   This recorded set is the ONLY machine-checkable definition of pre-existing
   red for this track; every later gate compares against it by name.

## What to build (contracts C1-C4; edits are content-anchored, before-strings
verbatim from the tree as of 2026-10-09)

C1 Button markup (`index.html`, inside `.traversal-head`), before (exact):

```
        <button class="traversal-close" onclick="closeTraversal()" title="Close">&#10005;</button>
```

after (exact; pure-ASCII source - `&#8592;` is a numeric entity, no literal
non-ASCII byte; no SVG - panel-head controls in this app are text/entity
buttons):

```
        <button class="traversal-back" onclick="backFromTraversal()" title="Back to the view you came from">&#8592; Back</button>
```

C2 `traversal.js` tail block (classic-script global scope; loads at
`index.html` before `hotspots.js` and `main.js`, so click-time resolution is
safe; `currentTab` is declared in `main.js` and is already read cross-file by
`graph.js` - same pattern). Before (exact block, 4-space file indentation):

```
    function closeTraversal() {
      // Switch main tab back to include (which restores inspector)
      switchTab('include');
    }
```

after (keep the file's indentation; add brief intention-revealing doc
comments):

```
    // Return-to-origin: the tab active when traversal was last entered.
    let traversalReturnTab = 'include';

    // Enter traversal from any tab, capturing the origin tab for Back.
    function openTraversalTab() {
      if (currentTab !== 'traversal') traversalReturnTab = currentTab;
      switchTab('traversal');
    }

    // Leave traversal back to the captured origin tab.
    function backFromTraversal() {
      switchTab(traversalReturnTab);
    }
```

Export swap in the same tail, before (exact):

```
    window.closeTraversal = closeTraversal;
```

after:

```
    window.openTraversalTab = openTraversalTab;
    window.backFromTraversal = backFromTraversal;
```

`window.closeTraversal` has no other caller (verified 2026-10-09: definition +
the `index.html` button only), so the rename is total.

C3 Entry-point swaps (exactly two; both must go through the capturing
function):
- `index.html`, the `#tabTraversal` button:
  `onclick="switchTab('traversal')"` becomes `onclick="openTraversalTab()"`;
  the `class="tab-btn"`, `id="tabTraversal"`, SVG, and label stay byte-equal.
- `hotspots.js::openTraversalForRoot`, first statement, before (exact, 6-space
  indentation; the only `switchTab('traversal')` occurrence in that file):

```
      switchTab('traversal');
```

after:

```
      openTraversalTab();
```

The root-select retry loop below stays untouched. After C3, neither
`INDEX_HTML` nor `HOTSPOTS_JS` contains `switchTab('traversal')` anywhere
(verified: each contains exactly the one occurrence swapped above).
Future programmatic `switchTab('traversal')` callers fall back to the default
`'include'` - today's behavior, strictly non-regressive.

C4 CSS rename (`styles.css`; net delta ~2 lines; this file is exempt from the
R2 suffix scan - a known deferred ledger row - so rename in place). Before
(exact):

```
    .traversal-close {
      margin-left: auto;
      background: transparent;
      border: none;
      color: var(--text-muted);
      cursor: pointer;
      font-size: 11px;
      padding: 2px 4px;
    }

    .traversal-close:hover {
      color: var(--text-main);
    }
```

after (declarations unchanged except the two additions per shape C4; no hex
literal anywhere in the diff):

```
    .traversal-back {
      margin-left: auto;
      background: transparent;
      border: none;
      color: var(--text-muted);
      cursor: pointer;
      font-size: 11px;
      padding: 2px 4px;
      gap: 4px;
      align-items: center;
    }

    .traversal-back:hover {
      color: var(--text-main);
    }
```

C5 New lock module `crates/ui/tests/ui_test/traversal_back.rs` + the `mod`
line. Why a new file: `static_html.rs` sits at 999 lines - AT the 1000 R2
ceiling - and is frozen by I7; the split-file precedent is
`static_html_sections.rs`. The module reads only the four `ariadne_ui`
constants; no server fixture, no fetch, no new error shapes. Doc-comment
style mirrors `hotspot_direction.rs`.

## RGB protocol (per this file order)

1. Red: write `traversal_back.rs` + the `mod` line FIRST. Run
   `python tools/testrun.py --distill -- -p ariadne-ui` (timeout 600000 ms).
   The file MUST compile (the constants already exist) - a test that fails to
   compile is not Red. Expected: exactly three NEW assertion failures on top
   of the Step 0 baseline set, each with its own message:
   - `traversal_back::test_back_control_markup` - first failing assert is the
     missing `class="traversal-back"` containment on `INDEX_HTML`;
   - `traversal_back::test_return_capture_wiring` - first failing assert is
     the missing `let traversalReturnTab = 'include'` containment on
     `TRAVERSAL_JS`;
   - `traversal_back::test_back_css_token_only` - first failing assert is the
     missing `.traversal-back` rule on `STYLES_CSS`.
   Capture the failing set verbatim in the journal (must equal
   BASELINE set UNION these three).
2. Green: apply C1-C4. Re-run the same scoped command. NEVER weaken an
   assertion to pass. Expected: failing set is EXACTLY the BASELINE set again
   (same test names, by name-set identity), and passed count = Step 0 passed
   + 3. The graph.js budget test must still fail with the same message -
   unchanged, untouched, not yours.
3. Refactor: doc comments on the three JS functions, honest names (the C2
   names are the contract), dedupe any copied assert helper. Re-run scoped
   suite: same expectation as Green. Run `python tools/hygiene_check.py`.

## Required tests (exact expected behavior of each assert)

- `test_back_control_markup` (I1): `INDEX_HTML` contains
  `class="traversal-back"`, `onclick="backFromTraversal()"`, and
  `title="Back to`; `INDEX_HTML` contains neither `traversal-close` nor
  `closeTraversal`; `TRAVERSAL_JS` contains no `closeTraversal`; `STYLES_CSS`
  contains no `.traversal-close`.
- `test_return_capture_wiring` (I2): `TRAVERSAL_JS` contains
  `let traversalReturnTab = 'include'`, `function openTraversalTab()`,
  `currentTab !== 'traversal'`, `function backFromTraversal()`, and
  `switchTab(traversalReturnTab)`; `INDEX_HTML` contains
  `onclick="openTraversalTab()"` and contains NO `switchTab('traversal')`;
  `HOTSPOTS_JS` contains `openTraversalTab()` and contains NO
  `switchTab('traversal')`.
- `test_back_css_token_only` (I6): locate the `.traversal-back` rule body in
  `STYLES_CSS` (selector to first closing brace, mirrors the
  `step_row_css_block` token-only pattern); assert the body contains `var(--`
  and contains no `#` (no hex); assert `STYLES_CSS` contains no
  `.traversal-close`.

## Hygiene (review-enforced)

Functions < 100 lines (largest JS addition is 4 lines); files < 1000 lines -
`traversal.js` grows 406 to ~420, `traversal_back.rs` stays small,
`static_html.rs` is frozen and at the ceiling (do not add there);
`styles.css` is R2-scan-exempt (known deferred row), rename in place. No
banned names (`tmp`/`res`/`foo`/`bar`/`test1`, single-letter locals);
intention-revealing identifiers (the C2 names are contractual). Source lines
< 2000 chars. `python tools/hygiene_check.py` stays exit 0.

## Frozen locks (all inherited; proof commands scoped)

| Lock | Proof |
|---|---|
| 9-script roster + tag-start asset whitelist (`theme_contract.rs` incl. `/app/traversal.js` entry) unedited and green | scoped ui suite |
| graph.js line budget `<= 998` at `static_html_sections.rs:215` - preserved trivially by graph.js zero-diff; do NOT amend the number or the stale "(cap 990)" message | `git diff TRACK_BASE -- crates/ui/assets/graph.js crates/ui/tests/ui_test/static_html_sections.rs` EMPTY (the pre-existing red rides through unchanged) |
| Asset byte-serving + `no-store` route test and `test_script_tag_order_preserves_original_execution_order` unedited (neither lists `traversal.js` content, so the content change rides through; do NOT extend the route roster to include it - I5 relay) | scoped ui suite, failing set = baseline set |
| Standalone export inlines split assets, no `src="/app/` (`test_standalone_export_inlines_all_split_assets`) unedited | scoped ui suite |
| Zero added hex literals | `python tools/theme_token_check.py --check-added --base TRACK_BASE` silent |
| No mojibake; pure-ASCII authored content | `python tools/encoding_guard.py --staged` exit 0 before each commit (`--all` is the reviewer hop) |
| Hygiene R1/R2 blocking | `python tools/hygiene_check.py` exit 0 |
| Goldens (`m00` parity), `tests/fixtures/**`, extraction-card story/playbook suites untouched | they live outside the touch-set; reviewer `git diff --name-only` set equality |

## Gates (exact commands; implementer hop; timeout 600000 ms each; scoped only)

1. Step 0 baseline capture: `python tools/testrun.py --distill -- -p ariadne-ui`
   (journal: BASELINE FAILING SET recorded before any edit).
2. Red: same command (3 new assertion failures on top of baseline; capture).
3. Green: same command (failing set == baseline set by name identity; passed
   = baseline + 3; digest header + canonical line verbatim in the journal).
4. Refactor re-run: same command (same verdict as Green).
5. `python tools/hygiene_check.py` - exit 0.
6. `python tools/theme_token_check.py --check-added --base TRACK_BASE` - silent.
7. `python tools/encoding_guard.py --staged` - exit 0 before EACH commit.
8. Commit with `python tools/commit_scoped.py --scope crates/ui/assets/*
   --scope crates/ui/tests/ui_test/traversal_back.rs --scope crates/ui/tests/ui_test/main.rs
   --scope tasks/ui-traversal-back-button-t1-journal.md -m "feat(ui): traversal Back control returns to origin tab"`
   (one landing commit preferred; never `git add .`; if your session's
   permissions deny `commit_scoped.py`/`git`, STOP and report - commit duty
   routes to the orchestrator per the standing allowlist errata row; do not
   improvise raw git staging).
Journal duty (ledger law, M28 T0b finding 1): itemize per-gate invocation
counts and DECLARE any overshoot beyond 10 as a dated D-row.

NOT this session's gates (workspace-wide; reviewer hop only): `ui_smoke`,
`encoding_guard --all`, workspace census, fresh-session full scoped re-run.

## Reviewer hop (fresh session; adjudicate via `review-gate`; re-run FRESH)

1. Scoped ui suite fresh; failing set == the journal's BASELINE FAILING SET by
   name identity; the three `traversal_back` tests green; the graph.js budget
   test's pre-existing red is NOT attributed to this track (it survives only
   while the two zero-diff proofs below hold).
2. `git diff --name-only TRACK_BASE..HEAD` set-equality against the Touch-set
   minus the journal; specifically `git diff TRACK_BASE --
   crates/ui/assets/graph.js crates/ui/tests/ui_test/static_html_sections.rs`
   EMPTY; `git diff TRACK_BASE -- crates/ui/src` EMPTY (R5: Rust-source
   zero-diff; asset changes ride `include_str!`; no manifest/lockfile diff);
   the I3 list (api.js, main.js, themes.css, theme.js, tools/**, tests/**
   outside crates/ui/tests) diff-empty; `git diff --name-only` filtered to
   `crates/ui/tests` == exactly `traversal_back.rs` + `main.rs` (I7).
3. `python tools/ui_smoke.py` PASS (route via
   `python tools/distill.py -- python tools/ui_smoke.py` if needed).
4. `python tools/theme_token_check.py --check-added --base <landing-parent>`
   silent; `python tools/hygiene_check.py` exit 0;
   `python tools/encoding_guard.py --all` exit 0.
5. Workspace census `python tools/testrun.py --distill -- --workspace`:
   passed count = live baseline passed + 3, reconciled from the live tree
   (the `--baseline` FILE form is the standing 5-encounter errata - use
   `--distill`; NEVER reconcile against a guessed number).
6. Per-commit `git show` oldest-first; commit-vs-touch-set enforcement;
   journal carries the G7 manual-smoke line: enter via tab button from symbol
   tab -> Back lands on symbol; from call tab -> Back lands on call; enter via
   hotspot card -> Back returns to include with the hotspot panel restored;
   degree behavior unchanged from baseline (still the known-open bug, not
   worsened). If the implementer could not run the browser, the journal must
   say "manual smoke pending operator" - an unexecuted matrix must not be
   checked off; flag to the orchestrator instead.

## Discipline clauses

- UTF-8: never `Set-Content`/`Out-File`/`>`/`Get-Content -replace`;
  Read/Edit/Write tools only; authored content pure ASCII (`&#8592;` entity,
  not the byte) - see AGENTS.md File-Edit Discipline.
- Touch only the touch-set; on discovering out-of-scope breakage: report, do
  not fix. The degree bug is the prime trap here (I4).
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`, raw workspace census) in
  this implementer session are a reviewer blocker; everything routes through
  the distilling entry points (`docs/specs/tool-output-distillation.md`).
- After EACH per-module Green gate, append a 1-3 line checkpoint to
  `tasks/ui-traversal-back-button-t1-journal.md`: module + commit hash, any
  deviation, any lock/contract finding - including the Step 0 baseline set and
  any anchor mismatch.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ui-traversal-back-button-t1-handoff.md --scope crates/ui --cmd "python tools/testrun.py --distill -- -p ariadne-ui"`.

## Report back (final message)

1. Commit hash + `git show --stat`. 2. Red evidence: the 3 new assertion
messages + the failing set equal to BASELINE UNION the three. 3. Green
evidence: distill digest header line + canonical summary line verbatim +
capture path, failing set name-identical to BASELINE, passed = baseline + 3.
4. Any deviation + reason (declare gate-count overshoot as a D-row).
5. Out-of-scope observations (report-only; e.g. anything you noticed about the
degree bug - do not touch it).
Never paste raw tool output: cite command + canonical line + exit code. The
reviewer re-runs gates against the commits; pasted output is context rot.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill (kickoff gates G1-G8 map: G1/R1, G3-G5/R2+R4, G6/R2, G7/R6, G8/R5); its
commit-vs-touch-set check enforces the Touch-set above, which therefore lists
every file this task may legitimately change.
