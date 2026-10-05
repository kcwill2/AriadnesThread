# V2 Verdict - M26 boot-progress-visibility: additive `discover_with_progress` hook

- Handoff: `tasks/boot-progress-visibility-v2-handoff.md`
- Journal: `tasks/boot-progress-visibility-v2-journal.md`
- Start rev: `f325bee` (V1 landing) -> commits `3f8be39` (repo hook) + `5c81881`
  (journal-only landing bookkeeping). HEAD `5c81881`, branch `main`.
- Verdict: **APPROVED** (no blockers; three minor findings, none requiring
  rework by the implementer).
- Reviewer: review-gate skill, fresh full verification, 2026-10-05.

## Ground truth (checkpoint_guard status)

HEAD `5c81881`, branch `main`, ahead=0/behind=0. Dirty = one pre-existing
`.kilo/plans` file; untracked set contains only pre-existing plans/verdict/
execution-results docs - none attributable to this hop. Worktree census clean
apart from pre-existing foreign worktrees.

## Commit-vs-touch-set (enforcement)

`git diff --name-only f325bee..HEAD` = exactly the 3 declared touch-set files:
`crates/repo/src/discovery.rs`, `crates/repo/src/lib.rs`,
`tasks/boot-progress-visibility-v2-journal.md`. Everything outside
`crates/repo` is journal-only; goldens, fixtures, `types.rs`, `pipeline`,
`tools/**` zero-diff. Both commits individually in-set:
`3f8be39` = 182/+ discovery.rs, +10/-1 lib.rs, +66 journal;
`5c81881` = +16 journal only.

## Per-commit review (oldest-first)

`3f8be39`:
- `lib.rs`: `discover()` body replaced by
  `self.discover_with_progress(&mut |_hashed, _discovered| {})` - the exact
  no-op delegation the contract pins. `discover_with_progress` signature and
  doc comment match handoff sec. "Objective and invariants" verbatim,
  parameters `(hashed_so_far, discovered_so_far)` in order. Public API
  additive-only: `lib.rs:7` re-export line unchanged (still
  `build_discovery_result, compute_file_hash, discover_repository_files`).
- `discovery.rs`: threading is pure insertion. Hook fires at
  `discovery.rs:91-92`, strictly AFTER `compute_file_hash(&path)?` succeeds
  (line 77 - hash errors propagate without firing) and after the record push,
  so `files.len()` = discovered-so-far including the current file. Fire site
  is inside the walk loop = WALK order; path sort at line 98 sits after the
  loop, untouched. Skip list (hidden/`target`/`artifacts`/`node_modules`,
  lines 54-60) and `is_analyzable` gate (line 69) byte-identical - the diff
  contains zero deletions in the body. `discover_repository_files` delegates
  with a no-op closure.
- `discover_repository_files_with_progress` is `pub(crate)` (discovery.rs:34),
  not re-exported - crate-internal, additive-only. "Private" in the handoff
  is satisfied: a plain `fn` could not serve the lib.rs delegation; this is
  the minimum visibility making it compile.
- `progress_hook_tests` mod appended at file end; pre-existing `mod tests`
  below unchanged (census 1226->1230 = +4 only, proving zero pre-existing
  repo tests edited).
- Identity test (test 1) compares root, full `files` vec (ids, paths, hashes,
  order via `assert_eq!` on Vec<FileRecord>), and both summaries.

`5c81881`: journal-only bookkeeping (+16), in-set, matches the
journal-commit precedent cited in dispatch. Its "258 insertions, 1 deletion"
stat is self-consistent at writing time (it describes the range as of
`3f8be39`, before its own landing).

## Hook-semantics audit (handoff's named specific risk)

- Fires post-hash in walk order with `(hashed_so_far, files.len())` -
  argument ORDER matches contract. Verified at code level, not from journal.
- Test 2 asserts monotone counts in CALL order (`[1,2,3]`) and
  first==second per call; asserts nothing about paths - no
  walk-order==path-order assumption. Reviewer extra check satisfied.
- Test 4 shipped via the declared ELSE branch (see deviation 1); it locks
  hook-fires-while-walking (count==4==result.len()) and result-path-sorted,
  exactly as the ELSE wording requires.
- Test 3 covers the skip list (target/artifacts/node_modules/.hidden present,
  inert) with 5 analyzable files across 3 dirs.

## Deviation dispositions

1. **Test 4 as ELSE-branch `test_hook_order_independent_of_path_sort`** -
   ACCEPTED. The ELSE branch is the handoff's own text; its precondition
   holds (std::fs on Windows has no chmod; ACL manipulation via `icacls` is
   a process call, not a cheap unit-test primitive). Journal records the
   choice as required. The fallback locks the two properties the ELSE names.
   Independent check: the dropped error-propagation property is unchanged by
   this diff anyway (hash error still propagates via `?` before any hook
   fire - verified in code).
2. **Gate-1 compound-echo allowlist refusal, re-ran verbatim** - ACCEPTED.
   Same family as V1 finding 1; I reproduced the family twice today (piped
   `Select-Object`, `; echo` suffix - both catch-all-denied, both succeeded
   unpiped). Exit code lives in the digest header.
3. **Direct `commit_scoped.py` (distill-wrapped commit refused "per the
   dispatch ruling") + explicit `git add -- <touch-set>` staging** -
   ACCEPTED per V1 verdict finding 4 precedent; commit content independently
   verified as exactly the declared scope, so the route cannot have smuggled
   anything in.

## Gates (reviewer hop, all re-run fresh, canonical lines verbatim)

- Scoped repo (`python tools/testrun.py --distill -p ariadne-repo`):
  `15 passed / 0 failed / 0 ignored / 2 suites`, exit 0. Exact match.
- Workspace census: instructed form `python tools/testrun.py --workspace
  --no-fail-fast` REFUSED: `usage error: unknown flag: --no-fail-fast`
  (testrun.py's allowlist at tools/testrun.py:58-70 does not carry the flag
  and does not inject it internally). Ran the V1-sanctioned distill form
  (`python tools/testrun.py --distill` = `cargo test --workspace`):
  `1230 passed / 0 failed / 4 ignored / 86 suites`, exit 0.
- Reconciliation from ledger/V1 anchor `1226 passed / 0 failed / 4 ignored /
  86 suites` @ f325bee: +4 shipped tests (identity, monotone-counts,
  mixed-tree, sort-independent) -> 1230. Exact. See finding 2 on the
  dispatch's expected 1229.
- `python -m unittest discover -s tools -p "*_tests.py"`:
  `Ran 188 tests in 20.463s` / `OK`.
- `python tools/hygiene_check.py`: exit 0; output is entirely pre-existing
  R3 warn-only rows; zero rows in `crates/repo/src/discovery.rs`;
  `crates/repo/src/lib.rs:92-96` `res` rows predate the hop (diff touches
  only lines 29-44).
- `python tools/encoding_guard.py --all`: exit 0 (no output).
- `python tools/distill.py -- python tools/ui_smoke.py`:
  `### generic PASS (exit 0)` / `ui_smoke: PASS`; capture
  `target/distill-capture.txt`.
- bench_compare: NOT run, exemption verified. Ledger M26 row carries the
  written gate list ("NO bench gate (written behavioral-no-op exemption;
  PR-2 bench refresh belongs to M27 only)"). Premise verified independently:
  `discover_with_progress` has zero callers outside `crates/repo` (grep), so
  the boot path is byte-for-byte the V1 path - bench-relevant behavior
  cannot have moved.
- `testrun.py --baseline`: not run - no test-count baseline file exists
  anywhere in the workspace (only `tools/theme_token_baseline.json` and
  `tools/bench_baseline.json`); the census was reconciled against the V1
  verdict line instead, same as the V1 reviewer hop. See finding 1.

## Red-protocol check

Journal: `12 passed / 3 failed / 0 ignored / 1 suites`, exit 1, failing list
= exactly tests 2-4, each `left: 0`, test 1 green on the discarding stub -
matches the handoff's predicted shape. Arithmetic self-consistent:
12+3 = 15 = green total. Suite 1->2 across Red->Green is expected cargo
fail-fast (doctest target skipped after the lib target fails), not drift.
Red capture itself overwritten by later runs - see finding 3.

## Report-evidence check (distillation contract)

Journal is digest-shaped: verdict headers with exit codes, canonical lines,
capture paths, zero raw dumps; gate commands all through
`testrun.py --distill` / `distill.py --`. Satisfied.

## Sizing-law confirmation

1 crate; 5 declared implementer gates (<= ~10); produced diff 258 insertions
(+16 bookkeeping) vs <= ~1200 ceiling. Within ceiling, confirmed.

## Findings

1. minor (config, not handoff): the dispatch's census command `--workspace
   --no-fail-fast` is not runnable - `testrun.py` refuses the flag (verbatim:
   `usage error: unknown flag: --no-fail-fast`) and never injects
   `--no-fail-fast` internally, while the ledger carries a dated
   "fail-fast MUST errata" row. The flag-surface fix belongs to the
   testrun.py owner; census ran in the V1-sanctioned distill form.
2. minor (dispatch arithmetic): the dispatch expected
   `1229 = 1226 + 3 new`; V2 ships **four** tests (test 4's ELSE swap is 1:1,
   not a drop), so the arithmetic is `1226 + 4 = 1230`. The actual census is
   `1230 passed / 0 failed / 4 ignored / 86 suites` - the workspace number
   is right; the dispatch's expectation was short one test. No test is
   missing or extra: the four shipped names are enumerated above.
3. minor (evidence quality, no action): the journal reports the Red failing
   list as a named claim ("exactly tests 2-4") rather than a verbatim list,
   and the Red capture was overwritten by the green run. Acceptable under the
   handoff's own no-raw-paste report rule; arithmetic cross-check closes it.

## Follow-up dispositions (out-of-scope, report-only -> ledger duty at
M26 closeout, NOT V2 findings)

- Ledger M26 row still reads PLANNED and lacks the V1 landing row + census
  anchor; `tasks/boot-progress-visibility-v1-verdict.md` is untracked. V2's
  journal cites the on-disk V1 verdict correctly. V3 closeout (milestone-
  lifecycle) must land both.
- Finding 1's testrun.py flag-surface gap: route to the tools owner or a
  dated ledger errata row.

## V3 dispatch

**CLEARED.** `discover_with_progress` is landed, contract-exact, and
order-preserving; V1 (`stage_spans.rs`, boot frame-1-first) is landed; the
V3 preconditions named in the handoff's sequencing note ("V3 consumes
discover_with_progress - dispatch V3 only after this lands") are met. T3
remains separately BLOCKED pending its own written T1-sufficiency FAIL +
M26 V3 landing - unaffected by this verdict.
