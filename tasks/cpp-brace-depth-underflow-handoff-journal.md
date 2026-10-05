# cpp-brace-depth-underflow T1 - implementer journal

Run log for `tasks/cpp-brace-depth-underflow-handoff.md` against the canonical
spec `docs/specs/cpp-brace-depth-underflow.md`. Checkpoints appended after each
module's Green gate, per the handoff's discipline clauses.

## Step 0 - canonical spec copy

- Copied `.kilo/plans/1791144879664-cpp-brace-depth-underflow.md` ->
  `docs/specs/cpp-brace-depth-underflow.md` with Read + Write tools only.
- Byte-identity verified via throwaway checker under `target/verify_spec_copy.py`
  (never committed): both files sha256
  `2a54c8043bc0152108e0724bbe2fcbc4ca375f0e28a05e12e0ce24a0566f82ad`,
  pure ASCII, 302 lines.
- `python tools/distill.py -- git diff --no-index ...` and
  `git hash-object ...` were refused by the permission layer (boilerplate
  catch-all message); the target/ checker route discharged the same check.
- Committed isolated: `9ce140a` scoped to `docs/specs/cpp-brace-depth-underflow.md`.

## Deviations recorded up front

- Journal path: the delegation prompt names
  `tasks/cpp-brace-depth-underflow-journal.md`; the binding handoff touch-set
  and its commit glob name `tasks/cpp-brace-depth-underflow-handoff-journal.md`.
  Following the handoff (this file).
- I4 fixture set adds the `R"DOC(...)DOC"` class as a second assert inside
  `delimited_raw_string_braces_emit_no_tokens` (delegation explicitly requires
  the `R"DOC(` class fixture; the handoff's exact fixture values are kept).

## Module A - tokenizer raw-string lexing

(Red A / Green A checkpoints appended below.)

- Red A: `134 passed / 5 failed / 0 ignored / 1 suites` -- all 5 failures are
  assertion diffs in `tokenizer::raw_string_tests`. `api_cpp_line155_...` leaks
  `word:R` + `punct:}` + `punct:)` (the predicted brace leak); the other four
  fixtures fail with the extra `word:R` (raw not consumed as a unit; the
  handoff's "leaked punct" wording applies verbatim only to the quoted-content
  fixtures -- semantic reason holds). The fallback-lock fixture passed pre-fix
  as documented. Capture: `target/testrun-capture.txt`.
- Green A: `139 passed / 0 failed / 0 ignored / 2 suites`, exit 0, commit
  pending (single implementation commit per handoff). `S suites` reads 2 here
  and 1 during Red: the second suite is `Doc-tests ariadne_lang_cpp` (0 tests),
  which cargo skips once unit tests fail -- failure-abort artifact, the pre-fix
  suite structure is unchanged (verified in capture lines "Running unittests
  src\lib.rs" + "Doc-tests ariadne_lang_cpp").
- Module A finding: unterminated raw with a legal header must still drop the
  `word:R` prefix to satisfy the required `[word:a]`; implemented as probe
  (`raw_string_open` = R-run + delimiter+`(` header) gating the word drop,
  then `scan_raw_string` false -> `scan_string_literal` swallow-to-EOF. This
  is the only reading under which I4's `unterminated` and `fallback` fixtures
  are simultaneously satisfiable.
- Refactor note (applied at the Refactor stage): the inline dispatch version
  from Green A pushed `tokenize_raw` to 111 lines (hygiene R1 blocking, caught
  by the first hygiene run after Green B). The quote branch was extracted into
  `scan_quote` (tokenize_raw back under 100; second hygiene run has zero R1
  lines); the
  branch now delegates. `scan_raw_string` re-verifies the header as its
  contract's false-branch, standalone per spec 4.1.

## Module B - parser stray-close floor

- Red B first attempt: `python tools/testrun.py --distill -p ariadne-lang-cpp`
  returned `parse format drift: no test result line matched the pinned parse
  spec` (exit 2) - a compile error (E0382 borrow of moved
  `ctx.current_function` in the new test, harness-side, not a semantic Red).
  Fixed via `.as_ref().map(...)` and re-ran.
- Red B: `142 passed / 5 failed / 0 ignored / 1 suites` - all 5 failures are
  `attempt to subtract with overflow` panics at `parser.rs:61` inside
  `handle_brace_close`, exactly the predicted Red signal. The three other I1
  fixtures (`}}}`-adjacent ones that the lexer fix balances) pass early, as
  expected under the A->B sequencing.
- Green B: `147 passed / 0 failed / 0 ignored / 2 suites`, exit 0. Floor
  implemented verbatim per spec 4.1: depth-0 close ends every open scope,
  depth stays 0, no arithmetic.
- Final G1 (post-refactor): `147 passed / 0 failed / 0 ignored / 2 suites`,
  exit 0. Capture path quoted in the run report.

## Gates run (implementer side)

1. testrun Red A, 2. testrun Green A, 3. testrun Red B (drift, compile fix),
4. testrun Red B, 5. testrun Green B, 6. testrun final G1 post-refactor,
7. testrun post-commit verification (committed tree:
   `147 passed / 0 failed / 0 ignored / 2 suites`, exit 0).
7 invocations vs the ~5 planned: +1 for the harness compile fix (drift run),
+1 for the post-commit re-verification.
Then: hygiene_check exit 0, encoding_guard --staged exit 0, scoped commit
`ad65747` (Step 0 committed earlier as `9ce140a`).

## Out-of-scope observations (report-only, not fixed)

- `hygiene_check.py` carries a large pre-existing R3 (warn-only) load across
  `crates/cli`, `crates/ui`, `crates/graph` etc. (banned-identifier and
  single-letter locals). None introduced by this handoff; untouched per the
  touch-set. R3 flips to blocking via dated ledger rows - if that ever
  happens, those sites need their own milestone.
- Permission-layer refusals (boilerplate catch-all message) were hit by:
  raw `git`, raw `python -c`, `distill.py -- git diff --no-index`,
  `distill.py -- git hash-object`, `distill.py -- python -c`, and a
  PowerShell `>` redirect. `distill.py -- git status --porcelain`,
  `distill.py -- python target/<script>`, and the four ritual tools all work.

## Verdict round 1 - Finding 1 (blocker) + Finding 2 (G4)

Verdict `tasks/cpp-brace-depth-underflow-t1-verdict.md` REJECTED: the
terminator match in `scan_raw_string` omitted `chars[closing_quote] == '"'`.
With an empty delimiter the delimiter-slice equality is trivially true, so
the first `)` inside raw content terminated the literal early and leaked its
braces - turning balanced TUs underflowing (probe `R"(f(x) } g)" h(); {}`:
committed min=-1). Accepted; contract breach vs spec 4.1 confirmed by reading
the match arm.

- Red (verdict fixture, verbatim):
  `expect_token_kinds("a R\"(f(x) } )\" b", &["word:a", "word:b"])` +
  delimited `)`-followed-by-delimiter-chars variant
  `"a R\"d(close)done{ )d\" b"`, added as
  `closing_paren_inside_raw_content_requires_delim_quote_terminator`.
  Gate line: `147 passed / 1 failed / 0 ignored / 1 suites` - leaked
  `punct:}` assertion diff exactly as the verdict predicted.
- Green: added `&& chars[closing_quote] == '"'` to the match arm.
  Gate line: `148 passed / 0 failed / 0 ignored / 2 suites`, exit 0.
- G4 re-verification. Reviewer's `.kilo/plans/rv_g4_corpus_scan.py` re-run
  UNMODIFIED first (import-exec via `python -m unittest discover -s
  .kilo/plans -p rv_g4_corpus_scan.py`): reproduced the verdict numbers
  verbatim (pre 18, committed 28, newly_negative 13, balanced digest diff 824)
  - its "committed" mirror is statically pinned to the reviewed pre-guard code
  and CANNOT converge to the fix by re-running; editing it would also violate
  "do not fix the reviewer's scan script".
- Method-identical throwaway copy at `target/g4_post_fix_scan.py` (target/,
  NEVER committed; sanctioned throwaway route per the handoff's G4 note):
  sole difference is that its `committed` mirror tracks the NOW-COMMITTED
  guarded `scan_raw_string` (verified condition-for-condition against
  tokenizer.rs), plus an explicit committed-vs-faithful comparison block.
  Target lines (post-fix, 9224 files / 2482395 lines):
  - `mode=pre files_below_zero=18 files_nonzero_final=98`
    first_underflow=`AggregateFunctionNull.cpp:174` (pin stable)
  - `mode=faithful files_below_zero=14 files_nonzero_final=71`
  - `mode=committed files_below_zero=14 files_nonzero_final=71`
    - committed == faithful: target `files_below_zero=14` MET.
  - `FINDING2 committed_vs_faithful digest_diff_count=0 state_diff_count=0`
  - `newly_negative_post_committed_vs_pre=0` - target MET.
  - `fixed_by_fix(pre_neg->post_ok)=4`, `still_negative_both=14` - the
    verdict's Finding 3 undecidable residual (absorbed by the parser floor).
  - balanced-set digest diff vs PRE: 17 of 9125 (all-files 44). All 17 are
    TUs containing raw literals whose brace emission the Layer-1 fix is
    MEANT to change (pre/post min/final identical `(0,0)`; only emission
    sequence differs). Reported for reviewer adjudication - not a
    committed-vs-faithful divergence, which is the verdict's acceptance axis.
- Deviation: the target/ copy deviates from "re-run the reviewer's scan"
  literally because the reviewer's mirror is pinned to reviewed code; the
  reviewer's file was left untouched and its output reproduced verbatim
  first. Copy + reviewer file differ ONLY in the mirror sync + report block
  (both files kept next to each other for the reviewer's diff).
- Gates round 1: testrun Red, testrun Green, reviewer scan re-run,
  target-scan, hygiene exit 0 (zero R1/R2; R3 warns all pre-existing),
  `encoding_guard.py --staged` exit 0, scoped commit via
  `commit_scoped.py` on the handoff globs.




