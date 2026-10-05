# T0 evidence report - M27 tu-index-hang-throughput: the "hang" was a CONSEQUENCE

Hop: `tasks/tu-index-hang-throughput-t0-report-handoff.md` (T0-report, doc-only).
Milestone: `tu-index-hang-throughput` (M27, ledger row 2026-10-05), retitled per
the shape: the hang was a CONSEQUENCE of a swallowed panic, never its own bug.

Governing inputs (read, not restated): shape
`.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md` rework 3
(sec. 0 verified facts + E1/E2/E3, sec. 3.2 this doc's contract, sec. 3.4 fix
seam + data-integrity flag, sec. 6 hypothesis ledger, sec. 7 capture protocol),
`tasks/milestone_ledger.md` rows M25 `cpp-brace-depth-underflow`,
M26 `boot-progress-visibility`, M27 `tu-index-hang-throughput`.
Code citations were re-verified against the live tree on 2026-10-05; this hop
changes zero source (cites only).

## 1. Verdict (root cause CONFIRMED)

The "hang" was a CONSEQUENCE, not a bug in itself.
The boot worker died mid-run from a panic, and the UI boot hole swallowed that
death into a frozen loading row: the boot `tokio::task::spawn_blocking`
(`crates/cli/src/commands/ui.rs:786-805`) has NO `catch_unwind` and drops the
JoinHandle (the background watcher at 636-673 DOES catch and log - the boot
path lacks that precedent's protection).
A panic escaping the build closure therefore leaves: progress frozen at the
last frame, state stuck `"loading"`, the error arm never reached, the watcher
never started, and the server serving forever with zero build CPU.
Today a dead boot is INDISTINGUISHABLE from a hung boot.

The specimen self-reported its death - E3 verbatim from the operator's console,
2026-10-05T04:22Z:

    thread 'tokio-rt-worker' (55408) panicked at crates\lang-cpp\src\tokenizer.rs:331:21: attempt to subtract with overflow

The thread name is consistent with the boot's blocking-pool task thread
(blocking-pool threads carry the runtime's worker name), and the frozen counter
fits a panic escaping the build closure into the verified swallow above.
The `subtract with overflow` panic text is preserved exactly; the operator's
preserved scrollback agrees with the shape sec. 0 and handoff records of this
line (same thread id 55408, same site).
M26 V1/I11 closes this boot hole; its necessity is PROVEN by this incident,
independent of the arithmetic root cause fixed in T2.

## 2. Field evidence E1/E2/E3 (dated, per shape sec. 0)

- E1 2026-10-05T04:11Z: stage 2 at ~30% after 90+ minutes on the large corpus
  (5744 TUs); counter consistent with ~1723/5744 done; mean ~3.1 s/TU
  single-threaded; projection was a multi-hour cold boot.
- E2 2026-10-05T04:17Z (ESCALATION): counter frozen at 1780/5744 for several
  minutes; total CPU ~20%; RAM free; the UI server stayed alive with no build
  CPU.
- E3 2026-10-05T04:22Z (ROOT CAUSE, live specimen self-report): the panic line
  quoted verbatim in sec. 1 above, plus the per-core Task Manager graph at
  04:21Z on the 8-core/16-thread Ryzen 9800X3D (~20% total, 25/31 GB RAM used):
  several cores active, NONE PINNED.
  That is the "worker died mid-run, siblings wind down, frozen counter"
  signature - not a mutual deadlock.

Profile read: the panic itself proves overflow-checks were ON, so the user's
run was a DEBUG/dev-profile build.
Part of "indexing takes forever" is therefore profile, not algorithm; T1's
SUFFICIENCY gate quantifies what the release profile actually costs before any
parallelism machinery is considered.

## 3. Verified code mechanism (cites re-verified 2026-10-05; no source touched)

`crates/lang-cpp/src/tokenizer.rs:49` tokenizes over a `Vec<char>` with `col`
advancing per CHAR.
`flush_word` at `tokenizer.rs:311` computes `column: column - word.len()` and
`flush_punct` at `tokenizer.rs:331` computes `column: column - punct.len()` -
each subtracting the BYTE length of the run from a CHAR-counted column - while
the adjacent `end: start + len` (`:313` / `:333`) correctly uses
`word.chars().count()` / `punct.chars().count()` (`:307` / `:327`).
The unit inconsistency is internal to the file: same value, two units.
Any non-ASCII bytes in a word or punct run near a line start make
`column < byte_len`, and the subtraction underflows the `usize` column.
In DEBUG (overflow-checks ON) it PANICS - `tokenizer.rs:331:21` is the E3
panic site, and `tokenizer.rs:311` is the same fault one flush earlier.
In RELEASE it WRAPS SILENTLY to huge columns that flow downstream unchecked,
with no panic, no log, and no error signal.

## 4. M25 precedent (same underflow class; why one-site fixes are forbidden)

M25 `cpp-brace-depth-underflow` (ledger row, CLOSED 2026-10-04 at `af11113`)
is the same usize-underflow class: brace-depth subtraction in
`crates/lang-cpp/src/parser.rs`, fixed by spec `9ce140a` + fix `ad65747` +
round-2 `+1` guard `af11113` after a one-site REJECTION round.
The one-site guards LANDED and the class SURVIVED: the 2026-10-05 kilo-memory
correction `m25.cpp_brace_depth_underflow_closed` records that the fix or its
guard "does not fully cover subtraction underflow" at
`crates\lang-cpp\src\parser.rs:61` during TU indexing, with a runtime panic
observed on 2026-10-05 while running the UI.
That precedent is the direct basis for T2's obligation shape: fix BOTH flush
sites in one commit and run a same-class usize audit over the position math in
`tokenizer.rs` + `parser.rs`; a one-site fix here repeats M25's documented
failure mode and is a plan-literal violation.

## 5. Hypothesis ledger (FINAL, post-E3, evidence-decided)

- H-B CONFIRMED = ROOT CAUSE: the boot task DIED from a swallowed panic at
  `tokenizer.rs:331:21` (E3) while siblings wound down; the per-core graph
  (several cores active, NONE pinned) matches died-mid-run and excludes mutual
  deadlock. The arithmetic bug is T2's confirmed content; the swallow that
  made it LOOK like a hang is M26 I11's confirmed necessity.
- H-A (one pinned core; pathological TU traps the parser thread): KILLED for
  this incident - no pinned core, the thread no longer existed. RETAINED as
  the standing signature for a FUTURE one-core stall; the parser-pathological
  class is still live in the wild (M25 history; T2's audit narrows it).
- H-C (genuinely slow): PARTIALLY CONFIRMED as a throughput contributor, not a
  hang. E1's ~3.1 s/TU was measured in DEBUG (E3 proves overflow-checks ON),
  inflating it an unknown factor; whether release+single-thread suffices is
  exactly what T1's SUFFICIENCY gate measures before any DP-3 dispatch.
- H-D (blocking I/O / Defender storm): KILLED for this incident - the trace
  supersedes; the technique is retained for future zero-CPU stalls.
- H-E (Mutex convoy): KILLED twice over - there is NO Mutex on the UI-boot
  stage-2 path, AND the path is single-threaded.

Reusable discriminators (shape sec. 6) - standing methodology for the NEXT
stall, NOT outstanding work for this one:
(a) per-logical-core CPU graph - one pinned core vs all idle;
(b) counter-advance poll - the row sampled twice, 5 minutes apart;
(c) shard-write liveness - newest mtime + count of `artifacts\shards` sampled
twice; every reanalyzed TU writes a shard, a free progress oracle; on warm
runs shard COUNT exceeds `done` (reused TUs count done without rewriting), so
only MTIME liveness discriminates;
(d) thread stacks + `!runaway` CPU deltas (shape sec. 7);
(e) console/stderr capture - E3 proves the default panic hook is today's ONLY
error channel, which M26 I11 replaces.

## 6. Offender-TU identification rule (feeds T0b)

The single-threaded boot loop processes `build.translation_units` IN ORDER and
writes one shard per reanalyzed TU, so the offender is deterministically
locatable:
- First try: the first TU in that order whose shard file is absent or
  stale-fingerprinted, cross-checked as the SUCCESSOR of the newest-mtime
  shard in `artifacts\shards`.
- Fallback: binary search by pruning TU sets in a repo copy until the PANIC
  reproduces at half size (a panic, not a hang - E3 changed the target
  signal).
The E3 panic message itself carried no file path; M26 I10 (`TuStarted`
path-in-message) fixes that going forward, so the next mid-stage death names
its own unit.
T0b then extracts the minimized corpus from the identified TU as the approved
regression fixture.

## 7. OPTIONAL capture protocol (demoted by E3; minimal summary)

Nothing in this milestone waits on a dump: the panic self-reported its
file:line, so the PR-0 specimen emergency is CLOSED.
Shape sec. 7 stays the verbatim standing procedure for a future stall whose
thread does NOT self-report. Compact summary of its 5 steps:
1. Liveness counters: total CPU sampler + per-core graph; save text output.
2. Shard mtime pair: shard count + newest-mtime listing, two samples 5 min
   apart.
3. Dump + stacks BEFORE any kill: `procdump -ma` then offline
   `cdb ... !runaway; ~*e k; q`, or live attach `cdb -p` whose detach rule is
   `qd` - NEVER `q` while attached.
4. Scene record: last visible count, exact command line, corpus TU count,
   shard listings, artifacts tree, OS/Defender state, stack outputs.
5. Kill LAST, only after steps 1-4 artifacts exist; keep the dump outside the
   corpus, text excerpts only in journals (R9 - no raw dumps).

## 8. Decision list (what this report routes)

- T2 = fix BOTH flush sites (`tokenizer.rs:311` AND `tokenizer.rs:331` - a
  fix touching one is a plan-literal violation) + the same-class usize audit
  over `tokenizer.rs` + `parser.rs` position math (M25 precedent, sec. 4) +
  release-profile characterization that pins the silent-wrap path so wraps are
  mathematically impossible, not merely undetected.
- T1 = RELEASE-profile measurement + launch-profile record (how the app is
  NORMALLY built - AGENTS.md build lines, `ui_smoke.py` build step, run
  scripts) + the SUFFICIENCY gate: release+single-thread at corpus scale
  against the ratified wall-time bar; T3 dispatches ONLY on a written gate
  FAIL.
- T0b = minimized fixture (non-ASCII in a word run AND in a punct run) via the
  documented frozen-set approval path, sibling dir only.
- DP-6 = RESOLVED by the user 2026-10-05: YES - `CURRENT_ANALYSIS_VERSION`
  2.0.0 -> 2.1.0 rides the T2 fix commit as automatic cache invalidation.
- M26 I11 (dead-boot truth / boot error bridge) stands INDEPENDENT of the root
  cause fix: it is the truth-surface lesson of this incident - E3 proves a
  frozen loading row must be made impossible regardless of which bug kills
  the worker.

## 9. Data-integrity disposition (mandatory)

Artifacts, shards, and index from RELEASE-profile runs over non-ASCII corpora
- including E1's completed 30% - carry silently-wrapped columns and are
RELOADED, not rebuilt, on fingerprint match after the fix lands, because the
fingerprint chain hashes file CONTENT, not analyzer CODE, and will not
self-invalidate.
Disposition: re-index every affected corpus after T2 lands; the DP-6
`CURRENT_ANALYSIS_VERSION` 2.0.0 -> 2.1.0 bump is the clean global
invalidation that makes the reload impossible without a rebuild.
The column-vs-line-length shard JSON scan (wraps are huge, so detection is
cheap) is a CANDIDATE detector for validating existing caches only - NOT a
commitment.
ASCII-path tokenization bytes are NOT affected and need no re-index for
correctness: the fix corrects non-ASCII column values only; the m00/m18 parity
locks (T2 gate G2-3, UNEDITED) prove pure-ASCII corpora keep byte-identical
output before and after.
