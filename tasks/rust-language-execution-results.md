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
