# AR-24 Verdict - Typed errors at the plugin seam (A7)

**VERDICT: APPROVED.**

Range: `cefbd6f..d4c608c` on `shining-enthusiasm`.
Commits reviewed per-commit, oldest first:
- `8497a67` refactor(lang-api): typed error enum at the plugin seam (A7) - 18 paths, single cross-crate commit
- `d4c608c` docs(tasks): AR-24 journal - landing hash record (journal-only rider commit, AR-15/AR-16 precedent)

Fresh verification ran on the committed tree (tree clean, `checkpoint_guard status`:
HEAD `d4c608c`, dirty = untracked task docs only, no other lane live).
All gate lines are reviewer own-run stdout in `tasks/ar-24-execution-results.md`.

## 1. Scarcity claim - VERIFIED (central)

Independently checked against the pre-change tree, not the journal:

- `LanguagePlugin` trait (pre-change `cefbd6f:crates/lang/src/registry.rs`, read whole):
  all methods value- or Option-returning. `analysis_fingerprint` returns a bare
  `String` (a value, not an error surface) - correctly out of scope. The only
  error-typed path through the seam is `validator() -> &dyn CompilerValidator`.
- Pre-change `crates/lang/src/validator.rs`: `validate_compile` was the trait's
  only `Result<_, String>` method; `toolchain_status` returns `ToolchainStatus`.
- Whole pre-change tree: every remaining `Result<[^>]*String>` site is either a
  free function outside the seam (`cli/registry.rs::parse_language_arg`,
  `incremental_backend` - both named in the decision table as AR-25 territory,
  both UNEDITED and green), a test helper (`pipeline/tests/for_incremental_parity_test.rs`),
  an inherent non-trait method (`validate/src/lib.rs:323 verify_restored`), or a
  payload-generic (`Result<String>`/`Result<Vec<String>>` success types).
  The lang sub-traits (analyzer/detector/resolver/constraints/extractor/
  rewriter/unit) carry zero `Result<_, String>` pre- and post-change (files
  untouched by the commit; current-tree grep = pre-state).

No second String-erroring seam method exists. The single-method conversion
discharges "every trait method returning String". Not an incompleteness.

## 2. Byte-stability - VERIFIED against pre-change blobs

- All six seam strings independently re-derived from `cefbd6f` blobs and
  compared to the lock literals in `8497a67`: C++ placeholder (plugin.rs),
  `COMPILE_DEFERRAL` constant (line-continuation whitespace semantics applied,
  byte-equal to `ar24_deferral_error_text_is_byte_locked`), the three
  `{binary} exited unsuccessfully: {detail}` variants (stderr-first / stdout
  fallback / "no compiler output" chain byte-identical pre/post), and
  `{binary} became unrunnable between probe and compile`. All exact.
- `Display` is pure passthrough: `#[error("{message}")]` on all three
  `PluginError` variants (read `crates/lang-api/src/error.rs`).
- Counts reconcile: pre-change characterization capture added the byte-locks
  first (cpp 153->154, scala 221->226, green BEFORE conversion - they compile
  against `String` via `to_string()`); the +1 each post-change is the new
  kind test (`ar24_validate_compile_error_branches_by_kind`,
  `ar24_error_kinds_discriminate_unrunnable_from_failed_validation`).
  lang-api 37->41 = the 4 seam tests.
- Mutation evidence (journal-recorded, reverted): Mutation A (CompileFailure
  Display prefix) -> exactly the 3 compiler-failure byte-locks red
  (`224 passed / 3 failed`), rest green. Mutation B (kind-collapse with
  byte-identical message) -> only the kind test red (`5 passed / 1 failed`
  under ar24 filter; the 5 byte-locks green). Text and kind lock independently.
  Reviewer cannot re-run mutations (read-only); the arithmetic is internally
  consistent and the locks are independently non-vacuous: literals are test-
  local constants, not references to implementation strings.

## 3. Touch-set - 18/18 inside declared scope + declared riders

`git diff --name-only cefbd6f..HEAD` = exactly 18 paths, all ∈ handoff touch-set
∪ declared riders ∪ dispatch-preauthorized dep files:

- Inside literal touch-set: lang-api (error.rs, lib.rs, tests/, Cargo.toml),
  lang (validator.rs, lib.rs [D3], registry.rs mock - in-lane test), lang-cpp
  plugin.rs, lang-scala validator.rs + tests, journal.
- Riders: workspace `Cargo.toml` + `Cargo.lock` + lang-api `thiserror` (R1,
  dispatch pre-authorized "workspace Cargo.toml dep addition allowed for it");
  lang-api dev-dep `anyhow` (R2 = D1); compile-forced mock signatures
  graph/builder.rs, scheduler/worker.rs, validate/lib.rs, m17, m24 (D2).
- `crates/cli/src/registry.rs` UNTOUCHED as mandated; cli suite matches baseline
  exact (`114 passed / 0 failed / 8 ignored / 6 suites`), zero cli paths in the diff.
- Undeclared over-reach: none found.

**D2 adjudication (independent, on merits): ACCEPTED.** Verified each hunk:
every rider hunk changes only the `fn validate_compile(...) -> Result<(),
ariadne_lang::PluginError>` signature (reformatted multi-line + one `AR-24`
comment line); bodies stay `Ok(())`; `toolchain_status` untouched; zero
assertion/wording-line edits anywhere. `git diff cefbd6f..HEAD -- tests/
integration` returns ONLY m17 (1 signature line) and m24 (1 signature line);
m18 parity and both m16 files are file-ZERO-diff. E0053 is genuinely
unavoidable: the trait signature change breaks every impl by construction, so
the handoff's own STRICT COMPILE-GREEN mandate forces it. The *purpose* of the
UNEDITED constraint - external proof that messages didn't shift - survives
intact: the wording locks are byte-untouched and green at pinned counts.
**D1 adjudication: ACCEPTED** (see findings-free disposition below - dev-only
transport lock for exactly the AR-25 downcast contract; anyhow already a
workspace dep; production seam stays thiserror-typed; not speculative).
**D3 adjudication: ACCEPTED** - one-line targeted re-export per ToolchainStatus
precedent; without it graph/scheduler/validate need a new lang-api dep (worse).

## 4. Zero production string-matchers - VERIFIED

Workspace grep of `validate_compile`: 40 hits, ALL are trait definitions,
impls, mocks, or tests. Zero production callers. Grep for `.contains(` /
`.starts_with(` on the error texts ("not wired", "deferred", "exited
unsuccessfully", "unrunnable", "unhandled"): zero production matchers. Claim holds.

## 5. m16 oddity - reconciled

The journal's two m16 lines are TWO different files, not one contradictory
figure: `m16_traversal_test` and `m16_clang_worker_test`. Fresh own-run:
traversal `2 passed / 0 failed / 0 ignored / 1 suites`; clang-worker
`3 passed / 0 failed / 0 ignored / 1 suites` - both file-ZERO-diff (zero paths
in range). The "2 vs 3" tension is the KNOWN stale pin from the AR-20a review
(D1 there: clang-worker pinned `2`, measured `3`, ACCEPTED, row-banked for the
ledger, finding 3) - AR-24's journal correctly cites the post-shift measured
values. **True current m16 census for the ledger: traversal `2/0/0/1`,
clang-worker `3/0/0/1`.**

## 6. Fresh gates (own stdout; full evidence in execution-results addendum)

1. `-p ariadne-lang-api` -> `41 passed / 0 failed / 0 ignored / 4 suites`
2. `-p ariadne-lang` -> `35 passed / 0 failed / 1 ignored / 2 suites`
3. `-p ariadne-lang-cpp` -> `155 passed / 0 failed / 0 ignored / 2 suites`
4. `-p ariadne-lang-scala` -> `227 passed / 0 failed / 0 ignored / 2 suites`
5. m18 parity `23 passed / 0 failed / 0 ignored / 1 suites` · m17 seam
   `8 passed / 0 failed / 0 ignored / 1 suites` · m24 identity
   `8 passed / 0 failed / 0 ignored / 1 suites` · m16 traversal
   `2 passed / 0 failed / 0 ignored / 1 suites` · m16 clang-worker
   `3 passed / 0 failed / 0 ignored / 1 suites` · m00
   `4 passed / 0 failed / 0 ignored / 2 suites`
6. cli `114 passed / 0 failed / 8 ignored / 6 suites` · scheduler
   `26 passed / 0 failed / 0 ignored / 2 suites` · graph
   `129 passed / 0 failed / 0 ignored / 6 suites`
7. `encoding_guard.py --all` silent; `hygiene_check.py` zero R1/R2 blocking
   rows (R3 warn-only, none inside this diff - builder.rs rows :574+ and
   validate :104 are pre-existing lines outside the hunks at :452 / :798).
8. Workspace census (no other lane live):
   `1393 passed / 0 failed / 13 ignored / 102 suites`; tools
   `Ran 219 tests in 19.908s` / `OK`.

## 7. Report-evidence check (distillation contract)

Journal is digest-shaped: canonical testrun lines verbatim, capture paths
named (`target/testrun-capture.txt`, `target/distill-capture.txt`), zero raw
dumps, gate commands in testrun/distill form. PASS.

## Findings

1. **[minor, ledger-row-bank]** m16 census: true current values are traversal
   `2/0/0/1` and clang-worker `3/0/0/1` (both UNEDITED, green). The AR-20a
   row-bank item (clang-worker pin `2` -> measured `3`) is still open at the
   next ledger touch. Not AR-24's error - AR-24 cited measured truth.
2. **[minor, tooling, re-confirmed]** `testrun.py` bare-mode echo crashes
   `UnicodeEncodeError` (cp1252) when cargo output carries non-ASCII -
   reproduced this review on `m16_traversal_test` (traceback at
   `tools/testrun.py:262`); workaround `PYTHONIOENCODING=utf-8`. Re-confirms
   AR-20a finding 4; the tooling-owner row should finally land.
3. **[minor, environment/config]** This session's global bash deny rule
   (`{"permission":"bash","pattern":"*","action":"deny","source":"global"}`)
   refuses `python -c`, `git grep`, and chained `echo` (all refused with that
   boilerplate). Every expected gate still ran (via `git show` blobs, the
   grep/glob tools, and the `tools/*.py` entry points); no gate was skipped or
   substituted. Exit codes remain unechoable - hygiene/encoding adjudicated
   on printed contracts per AR-01/02/03 precedent.
4. **[minor, journal wording]** Landing paragraph cites "declared riders
   R1-3" but only R1/R2 carry labels; the third rider (compile-forced mock
   signatures = D2, and/or the journal-only docs rider commit) is never
   explicitly labelled R3. Cosmetic; label it on next journal touch.
5. **[info, no action]** Mutation A's capture cites the lang-scala suite only;
   the lang-api Display-passthrough sentinel test would also go red under that
   mutation (uncited, implied). Non-vacuity is proven regardless.

## Dispositions to record (not dropped)

- D1 (dev-dep anyhow): ACCEPTED as sanctioned forward-compatibility - locks
  the live anyhow-downcast behavior AR-25 consumes, at the boundary that just
  landed; zero production-graph change.
- D2 (mock signatures): ACCEPTED - compile-forced, signature-only, lock
  semantics fully preserved (proofs in §3).
- D3 (lang/lib.rs re-export): ACCEPTED.
- Findings 1-2 go to `tasks/milestone_ledger.md` deferred-findings / row bank.

## Clearance

On this APPROVED: **AR-25** (cli riders, consumes the D1 downcast lock; owns
the analyze.rs:39 `.map_err(anyhow::Error::msg)` drop and the cli String
helpers) and **AR-21** (lang-cpp resolver/gate) are cleared for parallel
dispatch - file-disjoint per the locked tail plan; lang-cpp is quiet after
AR-21 for AR-31 (tokenizer), unchanged.
