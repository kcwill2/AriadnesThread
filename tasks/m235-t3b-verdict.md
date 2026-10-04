# M23.5 T3-B Verdict - CLI Consumer Rewiring + Stage-5 Bench Re-Pin (crate:cli)

**Reviewer:** reviewer role, `review-gate` skill
**Date:** 2026-10-04
**Handoff:** `tasks/m235-t3b-handoff.md` (Phase 3 unit B of 5, plan T3.1 CLI half + T3.2 CLI/payload wiring)
**Range reviewed:** `ea82422..91967c9` (base `ea82422` = T3-A tip; HEAD `91967c9`, branch `tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`)
**Commits (oldest first):** `9d831bf`, `52f8aa2`, `a55091c`, `9194c25`, `9f30c9a`, `91967c9`
**Execution-results addendum:** `tasks/m235-t3b-execution-results.md`
**Deferred item discharged here:** T3-A verdict sec 8 / finding 1 (two-process export byte-stability), owner reassigned to unit B.

This file is written INCREMENTALLY, cheap gates first, per the review brief (a
prior reviewer session on a sibling unit died three times to an upstream idle
timeout having written nothing). Sections 1-7 are final; section 8 onward is
appended as each gate completes.

---

## 1. Ground truth

`python tools/checkpoint_guard.py status`, reviewer-run at session start:

- HEAD `91967c9`, branch `tarjan-s-scc`
- DIRTY FILES (`git diff HEAD --numstat`): `(none)`
- UNTRACKED FILES: `(none)`
- RECENT COMMITS (last 5): `91967c9`, `9f30c9a`, `9194c25`, `a55091c`, `52f8aa2`
- WORKTREE CENSUS: 8 worktrees; `.kilo/worktrees/tarjan-s-scc` `ahead=16 behind=13 dirty=1`

**The `dirty=1` census entry is adjudicated in section 6** (F12 stat-cache
mechanism, confirmed: the numstat dirty section is EMPTY, so the forbidden
fixture's content is byte-equal to HEAD).

**Base identification.** The handoff pins line references at `b950963`, but the
unit's actual base is `ea82422` (T3-A's tip - handoff sec13 makes an APPROVED
T3-A a precondition and sec0 step 0.3 requires the T3-A entry point to exist).
Confirmed: `build_include_elements_from_groups` is present (T3-A landed), and
`ea82422` is the commit immediately preceding `9d831bf`. Reviewer-measured range
`ea82422..91967c9` = the six declared commits, nothing else.

---

## 2. Commit-versus-touch-set check (PASS, 6 of 6 commits)

**Enforced touch-set.** Handoff sec2 four paths:
`crates/cli/src/commands/ui.rs`, `crates/cli/src/commands/includes.rs`,
`crates/cli/tests/payload_tail_bench.rs`, `tasks/m235-t3b-journal.md`.
Plus the orchestrator-declared records carried by the task message (T3-A
reviewer's `tasks/m235-t3a-verdict.md` + `tasks/m235-t3a-execution-results.md`,
recorded in journal Checkpoint 0).

### Range-level name list, reviewer-measured

`git diff --name-only ea82422..HEAD`:

```
crates/cli/src/commands/includes.rs
crates/cli/src/commands/ui.rs
crates/cli/tests/payload_tail_bench.rs
tasks/m235-t3a-execution-results.md
tasks/m235-t3a-verdict.md
tasks/m235-t3b-journal.md
```

Three production paths, all in sec2; three records, all in-set or
orchestrator-declared. **Absent, as required:** `crates/graph/**`,
`crates/ui/**`, `crates/report/**`, `crates/cli/src/commands/ui/node_maps.rs`,
every other `crates/cli/src/**` file, `crates/cli/Cargo.toml`, `tests/**`,
`tools/**`, `.kilo/plans/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`,
`crates/lang-scala/**`. **The Scala lock holds with no plugin-side change
(IV-12):** `tests/integration/tests/m20_scala_dependency_graph_test.rs:154` and
`m22_scala_load_analysis_test.rs:442` still call `graph.detect_cycles()`
(grep-verified at HEAD) and no `tests/**` or `crates/lang-scala/**` path appears
in the range. They migrate in unit C, as the handoff's unit boundary intends.

### Per-commit (oldest first), each judged alone

| # | SHA | Subject | Paths | In touch-set? |
|---|---|---|---|---|
| 1 | `9d831bf` | `refactor(cli): extract cycle_groups_label seam with the current label bytes` | `includes.rs` only (+9/-1) | yes (sec2) |
| 2 | `52f8aa2` | `feat(cli): label includes summary line as Cycle Groups Detected` | `includes.rs` only (+12/-1) | yes (sec2) |
| 3 | `a55091c` | `feat(cli): wire UI payload and includes summary to cycle groups` | `ui.rs` (+3/-3), `payload_tail_bench.rs` (+127/-5) | yes (sec2) |
| 4 | `9194c25` | `docs(cli): re-pin stage-5 bench row and calibration to detect_cycle_groups 5.6ms` | `ui.rs` (+13/-1), `payload_tail_bench.rs` (+29/-2) | yes (sec2) |
| 5 | `9f30c9a` | `docs(tasks): land M23.5 T3-A reviewer records` | `tasks/m235-t3a-verdict.md`, `tasks/m235-t3a-execution-results.md` (+1168/-0, records only) | yes (orchestrator-declared) |
| 6 | `91967c9` | `docs(tasks): M23.5 T3-B journal ...` | `tasks/m235-t3b-journal.md` (+309/-0) | yes (sec2) |

**The forbidden fixture.** `tests/fixtures/basic/artifacts/includes/report.json`
appears in **NO** commit of this range. Verified from the six per-commit
`--stat` name lists above (stat 1 and 2 each show exactly one path; stat 3, 4,
5, 6 likewise contain no `tests/` path at all). **Lock HOLDS**, independently of
the journal's claim.

**Scope verdict: PASS.** No commit lands outside the declared touch-set plus the
orchestrator-declared records. This is the check that would catch an over-broad
touch-set, and it is clean in both directions: the set is enforced AND it is not
over-broad.

---

## 3. STEP 1 gates, reviewer-run (hygiene, encoding, theme tokens)

| Gate | Command | Reviewer-measured result |
|---|---|---|
| hygiene | `python tools/hygiene_check.py` | **ZERO R1/R2 lines workspace-wide.** Every printed line is `R3 (warn)`. |
| encoding, all tracked | `python tools/encoding_guard.py --all` | **no output** (exit 0) |
| theme tokens | `python tools/theme_token_check.py --check-added --base ea82422` | **no output**, exit 0 = zero added hex literals |

### The R3 claim, independently checked (not relayed)

`crates/cli/src/commands/ui.rs` carries exactly five R3 warnings, at `:557`,
`:656`, `:659`, `:665`, `:732` (`single-letter local p` x3, `banned identifier
bar` x2). The journal claims all five are pre-existing lines in
`swap_into_state` / `make_bar_progress` / `make_boot_progress` / the error path,
shifted +12 by the implementer's comment block. **CONFIRMED by construction,
not by trusting the shift arithmetic:** the two `9194c25` hunks that touch
`ui.rs` are confined to the calibration comment block at `:100-118`, and
`a55091c`'s `ui.rs` hunk is confined to `:307-322`. Every flagged line number
(557 and above) is far below both regions, and `ui.rs` is 766 lines at HEAD
(under the 1000-line R2 ceiling, and net-neutral-to-smaller per sec7).
`crates/cli/src/commands/includes.rs` and
`crates/cli/tests/payload_tail_bench.rs` carry **zero** flags each - the
scanner demonstrably reaches both trees (it flags sibling `ui.rs`,
`builder_tests.rs`, `lib.rs` in the same crate), so the zero is measured, not a
skip. R3 is warn-only per `AGENTS.md`.

**Theme tokens.** Zero added hex literals against base `ea82422`. Structurally
confirmed as well: the range touches **no** file under `crates/ui/assets/**` and
no CSS/JS/template anywhere, so the theme surface is not in the diff at all and
light / dark / dark-steampunk semantic verdict colors cannot have moved (handoff
sec1's "this unit touches no UI asset"). Only red-edge *coverage* grows, which
is plan sec1.1 item 1's intent.

**Encoding.** `--all` silent over the whole tracked tree, so authored content is
ASCII-clean and no double-encoded UTF-8 entered through any of the six commits.

---

## 4. STEP 2: scoped gates - reviewer-run, nothing relayed

| Gate | Command | Reviewer-measured canonical line |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-cli` | `88 passed / 0 failed / 1 ignored / 5 suites` (exit 0, capture 120 lines) |
| 2 | `python tools/testrun.py --distill --timeout 3600 --release -p ariadne-cli --test payload_tail_bench -- -- --ignored --nocapture --test-threads=1` | `1 passed / 0 failed / 0 ignored / 1 suites` (exit 0, capture 26 lines) |

Gate 1's line matches the implementer's final gate-1 report **verbatim**, and
gate 2's matches the handoff sec8 table **verbatim**.

### Gate 1 is not vacuous - the suite decomposition, from the capture

`target/testrun-capture.txt` lines 116-120 name exactly five suite headers, and
lines 91/96/101/108/113 are exactly five `test result:` lines:

| Suite header (capture line) | `test result:` |
|---|---|
| `Running unittests src/lib.rs` (116) | `87 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` (91) |
| `Running unittests src/main.rs` (117) | `0 passed; ... 0 filtered out` (96) |
| `Running unittests src/bin/tool.rs` (118) | `0 passed; ... 0 filtered out` (101) |
| `Running tests\payload_tail_bench.rs` (119) | `1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out` (108) |
| `Doc-tests ariadne_cli` (120) | `0 passed; ... 0 filtered out` (113) |

**Every `test result:` line reads `0 filtered out`** - nothing was silently
filtered, which is the exact vacuity mode handoff sec8 warns about. 87 + 1 = 88
passed, 1 ignored, 5 suites. **Zero unattributed residual.**

Both required tests are named PASSING in the capture:
`commands::includes::tests::includes_summary_labels_report_cycle_groups_not_cycle_paths ... ok`
(line 10) and `export_bytes_are_identical_across_two_separate_processes ... ok`
(line 106, in `0.15s`). The bench is correctly ignored at gate 1 (line 105,
`payload_tail_phase_timing ... ignored, synthetic 520-header stress fixture`).

### D3's suite arithmetic - verified, and the planner's is wrong

`crates/cli/Cargo.toml:7-13` declares **two** `[[bin]]` targets:

```toml
[[bin]]
name = "ariadne"
path = "src/main.rs"

[[bin]]
name = "tool"
path = "src/bin/tool.rs"
```

Cargo builds and runs an unittests binary for each declared bin target even at
zero tests - which is exactly what capture lines 96 and 101 show. So
`suites = lib + main.rs bin + tool.rs bin + payload_tail_bench + doc-tests = 5`.
The handoff's `3` counted only lib + integration + doc-tests and omitted both bin
targets. **The implementer's D3 arithmetic is exactly right and is proven from
the tree, not merely asserted; the planner's `3` is a slip.** The same slip is
baked into the ledger's `-p ariadne-cli = 86/0/1/3` row (the implementer notes
it), so that row is wrong too and should be corrected at closeout.

### Gate 2 is not vacuous, and the bench is RELEASProfiled

The bench is `#[ignore]`d, so "did it actually run" had to be measured, not
assumed - an unrun bench reporting zero would not be a pass. Capture evidence:

- line 2-3: `running 1 test` / `test payload_tail_phase_timing ... TAIL_BENCH headers=520 tus=40 reduced_symbols=2160 file_metrics=560 justification_entries=8960 plans=520 conservative_rows=520`
- line 20: `ok`; line 22: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 13.22s`
- line 25: `Finished \`release\` profile [optimized] target(s)`; line 26: `Running tests\payload_tail_bench.rs (target\release\deps\payload_tail_bench-....exe)`

**It genuinely ran, over the full 520-header / 40-TU fixture, in the RELEASE
profile.** `1 filtered out` is the export-stability test, correctly excluded by
`--ignored`.

### Bench re-pin honesty - reviewer reproduced the number

Reviewer-measured release run, capture line 14:

```
generate_report+detect_cycle_groups (combined)   |      5.5
```

Pinned value in the tree: **5.6** (`ui.rs:103` and `payload_tail_bench.rs:40`).
The implementer's single local measurement was 5.6; my independent release run
gives 5.5. That is a 0.1 ms / ~2% spread and it **independently reproduces the
re-pin**, which is materially stronger evidence than the implementer's own
single sample.

Same-run neighbours for context, implementer -> reviewer:
`build_include_elements` 5.7 -> 5.3; `path_to_id+file_collateral` 30.3 -> 29.6;
`ensure_load total` 2067.3 -> 2273.5. **Run-to-run variance on this machine is
5-10% on several rows simultaneously**, which is direct evidence that the
`5.8` -> `5.6` comparison is "parity within single-run noise" and NOT a measured
regression - and equally that it is not a verified perf result.

**Three honesty properties, each independently confirmed:**

1. **The pre-existing baseline discrepancy was RECORDED, not silently resolved
   in the convenient direction.** `payload_tail_bench.rs:62-85` states the prior
   figure was `5.8` (this bench's own 2026-09-29 table), that the plan's stated
   baseline was a DIFFERENT number `4.7` (`ui.rs:103`), that "`4.7` and `5.8`
   were never reconciled in the tree", that the re-pin was made against the
   bench's OWN `5.8`, and that "the split `5.8`-vs-`4.7` pre-unit baseline is
   recorded here and in `tasks/m235-t3b-journal.md` rather than silently
   resolved". `ui.rs:106-113` carries the mirror statement. Neither direction was
   picked to flatter the result: the measured 5.6 sits with the bench table, and
   the implementer says so explicitly rather than pretending the 4.7 baseline
   was the comparand.
2. **The UNVERIFIED label appears in the journal AND in the file comments.**
   Journal Checkpoint 2 carries it. `payload_tail_bench.rs:79-85` carries it
   ("UNVERIFIED (handoff sec 5): `5.6` is a SINGLE LOCAL MEASUREMENT on this
   machine on 2026-10-04, NOT a bench-compare-verified result ...
   `"Expected parity" is not "measured parity"` ... and it was NOT routed through
   `distill.py` to evade the denial"). `ui.rs:114-118` carries it again. Handoff
   sec10 audit 2's blocking condition ("a handoff that implies reviewer-verified
   perf evidence is a blocking finding") is therefore **not** triggered.
3. **No perf evidence is claimed anywhere.** Reviewer independently confirms
   `bench_compare.py` is still not runnable (section 8) and that the pinned
   number's own comment says so. There is **no verified perf-regression evidence
   for `detect_cycle_groups` in this milestone**, and the tree says so at both
   sites.

### The Red - genuine, and its location is pinned to a surviving line

The Red revision was correctly never committed ("Never commit a Red state"), so
the raw capture is unrecoverable from disk. The committed trail plus the code
pin it as tightly as is possible without it:

- `9d831bf` adds `cycle_groups_label(_vocabulary)` returning the **OLD** bytes
  `"  Cycles Detected"` and repoints `:121` through it, with **no test in the
  commit**. `52f8aa2` then flips the literal **and** adds the test. So the
  "helper with current bytes first, then the test, then the string" ordering
  required by sec10 audit 1 is **provable from history**, not merely claimed.
- Journal's verbatim panic: `panicked at crates\cli\src\commands\includes.rs:220:9: assertion left == right failed / left: "  Cycles Detected"`.
- At HEAD, `includes.rs:220` is exactly
  `assert_eq!(cycle_groups_label(cpp), "  Cycle Groups Detected");` and its
  leading indent is 8 spaces, i.e. column 9 - **matching the reported column**.
  The string flip is an in-place single-line edit, so the Red revision's line
  numbering is identical to HEAD's; there is no drift to hide behind.
- `left: "  Cycles Detected"` is the helper's return value under `assert_eq!` and
  is precisely what `9d831bf` committed, while `right` is the new expectation.
  A fabricated Red would have had to reproduce the pre-flip literal exactly.
- The committed test at `:220-221` asserts BOTH `Some(&cpp_vocab)` and `None`
  yield `"  Cycle Groups Detected"` - a strictly stronger lock than the handoff's
  single assertion, and not a weakened one.
- Scope completeness: `crates/cli/src/**` carries zero `#[ignore]`, so the
  failure set is fully observed, and the label test is the only lib test added.

**Verdict: the Red is genuine, compiling, non-vacuous, and its evidence ordering
is auditable from history.**

---

## 5. THE HEADLINE ITEM: the two-process export lock - independent verdict

**VERDICT: GENUINE. ACCEPTED. This is a real two-process lock, not a
same-process double call dressed as one.**

Placed at `crates/cli/tests/payload_tail_bench.rs:501-621`
(`generate_cyclic_export_fixture`, `run_cold_cli_export`,
`export_bytes_are_identical_across_two_separate_processes`). Four properties
audited independently, each from source and from the reviewer-run capture rather
than from the journal.

### 5.1 Two genuinely separate OS processes - CONFIRMED, not a fake

`run_cold_cli_export` (`:552-567`) is the whole spawn path:

```rust
let cli_binary = env!("CARGO_BIN_EXE_ariadne");
let run = std::process::Command::new(cli_binary)
    .arg("ui").arg(repo).arg("--export").arg(export_path)
    .output()
    .unwrap_or_else(|spawn_failure| panic!("export process failed to start: {spawn_failure}"));
assert!(run.status.success(), "ariadne ui --export exited {}: {}", run.status, ...);
```

- `Command::output()` **spawns a child and waits for it**. The test calls
  `run_cold_cli_export` twice (`:577`, `:582`), so two children, two PIDs, each
  with its own `RandomState` seeds and its own ASLR layout. A same-process double
  call would be structurally impossible here.
- **`CARGO_BIN_EXE_ariadne` resolves to a real binary.** Cargo sets
  `CARGO_BIN_EXE_<name>` at compile time for integration-test targets of a
  package declaring that `[[bin]]`, and `crates/cli/Cargo.toml:7-9` declares
  `[[bin]] name = "ariadne"`. `env!` is compile-time, so the path cannot be
  empty or silently unset - a bad value is a compile error, not a skip.
- **The export is produced ONLY by the child.** `ui.rs:682-698`: the
  `export_opt` branch runs `build_ui_payload` + `export_standalone_html` and
  never reaches the `else` branch at `:699-703` that binds the TCP listener -
  so the invocation cannot hang on a port. The test never calls `run_ui` or
  `build_ui_payload` itself; grep over `crates/cli/tests/` finds `export` only at
  this test's own lines 502-618.
- **The gate is GREEN in the reviewer run**, capture line 106:
  `test export_bytes_are_identical_across_two_separate_processes ... ok`,
  completing in `0.15s` total for the bench target (line 108). Two full pipeline
  runs over a 2-header / 1-TU fixture in 0.15 s from the harness is consistent
  with the work being done in children, not in-process.

### 5.2 Non-vacuity - CONFIRMED, and the assertion would fail without cycle groups

The test does not merely compare bytes. Before the compare it requires (`:590-604`):

1. `.expect("export must embed the payload behind the data marker")` - the export
   must contain `window.__ARIADNE_DATA__ = ` at some offset `payload_start`.
   That string is exactly what `crates/ui/src/export.rs:50` emits
   (`format!("window.__ARIADNE_DATA__ = {};", json_data)`), injected by
   `export.rs:51` replacing `/*__ARIADNE_DATA__*/` from `assets/index.html:548`.
   Grep confirms the token `cycle_group_size` appears in **zero** files under
   `crates/ui/assets/**`, so the payload-region assertion below cannot be a
   template echo. (The three `window.__ARIADNE_DATA__` hits in `assets/api.js` are
   `&&`-guarded reads, never ` = `, so `.position()` cannot latch onto one.)
2. `assert!(first_bytes[payload_start..].windows(len).any(|w| w == b"cycle_group_size"), ...)`.

Independently verified that this key is **only reachable for a grouped node**:
`crates/ui/src/dto.rs:717-718` assigns `data["cycle_group_size"]` solely inside
`if let Some(group_size) = cycle_class.group_size(*file_id))`, and the doc comment
at `dto.rs:635-636` states the OMITS rule verbatim: "a node in no group gets
NEITHER key, never `null` and never `0`".

Therefore: **the assertion holds if and only if at least one node in the export
payload is a member of a cycle group.** The fixture (`include/a.h` includes
`b.h`, `include/b.h` includes `a.h`, pulled into the graph by `src/tu.cpp` plus
`compile_commands.json`) yields a nontrivial SCC of size 2, so IV-2 guarantees
group membership and the key must appear. If the fixture had produced no cycle
group - or if the T3-B rewiring had not reached `--export` at all - the assert
would fire with its self-explaining message. **It is a real gate, it is green,
and it was satisfiable only because unit B is the commit that put the group
currency into the export bytes.**

### 5.3 The `artifacts/` wipe is what makes the comparison symmetric - CONFIRMED

`fs::remove_dir_all(repo.join("artifacts"))` at `:581` sits between the two runs.
Verified that this is the COMPLETE persisted-state surface, so nothing survives to
make run 2 warm:

| Persisted state | Location | Source |
|---|---|---|
| all pipeline stage artifacts (repository, build context, include graph, index, reduced, callgraph, load) | `<root>/artifacts` | `crates/report/src/manager.rs:109` (`let artifact_root = root.as_ref().join("artifacts");`), reached from `pipeline/src/lib.rs:222, 225, 348, 433, 563, 606, 703` |
| TU shard cache | `<root>/artifacts/shards` | `crates/scheduler/src/tu_store.rs:33-35` (`pub fn shard_dir`) and `crates/scheduler/src/engine.rs:440` |
| optimize state | `<root>/artifacts/optimize` | `crates/cli/src/optimize.rs:445` |

Without the wipe, run 2 would read run 1's cache and the comparison would be
**cold-vs-warm**, which can differ for reasons having nothing to do with
cross-process determinism (reuse counts, stage fingerprints, cache-hit
branches). With the wipe, both processes recompute the whole pipeline from an
identical cold starting state in an identical directory. **Symmetry is real and
the mechanism is exactly the one the brief asks about.** Additionally the SAME
repo directory is used for both runs, so no absolute path can differ; only the
export output path differs, and that path does not enter the payload.

### 5.4 The comparison covers the whole payload - CONFIRMED

`fs::read` reads each export in full (`:584-585`). The test first asserts
`first_bytes.len() == second_bytes.len()` (`:606-611`), THEN
`first_bytes.iter().zip(second_bytes.iter()).position(|(a, b)| a != b)` must be
`None` (`:612-620`), and the message reports the first diverging byte offset.
`zip` stops at the shorter iterator, which is safe **only because the length
equality was asserted first** - the ordering is correct. So the comparison covers
**every byte of both files**, template and embedded CSS/JS included, not a slice
of the payload. The failure message even names `payload_start` so a divergence
can be located relative to the payload.

### 5.5 The residual two-sample limit - ACCEPTED

The implementer's declared limit is accurate: two samples can miss an order bug
that only manifests under a different hash seed. I still accept it, for reasons
that are stronger than "it was declared":

1. **The structural argument is the real backstop**, and it is independent of
   sample count: every map that could reach payload bytes is consumed only by
   `get`/`contains`, or sorted at emission (`dto.rs:681-683` sorts by the unique
   key `(path, file_id)`), or is a `Vec` in insertion order (`&graph.edges`,
   `representative_cycle`); `serde_json::Value` objects serialise in `BTreeMap`
   (alphabetical) order, which is insertion-independent. T3-A verdict sec 8(d)
   measured all of this.
2. **The detection floor argument favours two samples.** If a leak existed in a
   map iterated to build a payload ARRAY, two independent seeds would almost
   certainly catch it. If a leak existed in a map whose iteration order does not
   affect the emitted order (e.g. feeding a `count()`), then no number of runs
   would catch it, because the property is not order-sensitive. The scenario in
   which more samples would help is narrow; the scenario in which they would not
   help is broad.
3. **A byte-lock is a REGRESSION lock, not an exhaustive proof.** Its job is to
   fail loudly the day someone introduces a hash-iteration-order leak, and two
   independent processes give it a genuine chance on that day. It costs 0.15 s in
   the gate-1 run, so there is no cost argument for fewer or more.

**Accepted, with the limit declared in the test's own header comment** - which
the implementer did (`:501-511` names the property and why two calls in one
process cannot detect it).

### 5.6 The companion parity test: where it lives, who owns it - ADJUDICATED

The implementer leaves it "open for unit E or T4.1". **That is the one part of
the disposition I disagree with, and the disagreement is about ownership, not
about this unit.** I checked the actual expiry:

- **Unit E** (`tasks/m235-t3e-handoff.md:48-49, 117-120`) **DELETES** the
  path-based `build_include_elements` and renames `..._from_groups` back to
  `build_include_elements`. After unit E there is exactly one builder and the
  comparison the companion test is made of **cannot be written at all**.
- **Unit E also cannot host it**: its handoff pins "You add and remove no test"
  (`:161`) and its suite-count gate.
- **Units C and D cannot host it either**: unit C's touch-set is exactly two
  files under `tests/integration`, unit D's gate-10 scope is
  `crates/graph/src/{graph.rs, closure_equivalence_tests.rs, cycle_groups.rs}` +
  journal. A `crates/ui` test file is outside both.

**Disposition: the companion parity test has a HARD EXPIRY at unit E and must be
authored as a NEW small `crates/ui`-only unit sequenced BETWEEN unit D and unit
E - not appended to unit E, and not deferred to T4.1 (T4.1 is after E, by which
point the legacy builder is gone).** Its value is specifically to prove that
unit E's deletion of the legacy builder is behaviour-preserving on acyclic
graphs (`build_include_elements_from_groups(g, .., &[], ..)` byte-equal to
`build_include_elements(g, .., &[], ..)`), which is exactly the evidence unit E
needs and is forbidden to produce itself.

**Owner: planner** (to author the handoff); **trigger: before unit E is
dispatched.** Recorded as finding 3 and as a recommended ledger deferred row
annotated "expires at unit E".

---

## 6. `checkpoint_guard status` and the F12 fixture - fourth identical reproduction

Reviewer-measured at session start AND again after my own gate-2 (520-header,
full-pipeline) run:

| | before my gates | after my gate 2 |
|---|---|---|
| DIRTY FILES (`git diff HEAD --numstat`) | `(none)` | **`(none)`** |
| UNTRACKED | `(none)` | `tasks/m235-t3b-verdict.md` (my own artifact) |
| census `dirty=` for this worktree | `1` | `2` (the +1 is my untracked verdict file) |

`git status` reports exactly one tracked modification:
`modified: tests/fixtures/basic/artifacts/includes/report.json` - the forbidden
fixture - while the numstat dirty section is **EMPTY**, which is a content
comparison and therefore proves its bytes are equal to HEAD after EOL
normalisation.

**Adjudication: the known F12 mechanism, CONFIRMED, not a new problem.**
Signature reproduced for the fourth time (T2-A finding 9, T2-B finding 10,
T3-A sec 7, now T3-B): numstat clean, porcelain dirty, content byte-equal, path
in no commit. The path is unpinned by `.gitattributes` and this worktree has
`core.autocrlf=true`, so a run that rewrites the artifact invalidates the
stat-cache entry while `git diff` normalises line endings and finds no delta.
The implementer's distinction - "numstat clean -> the Read/Write restore was
never needed at any point" versus "porcelain-only `dirty=1`" - is literally
true of my own measurement, and correctly no fix was attempted (`git checkout --`
/ `git restore` are denied to every role; rewriting a byte-equal file would be a
content change with no delta to make).

**Standing closeout obligation, unchanged and still open:** pin the path in
`.gitattributes` or make the artifact writer emit LF. Must land before T4.1.
**No CRLF bytes entered the repository through this unit** - the path is in no
commit of `ea82422..91967c9`.

## 7. Frozen locks that must not have moved - reviewer-verified

| Lock | How verified | Result |
|---|---|---|
| `detect_cycles` still exists | grep `fn (detect_cycles\|dfs_cycle\|detect_cycle_groups)` under `crates/graph/src` -> `graph.rs:234 pub fn detect_cycles`, `graph.rs:260 fn dfs_cycle` | **HOLDS** |
| `dfs_cycle` still exists | same grep, `graph.rs:260` | **HOLDS** |
| still CALLED (not merely present) | grep `detect_cycles()` workspace-wide: `graph.rs:356` (`generate_report_with_cycle_policies`), `graph.rs:534` (unit test), `closure_equivalence_tests.rs:515`, `m20:154`, `m22:442` | **HOLDS** - four internal/external call sites intact |
| no `crates/graph` internal call site repointed | no `crates/graph/**` path in any of the six commits; `graph.rs:356` still reads `let detected_cycles = self.detect_cycles();` and `:369` still reads `cycle_count: detected_cycles.len()` | **HOLDS** |
| IV-9 `cycle_count` field name + `ui.rs:390` byte-unchanged | `grep cycle_count crates/cli/src/commands/ui.rs` -> single hit `:390 cycle_count: report.cycle_count,`; the line appears in NEITHER `a55091c` nor `9194c25`'s `ui.rs` hunks (both confined to `:307-322` and `:100-118`) | **HOLDS** |
| IV-11 `informational_cycle_count` field name + skip-when-zero serde shape | no `crates/report/**` and no `crates/graph/src/types.rs` path in any commit; `crates/graph/src/graph.rs` untouched | **HOLDS** |
| D27 `node_maps::build` 5-arg shape | `ui.rs:323` `let node_maps = node_maps::build(` (still at **column 0**, the pre-existing defect handoff sec2 forbids "fixing"), followed by the 5-argument `build_include_elements_from_groups(` at `:328` carrying `&include_graph, &closures, &cycle_groups, &node_maps.languages, &node_maps.unit_kinds` | **HOLDS** - and the column-0 line is a diff CONTEXT line in `a55091c`, i.e. byte-untouched, exactly as sec2 required |
| `crates/cli/src/commands/ui/node_maps.rs` byte-unchanged | absent from all six commits | **HOLDS** |
| existing three `includes.rs` tests unedited | `includes.rs:177 cpp_vocabulary_composes_todays_exact_labels`, `:197 default_labels_match_the_cpp_vocabulary_composition`, `:225 scala_vocabulary_composes_import_flavored_labels` - all present as diff CONTEXT lines in `9d831bf`/`52f8aa2`, none is a `+`/`-` line | **HOLDS** |
| the three pre-existing label helpers unchanged | `aggregate_kind_label`, `unresolved_label`, `top_closure_heading`, `unique_aggregates_label`, `graph_summary_title` - none appears on a `+`/`-` line in any commit | **HOLDS** |
| two-site comment sync (sec10 audit 3) | `ui.rs:103` reads `// generate_report+detect_cycle_groups 5.6ms`; `payload_tail_bench.rs:40` table row reads `generate_report+detect_cycle_groups (combined)   \|      5.6`; `:20` phase note and `:388` emitted column label both name `detect_cycle_groups`. **No stale `detect_cycles` and no surviving bare `4.7ms` next to the new figure** (the `4.7` value appears ONLY inside the provenance note that explains its retirement) | **HOLDS** |
| label shape: two leading spaces | `includes.rs:83` `"  Cycle Groups Detected"`; asserted at `:220` and `:221`; the print at `:121` is `println!("{}: {}", cycle_groups_label(vocabulary.as_ref()), report.cycle_count);` - same shape as its neighbours | **HOLDS** |
| no over-claim on the payload change | journal Checkpoint 1 and Checkpoint 5 both state the chord-classification change is unit A's proof and that this unit's test locks byte-STABILITY only | **HOLDS** |
| report evidence digest-shaped | journal quotes canonical lines, exit codes, capture paths; zero raw cargo/git output pasted; the gate-2 capture-overwrite caveat is disclosed and the measured value is quoted in journal text per sec8 | **HOLDS** |

---

## 8. Workspace-wide acceptance pass

### 8.1 Every workspace-wide gate: ran, could not run, and why

Each known-broken / known-refused item attempted **INDEPENDENTLY, exactly once**,
by this reviewer, directly. **None was retried, and none was routed through
`distill.py` or any other wrapper to evade a denial.**

| Gate | Command | Result |
|---|---|---|
| baseline | `python tools/testrun.py --baseline` | **BROKEN.** Verbatim: `usage error: missing value for --baseline`. Ledger M23.5 F9, reproduced. NOT treated as a gate failure, NOT worked around, NOT routed through `distill.py`. |
| full workspace (what `--baseline` wraps) | `python tools/testrun.py --distill --timeout 3600` | see 8.2 - **COMPLETED** |
| tool unit suites | `python -m unittest discover -s tools -p "*_tests.py"` | see 8.3 |
| encoding, all tracked | `python tools/encoding_guard.py --all` | **no output**, exit 0 |
| encoding, staged | `python tools/encoding_guard.py --staged` | **no output**, exit 0 |
| hygiene | `python tools/hygiene_check.py` | **zero R1/R2**; every printed line a legacy-file R3 warn |
| theme tokens | `python tools/theme_token_check.py --check-added --base ea82422` | **no output**, exit 0 = zero added hex literals |
| fixture state | `python tools/checkpoint_guard.py status` | run at session start AND after my gate 2; numstat DIRTY `(none)` both times (section 6) |
| UI runtime smoke | `python tools/ui_smoke.py` | **REFUSED.** Verbatim: `The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}` |
| load-perf regression | `python tools/bench_compare.py` | **REFUSED.** Verbatim: identical text. |

**On the two refusals.** A refusal ALWAYS reports the catch-all deny rule whatever
the real cause - here the real cause is that neither script is on the reviewer
bash allowlist (ledger M23.5 F10), not that bash is unavailable. Both were
attempted directly and once each; neither was retried through a wrapper. **The
denials did not cause any gate to be skipped**: hygiene, `encoding_guard --all`,
`encoding_guard --staged`, `theme_token_check`, `checkpoint_guard status` (twice),
gate 1, gate 2, the full-workspace run and the Python tool suite all ran.

**Two additional bash refusals occurred on reviewer-shaped commands, recorded for
honesty and for the benefit of the next reviewer.** (a) A compound command
(`git log ...; echo ...; git diff --name-only ...`, with `2>&1`) was refused with
the same catch-all text; I re-issued the queries as plain single commands and via
the dedicated git tools, and both succeeded. (b) `git status --porcelain <path>`
was refused with the same text; the dedicated status tool returned the same
information. In both cases the refusal was caused by my own command shape, not by
the gate being unavailable, and in both cases I did not narrow the acceptance set
as a result.

### 8.2 The canonical workspace line - COMPLETED

`python tools/testrun.py --distill --timeout 3600`, exit 0, capture
`target/testrun-capture.txt` (2065 lines). Verbatim:

```
1176 passed / 0 failed / 4 ignored / 88 suites
```

Compared by hand against the T3-A-measured line and the handoff's prediction:

| | passed | failed | ignored | suites |
|---|---|---|---|---|
| ledger baseline before this unit (T3-A measured at `ea82422`) | 1174 | 0 | 4 | 88 |
| **T3-B measured (`91967c9`)** | **1176** | **0** | **4** | **88** |
| handoff sec8 prediction | 1175 | 0 | 4 | 88 |
| **Delta vs 1174** | **+2** | **+0** | **+0** | **+0** |

**The measured delta is +2, the handoff predicted +1, and the difference is
exactly D1's test.** Attributed path by path, not merely consistent in total:

| New `#[test]` | Target | Counted in |
|---|---|---|
| `includes_summary_labels_report_cycle_groups_not_cycle_paths` (`crates/cli/src/commands/includes.rs:216`) | `unittests src/lib.rs` | +1 |
| `export_bytes_are_identical_across_two_separate_processes` (`crates/cli/tests/payload_tail_bench.rs:569`) | `tests\payload_tail_bench.rs` | +1 |
| | | **+2** |

`+0 suites` is correct: both tests live in pre-existing targets, so no new test
binary appears - which is the same tree-shape fact that makes D3's suite count
**5** rather than 3 while the *workspace* suite count stays **88**. There is
**zero unattributed residual**, and the handoff's own `+1` prediction is exactly
"the Red's test only", which is what a handoff written before the T3-A verdict
reassigned the export lock would predict.

### 8.3 Tool unit suites - COMPLETED

`python -m unittest discover -s tools -p "*_tests.py"`:

```
Ran 184 tests in 18.444s

OK
```

Identical count to the T3-A verdict's acceptance pass (`Ran 184 tests`, `OK`) -
correct, since this unit changes no `tools/**` file.

### 8.4 Fixture path, post-my-workspace-run state

`python tools/checkpoint_guard.py status` re-run AFTER my full workspace
invocation: HEAD `91967c9`, **DIRTY FILES (none)**, UNTRACKED only
`tasks/m235-t3b-verdict.md` (my own artifact), census `dirty=2` = 1 x F12
stat-cache entry + my untracked verdict file. **The forbidden fixture is still
absent from the numstat dirty section after the heaviest run of the unit and
after my own full-workspace run** (section 6).

### 8.5 Everything that was NOT run

Stated plainly, as required:

- `python tools/testrun.py --baseline` - **could not run**, broken
  (`usage error: missing value for --baseline`, F9). Attempted once, verbatim
  recorded, not worked around. The invocation it wraps WAS run instead (8.2).
- `python tools/ui_smoke.py` - **could not run**, REFUSED (F10). No runtime UI
  evidence exists for this milestone (finding 6).
- `python tools/bench_compare.py` - **could not run**, REFUSED (F10). **No
  verified perf-regression evidence exists for `detect_cycle_groups`**; the
  pinned `5.6` is a single local measurement, labelled as such at both sites
  (section 4).
- The raw Red capture - **not recoverable.** The Red revision was correctly
  never committed, and `target/` is ephemeral, so the original Red digest text
  survives only in the committed journal (section 4). Not a shortfall.
- No commit was made by this reviewer (write git is denied to this role).

---

## 11. VERDICT

# APPROVED

**0 blocking / 8 non-blocking.**

Range `ea82422..91967c9`, six commits, every one inside the declared touch-set
plus the orchestrator-declared records; the forbidden fixture in none of them.
All frozen locks hold, including the four the milestone's sequencing depends on
(`detect_cycles`/`dfs_cycle` still present AND still called, no `crates/graph`
call site repointed, IV-9 `cycle_count` byte-unchanged, IV-11 serde shape
untouched, D27 5-arg shape intact, Scala paths untouched).

The two-process export lock - the item this unit was specifically dispatched to
discharge - is **genuine on all four audited properties** (section 5): real
separate OS processes via `CARGO_BIN_EXE_ariadne`, a non-vacuity assertion that
can only pass if the payload really carries grouped nodes, an `artifacts/` wipe
that verifiably restores symmetric cold starts, and a whole-file byte comparison.
Its two-sample residual is accepted and correctly declared (5.5).

The bench re-pin is honest on every axis the brief asks about: the ignored bench
**genuinely ran** in the release profile over the full 520-header fixture, the
pre-existing `5.8`-vs-`4.7` baseline split was **recorded rather than silently
reconciled**, and the UNVERIFIED-against-`bench_compare.py` label appears in the
journal **and** in both file comments. Reviewer independently reproduced the
pinned figure (5.5 vs 5.6).

Scoped canonical lines, reviewer-measured, verbatim:
`88 passed / 0 failed / 1 ignored / 5 suites` (gate 1),
`1 passed / 0 failed / 0 ignored / 1 suites` (gate 2),
`1176 passed / 0 failed / 4 ignored / 88 suites` (workspace).
Tool suites: `Ran 184 tests in 18.444s` / `OK`.

---

## 9. The six declared deviations, adjudicated independently

Adjudicated on evidence - the spec and the code, not the implementer's
justification.

| # | Deviation | Disposition | Attributed to |
|---|---|---|---|
| D1 | second new `#[test]` beyond sec6's "exactly one"; gate 1 `88 ... / 5 suites` vs the table's `87 ... / 3` | **ACCEPTED** | authority = T3-A verdict + plan, not the handoff |
| D2 | gate 2 run with a DOUBLED `--` | **ACCEPTED** - and the CONTRACT's gate-2 line is the defect | planner / handoff-spec |
| D3 | gate 1 suite count 5, not 3 | **ACCEPTED** - planner arithmetic proven wrong | planner |
| D4 | Red digest line differs from sec4's predicted line | **ACCEPTED** - sec4 is self-contradictory | planner |
| D5 | `encoding_guard.py --staged` runs post-commit | **ACCEPTED** (T2-A/T2-B D-E, T3-A D5 precedent) | configuration (`commit_scoped.py` has no stage-only mode) |
| D6 | sec8 gate 7's single commit split into five scoped commits | **ACCEPTED** - and REQUIRED by sec10 audit 1 | n/a (implementer judgement, contract-conformant) |

**D1 - ACCEPTED.** The second test is not an implementer invention; it is a plan
requirement that the handoff predates. Plan:128 lists "byte-stable export test
across two processes with cycle fixtures present" as a **T3.2 Red**, and
plan:147 lists "export byte-stable" as **acceptance criterion 5**. The T3-A
verdict sec 8 / finding 1 (APPROVED, committed as `tasks/m235-t3a-verdict.md` in
`9f30c9a`, and orchestrator-declared into this unit) reassigned the lock to unit
B. Feasibility is confirmed from the tree, not accepted on assertion:
`crates/cli/tests/payload_tail_bench.rs` is an integration-test target of a
package declaring `[[bin]] name = "ariadne"` (`Cargo.toml:7-9`), so
`env!("CARGO_BIN_EXE_ariadne")` resolves; a `src/**` unit test has no
`CARGO_BIN_EXE_*` and cannot spawn the binary, and a same-process double call is
explicitly forbidden. Attribution is exact: reviewer gate-1 capture line 91 is
`87 passed` for the lib suite and line 108 is `1 passed; 1 ignored` for the bench
target, so `88 = 87 + 1` with **zero unattributed residual**. *Consequence for
the corpus: handoff sec6's "exactly one new `#[test]`" is now stale and must not
be inherited by a successor.*

**D2 - ACCEPTED; the mechanism verified in the harness source, and the CONTRACT is
what is wrong.** `tools/testrun.py:188-190`:

```python
if arg == "--":
    cargo_args.extend(argv[index:])
    break
```

The first `--` is consumed by `testrun.py` itself and the remainder is extended
into `cargo_args` **without re-inserting cargo's own separator**. So sec8's
literal gate-2 line hands cargo `--ignored` as a *cargo* flag, which is why the
verbatim first attempt fails with `error: unexpected argument '--ignored' found`
(tip: `to pass '--ignored' as a value, use '-- --ignored'`). The doubled `--`
leaves exactly one separator for cargo, and the resulting cargo argv **equals the
bench's own run instruction at `payload_tail_bench.rs:62-63` verbatim** - so it
is an invocation-shape correction at the entry point, not a substituted or
weakened gate. I re-ran the corrected form myself (section 4) and it reproduces
the handoff's pinned line. *This is a `tools/testrun.py` / handoff-spec defect;
fixing the tool is outside this unit's touch-set and is not demanded of the
implementer.*

**D3 - ACCEPTED; the planner's arithmetic is proven wrong from the tree.**
`crates/cli/Cargo.toml:7-13` declares two `[[bin]]` targets (`ariadne` at
`src/main.rs`, `tool` at `src/bin/tool.rs`); cargo runs a unittests binary for
each even at zero tests, which the reviewer capture shows at lines 96 and 101.
The five suite headers are named at capture lines 116-120. `3 + 2 = 5`. The
handoff counted only lib + integration + doc-tests. The implementer's `5` is
correct and its attribution ("pre-existing tree shape; nothing in this unit adds
a test binary") is literally true. **The same slip is baked into the ledger's
`-p ariadne-cli = 86/0/1/3` row and should be corrected at closeout** - that
ledger number is wrong and is inherited by every future unit scoped to that crate.

**D4 - ACCEPTED; the CONTRACT is internally impossible and the implementer
reported the contradiction instead of papering over it.** Sec4 says: "the
expected line at that point is `86 passed / 0 failed / 1 ignored / 3 suites`
(gate 1 below with one new lib test, **one failing**)". A single canonical line
cannot report `0 failed` and simultaneously describe a state containing a failing
test. The contract is self-contradictory, and its `3 suites` additionally
inherits D3's error. The measured `86 passed / 1 failed / 0 ignored / 1 suites`
is the coherent line for that state. **This finding is attributed to the
planner, not the implementer** - and reporting it rather than quietly producing a
line that matched the contract is exactly the behaviour the gate should reward.
Independent corroboration of the Red's location is in section 4.

**D5 - ACCEPTED.** `commit_scoped.py` has no stage-only mode, so `--staged`
cannot interpose between staging and committing in one command. The
implementer ran the STRONGER `--all` form before every commit and the literal
`--staged` form afterwards. Reviewer re-ran both independently: `--all` silent,
`--staged` silent, exit 0 each. Substitution cost is zero. Same precedent as
T2-A gate 5, T2-B D-E and T3-A D5.

**D6 - ACCEPTED, and it was mandatory rather than discretionary.** Sec10 audit 1
is explicit: "A commit that shows the helper and the new string together has no
Red." `9d831bf` shows the helper carrying the **OLD** bytes with no test;
`52f8aa2` shows the flip plus the test. A single gate-7 commit would have
**failed the review contract**, so the split was required. Reviewer-verified
supporting properties: all six per-commit path lists are inside the touch-set
(section 2), so every explicit `--scope` was honest; sec0 step 0.4's requirement
that every commit carry an explicit scope is satisfied; and the gate-7 message is
preserved verbatim on `a55091c`, which is the commit that actually carries the
B1-B4 wiring.

### Undeclared deviations found by this reviewer

**None in the implementation.** Every divergence from upstream text surfaced
during the audit traces to a handoff or plan sentence and is recorded as a
finding below (1, 2, 3, 4). No undisclosed implementer choice was found: the
report-shaping divergences, the doubled `--`, the suite count, the Red line and
the stale bench row label were all declared or are plan/handoff artefacts.

---

## 10. Findings

**Verdict: APPROVED.** 0 blocking / 8 non-blocking.

1. **NON-BLOCKING, planner.** Handoff sec4's predicted Red line
   (`86 passed / 0 failed / 1 ignored / 3 suites`, "with one new lib test, one
   failing") is **internally impossible** - it cannot report `0 failed` and
   describe a failing test - and its `3 suites` additionally inherits finding 2's
   arithmetic error. **Action:** correct sec4 before the text is inherited; the
   measured line is `86 passed / 1 failed / 0 ignored / 1 suites`. Credit to the
   implementer for reporting the contradiction rather than producing a
   contract-matching line.
2. **NON-BLOCKING, planner.** Handoff sec8's suite arithmetic (`3 suites`) and
   the ledger's `-p ariadne-cli = 86/0/1/3` row both omit the two `[[bin]]`
   targets' unittests binaries. Reviewer-measured truth is **5 suites**
   (`crates/cli/Cargo.toml:7-13` + capture lines 116-120). **Action:** correct
   the ledger row and any future handoff scoped to `ariadne-cli`.
3. **NON-BLOCKING, planner (ownership with a hard expiry).** The companion
   parity test (`from_groups` vs legacy builder byte-equality on an acyclic
   graph) is left "open, unassigned" by the implementer, who suggests unit E or
   T4.1. **Both are too late or forbidden:** unit E DELETES the legacy builder
   (`tasks/m235-t3e-handoff.md:48-49, 117-120`), after which the comparison
   cannot be written at all, and unit E also pins "You add and remove no test"
   (`:161`). Units C and D cannot host a `crates/ui` test (their touch-sets are
   `tests/integration` and `crates/graph` respectively). **Action:** author a
   small `crates/ui`-only unit sequenced BETWEEN unit D and unit E, and record a
   ledger deferred row annotated "expires at unit E". Its value is specifically
   to evidence that unit E's deletion is behaviour-preserving on acyclic graphs.
4. **NON-BLOCKING, planner.** Handoff sec3.4's site list is incomplete: it names
   `payload_tail_bench.rs:361` (the combined cycle row label) but not `:389`,
   whose emitted row label still reads `"build_include_elements"` while B4 now
   calls `build_include_elements_from_groups` - the identical staleness sec3.4
   was written to eliminate, one row below. The implementer correctly reported it
   as an out-of-scope observation rather than editing an unnamed site.
   **Action:** rename the emitted row label in unit E alongside the entry-point
   rename, so the two never disagree.
5. **NON-BLOCKING, planner / architect (transient, self-closing).** At this HEAD
   the CLI prints `  Cycle Groups Detected: N` where **`N` is still a
   DFS-path count, not a group count**: `crates/graph/src/graph.rs:356` still
   reads `let detected_cycles = self.detect_cycles();` and `:369` still sets
   `cycle_count: detected_cycles.len()`. The label therefore mis-describes its own
   number for the duration of Phase 3. This is inherent to the planner's 5-unit
   split - the implementer was forbidden by sec1/IV-9 from touching report code
   and pinned `cycle_count: report.cycle_count` byte-identical, so this is
   **contract compliance, not a defect** - and it self-closes at unit D, whose
   handoff owns the port (`tasks/m235-t3d-handoff.md:196-197, 218-219`,
   `cycle_count_counts_entangled_groups_not_reported_paths`). **Action:** do not
   read the Phase 3 CLI label as group-accurate until unit D lands, and do not
   treat the label as evidence that IV-9's group-count migration has happened.
6. **NON-BLOCKING, configuration (standing, F10).** `python tools/ui_smoke.py`
   and `python tools/bench_compare.py` each attempted once, directly, each
   refused with identical verbatim catch-all text (section 8.1). Neither was
   routed through `distill.py`. **Impact is now materially higher than for any
   prior M23.5 unit:** T3-A verdict finding 9 escalated exactly this - this unit
   is the commit that puts the group currency into the bytes the UI server
   serves, so there is still **no runtime evidence that the UI cold-boots,
   routes, and serves the new payload**, and the `Cache-Control: no-store` route
   contract is unverified for it. The implementer independently measured the same
   refusal and escalated it in journal Checkpoint 3. **Action:** close F10
   before T4.1; it is an operator/allowlist action, not implementer work.
7. **NON-BLOCKING, configuration (standing, F9).**
   `python tools/testrun.py --baseline` -> `usage error: missing value for
   --baseline`, reproduced verbatim by this reviewer. Attempted once, recorded
   once, NOT treated as a gate failure, NOT worked around. The full-workspace
   invocation that `--baseline` wraps was run instead and compared by hand
   (section 8.2).
8. **NON-BLOCKING, implementer (residual honesty, correctly declared).** The
   two-process lock samples two runs. Section 5.5 sets out why that is an
   acceptable regression lock rather than a proof, and the limit is declared in
   the test's own header. Recorded so the next reviewer does not mistake it for
   an undiscovered weakness. No action beyond keeping finding 3's parity test
   alive until unit E.

### Locks, verified (summary)

All of section 7's lock rows **HOLD**, including the four that matter most for
the milestone's sequencing: `detect_cycles` and `dfs_cycle` both still exist AND
are still called (`graph.rs:234`, `:260`, called from `graph.rs:356`, `:534` and
`closure_equivalence_tests.rs:515`); no `crates/graph` internal call site was
repointed; IV-9's `cycle_count` field name and `ui.rs:390` are byte-unchanged;
IV-11's `informational_cycle_count` name and skip-when-zero serde shape are
untouched (`crates/report/**` and `crates/graph/src/types.rs` absent from every
commit); the D27 5-arg `node_maps::build` shape survives with its pre-existing
column-0 indent byte-identical; IV-12's Scala paths are untouched with no
plugin-side change; and the forbidden fixture is in no commit.