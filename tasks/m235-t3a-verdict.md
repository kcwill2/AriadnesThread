# M23.5 T3-A Verdict - Group-Based Include-Element DTO (crate:ui, additive)

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Handoff:** `tasks/m235-t3a-handoff.md` (Phase 3 unit A of 5, plan T3.2 first half)
**Range reviewed:** `b950963..ea82422` (base `b950963`, HEAD `ea82422`, branch
`tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`)
**Commits:** `31513b9` (ui production + tests), `3c73cd3` (planning artifacts +
records), `ea82422` (journal)
**Execution-results addendum:** `tasks/m235-t3a-execution-results.md`

## VERDICT: APPROVED

0 blocking / 11 non-blocking. Canonical workspace line, reviewer-measured:
`1174 passed / 0 failed / 4 ignored / 88 suites` (predicted `1174 / 0 / 4 / 88`,
baseline `1169 / 0 / 4 / 88`, delta +5 tests / +0 suites, zero unattributed
residual). Cross-process export stability: **genuinely non-blocking** for this
unit, owner reassigned to unit B - see section 8.

This file is written INCREMENTALLY, cheap gates first, per the review brief (a
prior reviewer session on this unit died three times to an idle timeout having
written nothing). Sections 1-3 are final; later sections are appended as each
gate completes.

---

## 1. Ground truth

`python tools/checkpoint_guard.py status`, reviewer-run at session start:

- HEAD `ea82422`, branch `tarjan-s-scc`
- DIRTY FILES: `(none)`
- UNTRACKED FILES: `(none)`
- RECENT COMMITS (last 5): `ea82422`, `3c73cd3`, `31513b9`, `b950963`, `08741b8`
- WORKTREE CENSUS: 8 worktrees; `.kilo/worktrees/tarjan-s-scc` `ahead=10
  behind=13 dirty=1`

The single `dirty=1` census entry is adjudicated in section 6 below (F12
mechanism, confirmed).

---

## 2. Commit-versus-touch-set check (PASS, 3 of 3 commits)

**Enforced touch-set.** Handoff sec2 six paths:
`crates/ui/src/dto.rs`, `crates/ui/tests/ui_test/dto_builders.rs`,
`crates/ui/tests/ui_test/graph_api.rs`, `crates/ui/tests/ui_test/export_order.rs`,
`crates/ui/tests/ui_test/d27_node_provenance.rs`, `tasks/m235-t3a-journal.md`.
Plus the sec6 extension clause: `crates/ui/src/lib.rs` (one mod line) and
`crates/ui/src/cycle_groups_view.rs` (new). Plus the orchestrator-declared
record files carried in commit B.

### Commit A `31513b9` - "feat(ui): add group-based include element builder with cycle group data"

Eight paths, all accounted for:

| Path | In touch-set? |
|---|---|
| `crates/ui/src/dto.rs` | yes (sec2) |
| `crates/ui/src/cycle_groups_view.rs` | yes (sec6 extraction clause, declared) |
| `crates/ui/src/lib.rs` | yes (sec6, one line `pub(crate) mod cycle_groups_view;`) |
| `crates/ui/tests/ui_test/dto_builders.rs` | yes (sec2) |
| `crates/ui/tests/ui_test/graph_api.rs` | yes (sec2) |
| `crates/ui/tests/ui_test/export_order.rs` | yes (sec2) |
| `crates/ui/tests/ui_test/d27_node_provenance.rs` | yes (sec2) |
| `tasks/m235-t3a-journal.md` | yes (sec2) |

`crates/ui/tests/ui_test/main.rs` is ABSENT - the sec2 byte-identity lock holds.
No `crates/cli/**`, no `crates/graph/**`, no `crates/report/**`, no `crates/ui/assets/**`,
no `tests/**`, no `tools/**`, no `Cargo.toml`, no `Cargo.lock`, no
`.gitattributes`. Zero production paths outside `crates/ui/src/`.

### Commit B `3c73cd3` - "docs(tasks): land M23.5 phase 3 authorization records and T2 verdict files"

Ten paths, ALL records, zero production:

```
.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md
tasks/m235-t2b-execution-results.md
tasks/m235-t2b-verdict.md
tasks/m235-t3a-handoff.md
tasks/m235-t3a-journal.md
tasks/m235-t3b-handoff.md
tasks/m235-t3c-handoff.md
tasks/m235-t3d-handoff.md
tasks/m235-t3e-handoff.md
tasks/milestone_ledger.md
```

`tasks/m235-t3a-journal.md` is in-set; the other nine are the
orchestrator-declared record files (journal Checkpoint 0). No `crates/**`, no
`tests/**`, no `tools/**`.

### Commit C `ea82422` - "docs(tasks): M23.5 T3-A journal -- SHAs 31513b9/3c73cd3, after-state, deviations inventory"

One path: `tasks/m235-t3a-journal.md` (in-set). Adds Checkpoint 5 with both
SHAs self-attested, resolving the `PENDING-B` marker that Checkpoint 4 left
open. This is the T2-A/T2-B precedent form.

### The forbidden fixture

`tests/fixtures/basic/artifacts/includes/report.json` appears in **NO** commit
of this range. It is absent from all three per-commit file lists above. Lock
HOLDS.

---

## 3. Hygiene, encoding, theme tokens (STEP 1 gates, reviewer-run)

| Gate | Command | Reviewer-measured result |
|---|---|---|
| hygiene | `python tools/hygiene_check.py` | **ZERO R1/R2 lines workspace-wide.** Every printed line is `R3 (warn)`. |
| encoding, all tracked | `python tools/encoding_guard.py --all` | no output (exit 0) |
| encoding, staged | `python tools/encoding_guard.py --staged` | no output (exit 0) |
| theme tokens | `python tools/theme_token_check.py --check-added --base b950963` | **no output, exit 0 = zero added hex literals** |

### Hygiene: the R3 claim, independently checked

The implementer claims every R3 warning in a file it touched sits on a
pre-existing line outside its authored region. Verified by line number against
the committed file:

- `crates/ui/src/dto.rs:330`, `:418`, `:445` - all three `single-letter local
  `s``. The authored region is `598-748` (both wrappers, the shared emitter,
  the doc comments). All three are far below it, in the legacy
  justifigation/signature helpers. **CONFIRMED pre-existing.**
- `crates/ui/tests/ui_test/graph_api.rs` - 14 `res` warnings at lines 52, 53,
  59, 60, 66, 67, 73, 74, 123, 124, 125, 228, 229, 231. The single authored
  `#[test]` starts at line ~697 (`+` block begins after `test_graph_collateral_
  invalid_file_returns_200_zeros_even_while_loading`). Every flag is an order
  of magnitude below that. **CONFIRMED pre-existing.**
- `crates/ui/tests/ui_test/dto_builders.rs`,
  `crates/ui/tests/ui_test/export_order.rs`,
  `crates/ui/tests/ui_test/d27_node_provenance.rs`,
  `crates/ui/src/cycle_groups_view.rs` - **zero flags each.** The scanner
  demonstrably scans these trees (it flags sibling test files such as
  `server_api.rs`, `theme_contract.rs`, `justification_golden.rs`), so the
  zero is measured, not a skip.

R3 is warn-only per `AGENTS.md`; no name the implementer authored is flagged.

### Theme tokens: zero new hex literals, and verdict semantics are structurally untouched

Gate 5 returns no output against base `b950963`. Independent structural
confirmation: the commit range touches **no** file under `crates/ui/assets/`
and no CSS, JS or template anywhere - the theme surface is not in the diff at
all, so light / dark / dark-steampunk semantic verdict colors (green / amber /
red) and the cool-to-hot heat ramp cannot have moved. Only red-edge
**COVERAGE** grows, which is exactly the intent (plan sec1.1 item 1).

---

## 4. Scoped gates (STEP 2, reviewer-run, nothing relayed)

| Gate | Command | Reviewer-measured result |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test` | `184 passed / 0 failed / 0 ignored / 1 suites` |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui` | `184 passed / 0 failed / 0 ignored / 3 suites` |
| 3 | `python tools/distill.py -- cargo build -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` |
| extra | `python tools/distill.py -- cargo build -p ariadne-cli` | `ok (exit 0, 0 warnings emitted)` |

Both pinned lines from handoff sec7 reproduce **verbatim**. Neither gate is
vacuous:

- Gate 1's single suite is the real `ui_test` binary. Its capture carries
  `test result: ok. 184 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.
  `0 filtered out` on every `test result:` line - nothing was silently
  filtered, which is the exact failure mode handoff sec7 warns about.
- Gate 2's three suites are `unittests src/lib.rs` (0 tests),
  `tests\ui_test\main.rs` (184 tests), `Doc-tests ariadne_ui` (0 tests) - read
  from the capture. `crates/ui/src/` contributes 0 tests, exactly as the
  handoff's arithmetic claims.
- All six required tests are named as PASSING in the capture, each selecting
  exactly one test:
  `dto_builders::intra_group_non_tree_edge_carries_the_cycle_class`,
  `dto_builders::node_cycle_data_reports_group_size_and_witness_only_for_grouped_nodes`,
  `dto_builders::grouped_nodes_and_edges_are_classed_cycle_from_membership_not_from_reported_paths`,
  `export_order::grouped_nodes_keep_the_canonical_emission_order_and_edge_order`,
  `graph_api::include_elements_export_is_stable_across_two_calls_with_cycle_fixtures`,
  `d27_node_provenance::d27_file_node_decoration_is_omitted_when_map_entry_absent`.

### The `184 / 179` arithmetic, re-derived from the tree rather than trusted

Counted `#[test]` + `#[tokio::test]` attributes under
`crates/ui/tests/ui_test/` at HEAD: **184**, with **zero** `#[ignore]`
anywhere under `crates/ui`. That matches gate 1's `184 passed` exactly (each
`#[tokio::test]` expands to one `#[test]`).

The delta is attributable path by path:

| File | `#[test]` attrs | New |
|---|---|---|
| `dto_builders.rs` | 10 (attrs at 9, 41, 68, 96, 163, 244, 330, **457**, **490**, **562**) | 3 (tests 1, 2, 3) |
| `graph_api.rs` | 30 (new attr at **741**) | 1 (test 4) |
| `export_order.rs` | 4 (new attr at **151**) | 1 (test 5) |
| `d27_node_provenance.rs` | 10 - UNCHANGED count | 0 - the D27 test was EXTENDED |
| all other 15 modules | unchanged | 0 |

179 + 5 = 184. **Confirmed, and confirmed non-vacuously:** the five new
functions are exactly the five the handoff names, and no sixth `#[test]` was
added anywhere.

### The overwritten capture: did it destroy evidence for a pinned line?

**No pinned line lost its evidence, because I re-ran every pinned gate.** The
overwrite chain the implementer reports is real (R1 capture -> R2 capture ->
gate 1 -> gate 2), and `target/` is ephemeral and gitignored, so at disk only
gate 2's 203-line capture survives. That is why sections 3 and 4 above are
reviewer-measured rather than journal-relayed.

What the overwrite **did** destroy is the raw Red evidence: R1's and R2's
original capture text is unrecoverable from disk. It survives verbatim in the
committed journal (`tasks/m235-t3a-journal.md` Checkpoint 1, in `31513b9` and
`ea82422`), which is exactly where handoff sec9 directs it
("say so; do not cite it as retained evidence"). The implementer did say so.
**Disposition: no finding.** The loss is bounded and the surviving substitute
is a committed, auditable artifact.

---

## 5. Substantive code audit (STEP 3)

### 5.1 The two Reds

**R1 - `dto_builders.rs:457` `intra_group_non_tree_edge_carries_the_cycle_class`.**

Selected exactly one test: the journal's digest is `0 passed / 1 failed / 0
ignored / 1 suites` from `--test ui_test -- intra_group_non_tree_edge_carries_the_cycle_class`,
and the failure count itself proves non-vacuity. It was a genuine runtime
assertion failure, not a compile error: the message quoted in the journal
("intra-group chord edge (a.h -> c.h), pair (1, 3), must carry is_cycle =
true; left: Bool(false)") matches committed line 481 **verbatim**, and
`left: Bool(false)` is exactly what a `serde_json::Value::Bool(false)` renders
as in an `assert_eq!` against `serde_json::json!(true)`.

**R1's mechanism, independently re-derived from source (not from the journal).**

I re-executed `detect_cycles` by hand against the fixture
(edges `1->2, 2->3, 3->1, 1->3, 3->4`; `file_paths` = 1,2,3,4):

- `graph.rs:240-241` seeds `all_files` from `file_paths.keys()` **sorted** ->
  `[1, 2, 3, 4]`.
- `graph.rs:242` calls `adjacency()`, which (per T2-B verdict sec4, verified
  against `graph.rs:83-95`) `sort()`s and `dedup()`s each destination list, so
  `adj[1] = [2, 3]` - **the chord destination 3 is explored second, not
  first**. That ordering is what makes the fixture bite.
- DFS from 1: 1 -> 2 -> 3; at 3 the neighbours `[1, 4]` are visited in sorted
  order, and `1` is on `rec_stack`, so `current_path[0..]` = `[1, 2, 3]` is
  pushed as the ONE reported cycle. Back at 1, neighbour `3` is visited but no
  longer on `rec_stack`, so no second cycle is reported.

So `detect_cycles()` on this fixture returns exactly `[[1, 2, 3]]` - which
makes the handoff's parenthetical "what the old representation reports for this
graph" **literally correct**, and the Red's hand-fed
`vec![vec![a, b, c]]` byte-identical to real producer output. This is a
faithful reproduction of production, not a contrived input.

Feeding `[[1,2,3]]` through the OLD body (recovered verbatim from the `-` lines
of the `31513b9` diff): `windows(2)` yields `[(1,2), (2,3)]`; the closing pair
adds `(3,1)`. So `cycle_edges = {(1,2), (2,3), (3,1)}` and **`(1,3)` is
absent** -> the chord emits `is_cycle: false`, `classes: None`. Meanwhile
`cycle_nodes = {1,2,3}`, so nodes `a.h` and `c.h` BOTH carry
`classes: Some("cycle")`.

`{1,2,3}` is strongly connected and nontrivial (size 3), so the chord is an
edge **inside a nontrivial SCC** whose two endpoints are both classed `cycle`.
**R1's mechanism CONFIRMED on all three counts**: chord, inside a nontrivial
SCC, left UNSTYLED by the old path-derived logic.

**R2 - `dto_builders.rs:490` `node_cycle_data_reports_group_size_and_witness_only_for_grouped_nodes`.**

Also one test (`0 passed / 1 failed / 0 ignored / 1 suites`). The quoted
message matches committed lines 512-513 verbatim. The decisive corroboration is
the printed data map the journal quotes:

```
{closure,id,in_degree,is_cycle,kind,label,out_degree,path}
```

That is EXACTLY the eight pre-existing keys of handoff sec3.3, in alphabetical
order, with neither new key present. A fabricated Red would have had to invent
the precise eight-key pre-implementation node map; and the map's alphabetical
ordering independently proves `serde_json` is on its default `BTreeMap` here
(see finding 4). `left: None` from `data.get(..)` is the honest report of an
absent key, i.e. the same missing-key semantics the handoff predicted as
`Null`. **Genuine runtime failure, non-vacuous.**

**Reported panic line numbers: one line off, and correctly so.** The journal
reports R1 at `dto_builders.rs:477` and R2 at `:508`. At HEAD, line 477 is
`let chord = edge_element(&elements, 1, 3);` and 478 is the `assert_eq!`;
line 508 is `let node_b = ...` and 509 is the `assert_eq!`. Each is one line
below the committed `assert_eq!`. That is the expected signature of the sec4
**re-point**, which replaced a short `build_include_elements` call with a
longer `single_group_containing` + `build_include_elements_from_groups` call.
It is corroboration, not proof: the Red revision was never committed (correct -
"Never commit a Red state"), so exact-location matching is unavailable here, in
contrast to T2-B where the mutation WAS committed. Recorded as a reviewer
capability limit (finding 7), not as a suspicion.

**The re-point itself (handoff sec4's mandated step): DONE and logged.** Both
Reds now call `build_include_elements_from_groups` with a REAL group from
`graph.detect_cycle_groups()` via `single_group_containing` (lines 468-469,
499-500) - the real producer, not a hand-built literal, exactly as sec4
prescribes. **No assertion was weakened**: every assertion that failed in Red
is present in the committed test (lines 478-487 keep both R1 assertions; lines
509-536 keep all four R2 assertions). The TDD trail is recorded in a comment at
each test and in journal Checkpoint 2.

### 5.2 IV-3 / IV-4 at the DTO layer - the decisive same-group test

Read from `crates/ui/src/cycle_groups_view.rs` (new file, 149 lines):

- **Node rule (IV-2).** `from_groups` sets
  `grouped_nodes = group_index_by_member.keys().copied().collect()`; `node_is_cycle`
  is a membership test. A node is classed `cycle` iff it appears in some
  group's `member_files`. CORRECT, and it reads `member_files` and nothing else.
- **Edge rule (IV-3) - GROUP IDENTITY, not co-membership.** This is the point
  the audit turns on, and the code is unambiguous:

  ```rust
  fn endpoints_share_group(index_by_member: &HashMap<FileId, usize>, source: FileId, destination: FileId) -> bool {
      match (index_by_member.get(&source), index_by_member.get(&destination)) {
          (Some(source_group), Some(destination_group)) => source_group == destination_group,
          _ => false,
      }
  }
  ```

  Two independent lookups into a `FileId -> group index` map followed by
  `usize` **equality**. It cannot degrade into "both endpoints are in SOME
  group": an ungrouped endpoint falls to `_ => false`, and two endpoints in
  different groups compare unequal indices. `group_index_by_member` builds the
  map by `enumerate()` over the groups slice, so the index IS the group
  identity. **CONFIRMED: the implementation compares group identity.**
- **Self-loop.** `endpoints_share_group(u, u)` is trivially index equality, so
  a self-loop is cyclic iff `u` is grouped - matching handoff sec3.2 and plan
  IV-3's "(or a self-loop)" clause.

**The `(12, 20)` cross-group lock is genuinely asserted.**
`dto_builders.rs` test 3 builds two disjoint hand-built groups -
`{10,11,12}` (index 0) and `{20,21}` (index 1) - and then asserts on the edge
`12 -> 20`:

```rust
let cross_group_edge = edge_element(&elements, 12, 20);
assert_eq!(cross_group_edge.data["is_cycle"], serde_json::json!(false),
    "edge (12, 20) joins group 1 to group 2: ... IV-3 identity requires it to stay unclassed");
assert_eq!(cross_group_edge.classes, None,
    "cross-group edge (12, 20) must not carry the cycle class");
```

Both `data["is_cycle"] == false` AND `classes == None` are asserted, so this is
a real lock, not a decorative one. A co-membership implementation would return
`true` here and fail. The same test additionally asserts the grouped-to-
ungrouped tail edge `(21, 30)` is unclassed. **This is the decisive test and it
holds.**

Test 3 also carries a second, independent property worth crediting: file `12` is
in `member_files` but appears on **no** `representative_cycle`, and the test
asserts `(11,12)` and `(12,11)` are classed `cycle` "witness coverage aside".
So it independently proves the classification reads `member_files` rather than
the witness walk - a stronger lock than the handoff asked for.

- **`cycle_group_size` / `cycle_witness` OMITS rule.** `group_size()` and
  `witness_labels()` both return `Option`, and `dto.rs:717-722` only assigns
  when `Some`. An ungrouped node therefore has NEITHER key - not `null`, not
  `0`. Test 2 asserts both absences via `data.get(..).is_none()`, and the
  D27 extension asserts presence on a grouped node. **CORRECT on both halves.**
- **JSON key order.** The two new keys are assigned after `language` /
  `unit_kind`, so nothing existing moved. (Mechanism note in finding 4: this
  repo's `serde_json` is NOT `preserve_order`, so ordering is BTreeMap
  alphabetical - a stronger determinism property than the handoff assumed.)

### 5.3 D27 regression - EXTENDED, not replaced

The `language` / `unit_kind` omission code at `dto.rs:711-716` appears as
**context lines** in the `31513b9` diff, i.e. it is not a single byte changed.
The pre-existing test
`d27_file_node_decoration_is_omitted_when_map_entry_absent` keeps its original
two assertions (`data.get("language").is_none()`,
`data.get("unit_kind").is_none()`) and gains five more, including that a node
that IS in a cycle group still omits both keys and does carry
`cycle_group_size == 2` plus a `cycle_witness` array and `classes ==
Some("cycle")`. The file's `#[test]` count is unchanged at 10, so the sec5
"extend, do not add a sixth `#[test]`" instruction is honoured literally.

**Lock HOLDS: extended, not replaced; and the omission path now serves both
entry points from one shared code block, so it cannot drift between them.**

### 5.4 Sequencing invariant - the old entry point is intact and `crates/cli` compiles

`build_include_elements`'s signature is byte-identical to handoff sec3's quoted
form. In the `31513b9` diff the whole signature block plus the doc comment
above it are **context lines**; the only `-` lines are the old body's inline
cycle-set derivation, and they are replaced by a 6-line call into the shared
emitter. The 5-argument D27 shape survives; `crates/ui/tests/ui_test/main.rs`
is absent from all three commits (byte-identity lock holds).

Independently measured, not assumed: `cargo build -p ariadne-cli` -> `ok (exit
0, 0 warnings emitted)`. **`crates/cli` compiles clean across the boundary**,
which is the entire point of the additive seam.

Nothing outside `crates/ui` was touched in any commit (section 2).

### 5.5 The extraction - a real module boundary, ceiling genuinely satisfied

- **`dto.rs` is 901 lines at HEAD** (verified: the file ends at line 901).
  Under the 1000-line R2 ceiling, and `hygiene_check.py` reports zero R1/R2
  lines - so the ceiling is satisfied by measurement, not by assertion.
- The implementer's arithmetic is credible and self-consistent: 852 at base,
  net +49 landed = 901, and the journal states the un-extracted variant
  measured ~1011. I did not re-measure the 1011 figure (it would require
  re-inlining a deleted file, which `crates/**` read-only forbids), but the
  landed 901 is the number that matters and it is under the ceiling with the
  split in place. The claim is therefore **not load-bearing** - had the
  estimate been wrong, the result would still pass.
- **The split is a real module boundary, not a file move.** `cycle_groups_view.rs`
  owns classification (`from_paths`, `from_groups`, `node_is_cycle`,
  `edge_is_cycle`, `group_size`, `witness_labels`, plus three free helpers);
  `dto.rs` owns the two public entry points and the single emission tail. The
  boundary is "how to classify" versus "how to emit", which is the right seam.
- **No logic is duplicated across the two files.** `emit_include_elements`
  exists exactly once (`dto.rs:659-745`) and contains exactly ONE node loop
  (`685`) and ONE edge loop (`729`) - the degree-tally loop at `670` iterates
  edges a third time but emits nothing. The classification loops exist exactly
  once, in `cycle_groups_view.rs`. Handoff sec3.2's "one responsibility, no
  duplicated emission loop" and sec9 check 3 are both satisfied.
- **Function lengths.** `emit_include_elements` spans `659-745` = 87 lines
  including signature and doc, under the 100-line rule; the two public wrappers
  are 15 and 15 lines. `from_groups` is ~35 lines. All clear, and consistent
  with `hygiene_check.py`'s zero R1 output.
- **No parameter soup** (sec6): the shared helper takes a single
  `&IncludeCycleClass` view object, not five booleans. The handoff's own
  tripwire was satisfied by construction.
- **Sanctioned dual representation, explicitly.** `IncludeCycleClass::from_paths`
  retains the legacy path rule next to `from_groups`. That is a *declared*,
  plan-sanctioned temporary coexistence (handoff sec1: unit B migrates the
  caller, unit E retires the old entry point), not an unflagged second
  representation. `from_paths` is semantically byte-identical to the old
  inline code and leaves both new maps empty, so the legacy payload is
  byte-compatible. **Accepted, with unit E owning the retirement.**

### 5.6 `--export` reachability - load-bearing for the cross-process call

`crates/cli/src/commands/ui.rs` at HEAD still reads:

- `:310  let cycles = include_graph.detect_cycles();`
- `:316  let include_elements = ariadne_ui::build_include_elements(`

So the `--export` path still uses the LEGACY builder and the LEGACY producer.
The new cycle currency is not in the exported bytes at this commit. This fact
governs the cross-process classification in section 8.

---

## 6. The six declared deviations, adjudicated

Adjudicated on evidence, not accepted because declared.

| # | Deviation | Disposition |
|---|---|---|
| D1 | touch-set extension to `crates/ui/src/lib.rs` + new `crates/ui/src/cycle_groups_view.rs` (handoff sec6 extraction branch) | **ACCEPTED** |
| D2 | gate 8's `--scope` list carries those two files beyond sec2's literal six | **ACCEPTED** |
| D3 | R1 fired its FIRST assertion where the handoff predicted "assertion (2)" | **ACCEPTED**, immaterial |
| D4 | R2 reported `left: None` (`.get`) where the handoff predicted `Null` (index) | **ACCEPTED**, and strictly better |
| D5 | `encoding_guard.py --staged` cannot interpose before `commit_scoped` (D-E precedent) | **ACCEPTED** |
| D6 | orchestrator-named `t2a-verdict` / `t2a-execution-results` were already tracked -> scope entries were no-ops | **ACCEPTED**, verified |

**D1 - ACCEPTED, fully compliant including the mandatory declaration.** Handoff
sec6 pre-authorizes exactly this branch: if `dto.rs` would exceed ~940 lines,
extract the group-membership helpers into `crates/ui/src/cycle_groups_view.rs`
plus a one-line `pub(crate) mod cycle_groups_view;` in `lib.rs`, "and if you
take that branch you MUST declare `crates/ui/src/lib.rs` in your journal as a
deliberate touch-set extension with your reason (one line, no other file
changes)." Verified against the diff: the `lib.rs` change is **exactly one added
line and nothing else** - the "no other file changes" clause holds. The branch
condition is met on the PRE-extraction measurement (journal: ~1011 > the 1000
R2 ceiling), which is the correct thing the condition tests; the 901 that landed
is the post-split number and is a consequence, not a contradiction. The
declaration appears three times (Checkpoints 0, 3, 4) with a one-line reason.
Nothing left undeclared.

**D2 - ACCEPTED.** Gate 8's literal `--scope` list is a verbatim transcription
of sec2 and therefore predates the sec6 branch. Carrying the two extension
files is not scope creep but a buildability requirement: `lib.rs` declares a
module that the commit must contain, so a commit scoped to sec2's six paths
alone would reference a module it does not carry. The broadened scope was
EXPLICIT (sec0 step 0.4 satisfied), not `git add .`, and it is declared. No
production path outside the extension pair was added.

**D3 - ACCEPTED, immaterial.** `assert_eq!` failures abort the test, so the
first failing assertion is the only one observed. The handoff predicted the
`classes` assertion would fail first, but the `is_cycle` assertion is written
first and fails first. The two are not independent discriminators here: in the
old body both are driven by the same `is_cycle` local, so `is_cycle: false` and
`classes: None` were both true for the chord. Same failure class, and the pair
`(1, 3)` IS named as the handoff required. No assertion was dropped: both R1
assertions survive in the committed test at lines 478-487.

**D4 - ACCEPTED, and better than the contract asked.** Indexing
(`data["cycle_group_size"]`) yields `Value::Null` for a missing key; `.get(..)`
yields `None`. For an OMITS-rule lock, distinguishing "key absent" from "key
present and null" is the point of the contract, so `.get()` is the correct
assertion API and the handoff's predicted `Null` rendering was the weaker
choice. Disclosed in the journal.

**D5 - ACCEPTED.** `commit_scoped.py` has no stage-only mode, so `--staged`
cannot interpose between staging and committing in one command. The
implementer ran the STRONGER `--all` form pre-commit and the literal `--staged`
form post-commit. I ran both independently: `--all` silent, `--staged` silent,
exit 0 each. Substitution cost is zero. Same precedent as T2-A gate 5 and T2-B
deviation D-E.

**D6 - ACCEPTED, verified.** `tasks/m235-t2a-verdict.md` and
`tasks/m235-t2a-execution-results.md` were already committed by the T2-B
implementer in `08741b8` (T2-B verdict sec2). Including their globs in a scope
is a content no-op, and neither path appears in commit B's file list - exactly
what "already tracked, nothing to commit" predicts. The implementer recorded
this rather than silently dropping the scope entries or claiming to have
"restored" the files.

### Undeclared deviations

**None found.** Three divergences from upstream text surfaced during the audit;
all three are traced to a handoff or plan sentence rather than to an
undisclosed implementer choice, and all three are recorded as findings 4, 5
and 6 below. The `witness_path_labels` label-drop policy (finding 3) is a
design decision, and it IS disclosed - in the function doc comment and in the
journal's Deliberate-shape record.

---

## 7. `checkpoint_guard status` and the F12 fixture - KNOWN mechanism, not a new problem

### What I measured

`python tools/checkpoint_guard.py status`: HEAD `ea82422`, **DIRTY FILES
(none)**, **UNTRACKED FILES (none)**. The forbidden fixture
`tests/fixtures/basic/artifacts/includes/report.json` is **absent from the
numstat dirty section** - which is a content comparison, so its content is
byte-equal to HEAD.

The porcelain view disagrees: `git status` reports that path as
`modified: ... (Changes not staged for commit)`, and the worktree census row
prints `dirty=1` for `.kilo/worktrees/tarjan-s-scc`.

### Adjudication: known F12 mechanism, NOT a new problem

1. **Third identical reproduction.** T2-A finding 9, T2-B finding 10, and now
   this unit each measured the same signature: numstat clean, porcelain dirty,
   content byte-equal, path in no commit.
2. **The mechanism is documented and unchanged.** Handoff step 0.2 and ledger row
   F12: the path is unpinned by `.gitattributes` (which sets `text eol=lf` only
   for `crates/ui/tests/fixtures/*.json` and `tests/golden/**/*.json`), and this
   worktree has `core.autocrlf=true`. Git's index holds LF; the worktree copy
   is CRLF-normalised; a test run that rewrites the artifact invalidates the
   stat-cache entry. `git status` reports the stat mismatch; `git diff`
   normalises line endings and finds zero content delta, hence the empty
   numstat section. Stat-cache churn, not content dirt.
3. **No CRLF bytes entered the repository through this unit.** The path appears
   in NO commit of `b950963..ea82422` (section 2). The defect is confined to the
   working tree, and even there the bytes are content-identical after
   normalisation.
4. **The implementer's distinction was exactly right and I confirm it.** "numstat
   dirty section does NOT list that path -> clean, no restoration was ever
   needed" and "the census row prints `dirty=1` ... porcelain-only" are both
   literally true of my own measurement. The implementer also correctly did NOT
   attempt a fix: `git checkout --` / `git restore` are denied to every role,
   and rewriting a byte-equal file would have been a content change with no
   delta to make.

**Standing closeout obligation, unchanged and still open:** pin the path in
`.gitattributes`, or make the artifact writer emit LF. This must land before
T4.1. Tracked as a milestone deferred row, not as a T3-A finding.

---

## 8. Cross-process export stability - the classification

**CLASSIFICATION: GENUINELY NON-BLOCKING for T3-A.** The premise behind the
worry is correct and I am not going to soften it: in-process stability does NOT
satisfy a two-process byte-stability contract. An export that varies run to run
would pass `include_elements_export_is_stable_across_two_calls_with_cycle_fixtures`
and fail the contract. So the implementer was right not to claim the stronger
property, and right to say so in the journal instead.

The item is non-blocking for THIS unit on four independent grounds, none of
which is "the implementer should have done more".

**(a) The implementer's actual contract allocated it out, explicitly and
correctly.** Handoff sec5, on test 5: "The **cross-process** half of that lock
is not yours: `tests/golden/**` is byte-locked out of your scope and the
`--export` byte-stability check belongs to the reviewer's acceptance pass. Say
so in your journal rather than claiming you proved it." The implementer wrote
precisely that sentence into journal Checkpoint 5's NOT-run list. Under the
plan-literal-compliance rule this is **compliance**, not shortfall.

**(b) It is UNREACHABLE at this HEAD, which is stronger than "out of scope".**
`crates/cli/src/commands/ui.rs:310` still calls `include_graph.detect_cycles()`
and `:316` still calls `ariadne_ui::build_include_elements(...)` (section 5.6).
The `--export` bytes at this commit contain **no cycle-group data at all**. A
two-process export diff run now would compare two LEGACY-path exports and would
be structurally incapable of failing on anything this unit changed. No amount of
testing inside this touch-set could have produced the evidence, because the
currency under test does not reach the artifact.

**(c) The requirement currently has no owner and no executable command.** The
contract text is real and threefold: plan:128 lists as a T3.2 Red "byte-stable
export test across two processes with cycle fixtures present"; plan:147 lists
"export byte-stable" as acceptance criterion 5; IV-5 (plan:29) requires "export
stays byte-stable across processes". But the handoff deferred the check to the
"reviewer's acceptance pass", and NONE of the four workspace-wide reviewer gates
performs it: `--baseline` is broken (`usage error: missing value for
--baseline`, F9), `ui_smoke` and `bench_compare` are refused (F10), and
`unittest discover` is a Python tool suite. So the requirement was assigned to a
gate that does not exist. That is a planning defect, not an implementation one.

**(d) The residual risk is low and I measured it structurally rather than
assuming it.** Everything that could inject cross-process nondeterminism into
this payload was checked:

- **No hash iteration reaches payload bytes in the new code.**
  `group_index_by_member`, `cyclic_pairs`, `group_size_by_file` and
  `witness_labels_by_file` are consumed ONLY through `get` / `contains`. Nothing
  iterates them to produce output, so per-process hash seeds cannot reach the
  payload.
- **Node order comes from a total-order sort.** `dto.rs:681-683` sorts
  `file_paths` (a std `HashMap` with a per-process seed) by `(path, file_id)`.
  The key is unique per entry, so the stable sort's tie-break is never
  exercised and the order is seed-independent. This is pre-existing code whose
  doc comment states the intent outright: "raw iteration would shuffle the node
  array on every run - `--export` must be byte-stable across processes."
- **Edge order comes from a `Vec`.** `for edge in &graph.edges` is
  insertion-ordered, not hash-ordered.
- **`cycle_witness` order comes from a `Vec`.** `witness_path_labels` iterates
  `representative_cycle` in slice order; no hash order is involved.
- **`serde_json::Value` objects serialise in `BTreeMap` (alphabetical) order**,
  proven empirically by the Red's printed data map
  `{closure,id,in_degree,is_cycle,kind,label,out_degree,path}`. That is
  order-independent of insertion, which is a stronger cross-process guarantee
  than the handoff's stated mechanism.
- **No clock, no RNG, no address-derived value exists anywhere in
  `crates/ui/src`** - grep for `SystemTime|Instant::now|Utc::now|Local::now|
  rand::|thread_rng|random` returns zero matches. Element ids are numeric
  (`file_id.0.to_string()`, `format!("{}-{}", ...)`).
- **The payload's two `HashMap` fields already carry an explicit cross-process
  discipline.** `GraphExportPayload.file_collateral` and `.file_justification`
  both use `serialize_with = "serialize_sorted_nested_maps"`, and
  `dto.rs:202-206` documents why: "std `HashMap` iteration order comes from a
  per-process seed, so every map in the export payload must sort at emission
  for the `--export` bytes to be stable across processes." That mechanism is
  pre-existing, is not touched by this unit, and covers the parts of the payload
  this unit does not reach.
  (`CollateralResponse.direct_collateral` / `.transitive_collateral` are
  unsorted `HashMap`s, but `CollateralResponse` is a REST response type, not a
  member of `GraphExportPayload`, so it does not affect `--export` bytes. Noted
  for completeness, not a finding, and out of T3-A's scope.)

**Recommended disposition - a ledger deferred row with a named owner, NOT work
demanded from this implementer:**

- **Owner: unit B** (`tasks/m235-t3b-handoff.md`), because unit B is the commit
  that repoints `cli/commands/ui.rs:310`/`:316` and therefore the first point at
  which the group currency is present in `--export` bytes at all. Mandatory gate
  there: two `--export` invocations over a cycle-bearing fixture into two files,
  then a byte comparison, in one session.
- **Must close before T4.1**, because plan acceptance criterion 5 ("export
  byte-stable") and IV-5 cannot be claimed until it has been executed against
  the reachable path.
- **Do not demand it here.** A two-process export test needs a CLI invocation
  and/or a golden under `tests/golden/**`, both byte-locked out of this unit's
  touch-set; demanding it would be an unexecutable demand against an artifact
  that cannot yet contain the data under test.
- Recommended cheap companion, also for unit B (not demanded here, and it would
  break this unit's 184 count lock): a parity test asserting
  `build_include_elements_from_groups(graph, .., &[], ..)` is byte-equal to
  `build_include_elements(graph, .., &[], ..)` on an acyclic graph, which would
  behaviourally lock "no hash leakage, no order change" without spawning a
  process (see finding 8).

---

## 9. Findings

**0 blocking. 11 non-blocking.** Verdict: **APPROVED**.

1. **NON-BLOCKING, architect / planner.** The milestone requires a two-process
   export byte-stability lock (plan:128 Red, plan:147 acceptance 5, IV-5 at
   plan:29) but **no gate in the milestone implements it**. The handoff
   delegated it to "the reviewer's acceptance pass", and none of the four
   workspace-wide reviewer gates is a two-process export diff: `--baseline` is
   broken (F9), `ui_smoke` and `bench_compare` are refused (F10). The
   requirement is owner-less. **Action:** ledger deferred row, owner unit B,
   closes before T4.1. See section 8.
2. **NON-BLOCKING, architect / planner.** Even if a reviewer tried, the check is
   unreachable at this HEAD: `crates/cli/src/commands/ui.rs:310` and `:316` still
   call `detect_cycles()` and the legacy `build_include_elements`, so `--export`
   bytes carry no cycle-group data and a two-process diff could not fail on
   anything this unit changed. Unit B is the first commit at which the check
   becomes meaningful. **Action:** same deferred row; do not treat "not run at
   T3-A" as a coverage gap that T3-A could have closed.
3. **NON-BLOCKING, implementer (accepted and documented, follow-up owed).**
   `cycle_groups_view.rs::witness_path_labels` uses
   `filter_map(|member| graph.file_paths.get(member))`, so a witness member the
   graph never registered is silently dropped from `cycle_witness`. For such a
   group the emitted array is **no longer a closed walk** - a vertex is missing,
   so consecutive labels may not be joined by any real include edge, which is
   the IV-4 property the field is supposed to display. Deterministic (the drop
   follows `representative_cycle` slice order), so byte-stability is
   unaffected; classes, sizes and ordering are unaffected; the decision is
   disclosed in the function doc and in journal Checkpoint 0. Reachable only on
   graphs carrying unregistered edge endpoints (the T2-B divergence shape).
   **Action:** at unit E or T4.1, decide the label for an unregistered witness
   member (numeric `FileId` string, as node `id` already does, is the obvious
   candidate) and add a lock for whichever behaviour is chosen. Not blocking: it
   is a documented design decision on an exotic input, it degrades display
   fidelity only, and choosing the behaviour is a contract decision above this
   unit.
4. **NON-BLOCKING, handoff-spec / planner.** Handoff sec3.3 states the existing
   keys' "JSON insertion order is unchanged (`serde_json::json!` preserves it)".
   That mechanism is **factually wrong in this repository**: `serde_json` is on
   its default `BTreeMap`, proven by the Red's printed data map being
   alphabetical (`closure,id,in_degree,is_cycle,kind,label,out_degree,path`)
   while the code inserts `is_cycle,id,label,path,kind,closure,...`. The
   CONCLUSION is right and the true mechanism is stronger (alphabetical order is
   insertion-independent, hence better for cross-process byte stability).
   **Action:** correct the sentence; it is cited as a determinism mechanism and
   would mislead a successor into trusting insertion order.
5. **NON-BLOCKING, handoff-spec / planner.** Handoff sec1's determinism clause
   requires witness labels "in a fixed order (ascending `FileId`, matching
   `representative_cycle` order)". Those two halves are mutually inconsistent -
   a closed walk `[c0..ck, c0]` is not ascending by `FileId`. The normative
   sec3.3 pins "`representative_cycle` order", which is what the code does, so
   the implementation is right and the sec1 wording is wrong.
   **Action:** fix the wording before it is inherited.
6. **NON-BLOCKING, planner (declared sequencing divergence, accepted).** Plan:129
   says `build_include_elements`' 3rd parameter "becomes `&[CycleGroup]`". The
   implemented sequence instead adds the sibling
   `build_include_elements_from_groups` and defers the rename to unit E. The net
   end-state is preserved and every intermediate commit is buildable - verified,
   not assumed: `cargo build -p ariadne-cli` -> `ok (exit 0, 0 warnings
   emitted)`. This is the additive seam handoff sec1 and sec12 argue for and
   sec0 step 0.4's sequencing rationale depends on. **Action:** do not read
   plan:129 as satisfied until unit E lands; note it in the ledger so the plan
   text and the implemented sequence are not confused at closeout.
7. **NON-BLOCKING, reviewer capability.** The two Reds could not be
   re-executed: `crates/**` is read-only for this role, and the Red revision was
   never committed (correctly - "Never commit a Red state"), so exact
   panic-location/column matching is unavailable, unlike T2-B where the mutation
   WAS committed and the location could be pinned to the token column. The
   adjudication in section 5.1 therefore rests on exact-message matching against
   the committed assertion text, exact `left`-value derivation from the OLD
   source recovered from the diff's `-` lines, and a hand-execution of
   `detect_cycles` on the fixture that confirms the reported cycle list is
   literally what the real producer returns. Failure-set completeness is
   guaranteed because `crates/ui` contains zero `#[ignore]`. Same class as
   T2-B finding 1.
8. **NON-BLOCKING, implementer (coverage observation, follow-up owed).** No
   test compares the new builder's output against the legacy builder's on an
   acyclic graph, i.e. there is no behavioural "no hash leakage, no order
   change" parity lock - the most direct available proxy for the deferred
   cross-process half. Handoff sec5's exact-count lock (179 + 5 = 184) forecloses
   adding a sixth `#[test]` here, so this is not a shortfall against the
   implementer's contract. **Action:** add it in unit B, which owns the CLI seam
   (see section 8's companion recommendation).
9. **NON-BLOCKING, configuration (standing) - materially higher impact here than
   for any prior M23.5 unit.** `python tools/ui_smoke.py` and
   `python tools/bench_compare.py` were each attempted INDEPENDENTLY, once, and
   each refused with the identical verbatim catch-all text (section 10). Neither
   was routed through `distill.py` to evade the denial, and neither was retried.
   **UI-smoke evidence is MISSING, and this is the milestone's first unit that
   materially changes UI DTO output.** T2-A was graph-only and T2-B was
   test-only, so neither could have moved UI runtime behaviour and neither lost
   anything by going unsmoked. This unit changes the bytes the UI server serves:
   a new node-data key pair on grouped nodes, and - once unit B repoints the CLI
   - a changed `is_cycle`/`classes` population. There is therefore **no runtime
   evidence that the UI still boots, routes, and serves the new payload**, and
   the `Cache-Control: no-store` route contract is unverified for it. **Action:**
   close F10 before unit B repoints the CLI, and before T4.1.
10. **NON-BLOCKING, configuration (F9, reproduced).**
    `python tools/testrun.py --baseline` -> `usage error: missing value for
    --baseline`. Attempted once, recorded once, NOT treated as a gate failure and
    NOT worked around. The full-workspace invocation that `--baseline` wraps was
    run instead and compared by hand (section 10).
11. **NON-BLOCKING, implementer (touch-set breadth).** Commit `3c73cd3` lands nine
    planner- or reviewer-authored record files (the plan's T2.1 AMENDED row, the
    milestone ledger rows, the five `m235-t3{a,b,c,d,e}-handoff.md`, and the two
    T2-B reviewer records). Authorized by the orchestrator's declaration and
    consistent with T2-B deviation D-A, and every one of them is a record with no
    production content. The cost is attribution: planner work is carried by an
    implementer commit. **Action:** prefer having the milestone-lifecycle step own
    record landing, so implementer commit scopes stay single-purpose.

### Locks, verified

| Lock | Measured | Result |
|---|---|---|
| `crates/ui/src/lib.rs` byte-unchanged | superseded by the ONE sanctioned mod line (sec6 branch) | branch honoured, D1 |
| `crates/ui/tests/ui_test/main.rs` byte-unchanged | absent from all three commits | HOLDS |
| path-based `build_include_elements` present and unedited | signature lines are diff CONTEXT; `cargo build -p ariadne-cli` exit 0, 0 warnings | HOLDS |
| IV-9 `cycle_count` + `crates/report/**` untouched | no `crates/report` path in any commit | HOLDS |
| zero new hex literals, verdict colors across 3 themes | gate 5 no output; no `crates/ui/assets/` path in the range | HOLDS |
| hygiene R1/R2, `dto.rs` < 1000 | zero R1/R2; `dto.rs` = 901 lines | HOLDS |
| UTF-8 mojibake | `--all` and `--staged` both silent | HOLDS |
| no cross-crate dependency change | no `Cargo.toml` / `Cargo.lock` in any commit | HOLDS |
| `tests/fixtures/**`, `tests/golden/**` | absent from all three commits | HOLDS |
| fixture path in NO commit | absent from all three per-commit file lists | HOLDS |
| exactly one node loop + one edge loop | `dto.rs:685`, `dto.rs:729`, both inside the single shared emitter | HOLDS |
| `detect_cycles` / `dfs_cycle` still present (unit D owns the delete) | `graph.rs:234` and `graph.rs:260`, confirmed by direct read | HOLDS |
| no third cycle representation | `detect_cycle_groups` + `detect_cycles` only; `cycle_groups_view.rs` holds a CLASSIFICATION view, not a third detector | HOLDS |
| authored content pure ASCII | `encoding_guard.py --all` silent over the whole tracked tree | HOLDS |
| report evidence digest-shaped | journal quotes canonical lines, exit codes, capture paths; zero raw cargo output; overwriting caveat disclosed per sec9 | HOLDS |

---

## 10. Workspace-wide acceptance pass

### 10.1 The canonical workspace line - COMPLETED

`python tools/testrun.py --distill --timeout 3600`, exit 0, capture
`target/testrun-capture.txt` (2062 lines). Verbatim:

```
1174 passed / 0 failed / 4 ignored / 88 suites
```

Compared by hand against the ledger's T2-B-pinned `1169 / 0 / 4 / 88` and the
handoff's predicted `1174 / 0 / 4 / 88`:

| | passed | failed | ignored | suites |
|---|---|---|---|---|
| T2-B baseline (`b950963`) | 1169 | 0 | 4 | 88 |
| **T3-A measured (`ea82422`)** | **1174** | **0** | **4** | **88** |
| Handoff prediction | 1174 | 0 | 4 | 88 |
| **Delta** | **+5** | **+0** | **+0** | **+0** |

**The measured delta equals the predicted delta with ZERO unattributed
residual.** +5 tests is exactly this unit's five new `#[test]` functions, and
`+0 suites` is correct because `ui_test` is an existing target. This is the
F11 unattributed-residual discipline applied and it is clean. I independently
re-derived the per-file test attribution from the tree (section 4), so the +5
is attributed path by path, not merely consistent in total.

### 10.2 Every workspace-wide gate: ran, could not run, and why

| Gate | Command | Result |
|---|---|---|
| baseline | `python tools/testrun.py --baseline` | **DID NOT RUN** - verbatim: `usage error: missing value for --baseline`. Ledger M23.5 F9. Attempted ONCE, recorded once, NOT treated as a gate failure, NOT worked around, NOT routed through `distill.py`. |
| full workspace (what `--baseline` wraps) | `python tools/testrun.py --distill --timeout 3600` | `1174 passed / 0 failed / 4 ignored / 88 suites`, exit 0, capture 2062 lines |
| tool unit suites | `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 184 tests in 18.495s` then `OK` |
| encoding, all tracked | `python tools/encoding_guard.py --all` | no output, exit 0 |
| encoding, staged | `python tools/encoding_guard.py --staged` | no output, exit 0 |
| hygiene | `python tools/hygiene_check.py` | zero R1/R2; every printed line a legacy-file R3 warn |
| UI runtime smoke | `python tools/ui_smoke.py` | **REFUSED.** Verbatim: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}` |
| load-perf regression | `python tools/bench_compare.py` | **REFUSED.** Verbatim: identical text. |

**On the two refusals - both were attempted INDEPENDENTLY, once each, and
NEITHER was retried through `distill.py` or any other wrapper.** The catch-all
deny text is boilerplate and is recorded verbatim for both, as required.

**UI-smoke evidence is MISSING, and its absence is materially more significant
for this unit than for any earlier M23.5 unit** (finding 9). T2-A was
graph-only and T2-B was test-only, so neither could have moved UI runtime
behaviour and neither lost evidence by going unsmoked. T3-A is the milestone's
**first unit that materially changes UI DTO output**: it introduces two new
node-data keys on grouped nodes and, from unit B onward, a changed
`is_cycle`/`classes` population in the bytes the UI server serves. There is
consequently no runtime evidence that the UI still cold-boots, routes, and
serves the new payload, and the `Cache-Control: no-store` route contract is
unverified for it. Close F10 before unit B repoints the CLI and before T4.1.

**One bash refusal was self-inflicted and is recorded for honesty.** My first
`unittest discover` invocation appended `2>&1 | Select-Object -Last 6`; that
PIPELINE was refused with the same catch-all text. Recognising that the refusal
was caused by my own shell pipeline rather than by the command, I re-ran the
gate in plain form and it succeeded. Recorded because the failure mode this
brief warns about - reading one refusal as "bash is gone" and silently dropping
the remaining gates - is exactly what this instance could have become. No gate
was dropped: all eight ran, six on the first attempt and the seventh after
removing my own pipeline. (The eighth, `git`-shaped commands via bash, is
covered in 10.4.)

### 10.3 Fixture path, post-my-run state

`python tools/checkpoint_guard.py status`, re-run AFTER my own full workspace
test invocation: HEAD `ea82422`, **DIRTY FILES (none)**, UNTRACKED (only
`tasks/m235-t3a-verdict.md`, my own artifact).

The forbidden fixture `tests/fixtures/basic/artifacts/includes/report.json` is
**absent from the numstat dirty section both before and after my full
workspace run**. Content byte-equal to HEAD at both points. The porcelain view
still flags its stat entry and the census now reads `dirty=2` for this worktree
- accounted for exactly: `1` = the F12 stat-cache entry (section 7),
`1` = this review's own untracked verdict file. Nothing else. The implementer's
"no restoration was ever needed" claim is CONFIRMED, not assumed.

### 10.4 Worktree state left behind

HEAD `ea82422`, branch `tarjan-s-scc`, DIRTY FILES (none). Census unchanged at
8 worktrees. This review added two untracked files under `tasks/` by design:
`tasks/m235-t3a-verdict.md` and `tasks/m235-t3a-execution-results.md`. They are
mine to land, not the implementer's; nothing else in the tree was modified and
no git write was performed (write git is denied to this role).

For the record on the read-only-git constraint: bash-issued git was refused
throughout (the dedicated `git_git_log` / `git_git_status` / `git_git_show`
tools were used instead, and every `git show` argument was a quoted whole
revision). The affected evidence was obtained by other permitted forms: the
per-commit file lists came from the `git_git_show` tool output directly, and
the fixture's numstat state came from `python tools/checkpoint_guard.py status`,
whose dirty section is exactly that numstat. No commit was piped through
PowerShell into a file; all blob content was read from the checkout with the
Read tool.

---

## VERDICT: APPROVED

**0 blocking findings. 11 non-blocking findings:** 2 planner/architect
(an owner-less cross-process requirement and its unreachability at this HEAD),
1 architect/planner (the deferred two-process export lock), 2 handoff-spec
wording defects, 1 planner sequencing divergence from plan:129, 2 implementer
follow-ups owed (witness label drop; no old-vs-new parity lock), 1 reviewer
capability limit, 2 standing configuration items (F9, F10), 1 implementer
touch-set-breadth observation.

Scope discipline held in both directions: Phase 3 units B..E were neither
demanded nor credited; no work outside `crates/ui` was requested; the two
sec6 extension files were adjudicated on their evidence and ACCEPTED because
the handoff pre-authorised them and the implementer met every clause of that
clause including the mandatory journal declaration.

The milestone's headline invariant, IV-3, is not merely asserted at the DTO
layer but implemented as **group identity** (`usize` index equality, not
co-membership), and the decisive cross-group edge `(12, 20)` is locked on both
`data["is_cycle"] == false` and `classes == None`. The 184-test arithmetic is
independently re-derived and clean. The workspace delta is +5 with zero
unattributed residual.

**Follow-up dispositions carried to the ledger (nothing dropped):**

| Item | Owner | Due |
|---|---|---|
| Two-process `--export` byte-stability lock (plan:128, plan:147, IV-5) | unit B | before T4.1 |
| Old-vs-new builder parity test on an acyclic graph (no process spawn) | unit B | before T4.1 |
| F10 - `ui_smoke` / `bench_compare` off the allowlist; no runtime UI evidence for the new DTO output | configuration | before unit B repoints the CLI |
| F12 - pin `tests/fixtures/basic/artifacts/includes/report.json` in `.gitattributes` or make the writer emit LF | configuration | before T4.1 |
| F9 - `testrun.py --baseline` broken (`missing value for --baseline`) | configuration | before T4.1 |
| Witness label for an unregistered `representative_cycle` member | unit E or T4.1 | with the contract decision |
| Correct handoff sec3.3 insertion-order and sec1 "ascending FileId" wording | planner | next handoff revision |
| Note plan:129 (`build_include_elements` 3rd parameter "becomes `&[CycleGroup]`") as satisfied only at unit E | planner | ledger row |
| Prefer milestone-lifecycle to land planner/reviewer records rather than implementer commits | orchestration | next milestone |