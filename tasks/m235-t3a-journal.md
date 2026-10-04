# M23.5 T3-A Implementer Journal - Group-Based Include-Element DTO (crate:ui)

**Handoff:** `tasks/m235-t3a-handoff.md` (Phase 3 unit A of 5, plan T3.2 first half)
**Worktree:** `.kilo/worktrees/tarjan-s-scc`, branch `tarjan-s-scc`, base HEAD `b950963`
**Authorizations:** this handoff + `tasks/m235-kickoff.md` + M23.5 milestone row
(`tasks/milestone_ledger.md`) + orchestrator task message declaring the
Phase-2 record files and planning artifacts into this unit's commit scope.

## Checkpoint 0 - preflight (before any edit)

`python tools/checkpoint_guard.py status` at session start:
- HEAD `b950963`, branch `tarjan-s-scc`.
- DIRTY: `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` (1/1) and
  `tasks/milestone_ledger.md` (5/5). Inspected via the read-only
  `git diff HEAD` MCP tool: BOTH are planner-authored records (the plan's
  T2.1 AMENDED row; the M23.5 milestone row + updated F9/F10/F11/F12 deferred
  rows) left uncommitted at kickoff. Orchestrator declared them into this
  unit's commit scope. NOT touched by the implementer.
- UNTRACKED: `tasks/m235-t2b-execution-results.md`, `tasks/m235-t2b-verdict.md`,
  and the five `tasks/m235-t3{a,b,c,d,e}-handoff.md`.
  FINDING: the orchestrator message also names `tasks/m235-t2a-verdict.md`
  and `tasks/m235-t2a-execution-results.md`, but both are ALREADY TRACKED
  (committed by the T2-B implementer in `08741b8`, per T2-B verdict sec 2).
  Including their globs in a scope is therefore a content no-op; recorded,
  not "restored".
- Fixture preflight (handoff step 0.2, `tests/fixtures/basic/artifacts/
  includes/report.json`): the numstat dirty section does NOT list the path
  BEFORE any test run -> nothing to restore. BEFORE-STATE: clean
  (no numstat rows). F12 predicts the stat entry may re-dirty after a full
  run; after-state check is scheduled post-gates.

## Checkpoint 1 - Red (both Reds genuinely observed)

Wrote R1 + R2 in `crates/ui/tests/ui_test/dto_builders.rs` against the
EXISTING `build_include_elements` (compiling, real assertions, no todo!).
Both observed failing for exactly the predicted semantic reason:

- R1 `intra_group_non_tree_edge_carries_the_cycle_class`:
  `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test
  ui_test -- intra_group_non_tree_edge_carries_the_cycle_class` ->
  `0 passed / 1 failed / 0 ignored / 1 suites` (exit 1). Panic at
  dto_builders.rs:477 naming the chord pair: "intra-group chord edge
  (a.h -> c.h), pair (1, 3), must carry is_cycle = true; left: Bool(false)".
  NOTE (verbatim-drift, immaterial): the handoff predicted "assertion (2)"
  fails; the FIRST assertion (is_cycle) fails before the classes assertion
  and already names the pair - same failure class, pair named as required.
  Capture: `target/testrun-capture.txt` (superseded by the R2 run below;
  this digest line is the retained evidence).
- R2 `node_cycle_data_reports_group_size_and_witness_only_for_grouped_nodes`:
  same form with its exact test-name filter -> `0 passed / 1 failed /
  0 ignored / 1 suites` (exit 1). Panic at dto_builders.rs:508: "grouped node
  b.h (FileId 2) must expose cycle_group_size = 3; actual data map:
  {closure,id,in_degree,is_cycle,kind,label,out_degree,path}" - names the
  missing key and prints the actual node data map, self-explaining.
  NOTE: handoff predicted `data["cycle_group_size"]` "is Null"; the assertion
  used `data.get(...)` which honestly reports `None` (key absent, left: None).
  Same missing-key semantics. Capture: `target/testrun-capture.txt` (23 lines).

Both testrun filters used `--test ui_test` (a real binary) plus the FULL
exact test name after `--`; each selected exactly 1 test (non-vacuous, shown
by `1 failed`), per handoff sec 7's filter warning.

## Deliberate-shape record (decisions made from the artifacts, pre-empting review)

- **Touch-set extension DECLARED per handoff sec 6:** `dto.rs` is 852 lines
  at base; the in-file additions (classification view + both wrappers +
  shared emitter + docs) measure ~1011 total > the 1000 R2 ceiling, so the
  sanctioned extraction branch is taken: group-membership classification
  lands in NEW `crates/ui/src/cycle_groups_view.rs` plus the ONE-LINE
  `pub(crate) mod cycle_groups_view;` in `crates/ui/src/lib.rs`. Reason
  (one line): ceiling pressure - keep `dto.rs` under ~940. No other lib.rs
  change; both public entry points stay in `dto.rs` (re-exported by
  `pub use dto::*;`), and `tests/ui_test/main.rs` stays byte-identical.
- **Test 3 uses hand-built `CycleGroup` literals deliberately** (two disjoint
  groups, member_files superset of representative_cycle): the property locked
  is "classification reads member_files, not witness or reported paths", and
  a real producer group can carry a member off the witness on some graphs
  but the DTO must not care even for that input. The real producer IS
  exercised by R1/R2 (re-pointed), tests 4 and 5 via `detect_cycle_groups()`.
- **`cycle_witness` label resolution:** witness members that the graph never
  registered in `file_paths` (possible per T2-B's divergence case) have no
  path label; they are OMITTED from the label array (documented in code,
  journal record). Emitted-node membership never depends on it (unregistered
  members get no node at all).
- **Encoding-gate interposition:** `commit_scoped.py` has no stage-only mode
  (verified in `tools/commit_scoped.py`), so `encoding_guard.py --staged`
  cannot run between stage and commit; per T2-A/T2-B accepted-deviation
  precedent (D-E) the stronger `--all` form is run pre-commit AND the
  literal `--staged` gate is executed (vacuously clean after commit).

## Checkpoint 2 - Green + re-point (the handoff sec4 explicit step)

Implemented `build_include_elements_from_groups` (contract shape verbatim)
and the shared emission tail:
- NEW `crates/ui/src/cycle_groups_view.rs` (149 lines):
  `IncludeCycleClass::from_paths` (byte-compatible legacy rule) and
  `from_groups` (IV-2 node rule; IV-3 edge rule as group-INDEX equality via
  `endpoints_share_group`, so a group-1 -> group-2 edge stays unclassed;
  self-loop = trivial index equality; D27 OMITS honoured -- ungrouped nodes
  get neither new key).
- `crates/ui/src/dto.rs`: `build_include_elements` body replaced by a thin
  wrapper over `emit_include_elements` (signature UNCHANGED, contract text
  preserved, new callers directed to the group builder); new
  `build_include_elements_from_groups` wrapper; ONE node loop + ONE edge loop
  in the shared emitter. dto.rs: 852 -> 901 lines (net +49, under the ~940
  extraction threshold BECAUSE the view was extracted; without the branch it
  measured ~1011 > 1000 R2). `crates/ui/src/lib.rs`: the ONE sanctioned line
  `pub(crate) mod cycle_groups_view;` (declared extension, see above).
  `tests/ui_test/main.rs` untouched.
- RE-POINT (journal step required by handoff sec4, phase-2 T2-A precedent):
  R1 and R2 in `dto_builders.rs` switched from `build_include_elements`
  (cycles = reported path `vec![vec![a,b,c]]`) to
  `build_include_elements_from_groups` with a REAL group from
  `graph.detect_cycle_groups()` via `single_group_containing` -- the real
  producer, not a hand-built literal, as sec4 prescribes. No assertion was
  weakened: every R1/R2 assertion that failed in Checkpoint 1 is present in
  the committed tests and now passes against the new API.
- Added the remaining required tests (3, 4, 5) and extended the existing
  D27 test (no sixth #[test]); test-count arithmetic 179 + 5 = 184.
- Gate 1 first attempt returned testrun exit 2 `parse format drift` caused
  by my own fixture-message literal braces in one assert message (compile
  error, capture names export_order.rs:163) -- an ordinary Green-phase
  compile iteration (T2-A deviation D precedent), fixed and re-run.
- Gate 1 GREEN: `python tools/testrun.py --distill --timeout 1200 -p
  ariadne-ui --test ui_test` -> `184 passed / 0 failed / 0 ignored /
  1 suites` (capture target/testrun-capture.txt, 192 lines).
- Gate 2 GREEN: `python tools/testrun.py --distill --timeout 1200 -p
  ariadne-ui` -> `184 passed / 0 failed / 0 ignored / 3 suites` (204-line
  capture). Both pinned lines match the handoff table verbatim.
- Gate 3 GREEN: `python tools/distill.py -- cargo build -p ariadne-ui` ->
  `ok (exit 0, 0 warnings emitted)` (capture target/distill-capture.txt).
  Dead-code posture: every pub(crate) item in cycle_groups_view.rs is called
  from the shared emitter, so the warning-free line is load-bearing.

## Checkpoint 3 - hygiene, theme, encoding (pre-commit gates)

- Gate 4 `python tools/hygiene_check.py`: ZERO R1/R2 lines workspace-wide.
  Every printed line is a warn-only R3 on pre-existing lines. Cross-checked
  each flag in files I touched: dto.rs:330/418/445 (single-letter `s`) are
  legacy justification functions, outside my region (598-748);
  graph_api.rs `res` flags are all at lines 52-231 (legacy route tests), my
  additions start after line 698. dto_builders.rs, export_order.rs,
  d27_node_provenance.rs, cycle_groups_view.rs: zero flags. No R3 in any
  line I authored.
- Gate 5 `python tools/theme_token_check.py --check-added --base b950963`:
  NO OUTPUT = zero added hex literals. No CSS/JS/template file touched;
  semantic verdict colors are structurally untouched; only red-edge
  COVERAGE grows (plan reviewer-check).
- Encoding pre-commit: `python tools/encoding_guard.py --all` -> no output
  (stronger-than-staged form, run before staging per D-E precedent recorded
  above; the literal `--staged` gate runs after the commits).
- DEVIATION DECLARED (required for commit self-consistency, sec6 branch):
  gate 8's literal `--scope` list predates the extraction decision and names
  the six touch-set paths; the commit therefore carries the SIX + the TWO
  sec6-sanctioned extension files (`crates/ui/src/lib.rs` one mod line,
  NEW `crates/ui/src/cycle_groups_view.rs`). A commit without them would
  reference a module it does not contain (unbuildable commit); the sec6
  extension clause is the authorization and this journal is its declaration.

## Checkpoint 4 - commits

- COMMIT A (gate 8): `31513b9` "feat(ui): add group-based include element
  builder with cycle group data" -- carries exactly the 6 touch-set paths
  plus the 2 sec6-declared extension files (dto.rs +65/-16, dto_builders.rs
  +283/-2, graph_api.rs +94/-2, export_order.rs +92/-1,
  d27_node_provenance.rs +61/-1, journal, cycle_groups_view.rs +149/-0 new,
  lib.rs +1/-0). `tests/fixtures/basic/artifacts/includes/report.json` is in
  NO commit of this unit.
- COMMIT B (this row's subject, recorded after creation): planning record
  commit carrying the planner-authored dirty files (plan AMENDED row,
  milestone ledger rows) + the five untracked m235-t3*-handoff.md +
  m235-t2b-verdict.md + m235-t2b-execution-results.md (all orchestrator-
  declared) + this journal SHA fill-in; its SHA is `PENDING-B` -- filled by
  checkpoint_guard `status` RECENT COMMITS in the same row below, per the
  T2-A/T2-B row-2 precedent (T2-B verdict finding 5 accepted that form).
- Post-commit gates: `python tools/encoding_guard.py --staged` (literal
  gate-6 form, executed after the scoped commits -> no output) and the
  after-state `python tools/checkpoint_guard.py status` follow this commit;
  the fixture after-state check is in Checkpoint 5.
