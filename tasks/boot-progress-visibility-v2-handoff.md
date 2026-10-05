# V2 Handoff - M26 boot-progress-visibility: additive `discover_with_progress` hook

Milestone: `boot-progress-visibility` (M26, ledger row 2026-10-05). Single-crate
handoff (repo). Sizing law held: 1 crate, 5 implementer gate invocations
(<= ~10), ~150-250 expected produced lines (<= ~1200). Workspace-wide gates
(census, `ui_smoke`, `bench_compare`, `encoding_guard --all`) belong to the
reviewer hop, not to this session.

Why: at boot, all of `Pipeline::new`'s construction (serial SHA-256 discovery
hashing included) runs before any progress callback exists - symptom S1. V3
wires boot events; it needs this additive, order-preserving hook first.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md`
   sec. 3.1 (`Repository::discover_with_progress` contract, verbatim),
   sec. 4 (V2 row: additive, order-preserving).
3. `crates/repo/src/lib.rs` lines 15-35 (`Repository::discover`) and
   `crates/repo/src/discovery.rs` lines 15-111 (hash + walk + record build).
4. `crates/repo/src/types.rs` (`RepositoryDiscoveryResult`, `FileRecord`).
5. `docs/specs/tool-output-distillation.md` (distilling entry points).

## Objective and invariants

Add the exact contract (shape sec. 3.1, binding):

```rust
impl Repository {
    /// Additive; per hashed file, (hashed_so_far, discovered_so_far).
    /// discover() delegates no-op. Record ORDER unchanged.
    pub fn discover_with_progress(
        &self, on_hashed: &mut dyn FnMut(usize, usize),
    ) -> Result<RepositoryDiscoveryResult>;
}
```

- `Repository::discover` becomes a delegation: `self.discover_with_progress(
  &mut |_hashed, _discovered| {})` - behavior byte-identical, zero extra cost.
- Threading: add a private `discover_repository_files_with_progress(root,
  on_hashed)` in `discovery.rs`; `discover_repository_files` delegates to it
  with a no-op closure. The hook fires AFTER `compute_file_hash` succeeds for
  each analyzable file, with `(hashed_so_far, discovered_so_far)` - both equal
  to the count-so-far on this single-pass walk; the pair is spelled as two
  arguments per the contract (a future 2-pass walk can split them).
- Invariants: (a) result byte-identical to `discover()` for any tree (same
  records, same sorted-by-path order, same summaries); (b) walk/skip rules
  unchanged (hidden, `target`, `artifacts`, `node_modules`,
  `is_analyzable`); (c) hook order is WALK order, not path order - tests must
  assert monotone counts, never path ordering of hook calls; (d) hashing is
  NOT parallelized here (DP-4 territory, evidence-gated, this hop does not
  open it).

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/boot-progress-visibility-v2-journal.md` (checkpoint journal; always
  in-set; create it alongside this handoff)
- `crates/repo/src/lib.rs` (`discover` delegates; new
  `discover_with_progress`)
- `crates/repo/src/discovery.rs` (hook threading; appended `#[cfg(test)]
  mod progress_hook_tests { ... }` at file end - existing lines below the
  threading change stay byte-identical)

You MUST NOT touch: any other crate; `crates/repo/src/types.rs` (no type
change - the contract closes over plain usizes); `crates/pipeline/**` (V3
consumes your API next); `tests/golden/**`, `tests/fixtures/**`; `tools/**`.
No `git stash`, no branch change.

## Required tests (in the appended `progress_hook_tests` mod)
1. `test_discover_with_progress_result_identical_to_discover` - temp tree
  (analyzable + skipped + non-analyzable files, nested dirs):
  `discover()` and `discover_with_progress(&mut |_,_| {})` produce equal
  `files` (ids, paths, content hashes, order) and equal summaries.
2. `test_hook_fires_once_per_analyzable_file_with_monotone_counts` - tree with
  exactly 3 analyzable files (plus one skipped-dir file and one `.md`):
  hook call count == 3; the first argument runs 1,2,3; first == second at
  every call (single-pass walk per the contract).
3. `test_hook_count_matches_result_len_on_mixed_tree` - hook count ==
  `result.files.len()` on a tree with 5 analyzable files across 3 dirs and
  the skip list present.
4. `test_hook_surfaces_hash_error_propagation` - make one analyzable file
  unreadable only if cheaply expressible on Windows, ELSE replace this test
  with `test_hook_order_independent_of_path_sort` (hook fires while walking;
  result order still path-sorted) - journal records which you shipped.

Red protocol (valid Red, no todo!()): author tests + the API seam where
`discover_with_progress` compiles but discards the hook (delegates to the
plain walk) - tests 2-4 then FAIL at runtime ("hook call count 0 != 3");
test 1 already passes (identity holds for the delegating stub) - that is
expected; capture the failing list verbatim. Green: thread the hook.

## Hygiene (review-enforced)
Functions < 100 lines (the walk is ~60; threading must keep it under 100 -
if it would not, extract the per-file record build into a helper); files
< 1000; no banned names; pure ASCII authored content.
`python tools/hygiene_check.py` exit 0.

## Frozen locks (each with its proving command)

| Lock | Proof |
|---|---|
| `discover()` behavior byte-identical (I1 lineage) | test 1 green + scoped suite: all pre-existing `ariadne-repo` tests green UNEDITED |
| goldens/fixtures frozen; no other crate touched | `git diff --stat <start-rev>..HEAD` lists touch-set only |
| mojibake gate | `python tools/encoding_guard.py --staged` exit 0 pre-commit |
| hygiene | `python tools/hygiene_check.py` exit 0 |

Record start rev in the journal at bootstrap.

## Gates (exact commands, timeout 600000 ms each; scoped only)
All gates are distilling entry points (ledger row 58;
`docs/specs/tool-output-distillation.md`). Workspace-wide gates belong to the
reviewer hop.

1. Red: `python tools/testrun.py --distill -p ariadne-repo` - failing list =
   tests 2-4; exit 1.
2. Green: same command - `0 failed`, exit 0.
3. `python tools/hygiene_check.py` - exit 0.
4. `python tools/encoding_guard.py --staged` - exit 0 before committing.
5. Commit: `python tools/distill.py -- python tools/commit_scoped.py --scope
   crates/repo/src/lib.rs --scope crates/repo/src/discovery.rs --scope
   tasks/boot-progress-visibility-v2-journal.md -m "feat(repo): additive
   discover_with_progress hashing hook, discover() delegates no-op
   (boot-progress-visibility V2)"` (never `git add .`).

Total implementer gate invocations: 5 (<= 10).

## Refactor and hygiene targets (named ceilings)
- Ceiling at risk: `discover_repository_files_with_progress` line count
  (100-line fn cap): if threading the hook pushes it past ~90, extract
  `fn record_for_path(...) -> Option<FileRecord>` and keep the hook call in
  the loop body. Nothing else at risk.

## Acceptance (machine-checkable; implementer hop)
1. `python tools/testrun.py --distill -p ariadne-repo` -> `0 failed` exit 0.
2. `python tools/hygiene_check.py` -> exit 0.
3. `python tools/encoding_guard.py --staged` -> exit 0.
4. `git show --stat` lists exactly the touch-set.
Reviewer hop: scoped `-p ariadne-repo` re-run; census; `--all` encoding;
commit-vs-touch-set; confirm `Repository::discover` call sites unchanged
behaviorally (grep call sites - none may switch to the new fn except the
delegation itself).

## Discipline clauses
- UTF-8: never Set-Content/Out-File/`>`/Get-Content-replace; Write/Edit tools
  only; pure-ASCII content (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; on discovering out-of-scope breakage: report, do
  not fix.
- Bare high-volume commands in this session are a reviewer blocker; use the
  distilling entry points.
- After EACH per-module Green gate, append a 1-3 line checkpoint to
  `tasks/boot-progress-visibility-v2-journal.md`.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/boot-progress-visibility-v2-handoff.md --scope ... --cmd ...`.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence (first failing test list).
3. Green evidence (distill digest header + canonical line + capture path).
4. Any deviation + reason. 5. Out-of-scope observations (report-only).
Never paste raw tool output: cite command + canonical line + exit code.

## Reviewer-gate reference
Adjudicated via the `review-gate` skill; its commit-vs-touch-set check
enforces the touch-set. Extra reviewer checks: hook arguments exactly
`(hashed_so_far, discovered_so_far)`; `discover()` = no-op delegation;
skip-list and sort unchanged; test 2's assertions do NOT assume walk order ==
path order.

## Sequencing note
Parallel-safe with V1 (disjoint crates). V3 (`boot-progress-visibility-v3`)
consumes `discover_with_progress` - dispatch V3 only after this lands. Not
parallel with any other hop touching `crates/repo` (none exists this
milestone; DP-4/T4 would, but it is evidence-gated).
