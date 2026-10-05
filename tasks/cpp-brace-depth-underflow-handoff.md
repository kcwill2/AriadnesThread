# cpp-brace-depth-underflow T1 Handoff - C++ raw-string lexing fidelity + parser brace-depth floor

Milestone key `cpp-brace-depth-underflow` (M-number + dates assigned by
milestone-lifecycle kickoff; do not guess them). Single-handoff milestone:
planner sizing verdict in `docs/specs/cpp-brace-depth-underflow.md` section 4.6
and `tasks/milestone_ledger.md` row - split rejected (layer-1-only still
underflows on residual drift; layer-2-only leaves scoping incorrect-by-accident).

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `docs/specs/cpp-brace-depth-underflow.md` - the canonical spec. You create
   it verbatim in Step 0 below; then read it in full. It is the byte-identical
   copy of `.kilo/plans/1791144879664-cpp-brace-depth-underflow.md` (302 lines,
   pure ASCII - planner-verified). Sections 3.2 (fix layers), 3.3 (invariants
   I1-I7), 4.1 (contracts), 4.2 (module map) are binding.
3. Model files to mirror in structure/doc style:
   `crates/lang-cpp/src/tokenizer/characterization_tests.rs` (303 lines - test
   style, `expect_token_kinds` helper shape) and
   `crates/lang-cpp/src/parser/characterization_tests.rs` (376 lines - direct
   `ParserCtx` harness: `ParserCtx::new` + `handle_brace_open/close` called
   directly, e.g. lines 236-270).
4. Exact field names / literals sources:
   `crates/lang-cpp/src/analyzer.rs:14` -
   `CppSemanticAnalyzer::analyze_source(tu_file_id: FileId, source_code: &str, context_name: &str) -> TuAnalysis`;
   `FileId::from_relative_path("...")` and context literal `"Debug|x64"` as used
   in `analyzer.rs` tests; `crates/lang-cpp/src/tokenizer.rs:69-74` (the `"` /
   `'` dispatch branch where the raw-string probe attaches) and
   `:152-180` (`scan_string_literal` - the line/col tracking model to mirror);
   `crates/lang-cpp/src/parser.rs:56-72` (`handle_brace_open` /
   `handle_brace_close` - the exact code being floored).

## Step 0 - canonical spec copy (first commit, isolated)

Copy `.kilo/plans/1791144879664-cpp-brace-depth-underflow.md` ->
`docs/specs/cpp-brace-depth-underflow.md` VERBATIM using Read + Write tools
only (repo UTF-8 hazard: never Set-Content/Out-File/`>`/Get-Content-replace;
never PowerShell pipes for file content). The planner verified the source is
pure ASCII, 302 lines; the copy must be byte-identical (the reviewer byte-diffs
it against the source artifact). Commit it alone:
`python tools/commit_scoped.py --scope "docs/specs/cpp-brace-depth-underflow.md" -m "docs(specs): add cpp-brace-depth-underflow canonical spec (verbatim copy of architect artifact)"`.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/cpp-brace-depth-underflow-handoff-journal.md` (checkpoint journal; always in-set)
- `docs/specs/cpp-brace-depth-underflow.md` (Step 0 verbatim copy only - no editing of spec content)
- `crates/lang-cpp/src/tokenizer.rs` (raw-string scan + dispatch branch + one `mod raw_string_tests;` line; 223 -> ~290 lines)
- `crates/lang-cpp/src/tokenizer/raw_string_tests.rs` (NEW - I4 fixtures)
- `crates/lang-cpp/src/parser.rs` (stray-close floor + one `mod brace_depth_tests;` line; 494 -> ~510 lines)
- `crates/lang-cpp/src/parser/brace_depth_tests.rs` (NEW - I1/I2/I5 fixtures)

You MUST NOT touch: `crates/lang-cpp/src/analyzer.rs` (dispatch loop is
already symmetric - see spec 2.1); BOTH `characterization_tests.rs` files
(frozen behavioral snapshot - must stay green UNEDITED, diff must stay empty);
`symbol_locator*`; `source_extractor.rs`, `shaper.rs`, `resolver.rs`,
`plugin.rs`, `planning.rs`, `directive_parser.rs`, `dependency_rewriter.rs`,
`backend.rs`, `lib.rs`; any `Cargo.toml`/`Cargo.lock` (no new dependencies;
lockfile must stay untouched); `crates/lang-cpp/tests/` (does not exist - do
not create it); `tests/golden/**`; `tests/fixtures/**`; `tests/integration/**`;
`crates/index/**`, `crates/pipeline/**`, `crates/cli/**`, `crates/lang-api/**`.
No `git stash`, no branch change.

## What to build

Two fix layers, one crate, per spec 3.2 - exact contracts are the fenced code
blocks in spec 4.1 (reproduced here for zero-ambiguity):

**Module A - `tokenizer.rs` raw-string lexing.** At a `"` whose immediately
preceding chars form a non-empty `[A-Za-z0-9_]+` run ending in `R` (covers
`R"`, `u8R"`, `uR"`, `UR"`, `LR"`), read the delimiter run `[A-Za-z0-9_]*` up to
`(`; then consume through `)delim"` inclusive, tracking line/col like
`scan_string_literal`; emit NO tokens. Signature (spec 4.1):

```rust
fn scan_raw_string(chars: &[char], quote_at: usize,
                   i: &mut usize, line: &mut usize, col: &mut usize) -> bool
```

Returns `false` with position untouched when no legal delimiter+`(` header
exists (rejects `R""` oddities and identifier+quote adjacency) or when no
closing `)delim"` exists before EOF - caller then falls back to
`scan_string_literal` (unterminated raw swallows to EOF, matching the locked
`tokenizer/characterization_tests.rs:116` unterminated-string behavior;
document this in the fn doc-comment). No backslash escaping inside raw content
(spec 2.2, `gtest_PromQLParser.cpp:1833` class). Dispatch attaches at
`tokenizer.rs:69-74`: probe raw-string first, fall through to the existing
string scan when the probe returns false. Flush word/punct before scanning,
exactly as the existing branch does.

**Module B - `parser.rs` stray-close floor.** `handle_brace_close` (signature
unchanged, `pub(crate)` visibility preserved - the characterization test calls
it directly at `parser/characterization_tests.rs:251`):

```rust
pub(crate) fn handle_brace_close(&mut self) {
    if self.brace_depth == 0 {
        // Stray close (unbalanced TU, e.g. preprocessor-discarded branch):
        // defined degradation - end every open scope, depth stays 0.
        self.current_class = None;
        self.current_function = None;
        return;
    }
    self.brace_depth -= 1;
    // ...existing scope-exit checks, unchanged...
}
```

`handle_brace_open` stays `self.brace_depth += 1` unchanged. Balanced streams
never reach the floor (structural argument, spec I3). No new public items, no
error-type changes - the fix turns a panic into defined data (I5).

**Test helpers.** The new test modules get their own local `expect_token_kinds`
helper (the characterization files are frozen and their helpers are private to
those modules; this one small helper duplication is the ONLY sanctioned
duplication). Parser-side tests drive `CppSemanticAnalyzer::analyze_source`
(I1/I2/I5) and, where brace-bookkeeping is pinned directly, the
`ParserCtx::new` + direct-call pattern from `parser/characterization_tests.rs`.
Fixtures are inline string literals - NEVER new files under
`tests/fixtures/**` (M18 lock).

## RGB protocol (per module, in this file order; modules are SEQUENCED A -> B,
not parallel - B's raw-string fixture in the I1 suite leans on A's lexer)

1. **Red A** - write `tokenizer/raw_string_tests.rs` (I4 set below) + the
   `mod raw_string_tests;` line at `tokenizer.rs` end. Run gate 1. Expected
   failures (assertion failures, NOT compile errors):
   - `raw_string_braces_emit_no_tokens`:
     `expect_token_kinds("a R\"( }{ )\" b", &["word:a", "word:b"])` fails with
     leaked `punct:}` / `punct:{` tokens (quote pairing breaks: the raw content
     `"` closes the opened string early).
   - `delimited_raw_string_braces_emit_no_tokens`:
     `expect_token_kinds("a R\"d( } )d\" b", &["word:a", "word:b"])` fails the
     same way.
   - `multi_line_raw_string_advances_line_counter`: token `b` line is wrong
     (content lines leak as code instead of being tracked by the scan).
   - `unterminated_raw_string_swallows_to_eof`: `"a R\"( }{"` ->
     `[word:a]` fails today (content leaks).
   - `raw_quote_without_delim_falls_back_to_string_scan`:
     `expect_token_kinds("a R\"x\" b", ...)`-family oddity case (`R"` not
     followed by legal delimiter+`(`) passes PRE-fix and must pass POST-fix -
     it is a fallback lock, not a Red; that is expected and fine.
2. **Green A** - implement `scan_raw_string` + detection helper + dispatch
   branch; re-run until green. NEVER weaken an assertion to pass.
3. **Red B** - write `parser/brace_depth_tests.rs` (I1/I2/I5 set below) + the
   `mod brace_depth_tests;` line at `parser.rs` end. Run gate 1. Expected
   failures: the underflow fixtures PANIC with
   `attempt to subtract with overflow` inside `handle_brace_close`
   (`parser.rs:61`) - debug (overflow-checks on) `cargo test` panic is the
   Red signal; a panicking test compiles and runs, so this is a valid Red.
4. **Green B** - implement the floor; re-run until green.
5. **Refactor** - dedupe, honest names, doc comments (including the
   documented EOF-fallback rationale); re-run.

## Required tests (exact expected values)

`tokenizer/raw_string_tests.rs` (I4):
- `a R"( }{ )" b` -> `[word:a, word:b]`.
- `a R"d( } )d" b` -> `[word:a, word:b]`.
- multi-line raw (content spans >= 2 newlines): `b` token `line` equals the
  physical line of `b` in the source; braces inside content emit nothing.
- `R"(` unterminated at EOF: `"a R\"( }{"` -> `[word:a]` (swallow-to-EOF;
  mirrors the locked behavior at `tokenizer/characterization_tests.rs:116`).
- `R"` + non-legal delimiter/header (e.g. `R"x`, `R""`) falls back to the
  ordinary string scan (documented pre-fix behavior preserved).
- API.cpp:155 reproduction (spec 2.2):
  `char* s = "a" + R"("})" ;` region lexes with no brace/punct leak between
  the words - pin a minimal standalone form:
  `expect_token_kinds("f(x + R\"(\"})\" );", &["word:f", "punct:(", "word:x", "punct:+", "punct:)", "punct:;"])`.

`parser/brace_depth_tests.rs` (I1/I2/I5):
- I1 no-panic adversarial suite - each completes without panic under debug:
  `}}}`; `{}}`; a raw string with internal quotes and a stray `}` (the
  API.cpp-style line above wrapped in a function body); `R"(` unterminated at
  EOF; a `#if 0` fragment `void a() { }\n#if 0\n}\n#endif\nvoid b() { c(); }`;
  a stray-apostrophe fragment `char c = '; } void d() { e(); }`-style swallow.
- I2/I5 defined stray-close semantics:
  `analyze_source(FileId::from_relative_path("stray.cpp"), "} void f() { g(); }", "Debug|x64")`
  records `f` with `is_definition == true` AND a `call_site` with caller `f`
  callee `g` - scoping RESUMES after the stray instead of poisoning the TU
  (today's release symptom: depth wraps to `usize::MAX`, every later scope
  check passes, all later call sites vanish; today's debug symptom: panic).
- Direct-bookkeeping pin (mirror of the `characterization_tests.rs:251`
  pattern): `ParserCtx::new`, then `handle_brace_close()` at depth 0 twice:
  no panic, and a following `handle_brace_open()` + function definition still
  records normally.

## Hygiene (review-enforced)

Functions < 100 lines (`scan_raw_string` target <= 60, detection helper <= 25,
`handle_brace_close` stays well under 100), files < 1000 lines (ceilings at
risk: NONE numerically - `tokenizer.rs` 223 -> ~290 and `parser.rs` 494 -> ~510
are far under, but the characterization files must NOT grow - new tests live
in separate NEW files precisely so the 303/376-line frozen suites stay put),
no banned identifier names (tmp/res/foo/bar/test1), no single-letter locals,
intention-revealing identifiers, one responsibility per function, no source
line near 2000 chars. `python tools/hygiene_check.py` must stay exit 0.

## Frozen locks

| Lock | Proving check |
|---|---|
| `tests/golden/**` byte-locked; `UPDATE_GOLDENS=1` FORBIDDEN this milestone | reviewer: `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_golden_baseline_test` (G3) |
| `tests/fixtures/**` frozen; no new fixture files | review-gate touch-set check |
| `crates/lang-cpp/src/tokenizer/characterization_tests.rs` (303 lines) UNEDITED, green | `git show` per commit + gate G1 |
| `crates/lang-cpp/src/parser/characterization_tests.rs` (376 lines) UNEDITED, green | `git show` per commit + gate G1 |
| `m18_cpp_plugin_parity` 22/22 UNEDITED | reviewer: G2 |
| `m17_plugin_seam_test` 5/5 (architecture guard, deps inward) | reviewer |
| M23 verbatim `symbol_locator` tests UNEDITED green (raw strings emit zero tokens -> offset-slicing locator unaffected) | `git show` + G1 |
| No new public items, no new deps, `TokenKind`/`Token`/`RawToken`/`tokenize`/`tokenize_raw`/`ParserCtx` field set + visibility, `CppSemanticAnalyzer` API, `ariadne-lang-api` DTOs, `TuAnalysis` shape frozen | spec 4.1 "Frozen surfaces"; reviewer diff inspection incl. `brace_depth` decrement paths (I2 structural: floor is the ONLY decrement guard) |
| UTF-8/mojibake gate; all authored content pure ASCII | `python tools/encoding_guard.py --staged` (pre-commit); reviewer `--all` |

Until `tools/golden_guard.py` lands (deferred finding): manual byte-identical
discipline - do NOT touch the locked paths above at all.

## Gates (exact commands, timeout 600000 ms each; scoped only - workspace-wide
gates are the reviewer/orchestrator's)

1. `python tools/testrun.py --distill -p ariadne-lang-cpp` - run at each of:
   Red A, Green A, Red B, Green B, final (gate G1). Canonical summary line
   `N passed / M failed / K ignored / S suites` and the digest header must
   appear verbatim in the report; red first, then green; final verdict =
   `0 failed`, exit 0.
2. `python tools/hygiene_check.py` - exit 0 (G5).
3. `python tools/encoding_guard.py --staged` - exit 0 before committing.
4. Commit with
   `python tools/commit_scoped.py --scope "crates/lang-cpp/src/tokenizer.rs" --scope "crates/lang-cpp/src/tokenizer/raw_string_tests.rs" --scope "crates/lang-cpp/src/parser.rs" --scope "crates/lang-cpp/src/parser/brace_depth_tests.rs" --scope "tasks/cpp-brace-depth-underflow-handoff-journal.md" -m "fix(lang-cpp): lex raw string literals and floor brace depth at stray closes"`
   (never `git add .`; Step 0 commit scoped to the spec file alone).

Expected implementer gate-command total: <= 10 invocations (4 Red/Green runs +
1 final G1 + hygiene + encoding + 2 scoped commits). Count them in the report.

Reviewer-hop gates (NOT yours; the reviewer/orchestrator runs these - do not
run them in your session): G2 `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` (22/22); G3 `... --test m00_golden_baseline_test` (zero drift); `... --test m17_plugin_seam_test` (5/5); G4 corpus evidence scan (throwaway script under `target/`, NEVER committed; PRE-fix first underflow TU pinned to file:line, POST-fix zero negatives, balanced-set digest diff empty - spec 3.5 G4 + 4.3); `bench_compare` (hot-path load gate); workspace census + full `unittest discover` + `encoding_guard --all` in an orchestrator fresh session.

## Discipline clauses

- UTF-8: never Set-Content/Out-File/`>`/Get-Content-replace; Write/Edit tools
  only; pure-ASCII content (AGENTS.md File-Edit Discipline). This applies
  with extra force to Step 0 (the verbatim spec copy).
- Touch only the touch-set; on discovering out-of-scope breakage: report, do
  not fix.
- Bare high-volume commands (raw `cargo test`/`build`/`clippy`, full
  `unittest discover`, `ui_smoke`, `bench_compare`) in an implementer session
  are a reviewer blocker; run everything through the distilling entry points
  (`docs/specs/tool-output-distillation.md`). If a gate command is refused by
  the permission layer, try the `python tools/distill.py -- <cmd>` route; if
  that is also refused, report the denial - NEVER skip a gate silently
  (topology-row lesson, 2026-10-04).
- After EACH per-module Green gate, append a 1-3 line checkpoint to
  `tasks/cpp-brace-depth-underflow-handoff-journal.md`: module + commit hash,
  any deviation, any lock/contract finding.
- If context exhaustion looms: STOP, return a disk-state inventory; a dead
  session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/cpp-brace-depth-underflow-handoff.md --scope "crates/lang-cpp/**" --cmd "python tools/testrun.py --distill -p ariadne-lang-cpp"`.

## Acceptance (machine-checkable verdict lines)

- G1: `python tools/testrun.py --distill -p ariadne-lang-cpp` ->
  `0 failed`, exit 0; canonical line quoted verbatim (new tests raise `N`
  above the pre-fix count; `S suites` must not change).
- G2 (reviewer): `22 passed / 0 failed` - exit 0.
- G3 (reviewer): exit 0, zero golden drift; `tests/golden/**` byte-identical.
- G4 (reviewer, verbatim record): PRE-fix scan names the first
  prefix-negative TU as `path:line`; POST-fix scan reports `0` TUs below zero;
  balanced-set digest diff empty.
- G5: `python tools/hygiene_check.py` exit 0. G6 (reviewer):
  `python tools/encoding_guard.py --all` exit 0.
- I6/I7: characterization suites + `symbol_locator` + parity test diff-empty
  (reviewer `git show`, oldest-first); every commit inside the touch-set.

## Report back (final message)

1. Commit hashes + `git show --stat` per commit (Step 0 + implementation).
2. Red evidence (first failing tests per module, with the failure reason:
   leaked-token assertion diffs for A, `attempt to subtract with overflow`
   panic for B).
3. Green evidence (distill-mode digest header line + canonical summary line +
   capture path).
4. Any deviation + reason.
5. Out-of-scope observations (report-only).
Never paste raw tool output (cargo/git dumps) into the report: cite command +
canonical summary line + exit code.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill. Its commit-vs-touch-set check enforces the touch-set above (Step 0
commit additionally byte-diffed against `.kilo/plans/1791144879664-cpp-brace-depth-underflow.md`);
reviewer also runs the reviewer-hop gates listed above and inspects the
`handle_brace_close` diff for any unguarded decrement path (I2 structural).
