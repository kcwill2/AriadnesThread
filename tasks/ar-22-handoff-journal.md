# AR-22 Journal - `Pipeline::for_incremental(...)` (A6 part 1)

## Scope / outcome

Purely additive constructor in `crates/pipeline`: new module
`src/for_incremental.rs` (`impl Pipeline` block, the `stage_index.rs` /
`stage_load.rs` submodule precedent), a new `Option<IncrementalView>` state
field + import in `src/lib.rs`, new lock suite
`tests/for_incremental_parity_test.rs`. Existing constructors, m00 goldens,
m00 facts-path parity, and every CLI file are behaviorally untouched.

`Pipeline::for_incremental(repo_path: &Path, language: Option<SourceLanguage>)`
replicates the `analyze.rs::run_incremental_analysis_filtered` sequence
(lines 32-56 + helpers): default-registry construction, the
`ensure_language_registered` guard (registration half of §3.3 fact 3), the
shared locate/discover/shaper/fingerprint `compute_inputs` call (byte-identical
to every other constructor's input stage), the `--lang` unit-set restriction,
the per-unit unhandled-language check (§3.3 fact 3), the
`artifacts_are_authoritative` rule, and the `ArtifactManager` +
`TuStoreContext` setup (minus artifact refresh - see non-goals).

## Parity surface (what the test actually locks)

`tests/for_incremental_parity_test.rs` replicates the CLI sequence test-local
(same public API calls: `Repository::locate`, `repo.discover`,
`BuildDiscoverer::new().with_repo_discovery().with_unit_shaper().discover()`,
`StageFingerprints::from_inputs`, the retain-by-`detect_language` restriction,
the unhandled-unit check, the authority rule, `ArtifactManager::init`) and
compares it against the constructor's pipeline, fixture by fixture:

- `repo.root` (canonicalized located root)
- `discovery.root` + `discovery.files` (`FileRecord` vector equality -
  ids, paths, kinds, sizes, content hashes, language labels)
- `build` (full `BuildDiscoveryResult`: contexts + the RESTRICTED unit set)
- `fps` (full `StageFingerprints` chain - computed over the pre-restriction
  inputs by both sides)
- `artifact_root()` (the `repo.root/artifacts` path)
- `incremental_view` (`Some` on this path only): `language_filter`,
  `discovered_units`, `retained_units`, and the derived
  `artifacts_are_authoritative()` vs the replicated rule
- error parity: the unregistered-language rejection text compared verbatim
  (`"language plugin not registered: Other"`)

Not observable from outside the crate (stated honestly, NOT faked into the
lock): `language_filter`, `registry`, `workers`, `progress`, and `store_ctx`
are private `Pipeline` fields with no accessors. `language_filter` is
instead locked transitively through `incremental_view.language_filter` (both
carry the same `--lang` value; the view is the honest public surface).
`registry` equality is not directly assertable (no `PartialEq` on
`LanguageRegistry`); the D9 registries-identical lock already pins the
cli/pipeline enumeration.

Fixtures are mutation-sensitive by shape: a Scala-marker repo
(`build.sbt`, main-root `App.scala` + a stray `legacy.cpp` so the Cpp filter
truly narrows a Scala-shaped unit set) and a no-marker C++ repo (first-wins
falls back to the C++ shaper, so the Scala filter truly narrows). Four
construction cases (unrestricted, Cpp-filter, Scala-filter, retain-everything
filter) plus the error case.

## Mutation proof (the lock is real)

- Baseline green: `5 passed / 0 failed / 0 ignored / 1 suites`.
- Mutation: commented out the `restrict_and_validate_units(...)` call in the
  ctor (uncommitted, then reverted).
  Result: exactly `3 passed / 2 failed / 0 ignored / 1 suites`, with
  `cpp_filter_on_scala_marker_repo_*` and
  `scala_filter_on_cpp_repo_*` failing on the
  "build discovery result (contexts + restricted unit set)" parity assertion
  (the ctor retained the filtered-out unit the mirror removed). The
  unrestricted and retain-everything cases stayed green, as they must.
- Reverted: full `-p ariadne-pipeline` back to green.

## TDD phases (command digests, verbatim)

- Compile-green discipline held throughout (shared worktree):
  `python tools/distill.py -- cargo check --workspace` ->
  `ok (exit 0, 0 warnings emitted)` before and after the implementation.
- RED: first implementation shipped WITHOUT the restriction guard
  (language_filter stored but never applied); scoped parity run:
  `python tools/testrun.py --distill -- -p ariadne-pipeline --test for_incremental_parity_test` ->
  `0 passed / 5 failed / 0 ignored / 1 suites`, failing for exactly the
  semantic reasons: the two narrowing cases on the build-result parity
  assertion, the authority/view cases on the missing `Some(view)`, and the
  unregistered-language case on the absent guard. No compile errors.
- GREEN: implemented the restriction sequence; same scoped run ->
  `5 passed / 0 failed / 0 ignored / 1 suites`.
- Mutation (above) -> reverted -> final green.

## Gate commands (own stdout, verbatim digest lines)

1. `python tools/testrun.py --distill -- -p ariadne-pipeline` ->
   `40 passed / 0 failed / 4 ignored / 9 suites` (prior 35 passed / 4 ignored
   / 8 suites + the new 5-test parity suite; exit 0).
2. `python tools/testrun.py --distill -- --test m00_golden_baseline_test` ->
   `2 passed / 0 failed / 0 ignored / 1 suites`;
   `python tools/testrun.py --distill -- --test m00_facts_path_parity_test` ->
   `2 passed / 0 failed / 0 ignored / 1 suites` (per-suite 2/0/0/1 each, 4/0/0/2
   combined as the handoff predicted). Both UNEDITED, no build contention observed.
3. `python tools/encoding_guard.py --all` -> silent exit 0 (clean).
   `python tools/hygiene_check.py` -> blocking rows are exactly the two known
   `crates/ui/src/server.rs` ones (R2 1172>1000 + R1 `api_graph_traversal`
   161 lines); zero NEW blocking rows, zero warnings from AR-22 files.

## Deliberate non-goals / deviations (with reasons)

- `lib.rs` carries two additive lines: the field + `incremental_view: None`
  in `build_with_progress`'s literal. `build`/`build_with_progress`/`new`/
  `with_registry`/`new_with_progress` bodies are otherwise untouched - the
  handoff's "ctor untouched" means the construction BEHAVIOR (inputs,
  artifact writes, fingerprints), which is unchanged; a struct adding a
  field must initialize it in every literal. All construction behavior is
  locked by the 35 pre-existing pipeline tests, green.
- No repository/contexts artifact refresh at construction (unlike
  `build_with_progress`): the CLI mirror's sequence writes nothing at
  construction, and writing artifacts the mirror does not write would
  change observable artifact state. The ctor still creates `ArtifactManager`
  (so `artifact_root()` is populated and parity-checked). Consequence:
  `for_incremental` never persists repository/contexts artifacts;
  `IncrementalAnalysisResult::save_artifacts` owns the authoritative writes,
  as before.
- The ctor stores `language_filter` but the `for_incremental` path applies
  the restriction through its OWN retained set (the mirror's restriction is
  a unit-set retain against `detect_language`, not the index stage's
  re-shaping `indexed_build()`). Storing the filter keeps the field honest
  (it is what the run was asked for, exactly like the CLI passing `--lang`
  to `ensure_language_registered` before mirroring) and seeds
  `incremental_view.language_filter`; no stage is run by this constructor.
- Signature: `(repo_path: &Path, language: Option<SourceLanguage>)`. The
  handoff sketch showed `(..., ...)`: workers is not part of the mirror
  (the incremental analyzer has no worker pool), and a `workers` knob would
  be speculative configurability. Registry is the crate's own
  `default_registry()` (the mirror's cli-side `default_registry()` delegates
  to `build_default_registry`, locked identical by the D9 test).
- No backend plugin selection: `incremental_backend` stays cli-owned until
  AR-23 (the mirror's `plugin = ...` line operates on the RESTRICTED build,
  which AR-23 will read from `pipeline.build` - that pairing is AR-23's
  integration point and parity surface, not this ctor's).
- `--lang Other` is rejected before any discovery (guard order mirrored);
  the unhandled-language bail text is copied verbatim from analyze.rs even
  though shaper-shaped units cannot currently trigger it (the CLI unit tests
  lock it with synthetic builds; the replicated test exercises only the
  reachable unregistered case).
- No UI/hex involvement; pure ASCII authored; no golden fixture touched.
