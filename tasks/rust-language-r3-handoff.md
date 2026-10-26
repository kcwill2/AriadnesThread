# R-3 Handoff - rust-language: the lang-rust crate skeleton (unregistered, caps all-false, A8 ride-along)

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). Hand 3
of 10, serial. Creates `crates/lang-rust/` with the COMPLETE required trait
surface: real detector + shaper + modeling literals, honest minimal
never-dispatched implementations for the components later hands grow, all
capabilities false (the crate is UNREGISTERED at this hop - nothing can be
dispatched to it; capability monotonicity INV-5 in its strongest form). Lands
A8 (both `PLUGIN_CRATES` appends) in the SAME commit as the manifest (INV-11).
Start rev: your dispatch head (R-2's landing + verdict via your duty 1).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/rust-language-shape.md` sec. 2 (INV-1, INV-4 identity bytes land at
   R-4, INV-5, INV-11, INV-12), sec. 3.2 (registration lands at R-4, workspace
   deps), sec. 3.3 (the modeling decision table - your literals), sec. 3.4
   (capability schedule - this hop is ALL-FALSE), sec. 3.5 (detector/shaper
   contracts VERBATIM), sec. 7 (file-size plan: analyzer pre-split at R-5).
   Kickoff A8 + L6; D-2 (syn deps, `span-locations` load-bearing).
3. Model files to mirror (structure/doc style ONLY - zero edits):
   `crates/lang-scala/src/{lib.rs,plugin.rs,detector.rs,shaper.rs,boundary.rs}`
   and `crates/lang-scala/Cargo.toml`.
4. Trait surface of record: `crates/lang/src/registry.rs` (trait at :23,
   12 methods / 10 required - guide `docs/adding-a-language.md` sec. "Step 2"
   table) and `crates/lang/src/unit.rs` (`UnitShaper`).
5. The guard you activate: `tests/integration/tests/m17_plugin_seam_test.rs`
   (:271 manifest guard list, :486 transitive-tree guard list; both
   `[&str; 2]`, self-skip a missing manifest).
6. Your duty-1 targets: `tasks/rust-language-r2-verdict.md` + ledger.

## Duty 1 (before any R-3 mutation)
Land R-2's verdict + ledger flip exactly as R-2's duty-1 clause shapes it
(`commit_scoped.py --scope tasks/rust-language-r2-verdict.md --scope
tasks/milestone_ledger.md`; ledger clause: `R-2 LANDED <SHA> APPROVED <date>,
census anchor <line> @ <SHA>, R-3 dispatched.`). The landed verdict must be
APPROVED; journal both SHAs.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/rust-language-r3-journal.md (create)
- tasks/rust-language-r2-verdict.md + tasks/milestone_ledger.md (duty 1 only)
- crates/lang-rust/** (NEW crate, full file list below)
- Cargo.toml (workspace: `members` append + `[workspace.dependencies]` gain)
- Cargo.lock (cargo regenerates it in this commit; reviewer rule - it MUST be
  in the touch-set or the commit-vs-touch-set check trips; regenerated lines
  are machine bytes, NOT counted against the produced-line budget)
- tests/integration/tests/m17_plugin_seam_test.rs (A8 ONLY: the two
  `PLUGIN_CRATES` lines :271 and :486, `[&str; 2]` -> `[&str; 3]` appending
  `"crates/lang-rust"`; nothing else in the file)

You MUST NOT touch: `crates/pipeline/**` and `crates/cli/**` (registration is
R-4's hop - this hand does NOT register), `crates/repo/**`, `crates/lang-api/**`,
`crates/lang/**`, `crates/lang-cpp/**`, `crates/lang-scala/**` (zero-diff
L7/L8), any other `tests/**` file, `tools/**`, fixtures, goldens. No `git
stash`, no branch change.

## What to build

Crate files (mirror Scala's inward shape, shape sec. 4):
- `crates/lang-rust/Cargo.toml` - model on lang-scala's manifest verbatim in
  shape (package fields workspace-pinned; deps: `ariadne-repo`,
  `ariadne-lang-api`, `ariadne-lang` path edges + the three parser deps as
  workspace-pinned: `syn.workspace = true`, `proc-macro2.workspace = true`,
  `unicode-ident.workspace = true`; dev-deps: `tempfile.workspace = true`).
  Keep the m17 "depends inward only / deliberately NOT depended on" comment
  block, restated for this crate. The three parser deps are the D-2 deliberate
  deviation - the R-10 docs rider records it; the m17 guards forbid only
  ariadne-graph/cluster/transform/validate (manifest) +
  scheduler/report/cluster/graph/transform/validate (transitive) - third-party
  parser deps cannot reach them (shape sec. 6 RISK-GUIDE-DEPS, read-verified).
- Root `Cargo.toml`: `"crates/lang-rust"` appended to `members` after
  `"crates/lang-scala"`; `[workspace.dependencies]` gains (house style,
  shape sec. 3.2 literals):
  `syn = { version = "2", features = ["full"] }`
  `proc-macro2 = { version = "1", features = ["span-locations"] }`
  `unicode-ident = "1"`
  `span-locations` is load-bearing for INV-7 (D-2): no version bump may drop it.
- `src/lib.rs` - module decls + crate doc (third language plugin; parses with
  `syn`; macro-expansion limit = `analysis_caveat`, D-2).
- `src/boundary.rs` - single-sourced markers const mirroring Scala's
  `boundary.rs` idiom: `["target", "vendor", ".cargo"]` (shape sec. 3.3 row
  `external_path_markers`), one function or const the plugin consumes.
- `src/detector.rs` - shape sec. 3.5 VERBATIM: `name() == "rust"`;
  `detect()`: basename `Cargo.toml` (case-folded) -> BuildDefinition BEFORE
  the ext rule; ext `rs` (case-folded) -> Source; everything else -> Other.
  Mirror Scala's detector return-type spelling (do NOT invent a new enum).
- `src/shaper.rs` - shape sec. 3.5 VERBATIM: `build_markers() ==
  ["Cargo.toml"]`; `default_compiler() == "rustc"`; `shape_units`:
  `FileKind::RustSource` records whose path contains a consecutive lowercase
  `["src"]` component, EXCLUDING paths carrying a `tests`/`examples`/`benches`
  component (exclusion-tag idiom, Scala's `src/test/scala` precedent);
  context name = nearest ancestor dir holding a `Cargo.toml` (walk from the
  file's dir toward `repo_root`), fallback `"root"`.
- `src/plugin.rs` - `RustPlugin::new()`; `id() == LanguageId::Rust`;
  `modeling()` per the shape sec. 3.3 table EXACTLY: unit `SourceFile`,
  aggregation `AggregationKind::ModDeclaration`, guard `GuardKind::None`,
  symbol_placement `SingleFile` (declared record only - no dispatch, guide
  trap), `aggregation_reexports: false`, `cycle_policy
  TolerateWithinPackage`, external_path_markers from `boundary.rs`, vocabulary
  `directive_label "Use"` / `directive_passive_verb "used"` /
  `aggregation_noun "mod aggregation"`, `analysis_caveat` = `Some(...)` text
  naming the macro limit and containing the word `macro` (Scala's
  `contains("heuristic")` test idiom). `capabilities()`: the ALL-FALSE
  skeleton literal - all eight stage bits false AND
  `requires_toolchain: false` (A3/D-2) - pinned by in-src test
  `capabilities_match_the_skeleton_literal` (pattern of Scala's
  `capabilities_match_the_m23_flip_literal`, plugin.rs:431). NO
  `analysis_fingerprint` override (trait default `"v1"` stays; `"rust-1"`
  lands at R-5 with the real analyzer - state this in a doc comment).
  `shaper()` returns Some over your shaper. The seven required `&dyn`
  accessors return owned minimal implementations. NO `aggregation_filter` /
  `target_path_for` overrides (trait defaults None - planning enrollment is
  R-9's hop; INV-5).
- `src/analyzer.rs` - HONEST NEVER-DISPATCHED stub: returns an empty
  `TuAnalysis`; doc: `semantic_index` is false so this is never dispatched;
  the real `syn` analyzer lands at R-5 (fingerprint `"rust-1"` rides it).
  It must still be a compiling, tested (returns neutral) implementation -
  NOT `todo!()`/`unimplemented!()` (SM-1's red rule applies to shipped code
  too).
- `src/resolver.rs` - stub: empty edge vec, same doc discipline; real
  resolver at R-6.
- `src/constraints.rs` - FINAL honest shape already: `colocated_groups`
  empty vecs, `frozen_symbols` empty (shape sec. 3.7 - no companion-object
  idiom; IV3: policy owns freezing, Scala parity). This file does not grow.
- `src/extractor.rs` - stub: emits a `GeneratedSourceFile` whose content is
  only an ASCII header comment naming the deferral (real extractor at R-8);
  never dispatched (`extraction: false`).
- `src/dependency_rewriter.rs` - stub: `rewrite` returns `source` unchanged
  (identity - honest: `rewriting: false`; the use-path rewrite + mod-tree
  mechanism lands at R-8).
- `src/validator.rs` - stub: `toolchain_status()` -> `Missing`;
  `validate_compile` -> `Err(...)` whose message names the deferral
  ("probe-only validator lands at R-9" - honest today, replaced then;
  TierValidator never reaches it while `Missing`, INV-10 class).
- In-src tests (module-level `mod tests`, house names): plugin literal pins
  (all ten modeling fields, id, caps skeleton literal); detector matrix
  (`.rs`, `.RS` folded, `Cargo.toml`, `cargo.toml` case-folded basename,
  `pom.xml` -> Other, `README.txt` -> Other); shaper tempdir tests (src-root
  unit, `tests/`-component exclusion, nearest-Cargo.toml context name, root
  fallback); boundary markers pin; one neutral-returns test per stub file.

A8 (INV-11): append `"crates/lang-rust"` as the 3rd element of BOTH
`PLUGIN_CRATES` lists in `m17_plugin_seam_test.rs` (:271 and :486), element
appends only, in the SAME commit that creates `crates/lang-rust/Cargo.toml`
(guards self-skip a missing manifest - append-early would silently pass, so
the reviewer treats append-late as the violation).

## RGB protocol

1. Red - JUSTIFIED RED-SKIP (SM-1 / W-1 / CW-2 precedent trio, cite all
   three): no assertion can fail against a tree lacking the crate; a
   compile-failing test against a non-existent crate is not a Red. Journal
   the falsifiability statement instead: every pin test above fails on
   literal drift (caps bits, markers, vocabulary, detector answers, shaper
   rules), and the m17 manifest/transitive guards are the historical Red
   that would catch a forbidden dep edge on any future hop.
2. Green: build the crate until all gates green.
3. Refactor: honest names, Scala-mirroring docs; re-run.

## Hygiene (review-enforced)
Every new file < 1000 with margin (plugin.rs ~330, shaper.rs ~300); analyzer
stays a stub (the ~600-line pre-split rule bites at R-5, not here); functions
< 100; no banned names / single-letter locals; pure ASCII (incl. every doc
comment). `python tools/hygiene_check.py` exit 0.

## Frozen locks (commands that prove them)
- L6: `git diff <start>..HEAD -- tests/integration/tests/m17_plugin_seam_test.rs`
  shows EXACTLY two hunks, each a `[&str; 2]` -> `[&str; 3]` element append.
- L7/L8/L9: `git diff --name-only <start>..HEAD` touches no path under
  `crates/lang-cpp`, `crates/lang-scala`, `tools`.
- L2/L3/L4: goldens and existing fixtures absent from any diff; m00 green
  UNEDITED via gate 4 census.
- Registration absence: `git diff <start>..HEAD -- crates/pipeline
  crates/cli` EMPTY (this hand must not register).
- L12: `python tools/encoding_guard.py --all` exit 0.

## Gates (exact commands, timeout 600000 ms each; distilling entry points only)
1. `python tools/testrun.py --distill -p ariadne-lang-rust` - Red state is
   skipped (declared); run this AFTER the skeleton compiles: expect exit 0,
   canonical line with the new suite's passed count recorded verbatim (this
   is the suite's first census appearance).
2. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m17_plugin_seam_test` - A8 activated: both guards now assert on your
   manifest and transitively, green (INV-11).
3. `python tools/testrun.py --distill -p ariadne-pipeline` - green UNEDITED
   (the crate exists but is unregistered; the registry is byte-clean).
4. `python tools/testrun.py --distill` - full census: passed = previous +
   your suite's N (+1 suite, +1..+2 if cargo adds a Doc-tests group for the
   new crate - attribute whatever the line shows by name), failed 0, ignored
   13, suites 119 -> 120 (or 121, attributed). Anchor from your duty-1 ledger
   clause; `--distill` governs (F9).
5. `python tools/hygiene_check.py` - exit 0.
6. Commit cycle: `python tools/encoding_guard.py --staged` (or `--all`) then
   `python tools/commit_scoped.py --scope crates/lang-rust --scope Cargo.toml
   --scope Cargo.lock --scope tests/integration/tests/m17_plugin_seam_test.rs
   --scope tasks/rust-language-r3-journal.md -m "feat(lang-rust): add the
   plugin crate skeleton with honest never-dispatched defaults and the A8
   guard appends (rust-language R-3)"`. Journal inside.

Declared gate budget: 5 scoped invocations + commit cycle - inside the law.

## Discipline clauses
- UTF-8: Write/Edit tools only; pure ASCII; no PowerShell redirection writes.
- Touch only the touch-set; report, don't fix, out-of-scope breakage.
- Journal checkpoint after EACH Green gate (per file-group).
- Context exhaustion: STOP + disk-state inventory; successor runs
  `checkpoint_guard.py resume --spec tasks/rust-language-r3-handoff.md
  --scope crates/lang-rust --cmd ...` (incident E1/E16 discipline).
- Landing SHA relay: report it; reflog tip; R-4's duty 1 cites it.
- F14: never pin warning counts; never state a count without its command.

## Report back (final message)
1. Duty-1 SHA + landing SHA + `git show --stat`. 2. Justified Red-skip with
its three citations + falsifiability statement. 3. Green evidence: canonical
lines for gates 1-4 + capture path. 4. Deviations + reasons. 5. Out-of-scope
observations. No raw output.

## Reviewer acceptance pass
- `python tools/distill.py -- python tools/bench_compare.py` - RUN fresh
  (new crate under `crates/*/src`; kickoff sec. 4 fully-run stance; WARNs
  recorded not attributed - PR-2; no baseline/corpus change, D-3).
- Per-commit `git show` oldest-first vs touch-set (Cargo.lock is IN the set -
  the known reviewer trap, named by shape RISK-LOCKFILE); A8 exactly two
  hunks; manifest parser deps workspace-pinned (no version literals in the
  crate manifest); `span-locations` feature present; no pipeline/cli hunk.

Reviewer-gate reference: `review-gate` skill. Extra MUSTs: caps literal is
ALL-false including `requires_toolchain: false`; no `analysis_fingerprint`
override; no planning-hook overrides; stubs are compiling neutral returns,
never `todo!`/`unimplemented!`.

Sizing (planner, itemized): manifest ~22, lib ~40, boundary ~40, detector
~180 (110+tests), shaper ~280 (200+80), plugin ~310, constraints ~110,
analyzer ~60, resolver ~55, extractor ~70, rewriter ~65, validator ~75,
root Cargo ~6, A8 ~4/-2, journal ~15 - total ~1150-1350 produced lines
(LOCKER-LOCK: Cargo.lock regeneration excluded as machine bytes). Ceiling
pressure is known and declared: if your produced volume exceeds ~1350 at the
green gate, STOP per the law and journal a split proposal - the sanctioned
pre-designed split is shaper.rs + its tempdir tests (~280 lines) MOVING to
R-4 (R-4's estimate absorbs +330; the seam is clean because the crate is
unregistered and `shaper()` would return None in the interim, an honest
no-shaper state for a plugin with no capabilities). 5 gates + commit trio;
one crate dir + one two-line integration append justified by INV-11's
same-commit law.
