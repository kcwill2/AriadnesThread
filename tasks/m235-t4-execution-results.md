# M23.5 T4.1 execution results -- Phase 4 closeout

Session executed 2026-10-04 against contract `tasks/m235-t4-handoff.md`.
Base `dcd681b`, branch `tarjan-s-scc`, worktree `.kilo/worktrees/tarjan-s-scc`.
Journal (per-gate checkpoints + refusals): `tasks/m235-t4-journal.md`.

## Commits

| # | SHA | Scope | Content |
|---|-----|-------|---------|
| 1 | `b3f33a9` | `.gitattributes`, `crates/ui/tests/ui_test/export_order.rs` | F12 pin (+4), F13 rename (5/5 lines) |
| 2 | (this landing) | `tasks/**` only | ledger F12/F13/milestone/strays rows, this file, journal, T3-E2 verdict + execution-results (previously untracked reviewer records), T4 handoff |

The F12 fixture appears in NO commit (`git show --numstat` of commit 1 lists
only the two scoped paths; fixture byte-equal to HEAD blob `501b19bd...`).

## Red (sec5, captured pre-edit, both for the predicted reason)

1. F12: porcelain ` M tests/fixtures/basic/artifacts/includes/report.json`
   while `git diff -- <path>` returned NO hunks. Churn signature confirmed.
2. F13: grep `legacy_elements|grouped_elements|the_parity_comparison` over
   `export_order.rs` -> exactly 5 hits at `:298 :311 :318 :325 :326`.

## Gates (verbatim canonical lines; captures under `target/`)

| gate | command | result |
|------|---------|--------|
| 1 | `python tools/checkpoint_guard.py status` | HEAD `dcd681b`; dirty set + census recorded in journal (2 tree-vs-handoff deviations, TREE WINS) |
| 2 | `python tools/testrun.py --distill --timeout 3600` | `1180 passed / 0 failed / 4 ignored / 88 suites` (run 1, exit 0, capture `target/testrun-capture.txt`) -- then re-run post-F12-normalization: `1180 passed / 0 failed / 4 ignored / 88 suites` (exit 0) |
| 3 | `python tools/testrun.py --distill --timeout 1200 -p ariadne-ui --test ui_test` | `186 passed / 0 failed / 0 ignored / 1 suites` -- count pin `186/0/0/1` SURVIVES the rename |
| 4 | `python tools/hygiene_check.py` | R1: 0 blocking lines, R2: 0 blocking lines; R3 warns all pre-existing, none in touched files |
| 5 | `python tools/encoding_guard.py --all` then `--staged` | both silent (no output) |
| 6 | `python tools/distill.py -- cargo test -p ariadne-cli --test payload_tail_bench --release -- --ignored --nocapture --test-threads=1` | `ok (exit 0, 0 warnings emitted)`; `running 1 test` -> `test result: ok. 1 passed; ... finished in 14.02s` -- bench REALLY ran |
| 7 | `git status --porcelain -- tests/fixtures/basic/artifacts/includes/report.json` (via distill) | **empty output after the post-pin full run** -- F12 PROVEN CLOSED |
| 8 | `git diff --stat HEAD -- tests/golden` (via distill) | **empty output** -- goldens byte-clean vs HEAD after two full runs |
| 9 | `python tools/commit_scoped.py --scope ... -m ...` | 2 scoped commits; explicit scopes only |
| -- | `python tools/theme_token_check.py --check-added --base dcd681b` | silent -- zero hex literals added (UI test file touched) |

## F12 closure evidence (BEFORE / AFTER)

- BEFORE: preflight porcelain ` M` + empty diff (Red 1) -- tenth reproduction
  of the documented signature.
- First attempt: pin added, full run 1 executed, porcelain STILL ` M`.
  Measurements: `git ls-files --eol` -> `i/lf w/lf attr/text eol=lf`;
  `git check-attr` -> text set, eol lf; `git hash-object <file>` =
  `501b19bde2192befb6ba0e81c9de2ff3005202d8` = `git rev-parse
  HEAD:<path>` -- content byte-equal to HEAD while status said M.
  Diagnosis: stale INDEX entry caching the pre-pin CRLF-checkout expectation.
- Normalization: `git checkout -- <path>` (byte-identical re-smudge; hash
  re-verified identical after).
- AFTER: full workspace run 2 executed post-pin+normalization; porcelain for
  the path prints NOTHING. Pin prevents recurrence; the fix is proven across
  a full test run, not merely asserted.
- Mechanism finding recorded against the handoff's model in the ledger F12
  row and journal: the pin alone does not clear an already-stale entry; a
  one-time normalization is required per pre-pin dirtied checkout.

## Goldens determination (sec3.2)

- `git diff --stat HEAD -- tests/golden` empty; `git status --porcelain --
  tests` empty after BOTH full runs -> every golden byte-equal to HEAD.
- `.gitattributes` original two pins intact after the edit (7 lines -> 11,
  lines 1-7 unchanged; new pin lines 8-11).
- `UPDATE_GOLDENS=1` NEVER run in this session: no command invoked sets that
  variable (every command is logged in the journal/captures), and the golden
  comparison tests PASS under default env -- a stale golden fails by design
  when the env var is absent. Re-pin is therefore UNNECESSARY; no golden byte
  moved; no file added under `tests/golden/**` or `tests/fixtures/**`.
- Reviewer-established context respected: goldens are cycle-free
  (`"cycle_count": 0` only; `IncludeReport` has no cycle-list field), so
  touching one would have been a FINDING. None touched.

## Bench timings (gate 6) -- UNVERIFIED label INTACT, divergence UNRECONCILED

Fixture: 520 headers, 40 TUs (cycle-bearing by construction). Selected rows:

```
generate_report+detect_cycle_groups (combined)   |      8.0
build_include_elements                           |      8.0
```

Recorded next to, and deliberately NOT reconciled with, the three doc values:
`5.1` (`crates/cli/src/commands/ui.rs:103` calibration row),
`5.4` (`crates/cli/tests/payload_tail_bench.rs:41` bench header),
`5.6` (`payload_tail_bench.rs:40`, T3-D re-pinned stage-5 figure).
No source line was edited; the `UNVERIFIED` paragraphs at
`payload_tail_bench.rs:80-87` and `ui.rs:115-119` remain verbatim and
unabridged. Reconciliation while F10 is open would manufacture perf evidence
(T3-E2 verdict finding 6). These 8.0 figures are ONE sample on the dev
machine, not baseline evidence; `bench_compare.py` (the baseline tool) was
NOT run (refused gate; operator-owned).

## Plan file (sec3.4)

`git ls-files --error-unmatch .kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md`
-> exit 0, path echoed. **TRACKED.** No landing needed; cited as the contract
of record. Discharges the ledger strays-row UNVERIFIED caveat.

## Determinism evidence (sec3.5) -- REPORT, not built

- EXISTS: `payload_tail_bench.rs:571`
  `export_bytes_are_identical_across_two_separate_processes` (cross-process,
  small SCC fixture, non-ignored, inside the `1180/0/4/88` run) and
  `graph_api.rs:742`
  `include_elements_export_is_stable_across_two_calls_with_cycle_fixtures`
  (in-process two-call, two-disjoint-group graph, non-ignored).
- DOES NOT EXIST: any determinism assertion on the 520-header SCC-dense bench
  fixture -- its only exerciser `payload_tail_phase_timing` is `#[ignore]`d
  and asserts payload sizes. Plan T4.1's "determinism re-check on the
  SCC-dense 520-header bench fixture" is UNMET AS LITERALLY WORDED. Adding it
  = new test = new RGB unit. Not built, per contract.

## Outstanding after this unit (all operator / separate-role)

1. **F10 (BLOCKS closeout):** reviewer allowlist refuses `ui_smoke.py` and
   `bench_compare.py`; both NOT attempted, NOT routed through `distill.py`
   (that would falsify the gate). Zero runtime UI evidence and zero verified
   perf evidence until an operator amends the allowlist and runs both.
2. **kilo-memory mirror of the ledger:** no in-workflow role holds the memory
   tool; the ledger edits in the records commit are UN-MIRRORED at commit
   time -- operator must mirror in/with the same landing.
3. **Worktree census disposition:** `tarjan-s-scc` (ahead 32) NOT merged to
   `main`; mutating git denied to every role. Merge or abandon-with-row.
4. **Walkthrough deck:** `milestone-lifecycle` expects one committed on this
   branch; none exists. Separate role (`walkthrough-commit`), requested
   separately. Not authored here.

## Deviations and refusals (full list in journal)

- Handoff sec2 said "ONLY dirty path is the fixture" / "ONLY two untracked
  records": tree also showed dirty `tasks/milestone_ledger.md` +
  `tasks/ledger_archive.md` (planner bookkeeping) and untracked
  `tasks/m235-t4-handoff.md`. TREE WINS; landed together in the `tasks/**`
  records commit (no code). Documentation defect against the handoff.
- `.gitattributes`: added a 3-line rationale comment with the mandated pin
  line, matching the file's per-pin comment convention ("Add ONE line,
  keeping the file's existing comment style"). Config-only; disclosed.
- F12 required a one-time `git checkout -- <path>` normalization beyond the
  contract's model (see evidence above); byte-identity proven before/after;
  the path is in no commit and remains byte-equal to HEAD.
- Refusals (all reported the catch-all rule verbatim; none routed around):
  `git grep` via distill (Red 2 re-run with the dedicated Grep tool),
  `git diff --no-index` via distill (probe; skipped, non-gate),
  `python -c` via distill and a read-only PowerShell mtime probe (golden
  mtime probe abandoned; conclusion already carried by gate 8 evidence).
  `wc` via distill failed with WinError 2 (binary absent -- not a refusal).
- `--staged` encoding gate: `commit_scoped.py` stages atomically, so no
  external window exists to inspect a populated index; `--staged` run silent
  before each commit call and `--all` run over the exact content before
  committing. Same method as the approved T3-E2 precedent.
