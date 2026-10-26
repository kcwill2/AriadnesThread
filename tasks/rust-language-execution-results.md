# rust-language - Execution Results (per-hand, reviewer acceptance)

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). This
file accumulates the reviewer-run acceptance evidence per hand. Hands gated so
far: R-1, R-2, R-3.

## R-1 - spine, part repo (FileKind::RustSource + LanguageId::Rust)

Landing commit `e862acf` @ start rev `bc0c11c`. Verdict: APPROVED - see
`tasks/rust-language-r1-verdict.md`. All lines below are the reviewer's own
fresh runs (2026-10-26), canonical lines quoted verbatim.

| # | Acceptance criterion | Result | Evidence line (verbatim) |
|---|----------------------|--------|--------------------------|
| 1 | Scoped repo suite green at HEAD (handoff gate 3 form) | PASS | `39 passed / 0 failed / 0 ignored / 5 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-repo`) |
| 2 | Matrix pins re-run green - rows :103 AND :114 BOTH flipped in this commit (reviewer MUST) | PASS | `2 passed / 0 failed / 0 ignored / 4 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-repo pin_file_kind`); final file carries `("a.rs", FileKind::RustSource),` at types.rs:106 and `("a.h.rs", FileKind::RustSource),` at types.rs:117 |
| 3 | cli green UNEDITED (handoff gate 4; amendments A1-A4 intact) | PASS | `182 passed / 0 failed / 8 ignored / 8 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-cli`); zero `crates/cli` paths in `git diff --name-only bc0c11c..HEAD` |
| 4 | Full census reconcile vs anchor `1853 passed / 0 failed / 13 ignored / 119 suites @ 522a770` | PASS | `1854 passed / 0 failed / 13 ignored / 119 suites` (exit 0, `python tools/testrun.py --distill`); delta +1 = the one new pin `identity::tests::rust_source_kind_carries_the_rust_identity`; suites flat |
| 5 | Hygiene exit 0 (gate 6) | PASS | `python tools/distill.py -- python tools/hygiene_check.py` -> `### generic PASS (exit 0)` (capture 173 lines); all output warn-only R3, none naming the two touched files; types.rs 308 lines, identity.rs 437 lines |
| 6 | L12 encoding: `encoding_guard.py --all` exit 0 | PASS | `python tools/distill.py -- python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)` |
| 7 | Reviewer acceptance bench (handoff reviewer-pass, RUN fresh) | PASS | `python tools/distill.py -- python tools/bench_compare.py` -> `### generic PASS (exit 0)`; `bench compare: 3 run(s), no regressions past 1.25x baseline`; zero WARNs emitted this run (PR-2 stance moot) |
| 8 | Tool unit suites (CI form) | PASS | `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 296 tests in 30.434s` / `OK` |
| 9 | L1 frozen trio in NO hunk; L11 pure append | PASS | identity.rs hunk ranges (old 59-61, 85-90, 96-101, 121-126, 136-141, 398-401) exclude old :149-175, :210-231, :235-242 entirely; enum hunk adds only after `Other,` - zero deletions, zero reordering; final variant order `Header, Source, ScalaSource, ScalaScript, BuildDefinition, Other, RustSource` |
| 10 | Commit-vs-touch-set (L2/L4/L7/L8/L9) | PASS | `git diff --name-only bc0c11c..HEAD` -> `crates/repo/src/identity.rs`, `crates/repo/src/types.rs`, `tasks/rust-language-r1-journal.md` - all inside the touch-set; `Cargo.lock` untouched |
| 11 | Red was the matrix pin (historical, journal-recorded; not reproducible by reviewer) | RECORDED | Journal gate 2: exit 1, `30 passed / 1 failed / 0 ignored / 1 suites`, failure `types::file_kind_matrix_tests::pin_file_kind_lowercase_extension_matrix`, `left: Other`, right Other, at `crates\repo\src\types.rs:87:9`, capture `target/testrun-capture.txt`; Green state proves the honest replacement (criterion 2) |
| 12 | Plan-literal compliance (shape sec. 3.1 / handoff items 1-7) | PASS | enum line, `Some("rs") => FileKind::RustSource,`, `matches!` addition, `pub const RUST: &str = "Rust";`, `pub const RUST: &'static str = RUST;`, `pub const Rust: Self = Self(Cow::Borrowed(RUST));` with `#[allow(non_upper_case_globals)]`, `pub fn rust() -> Self`, `FileKind::RustSource => Self::Rust,`, matrix rows, `("a.RS", FileKind::RustSource),`, pin test name + INV-12/INV-9 messages - all verified verbatim against final files |

### Not run / refused (explicit statement)

- `python tools/testrun.py --baseline` (bare acceptance-pass form): refused
  with `usage error: missing value for --baseline`. This FILE form is named
  superseded by the handoff (F9 errata - `--distill` governs); the governing
  fresh `--distill` census is criterion 4.
- `python tools/ui_smoke.py`: not required by this hand (no UI/runtime-surface
  bytes); NOT run.
- `python tools/golden_guard.py --base <rev>`: tool not landed; substituted by
  the handoff's prose byte-check (criteria 9-10 hunk-range + name-only proof).
- Two commands refused by the catch-all deny rule (verbatim
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
  `python -c` line-count probe (substituted via Read tool) and a chained
  `$LASTEXITCODE` probe (substituted via `distill.py` exit lines). No expected
  gate was skipped.

## R-2 - spine, part lang-api (AggregationKind::ModDeclaration + advisory row A6)

Chain from start rev `e862acf`: duty 1 `f7c2f93` + sanctioned rider `1cf6786`
+ landing `3d4dfae` (+ orchestrator rider `1dd042d`, not the implementer's).
Verdict: APPROVED - see `tasks/rust-language-r2-verdict.md`. All lines below
are the reviewer's own fresh runs (2026-10-26), canonical lines verbatim.

| # | Acceptance criterion | Result | Evidence line (verbatim) |
|---|----------------------|--------|--------------------------|
| 1 | Scoped lang-api green at landing HEAD = journalled baseline 47 + 1 pin (handoff gate 2 form) | PASS | `48 passed / 0 failed / 0 ignored / 5 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-lang-api`); baseline `47 passed / 0 failed / 0 ignored / 5 suites` journal-recorded (gate 1 pre-mutation, unreproducible post-landing by a reviewer) |
| 2 | Full census reconcile vs duty-1 ledger anchor `1854/0/13/119 @ e862acf` | PASS | `1855 passed / 0 failed / 13 ignored / 119 suites` (exit 0, `python tools/testrun.py --distill`, capture 3215 lines); delta +1 = `aggregation_kind_append_is_serde_index_stable`; ignored 13, suites flat 119 - goldens, m00, m17, lang-cpp, lang-scala, tools green UNEDITED |
| 3 | Hygiene exit 0 (gate 4) | PASS | `python tools/distill.py -- python tools/hygiene_check.py` -> `### generic PASS (exit 0)` (capture 173 lines); all output warn-only R3; capture grep `lang-api` -> no match; modeling.rs 232 lines, identity.rs 53 lines |
| 4 | L12 encoding `encoding_guard.py --all` exit 0 | PASS | `python tools/distill.py -- python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)` |
| 5 | Reviewer acceptance bench, RUN fresh (kickoff fully-run stance; lang-api/src touched) | PASS | `python tools/distill.py -- python tools/bench_compare.py` -> `### generic PASS (exit 0)`; `bench compare: 3 run(s), no regressions past 1.25x baseline`; zero WARN lines in capture (grep: no match; PR-2 moot) |
| 6 | Tool unit suites (CI form) | PASS | `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 296 tests in 29.127s` / `OK` |
| 7 | L11 pure append - THE lock of this hand | PASS | `git show "3d4dfae" -- crates/lang-api/src/modeling.rs`: enum hunk `@@ -17,6 +17,13 @@` adds 7 lines after `None,` (doc + `ModDeclaration,`), 0 deletions, 0 reordered lines; tests-module hunk appends the 21-line pin; final order `Header, PackageObject, None, ModDeclaration` = shape sec. 3.1 literal; pre-existing indices 0/1/2 unmoved (INV-12) |
| 8 | A6 row = APPEND-only tests-module change; :15 re-export byte-identical | PASS | identity.rs diff is exactly one added line `(FileKind::RustSource, LanguageId::Rust),` at the `cases` tail (`@@ -40,6 +40,7 @@`); :15 `pub use ariadne_repo::identity::{LanguageId, CPP, OTHER, SCALA};` outside every hunk; advisory arm pinned by A6 verified shipped at R-1: `crates/repo/src/identity.rs:152` `FileKind::RustSource => Self::Rust,` |
| 9 | RISK-AGGKIND: no new exhaustive match surfaced | PASS | fresh census compiled the whole workspace exit 0; workspace grep: all 21 `AggregationKind` occurrences are imports/construction sites/test modules; no `match` - no STOP |
| 10 | Commit-vs-touch-set (L1-L9) | PASS | `git diff --name-only e862acf..HEAD` = 7 paths, all inside the touch-set + the sanctioned rider (`r1-journal`, R-1 verdict finding-1 disposition) + the orchestrator's own docs rider (`execution-results`); `git diff --name-only f7c2f93..HEAD` shows zero bytes under `crates/repo/`, `crates/lang/`, `crates/lang-rust/`, `crates/cli/`, `tests/`, `tools/`; no `Cargo.lock` delta; landing commit stat = the two lib files + journal, 188 insertions, 0 deletions |
| 11 | Red-skip disposition (pins-of-shipped-truth + pure-append precedent trio) | ACCEPTED | cited journals verified on their merits: `extraction-symbol-move-sm1-journal.md:21-26`, `ui-extraction-wizard-w1-journal.md:14-17`, `extraction-consumer-rewrite-cw2-journal.md:153-160`; A6 truth independently verified shipped (criterion 8); falsifiability journalled - one leg overstated, recorded as verdict finding 1 (plan-level, follow-up to planner) |
| 12 | Duty-1 completeness vs handoff text | PASS | `f7c2f93` = verdict + one-line ledger clause ONLY (stat verified); clause matches the R-1 APPROVED verdict state element-for-element (`R-1 LANDED e862acf APPROVED 2026-10-26, census anchor 1854 passed / 0 failed / 13 ignored / 119 suites @ e862acf, R-2 dispatched.`); handoff names no execution-results (grep: no match), implementer correctly left the untracked file, orchestrator landed it docs-only as `1dd042d` (stat: 1 tasks path, +42) |

### Not run / refused (explicit statement)

- `python tools/testrun.py --baseline` (bare acceptance-pass form): refused
  with `usage error: missing value for --baseline` - the F9-errata FILE form,
  superseded track-wide; the governing fresh `--distill` census is criterion
  2.
- `python tools/testrun.py --distill -p ariadne-lang-api` at the PRE-mutation
  baseline (handoff gate 1): structurally impossible for a reviewer at
  landing HEAD (would require reverting landed mutations); baseline evidence
  is journal-recorded and the 48 = 47 + 1 arithmetic verified at HEAD
  (criterion 1).
- `python tools/ui_smoke.py`: not required by this hand (no UI/runtime-surface
  bytes); NOT run.
- `python tools/golden_guard.py --base <rev>`: tool not landed; substituted by
  the handoff's prose byte-check (criterion 7 diff review + criterion 10
  name-only proof + census UNEDITED oracle).
- One compound bash call (chained git diffs + `Select-Object`) refused with
  the catch-all boilerplate
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`;
  substituted by the two separate `git diff --name-only` invocations - both
  ran, reported in criterion 10. No expected gate was skipped.

## R-3 - lang-rust crate skeleton (unregistered, caps all-false, A8 ride-along)

Chain from start rev `1dd042d`: duty 1 `3a7ec48` + landing `fe34e23` +
journal tail `b64577c` (all implementer, all inside the touch-set), plus the
orchestrator's docs rider `eeddfed` (R-2 execution-results landing - outside
the R-3 contract, adjudicated out of code scope; R-2 addendum above was
landed by that rider). Verdict: APPROVED - see `tasks/rust-language-r3-verdict.md`.
All lines below are the reviewer's own fresh runs (2026-10-26, HEAD
`eeddfed`, tree clean), canonical lines quoted verbatim.

| # | Acceptance criterion | Result | Evidence line (verbatim) |
|---|----------------------|--------|--------------------------|
| 1 | Scoped lang-rust suite green, first census appearance (handoff gate 1) | PASS | `23 passed / 0 failed / 0 ignored / 2 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-lang-rust`) |
| 2 | m17 seam suite A8-activated green (gate 2, INV-11) | PASS | `10 passed / 0 failed / 0 ignored / 1 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test`) |
| 3 | Pipeline green UNEDITED, unregistered (gate 3) | PASS | `46 passed / 0 failed / 4 ignored / 9 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-pipeline`) |
| 4 | Full census +23/+2-suites attributed by name vs anchor `1855/0/13/119 @ 1dd042d` (gate 4) | PASS | `1878 passed / 0 failed / 13 ignored / 121 suites` (exit 0, `python tools/testrun.py --distill`, capture 3250 lines); delta +23 named `Running unittests src\lib.rs (target\debug\deps\ariadne_lang_rust-12170b58a92848f1.exe)` (capture:3202); the +2 suites are that lib suite + `Doc-tests ariadne_lang_rust` (capture:3241); ignored 13, failed 0 |
| 5 | Hygiene exit 0 (gate 5) | PASS | `python tools/distill.py -- python tools/hygiene_check.py` -> `### generic PASS (exit 0)` (capture 173 lines); output is warn-only R3 on pre-existing files; zero lines name `crates/lang-rust` or any touched path |
| 6 | L12 encoding `encoding_guard.py --all` exit 0 | PASS | `python tools/distill.py -- python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)` (capture 0 lines) |
| 7 | Tool unit suites (CI form) | PASS | `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 296 tests in 30.369s` / `OK` |
| 8 | Reviewer acceptance bench, RUN fresh (kickoff fully-run stance; new crate under crates/*/src) | RECORDED, NOT ATTRIBUTED (kickoff PR-2) | `python tools/distill.py -- python tools/bench_compare.py` -> `### generic FAIL (exit 1)`; WARN lines verbatim: `WARN get_or_build_include_graph clone (cold): 1.7 ms vs baseline 1.3 ms (+31%)`, `WARN get_or_build_include_graph clone (warm): 1.3 ms vs baseline 1.0 ms (+30%)`; sub-millisecond deltas on the C++ include-graph phases with pipeline/cli byte-clean and the crate unregistered - no dispatch route exists to lang-rust bytes; no baseline/corpus change (D-3 held) |
| 9 | A8 EXACTLY two hunks, `2]`->`3]` element appends, same commit as the manifest (L6/INV-11) | PASS | `git diff "1dd042d..HEAD" -- tests/integration/tests/m17_plugin_seam_test.rs` = two hunks `@@ -268,7 +268,7 @@` / `@@ -483,7 +483,7 @@`, each `const PLUGIN_CRATES: [&str; 2]` -> `[&str; 3]` with `"crates/lang-rust"` third (lines :271, :486), nothing else; `git show --stat "fe34e23"` names BOTH the m17 file and `crates/lang-rust/Cargo.toml` |
| 10 | Registration absence + L7/L8/L9/L2/L3/L4 locks | PASS | `git diff "1dd042d..HEAD" -- crates/pipeline crates/cli` EMPTY; `git diff --name-only "1dd042d..HEAD"` = 19 paths: the 13 crate files, root `Cargo.toml`, `Cargo.lock`, the m17 file, 4 tasks paths - nothing under `crates/lang-cpp`, `crates/lang-scala`, `crates/pipeline`, `crates/cli`, `tools`, goldens, or fixtures |
| 11 | Caps literal ALL-FALSE incl. `requires_toolchain: false`; no fingerprint bump; no planning-hook overrides | PASS | `crates/lang-rust/src/plugin.rs` lines 143-155 all-false struct + no `analysis_fingerprint`/`aggregation_filter`/`target_path_for` methods; default seams test-pinned (`plugin_default_seams_stay_at_trait_defaults`: fingerprint `"v1"`, both hooks `None`) |
| 12 | Modeling/detector/shaper/boundary literals verbatim vs shape sec. 3.3/3.5 | PASS | `src/detector.rs:21/:40` (`"rust"`, case-folded basename-before-ext), `src/shaper.rs:104/:111` (`vec!["Cargo.toml"]`, `"rustc"`), `src/boundary.rs:15` (`&["target", "vendor", ".cargo"]`), `src/plugin.rs` vocabulary `"Use"`/`"used"`/`"mod aggregation"` + caveat containing `macro` - all byte-checked against the handoff literals, each test-pinned in-src |
| 13 | Stubs are compiling neutral returns (no `todo!`/`unimplemented!`) | PASS | grep `todo!|unimplemented!|panic!` over `crates/lang-rust`: only a test-arm `panic!` at `validator.rs:65` and two doc-comment/string mentions; analyzer/resolver/constraints/extractor/rewriter/validator each return neutral values with one neutral-returns test per file |
| 14 | Manifest hygiene: workspace-pinned deps, `span-locations` load-bearing, no version literals | PASS | `crates/lang-rust/Cargo.toml` deps all `.workspace = true`; root `Cargo.toml` gains verbatim `syn = { version = "2", features = ["full"] }`, `proc-macro2 = { version = "1", features = ["span-locations"] }`, `unicode-ident = "1"`; member appended after `"crates/lang-scala"` |
| 15 | Cargo.lock purely additive | PASS | `git diff "1dd042d..HEAD" -- Cargo.lock` = one +13 package block `ariadne-lang-rust`, `syn 2.0.119` referenced (pin pre-existing), zero deletions, no existing entry touched |
| 16 | Split trigger: ~1231 changed lines vs ~1350 ceiling | PASS | `git show --stat "fe34e23"` = `16 files changed, 1242 insertions(+), 2 deletions(-)`; minus 13 Cargo.lock machine bytes = 1229/2 (~1231 changed lines with journal 138); under ceiling, split proposal not triggered, nothing moved to R-4 |
| 17 | Duty-1 completeness (R-2 verdict APPROVED + ledger clause in `3a7ec48`) | PASS | `git show --stat "3a7ec48"` = 2 files (`tasks/rust-language-r2-verdict.md` +244, `tasks/milestone_ledger.md` 1-line flip); `tasks/rust-language-r2-verdict.md:3` `Verdict: APPROVED`; ledger:62 clause verbatim: `R-2 LANDED 3d4dfae APPROVED 2026-10-26, census anchor 1855 passed / 0 failed / 13 ignored / 119 suites @ 1dd042d, R-3 dispatched.` |
| 18 | RED-SKIP pre-authorization verified in handoff text (not trusted) | PASS | Handoff `tasks/rust-language-r3-handoff.md` RGB item 1 reads `Red - JUSTIFIED RED-SKIP (SM-1 / W-1 / CW-2 precedent trio, cite all three)`; all three citations verify on disk (sm1 journal :24, w1 journal :14, cw2 journal :160); falsifiability statement present in journal |

### Not run / refused (explicit statement)

- `python tools/testrun.py --baseline` (bare acceptance-pass form): refused
  with `usage error: missing value for --baseline` - the F9-errata FILE form,
  superseded track-wide; the governing fresh `--distill` census is criterion
  4.
- `python tools/ui_smoke.py`: not required by this hand (no UI/runtime-surface
  bytes; the crate is unregistered); NOT run.
- `python tools/golden_guard.py --base <rev>`: tool not landed; substituted by
  the handoff's prose byte-check (criteria 9/10 diff proofs + census
  UNEDITED oracle).
- Two commands refused by the catch-all deny rule (verbatim
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
  a chained `git log; echo; git status` (substituted by the dedicated git
  tools) and `rg` (not on the allowlist; substituted by the grep tool). No
  expected gate was skipped.

## R-4 - registration hop: RustPlugin registered, discovery flipped, A1-A4+A7, fixture rust-basic, discovery suite

Chain from start rev `750ff8d`: duty 1 `cb5e0f4` + landing `ec552bd` (ONE
registration commit, 16 files, +785/-31 measured via `git show --numstat`) +
journal tail `a9b6864` (all implementer; the only touch-set extension is the
declared deviation-1 pair `crates/pipeline/Cargo.toml` + `Cargo.lock`,
ACCEPTED as a sanctioned rider). Verdict: APPROVED - see
`tasks/rust-language-r4-verdict.md`, incl. FIXTURE APPROVED for
`tests/fixtures/rust-basic/` (L3 path discharged; dated ledger note rides
R-5 duty 1). All lines below are the reviewer's own fresh runs (2026-10-26,
HEAD `a9b6864`, tree clean), canonical lines quoted verbatim.

| # | Acceptance criterion | Result | Evidence line (verbatim) |
|---|----------------------|--------|--------------------------|
| 1 | cli green WITH A1-A4; passed = baseline + 0 (handoff gates 1/4) | PASS | `182 passed / 0 failed / 8 ignored / 8 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-cli`); amended test names green UNEDITED-except-named (`default_registry_enums_cpp_then_scala`, `parse_language_arg_rejects_unknown_values`, `pin_parse_language_arg_exact_accepted_set_not_case_insensitive`, `pin_plan_backend_honest_errors_unclaimed_target_and_capability_off`, `planning_hooks_answer_honestly_for_paths_no_plugin_claims`) |
| 2 | Discovery suite green (gate 3) | PASS | `7 passed / 0 failed / 0 ignored / 1 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-integration-tests --test rust_language_discovery_test`) |
| 3 | m19 A7 green, rest UNEDITED (gate 6) | PASS | `5 passed / 0 failed / 0 ignored / 1 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-integration-tests --test m19_scala_discovery_test`); range diff for m19 = exactly the one-hunk 1-line vec literal at :217 |
| 4 | Pipeline green UNEDITED except the register line (gate 5) | PASS | `46 passed / 0 failed / 4 ignored / 9 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-pipeline`) - identical to the R-3 UNEDITED figures |
| 5 | lang-rust green incl. renamed caps pin + new detect pin | PASS | `24 passed / 0 failed / 0 ignored / 2 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-lang-rust`); relay's `7/0/0/1` is the lib target alone - census attribution of the +1 pin holds either way |
| 6 | Full census +8/+1-suite attributed by name vs anchor `1878/0/13/121 @ eeddfed` (gate 8) | PASS | `1886 passed / 0 failed / 13 ignored / 122 suites` (exit 0, `python tools/testrun.py --distill`, capture 3274 lines); +8 = the 7 discovery-suite tests named at source + in-src `rust_plugin_answers_detect_language_for_rs_paths` (caps-pin rename 1-for-1, net zero); +1 suite = `rust_language_discovery_test`; ignored flat 13, failed 0 |
| 7 | INV-9 grep witness re-run (gate 7) | PASS | grep regex `unclaimed\.rs` over `crates/` -> `No files found` (zero count; oracle of shape INV-9) |
| 8 | Hygiene exit 0 (gate 9) | PASS | `python tools/hygiene_check.py` -> `hygiene exit 0`; all output warn-only R3 on pre-existing files; zero lines name any R-4 path; cli/registry.rs 784 lines, pipeline registry 42, suite 425, plugin.rs 297 |
| 9 | L12 encoding | PASS | `python tools/encoding_guard.py --all` -> `encoding exit 0` (silent) |
| 10 | Tool unit suites (CI form) | PASS | `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 296 tests in 29.899s` / `OK` |
| 11 | Reviewer acceptance bench, RUN fresh (kickoff fully-run stance; pipeline/src + cli/src touched) | PASS | `python tools/distill.py -- python tools/bench_compare.py` -> `### generic PASS (exit 0)`; `bench compare: 3 run(s), no regressions past 1.25x baseline`; zero WARN lines (capture `target/distill-capture.txt`) |
| 12 | Registration order + caps flip literal byte-checked (shape sec. 3.2/3.4, handoff items 1/4) | PASS | register line appended LAST (C++, Scala, Rust); pipeline diff = doc + one line only, first-wins/`unit_shaper_for_repo` marker rule intact; caps literal `discovery: true` ONLY + `requires_toolchain: false` (plugin.rs:147-158); error text byte-exact; modeling literals byte-identical (no modeling hunk) |
| 13 | Red lands in the SAME commit as its green (protocol a/b/c) | PASS | Range = 3 commits; the two non-landing commits touch only `tasks/` files; every red-turning edit is inside `ec552bd`; journal captures the (a) `2 passed / 5 failed` and (b) `172 passed / 2 failed` red runs with the named assertions |
| 14 | Frozen locks L1-L9 + Reviewer MUSTs | PASS | zero-diff measured for `crates/repo`, `crates/lang-cpp`, `crates/lang-scala`, `tools`, `tests/golden`, `tests/fixtures/basic`, `tests/fixtures/expensive`, `m24_language_identity_test.rs`, `crates/repo/src/identity.rs`; `CyclePolicy::Reject` planner literals untouched (cli diff = src/registry.rs only); `tests/integration/Cargo.toml` in NO hunk |
| 15 | Deviation 1 (pipeline Cargo.toml + Cargo.lock) | ACCEPTED (sanctioned rider) | no other home for the registration dep edge; M19.3 Scala precedent at pipeline Cargo.toml:29-32; Cargo.lock hunk purely additive (+1 line, -0); declared journal-first |
| 16 | Deviation 2 (INV-4 positional bytes via live production writer) | ACCEPTED (on the merits) | ar27 locks verified UNEDITED with the cited names (`discriminator_selects_the_wire_arm_by_format`, `positional_form_names_an_open_id_through_the_escape_arm`, repo `ar27_old_bytes_lock_test.rs`) and the identity.rs arm docs; the suite searches the REAL `files.bin` for `[3,0,0,0][4,0,0,0,0,0,0,0]"Rust"` and decodes it back - stronger than a synthetic serialize; m24 verified to carry no bincode mechanism |
| 17 | Fixture rust-basic vs docs/adding-a-language.md Step 4 | APPROVED | all four properties verified at source (build marker `Cargo.toml` at root; `src/` sources in BOTH module forms; `tests/cli_smoke.rs` exclusion; `vendor/legacy/legacy.rs` external/frozen side) + macro caveat specimen; 8 files pure ASCII; path-diff proof: only rust-basic additions, basic/expensive zero new bytes |
| 18 | Duty-1 completeness (R-3 verdict + ledger clause in `cb5e0f4`) | PASS | `git show --stat cb5e0f4` = 2 tasks files; ledger clause verbatim: `R-3 LANDED fe34e23 APPROVED 2026-10-26, census anchor 1878 passed / 0 failed / 13 ignored / 121 suites @ eeddfed, R-4 dispatched (registration hop).` |

### R-4 not run / refused (explicit statement)

- `python tools/testrun.py --baseline` (bare): refused with `usage error:
  missing value for --baseline` - the F9-errata FILE form, superseded
  track-wide; criterion 6 is the governing fresh `--distill` census.
- `python tools/testrun.py` raw (non-distill): crashed this console's cp1252
  echo (`UnicodeEncodeError: 'charmap' codec can not encode ...` at
  `testrun.py:262`); all gates run via the handoff's prescribed `--distill`
  entry point instead - identical count semantics.
- `python tools/ui_smoke.py`: not required by this hand (no UI/runtime-
  surface bytes); NOT run.
- `python tools/golden_guard.py --base <rev>`: tool not landed; substituted
  by the handoff's prose byte-check (criteria 3/12/14 diff proofs).
- Catch-all-deny refusals (verbatim
  `{"permission":"bash","pattern":"*","action":"deny","source":"global"}`):
  chained `git log; git status`, `git status --short`, `python -c` line
  counter, and one `Select-Object`-piped unittest run (re-ran un-piped -
  succeeded). All substituted by dedicated tooling or un-piped equivalents.
  No expected gate was skipped.

## R-5 - syn analyzer, fingerprint "rust-1", semantic_index flip, span truth

Chain from start rev `c54035d`: duty 1 `80a2e28` + landing `673b22b`
(implementer, 7 files +1395/-40) + orchestrator rider `8622b2c` (one-literal
amendment of the superseded R-4 flip-lock, m23 precedent - adjudicated
rider-of-record, ACCEPTED: `5+/3-`, one literal `false->true` + doc-comment +
assert message, full-struct byte-exact discipline intact). Verdict: APPROVED
- see `tasks/rust-language-r5-verdict.md`. All lines below are the
reviewer's own fresh runs (2026-10-26, HEAD `8622b2c`, tree clean),
canonical lines quoted verbatim.

| # | Acceptance criterion | Result | Evidence line (verbatim) |
|---|----------------------|--------|--------------------------|
| 1 | lang-rust green at landing, baseline 24 - 1 superseded stub pin + 19 attributed (handoff gate 3) | PASS | `43 passed / 0 failed / 0 ignored / 2 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-lang-rust`); +4 analyzer / +1 plugin fingerprint pin / +8 parser_items / +7 parser_uses, all counted by name at source |
| 2 | New suite green incl. literal-exact span pins (gate 4) | PASS | `11 passed / 0 failed / 0 ignored / 1 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-integration-tests --test rust_language_semantic_index_test`); all 9 fixture span literals re-read by the reviewer against the frozen bytes - every pin exact, incl. the attribute-immune `User (4,12)` and the handoff's named `mod models; (7,5)` oracle |
| 3 | Discovery suite green post-rider (flip-lock amendment) | PASS | `7 passed / 0 failed / 0 ignored / 1 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-integration-tests --test rust_language_discovery_test`) |
| 4 | m21 Scala witness green UNEDITED (gate 5; cache-digest non-perturbation) | PASS | `5 passed / 0 failed / 0 ignored / 1 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-integration-tests --test m21_scala_semantic_index_test`); m21 file in NO hunk of the range |
| 5 | Full census reconcile vs duty-1 anchor `1886/0/13/122 @ a9b6864`, failed 0, suites +1 (gate 6) | PASS | `1916 passed / 0 failed / 13 ignored / 123 suites` (exit 0, `python tools/testrun.py --distill`, bare fail-fast form, single run, capture 3310 lines); 1886 + 19 (lang-rust net) + 11 (new suite) + 0 (rider restores the flip-touched lock) = 1916, attribution verified independently; the implementer's `1915/1/13/123` reconciles by name - their lone failure was the stale lock now rider-fixed |
| 6 | Transient fixture-copy flake re-check (cli extraction_apply end-to-end) | TRANSIENT-NOTED | fresh full census passed it green in one bare 123-suite run; not reproduced; pre-existing Windows copy race class copying the untouched `basic` fixture, outside this hand's bytes; recorded for closeout pattern census |
| 7 | Hygiene exit 0 (gate 7) | PASS | `python tools/distill.py -- python tools/hygiene_check.py` -> `### generic PASS (exit 0)` (capture 173 lines); all rows warn-only R3, zero name a file of this hand; analyzer 143 / parser_items 338 / parser_uses 219 / suite 425 lines - pre-split ceilings held |
| 8 | L12 encoding `encoding_guard.py --all` exit 0 (gate 8/L12) | PASS | `python tools/distill.py -- python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)` (capture 0 lines) |
| 9 | Tool unit suites (CI form) | PASS | `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 296 tests in 32.080s` / `OK` |
| 10 | Reviewer acceptance bench, RUN fresh (kickoff sec. 4 fully-run; lang-rust/src touched) | PASS | `python tools/distill.py -- python tools/bench_compare.py` -> `### generic PASS (exit 0)`; `bench compare: 3 run(s), no regressions past 1.25x baseline`; zero WARN lines in capture (PR-2 moot); no baseline/corpus change (D-3) |
| 11 | Commit-vs-touch-set | PASS | `git diff --name-only c54035d..HEAD` = 10 paths: 9 inside the declared touch-set; the tenth (`rust_language_discovery_test.rs`) is the rider file, the ONLY `tests/**` hunk outside fixtures - rider-of-record ACCEPTED, implementer never touched it; landing stat = the six contract paths + journal (gate 8 "Journal inside") |
| 12 | Caps literal byte-exact: discovery + semantic_index true ONLY, requires_toolchain false, locked by independent literals | PASS | implementation `plugin.rs:147-157`; THREE agreeing byte-literals: in-src `capabilities_match_the_semantic_index_flip_literal`, integration-side `capabilities_lock_the_semantic_index_flip_literal` (suite :165-187, registry seam), amended rider lock (discovery test :294-315) |
| 13 | Fingerprint `"rust-1"` in the FIRST analyzer-shipping commit, pinned twice; CURRENT_ANALYSIS_VERSION read-proof | PASS | `"rust-1".to_string()` override + in-src `analysis_fingerprint_is_rust_1` + integration `registered_plugin_fingerprint_is_rust_1`, all in `673b22b`; read-check: `crates/index/src/cache.rs:30` = `"2.2.0"` (handoff's 2.1.0 quote STALE - registered M28 T4 DP-2 exception; m21 pins 2.2.0); zero bytes under `crates/index` |
| 14 | syn mechanism + span truth (shape decision 2 / INV-7 / RISK-SYN-SPAN) | PASS | `syn::parse_file` + `Spanned`/`Span::start()` discipline end to end; hand-rolled-scanning grep (`.lines()`/`char_indices`/`find`/`split`/`matches`) over `parser_*.rs` = zero hits; macro-fabrication audit: every pinned symbol has written source, `generated_by_macro` pinned absent |
| 15 | Frozen locks incl. fixture read-only + Cargo.lock untouched | PASS | `git diff c54035d..HEAD -- Cargo.toml Cargo.lock crates/index crates/repo crates/cli crates/pipeline crates/lang crates/lang-cpp crates/lang-scala tests/fixtures tests/golden tools` EMPTY; m00/m18-m23/ar27 green UNEDITED inside the census; suite copies `rust-basic` to tempdirs (m19/m21 idiom), writes nothing to the fixture |
| 16 | Deviations D1-D5 | ACCEPTED | D1 plan erratum (2.2.0 verified, zero bytes); D2 protocol-implied red-suite run; D3 report-not-fix correct, rider resolved it; D4 retirement-of-superseded (superseded) - positively pinned twice since; D5 verified design judgment (seam receives no file-position input), parked to R-6 resolver / R-9 isolation |
| 17 | Duty-1 completeness (`80a2e28`) | PASS | stat = verdict +237 + 1-line ledger flip; BOTH clauses verbatim (`R-4 LANDED ec552bd APPROVED 2026-10-26, census anchor 1886 passed / 0 failed / 13 ignored / 122 suites @ a9b6864, R-5 dispatched (analyzer hop).` + `fixture rust-basic APPROVED 2026-10-26 at ec552bd via R-4 verdict - L3 approval path discharged.`); memory mirror `ledger.milestone.rust-language` verified present |

### R-5 not run / refused (explicit statement)

- `python tools/testrun.py --baseline` (bare): refused - `usage error:
  missing value for --baseline`; F9-errata FILE form, superseded track-wide
  (no baseline file exists for this track). Criterion 5 is the governing
  fresh `--distill` census (handoff gate 6's prescribed form).
- `python tools/ui_smoke.py`: not required by this hand (no UI/runtime-
  surface bytes); NOT run.
- `python tools/golden_guard.py --base <rev>`: tool not landed; substituted
  by the handoff's prose byte-check (criteria 11/15 proofs).
- Catch-all-deny refusals (verbatim `{"permission":"bash","pattern":"*",
   "action":"deny","source":"global"}`): one chained `git show --stat a;
   echo ...` (substituted by three individual `git show --stat` runs), two
   `python -c` probes, one `findstr`, one `Get-Content .Count`, one chained
   `$LASTEXITCODE` probe (substituted by Read-tool line totals and distiller
   `PASS (exit 0)` lines). No expected gate was skipped.

## R-6 - resolver (mod tree + use paths), dependency_graph flip, graph-shape truth

Chain from start rev `f18f57f`: duty 1 `bd44e41` + landing `baf24f8`
(implementer, 4 files +1293/-34) + orchestrator rider `470ae22`
(three-witness amendment, 2 files +21/-13 - adjudicated rider-of-record,
ACCEPTED: two one-literal flip-lock evolutions + the warm-reindex sentinel
relocation, both the shapes the implementer's board ASK option (a) proposed,
m23 precedent). Verdict: APPROVED - see `tasks/rust-language-r6-verdict.md`.
All lines below are the reviewer's own fresh runs (2026-10-26, HEAD
`470ae22`, tree clean), canonical lines quoted verbatim.

| # | Acceptance criterion | Result | Evidence line (verbatim) |
|---|----------------------|--------|--------------------------|
| 1 | lang-rust green, baseline 43 - 1 superseded stub pin + 14 named resolver tests (handoff gate 4) | PASS | `56 passed / 0 failed / 0 ignored / 2 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-lang-rust`); 43 - 1 + 14 = 56 counted by name at source; in-src pin `capabilities_match_the_dependency_graph_flip_literal` (renamed byte-exact) green |
| 2 | New suite green incl. the RISK-CYCLE report-data assertion (gate 3) | PASS | `9 passed / 0 failed / 0 ignored / 1 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-integration-tests --test rust_language_dependency_graph_test`); mod-tree both layouts at written spans, use-facts verbatim, vendor boundary marker-only, cycle informational via the existing `cycle_policy_by_file`-shaped lookup with zero core edits |
| 3 | m20 model suite green UNEDITED (gate 5) | PASS | `5 passed / 0 failed / 0 ignored / 1 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-integration-tests --test m20_scala_dependency_graph_test`); m20 file in no hunk of `f18f57f..470ae22` |
| 4 | Rider-touched suites green (relay verified personally) | PASS | semantic_index: `11 passed / 0 failed / 0 ignored / 1 suites`; discovery: `7 passed / 0 failed / 0 ignored / 1 suites` (both exit 0, `--distill` scoped) |
| 5 | m18.7 SCC-sharing oracle green UNEDITED (shape sec. 6) | PASS | `6 passed / 0 failed / 0 ignored / 1 suites` (exit 0, `python tools/testrun.py --distill -p ariadne-cluster --test planner_injection`); `crates/cluster` zero-diff in the range |
| 6 | Full census reconcile vs duty-1 anchor `1916/0/13/123 @ 8622b2c`, failed 0, suites +1 (gate 6) | PASS | `1938 passed / 0 failed / 13 ignored / 124 suites` (exit 0, bare fail-fast `python tools/testrun.py --distill`, capture 3356 lines); 1916 + 13 (lang-rust net) + 9 (new suite) = 1938; pre-rider the three superseded witnesses sat failed (journal `1935 passed / 3 failed / 13 ignored / 124 suites`) and the rider turned all three green without changing the test count; +1 suite = `rust_language_dependency_graph_test`; ignored flat 13; cross-check 1886 @ a9b6864 + 30 (R-5) + 22 (R-6) = 1938 |
| 7 | Caps literal byte-exact: EXACTLY three true bits, five independent literals agreeing | PASS | plugin.rs impl + in-src pin + new integration lock + the two rider-amended older locks ALL carry `discovery: true, dependency_graph: true, semantic_index: true`, the five other bits false, `requires_toolchain: false`; all five witnesses green in reviewer runs (criteria 1/2/4) |
| 8 | Commit-vs-touch-set (three commits, oldest first) | PASS | `bd44e41` = verdict + 1-line ledger flip; `baf24f8` = the four touch-set paths only; `470ae22` = the two named rider files (orchestrator's sanctioned band); `git diff --name-only f18f57f..470ae22` = 8 paths, nothing else |
| 9 | Frozen locks incl. fixture bytes UNTOUCHED + `crates/pipeline/src/lib.rs` no hunk | PASS | `git diff --name-only f18f57f..470ae22 -- tests/fixtures tests/golden Cargo.toml Cargo.lock crates/repo crates/pipeline crates/cli crates/lang-cpp crates/lang-scala crates/index crates/graph crates/report crates/validate crates/artifact crates/cluster crates/transform crates/scheduler crates/ui crates/worker-core tools` -> EMPTY; the sentinel is runtime envelope mutation in `tempfile::TempDir` copies only (suite source: all writes under the temp root) |
| 10 | Merged-closure root cause + sentinel relocation = correct-behavior accommodation, cache-serve witness intact | ACCEPTED (rider) | `crates/worker-core` zero-diff proves zero core edits (tu_store consumed the stub all along and now walks real resolver edges); `src/macro_specimen.rs` declared nowhere in the fixture (grep: no `mod`/`use` references it - the fixture carries zero top-level `use` statements) and holds one written symbol; sentinel id exists only in the stored envelope JSON (a re-parse loses it) and the `surface_written` precondition still fails on an empty surface |
| 11 | D6 stash breach verification | MINOR/RECORD (verdict finding 1) | no stash remains: `git rev-parse --verify refs/stash` -> `fatal: Needed a single revision` (no `refs/stash` at all); plugin.rs landing hunk bounded to the caps/pin/doc lines (28 lines, no EOL churn); suites re-ran green on the restored bytes in reviewer runs; RULING: minor/record-class process breach, not a blocker (single in-touch-set file, popped immediately, self-reported, zero residue); recurrence = blocker |
| 12 | Resolver honesty vs the Scala IV-5/D10 precedent (syn per shape decision 2, no hand-rolled scanning; rel_path owns cross-file truth per the R-5 D5 relay) | PASS | `syn::parse_file` + `parser_uses` facts verbatim; marker-first External, zero-match External (never Unresolved - out-of-repo resolves to NOTHING), sorted Wildcard, matched-under-marker External, `directive_edge_kind() -> Some(Import)` - each mirrored 1-for-1 and test-pinned in-src; Red list structurally true (start-rev stub empty vec + trait default `ResolveOutcome::Unresolved` at crates/lang/src/resolver.rs:159) |
| 13 | Hygiene exit 0 (gate 7) | PASS | `python tools/distill.py -- python tools/hygiene_check.py` -> `### generic PASS (exit 0)` (capture 173 lines); all rows warn-only R3, none naming resolver.rs (542 lines) / plugin.rs / the new suite (489 lines) / the journal |
| 14 | L12 encoding | PASS | `python tools/distill.py -- python tools/encoding_guard.py --all` -> `### generic PASS (exit 0)` (capture 0 lines) |
| 15 | Tool unit suites (CI form) | PASS | `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 296 tests in 29.323s` / `OK` |
| 16 | Reviewer acceptance bench, RUN fresh (kickoff fully-run; lang-rust/src touched - this hand changes analysis payloads flowing into indexing) | PASS | `python tools/distill.py --timeout 3000 -- python tools/bench_compare.py` -> `### generic PASS (exit 0)`; `bench compare: 3 run(s), no regressions past 1.25x baseline`; zero WARN lines so the PR-2 record-not-attribute stance was moot; no baseline/corpus change (D-3) |
| 17 | Duty-1 completeness (`bd44e41`) | PASS | stat = verdict +219 + 1-line ledger flip; clause verbatim: `R-5 LANDED 673b22b (+rider 8622b2c) APPROVED 2026-10-26, census anchor 1916 passed / 0 failed / 13 ignored / 123 suites @ 8622b2c, R-6 dispatched (resolver hop).`; memory mirror `ledger.milestone.rust-language` verified holding the same state |
| 18 | ASK 2 disposition (shape sec. 8 docs riders) | RECORDED | shape sec. 8 "five docs riders land at R-6" conflicts with the handoff touch-set (which omits the path) - touch-set law governs; riders CONFIRMED at R-10 (planner hand already carries "5 dated guide riders"); no action R-7..R-9; verdict finding 4 asks the planner to retire/date-annotate the stale line |

### R-6 not run / refused (explicit statement)

- `python tools/testrun.py --baseline` (bare): refused - `usage error:
  missing value for --baseline`; F9-errata FILE form, superseded track-wide
  (no baseline file exists for this track; probing the FILE form confirmed
  the pre-existing cp1252 `_echo` crash noted in the R-4 addendum - carried
  as verdict finding 3, not attributable to this hand). Criterion 6 is the
  governing fresh `--distill` census (handoff gate 6's prescribed form), and
  it matches the anchor exactly.
- `python tools/testrun.py --distill` Red run at the stub: structurally
  unreproducible for a reviewer at landing HEAD; honesty verified at source
  instead (criterion 12) - the start-rev stub's empty vec and the trait's
  `Unresolved` default make every journalled `left:` value true.
- `python tools/ui_smoke.py`: not required by this hand (no UI/runtime-
  surface bytes; `crates/ui` zero-diff); NOT run.
- `python tools/golden_guard.py --base <rev>`: tool not landed; substituted
  by the handoff's prose byte-check (criteria 3/8/9 name-only proofs).
- Catch-all-deny refusals (verbatim `{"permission":"bash","pattern":"*",
  "action":"deny","source":"global"}`): one chained
  `git show --stat x3; git stash list` (substituted by three individual
  `git show --stat` runs + `git rev-parse --verify refs/stash` - criterion
  11), one `git grep` (grep tool), one `Set-Content` chain (Write tool), one
  scratchpad write outside the reviewer edit grant (dropped as unnecessary),
  one `> $null` chained exit-code probe (substituted by distiller
  `PASS (exit 0)` lines - criteria 13/14/16). No expected gate was skipped.
