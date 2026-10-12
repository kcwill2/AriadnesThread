# M23.5 T1 Journal - Symbol SCC De-recursion (crate:cluster)

**Implementer run:** 2026-10-04
**Handoff:** `tasks/m235-t1-handoff.md` (T1.1 + T1.2)
**Kickoff:** `tasks/m235-kickoff.md`
**Touch-set honored:** `crates/cluster/src/scc.rs`, `crates/cluster/tests/scc_iterative_parity.rs`, this journal. Nothing else modified.

---

## What was done

### T1.1 - Red (parity oracle frozen)

- Created `crates/cluster/tests/scc_iterative_parity.rs`:
  - `recursive_tarjan_oracle` - verbatim test-only copy of the pre-T1.2
    `compute_symbol_sccs_from_adj` + `strongconnect` (kickoff tree `f9064f4`).
    Renaming anything inside the copied body was deliberately avoided; see
    the R3 note below.
  - `test_fixtures_parity` - all 9 unit fixtures from `crates/cluster/src/scc.rs`,
    rebuilt as symbol-list + edge-pair tables through a test-local
    `build_adjacency_from_edge_pairs` that mirrors `build_symbol_adjacency`
    push order (including duplicates).
  - `test_seeded_random_parity` - 200 random digraphs; documented seed
    `SEEDED_SWEEP_BASE_SEED = 0x0DA7ED5EED123456`, per-graph re-seed via
    `base ^ (ordinal * 6364136223846793005)`, Knuth MMIX LCG constants
    (platform-independent).
  - `test_100k_chain_parity` - open chain `s0->...->s99999` and fused variant
    `+s99999->s0`. Production runs on a 64 KiB `Builder::stack_size` thread;
    oracle runs on a dedicated large-stack thread (see Deviation D2).
- Red demonstrated (gate 1): the binary aborts in `test_100k_chain_parity`
  with `thread '<unknown>' (45248) has overflowed its stack` /
  `STATUS_STACK_BUFFER_OVERRUN (0xc0000409)` while `test_fixtures_parity`
  and `test_seeded_random_parity` pass - exactly the overflow class IV-7
  targets (failures pass is as expected pre-swap: the oracle is a copy of
  the then-live implementation).
  Capture: `target/testrun-capture.txt` at that step.

### T1.2 - Green (iterative frame machine)

- `crates/cluster/src/scc.rs`: replaced recursive `strongconnect` with an
  explicit `Vec<TarjanFrame>` machine. Public API unchanged
  (`compute_symbol_sccs_from_adj`, wrapper `compute_symbol_sccs`,
  `build_symbol_adjacency` untouched; `SymbolScc` untouched).
- Structure: `run_tarjan` (seed loop) -> `TarjanRun::explore_from` (frame
  loop) -> `advance_frame` / `claim_next_edge` / `enter_vertex` /
  `relax_lowlink_toward_index` / `retire_frame` -> `emit_group_if_root` +
  `finish_child`. All helpers well under 100 lines; file is 455 lines.
- **Lowlink propagation is on frame POP** (`retire_frame` -> `finish_child`),
  mirroring the recursive `min(lowlink[v], lowlink[w])` applied after the
  call; back-edge relaxation (`min(lowlink[v], index[w])` for on-stack
  neighbors) happens at edge claim. SCC emission at pop happens BEFORE
  propagation - same event ordering as the recursive body, which is what
  locks byte-identical output (IV-8).
- Frame state: plain HashMap/HashSet/Vec of borrowed `&SymbolId` - no
  `Rc`/`RefCell`, adjacency read-only: `Send + Sync`-compatible for the
  rayon planner.
- Production `scc.rs` carries no test-only copy of itself; the existing
  `#[cfg(test)] mod tests` (public-API fixture tests, not an oracle copy)
  was left untouched.

### Verification (canonical digest lines, verbatim)

| Gate | Command | Result |
|------|---------|--------|
| T1.1 Red (binary-scoped) | `python tools/testrun.py --distill --timeout 1100 -p ariadne-cluster --test scc_iterative_parity` | `### cargo test DRIFT (exit 2)` + capture shows `has overflowed its stack` (Red, expected) |
| T1.2 Green parity | same command after swap + D2 | `3 passed / 0 failed / 0 ignored / 1 suites` |
| Full cluster crate | `python tools/testrun.py --distill --timeout 1100 -p ariadne-cluster` | `54 passed / 0 failed / 0 ignored / 8 suites` (ran twice: post-Green and post-refactor; identical) |
| IV-8 m00 byte-lock | `python tools/testrun.py --distill --timeout 1100 -p ariadne-integration-tests --test m00_facts_path_parity_test` | `2 passed / 0 failed / 0 ignored / 1 suites` - unedited |
| IV-8 planner locks | covered by full-cluster run above | `planner_facts_parity`, `planner_characterization` green, unedited |
| Build | `python tools/distill.py -- cargo build -p ariadne-cluster` | `ok (exit 0, 0 warnings emitted)` -> `target/distill-capture.txt` |
| Hygiene | `python tools/hygiene_check.py` | no R1/R2 violations; R3 warn-only (see below) |
| Encoding | `python tools/encoding_guard.py --all` | clean (no output) |
| Encoding (staged) | `python tools/encoding_guard.py --staged` | run after staging - result recorded in report |

## What was deliberately NOT done

- Phase 2 (graph crate `CycleGroup`) untouched - kickoff authorizes Phase 1 only.
- `representative_cycle_for` from the handoff 5-refactor list was NOT stubbed
  into `scc.rs`: it is a Phase 2 / crate:graph (T2.2) concept; a dead stub in
  cluster would emit a `dead_code` warning and violate the no-speculative-
  helpers rule. The other four named helpers (`run_tarjan`, `advance_frame`,
  `finish_child`, `emit_group_if_root`) all exist verbatim as named.
- No golden fixtures, byte-lock tests, planner code, or other crates touched.
- The Red state was never committed: committing a process-aborting test
  would poison bisection; TDD discipline was exercised in the working tree.

## Deviations (self-flagged, with evidence)

- **D1 - handoff sec 2 pseudo-signatures.** Sec 2 writes the API as
  `SymbolPtr`, `&[Vec<usize>]`, `SymbolScc { members, is_cyclic }`. The real
  kickoff-tree API is `SymbolId`, `HashMap<&SymbolId, Vec<&SymbolId>>`,
  `SymbolScc { symbols }`. IV-8 freezes the real signature and output, so the
  real API was preserved and the tests written against it. Sec 2 treated as
  descriptive drift, not an interface-change order (which IV-8 + the kickoff
  locks forbid). Attribution: planner artifact.
- **D2 - oracle stack 2 MiB is insufficient.** First Green run aborted with
  `STATUS_STACK_OVERFLOW` on the ORACLE thread: the recursive oracle needs
  more than 2 MiB for 100k recursion levels (capture retained). Raised
  `ORACLE_STACK_BYTES` to 64 MiB (reserve is virtual; committed memory is
  touched pages only). The IV-7-bearing constant `SMALL_STACK_BYTES = 64 KiB`
  for the production thread is unchanged, and the passing suite now proves
  the 64 KiB production claim discriminates (pre-swap abort vs post-swap
  pass on the same fixture). Attribution: plan/handoff sizing estimate.
- **D3 - gate filter form.** The listed gate `... -- scc_iterative_parity`
  matches against TEST names, not the file name, so it selects 0 tests and
  passes vacuously (`0 passed / 0 failed / 0 ignored / 7 suites`, exit 0).
  Ran it once, logged it, then used the substantive form
  `--test scc_iterative_parity` (same binary, all 3 tests). Attribution:
  planner gate wording.
- **R3 warnings on the oracle.** `hygiene_check` flags single-letter `w`
  (lines ~380/411) inside `oracle_strongconnect` / the copied SCC block.
  These are verbatim-copied locals; renaming them would break the "frozen
  verbatim copy" requirement. R3 is warn-only; all names introduced by my
  own code are intention-revealing (the earlier single-letter fixture
  destructuring was renamed to `symbol_a..symbol_e` during refactor).

## Command refusals (verbatim)

Raw `git` and compound PowerShell forms are denied by the implementer
allowlist. Both refusals printed the identical catch-all boilerplate:
`The user has specified a rule which prevents you from using this specific
tool call. Here are some of the relevant rules
{"permission":"bash","pattern":"*","action":"deny","source":"global"}`
Refused: (1) `git status --short; git log --oneline -3` (worktree census -
substituted by tool outputs and this journal's commit list);
(2) a `> $null; Write-Output $LASTEXITCODE` hygiene/encoding exit-code probe
(substituted by reading `hygiene_check.py`/`encoding_guard.py` exit semantics).
Neither refusal is a handoff gate; all five gates ran (see table).

## Commit list

Recorded after commit; see implementer report for SHAs.
