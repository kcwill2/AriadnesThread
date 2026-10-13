# A-2 Verdict - extraction-auto-apply: the `apply_plan_with` transactional core

Verdict: **APPROVED** (zero blockers; two minor record findings, both non-blocking).

- Handoff of record: `tasks/extraction-auto-apply-a2-handoff.md`
- Journal: `tasks/extraction-auto-apply-a2-journal.md`
- Review range: `ff93508..d7df94f` (HEAD `d7df94f`), branch `main`, tree clean at adjudication.
- Commits reviewed oldest-first: `ff93508` (duty 1, out of scope), `fdfe950` (A-2 landing, parent `ff93508`), `d7df94f` (journal rider).

## 1. Ground truth (fresh)

`python tools/checkpoint_guard.py status`: HEAD `d7df94f`, branch `main`, zero dirty files, zero untracked files. Worktree census: primary worktree clean; the other six entries are unrelated pool/temp/aux worktrees (none carrying A-2 work).

## 2. Per-commit review, oldest-first, and commit-vs-touch-set

### `ff93508` - duty 1 (out of review scope)

`git show --stat ff93508`: exactly one file, `tasks/extraction-auto-apply-a1-verdict.md`, 473 insertions, nothing else (numstat `473 0`). This is exactly the A-1 verdict already adjudicated APPROVED in that file; per the reviewer instruction it is verified as that one file and treated as out of scope.

### `fdfe950` - A-2 landing (parent `ff93508`)

`git show --stat fdfe950`: `crates/cli/src/commands/transform.rs` (88 lines changed, +74/-14) and `tasks/extraction-auto-apply-a2-journal.md` (222 insertions). 2 files, 296 insertions, 14 deletions. Message verbatim from handoff gate 8. Both paths are inside the declared touch-set. Zero paths outside it.

The full diff was read hunk by hunk. It is a faithful move-plus-thin-wrapper refactor:

- Wrapper `apply_transform_plan` (:129-144) keeps its exact signature at `:129` and performs exactly the seven handoff steps in order: locate repository (:130), prepare artifacts (:131), `load_plan_for_action` (:133), load the reduced model (:135), print `Applying {}...` (:137), call the core (:139), print `outcome.format_output()` (:141).
- `ApplyTransactionReport` (:149-152) and `apply_plan_with` (:162-223) match shape C1 verbatim, including the struct's mandated doc comment and the core's doc comment.
- The core holds every element handoff "What to build" item 2 enumerates, in order: locate repository, `ArtifactManager::prepare_for_repo`, `IsolationEngine::new(reduced_model)`, `repository_source_reader`, plan-language extraction backend, `plan_rewrite_targets`, `find_plan_consumers`, `IncludeRewriter::build_preview`, plan-language rewriting backend, `TransactionApplier::apply_with`, the plugin tier-2 header-staging decision, `commit_or_rollback` returning the report. Moved comments and citations (M23 T1/T7, T7, T8b; M18.10) moved with the code.
- `commit_or_rollback` (:372-430) and `report_rollback` (:432-450) lost their `println!` and return `Result<ApplyTransactionReport>`; the artifact-record `save_transaction_artifacts` writes stayed in their exact positions (commit: after outcome construction; rollback: rollback-then-save order preserved).
- Public surface is additive: `repository_source_reader`, `extract_plan_headers`, `build_migration_preview`, `plan_rewrite_targets`, `plan_consumer_discovery_targets`, `plan_target_consumers`, `load_plan_for_action` signatures unchanged (call sites only gained `&plan` to `plan` adjustments).
- The single added test module (:452-475) is the mandated one test, using the real `progress_contract_tests.rs:73` idiom (`format!` needle over `include_str!` of its own file). No second `TransactionApplier::apply_with` call was authored.

### `d7df94f` - journal rider

`git show --stat d7df94f`: journal only (+55/-4). The diff was read: it adds the landing record, the acceptance table and the out-of-scope-observation section; it does not retroactively alter any earlier journal claim. In touch-set.

### Range touch-set verdict

`git diff --name-only "ff93508..d7df94f"` = exactly two paths, both in the touch-set: `crates/cli/src/commands/transform.rs`, `tasks/extraction-auto-apply-a2-journal.md`. `git diff ... --name-only -- crates/cli` = `crates/cli/src/commands/transform.rs` alone. No over-broad commit, no undeclared path, no Cargo.lock churn.

## 3. Freeze-set and untouched-crate proofs (fresh)

All of the following range diffs are EMPTY (no output):

- `git diff "ff93508..d7df94f" -- crates/validate crates/ui crates/transform crates/artifact tests tools Cargo.toml Cargo.lock`
- `git diff "ff93508..d7df94f" --name-only -- tests/integration/tests/m10_validation_test.rs tests/integration/tests/m20_apply_migrate_characterization_test.rs tests/integration/tests/m23_symbol_isolation_test.rs`
- `git diff --numstat "ff93508..d7df94f" -- crates/cli/tests` (bench producer zero-diff proof)

So `crates/validate` is untouched, the three frozen apply suites are in zero hunks, `tests/golden/**`, `tests/fixtures/**` and assets are untouched, and the bench producer set is byte-identical across the range.

## 4. Red evidence adjudication (necessarily failed pre-change)

I read the base blob `ff93508:crates/cli/src/commands/transform.rs` in full (via the git show tool, no PowerShell pipe). Findings:

- The base file contains neither `pub fn apply_plan_with` nor `pub struct ApplyTransactionReport` anywhere (the whole file was read; it ends at `report_rollback`), and it contains no test module at all. Both needles the mandated test builds with `format!` are therefore absent at the base.
- The mandated test reads its own file with `include_str!` at compile time, so on the Red tree (base plus the test module) the first `assert!` necessarily panics with `C1: transform.rs must define the shared transactional core apply_plan_with`. That is a semantic Red: the module compiles against the base tree (it references no not-yet-existing symbol), matching the handoff's "a compile failure is NOT the Red".
- No other test changed state, established three ways: (a) the Red line `124 passed / 1 failed / 0 ignored / 1 suites` is exactly the lib binary shape, because cargo aborted after the lib binary failed; (b) I re-measured the lib binary at HEAD: `125 passed / 0 failed / 0 ignored / 1 suites` = the 124 baseline passes plus the one new test, so at the Red tree every previously passing lib test still passed and exactly one test failed; (c) the crate line moved 130 -> 131 with suites 7 and ignored 8 held, and the full workspace census moved 1486 -> 1487 (exactly +1).
- O1 is an honest observation, not a concealed deviation: with both needles absent, `assert!` short-circuits and only the first panic is observable in one run; the second message is verbatim in the source at `transform.rs:472` and stays armed forever after. The handoff's Red wording describes each assertion's failure text; the single-run mask is inherent to the mandated two-assert single test.

## 5. Success-path stdout byte-identity adjudication

Base wrapper (`ff93508`, read in full): prints `Applying {}...\n` after `build_preview`, then `commit_or_rollback` prints `outcome.format_output()` (Committed; or RolledBack via `report_rollback`). Head wrapper: prints `Applying {}...\n` at :137 then `report.outcome.format_output()` at :141. Same two prints, same order, same text, and the outcome construction is byte-identical. Error-path: `Plan {} not found` (transform.rs:242, inside `load_plan_for_action`) and the locate failure (`Repository::locate`, :130) both fire before the first print (:137), so they still print nothing first; m20 locks both error texts and is green unedited. The contract-sanctioned delta (transaction errors now surface after the `Applying` line) is exactly what handoff "What to build" item 3 and shape C1 describe, and the frozen suites assert error content plus success-path artifact state, not error ordering.

Core prints nothing: the `println!` census of `transform.rs` at HEAD shows 12 hits; ten are in the untouched preview families (`extract_plan_headers` :56-74, `build_migration_preview` :100-114) and the only two on the apply path are the wrapper's :137 and :141. The core family (`apply_plan_with`, `commit_or_rollback`, `report_rollback`, `load_plan_for_action`, `repository_source_reader`, `plan_rewrite_targets`, `find_plan_consumers`) holds zero prints.

Coverage shape noted, not raised as a finding: no frozen suite captures stdout (m10 asserts the `Committed` manifest artifact at :83-85 plus the `format_output` texts at :180/:197), so success-path stdout byte-identity is inspection-verified per the handoff's own model; I reproduced that inspection independently.

## 6. Caller census and single-write-path checks (fresh)

- `apply_plan_with(` in `crates/cli/src/commands/transform.rs`: 2 lines - the definition (:162) and the wrapper's single call (:139). A working-tree census over all of `crates/**` and `tests/**` finds `apply_plan_with` nowhere else: zero hits in `tests/`, zero in every other crate.
- `ApplyTransactionReport` exists only in `transform.rs`.
- `TransactionApplier::apply_with` is called exactly once in the file (:195); `crates/validate/**` untouched.
- Follow-up observation (my own, out of the hand's scope, relayed not dropped): a SECOND, pre-existing `TransactionApplier::apply_with` call lives at `crates/cli/src/optimize.rs:406` (the `optimize` verb's own transaction), last touched by earlier commits and byte-identical across this range. The hand did not author it, so the handoff's "no second call is authored anywhere" holds for A-2 and the A1 census of record (`apply_plan_with(` = 2) is unaffected; shape A1 binds UI-triggered mutations to the core, which this extraction makes possible. Recommend the milestone ledger record whether `optimize` later migrates onto the shared core.

## 7. Encoding, hygiene, ASCII discipline (fresh)

- `python tools/encoding_guard.py --all`: exit 0.
- `python tools/hygiene_check.py`: exit 0. R1/R2 zero. The R3 warn-only population is entirely pre-existing rows in other files; `transform.rs` contributes zero rows. File is 475 lines (415 -> 475, inside the handoff's "~470, well under the ceiling" band; under 1000).
- Function lengths measured at HEAD: wrapper 16 (:129-144), core 62 (:162-223), `commit_or_rollback` 59 (:372-430), `report_rollback` 19 (:432-450) - all under 100.
- Pure-ASCII authorship: every non-ASCII byte in the file at HEAD (box-drawing glyphs at :72/:74; `-`/`=>`/`<=>` glyphs in pre-existing doc comments at :13/:47/:274/:299/:302 and in the moved M18.10/M23 T7 comments at :386/:392) is either pre-existing or moved verbatim; all newly authored content (wrapper prints, struct, core, returns, test module) is ASCII. `encoding_guard --all` exit 0 confirms the moved non-ASCII is legitimate UTF-8, not mojibake.

## 8. Deviation dispositions

- **D1 (gate-7 staging route via distill `git add`) - ACCEPTED.** With an empty index, `encoding_guard --staged` would scan nothing and the gate would be vacuous. The implementer staged exactly the two touch-set paths by explicit path (never `.`, no glob), ran the guard so it inspects the blobs actually landing, then ran gate 8 verbatim. The landed commit contains exactly those two paths (verified above), and gate 9's `encoding_guard --all` (re-run fresh here: exit 0) provides the independent full-tree check. Net effect is strictly stronger evidence than the literal gate. Precedent: playbook T-C D2, ledger-recorded.
- **D2 (journal rider commit) - ACCEPTED.** The landing record cannot exist inside its own commit; the rider is journal-only (+55/-4), in touch-set, adds no source change, and follows the W-1 `82e5ffe` / A-1 `7d819bb` precedents. Verified: `d7df94f` touches only the journal.
- **O1 (Red second-assert mask) - ACCEPTED as an honest observation.** Inherent to the mandated single-test two-assert shape; disclosed in the journal's Red section; the second panic text is verbatim in the source and remains armed as the anti-regression lock. No concealment.
- **O2 (refused combined command) - no impact; see Finding 1.** The refusal class is environment-verified: I attempted chained/pipe/`python -c` forms myself this hop and each returned the same catch-all denial, which the ledger records as a known reviewer/implementer configuration gap (dated row, M26 V3 recurrence). A refused command has no tree impact; the disclosures were made in the final report.
- **Phantom mid-session narration note - honest per the tree evidence; see Finding 1.** The tree shows exactly two commits with exactly the two touch-set paths, exact per-commit stats, and every acceptance item reproducing fresh; there is no tree trace of any extra or hidden change, so the "changed nothing" claim is corroborated. The note lives only in the final report, not the journal.

## 9. Findings (numbered; zero blockers)

1. **MINOR (record completeness, docs-only).** `tasks/extraction-auto-apply-a2-journal.md:195-216` (the Deviations section) declares D1, D2 and O1 but omits two disclosures the implementer's final report made: O2 (refused combined command) and the phantom mid-session narration transparency note. Zero tree impact: the refusal class is environment-verified (ledger dated row; reproduced this hop) and the "changed nothing" claim is corroborated by the two-commit, two-path tree with all gates reproducing fresh. Nothing is hidden from the reviewer, but the journal of record should match the report. Remediation: the next hop's docs-only rider (tasks/** scope, no source) appends both disclosures to the A-2 journal's Deviations section.
2. **MINOR (journal erratum, docs-only).** `tasks/extraction-auto-apply-a2-journal.md:133-135` ("Refactor") records `commit_or_rollback` 60 lines and `report_rollback` 17 lines; measured at HEAD they are 59 (:372-430) and 19 (:432-450). `apply_plan_with` 62 and the wrapper 16 are correct. All four are far under the 100-line ceiling, so the hygiene verdict is unaffected; this is the "claim stands, number wrong" erratum class. Remediation: correct the two counts at the next journal touch (docs-only rider; this reviewer must not edit the audited journal).

## 10. Workspace-wide acceptance pass (fresh this hop, verbatim)

- Full census: `python tools/testrun.py --distill` -> `### cargo test PASS (exit 0)`, `1487 passed / 0 failed / 13 ignored / 112 suites`. Reconciles to the post-A-1 anchor `1486 passed / 0 failed / 13 ignored / 112 suites` (A-1 verdict / ledger) at exactly +1 from A-2, suites 112 held, zero failures.
- Handoff gate 4 reproduced: `python tools/testrun.py --distill -p ariadne-cli` -> `131 passed / 0 failed / 8 ignored / 7 suites` (baseline 130 + the one mandated test).
- Handoff gates 2 and 5 reproduced: `python tools/testrun.py --distill -p ariadne-integration-tests --test m10_validation_test --test m20_apply_migrate_characterization_test --test m23_symbol_isolation_test` -> `18 passed / 0 failed / 0 ignored / 3 suites`, byte-identical to the journal's gate 2 line.
- Red-shape corroboration: `python tools/testrun.py --distill -p ariadne-cli --lib` -> `125 passed / 0 failed / 0 ignored / 1 suites` (124 baseline + 1 new; matches the Red line's 124 passed + 1 failed arithmetic).
- Tool suites: `python -m unittest discover -s tools -p "*_tests.py"` -> `Ran 219 tests in 23.377s` / `OK`.
- UI smoke (routed via the distilling entry point per instruction; direct invocation is in the ledger's refusal class): `python tools/distill.py -- python tools/ui_smoke.py` -> `### generic PASS (exit 0)`, `boot+route checks clean (cold fixture, attempt 1, 1.0s)`, `ui_smoke: PASS`.
- `python tools/theme_token_check.py --check-added --base ff93508` -> silent, exit 0 (zero added hex; no asset touched).
- `python tools/hygiene_check.py` -> exit 0 (R3 warn-only, all pre-existing, zero rows in `transform.rs`).
- `python tools/encoding_guard.py --all` -> exit 0.

### Bench stance (exemption with proof; no timing verdict attributable to A-2)

- `git diff --numstat "ff93508..d7df94f" -- crates/cli/tests` is EMPTY (producer zero-diff proof, independently reproduced).
- The timing gate is deterministically blocked and pre-existing: `python tools/distill.py -- python tools/bench_compare.py` at this tree -> `### generic FAIL (exit 2)`, capture `ERROR parse format drift: documented phase row(s) missing from bench output (never substituting zeros):` / `missing documented phase: generate_report+detect_cycles (combined)`. Cause confirmed at line level: `tools/bench_compare.py:84` pins the stale tuple label while the producer `crates/cli/tests/payload_tail_bench.rs:389` prints `generate_report+detect_cycle_groups (combined)`; the exit-2 path (`tools/bench_compare.py:724-727`) fires before any median comparison. `tools/**` is forbidden to this reviewer; the ledger dated row owns the fix.
- The compensating `--bench stage5-memory --runs 6` was NOT run: machine idleness could not be verified (both idle probes were refused with the catch-all denial), and the ledger's stance designates that compensating check at A-4. Per-hand zero-diff proof plus the deterministic blocker mean NO timing number is attributable to A-2.

### Gates not run / refused this hop (configuration findings, recorded not substituted)

- `python tools/testrun.py --baseline` (bare): usage error `usage error: missing value for --baseline` - the documented standing errata (no baseline file exists); the `--distill` census substitute was used instead, as the A-1 verdict and ledger record.
- Refused with the verbatim catch-all `{"permission":"bash","pattern":"*","action":"deny","source":"global"}` (boilerplate; each was worked around and each workaround is named): chained `git status --porcelain; ...` (worked around with plain `git status` plus `checkpoint_guard.py status`); rev-qualified `git grep -c "apply_plan_with(" <rev> -- ...` (worked around with the grep tool over the clean worktree, which equals HEAD); `python -c ...` byte scans (worked around with the dedicated git show tool on blobs); `git show ... | Select-String` pipes (same workaround); `git merge-base --is-ancestor` (worked around with `git log -- path` ancestry); `Get-Process`/`Get-Counter` idle probes (bench compensating check therefore not run); the piped unittest form (run plainly); `Write` to the agent-external temp dir (out of the tasks/** edit grant, expected).
- `python tools/encoding_guard.py --staged` was not re-run at the committed tree (nothing is staged at a clean HEAD); the `--all` superset covers it and exits 0.

## 11. Acceptance adjudication (handoff items 1-7)

1. Commit/touch-set = transform.rs + journal, exactly - MET (Section 2).
2. `apply_transform_plan` signature byte-identical at `:129` on both sides - MET.
3. `apply_plan_with(` census = 2 in `transform.rs`, zero elsewhere in `crates/**`/`tests/**` - MET (Section 6).
4. Gate 5 line identical byte-for-byte to gate 2 - MET (Section 10).
5. `hygiene_check.py` exit 0 - MET.
6. `encoding_guard.py --all` exit 0 - MET.
7. m10/m20/m23 in no hunk of the range; `crates/validate`, `crates/ui`, `crates/transform`, `crates/artifact`, `tests/**`, `tools/**`, assets, manifests zero-diff - MET (Section 3).

Shape C1 plan-literal compliance: struct and function signatures verbatim, wrapper exactly the seven-step shape, core holds every enumerated element in order, comments moved with code, public surface additive, names per shape section 9 - all MET.

Verdict: **APPROVED**. The two minor findings are docs-only record corrections carried by the next hop's rider; neither touches source and neither blocks A-3 dispatch. Follow-ups for the ledger: the `optimize.rs:406` pre-existing second `TransactionApplier::apply_with` call (Section 6), the bench tuple fix and PR-2 baseline refresh (unchanged operator tails), and the reviewer-allowlist config row (chained/pipe command denials) already dated in the ledger.
