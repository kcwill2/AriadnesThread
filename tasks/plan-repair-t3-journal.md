# Plan Repair T3 -- Checkpoint Journal

Task: D.4, add `docs/adr/0002-symbol-placement-and-isolation.md`.
Base: branch `multi-language-support`, ADR-0001 present (committed `f6a995f`).

## Checkpoint 1 -- contract read, facts verified, ADR-0002 written

Read the handoff in full, then verified every citable fact by inspection before writing rather
than after. `SymbolPlacement` has exactly two variants, read from
`crates/lang-api/src/modeling.rs`: `SplitDeclaration` and `SingleFile`. Reduced-model
provenance, the `IsolationEngine`, the `Missing -> Skipped` tier-3 mapping, the bincode epoch
coupling, and the M23 Gate-G3 record all resolve. No source file touched.

Two handoff-mandated claims were found inaccurate against the tree and were narrowed rather than
asserted. (1) "No extension test or `.h` suffix check appears anywhere in the core crates" is
false: `crates/graph/src/builder.rs` holds lowercase `"cpp"` extension checks inside its
`#[cfg(test)]` seam module, and `crates/repo` maps `FileKind` to `SourceLanguage` in the spine.
The ADR states three narrower verified claims instead. (2) The handoff implies `SymbolPlacement`
drives the isolation unit; no production code branches on it. That is the recorded M23 ruling
D-M23-7 (iv), so the ADR presents the field as the declared contract and the plugin hook pair as
the mechanism.

Also recorded, per instruction: the reduced symbol blob is bincode, so `#[serde(default)]` cannot
rescue a stale blob and the cache-epoch bump is the only thing that makes the field addition safe;
and the M23 record supersedes the spec's "C++ tier 3 = real compile" row under amendment
D-M23-12, so tier 3 stays `Skipped` for both languages and no compiler is claimed anywhere.

## Checkpoint 2 -- gates discharged by inspection (no shell in this session)

Shell is denied in implementer sessions, so all six gates were discharged by reading the tree with
the file tools rather than executing commands, and the commit block was skipped as instructed.
Files left uncommitted for the orchestrator. Gate-by-gate method and per-claim verification
evidence are in the final report.
