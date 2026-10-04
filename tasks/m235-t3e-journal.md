# M23.5 T3-E Journal - STOPPED AT PREFLIGHT: contract defect #5 (workspace-breaking rename)

Contract: `tasks/m235-t3e-handoff.md` (AMENDED 2026-10-04 version).
Dispatch base (my `checkpoint_guard.py status` preflight): HEAD `b0a91d2`,
branch `tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`.
Outcome: **STOPPED before implementation.** The amended contract is internally
unsatisfiable: sec3.3's mandated rename provably breaks `crates/cli`, which
sec2 and the sec8 lock freeze byte-unchanged.
Evidence trail below; no production or test file was modified; zero commits.

## Checkpoint 0 - preflight (sec0), all steps run

- Step 0.1 `checkpoint_guard.py status`: HEAD `b0a91d2`, branch `tarjan-s-scc`.
  DIRTY (content numstat): `tasks/ledger_archive.md` 13/1,
  `tasks/milestone_ledger.md` 31/6, `tasks/m235-t3e-handoff.md` 737/170 (the
  planner's repair -- uncommitted, as the dispatch stated).
  UNTRACKED: `tasks/m235-t3d2-verdict.md`.
  Worktree census shows `dirty=5` (the four above plus the F12 fixture stat).
- Step 0.2 F12 fixture `tests/fixtures/basic/artifacts/includes/report.json`:
  `git diff --numstat HEAD -- <path>` (via `distill.py`) returned ZERO numstat
  rows -- only the known autocrlf stat-cache warning line
  (`warning: in the working copy of '...report.json', LF will be replaced by
  CRLF the next time Git touches it`).
  Content byte-equal, so per sec0.2 ("if and only if ... shows rows") NO
  restore was needed and none was attempted. Path appears in no commit (there
  are no commits).
- Step 0.3 caller census, grep `build_include_elements` + OPEN PAREN:
  exactly TEN textual hits -- the definition `dto.rs:608` plus NINE call sites,
  all inside `crates/ui/tests/ui_test/`: `common.rs:157`, `dto_builders.rs:15`,
  `d27_node_provenance.rs:141` and `:152`, `graph_api.rs:136`,
  `export_order.rs:91`, `:108`, `:310`, `:381`.
  Every file:line matches the handoff's sec1 table exactly.
  Zero legacy-name calls outside `crates/ui/**`.
  The three sanctioned non-code hits confirmed present:
  `crates/cli/tests/payload_tail_bench.rs:389` (bench row LABEL string),
  `crates/cli/tests/payload_tail_bench.rs:41` and
  `crates/cli/src/commands/ui.rs:103` (prose in doc/`//` comments).
- Entry counts re-derived from the tree: `dto.rs` 901 lines,
  `cycle_groups_view.rs` 149, `export_order.rs` 446 -- all exactly as the
  handoff's sec6 measured values. Gate 1 baseline at the untouched base (see
  Gates): `186 passed / 0 failed / 0 ignored / 1 suites`.

## BLOCKER - defect #5: the mandated rename breaks crates/cli, which is locked

### The contradiction, in three tree facts and two contract clauses

1. TREE FACT 1: `crates/cli/src/commands/ui.rs:328` (production `build_ui_payload`)
   calls `ariadne_ui::build_include_elements_from_groups(...)` -- real,
   ungated lib/bin code.
2. TREE FACT 2: `crates/cli/tests/payload_tail_bench.rs:325` calls
   `ariadne_ui::build_include_elements_from_groups(...)` inside `run_assembly`
   -- real, ungated bench-target code.
   A full-repo grep for `build_include_elements_from_groups` finds exactly
   these two code references outside `crates/ui`; every other hit is under
   `tasks/**` or the handoff/plan.
3. TREE FACT 3: `crates/ui/src/lib.rs:6` is `pub use dto::*;` -- the only
   export path; no alias, no shim, so removing the name from `dto.rs` removes
   `ariadne_ui::build_include_elements_from_groups` from the workspace.
4. CONTRACT CLAUSE 1 (sec3.3): "Rename `build_include_elements_from_groups`
   (`dto.rs:637`) to `build_include_elements`" -- the grouped name must cease
   to exist; sec8's lock requires the reviewer grep to find ONE definition.
5. CONTRACT CLAUSE 2 (sec2 / sec8): `crates/cli/**` is "MUST NOT touch" and
   frozen "byte-unchanged | reviewer's `git diff --name-only` | absent".

After sec3.3, facts 1-2 dangle against clause 1, and clause 2 forbids the only
repair. The unit as contracted therefore cannot land without either violating
its own touch-set or leaving the workspace non-compiling -- the exact
handoff-boundary law its sec1 ("keeps every handoff boundary compiling and
green") and the dispatch's `1178/0/4/88` green-workspace premise assert.

### Executed proof (probe, reverted, byte-for-byte)

- Baseline `certutil -hashfile crates/ui/src/dto.rs SHA256` (via `distill.py`):
  `e56ede8b95ec41d682b63f24db0e9da5e9f2e8a0f0d6340a5adfce4f283a9175`
- Probe: renamed `dto.rs:637` `build_include_elements_from_groups` ->
  `t3e_probe_grouped_name_removed` (one line; legacy function left intact, so
  the probe isolates exactly what the sec3.3 rename removes: the grouped
  export name). Everything else untouched.
- `python tools/distill.py -- cargo check -p ariadne-cli --all-targets`:
  **FAIL (exit 101)**, verbatim (capture `target/distill-capture.txt`):
      error[E0425]: cannot find function `build_include_elements_from_groups` in crate `ariadne_ui`
         --> crates\cli\src\commands\ui.rs:328:40
      error: could not compile `ariadne-cli` (lib) due to 1 previous error
      error: could not compile `ariadne-cli` (lib test) due to 1 previous error
  The bench call at `payload_tail_bench.rs:325` is the identical textual form;
  its target was not reached because the lib failed first.
- Revert: the exact inverse one-line Edit.
  Post-revert `certutil` SHA256 of `dto.rs`:
  `e56ede8b95ec41d682b63f24db0e9da5e9f2e8a0f0d6340a5adfce4f283a9175` --
  identical to the pre-probe value; byte-for-byte revert proven (content hash;
  `checkpoint_guard.py status` after shows `crates/**` absent from DIRTY).

### Why the contract missed it (attribution: planner, 6th defect, class of sec13.2)

- Step 0.3 grepped for callers of the LEGACY name (`build_include_elements(`)
  and found none in `crates/cli` -- true, but the hazard is callers of the
  GROUPED name, which the rename also deletes. The sec0.3 hit-list cites
  `:325` only as the bench's timed call and calls the cli occurrences
  "report-only, do not touch"; the author's model ("This unit makes that label
  accidentally correct") presumes cli keeps referencing the surviving name,
  yet cli's source text names `_from_groups` and cannot be edited.
- Chain of severance: `tasks/m235-t3a-verdict.md:739` records the rename
  "deferred to unit E"; T3-B migrated cli to `_from_groups`
  (`tasks/m235-t3b-handoff.md` sec3.1 B2, confirmed `MET` at
  `tasks/m235-t3b-execution-results.md:44` -- `ui.rs:328`);
  plan T3.2 (plan line 129) originally expected the same step to update
  `cli/src/commands/ui.rs` + the bench, but the A/B/E crate-split dropped
  that cli re-point from every later handoff. No scheduled unit owns
  `_from_groups` -> final name inside `crates/cli`.
- The unit's gate set masks it: gates 1-2 are `-p ariadne-ui` scoped, gate 3
  builds only `ariadne-ui`; no gate in E's set type-checks `crates/cli`.
  The breakage would first surface at T4.1 -- a silent, milestone-level
  regression, exactly the class this milestone's preflights exist to catch.

### Minimal resolutions (report-only; the architect's call, not mine)

- (i) PREFERRED: amend sec2/sec7/sec8 to add the two cli call sites to the
  touch-set (`crates/cli/src/commands/ui.rs`,
  `crates/cli/tests/payload_tail_bench.rs`, callee-name-only lines `:328` /
  `:325`; the `:389` label and `:41`/`:103` prose then need no change and the
  label becomes correct as the handoff predicted), and add a cli gate
  (`python tools/distill.py -- cargo build -p ariadne-cli`) to E. One commit,
  two crates, but the delete+rename must be ATOMIC with the cli re-point --
  any split breaks an intermediate boundary (plan line 108 states this law
  for the `detect_cycles` deletion; it applies verbatim here).
- (ii) Alternative: do NOT rename; retire the legacy name only (delete
  `build_include_elements` + `from_paths`, repoint the nine ui-test sites and
  four import lists to `build_include_elements_from_groups`).
  Workspace stays green, `crates/cli` stays byte-unchanged, D1-B closes;
  but this contradicts sec3.3's mandated name and the bench label stays stale.
- Under (i) everything else in the repaired contract survives unchanged: the
  sec5 P1 conversion, the frozen expectation, the mutation Red, the nine ui
  sites and the four import lists all stand.

## Gates actually run (all at the untouched base; none refused, none skipped
## silently), per the "attempt each gate independently" discipline

| # | Command | Measured |
|---|---|---|
| 3 (base run, as sec7 prescribes) | `python tools/distill.py -- cargo build -p ariadne-ui` | `ok (exit 0, 0 warnings emitted)` (capture `target/distill-capture.txt`) |
| 1 (baseline corroboration) | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test` | `186 passed / 0 failed / 0 ignored / 1 suites` (capture `target/testrun-capture.txt`, 194 lines) |
| 5 (baseline) | `python tools/hygiene_check.py` | zero R1, zero R2; R3 warns all pre-existing legacy files; none introduced by me (I authored no code) |
| 6 (baseline, `--all`) | `python tools/encoding_guard.py --all` | no output (clean) |

Gate 1's four contracted states (capture run, R1 compile-fail, R2 Green,
R3 mutation Red, final Green), gate 1 #2 (gate 2 `-p ariadne-ui` full),
gate 3 #2, gate 4 (theme tokens), gate 8 (commit): NOT run -- they all
presuppose the edits that the contract defect forbids. No Red trail was
executed because no P1 conversion exists yet; the dispatch's
`185 / 1 / 0 / 1` mutation pin was therefore never observable this session,
and I claim none.

## Not executed / not landed, with reasons

- No capture line, no P1 conversion, no deletions, no renames, no commits of
  code. The unit's mandated deliverable is unsatisfiable in scope; per
  dispatch rule "If the handoff is wrong, report it and stop. Do not
  improvise a fix." and scope discipline "stop and report ... instead of
  expanding scope".
- The dispatch-declared planner records (`tasks/milestone_ledger.md` 31/6,
  `tasks/ledger_archive.md` 13/1, untracked `tasks/m235-t3d2-verdict.md`,
  and the repaired `tasks/m235-t3e-handoff.md` 737/170) were left UNCOMMITTED
  for the architect. DEVIATION from the dispatch's "they are yours to land",
  justified: landing a repaired handoff whose sec3.3 I am simultaneously
  adjudicating unsatisfiable would commit a known-unimplementable contract as
  ground truth, and a ledger claiming E dispatchable would be false one row
  later. The architect should land them in the same pass that repairs
  defect #5. Files are safe in the worktree and enumerated here.
- `tasks/m235-t3d2-execution-results.md` was named in the dispatch's
  land-list but DOES NOT EXIST on disk (tree wins): `tasks/` holds only
  `m235-t3d2-verdict.md`, `m235-t3d2-journal.md`, `m235-t3d2-handoff.md`.
  Record-only; the T3-D2 records were landed as verdict + journal + handoff.

## Refusals encountered (verbatim, per sec10 "Reading a refusal")

- One refusal: a combined invocation `Test-Path ...; Test-Path ...; python
  tools/distill.py -- git diff --numstat HEAD -- tests/fixtures/...` returned
  `The user has specified a rule which prevents you from using this specific
  tool call. Here are some of the relevant rules
  {"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
  Boilerplate catch-all; retried each part independently: file existence via
  the Glob tool, the fixture numstat via a standalone `distill.py` call
  (succeeded, see Checkpoint 0). No gate was dropped because of it.
  No other refusal occurred; `certutil` via `distill.py`, `cargo build`,
  `cargo check`, `testrun.py`, `hygiene_check.py`, `encoding_guard.py`,
  `checkpoint_guard.py` all ran on first independent attempt.

## Disk-state inventory at stop (fresh-session bootstrap)

- `crates/**`: ZERO edits. `dto.rs` SHA256 equals the pre-probe hash
  (`e56ede8b...`, proven above); probe line absent from every file.
- DIRTY (untouched, pre-dispatch planner work): `tasks/ledger_archive.md`,
  `tasks/milestone_ledger.md`, `tasks/m235-t3e-handoff.md`.
- UNTRACKED: `tasks/m235-t3d2-verdict.md`, this journal.
- HEAD `b0a91d2`; no commit made by this session; fixture path in no commit.
- A successor after the contract repair runs
  `python tools/checkpoint_guard.py resume --spec tasks/m235-t3e-handoff.md ...`
  from this state; nothing here needs undoing.

## Line counts at stop (unchanged from dispatch base, measured)

- `crates/ui/src/dto.rs`: 901. `crates/ui/src/cycle_groups_view.rs`: 149.
  `crates/ui/tests/ui_test/export_order.rs`: 446.
