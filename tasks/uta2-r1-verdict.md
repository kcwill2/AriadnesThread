# UTA2-R1 Verdict - ui traversal projection fixes + endpoint coverage

**APPROVED** - commits `af8a997` + `64314d7` (journal rider) on handoff base
`19349b3`; commit parent is `89fb908` because the sibling hand `uta2-r2`
landed between (sequencing artifact, not a scope issue - the hand's two
commits are exactly the declared ones). Verdict artifacts left untracked per
T1-T7b precedent (reviewer git-write denied).

Contract: `tasks/uta2-r1-handoff.md`. Spec: `docs/specs/unit-test-audit.md`
Round 2 (UTB-1..4). Journal: `tasks/uta2-r1-journal.md` (dead predecessor's
consumer-greps + successor completion log).

## Fresh verification (reviewer's own runs)

| Gate | Result |
|---|---|
| `python tools/testrun.py --distill -- -p ariadne-ui` | `### cargo test PASS (exit 0) [capture: C:\Users\Kurt\Code\AriadnesThread\target\testrun-capture.txt, 406 lines]` / `369 passed / 0 failed / 0 ignored / 3 suites` - matches the contract's expected 369/0/0/3 verbatim |
| `python tools/distill.py -- python tools/ui_smoke.py` | `ui_smoke: PASS` (`boot+route checks clean (cold fixture, attempt 1, 1.0s)`) |
| `python tools/hygiene_check.py` | exit 0 (`### generic PASS (exit 0)` via distill); R3-only; every warned line in the two touched src/test files sits OUTSIDE every modified hunk (hunk-header audit; blame denied) -> all pre-existing |
| `python tools/encoding_guard.py --all` | exit 0 (`### generic PASS (exit 0)`, capture 0 lines) |
| `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 274 tests in 22.454s` / `OK` (274, not the T7b-era 232: the sibling `uta2-r2` commit 89fb908 added 42 tools tests; zero tools paths in THIS hand's diff) |

Suite arithmetic: red `358 passed / 11 failed` (journal capture) + zero test
deletions -> 369 green exactly accounts every red. Report evidence is
digest-shaped with capture paths, zero raw dumps.

## Per-commit review + touch-set

- `git diff --name-only af8a997^..64314d7` -> 14 paths: 2 src
  (`api_graph.rs`, `api_source.rs`), 10 ui_test files, journal. All inside
  the hard touch-set. **No assets JS touched, no `crates/graph` changes, no
  Cargo files, `common.rs` (frozen harness) not in the commit.**
- `af8a997` (1844+/117-): the 117 deletions are (a) the UTB-1/2/3/4
  pre-fix emission code, (b) the renamed test's 200-only bodies replaced by
  stronger body assertions, (c) the relocated standalone-export test. No
  existing assertion weakened anywhere (graph_api.rs diff reviewed hunk by
  hunk: rename + strengthening only, handoff-authorized item 6).
- `64314d7`: journal witness rider (+9) - truthful.

## Audit-focus verification (this hand's named risk)

**UTB-1/2 single-node-set mechanism.** The hunk shows ONE
`closure_node_ids(closure, gen, exclude)` - closure minus dep, minus
`file_paths`-unregistered, `sort_unstable_by_key(|id| id.0)` ascending
numeric - driving BOTH `build_subtree_nodes_from_closure` (maps the ordered
slice) and `build_subtree_edges_from_closure` (iterates the ordered slice,
emits an edge only when `node_set.contains(&target)`). Ghost-id exclusion
holds BOTH directions (source from the induced list, target gated by the
same set); the dep can never be an endpoint. Edge emission order is
deterministic: `direct_includes` is sorted+dedup by construction
(`crates/graph/src/graph.rs:233-240`, AR-09, locked by
`ar09_adjacency_equivalence_test`) so edges emit (source asc, target asc).
**No dual representation retained**: both old closure-HashSet functions were
REPLACED in the hunks, not paralleled.

**Mechanism correction verified TRUE.** `closures_tests.rs:88-98` locks the
BFS seed-exclusion convention ("a member's closure never contains itself"),
so the audit's dep-as-endpoint phrasing is vacuous for acyclic graphs; the
real orphan channel is nodes=`closure ∩ file_paths` vs edges testing bare
`closure` membership. The shipped red asserts the invariant honestly: ghost
FileId(44) fixture, per-edge both-direction loop asserts, exact induced-edge
literal `["4->5"]`, plus an explicit never-references-dep assert.

**UTB-3.** `api_repo_info` emits sorted `strip_prefix(repo_root)` VALUES
(outside-root keeps the full lossy string). Consumer greps independently
spot-checked by the reviewer: `extensions/**` contains exactly ONE
`file_paths` occurrence (`extension.ts:18`, the type annotation - no reader);
`m16_traversal_test.rs:142` is existence-only for hubs; integration
`file_paths` hits are all `IncludeGraph` fixture plumbing; zero `repo-info`
references in `tests/integration`; ui assets contain zero `repo-info` /
`file_paths` references (reviewer's own grep). Red is genuine (pre-fix
emitted `["12","3","9"]`).

**UTB-4.** `get_entrypoints` ranks hubs degree-desc/id-asc and truncates
AFTER sorting (`traversal.rs:256-264`); the old handler dropped that order
(the admitted in-code skip). The fix maps `hubs_ids` through an
`element_by_id` lookup, restoring rank order. The R1 extraction is
behavior-equivalent for roots/leaves (loop moved verbatim, identical
`source.is_none()` + id guards). Consumer claim verified read-only:
`api.js:343-346` calls `.map` on the response OBJECT -> TypeError ->
catch -> the modal always renders "Error loading entrypoints." - no live
order dependency; `traversal.js` never calls the endpoint. Every test
compares hub ids sorted (`element_ids` sorts at `graph_api.rs:283`) or
existence-only; the `limit=1` pin `"2"` survives honestly because 2 is the
fixture's degree-2 rank #1 (rank order `[2,7,1,30]` vs emission
`[1,2,7,30]` - the new fixture makes the rank sequence observable).

**Re-pins (highest-risk items) - all three are re-pins of BROKEN tests,
none a weakening:**
1. Keys literal: the assertion ran `keys.sort()` and compared against a
   literal whose order (`analyzed_symbol_ids` before `actual_subtree`) can
   never equal any sorted sequence - self-contradictory by construction.
   Re-pin keeps the identical 7-key set in honest ascending order; reviewer
   verified the new literal IS correctly sorted - the assertion is now
   meaningfully falsifiable.
2. `sorted_node_ids` sorts Strings LEXICOGRAPHICALLY (`traversal_matrix.rs:125-129`,
   `"30" < "4"`); it could never equal the numeric-order literal
   `["4","5","30"]` under ANY emission order. Re-pin asserts the RAW
   emission order against that literal - same set, strictly stronger
   (order-pinning).
3. `symbols=` necessary-set arms contradicted the LOCKED graph A2 rule:
   `required_headers_for_symbols` drops any closure symbol whose defining
   file equals ANY seed defining file (`load.rs:336-357`, the
   `!seed_files.contains(&f)` filter), locked by
   `test_required_headers_seed_defining_file_dropped_wholesale` and
   `test_required_headers_isolated_seed_returns_empty` (`load_justification_tests.rs:433/:479`).
   drag@aux.h and unused@spare.h are seed files -> R={}/`{req.h}` are the
   only correct values; the authored expectations were provably wrong. The
   override INTENT is preserved untouched (`analyzed_symbol_ids` asserts).

**Deviation + rider dispositions (all ACCEPTED on independent merits):**

| # | Deviation | Disposition |
|---|---|---|
| D1 | UTB-1 red-shape change (handoff's literal red cannot fail; ghost fixture shipped instead) | ACCEPTED - correction verified TRUE; shipped red is stronger |
| D2 | `file_paths` sort added (not verbatim in handoff) | ACCEPTED - journal'd first; without it the repaired payload re-introduces the UTB-2 nondeterminism class; no consumer parses values |
| D3 | UTB-4 landed on the FIX branch | ACCEPTED - handoff's own conditional rule; the allowance condition (JS ignores order) independently verified |
| D4 | `arms_state` via `UiState::new_loaded` | ACCEPTED - `common::ready_state` hardwires `empty_reduced_model()` (`common.rs:215-228`), so the symbol arm could never see the chain; harness file NOT in the commit; zero assertions touched |
| D5 | Re-pin #1 (keys) | ACCEPTED - broken-test re-pin, same set, stronger |
| D6 | Re-pin #2 (node order) | ACCEPTED - broken-test re-pin, strictly stronger |
| D7 | Re-pin #3 (symbols= sets) | ACCEPTED - re-pin to locked A2 semantics, intent pins intact |
| D8 | R1 extraction `partition_entrypoint_elements` | ACCEPTED - verbatim roots/leaves projection; hubs rank-map (the intended fix); handler back under 100 (hygiene R1 exit 0) |
| D9 | R2 split `static_html_standalone_export.rs` | ACCEPTED - test body matches the removed hunk line-for-line; registered in `main.rs`; 9th asset row `/app/traversal.js`+`TRAVERSAL_JS` added without editing any existing row |
| D10 | ui_smoke via `distill.py` (direct invocation refused) | ACCEPTED - sanctioned fallback; PASS witnessed |

## Findings (none blocking)

1. **Unverifiable-before-state on re-pins (process note).** The
   predecessor died with its tests uncommitted, so `af8a997` is the first
   git presence of the pre-re-pin test shapes; the "broken" property was
   adjudicated from internal contradictions checkable in the committed code
   (sortedness self-contradiction, lexicographic-vs-numeric impossibility,
   locked A2 rule) plus the journal, not from a byte-level before/after.
   The suite arithmetic (358+11=369, zero deletions) closes the
   "never weakened" question regardless.
2. **Quirk pins = owner decisions, route to ledger deferred-findings at
   closeout:** `actual_subtree.file_count` undercount (`api_graph.rs:672`
   `saturating_sub(1)` over an already dep-excluded closure; both fixtures
   pin it); collateral edges hardcoded `is_necessary=true`; ghost-id
   asymmetry (survives collateral as path `"unknown"`, dropped from actual
   nodes); unreachable `Err(_) => empty_traversal_response` arm
   (`load.rs:506-557` has no Err path); **`api.js:343-346` showEntrypoints
   object-vs-array bug - the entrypoints modal NEVER renders** (JS is the
   locked degree-bug file family; explicitly out of this hand);
   pre-existing ui-lib warnings (unused `INDEX_HTML` import, unused `gen`
   param); `common::ready_state` empty-model trap (successor proposes a
   `ready_state_with_model` variant).
3. **Configuration note.** Catch-all deny boilerplate hit compound shapes
   (`; echo $LASTEXITCODE` chains, `| Select-Object`, `Get-Content -Tail`,
   `git blame`); every gate reran plain or via `distill.py` exit-code
   witnesses; no gate skipped.
4. **Worktree census.** Untracked `tools/package_offline_tests.py` belongs
   to the concurrently-running `uta2-r2` session, not this hand (zero tools
   paths in this hand's diff); tracked tree clean at HEAD `64314d7`.

## Not run (explicit)

- `python tools/testrun.py --baseline` and `python tools/bench_compare.py`:
  NOT run at this hand - milestone-closeout gates per the T1-T7b precedent
  and the AGENTS.md sizing rule. NOTE for the orchestrator: round 2 is
  live, so the ledger's "CLOSED 2026-10-19" row needs a re-closeout with a
  NEW ANCHOR after R2/R3 land.
- `encoding_guard.py --staged`: vacuous post-commit (clean tree); superseded
  by the `--all` run at HEAD (strict superset).
