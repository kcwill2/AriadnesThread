# M23.5 T3-E1 Journal - Retire the path-based include-element DTO (crate:ui)

Contract: `tasks/m235-t3e1-handoff.md`. Dispatch base: HEAD `b0a91d2`, branch
`tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc` (confirmed by
`checkpoint_guard.py status` before any edit).
Outcome: COMPLETE. Two scoped commits -- commit 1 `ab59bb4` (planner records,
out of band), commit 2 = this journal plus the seven code paths (its own SHA
cannot appear here; read `git log -1`).
No production behaviour changed: this unit deletes an entry point that unit A
proved and unit B put into production use, repoints callers at the survivor,
and converts one test into a frozen-byte lock. Per sec4, stated plainly: **no
production-behaviour Red exists for E1**; the executed Reds are R1 (stale
reference) and R2 (mutated expectation) below.

## Checkpoint 0 - preflight (sec0), all three steps run

- 0.1 `checkpoint_guard.py status`: HEAD `b0a91d2`, branch `tarjan-s-scc`.
  DIRTY (content numstat): `tasks/ledger_archive.md` 13/1,
  `tasks/milestone_ledger.md` 107/6, `tasks/m235-t3e-handoff.md` 737/170.
  UNTRACKED: `tasks/m235-t3d2-verdict.md`, `tasks/m235-t3e-journal.md`,
  `tasks/m235-t3e1-handoff.md`, `tasks/m235-t3e2-handoff.md`.
  All eight are planner records; none is mine to edit (sec8.2 commit 1).
- 0.2 F12 fixture `tests/fixtures/basic/artifacts/includes/report.json`:
  `git diff --numstat HEAD -- <path>` (via `distill.py`) printed **zero numstat
  rows** -- only the autocrlf stat warning (`LF will be replaced by CRLF`).
  Content byte-equal, so per "if and only if it prints rows" NOTHING was
  restored and nothing was written. The path also stayed out of the dirty set
  after both full test runs (verified by the post-gate `checkpoint_guard.py
  status`), and it appears in neither commit.
- 0.3 Caller census, all three greps over `crates/`, `tests/`, `tools/`, run at
  `b0a91d2` BEFORE any edit. **Matched the contract exactly, no STOP needed:**
  - `build_include_elements(_from_groups)?\s*\(`: 22 textual hits under
    `crates/` = **20 call sites + 2 definitions**, zero elsewhere.
    Legacy 9, all in `crates/ui/tests/ui_test/`: `common.rs:157`,
    `d27_node_provenance.rs:141` and `:152`, `dto_builders.rs:15`,
    `export_order.rs:91`, `:108`, `:310`, `:381`, `graph_api.rs:136`.
    Grouped 11: 9 in `ui_test/` (`d27_node_provenance.rs:174`,
    `dto_builders.rs:470`/`:501`/`:588`, `export_order.rs:167`/`:317`/`:382`,
    `graph_api.rs:752`/`:759`) + 2 in `crates/cli`
    (`src/commands/ui.rs:328`, `tests/payload_tail_bench.rs:325`).
    Definitions: `dto.rs:608` (legacy, `cycles: &[Vec<FileId>]`) and `dto.rs:637`
    (grouped, `cycle_groups: &[CycleGroup]`).
  - `from_paths`: exactly 3 hits -- `cycle_groups_view.rs:4` (module doc),
    `:32` (definition), `dto.rs:618` (its only caller, inside the deleted fn).
  - `tests/` and `tools/`: zero hits for either name or for `from_paths`.
  - Non-paren textual mentions (outside the census pattern, all left alone):
    `export_order.rs:336-337` (P1 header, replaced by 5.2), and three `crates/cli`
    prose sites (`ui.rs:103`, `payload_tail_bench.rs:41`, `:389`).
- 0.4 Reverse dependencies, enumerated from every `Cargo.toml` in the workspace
  (grep `ariadne-ui`): exactly two consumers of this API --
  **`ariadne-cli`** (`crates/cli/Cargo.toml:34`, live textual references in lib
  and bench targets) and **`ariadne-integration-tests`**
  (`tests/integration/Cargo.toml:25`, declared path dependency, zero textual
  references: the `tests/` grep above returned none). Gate 3 covers the first;
  per sec7 the second is E2's gate because only a *rename* can break it, not a
  deletion of a name it never mentions.
- Entry line counts re-derived: `dto.rs` 901, `cycle_groups_view.rs` 149,
  `export_order.rs` 446. Baseline gate 1 (capture run): `186 / 0 / 0 / 1`.

## Checkpoint 1 - base state of the reverse-dependency gates (sec7)

| Gate | Command | Verbatim |
|---|---|---|
| 3 (base) | `python tools/distill.py -- cargo check -p ariadne-cli --all-targets` | `ok (exit 0, 0 warnings emitted)` (capture `target/distill-capture.txt`) |
| 4 (base) | `python tools/distill.py -- cargo build -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` (capture `target/distill-capture.txt`) |

Gate 1 capture-and-green run, at the untouched base with only the temporary
capture line added (see the Capture section):
`python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test`
-> `186 passed / 0 failed / 0 ignored / 1 suites` (capture
`target/testrun-capture.txt`, 194 lines). Green.

## Capture (4.4) - how the frozen expectation was obtained

1. One temporary line in P1 immediately after `export_order.rs:432`:
   `std::fs::write(std::env::temp_dir().join("t3e1_p1_payload.json"), &grouped_json).expect("capture payload");`
   Run once (the capture-and-green gate 1 above, still 186/0/0/1); it wrote only
   into the OS temp directory, never the worktree.
2. Read the file with the Kilo Read tool: 1605 bytes, single line, printable
   ASCII, no trailing newline -- the exact `serde_json::to_string` bytes of the
   **group-based** side.
3. The temporary line was deleted before any commit; `export_order.rs` numstat
   is `80/50` and contains no capture code (`git diff --numstat` evidence below).
4. The 1605 bytes were mechanically escaped (`"` to `\"`) and split into 25
   `concat!` chunks of at most 85 escaped characters, so every committed chunk
   line is at most 92 characters (sec6's bound is 96), one chunk per line. The escaping/splitting helper lived in the OS
   temp dir and printed to stdout only; it is not in the worktree and not in any
   commit (disclosed as Dev-6; a raw `python <script>` call was refused, see
   Refusals, and re-attempted through `distill.py`).
- **Provenance, as sec4.4 requires:** the committed byte lock at `42bb44e`
  asserted `grouped_json == legacy_json` on this exact input, so these bytes are
  the agreed output of BOTH builders and E1's deletion is behaviour-preserving
  on this input. The capture alone is not claimed as parity proof, and the new
  test header says so.
- **First-run behaviour:** the freshly captured expectation passed its first
  executed run (the R2 run below is a deliberately mutated expectation, not a
  transcription failure). Nothing was ever weakened or re-pasted to make a
  failure go away. Independent corroboration of transcription fidelity: the R2
  run's `left:` value (the emitted payload) is byte-identical to the captured
  file apart from the one mutated character.

## Green (3.1-3.4) and the two Reds

### R1 - executed, deliberate stale reference (completeness proof)

With 3.1-3.4 and the P1 conversion landed but `export_order.rs:91` (the
contract-nominated bare call) still naming the deleted function, gate 1
compiled to a FAILURE. The distiller's digest line read
"### cargo test DRIFT (exit 2) [capture: target/testrun-capture.txt, 9 lines]"
-- exit 2 is `testrun.py`'s "no test result line matched the pinned parse spec"
branch, and its cause here is a compile failure, not parse drift (Dev-5).
Verbatim from the capture, one error only:

```
error[E0425]: cannot find function `build_include_elements` in this scope
  --> crates\ui\tests\ui_test\export_order.rs:91:20
   |
91 |     let elements = build_include_elements(
   |                    ^^^^^^^^^^^^^^^^^^^^^^ not found in this scope
```

`error: could not compile `ariadne-ui` (test "ui_test") due to 1 previous error`
**One** error after the deletion is the completeness evidence: the other eight
legacy sites were repointed, no ninth missed site exists in `crates/ui`, and the
deleted definition left no other in-crate reference. R1 was never committed.

### Green implementation, as landed (numstat `git diff --numstat HEAD -- crates/ui`)

| Path | +/- | What changed |
|---|---|---|
| `crates/ui/src/dto.rs` | 4/23 | legacy `pub fn build_include_elements` + its legacy doc paragraph deleted; the D27 paragraph moved onto the survivor; `dto.rs:625-626` ("the group-based sibling of `build_include_elements`") left ALONE -- still true in E1, E2's obligation |
| `crates/ui/src/cycle_groups_view.rs` | 6/36 | `IncludeCycleClass::from_paths` + its doc + the now-redundant blank deleted (sole caller was the deleted fn); module doc restated for the single surviving currency; `from_groups`, the five query methods and the three free functions untouched |
| `crates/ui/tests/ui_test/export_order.rs` | 80/50 | sites `:91`, `:108`, `:310` repointed (callee name only); import `:15` de-duplicated; P1 converted per 5.2 + `ACYCLIC_EMPTY_CURRENCY_PAYLOAD` const |
| `crates/ui/tests/ui_test/dto_builders.rs` | 5/5 | site `:15` repointed; import `:4` de-duplicated; local `cycles` -> `cycle_groups` (the sec3.3-permitted rename) |
| `crates/ui/tests/ui_test/graph_api.rs` | 3/3 | site `:136` repointed; import `:4` de-duplicated |
| `crates/ui/tests/ui_test/d27_node_provenance.rs` | 3/3 | sites `:141`, `:152` repointed; import `:9` de-duplicated |
| `crates/ui/tests/ui_test/common.rs` | 3/2 | site `:157` repointed; import `:3` names the survivor (Dev-1) |

Zero type annotations changed: all nine sites pass an empty currency, so the
element type is inferred from the parameter (sec3.3).
Line counts before -> after: `dto.rs` **901 -> 882**,
`cycle_groups_view.rs` **149 -> 119**, `export_order.rs` **446 -> 476**
(converted test 86 lines, const at module scope), `common.rs` 226 -> 227.
All files far below the 1000-line ceiling; no file extracted.

### P1 conversion attestation (5.1/5.2)

- Exactly ONE builder invocation in the converted body:
  `build_include_elements_from_groups(&graph, &closures, &[], &file_languages,
  &file_unit_kinds)` (step 3, single call).
- The structural builder-to-builder `assert_eq!` (old step 6) is **DELETED, not
  ported**; with one invocation it would read `x == x`. No `contains`-style
  loosening was added, and nothing was added to "strengthen" the test.
- The four non-vacuity guards stay **ahead** of the byte comparison (step 4
  before step 5), reading the single `elements` vector, messages preserved.
- Renamed once, as part of the single sanctioned change:
  `acyclic_include_elements_are_byte_identical_under_both_cycle_currencies` ->
  `acyclic_include_elements_match_the_frozen_acyclic_payload`. No test added,
  removed, or otherwise altered; the module doc at `:1-8` was not amended.
- **P2 untouched**: `the_parity_comparison_detects_a_cycle_classification_difference`
  keeps its `assert_ne!` and its `&[]` vs `&groups` shape; only its
  callee name changed, so it stays a real comparison.
- No new file entered `tests/golden/**`, `tests/fixtures/**` or
  `crates/ui/tests/ui_test/**`; no `UPDATE_GOLDENS` run.

### R2 - executed, one-character mutation of the frozen expectation

Target substring `translation-unit`, counted first: **exactly one** occurrence
inside the const (`export_order.rs:348`; the other hit at `:417` is the input
map, not the expectation). Mutation: `translation-unit` -> `transmation-unit`,
same line count, same line lengths. Gate 1:

`185 passed / 1 failed / 0 ignored / 1 suites`
(capture `target/testrun-capture.txt`, 209 lines) -- the pinned line, met.

It failed **at the byte lock**, not at a guard and not at compile:

```
thread 'export_order::acyclic_include_elements_match_the_frozen_acyclic_payload' (21356) panicked at crates\ui\tests\ui_test\export_order.rs:471:5:
assertion `left == right` failed: frozen payload: on acyclic input with empty cycle currency the surviving builder must emit the bytes recorded at b0a91d2
```

`left:` is the ~1.6 KB emitted payload and `right:` the ~1.6 KB expectation; both
are quoted by capture path plus identifying key rather than pasted (same
construction as D2's Dev-6 -- pasting them would breach the 2000-character
source-line rule). The one visible difference is
`"unit_kind":"translation-unit"` (left) versus `"transmation-unit"` (right); all
four guards passed under the mutation, as they must, since they are
value-insensitive to that key's spelling.
Revert: the exact inverse one-character Edit. Byte-for-byte proof, content hash
(`certutil` through `distill.py`), pre-mutation and post-revert identical:

```
34ef135686732b59f3b9f9d3daeb061fb3d63193a7bf69a082c4a1fac7e2ae2f
```

and the following final-green gate 1 is the fix-verification and the final
measurement in one run (sec4: no separate R2 green run).

## Checkpoint 2 - final gates (sec7), all DIRECT form

| # | Command | Verbatim |
|---|---|---|
| 1 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test` | `186 passed / 0 failed / 0 ignored / 1 suites` (capture `target/testrun-capture.txt`, 194 lines) |
| 2 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui` | `186 passed / 0 failed / 0 ignored / 3 suites` (capture `target/testrun-capture.txt`, 206 lines) |
| 3 | `python tools/distill.py -- cargo check -p ariadne-cli --all-targets` | `ok (exit 0, 0 warnings emitted)` (capture `target/distill-capture.txt`) |
| 4 | `python tools/distill.py -- cargo build -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` (capture `target/distill-capture.txt`) |
| 5 | `python tools/hygiene_check.py` | zero R1 lines, zero R2 lines; 120 R3 warn lines, the same 120 the T3-D2 reviewer measured at this base -- **no new R3 line in any file I touched** (the `dto.rs:330/:418/:445` and `graph_api.rs` `res` warns are pre-existing lines outside my edits) |
| 6 | `python tools/encoding_guard.py --all` | no output, twice (immediately before each commit) |
| 6 | `python tools/encoding_guard.py --staged` | no output, once after commit 2 (ordering substitution, Dev-4) |
| 7 | `python tools/commit_scoped.py --scope ... -m ...` | two commits, each with explicit `--scope` per path |

Gate 1's four contracted invocations: capture-and-green, R1 (compile FAILURE),
R2 (mutation, 1 failure), final green. Gate 3 x2 (base, final), gate 4 x2 (base,
final) -- all four `ok (exit 0)`, so the reverse dependency `ariadne-cli` was
type-checked at both ends of the deletion and never broke. Gate 4's zero warnings
is the proof that `from_paths` really is gone (an un-deleted `pub(crate) fn` in
that module is unreachable code and would emit `dead_code`) and that no dangling
import survived the five import-list edits. `#[allow(dead_code)]` was not used
and no warning was suppressed.
Count identity, re-derived from my own capture: 186 - 0 + 0 = 186 (P1 was
rewritten and renamed; no test added or removed).
**Workspace line: not measured, deliberately** (sec7). Last measured
`1178/0/4/88` at `aebfa9b`; D2's `+2` is a prediction; E1 adds 0.
Not run, per sec7 (not worked around, not routed through `distill.py`):
`testrun.py --baseline` (F9), `unittest discover`, `ui_smoke.py`,
`bench_compare.py` (F10). `theme_token_check.py` not run: the touch-set contains
no `crates/ui/assets/**` path, so zero new hex literals is structural.

## Zero warnings with `from_paths` orphaned - how

Deleting `build_include_elements` removed the only caller of
`IncludeCycleClass::from_paths` (`dto.rs:618`). Because `cycle_groups_view` is
`pub(crate)` (`lib.rs:1`), the function became unreachable code and `cargo build
-p ariadne-ui` would emit `dead_code`. The requirement is met by **deletion**:
`from_paths`, its six-line doc comment and the blank line that separated it from
`from_groups` are gone (gate 4: 0 warnings emitted, twice), and the module doc no
longer names a deleted function. Nothing else in the file moved.

## Deviations

- **Dev-1 (documentation defect in the contract; `common.rs:3` DOES need an
  edit).** Sec2 line 125-126 and sec3.4's last row say `common.rs:3` needs no
  edit because it "imports only `build_include_elements`, which after E1 denotes
  the survivor". The tree disagrees: after E1 the sole surviving name is still
  `build_include_elements_from_groups` (the rename is E2), so the import named a
  symbol E1 deletes and `ui_test` could not compile. Correcting source:
  `crates/ui/tests/ui_test/common.rs:3` + `:157` and the R1 compile error, which
  proves the name resolution. The import now names the survivor, rewrapped to
  4 lines to stay inside the repo's 100-column style (226 -> 227 lines). The
  claim becomes true only at E2, where `common.rs` is indeed frozen.
- **Dev-2 (documentation defect; sec3.4's "After E1" column is inverted).** The
  table says to "drop the grouped name" from the four two-name lists. Dropping it
  would delete the survivor and break all eleven grouped call sites. The tree
  wins: the **legacy** name was dropped in each of `export_order.rs:15`,
  `dto_builders.rs:4`, `graph_api.rs:4`, `d27_node_provenance.rs:9`; each list
  now names one builder. The sentence as written is true for E2, not E1.
- **Dev-3 (bounded doc scope, restated beyond "keep :5-9").** Sec3.2 says keep
  `cycle_groups_view.rs:1-3` and `:5-9` and restate only `:4`. `:3` and `:4` are
  one sentence ("Two cycle currencies ... the legacy ... and the milestone's"),
  and `:6-7`/`:9` assert "Both entry points ... the two entry points", which the
  3.1 deletion makes false -- exactly the `build_include_elements_legacy_paths`
  defect class. The module doc is now 8 lines: `:1-2` byte-identical, the
  `from_groups` + plan citation preserved verbatim, the byte-shape clause
  preserved, and the currency/count clauses made true. Kept inside the
  `:1-9` region sec3.2 designates; disclosed rather than silently absorbed.
- **Dev-4 (gate 6 ordering, pre-authorised by sec7).** `commit_scoped.py` stages
  and commits atomically and raw git staging is denied, so `--staged` cannot
  interpose. Ran the strictly stronger `--all` immediately before each commit
  (both clean) and the literal `--staged` once after commit 2. Precedent cited in
  sec7: T2-A gate 5, T2-B Dev-E, T3-A D5, T3-B D5, T3-C Dev-3, T3-D Dev-4,
  T3-D2 Dev-4.
- **Dev-5 (digest-shape report of R1).** Gate 1's R1 invocation returns
  `DRIFT (exit 2)` / "no test result line matched the pinned parse spec" because
  a compile failure emits no test result line. Recorded verbatim so nobody reads
  exit 2 as a harness problem; the 9-line capture holds the compile error. Not a
  gate to drop and not parse drift.
- **Dev-6 (helper used to escape/split the capture).** The 25 `concat!` chunks
  were produced by a temp-dir helper that read only the captured file and printed
  the escaped chunks; content was not typed by hand and the helper is not in the
  worktree or any commit. Byte-correctness is proven by the lock itself: the
  reverted, unmutated const makes gate 1 green (186/0/0/1), and the mutated
  variant makes it fail at `:471` with the emitted payload differing in exactly
  the one mutated character.
- **Dev-7 (line-count landing).** Sec6 predicted `dto.rs` ~886 and
  `cycle_groups_view.rs` ~123; landed 882 and 119. The difference is doc
  separators and the blank line sec3.2's "-19/-28" spans did not itemise. Both
  are under every ceiling; `dto.rs` is nowhere near the 940 stop line, and no
  module was extracted.
- **Sanctioned, not a deviation:** the P1 rename (part of the single conversion),
  the `dto_builders.rs:13` local rename, the two doc obligations (D27 paragraph
  moved, module doc restated), and commit 1's out-of-band `tasks/**` landing
  (sec8.2, precedent T3-D2 Dev-5).

## Report-only observations (not acted on)

- P2's doc comment (`export_order.rs`, above
  `the_parity_comparison_detects_a_cycle_classification_difference`) still names
  the former P1 name and calls it "the parity lock below". Editing it is outside
  sec5.1's "byte-identical except the callee name" allowance, so it is left for
  the reviewer/E2 to disposition -- it is the same stale-identifier class as
  ledger F13.
- `dto.rs:600-601` ("the group-based sibling of `build_include_elements`") is now
  a sentence about a deleted function that is *still defensible* only because
  E2 will make the plain name denote this survivor; sec3.1 forbids fixing it in
  E1. E2 carries it.
- `d27_node_provenance.rs:141` was already ~107 columns pre-existing; the callee
  rename lengthens it to ~121 (far under the 2000-character R1 rule, above
  rustfmt's 100, which no gate in this unit enforces). Reformatting was not in
  scope.
- P2's locals are still named `legacy_elements`/`grouped_elements` while both now
  come from the surviving builder. Sec5.0 permits a callee-only change, so the
  names stay.
- Commit 1 verification: the ledger's claimed repairs are present as-found --
  the line-width rule at `milestone_ledger.md:17-19`, the updated M23.5 row at
  `:34`, the superseded-E disposition at `:43` and `:47`, deferred row F14 at
  `:86` with its detail sub-block from `:89`, and no ledger line at or above 2000
  characters (`^.{2000,}$` -> no match). The superseded
  `tasks/m235-t3e-handoff.md` was landed byte-unchanged (737/170, as found) and
  was not edited or deleted.

## Refusals encountered (verbatim, per sec10)

- One refusal, for the temp-dir escaping helper invoked as a raw script
  (`python C:\Users\Kurt\AppData\Local\Temp\kilo\make_chunks.py`):
  `The user has specified a rule which prevents you from using this specific
  tool call. Here are some of the relevant rules
  {"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
  Boilerplate catch-all. Re-attempted independently through the sanctioned
  `python tools/distill.py -- ...` entry point, which succeeded (Dev-6).
- Every gate was attempted independently and every one ran: gates 1 (x4), 2, 3
  (x2), 4 (x2), 5, 6 (`--all` x2, `--staged` x1), 7 (x2), plus
  `checkpoint_guard.py status` before and after the test runs. No gate was
  dropped, narrowed, or skipped because of a refusal.

## Commits

| SHA | Scope | Content |
|---|---|---|
| `ab59bb4` | 7 explicit `--scope`s under `tasks/` | planner records, byte-as-found: `milestone_ledger.md` 107/6, `ledger_archive.md` 13/1, `m235-t3e-handoff.md` 737/170, plus pure additions `m235-t3d2-verdict.md` 537/0, `m235-t3e-journal.md` 203/0, `m235-t3e1-handoff.md` 697/0, `m235-t3e2-handoff.md` 518/0 |
| (this commit) | 7 code paths + this journal | the retirement plus the converted lock |

Neither commit contains `tests/fixtures/basic/artifacts/includes/report.json`
(F12), `crates/cli/**`, `crates/graph/**`, `crates/report/**`, `tests/**`,
`tools/**`, `Cargo.toml`, `Cargo.lock`, `.gitattributes`,
`crates/ui/src/lib.rs` or `crates/ui/tests/ui_test/main.rs` -- verified by
`checkpoint_guard.py status` (only the seven code paths dirty) and by
`git diff --numstat HEAD -- crates/ui`.
