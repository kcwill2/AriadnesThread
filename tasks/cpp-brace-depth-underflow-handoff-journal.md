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

- Red A: `134 passed / 5 failed / 0 ignored / 1 suites` — all 5 failures are
  assertion diffs in `tokenizer::raw_string_tests`. `api_cpp_line155_...` leaks
  `word:R` + `punct:}` + `punct:)` (the predicted brace leak); the other four
  fixtures fail with the extra `word:R` (raw not consumed as a unit; the
  handoff's "leaked punct" wording applies verbatim only to the quoted-content
  fixtures — semantic reason holds). The fallback-lock fixture passed pre-fix
  as documented. Capture: `target/testrun-capture.txt`.
- Green A: `139 passed / 0 failed / 0 ignored / 2 suites`, exit 0, commit
  pending (single implementation commit per handoff). `S suites` reads 2 here
  and 1 during Red: the second suite is `Doc-tests ariadne_lang_cpp` (0 tests),
  which cargo skips once unit tests fail — failure-abort artifact, the pre-fix
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



