# M23.5 T3-B Handoff - CLI Consumer Rewiring + Stage-5 Bench Re-Pin (crate:cli)

**Milestone:** M23.5 - Cycle Substrate: SCC Condensation + Iterative Tarjan
**Phase:** 3 of 4, unit B of 5. Plan task: **T3.1 CLI label + T3.2 CLI/payload
wiring + bench re-pin**. Unit A: `tasks/m235-t3a-handoff.md` (MUST be APPROVED
and landed -- this unit calls the entry point A adds).
**Plan:** `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md` sec2 Phase 3
(T3.1, T3.2), sec1.2 IV-9/IV-11, sec1.3
**Worktree:** `.kilo/worktrees/tarjan-s-scc`, branch `tarjan-s-scc`
**Journal:** `tasks/m235-t3b-journal.md` (you create it; record your own SHAs in it)

Every line reference was read out of the tree at `b950963`. **If a line
reference disagrees with the file you read, the FILE wins** -- report the drift
in your journal.

---

## 0. PREFLIGHT

**Step 0.1** - `python tools/checkpoint_guard.py status`; record HEAD, branch,
dirty files, untracked files.

**Step 0.2** - the CRLF fixture. `tests/fixtures/basic/artifacts/includes/report.json`
is UNPINNED by `.gitattributes` and this worktree has `core.autocrlf=true`, so a
full test run re-dirties its **stat entry** even when content is byte-identical
(ledger deferred row **M23.5 F12**; measured three times). If and only if
`git diff --numstat HEAD -- <path>` shows rows: restore with the Kilo `Read` tool
followed by the Kilo `Write` tool. NEVER `Set-Content`, `Out-File`, `>`, `>>`, or
`Get-Content | -replace`. `git checkout --` / `git restore` are mutating git and
are DENIED to every role.

**Step 0.3** - confirm unit A landed: `build_include_elements_from_groups` must
exist in `crates/ui/src/dto.rs`. If it does not, STOP and report -- you are
dispatched out of order.

**Step 0.4** - every commit carries an EXPLICIT `--scope` (gate 7). That fixture
path must appear in NO commit.

---

## 1. Objective and Invariants

### Objective

Repoint every `detect_cycles()` consumer in `crates/cli` at the group API, change
the one user-facing CLI label, update the stage-5 timing comment, and **re-pin
`payload_tail_bench.rs` against the recorded baseline while recording the new
number**. This is a rewiring unit: it introduces no new algorithm.

### The four call sites, re-verified against the tree at `b950963`

| # | Site | Current text |
|---|---|---|
| B1 | `crates/cli/src/commands/ui.rs:310` | `let cycles = include_graph.detect_cycles();` |
| B2 | `crates/cli/src/commands/ui.rs:316-322` | `ariadne_ui::build_include_elements(&include_graph, &closures, &cycles, &node_maps.languages, &node_maps.unit_kinds)` |
| B3 | `crates/cli/tests/payload_tail_bench.rs:295` | `timed(\|\| (graph.generate_report(), graph.detect_cycles()));` |
| B4 | `crates/cli/tests/payload_tail_bench.rs:298` | `ariadne_ui::build_include_elements(&graph, &closures, &cycles, &no_decoration, &no_decoration)` |

Plus the two label/comment sites:

| # | Site | Current text |
|---|---|---|
| B5 | `crates/cli/src/commands/includes.rs:113` | `println!("  Cycles Detected: {}", report.cycle_count);` |
| B6 | `crates/cli/src/commands/ui.rs:103` | `// generate_report+detect_cycles 4.7ms . build_include_elements 5.1ms .` (inside the stage-5 calibration comment block, `:95-112`) |

And the three bench-doc sites that must move with B3:
`payload_tail_bench.rs:20` (phase-boundary note naming `generate_report`/`detect_cycles`),
`:39` (baseline table row `generate_report+detect_cycles (combined) | 5.8`),
`:361` (emitted column label `("generate_report+detect_cycles (combined)", report_cycles_ms)`).

**Do NOT confuse `detect_cycles` with `detect_cycles_in_units`** at
`crates/transform/src/engine/mod.rs:72` and `:133`. That is a different,
unrelated function on `GeneratedUnit` and is **byte-locked out of scope**.

### Invariants this unit preserves

- **IV-9 `cycle_count` naming survives.** You do not rename any DTO field.
  `ui.rs:378` (`cycle_count: report.cycle_count`) stays byte-identical.
- **D27 5-arg shape preserved.** `node_maps::build` stays where it is
  (`ui.rs:311-315`) and the DTO call keeps all five arguments.
- **Zero new hex literals**, verdict colors unchanged across light, dark and
  dark-steampunk -- this unit touches no UI asset.
- **IV-11 field name + skip-when-zero serde shape** untouched: you change no
  report code. `crates/graph/src/graph.rs` is read-only in this unit.

---

## 2. Touch-set (hard; any edit outside is a blocker -- report, do not fix)

- `crates/cli/src/commands/ui.rs` (B1, B2, B6)
- `crates/cli/src/commands/includes.rs` (B5 + the label helper + its Red test)
- `crates/cli/tests/payload_tail_bench.rs` (B3, B4, `:20`, `:39`, `:361`)
- `tasks/m235-t3b-journal.md` (**new**, yours)

You MUST NOT touch: `crates/cli/src/commands/ui/node_maps.rs`, any other file
under `crates/cli/src/**`, `crates/cli/Cargo.toml`, `crates/ui/**`,
`crates/graph/**`, `crates/report/**`, `tests/**`, `tools/**`,
`.kilo/plans/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`.

**Pre-existing defect you must NOT silently "fix":** `ui.rs:311` reads
`let node_maps = node_maps::build(` at column 0 while its neighbours are
indented 4. Leave it byte-identical unless your edit forces the line to change;
if it does, log it as a declared deviation. It is not an R1/R2 hygiene finding
and it is not your defect.

---

## 3. Green - the minimal implementation bound

1. **B1/B2** - `let cycle_groups = include_graph.detect_cycle_groups();` and call
   `ariadne_ui::build_include_elements_from_groups(&include_graph, &closures,
   &cycle_groups, &node_maps.languages, &node_maps.unit_kinds)`. Keep the
   `node_maps::build` call and the 5-argument shape exactly as they are. Do not
   rename anything else.
2. **B5** - the label becomes `Cycle Groups Detected`, keeping the two leading
   spaces that every other line in that summary block carries. See sec4 for the
   extraction you must do first.
3. **B6** - in the stage-5 calibration comment, replace `detect_cycles` with
   `detect_cycle_groups` **and** replace the `4.7ms` figure with the number you
   measure in gate 2. If your measured release number lands within noise of
   `4.7`, say so explicitly in the comment rather than inventing a precision the
   measurement does not support.
4. **B3/B4** - `timed(|| (graph.generate_report(), graph.detect_cycle_groups()))`
   and the `..._from_groups(...)` DTO call; rename the column label at `:361` to
   `generate_report+detect_cycle_groups (combined)`; update the phase-boundary
   note at `:20`; re-pin the baseline table row at `:39` with your measured
   number.

---

## 4. Red - one compiling Red, on the label

This unit is a rewiring and has **no** algorithmic Red. The one genuinely new,
observable behaviour is the CLI label, and it has a legitimate compiling Red --
extract the label to a named helper first (a pure refactor that keeps the
current bytes), then write the Red against the helper.

**Step 1 - extract the seam (refactor, stays Green).** `includes.rs` already
extracts its other labels into named helpers (`aggregate_kind_label` used at
`:95-102`, `unresolved_label` at `:110`, `top_closure_heading` at `:117`,
`unique_aggregates_label` and `graph_summary_title` locked by the test at
`:169`). Add one sibling in the same style:

```rust
fn cycle_groups_label(_vocabulary: Option<&DependencyVocabulary>) -> &'static str {
    "  Cycles Detected"
}
```

and make `:113` print `println!("{}: {}", cycle_groups_label(vocabulary.as_ref()), report.cycle_count);`
so the line shape matches its neighbours. Take the `vocabulary` parameter for
signature symmetry with the other label helpers even though this label does not
need it -- and say why in a one-line comment, so the next reader does not read
it as an oversight.

**Step 2 - the Red.** Add to `crates/cli/src/commands/includes.rs`'s existing
`#[cfg(test)] mod tests` (it starts at `:152`; existing tests at `:168`, `:188`,
`:205`):

```
includes_summary_labels_report_cycle_groups_not_cycle_paths
```

asserting `cycle_groups_label(cpp) == "  Cycle Groups Detected"`.

**Expected failure reason:** assertion fails, left `"  Cycles Detected"`, right
`"  Cycle Groups Detected"`. It compiles; it is a real assertion failure; the
expected line at that point is
`86 passed / 0 failed / 1 ignored / 3 suites` (gate 1 below with one new lib
test, one failing).

**Step 3 - Green.** Change the helper's returned string literal. Nothing else.

**What is NOT Red, and must not be claimed as such:** the payload behaviour
change (an intra-group chord now carries the `cycle` class) is proven by unit A's
`intra_group_non_tree_edge_carries_the_cycle_class`, not by anything you can
observe in `crates/cli`. Say so in your journal.

---

## 5. The bench re-pin - read this before you touch the numbers

**Two different baseline numbers exist and you must not conflate them:**

| Number | Where it lives | What it is |
|---|---|---|
| `5.8` | `crates/cli/tests/payload_tail_bench.rs:39` | the bench's own recorded baseline table row, recorded 2026-09-29 in this bench, `--release` |
| `4.7ms` | `crates/cli/src/commands/ui.rs:103` | the stage-5 boot calibration figure quoted in the CLI's ms table, same 2026-09-29 calibration round |

The plan and the milestone brief quote `4.7 ms` as "the `generate_report +
detect_cycles` baseline". That is the `ui.rs:103` figure. Your re-pin measures
the bench row (`5.8`) and updates `ui.rs:103` with the number you measured. Say
in your journal which baseline you compared against and why they differ.

**The measurement must be `--release`.** The bench's own run instruction at
`payload_tail_bench.rs:62-63` is:

```
cargo test -p ariadne-cli --test payload_tail_bench --release -- \
  --ignored --nocapture --test-threads=1
```

A debug-profile number is NOT comparable to a release baseline and must not be
written into either table. If the release build will not fit your gate timeout,
report that as a blocker in your journal rather than substituting a debug number.

**The test is `#[ignore]`d** (`payload_tail_bench.rs:415-417`, attribute text:
`synthetic 520-header stress fixture; run explicitly for stage-5 tail timing`),
so gate 1 runs zero tests by design and gate 2 runs exactly one.

**The comparison is UNVERIFIED.** `python tools/bench_compare.py` is **refused**
by the reviewer bash allowlist (ledger deferred row **M23.5 F10**; reconfirmed
independently in T2-A finding 9 and T2-B finding 9). You therefore have **no
reviewer-run perf-regression evidence** for `detect_cycle_groups`. Write this
sentence into your journal and into the comment you add at `payload_tail_bench.rs`:
the new number is a **single local measurement on this machine**, not a
bench-compare-verified result. Do NOT run `bench_compare.py`; do NOT route it
through `distill.py` to evade the denial. SCC detection is linear in V+E, so
parity is the expected outcome -- "expected parity" is not "measured parity".

---

## 6. Required tests

Exactly **one** new `#[test]` in `crates/cli/src/commands/includes.rs`:
`includes_summary_labels_report_cycle_groups_not_cycle_paths` (sec4).

The three existing `includes.rs` tests (`:168`, `:188`, `:205`) must pass
unedited. The `-p ariadne-cli` lib target carried **86** `#[test]` functions at
`b950963` (counted across `crates/cli/src/**`; zero `#[ignore]` there), so after
this unit it is **87**.

---

## 7. Refactor and hygiene targets

- Functions < 100 lines. `crates/cli/src/commands/includes.rs` is 224 lines and
  `crates/cli/src/commands/ui.rs` is 754 lines -- **neither approaches the
  1000-line ceiling and your change makes both smaller or neutral.** Do not
  extract anything else from `ui.rs`; it is not your seam.
- No `tmp`, `res`, `foo`, `bar`, `test1`; no single-letter locals in code you
  author; one responsibility per function; no source line >= 2000 characters.
- `python tools/hygiene_check.py`: zero R1/R2.
- **Authored content is pure ASCII.** The bench table uses `|` and spaces only;
  do not introduce `--`, `->`, or any typographic punctuation into the table.

---

## 8. Acceptance - gates, exact commands, expected verdict lines

Every gate in DIRECT form. **Never write a gate as
`python tools/distill.py -- <argv>`** unless `<argv>` is a `cargo build`.

**Never write a bare-word test filter** -- `testrun.py ... -- <word>` passes
`<word>` to cargo as a TEST-NAME filter and selected zero tests in Phase 1,
returning `0 passed / 0 failed / 0 ignored / 7 suites` with exit 0 (Phase 1
verdict finding 2). Use `--test <target>`.

**7 invocations.**

```
1. python tools/testrun.py --distill --timeout 1200 -p ariadne-cli
2. python tools/testrun.py --distill --timeout 3600 --release -p ariadne-cli --test payload_tail_bench -- --ignored --nocapture --test-threads=1
3. python tools/hygiene_check.py
4. python tools/encoding_guard.py --staged
5. python tools/checkpoint_guard.py status
6. python tools/checkpoint_guard.py status
7. python tools/commit_scoped.py --scope 'crates/cli/src/commands/ui.rs' --scope 'crates/cli/src/commands/includes.rs' --scope 'crates/cli/tests/payload_tail_bench.rs' --scope 'tasks/m235-t3b-journal.md' -m "feat(cli): wire UI payload and includes summary to cycle groups"
```

Gate 6 is a deliberate duplicate of gate 5, run AFTER gate 2: gate 2 executes the
full pipeline over a 520-header synthetic fixture and therefore re-dirties the
unpinned artifact path (sec0 step 2). Prove the restore held after the heaviest
run in the unit.

**Expected verdict lines:**

| Gate | Expected verbatim line |
|---|---|
| 1 | `87 passed / 0 failed / 1 ignored / 3 suites` |
| 2 | `1 passed / 0 failed / 0 ignored / 1 suites` |
| 3 | zero R1/R2 lines; R3 warns, if any, are legacy files only |
| 4 | clean, no output |

**The arithmetic behind gate 1:** `crates/cli/src/**` carries exactly **86**
`#[test]` functions at `b950963` (no `#[ignore]` in `src`); this unit adds one
-> **87**. The `payload_tail_bench` integration target contributes
`0 passed / 1 ignored` (its single test is `#[ignore]`d), and doc-tests
contribute 0. Three suites = lib + `payload_tail_bench` + doc-tests.

**Gate 2 is not vacuous:** it executes one real test that builds a 520-header /
40-TU synthetic repo, runs the full pipeline, and prints the stage-5 tail table
to the capture at `target/testrun-capture.txt`. Read that capture for the new
`generate_report+detect_cycle_groups` row value and quote the number in your
journal and in `ui.rs:103`. **That capture path is overwritten by gate 1 if you
re-run it** -- quote the value in your journal text so it survives.

**Effect on the canonical workspace line:** none. This unit adds one test, so the
reviewer's acceptance pass should read **`1175 passed / 0 failed / 4 ignored /
88 suites`** (1174 after unit A, `+1`). You do not run the workspace-wide gate.

---

## 9. Frozen locks - each with a proving command

| Lock | Proving command | Expected |
|---|---|---|
| `node_maps::build` and the 5-arg DTO shape preserved | reviewer's `git diff` on `ui.rs:305-322` | structure unchanged, only the cycle argument and the callee name |
| `crates/cli/src/commands/ui/node_maps.rs` byte-unchanged | reviewer's per-commit `git diff --name-only` | absent |
| `crates/graph/**` byte-unchanged (`detect_cycles` still at `graph.rs:234`) | reviewer's `git diff --name-only`; reviewer grep | absent / still present |
| `crates/ui/**` byte-unchanged | reviewer's `git diff --name-only` | absent (unit A's and unit E's files) |
| `ui.rs:378` `cycle_count: report.cycle_count` unchanged | reviewer's `git diff` on that line | unchanged |
| existing `includes.rs` tests (`:168`, `:188`, `:205`) green | gate 1 | green |
| the three existing `includes.rs` label helpers unchanged | reviewer's `git diff` | unchanged |
| zero new hex literals | `python tools/theme_token_check.py --check-added --base <base>` is the reviewer's gate; you touch no asset | no asset in any commit |
| UTF-8 mojibake | gate 4 | clean; authored content pure ASCII |
| R1/R2 hygiene | gate 3 | no R1/R2 |
| touch-set | reviewer's per-commit `git diff --name-only` | exactly the four in-set paths |
| `tests/fixtures/**`, `tests/golden/**` | reviewer's `git diff --name-only` | absent |
| the unpinned fixture path | reviewer | in no commit (ledger F12) |

Workspace-wide gates are NOT yours: `python tools/testrun.py --baseline` (BROKEN,
ledger F9: `usage error: missing value for --baseline`),
`python -m unittest discover -s tools -p "*_tests.py"`,
`python tools/ui_smoke.py` and `python tools/bench_compare.py` (both REFUSED,
ledger F10). Do not work around any of them, and never route a refused command
through `distill.py`.

---

## 10. Reviewer gate reference

Adjudicated via the `review-gate` skill; its commit-vs-touch-set check enforces
sec2. The reviewer will audit:

1. **The Red's honesty.** The label helper must have been extracted with the
   CURRENT bytes first, then the test written, then the string changed. A commit
   that shows the helper and the new string together has no Red.
2. **The bench number's provenance.** Which baseline (`5.8` at `:39` or `4.7` at
   `ui.rs:103`) the comparison was made against; that the run was `--release`;
   that the journal carries the explicit UNVERIFIED sentence; and that no
   `bench_compare.py` was run or evaded. **A handoff that implies reviewer-verified
   perf evidence is a blocking finding** -- there is none.
3. **The two-site comment sync.** `ui.rs:103` and `payload_tail_bench.rs:20/:39/:361`
   must all name `detect_cycle_groups` and the measured number. A stale
   `detect_cycles` in any of them is a finding, and so is a `4.7ms` left in place
   next to a freshly measured number with no explanation.
4. **No over-claim on the payload change.** The chord-classification behaviour is
   unit A's proof; if this journal claims otherwise, that is a finding.
5. **Label shape.** Two leading spaces, matching every neighbouring line.
6. **Line-count and no-scope-creep** discipline on `ui.rs`.

**Report back (final message):** commit SHAs (also in your journal); the verbatim
Red failure message; Green evidence quoting gate 1's canonical line, gate 2's
canonical line and the **measured `generate_report+detect_cycle_groups` value**;
the baseline you compared against; the UNVERIFIED sentence; every deviation with
its reason; out-of-scope observations, report-only. Never paste raw cargo or git
output.

---

## 11. Discipline clauses

- **Command execution.** `bash` is an allowlist restricted to the distilling
  entry points and `commit_scoped.py`. Raw `cargo` and raw `git` are DENIED.
- **Reading a refusal.** A refused command ALWAYS reports the catch-all deny
  rule, whatever the real cause. Treat it as boilerplate. Attempt each gate
  INDEPENDENTLY; never drop a gate because its first invocation was refused. If a
  gate cannot run, record the VERBATIM refusal in your journal rather than
  silently narrowing the acceptance set.
- **Strict TDD.** Extract the label seam -> write the Red -> observe it -> change
  the string -> Green -> rewire B1..B4 and B6 -> Refactor. Never weaken an
  assertion to make it pass. Never commit a Red state.
- **Code hygiene.** Functions < 100 lines, files < 1000 lines, no
  `tmp`/`res`/`foo`/`bar`/`test1`, no single-letter locals in code you author, no
  source line >= 2000 characters.
- **UTF-8 file-edit discipline.** NEVER `Set-Content`, `Out-File`, `>`, `>>`, or
  `Get-Content | -replace` on a tracked file. Kilo Read/Edit/Write tools, or
  byte-level `open(path, "wb")`. Pure-ASCII authored content. Quote whole git
  arguments: `git show "rev^:path"`.
- **Scope.** Touch only sec2. Out-of-scope breakage: report it, do not fix it.
- **Journal.** Append a 1-3 line checkpoint after EACH Green gate, including the
  measured bench number. Record your own SHAs in it.
- **Context exhaustion.** If it looms, STOP and return a disk-state inventory. A
  dead session is never resumed -- a successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/m235-t3b-handoff.md --scope ... --cmd ...`.

---

## 12. Sizing check (handoff law, stated explicitly)

| Metric | Bound | This unit |
|---|---|---|
| Scope | <= 1 Cargo crate OR <= 1 top-level module directory | `crates/cli` only -- **1 crate** |
| Gate commands | ~10 | **7** |
| Produced lines (implementation + test diff) | ~1200 | **~120** (`ui.rs` +8/-6, `includes.rs` +18/-3, `payload_tail_bench.rs` +12/-9, journal ~70) |

**Single handoff unit -- fits with wide headroom.**

Why it is separate from unit A: changing `build_include_elements`'s existing
3rd parameter type would have broken `crates/cli` at the A/B boundary, which the
hard sequencing constraint forbids. Adding a sibling entry point in A lets the
CLI rewiring land as a `crates/cli`-only commit. Why it is separate from unit C
(`tests/integration`): different top-level module directory. **Unit C is
file-disjoint from this unit** and the two may run in parallel in separate
worktrees; in this shared worktree dispatch them sequentially.

---

## 13. Authorization and ordering

This handoff, `tasks/m235-kickoff.md`, the M23.5 milestone row in
`tasks/milestone_ledger.md`, and an APPROVED `tasks/m235-t3a-handoff.md` are the
sole authorization for this unit.

Runs after unit A. Unit D (`tasks/m235-t3d-handoff.md`) -- the atomic delete of
`detect_cycles`/`dfs_cycle` -- requires this unit AND unit C to have landed,
because after both, `detect_cycles` has no caller outside `crates/graph`. Do not
delete `detect_cycles` or `dfs_cycle` yourself, do not touch
`crates/graph/**`, and do not begin Phase 4 (T4.1), which remains UNAUTHORIZED.