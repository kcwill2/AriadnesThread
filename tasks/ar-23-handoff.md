# AR-23 Handoff - Analyze incremental path consumes the pipeline (A6 part 2)

Wave 6. Crate (`cli`) + integration lock. Depends: AR-22 landed (ctor
`Pipeline::for_incremental(repo_path, Option<SourceLanguage>)` +
`IncrementalView` — read `crates/pipeline/src/for_incremental.rs`) AND AR-19
APPROVED (same-crate cli serialization — you own `crates/cli` after it).

## Objective

`crates/cli/src/commands/analyze.rs` incremental path calls
`Pipeline::for_incremental`; DELETE the mirrored policy it hand-rolls today
(locate/discover/shaper/restrict sequence at ~:32-56 base — re-anchor, AR-19
may have shifted line numbers) and remove the self-admitted mirror comment
(~:110-114). Observable behavior locked identical BEFORE deletion.

## Impact

Unit-restriction and artifact-authority policy become ONE implementation;
the drift class the audit flagged (fix one path, rot the other) is closed.

## Touch-set (hard)

- `crates/cli/src/commands/analyze.rs`
- `crates/cli/tests/**` or the existing cli test module(s) covering analyze
  (journal which)
- NEW parity integration lock (or extend m-series): analyze-path behavior
  pre/post identical — capture the pre-change observable (run incremental
  analyze on the basic fixture: stage reuse lines, artifact set written,
  exit text) and assert it holds post-change
- `tasks/ar-23-handoff-journal.md`

## Constraints

- PARITY-FIRST: record pre-change observables in the journal (from a real run
  on the fixture) BEFORE touching code; the new lock encodes them.
- STRICT COMPILE-GREEN discipline (shared worktree).
- Pipeline crate UNEDITED (consumption only). If the ctor lacks a knob the
  mirror had (e.g. incremental_backend pairing AR-22 journaled), STOP and
  report rather than editing pipeline — route the gap as a finding.
- Pure ASCII; commit_scoped cli + integration + journal.

## Gate commands

1. `-p ariadne-cli` (baseline 118/0/8/6 + yours, reconciled)
2. `-p ariadne-pipeline` UNEDITED green (40/0/4/9)
3. m00 golden + facts-path parity UNEDITED
4. new analyze-parity lock green; incremental re-run stability (run twice,
   second run's reuse lines prove warm path unchanged)
5. `encoding_guard --all`; `hygiene_check.py` (only 2 server.rs rows)

## Invariants / locks

- Mirror comment GONE; hand-rolled sequence GONE; behavior byte-identical
  (journal's captured observables).
- m00 parity is the authority for facts-path stability.

## Carried from AR-22 verdict (reviewer dispositions for YOU)

- Finding 2: the ctor eagerly `create_dir_all`s `artifacts/` while the old
  mirror created it only at persist time. Harmless + parity-locked, but
  AR-23's pre-change observable capture must NOT treat a created-early empty
  dir as a behavior delta — journal how you distinguished it.
- Finding 3: the StageFingerprints parity lock is shared-formula shape only
  (analyze.rs computes no fps on that path) — do not claim fps-value parity
  in your journal; reuse the ctor as-is.
