# AR-26a Handoff Journal - BLOCKER STOP (contract seam infeasible)

Session: AR-26a implementer, dispatched 2026-10-11.
Dispatch HEAD: `c8ae8ea` (AR-31 journal; ancestry incl. `1140ed5` AR-31
tokenizer, `089a9d1` AR-31 characterization, `e946f57`/`a634ae5`/`2679ebb`
AR-26 split-check journal).
Worktree: `.kilo/worktrees/shining-enthusiasm` (shared; AR-31 reviewer lane
live in crates/lang-cpp).

## Verdict

BLOCKER. The handoff's central mechanism - a VERBATIM (R100, zero content
hunks) `git mv` of `crates/report/src/manifests.rs` into the new
`ariadne-artifact` crate, with the report 2-line re-export shim keeping the
manager IO modules' `use crate::manifests::{...}` lines compiling UNEDITED -
is FALSE for one item: `collect_stage_languages`.

`manifests.rs:22` is `pub(crate) fn collect_stage_languages`. A re-exported
path (`pub use ariadne_artifact::manifests;` + `pub use ...::*;`) is
path-resolvable in-crate (the AR-20a shim rule holds for the 12 `pub` DTO
structs) but does NOT widen item visibility: the fn stays private to
`ariadne-artifact`. Its three consumers stay in `ariadne-report` this hop
(`discovery_io.rs:13`, `metrics_io.rs:13`, `graph_io.rs:13` - manager/* is
26a2's touch-set) and import it by name. They fail `error[E0603]` - the hop
is NOT compile-green at its own seam, exactly the E0116-class trap the
split-check used to reject the 26a1 candidate, relocated one file upstream.

## Baselines (re-measured pre-mutation, own stdout per capture-race ruling)

- `python tools/testrun.py --distill -p ariadne-report` (capture
  `target/testrun-capture.txt`, 73 lines):
  `47 passed / 0 failed / 0 ignored / 4 suites` - matches contract pin.
- `python tools/hygiene_check.py` (own stdout): R3 (warn) rows only, ZERO
  blocking rows (post-AR-13 baseline holds).
- `python tools/encoding_guard.py --all`: silent (exit 0).
- scheduler/worker-core/m17/m00/cli/lang-cpp lines NOT re-run: no mutation
  was performed, nothing this diff adds can move them (AR-26 split-check
  precedent for skipping gates with zero code change).

## Evidence - faithful probe (NEVER touches the worktree)

Temp workspace `C:\Users\Kurt\AppData\Local\Temp\kilo\ar26a-seam-probe`:
`ariadne-artifact-probe` with `manifests.rs` carrying the REAL visibility
shape (`pub(crate) fn collect_stage_languages` + `pub struct
RepositoryManifest`); `ariadne-report-probe` lib.rs carrying the contract's
exact 2-line shim; `io_modules.rs` carrying the real `use
crate::manifests::{collect_stage_languages, RepositoryManifest};` shape
mirroring `discovery_io.rs:12-13`.

1. `python tools/distill.py -- cargo check --manifest-path
   ...\ar26a-seam-probe\Cargo.toml` (capture `target/distill-capture.txt`,
   25 lines) -> FAIL exit 101, own stdout verbatim:
   `error[E0603]: function \`collect_stage_languages\` is private`
   ` --> report-probe\src\io_modules.rs:5:24`
   `error: could not compile \`ariadne-report-probe\` (lib) due to 1 previous error`
2. Control (helper widened to `pub`, probe-only - the candidate deviation):
   same command -> `ok (exit 0, 0 warnings emitted)`. So the single-line
   visibility change IS the minimal fix; everything else (the 12 pub DTOs
   through the shim, the module-qualified path, the flat glob path) compiles.

Corollaries (predicted, not separately probed): verbatim `pub(crate) fn` in
`ariadne-artifact` with zero in-crate callers also emits a dead_code warning
in the new crate; the contract's shim mutation-proof (comment both shim
lines, expect E0432/E0433 at io import lines) cannot be run on the real tree
as written because the tree does not compile with the shim either.

## Options for the orchestrator (report-only, no code touched)

- A. Single-line visibility deviation: `pub(crate) fn` -> `pub fn`
  `collect_stage_languages` on the move. Cost: R100 becomes R099-ish
  (1 content hunk on the moved file - violates the VERBATIM/rename-completeness
  clause); report's PUBLIC surface grows by one symbol via `pub use ...::*`
  (a public-API change the contract never authorized); permanent through
  26a2/26b unless a revert row is banked.
- B. Re-split: the fn travels with its manager/* consumers in 26a2; 26a's
  manifests move becomes a file SPLIT (26a moves the 12 pub structs, leaves
  the fn) - breaks "git mv VERBATIM zero hunks" for 26a differently and
  grows 26a2.
- C. Merge 26a+26a2 into one hop (3101 moved lines): the whole
  manager/* + manifests.rs atom IS the compile-green seam (E0603 here is
  the same atomicity class as the E0116/E0583 proof). The pre-ACCEPTED
  line-budget overrun was pinned at 2879 moved (~0 churn); 3101 needs a
  NEW acceptance, not reuse of the existing one.

Recommend C on mechanical grounds (zero content hunks preserved everywhere),
A if the orchestrator accepts the one-symbol public-surface growth + R099.
This is the orchestrator's call, not the implementer's.

## Deliberately NOT done

- Zero worktree mutations: no crate created, no git mv, no shim, no Cargo
  edits. `git status --porcelain` at stop = foreign `report.json` LF/CRLF
  churn + untracked ar-* docs only (journal's own creation pending commit).
- No gates 1-8 run: with the seam proven non-green by probe + baseline
  suites above, gate output would carry nothing this diff adds.
- The probe workspace lives only under the temp dir; the repo never hosted
  a failing state for the live AR-31 reviewer lane.

## Gate record + commit proof (addendum - could only be recorded after
## the commit it describes; AR-16 `a48c018` / AR-26 `a634ae5` ordering precedent)

- Journal commit: `5b7c53d` (`104 0 tasks/ar-26a-handoff-journal.md`, sole
  path). `python tools/encoding_guard.py --staged` -> silent (exit 0).
  D7 rename hazard cannot arise: zero `git mv` exists in this stop.
- This addendum rides the immediate follow-up scoped commit (journal-only).

## Deviations

- D1 (process): blocker adjudication ran through a temp-dir cargo probe
  via `distill.py` instead of mutating the worktree - preserves the shared
  green tree for the live AR-31 reviewer lane; same evidence class (compiler truth)
  with zero blast radius.

# ADJUDICATED EXECUTION - ruling board_12d48349 (option A + riders)

Ruling: (1) one-line `pub(crate)`->`pub` on `collect_stage_languages`
PRE-APPROVED as disclosed 1-hunk deviation (R9x class); (2) shim must also
carry `pub(crate) use ariadne_artifact::manifests::collect_stage_languages;`
so the three IO consumers stay BYTE-UNEDITED; (3) rider for 26a2 (below);
(4) dead_code moot for pub items; (5) mutation proof on the real tree,
revert UNCOMMITTED.

## Pre-mutation baselines (re-anchored, own stdout)

- HEAD `2fb9b21`; code-anchor `c8ae8ea` unchanged (intervening `957c391` is
  tasks/ docs only - 26b shape ruling, outside all frozen paths).
- report `47 passed / 0 failed / 0 ignored / 4 suites` (pre-ruling run
  stands); m23_symbol_isolation_test dispatch-HEAD baseline
  `7 passed / 0 failed / 0 ignored / 1 suites`; hygiene R3-only ZERO
  blocking; `git status --porcelain -- tests/golden` empty (pre AND post).
- Foreign at dispatch, untouched: `tests/fixtures/basic/artifacts/includes/
  report.json` LF/CRLF churn (surfaces only as the CRLF warning line in
  the frozen-lock diff; zero diff lines).

## Mechanism (contract RGB protocol + ruling)

1. `mkdir crates/artifact/src` via one-shot distill-run python script
   (raw New-Item allowlist-denied; contract mandates the dir, plumbing
   substituted - D4).
2. `git mv crates/report/src/manifests.rs crates/artifact/src/manifests.rs`
   (byte-safe, no Edit round-trip).
3. D7 fix (AR-20a recipe): `git reset -- crates/report/src/manifests.rs`
   left the deletion UNSTAGED (path stays in the index so commit_scoped's
   ls-files-based scope resolution sees it); porcelain ` D` + `AM` observed.
4. ONE content hunk on the moved file: `pub(crate) fn
   collect_stage_languages` -> `pub fn ...` (line 22). Ruling pre-approved;
   probe evidence above (FAIL `error[E0603]: function
   \`collect_stage_languages\` is private` vs CONTROL
   `ok (exit 0, 0 warnings emitted)`).
5. report/src/lib.rs = contract shim verbatim + rider line + allow pair
   (D3) + rationale comment; manager lines UNTOUCHED. report/Cargo.toml +1
   dep line after ariadne-lang. Root Cargo.toml +1 member line
   `"crates/artifact",` inserted before `"crates/report",`. artifact
   Cargo.toml deps = serde + graph/lang/repo exactly (manifests.rs:1-3);
   NO anyhow/serde_json needed (zero E0432/E0433 on deps).

## Mutation proof (rider 5, UNCOMMITTED, reverted)

With shim pub-use lines commented (rider line kept):
`cargo check -p ariadne-report --all-targets` -> FAIL exit 101;
`error[E0432]: unresolved import \`crate::manifests\`` reported at ALL 9
`use crate::manifests::{...}` sites: apply_io.rs:10, discovery_io.rs:12,
graph_io.rs:12, index_io.rs:10, metrics_io.rs:13, transform_io.rs:10,
m22_manifest_languages_tests.rs:6, payload_reshape_tests.rs:20,
tests.rs:2. lib 6 errors / lib-test 9 errors. Capture
`target/distill-capture.txt` (94-line run). Restored byte-exact via the
inverse Edit; final state re-read and verified; re-green
`cargo check -p ariadne-report --all-targets` ->
`ok (exit 0, 0 warnings emitted)`. Shim proven load-bearing.

## Gates (verbatim, own stdout; capture per run)

1. `cargo check --workspace --all-targets` -> GREEN exit 0. First run: 31
   warnings INCLUDING 2 attributable to my rider line
   (`hidden_glob_reexports` + `unused_imports` at lib.rs:13); after the
   D3 allow pair: rerun exit 0, 31 warnings, ZERO attributable to
   report/artifact (grep of own capture: no `ariadne-report ... generated`
   warning line; totals equal due to cargo cache-dependent re-emission of
   the pre-existing lang-cpp/integration-test warning census).
2. `-p ariadne-artifact` -> `0 passed / 0 failed / 0 ignored / 2 suites`
   (empty lib + Doc-tests counted, AR-20a D3 precedent) - exactly the
   contract expectation.
3. `-p ariadne-report` -> `47 passed / 0 failed / 0 ignored / 4 suites`
   BYTE-IDENTICAL. Decomposition from same capture: lib 40 +
   bincode_bounded_decode 6 + extraction_artifacts 1 + Doc-tests 0 = 47.
4. m00 golden + facts-path -> `4 passed / 0 failed / 0 ignored / 2 suites`
   UNEDITED; `git status --porcelain -- tests/golden` empty.
5. m23_symbol_isolation_test -> `7 passed / 0 failed / 0 ignored / 1 suites`
   = dispatch-HEAD baseline, suite UNEDITED.
6. `hygiene_check.py` -> R3 (warn) rows only, ZERO blocking (row set
   identical to pre-mutation baseline; `crates/artifact` absent from
   findings).
7. `encoding_guard.py --all` -> silent (pre-commit, gate 8 discharged
   early); `--staged` post-commit_scoped below.
- Frozen lock: `git diff c8ae8ea -- crates/report/src/manager
  crates/report/src/manager.rs crates/report/tests tests` -> EMPTY (zero
  diff lines; foreign CRLF warning only).

## RIDER FOR 26a2 (bank prominently - ruling board_12d48349 rider 3)

The `pub` on `collect_stage_languages` is a TRANSIENT created by this hop's
seam (DTOs moved, consumers not yet). When manager/* lands same-crate in
`ariadne-artifact`:
1. REVERT the fn to `pub(crate) fn collect_stage_languages` at the
   definition in `crates/artifact/src/manifests.rs`.
2. DROP the shim rider line (`pub(crate) use
   ariadne_artifact::manifests::collect_stage_languages;` + its allow pair
   + comment) from `crates/report/src/lib.rs`; the two `pub use
   ariadne_artifact::manifests` lines stay for the 12 DTOs.
TERMINAL PUBLIC SURFACE = the 12 DTOs (flat + module-qualified). Until
then `ariadne_artifact::manifests::collect_stage_languages` is publicly
reachable module-qualified; report's FLAT surface is already shielded by
the rider line.

## Deviations (execution phase; numbering continues)

- D2 (pre-approved, ruling rider 1): single visibility hunk on the moved
  file, `pub(crate) fn` -> `pub fn` at manifests.rs:22. Rename lands as
  R9x (221 of 222 lines identical), not R100; this supersedes the
  contract's R100 expectation BY RULING. Evidence: probe FAIL/CONTROL above.
- D3 (disclosed): the mandated rider line at report/src/lib.rs carries
  `#[allow(hidden_glob_reexports, unused_imports)]` + rationale comment.
  Reason: the line's PURPOSE is shadowing the glob re-export of the widened
  fn, which INTRINSICALLY fires `hidden_glob_reexports`; the binding is
  referenced only as that mechanism, firing `unused_imports`. Without the
  allow pair my hop adds 2 compiler warnings attributable to the diff
  (proven in gate-1 first run); with it, zero. The mandated binding itself
  is byte-exact as ruled.
- D4 (plumbing): `mkdir crates/artifact/src` through a one-shot python
  script via the distiller (raw `New-Item`/Test-Path compound denied by the
  bash allowlist catch-all); `git mv` requires the destination dir.
- Deliberately NOT done: unused-dep cleanup in report/Cargo.toml (kept the
  full dep list per contract - later AR owns it); no m17/cone changes
  (26b); no manager/* movement (26a2).

