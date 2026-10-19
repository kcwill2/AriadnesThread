# UTA-T1 Verdict - APPROVED

- Handoff: `tasks/uta-t1-handoff.md` (declared complete contract)
- Commit under review: `994e5c3` (base `43e6ebe`), single commit in range, main, clean tree
- Spec context: `docs/specs/unit-test-audit.md` (UTA-B4, UTA-B5 adjudicated fixes; H1 hand)
- Reviewer session: reviewer subagent, 2026-10-18
- Verdict: **APPROVED** - no blockers; two minor findings + follow-up dispositions below.

## Canonical verification line (reviewer's fresh run)

`python tools/testrun.py --distill -- -p ariadne-graph`:

```
### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 246 lines]
170 passed / 0 failed / 0 ignored / 11 suites
```

Matches the handoff-mandated `170 / 0 / 0 / 11` exactly. New pins sampled present-and-ok in
the capture (`find_file_id_prefers_full_path_then_smallest_file_id_deterministically`,
`two_scc_graph_enumerates_every_cycle_once_in_normalized_order`,
`deep_call_chain_cycle_detection_completes_on_small_stack`,
`fused_100k_cycle_tarjan_frame_machine_completes_on_small_stack`,
`pin_include_load_equals_total_load`, `pin_call_edge_without_caller_node`).

## Gate results (all run fresh by reviewer)

| Gate | Result |
|---|---|
| `testrun.py --distill -- -p ariadne-graph` | PASS - `170 passed / 0 failed / 0 ignored / 11 suites` |
| `git log --oneline 43e6ebe..HEAD` | single commit `994e5c3`, reviewed alone (oldest-first trivial) |
| `git diff --name-only 43e6ebe..HEAD` | 14 paths, ALL inside touch-set (`crates/graph/**`, `tasks/uta-t1-journal.md`); no Cargo.toml/lock, no cli, no other crate |
| `python tools/hygiene_check.py` | exit 0 (R1/R2 clean; R3 warn-only, see Finding 2) |
| `python tools/encoding_guard.py --all` | exit 0 (no output) |
| Frozen locks / existing tests | `git diff --numstat`: deletions only in the two functions being fixed (graph.rs -7, callgraph.rs -37, both inside the rewritten bodies); every other touched file 0 deletions; `load_impact_tests.rs` untouched - no existing test edited anywhere |
| Report evidence (distillation contract) | journal is digest-shaped: red/green digest lines, capture path `target/testrun-capture.txt`, exit-2 red class named, zero raw pastes; gates used `testrun.py --distill` |

## Per-commit review - `994e5c3`

**Item 1 (UTA-B4, graph.rs:135):** two-pass resolution matches the contract verbatim -
pass 1 exact full-path (smallest FileId tie-break), pass 2 basename (smallest FileId);
order- and hash-seed-independent. The test-only scan counter stays once per call, so the
LoadPerf A2 counter locks hold (journal claim, code-checked). Red pin meets every fixture
requirement named in the handoff (`a/util.h`/`b/util.h` registration, repeated-call and
reverse-registration agreement, full-path `b/util.h` beats basename `a/util.h`); the
32-trial fresh-HashMap construction gives the old rule ~2^-32 survival odds - adequate
discrimination. Red evidence in journal is specific: `108 passed / 1 failed / 0 ignored /
1 suites`, `left: Some(FileId(1)) right: Some(FileId(2))`.

**Item 2 (UTA-B5, callgraph.rs:205):** `explore_call_frames` is a faithful,
non-recursive translation of the retired DFS, verified clause by clause against the
removed body: on-path hit pushes `path[position..]` and does NOT mark/descend (old code's
cycle branch did neither); `visited.insert` gates descent (old code's visited-check);
parent cursor advanced before child push (call-return resume); frame pop retires path
entry and position entry together (old tail `path.pop()`); root entry replicates the
root call head; root and neighbor orders unchanged. Push/advance/pop are the equivalence
critical points and all three hold. `path_position` replaces the O(depth) on-path scan -
required for the 100k fixture's linearity and behavior-preserving. The retired recursive
`dfs` is fully deleted - no retained dual representation. `debug_assert` guards full
unwind per root. 100k fused-chain test asserts exactly one cycle covering all 100 000
members on a 64 KiB thread; two-SCC fixture asserts exact enumerated list
`[[A,B],[C,D]]` (all cycles, once each, normalized). Red evidence is the correct class:
aborted test binary (`has overflowed its stack`), testrun exit-2 parse drift, mirroring
the sanctioned M23.5 T2.1-R1 pattern. The journal is candid that the two-SCC fixture
passes against both implementations - it is the equivalence guard; the discriminator is
the stack-overflow red.

**Items 3-15 pins:** all present, plan-literal where literals are pinned -
item 7 asserts (size desc, path asc), `take(10)`, `{:016x}` fallback exactly as
specified; item 8 literal byte snapshot with `(cycle)` marker and connector prefixes
(ASCII via unicode escapes); item 9 pins both `"No ..."` error strings on both lanes;
item 10 pins the mandated name `pin_parallel_directive_edges_count_toward_degree`;
item 12 pins the mandated name `pin_include_load_equals_total_load` against the real
degeneracy (load.rs:548/550 - `include_load` and `total_load` both
`physical_symbols.len()`, journal's cite confirmed); item 15 covers all four named
pins; item 14 correctly report-only. No assertion weakening found anywhere; no
required-test turned red (none of the pins exposed an additional ledger bug).

## Deviation dispositions (independently adjudicated, not rubber-stamped)

1. **Item 13 split (100k stress)** - **ACCEPTED.** The infeasibility claim verifies in
   code: `assemble_closures` (closures.rs:244-256) materializes AND sorts a per-member
   `transitive_includes` Vec; a fused 100k cycle is ~10^10 element copies plus ~100k
   sorts of 100k elements (~10^11 ops) and ~80 GB of closure rows - unrunnable in the
   600 s gate on any stack, recursion or not. The split preserves both properties the
   pin exists to test: the FULL 100k fused cycle runs on the 64 KiB thread through
   `tarjan_components` (the module's only recursive-overflow risk surface - the rest of
   `compute_closures` is iterative reach-set DP + assembly, verified by read), and the
   PUBLIC pipeline runs end-to-end at 3k on the same 64 KiB thread, an order of
   magnitude past the few-hundred-frame survivable depth of a recursive walk. The
   literal wording was physically unsatisfiable; the surrogate is faithful.
2. **Item 6 skipped (already pinned)** - **ACCEPTED.**
   `test_impact_guards_return_none` (load_impact_tests.rs:313-326, untouched by this
   commit) pins all three named guard rows: root==source both lanes, empty justified
   set, unregistered ids in both positions -> None. Re-authoring would duplicate a
   frozen contract.
3. **Item 12 file name (`load_degeneracy_tests.rs` vs planned `load_guard_tests.rs`)**
   - **ACCEPTED.** Content matches the plan verbatim (pin name + degeneracy at
     load.rs:548-550 + report); the file is new, inside the touch-set, wired via
     `#[path]`; naming-after-payload is within the handoff's "cleaner than inline"
     latitude.
4. **Preserved global-visited cycle undercount** - **ACCEPTED as out of contract
     scope; the shipped fixture MEETS the item-2 contract.** Verified independently,
     two ways: (a) the two-SCC fixture required by item 2 has exactly two cycles
     (A<->B, C<->D) and the test asserts both appear once each in normalized order -
     the contract sentence is fixture-scoped and satisfied; (b) the frame machine is a
     strict clause-by-clause translation of the retired DFS, so the diamond-feed
     undercount (1->2, 1->3, 2->4, 3->4, 4->1 reporting only [1,2,4]) is behavior
     PRESENT IN THE OLD CODE TOO - traced by hand and matching the journal's claim
     exactly. The handoff (the declared complete contract) builds a no-recursion
     replacement + two fixtures; a full elementary-cycle enumeration changes
     `CallGraphManifest.recursion_cycles` artifact content - an owner decision,
     correctly refused unilaterally. Follow-up disposition: ledger deferred-findings
     (see below).
5. **`encoding_guard --staged` run around an atomic `commit_scoped.py` commit** -
   **ACCEPTED.** `commit_scoped.py` stages+commits atomically (no hook point), so the
   implementer ran `--staged` against the pre-commit index (exit 0, journal) and
   `--all` afterward; the reviewer independently ran `--all` over all tracked files at
   the shipped commit: exit 0. Coverage is equivalent or stronger than the letter of
   gate 3.

## Findings (numbered)

1. **[minor] Item 10 matrix gap: IncludeGraph's `get_entrypoints` has no all-cycle
   (zero roots) row.** `include_graph_entrypoints_lock_tie_order_and_truncation`
   covers tie order + truncation (and the unresolved-only and parallel pins exist as
   separate tests), but the "all-cycle graph (zero roots)" case named in handoff item
   10 is pinned only on the ReducedSemanticModel and CallGraph implementations
   (traversal.rs tests; capture lines 121-125 confirm test inventory). The three
   `get_entrypoints` implementations are separate code paths, so the IncludeGraph
   cycle behavior is unexercised. Not a blocker: every named case is pinned at least
   once, tie order + truncation are pinned on all three, and the case list reads as a
   case inventory across the three lanes rather than a strict full matrix. Disposition:
   record in `tasks/milestone_ledger.md` deferred-findings for a one-test follow-up or
   fold into a later graph hand.
2. **[minor, informational] Journal's "all listed R3 warns are pre-existing lines"** -
   **claim verified, no action.** Hunk ranges prove it: traversal.rs additions are
   `@@ -586,0 +587,328 @@` and reduced.rs `@@ -790,0 +791,171 @@`, while the R3 warn
   lines in those files (traversal.rs:247/406/514, reduced.rs:526/707/708/760) all sit
   outside the added ranges. No new R3 warning is introduced by this commit; R3 is
   warn-only per AGENTS.md.

No blockers. APPROVED.

## Follow-up dispositions (never dropped, per review-gate step 9)

To `tasks/milestone_ledger.md` deferred-findings (orchestrator's write, recorded here):
- UTA-B5 residual: global-visited skip still undercounts cycles on diamond-feed shapes
  (journal's exact trace reproduced by reviewer); full enumeration changes
  `CallGraphManifest.recursion_cycles` - owner decision.
- builder.rs:89-92 per-file index failure swallow (`Err(_) => continue`, no counter/log)
  - item 14 report-only observation for the owner (diagnostic policy decision).
- `include_load == total_load` degeneracy and parallel-directive degree inflation -
  pinned, semantics undecided, owner decision (pins will turn red on change).
- `pin_call_edge_without_caller_node` floating edge-without-caller-node shape - pinned,
  reported for owner decision per item 5.
- Finding 1 (IncludeGraph all-cycle entrypoints row).

## Command refusals observed during review (recorded per protocol)

Two compound/shaped invocations were refused with the catch-all deny rule; neither gate
was skipped as a result (both gates then ran as plain individual commands and passed):

- `python tools/hygiene_check.py; echo "HYGIENE_EXIT=$?"; python tools/encoding_guard.py --all; echo "ENCODING_EXIT=$?"` -> refused: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}`
- `python -c "import subprocess,sys; sys.exit(subprocess.call(['python','tools/hygiene_check.py'], stdout=subprocess.DEVNULL))"` -> refused with the identical boilerplate.

Plain `python tools/hygiene_check.py` and `python tools/encoding_guard.py --all` each
completed cleanly (nonzero exit would have been reported by the shell tool); the
compound shape (`;` chaining with echo, and `python -c`) is what the allowlist rejects.
