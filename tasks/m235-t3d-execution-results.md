# M23.5 T3-D Execution Results (acceptance-criterion addendum)

Unit: M23.5 Phase 3 unit D of 5 - IV-11 policy port + **atomic D1-B delete** of
`detect_cycles`/`dfs_cycle`, crate `graph`.
Handoff: `tasks/m235-t3d-handoff.md`. Journal: `tasks/m235-t3d-journal.md`.
Verdict: `tasks/m235-t3d-verdict.md` - **APPROVED, 0 blocking / 13 non-blocking**.
Range reviewed: `6c0c2b6..aebfa9b` (base = T3-C tip). Branch `tarjan-s-scc`,
worktree `.kilo/worktrees/tarjan-s-scc`.

Every line below is **reviewer-measured**, not relayed from the journal. Commands
are cited by name; capture path `target/testrun-capture.txt` /
`target/distill-capture.txt` unless stated.

---

## 1. Acceptance criteria 1, 2 and 6 (plan sec3) - what actually happened

### Criterion 1 - `detect_cycles` no longer exists; `CycleGroup` is the only cycle representation

| Clause | Outcome | Evidence |
|---|---|---|
| `detect_cycles` gone | **MET** | Reviewer grep over `crates/**` for `fn detect_cycles\|fn dfs_cycle\|\.detect_cycles(\|Self::dfs_cycle\|::dfs_cycle(` returns **2 hits**: one doc comment in the frozen oracle, one unrelated `crates/transform` fn. **Zero definitions, zero call sites.** `graph.rs` itself: **zero** occurrences. `tests/`: **no files found**. |
| delete atomic with its last repoint | **MET** | `git show --stat acea7f2` carries `closure_equivalence_tests.rs` +1/-1, `cycle_groups.rs` +3/-2 and `graph.rs` +9/-61 **in one commit**. `adb53a3` (the preceding commit) contains **no deletion at all**. **No commit in the range leaves the build broken.** |
| `CycleGroup` the ONLY representation | **PARTIAL - by design, owned by T3-E** | The only **detector** is `detect_cycle_groups`. One path-based **classification** view survives: `crates/ui/src/dto.rs:604-622` (`build_include_elements_legacy_paths`), documented in-source at `crates/ui/src/cycle_groups_view.rs:3-9` as "retired in T3-E". `tasks/m235-t3e-handoff.md` exists and owns that deletion. **Criterion 1 is NOT closable at T3-E's dispatch** - recorded as verdict finding 8. |

Evidence line: the 15 surviving `detect_cycles` mentions across `crates/**`,
exhaustively classified by the reviewer as: frozen IV-6 oracle
(`crates/graph/tests/cycle_groups_equivalence_test.rs`, 11 hits),
`crates/graph/src/cycle_groups.rs:64` (doc), `crates/graph/src/closure_equivalence_tests.rs:67`
(fixture label string), `crates/graph/tests/cycle_groups_test.rs:5/:101/:142`
(doc/Red-evidence comments), `crates/transform/src/engine/mod.rs:72/:133`
(**different function**), `crates/cli/tests/payload_tail_bench.rs:62/:64`
(stale prose, report-only).

`detect_cycles_in_units` was read in full
(`crates/transform/src/engine/mod.rs:133-164`): takes `&[GeneratedUnit]`, builds a
`BTreeMap<PathBuf, BTreeSet<PathBuf>>`, delegates to a nested free fn
`has_cycle`, returns `bool`. **Different type domain, different return type,
different crate. Not conflated.**

### Criterion 2 - `cycle_count` reports entangled-group count; `m3_include_graph_test` green without edit

**MET, and this is the criterion the predecessor's finding 5 was about.**

| Sub-clause | Outcome | Evidence |
|---|---|---|
| `cycle_count` carries GROUP count | **MET** | `crates/graph/src/graph.rs:325` reads `cycle_count: detected_groups.len(),`, fed by `let detected_groups = self.detect_cycle_groups();` at `:312`. Zero-context diff hunk `@@ -369 +325 @@` (line moved only because 44 lines of recursion above it were deleted). |
| field NAME survives (IV-9) | **MET** | `types.rs:56` `pub cycle_count: usize`; `types.rs` absent from every commit in the range. |
| `m3_include_graph_test` green WITHOUT edit | **MET** | `git diff b950963..aebfa9b -- tests/integration/tests/m3_include_graph_test.rs` -> **empty**. Blob SHA-1 `b6a54d5881dcf3d82ae8fcf56885fabd7e892a23` **identical at both revs**. **Zero differing bytes.** |

Evidence line (gate 5, the IV-9 acceptance gate, run AFTER the port):
`3 passed / 0 failed / 0 ignored / 1 suites`, capture naming
`test_m3_edge_cases_duplicates_system_missing_cycles ... ok` and `0 filtered out`
- the exact carrier of `report.cycle_count > 0` at `m3_include_graph_test.rs:118-121`.
**This is the first run in the milestone where that assertion was witnessed over a
group count rather than a path count**, which is what T3-C's finding 5 asked for.

The semantic-shift half has its own falsifier: **R1**
(`graph.rs:588-600`) asserts `cycle_count == 1` on `double_reporting_scc()`, a
fixture the frozen oracle enumerates as **two** paths (`[1,2,3]` and `[2,3]`) for
**one** SCC. A reversion to path semantics turns it red with `left: 2, right: 1`.

### Criterion 6 - Cycle-policy seam preserved (IV-11)

**MET.**

| Sub-clause | Outcome | Evidence |
|---|---|---|
| callback signature preserved | **MET** | `graph.rs:266-269` `generate_report_with_cycle_policies(&self, policy_of: &dyn Fn(FileId) -> CyclePolicy)` - **not in the diff**. |
| classification rule migrated to groups | **MET** | `graph.rs:249-257` `group_is_informational(group: &CycleGroup, policy_of: ...)` -> `group.member_files.iter().all(|&f| !matches!(policy_of(f), CyclePolicy::Reject))`. Name is the one plan sec1.4 mandated and T2-A's deviation E deferred; it is **live**, called at `:315` as `Self::group_is_informational` (inherent associated fn shape preserved, not converted to a free fn). |
| `informational_cycle_count` name + skip-when-zero serde shape | **MET** | `types.rs:61-62` attribute and `types.rs:69-71` `informational_cycles_absent` predicate read at HEAD; `types.rs` absent from every commit. |
| six unit tests ported, values unmoved | **MET** | Zero-context diff over the six-test range: **exactly one line changed**, test 1's assertion **message**. Compared values (`report.cycle_count`, `1`) character-identical. Tests 2-6 not in the diff at all. Stances at HEAD: `1/1` (`:644-648`), `1/0` (`:661-662`), `1/1` (`:670-671`), `1/0` (`:677-678`), and both serde locks byte-for-byte (`:686-696`, `:709`). |
| all-`Reject` C++ artifact byte-stability | **MET** | `generate_report()` untouched at `graph.rs:242-244`; gate 3's capture names `all_reject_policy_json_is_byte_identical_to_default_report ... ok`; the workspace capture names `golden_baseline_basic ... ok` and `golden_baseline_expensive ... ok` - **no golden drift**. |
| m20/m22 stance preserved | **MET** | Workspace capture names `fixture_cycle_is_informational_under_tolerate_policy ... ok` (m20) and `auto_shaped_graph_keeps_the_cycle_informational ... ok` (m22) - **both Scala consumers green against the ported report layer**, with no plugin-side change (`crates/lang-scala/**` absent from the range). |

---

## 2. Adversarial probes the brief asked for - explicit answers

### 2.1 EMPTY-group vacuity - `all()` over an empty group: **structurally impossible**

`Iterator::all` on an empty iterator is `true`, so an empty group **would**
classify informational. It cannot exist. Two independent barriers in the producer:

1. `crates/graph/src/cycle_groups.rs:297` - `group_is_cyclic` matches
   `[] => false`. **An explicit, named rejection of the empty case.**
   `filter_cyclic_groups` (`:282-290`) drops any empty group.
2. `cycle_groups.rs:236-254` - `emit_group_if_root` pushes `popped` (including
   the root `vertex`) and only breaks after pushing `vertex`, so
   `group.len() >= 1` always.

A third barrier: `order_reverse_topological` indexes `groups[index][0]`
(`:323`, `:332`) as the heap key and would **panic** on an empty group;
`group_count == 0` short-circuits to `Vec::new()` at `:316-318`.
`CycleGroup` has exactly **one** construction site (`cycle_groups.rs:52-57`).

**Therefore `group_is_informational` never evaluates `all()` over an empty
iterator.**

### 2.2 MIXED group with one `Reject` member: **BLOCKING, pinned by direct assertion**

`all()` short-circuits on the first `Reject`. Two tests assert it head-on:

- **R2** `graph.rs:602-628` `double_reporting_scc_with_one_rejecting_member_is_blocking`:
  group `{1,2,3}`, delegate `FileId(1) -> Reject` / else `Tolerate`. Asserts
  `informational_cycle_count == 0` **and** that the serialized JSON does not
  contain `"informational_cycle_count"`.
- `graph.rs:651-663` `cycle_with_one_rejecting_member_is_not_informational`:
  the 2-cycle fixture, `cycle_count == 1` **and** `informational_cycle_count == 0`.

**The fixture genuinely discriminates** - re-derived by the reviewer from the
frozen oracle's source, not accepted from the journal: seeds sorted, push
1 -> 2 -> 3; `3`'s sorted neighbours `[1, 2]` give `1` on `current_path` at
position 0 -> report `[1,2,3]`, then `2` at position 1 -> report `[2,3]`. Under
the old per-path rule that yields `informational == 1`. R2's observed Red
(`left: 1`, `right: 0`) matches exactly.

**A vacuity mode found and closed:** R2 **in isolation** could pass for the wrong
reason - zero groups would satisfy both its assertions. **R1 uses the identical
fixture and asserts `cycle_count == 1`**, so the pair is non-vacuous. Both are in
the lib target and both are named PASSING in gate 3's capture (`:14`, `:18`).

### 2.3 Was any assertion silently re-expressed into a weaker truth condition? **No**

The entire range's `graph.rs` diff is **twelve hunks at zero context**. Only two
predicates changed shape - hunk 7 (`.filter(|cycle| ...)` -> `.filter(|group| ...)`)
and hunk 10 (`!cycles.is_empty()` -> `!cycle_groups.is_empty()`) - and both are the
**same predicate on a renamed binding**. No operator changed, no bound loosened,
no fixture weakened, no assertion dropped.

### 2.4 The six ported tests - each stance verified individually

See criterion 6 in section 1. All six verified; **only one line in the whole range
changed inside them**, and it was a message. **The reword did not weaken the
assertion**: `assert_eq!`'s truth condition is `left == right` on two
character-identical expressions, and the new message ("IV-9: cycle_count keeps
counting every entangled group") is **more** accurate than the old one ("IV-2:
cycle_count keeps counting all cycles"), which is no longer true after this unit.

### 2.5 `cycle_count` carries GROUP count with its NAME, and `types.rs` untouched

See criterion 2 and criterion 6 above. Both verified from the tree at HEAD, not
inferred from an absent diff.

---

## 3. Per-criterion summary

| Plan sec3 criterion | Status at this unit |
|---|---|
| 1. `detect_cycles` gone; `CycleGroup` the only cycle representation | **first clause MET; second clause carried to T3-E** (finding 8) |
| 2. `cycle_count` reports group count; `m3_include_graph_test` green without edit | **MET** |
| 3. no unstyled/amber edge between two `cycle` nodes | not this unit's scope (T3-A/T3-B UI work; `theme_token_check.py` owner) |
| 4. deep-chain stress + equivalence oracles on fixtures and 200 seeded digraphs | **MET** - `test_deep_include_chain_completes_on_default_stack ... ok` (18.81s) and the frozen oracle's 4 tests green in gates 2/3 |
| 5. planner output byte-identical (IV-8); closure equivalence suite green (IV-10) | **MET** - `planner_facts_parity`, `planner_characterization`, `m00_facts_path_parity_test` all present and green in the workspace capture, all unedited; IV-10 witnessed in gate 3 (`:87`) |
| 6. cycle-policy seam preserved (IV-11) | **MET** |

---

## 4. Gate results - every command, reviewer-run

### 4.1 Scoped gates (handoff sec8), all six pinned lines reproduced verbatim

| Gate | Command | Result | Pin matched |
|---|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_test` | `9 passed / 0 failed / 0 ignored / 1 suites` | yes |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph --test cycle_groups_equivalence_test` | `4 passed / 0 failed / 0 ignored / 1 suites` | yes |
| 3 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-graph` | `118 passed / 0 failed / 0 ignored / 5 suites` | yes |
| 4 | `python tools/distill.py -- cargo build -p ariadne-graph` | `ok (exit 0, 0 warnings emitted)` | yes |
| 5 | `python tools/testrun.py --distill --timeout 1800 -p ariadne-integration-tests --test m3_include_graph_test` | `3 passed / 0 failed / 0 ignored / 1 suites` | yes |
| 6 | `python tools/distill.py -- cargo build -p ariadne-cli -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` | yes |
| 7 | `python tools/hygiene_check.py` | **zero R1/R2 workspace-wide** | yes |
| 8 | `python tools/encoding_guard.py --staged` / `--all` | **no output, exit 0** each | yes |
| 9 | `python tools/checkpoint_guard.py status` | HEAD `aebfa9b`, DIRTY `(none)` | yes |

Gate 3's arithmetic, recounted from the 149-line capture:
**103 (lib) + 4 (`cycle_groups_equivalence_test`) + 9 (`cycle_groups_test`) + 2
(`m22_marker_unit_shaper_test`) + 0 (doc-tests) = 118, five suites, zero
unattributed residual.** Lib 103 = 101 at the base + the 2 mandated Reds.

**Non-vacuity.** Every scoped capture reads **`0 filtered out`** - the exact
vacuity mode handoff sec8 warns about - and each names its load-bearing carrier
PASSING: gate 3 names R1 (`:14`), R2 (`:18`), all six IV-11 tests including both
serde locks (`:15`, `:16`, `:19`, `:20`, `:23`, `:28`),
`test_random_digraphs_with_cycles_match_bfs_oracle` (`:87`, the IV-10 witness with
the repointed predicate), all 8 `closure_equivalence` tests,
`test_acyclic_and_empty_graphs_yield_no_groups` (`:122`),
`test_deep_include_chain_completes_on_default_stack` (`:128`), and the frozen
oracle's `test_unregistered_endpoint_cycle_is_found_by_groups_but_not_by_old_paths`
(`:111`, the named divergence). Gate 5 names the `m3` carrier.

### 4.2 Workspace-wide acceptance pass

| Gate | Command | Result |
|---|---|---|
| baseline | `python tools/testrun.py --baseline` | **BROKEN.** Verbatim: `usage error: missing value for --baseline`. Ledger F9. Not treated as a gate failure, not worked around, not wrapped. **The invocation it wraps was run.** |
| full workspace | `python tools/testrun.py --distill --timeout 3600` | **COMPLETED** - see 4.3 |
| tool unit suites | `python -m unittest discover -s tools -p "*_tests.py"` | **COMPLETED** - `Ran 184 tests in 18.164s` / `OK` |
| hygiene | `python tools/hygiene_check.py` | **zero R1/R2**; every printed line an `R3 (warn)` in a pre-existing line |
| encoding, all tracked | `python tools/encoding_guard.py --all` | **no output, exit 0** |
| encoding, staged | `python tools/encoding_guard.py --staged` | **no output, exit 0** |
| fixture state | `python tools/checkpoint_guard.py status` | run at session start and again after the workspace run |
| UI runtime smoke | `python tools/ui_smoke.py` | **REFUSED.** Verbatim: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}` |
| load-perf regression | `python tools/bench_compare.py` | **REFUSED.** Verbatim: identical text. |

Each known-broken / known-refused item attempted **INDEPENDENTLY, exactly once,
directly**. **None retried; none routed through `distill.py` or any wrapper to
evade a denial.** On the refusals: the real cause is that neither script is on the
reviewer bash allowlist (F10), **not** that bash is unavailable - the same session
successfully ran `hygiene_check.py`, `encoding_guard.py` (both forms),
`checkpoint_guard.py status`, `unittest discover`, six scoped `testrun.py`
invocations, a full-workspace `testrun.py`, and two `distill.py` builds.
**The denials caused no gate to be skipped.**

### 4.3 The canonical workspace line - COMPLETED

`python tools/testrun.py --distill --timeout 3600`, exit 0, capture
`target/testrun-capture.txt` (2078 lines). Verbatim:

```
1178 passed / 0 failed / 4 ignored / 88 suites
```

| | passed | failed | ignored | suites |
|---|---|---|---|---|
| baseline `6c0c2b6` (reviewer-measured by T3-C) | 1176 | 0 | 4 | 88 |
| **reviewer-measured `aebfa9b`** | **1178** | **0** | **4** | **88** |
| **Delta** | **+2** | **+0** | **+0** | **+0** |

**Exactly +2 passed - the two mandated Reds - and +0 in the other three columns.
Zero unattributed residual.** Handoff sec8 predicted `1177`; that prediction is
**off by one** (its chain used `+1` for unit B; T3-B actually added 2). Recorded
as verdict finding 2.

Non-vacuity: grep of the capture for `test result: FAILED|^failures:|panicked at|
error\[E` returns **no matches**, matching `0 failed`. Suite count unchanged at
**88**. The capture additionally witnesses `golden_baseline_basic ... ok` and
`golden_baseline_expensive ... ok` (**no golden drift**),
`fixture_cycle_is_informational_under_tolerate_policy ... ok` and
`auto_shaped_graph_keeps_the_cycle_informational ... ok` (**both Scala consumers
green against the ported layer**), and the presence + green run of
`planner_characterization`, `planner_facts_parity`, `m00_facts_path_parity_test`
and `m23_symbol_isolation_test` (**IV-8 holds unedited**).

The two `warning:` lines in the capture (20 in `m00_facts_path_parity_test`, 1 in
`m23_symbol_isolation_test`) are pre-existing **test-target** warnings in files
this unit did not touch; gates 4 and 6 both reported `0 warnings emitted` for
every crate this unit did touch.

### 4.4 Fixture state - sixth identical F12 reproduction

| Signal | session start | after my workspace run |
|---|---|---|
| HEAD | `aebfa9b` | `aebfa9b` |
| DIRTY FILES (`git diff HEAD --numstat`) | **`(none)`** | **`(none)`** |
| UNTRACKED | `tasks/m235-t3d-verdict.md` (mine) | `tasks/m235-t3d-verdict.md` (mine) |
| census `dirty=` | `2` | `2` |

`tests/fixtures/basic/artifacts/includes/report.json` is in **NO** commit (checked
two ways: `-- tests/fixtures` empty across the range, `-- tests` empty across the
range, and **no `tests/` path in any of the five per-commit stat lists**), and is
still absent from the numstat dirty section after my full-workspace run.
**Adjudicated: the known F12 mechanism (unpinned path + `core.autocrlf=true`),
sixth reproduction (T2-A finding 9, T2-B finding 10, T3-A, T3-B, T3-C, now
T3-D). Not a new problem, and correctly no fix attempted** - `git checkout --` /
`git restore` are mutating git and denied to every role. **Standing closeout
obligation unchanged: pin the path in `.gitattributes` or make the writer emit LF,
before T4.1.**

---

## 5. Touch-set and per-commit evidence

`git diff --name-only 6c0c2b6..aebfa9b` - six paths:
`crates/graph/src/closure_equivalence_tests.rs` (1/1),
`crates/graph/src/cycle_groups.rs` (3/2), `crates/graph/src/graph.rs` (105/75),
`tasks/m235-t3c-execution-results.md` (240/0),
`tasks/m235-t3c-verdict.md` (1142/0), `tasks/m235-t3d-journal.md` (283/0).

Negative checks, reviewer-measured, all empty: `-- tests` (covers `tests/golden`
and `tests/fixtures`), `-- Cargo.toml`, `-- Cargo.lock`,
`crates/graph/src/types.rs`, `crates/graph/Cargo.toml`,
`crates/graph/tests/cycle_groups_test.rs`,
`crates/graph/tests/cycle_groups_equivalence_test.rs`. Absent from the range name
list and every per-commit stat: `.gitattributes`, `crates/cluster/**` (IV-8),
`crates/report/**`, `crates/lang-scala/**`, `crates/graph/src/lib.rs`,
`crates/graph/src/closures.rs`, `crates/graph/src/load*.rs`, `tools/**`,
`.kilo/plans/**`.

**Enforced AND not over-broad - both directions clean.**

| # | SHA | Paths (+/-) | In touch-set? |
|---|---|---|---|
| 1 | `adb53a3` | `graph.rs` +96/-14 | yes |
| 2 | `acea7f2` | `closure_equivalence_tests.rs` +1/-1, `cycle_groups.rs` +3/-2, `graph.rs` +9/-61 - **all three in ONE commit** | yes |
| 3 | `3e1efa6` | the two T3-C reviewer records, +1382/-0 | orchestrator-declared (finding 12) |
| 4 | `ed5c467` | `tasks/m235-t3d-journal.md` +277/-0 | yes |
| 5 | `aebfa9b` | `tasks/m235-t3d-journal.md` +9/-3 | yes |

---

## 6. Explicitly NOT run / NOT verifiable

- `python tools/testrun.py --baseline` - **could not run**, broken (F9), verbatim
  recorded, not worked around. **Its wrapped invocation WAS run** (4.3).
- `python tools/ui_smoke.py` - **could not run**, REFUSED (F10), once, verbatim,
  not retried, not wrapped. **No runtime UI evidence exists for this milestone at
  this HEAD.** See section 7 for why that is materially worse here.
- `python tools/bench_compare.py` - **could not run**, REFUSED (F10), once,
  verbatim. **No verified perf evidence exists for this milestone**, and this
  unit changed what `generate_report()` computes internally.
- **The implementer's two Red captures** - not independently recoverable
  (`target/` is ephemeral; a Red state was correctly never committed). Both Red
  failures were instead **re-derived by the reviewer from the frozen oracle's
  source** and predict `left: 2 / right: 1` (R1) and `left: 1 / right: 0` (R2),
  matching the journal's quoted verbatims exactly. The temporal ordering of the
  observation is taken on report.
- **The "orchestrator-declared" status of `3e1efa6`'s two out-of-set records** -
  taken on report; no declaration was visible in this session's dispatch or the
  board. Corroborated by the exact T3-C Dev-1 precedent on a **pure-additions**
  commit (zero deletions, so it cannot have modified any existing byte) and by the
  fact that reviewers cannot commit at all.
- **No commit was made by this reviewer** (write git is denied to this role).
- **Three of this reviewer's own commands were refused by the allowlist** (a
  `python -c` inline, a six-path `git diff --name-only`, and any `git diff` scoped
  to `.gitattributes`). **All three were my own command shapes, not unavailable
  gates**; each was re-issued in an allowed form and succeeded, and the range
  `--name-only` list is dispositive for `.gitattributes`. **No acceptance gate was
  narrowed as a result.**

---

## 7. Is the absence of runtime UI evidence more material for this unit? - YES

**It is at its most material for this unit, for a specific and checkable reason.**

The chain: the UI's include-graph payload derives its cycle classification from
`CycleGroup` (unit A added `from_groups`; unit B repointed the CLI), while the
number the UI and `crates/cli/src/commands/includes.rs` display comes from
`IncludeReport::cycle_count` - the field **this unit** changed from a DFS-path
count to a group count.

- **Before** this unit those two numbers were **inconsistent by construction**:
  one came from SCC membership, the other from DFS-path enumeration, and no test
  compared them, so no runtime run could have detected the discrepancy.
- **After** this unit both derive from the same `detect_cycle_groups()` call, so
  they agree by construction. That is a genuine, real improvement in consistency.
- **But** `ui_smoke.py` is the only gate in the repo that cold-boots the UI,
  asserts its routes, and asserts `Cache-Control: no-store` on the payload. It has
  been refused for every unit of this milestone. At this HEAD the milestone's
  semantic centre has landed and **zero runtime evidence exists that the UI boots
  and serves a report whose `cycle_count` is a group count.** The legacy path
  builder that T3-E will delete has likewise never been exercised at runtime
  against the post-T3-D report.
- **Perf:** this unit changed what `generate_report()` computes internally
  (iterative Tarjan instead of DFS path enumeration), so the timing measured at
  `crates/cli/tests/payload_tail_bench.rs:322` has changed meaning again. With
  `bench_compare.py` refused, there is **still no verified perf evidence for the
  milestone**, and the bench row label still reads
  `generate_report+detect_cycles (combined)` (finding 7).

**This remains an operator/allowlist action (F10), not implementer work - but it is
now the highest-value unclosed gate in the milestone and must close before T4.1.**

---

## 8. Verdict

**APPROVED. 0 blocking / 13 non-blocking.** Full adjudication, per-section audit
A-H, all thirteen numbered findings with attribution, and the per-commit evidence
table are in `tasks/m235-t3d-verdict.md`.