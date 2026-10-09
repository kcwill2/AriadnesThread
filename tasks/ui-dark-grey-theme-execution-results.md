# Execution results - ui-dark-grey-theme (milestone-level acceptance pass, reviewer hop T1)

Written by the reviewer gate for `tasks/ui-dark-grey-theme-t1-handoff.md`
(verdict: `tasks/ui-dark-grey-theme-t1-verdict.md`). All commands run fresh
in this session against HEAD `4900e04` (commits `41ec354` + `4900e04`), base
`d76a559`. Environment: Windows, `main`, warm cargo cache.

## Acceptance criteria vs what actually happened

### A1. Handoff scoped gate - `python tools/testrun.py --distill -p ariadne-ui`
PASS (exit 0). Verbatim:

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 207 lines]
188 passed / 0 failed / 0 ignored / 3 suites
```

Independently reproduces the implementer's claimed canonical line exactly.
Note: the capture file path is shared; a concurrent session later overwrote
it (see A5) - the run happened and is attested by the two lines above.

### A2. Frozen-hex drift guard - `python tools/theme_token_check.py --check-added --base d76a559`
PASS. Exit 0, zero output (the tool prints violation lines on exit 1 and
`ERROR ...` on exit 2; none emitted). themes.css is TOKEN_FILES-exempt from
the added scan; the frozen COUNT scan validated the regenerated baseline in
the same commit (211 -> 281, the only changed count).

### A3. Hygiene - `python tools/hygiene_check.py`
PASS (exit 0). Emitted rows are all warn-only R3 pre-existing sites
(`tmp`/`res`/`bar`/single-letters in untouched crates and pre-existing test
files); zero rows in `theme_dark_grey_palette.rs` or in any file this hop
edited. No R1/R2 blockers.

### A4. Encoding - `python tools/encoding_guard.py --all`
PASS. Exit 0, no output. Covers the em-dash-bearing bootstrap comment line
that the Edit tool rewrote ASCII-substring-wise: no mojibake, no non-UTF-8
blob. (AGENTS.md forbids piping git blobs through PowerShell; the byte-safe
Read tool was used for direct line comparison instead.)

### A5. Workspace-wide census - `python tools/testrun.py --baseline` / substitute `--distill`
NOT RUN (blocked; environment, not the adjudicated tree). Two distinct
blockers, each with its own verbatim evidence:

- Instructed form `python tools/testrun.py --baseline` (no FILE arg):
  `usage error: missing value for --baseline`. `--baseline` is a
  baseline-FILE comparison flag (`tools/testrun.py:20,60,119-128`); this is
  the already-recorded G7 erratum (ledger `cad6efb`: "baseline-FILE form
  unrunnable -> errata row"). No baseline file with the current census
  quadruple exists in the repo to point it at.
- Substitute full-workspace form `python tools/testrun.py --distill` (runs
  `cargo test --workspace`): three attempts, all:
  `### cargo test DRIFT (exit 2) [capture: ...testrun-capture.txt, ~185 lines]`
  `parse format drift: no test result line matched the pinned parse spec`.
  Capture root cause (verbatim):
  `error: failed to remove file C:\Users\Kurt\Code\AriadnesThread\target\debug\tool.exe`
  `Access is denied. (os error 5)`
  `warning: build failed, waiting for other jobs to finish...`
  A concurrent ui-extraction-card-story implementer session went RED in this
  shared worktree mid-review: uncommitted
  `crates/ui/tests/ui_test/extraction_card_story.rs` (9 tests, its own scoped
  capture shows `189 passed; 8 failed` = our 188 + their 1, all 8 failures
  `extraction_card_story::*`) and a live process holds `tool.exe` (the
  ariadne-cli bin, `crates/cli/Cargo.toml` `name = "tool"`). The parser is
  healthy: the same binary parsed the A1 scoped run cleanly.
  DISPOSITION: orchestrator re-runs the census in a quiet worktree (G7
  fresh-session precedent). For attribution: the 8 failures observed in the
  contaminated capture belong to ui-extraction-card-story WIP, NOT to the
  adjudicated commits.

### A6. Tool unit suites - `python -m unittest discover -s tools -p "*_tests.py"`
PASS. Verbatim:

```
Ran 188 tests in 19.987s

OK
```

### A7. UI runtime smoke - `python tools/ui_smoke.py`
NOT RUN - refused by the reviewer permission profile, twice (retry to
confirm determinism), each with verbatim:

```
The user has specified a rule which prevents you from this tool call: {"permission":"bash","pattern":"*","action":"deny","source":"global"}
```

(approximate quoting of the tool-result field; the JSON rule object is exact).
Secondary blocker had it been allowed: its `cargo build --bin tool` step
would hit the A5 `tool.exe` lock. DISPOSITION: orchestrator/fresh session;
closes shape acceptance gate 4 (`ui_smoke` default-theme boot unchanged).

### A8. Load-perf regression - `python tools/bench_compare.py`
NOT RUN - refused identically by the permission profile (verbatim refusal as
A7). Had it run, the same `tool.exe` debug-bin lock applies; the hop is
assets/tests-only, so the ui-progress-parity precedent's assets-only
exemption ("bench_compare optional-not-run: assets-only, pipeline
byte-identical") is available to the orchestrator with a verdict sentence.

### A9. Manual browser pass in Dark (Grey) (shape acceptance 6)
NOT RUN by this reviewer - a human/browser item. Machine surrogates all
green in A1: semantic-layer byte-equality with the default block (cycle card
stays red, heat ramp cool->hot, verdict spine hues identical by
construction), `test_standalone_export_inlines_theme_tokens` extended to the
grey block + option presence, bootstrap/registry validators degrade unknown
ids to `dark` (I10), option ordering and `selected` locked (I1). DISPOSITION:
hand to the walkthrough/closeout hop for the human pass.

### A10. Same-commit orchestrator duties (shape acceptance 7)
OPEN by design - the `ui.three_theme_set` memory row -> four-theme flip and
the ledger row status flip are orchestrator-owned landing-window edits; at
verdict time the ledger row still reads PLANNED (dirty +2 in the worktree
belongs to that pending edit). Recorded in the verdict; not implementer
chargeable.

## Reviewer-side static adjudication evidence (non-command)

- Commit-vs-touch-set: `git diff --name-only d76a559..HEAD` = 9 paths, all
  inside the handoff touch-set plus the handoff file itself, authorized by
  gate 7's `--scope "tasks/ui-dark-grey-theme-t1-*"` glob. styles.css, other
  assets, fixtures, pipeline, progress.rs: absent from every commit.
- Per-commit `git show` oldest-first: `41ec354` (9 files, +990/-84, baseline
  justification sentence verbatim in the message), `4900e04` (journal +25
  lines only).
- Value-copy audit via byte-safe Read: grey block 106 tokens, default order,
  only the 17 I7 chrome tokens differ from default; every decorative/node/
  proposed value byte-equal to default; +70 hexes exactly (211 -> 281).
- Precondition: T3 commits `e9cf10d`, `9d96272`, `17e2299` are ancestors of
  `d76a559`; `<div id="loadingSteps"></div>` live at index.html:32;
  `crates/ui/tests/ui_test/static_html.rs` present;
  `crates/ui/src/static_html.rs` confirmed nonexistent (deviation D1).
- R7 sweep: zero "three"/"Three" hits in `index.html`.

## Explicit not-run statement

Not run in this hop: the workspace census quadruple (A5, environment-
blocked), `ui_smoke.py` (A7, permission-blocked), `bench_compare.py` (A8,
permission-blocked), and the manual Dark (Grey) browser pass (A9, human
item). Everything else on both the handoff gate list and the reviewer
acceptance list was run fresh and passed; quoted lines above are verbatim.
