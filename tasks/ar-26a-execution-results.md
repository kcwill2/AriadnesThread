# AR-26a Execution Results - Reviewer Acceptance Pass (075e3bf, 2026-10-11)

Reviewer session `ses_ed40c8829ffe2vNIVu0ut32TIn`. All gate lines below are
from my own stdout per the capture-race ruling; capture paths quoted where
the run wrote one. Tree at review start: HEAD `075e3bf`, `git diff HEAD`
clean (checkpoint_guard: "DIRTY FILES ... (none)"), untracked = known ar-*
docs only, no `crates/**` untracked. Only live lane.

## Ground truth (step 1)

`python tools/checkpoint_guard.py status` -> HEAD `075e3bf`, branch
`shining-enthusiasm`, dirty (tracked) none. Range `c8ae8ea..HEAD`:

```text
075e3bf docs(ar-26a): landing addendum - commit 4794467, R099 rename-completeness proof, gate record
4794467 refactor(artifact): new ariadne-artifact crate; move manifests with ruling-preapproved visibility widening; report re-export shim (AR-26a, ruling board_12d48349)
2fb9b21 docs(ar-26a): addendum - commit hash record 5b7c53d and encoding proof
5b7c53d docs(ar-26a): BLOCKER evidence - collect_stage_languages pub(crate) breaks the verbatim-move seam; E0603 probe + control, options A/B/C, zero mutations
957c391 docs(tasks): AR-26b OPEN ITEM resolved by shape ruling - Option A cone lock, P1 exact-token tree predicate, P2 expected cone members, P3 deferred-row disposition
```

`957c391` is the planner's docs commit (adds `tasks/ar-26a2-handoff.md`,
`tasks/ar-26b-handoff.md`, edits `tasks/ar-26-handoff.md`; 564 insertions,
docs only) - outside this handoff's adjudication scope, acknowledged in the
review dispatch. `5b7c53d`/`2fb9b21`/`075e3bf` are journal-only per
`git show --stat` (104+/8+/30+ to `tasks/ar-26a-handoff-journal.md`).

## Landing commit `4794467` - per-commit review + touch-set

`git show --stat -M "4794467"` (own run):

```text
 Cargo.lock                                   |  11 +++
 Cargo.toml                                   |   1 +
 crates/artifact/Cargo.toml                   |  11 +++
 crates/artifact/src/lib.rs                   |   7 ++
 crates/{report => artifact}/src/manifests.rs |   2 +-
 crates/report/Cargo.toml                     |   1 +
 crates/report/src/lib.rs                     |  17 +++-
 tasks/ar-26a-handoff-journal.md              | 125 ++++++++++++++++++++++++++-
```

All 8 paths inside the declared touch-set; no others.
`git show --name-status -M "4794467"` -> `R099 crates/report/src/manifests.rs
crates/artifact/src/manifests.rs` plus M/A entries for the 7 other paths;
deletion+addition landed in the ONE commit (no stray-deletion follow-up).
`crates/{report => artifact}/src/manifests.rs | 2 +-` reproduced verbatim.
Journal numstat `124+1-` consistent with stat's 125 change-lines.

Content verified from `git show "4794467"`:
- root `Cargo.toml`: `+    "crates/artifact",` inserted before
  `"crates/report",` (contract: +1 members line) - MATCH.
- `crates/artifact/Cargo.toml` (NEW, 11 lines): package block mirrors
  `crates/report/Cargo.toml:1-5` (`[package]`/name/version/edition/authors
  workspace fields); deps exactly `serde.workspace`, `ariadne-graph`,
  `ariadne-lang`, `ariadne-repo` - no `anyhow`/`serde_json` (none needed,
  zero E0432/E0433) - MATCH.
- `crates/artifact/src/lib.rs` (NEW, 7 lines): crate doc + `pub mod
  manifests;` + `pub use manifests::*;` - MATCH contract (~6 lines + doc).
- `crates/report/Cargo.toml`: `+ariadne-artifact = { path = "../artifact" }`
  after `ariadne-lang` line; full dep list kept - MATCH.
- `crates/report/src/lib.rs`: contract shim byte-present - doc comment text
  matches the contract's block, `pub mod manager;` / `pub use manager::*;`
  UNTOUCHED, the exact pair `pub use ariadne_artifact::manifests;` /
  `pub use ariadne_artifact::manifests::*;`, then the ruling rider line
  `pub(crate) use ariadne_artifact::manifests::collect_stage_languages;`
  with `#[allow(hidden_glob_reexports, unused_imports)]` + 6-line rationale
  comment (deviation D3, see adjudication 2).

## Adjudication 1 - D2 widening scope (VERIFIED)

Independent old-vs-new blob diff:
`git diff "c8ae8ea:crates/report/src/manifests.rs" "HEAD:crates/artifact/src/manifests.rs"`

```text
@@ -19,7 +19,7 @@ use std::path::PathBuf;
 /// parameter is generic over `Into<LanguageId>` because stages still mix closed
 /// file-kind labels (discovery, build contexts) with open plugin identities
 /// (include edges).
-pub(crate) fn collect_stage_languages(
+pub fn collect_stage_languages(
     languages: impl IntoIterator<Item = impl Into<LanguageId>>,
 ) -> Vec<LanguageId> {
```

EXACTLY one hunk, one line, at the fn definition (file line 22, hunk header
@@ -19,7 @@). A single-line blob diff is exhaustive: no other item's
visibility (or any other byte) differs. `git diff --find-renames -M --stat`
across the pair -> `crates/{report => artifact}/src/manifests.rs | 2 +-`,
1 insertion + 1 deletion. R099 consistent (221/222 lines identical).
Everything else R099-verbatim - CONFIRMED.

## Adjudication 2 - D3 allow pair (NECESSITY CONFIRMED; RIDER COMPLETE)

Necessity proved without mutating the tree: force-overrode the `allow`
attribute via rustc `--force-warn` (which beats any allow) against the
COMMITTED tree -
`python tools/distill.py -- cargo check -p ariadne-report --all-targets
--config "build.rustflags=['--force-warn=hidden_glob_reexports','--force-warn=unused_imports']"`
-> `ok (exit 0, 2 warnings emitted)`
(capture `target/distill-capture.txt`, 96-line run). Exactly the two
predicted lints fire, both AT the rider line:

```text
warning: private item shadows public glob re-export
  --> crates\report\src\lib.rs:16:16
...
note: the name `collect_stage_languages` in the value namespace is supposed to be publicly re-exported here
  --> crates\report\src\lib.rs:8:9
...
warning: unused import: `ariadne_artifact::manifests::collect_stage_languages`
  --> crates\report\src\lib.rs:16:16
```

(The capture's two other force-warn artifacts - `crossbeam-utils`, `syn` -
are registry-dep byproducts of the global force-warn flag, not tree warnings;
the normal gate-1 run shows zero warnings for them.) Without the allow pair
the hop intrinsically adds 2 warnings attributable to the diff, breaking
zero-warning gate 1; with it, zero. The binding itself is byte-exact as
ruled. Necessity claim ACCEPTED on independent evidence.

RIDER completeness: journal `## RIDER FOR 26a2` item 2 reads "DROP the shim
rider line (`pub(crate) use
ariadne_artifact::manifests::collect_stage_languages;` + its allow pair +
comment) from `crates/report/src/lib.rs`; the two `pub use
ariadne_artifact::manifests` lines stay for the 12 DTOs." - all three
components (line + allow pair + comment) are ENUMERATED for removal, plus
item 1 reverts the definition to `pub(crate)`. The 26a2 mandate is complete
in this hop's journal. (See finding 1: the 26a2 CONTRACT TEXT does not yet
carry it.)

## Adjudication 3 - mutation trail revert (COMPLETE)

Tree == `4794467`/`075e3bf` for all source: checkpoint_guard
`git diff HEAD --numstat` = "(none)"; Glob census finds exactly one
`crates/**/manifests.rs` (`crates/artifact/src/manifests.rs`) - no stale
file, no untracked `crates/**` entries; committed `crates/report/src/lib.rs`
carries BOTH shim `pub use` lines active (no residue of the commented
mutation). The 9-site E0432 trail claim is mechanically consistent: with
both `pub use ariadne_artifact::manifests...` lines commented the crate-root
`manifests` binding dies entirely, so all 9 `use crate::manifests::{...}`
sites fail - strictly wider than the 3 fn-importers, as the journal states.

## Adjudication 4 - report conservation + frozen locks (VERIFIED)

- `python tools/testrun.py --distill -p ariadne-report` ->
  `47 passed / 0 failed / 0 ignored / 4 suites` - BYTE-IDENTICAL to pin.
  Decomposition from own capture (`target/testrun-capture.txt`, 72-line run,
  tree quiet): lib unittests `40 passed`,
  `bincode_bounded_decode_test` `6 passed`,
  `extraction_artifacts_test` `1 passed`, `Doc-tests ariadne_report`
  `0 passed` = 47+0+0 - decomposition 40+6+1+0 REPRODUCED.
- Frozen-lock `git diff "c8ae8ea" "HEAD" -- crates/report/src/manager
  crates/report/src/manager.rs crates/report/tests tests` -> EMPTY (no
  output), `--stat` form also EMPTY. Rev-vs-rev form shows no CRLF warning
  line (churn is worktree-side only and tree is now clean vs HEAD anyway).
  All of `manager/**`, `report/tests/**`, `tests/**` byte-UNEDITED.
- Consumer imports therefore byte-UNEDITED at
  `crates/report/src/manager/discovery_io.rs:13`
  (`collect_stage_languages, BuildContextManifest, RepositoryManifest,`),
  `metrics_io.rs:13`
  (`use crate::manifests::{collect_stage_languages, LoadManifest, LoadReport, PlanManifest};`),
  `graph_io.rs:13`
  (`collect_stage_languages, CallGraphManifest, IncludeManifest, ReducedManifest,`).
- `python tools/testrun.py --distill -- -p ariadne-integration-tests
  --test m00_golden_baseline_test --test m00_facts_path_parity_test` ->
  `4 passed / 0 failed / 0 ignored / 2 suites` UNEDITED (goldens inside the
  empty frozen diff - `tests/golden` covered).

## Adjudication 5 - artifact hop-shape (VERIFIED)

- `python tools/testrun.py --distill -p ariadne-artifact` ->
  `0 passed / 0 failed / 0 ignored / 2 suites` (empty lib + Doc-tests) -
  exactly the contract expectation.
- m23: `python tools/testrun.py --distill -p ariadne-integration-tests
  --test m23_symbol_isolation_test` ->
  `7 passed / 0 failed / 0 ignored / 1 suites` = dispatch-HEAD baseline,
  suite UNEDITED (inside empty frozen diff).

## Gate lines (verbatim, own stdout)

| gate | command | result |
|---|---|---|
| compile-green | `python tools/distill.py -- cargo check --workspace --all-targets` | `ok (exit 0, 31 warnings emitted) -> capture: ...\target\distill-capture.txt` |
| - | grep own capture for `generated N warning` attribution | lines for `ariadne-lang-cpp`(lib test) / `ariadne-ui` / `ariadne-cli` / `ariadne-integration-tests` only - ZERO `ariadne-report` or `ariadne-artifact` warning lines |
| g2 | `python tools/testrun.py --distill -p ariadne-artifact` | `0 passed / 0 failed / 0 ignored / 2 suites` |
| g3 | `python tools/testrun.py --distill -p ariadne-report` | `47 passed / 0 failed / 0 ignored / 4 suites` |
| g4 | `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test` | `4 passed / 0 failed / 0 ignored / 2 suites` |
| g5 | `python tools/testrun.py --distill -p ariadne-integration-tests --test m23_symbol_isolation_test` | `7 passed / 0 failed / 0 ignored / 1 suites` |
| g6 | `python tools/hygiene_check.py` | R3 (warn) rows only; ZERO blocking rows; `crates/artifact` absent from findings (exit code not echoable - adjudicated on printed output: every printed row is R3 warn, per AR-01/02/03 precedent) |
| g7 | `python tools/encoding_guard.py --all` | silent (exit 0) |

## Workspace census - reconciliation and NEW anchor row

`python tools/testrun.py --distill -- --workspace` ->

```text
1398 passed / 0 failed / 13 ignored / 105 suites
```

(exit 0; capture `target/testrun-capture.txt`, 2677 lines)

Attribution table:

| step | delta | cumulative |
|---|---|---|
| `@AR-24 reviewer` anchor | - | `1393/0/13/102` |
| AR-25 (cli `ar25_*` locks; cli 114->118, +1 suite) | `+4/0/0/+1` | `1397/0/13/103` |
| AR-31 (lang-cpp tokenizer; 155->156, suites unchanged) | `+1/0/0/0` | `1398/0/13/103` |
| AR-26a (new member `ariadne-artifact`: empty lib + Doc-tests targets) | `+0/0/0/+2` | `1398/0/13/105` |
| MEASURED @ `075e3bf` | - | `1398/0/13/105` - MATCH |

Delta fully attributed; every step reconciled. NEW ANCHOR ROW:
`1398 passed / 0 failed / 13 ignored / 105 suites` at `075e3bf`.
Note: the handoff's projection line `1397/0/13/103` at dispatch HEAD was
stale by one test (it predates/omits AR-31's +1); superseded by this table.

## Tools suite

`python -m unittest discover -s tools -p "*_tests.py"` ->

```text
Ran 219 tests in 20.633s

OK
```

## Refusals recorded verbatim (all substituted, no gate skipped)

Catch-all refusal text on each (identical):
`The user has specified a rule which prevents you from using this specific tool call. Here are some of the relevant rules {"permission":"bash","pattern":"*","action":"deny","source":"global"}`

1. compound frozen-lock command
   (`git diff ...; if ($?) { git diff --stat ...; Write-Output ... }`) -
   substituted with two plain `git diff` rev-vs-rev invocations (both EMPTY).
2. `git ls-files crates/report/src/manifests.rs` - substituted: Glob census
   (exactly one `manifests.rs` on disk, artifact path) + `R099` rename
   status (proves tracked deletion in `4794467`) + clean tree with no
   untracked `crates/**` (no stale tracked or on-disk copy).
3. `git ls-tree -r --name-only HEAD crates/report/src` - same refusal, same
   substitution.

## NOT run (with authority)

- `python tools/testrun.py --baseline` - errata per dispatch: file-mode
  tool, no repo baseline file exists.
- `python tools/ui_smoke.py`, `python tools/bench_compare.py` - OUT of this
  contract per dispatch (quiet-tree bench is a closeout duty).
- Gate 8 `encoding_guard --staged` not re-runnable by reviewer (nothing
  staged; tree clean) - `--all` silent covers tracked content; implementer
  evidence for `--staged` runs recorded in journal.

## Report-evidence (distillation contract)

Journal is digest-shaped: canonical testrun lines, `ok (exit ...)` distill
verdicts, capture paths, zero raw dumps; all commands through
`testrun.py --distill` / `distill.py --`. Gate commands conform to
`docs/specs/tool-output-distillation.md`.
