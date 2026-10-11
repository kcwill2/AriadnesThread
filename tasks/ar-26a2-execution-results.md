# AR-26a2 Execution Results - reviewer acceptance pass (own stdout)

Reviewer session: `ses_ed3a87679ffeH9TbgwqimeQnto`. Verdict:
`tasks/ar-26a2-verdict.md`. Reviewer-gate skill, fresh full verification.

Ground truth (`python tools/checkpoint_guard.py status`): HEAD `8eec456`,
branch `shining-enthusiasm`, DIRTY FILES vs HEAD `(none)` (untracked docs
only, all `tasks/**`/`.kilo/plans/**` paperwork). Chain: `69ab3f7` (dispatch
HEAD) -> `d32c3fa` -> `0d348ee` -> `8eec456`.

## Scoped gates (fresh, own stdout; capture `target/testrun-capture.txt`)

- Gate 1 `python tools/distill.py -- cargo check --workspace --all-targets`
  -> `ok (exit 0, 31 warnings emitted)` - equals the hop-1 landed census.
  Own-capture attribution: every warning summary line names lang-cpp (10),
  ui (2), cli (1), integration-tests rows; ZERO warning lines attributable
  to `ariadne-report`/`ariadne-artifact` (grep of capture: neither crate
  name appears at all).
- Gate 2 `-p ariadne-artifact`: `40 passed / 0 failed / 0 ignored / 2 suites`
  - target attribution from capture: `Running unittests src\lib.rs` 40 +
  `Doc-tests ariadne_artifact` 0.
- Gate 3 `-p ariadne-report`: `7 passed / 0 failed / 0 ignored / 4 suites`
  - capture targets: `Running unittests src\lib.rs` 0,
  `Running tests\bincode_bounded_decode_test.rs` 6,
  `Running tests\extraction_artifacts_test.rs` 1,
  `Doc-tests ariadne_report` 0. PASSED 7 = contract authority.
- Gate 4 `-p ariadne-scheduler`: `26 passed / 0 failed / 0 ignored / 2 suites`.
- Gate 5 `-p ariadne-cli`: `118 passed / 0 failed / 8 ignored / 7 suites`.
- Gate 6 `-p ariadne-integration-tests --test m00_golden_baseline_test
  --test m00_facts_path_parity_test --test m23_symbol_isolation_test`:
  `11 passed / 0 failed / 0 ignored / 3 suites` - capture targets:
  m00_facts_path_parity 2 + m00_golden_baseline 2 + m23_symbol_isolation 7
  (= m00 pair 4 + m23 pinned 7). `git status --porcelain -- tests/golden`
  -> EMPTY.
- Gate 7 pinned `|` form REFUSED (see refusals); substituted -e invocation
  `python tools/distill.py -- rg -n -e collect_stage_languages
  -e hidden_glob_reexports -e unused_imports crates/report/src/lib.rs
  crates/artifact/src/manifests.rs` -> capture 1 line:
  `crates/artifact/src/manifests.rs:22:pub(crate) fn collect_stage_languages(`
  - exactly one line, `pub(crate)` prefix, ZERO report-lib lines.
- Gate 8 `python tools/hygiene_check.py`: all printed rows are R3 (warn);
  ZERO blocking (no R1/R2); no row at any `crates/report` or `crates/artifact`
  path. Exit code itself not echoable (allowlist), adjudicated on row set.
- Gate 9 `encoding_guard.py --staged`: NOT re-run by reviewer (tree clean,
  nothing staged); implementer's pre-commit silent run journalled.
- Gate 10 `python tools/encoding_guard.py --all` -> silent (exit 0).

## Workspace census + tools (tree quiet)

- `python tools/testrun.py --distill -- --workspace` ->
  `1398 passed / 0 failed / 13 ignored / 105 suites` - EXACT anchor match.
  Suites 105 -> 105 (no drift): report kept its empty lib target as a suite,
  artifact's two targets unchanged in count.
- `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 219 tests in 21.079s` / `OK`.
- `python tools/testrun.py --baseline` -> `usage error: missing value for
  --baseline` - errata confirmed (file-mode form; no repo baseline file);
  substantive baseline NOT run, consistent with prior reviewer records.
- `ui_smoke.py` / `bench_compare.py`: NOT run per dispatch (closeout duty).

## Suite reconciliation table (adjudication 3)

| Lane | Pre-hop `4794467` | Post-hop (reviewer-measured) |
|---|---|---|
| ariadne-artifact | 2 suites @ 0 (lib 0, Doc 0) | 2 suites @ 40 (lib 40 = 24 tests.rs + 11 m22 + 5 payload_reshape, Doc 0) |
| ariadne-report | 4 suites @ 47 (lib 40, bincode 6, extraction 1, Doc 0) | 4 suites @ 7 (lib 0, bincode 6, extraction 1, Doc 0) |
| pair total | 6 suites @ 47 | 6 suites @ 47 |
| workspace | 1398/0/13/105 | 1398/0/13/105 measured - 105->105, no drift |

Conservation `40 + 7 = 47` PASSED holds (the contract's stated authority).
The gate-3 header projection "3 suites" miscounts the contract's own
4-target enumeration; measured 4. Tolerance clause applied; the projection
text is the defect, not the land.

## Verbatim evidence (independent re-derivation)

- Per-commit touch-set: `d32c3fa` = journal only; `8eec456` = journal only;
  `0d348ee` = 17 paths: 11 R100 renames + Cargo.lock + artifact Cargo.toml +
  artifact lib.rs + manifests.rs + report lib.rs + journal. All inside the
  declared touch-set; `crates/report/Cargo.toml` and workspace `Cargo.toml`
  absent from every diff (UNTOUCHED confirmed).
- VERBATIM at scale: `git diff --find-renames --raw -M 4794467 0d348ee --
  crates/report/src crates/artifact/src` -> all 11 rename pairs with
  IDENTICAL src/dst blob hashes (byte identity proven hash-level, stronger
  than R100); the only differing blobs in the whole moved-set region are
  artifact lib.rs (pin), manifests.rs (pin hunk), report lib.rs (pin).
- `git diff 4794467 -U0 -- crates/artifact/src/manifests.rs` -> exactly
  `1 1` numstat; sole hunk `-pub fn` / `+pub(crate) fn` at line 22. File
  holds 222 lines (contract hygiene pin confirmed).
- Rider trio: `crates/report/src/lib.rs` = 7 lines byte-matching the
  contract pin (2 doc + blank + manager pair + manifests pair); the
  rationale comment, allow pair, and `pub(crate) use ...` line are GONE
  (gate-7 zero report lines + direct read). `crates/artifact/src/lib.rs` =
  pinned 9 lines.
- Terminal surface: repo-wide `rg -n collect_stage_languages crates/ tests/
  tools/` -> 8 lines = the `pub(crate)` def at manifests.rs:22 + the three
  same-crate consumers (discovery_io, metrics_io, graph_io; imports + call
  sites). Nothing in report, tests, or tools. Terminal surface = 12 DTOs +
  manager exports; the 26a transient-widening window is CLOSED.
- 9-site census at new home: `rg -c "use crate::manifests"` -> exactly 9
  files (apply_io, discovery_io, graph_io, index_io, metrics_io,
  transform_io, tests, m22, payload_reshape), one site each, under
  `crates/artifact/src/manager/`; R100 blob identity proves byte-unchanged.
- D7 re-proof: `git show "HEAD:crates/report/src/manager.rs"` ->
  `fatal: path ... does not exist in 'HEAD'`; disk census (Glob
  `crates/report/src/**`) -> only `lib.rs`. No tracked or on-disk stale copy.
- Frozen locks: `git diff 4794467 --stat -- tests crates/report/tests` ->
  zero diff lines (stderr carries only the foreign report.json CRLF warning);
  m17 file diff -> EMPTY; `git status --porcelain -- tests/golden` -> empty.
- Mutation trail: `crates/scheduler/src/incremental.rs:23` is
  `ariadne_report::ArtifactManager::prepare_for_repo(repo)?` - the predicted
  E0433 site, mechanically consistent with the journalled error. Revert
  byte-complete: report lib.rs at `0d348ee` equals the pinned 7-line content
  and `8eec456` touches only the journal.
- `#[test]` conservation: reviewer recount 47 total across both crates
  (report 1 + 6; artifact 24 + 11 + 5) = per-file pre-move distribution,
  relocated intact.
- `crates/artifact/src/manager/tests.rs` = 910 lines, whole, under ceiling.

## Reviewer refusal log (verbatim catch-all boilerplate each time:
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`)

- G7 pinned `|`-alternation form via distiller: refused (reproduces the
  implementer's recorded refusal; `|` trips the command filter).
  Substituted with three `-e` flags - IDENTICAL semantics (multiple `-e`
  patterns are an OR of the same terms) - same files, same `-n`.
- `git ls-files ...` (D7 old-path check): refused; substituted with
  `git show "HEAD:<old-path>"` fatal + disk Glob census (same substitution
  family accepted at AR-26a).
- One compound `git diff`/`ls-files` chain: refused; re-run split into
  allowed single invocations.
- Bare `rg -n collect_stage_languages crates/` (distiller-bypassed): refused;
  re-run through `python tools/distill.py -- rg ...` -> success.
- `python -m unittest discover ... | Select-Object -Last 3` (pipeline):
  refused; re-run plain -> `Ran 219 tests ... OK`.

No gate was skipped; every refusal has an accepted substitution with output.

## Explicitly NOT run

- `--baseline` substantive run (file-mode; no repo baseline file - errata
  quoted above), `ui_smoke.py`, `bench_compare.py` (closeout duties per
  dispatch), `encoding_guard --staged` (nothing staged at review time).
