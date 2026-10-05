# T2 Handoff - M27 tu-index-hang-throughput: kill the byte-vs-char column class (both sites) + audit + DP-6 bump

Milestone: `tu-index-hang-throughput` (M27, ledger row 2026-10-05). Primary
crate: `crates/lang-cpp` - with TWO declared rider exceptions that user
decision DP-6 (RESOLVED YES) binds to this landing commit: the one-constant
`CURRENT_ANALYSIS_VERSION` bump in `crates/index` (cache invalidation must be
ATOMIC with the fix it invalidates for - a separate commit would leave a
window where corrected code reuses corrupt shards) and two one-line
integration-test riders (the M27 pin + the T0b un-ignore). Sizing law held
for the primary scope; total implementer gates: 10 (= the ~10 cap); expected
produced lines ~400-600 (<= ~1200). Workspace-wide gates belong to the
reviewer hop.

ROOT CAUSE IS CONFIRMED (E3 live panic) - do not re-investigate it; implement
it. A fix that touches ONE flush site is a plan-literal violation.

DISPATCH PRECONDITIONS: `tasks/tu-index-hang-throughput-t0-evidence.md`
landed (T0-report) AND `tests/fixtures/m27-nonascii-underflow/` +
`m27_nonascii_corpus_test.rs` landed (T0b). Parallel-safe with T1a/T1b.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `.kilo/plans/1791168710915-large-corpus-boot-responsiveness-shape.md`
   sec. 0 (verified mechanism), sec. 2 (I1/I7/I12/I14), sec. 3.4 (fix seam,
   audit obligation, data-integrity flag), sec. 6 (H-B), DP-6.
3. `crates/lang-cpp/src/tokenizer.rs` lines 40-53, 299-337 (the two flush
   sites + the correct `end: start + len`) and the whole file for the audit.
4. `crates/lang-cpp/src/parser.rs` lines 40-80 (the landed M25 floor guard on
   `handle_brace_close` - DO NOT WEAKEN) and the whole file for the audit.
5. `crates/index/src/cache.rs` lines 10-16 (const + epoch doc comment) and
   line 331 (`assert_eq!(CURRENT_ANALYSIS_VERSION, "2.0.0")`).
6. `tests/integration/tests/m12_incremental_analysis_test.rs` lines 90-100
   and 190-200 (the pinned epoch literal + comment; precedent: the M18.10
   bump updated this same pin).
7. `tasks/tu-index-hang-throughput-t0b-handoff.md` (the un-ignore rider
   wording) and `tasks/tu-index-hang-throughput-t0-evidence.md` (mechanism).
8. `docs/specs/tool-output-distillation.md`.

## Objective and invariants

**Commit 1 - the fix + class kill (lang-cpp):**
1. `flush_word` (`:306-316`): `column: column - word.len()` becomes
   `column: column - len` reusing the existing `let len =
   word.chars().count();` (the value `end` already uses - one unit for both
   fields).
2. `flush_punct` (`:326-336`, the E3 panic site `:331:21`):
   `column: column - punct.len()` becomes `column: column - len` reusing its
   `let len = punct.chars().count();`.
3. SAME-CLASS AUDIT: every usize subtraction on position math in
   `tokenizer.rs` AND `parser.rs` - enumerate each site (file:line) in the
   journal with a disposition: FIXED / SAFE-BY-INVARIANT (state the
   monotonicity or bound proof in one line) / ALREADY-GUARDED (M25 floor
   guard; do not alter). M25 precedent is WHY: one-site guards left the
   class alive. If any audit site is a real second bug, fix it in commit 1
   and add its regression test; if a fix would reach beyond `tokenizer.rs`/
   `parser.rs`, STOP and report (scope door, R4).
4. Tests (new module `crates/lang-cpp/src/tokenizer/underflow_fix_tests.rs`
   + appended `#[cfg(test)] mod underflow_fix_tests;` in `tokenizer.rs`):
   authored sources PURE ASCII - multi-byte content via byte-level
   `String::from_utf8(vec![...])` escapes, never literals.
   - `test_e3_punct_shape_no_panic_and_bounded_columns` - single line, a
     punct run containing U+00AB at/near line start (the `:331` class):
     `tokenize_raw` neither panics (debug) nor wraps; every `RawToken.column`
     within `[1, line_chars + 1]`; `end - start == chars().count()` per
     token. (Pre-fix debug: panic - the RED evidence is exactly E3's
     `attempt to subtract with overflow`.)
   - `test_word_shape_no_panic_and_bounded_columns` - same property for a
     word run containing U+00E9 (the `:311` class).
   - `test_mixed_multibyte_matrix_bounded` - small matrix: leading/middle/
     trailing multi-byte in word runs, in punct runs, adjacent to comment
     (`//`/`/*`) and string paths, each line bounded per the same property
     (I14's falsifiable statement).
   - `test_ascii_path_tokenization_unchanged` - for ASCII-only inputs the
     token stream (kind/line/column/start/end) byte-equals the pre-fix
     behavior baseline recorded in the test (the I1 promise: only non-ASCII
     column values are corrected).
5. I14 release-profile characterization: THE SAME tests must pass under
   `cargo test --release -p ariadne-lang-cpp` (overflow-checks off): pre-fix
   they FAIL on boundedness with huge wrapped columns (the silent-wrap
   pinned in red), post-fix they pass - wraps become mathematically
   impossible, not merely absent. No `debug_assert!`-only coverage
   (debug_asserts vanish exactly where the corruption lived).

**Commit 2 - DP-6 bump + riders (machine-pinned one-line edits):**
6. `crates/index/src/cache.rs:16`: `"2.0.0"` -> `"2.1.0"` + one appended
   doc-comment sentence: epoch bump for the tokenizer column-underflow class
   (M27 T2): shards written by RELEASE builds over non-ASCII corpora carry
   silently-wrapped columns; the fingerprint chain hashes CONTENT not
   analyzer code, so this version bump is the invalidation (user DP-6,
   2026-10-05). Update `cache.rs:331` assert literal to `"2.1.0"`.
7. `tests/integration/tests/m12_incremental_analysis_test.rs`: update the
   pinned literal at :95 to `"2.1.0"` and the epoch comments at :93-94 /
   :194-198 (same edit shape as the M18.10 bump precedent).
8. `tests/integration/tests/m27_nonascii_corpus_test.rs`: REMOVE exactly
   the `#[ignore = "..."]` attribute line of
   `test_m27_nonascii_corpus_cold_index_completes` (T0b's sanctioned
   single-line rider; assertions untouched).
9. Data-integrity disposition (journal + report text, mandatory): with the
   bump, stale RELEASE-built shards/index over non-ASCII corpora (including
   the completed 30%) no longer fingerprint-match, so affected corpora
   re-index on next run; re-index every affected corpus after this lands;
   the column-vs-line-length shard scan remains a candidate detector only.

STOP clauses (report, do not fix): goldens/parity failing on anything other
than the pinned literals above; m00/m18 parity needing edits (they must stay
UNEDITED - goldens are version-neutral via `common/mod.rs` masking, expected
outcome for pure-ASCII corpora is byte-identical output); any "fix" tempting
a `crates/graph`/`crates/scheduler`/`tests/fixtures` change; the T0b corpus
still panicking post-fix (then commit 1 is wrong - fix it).

## Touch-set (hard; any edit outside = blocker, report don't fix)
- `tasks/tu-index-hang-throughput-t2-journal.md` (journal; always in-set -
  carries the audit table)
- `crates/lang-cpp/src/tokenizer.rs` (both flush sites + one appended mod
  line)
- `crates/lang-cpp/src/tokenizer/underflow_fix_tests.rs` (NEW)
- `crates/lang-cpp/src/parser.rs` (ONLY if the audit finds a real second
  bug there; journal-cited; otherwise byte-identical)
- `crates/index/src/cache.rs` (commit 2: const + doc sentence + assert
  literal)
- `tests/integration/tests/m12_incremental_analysis_test.rs` (commit 2:
  literal + comments only)
- `tests/integration/tests/m27_nonascii_corpus_test.rs` (commit 2: delete
  exactly the `#[ignore]` line)

You MUST NOT touch: `tests/golden/**`; `tests/fixtures/**` (incl. the T0b
specimen); `crates/pipeline/**`, `crates/cli/**`, `crates/ui/**`,
`crates/graph/**`, `crates/scheduler/**`; `crates/cli/src/commands/
progress.rs`; shard-envelope FIELDS anywhere (I7 - the version CONSTANT is
the only cache-visible change, DP-6); `tools/bench_baseline.json`;
`tests/integration/tests/common/mod.rs` (masking already version-neutral).
No `git stash`, no branch change.

## Required tests
See 4-5 above; plus the release-profile run of the identical module (Gate 4).
Red protocol (valid Red, no todo!()):
- Debug RED (Gate 1): the three multibyte tests panic `attempt to subtract
  with overflow` (E3 class on unit sources - word test hits `:311`, punct
  `:331`); `test_ascii_path_tokenization_unchanged` passes at Red (it locks
  what must not change).
- Release RED evidence (Gate 1b, BEFORE the fix): boundedness fails with
  huge wrapped columns (exit 1) - pins that the silent wrap was real.

## Hygiene (review-enforced)
Functions < 100; files < 1000 (`tokenizer.rs` 343 + small mod line; tests
module <= ~250 lines); pure-ASCII authored test source; no banned names;
`python tools/hygiene_check.py` exit 0.

## Frozen locks (each with its proving command)

| Lock | Proof |
|---|---|
| stage-5 13-frame replay + `progress.rs` (DP-1; inherited M27 row) | files absent from diff; cli scoped suite at the reviewer hop |
| m00 parity + golden manifests UNEDITED (I1) | `python tools/testrun.py --distill -p ariadne-integration-tests` full scoped run green, `tests/golden/**` absent from diff |
| m18 parity UNEDITED | same run (`m18_cpp_plugin_parity_test` green unedited) |
| shard envelope FIELDS byte-identical (I7) | `crates/index/src/cache.rs` diff = const + doc sentence + assert literal ONLY (reviewer eyeballs the hunks) |
| M25 floor guard not weakened (I14 lineage) | `parser.rs` diff-empty or audit-only with journal-cited disposition; `crates/lang-cpp` scoped suite incl. M25 lock tests green UNEDITED |
| `CURRENT_ANALYSIS_VERSION` moves EXACTLY 2.0.0 -> 2.1.0 (DP-6) | grep the constant across `crates/` - one definition site changed |
| mojibake + hygiene | `encoding_guard.py --staged` / `hygiene_check.py` exit 0 |

## Gates (exact commands, timeout 600000 ms each; scoped only)
Distilling entry points only (ledger row 58). Workspace-wide gates belong to
the reviewer hop.

1. Release-RED (before fixing, pins the wrap): `python tools/distill.py --
   cargo test --release -p ariadne-lang-cpp` with the new tests authored -
   boundedness fails (exit 1; capture).
2. Debug-RED: `python tools/testrun.py --distill -p ariadne-lang-cpp` - the
   multibyte tests panic with `subtract with overflow` (exit 1).
3. Debug-GREEN: same command - `0 failed`.
4. Release-GREEN (I14): `python tools/testrun.py --distill --release -p
   ariadne-lang-cpp` - `0 failed`.
5. Rider scoped: `python tools/testrun.py --distill -p ariadne-index` ->
   `0 failed` (assert line updated).
6. Integration scoped (m12 pin, m27 un-ignored completing, m00/m18 parity
   UNEDITED green): `python tools/testrun.py --distill -p
   ariadne-integration-tests` -> `0 failed`; ignored count back to 4.
7. `python tools/hygiene_check.py` - exit 0.
8. `python tools/encoding_guard.py --staged` - exit 0.
9. Commit 1: `python tools/distill.py -- python tools/commit_scoped.py
   --scope crates/lang-cpp/src/tokenizer.rs --scope
   crates/lang-cpp/src/tokenizer/underflow_fix_tests.rs --scope
   crates/lang-cpp/src/parser.rs --scope
   tasks/tu-index-hang-throughput-t2-journal.md -m "fix(lang-cpp): char-count
   column at BOTH tokenizer flush sites; same-class usize audit
   (tokenizer+parser) dispositioned; debug+release underflow locks (M27 T2)"`.
10. Commit 2: `python tools/distill.py -- python tools/commit_scoped.py
   --scope crates/index/src/cache.rs --scope
   tests/integration/tests/m12_incremental_analysis_test.rs --scope
   tests/integration/tests/m27_nonascii_corpus_test.rs --scope
   tasks/tu-index-hang-throughput-t2-journal.md -m "chore(index): analysis
   version 2.0.0 -> 2.1.0 (DP-6: invalidate silent-wrap shards) + m12 pin +
   m27 corpus un-ignore (M27 T2 rider)"`.

Total implementer gate invocations: 10 (= cap; if you need an 11th, it is a
sign of scope drift - report instead).

## Acceptance (machine-checkable; implementer hop)
1. Gates 2/1b: both-profile RED captures (panic debug / wrapped columns
   release). 2. Gates 3/4: `0 failed` both profiles. 3. Gate 6: integration
   `0 failed`, m27 corpus COMPLETES (I12). 4. hygiene/encoding exit 0.
5. `git show --stat` per commit = its scoped files; commit 2 diffs are
   literal-only. 6. Journal carries the complete audit table + the
   data-integrity disposition text.
Reviewer hop: all scoped re-runs (debug+release lang-cpp, index,
integration); full census (`+4-ish passed / ignored back to 4` reconcile,
`--distill` unscoped, standing errata); `encoding_guard --all`; hygiene;
per-commit `git show` with the rider-diff eyeball (const + doc + assert;
m12 literals; one deleted `#[ignore]` line); verdict records the DP-6 bump
commit + the re-index disposition routed to the user (G2-3).

## Discipline clauses
- UTF-8: Write/Edit only; test sources pure ASCII (byte arrays, `\u{...}`
  escapes); fixture corruption signatures never as literals (AGENTS.md).
- Both sites or REJECT - self-check `git show` shows `:311`-class AND
  `:331`-class hunks before reporting.
- Touch only the touch-set; second-bug discoveries outside it: report.
- Journal checkpoint after each gate (audit table included).
- Context wall: STOP + disk-state inventory; successor runs
  `python tools/checkpoint_guard.py resume --spec
  tasks/tu-index-hang-throughput-t2-handoff.md --scope ... --cmd ...`.

## Report back (final message)
1. Both commit hashes + `git show --stat` each. 2. Red evidence both
profiles (panic line / wrap-failure line, capture paths). 3. Green canonical
lines both profiles + index + integration. 4. Audit table summary (counts by
disposition; file:line list lives in the journal). 5. Data-integrity
disposition text. 6. Deviations. Never paste raw output.

## Reviewer-gate reference
Adjudicated via the `review-gate` skill. Extra checks: both-site rule; audit
covers every `- ` on usize position math in the two files (independent grep);
m00/m18/goldens diff-EMPTY; rider diffs literal-only; release profile ran the
SAME tests (I14); G2-3 verdict sentences.

## Sequencing note
After T0-report + T0b. Parallel-safe with T1a/T1b and ALL of M26 (disjoint
trees). T3 (if ever dispatched) does not depend on this hop's files, but the
T1 sufficiency verdict + this fix's landing both precede any T3 closeout.
