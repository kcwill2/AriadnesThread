# cpp-brace-depth-underflow - Kickoff + Shape

Status: CLOSED 2026-10-04 as M25 (number assigned at closeout - no kickoff doc
existed; M15-M24 and M23.5 already consumed). Shipped on `main` through
`af11113`; verdict `tasks/cpp-brace-depth-underflow-t1-verdict.md` round 2
APPROVED. Closeout reword: G4(b), I3, and 4.3 ("zero TUs below zero" /
"balanced-set digest diff empty") proved unsatisfiable by layer 1 - the scan
replays raw token arithmetic and the fix deliberately changes raw-hazard
token emissions - and were reworded to the shipped acceptance axis (see those
sections; evidence in `tasks/cpp-brace-depth-underflow-execution-results.md`
round-2 G4). Parity counts corrected at closeout: m18 22->23, m17 5->6 (the
UNEDITED files always held 23 and 6 tests).
Requested canonical location was `docs/specs/cpp-brace-depth-underflow.md`; the
architect session's edit permissions deny that path, so this doc is the
verbatim source for it (copy there at handoff authoring).

## 1. Intent

Fix a `usize` underflow panic in the C++ parser's brace-depth counter, hit while
indexing a real-world corpus: `target\debug\tool.exe ui C:\Code\Corpora` panicked
at `crates/lang-cpp/src/parser.rs:61` (`self.brace_depth -= 1` in
`handle_brace_close`) during step 2/5 "Indexing translation units" at ~336/5744
TUs. The release-build consequence is worse than the debug panic: `-=` at depth
0 wraps to `usize::MAX`, `brace_depth < depth` is then true at every subsequent
close, and class/function scoping is silently lost for the rest of the TU
(symbols, edges, and call sites vanish; a recorded function depth of
`brace_depth + 1` also wraps). The fix must make scoping *correct*, not merely
panic-free: restore token-stream fidelity where lexing is decidable, and give
unbalanced token streams defined, non-poisoning scope semantics where it is not.

Blast radius: `crates/lang-cpp` only (tokenizer + parser, plus tests). The crash
surfaces through `crates/cli` step 2, but cli/index/pipeline are consumers of
the frozen `SemanticAnalyzer` trait and are NOT part of the fix.

## 2. Root cause (established by code reading + corpus grep)

### 2.1 Parser arithmetic is symmetric - the fast-forward suspects are innocent

`analyzer.rs:18-40` dispatches every emitted `{` token to `handle_brace_open`
and every `}` token to `handle_brace_close`, unconditionally. The
`handle_class_or_struct` fast-forwards cannot skip brace tokens:

- `{` case (`parser.rs:96-100`): `self.i += 1` lands on the *name*; the loop's
  `ctx.i += 1` then lands on `{`, which is dispatched and counted.
- `:` case (`parser.rs:101-124`): the `j` scan stops at the first non-access
  Word, which in valid C++ precedes any brace (base-name list).
- `;` case: skips only the `;` of a forward declaration; no braces involved.
- `handle_global_function` and the other handlers never move `i`.

Therefore `brace_depth` is exactly the running prefix sum of emitted `{` minus
`}` token counts, and underflow happens if and only if the *emitted token
stream* contains a prefix with more `}` than `{`. The bug is token-stream
fidelity, not parser bookkeeping. The "`{` fast-forward" suspect is ruled out
with the traces above.

### 2.2 Primary cause: raw string literals are not lexed (verified class in corpus)

`tokenizer.rs` treats every `"` as a quote-pair string scan
(`scan_string_literal`, lines 152-180). C++ raw string literals
`R"delim(...)delim"` whose content contains `"` break quote pairing:

- Odd-indexed content chunks between paired quotes LEAK into the code stream
  (their braces are counted as real).
- The raw terminator `)"` opens a phantom string that swallows REAL code until
  the next genuine quote in the file (their braces are NOT counted).

Either effect can make the prefix sum go negative (debug panic / release
wraparound) or silently hide whole functions behind a phantom string (release
symptom the panic would have masked).

Repro TU facts (from `C:\Code\corpora\clickhouse\src`, ~2.47M lines):

- `src/Server/ACME/API.cpp` - strongest candidate. Line 155 is
  `+ R"("})";` inside function scope (opened line 130). Quote-by-quote: the
  scan opened at the raw-opening quote closes at the content `"`; the content
  `}` then sits outside any hidden region until the raw terminator opens the
  next phantom scan - i.e., that `}` provably emits a `}` Punct token with no
  `{` partner. The file concentrates the hazard: lines 135, 139
  (`R"({{"alg":...}})")` quote-heavy fmt-format templates), 152-155 (four
  odd-quote raws in one expression), and 309 (`R"({"csr":")" + csr + R"("})"`)
  all inside function bodies. Whether this file is the exact 336th TU in index
  order, or whether its drift compensates before EOF (scope corruption without
  panic), is decided by the deterministic replay scan required in gate G4 -
  not by eye.
- `src/Client/tests/gtest_oauth_device_flow.cpp:142,154,247-278` - raw JSON
  arguments (`R"({"error":"..."})"`) inside `TEST` bodies; same leak/phantom
  mechanism, several instances per TU.
- `R"DOC(...)DOC"` documentation raws are widespread (`TableFunctions/*.cpp`,
  `AggregateFunctions/*.cpp`, `Storages/StorageQueryRunner.cpp`, 25+ files,
  grep capped): currently benign only because their content happens to contain
  no `"` - one quoted phrase corrupts the TU.
- `src/Parsers/Prometheus/tests/gtest_PromQLParser.cpp:1833`
  (`R"(up{label='a\"b'})"`) shows the second mismatch: the tokenizer applies
  backslash ESCAPING inside raw content, which the standard forbids.

### 2.3 Residual, undecidable-at-lexer-level drift (must be bounded, not fixed)

- Preprocessor: both `#if/#else` branches are tokenized (characterized:
  `tokenizer/characterization_tests.rs:271-283` keeps `#` lines as code).
  ClickHouse relies on per-branch balance (`src/Common/waitForPid.cpp:35-145`
  idiom) but also carries dead `#if 0` blocks (10+ instances in src, e.g.
  `Access/Common/RowPolicyDefs.cpp:58`) which may legitimately be
  brace-unbalanced. A dead or compiler-discarded imbalanced branch contributes
  net brace drift no pure lexer can remove.
- Macro bodies with braces are counted once at the definition site and never at
  use sites (e.g. `waitForPid.cpp:16-22`, balanced there by construction).
- Unpaired `'`: digit separators are fine when paired (locked by
  `tokenizer/characterization_tests.rs:286-303`); a stray `'` swallows code.

These cannot be eliminated without a real preprocessor. The parser-level
defense in 3.2 makes them produce defined, deterministic scope degradation.

## 3. Kickoff decisions

### 3.1 Scope

In: `crates/lang-cpp/src/tokenizer.rs`, `crates/lang-cpp/src/parser.rs`,
new test modules in `crates/lang-cpp/src/`, evidence-scan procedure (throwaway,
under `target/`, never committed).
Out (with reasons): a real preprocessor or macro expander (undecidable
scope-creep, breaks every output lock on first divergence); digit-separator /
apostrophe lexing (paired case already correct, stray case absorbed by the
clamp); `analyzer.rs` dispatch (already symmetric); any other crate (consumers
of frozen API; m17 guard keeps deps inward). NOT a two-crate fix: the defect is
entirely in `lang-cpp`; `crates/index`, `crates/pipeline`, `crates/cli` are not
touched.

### 3.2 Fix layer decision (parser-level defense AND tokenizer fidelity)

- Layer 1 - tokenizer raw-string lexing. `R"d(...)d"` is context-free
  detectable; lex it like a string (emit no tokens, track lines). This makes
  scoping *correct* for the verified largest corpus class instead of merely
  clamping it. Without this layer, API.cpp-style TUs keep losing every function
  after the first quoted raw string to a phantom scan - accidental corruption,
  which the task statement explicitly forbids as an end state.
- Layer 2 - parser depth floor. In `handle_brace_close`, a `}` arriving at
  depth 0 is a *stray close*: it clears `current_class` and
  `current_function` and depth stays 0 (no arithmetic). This is the only sound
  defense for the residual drift in 2.3, and the direct cause-kill of both the
  debug panic and the release wraparound. Balanced streams never hit it
  (structural argument in I3).
- Rejected: parser-only guard (leaves 2.2 scoping incorrect-by-accident).
  Rejected: tokenizer-only fix (residual 2.3 drift still underflows).

### 3.3 Invariants (numbered, each independently checkable)

I1 No-panic floor. `CppSemanticAnalyzer::analyze_source` completes without
    panic on an adversarial-fixture suite (`}}}`, `{}}`, raw strings with
    internal quotes, unterminated raw, `#if 0` fragments, stray `'`) under a
    debug (overflow-checks on) test run. Test: new `brace_depth` test module.
I2 Debug/release equivalence. `brace_depth` has no unguarded decrement path
    (structural); locked behaviorally: a stray close leaves depth 0 and the
    NEXT function definition still records its body and call sites. Test in
    the same module; reviewer also inspects the diff for arithmetic.
I3 Balanced-stream freeze (reworded at M25 closeout - verdict finding 3/D7;
    the original absolute form was unsatisfiable by layer 1). For every input
    whose emitted token stream is prefix-nonnegative, closes at zero, and
    contains no raw-string literal (all current tests, all
    `tests/fixtures/**` - grep-verified raw-free - and the non-raw balanced
    corpus TUs), symbols/declarations/edges/call_sites are byte-identical to
    the pre-fix implementation. Raw-hazard TUs are layer 1's intended change
    class (3.2): pre-fix quote flips emitted phantom brace tokens whose
    removal is the point of the fix; for those TUs the freeze holds at
    depth-state level (pre == post state). Oracles: characterization suites
    green UNEDITED, `m18_cpp_plugin_parity` 23/23 UNEDITED (wording drift:
    file holds 23 tests), `m00_golden_baseline` zero drift, corpus scan with
    zero committed-vs-faithful digest/state diffs, zero newly-negative TUs,
    and every balanced-set-vs-PRE digest diff attributable to a raw-hazard TU
    with unchanged depth state (shipped: 17 such TUs, zero state changes).
I4 Raw-string tokens. `expect_token_kinds("a R\"( }{ )\" b")` ->
    `[word:a, word:b]`; delimited `R"d( } )d"` likewise; a multi-line raw
    advances the line counter of `b`; `R\"(` unterminated at EOF swallows to
    end (consistent with the locked unterminated-string behavior,
    `tokenizer/characterization_tests.rs:116`); `R"` NOT followed by a valid
    delimiter+`(` falls back to the ordinary string scan.
I5 Defined stray-close semantics. `analyze_source("} void f() { g(); }")`
    records `f` as a definition with the `g()` call site - i.e. scoping
    RESUMES correctly after the stray, rather than poisoning the TU (the exact
    release-build symptom today).
I6 Lock integrity. `tests/golden/**` byte-identical (fixtures verified raw-
    string-free by grep), `tests/fixtures/**` untouched, `m17_plugin_seam_test`
    green, M23 verbatim `symbol_locator` tests green UNEDITED (raw strings emit
    no tokens, so the offset-slicing locator is unaffected).
I7 Scope containment. The diff touches `crates/lang-cpp/**` only. Enforced by
    the review-gate commit-vs-touch-set check.

### 3.4 Frozen locks inherited from `tasks/milestone_ledger.md` (with proving commands)

| Lock | Command |
|---|---|
| `tests/golden/**` byte-locked; `UPDATE_GOLDENS=1` forbidden this milestone | `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_golden_baseline_test` |
| `tests/fixtures/**` frozen | review-gate touch-set check |
| `m18_cpp_plugin_parity` 23/23, UNEDITED (closeout correction: wording drift, file holds 23 tests) | `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` |
| `m17_plugin_seam_test` 6/6 (architecture guard; closeout correction: file holds 6 tests) | `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test` |
| UTF-8/mojibake gate | `python tools/encoding_guard.py --staged` pre-commit; `--all` in CI; all authored content pure ASCII |
| Hygiene R1/R2 blocking | `python tools/hygiene_check.py` |

### 3.5 Acceptance gate for the milestone (scoped gate commands)

- G1 `python tools/testrun.py --distill -p ariadne-lang-cpp` - unit +
  characterization suites, including the new raw-string and brace-depth tests.
- G2 `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test` - 23/23 (closeout correction: wording drift; round-2 line `23 passed / 0 failed / 0 ignored / 1 suites`).
- G3 `python tools/testrun.py --distill -p ariadne-integration-tests --test m00_golden_baseline_test` - zero golden drift.
- G4 corpus evidence scan (bounded, throwaway script under `target/`, NEVER
  committed): replay the tokenizer+counter brace arithmetic over every TU under
  `C:\Code\corpora\clickhouse\src` and report (a) PRE-fix: the first TU in
  walk order whose token-stream prefix goes negative, with file path and the
  line of first underflow; (b) POST-fix acceptance axis (reworded at M25
  closeout - the literal "zero TUs go below zero" is unsatisfiable by layer 1:
  the scan replays raw token arithmetic and therefore surfaces the spec-2.3
  residual class the layer-2 parser floor exists to absorb): zero
  committed-vs-faithful digest/state diffs, zero newly-negative TUs vs PRE,
  and the below-zero set equal to the spec-2.3 undecidable class (shipped:
  14 residual TUs, faithful mode also 14). The reviewer records the output
  verbatim; (a) pinned the 336th-TU question to
  `AggregateFunctionNull.cpp:174`.
- G5 `python tools/hygiene_check.py` exit 0; G6 `python tools/encoding_guard.py --all` exit 0.
- Workspace census (`testrun.py --distill` full), full unittest discover, and
  `bench_compare` are orchestrator fresh-session gates per the Planning Law,
  outside any single handoff; `bench_compare` matters here (hot-path change)
  and belongs to the reviewer hop.

## 4. Shape

### 4.1 Contracts

`crates/lang-cpp/src/tokenizer.rs` (additive, internal):

```rust
/// Called when a `"` is seen immediately after an identifier run ending in `R`
/// (covers `R"`, `u8R"`, `uR"`, `UR"`, `LR"`). Reads the delimiter run
/// ([A-Za-z0-9_]* up to `(`), then consumes until `)delim"` inclusive,
/// tracking line/col like scan_string_literal. Emits NO tokens. Returns false
/// (position untouched) if no valid delimiter+`(` header exists or no closing
/// `)delim"` exists before EOF - caller falls back to scan_string_literal.
fn scan_raw_string(chars: &[char], quote_at: usize,
                   i: &mut usize, line: &mut usize, col: &mut usize) -> bool
```

Detection rule: at a `"`, walk back while `is_alphanumeric() || '_'`; raw iff
the run is non-empty and ends with `R`, and the char after `"` starts a legal
delimiter terminated by `(`. Identifier-immediately-followed-by-`"` is invalid
C++ outside raw contexts, so false positives cannot appear in code that
compiles; the `(`-delimiter requirement rejects `R""`-style oddities.
Unreachable-at-EOF falls back to today's swallow-to-EOF string behavior
(documented, matches `characterization_tests.rs:116`).

`crates/lang-cpp/src/parser.rs` (behavioral, signature unchanged):

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

Frozen surfaces: `TokenKind`, `Token`, `RawToken`, `tokenize`, `tokenize_raw`
signatures and the emitted-token-only view (raw strings emit zero tokens, so
`RawToken.start/end` are untouched for them exactly as for today's strings);
`ParserCtx` field set and `pub(crate)` visibility of `handle_brace_open/close`
(the characterization test calls them directly - `characterization_tests.rs:251`);
`CppSemanticAnalyzer` public API; all `ariadne-lang-api` DTOs; `TuAnalysis`
shape. No new public items, no new dependencies, no error-type changes (the fix
turns a panic into defined data, not into an error).

### 4.2 Module map

| File | Role |
|---|---|
| `crates/lang-cpp/src/tokenizer.rs` | raw-string scan + dispatch branch (touched; 223 lines -> ~290) |
| `crates/lang-cpp/src/parser.rs` | stray-close floor (touched; 494 -> ~510) |
| `crates/lang-cpp/src/tokenizer/raw_string_tests.rs` | NEW: I4 fixtures (do NOT extend characterization suites - those lock old quirks and stay UNEDITED) |
| `crates/lang-cpp/src/parser/brace_depth_tests.rs` | NEW: I1/I2/I5 fixtures |
| `tokenizer.rs` / `parser.rs` test-mod declarations | one `mod` line each |
| MUST NOT touch | `analyzer.rs` dispatch loop, both `characterization_tests.rs`, `symbol_locator*`, `tests/golden/**`, `tests/fixtures/**`, `crates/index/**`, `crates/pipeline/**`, `crates/cli/**`, `ariadne-lang-api` |

### 4.3 Oracle discipline

The equivalence oracle for I3 is three-part: (1) the two existing
characterization suites act as the frozen behavioral snapshot and must stay
green UNEDITED; (2) `m00_golden` + `m18_parity` pin fixture-derived output
bytes; (3) G4's throwaway replay scan is the corpus-wide equivalence check
(reworded at M25 closeout): run the balanced-set digest before and after; a
depth-STATE diff, or a digest diff on a TU with no raw-string header, is a
blocking finding; digest diffs confined to raw-hazard TUs with unchanged depth
state are the intended layer-1 change and must be enumerated with their
raw-header counts (shipped: 17 TUs, raw_headers 1..1375, incl.
`Server\ACME\API.cpp`; zero state changes, zero non-raw diffs). No new fixture directories under
`tests/fixtures/basic|expensive` (M18 lock: new files inside them are not
allowed) - new fixtures are inline string literals in the new Rust test modules.

### 4.4 Risks -> mitigation -> test/oracle

| Risk | Mitigation / oracle |
|---|---|
| Raw-detection false positive on identifier+quote adjacency | `(`-delimiter grammar rejects it; such code does not compile; I4 `R""` fallback fixture + characterization suites UNEDITED |
| Unclosed raw string swallows rest of file | Explicit, matches locked unterminated-string behavior; I4 fixture; parser floor bounds damage |
| Clamp fires on balanced code and changes scoping | Structurally impossible (fires only at depth 0); I3 scan diff on balanced corpus set |
| `#if 0`-style residual drift stays wrong-but-defined and is mistaken for a bug | I5 defines the expected degradation and a test pins it; G4 pass axis is newly_negative=0 with the residual below-zero set matching the spec-2.3 floor-absorbed class (G4(b) reworded at M25 closeout) |
| Golden drift if a fixture hid a raw string | Grep-verified none; G3 catches any miss |
| Hot-path slowdown in `tokenize` | Detection is a backward-identifier walk only at `"` after `R`; reviewer hop runs `bench_compare` (load gate) |
| 336th TU turns out not to be in clickhouse/src (walk-order guess wrong) | G4 PRE-fix scan names the true first underflow TU; the mechanism class and both fix layers are unchanged either way |
| `index_index.json` (golden) embeds parser output | Balanced-fixture invariant I3 keeps it byte-identical; resync forbidden this milestone |

### 4.5 Hygiene notes

- `tokenizer.rs` ~223 -> ~290, `parser.rs` ~494 -> ~510: both far under 1000.
- New functions: `scan_raw_string` <= 60 lines; detection helper <= 25;
  `handle_brace_close` stays well under 100.
- New test modules are separate files precisely so the 376/303-line
  characterization files are not grown (their "frozen quirks" intent must not
  mix with new-behavior locks).
- All authored content pure ASCII; no PowerShell file-edit pipelines
  (repo UTF-8 hazard); no source line near 2000 chars.

### 4.6 Sizing verdict (for the planner)

Single handoff: 1 crate (`crates/lang-cpp`), ~6 scoped implementer gate
invocations (G1, G5, G6 + scoped test filters; G2/G3/G4 are reviewer-hop),
expected produced lines ~500-700. The work does NOT span `lang-cpp` + `index`:
established in 2.1/3.1 - `index` is not even touched.
