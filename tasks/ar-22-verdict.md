# AR-22 Verdict - `Pipeline::for_incremental(...)` (A6 part 1)

**APPROVED**

Range: `d937ac0..15ffbfd` on `shining-enthusiasm`. Commits reviewed per-commit,
oldest first: `bcc08fc` (feat: constructor + module + parity test + journal),
`15ffbfd` (docs: journal RED-phase wording fix - pure documentation).

## Gate results (fresh, own stdout - see ar-22-execution-results.md)

- `-p ariadne-pipeline`: `40 passed / 0 failed / 4 ignored / 9 suites` (matches
  the expected 40/0/4/9; prior 35/4/8 suites + the new 5-test parity suite).
- m00 locks UNEDITED: `2 passed / 0 failed / 0 ignored / 1 suites` each for
  `m00_golden_baseline_test` and `m00_facts_path_parity_test`. No build
  contention observed; no rerun needed.
- `python tools/encoding_guard.py --all`: silent, exit 0.
- `python tools/hygiene_check.py`: exit 1 solely from the two pre-existing
  `crates/ui/src/server.rs` blocking rows (R2 1172>1000, R1
  `api_graph_traversal` 161 lines). Zero new blocking rows; zero rows from
  `crates/pipeline`. All other rows are warn-only R3, all foreign/pre-existing.

## Checks 1-6 adjudication

1. **Touch-set EXACT.** `git diff --name-only d937ac0..15ffbfd` returns exactly
   the 4 handoff-declared paths (`for_incremental.rs` is the handoff-permitted
   lib.rs module-file rider). The lib.rs +13 diff is read in full: `mod
   for_incremental;` + `pub use for_incremental::IncrementalView;` (with
   3 comment lines + blank), the `pub incremental_view: Option<IncrementalView>`
   field (5 doc lines), and `incremental_view: None` in `build_with_progress`'s
   literal. Nothing else.
2. **PARITY AUTHENTICITY - the A6 core.** The analyze.rs blob is identical at
   base and HEAD (`git rev-parse` of `crates/cli/src/commands/analyze.rs` at
   `d937ac0` and `15ffbfd` both yield `19818fdca1521892505e960eee9f999337ff0719`),
   so the work-tree copy IS the commit-time mirror. Diffed `mirror_construct`
   (test lines 131-167) line-for-line against
   `run_incremental_analysis_filtered` (analyze.rs:27-46): same
   `build_default_registry` enumeration (cli's `default_registry` delegates to
   it - `crates/cli/src/registry.rs:16-18`; D9 identical-enumeration test at
   `:219`), same guard-before-discovery order, `Repository::locate`,
   `repo.discover()`, the identical
   `BuildDiscoverer::new().with_repo_discovery().with_unit_shaper(registry.unit_shaper_for_repo(&repo.root)).discover()`
   chain, pre-restriction `StageFingerprints::from_inputs(root, discovery,
   contexts, tus)`, the pre-restriction discovered count, and the same
   restriction. The restriction mirror and the ctor's private copy are
   verbatim-equal to analyze.rs:82-104 (retain predicate AND the
   `unhandled language for {} (no registered plugin claims it)` bail). The
   guard message is verbatim-equal to `crates/cli/src/registry.rs:29`. The
   authority rule (`language_filter.is_none() && retained >= discovered`) is
   verbatim-equal to analyze.rs:115-121. `compute_inputs` (lib.rs:342-373,
   progress=None) is the locate/discover/shaper/fingerprint sequence itself -
   the ctor does not replicate it, it calls the shared helper every other
   constructor uses. The lock is authentic, not a fiction.
3. **Honest-unobservables ACCEPTED.** `IncrementalView`
   (for_incremental.rs:40-58) carries `language_filter: Option<SourceLanguage>`
   (the same closed label `Pipeline::language_filter` holds), `discovered_units`,
   `retained_units`, and `artifacts_are_authoritative()` reproducing the
   mirror's rule. The test locks `view.language_filter == state.language_filter`
   (test:199-201) plus `view.retained_units == pipeline.build.translation_units.len()`
   (test:216-220), so the transitive filter lock genuinely carries the filter's
   information. The journal additionally (and correctly) lists workers/progress
   as unobservable; the ctor sets both `None`, matching the mirror (no pool).
4. **Mutation proof credible; committed code intact.** `for_incremental.rs:125`
   carries `restrict_and_validate_units(&registry, language, &mut build)?`.
   Structurally, removing it turns exactly the two narrowing cases red: the
   Scala-marker fixture shapes `App.scala` as the sole unit and the Cpp filter
   retains 0 of 1 (fixture guard test:243-247), the no-marker C++ fixture
   shapes `tu.cpp` and the Scala filter retains 0 of 1 (test:262-266) - both
   fail the build-parity assertion (plus the retained-count view assertion);
   the unrestricted, retain-everything, and error cases are restriction-neutral
   and stay green. The journal's `3 passed / 2 failed` mutation result is
   consistent with this. (As reviewer I cannot and did not mutate source; the
   structural verification + the intact committed restriction + the reported
   mutation run constitute the evidence.)
5. **Non-goals confirmed.** analyze.rs's construction sequence does no workers
   setup, no `incremental_backend` plugin selection at construction (its
   `plugin = ...` at :48 is a run-time dispatch over the restricted build,
   which AR-23 pairs at the switch site), and no artifact persistence
   (`save_artifacts` is gated on `authoritative` after `analyze()`). The ctor
   mirrors all three absences: `workers: None`, no backend call, no
   `save_repository_artifacts`/`save_build_context_artifacts` (contrast
   `build_with_progress` lib.rs:268-273, which the ctor deliberately omits).
6. **Existing constructors untouched.** The lib.rs diff is the +13 only;
   `new`/`with_registry`/`build`/`build_with_progress`/`new_with_progress`
   bodies appear in the diff as context, not changes. The 35 pre-existing
   pipeline tests are green inside the 40/0/4/9 line.

## Findings (numbered)

1. **Minor (evidence shape).** `tasks/ar-22-handoff-journal.md` gate/TDD
   digests quote verdict lines and exit codes from the distilling entry points
   (`testrun.py --distill`, `distill.py --`) with zero raw paste, but omit the
   capture paths (`target/testrun-capture.txt`, `target/distill-capture.txt`)
   that `docs/specs/tool-output-distillation.md` says the digest prints.
   Non-blocking; quote them next handoff.
2. **Minor (forward note for AR-23, not a defect here).** `for_incremental`
   calls `ArtifactManager::prepare_for_repo` = `init` = `create_dir_all` of
   `artifacts/` (crates/report/src/manager.rs:111-121) at construction, while
   the CLI mirror today creates that directory only when the analyzer/persist
   path first needs it. Harmless, consistent with every other Pipeline
   constructor, and parity-locked against `ArtifactManager::init`; the journal
   states it. Fold a byte-level confirmation into AR-23's switch-site parity.
3. **Informational.** The fps parity surface locks the ctor against a
   replication of the shared `compute_inputs` fingerprint formula; analyze.rs's
   incremental path never computes `StageFingerprints` itself, so no
   CLI-output comparison exists or is claimed. The journal declares this; it is
   a shape lock (pre-restriction inputs, matching `rehash_inputs` and every
   other constructor), not a CLI-output byte lock. Correct invariant.

## Declared deviations - individual dispositions

- D1 `lib.rs` carries the field + `incremental_view: None` literal init
  (+13 rider): **accepted.** A struct that gains a field must initialize it in
  every literal; construction behavior (inputs, artifact writes, fingerprints)
  is unchanged and locked by the 35 pre-existing tests.
- D2 no repository/contexts artifact refresh at construction: **accepted.**
  The mirror writes nothing at construction (verified at analyze.rs:27-58);
  writing what the mirror does not write would change observable state.
- D3 `language_filter` stored but restriction applied through the ctor's own
  retained set: **accepted.** The mirror's restriction IS a unit-set retain,
  not the index stage's re-shaping; the stored filter seeds the view and keeps
  the field honest, no stage is run.
- D4 two-parameter signature, no workers/registry knob: **accepted.** The
  mirror has no worker pool; registry delegation to `build_default_registry`
  is D9-locked both sides.
- D5 no backend plugin selection: **accepted.** Run-time dispatch, AR-23's
  integration point per the handoff's own scope ("locate/discover/shaper/
  restrict ... artifact-authority").
- D6 `--lang Other` rejected before discovery; unhandled-unit bail text copied
  verbatim though unreachable via shaper-shaped fixtures: **accepted.** Guard
  order matches the mirror; text parity is the point.

## Follow-up dispositions

- Findings 2 (AR-23 byte-level `artifacts/`-creation confirmation) and 1
  (capture-path quoting) travel to AR-23's handoff / the ledger
  deferred-findings row at the orchestrator's closeout - recorded here, not
  dropped.

## Git constraints honored

Read-only git only (`status` via checkpoint_guard, `diff --name-only`, `show`,
`log -- <path>`, `rev-parse` with whole-quoted `rev:path` args). No source,
test, or journal file was modified by this review.
