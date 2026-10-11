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

(Pending: filled at the red commit.)

## GREEN + gates

(Pending: filled at the green commit.)

## StaleArtifact fold disposition

(Pending.)

## Deviations

(Pending.)
