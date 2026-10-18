# UTA-T2 Handoff - lang-cpp: directive word boundary, parser/locator/backend coverage

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. docs/specs/unit-test-audit.md (UTA-B3, pin-only list rows for backend/case-divergence).
3. crates/lang-cpp/src/directive_parser.rs (full), parser.rs:83-135,171-240,423-490,
   symbol_locator.rs:280-460, tokenizer.rs:270-380, backend.rs:1-60,180-200,
   analyzer.rs:50-70, plugin.rs:30-80, planning.rs:10-40, shaper.rs:10-30.
4. Model: existing adversarial suites tokenizer/underflow_fix_tests.rs, parser/brace_depth_tests.rs.

## Touch-set (hard; any edit outside = blocker, report don't fix)
- tasks/uta-t2-journal.md (journal; always in-set)
- crates/lang-cpp/src/**
- crates/lang-cpp/tests/** (new files only if cleaner)
You MUST NOT touch: other crates (the m18 parity oracle reads lang-cpp behavior -
if a fix flips a parity oracle, STOP and report before committing the fix),
Cargo.toml/lock. No `git stash`, no branch change.

## What to build
FIX (red test first):
1. UTA-B3 directive_parser.rs:24: require word boundary after `include` - the
   char after the keyword must not be [A-Za-z0-9_]. Red tests:
   `#includemy "a.h"` -> 0 directives; `#include_next <x>` -> 0 directives
   (pin `include_next` explicitly as report-flagged behavior change); normal
   quoted/angled still parse. Check every existing fixture first; if any
   fixture depends on non-boundary matching, report.

REQUIRED TESTS (pins, no src change unless red exposes listed bugs):
2. handle_new: `class A { B* p = new B; };` -> Complete A->B edge; trailing `new` at EOF no panic.
3. handle_call_site reserved words: `void f(){ if(a) while(b) z(); }` -> call_sites == [z].
4. collect_compiler_flags table: `["-DA=1","-Iinc","/std:c++20","-D","-I"]` -> exact split; bare-token shapes pinned.
5. symbol_locator masked_region: `#if 0 A #else A2 #endif`, `#elif` branch, nested `#if 0` in `#if 1`, unmatched `#endif` -> byte-exact locate().
6. extend_with_template: nested `<>` (depth>1), function template, `template <class A, class B>` byte-exact.
7. scan_function_tail/match_group: unclosed `(`/`{` candidate omitted: `void broken( { x(); }`.
8. handle_class_or_struct: EOF early-return, `class A : public B;` base-scan-to-EOF, anonymous `class {` -> no panic + symbol counts pinned.
9. handle_variable_decl: `unique_ptr<B> p;` -> Incomplete A->B; `A a();` ctor-shape pinned.
10. tokenizer: unterminated char-literal across newlines (byte-escape specimen, assert swallowed `}` and next-token line/col); raw-string multibyte + newline fallback position bookkeeping.
11. backend.rs:189 missing source -> current empty-Ok behavior pinned as
    `pin_backend_missing_source_analyzes_empty`; report as known swallow.
12. Case divergence B3-C: pin full matrix (`.H`, `.CPP`, `.Scala`-analogous) across
    CppDetector::detect / planning path filter / shaper as `pin_cpp_extension_case_matrix`; do NOT change behavior; report for owner decision.
13. parser.rs:239 `::f()` at stream start -> symbol recorded; i==0 return-type guard pinned; handle_sizeof top-level fn -> 0 edges pinned.
14. directive column: exact `column` for tabbed/angled/quoted specimens.
15. sanitize_symbol_stem: empty -> "symbol", punctuation-only, non-ASCII byte specimen pinned.
16. analyzer analyze_tu on nonexistent path -> Err (if current returns Ok-empty, pin + report).

## RGB protocol
Per item: Red (capture failing list), Green, Refactor. NEVER weaken an assertion to pass.

## Hygiene (review-enforced)
Functions < 100 lines, files < 1000 lines, banned-name ban (tmp/res/foo/bar/test1),
no single-letter locals. `python tools/hygiene_check.py` exit 0.

## Frozen locks
Existing tests + crates/lang-cpp/tests/tokenizer_oracles byte locks are contracts:
do not edit except one literally locking OLD behavior of item 1 (journal-justified;
reviewer adjudicates). m18 parity oracle: report-only on conflict.

## Gates (exact commands, timeout 600000 ms each)
1. `python tools/testrun.py --distill -- -p ariadne-lang-cpp` (red then green; quote digest + canonical line)
2. `python tools/testrun.py --distill -- -p ariadne-lang --testfilter m18` (parity guard if quick; else report why skipped and let reviewer rerun full)
3. `python tools/hygiene_check.py` -- exit 0.
4. `python tools/encoding_guard.py --staged` -- exit 0.
5. Commit: `python tools/commit_scoped.py --scope "crates/lang-cpp/**" --scope "tasks/uta-t2-journal.md" -m "test(lang-cpp): uta-t2 directive boundary fix + parser/locator coverage"`

## Discipline clauses
- UTF-8: Write/Edit tools only, pure-ASCII (corruption specimens via byte escapes).
- Touch only the touch-set; report out-of-scope breakage, do not fix.
- No bare cargo/clippy; distilling entry points only.
- Append 1-3 line checkpoint per Green gate to tasks/uta-t2-journal.md.
- Context exhaustion imminent: STOP + disk-state inventory.

## Report back (final message)
1. Commit hash + `git show --stat`. 2. Red evidence. 3. Green distill header +
canonical line + capture path. 4. Deviations. 5. Out-of-scope observations.
Never paste raw tool output.
