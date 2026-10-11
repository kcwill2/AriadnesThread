# AR-01 Verdict - APPROVED

Handoff: `tasks/ar-01-handoff.md` (S1 CRIT, S3 HIGH, S4-partial MED).
Landing commit: `6d7f96b06d795a7b5de3b538d16879ebab2af148` (single commit; range
`6d7f96b~1..6d7f96b` per `git log --oneline`).
Adjudicated 2026-10-09 via `review-gate` skill, fresh session, full re-run of
every gate. No implementer claim taken on trust that could be measured.

## 1. Gate transcript (reviewer-run, verbatim)

- `python tools/testrun.py --distill -- -p ariadne-ui`
  -> `245 passed / 0 failed / 0 ignored / 3 suites`
  Delta vs journal's 242 is fully explained: the shared worktree carries
  uncommitted AR-02 WIP whose `server_api.rs` diff adds exactly 3 CORS tests.
  All 8 `source_containment::` locks individually observed `ok` in
  `target/testrun-capture.txt` (lines 171-184 of the reviewer's capture).
- `python tools/distill.py -- python tools/ui_smoke.py` -> `ui_smoke: PASS`
  (generic PASS, exit 0; capture `target/distill-capture.txt`).
- `python tools/encoding_guard.py --all` -> silent; exit 0. Exit contract
  confirmed from source (`return 1 if violations else 0`; violations printed):
  zero violations means zero output. (`--all` exit-0 is strictly stronger than
  the handoff's `--staged` gate.)
- `python tools/hygiene_check.py` -> exit 1 with exactly the two pre-existing
  server.rs blockers: `crates/ui/src/server.rs: R2 1155 lines > 1000` and
  `crates/ui/src/server.rs:799: R1 fn 'api_graph_traversal' is 161 lines (> 100)`.
  ZERO entries for any AR-01 file (export.rs, hotspots.js, main.rs,
  source_containment.rs absent from R1/R2/R3 output).
- Tools suites: `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 219 tests / OK` (matches plan anchor exactly).
- Workspace census `python tools/testrun.py --distill` -> NOT RUNNABLE in this
  worktree: `parse format drift` (exit 2) because uncommitted cancelled-AR-02
  WIP adds a 7th `run_ui` parameter (`crates/cli/src/commands/ui.rs`,
  `+    expose_opt: Option<String>` visible as a dirty `+` hunk) while the
  committed integration callers stay at 6 args (E0061 in
  `m16_traversal_test`, `m17_dir_nav_test`). Attribution to AR-01 is
  disproved: the commit's file list (`git diff --name-only 6d7f96b~1..6d7f96b`)
  contains no cli/integration file. Fallback census
  `python tools/testrun.py --distill -- --workspace --exclude ariadne-integration-tests`
  -> `1169 passed / 0 failed / 13 ignored / 58 suites` (13 ignored matches the
  plan anchor). See execution-results addendum.
- `python tools/testrun.py --baseline`: NOT RUN - not runnable as written:
  `--baseline` requires a FILE argument and no testrun baseline file exists in
  the tree (only `tools/theme_token_baseline.json` / `tools/bench_baseline.json`).

## 2. Commit-vs-touch-set - EXACT (pass)

Touched by `6d7f96b`: `crates/ui/assets/hotspots.js`, `crates/ui/src/export.rs`,
`crates/ui/src/server.rs`, `crates/ui/tests/ui_test/main.rs`,
`crates/ui/tests/ui_test/source_containment.rs`, `tasks/ar-01-handoff-journal.md`.
Each is inside the handoff touch-set or a journal-declared deviation file
sanctioned by the plan's verbatim "(or new mod)" clause (plan AR-01 Touch line).
No undeclared path. `file-view.js` untouched and `server_api.rs` unchanged.

## 3. Security audit of the fixes

S1 (`api_source`, reviewed from the committed blob, not the dirty working
copy): canonicalizes BOTH ends, `contained_path.starts_with(&canonical_root)`
gate -> 403. Vectors closed: `..` (canonicalize resolves before the check);
absolute `file=` escaping the repo (`join` replaces, resolves outside -> 403);
escaping `local_prefix=` (containment applied to the constructed result -
plan's "sandbox prefix into repo" option); in-repo symlinks resolving outside
rejected. `Path::starts_with` is component-wise, so no string-prefix bypass
(`/repo-evil` cannot match `/repo`). Fail-closed on root canonicalize (500) and
404-preserving on candidate canonicalize (missing-file behavior unchanged).
No retained dual representation: the old uncontained `let path` binding is
replaced, not shadowed. All five S1 behaviors locked by tests that passed in
the reviewer's fresh run, including contained-read byte-identity and the
prefix==root mapping staying 200.

S3 (`export.rs`): `escape_json_for_script` replaces `<`, U+2028, U+2029 with
JSON escape sequences. Soundness independently verified: the three characters
cannot appear in JSON structural syntax (only in string values), and each
inserted escape body contains no subsequent matchable character, so the chained
single pass is total and parse-identical; round-trip equality and clean-payload
byte-identity are locked by passing tests.

S4-partial (`hotspots.js`): `escapeAttribute` (& first, then < > " ') shipped
and wired at the two payload-derived attributes `data-preview-root` /
`data-preview-source`; ships-in-asset lock passed.

## 4. Deviation dispositions

- D1 (locks in new mod `source_containment.rs` + 1-line `main.rs` registration):
  ACCEPTED. Ground verified independently: `server_api.rs` measured 1000 lines
  with the dirty +58 AR-02 WIP, so 942 at commit is arithmetically confirmed;
  ~130 lock lines would have made it 1072 > 1000 (R2 blocker). The plan names
  "(or new mod)" verbatim, and the one-line `mod source_containment;` is its
  mechanical consequence. New mod measured 223 lines (journal's "~130" is an
  understatement of produced lines, immaterial).
- D2 (hygiene exit 1 pre-existing, zero new): ACCEPTED, verified below.
- D3 (`ui_smoke` via distiller): ACCEPTED - not a deviation; the handoff's own
  Constraints line mandates distilling entry points for all gates.
- `file-view.js` "not applicable" judgment: CONFIRMED independently, not
  rubber-stamped. Its repo-derived values reach the DOM only via `textContent`,
  `escapeHtml` text nodes, `encodeURIComponent` (URL context), or DOM
  `setAttribute` (no HTML parse context); there is no HTML-string attribute
  interpolation for the helper to guard. Quote-extension is AR-11 per plan.

## 5. Hygiene baseline claim - VERIFIED (with a documented method limit)

The literal instruction (run `python tools/hygiene_check.py` at `6d7f96b~1`)
is not executable under the reviewer's read-only-git boundary (no checkout of
another revision; `hygiene_check.py` takes only a directory root, no `--rev`).
Equivalent verification performed instead:
(a) fresh full run: violation set is exactly the two pre-existing server.rs
entries; no AR-01 file appears in R1/R2/R3;
(b) complete commit diff reviewed line by line: the only functions touched or
created (`api_source` ~35 lines, `export_standalone_html`,
`escape_json_for_script` ~7, `escapeAttribute`, all test fns) are far under
100; `source_containment.rs` is 223 lines and `main.rs` +1, both under 1000;
new R1/R2 entries are therefore impossible;
(c) parent numbers reconcile across independent sources: plan census "hygiene
FAIL exit 1 (R1x1, R2x1)" and "server.rs (1120)"; journal "1120 -> 1137";
diff net +17; current 1155 = 1137 + 18 (AR-02 dirty +20/-2). The R1 entry
`api_graph_traversal` 161 lines is byte-untouched by the commit.
Conclusion: R1/R2 set identical before/after; only pre-existing R3 warn line
numbers shift. Journal claim true.

## 6. TDD discipline - VERIFIED (as far as read-only permits)

RED `237 passed / 5 failed / 0 ignored / 2 suites`, GREEN `242 passed / 0 failed
/ 0 ignored / 3 suites`. The 5 predicted semantic failures are exactly the 5
of 8 new tests that must fail against the vulnerable code (3 S1 escapes served
the planted secrets; export carried raw `<`; helper absent); the other 3
(contained byte-identity, prefix==root mapping, clean-payload rider) are
before-and-after invariants and correctly passed at RED - the 5/3 split is
what a true red-first run necessarily produces. The 2-vs-3 suites discrepancy
reconciles against `testrun.py`: suites count `Running`+`Doc-tests` headers,
`--tests` is an allowed passthrough flag (excludes doc-tests -> 2), so a
`--tests`-scoped RED run at 242 total is arithmetically consistent. The RED
run itself cannot be re-executed by a read-only reviewer (would require
reverting the fix); accepted on journal evidence plus the failure-identity
derivation above.

## 7. Findings

1. MINOR (plan-literal, disposition: accepted on merits, no rework): the
   handoff line "Reject absolute `file=` paths" reads unconditionally, but the
   implementation rejects only absolute paths that ESCAPE the repo; a contained
   absolute `file=` (path resolving under the canonical root) is served 200.
   Security-equivalent (the invariant actually locked - "starts_with must
   reject any path that escapes repo" - holds), it follows the plan's
   "(or sandbox prefix into repo)" alternative, and blanket rejection would
   break the documented cross-environment mapping shape. It was not listed as
   a journal deviation, and the journal's phrasing ("Absolute file= values ...
   are rejected") could mislead a later reader into assuming blanket reject.
   Recorded here; AR-13's verbatim-move review must not "fix" it into a
   stricter or looser shape.
2. MINOR (report evidence): journal gate line "encoding_guard.py --staged ->
   see commit transcript" carries no digest line and the transcript is not in
   the artifact set; the RED command form (`--tests` scoping) is not recorded.
   Both gaps are covered by the reviewer's own fresh `--all` exit 0 and the
   suite-count reconciliation, but per the distillation evidence rule future
   journals must self-quote the line.
3. STATE (not an AR-01 defect; coordinator action): the shared worktree holds
   uncommitted WIP of the CANCELLED AR-02 session (cli `--expose` 7th param,
   server.rs same-origin CORS, 3 CORS tests, cli/lib.rs). It breaks the
   workspace test build (`error[E0061]` in committed `m16_traversal_test` /
   `m17_dir_nav_test`), so no workspace census/`--baseline` gate can run until
   it is completed (with m16/m17 caller updates) or reverted. Noted side
   effect: dirty `server_api.rs` now sits at exactly 1000 lines - one added
   line trips R2; AR-02 must split before adding.
4. INFO: pre-existing server.rs R2 is now 1155 (1137 at 6d7f96b + AR-02 WIP);
   AR-13 owns clearing it; hygiene exit-0 gate remains plan-known-red until
   then (plan census says FAIL exit 1 at dispatch).

None of the findings is a blocker; the two MINORs require no code change to
6d7f96b.

## 8. Reviewer environment notes (gate coverage disclosure)

Ran: scoped testrun --distill; distill-wrapped ui_smoke; encoding_guard --all;
hygiene_check (current tree, full analysis for parent-equivalence); tools
unittest discover; full workspace census (blocked by foreign WIP, quoted above)
and fallback census. Read-only git used: log, diff, show (revisions quoted
whole). Bash probes refused by the global catch-all
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`:
`python -c` blob extraction, `encoding_guard --all; echo $LASTEXITCODE` exit
probe, `git status --short`, `git grep`. Every expected gate was still run;
encoding exit-0 established from tool source instead of an exit-code echo.
