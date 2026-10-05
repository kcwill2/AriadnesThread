# T0-report verdict - M27 tu-index-hang-throughput (reviewer gate)

Verdict: **APPROVED**

Hop: `tasks/tu-index-hang-throughput-t0-report-handoff.md` (doc-only).
Commit under review: `62ce72b` (parent `b0f89f8`), subject byte-identical to
the handoff gate-3 `-m` string. Journal:
`tasks/tu-index-hang-throughput-t0-report-journal.md`.
Review date 2026-10-05. Full gate evidence:
`tasks/tu-index-hang-throughput-execution-results.md`.

## Touch-set and diff scope (exact PASS)

- `git diff --name-only b0f89f8..62ce72b` = exactly the two declared touch-set
  files; `git diff --name-status` = `A`/`A` (both pure additions; Red
  equivalent - neither tracked at `b0f89f8`).
- `git show --stat 62ce72b` = `tasks/tu-index-hang-throughput-t0-evidence.md |
  200 +++`, `tasks/tu-index-hang-throughput-t0-report-journal.md | 84 +++`,
  `2 files changed, 284 insertions(+)`, zero deletions.
- Zero mutation outside `tasks/`: no path under `crates/`, `tools/`, `tests/`,
  goldens, fixtures, or `progress.rs` appears in the diff. Ledger NOT touched
  by the commit (its uncommitted 2-line edit belongs to the planner's
  in-flight work, not this hop).

## Plan-literal and citation checks (all PASS)

- All 13 mandatory grep-present strings land in the committed doc;
  reviewer re-grepped the high-risk set with ripgrep (literal):
  E3 line (line 32, single line), `RELOADED, not rebuilt` (:188),
  `NOT outstanding work` (:118), `binary search` (:137), `2.0.0 -> 2.1.0`
  (:178, :192).
- E3 verbatim: report line 32 is byte-identical to shape sec. 0 line 74
  (`thread 'tokio-rt-worker' (55408) panicked at
  crates\lang-cpp\src\tokenizer.rs:331:21: attempt to subtract with
  overflow`); E1/E2/E3 datetimes (04:11Z/04:17Z/04:22Z, graph 04:21, 25/31 GB)
  match shape lines 67-76.
- Live-tree citations re-checked line by line: `tokenizer.rs:49` =
  `Vec<char>` scan; `:311` = `column: column - word.len(),`; `:331` =
  `column: column - punct.len(),` (col 21 = the subtraction expression -
  the E3 site); `:307`/`:327` = `chars().count()`; `:313`/`:333` =
  `end: start + len`. `ui.rs:786-805` = `spawn_blocking` with no
  `catch_unwind`, JoinHandle discarded (statement-form call at :786, block
  closes at :805). Watcher contrast site `ui.rs:635-676`: `let _handle` +
  `catch_unwind` at :636. M25 cite `parser.rs:61` = brace-depth floor guard.
  DP-6 literal: `crates/index/src/cache.rs:16` `CURRENT_ANALYSIS_VERSION =
  "2.0.0"` - the `2.0.0 -> 2.1.0` bump rides T2 as stated. Offender-TU rule
  matches `crates/pipeline/src/lib.rs:507` in-order single-threaded loop.
  Cited hashes `9ce140a`, `ad65747`, `af11113` all exist.
- Sections 1-9 present, in contract order, matching handoff obligation text
  and shape sec. 3.2; hypothesis ledger complete (H-A..H-E) with the (a)-(e)
  discriminators marked "NOT outstanding work"; DP-6 recorded RESOLVED YES,
  not re-opened.

## Hygiene / encoding (PASS)

- `python tools/hygiene_check.py` exit 0 (warn-only R3 rows, all in legacy
  sources this hop never touches).
- `python tools/encoding_guard.py --all` exit 0; `--staged` exit 0 (vacuous at
  review time - staging area clean; see finding 3 for the substituted
  load-bearing proof).
- Independent byte check by this reviewer (grep `[^\x00-\x7F]`): zero
  non-ASCII bytes in BOTH committed files; no line >= 124 chars (journal's
  max-123 claim held). This matters because `encoding_guard.py:36` allow-lists
  `tasks/` from its scan - the journal's disclosure of that exemption is
  accurate, and the byte grep is the real purity proof.

## Findings

1. **minor - journal gate-3 route is contradicted by the implementer
   report-back.** Journal gate 3 asserts the commit ran through
   `distill.py -- python tools/commit_scoped.py ...`; the report-back (relayed
   at dispatch) says the distill-wrapped form hit permission denials and the
   commit landed via direct `commit_scoped.py`, with the fallback absent from
   the journal's Deviations section. Disk is inconclusive:
   `target/distill-capture.txt` was overwritten post-commit (now holds
   porcelain-status text, no commit_scoped output). Outcome is machine-verified
   in-spec either way (two files, scoped, literal subject), so not a blocker.
   Required follow-up: one journal errata line recording the true route and
   the verbatim denial text, on the next authorized touch of the journal.
2. **minor - report line 22 "the background watcher at 636-673 DOES catch and
   log" is slightly generous.** The catch is real (`catch_unwind` at
   `ui.rs:636`); logging covers the `Err` arms (`:641`, `:669`), while a caught
   panic's outcome is itself discarded at `ui.rs:674` (`let _ = outcome;`).
   The load-bearing contrast - boot has no `catch_unwind`, watcher does - is
   accurate, and the point strengthens rather than weakens M26 I11's case.
   Optional errata; no change required for approval.
3. **info - reviewer-session gate reachability (config, not implementer
   fault).** Refused verbatim by the global catch-all
   `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`:
   `python -c` (both multi-line and single-line), PowerShell pipe forms
   (`| Out-Null`, `| findstr`), and `git ls-tree` (both forms). Ran fine:
   bare tool commands, bare `git status/log/show/diff/rev-parse`/ls-tree-free
   compounds with `;`. The byte-check duty was discharged via the Grep tool
   instead. Ledger line 59 errata recurs once more: literal
   `python tools/testrun.py --baseline` fails `usage error: missing value for
   --baseline` (FILE-value flag, no committed baseline file); census substitute
   ran via `--distill` per the standing precedent. No new deferred row needed;
   lines 59/60 already own these.

## Declared-deviation dispositions

- **D1 (gate-3 via direct `commit_scoped.py` after distill denial): accepted**
  on merits - the handoff's substantive prohibitions (touch-set scoping,
  never `git add .`) are honored and machine-proven; the wrapper is transport,
  not the guarantee. Journal fidelity follow-up is finding 1.
- **D2 (one `distill.py -- git add` precondition so gate 1's `--staged` was
  non-vacuous): accepted** - scoped-path staging, ledger line 60
  proven-sanctioned route, and it is what makes the mojibake gate meaningful.

## Workspace-wide gate exemption (explicit, per dispatch ruling)

This hop carries a proven zero-code diff (A/A doc additions only), so
`ui_smoke` and `bench_compare` measure byte-identical binaries as the start
rev: **exempted at this hop by explicit coordinator ruling; not run.**
Workspace gates the reviewer hop did owe were run: census (checkpoint
ground-truth), `--distill` census substitute, tools `unittest discover`,
hygiene, encoding.

## Follow-ups routed (report-only, no drops)

- Journal route errata line (finding 1) - next authorized hop.
- Ledger line 59 recurrence count +1 for the `--baseline` usage-error
  errata (planner-owned).
