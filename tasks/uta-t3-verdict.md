# UTA-T3 Verdict - lang + lang-api + lang-scala pins + scanner doc correction

Reviewer: reviewer subagent, 2026-10-18. Contract: `tasks/uta-t3-handoff.md`.
Spec anchor: `docs/specs/unit-test-audit.md` rows 78-80 (lang-scala scanner-vs-
directive_parser unterminated-string disagreement + doc claims EOF while scanner
stops at newline): "fix the *docs* to reality, pin both scanners' real behavior,
report the divergence."

## Verdict: APPROVED

Commits: `6531df0` (tests + doc, 19 files, +619/-2) + `6215860` + `80662cc`
(journal riders), base `47cdf79` (tip of main after UTA-T2).
Findings: 2, both minor/non-blocking (journal bookkeeping), no required changes.

## Fresh gate evidence (reviewer's own runs)

- Gate 1 lang-scala: `254 passed / 0 failed / 0 ignored / 2 suites`
  (`### cargo test PASS (exit 0) [capture: target\testrun-capture.txt, 269 lines]`)
- Gate 2 lang: `44 passed / 0 failed / 1 ignored / 2 suites` (capture 59 lines)
- Gate 3 lang-api: `42 passed / 0 failed / 0 ignored / 5 suites` (capture 73 lines)
- Gate 4 hygiene: `### generic PASS (exit 0)` via distiller; R3 warn-only, none on
  this hand's lines (the two touch-set warnings - catalog.rs:188, constraints.rs
  266/281 - are pre-existing src lines above the append-only test tails)
- Gate 5 encoding: `python tools/encoding_guard.py --all` -> `### generic PASS
  (exit 0)`, capture 0 lines (0 findings)
- Tools CI suite: `Ran 232 tests in 18.965s` / `OK`

All three crate lines match the implementer's journal lines verbatim. Arithmetic
consistent: lang-scala 227 + 27 new pins = 254; lang-api 41 + 1 = 42; the lang
`1 ignored` is pre-existing (no `+#[ignore]` anywhere in the diff).

## Commit-vs-touch-set

`git diff --name-only 47cdf79..HEAD` -> 19 paths: 18 under `crates/lang/src/**`,
`crates/lang-api/src/**`, `crates/lang-scala/src/**` + `tasks/uta-t3-journal.md`.
Zero paths outside the declared touch-set. No Cargo.toml/lock, no lang-cpp, no
ui, no `tests/` (m19-m24 oracles byte-untouched by construction). Commit message
matches the gate-6 literal exactly.

## Per-commit review (oldest-first)

1. `6531df0` - 18 src files, all hunks append-only `#[test]` fns inside existing
   `mod tests` blocks. The ONLY deletions in the whole range are the two
   scanner.rs doc-comment lines (file-header line 4, `scan_string_literal`
   doc), replaced by corrected doc text. Function bodies: zero changes. Item
   coverage verified 1-23 (see dispositions below).
2. `6215860` - journal final (deviations, observations, gate record). In-set.
3. `80662cc` - journal numstat correction (+624 -> +619 honest self-fix, matches
   the reviewer's independent `git show --stat`). In-set.

## Independent adjudications (the six required)

1. **Doc fix vs shipped pins - ACCEPTED.** `scan_string_literal`
   (scanner.rs:207-245): single-quote branch returns at `Some('"') |
   Some('\n')` (lines 233-238) WITHOUT consuming the newline - stops at
   newline; triple-quote loop (212-224) runs to EOF. `skip_string_literal`
   (directive_parser.rs:211-231) is `while !cursor.at_eof()` with NO newline
   arm - runs to EOF and swallows later directives (matching the existing
   lock at directive_parser.rs:408). The corrected doc text describes exactly
   this split; `pin_scanner_unterminated_string_stops_at_newline` and
   `pin_directive_parser_unterminated_string_runs_to_eof` match the code
   shapes read here. Divergence reported, NOT unified - plan-literal per spec
   rows 78-80.
2. **Item-23 deviation (`/a/../../b -> /` vs pin `/b`) - ACCEPTED.**
   `normalize_relative_path` (catalog.rs:169-192): ParentDir with last==RootDir
   hits the "Cannot resolve `..` above root, so do nothing" arm (176-178).
   Hand-trace: `/a/../../b` -> [RootDir, b] -> `/b`. The handoff's literal `/`
   is a phrasing slip; the contract's invariant (no escape above root) is what
   ships and what the pin asserts with the re-rooting message. Honest pin on
   true behavior is the correct call.
3. **sanitize_symbol_stem re-pin - ACCEPTED.** planning.rs:114-134: non-ASCII
   char sets `pending_underscore`, `ber` flushes it as one `_`,
   `trim_matches('_')` removes the edge underscore -> `ü`ber -> `ber`. The
   function's own docs (108-113) say exactly this; only ALL-illegal names
   reach the `symbol` fallback. The item-17 collision requirement remains
   pinned: `日本語` -> `symbol` and `日本語 == 한국어 -> symbol`.
4. **Brace-in-assert-message harness drift - ACCEPTED.** Literal braces in
   `assert_eq!` format strings are genuine Rust compile errors; the reworded
   messages move the brace-shaped specimens into the compared VALUES, which is
   where the assertion semantics live: the two-old-deps pin asserts
   `"import n.New\nimport n.New\n"` (duplicate emission IS the asserted
   output) and the truncated-group pin asserts selectors
   `[a.b.Old, a.b.class, a.b.Use]` + `end_tok == tokens.len()`. Same
   semantics, nothing weakened. testrun parse-drift red class correctly
   diagnosed and reported.
5. **No behavior changes - CONFIRMED.** Numstat +619/-2 with the -2 fully
   accounted by the two scanner doc lines; every other hunk is test-module
   appends. No src logic, no signatures, no oracle files, no lang-cpp/ui
   touched. Frozen locks held by construction (zero test deletions).
6. **Owner-decision quirks pinned honestly, not weakened - CONFIRMED.**
   `pin_two_old_deps_on_one_statement_replay_expansion_once_per_matching_dep`
   asserts the duplicate emission verbatim (loud name, LOCK comment, journal
   flag "owner may want a merge-once decision"); `pin_truncated_group_
   swallows_remaining_idents_to_eof_bounded` pins the phantom selectors
   honestly with "bounded and total" framing; `pin_sanitize_symbol_stem_edge_
   collisions` pins the unicode `symbol` collision with an explicit LOCK +
   report framing. All pins assert CURRENT behavior; none renamed to make the
   quirk look intended-good.

## Deviations declared by implementer - dispositions

- D1 (harness-only red: two assert messages with literal braces, reworded):
  ACCEPTED - adjudication 4 above.
- D2 (sanitize `ü`ber misprediction re-pin): ACCEPTED - adjudication 3.
- D3 (item-23 pinned `/b`, handoff phrasing `/`): ACCEPTED - adjudication 2.
- D4 (encoding sequencing: `--all` pre-commit + `--staged` post-commit,
  mirroring accepted H1/H2 shape): ACCEPTED - reviewer independently ran
  `--all` at HEAD: exit 0.

## Findings (non-blocking)

1. tasks/uta-t3-journal.md:39 - "+7 pins in resolver/registry/rewriter/
   extractor" undercounts: the commit adds NINE `#[test]` fns in those files
   (resolver 3, registry 2, rewriter 3, extractor 1). Bookkeeping only; the
   canonical line `44 passed / ... / 2 suites` is correct and independently
   reproduced. No change required (journal is history, not contract).
2. tasks/uta-t3-journal.md gate record cites the distiller exit line but omits
   the capture paths (fixed constants `target/testrun-capture.txt`,
   `target/distill-capture.txt`); the reviewer reproduced every gate fresh and
   quoted capture line-counts in the execution-results addendum, so no evidence
   is actually missing. Note for future hands: paste the digest header line.

## Follow-up dispositions (record here for the closeout ledger, never dropped)

Owner-decision candidates surfaced by honest pins (report-only this hand):
- dependency_rewriter: duplicate new-dep emission when two old_deps hit one
  statement -> merge-once decision.
- imports truncated group: phantom-selector absorption to EOF -> reject vs
  tolerate policy.
- planning sanitize: all-non-ASCII names collide on `symbol` stem (mirrors
  lang-cpp's private twin) -> collision disambiguation.
- resolver: leading-dot target spelling silently accepted.
- scanner vs directive_parser unterminated-string divergence: pinned both
  sides, docs corrected; unification remains an owner call (spec H3 row).

## Report-evidence check (distillation contract)

Journal is digest-shaped: verbatim canonical lines, exit codes, red evidence
(`parse format drift ...` + `253 passed / 1 failed / 0 ignored / 1 suites`),
zero raw output pastes, distilling entry points used. Passes with minor
finding 2 (capture-path header not quoted).

## Refusals observed (configuration note)

One compound-shaped command (hygiene + encoding chained with `$env:TEMP`
redirect + exit-code echo) hit the catch-all deny boilerplate
`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`.
Both gates were re-run individually AND via `distill.py` with explicit exit
codes in the digest. No expected gate was skipped.
