# AR-25 Journal - CLI consumes typed seam errors (A7 riders)

Base HEAD at start: `d4c608c` (AR-24 landing). Lane: cli (+ report disposition).
Parallel lane live (AR-21, lang-cpp): all evidence quoted from this session's
own stdout; shared capture files treated as clobber-prone.

## Re-anchoring (handoff line cites vs HEAD)

- `crates/cli/src/registry.rs` "~:105": VERIFIED at both `5de3884^` (pre-AR-19)
  and HEAD - line 105 is `incremental_backend`'s
  `Result<&dyn LanguagePlugin, String>` return. The "re-wrap layer" there is the
  String-typed return itself; the `.map_err(anyhow::Error::msg)` consumer site is
  `crates/cli/src/commands/analyze.rs:39` (handoff cite ~:49, shifted by AR-23's
  `d8559ae`). Workspace grep: this is the ONLY `.map_err(anyhow::Error::msg)`
  remaining in any crate's production code.
- `crates/cli/src/commands/analyze.rs` "~:49": re-anchored to :38-39.
- `parse_language_arg` stays `Result<_, String>` by design: it is clap's
  `value_parser` contract (`Err: Display`), not an anyhow seam, and no re-wrap
  site consumes it. AR-24's decision table names it AR-25 territory but AR-25's
  objective (re-wrap drops + StaleArtifact fold) does not require its
  conversion; converting would alter clap-rendering plumbing for zero typed-flow
  gain. Deliberately NOT done; journal reason per handoff discipline.

## Pre-change baselines (own-run stdout, verbatim)

- `python tools/testrun.py --distill -- -p ariadne-cli`
  -> `114 passed / 0 failed / 8 ignored / 6 suites`
  (handoff's "118/0/8/6 + AR-19/AR-23 deltas reconciled" superseded by the
  re-brief's 114 pin; own run confirms 114).
- `python tools/testrun.py --distill -- -p ariadne-report`
  -> `47 passed / 0 failed / 0 ignored / 4 suites`
  (handoff's "47/0/4i" stale-pin note: 4 = suites, measured ignored = 0).
- hygiene `python tools/hygiene_check.py` at HEAD: zero blocking rows (R1/R2);
  R3 warn-only rows pre-existing (~179) - per re-brief, pin "zero blocking".
- captures: `target/testrun-capture.txt` / `target/distill-capture.txt` (shared,
  clobbered by parallel lane between runs; canonical lines quoted from own
  stdout at run time).

## Design (what the contract's new lock forces)

- The lock "downcast_ref::<TypedSeamError> (or source() chain walk) succeeds on
  a forced error where today the re-wrap made it impossible" cannot target
  `PluginError` at this seam: `incremental_backend` errors are registration/
  dispatch facts, and `PluginError`'s three variants are documented compiler-
  validation kinds (AR-24 error.rs docs) - misfiling "unhandled language" into
  them would falsify the kind taxonomy. AR-24 verdict section 4 also verified
  zero production callers of `validate_compile` workspace-wide, so no
  PluginError physically reaches cli today.
- Therefore the cli seam gets its own typed error, `IncrementalDispatchError`,
  carrying the three dispatch failures as variants (kind, not string), hand-rolled
  `Display + std::error::Error` following the crate-family precedent that needs
  no new dependency (report's `StaleArtifact` pattern; AR-24's thiserror enum
  would mean a new cli dep = rider). Display renders the existing message
  strings BYTE-IDENTICALLY (three test-local literals in the new lock).
- The new integration lock additionally demonstrates the PluginError leg of the
  mission: forced `ariadne_lang::PluginError` through the dropped re-wrap form
  (downcast -> None, the erasure witness) vs the consuming form the seam now
  uses (downcast -> Some).
- `analyze.rs` drops the re-wrap: plain `?` over `anyhow::Result` - error chains
  stay intact end-to-end (AR-25 impact statement).

## RED evidence

Red commit adds the enum declaration (unwired) + `crates/cli/tests/
ar25_seam_error_chain_test.rs` (public-API integration lock). The lock helper
`forced_seam_error` mirrors the live call-site propagation verbatim at each
commit (`.map_err(anyhow::Error::msg)` at RED; flips to `?` together with the
production line at GREEN - one-line harness edit mirroring production).

Own-run digest (verbatim): `114 passed / 3 failed / 0 ignored / 4 suites`
(capture: target/testrun-capture.txt). The three failures are exactly the
predicted lines - each test's byte-lock `assert_eq` PASSED first (proving the
pre-fix forced-scenario string equals the locked literal), then the downcast
expect panicked:
- `ar25_sole_plugin_rejection_is_a_typed_source_through_the_seam` -
  "AR-25 layer-drop proof: the seam error survives anyhow as a typed source"
- `ar25_unhandled_language_is_a_typed_source_through_the_seam` - same panic
- `ar25_mixed_languages_is_a_typed_source_through_the_seam` - same panic
Semantic reason confirmed: pre-fix the seam payload is String; no typed kind
exists in the chain. Fourth test (mechanism leg) GREEN at RED.

Adjudication during red (anyhow 1.0.104, Cargo.lock:66): my first witness
asserted `.map_err(anyhow::Error::msg)` destroys TYPE identity for a
`PluginError` payload - that assertion FAILED, i.e. in 1.0.104 msg preserves
the payload's own type. Adjudicated on own-run evidence: the erasure AR-25
removes is of the dispatch KIND (String payload = no branchable kind), not of
bytes. Witness rewritten to that claim (`ar25_msg_string_payload_carries_no_
kind_the_consuming_form_transports_one`) and green at RED; the three seam locks
above are the contract's RED-before/GREEN-after proof.

## GREEN + gates

Green commit `2f3b5e7` (red `96739ad`, skeleton `2aeae5e`; base `d4c608c`).
Wiring: `incremental_backend` returns `anyhow::Result` carrying
`anyhow::Error::from(IncrementalDispatchError::…)`; `analyze.rs` propagates
via plain `?`; test-helper body mirrored to `?` with the production line;
m24 consumer got one `.to_string()` render token (wording assertions byte-
untouched - compile-forced, see D1).

Own-run gate lines (verbatim, this session's stdout; capture:
target/testrun-capture.txt):

- `-p ariadne-cli`: `118 passed / 0 failed / 8 ignored / 7 suites`
  (= 114 baseline + 4 new locks; ignored/suite delta +1 new binary as expected)
- `-p ariadne-report`: `47 passed / 0 failed / 0 ignored / 4 suites`
  (byte-identical to pre-change run - report untouched)
- m24 identity (rider consumer): `8 passed / 0 failed / 0 ignored / 1 suites`
  (matches AR-24 verdict census 8/0/0/1)
- m00 facts-path: `2 passed / 0 failed / 0 ignored / 1 suites`;
  m00 golden-baseline: `2 passed / 0 failed / 0 ignored / 1 suites`
  (together the AR-24-era 4/0/0/2; both files ZERO-diff in this lane)
- m18 cpp parity (error-text authority): `23 passed / 0 failed / 0 ignored /
  1 suites` - UNEDITED, green: external proof no CLI-visible string shifted
- `cargo check --workspace --all-targets`: ok exit 0. Warning census note:
  `21 warnings emitted` observed post-change vs `1` in a stash-baseline probe;
  adjudicated: all 21 are PRE-EXISTING (common/mod.rs dead-code x20
  duplicates surfaced by stash-mtime rebuild, m7 unused-variable, ui
  unused-gen, cli/src/lib.rs:804 unused_mut known-deferred row) - zero
  warnings name any file this lane touched (grep of capture: no hits for
  registry/analyze/ar25/m24).
- `python tools/hygiene_check.py`: zero R1/R2 blocking rows; every printed row
  R3-warn and pre-existing; zero rows in this lane's files.
- `python tools/encoding_guard.py --staged`: silent (after each commit);
  `--all`: see landing record below.
- Grep lock: `.map_err(anyhow::Error::msg)` now ZERO occurrences in all
  `crates/*/src` production code; the only code occurrence left anywhere is
  the deliberate pre-form witness inside `ar25_seam_error_chain_test.rs`.

Forced-scenario before/after invariant: the SAME three forced scenarios (zero
units / unclaimed language / mixed languages) render through the seam in both
states; the byte-lock `assert_eq` on `to_string()` passed in the red run
(pre-fix) against literals byte-equal to the pre-change format strings
(re-derived from the `5de3884^` blob) and passes in the green run.

## StaleArtifact fold disposition: LEFT (non-trivial), with reason

The handoff's conditional ("fold onto the shared error pattern where trivial;
where non-trivial, journal why and leave it") resolves to LEAVE:
- `report/manager.rs::StaleArtifact` already IS the shared typed-error pattern
  in this crate family: hand-rolled `Display + std::error::Error`, carried via
  `anyhow::Error::from`, consumed by `is_stale` = `downcast_ref`. It needs no
  folding onto itself.
- A "fold" could only mean rewriting its Display/Error impls onto thiserror
  (= new `report` dependency - not trivial, workspace-dep rider) or onto
  `PluginError` (wrong taxonomy - artifact staleness is not a compiler-
  validation kind, same argument as the design section).
- The fold's consumer premise is also void: workspace grep shows ZERO
  bespoke StaleArtifact handling in cli (all `is_stale` consumers live in
  report/pipeline internals, already typed). No bespoke reporting existed to
  fold; `crates/report/src/manager.rs` is therefore ZERO-diff (gate 2 run
  byte-identical to baseline confirms).

## Deviations

- D1: `tests/integration/tests/m24_language_identity_test.rs` touched (one
  `.to_string()` render token on the seam-error extraction; assert wording
  byte-untouched). Compile-forced by the signature change - the same class
  the AR-24 verdict adjudicated ACCEPTED as D2 ("E-unavoidable: the signature
  change breaks every consumer by construction; STRICT COMPILE-GREEN forces
  it"). The UNEDITED constraint's PURPOSE survives: m24's wording locks are
  byte-untouched and green at pinned census 8/0/0/1. m00/m18 files: ZERO-diff.
- D2: mechanism-witness premise corrected mid-red on own-run evidence
  (anyhow 1.0.104 `Error::msg` preserves payload type identity; the erasure
  AR-25 removes is of the dispatch KIND via String payload, not of bytes).
  Adjudicated inside this lane's own test+capture, no scope change; see RED
  section. No production-code consequence.
- D3: `parse_language_arg` deliberately NOT converted (named "AR-25 territory"
  by the AR-24 decision table): it is clap's `value_parser` surface
  (`Err: Display`), not an anyhow re-wrap; conversion would touch clap error
  rendering for zero typed-flow gain. Objective-scoped leave, per the same
  table's "cli lock stays green UNEDITED" disposition.
- D4: AR-23 deferred-row F4 (analyze.rs builds a second `default_registry()`
  while `pipeline.registry` is public, "cheap cleanup candidate for AR-25's
  cli visit") deliberately NOT folded in: D9-locked registry selection, zero
  behavior impact, and unrelated to the re-wrap drop - scope discipline; row
  remains open for closeout.

## Landing record

Range: `d4c608c..HEAD` for this lane; my commits: `2aeae5e` (journal
skeleton), `96739ad` (red: enum declaration + ar25 lock suite red at the
re-wrap layer), `2f3b5e7` (green: wiring + re-wrap drop + m24 token).
Range-wide note: `tasks/ar-21-handoff-journal.md` appears in `git diff
--name-only d4c608c..HEAD` from the PARALLEL AR-21 lane's own commit - it is
not part of any AR-25 commit (each AR-25 commit's diffstat lists only AR-25
touch-set paths). `encoding_guard.py --all`: silent (this final commit
carries that verified content; final journal rider per the AR-15/16/24
landing-record precedent).
