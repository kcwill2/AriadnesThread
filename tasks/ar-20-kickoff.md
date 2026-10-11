# AR-20 Kickoff - A4: plugin crates must not transitively compile the core pipeline

Shape decision for the re-dispatch of AR-20 (finding A4 MED). The original
handoff (verbatim move of `WorkerBackend`/`TuTask` into `lang-api`, drop the
`lang-cpp -> scheduler` dep, guard GREEN) was BLOCKED and reported with zero
code landed (journal commits `d63c0d0` + `7adbeca`). This kickoff re-scopes
the milestone as a two-handoff chain; the shape is in `tasks/ar-20-shape.md`.

## Intent

- **Behavior change:** none at runtime. Build-graph change: `ariadne-lang-cpp`
  (and by lock, `ariadne-lang-scala`) stops transitively compiling
  `ariadne-scheduler`, `ariadne-report`, `ariadne-cluster`, `ariadne-graph`,
  `ariadne-transform`, `ariadne-validate`. Today the single edge
  `lang-cpp -> scheduler` drags all six (`scheduler` deps `graph` + `report`
  directly; `report` deps `cluster`/`transform`/`validate` directly).
- **Blast radius:** new crate `crates/worker-core`; `crates/scheduler` (loses
  ~705 engine lines, keeps its full public path surface via re-exports);
  `crates/lang-cpp` (one manifest dep dropped, three references repointed);
  `tests/integration/tests/m17_plugin_seam_test.rs` (transitive guard added
  GREEN). Everything else - pipeline, cli, ui, index, build, lang, report,
  all integration suites - compiles UNEDITED against scheduler's preserved
  surface.
- **Underlying why:** plan rule R1 ("plugin crates depend inward, never on
  the pipeline") is currently a lie: the direct-dep guard passes while the
  compiled cone is contaminated. A4's claim is the transitive cone itself,
  so the fix must remove the cone, not redefine the guard. Root cause of the
  blockage: the closure-merge engine (`tu_store.rs`) has ZERO scheduler
  imports - its real dependencies are `index`/`build`/`lang`/`repo`/`anyhow`
  - scheduler was merely its historical location. Relocating it BELOW both
  plugin crates and scheduler turns the load-bearing call at
  `crates/lang-cpp/src/backend.rs:180` into a legal downward call.

## Scope in

- New workspace crate `crates/worker-core` (package `ariadne-worker-core`)
  owning, verbatim: `scheduler/src/tu_store.rs` (all 705 lines incl. its test
  module), `TuTask` (struct + 4 trait impls), `WorkerBackend` trait, and the
  `impl<T: WorkerBackend + ?Sized> WorkerBackend for Arc<T>` blanket impl
  (orphan rule requires it to live with the trait).
- `crates/scheduler`: delete-moves + re-export shims preserving EVERY public
  path (`ariadne_scheduler::tu_store::X`, `...::queue::TuTask` resolves via
  re-export, flat glob exports) so all external consumers compile untouched.
- `crates/lang-cpp`: drop `ariadne-scheduler` dep, repoint `backend.rs` to
  `ariadne-worker-core`, rewrite the stale manifest note (Cargo.toml:11-15).
- `m17_plugin_seam_test.rs`: add `plugin_crates_transitive_tree_excludes_core_pipeline`
  (machine-proven RED by the AR-20 journal: `6 passed / 1 failed / 0 ignored /
  1 suites`, offender cone exactly the six forbidden crates), landed ONLY in
  the commit that makes it GREEN.

## Scope out (with reasons)

- **B (weaken the guard / exempt the scheduler edge):** orchestrator
  pre-ruled unacceptable, and mechanically confirmed: `scheduler` deps
  `ariadne-graph` DIRECTLY (`shard.rs:1`, `engine.rs:9`, `incremental.rs:34`),
  so an exempted scheduler edge still compiles `graph` into plugin crates
  even after AR-26 removes `scheduler -> report`. The exemption would not
  even bound the cone honestly.
- **A (lang-api closure-merge handle + engine injection):** feasible but
  strictly inferior - see shape "Options adjudicated" (visibility break,
  lang-api manifest growth toward engine state, `TuCacheFingerprint`
  relocation or API shape-change, `ClangWorkerBackend` construction churn
  editing the UNEDITABLE m16 suite).
- **A8 (neutralizing the engine's `clang++` fallback toolchain string and
  search policy):** A8 stays deferred per the plan; worker-core merely
  becomes the right future home for it. Verbatim move keeps the string;
  `worker-core` is deliberately NOT added to the R2 walk scope (7-crate
  list unchanged).
- Re-export repointing of scheduler-internal call sites (`engine.rs`,
  `incremental.rs`, `pool.rs` keep `crate::tu_store::...` paths): they
  resolve through the shims; churn without value inside this milestone.
- AR-26's artifact-store extraction: separate milestone; ordering constraint
  is stated in the shape (AR-20b gates before AR-26's scheduler-side reparent).
- Touching `crates/ui`, `crates/cli`, fixtures, goldens: no lane overlap.

## Invariants (numbered; proof commands in the shape)

1. **m18 byte-parity lock UNEDITED:** `m18_cpp_plugin_parity_test` files are
   byte-identical before and after the chain, and green at both landings.
2. **m00 golden lock UNEDITED:** `tests/golden/**` blobs unchanged; parity +
   golden suites green at both landings.
3. **Public-path compatibility:** no external file (pipeline, cli,
   tests/integration) is edited; `rg "scheduler::(tu_store|queue|worker)::"`
   hit-set identical before 20a and after; the module-qualified
   `ariadne_scheduler::tu_store::ShardCheckpoint` use in
   `m18_persistence_incremental_test.rs:226` keeps compiling.
4. **Test census conservation:** at 20a landing, scheduler-suite +
   worker-core-suite counts equal the pre-move scheduler totals with zero
   tests added or removed (8 tu_store tests travel).
5. **Guard never RED in history:** the transitive guard's single introducing
   commit is the 20b landing commit (`git log -S` proves one commit); m17
   suite goes `6 passed` -> `7 passed`, `0 failed`.
6. **End-state cone:** at 20b landing, `cargo tree -p ariadne-lang-cpp`
   contains none of `ariadne-scheduler`, `ariadne-report`, `ariadne-cluster`,
   `ariadne-graph`, `ariadne-transform`, `ariadne-validate` over compiled
   (non-dev) edges.
7. **lang-api stays vocabulary-thin:** `crates/lang-api/Cargo.toml` and
   `crates/lang-api/src/**` byte-untouched across the whole chain.

## Frozen locks inherited (with proving commands)

- m17 seam type-identity lock (`lang_api_types_are_the_same_types_index_re_exports`;
  also the reason `lang-api -> index` is impossible - the cycle the journal
  hit): `python tools/testrun.py --distill -p ariadne-integration-tests --test m17_plugin_seam_test`.
- m18 byte-parity locks: `python tools/testrun.py --distill -p ariadne-integration-tests --test m18_cpp_plugin_parity_test`
  (file set `git diff`-empty throughout).
- m18 persistence envelope lock: same runner, `--test m18_persistence_incremental`.
- m16 ClangWorkerBackend degradation suite: `--test m16_clang_worker_test`.
- m00 golden/parity: `--test m00`-scoped run; `git status --porcelain -- tests/golden` empty.
- Encoding: `python tools/encoding_guard.py --staged` (pre-commit), `--all` (CI).
- Hygiene: `python tools/hygiene_check.py` - expect exactly the 2 pre-existing
  `server.rs` rows (AR-13 owned), zero new.
- AR-19 precedent (plugin-free production, dev-dep plugin registration):
  `crates/cli` production keeps zero plugin deps; cargo dev-dep cycles into
  `lang-cpp` are the established pattern (`crates/build/Cargo.toml:16-21`
  note, `crates/report` dev-deps, `crates/scheduler` dev-deps).

## Acceptance gate (milestone-level)

- 20a: `cargo check --workspace --all-targets` green; scheduler+worker-core
  scoped suites match the conservation equation (34 total pre-move baseline);
  m18/m16/m18_persistence/m00 UNEDITED green; encoding + hygiene no-new.
- 20b: m17 `7 passed / 0 failed / 0 ignored / 1 suites` incl. the new
  transitive guard + its planted-edge self-test (non-vacuity); lang-cpp
  `153 passed / 0 failed / 0 ignored / 2 suites` maintained; m18 parity
  UNEDITED green; cargo-tree fact (invariant 6) quoted from own stdout;
  encoding + hygiene no-new.
- Chain closeout: workspace census row + ledger/memory mirror per workflow.
