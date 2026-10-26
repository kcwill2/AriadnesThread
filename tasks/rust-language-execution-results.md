# rust-language - Execution Results (per-hand, reviewer acceptance)

Milestone: `rust-language` (UNREGISTERED post-M28 track, NO M-number). This
file accumulates the reviewer-run acceptance evidence per hand. Hands gated so
far: R-1.

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
