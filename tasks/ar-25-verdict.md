# AR-25 Verdict - CLI consumes typed seam errors (A7 riders)

**VERDICT: APPROVED.**

Range: `d4c608c..b7136db` on `shining-enthusiasm`. Commits under review are
the four AR-25 commits, interleaved with the parallel AR-21 lane (`7797edb`,
`3e874ab`, `9fa9a1c` - not judged here; the AR-21 journal appearing in the
range-wide `git diff --name-only` is the parallel lane's own commit, correctly
disclosed in the AR-25 landing record and absent from every AR-25 commit's
diffstat):

- `2aeae5e` docs(tasks): journal skeleton - journal-only, in touch-set
- `96739ad` test(cli): RED - `registry.rs` +52 (enum declaration, unwired) +
  `crates/cli/tests/ar25_seam_error_chain_test.rs` +184 + journal; production
  `analyze.rs` untouched (re-wrap still live at RED - correct TDD shape)
- `2f3b5e7` refactor(cli): GREEN - re-wrap drop + wiring + test mirror + m24
  render token
- `b7136db` docs(tasks): journal landing record - journal-only

Ground truth at adjudication (`checkpoint_guard status`): HEAD `b7136db`,
tracked tree clean (untracked `tasks/ar-*` docs only). All gate lines below
are reviewer own-run stdout; verbatim table in
`tasks/ar-25-execution-results.md`.

## 1. Re-anchoring - VERIFIED

- Base blob `d4c608c:crates/cli/src/commands/analyze.rs`: the sole
  `.map_err(anyhow::Error::msg)` sits at lines 38-39 (the GREEN hunk
  `@@ -35,8 +35,10 @@` positions `.map_err(anyhow::Error::msg)?;` at old
  line 39) - the contract's `~:49` cite is stale, the journal's `:39`
  re-anchor is exact.
- `registry.rs::incremental_backend` at base returns
  `Result<&'a dyn ariadne_lang::LanguagePlugin, String>` at ~:104-105 (blob
  read; journal's :105 within count) - the String-typed return IS the
  re-wrap layer there, as the journal states.
- Workspace grep at HEAD: zero `.map_err(anyhow::Error::msg)` code
  occurrences in `crates/*/src`; the sole code occurrence anywhere is the
  deliberate pre-form witness at
  `crates/cli/tests/ar25_seam_error_chain_test.rs:149` (grep-tool sweep,
  `git grep` denied by allowlist). The grep lock holds.

## 2. Plan-literal compliance + byte-stability - VERIFIED against base blobs

- The contract's objective (drop the two re-wrap sites) is implemented
  literally: `analyze.rs:39` becomes plain `?`; `incremental_backend` returns
  `anyhow::Result` carrying `anyhow::Error::from(IncrementalDispatchError::…)`.
- All three Display renderings re-derived byte-identical from the
  `d4c608c` blob: the zero-units literal (same line-continuation
  whitespace), `unhandled language: no plugin is registered for {language:?}
  units` (same Debug-formatted `SourceLanguage`), and the mixed form where
  `{languages:?}` receives the SAME first-seen-order `distinct` vec the old
  `{distinct:?}` interpolated - order and bytes preserved.
- The RED suite's byte locks are test-local literals, not references to
  production strings (non-vacuous by AR-24 §2 standard), and the red-run
  ordering (byte-lock `assert_eq` passes BEFORE the downcast panic) is the
  forced-scenario same-bytes-before/after evidence the handoff invariant
  demands.
- External string authority: m18 parity file-ZERO-diff (git diff empty) and
  green at `23 passed / 0 failed / 0 ignored / 1 suites` own-run, including
  `empty_registry_backend_reports_unhandled_language_never_cpp`; m00 both
  files ZERO-diff, green `4 passed / 0 failed / 0 ignored / 2 suites`.

## 3. Touch-set - exact on 3/4 commits; m24 is the declared D1 (accepted)

- `2aeae5e`, `96739ad`, `2f3b5e7` (ex m24), `b7136db`: every path ∈ declared
  touch-set. `crates/report/src/manager.rs` ZERO-diff (fold disposition
  below). `crates/lang-api` ZERO-diff across the range - D1 (AR-24) was
  CONSUMED from the cli side (the new test's second leg downcasts
  `ariadne_lang::PluginError` through `anyhow::Error::new`) and neither moved
  nor re-created.
- `2f3b5e7` also touches `tests/integration/tests/m24_language_identity_test.rs`
  - OUTSIDE the literal touch-set, pre-declared as D1. Adjudicated on merits
  below: ACCEPTED.
- Undeclared over-reach: none found. The three `.to_string()` render tokens
  inside `crates/cli/src/registry.rs`'s own `mod tests` (GREEN hunk) are
  inside the declared file, compile-forced by the signature change, and
  assertion wording is byte-untouched (finding 3 records the framing gap).

## 4. Deviation dispositions

- **D1 (m24 one `.to_string()` token): ACCEPTED.** Verified the whole hunk:
  one `.to_string()` appended after `.expect(...)` on the seam-error
  extraction (anyhow::Error lacks `str::contains`; the wording assert below
  is byte-untouched). This is exactly the AR-24 verdict §3 D2 class the bank
  already adjudicated ("E-unavoidable: the signature change breaks every
  consumer by construction; STRICT COMPILE-GREEN forces it"). The UNEDITED
  constraint's PURPOSE - external proof that CLI-visible strings did not
  shift - survives intact: m18 and both m00 files are file-ZERO-diff and
  green, m24 green at pinned `8/0/0/1`.
- **D2 (anyhow mechanism correction): ACCEPTED - independently CONFIRMED
  TRUE against the pinned dependency source.** `Cargo.lock` pins anyhow
  1.0.104; `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-
  1.0.104/src/error.rs:77-82` shows `Error::msg` -> `construct_from_adhoc`,
  and error.rs:167-188 wraps `M` in the `#[repr(transparent)]`
  `MessageError<M>` while binding `object_downcast: object_downcast::<M>` -
  so msg PRESERVES the payload's own type identity. The contract's assumed
  mechanism (type erasure by msg) was wrong; the erasure actually removed is
  of the dispatch KIND (a String payload never had a branchable kind).
  Non-vacuity of the lock adjudicated by mutation reasoning (tree untouched,
  read-only): (a) RED was mechanically forced regardless of the msg
  question - pre-fix the payload type was String, so
  `downcast_ref::<IncrementalDispatchError>()` cannot be Some at 96739ad;
  (b) a hypothetical half-fix that left `.map_err(anyhow::Error::msg)` at
  the call site over the new `anyhow::Result` would produce an
  `M = anyhow::Error` payload - direct downcast to the seam enum still
  fails (anyhow's `downcast_ref` does not walk into a MessageError-wrapped
  inner error) - so all three seam locks go red: the lock has teeth against
  re-wrap reintroduction; (c) the consuming-form leg (Error::new over a real
  PluginError -> downcast Some, Display verbatim) would fail if the
  transport broke. The committed witness's `is_none()` leg is necessarily
  true under the corrected mechanism - that IS its claim (String payload =
  no kind); the three seam locks carry the RED/GREEN burden.
- **D3 (`parse_language_arg` stays String): ACCEPTED.** Verified the premise:
  wired as clap's `value_parser` at `crates/cli/src/lib.rs:148` and `:181`
  (clap requires `Err: Display`, not an anyhow seam), no
  `.map_err(anyhow::Error::msg)` ever consumed it, and the contract's
  objective (re-wrap drops + StaleArtifact fold) does not name it.
  Objective-scoped leave per the AR-24 decision table's own disposition.
- **D4 (AR-23 F4 double-`default_registry()` cleanup SKIPPED): ACCEPTED -
  the contract never mandated it.** The contract text (objective, touch-set,
  constraints, gate list, invariants) contains no mention of the second
  registry build. The bank row itself is labelled "AR-23 informational F4 ...
  cheap cleanup candidate for AR-25's cli visit"
  (`tasks/ar-closeout-deferred-rows.md:87-89`) - a candidate, not an
  obligation, and the implementer is right that collapsing it touches
  D9-locked registry selection for zero behavior impact, unrelated to the
  re-wrap drop. **Disposition: the row stays OPEN for closeout; not AR-25
  debt.** (F4's premise verified intact: `analyze.rs:35` still builds its
  own `default_registry()`.)

## 5. StaleArtifact fold conditional - EXISTS in the contract; disposition VERIFIED

The conditional is contract text (`tasks/ar-25-handoff.md:12-13`: "fold the
bespoke `StaleArtifact` reporting onto the shared error pattern where trivial
(where non-trivial, journal why and leave it)"). The LEAVE disposition is
correct on the merits, all three legs independently verified:
1. `report/manager.rs`'s `StaleArtifact` already IS the shared typed-error
   pattern - hand-rolled `Display` (:31) + `Error` (:41), carried via
   `anyhow::Error::from` (:52), consumed by `is_stale` = `downcast_ref`
   (:43-44). Nothing to fold onto itself.
2. The only plausible "folds" (thiserror = new report dep; PluginError =
   taxonomy falsification) are non-trivial/wrong, exactly as journaled.
3. The fold's consumer premise is void: workspace grep shows zero bespoke
   StaleArtifact handling in cli (every `is_stale` consumer is report-internal).
`crates/report/src/manager.rs` is file-ZERO-diff and the report census is
byte-identical to baseline (`47 passed / 0 failed / 0 ignored / 4 suites`).

## 6. Fresh gates (own stdout - full table in the addendum)

cli `118 passed / 0 failed / 8 ignored / 7 suites` · report
`47 passed / 0 failed / 0 ignored / 4 suites` · m24 `8 passed / 0 failed / 0
ignored / 1 suites` · m00 combined `4 passed / 0 failed / 0 ignored / 2
suites` · m18 parity `23 passed / 0 failed / 0 ignored / 1 suites` ·
`cargo check --workspace --all-targets` exit 0 (31 warnings, zero naming any
AR-25-touched file) · hygiene zero R1/R2 blocking rows (R3 warn-only, all
pre-existing) · `encoding_guard --all` silent.

## 7. Report-evidence check (distillation contract)

Journal is digest-shaped: canonical testrun lines verbatim, capture paths
named (`target/testrun-capture.txt` / `target/distill-capture.txt`) with the
capture-race caveat quoted from own stdout, exit codes stated, zero raw
dumps, gate commands in testrun/distill form. PASS.

## Findings

1. **[minor, tooling - re-confirmed]** The cp1252 crash fired again this
   review on the BARE `python tools/testrun.py -p ariadne-cli` (traceback
   `tools/testrun.py:262`, progress-test box-drawing bytes): the banked
   one-tools-ticket row (`tasks/ar-closeout-deferred-rows.md:80-83`) is now
   4 reviews deep (AR-23, AR-24, AR-25 reviewers + implementer lanes).
   Workaround `PYTHONIOENCODING=utf-8`.
2. **[minor, journal wording]** "cargo check ... 21 warnings emitted" is
   cache-attribution-dependent: this review measured 31 under
   `--all-targets` with a cold lang-cpp lib-test cache (the delta is exactly
   the 10 AR-21 `IncludeResolver` deprecation warnings). The load-bearing
   claim - zero warnings name any AR-25-touched file - verified true from
   this run's own capture. Quote warning counts as cache-scoped at the next
   journal touch.
3. **[minor, framing]** The journal's D-list doesn't enumerate the three
   `.to_string()` render tokens added inside `crates/cli/src/registry.rs`'s
   own test mod (GREEN hunk). In-declared-file and compile-forced, so not a
   violation - but same class as D1 and worth one journal line next touch.
4. **[minor, contract authoring - for closeout]** The contract's lock
   mechanism premise (`Error::msg` destroys type identity) is factually
   wrong for the pinned anyhow 1.0.104 (see §4 D2); the AR-25 design/journal
   section now documents the correct mechanism. Worth an erratum row so the
   wrong premise doesn't propagate into future handoff text (the dispatch
   phrase "PluginError propagates and callers downcast" inherits it: no
   PluginError physically reaches this seam - AR-24 §4 proved zero
   production `validate_compile` callers - and the design's separate
   `IncrementalDispatchError` was the correct, taxonomy-preserving reading
   of the contract's generic "TypedSeamError" lock spec).
5. **[info, ledger note]** Expected census delta from this handoff: cli
   114->118 (new 7th suite +4 tests). Next workspace census projects
   1393+4+AR-21's delta from the `1393 passed / 0 failed / 13 ignored / 102
   suites` @AR-24 anchor (this review ran scoped gates only per dispatch
   constraint - see addendum).

## Clearance

On this APPROVED: cli keeps NO `.map_err(anyhow::Error::msg)` - the A7 drop
is complete end-to-end (AR-24 seam types + AR-25 consumer). F4 stays a
closeout candidate (D4 disposition). Findings 1-4 go to
`tasks/milestone_ledger.md` deferred-findings / the closeout bank.
