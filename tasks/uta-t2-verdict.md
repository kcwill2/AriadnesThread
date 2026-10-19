# UTA-T2 Verdict - lang-cpp directive boundary + parser/locator/backend coverage

**Verdict: APPROVED** (commits `55262e8` + `4ce7032`, base `994e5c3`).

Reviewer session 2026-10-18. Ground truth: `checkpoint_guard.py status` -> HEAD
`47cdf79` (orchestrator docs commit, outside this hand), branch main, tree clean
except untracked T1 artifacts (`uta-execution-results.md`, `uta-t1-verdict.md`).
Hand range reviewed: `994e5c3..4ce7032`.

## Fresh gate evidence (reviewer-run, verbatim canonical lines)

| Gate | Line |
|---|---|
| `python tools/testrun.py --distill -- -p ariadne-lang-cpp` | `193 passed / 0 failed / 0 ignored / 2 suites` (digest: `### cargo test PASS (exit 0) [capture: ...\target\testrun-capture.txt, 269 lines]`) |
| `python tools/testrun.py --distill -- -p ariadne-integration-tests m18` (parity) | `30 passed / 0 failed / 0 ignored / 37 suites` (PASS exit 0, capture 503 lines) |
| `python tools/hygiene_check.py` | exit 0; R1/R2 blocking: none; R3 warns all pre-existing (see D5) |
| `python tools/encoding_guard.py --all` | exit 0 (`ENC_EXIT_0` probe), no violation output |
| `python -m unittest discover -s tools -p "*_tests.py"` | `Ran 232 tests in 18.982s` / `OK` |

Both cargo gate lines match the implementer's reported lines exactly.

## Per-commit review (oldest-first)

- `55262e8` `test(lang-cpp): uta-t2 directive boundary locks + parser/locator/backend coverage` -
  18 files, +774/-0. Nine new test modules + journal opening. The eight modified
  src files (analyzer.rs, backend.rs, directive_parser.rs, lib.rs, parser.rs,
  planning.rs, symbol_locator.rs, tokenizer.rs) carry **only** appended
  `#[cfg(test)] mod <name>;` declarations - verified hunk by hunk; zero behavior
  lines changed, zero deletions anywhere. Test content is plan-literal for every
  item: item 2 (`class A { B* p = new B; };` trailing Complete edge; EOF `new` no panic),
  item 3 (`void f(){ if(a) while(b) z(); }` -> callees `[z]`), item 4 (exact table
  `["-DA=1","-Iinc","/std:c++20","-D","-I"]`, bare-token empty-string shapes),
  items 5-7 (byte-exact `locate()` bodies incl. the handoff's verbatim
  `void broken( { x(); }`), items 8-9, item 10 (byte-escape specimens, `y` at
  (3,11), char-based fallback bookkeeping), item 11 (pinned under the exact
  directed name `pin_backend_missing_source_analyzes_empty`), item 12 (exact name
  `pin_cpp_extension_case_matrix`, `.H`/`.CPP`/`.Scala` across detector/planning/
  shaper, behavior untouched), items 13-16 (column anchoring arithmetic verified
  by hand: tabbed `<` at col 11, spaced at 12, quoted at 10).
- `4ce7032` `docs(tasks): uta-t2 journal final` - journal only (+52), matches the
  on-disk artifact adjudicated here.

## Commit-vs-touch-set

`git diff --name-status 994e5c3..4ce7032` -> 18 paths: 9 `A` new test files and
8 `M` src files under `crates/lang-cpp/src/**`, plus `tasks/uta-t2-journal.md`.
Every path inside the declared touch-set. No Cargo.toml/lock, no other crate,
no `crates/lang-cpp/tests/**` (tests shipped as src child modules - inside the
touch-set, allowed). Zero deletions (`+826/-0` across the range), so the frozen
locks (existing tests incl. `symbol_locator/tests.rs`, `fake_process_tests`,
`tokenizer_oracles`) are byte-untouched by construction.

## Deviation dispositions (independent adjudication)

**D1 - UTA-B3 "fix" not shipped; claim non-reproducible. ACCEPTED.**
Verified independently at `crates/lang-cpp/src/directive_parser.rs:24-45`:
after `after_hash.starts_with("include")`, line 28 takes
`after_hash["include".len()..].trim_start()`; `trim_start` strips whitespace
only, so if the char after `include` is any `[A-Za-z0-9_]`, `after_include`
still starts with that word char, matches neither `"` (line 29) nor `<`
(line 36), and hits `else { continue }` (lines 43-44). The phantom-include
push at line 47 is structurally unreachable for `#includemy "a.h"`,
`#include_next <x>`, or any keyword run - the missing word boundary is real
but cannot change output. The directed guard is therefore provable dead code:
shipping it would violate the binding RGB protocol ("Red (capture failing
list)" - the predicted red never existed; TDD forbids code without red).
The three shipped locks (`boundary_tests.rs`) pin exactly the handoff's
specimens: `#includemy "a.h"` -> 0, `#include_next <x>` -> 0, and a control
proving normal quoted/angled/no-separator `#include"tight.h"` still parse -
so the locks lock behavior, not a phantom. I also re-ran the fixture-dependence
scan myself (`#\s*include[A-Za-z0-9_]` repo-wide): hits only in claim texts,
the new locks, and unrelated prose - no fixture or oracle depends on
non-boundary matching. The orchestrator's ledger downgrade (spec row 32-36 +
bug-ledger row `NOT REPRODUCIBLE (H2 locks)`, commit `47cdf79`) rests on this
evidence, which I independently reproduce. ACCEPTED.

**D2 - Commit message "fix" -> "locks". ACCEPTED.**
The handoff's literal message described a code fix; the commit ships locks, no
logic change. Renaming to "locks" is the truthful label; the scope discipline
clauses were kept verbatim and journal-declared. Message honesty outranks
literal-message fidelity; scope is enforced by the touch-set check, not by the
subject line.

**D3 - Gate-2 command rewritten. ACCEPTED.**
The handoff's command (`-- -p ariadne-lang --testfilter m18`) is invalid twice
over: `--testfilter` is not a cargo-test flag (cargo test takes a positional
filter), and the m18 parity suites are not in `ariadne-lang` at all - they are
`tests/integration/tests/m18_cpp_plugin_parity_test.rs`,
`m18_persistence_incremental_test.rs`, `m18_cache_epoch_test.rs` in the
`ariadne-integration-tests` package. The implementer's rewrite
(`-- -p ariadne-integration-tests m18`) runs the handoff's declared intent
(parity guard over lang-cpp behavior); I reran it fresh and reproduced the
reported line exactly: `30 passed / 0 failed / 0 ignored / 37 suites`.
No parity conflict existed, so the "report-only on conflict" branch was
correctly not triggered.

**D4 - Pinned quirks pinned-not-fixed. ACCEPTED per contract.**
All three are contract pins, and I verified each is pinned verbatim with loud
report-flagged comments, never weakened: (a) ParserCtx `f -> f Complete`
self-edge for `::f()` at stream start (`handle_branch_tests.rs:151-166`,
mechanism note matches: `qualified_function_name` needs `i >= 3`, then
handle_variable_decl's `(` branch fires on the just-set `current_function`);
(b) `A a();` double-record `Holder->A` + `Holder::a->A`, both Incomplete,
pinned as the two-edge pair after a mid-loop observation widened the pin
(journal checkpoint 2: assertion was *strengthened* to match observed output,
not weakened); (c) `#else`-of-disabled-`#if 0` masks both branches
(`pin_if_zero_else_masks_both_branches`, conservative-miss comment). Items
11 (backend empty-Ok swallow) and 12 (case divergence) likewise pin-only per
their handoff rows; item 16 found analyzer `analyze_tu` to be Err (no swallow
to pin) and the asymmetry is documented as directed ("if current returns
Ok-empty, pin + report" - it doesn't, and the lock documents that).

**D5 - R3 warns pre-existing. ACCEPTED, verified.**
Fresh `hygiene_check.py` exit 0 lists ~160 R3 warn lines; none is in the nine
new test modules (zero hits for all nine filenames). The four lang-cpp files
with warns (analyzer.rs:24/34, parser.rs:113/299/315/388,
symbol_locator.rs:142/164/216/297) were modified only by append-only mod
declarations at file tails - every warned line predates the hand per the
per-file diffs. Claim holds.

## Report-evidence check (distillation contract)

Journal is digest-shaped: canonical lines, exit codes, capture path
(`target/testrun-capture.txt`), zero raw pastes; gate commands use the
distilling entry points. Passes.

## Numbered findings (all minor; none blocking)

1. **[minor, report completeness]** `#include_next <x>` -> 0 directives is the
   one handoff-mandated report-flag ("pin explicitly as report-flagged") that
   appears only in the test's comment (`boundary_tests.rs:20-21`), not in the
   journal's Report-flagged list. Behavior and pin are correct; the ledger/
   closeout report should carry the GCC `#include_next` silent-drop alongside
   the four journal-listed flags so the owner sees it.
2. **[minor, config]** Three reviewer command shapes hit the catch-all deny
   boilerplate (`{"permission":"bash","pattern":"*","action":"deny",
   "source":"global"}`): `hygiene_check.py; echo "EXIT=..."`,
   `unittest discover ... 2>&1 | Select-Object -Last 3`, and
   `git status --short` (while plain `git status` passed - the allowlist is
   also flag-shape sensitive, not only compound-shape). Every affected gate
   was rerun in an admitted form and passed with explicit exit-code capture
   (`HYGIENE_EXIT_0`, `ENC_EXIT_0`, `Ran 232 tests ... OK`); no expected gate
   evidence is missing. Mirrors T1's note - allowlist rows for exit-code-echo
   chaining and `--short`-style flags would remove the boilerplate noise,
   which per the T1 incident is exactly what tempts a reviewer to misreport
   the whole bash surface as denied.

## Follow-up dispositions (for ledger at closeout, per review-gate step 9)

Owner-decision candidates surfaced by this hand, recommend dated
deferred-findings rows in `tasks/milestone_ledger.md`: ParserCtx stream-start
self-edge; `A a();` ctor-shape double-record; `#if 0`/`#else` conservative
double-mask; backend.rs:189 missing-source empty-Ok swallow; extension-case
divergence (detector lowercases vs planning/shaper case-sensitive);
`#include_next` silent drop. Also: UTA-B3 spec/ledger downgrade is recorded
(orchestrator commit `47cdf79`, evidence independently confirmed in D1).

## Artifacts

`tasks/uta-t2-verdict.md` and the H2 addendum appended to
`tasks/uta-execution-results.md` are left **untracked**, mirroring the T1
precedent (reviewer git-write is denied; the orchestrator lands them if the
milestone wants them versioned).

Milestone-wide gates (`--baseline`, `ui_smoke`, `bench_compare`) remain the
orchestrator's closeout obligation, not this hand's - see the execution-results
"Not run" section.
