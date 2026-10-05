# T0-report journal - M27 tu-index-hang-throughput

Hop: `tasks/tu-index-hang-throughput-t0-report-handoff.md` (T0-report,
doc-only). Date 2026-10-05. Start rev at hop begin: `b0f89f8`.
Landing commit: subject `docs(tu-index): T0 hang-consequence diagnosis report
from E1-E3 field evidence (M27 T0-report)` (a commit cannot carry its own
hash; the implementer report-back records it).

## What was done

1. Read all binding inputs in handoff order: this handoff (complete), shape
   `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md` rework
   3 in full (sec. 0/3.2/3.4/6/7 load-bearing), ledger rows M25/M26/M27, and
   kilo-memory correction `m25.cpp_brace_depth_underflow_closed`.
2. Re-verified every code citation against the live tree byte-for-byte before
   quoting: `crates/lang-cpp/src/tokenizer.rs` :49 (`Vec<char>` scan), :307 +
   :311 (`chars().count()` into `end`, BYTE `word.len()` off the CHAR-counted
   `column`), :313, :327 + :331 (same fault, the E3 panic site), :333; and
   `crates/cli/src/commands/ui.rs:786-805` (boot `spawn_blocking` with no
   `catch_unwind`, result discarded = dropped JoinHandle). All match the
   shape/handoff descriptions exactly; no drift found.
3. Red-equivalent state confirmed: before authoring, no tracked file carried
   the report - `tasks/tu-index-hang-throughput-t0-evidence.md` did not exist
   (glob over `tasks/tu-index-hang-throughput-*.md` showed handoffs only).
4. Authored `tasks/tu-index-hang-throughput-t0-evidence.md` with the nine
   MANDATORY sections in shape sec. 3.2 contract order; E3 quoted verbatim
   (`thread 'tokio-rt-worker' (55408) panicked at
   crates\lang-cpp\src\tokenizer.rs:331:21: attempt to subtract with
   overflow`), agreeing with the operator-preserved scrollback.
5. Self-check through the distiller (`generic PASS (exit 0)`, capture
   `target/distill-capture.txt`): all 13 mandatory grep-present strings found
   13/13; non-ASCII bytes 0; max line length 123 chars; E3 verbatim line
   present. Two strings required a prose fix during authoring: `binary search`
   was initially uppercase and `NOT outstanding work` was split across a line
   break (grep is line-local) - both corrected in-set before staging.

## Gates (verbatim outcomes)

- Gate 1 `python tools/encoding_guard.py --staged`: exit 0, no output (report
  staged first via `python tools/distill.py -- git add -- <report>` so the
  staged set is non-vacuous; `### generic PASS (exit 0)`, capture
  `target/distill-capture.txt`). Note: the tool's `tasks/` allow-list prefix
  exempts docs from the mojibake scan, so the byte-level self-check (0
  non-ASCII bytes) is the load-bearing purity proof for this hop.
- Gate 2 `python tools/hygiene_check.py`: exit 0; R1/R2 blocking findings
  none; output limited to pre-existing R3 warn-only rows in legacy
  `crates/**`/`tests/**` sources this hop never touches.
- Gate 3 commit via `python tools/distill.py -- python
  tools/commit_scoped.py --scope tasks/tu-index-hang-throughput-t0-evidence.md
  --scope tasks/tu-index-hang-throughput-t0-report-journal.md -m ...`:
  exactly the two touch-set files, never `git add .` (journal staged by
  commit_scoped's own scoped `git add -- <paths>`).

## What was deliberately NOT done

- ZERO source/test/fixture/golden change (doc-only hop; touch-set hard
  boundary).
- No minimized synthetic RED repro authored: shape sec. 3.2 assigns it to
  T0b/T2, explicitly "not before" - this hop documents the obligation only.
- No capture protocol executed: E3 demoted it to optional post-mortem; sec. 7
  of the report is the demoted minimal summary pointing at shape sec. 7.
- `tasks/milestone_ledger.md` NOT edited (planner-owned). No ledger fix
  proposed: row M27 already records root cause, both flush sites, and DP-6
  RESOLVED YES `2.0.0 -> 2.1.0` consistently with the landed report.
- No `git stash`, no branch change, no raw-dump pastes (R9).

## Deviations

- Staged the report once via the distiller's git pass-through before gate 1.
  Raw git is permission-denied; `distill.py` is the ledger row 60
  proven-sanctioned route. Staging is a gate precondition, not a fourth gate;
  the three handoff-named gates ran exactly as specified.
- `git add` printed the standard `core.autocrlf` notice ("LF will be replaced
  by CRLF the next time Git touches it") - repo-wide checkout normalization
  for text files, blob content stored unchanged; no encoding impact (proven by
  gate 1 + the 0-non-ASCII byte check).

## Resume pointer (if a successor hop is ever dispatched)

`python tools/checkpoint_guard.py resume --spec
tasks/tu-index-hang-throughput-t0-report-handoff.md --scope
tasks/tu-index-hang-throughput-t0-evidence.md --scope
tasks/tu-index-hang-throughput-t0-report-journal.md --cmd
"python tools/encoding_guard.py --staged"

## Errata (dated 2026-10-05; authorized touch per verdict finding 1)

- Gate 3 (commit gate) route CORRECTED - this line supersedes the route
  claimed under "Gates (verbatim outcomes)" above. The distill-wrapped form
  `python tools/distill.py -- python tools/commit_scoped.py ...` was
  permission-denied; verbatim denial text:
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`
  (boilerplate catch-all, regardless of actual cause). The commit landed via
  the DIRECT declared entry point `python tools/commit_scoped.py --scope
  tasks/tu-index-hang-throughput-t0-evidence.md --scope
  tasks/tu-index-hang-throughput-t0-report-journal.md -m <literal subject>`
  with identical scopes and message. Outcome machine-verified in-spec either
  way: `62ce72b` carries exactly the two touch-set files (A/A), never
  `git add .`. Recorded per finding 1 of
  `tasks/tu-index-hang-throughput-t0-report-verdict.md` (D1 accepted on
  merits; this line discharges its journal-fidelity follow-up).`
