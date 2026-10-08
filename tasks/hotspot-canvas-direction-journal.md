# Hotspot Canvas Direction - implementer journal (T1)

Track: `ui-hotspot-canvas-direction` (post-M28, no M-number).
Handoff: `tasks/hotspot-canvas-direction-t1-handoff.md` incl. AMENDMENT A1
(contracts verbatim; shape `tasks/hotspot-canvas-direction-shape.md`; kickoff
`tasks/hotspot-canvas-direction-kickoff.md`).
Date: 2026-10-08.

Start HEAD: `c00576d4a574052e00c88ad4e36ca2f1b3aeefa0`.
Baseline (planner at c00576d): `209 passed / 0 failed / 0 ignored / 3 suites`.

## STATUS: GREEN LANDED via single scoped commit_scoped landing (this commit).

Hop 1 stopped on the provable cap-990 self-contradiction (recorded below);
orchestrator authorized Option A as AMENDMENT A1 in the handoff; hop 2
re-applied the authorized line and landed.

## STOP guard checks

Hop start and hop-2 start: `git status --porcelain` +
`git diff --cached --name-only` show NO staged hunks anywhere (empty index);
`main.js`, `styles.css`, `themes.css`, `theme.js`, `crates/ui/src/**`
(`static_html.rs` in particular) untouched by this hop, before and after.

## Hop 1 - RGB trail up to the blocker

- RED (tests + mod line written FIRST): `209 passed / 8 failed / 0 ignored /
  2 suites` exit 1; the 8 failures were exactly the new `hotspot_direction`
  locks on their predicted string asserts (missing `function
  hotspotMetricDirection(`, missing lens flags, missing delegate branch,
  missing copy); no compile errors.
- GREEN attempt (pre-A1): `216 passed / 1 failed / 0 ignored / 2 suites`;
  all 8 new locks green; the single failure was the frozen lock
  `static_html_sections.rs:215` `GRAPH_JS.lines().count() <= 990` (graph.js
  baseline 990 = zero headroom vs mandated +6 contract shape). STOPped:
  pre-existing test files were then forbidden; reported, did not fix; nothing
  staged or committed.

## AMENDMENT A1 + live-tree drift incident (hop 2)

- A1 authorizes exactly one line: `static_html_sections.rs` :215
  `<= 990` -> `<= 998` (this handoff's own +8 HARD CAP); the :216
  "(cap 990)" message literal stays stale BY AUTHORIZATION (deferred ledger
  row for the heat/legend extraction follow-up owns its correction).
- Drift found at hop-2 start: HEAD had moved c00576d -> 7036c9e (three
  `docs(tasks)` commits by the parallel ui-extraction-card-clarity T-A
  session; `git diff --name-only c00576d..HEAD` = only `.kilo/plans/**` and
  `tasks/*.md` - no crates overlap; read-only claim verified). The applied
  A1 line had been REVERTED by an unidentified party (file back to zero-diff
  with `<= 990` while graph.js sat at the 996 WIP).
- Cross-check before staging: full `git diff --stat` matches this journal's
  hop-1 numstat exactly for every touch-set file (graph.js 8 [7+/1-],
  hotspots.js 147 [146+/1-], index.html +4, main.rs +1); the only other
  modified entries are foreign-owned unstaged files recorded in the hop-1
  census (`.kilo/plans/1790697359283-*`, `tasks/ledger_archive.md`,
  `tasks/milestone_ledger.md`, `tools/bench_baseline.json`). No other drift
  -> proceed.
- Re-applied the single A1 line via the Edit tool. Proof:
  `git diff --stat -- crates/ui/tests/ui_test/static_html_sections.rs` ->
  `1 file changed, 1 insertion(+), 1 deletion(-)` - exactly the authorized
  delta, nothing else in the file.

## Implementation summary (all inside the touch-set)

- `hotspots.js`: lens globals + doc comment, onchange reset,
  `hotspotMetricDirection`, new `selectHotspotRow` (bare `selectNode(id)`
  literal preserved), `computeFullConsumers` (reverse BFS),
  `renderInboundHeatView` (95 lines < 100; consumer-first payer badges; own
  legend copy; token-only colors; degradation contract). 465 -> 610 lines.
- `graph.js` +7/-1 (net +6 <= +8 HARD CAP): selectNode consumes the lens as
  its first two statements (comment included in the +3 hunk); renderSvg
  leading provenance branch + `} else if` rewrite; outbound heat block
  BYTE-IDENTICAL (I5). 990 -> 996.
- `index.html`: one help `<li>`, copy verbatim, `&larr;`/`&rarr;` entities,
  sentence-per-line, ASCII.
- `hotspot_direction.rs`: the 8 named locks; local `function_body` copy
  (panic text asset-neutral, cosmetic-only deviation from the model helper);
  I6 via regex scan; `main.rs` one `mod` line at the alphabetical site.
- `static_html_sections.rs`: ONLY the A1 line (:215, 990 -> 998).
- R4 (transient same-frame render) ACCEPTED per the shape, recorded here
  because the area was touched.

## Gates - final results (landing state)

1. `python tools/testrun.py --distill -p ariadne-ui` ->
   `### cargo test PASS (exit 0)` / `217 passed / 0 failed / 0 ignored /
   3 suites` (suites reads 3 at GREEN as A1 requires; the `2 suites`
   panic-abort artifact appeared only in the failing runs). Pre-existing
   tests green UNEDITED except the A1 one-liner.
2. `python tools/hygiene_check.py` -> exit 0; no R1/R2 rows at all; R3 warn
   list carries only pre-existing rows in files outside the touch-set.
   graph.js 996 < 1000; renderInboundHeatView < 100 lines.
3. `python tools/theme_token_check.py --check-added --base
   c00576d4a574052e00c88ad4e36ca2f1b3aeefa0` -> silent (zero hex literals
   added; range also includes foreign docs commits - still silent).
4. `python tools/encoding_guard.py --staged` -> silent, exit 0, after
   staging the seven explicit touch-set paths, before commit (boot-overlay
   precedent for the staging step; commit itself via commit_scoped only).
5. `python tools/distill.py -- git diff HEAD -- crates/cli` -> empty capture
   (0 lines): DP-1 intact.
6. Commit: `python tools/commit_scoped.py` with exactly the amendment's
   seven `--scope` globs (hotspots.js, graph.js, index.html,
   hotspot_direction.rs, main.rs, static_html_sections.rs, this journal);
   single landing commit (this commit); commit_scoped's printed numstat is
   the graph.js <= +8 proof for the reviewer.

## Gate-invocation itemization (distilling entry points)

Hop 1 (11): 1 rev-parse HEAD; 2 status --porcelain (STOP guard); 3 git grep
function_body scoping; 4 testrun RED; 5 testrun GREEN-attempt (blocker
found); 6 diff --numstat; 7 hygiene; 8 theme_token; 9 encoding_guard
(vacuous, nothing staged at stop); 10 diff HEAD -- crates/cli; 11 scoped
final census.
Hop 2 (13): 12 log --oneline -5 (HEAD drift); 13 status --porcelain;
14 diff --stat (full cross-check); 15 diff --name-only c00576d..HEAD
(foreign commit scope); 16 diff --cached --name-only (index empty);
17 diff --stat static_html_sections.rs (A1 proof); 18 testrun GREEN;
19 hygiene; 20 theme_token; 21 diff HEAD -- crates/cli; 22 git add (seven
explicit paths, staged-scan setup); 23 encoding_guard --staged;
24 commit_scoped.
One denied invocation not counted (hygiene piped through Select-String;
non-canonical form refused; plain form re-run as gate 19).

## Deviations (declared D-rows, M28 T0b law)

- D1 (2026-10-08): journal at the handoff touch-set path
  (`hotspot-canvas-direction-journal.md`), not the prompt's `-t1-` variant;
  confirmed by ruling "journal-name ruling ... stands as written".
- D2 (2026-10-08): `2 suites` in FAILING digests only (RED, pre-A1 216/1):
  pre-existing panic-abort testrun artifact; GREEN reads `3 suites` per
  acceptance item 1.
- D3 (2026-10-08): cumulative invocation count 24 vs the pre-amendment
  budget <=10 - the amendment authorized a whole extra hop (drift census +
  re-apply + gate re-runs + landing); harmless declared overshoot; full
  itemization above.
- D4 (2026-10-08): gate 4 requires a STAGED scan before commit while the
  landing commit is commit_scoped-internal; staging done via
  `git add -- <seven explicit touch-set paths>` (identical explicit-path form
  commit_scoped itself uses; boot-overlay precedent; never `git add .`/`-A`;
  commit content still governed by commit_scoped `--only -- <paths>`).
- D5 (2026-10-08): `function_body` copy keeps the model's logic verbatim
  with an asset-neutral panic message (judges both hotspots.js and graph.js).

## Deliberately not done

- Touched nothing outside the touch-set + the A1 single line;
  `main.js`/`styles.css`/`static_html.rs` remain untouched and were never
  staged by this hop.
- Did not pursue Option B (statement-fusing / I3 weakening) - rejected by
  the amendment.
- :216 "(cap 990)" stale message literal intentionally NOT corrected (A1
  authorization; belongs to the deferred extraction-follow-up ledger row).
- Did not run reviewer-hop gates: `ui_smoke`, workspace-wide baseline census,
  per-commit `git show` adjudication.
- Did not split commits: the amendment lists a single seven-scope command.

## Closeout relay (hop 3, 2026-10-08)

Planner prepared the track closeout (its role lacks commit rights); routed
through this implementer session verbatim. Pre-commit verification all
green: HEAD at `c17d182`; index empty; `tasks/milestone_ledger.md` unstaged
delta exactly +3/-2 (track-row flip to CLOSED/APPROVED @ c17d182 at :35,
`--baseline` errata 5TH-encounter append at :63, deferred :215-lock
relocation row at :79/80 area — all `-`/`+` lines hotspot-closeout content,
foreign ledger rows context-only); the six closeout docs present untracked
(plan digest 69, execution-results 84, kickoff 131, shape 323, t1-handoff
430, t1-verdict 214 lines); gate 4 `encoding_guard --staged` silent exit 0
after staging the seven explicit paths.

- Closeout commit: `b4e2bb5a935e779c928ea69dd2a3128d1c17b046`
  (`git show --stat`: exactly the seven planner scopes, 1254 insertions /
  2 deletions; nothing from foreign/M28/clarity/boot-overlay artifact sets).
- This rider commit records the closeout hash per the repo 3-commit
  convention (code / docs / hash-record) named in the ledger template-
  collision row.
- Hop-3 invocation itemization (8): rev-parse HEAD; status census (one path-
  scoped variant denied, non-canonical form, re-run plain); diff --numstat
  ledger; diff -U2 ledger (hunk content proof); diff --cached --name-only;
  ls-files --others (untracked proof); git add (seven explicit paths);
  encoding_guard --staged; commit_scoped; git show --stat. Cumulative track
  count carried under the D3 declaration (amended hops each re-authorized
  their own gate surface; this rider commit adds staging + guard + commit +
  show per the convention above).

Manual smoke line (acceptance (e)): clicking a burden row shows the toolbar
reading "Consumers (<-)" with consumer cards badged with their payments, not
faded. (Code-path verified deterministically: `selectHotspotRow` sets
`dirSelect` to `in` -> option label "Consumers (&larr;)", arms the lens ->
`renderInboundHeatView` heat-styles payers with `+N col` / k-over-m badges
and fades only non-consumers; the interactive browser click remains with the
reviewer hop per R8 - no DOM harness exists in this repo.)
