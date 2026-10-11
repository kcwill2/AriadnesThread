# Plan: audit-remediation (all HIGH + MEDIUM findings, 2026-10-09 code audit)

Source: full-workspace code audit 2026-10-09 (4 independent passes; gates executed).
Census anchor at plan time: `1280 passed / 0 failed / 13 ignored / 91 suites`;
tools `Ran 219 tests / OK`; clippy 0 errors (~209 unique warnings);
`encoding_guard --all` PASS; `hygiene_check` FAIL exit 1 (R1x1, R2x1, R3x173).

Coverage: the Critical finding S1 is included because it shares files and
acceptance gates with High finding S6. Every other Critical/High/Medium
finding from the audit maps to exactly one handoff below (coverage table at
the end). Low findings are deferred (listed at bottom, not scheduled here).

Discipline (all hands): TDD red-first; touch-set = declared files only;
commits via `python tools/commit_scoped.py`; every gate via distilling entry
points (`testrun.py --distill`, `distill.py --`); anchor-match rule before
every mutation (line numbers advisory, before-strings binding); STOP on any
undeclared red base; pure-ASCII authored content; ledger + memory mirror in
the same commit when a row flips.

## Concurrency with LIVE tracks (hard rules)

The audit ran while these tracks are open; UI hands serialize behind them:

1. `ui-degree-opacity-ramp` (T-R budget relief + T1 ramp, AMENDED plan open):
   moves code out of `graph.js` and lands new UI test files. AR-11 (JS
   escaping) and AR-14 (styles.css) MUST start on the ramp's landing commit.
2. `ui-extraction-card-clarity` (T-A/T-B PLANNED): touches
   `crates/ui/assets` + `ui_test`. AR-11/AR-13/AR-14/AR-15 must interleave
   with T-A/T-B only one-at-a-time on `main`; declare the interleave order at
   dispatch time, never run two UI hands in parallel.
3. `ui-traversal-back-button`: operator G7 browser matrix pending -- AR-11
   changes traversal.js markup, so its manual smoke should cover Back too.
4. Known tree discrepancy: the ramp amendment measured `graph.js` at 1051
   lines; the audit census measured 961. Implementers re-measure at start;
   any handoff landing on a graph.js >= 1000 base is blocked until T-R lands.

`server.rs` is touched by AR-01, AR-10, AR-13, AR-15: strictly serialized,
one commit each, in that order.

---

## Wave 0 -- security hotfix (same day, ship immediately)

### AR-01 | ui | Path-traversal containment + export script escape (S1 CRIT, S3 HIGH, S4-partial MED)
- Change: canonicalize `repo_root.join(...)` inputs in `/api/source` and
  assert `starts_with(canonical repo_root)`; reject absolute `file=` and
  `local_prefix=` escapes (or sandbox prefix into repo). In `export.rs`,
  post-serialize `<` -> `\u003c`, U+2028/2029 in the embedded JSON. Add an
  attribute-safe escape helper used by the shared renderers (full JS call-site
  sweep is AR-11).
- Impact: closes the only confirmed whole-disk read primitive in the product
  and removes the script tag break-out from the *shareable* export artifact.
  After this handoff, a hostile repo's filenames/symbols cannot read
  `~/.ssh/id_rsa` via the UI nor execute script in an exported report.
  Legitimate same-repo file viewing is unchanged (byte-identical responses
  for contained paths; locked by test).
- Touch: `crates/ui/src/server.rs`, `crates/ui/src/export.rs`,
  `crates/ui/assets/file-view.js`+`hotspots.js` (helper only), new locks in
  `crates/ui/tests/ui_test/server_api.rs` (or new mod).
- Gates: scoped ui suite; `ui_smoke` PASS; hygiene+encoding exit 0. ~120-180
  lines. Effort S (0.5d).

### AR-02 | cli + ui | Localhost default bind + CORS same-origin (S6 HIGH)
- Change: `cli/src/commands/ui.rs` binds `127.0.0.1` by default; new opt-in
  `--expose <addr>` flag (documented, warns). Replace `CorsLayer::permissive()`
  (`ui/src/server.rs`) with same-origin restriction.
- Impact: removes the unauthenticated LAN exposure that turned S1/S2 from
  local risks into remote/browser-exfiltration risks. Default UX unchanged for
  the normal human launch (browser on same machine). Anyone who today
  LAN-accesses the UI must pass `--expose` -- one-time behavior change,
  noted in help text.
- Touch: `crates/cli/src/commands/ui.rs`, `crates/ui/src/server.rs`,
  cli + ui test locks. ~60-100 lines. Effort S (0.5d). Depends: after AR-01
  (same file).

### AR-03 | report | Bounded bincode decode (S5 MED)
- Change: one decode-options helper (bincode 1.3
  `options().with_limit(...)` sized from observed payloads + margin; record
  the measured sizes in the journal) used by all `bincode::deserialize` sites
  in `report/src/manager/*.rs`; regression fixture: blob declaring huge
  collection length fails fast with an error, not an allocation abort.
- Impact: a corrupt or attacker-authored `artifacts/` blob can no longer
  abort the process via declared-length allocation; error surfaces as a
  normal cache-miss/error. Honest reads unchanged. Defense-in-depth --
  premise is local FS write.
- Touch: `crates/report/src/manager/**` + report tests + fixture. ~80-140
  lines. Effort S-M (1d).

## Wave 1 -- correctness (independent crates; can parallelize)

### AR-04 | validate | Rollback on mid-apply failure (T1 HIGH)
- Change: `apply_with` captures pre-mutation copies per file and restores all
  written targets before propagating any internal error (mirror the existing
  tier-failure rollback at lib.rs:792-834); add a failing-writer oracle test
  that fails on the second write and asserts byte-identical tree afterwards.
- Impact: the advertised transactional guarantee becomes true for IO/hook
  failures too; after this, no partial-repo mutation can escape an `apply`
  that returned Err (locked by regression test). Error text gains a
  "rolled back N files" line; success path byte-identical, goldens unaffected.
- Touch: `crates/validate/src/lib.rs`, `crates/validate/tests/**`. ~120-200
  lines. Effort M (1-2d).

### AR-05 | index | Fingerprint sentinel on serialize failure (T2 MED)
- Change: in `cache.rs`, replace `unwrap_or_default()` with an explicit error
  or a random-process-sentinel mix-in recorded into the entry so failing
  contexts can never hash to a shared empty-bytes digest; regression test for
  serialize-failure behavior (inject a non-serializable context via a test
  type or feature seam).
- Impact: eliminates a silent false-cache-hit class (stale analysis served
  under a wrong context). Worst case after fix: cache miss + error/warn, i.e.
  slower but correct.
- Touch: `crates/index/src/cache.rs`, new `crates/index/tests/`. ~60-100
  lines. Effort S (0.5-1d).

## Wave 2 -- algorithmic perf on gate-benchmarked paths (bench-gated)

Every handoff here carries a `bench_compare` gate via distiller; expected
numbers stated so reviewers can reject "no effect" perf claims.

### AR-06 | graph | O(1) edge dedup in `add_edge` (P2 HIGH)
- Change: maintain a `HashSet<(FileId, Option<FileId>... key type per Edge)>`
  alongside the edge Vec in `FileGraph`; `builder.rs:129` caller unchanged.
- Impact: graph build goes O(E^2) -> O(E). At 50k edges that is ~2.5e9
  comparisons removed -- the dominant builder cost on large corpora. Output
  edge order locked byte-identical (Vec order preserved).
- Touch: `crates/graph/src/graph.rs` (+ `builder.rs` if ctor plumbing), graph
  tests incl. an order-parity test. ~60-100 lines. Effort S (1d).

### AR-07 | graph | Symbol name->id side map + by-id API (P1 HIGH, part 1)
- Change: add `symbol_id_by_name` map (build alongside symbol insertion) +
  `analyze_symbol_load_by_id` on the load API, mirroring the existing
  `analyze_file_load_by_id` precedent. Purely additive; old fn kept.
- Impact: none at runtime yet -- this handoff exists so AR-08 can switch the
  caller without touching the graph crate again. Public API additive only.
- Touch: `crates/graph/src/load.rs` + its inline test mod.
  ~60-90 lines. Effort S (0.5-1d).

### AR-08 | pipeline | Stage-5 by-id symbol metrics (P1 HIGH, part 2)
- Change: `stage_load.rs:347` collects `(symbol_id, name)` once (or resolves
  via the AR-07 map) and calls the by-id API.
- Impact: kills the O(S^2) string-compare term in the stage-5 tail -- the
  exact segment `bench_compare.py` measures. On a 10k-symbol TU-set that is
  ~5e7 -> ~1e4 map lookups; expected measurable stage-5 median drop (declare
  observed ratio in journal; REJECT if bench shows none beyond noise).
- Touch: `crates/pipeline/src/stage_load.rs` + pipeline tests. ~40-80 lines.
  Effort S (1d incl. bench). Depends: AR-07.

### AR-09 | graph | Adjacency-driven scans (P3 HIGH + P4 MED-HIGH)
- Change: back `direct_includes` and the BFS in `traversal.rs`
  (`traverse_neighborhood`, `shortest_include_path`) and `callgraph.rs` with
  the existing memoized adjacency (`LoadMemo::file_adjacency_map`,
  `load_caches.rs:401`) or a lazily built adjacency index on the graph;
  keep result order identical.
- Impact: `/api/graph/traversal` and `/api/graph/traverse`/`path` click cost
  goes O(k*E)/O(V_visited*E) -> O(k*deg). On a 1k-node/20k-edge include graph
  a typical neighborhood click drops from ~10^7 comparisons to ~10^4 --
  the difference between a sluggish and an instant interaction. `frontier.rs`
  and `load_explain.rs` callers inherit the win.
- Touch: `crates/graph/src/graph.rs`, `traversal.rs`, `callgraph.rs`,
  `load_caches.rs` + graph tests (equivalence locks vs current scans).
  ~150-250 lines. Effort M (2d).

### AR-10 | ui | Search path: hoist regex + single-pass lowercase (P7 MED)
- Change: compile the wildcard `Regex` once per request (case-insensitive
  regex replaces both `to_lowercase()` allocs).
- Impact: per-search over 10k elements stops recompiling 10k regexes
  (regex compile is microseconds-scale each: removes ~10^4 compiles and
  2*10^4 allocations per keystroke). Directly removes UI typing lag on the
  filter box.
- Touch: `crates/ui/src/server.rs:630-645` region + ui lock. ~20-40 lines.
  Effort S (0.5d). Depends: after AR-01, AR-02 (file serialization).

## Wave 3 -- UI injection hardening (after ramp T-R/T1 land)

### AR-11 | ui/assets | Escape-every-interpolation + data-id refactor (S2 HIGH, S4 MED)
- Change: apply the AR-01 attribute-safe escaper at every repo-derived
  interpolation: `graph.js:596,:610,:637,:640`; `api.js:304,:324`;
  `traversal.js:44,:131`; `hotspots.js:275,:278`; extend `escapeHtml` with
  quote escaping. Replace inline `onclick="...('${id}')"` with `data-id` +
  `addEventListener` delegation (graph/api/hotspots). New oracle tests in
  `ui_test` that fixture-serve a filename containing `<svg onload>` +
  attribute quotes and assert inert rendering (single-pass string lock, not
  browser).
- Impact: closes the confirmed XSS cluster: hostile analyzed-repo content can
  no longer execute in the analyst's browser or read via the local API
  through injected script. Visible behavior unchanged for normal names.
  Event-delegation swap is behavior-preserving (click handlers locked).
- Touch: `crates/ui/assets/{graph,api,traversal,hotspots,file-view,main}.js`
  (mind graph.js budget/line lock post-T-R),
  `crates/ui/tests/ui_test/**` new locks. ~250-400 lines. Effort M (2-3d).

## Wave 4 -- structural (gate-red + oversize)

### AR-12 | tools | hygiene R2 sees `.css` (Q3 tool-gap MED)
- Change: add `.css` (and `.md`? no -- scope: `.css`) to R2 suffix set; add
  R1/R2 parity test in `hygiene_check_tests.py`; land BEFORE AR-14 so the
  styles.css split is gate-locked from day one.
- Impact: closes the enforcement blind spot that let a 2105-line file live on
  a green-gate repo. NOTE: after this tool change the tree is statically RED
  (styles.css 2105) until AR-14 lands -- declare the known-red pair at
  dispatch; pair AR-12+AR-14 as the acceptance unit if dispatch prefers one
  green landing (then AR-12 rides in AR-14's commit).
- Touch: `tools/hygiene_check.py`, `tools/hygiene_check_tests.py`. ~20-40
  lines. Effort S (0.5d).

### AR-13 | ui | `server.rs` split + `api_graph_traversal` decomposition (Q1/Q2/Q3 MED, clears R1/R2 red)
- Change: split `server.rs` (1120) into route-family modules
  (`routes/api_source.rs`, `routes/api_graph.rs`, `routes/api_payloads.rs`,
  `state.rs`, `server.rs` shell re-exporting same pub surface); decompose
  `api_graph_traversal` (764-924, 161 lines) into seed-resolution /
  closure-computation / projection functions. Verbatim moves + declared
  amendments only; route table byte-identical (`ui_smoke` asserts routes).
- Impact: clears BOTH live hygiene violations (R1x1 R2x1 -> hygiene exit 0);
  future containment/security edits (like AR-01) get small reviewable files
  instead of a 1120-line module; no runtime change (locked by `ui_smoke` +
  full ui suite UNEDITED except declared moves).
- Touch: `crates/ui/src/**` (+ `lib.rs` mod lines), zero test edits beyond
  declared path amendments. ~1000-1200 moved lines (sizing ceiling -- if
  review finds drift, re-split as T-a state+source routes / T-b graph routes).
  Effort M (2-3d). Depends: after AR-01, AR-02, AR-10.

### AR-14 | ui/assets | `styles.css` split by cohesion boundary (Q3 MED, 2105 lines)
- Change: split into `styles/base.css` (tokens/reset/layout),
  `styles/views.css` (per-view component rules), `styles/palettes.css` (theme
  palettes incl. dark-steampunk/grey), keep served URL stable via the same
  concat/serve path used today (or `<link>` roster -- whichever static_html
  locks allow; declared amendments to `static_html*.rs` slices).
- Impact: the 2.1x-oversized file clears R2 with the AR-12 gate watching;
  theme tracks (dark-grey flip, clarity T-A) stop colliding on one 2105-line
  merge magnet. Rendered CSS byte-equivalence locked (concatenated served
  bytes or computed-token oracle).
- Touch: `crates/ui/assets/**`, `static_html*` locks (declared),
  `theme_token_check` silent vs base. ~600-900 moved lines. Effort M (2d).
  Depends: after ramp T1 + clarity T-A land (theme lock table of those rows).

### AR-15 | ui | Payload clone + blocking-handler offload (P5 + P6 MED)
- Change: keep the export payload pre-serialized as `Arc<Bytes>` on rebuild;
  serve clones of the Arc (`ETag` + 304 on the 2 s poll); move BFS/memo
  handlers to `spawn_blocking` (precedent: `ui.rs:567,:798`);
  `api.js:121-142` switches to ETag/delta-aware fetch.
- Impact: removes a full deep-clone of the whole graph payload per request
  (2x payload peak per concurrent request -> ~zero) and unblocks tokio
  worker threads, so rebuild no longer queues behind request serialization
  and heavy traversal clicks stop stalling other requests. Poll ticks become
  near-free when nothing changed.
- Touch: `crates/ui/src/**` (post-split state/routes),
  `crates/ui/assets/api.js`, ui locks. ~200-300 lines. Effort M (2-3d).
  Depends: after AR-13.

## Wave 5 -- test hardening (parallel-safe, disjoint crates)

### AR-16 | scheduler | Worker dispatch tests (T3 MED)
- Change: unit tests in `worker.rs` for missing-language dispatch (the M24
  third-language error contract), unknown-task and cancel paths.
- Impact: the cross-language error contract stops being locked only
  indirectly by C++/Scala parity; a refactor of dispatch that drops the
  error path now fails fast in-crate instead of shipping to the CLI.
- Touch: `crates/scheduler/src/worker.rs` (test mod only). ~60-100 lines.
  Effort S (0.5-1d).

### AR-17 | tests/integration | fixture paths + polling + m21 deterministic lock (T4, T5, T6 MED)
- Change: replace `PathBuf::from("../../tests/fixtures/basic")` with
  `common::fixture_path` in m10/m15/m16/m17; replace the three fixed 250 ms
  sleeps with the existing poll-until-ready idiom (m21:129-157); make the
  m21 serve-while-loading/gating lock deterministic (force loading state via
  the boot seam or count-assert the probe fired).
- Impact: suites stop depending on process CWD (CI dir-independence); the
  known fixed-sleep flake class on loaded machines is removed; the m21
  serve-while-loading guarantee gets a lock that CANNOT silently stop
  asserting today.
- Touch: `tests/integration/tests/{m10,m15,m16,m17,m18,m21}*.rs`,
  `tests/integration/tests/common/mod.rs`. ~80-140 lines. Effort S-M (1-2d).

## Wave 6 -- architecture seams (order matters; additive before switching)

### AR-18 | lang + lang-cpp + lang-scala | Planning hooks onto `LanguagePlugin` (A1 part 1)
- Change: add `aggregation_filter(&self, repo_root, path)` +
  `target_path_for(...)` to the plugin trait with a default
  "planning unsupported" impl; C++ + Scala override with their existing
  functions (normalize the curried-vs-direct signature divergence).
- Impact: none observable yet (defaults keep every caller compiling); removes
  the *need* for CLI edits when a language joins -- the seam now carries
  planning.
- Touch: `crates/lang/src/analyzer.rs`/plugin trait, `crates/lang-cpp/`
  plugin/plan, `crates/lang-scala/src/plugin.rs`, per-crate tests. ~120-180
  lines. Effort M (1-2d).
- Impact note for reviewers: multi-crate is intentional (seam change is
  indivisible); gates stay per-crate scoped.

### AR-19 | cli | Dispatch via registry; drop hardwired cpp imports (A1 part 2)
- Change: `optimize.rs:102` import removed; call sites use the registry
  plugin dispatch; `plan.rs` 2-arm match deleted.
- Impact: `tool optimize` and UI extraction proposals become CORRECT for any
  declared-planning language -- Scala stops silently yielding empty/incorrect
  proposals (new m24-adjacent integration lock demonstrates the Scala plan
  path). This is the first user-visible functional fix of Wave 6.
- Touch: `crates/cli/src/optimize.rs`, `crates/cli/src/commands/ui.rs`,
  `crates/lang-cpp/src/plan.rs` (removal), integration lock. ~100-150 lines.
  Effort M (1-2d). Depends: AR-18.

### AR-20 | lang-api + lang-cpp + scheduler | Move `WorkerBackend`/`TuTask` into `lang-api` (A4 MED)
- Change: relocate the worker-api types/traits (or a new `worker-api` module
  in lang-api); repoint `lang-cpp` impl imports; remove
  `lang-cpp -> scheduler` Cargo dep; keep the m17 guard name honest by also
  adding a transitive-dependency assert to the existing guard test.
- Impact: plugin crates stop transitively compiling against
  report/cluster/transform/validate -- the isolation the guard *claims*
  becomes real and machine-checked; plugin builds get faster; future plugin
  work can no longer accidentally reach core internals through scheduler.
- Touch: `crates/lang-api/src/**`, `crates/lang-cpp/src/backend.rs`,
  `crates/scheduler/**` (re-export), `crates/lang-cpp/Cargo.toml`, guard
  test. ~150-250 lines. Effort M (2d).

### AR-21 | lang-cpp | Dual resolver -> gated test oracle (A5 MED)
- Change: `#[deprecated]` + cfg-gate `IncludeResolver` behind a test-only
  feature (or `#[cfg(test)]`/dev-dep wiring for the parity oracles), record
  removal date-note; parity suites keep working UNEDITED.
- Impact: one fewer same-responsibility live implementation; drift risk
  becomes compile-time enforced instead of test-hopeful. No runtime change.
- Touch: `crates/lang-cpp/src/resolver.rs`, `lib.rs`, `Cargo.toml`,
  `tests/integration/tests/m18_cpp_plugin_parity/resolver_parity.rs`
  (feature wiring only if needed). ~50-100 lines. Effort S-M (1d).

### AR-22 | pipeline | `Pipeline::for_incremental(...)` (A6 part 1)
- Change: additive constructor encapsulating locate/discover/shaper/restrict
  for the incremental path (the sequence CLI mirrors today at
  `analyze.rs:32-56`).
- Impact: none yet; creates the single authority site that AR-23 switches to.
- Touch: `crates/pipeline/src/lib.rs` + pipeline tests. ~80-120 lines.
  Effort S (1d).

### AR-23 | cli | Analyze incremental path consumes pipeline (A6 part 2)
- Change: `analyze.rs` calls `for_incremental`; delete the mirrored policy;
  self-admitted mirror comment (analyze.rs:110-114) removed.
- Impact: unit-restriction and artifact-authority policy become ONE
  implementation; future policy work can no longer fix one path and rot the
  other (the drift class the audit flagged). Observable behavior locked
  identical by an added parity test run before deletion.
- Touch: `crates/cli/src/commands/analyze.rs`, cli tests, integration lock.
  ~60-100 lines. Effort S-M (1d). Depends: AR-22.

### AR-24 | lang-api + lang (+ plugin crates) | Typed errors at the seam (A7 MED)
- Change: `thiserror` enum in `lang-api` (or `anyhow::Result` where
  structure adds nothing -- decide per method); convert
  `lang/src/validator.rs` `Result<(), String>` and every trait method
  returning `String`; adapt lang-cpp/lang-scala impls; keep message text
  stable so CLI-visible error strings don't shift (locked).
- Impact: compiler diagnostics stop losing structure at the extensibility
  seam -- call sites can branch on error kind instead of string matching;
  new-language plugins get typed error contracts. CLI output unchanged
  (display impl renders identical text).
- Touch: `crates/lang-api/src/**`, `crates/lang/src/validator.rs`,
  impl crates. ~150-250 lines. Effort M (2d).

### AR-25 | cli (+ report) | Consumes typed seam errors (A7 riders)
- Change: drop `.map_err(anyhow::Error::msg)` re-wraps at `registry.rs:105`/
  `analyze.rs:49`; fold bespoke `StaleArtifact` reporting onto the shared
  pattern where trivial.
- Impact: removes the double-wrapping layer so error chains stay intact
  end-to-end (context preserved for debugging). No user-visible change.
- Touch: `crates/cli/src/registry.rs`, analyze paths, `report/manager.rs`.
  ~40-80 lines. Effort S (0.5-1d). Depends: AR-24.

### AR-26 | new crate | Artifact-store extraction (A3 MED)
- Change: new `crates/artifact` crate owning `manager/*` IO (transform_io,
  apply_io, metrics_io, graph/index/discovery io, manifest); `report` keeps
  presentation/report DTOs and depends DOWN on it; `scheduler` depends on it
  directly, dropping `scheduler -> report`.
- Impact: scheduler's dependency cone loses the entire
  cluster/transform/validate stack (today dragged in via a "reporting"
  crate); cleaner rebuild layering, honest crate names, smaller plugin
  cones after AR-20 compound with this. No behavior change (verbatim moves +
  declared paths).
- Touch: `crates/artifact/**` (new), `crates/report/**`,
  `crates/scheduler/**`, workspace `Cargo.toml`. ~400-700 moved lines;
  size-check at 500 -> split report-side from scheduler-side reparent.
  Effort M-L (3-4d). Depends: AR-20 preferred first.

### AR-27 | repo + lang-api + cli | `SourceLanguage` -> `LanguageId`, open `--lang` (A2 HIGH-risk)
- Change: migrate `TuAnalysis.language`, `FileRecord`, and `--lang`
  acceptance to `LanguageId` (serde-transparent, precedent exists); derive
  `--lang` candidates from `registry.languages()`; `FileKind` per-language
  variant sections replaced by language-id + kind-role; serde aliases keep
  artifact bytes stable across the bump (m23 bincode locks run UNEDITED or
  version-bumped deliberately with written justification).
- Impact: delivers the open-set plugin end to end -- a new language plugin
  can be detected, TAGGED, persisted, and CLI-selected with zero edits to
  `repo`/`lang-api`/`cli`. This is the single largest enabler for the
  Scala-then-more intent; after it, a third language is plugin-only work.
- Touch: `crates/repo/src/types.rs`, `crates/lang-api/src/types.rs`,
  `crates/cli/src/registry.rs` + consumers; golden/serde lock review.
  ~400-800 lines. Effort L (4-6d). Depends: AR-19 (hook shape), AR-24 (typed
  errors land first to avoid merge pressure on lang-api types.rs).

## Wave 7 -- mechanical hygiene (background, per-crate slices)

### AR-28 | per-crate | clippy clone-on-Copy + `&PathBuf` sweep (Q5 MED)
- Change: fix the 95 clone-on-Copy + 10 `&PathBuf` sites crate-by-crate
  (each commit <= 1 crate); no semantic edits; bench exemption justified per
  commit (behavioral no-ops).
- Impact: silences the dominant warning class so NEW clippy output becomes
  signal again (today ~209 unique warnings hide the next real one); removes
  needless refcount churn on hot graph structures (small but real).
- Effort S per crate slice, ~3-5 slices total (3-5d spread).

### AR-29 | per-directory | R3 naming cleanup (Q4 MED)
- Change: fix the 173 R3 sites (`res`->domain nouns, `tmp`->purpose,
  single-letter locals where not conventional indices), per-directory
  commits; update `tools/hygiene_r3_allowlist.txt` instead of code where a
  name is genuinely local-conventional.
- Impact: warning-level gate noise -> zero, making R3 enforceable by habit;
  test names in `server_api.rs`/`graph_api.rs` become readable claims rather
  than `res`. Zero behavior change.
- Effort S-M spread across slices (2-4d).

### AR-30 | docs | Ledger truth-sync + AGENTS.md pointers + memory mirror (M1 MED)
- Change: flip/archive stale live rows per the ledger's own archive rule
  (M19 merge status vs `crates/lang-scala` on `main`; M20 `STARTING`;
  dark-grey `PLANNED` vs cited landing `41ec354` + execution-results file);
  fix `AGENTS.md` dead `~/.config/devin/...` skill pointers to `.kilo/skills`
  + `~/.agents/skills`; add one entry-path line. Same-commit memory mirror
  per the authority rule.
- Impact: the declared source of truth stops lying to every kickoff/memory
  mirror -- every downstream orchestrator decision (numbering, merge state,
  theme state) reads this file first; the stale pointers stop sending new
  sessions to a nonexistent toolchain directory.
- Touch: `tasks/milestone_ledger.md`, `tasks/ledger_archive.md`,
  `AGENTS.md`. ~40-80 lines. Effort S (0.5d). VERIFY each claim against git
  before flipping (the audit's claims are evidence, not the ledger's word).

### AR-31 | lang-cpp | Tokenizer allocation rework (P8 MED)
- Change: iterate `&[u8]`/slices instead of materializing `Vec<char>` per TU;
  build tokens as `&str`/`StringWriter` with capacity; single scan for the
  unterminated-raw-string EOF case; preserve the M27 byte-vs-char column
  underflow locks EXPLICITLY (characterization suite + both dated lock files
  run UNEDITED).
- Impact: removes the 4 B/char per-TU allocation + 2+ allocs/token flush on
  the ~3.1 s/TU debug profile -- expected allocation-count drop reported per
  TU in the journal and bench-neutral-or-better stage timing. Highest-risk
  mechanical change in the plan: byte-parity locked per M27 precedent.
- Touch: `crates/lang-cpp/src/tokenizer.rs` only. ~150-300 lines. Effort M
  (2-3d).

---

## Sequencing / dependency summary

- Immediately parallel-safe: AR-01; then AR-02 -> AR-10 -> AR-13 -> AR-15
  (server.rs serial chain); AR-03/AR-04/AR-05 (disjoint crates); AR-16/AR-17;
  AR-06/AR-07; AR-12(+AR-14 pairing decision); AR-20/AR-21.
- Chains: AR-07 -> AR-08; AR-09 after AR-06 (same file region); AR-11 after
  ramp T1; AR-14 after clarity T-A + ramp; AR-18 -> AR-19; AR-22 -> AR-23;
  AR-24 -> AR-25; AR-20 -> AR-26; {AR-19, AR-24} -> AR-27; AR-28/AR-29 last
  or in idle slots (they mutate many files -- rebase cost grows until the
  structural hands land).
- First dispatch recommendation (highest impact-per-risk): AR-01, AR-02,
  AR-04, AR-05, AR-06, AR-16.

## Coverage table (finding -> handoff)

- S1 CRIT -> AR-01 | S2 -> AR-11 | S3 -> AR-01 | S4 -> AR-01+AR-11 | S5 -> AR-03 | S6 -> AR-02
- A1 -> AR-18+AR-19 | A2 -> AR-27 | A3 -> AR-26 | A4 -> AR-20 | A5 -> AR-21 | A6 -> AR-22+AR-23 | A7 -> AR-24+AR-25
- Q1/Q2/Q3 -> AR-13 + AR-14/AR-12 | Q4 -> AR-29 | Q5 -> AR-28
- P1 -> AR-07+AR-08 | P2 -> AR-06 | P3 -> AR-09 | P4 -> AR-09 | P5 -> AR-15 | P6 -> AR-15 | P7 -> AR-10 | P8 -> AR-31
- T1 -> AR-04 | T2 -> AR-05 | T3 -> AR-16 | T4 -> AR-17 | T5 -> AR-17 | T6 -> AR-17
- M1 -> AR-30

## Explicitly out of scope here (Low findings, deferred by policy)

A8 (neutral-crate C++ search policy), A9 (IncludeKind rename -- pair with
AR-27 serde aliases if cheap), A10 (glob re-exports), S7 (blob digest
binding), S8 (worker temp-file hardening), P9 (closure clone on memo hit),
P10 (clang-worker dead clang exec + cargo-run fallback), T7 (30 s wall-clock
assert), T8 (golden order-only blind spot), M2 index remainder, M3
(stage_spans calibration TODO), M4 (distill capture clobbering -- recommend a
tools ticket: per-session `--capture` defaults).

## Verification & completion

- Per-handoff acceptance = declared gates green + commit-vs-touch-set EXACT +
  reviewer hop (review-gate skill).
- After Wave 0+1: re-run full census via `testrun.py --distill --workspace`
  + `unittest discover` + `ui_smoke` + `bench_compare` and record the
  comparison against the `1280 / 0 / 13 / 91` anchor in an execution-results
  doc.
- After Wave 2: `bench_compare` truth rows (expected stage-5 drop; no 1.25x
  regressions).
- After Wave 4: hygiene exit 0 (R1/R2), styles.css cleared, `--expose`
  documented; the audit's structural section re-verified by re-running the
  size census.
- Ledger: register this as ONE tracked workstream at dispatch (milestone-
  lifecycle assigns any M-number at kickoff -- this plan does not guess it);
  memory mirrors in the same commits.
