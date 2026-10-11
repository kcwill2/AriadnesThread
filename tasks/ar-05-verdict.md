# AR-05 Verdict - Fingerprint sentinel on serialize failure (T2 MED)

Reviewer: review-gate skill, fresh adjudication 2026-10-09.
Commit under review: `3b960bc` (single commit; base `de0d9e0`, oldest-first -
it is its own oldest). HEAD at review: `932041a`. No later commit touched
`crates/index/src/cache.rs` (`git log --oneline 3b960bc..932041a -- crates/index/src/cache.rs`
empty) and cache.rs is not in the dirty set, so the working copy equals the
reviewed blob.

## VERDICT: APPROVED

Two minor findings (report discipline only). No blockers; no code changes
required. Merge is clear.

## Gate lines (verbatim, reviewer's fresh runs)

- `python tools/testrun.py --distill -- -p ariadne-index` ->
  `15 passed / 0 failed / 0 ignored / 2 suites` (PASS, matches expectation)
- `python tools/encoding_guard.py --all` -> zero output; source contract
  `return 1 if violations else 0` (tools/encoding_guard.py:173), violations
  print when present -> exit 0 with zero violations.
- `python tools/hygiene_check.py` -> zero entries in `crates/index/src/cache.rs`
  (529 lines, all fns < 100); the only R1/R2 entries are on dirty
  `crates/ui/src/server.rs` (foreign uncommitted WIP; AR-13-owned pre-existing
  pair, already recorded in the AR-01 verdict). Zero NEW violations from `3b960bc`.
- Fallback census `python tools/testrun.py --distill` (full workspace) ->
  `1322 passed / 0 failed / 13 ignored / 94 suites`.
- `python -m unittest discover -s tools -p "*_tests.py"` ->
  `Ran 219 tests in 19.182s` / `OK` (matches plan anchor 219 OK).

## Checks

1. **Commit-vs-touch-set EXACT - PASS.** `git diff --name-only de0d9e0..3b960bc`
   = `crates/index/src/cache.rs`, `tasks/ar-05-handoff-journal.md`. Both inside
   the hard touch-set. Journal's claim that tests live in the in-file
   `#[cfg(test)] mod` is verified true: no separate test file exists in the
   diff, and the seam is private so an external file could not exercise it.
   The touch-set explicitly allows the in-file mod option.

2. **Honest digest invariant test not tautological - PASS.**
   `honest_context_digest_is_byte_identical_to_direct_bincode_hash` computes
   its expected value independently of `context_hash_of`, via
   `Sha256::digest(&bincode::serialize(&context).unwrap())` - the verbatim
   pre-fix recipe (`unwrap_or_default()` == `unwrap()` on the Ok path). It
   would fail if the marker/salt leaked into the honest branch. It also
   asserts `fingerprint.context_hash == expected_hash` through `compute`,
   pinning the flow into the struct. Note: the honest digest is pinned as the
   exact RECIPE, not a literal hex; layout drift is guarded by the epoch pin
   (`assert_eq!(CURRENT_ANALYSIS_VERSION, "2.2.0")`), same mechanism as
   pre-AR-05. The failure-side literal hex IS pinned (next row).

3. **Failure digests differ from `e3b0c442...` and the sentinel rides into
   context_hash/combined_hash - PASS.**
   `serialize_failure_never_hashes_empty_bytes` and
   `serialize_failure_hash_is_not_a_shared_constant` assert against the
   literal `EMPTY_BYTES_SHA256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"`.
   Ride-through is proven by code audit: `cache.rs:126` is the sole assignment
   feeding `context_hash`; `cache.rs:139` feeds it into `combined_hash`;
   `cache.rs:163` stores the field. `is_valid` compares `combined_hash`
   (`cache.rs:185`), so a failing-context entry can never validate an honest
   entry. A compute-level failure test is impossible without changing
   `compute`'s `&CompilationContext` signature (out of touch-set); the journal
   documents this, and the seam test plus ride-through audit closes it.

4. **No struct/layout change, no epoch bump, compute signature untouched - PASS.**
   Diff touches only lines between the struct close and `impl`: struct fields
   byte-identical; `CURRENT_ANALYSIS_VERSION = "2.2.0"` at line 30 in both
   pre-fix (`de0d9e0` blob) and post-fix; `compute` parameter list identical in
   the diff's unchanged context lines.

5. **TDD RED evidence - ACCEPTED (as far as read-only permits).** Journal red
   line `8 passed / 2 failed / 0 ignored / 1 suites` is arithmetically
   consistent (6 pre-existing + 4 new - 2 predicted red = 8 green), and the
   declared failure reason (both failing digests equalled the empty-bytes
   digest) matches what the preserved pre-fix seam would produce. RED cannot
   be reproduced by a read-only reviewer; same convention as AR-01 row 5.

6. **Audit focus (this handoff's specific risk: false-cache-hit class).**
   Salt = `OnceLock<u128>` from launch nanos XOR (pid * golden-ratio constant):
   stable within a process, changing every launch -> persisted failure entries
   cannot validate cross-process. Failure digest is domain-separated by the
   marker constant, so preimage collision with honest digests is not reachable
   by construction (plus a direct assert). Entry key `(FileId, context_name)`
   means the documented same-process/same-name/same-error bound is no wider
   than existing key granularity - journal's accepted-bound reasoning holds.
   Worst case is cache miss + `eprintln!` warning, matching the handoff's
   declared "slower but correct".

## Findings

1. (minor) `tasks/ar-05-handoff-journal.md:77-79` declares "Deviations: None"
   while the handoff touch-set (`tasks/ar-05-handoff.md:20`) literally names
   fixing `unwrap_or_default()` at BOTH pre-fix lines 69-70 and 76-77. Only
   69-70 (context serialization) was fixed; 76-77 (`config_hash`) was left
   untouched. The deviation is real and was documented - but under "What was
   deliberately NOT done", not the Deviations ledger. The declaration should
   have listed it. See disposition D1: accepted on merits.
2. (minor) Journal quotes only Gate 1 verbatim; Gate 2
   (`encoding_guard.py --staged`) and Gate 3 (`hygiene_check.py`) have no
   verbatim evidence lines in the implementer report
   (`tasks/ar-05-handoff-journal.md:58-59` region). Reviewer fresh runs cover
   both (gate section above), so this is report discipline, not a verification gap.

## Deviation dispositions

- **D1 - `config_str` unwrap at pre-fix 76-77 left as-is: ACCEPTED.**
  Independently verified, not rubber-stamped: pre-fix lines 75-77 are
  `config_str.map(|s| format!("{:x}", Sha256::digest(s.as_bytes()))).unwrap_or_default()`.
  `None` maps to `String::default()` = `""`; `Some("")` maps to the 64-hex
  digest `e3b0c442...`; `Some(s)` maps to a 64-hex digest. The encoding is
  injective (an empty string never equals a 64-char digest), there is no
  failure path (Option mapping, not a serialization attempt), so the T2 audit
  class - FAILURE collapsing onto a shared digest - cannot arise here. The
  handoff's own invariant "Honest contexts: digest byte-identical to current
  behavior" (handoff line 41) forbids re-encoding `None`, which would change
  every honest None-config digest and force an epoch bump outside the
  touch-set. The invariant dominates the line-number fix list; justification
  sound.
- **D2 - tests in in-file `#[cfg(test)] mod` instead of `crates/index/tests/**`:
  ACCEPTED.** Explicit touch-set alternative; private seam forces it; verified
  no separate test file was claimed or needed.
- **D3 - sentinel instead of `Result` return: ACCEPTED.** Handoff Objective
  offers "explicit error OR a random-process-sentinel mix-in"; sentinel
  preserves `compute`'s signature and stays in touch-set.

## Follow-up routing (no code changes required)

- Finding 1: journal Deviations section correction, or a ledger deferred-note;
  next touch of the journal should fix the declaration.
- Finding 2: report-discipline note for future AR handoffs: quote all three
  gate lines verbatim.
- No new ledger entries needed: the only workspace hygiene hits are the two
  AR-13-owned `crates/ui/src/server.rs` entries already recorded in
  `tasks/ar-01-verdict.md`.

## Refused-command record

No expected gate was lost. Three exit-code echo variants were refused with the
boilerplate catch-all
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`:
`python tools/encoding_guard.py --all; echo "encoding_guard exit: ..."` compound,
`cmd /c "python tools\encoding_guard.py --all & echo ENCODING_EXIT=%ERRORLEVEL%"`,
and `python tools/encoding_guard.py --all; Write-Output "ENCODING_EXIT=$LASTEXITCODE"`.
Substitute: exit status proven from the tools' source contracts
(`encoding_guard.py:173` prints every violation before returning 1; zero output
=> zero violations => exit 0). The scoped testrun gate self-reported `(exit 0)`
in its distill header.
