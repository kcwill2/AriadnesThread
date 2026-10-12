# AR-26a Handoff - New `ariadne-artifact` crate + `manifests` move + report shim (A3 chain, hop 1/3)

Split of `tasks/ar-26-handoff.md` after its mandatory SPLIT CHECK STOP
(evidence journal `tasks/ar-26-handoff-journal.md`, commits
`e946f57`/`a634ae5`/`2679ebb`). That journal is binding input: its F1-F6
findings are numstat/grep proven - do NOT re-measure them.

## DISPATCH ORDER - AR-26 chain (read first)

Three SERIAL hops. Each hop dispatches only after the previous hop's
reviewer APPROVED and landed; each hop re-baselines its suite lines at its
own dispatch HEAD. The hops are NOT parallel-safe with each other: all
three touch `crates/report/**` plus the shared workspace manifests.
AR-31 (lang-cpp tokenizer lane) may run concurrently with any hop -
file-disjoint; the only shared file is `Cargo.lock` (rebase on fresh HEAD;
on lock collision regenerate, never hand-merge).

| hop | file | scope | produced lines | gates |
|---|---|---|---|---|
| 1 (this file) | `tasks/ar-26a-handoff.md` | new crate skeleton + `manifests.rs` move + shim | ~255 | 8 |
| 2 | `tasks/ar-26a2-handoff.md` | atomic `manager/*` move (11 files) + shim | 2879 moved (budget flag, justified there) | 10 |
| 3 | `tasks/ar-26b-handoff.md` | scheduler reparent + m17 cone-guard RED-first + cargo-tree lock | ~110 | 10 |

Per-hop reconciliation (what each hop must reconcile, spelled there too):
hop 1 - report suite stays `47 passed / 0 failed / 0 ignored / 4 suites`
byte-identical; artifact contributes `0 passed` targets. Hop 2 - passed
conservation `47 = 40 (artifact lib) + 7 (report tests)`; moved suites land
under `-p ariadne-artifact`. Hop 3 - scheduler `26` + worker-core `8`
conserved with ZERO test edits; m17 grows `8 -> 10`.

Why this split and not the implementer's candidate (journal section
"Split proposal"): the candidate 26a1 (manager.rs + 5 IO modules first,
graph_io/metrics_io second) is NOT compile-green at the seam - the 7 IO
submodules carry `impl ArtifactManager` blocks, and Rust rejects an
inherent impl for a type defined in another crate (E0116). All 7 IO
modules plus `manager.rs` are therefore ATOMIC; the 3 in-src test modules
are declared inside `manager.rs:15-20` and travel with it (E0583 otherwise;
docking their mod lines would be test edits, forbidden). Splitting what
remains means moving the leaf `manifests.rs` first via a path-preserving
re-export shim - the only seam that is independently shippable green.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ar-26-handoff-journal.md` (F1 manager.rs contents, F2 manifests
   coupling, F3 root-only consumer surface, F4 consumer census).
3. `tasks/ar-20a-handoff.md` (worked example of exactly this move+shim
   shape: shim rule, rename protocol, conservation equation).
4. `crates/report/src/lib.rs` (7 lines - the file you rewrite),
   `crates/report/Cargo.toml` (dep-list style to mirror), root `Cargo.toml`
   (members list).

## Objective

Create the new workspace crate `ariadne-artifact` (`crates/artifact`) and
move `crates/report/src/manifests.rs` (222 lines) into it VERBATIM via a
byte-safe rename. `report` keeps every public path resolving through a
2-line re-export shim so ALL 22 consumer files plus report's own 3 in-src
test modules compile UNEDITED. This hop moves NO manager code and NO
tests: it de-risks the chain by landing the new crate member, the shim
mechanism, and the Cargo topology while `report`'s suite line stays
byte-identical (47/0/0/4). Invariants preserved: bincode wire bytes
byte-identical (nothing serialization-relevant moves this hop); zero test
edits anywhere; scheduler cone unchanged (still `scheduler -> report`).

## Baselines (dispatch-HEAD 2026-10-11; re-measure and journal BEFORE any
mutation; contracts must not quote stale numbers)

- report `47 passed / 0 failed / 0 ignored / 4 suites` (decomposition
  measured 2026-10-11: lib 40 + bincode_bounded_decode 6 +
  extraction_artifacts 1 + Doc-tests 0).
- scheduler `26/0/0/2`; ariadne-worker-core `8/0/0/2`; m17 cone-guard
  `8/0/0/1`; m00 golden + facts-path `4/0/0/2`; cli `118/0/8/7`;
  lang-cpp `155/0/0/2`.
- `python tools/hygiene_check.py` ZERO blocking rows (never quote an R3
  count).
- workspace census projection at current HEAD `1397/0/13/103` (reviewer's
  acceptance pass, not an implementer gate).

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/ar-26a-handoff-journal.md` (checkpoint journal; always in-set)
- `crates/artifact/Cargo.toml` (NEW - package block mirrors
  `crates/report/Cargo.toml:1-5`; deps: `serde.workspace = true`,
  `ariadne-graph = { path = "../graph" }`, `ariadne-lang = { path = "../lang" }`,
  `ariadne-repo = { path = "../repo" }` - these are the crate imports of
  `manifests.rs:1-3`; add `anyhow`/`serde_json` ONLY if E0432/E0433 proves
  the need, journal each addition)
- `crates/artifact/src/lib.rs` (NEW - `pub mod manifests;` +
  `pub use manifests::*;` + crate doc, ~6 lines)
- `crates/artifact/src/manifests.rs` (MOVED VERBATIM via `git mv` - see
  RGB protocol; byte-identical, zero content hunks)
- `crates/report/src/lib.rs` (replace `pub mod manifests;` +
  `pub use manifests::*;` with the shim, see exact shim below; the two
  `manager` lines UNTOUCHED this hop)
- `crates/report/Cargo.toml` (+1 line `ariadne-artifact = { path = "../artifact" }`;
  everything else UNTOUCHED - keep the full dep list even where hop 2
  orphans it; unused-dep cleanup is a later AR, journal the note)
- workspace `Cargo.toml` (+1 members line `"crates/artifact"`)
- `Cargo.lock` (cargo regenerates for the new member)

You MUST NOT touch: `crates/report/src/manager.rs`,
`crates/report/src/manager/**`, `crates/report/tests/**`,
`crates/scheduler/**`, `crates/lang-cpp/**` (AR-31 live lane),
`tests/**` (every suite and golden UNEDITED), `tasks/ar-26-handoff.md`,
`tasks/ar-26-handoff-journal.md`. No `git stash`, no branch change.

## The exact shim (compat contract)

`crates/report/src/lib.rs` after this hop, in full:

```rust
//! Artifact generation and load reporting (AR-26a: manifest DTOs live in
//! ariadne-artifact; paths preserved).

pub mod manager;
pub use manager::*;

pub use ariadne_artifact::manifests;
pub use ariadne_artifact::manifests::*;
```

Mechanism proof (AR-20a shim rule, verified there): a root-level `pub use`
binding is path-resolvable in-crate, so the IO modules' and tests'
`use crate::manifests::{...}` lines (`apply_io.rs:10`, `discovery_io.rs:12`,
`graph_io.rs:12`, `transform_io.rs:10`, `tests.rs:2`, plus `metrics_io.rs:13`,
`index_io.rs:10`, `m22_manifest_languages_tests.rs:6`,
`payload_reshape_tests.rs:20`) keep compiling UNEDITED. Consumer paths:
module-qualified `ariadne_report::manifests::X` resolves via line 1 of the
pair; flat `ariadne_report::LoadReport` via line 2 (F4: all 22 external
consumers use root-only flat paths - zero module-qualified
`report::manifests` or `report::manager` references exist in the tree;
doc-comment mentions in `crates/graph` are comments, not paths).

## RGB protocol (extraction hop - RED-first is NOT mandated here)

Per the chain rule: extraction hops are behavior-free reparents; the
byte-locks are their regression, and RED-first applies only in AR-26b's
cone guard. Falsifiability evidence instead (AR-20a precedent, NEVER
committed):

1. Journal dispatch HEAD + all baseline lines above BEFORE touching
   anything.
2. Byte-safe move: `git mv crates/report/src/manifests.rs crates/artifact/src/manifests.rs`
   (create `crates/artifact/src` first). NEVER recreate the file through
   Edit-tool round-trips (core.autocrlf LF-slip precedent, AR-20a).
3. Shim load-bearingness mutation proof: with the shim in place, comment
   out BOTH `pub use ariadne_artifact::manifests...` lines, run
   `python tools/distill.py -- cargo check -p ariadne-report --all-targets`,
   capture the expected failure - `error[E0432]`/`error[E0433]` cannot find
   `manifests` in the crate root, reported at the io-module import lines -
   then restore the lines byte-exact (`git diff` on `crates/report/src/lib.rs`
   vs your intended final state is empty). This proves the shim is not
   decorative.
4. Green: all gates below. 5. Refactor: NONE on the moved file (verbatim is
   the contract); hygiene review applies only to the new lib.rs/Cargo.toml.

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned names, meaningful
names throughout. The R2 ceiling at risk is the NEW `crates/artifact/src/lib.rs`
(tiny, keep it honest with a doc comment stating the crate owns the
artifact store IO surface) - the moved `manifests.rs` (222 lines) cannot
trip any ceiling at its new home. `python tools/hygiene_check.py` exit 0
with ZERO blocking rows (post-AR-13 baseline; do not re-tolerate rows).

## Frozen locks (verify + quote)

- `python tools/distill.py -- git diff <dispatch-HEAD> -- crates/report/src/manager crates/report/src/manager.rs crates/report/tests tests` -> EMPTY (manager, report test files, all integration suites and goldens untouched this hop).
- `git status --porcelain -- tests/golden` empty.
- bincode wire bytes: byte-identical by construction - nothing
  serialization-relevant moves; the m00/m23/bincode lock suites prove it
  green UNEDITED.
- Until `golden_guard.py` lands: manual byte-identical checks of the
  above paths - do NOT touch them.

## Gates (exact commands, timeout 600000 ms each; scoped only - workspace-wide
final gates belong to the reviewer's acceptance pass)

1. `python tools/distill.py -- cargo check --workspace --all-targets`
   -- GREEN first (STRICT COMPILE-GREEN; also proves all 22 consumers
   compile UNEDITED through the shim).
2. `python tools/testrun.py --distill -p ariadne-artifact` -- expect
   `0 passed / 0 failed / 0 ignored / 2 suites` (empty lib target + empty
   Doc-tests target; quote verbatim, journal if testrun's suite census
   differs - passed-count conservation is the authority).
3. `python tools/testrun.py --distill -p ariadne-report` -- `47 passed /
   0 failed / 0 ignored / 4 suites` BYTE-IDENTICAL to baseline (no test
   moved this hop).
4. `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_golden_baseline_test --test m00_facts_path_parity_test` -- `4 passed / 0 failed / 0 ignored / 2 suites` UNEDITED + `git status --porcelain -- tests/golden` empty.
5. `python tools/testrun.py --distill -p ariadne-integration-tests --test m23_symbol_isolation_test` -- matches the baseline line you journalled at dispatch HEAD, suite UNEDITED.
6. `python tools/hygiene_check.py` -- exit 0, zero blocking rows.
7. `python tools/encoding_guard.py --staged` -- exit 0 before committing.
8. `python tools/encoding_guard.py --all` -- exit 0, journal the run.

Commit:
`python tools/commit_scoped.py --scope "crates/artifact/**" --scope "crates/report/**" --scope "Cargo.toml" --scope "Cargo.lock" --scope "tasks/ar-26a-handoff-journal.md" -m "refactor(artifact): new ariadne-artifact crate; move manifests verbatim with report re-export shim (AR-26a)"`
(never `git add .`).

## Rename-completeness proof clause (AR-20a D7 - mandatory)

`commit_scoped.py` cannot see staged `git mv` deletes. Immediately after
the commit run, via distiller, `git show --stat -M HEAD` and
`git ls-files crates/report/src/manifests.rs`; the stat must list the OLD
path as deleted and the NEW path as added (or as an R100 rename), with no
other paths; `git ls-files` must return NOTHING for the old path (no stale
tracked copy). If a delete fell out of the scoped commit (the D7 hazard),
land an immediate second scoped commit for the stray deletion ONLY and
journal it. A `git show --stat` carrying paths outside the touch-set is a
self-declared blocker: report, do not amend silently.

## Discipline clauses

- UTF-8: never Set-Content/Out-File/>/Get-Content-replace; Write/Edit tools
  only; authored content pure ASCII (AGENTS.md File-Edit Discipline).
- VERBATIM only: the moved `manifests.rs` carries zero content hunks
  (`git show --stat -M` rename with no line counts; cheap journal evidence:
  `git diff <dispatch-HEAD> --find-renames -M` shows R100).
- Touch only the touch-set; on discovering out-of-scope breakage: STOP,
  report, do not fix.
- No raw high-volume commands in your context: everything through the
  distilling entry points (`docs/specs/tool-output-distillation.md`); quote
  canonical summary lines from OWN stdout, never from shared capture files
  (capture-race ruling).
- After EACH Green gate, append a 1-3 line checkpoint to
  `tasks/ar-26a-handoff-journal.md` (module + commit hash, deviations,
  lock/contract findings). If context exhaustion looms: STOP, return a
  disk-state inventory; a dead session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ar-26a-handoff.md
  --scope ... --cmd ...`.
- Serialization: this hop runs alone; hop 2 dispatches only after this is
  APPROVED and landed. AR-31 (lang-cpp) may run concurrently; on a
  `Cargo.lock` conflict rebase and let cargo regenerate.

## Report back (final message)

1. Commit hash + `git show --stat -M` (distilled) + the D7 proof outputs.
2. Baseline lines (pre-mutation) and the mutation-proof failure capture
   (command + expected `E0432`/`E0433` + restore confirmation).
3. Green evidence: gates 2-5 canonical lines verbatim from own stdout +
   capture paths. 4. Any deviation + reason. 5. Out-of-scope observations
   (report-only). Never paste raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill (fresh full verification, per-commit `git show` oldest-first,
commit-vs-touch-set enforcement against the list above - it includes
`Cargo.lock` precisely because cargo regenerates it).
