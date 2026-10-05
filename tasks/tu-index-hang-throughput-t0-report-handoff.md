# T0-report Handoff - M27 tu-index-hang-throughput: land the diagnosis as a report doc

Milestone: `tu-index-hang-throughput` (M27, ledger row 2026-10-05). Doc-only
handoff. Sizing law held: 1 module dir (`tasks/`), 3 implementer gate
invocations (<= ~10), ~250-400 expected produced lines (<= ~1200).
Workspace-wide gates belong to the reviewer hop. T0's DIAGNOSIS is LANDED by
field evidence E3 (root cause CONFIRMED - see below); this hop's obligation is
BOOKKEEPING: fold the live evidence into the tracked report doc so no
load-bearing reasoning lives only in chat/console scrollback. No code in this
hop; the minimized synthetic RED repro is authored with T0b/T2, not here.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md`
   sec. 0 (verified facts + E1/E2/E3), sec. 3.2 (the T0 artifact contract -
   your doc's section list), sec. 6 (hypothesis ledger), sec. 7 (capture
   protocol - OPTIONAL), sec. 3.4 (data-integrity flag).
3. `tasks/milestone_ledger.md` rows: `cpp-brace-depth-underflow` (M25, the
   same-class precedent) and the two M26/M27 rows.
4. `crates/lang-cpp/src/tokenizer.rs` lines 40-53, 299-337 (cite, do not
   touch: `:49` `Vec<char>` scan, `:311` word byte-subtraction, `:331` punct
   byte-subtraction = the E3 panic site, `:313`/`:333` correct
   `chars().count()`).
5. `crates/cli/src/commands/ui.rs` lines 786-805 (the swallowed-panic boot
   hole M26 V1 closes - cross-reference only).

## Objective and invariants

Create `tasks/tu-index-hang-throughput-t0-evidence.md` (tracked) with these
MANDATORY sections, in this order (shape sec. 3.2 contract):

1. **Verdict (root cause CONFIRMED)**: the "hang" was a CONSEQUENCE - the
   boot worker died mid-run and the UI's boot hole (no `catch_unwind`,
   dropped JoinHandle at `ui.rs:786-805`) swallowed it into a frozen loading
   row. Cite E3 verbatim: `thread 'tokio-rt-worker' (55408) panicked at
   crates\lang-cpp\src\tokenizer.rs:331:21: attempt to subtract with
   overflow` (preserve the exact user-console text; if the operator's
   preserved scrollback differs in the thread id, use theirs and date it).
2. **Field evidence E1/E2/E3 (dated, as shape sec. 0)**: 30% at 90+ min
   (5744 TUs, ~3.1 s/TU); frozen 1780/5744, ~20% total CPU, server alive;
   panic self-report + per-core graph (8c/16t Ryzen 9800X3D, several cores
   active, NONE pinned = died-mid-run wind-down, not deadlock);
   overflow-checks ON proves the user's run was a DEBUG/dev-profile build -
   part of "forever" is profile, not algorithm (T1 quantifies).
3. **Verified code mechanism**: `tokenizer.rs:49` tokenizes over `Vec<char>`
   with `col` advancing per CHAR; `flush_word :311` computes
   `column - word.len()` and `flush_punct :331` computes
   `column - punct.len()` - BYTE length off a CHAR-counted column - while
   `end: start + len` (`:313`/`:333`) correctly uses `chars().count()`.
   Any non-ASCII bytes in a run near a line start underflow: DEBUG panics,
   RELEASE WRAPS SILENTLY to huge columns that flow downstream unchecked.
4. **M25 precedent**: same usize-underflow class (`parser.rs` brace-depth);
   the one-site guards landed and the class survived - basis for T2's
   BOTH-sites + audit obligation (cite ledger M25 row + the memory
   correction `m25.cpp_brace_depth_underflow_closed`).
5. **Hypothesis ledger (FINAL, post-E3)**: H-B CONFIRMED (root cause);
   H-A killed for this incident (no pinned core) but retained as the
   signature for future one-core stalls; H-C partially confirmed as a
   throughput contributor (DEBUG-priced; T1's sufficiency gate measures);
   H-D killed (trace supersedes); H-E killed twice (no Mutex, single
   thread). Include the reusable discriminators (a)-(e) as standing
   methodology, explicitly marked "for the NEXT stall - NOT outstanding
   work".
6. **Offender-TU identification rule** (feeds T0b): the single-threaded loop
   processes `build.translation_units` IN ORDER writing one shard per
   reanalyzed TU, so the offender is (first-try) the first TU in that order
   whose shard is absent/stale-fingerprinted, cross-checked as the successor
   of the newest-mtime shard; fallback = binary search by pruning TU sets in
   a repo copy until the PANIC (not a hang) reproduces at half size. The
   panic message carried no path - M26 I10 fixes that going forward.
7. **OPTIONAL capture protocol (demoted by E3; minimal)**: 5-step compact
   summary (liveness counters / shard mtime pair / dump+stacks via
   procdump+cdb with `qd` detach rule / scene record / kill-last), pointing
   to shape sec. 7 as the verbatim procedure; state plainly: nothing in this
   milestone waits on a dump, the panic self-reported.
8. **Decision list (what this report routes)**: T2 = BOTH flush sites +
   same-class usize audit over `tokenizer.rs`+`parser.rs` + release-profile
   characterization (one-site = plan-literal violation); T1 = RELEASE-profile
   measurement + launch-profile record + sufficiency gate; T0b = minimized
   fixture via the documented approval path; DP-6 = RESOLVED by the user
   2026-10-05: YES - `CURRENT_ANALYSIS_VERSION` 2.0.0 -> 2.1.0 rides the T2
   fix commit (cache auto-invalidation); M26 I11 stands independent of root
   cause (truth-surface).
9. **Data-integrity disposition (mandatory)**: artifacts/shards/index from
   RELEASE-profile runs over non-ASCII corpora (including the completed 30%)
   carry silently-wrapped columns and are RELOADED, not rebuilt, on
   fingerprint match post-fix - re-index affected corpora after T2 lands
   (DP-6 bump is the clean global invalidation); column-vs-line-length shard
   scan is a CANDIDATE detector only, not a commitment. ASCII-path
   tokenization bytes are NOT affected (the fix corrects non-ASCII column
   values only; m00/m18 locks prove it).

Invariants: ZERO source/test change (this hop); the doc quotes E3 exactly;
authored content pure ASCII EXCEPT the single verbatim panic-trace line,
which is pure ASCII anyway (backslashes + digits); no raw dump pastes (R9 -
text excerpts only).

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/tu-index-hang-throughput-t0-report-journal.md` (journal; always
  in-set)
- `tasks/tu-index-hang-throughput-t0-evidence.md` (NEW - the report)

You MUST NOT touch: ANY file under `crates/`, `tools/`, `tests/`; the shape/
kickoff plan files (read-only); `tasks/milestone_ledger.md` (planner-owned;
if you believe a row needs a fix, put the proposed line in your report-back
section, do not edit). No `git stash`, no branch change.

## Red equivalent (doc handoffs have no failing test)
Coverage checklist = the machine-checkable Red/Green: before authoring, no
tracked file contains section headers 1-9 above; the reviewer greps the
landed doc for the mandatory content list below and REJECTs on any miss.
Mandatory grep-present strings (exact, ASCII):
`subtract with overflow`; `tokenizer.rs:331`; `tokenizer.rs:311`;
`chars().count()`; `WRAPS SILENTLY`; `M25`; `SUFFICIENCY`;
`DP-6`; `2.0.0 -> 2.1.0`; `RELOADED, not rebuilt`; `binary search`;
`NOT outstanding work`; `DEBUG/dev-profile`.

## Hygiene (review-enforced)
No source lines: `python tools/hygiene_check.py` exit 0 anyway (scans repo);
authored doc content pure ASCII; every doc line well under 2000 chars (the
repo hazard AGENTS.md names - this handoff's own files obey it).

## Frozen locks (each with its proving command)
| Lock | Proof |
|---|---|
| zero code change | `git diff --stat <start-rev>..HEAD` lists the two `tasks/` files only |
| goldens/fixtures/progress.rs untouched | same diff-scope proof |
| mojibake gate | `python tools/encoding_guard.py --staged` exit 0 pre-commit |

## Gates (exact commands, timeout 600000 ms each)
Distilling entry points only (ledger row 58). Workspace-wide gates belong to
the reviewer hop.
1. `python tools/encoding_guard.py --staged` - exit 0 before committing.
2. `python tools/hygiene_check.py` - exit 0.
3. Commit: `python tools/distill.py -- python tools/commit_scoped.py --scope
   tasks/tu-index-hang-throughput-t0-evidence.md --scope
   tasks/tu-index-hang-throughput-t0-report-journal.md -m "docs(tu-index): T0
   hang-consequence diagnosis report from E1-E3 field evidence
   (M27 T0-report)"` (never `git add .`).

Total implementer gate invocations: 3 (<= 10).

## Acceptance (machine-checkable)
Implementer hop: 1. landing commit `git show --stat` = the two `tasks/`
files; 2. encoding + hygiene exit 0; 3. the mandatory grep-present strings
list above appears in the doc (self-check with the Read tool; reviewer
re-greps).
Reviewer hop: the grep list against the committed file; cross-check every
code citation (`tokenizer.rs:49/:311/:313/:331/:333`, `ui.rs:786-805`)
against the tree; verdict on whether sections 1-9 are complete. This doc
LANDS BEFORE T1a/T1b/T2 dispatch (shape sec. 1 ordering).

## Discipline clauses
- UTF-8: Write/Edit tools only; pure-ASCII content; never quote git blobs
  through PowerShell pipes (AGENTS.md).
- Touch only the touch-set; out-of-scope: report, do not fix.
- Journal checkpoint after the commit.
- If context exhaustion looms: STOP, disk-state inventory; successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/tu-index-hang-throughput-t0-report-handoff.md --scope ... --cmd ...`.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. The grep-present checklist
self-verified. 3. Deviations + reasons. 4. Any proposed ledger wording
(report-only; planner is sole ledger writer). Never paste raw tool output.

## Reviewer-gate reference
Adjudicated via the `review-gate` skill. Extra reviewer checks: E3 line
verbatim; mechanism section matches the live `tokenizer.rs` byte-for-byte on
the cited lines; hypothesis ledger complete (H-A..H-E); DP-6 recorded as
RESOLVED YES (do not accept a doc that re-opens it).

## Sequencing note
Parallel-safe: dispatchable immediately, disjoint from M26 V1/V2 and M27 T0b.
GATES (dispatch ordering): T1a, T1b, T2 wait for this doc to land.
