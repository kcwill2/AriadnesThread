# UTA-T4 Verdict - transform + validate (provenance BLOCKER, write containment, tier/rollback coverage)

Range: base `80662cc` -> `8ef784a` (main hand) -> `3044c28` (journal rider).
Review date: 2026-10-18. Reviewer: review-gate (fresh session, full re-run).

## VERDICT: APPROVED

No blockers. Five minor findings (all non-blocking; two are journal wording, one
adjudication-level, two implementation/process observations). The UTA-B1 STOP
branch was correctly invoked and is independently CONFIRMED: provenance is
unreachable at `compute_rewrite` under the current data shapes.

## Ground truth (checkpoint_guard)

HEAD `3044c28`, branch main, tree clean except pre-existing untracked
uta-verdict/results artifacts; worktree census matches T1-T3 baseline.

## Fresh scoped gates (reviewer's own runs, canonical lines verbatim)

- `python tools/testrun.py --distill -- -p ariadne-transform` ->
  `32 passed / 0 failed / 0 ignored / 4 suites` (matches expected; baseline 24 + 2 containment + 6 pins)
- `python tools/testrun.py --distill -- -p ariadne-validate` ->
  `31 passed / 0 failed / 0 ignored / 5 suites` (matches expected; baseline 16 + items 4-6 = 15)
- `python tools/distill.py -- python tools/hygiene_check.py` -> `### generic PASS (exit 0)`;
  R1/R2 clean; all R3 warns pre-existing (none on new code lines; `tests.rs:162/225` are
  the pre-existing `for u in` loops inside preserved test bodies)
- `python tools/distill.py -- python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)`, capture 0 lines
- `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 232 tests in 19.097s` / `OK`
- Workspace-wide `--baseline` / `ui_smoke` / `bench_compare`: NOT run at this hand
  (mid-milestone H4 of 8; orchestrator's fresh-session closeout gates per AGENTS.md
  sizing rule; T1-T3 accepted precedent). Nothing here substitutes for them.

## Per-commit review (oldest-first)

### 8ef784a - `fix(transform): uta-t4 include provenance + write containment; test(validate): tier/rollback coverage`

6 paths; `+1215/-230`. Touch-set: `git diff --name-only 80662cc..HEAD` ->
tests.rs, write.rs, placement.rs (transform src), lib.rs, orchestration_tests.rs
(validate src), journal. All inside declared touch-set; no cli/pipeline/Cargo
touches. Message is misleading on "include provenance" - Finding 1.
Content verified item-by-item below.

### 3044c28 - `docs(tasks): uta-t4 journal gate record + commit truthfulness note`

Journal-only (+17/-3). Discloses the landed numstat, the encoding sequencing,
and the message-truthfulness note; ALSO self-corrects two wrong pre-rider
numstat claims (+154/-24 -> +147/-7; +131/-230 -> +138/-223), both matching the
reviewer's own `git show --stat`. Honest rider, accepted (mirrors the T3
numstat self-fix precedent).

## Audit item 1 - THE UTA-B1 BLOCKER (core risk) - INDEPENDENTLY CONFIRMED

Reviewer's own code read, independent of the journal:

1. `crates/cluster/src/plan.rs:34-45`: the ONLY symbol->origin record is
   `ProposedExtraction.source_file: Option<FileId>`, `#[serde(default)]` - older
   plan artifacts load as `None`.
2. `crates/transform/src/engine/bodies.rs:19-24`: `locate_bodies_for_unit`
   consumes `ext.source_file` (line 27) into a local map; the return type is
   `(BTreeMap<String, String>, Option<String>)` - bodies-by-name + primary text.
   The origin value cannot escape that call.
3. `crates/transform/src/engine/mod.rs:109-114`: `build_generated_unit` builds
   `ExtractedSymbol { name, qualified_id, kind, body }` (lib.rs:29-38) with NO
   origin field; `GeneratedUnit` (lib.rs:42-54) = path/symbols/required_deps/
   primary_source_text - no per-symbol origin anywhere.
4. `placement.rs:74-101`: `IncludeRewriter` holds only `BTreeMap<String, PathBuf>`
   (name -> generated header) + `Vec<GeneratedUnit>`; `compute_rewrite` arguments
   are name strings + `Vec<PathBuf>` old-include spellings. The buggy loop
   (107-116) blanket-covers EVERY old include whenever ANY symbol maps -
   the spec's CONFIRMED row (unit-test-audit.md:21-25) reproduces exactly.
5. Every production `IncludeRewriter::new` call site (grep):
   `cli/src/commands/transform.rs:95,186` via `plan_rewrite_targets`
   (247-266: builds the map straight from `proposed_extractions`, units with
   `symbols: vec![]` - membership itself gone); `cli/src/optimize.rs:313-333,393`
   (rebuilds from `unit.symbols` only); `cli/src/commands/ui/extraction_reader.rs:167`
   (same shape). None carries FileId or origin. Even if carried, FileId->path
   needs the repository catalog (absent from every transform data shape) and the
   D6 non-canonical spelling would still miss.

Conclusion: provenance is NOT derivable at `compute_rewrite`. The STOP branch
(handoff item 1: "If provenance is NOT derivable from available data shapes,
STOP: journal BLOCKER with the analysis and proposed data-shape change; ship the
other items") was correctly invoked. The proposed data-shape change
(`ExtractedSymbol.origin_source_file: Option<FileId>` copied verbatim by
`build_generated_unit`; caller-resolved `symbol_name_to_origin: BTreeMap<String,
PathBuf>`; cover iff origin == old_inc, else side_effect_preservations) is
sound: it matches the handoff's own target semantics ("only mark old_inc
covered when provenance shows the referenced symbol came from it"), preserves
the conservative direction, and honestly scopes the cross-crate touch (cli +
cluster + tests/integration - all outside this hand's touch-set) plus the D6
spelling coupling. The correct reason it cannot land here.

`compute_rewrite` byte-unchanged: CONFIRMED - the placement.rs range diff has
0 deleted lines and 34 added lines, all inside `mod tests` (the item-8 pin);
the function body at 97-154 is byte-identical to base. Journal wording
"placement.rs stays byte-unchanged" overstates - Finding 2.

## Audit item 2 - validate growth-guard move - ACCEPTED with wording finding

Byte-level (git cat-file bytes, difflib): lib.rs 929 -> 844 (<1000; new file
537 <1000). All 223 deleted lines fall inside the OLD `#[cfg(test)]` region
(cfg(test) at line 492; min deleted line 495) -> production code in lib.rs
untouched. All 9 moved fns (`include_of`, `consumer_source`, `NoTextMutationRewriter`,
`RecordingDelegatingExtractor`, the 3 apply/apply_with tests, helpers) have
stripped bodies BYTE-IDENTICAL; ZERO test fns lost anywhere. Caveat: 221/223
moved lines are present in `orchestration_tests.rs` only after re-indent, and
2 use-import lines (`CompilerValidator, ... SymbolKind,` / `ToolchainStatus,`)
were adjusted at move time -> "byte-identical" is overstated; substance holds.
Finding 3. Composition of the new file: moved cluster + item-5 + item-6 tests
(privacy of `collect_move_removals` preserved as a crate-root child module).
Growth guard justified: 929 + ~150 would cross 1000.

## Audit item 3 - disjunctive engine test (authorized edit) - CONFIRMED

Old `engine/tests.rs:212` (base blob): `has_errors || !has_cycle,` with
`let has_errors = ...` (192) and `let has_cycle = a_reqs_b && b_reqs_a;` (210) -
the disjunction as specified. All 7 deleted lines in the range sit inside
`test_acyclic_includes`; every OTHER pre-existing fn's stripped body is
unchanged (the `dummy_model` insert attribution is a difflib anchoring
artifact - body comparison shows no diff). New test pins mutual required_deps
AND `result.errors == vec!["Cycle detected in generated headers"]` exactly
(tests.rs:243-252); companion `extract_accepts_diamond_shape_without_cycle_error`
(256-281) pins the DAG A->B,A->C,B->D,C->D: `errors == []` + exact
required_deps per unit. Characterization-green by design (engine already
behaved; the test was the defect) - consistent with the journal.

## Audit item 4 - write.rs containment quality - PASS

`write_to_disk` (write.rs:11-31) validates the WHOLE batch before any IO
(lines 19-21 before `create_dir_all` at 24); `reject_escaping_unit_path`
(37-49) rejects absolute paths and any `Component::ParentDir`, error bails
with the offending path via `unit_path.display()`. Both red shapes have tests:
`write_to_disk_rejects_absolute_unit_path_before_any_io` (error text must
contain the full leak path; asserts no leak file AND `!output_dir.exists()` -
the before-any-IO claim is pinned, not just asserted) and
`write_to_disk_rejects_parent_escaping_unit_path` (`../..` shape, same shape
of assertions; both clean up before asserting so a red run leaves no stray
file). Nested relative paths stay allowed (`write_to_disk_creates_nested_parent_directories`
green). Item 7's mid-batch pin is intact and honest
(`write_to_disk_midbatch_io_failure_keeps_earlier_writes`: earlier write
survives, failing unit wrote nothing, Err surfaces - cleanup ownership left
with `apply_with`, per the comment). Red evidence exists (journal:
`17 passed / 2 failed`). Nuance: rejects ANY `..` component, stricter than the
handoff parenthetical - Finding 4.

## Plan-literal checks

- item 2: both shapes + error names offender: PASS (above).
- item 3: exact error string + diamond: PASS (above).
- item 4: 6 tests present in lib.rs test mod (structural Pass-on-empty /
  Fail-joined-errors; generated-headers under-dir / parent-only / nowhere /
  no-parent-no-panic): PASS.
- item 5: `collect_move_removals_offers_plain_move` + the five named guard
  skips + multi-unit-per-file: PASS (6 fns in orchestration_tests.rs).
- item 6: `rolled_back_error_carries_original_cause_when_rollback_fails`,
  `rolled_back_error_counts_processed_paths_when_rollback_succeeds`,
  `verify_restored_reports_state_mismatch_on_external_mutation`: PASS (3 fns).
- item 7: `write_to_disk_midbatch_io_failure_keeps_earlier_writes`: PASS.
- item 8: pin named EXACTLY `pin_is_noop_is_order_sensitive` (placement.rs
  mod tests; asserts reorder == NOT noop, identical order == noop; semantics
  decision left to owner per the code comment): PASS.
- item 9: `extract_reports_cycle_error_for_a_three_unit_cycle`,
  `extract_ignores_model_edges_to_symbols_outside_the_plan`,
  `unit_cycle_check_survives_dep_that_is_not_a_unit`: PASS.
- Refactor: `model_with_edges` + `required_deps_of` helpers; old inline
  closure copies are GONE (0 old fns missing, no dual representation - all
  pre-existing bodies verified identical except item 3's authorized fn).

## Frozen locks

Only the authorized item-3 test was edited (7 deleted lines, all inside
`test_acyclic_includes`). Validate: zero test fns lost; moved bodies identical
stripped; production lib.rs untouched. Placement existing test content
untouched (0 deletions). No test weakenings found anywhere in the range.

## Deviations - independent dispositions

- D1 item-1 BLOCKER (STOP branch): **ACCEPTED** - independently re-derived the
  full evidence chain above; STOP condition is factually true; the shipped
  package (journal analysis + proposed data-shape change, placement.rs logic
  unchanged, other items shipped) is exactly what the clause prescribes.
- D2 verbatim-relocation growth guard: **ACCEPTED** - touch-set compliant,
  production intact, bodies intact stripped, all counts reconcile (31 = 16 + 15).
  Wording overstated: Finding 3.
- D3 item-1 red fixture NOT shipped: **ACCEPTED** - with placement.rs unchanged
  the fixture's expectation would fail (not green-able) and pinning today's
  blanket behavior would freeze the confirmed bug; the STOP branch journals the
  analysis instead. Consistent with plan-literal ("ship the other items").
- D4 encoding-gate sequencing: **ACCEPTED** - mirrors H1/H2/T2/T3 accepted
  sequencing; reviewer's own `--all` at HEAD -> exit 0 is the strict-superset
  witness (`--staged` vacuous post-commit on a clean tree).

## Findings (all minor, non-blocking)

1. **Commit message misleading** (`8ef784a`): "include provenance" claims a fix
   that shipped as a BLOCKER. Adjudication: the handoff gate 5 prescribed the
   command verbatim, the implementer ran it verbatim AND self-reported the
   mismatch in the same commit's journal (T2 precedent accepted the reverse
   move - rewriting a message for truthfulness; here compliance was chosen,
   with disclosure, and the commit's own journal carries the honest record).
   Accepted as non-blocking. Consequence for the OWNER: the spec bug ledger
   (`docs/specs/unit-test-audit.md:146`, `UTA-B1 ... CONFIRMED -> H4 fix`) must
   NOT be read as fixed - see follow-ups.
2. **Journal internal contradiction**: line 51 "placement.rs stays
   byte-unchanged" vs placement.rs +34/-0 (the item-8 pin, correctly disclosed
   at journal line 184 and in the disk-state inventory as "compute_rewrite
   region" untouched). compute_rewrite IS byte-unchanged; the sentence names
   the file where it means the function.
3. **"Test names and bodies byte-identical" (journal line 102) overstated**:
   9/9 moved fn bodies identical after whitespace strip; 221/223 lines re-indented;
   2 use-import lines adjusted at move time. The claim invited byte-level
   verification and does not survive it; substance (verbatim bodies, zero
   losses, production intact) does.
4. **Containment stricter than the handoff parenthetical**: any `ParentDir`
   component is rejected (e.g. contained `a/../b.h`), vs "(contains `..` above
   dir)". Conservative, deterministic, no canonicalization TOCTOU; headline
   clause satisfied; not journal-declared as a deviation (arguably not one).
5. **Journal checkpoint thinness for items 7-9**: no per-item checkpoint lines
   (evidence lives in code comments + aggregate totals). Greppable and honest,
   below T1-T3's bar for itemized disclosure but within the "1-3 line
   checkpoint" letter via the aggregate gate record.

## Follow-up dispositions for closeout / ledger (recorded here; all out of this hand's touch-set)

- UTA-B1 bug-ledger row (spec:146) needs an owner update: `CONFIRMED -> H4 fix`
  -> `CONFIRMED -> BLOCKER (provenance unreachable at compute_rewrite)`.
- New cross-crate handoff for the provenance data-shape change
  (`ExtractedSymbol.origin_source_file` + caller-resolved origin map + D6
  spelling fix together + m18/m20 parity amendments). Touch: cli + cluster +
  transform + tests/integration.
- `ConsumerRewrite::is_noop` order-sensitivity semantics decision (pinned, not decided).
- validate `apply_with` can leave its created intermediate directories after
  rollback (journal observation, item-6a fixture deterministic).
- validate `FileSnapshot::capture` fails on a pre-existing directory target
  before the AR-04 decorated path (journal observation).

## Refusals observed (configuration note, not skipped gates)

Catch-all deny boilerplate
(`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`) hit
on: a `;`-chained `git log; echo; git diff` command, `python <script.py>`
direct execution, `git ls-files tasks/`, and `Remove-Item`. All expected gate
evidence was still obtained: plain individual git commands worked; the generic
Python checks ran via the sanctioned `python tools/distill.py -- python ...`
entry point (capture at `target/distill-capture.txt`). The reviewer scratch
`tasks/uta-t4-reviewer-verify.py` could not be deleted (no delete permission);
neutralized to a 2-line comment - closeout may remove it.
