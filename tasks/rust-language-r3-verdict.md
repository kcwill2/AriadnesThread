# R-3 Verdict - rust-language: lang-rust crate skeleton (APPROVED)

Milestone: `rust-language`. Hand 3 of 10. Verdict: APPROVED.
Reviewer run fresh 2026-10-26 in worktree `balsam-basil` at HEAD `eeddfed`
(checkpoint_guard ground truth: branch `balsam-basil`, tree clean, no untracked).

Chain adjudicated from start rev `1dd042d`:

- duty 1 `3a7ec48` - `tasks/rust-language-r2-verdict.md` (244 new lines) +
  one-line ledger clause ONLY (stat verified). Landed verdict reads
  `Verdict: APPROVED` (r2-verdict.md:3); ledger clause present at ledger:62
  and matches the required form verbatim:
  `R-2 LANDED 3d4dfae APPROVED 2026-10-26, census anchor 1855 passed / 0
  failed / 13 ignored / 119 suites @ 1dd042d, R-3 dispatched.`
- landing `fe34e23` - 16 files, +1242/-2: the 13 crate files
  (`crates/lang-rust/Cargo.toml` + 12 src files, exactly the handoff file
  list), root `Cargo.toml` (+6), `Cargo.lock` (+13, 0 deletions), A8 (m17
  file, 2+/2-), journal (+138). All inside the declared touch-set.
- tail `b64577c` - journal-only split-trigger count correction (in touch-set).
- rider `eeddfed` - orchestrator's docs-only landing of the R-2
  execution-results addendum (stat: 1 tasks path, +45/-1). NOT part of the
  implementer's chain; same sanctioned pattern as `1dd042d` (R-2 verdict
  criterion 12). Adjudicated out of code scope.

## Findings

None. Zero blockers, zero minors.

## Dispositions (declared and independent)

1. RED-SKIP - ACCEPTED. The handoff itself pre-authorizes it: "RGB protocol"
   item 1 reads "Red - JUSTIFIED RED-SKIP (SM-1 / W-1 / CW-2 precedent trio,
   cite all three): no assertion can fail against a tree lacking the crate".
   The journal cites all three and each citation verifies on disk:
   `tasks/extraction-symbol-move-sm1-journal.md:24` ("No assertion can fail
   against" - within the cited 21-26 window),
   `tasks/ui-extraction-wizard-w1-journal.md:14` (D1 Red-skip declaration),
   `tasks/extraction-consumer-rewrite-cw2-journal.md:160` (falsifiability
   idiom - within the cited 153-160 window). The falsifiability statement is
   present and specific (every pin test named: caps bits, markers,
   vocabulary, caveat text, detector matrix, shaper rules, modeling fields,
   default seams, stub neutral returns; m17 guards as the historical Red).
2. Shaper root-context naming - ACCEPTED. Handoff: "context name = nearest
   ancestor dir holding a Cargo.toml (walk toward repo_root), fallback
   `\"root\"`". The implementation names the workspace root itself `root`
   rather than the machine-specific dir name - the Scala `ROOT_CONTEXT`
   precedent the handoff itself invokes, test-pinned at shaper.rs:203/:272.
3. Shaper case-insensitive component matching - ACCEPTED, judged on merits.
   The handoff's exclusion idiom IS Scala's `path_contains_components`
   (`crates/lang-scala/src/shaper.rs:31-42`), which lowercases components
   before the window match and whose semantics are pinned by Scala's own
   `pin_path_contains_components_is_case_insensitive_on_mixed_case_roots`
   (shaper.rs:311). The Rust mirror is faithful to the cited precedent, not
   a deviation from it.
4. Implementer duty-1 correctly refused to land the reviewer-owned
   `tasks/rust-language-execution-results.md` (journal records it as an
   out-of-scope observation; orchestrator rider `eeddfed` landed it - the
   disposition the R-2 verdict already fixed as correct).

## Plan-literal audit (handoff MUSTs, all verified against the code)

- Caps literal ALL-FALSE: eight stage bits false AND
  `requires_toolchain: false` (plugin.rs:143-155), pinned by in-src test
  `capabilities_match_the_skeleton_literal` (plugin.rs:233). No bit TRUE.
- No `analysis_fingerprint` override (doc-comment states the R-5 rule;
  default "v1" pinned at plugin.rs:256). No `aggregation_filter` /
  `target_path_for` overrides (defaults None pinned at plugin.rs:261-270).
- Modeling ten fields match the shape sec. 3.3 table verbatim:
  SourceFile / ModDeclaration / None / SingleFile / reexports false /
  TolerateWithinPackage / `["target","vendor",".cargo"]` from the single-
  sourced `boundary.rs` const / vocabulary "Use", "used", "mod aggregation" /
  caveat `Some(...)` containing "macro" (test asserts both equality and the
  contains-word idiom).
- Detector `name() == "rust"`; basename `Cargo.toml` case-folded BEFORE the
  ext rule; ext `rs` case-folded -> Source; else Other; return-type spelling
  `SourceFileKind` mirrors `crates/lang-scala/src/detector.rs:20` (no
  invented enum). Matrix tests cover `.rs`, `.RS`, `Cargo.toml`,
  `cargo.toml`, `pom.xml`, `README.txt`.
- Shaper: `build_markers() == ["Cargo.toml"]`, `default_compiler() ==
  "rustc"`, src-component inclusion, `tests`/`examples`/`benches` exclusion
  (exclusion wins on the pathological `src/tests/` path), nearest non-root
  Cargo.toml context, `root` fallback - all four required tempdir tests
  present.
- Stubs compiling neutral returns, no `todo!`/`unimplemented!` anywhere in
  shipped code (only a test-arm `panic!` and two doc/test-string mentions):
  analyzer empty `TuAnalysis`, resolver empty vec, constraints empty
  (FINAL shape), extractor ASCII deferral-header-only, rewriter identity,
  validator `Missing` + `ValidationNotWired` naming
  "probe-only validator lands at rust-language R-9".
- Manifest: package fields workspace-pinned, three parser deps as
  `.workspace = true` (zero version literals in the crate manifest),
  dev-dep tempfile, m17 inward-only comment block restated; root
  `Cargo.toml` gains exactly `syn = { version = "2", features = ["full"] }`,
  `proc-macro2 = { version = "1", features = ["span-locations"] }`,
  `unicode-ident = "1"` with the load-bearing note, members appended after
  `"crates/lang-scala"`.
- A8 / INV-11: both `PLUGIN_CRATES` appends land in the SAME commit as
  `crates/lang-rust/Cargo.toml` (fe34e23 stat). Chain diff shows EXACTLY two
  hunks on the m17 file, each `[&str; 2]` -> `[&str; 3]` adding
  `"crates/lang-rust"` at :271 and :486, nothing else in the file. A8
  activated: gate 2 green WITH the new element asserted by both guards.
- Cargo.lock PURELY ADDITIVE: one new `ariadne-lang-rust` package block;
  `syn 2.0.119` already resolved (no version moved); zero deletions.
- Registration absence: `git diff 1dd042d..HEAD -- crates/pipeline crates/cli`
  EMPTY; pipeline green UNEDITED at 46/0/4/9.
- L7/L8/L9 + L2/L3/L4: `git diff --name-only 1dd042d..HEAD` names no path
  under `crates/lang-cpp`, `crates/lang-scala`, `tools`, `tests/golden`,
  `tests/fixtures` (m17 file is the single named tests path).

## Census and split trigger (reviewer-measured)

- Fresh full census: `1878 passed / 0 failed / 13 ignored / 121 suites`
  (exit 0). Reconcile vs duty-1 anchor 1855 @ `1dd042d`: +23 = the
  `ariadne_lang_rust` lib suite (capture names it at line 3202; scoped gate
  confirms its 23 tests); suites 119 -> 121 attributed BY NAME:
  `Running unittests src\lib.rs (...ariadne_lang_rust-...)` + `Doc-tests
  ariadne_lang_rust` (capture lines 3202/3241) - the cargo Doc-tests group
  the handoff anticipated; ignored flat at 13.
- Split trigger NOT triggered: landing numstat 1242+/2- minus Cargo.lock 13
  machine bytes minus foreign-docs = 1229+/2- = ~1231 changed lines vs the
  pre-declared ~1350 ceiling; shaper stayed, no split proposal needed.

## Observations (non-blocking, not attributed to this hand)

- O-1: fresh `python tools/distill.py -- python tools/bench_compare.py`
  returned FAIL exit 1 with exactly two WARN lines:
  `WARN get_or_build_include_graph clone (cold): 1.7 ms vs baseline 1.3 ms (+31%)`
  and `WARN get_or_build_include_graph clone (warm): 1.3 ms vs baseline 1.0 ms (+30%)`.
  Recorded NOT attributed per the handoff's reviewer-pass stance ("WARNs
  recorded not attributed - PR-2") and the kickoff's bench posture (sec. 4:
  per-machine baseline, no baseline/corpus change this milestone).
  Attribution is structurally excluded: the crate is unregistered,
  pipeline/cli are byte-clean, and the include-graph clone phases are the
  C++ path - no dispatch route exists to lang-rust bytes. Sub-millisecond
  absolute deltas on a 0.0-ms-noise-known phase family (ledger bench-noise
  row). For orchestrator awareness only.
- O-2: hygiene output carries zero R3 findings under `crates/lang-rust`
  (kickoff's zero-new-R3 flip EVIDENCE row advances).

## Gate and refusal log (this reviewer run)

Ran fresh: gate 1 `23 passed / 0 failed / 0 ignored / 2 suites`; gate 2
`10 passed / 0 failed / 0 ignored / 1 suites`; gate 3
`46 passed / 0 failed / 4 ignored / 9 suites`; gate 4
`1878 passed / 0 failed / 13 ignored / 121 suites`; hygiene exit 0;
`encoding_guard.py --all` exit 0; tools discovery `Ran 296 tests in 30.369s`
/ `OK`; bench via distiller (see O-1); checkpoint_guard status; per-commit
`git show --stat` oldest-first over all four commits; all lock diff proofs.
Refused by the catch-all deny rule (verbatim
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`): one
compound `git log ...; echo ...; git status` chain (substituted by the git
tools), and two `rg` invocations (substituted by the grep tool). Refused
verbatim on its own terms: `python tools/testrun.py --baseline` ->
`usage error: missing value for --baseline` (F9-errata FILE form, superseded
track-wide; the governing `--distill` census is the acceptance line). No
expected gate was skipped.

Follow-up dispositions: none owed. The R-2 execution-results landing is
already closed by rider `eeddfed`. O-1 needs no handoff; it is a per-machine
baseline observation for orchestrator awareness at closeout.
