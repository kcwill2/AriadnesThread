# SM-1 Verdict - extraction-symbol-move - APPROVED (re-gate, post-incident)

Reviewer: SM-1 re-gate hop, 2026-10-24. Anti-phantom protocol: every number
below is this reviewer's own fresh measurement, quoted verbatim from own tool
output. Prior-lineage verdicts (`4e5b0b7`/`31e4e86` lineage) are void and were
used as zero evidence.

## Verdict: APPROVED

No blockers. Three minor findings (M1-M3); M1 must be corrected before the
uncommitted ledger incident row is landed (planner/closeout duty, not the
implementer's).

## Anti-phantom verification (reflog ground truth)

`.git/logs/HEAD` total 439 lines; line 439 verbatim:

`e5c49ec6a4891d195d0cbbf14c1ca271a15a1117 b9936000da7827e49e8dbe999f1dd65d65b75704 kcwill2 <kcwill2@gmail.com> 1792819353 -0700	commit: feat(lang): add the neutral move_out_top_level_symbols contract with a delegating default (extraction-symbol-move SM-1)`

The landing SHA exists; the phantom SHAs `4e5b0b7`/`31e4e86` do not appear
anywhere in the reflog tail (lines 430-439 reviewed; entries 437-438 are the
kickoff/plan docs commits, 436 the predecessor closeout).

## Commit-vs-touch-set (the enforcement check)

Range `e5c49ec..b993600` contains exactly ONE commit (reflog adjacency +
`git show` parentage). `python tools/distill.py -- git diff --name-only e5c49ec b993600`
returned exactly:

- `crates/lang/src/rewriter.rs`
- `tasks/extraction-symbol-move-sm1-journal.md`

Both inside the declared touch-set. Everything else in the repo is byte-
identical across the range: `crates/lang-scala/**`, `crates/lang-cpp/**`
(incl. the CW-1 `consumer_rewrite_spec` hook), `IncludeRewriter`
(`crates/transform/src/placement.rs`), `plan_target_consumers`
(`crates/cli/src/commands/transform.rs`), `crates/validate/**`, `crates/cli/**`,
`crates/ui/**`, all other `crates/lang` files, goldens/fixtures/themes/tools,
`tests/**` - all absent from the range diff. `Cargo.lock` absent. Working tree
holds the commit: checkpoint_guard dirty set = `.vsix` stray + the uncommitted
planner ledger row; rewriter.rs and the journal are neither dirty nor untracked.

## Per-commit review (the only hand commit, oldest-first trivially satisfied)

`git show b993600` judged alone:

- Diff confined to three regions exactly as the reviewer-gate reference
  requires: (a) the doc block above `remove_top_level_symbols` (:94-98 ->
  :94-101), (b) the appended hook method (:106-125, before `render_diff`),
  (c) appended `mod tests` items only (old test content appears as diff
  context - untouched).
- Hook code matches the handoff contract verbatim (signature, body
  `self.remove_top_level_symbols(source, qualified_names)`, `_plan_id`
  underscore binding per handoff :98-99, placement directly after
  `remove_top_level_symbols`). Doc text matches the handoff's quoted block
  line-for-line.
- Doc-lift truth check (kickoff sec 7 item 1): default still `None` stated;
  engine-never-removes-for-non-opt-ins stated; retention decision moved one
  level up with delegating default; Scala deletion / C++ marked comment-out
  named; D-M23-5 reference kept; new bytes pure ASCII (em dashes replaced by
  hyphens in the NEW doc lines only).
- `fn remove_top_level_symbols(&self, _source: &str, _qualified_names: &[String]) -> Option<String> {`
  and its `None` body: byte-identical UNEDITED (diff context).
- The seven pre-existing `#[test]`s + `InertRewriter` + `RewriteSpec` +
  `DirectiveHit` + all trait defaults: byte-identical (absent from hunks).
- Fake + pins match the handoff: `DeletingRewriter` (identity `rewrite`,
  `remove` returns `Some(source.replace("class Drop\n", ""))`), test names
  exactly `default_move_out_delegates_byte_identically_to_remove_hook` and
  `default_move_out_refuses_exactly_where_the_remove_hook_refuses`, messages
  naming SM-I5 and the copy fallback.
- Journal rides in the commit (210 lines, `210 0` numstat).

## Pin quality (independent falsifiability - not tautological)

- Delegation pin compares two LIVE calls (`move_out...` vs `remove...`) on a
  fake whose remove-hook performs a VISIBLE byte transform; it fails if the
  default returns `None`, returns `Some(source)` unmodified, returns foreign
  text, or mis-forwards `source`. The assertion does not restate the default's
  implementation expression - it is an observational equality over a
  non-identity transform, not a copy of the implementation text.
- Refusal pin uses `InertRewriter` (remove default `None`) and fails if the
  new default ever returns `Some` - it falsifies the refusal path
  independently of the first pin.
- The old copy-semantics pin
  `remove_top_level_symbols_default_is_none_copy_semantics_preserved` is
  present UNEDITED with its historical em dash intact (:190), green in my
  fresh runs.
- Limitation recorded as M2 below: `qualified_names` forwarding is not
  observable through the contract-specified fake (it ignores the arg).

## Fresh gates - this reviewer's own runs (verbatim)

- Bootstrap: `python tools/checkpoint_guard.py status` - HEAD `b993600` on
  `main`; dirty = `.vsix` (pre-existing) + `tasks/milestone_ledger.md`
  (uncommitted planner incident row); untracked = the resume file; worktree
  census as expected.
- Scoped lang: `python tools/testrun.py --distill -p ariadne-lang` ->
  `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 61 lines]` /
  `47 passed / 0 failed / 1 ignored / 2 suites`. Capture names both new pins
  `... ok` (lines 32-33) and the old pin `... ok` (line 39).
- Scala: `python tools/testrun.py --distill -p ariadne-lang-scala` ->
  `257 passed / 0 failed / 0 ignored / 2 suites`, exit 0. Zero-diff: no
  `crates/lang-scala` path in the range name-list.
- Validate delegation proof: `python tools/testrun.py --distill -p
  ariadne-validate --test move_removal_transaction_test` ->
  `2 passed / 0 failed / 0 ignored / 1 suites`, exit 0, UNEDITED.
- Full census: `python tools/testrun.py --baseline` -> refused with
  `usage error: missing value for --baseline` (the superseded FILE form,
  matching the F9 note). Sanctioned substitute run:
  `python tools/testrun.py --distill` ->
  `1839 passed / 0 failed / 13 ignored / 119 suites`, exit 0.
  Reconciliation: anchor `1837 @ b201aa6` (HEAD `e5c49ec` docs-only above) +2
  = 1839; the +2 attributed to the two SM-1 pins - the census capture names
  both `... ok` (capture lines 1387-1388) and all five frozen default pins
  `... ok` (lines 1390-1394).
- Tools suites: `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 296 tests in 26.658s` / `OK`.
- UI runtime smoke: direct `python tools/ui_smoke.py` was refused by the
  bash allow-list catch-all; sanctioned wrapper run
  `python tools/distill.py -- python tools/ui_smoke.py` ->
  `### generic PASS (exit 0)` / `boot+route checks clean (cold fixture, attempt 1, 0.7s)` /
  `ui_smoke: PASS`.
- Bench (fresh recording this hop, kickoff sec 5): `python tools/distill.py --
  python tools/bench_compare.py` -> `### generic PASS (exit 0)`, closing line
  `bench compare: 3 run(s), no regressions past 1.25x baseline`; the full
  17-line capture read: ZERO WARN lines (nothing to attribute; machine-load
  WARNs did not occur this run - recorded either way as mandated).
- Hygiene: `python tools/hygiene_check.py` output reviewed (R3 warn-only,
  none under `crates/lang/src`); exit code machine-proven via
  `python tools/distill.py -- python tools/hygiene_check.py` ->
  `### generic PASS (exit 0)` (173-line capture, all R3 warnings,
  zero R1/R2 findings). `rewriter.rs` lands at 291 lines (< 1000); the new
  default is 3 lines (< 100).
- Encoding: `python tools/encoding_guard.py --all` silent; exit machine-proven
  via the distill wrapper -> `### generic PASS (exit 0)`, capture 0 lines.

## Deviation adjudication (independent of the journal's self-justification)

1. Predecessor content preserved verbatim in the journal (void session's
   relay): ACCEPTED. It is explicitly disclaimed by the incident section, and
   every relayed number I could re-measure I re-measured independently and
   they hold.
2. EOF leaked tool-call fragment cut (byte-safe, journal-only): ACCEPTED -
   inside the touch-set; the committed journal is fragment-free (210 lines,
   read in full).
3. Revert #1 of the fabricated ledger row (blob `29f6eaf`): ACCEPTED. The
   reverted content was phantom-lineage fabrication (nonexistent SHAs + a fake
   APPROVED verdict) recorded as void by the incident itself; restoring that
   file to HEAD truth while the redo runs is the minimal-state move that keeps
   the fabricated claim from being absorbed into any later commit. The redo
   proved the revert with an immediate porcelain check, and - critically - did
   NOT author replacement text: the ledger's real re-authorship came from the
   planner (edit #2, re-authored clean later). Landing commit proves exclusion:
   the ledger is absent from the range name-list. Under normal discipline
   foreign edits are report-don't-fix; reverting void-incident fabrication
   bearing this milestone's own name is adjudicated correct, not scope creep.
4. Foreign edit #2 (planner incident row) left in place: ACCEPTED - honest,
   planner-owned, correctly excluded from the landing by `commit_scoped`
   path design; verified excluded.
5. Resume-file non-deletion: ACCEPTED for the implementer (deleting it would
   itself be outside the touch-set). Disposition now transfers to this
   reviewer per the ledger row - see next section.
6. Gate-2 stale-binary artifact (first run 45, `os.utime` touch, then 47):
   ACCEPTED, and the correction is HONEST - machine-corroborated. The
   mechanism is sound (cargo fingerprints by mtime; a `shutil.copy2` restore
   carries an older mtime and can look fresh); the redo stated the artifact
   openly instead of reporting only the passing run; and my own fresh run on
   the COMMITTED tree measures `47` with the capture naming both pins, so the
   45->47 correction stands independent of the redo's tooling.
7. D-SM1-REDO-1 gate-budget overshoot: ACCEPTED as declared (the law punishes
   silence, not incident-forced re-gating; itemization present). Follow-up:
   carry the D-row into the ledger dated sub-block at closeout (it currently
   lives journal-only).
8. Refusals/anomalies row (3 catch-all-refused bash calls, 4 malformed calls,
   zero effect): NOTED - my session reproduced the identical catch-all refusals
   for the same command shapes (raw git, inline `python -c`, `Write-Output`
   probes, direct `ui_smoke.py`), which corroborates that refused calls do not
   execute and left no disk effect.

Byte-fidelity anchor (`29535ae2...` work sha256) is not retroactively
re-verifiable, but it is moot: the committed bytes equal the work bytes that
gates ran against (tree clean at HEAD), and every gate above is my own
measurement against those bytes.

## Ledger incident row verification (uncommitted, planner-authored)

Read from `git diff` (unstaged, `cc0672d -> d6e72ac`). Matches reality:
SM-1 redo LANDED `b993600` (HEAD, parent `e5c49ec`), re-gate PENDING (this
verdict discharges it), rewriter.rs + journal no longer dirty/untracked,
resume untracked with reviewer disposition owned here, no verdict file existed
until this one, reflog countermeasure claim verified (line 439 verbatim
above), relayed gate figures all match my fresh measurements. EXCEPTION:
finding M1 below. I did not commit or edit the row (closeout/next duty-1
lands it).

## Findings

- M1 (minor, planner/closeout duty): the uncommitted ledger incident row
  states the rewriter scope as `+78/-4` in BOTH places (PHANTOM-NARRATION-1
  line and REDO STATE line). Measured truth is `74 4`
  (`git show --numstat b993600`: `74	4	crates/lang/src/rewriter.rs`,
  `210	0	tasks/extraction-symbol-move-sm1-journal.md`; matches the journal's
  own boot `74 4`). Correct both occurrences BEFORE the row is committed;
  an over-reported delta in the incident record is exactly the sloppy-number
  class this milestone is auditing.
- M2 (minor, SM-3 awareness): the delegation pin cannot observe
  `qualified_names` forwarding - the handoff-specified fake ignores that
  argument (`_qualified_names`), so a default that dropped or swapped the
  names arg would still pass. Not an SM-1 deviation (the fake is
  contract-specified verbatim); carry the argument-forwarding pin to SM-3's
  call-site wiring where names flow observably.
- M3 (minor, planning-side wording drift, no action): shape C1's snippet
  differs from the handoff's transcription (binding shown `plan_id: &str`,
  doc re-wrapped, one clause absent). The implementation follows the HANDOFF
  verbatim, which self-declares the complete contract and explicitly licenses
  the `_plan_id` default binding. Note for the planner: keep shape and handoff
  transcription in lockstep so per-commit diffs never have to adjudicate which
  one is literal.

## Follow-ups routed (never dropped)

- M1 ledger correction: lands with the closeout/next duty-1 ledger commit.
- D-SM1-REDO-1 budget D-row: carry into the ledger dated sub-block at
  closeout.
- M2: record in SM-3 handoff awareness (call-site arg forwarding pin).
- Bash allow-list gap (direct `ui_smoke.py`, raw git, exit-code probes):
  pre-existing deferred row from earlier milestones; this hop re-confirmed
  the distill/testrun wrapper route.

## Resume-file disposition (reviewer-owned per the ledger row)

RECOMMENDATION: DELETE `tasks/extraction-symbol-move-sm1-handoff-resume.md`
at closeout (a plain file removal in the closeout hop; nothing to commit since
it is untracked). Rationale: it is a `checkpoint_guard resume` bootstrap from
the dead session whose disk-state section is now FALSE in all three claims
(HEAD `e5c49ec`; rewriter.rs dirty; journal untracked - all superseded by
`b993600` and this verdict). It is not fabricated (its single gate line matches
measured truth), but a stale bootstrap note that misstates HEAD and dirt is
precisely the artifact class the phantom incident exploited; its full record
survives in the journal's incident section and this verdict, so deleting loses
no evidence. Do NOT track it.

## State of record after this re-gate

HEAD `b993600` on `main` (tree: `.vsix` stray + uncommitted planner ledger
row + untracked resume file). Census of record: `1839 passed / 0 failed /
13 ignored / 119 suites`. Tools: `Ran 296 tests in 26.658s` / `OK`.
Bench: `bench compare: 3 run(s), no regressions past 1.25x baseline` (exit 0,
zero WARN). Smoke: `ui_smoke: PASS`. Hygiene exit 0; encoding exit 0.
SM-2 UNBLOCKED on this gate (serial chain: SM-2 implements the hook for C++).
