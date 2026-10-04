# M23.5 T3-D2 Review Verdict

Unit: T3-D2, legacy/group include-builder parity lock. Crate `ui`, test-only.
Base `aebfa9b`, tip `b0a91d2`, branch `tarjan-s-scc`.
Reviewer scope: code audit + three scoped gates. The workspace-wide acceptance
pass is DEFERRED to T4.1 by documented decision; its absence here is not a
finding.

## A. Scope Confinement -- PASS

Range `aebfa9b..b0a91d2`, `git diff --name-only` returns exactly five paths:

```
crates/ui/tests/ui_test/export_order.rs
tasks/m235-t3d-execution-results.md
tasks/m235-t3d-verdict.md
tasks/m235-t3d2-handoff.md
tasks/m235-t3d2-journal.md
```

- `crates/ui/src/**`: ZERO paths. `dto.rs` and `cycle_groups_view.rs` absent, so
  both entry points and the shared emission tail are byte-unchanged and unit E's
  delete-and-rename plan is untouched. No production edit anywhere in the range.
- `crates/ui/tests/ui_test/main.rs`: absent (still 19 modules). Every other test
  module (`dto_builders.rs`, `graph_api.rs`, `d27_node_provenance.rs`,
  `common.rs`, the other fifteen): absent. `crates/ui/tests/**` outside
  `ui_test/`: absent.
- `crates/cli/**`, `crates/graph/**`, `crates/report/**`, `crates/cluster/**`,
  `tests/**`, `tools/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`:
  all absent.
- Commit `42bb44e` numstat: one path, `168  1
  crates/ui/tests/ui_test/export_order.rs`. Exactly as reported.

NO TEST REMOVED. The `-1` is precisely the permitted import line at
`export_order.rs:10` (the single-line `use ariadne_graph::{...}` replaced by a
3-line wrapped block adding `IncludeClosure`). The commit has exactly two hunks:
`@@ -7,7 +7,9 @@` (the import wrap) and `@@ -277,3 +279,168 @@` (a pure
append). No existing `#[test]` name, body, assertion or fixture was deleted or
altered anywhere. This matters because unit E deletes the legacy builder next
and depends on the surviving tests as its evidence base; that evidence base is
fully intact.

Line counts: 279 -> 446 (net +167). Far under the 1000-line ceiling.

Unit-E dependency preserved directionally: D2 added two NEW call sites of the
legacy builder (`export_order.rs:310` in P2, `:381` in P1) and touched none of
the seven pre-existing ones. The pre-existing legacy call-site count is still
exactly seven, so E's structural site table is not invalidated by a removal --
only extended (see finding F1c).

F12 fixture `tests/fixtures/basic/artifacts/includes/report.json`: absent from
all three commits and from the range name-only. Never committed.

Three of the five paths are `tasks/**` records outside the sec2 touch-set. See
the Dev-5 disposition under "Deviations D1..D6".

## B. Mutation Red Realness -- PASS

Executed, coherent, and independently corroborated. Not a fabricated Red.

**The mutation was genuinely one line.** The group builder's third argument sits
alone on its own line at `export_order.rs:385` (`&[],`) inside the rustfmt-wrapped
call, so replacing it with
`&cyclic_shuffled_file_graph().detect_cycle_groups(),` is a same-line-count edit
that keeps every later line number fixed.

**Independent proof of line-count preservation, from the recorded panic line.**
The journal's R2 panic is at `crates\ui\tests\ui_test\export_order.rs:433:5`. In
the committed, reverted file, line 433 column 5 is exactly the byte-lock
`assert_eq!` (4-space indent). If the mutation had added or removed a line, the
byte lock in the mutated tree could not have landed on the same line number as
in the reverted tree. The panic line number is therefore itself evidence that
(a) the mutation was confined to `:385` and (b) `:433` is the parity lock.

**The failure is at the parity lock, not at a precondition.** P1's five
non-vacuity asserts sit at `:392-426`, all upstream of `:433`, and all passed
under the mutation. That is correct, not suspicious: those asserts inspect the
legacy side's element count and decoration keys, and the mutation only changed
the GROUP builder's currency, so the legacy side was untouched. Under the
mutation both sides still emit a non-empty, decorated vector -- the preconditions
are currency-independent by construction, which is what makes them useful as
preconditions.

**The assertion message names the contract** (`:435-436`):
`parity: on acyclic input with empty cycle currency the group-based and legacy
builders must emit byte-identical elements`.

**The left/right difference is a coherent CONSEQUENCE of that mutation, verified
against source rather than accepted from the report.** `emit_include_elements`
(`dto.rs:659`) is the single shared tail and the ONLY input that differs between
the two entry points is the `IncludeCycleClass` view (`dto.rs:618` vs `:647`).
With empty currency, `IncludeCycleClass::from_paths(&[])`
(`cycle_groups_view.rs:32-53`) and `from_groups(&[], graph)` (`:63-88`) both yield
`grouped_nodes={}`, `cyclic_pairs={}`, `group_size_by_file={}`,
`witness_labels_by_file={}` -- structurally identical views, which is precisely
why unmutated P1 is green. Under the mutation, `from_groups` with one group
`{2,5,11}` sets `grouped_nodes={2,5,11}`, `group_size_by_file[each]=3`,
witness labels in `representative_cycle` order, and `cyclic_pairs` for every
intra-group edge of the cyclic fixture (`5->2`, `2->11`, `11->5`, `5->11`).
Emission (`dto.rs:697`, `:707`, `:717-722`, `:725`, `:731`, `:740`) then adds
`cycle_group_size`/`cycle_witness`, flips `is_cycle`, and sets
`classes=Some("cycle")` on those nodes, and flips the `5-2` edge. The journal's
reported keys -- `cycle_group_size:3`, `cycle_witness`, `classes:"cycle"`,
`is_cycle:true` present on the grouped side and absent on the legacy side, with
the `5-2` edge flipping -- are exactly what the source predicts, and `left` is
the grouped side because the committed source is
`assert_eq!(grouped_json, legacy_json, ...)` at `:433-434`. The test is not
comparing something unrelated to the mutation; it is comparing the one input the
mutation touched.

**The revert is byte-for-byte, and I reproduced the proof independently.**
`python tools/distill.py -- certutil -hashfile
crates/ui/tests/ui_test/export_order.rs SHA256` returns

```
35e9ab19bd4e72dec58a8ca9c7b1d46e126247fa4d06a05cad02d5bc0d053000
```

an exact match to the journal's pre-mutation AND post-revert value. Combined
with `checkpoint_guard.py status` showing the path absent from DIRTY FILES (so
the worktree file is content-identical to the committed blob), the committed
content IS that hash. The certutil-through-distill evidence suffices; hashing the
same path before and after is the correct construction, and I have confirmed the
result from the committed state rather than relying on the implementer's word.

**Trail coherence.** `185/0/0/1` -> `185/1/0/1` -> `186/0/0/1` is internally
consistent: R1 has P2 only (184+1=185); R2 adds P1 (186) with exactly one failure
(185 passing); G restores and all 186 pass. No line anywhere in the journal
reports `0 failed` while describing a failing test -- the T3-B finding-1
self-contradiction is absent.

## C. Test Proves What Unit E Needs -- PASS

**The fixture really is acyclic, and the test proves it rather than assuming
it.** `shuffled_file_graph()` (`export_order.rs:36-58`) builds 10 files with
exactly three edges: `5->2`, `1->9`, `3->6`. Targets 2, 9 and 6 have no outgoing
edges, so all three are one-hop sinks and no directed cycle is reachable. P1
asserts this from the tree at `:355-358` via
`graph.detect_cycle_groups().is_empty()`, with a message naming why. A cyclic
fixture would have made the test wrong rather than vacuous, and that cannot
happen here. The cyclic sibling `cyclic_shuffled_file_graph()` (`:129-151`, adds
`2->11`, `11->5`, `5->11`) is used only by P2, as specified.

**The group input really is empty on the unmutated path.** `:381` passes `&[]`
to the legacy builder and `:385` passes `&[]` to the group builder. Both views
collapse to the same empty classification (Section B), which is the property unit
E's deletion rests on.

**The comparison covers the full element payload, not a subset.**
`serde_json::to_string` runs over the whole `Vec<CytoscapeElement>` (`:431-432`).
Each element is `{ data: Value, classes: Option<String> }`, where `data` carries
`id`, `label`, `path`, `kind`, `closure`, `is_cycle`, `in_degree`, `out_degree`
(`dto.rs:701-710`) plus optional `language` (`:711-713`), `unit_kind`
(`:714-716`), `cycle_group_size` (`:717-719`) and `cycle_witness` (`:720-722`),
and `classes` (`:725`). Edge elements (`:732-742`) are inside the compared
bytes too.

**Non-vacuity is enforced BEFORE the lock and is self-falsifying** (`:392-426`),
so a green P1 cannot be a both-sides-empty pass:

- `nodes_with_closure > 0` (`:411`): `closures` decorates ids 5 and 1 with
  transitive sets of size 2 and 1, so exactly 2 of 10 nodes carry
  `closure > 0`. A wrong key name would yield 0 and fail.
- `nodes_with_language > 0` (`:419`): `file_languages` covers ids 1 and 3, so 2
  nodes carry `language`.
- `nodes_with_language < node_elements.len()` (`:423`): 8 nodes omit it, so the
  D27 OMITS rule is inside the compared bytes for both the present and the
  absent case. `file_unit_kinds` covers id 1 only, giving one node with both
  keys, one with `language` only, and eight with neither.
- `node_elements` cannot be empty, because the preceding
  `nodes_with_language > 0` assert would already have failed.

**The second test is genuinely additive, not a duplicate.** Proven by
call-site census rather than by reading prose: before D2 the legacy builder had
exactly seven call sites in `crates/ui/tests/ui_test/` --
`dto_builders.rs:15`, `graph_api.rs:136`, `d27_node_provenance.rs:141` and
`:152`, `common.rs:157`, `export_order.rs:91` and `:108` -- and every grouped
call site sits in a different test body. P1 and P2 are the ONLY test bodies in
the crate that invoke both builders on one graph, so no pre-existing test
performs a cross-builder serialization comparison. P2's fixture-sanity pair
(`:300-309`) does restate the neighbouring unit-A test at `:161-166`, but the
contract mandated exactly that, and P2's actual claim -- `assert_ne!` on the two
serializations -- is unique. Exactly +1 test, additive.

**Honest limitation recorded as required.** P1's header (`:342-349`) states it
does not lock cross-process emission determinism, explains that a hash-order
perturbation moves both sides equally, and cites `graph_api.rs:741-789` and the
CLI two-process lock as where determinism lives. The module doc at `:1-8` was
not amended, per contract, and its statement is true as written.

**One precision note (observation, non-blocking).** The workspace declares
`serde_json = "1.0"` with no `preserve_order` feature (`Cargo.toml:32`), so
`serde_json::Map` is a `BTreeMap` and both sides serialize keys in the same
sorted order. The byte lock at `:433` and the structural lock at `:442-445` are
therefore close to equivalent; the byte lock adds the serialized-form diff and
the `classes` spelling, not key ordering. The test's own comment at `:428-430`
overstates this slightly. The wording is inherited verbatim from handoff sec5.1
step 5, and the obligation -- byte-identity of the emitted payload -- is met
regardless, so this is recorded, not charged.

## D. Documentation Defect Adjudication -- PASS

The implementer followed the tree, renamed nothing, and recorded the
discrepancy. Verified independently:

**The three signatures, read from the tree at `b0a91d2`:**

| Function | Location | Third parameter |
|---|---|---|
| `build_include_elements` (legacy) | `crates/ui/src/dto.rs:608` | `cycles: &[Vec<FileId>]` (`:611`) |
| `build_include_elements_from_groups` (groups) | `crates/ui/src/dto.rs:637` | `cycle_groups: &[CycleGroup]` (`:640`) |
| `emit_include_elements` (shared tail) | `crates/ui/src/dto.rs:659` | `cycle_class: &IncludeCycleClass` (`:662`) |

Both entry points are 5-arg and both call the ONE private `emit_include_elements`
(`:615` and `:644`). A repo-wide grep for `pub fn build_include_elements` returns
exactly two hits, `:608` and `:637`. The handoff's own line references were
exactly right; the implementer's shape-based preflight (sec0.3) is what caught
the reviewer's error.

**The bad name exists only in documents.** `build_include_elements_legacy_paths`
occurs in exactly three places, all under `tasks/`: `m235-t3d-verdict.md:107`,
`m235-t3d-execution-results.md:24`, and this unit's journal recording the defect.
Zero occurrences under `crates/**` or `tools/**`.

**No rename and no shim were introduced.** No `#[deprecated]` alias, no wrapper,
no re-export. `crates/ui/src/**` has zero paths in the range, so nothing could
have been introduced there. The commit's only edits are the import wrap and two
appended test functions.

**The discrepancy is attributed, with a correcting source, not absorbed.** The
journal's Checkpoint 0 names the bad name, states the tree disagrees, cites the
correcting source (a shape grep of `crates/ui/src` at `aebfa9b`), and states
plainly that no function was invented and no rename performed -- and it correctly
identifies which document is at fault (the verdict text, not the handoff's
grep-by-shape preflight). Note the T3-D records cite the correct SPAN
(`dto.rs:604-622` is the legacy function's doc comment plus body) with the wrong
IDENTIFIER, which is the signature of a transcription slip, not tree drift.

**WHERE the correction must be recorded so the name stops propagating:**

1. `tasks/milestone_ledger.md` -- add an M23.5 deferred-findings row naming the
   defect, the two offending locations, and the correcting source. The ledger is
   the project's source of truth and currently carries no such row. This is the
   single place the correction must live.
2. `tasks/m235-t3e-handoff.md` -- **the bad name needs no correction there.** E
   correctly says "the path-based `build_include_elements`" (`:48`, `:117`) and
   never uses the wrong identifier. What E's handoff DOES need is correction is
   the D2-created count and line drift in findings F1a-F1d, which is the real
   propagation risk from this unit.
3. `tasks/m235-t3d-verdict.md:107` and `tasks/m235-t3d-execution-results.md:24` --
   committed, signed reviewer records. Do not silently rewrite them; the ledger
   row plus this verdict is the correction, so the audit trail shows the record
   was wrong and how it was fixed.

## Gate Results

Three scoped gates, each run in its own turn, per the documented lean-review
scope. Workspace-wide acceptance is deferred to T4.1 by decision.

| # | Command | Canonical result |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test` | `186 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `python tools/hygiene_check.py` | zero R1 lines, zero R2 lines; 120 R3 warn lines, all in files outside this unit's touch-set |
| 3 | `python tools/encoding_guard.py --all` | no output, no findings |

Gate 1 verbatim (capture `target/testrun-capture.txt`, 193 lines):

```
186 passed / 0 failed / 0 ignored / 1 suites
```

**None of the three is vacuous.**

- Gate 1: `1 suites` is correct and expected -- both new tests land in the
  existing `ui_test` target, and `crates/ui/tests/ui_test/main.rs` is byte-
  unchanged (absent from all three commits) and still declares exactly 19
  modules with `mod export_order;` at `:5`. So P1 (`:350`) and P2 (`:296`) are
  compiled and executed. The count moved 184 -> 186, which is itself proof the
  two new tests ran rather than being filtered: the handoff recorded zero
  `#[ignore]` under `ui_test/`, and the run reports 0 ignored and 0 filtered.
- Gate 2: the run is not empty and not silently skipped -- it emitted 120 lines,
  every one an R3 warn in a file this unit did not touch
  (`crates/build`, `crates/cli`, `crates/graph`, `crates/lang-*`,
  `crates/repo`, `crates/scheduler`, `crates/transform`, `crates/ui/src`,
  `crates/validate`, `tests/integration`, and six OTHER `ui_test` modules).
  **Zero lines name `crates/ui/tests/ui_test/export_order.rs`** -- no new R3
  warn was introduced in the file under review, which is exactly what handoff
  sec6 demanded. R1 (function/file length, line length) and R2 are both silent,
  so P1's 96 lines and the 446-line file are both inside the hard ceilings.
- Gate 3: `--all` scans tracked content and reported nothing, so no
  double-encoded UTF-8 mojibake and no non-UTF-8 blob exists anywhere in the
  repository -- which covers the pure-ASCII requirement on this unit's authored
  content.

Gate 1's `186` independently corroborates the handoff's own arithmetic (184 at
entry, two added, no test removed), and its `1 suites` corroborates that no new
test binary appeared.

All three gates were executed twice, each in its own turn, across this review's
two working sessions. Both passes produced byte-identical canonical results, so
nothing below is a single-sample reading. No gate was refused; no gate was
skipped; no gate was wrapped to evade a denial.

## Deviations D1..D6

**D1 (journal Dev-1) -- sizing: +167 landed against sec11's itemised +84.
ACCEPTED. Attributed to the planner, not the implementer.**
The journal's decomposition checks out against the tree: exactly two `#[test]`
functions (`:296`, `:350`), zero helpers, exactly one permitted import-line
edit, and every excess line inside mandated assertion messages or rustfmt-wrapped
map literals. There is no scope creep to charge. This is the second instance of
the defect class already recorded as `tasks/m235-t3d-verdict.md` finding 1
(predicted ~679, measured 734): additions estimated from anything other than the
mandated test bodies. Non-blocking.

**D2 (Dev-2) -- the `:10` import list became a 3-line wrapped block. ACCEPTED,
no semantic effect.** It is the only edit to any existing line, exactly as
permitted. One immaterial arithmetic imprecision for the record: the journal
says the unwrapped line "is 106 characters"; it is 104. The conclusion is
unchanged (it exceeds the repo's 100-column rustfmt width), and 104 is far below
the 2000-character R1 ceiling, so nothing turns on it.

**D3 (Dev-3) -- no fixture helpers extracted, so P1 is 96 lines against sec6's
~40-line readability guideline. ACCEPTED. Attributed to the planner.**
My adjudication: **this is a planner sizing error, not a hygiene finding
against the implementer.** Handoff sec6 sets the hard rule (<100 lines) and
offers ~40 as "a signal you have inlined something that should be a named
helper" -- with the explicit counter-clause that splitting assertions across
helpers would "hide the comparison". Handoff sec5.1 mandates six ordered steps
inside one function, and sec3.2 permits helpers ONLY if the maps cannot be
written inline (they can be). So the contract simultaneously mandated the
content and forbade the available remedy. The implementer chose the branch that
keeps the byte comparison visible in the test body, disclosed the judgement
call, and honoured the hard ceiling. Charging this to the implementer would be
charging it for a contradiction in its own contract. Line arithmetic verified
independently: P1 `:351-446` = 96 lines, P2 `:297-331` = 35 lines, headers 17
and 13 lines -- all exactly as reported. Non-blocking.

**D4 (Dev-4) -- encoding-gate ordering: `--all` immediately BEFORE each commit,
literal `--staged` immediately AFTER. ACCEPTED.** Pre-authorised by sec7 with
six named precedents (T2-A gate 5, T2-B D-E, T3-A D5, T3-B D5, T3-C D3, T3-D
D4). Independently correct: `commit_scoped.py` stages and commits atomically and
raw git staging is denied, so `--staged` genuinely cannot interpose, and `--all`
is strictly stronger (whole tracked tree vs index only). Correctly disclosed as
a pre-authorised substitution rather than dressed up as compliance.

**D5 (Dev-5) -- the third commit lands three `tasks/**` records outside the sec2
touch-set. ACCEPTED as an orchestrator action, with a template finding.**
`4d0f245` carries `tasks/m235-t3d-verdict.md` (1344/0),
`tasks/m235-t3d-execution-results.md` (371/0) and
`tasks/m235-t3d2-handoff.md` (638/0). Independently verified: all three are pure
additions with zero deletions, consistent with the journal's preflight UNTRACKED
listing, and the handoff's own 638-line length matches the file exactly. Each
was committed with an explicit `--scope`, and the reviewer role has read-only
git, so a reviewer cannot land its own records. This is an orchestrator dispatch
action, not implementer scope breach, and it matches accepted precedent (T3-B
`9f30c9a`, T3-D `3e1efa6`). It is nevertheless the over-broad-touch-set shape
that the review-gate commit check exists to catch, and the fact that it needed
an out-of-band waiver to be legal is the actual defect. Recommendation (F6), not
a code change.

**D6 (Dev-6) -- journal fidelity: the R2 `left`/`right` blobs are quoted by
identifying key rather than pasted. ACCEPTED.** Each blob is roughly 2.5 KB, so
pasting them would itself breach the <2000-character source-line rule (R1). The
`panicked at` and `assertion ... failed:` lines are verbatim. I reconstructed the
same key set independently from `cycle_groups_view.rs` and `dto.rs` (Section B),
so the summarisation loses nothing verifiable.

**Review-gate step 8 (report evidence): PASS.** The journal is digest-shaped --
canonical lines, capture paths (`target/testrun-capture.txt`,
`target/distill-capture.txt`), verdict descriptions, and zero raw cargo or git
pastes. Its one deviation from full quotation is D6, justified with a cited
rule.

**Journal SHA self-reference: PASS.** The journal's final paragraph states that
8b's own SHA cannot appear inside 8b, names only the two prior SHAs, and points
at `git log -1` for the tip. Confirmed: `b0a91d2` is the range tip and does not
appear in the journal's own commit table. No amend, no paradox.

**Workspace-line discipline: PASS.** The journal states `1180 passed / 0 failed /
4 ignored / 88 suites` as a PREDICTION and explicitly not measured, deferring
measurement to the reviewer. Correct, and this review makes the same deferral to
T4.1.

**Fixture (F12): ACCEPTED the implementer's handling.** Independently verified:
the path `tests/fixtures/basic/artifacts/includes/report.json` appears in no
commit of this range, and `.gitattributes` (7 lines, byte-unchanged) pins only
`crates/ui/tests/fixtures/*.json` and `tests/golden/**/*.json` -- the path
matches neither pattern, so it is genuinely unpinned while `core.autocrlf=true`
holds in this worktree. F12's stated mechanism is exactly right. Adjudication:
when the content numstat is clean, doing nothing is the correct action; a
Read/Write "restore" of a byte-identical file is a no-op at best and a
re-dirtying write at worst. The implementer neither restored nor attempted a
restore, and disclosed that it did not. F12 is a configuration defect (missing
pin plus autocrlf) and closes at operator closeout, not here.

## Per-Commit Evidence

Oldest first, each judged alone. Base `aebfa9b`, tip `b0a91d2`.

**1. `42bb44e` -- "test(ui): lock legacy/group include-builder parity on acyclic
graphs"**
numstat: `168  1  crates/ui/tests/ui_test/export_order.rs` -- exactly one path,
inside the sec2 touch-set. Two hunks: `@@ -7,7 +7,9 @@` (import wrap) and
`@@ -277,3 +279,168 @@` (pure append). Reviewed in full. Contains exactly the
two mandated `#[test]` functions and nothing else: no production code, no third
test, no helper module, no `main.rs` edit. The single deleted line is the
pre-existing `:10` import. Commit-vs-touch-set: CLEAN. Judged APPROVED.

**2. `4d0f245` -- "docs(tasks): land M23.5 T3-D reviewer records and T3-D2
handoff (orchestrator-declared)"**
numstat: `371/0` + `1344/0` + `638/0`, all `tasks/**`, all pure additions.
Three paths outside the sec2 touch-set, adjudicated under D5 as an
orchestrator-declared action with accepted precedent. No code, no fixture, no
lockfile. Judged APPROVED with finding F6 attached to the handoff template.

**3. `b0a91d2` -- "docs(tasks): M23.5 T3-D2 journal -- parity lock RGB trail,
mutation/revert hashes, gate verbatims, deviations"**
numstat: `243/0  tasks/m235-t3d2-journal.md` -- one path, inside the touch-set
(sec2 names the journal). Digest-shaped content, no raw output, deviations
declared. Its own SHA correctly absent. Judged APPROVED.

**Cross-cutting frozen locks, all verified from `git diff --name-only
aebfa9b..b0a91d2`:** `crates/ui/src/dto.rs` absent; `cycle_groups_view.rs`
absent; `crates/ui/tests/ui_test/main.rs` absent (19 modules confirmed);
`graph_api.rs`, `dto_builders.rs`, `d27_node_provenance.rs`, `common.rs` and the
other fifteen test modules absent; `crates/cli/**`, `crates/graph/**`,
`crates/report/**`, `crates/cluster/**`, `crates/lang-scala/**` absent;
`tests/fixtures/**` and `tests/golden/**` absent; `crates/ui/Cargo.toml`,
`Cargo.toml`, `Cargo.lock`, `.gitattributes` absent; no `crates/ui/assets/**`
path, so zero new hex literals is structural as well as reported. Both public
entry points intact at `dto.rs:608` and `:637`, 5-arg each, sharing the one
`emit_include_elements` at `:659`.

## Verdict

**APPROVED.**

Zero blocking findings against this unit. Sections A, B, C and D each PASS on
independent evidence. The unit's entire value -- changing exactly one test file
and nothing else -- holds: one test file, +168/-1, the single deletion being the
permitted import line, zero production edits, zero tests removed, zero renames,
the F12 fixture in no commit, and the two-commit-plus-declared-records form with
no amend.

The mutation Red is real and executed, and I confirmed it three ways the journal
did not: the R2 panic line `:433:5` coincides with the byte lock's own position
in the reverted file (which is only possible if the mutation preserved line
count), the left/right key differences are exactly what
`IncludeCycleClass::from_groups` versus `from_paths` predicts from source, and I
reproduced the revert hash `35e9ab19...` from the committed state myself rather
than accepting the `certutil` claim. On the SHA question the prompt raised: the
implementer's evidence suffices, and it is now corroborated from the blob.

Numbered findings, all non-blocking for T3-D2. F1 and F2 are blocking for the
NEXT unit and must be cleared before T3-E is dispatched.

**F1 (non-blocking here; BLOCKING for T3-E dispatch) -- architect/planner.
`tasks/m235-t3e-handoff.md` is stale against the tree this unit just landed, in
four concrete places.** (a) sec4 `:160-161` and sec7 `:226` pin gate 1 at `184
passed / 0 failed / 0 ignored / 1 suites`; the true count is now **186**, so a
correct implementer would record a false gate failure. (b) sec5 `:167-168` and
`:171-172` pin "179 pre-existing plus unit A's 5" and "184 at entry and 184 at
exit"; entry is now 186. (c) sec0.3 `:34-40` preconditions "The expected
remaining callers are exactly the **seven** inside `crates/ui/tests/ui_test/`";
there are now **NINE** legacy call sites -- the seven listed plus
`crates/ui/tests/ui_test/export_order.rs:310` (P2) and `:381` (P1). An
implementer obeying sec0.3 verbatim stops and reports; one that does not leaves a
dangling reference to a deleted function. (d) sec1's table `:70-71` cites
`export_order.rs:88` and `:105`; those sites were `:89` and `:106` at `aebfa9b`
(a 1-line pre-existing drift) and are `:91` and `:108` now, a 3-line drift after
this unit's import wrap. All four must be corrected in
`tasks/m235-t3e-handoff.md` and recorded as a `tasks/milestone_ledger.md` row.

**F2 (non-blocking here; must be DECIDED before T3-E) -- architect/planner.
P1 becomes a tautology after E.** E sec3.2 (`:120`) renames
`build_include_elements_from_groups` to `build_include_elements`. P1 at `:381-382`
would then invoke the SAME function twice with the same `&[]` third argument, so
the "parity lock" would compare a function with itself -- precisely the vacuity
this unit exists to prevent, and it would still pass green. E sec5 `:167-168`
also requires every test body "byte-identical in name and body except for the
callee name at the seven call sites", which cannot be satisfied for
`export_order.rs`. E needs an explicit instruction for the two D2 tests. P2
survives E meaningfully (it passes `&[]` versus `&groups` to one name, so it
still discriminates); P1 does not. Deleting P1 at E and retaining the R2 trail as
the recorded evidence is the obvious resolution, but that is the architect's
call, not E implementer's.

**F3 (non-blocking) -- planner.** D1: landed +167 against sec11's itemised +84.
Second instance of the T3-D finding-1 defect class. Recommend the handoff-spec
template require per-test-body line estimates derived from the mandated bodies.

**F4 (non-blocking) -- planner.** D3: P1's 96 lines are a consequence of sec5.1
mandating six ordered steps in one function while sec3.2 forbade the helper
extraction that would have shortened it. Adjudicated as a contract contradiction
and a sizing error, NOT as implementer hygiene.

**F5 (non-blocking) -- configuration.** F12: `tests/fixtures/basic/artifacts/
includes/report.json` is unpinned by `.gitattributes` under `core.autocrlf=true`.
Correctly handled (no restore attempted, path in no commit, disclosed). Closes at
operator closeout, not here.

**F6 (non-blocking) -- planner/template.** D5: three `tasks/**` records outside
the declared touch-set required an out-of-band orchestrator waiver to be legal.
Recommend the handoff-spec template carry a standing clause permitting the
orchestrator to land reviewer records and the unit's own handoff, so the next
unit does not need an ad-hoc exception.

Adjudications requested:
- A Scope Confinement: PASS.
- B Mutation Red Realism: PASS (mutation executed, one line, failure at the
  parity lock at `:433` with the acyclic-empty-currency contract message, the
  left/right difference a verified consequence of the mutation, revert
  byte-for-byte and independently reproduced).
- C Test Proves What Unit E Needs: PASS (fixture acyclic and asserted from the
  tree, group input empty unmutated, full element payload compared, message names
  the contract, P2 additive and proven so by call-site census).
- D Documentation Defect: PASS (three signatures verified, no rename or shim,
  defect attributed with a correcting source). The bad name
  `build_include_elements_legacy_paths` must be corrected in
  **`tasks/milestone_ledger.md`** as an M23.5 deferred-findings row naming
  `tasks/m235-t3d-verdict.md:107` and
  `tasks/m235-t3d-execution-results.md:24` with `crates/ui/src/dto.rs:608` as
  the correcting source. `tasks/m235-t3e-handoff.md` does NOT need the name
  corrected -- it names the function correctly -- but it does need F1's four
  count and line corrections. The two T3-D records are signed reviewer documents
  and should not be silently rewritten; the ledger row plus this verdict is the
  correction of record.
- D1 ACCEPTED (planner) / D2 ACCEPTED / D3 ACCEPTED (planner) / D4 ACCEPTED /
  D5 ACCEPTED (orchestrator, F6 raised) / D6 ACCEPTED.

Deferred, on the record and not charged here: `python tools/testrun.py
--baseline`, `python -m unittest discover -s tools -p "*_tests.py"`,
`python tools/ui_smoke.py`, `python tools/bench_compare.py`. This review's scope
was deliberately lean by documented decision; the workspace-wide acceptance pass
runs in full at T4.1. Ledger rows F9, F10, F11 and F12 remain open for operator
closeout.