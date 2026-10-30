# rust-language R-9 journal - implementer

Date: 2026-10-26.
Handoff: `tasks/rust-language-r9-handoff.md`.
Dispatch base: `4bd22c8` (branch `balsam-basil`).
Posture at return: STOP at a clean boundary (shape clause RISK-MODTREE + dispatch context-wall law: journal inventory, return; planner amendment, never silent narrowing).

## Section 1 - Duty 1 landed (complete)

- Commit `c52817c` `review(rust-language): land R-8 verdict APPROVED + ledger relay clause (R-9 duty 1)` - stat: `tasks/milestone_ledger.md` 1 line changed, `tasks/rust-language-r8-verdict.md` 300 lines added.
- The R-8 relay clause landed VERBATIM into the live row (line 62), appended after the R-7 relay's `... R-8 dispatched.` and before the state-cell terminator, count-of-1 verified against the landed commit blob byte-for-byte (probe on `git show "c52817c:tasks/milestone_ledger.md"`, capture under `target/distill-capture.txt`).
- Relay text landed: `R-8 LANDED 5f87944 APPROVED 2026-10-26, census anchor 1970 passed / 0 failed / 13 ignored / 125 suites @ 5f87944, R-9 dispatched (hooks + validator + flip hop).`
- Precedent followed: `53afb19` (R-8 duty 1) - same message shape, same single-line relay append, no row rebalance.
- Chain check (phantom-narration countermeasure): `git log --oneline` shows `c52817c` on top of `4bd22c8` on top of `5f87944` on top of `53afb19`.

## Section 2 - STOP adjudication: blocking findings from binding-input reconnaissance

The recon phase (before red) established that the handoff's mandated suite assertion cannot pass on the current tree, and that the fix lives outside the R-9 touch-set.
Detail follows; each finding is machine-checkable from the cited files.

### F1 (hard blocker) - "consumer `use` paths rewritten per spec" is unsatisfiable through the wired route

- Symbol-id flatness: `crates/lang-rust/src/parser_items.rs` builds symbol ids from the in-file module chain only; an out-of-line `mod x;` records the bare child name in its own file, so the records for `src/models/user.rs` carry ids `crate::User` and `crate::default_user` - no `models::user` segment exists in any Rust id.
- Landed-machine witness: the R-6/R-7 integration suites (both APPROVED on this tree) assert exactly those flat ids through the wired route (`crate::User`, `crate::default_user` in `tests/integration/tests/rust_language_dependency_graph_test.rs` and the load suite).
- Rewriter requirement: `consumer_rewrite_spec` in `crates/lang-rust/src/dependency_rewriter.rs` derives the consumer's old module path from the consumer file path (`module_path_of_src_file(old_includes)`) and retargets a use-tree only when its id equals `crate::<module segments from path>::<moved leaf>` or matches it as a glob prefix (`retarget_id`).
- Consequence: a moved FLAT id (`crate::User`) never carries the path-derived module segment, so the prefix match can never hold on any wired Rust plan; no consumer `use` rewrite can ever fire.
- The handoff's apply-stage bullet is the m23 mirror of `tests/integration/tests/m23_symbol_isolation_test.rs`, whose apply asserts a real rewritten consumer import line.
  Reading the Rust bullet weaker than the m23 assertion it mirrors would be silent narrowing, which the handoff forbids.
- No lever inside the R-9 touch-set (`planning.rs` new, `validator.rs`, `plugin.rs`, the new suite) changes id qualification or the rewrite spec; candidate fixes all sit outside:
  - (a) analyzer-level id re-qualification from the module tree (`analyzer.rs` / `parser_items.rs` - R-5/R-6 surface, outside touch-set; would re-open every landed id witness and the resolver edge grammar).
  - (b) suffix-based consumer retarget in `dependency_rewriter.rs` (outside touch-set; R-8 mechanism amendment).
  - (c) planner amendment of the handoff's apply-stage assertion (planner owns; this is the shape's named route).

### F2 - every Rust plan includes Module-kind proposals that clobber frozen children at apply

- The planner batches per-file symbol records (`crates/cluster/src/planner.rs`), and out-of-line `mod x;` declarations are recorded as Module-kind symbols (`parser_items.rs`), so each aggregation candidate yields proposals for its declared child modules.
- `target_path_for(candidate.path, anchor.name)` yields `{sanitized}.rs` beside the candidate; the T7b filter drops only `target == candidate.path`, so child-module proposals survive into the plan.
- The extractor's `locate_symbol_bodies` matches `Item::Mod` idents, so a Module proposal's "body" is the bare `mod x;` line.
- Consequence on a vanilla-copy fixture: a plan on `src/lib.rs` proposes `src/greeting.rs` (apply would overwrite the frozen child) and a plan on `src/models/mod.rs` proposes `src/models/user.rs` (same clobber class); a synthetic candidate with `mod ledger;` applies a self-referential stub.
- Needs planner/cluster adjudication (filter Module-kind proposals from applyable units, or a naming grammar that cannot collide).
  `crates/cluster` and the planner are outside the touch-set.

### F3 - touch-set contradiction on the move-retention hook

- The R-8 verdict relay carried forward says "R-9 owns: ... the retention hook (root-file-to-sibling `mod` insert)".
- That hook is `move_out_top_level_symbols`, whose override must live in `crates/lang-rust/src/dependency_rewriter.rs` - a file the R-9 handoff touch-set explicitly forbids.
- The trait default delegates to `remove_top_level_symbols` (None on the Rust rewriter), so the apply engine proceeds with the copy fallback and no move.
- Resolution belongs to the orchestrator: grant a `dependency_rewriter.rs` rider, or move the hook owner to R-10 / an explicit rider hop.

### F4 - superseded caps locks will need the standard rider band at re-dispatch

- The all-true flip supersedes four integration caps pins outside the touch-set (the partial-caps locks in the R-4/R-5/R-6/R-7 integration suites: dependency graph, discovery, semantic index, load analysis).
- Precedent (R-5/R-6/R-7): superseded flip locks are amended by an orchestrator rider commit, not by the implementer crossing the touch-set.
  Re-dispatch should ship with that rider or add the four files to the touch-set.

### F5 - the parked id-sharing adjudication was not re-scoped into R-9

- The R-5 journal (D5) parked the `crate::greeting` mod-record/fn-record id-sharing class to "R-9's named symbol-isolation suite", but the R-9 handoff's suite spec never names it; the planned suite would walk into it (fixture `src/greeting.rs` declares `fn greeting` with id `crate::greeting`, the same id as the `mod greeting;` record).
- Adjudication needed: in-scope for the amendment suite, or formally re-parked.

### F6 - tier-3 `Skipped` needs the scripted runner on this machine (design note, not a blocker)

- `crates/validate/src/lib.rs` maps only `ToolchainStatus::Missing` to `Skipped`; cargo/rustc ARE on PATH here, so a wired apply reports tier-3 `Pass`, not `Skipped`.
- The suite's Skipped claim is satisfiable through `RustToolchainValidator::with_runner(scripted all-None)` consumed by `TierValidator::validate_affected_tus` - the consumption mapping is planned; noted for re-dispatch.

## Section 3 - what proceeds cleanly within the touch-set (for the amendment)

- `planning.rs`: `aggregation_filter` (parsable AND declares >=1 non-inline `mod`) + `target_path_for` (`{sanitized_symbol}.rs`, sanitize mirroring `scala_target_path_for`), registry-dispatch wiring via `plugin.rs` overrides - satisfiable, designed.
- `validator.rs`: probe-only `RustToolchainValidator`, `TOOLCHAIN_PROBE_ORDER = ["cargo", "rustc"]`, `CommandRunner` injection seam mirroring `crates/lang-scala/src/validator.rs`, `validate_compile` single-input honesty doc, in-src probe tests - satisfiable, designed.
- `plugin.rs`: final all-true caps byte-lock literal (mirroring Scala D-M23-6) + in-src pin rename; `CURRENT_ANALYSIS_VERSION` stays `2.2.0`; `requires_toolchain: false`; cycle policies unchanged - satisfiable.
- Suite stages that pass as written: plan-stage (candidates + hook dispatch), preview-stage (parent gains the `mod` declaration once, consumer discovery targets listed), apply parent-insert idempotence, rollback via `TransactionApplier::apply_with` + `tree_digest` (m23 idiom), tier-3 `Skipped` via the scripted runner.
- Suite stage that fails as written: consumer `use` rewrite on apply (F1).

## Section 4 - quote-deferral disposition

- The R-8 dated deferral (`quote` direct dep + `syn::__private` import switch) needs `Cargo.toml` scope; the R-9 handoff forbids manifests (no new deps).
- Disposition per the dispatch rule: relayed to R-10 (docs hop) - applied ONLY if R-10's handoff grants a manifest-capable scope.
- The dated deferral row is already on record via orchestrator rider `4bd22c8` in `tasks/rust-language-execution-results.md`; no duplicate ledger deferred-findings row was added (the ledger band is orchestrator-owned; the R-8 verdict suggested it, the rider superseded, I did not double-record).

## Section 5 - deliberately NOT done (and why)

- No hook, validator, caps, or suite edits landed at all: the dispatch context-wall law names half-landing as the failure mode to avoid, and every landable piece without the suite would leave the hop's acceptance gate (the full suite green) unreachable.
- No silent narrowing of the F1 assertion (the handoff names planner amendment as the route for exactly this case).
- Gates 2-9 not run; even gate 1 (baseline census `-p ariadne-lang-rust`) not run - the blocker was proven from code plus landed APPROVED witnesses before red, and a census line cannot change the adjudication (recorded as a deviation with reason in Section 6).
- No git stash at any point (dispatch standing note; nothing used, nothing needed).

## Section 6 - deviations with reasons

- Deviation 1: returned before running gate 1 (baseline census).
  Reason: the STOP was adjudicated from binding inputs and landed-machine witnesses; spending the census gate could not alter the outcome and the shape clause prescribes the early clean-boundary return.
- Deviation 2: ledger live row is now 3776 bytes, above the 2000-char source-line guideline.
  Reason: the guideline governs source files; the ledger precedent chain (R-7 duty 1 landed the row at 3615 bytes unflagged) and the verbatim-landing law govern here; `python tools/hygiene_check.py` exit 0 was run over this tree state (digest in Section 8).

## Section 7 - disk-state inventory at return

- Working tree: clean (`git status --porcelain` empty) at `c52817c`.
- Landed this hop: `c52817c` only (duty 1 trio content: verdict doc + 1 ledger line).
- Source tree: zero bytes touched in `crates/`, `tools/`, `tests/`, manifests, fixtures, goldens.
- Scratch: one probe script under the machine temp dir (outside the repo), throwaway.

## Section 8 - gates run (verbatim digests)

- `python tools/distill.py -- git log --oneline -4` -> PASS: chain `c52817c` / `4bd22c8` / `5f87944` / `53afb19`.
- `python tools/distill.py -- git status --porcelain` -> PASS (0 lines, clean).
- `python tools/distill.py -- python <temp probe>` (relay byte-for-byte probe on landed blob) -> PASS: `COMMIT_BLOB_RELAY_COUNT= 1`.
- `python tools/encoding_guard.py --all` -> exit 0 (silent).
- `python tools/hygiene_check.py` -> exit 0; output R3 warn-only lines, all preexisting (zero R1/R2 lines; no lang-rust or this hop's files in the warn set).
- Exit-code probe (`HYGIENE_EXIT= 0 ENCODING_EXIT= 0`) run after the journal was authored, so both digests cover the journal file too.
- Commit trio per landing: `commit_scoped.py` ran the scoped commit above (staging owned by the tool).

## Section 9 - resume path for the re-dispatched hop

Fresh-session bootstrap after this return:

```
python tools/checkpoint_guard.py resume --spec tasks/rust-language-r9-handoff.md --scope crates/lang-rust --cmd "python tools/testrun.py --distill -p ariadne-lang-rust"
```

- The re-dispatch must carry the F1 amendment decision (analyzer re-qualification vs rewriter suffix-retarget vs assertion re-wording), the F2 adjudication, the F3 hook-ownership ruling, and the F4 rider band.
- Duty 1 is ALREADY landed (`c52817c`) - the re-dispatched implementer must not re-land it; its duty 1 is this R-9 verdict when it exists.

## Section 10 - coordination

- Board ASK posted to the orchestrator with the F1-F5 condensed evidence and the candidate amendment paths.
