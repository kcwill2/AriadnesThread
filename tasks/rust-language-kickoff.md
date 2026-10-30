# rust-language - Milestone Kickoff (2026-10-25)

Track key: `rust-language`. UNREGISTERED post-M28 change track - NO M-number
(M29+ appear nowhere as ledger registrations; `tasks/milestone_ledger.md`
stays numbering authority). Registration precedent:
`ui-hotspot-canvas-direction` / `extraction-symbol-move` live-row pattern.

Authored by the architect hop 2026-10-25 on branch `balsam-basil`
(start HEAD `f235c8a`, tree clean at authoring). Census start anchor
`1853 passed / 0 failed / 13 ignored / 119 suites @ 522a770`
(the `extraction-symbol-move` closeout anchor, per the ledger).

Ground-truth note (E16 class, honest disclosure): `python
tools/checkpoint_guard.py status` could not be run by this session (the
architect role has no shell). The clean-tree statement above is the dispatch
brief's, corroborated by the read-only `git status` this session ran
(`nothing to commit, working tree clean`). The landing hop (sec. 8) runs
`checkpoint_guard status` before committing and owns any worktree-census
surprise.

## 1. Objective

Add Rust as the THIRD language plugin (after C++ and Scala), at full
capability parity with Scala today - discovery, dependency_graph,
semantic_index, load_metric, planning, extraction, rewriting, validation -
through the existing plugin seam, in ONE milestone of a serial handoff arc
(Scala took six milestones; every seam Scala needed now exists, so the arc
is ~5-7 handoffs instead).

Blast radius: one new plugin crate `crates/lang-rust`, a handful of
guard-legal core-spine edits (FileKind table, from_file_kind arm, planning
vocabulary variant), one registration line, the CLI `--lang` closed set,
four named enumeration-suite amendments, one new fixture, and new
milestone-named integration suites. Zero edits to index/graph/cluster/
transform/validate/report/artifact/scheduler/ui production code (the m17
neutrality guard enforces the boundary; the registry builder stays the one
named composition root).

Underlying why: the project's core intent is multi-language support through
a plugin mechanism (saved decision `architecture.multilanguage_plugin_intent`),
and rule R1 ("a foreign language can be added without friction") has never
yet been exercised by a third language. Rust is the proof case the guide's
own honest-limits section says is missing.

## 2. User decisions of record (2026-10-25 - do not revisit)

D-1 (2026-10-25) FULL PARITY ARC. All 8 capabilities like Scala today:
discovery, dependency_graph, semantic_index, load_metric, planning,
extraction, rewriting, validation. One milestone, serial handoff arc.

D-2 (2026-10-25) SYN-CRATE PARSER. The user rejected heuristic-only
correctness: `lang-rust` parses Rust with `syn` + `proc-macro2` (the
proc-macro2 `span-locations` feature MUST be enabled - line/column are only
obtainable with it) + `unicode-ident`. In-process, offline: the pipeline
never invokes a compile (assumption A3 holds). The validator spawns only
`cargo --version` / `rustc --version` probes, mirroring
`ScalaToolchainValidator` (offline -> `ToolchainStatus::Missing` -> tier 3
`Skipped`). Honest limit recorded as `analysis_caveat`: macro-generated
module structure (`include!` / `macro_rules!`-generated `mod`) is not
expanded.

D-3 (2026-10-25) `.RS` COLLISION ACCEPTED, fixtures-only for tests. Once
the spine `FileKind` maps `.rs`, scanning THIS repo indexes its own Rust
tree. Accepted by the user. The test surface is a
`tests/fixtures/rust-basic/` fixture only - no test in this milestone
scans the repo's own Rust sources. Bench posture (sec. 5): benches run
against the existing corpora; no bench baseline change; if a load-perf gate
ever scans the own tree, that must be a deliberate decision, not an
accident of the extension flip.

## 3. Handoff arc (pointer)

The shape doc `tasks/rust-language-shape.md` holds the change shape and the
R-1..R-6 arc sketch. The planner authors and sizes the handoffs
(`handoff-spec` skill); nothing here pre-empts that. Each sketch hand must
respect the planning law: <= 1 crate/module dir, <= ~10 scoped gate
commands, <= ~1200 produced lines.

## 4. Acceptance commands (canonical forms)

Workspace census at every reviewer hop (the `--baseline` FILE form stays
superseded - M23.5 F9 standing errata; the `--distill` substitute governs,
identical count semantics):

- `python tools/testrun.py --distill` (whole workspace; capture path
  printed by the digest)
- scoped per-hand forms per the handoff (e.g. `python tools/testrun.py
  --distill -p ariadne-lang-rust`, `--test <suite>`)
- `python -m unittest discover -s tools` (CI form; expect `Ran 296 ... OK`)
- `python tools/ui_smoke.py` (or the sanctioned `python tools/distill.py --
  python tools/ui_smoke.py` route for subagent roles) - expect
  `ui_smoke: PASS`
- `python tools/hygiene_check.py` - exit 0 (R1/R2 blocking; R3 warn-only)
- `python tools/encoding_guard.py --all` - exit 0 (or `--staged` where the
  staging seam permits; `commit_scoped.py` owns staging)
- m00 parity UNEDITED: the `m00_facts_path_parity` suite green byte-lock
- m17 seam suite + m18 C++ parity green UNEDITED (except the two named
  `PLUGIN_CRATES` appends, sec. 5 lock L6)

Bench posture: `python tools/bench_compare.py` RUN fresh at every reviewer
hop that touches any `crates/*/src` byte (fully-run stance, the
`f20ad35`-onward precedent; the `--distill` route for subagent roles),
assets/docs-only hands exempt with a zero-`crates/*/src`-diff proof.
`bench_baseline.json` is operator-only (PR-2 per-machine row unchanged):
this milestone changes NO baseline and NO bench corpus. The `.rs` flip does
not re-baseline anything because benches/bench-adjacent gates keep running
on the existing corpora (decision D-3 posture, sec. 2).

Census expectation math: start `1853 / 0 / 13 / 119 @ 522a770`. Milestone
expectation: **>= 1853 passed, 0 failed at every hop; every delta
attributed per hand; zero foreign residue.** Per-hand placeholders the
planner fills into the ledger row as the chain lands:

- R-1: +<n1> (spine + crate + registration + fixture + discovery suite)
- R-2: +<n2> (syn parser + analyzer + fingerprint)
- R-3: +<n3> (resolver + graph edges + semantic-index acceptance)
- R-4: +<n4> (load metric + manifest/CLI vocabulary proofs + enumeration
  amendments)
- R-5: +<n5> (planning hooks + extractor + rewriter + validator +
  capability flip with byte-exact literal lock)
- R-6: +<n6> (full-parity acceptance suite + docs rider amendments)

Suite-count growth is expected (new suites; the 119 may rise - new
integration test files add suites; ignored 13 stays unless a hand carries a
bench-ignored test). Every rise is attributed by name at its hop.

## 5. Frozen locks (inherited; commands that prove each)

| # | Lock | Proof command |
|---|---|---|
| L1 | `ar27_*` byte-lock suites + `WireLanguageForm` frozen indices / `legacy_rank` / transparent-frame code UNEDITED (identity.rs:149-231, 235-242). The open-id escape (bincode tag 3 + length-prefixed UTF-8) already exists; Rust rides it, nothing near it moves. | census scoped runs of the `ar27_*` suites green UNEDITED; `git diff <start>..HEAD -- crates/repo/src/identity.rs` shows ONLY the `from_file_kind` arm + the new `Rust` const/`RUST` name const |
| L2 | `tests/golden/**` byte-locked; `UPDATE_GOLDENS=1` never run on this track | `git diff --stat <start>..HEAD -- tests/golden` EMPTY |
| L3 | `tests/fixtures/**` frozen EXCEPT the new `tests/fixtures/rust-basic/` added via the documented approval path (reviewer-approved commit + dated ledger note, the M27 T0b precedent); `tests/fixtures/basic` and `expensive` get NO new files | fixture-path-scoped diff shows only `rust-basic` additions |
| L4 | m00 parity UNEDITED and green every hop | `python tools/testrun.py --distill --test m00_facts_path_parity_test` (per handoff; suite name per tree) 2 passed |
| L5 | Scala suites (`m19_scala_*`, `m20_*`, `m21_*`, `m22_*`, `m23_*` + `crates/lang-scala`) UNEDITED except the ONE named amendment: `m19_scala_discovery_test.rs:217` enum `vec![Cpp, Scala]` -> `[Cpp, Scala, Rust]` (a registration fact, not a Scala behavior change) | per-path diffs EMPTY except the :217 hunk |
| L6 | m17/m18 UNEDITED except the two named `PLUGIN_CRATES` appends in `m17_plugin_seam_test.rs` (:271 and :486, both `[&str; 2]` -> `[&str; 3]` adding `crates/lang-rust`) and the m18 parity suite byte-identical | path-scoped diffs show only those two hunks / m18 diff EMPTY |
| L7 | C++ zero-diff: no byte under `crates/lang-cpp/` changes | `git diff <start>..HEAD -- crates/lang-cpp` EMPTY |
| L8 | Scala zero-diff: no byte under `crates/lang-scala/` changes (the third plugin must not perturb the second) | `git diff <start>..HEAD -- crates/lang-scala` EMPTY |
| L9 | `tools/**` zero-diff (implementers/reviewers are barred there by law; `tools/theme_token_baseline.json` untouched) | `git diff <start>..HEAD -- tools` EMPTY |
| L10 | Theme tokens n/a: zero UI asset/hex bytes touched | `python tools/theme_token_check.py --check-added --base <rev>` silent |
| L11 | `FileKind` / `AggregationKind` serde-append-only: `RustSource` appended AFTER `Other`; a new aggregation variant (shape decision) appended at the END - pre-existing variant indices do not move, so warm bincode blobs and JSON for C++/Scala keep bytes | `ar27_*` + L2 + L4 all green UNEDITED; enum diffs are pure appends |
| L12 | UTF-8: authored content pure ASCII, no PowerShell redirection writes | `python tools/encoding_guard.py --all` exit 0 |

Named amendment set for CLI in-src enumeration/behavior tests (each is a
lock AMENDMENT justified by the registration flip, per the CW-1
amendment-discipline precedent; the planner routes each into its handoff):

- A1 `crates/cli/src/registry.rs:302-318`
  `default_registry_enums_cpp_then_scala` -> cpp, scala, rust (name stays;
  the enum literal moves - the suite name is a name, not a contract)
- A2 `crates/cli/src/registry.rs:367-372`
  `parse_language_arg_rejects_unknown_values` - `"rust"` moves from the
  rejected list to the accepted list (the error text at :48-51 and the
  closed-set fn at :59-70 gain rust; test name at :383 `pin_parse_...exact_
  accepted_set` gains "rust"/"Rust" accepted rows)
- A3 `crates/cli/src/registry.rs:406-439`
  `pin_plan_backend_honest_errors_...` - its witness path
  `src/unclaimed.rs` is FALSE post-registration (a plugin claims `.rs` now);
  the unclaimed-target witness must move to a still-unclaimed extension
  (`.f90`, per the m24 suite's mock-vocabulary) - this site was MISSING from
  the dispatch brief's enumeration list; found by read-verification
- A4 `crates/cli/src/registry.rs:736-760`
  `planning_hooks_answer_honestly_for_paths_no_plugin_claims` - loop list
  contains `"src/unclaimed.rs"`; same remedy as A3 (also found by read
  verification, not in the brief's list)
- A5 `crates/repo/src/types.rs:103` - the matrix pin `("a.rs",
  FileKind::Other)` flips to `FileKind::RustSource` (the brief's own
  inventory: this amendment was known)
- A6 `crates/lang-api/src/identity.rs:34-51`
  `language_of_file_kind_maps_every_file_kind_exhaustively` - add the
  `(FileKind::RustSource, LanguageId::Rust)` row
- A7 `tests/integration/tests/m19_scala_discovery_test.rs:217` -
  `vec![Cpp, Scala]` -> `vec![Cpp, Scala, Rust]` (see L5)
- A8 `tests/integration/tests/m17_plugin_seam_test.rs:271 + :486` -
  `PLUGIN_CRATES` appends (see L6; both appends land in the SAME commit
  that adds `crates/lang-rust/Cargo.toml`, since both guards self-skip a
  missing manifest)

## 6. Inherited open rows (carried, not now)

From `extraction-symbol-move` (CLOSED 2026-10-24), per its closeout
re-registration. None is ingressed by `rust-language`; each is carried with
a one-line reason:

- Marked-region purge command follow-on - CARRIED, NOT NOW: it is a
  transform/CLI verb decision orthogonal to language plugins; rust-language
  stays additive.
- Generation-swap relay (deferred row "A-4 observations" half) -
  CARRIED, NOT NOW: record-not-execute relay; its trigger is the next
  `extraction_apply.rs` test touch, which this milestone does not perform.
- Include-closure / compilability of rewritten consumers + directive-spelling
  display (D-M23-12 deferral) - CARRIED, NOT NOW: no tier compiles, this
  track does not change that; language-plugin work must not smuggle a
  compilability change.
- Also open per the ledger, not re-litigated here: `optimize-verb move
  parity` (tripwire = the m19 deferral lock) and the ceiling watches; the
  ledger rows govern, this kickoff neither ingresses nor discharges them.

## 7. Graduation backlog audit (D4 - every warn-only row gets a disposition)

| Check | Disposition for rust-language |
|---|---|
| `hygiene_check.py` R3 naming (warn-only, allow-listed legacy) | STAYS warn-only. Reason: the flip condition is "one full milestone with zero new R3 findings"; the new `crates/lang-rust` code is R3-clean by birth requirement and the touched CLI/repo files carry no allow-list entries this track grows - so THIS milestone supplies the flip EVIDENCE, measured at closeout, for the NEXT kickoff to act on. Pre-committing a flip now would flip mid-evidence. No dated flip row; closeout carries the zero-new-R3 measurement. |
| clippy `-D warnings` (warn-only; 125-warning baseline at `f7ace60` per ci.yml comment) | STAYS warn-only. Reason: flip condition is baseline-reaching-0; no row in this repo tracks a clippy cleanup campaign, this milestone adds a new crate whose code will be clippy-clean but does not own the legacy baseline, and `tools/**` + CI stay zero-diff here. |
| `tools/bench_compare.py` stage-5 gate in CI | STAYS local-gate-only. Reason: CI adoption needs a dedicated self-hosted runner class (GitHub-runner timings incomparable); that is infrastructure capacity, not something a language-plugin milestone can honestly provide. Local fully-run stance (sec. 4) is this track's contribution. |
| `tools/theme_token_check.py --check-added` Phase B | STAYS warn-only / n/a. Reason: Phase B token parity belongs to the theming plan's own Rust tests (ledger row text); this milestone touches zero theme bytes (lock L10), so it cannot advance nor regress the flip. |

No row was silently ignored; no flip happened automatically.

## 8. Landing duty (orchestrator hop; memory-mirrors-ledger same-commit law)

Pre-checks (run before staging): `python tools/checkpoint_guard.py status`
(worktree census; E16), `python tools/hygiene_check.py` (expect exit 0 -
docs-only diff; hygiene walks `.rs`/`.py`/`.html`/`.js` only). Then, from
the repo root:

```
python tools/commit_scoped.py --scope tasks/rust-language-kickoff.md --scope tasks/rust-language-shape.md --scope tasks/milestone_ledger.md -m "chore(tasks): rust-language kickoff (kickoff+shape+ledger row; memory mirror same-commit)"
```

The kilo-memory mirror (key `ledger.milestone.rust-language`) is saved
session-side alongside this commit (memory tool writes outside the
workspace; the extraction-auto-apply precedent records exactly this seam).
The ledger row is the authority; on conflict, fix the memory.

This architect session authored the docs and the ledger row but has no
shell: the commands above are the landing hop's, and the ledger row's
landing SHA lands in its "Where" cell at that hop (or the closeout, per the
registration-wording precedent).

## 9. Scope-out (with reasons)

- No dynamic/out-of-process plugin loading - parked by ADR-0001; needs its
  own ADR, unchanged by this milestone.
- No macro expansion (`include!`/`macro_rules!`) - D-2 records it as the
  `analysis_caveat`, not a build target.
- No `cargo`/`rustc` invocation beyond `--version` probes - A3; tier-3
  real-compile invocation stays deferred for all languages (D-M23-12 class).
- No bench baseline changes; no own-tree corpus indexing - D-3.
- No edits to `crates/ui/**` or UI assets: language display is data-driven
  (`crates/cli/src/commands/ui/node_maps.rs` `format!("{:?}", language)`
  prints the bare open-id name "Rust"; verified this session).
- No `Cargo.lock`-adjacent vendoring of syn - workspace deps via
  `[workspace.dependencies]` per house style (shape sec. module map).
- Not this milestone: the R-2 analyzer does not resolve cross-crate
  (path-dependency) use edges beyond the repo's discovered file set -
  external-boundary stance mirrors C++/Scala.
