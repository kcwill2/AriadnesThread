# rust-language - Execution Results (per-hand, reviewer acceptance)

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). This
file accumulates the reviewer-run acceptance evidence per hand. Hands gated so
far: R-1, R-2.

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
