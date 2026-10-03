# Plan Repair T4 journal

## Checkpoint 1 -- Green (page written)

`docs/adding-a-language.md` created: 227 lines, required section order, all trait
names / method names / signatures read out of `crates/lang/src/registry.rs`,
`crates/lang/src/unit.rs`, and `crates/lang/src/lib.rs` before being written down.

Discrepancy against the handoff's Step 2 list, reported not papered over: the
handoff named 5 `LanguagePlugin` methods (`id`, `modeling`, `capabilities`,
`analysis_fingerprint`, `shaper`). The real trait has 12 -- it also requires
`detector`, `analyzer`, `resolver`, `constraints`, `extractor`, `rewriter`, and
`validator`. The page documents all 12.

Also corrected in the page: `SymbolPlacement` is a declared, tested record with no
production dispatch; the isolation mechanism is the `aggregation_filter` +
`target_path_for` pair injected into `OptimizationPlanner`.

Gates discharged by inspection (shell denied in implementer sessions):
touch-set (only this journal + the page were written), ASCII (regex scan, no
non-ASCII match), reference resolution (every cited path Glob-verified, every
trait/method Grep-verified), length (227, gate 150-230), hygiene (discharged by
reasoning: `R2_SCANNED_SUFFIXES` excludes Markdown, so a docs-only change cannot
move the hygiene exit code).

Not done, by instruction: `git add` / `git commit`. The orchestrator commits.