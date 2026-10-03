# M24-Part-A Kickoff -- Open language identity, plugin-authoritative detection

Spec source: the plan-repair lane's finding (see ADR-0001 § *Known exception*) and
`docs/adr/0001-language-plugin-boundary.md`. This is **Part A only** of the agreed
four-part third-language sequence. Parts B (dispatch), C (a third language plugin) and
D (the six CLI sites) are separate milestones.

Base: branch `multi-language-support` @ `0375bd9` (plan-repair lane closed; 15 markdown
files changed, zero source diff, hygiene exit 0, `cargo test --workspace` = 1088/0/3
unchanged by construction).

**Outcome in one sentence:** after this milestone, adding a language is *one crate plus
one registration line* -- because the language identity type can name a language that is
not C++ or Scala, and detection asks the plugin rather than a closed file-kind table.

---

## 0. Premise corrections (verified against the tree 2026-10-03, do not re-derive)

1. **The blocker is `crates/lang/src/registry.rs:109-114`, not the `FileKind` mapping.**
   `detect_language` already asks the plugin's detector and then *discards* the answer
   when the closed spine disagrees:

   ```rust
   pub fn detect_language(&self, path: &Path) -> Option<LanguageId> {
       if language_of_file_kind(FileKind::from_path(path)) == LanguageId::Other {
           return None;
       }
       self.plugin_for(path).map(|plugin| plugin.id())
   }
   ```

   A plugin claiming `.py` yields `None`, because `FileKind::from_path` sends unknown
   extensions to `FileKind::Other`. This is **locked by a test**:
   `detect_language_yields_none_when_file_kind_maps_to_other` (~registry.rs:499) asserts
   `None` for `src/main.mock` while `plugin_for` returns `Some`. Flipping this is a
   deliberate contract change, not a bug fix -- say so in the commit.

2. **`LanguageId` is an alias, not a new type.** `crates/lang-api/src/identity.rs:6`:
   `pub use ariadne_repo::SourceLanguage as LanguageId;`. `SourceLanguage` is
   `{Cpp, Scala, Other}` (`crates/repo/src/types.rs`). A plugin's `LanguagePlugin::id()`
   therefore *cannot name a language other than those three*.

3. **`Other` is reached, not hypothetical.** `FileKind::from_path` maps unknown
   extensions to `FileKind::Other`, and `language_of_file_kind` maps that to
   `SourceLanguage::Other`. Correction to a common misreading: `.py` does **not** become
   `FileKind::Source`; `FileKind::Source` is produced only by explicit `cpp|cc|cxx|c`.
   So a third language's files are labelled `Other` and refused, not mislabelled `Cpp`.

4. **The silent misroute is real.** `crates/cli/src/commands/plan.rs:70-81` -- two
   closures, each `match` on `SourceLanguage` with a `Scala` arm and a **`_` fallback to
   `ariadne_lang_cpp`**. A third language falls to `_` and silently receives C++
   aggregation and naming rules. That is a wrong answer, not a compile error.

5. **Opening the identity type is artifact-compatible.** `SourceLanguage` derives
   `Serialize`/`Deserialize` with **no `rename_all`**, so it serializes as a bare variant
   string. Verified in artifacts: `tests/golden/basic/**/*.json` contains
   `"language": "Cpp"`. A `#[serde(transparent)]` newtype over a string round-trips
   `"Cpp"` / `"Scala"` / `"Other"` byte-identically. This is the enabler for the whole
   milestone and the reason it is cheap -- **verify it with a round-trip test before
   relying on it.**

6. **Tooling reality on this branch.** `tools/` holds only `hygiene_check.py` and
   `hygiene_check_tests.py`. `testrun.py`, `distill.py`, `encoding_guard.py`,
   `golden_guard.py`, `commit_scoped.py`, `checkpoint_guard.py` exist on `main` but NOT
   here. Gate commands must be substituted; say which.
   `hygiene_check.py` has `R2_SCANNED_SUFFIXES = {".rs",".py",".html",".js"}` at line 19.

---

## 1. Intent and invariants

**Intent.** Make language identity *open-ended* rather than a closed three-variant enum,
and make the plugin's detector -- not a file-kind table -- the authority on which
language a file belongs to. This removes the structural reason the plan's promise
("a new language = 1 new crate + 1 registry entry, 0 edits to core crates") is false.

**Invariants that must hold before and after:**

- **I1.** Existing C++ artifacts are byte-identical. `tests/golden/**` must not change
  by one byte for `tests/fixtures/basic` and `tests/fixtures/expensive`.
- **I2.** The m17 guard stays green: `core_crates_do_not_name_languages` and
  `plugin_crates_do_not_depend_on_core_pipeline` are unmodified and passing.
- **I3.** Registration order remains the deterministic tiebreaker when two plugins claim
  one path (locked by `detect_language_first_registered_claiming_plugin_wins`).
- **I4.** No new dependency. No `libloading` / `wasmtime` / tree-sitter. A hand-written
  scanner is the established trust level.
- **I5.** Every core-crate site that previously branched on language identity now
  resolves through the registry or returns an explicit unhandled-language error. **No
  `_ => Cpp` fallback may survive anywhere.**
- **I6.** `crates/repo` keeps its policy role (`ThirdParty` / `Generated` / `Frozen`
  classification) unchanged.

---

## 2. Contracts (exact; adapt only if the tree forces it, and record why)

```rust
// crates/lang-api/src/identity.rs
/// Open-ended language identity. Serializes as a bare string, so every
/// pre-existing artifact ("Cpp" / "Scala" / "Other") round-trips unchanged.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LanguageId(Arc<str>);            // or Box<str> / Cow<'static, str>

impl LanguageId {
    pub const CPP: &'static str = "Cpp";
    pub const SCALA: &'static str = "Scala";
    pub const OTHER: &'static str = "Other";
    pub fn new(name: impl Into<Arc<str>>) -> Self;
    pub fn as_str(&self) -> &str;
}
```

`Other` is **not** constructible as a plugin identity -- see D-A3.

```rust
// crates/lang/src/registry.rs
/// Plugin-authoritative detection: the registered plugin's detector decides.
/// The `FileKind` spine no longer vetoes the answer.
pub fn detect_language(&self, path: &Path) -> Option<LanguageId>;

/// The plugin for a path, with no language-identity precondition at all.
pub fn plugin_for(&self, path: &Path) -> Option<&dyn LanguagePlugin>;   // already exists
```

---

## 3. Decisions

- **D-A1 -- open the type, do not add variants.** `LanguageId` becomes a string
  newtype. The alternative -- one new `SourceLanguage` variant per language -- keeps the
  promise false by construction, because every future language pays a `crates/repo`
  edit. Rejected.
- **D-A2 -- plugin detection is authoritative.** `detect_language` loses its
  `FileKind::Other` early return. The flip is intentional and the locking test is
  rewritten to assert the new contract.
- **D-A3 -- `Other` is legacy-only.** It stays the `#[serde(default)]` value so
  pre-existing artifacts lacking the field read as `Cpp`-compatible, and it remains the
  sentinel for "no plugin claimed this". It is never a plugin's `id()` and never returned
  by `detect_language`. Assert both.
- **D-A4 -- no C++ default, anywhere.** Any `match` on `LanguageId` in a core crate must
  resolve to a plugin lookup or an explicit error. The compiler is the discovery
  mechanism: opening the type should *surface* the six CLI sites as errors, not require
  hunting them.
- **D-A5 -- the registry, not `FileRecord.language`, is the dispatch authority.**
  `catalog_languages` and every consumer route through `plugin_for` /
  `detect_language`. `FileRecord.language` remains as an artifact label only.
- **D-A6 -- `FileKind` keeps policy, loses language.** `crates/repo/src/policy.rs` still
  needs `FileKind` for ThirdParty/Generated/Frozen. `FileKind::language()` is either
  removed or explicitly documented as advisory-only. Do not delete `FileKind` variants.
- **D-A7 -- `crates/repo` is not forbidden from changing**, but any change must be
  additive and serde-safe, and must be justified in the handoff. The m17 guard does not
  walk `crates/repo` (the "repository spine must be able to name languages" carve-out),
  so a new name there is invisible to the guard -- state that honestly in the report.

### Scope boundary that matters

D-A4 makes **`plan.rs:70` and `plan.rs:78` compile errors**, so Part A genuinely fixes
the silent misroute. It does **not** surface `optimize.rs:102/247/272-281` or
`ui.rs:359-368` -- those pass hook *functions* directly and contain no language `match`.
They belong to Part D (planner hooks become plugin-supplied). Do not attempt them here;
record them as remaining.

---

## 4. Task breakdown (RGB per task; author a `tasks/m24a-t<N>-handoff.md` per the
`handoff-spec` skill before executing each)

| # | Task | Red (failing test) | Green (implementation) | Refactor |
|---|---|---|---|---|
| T1 | Open `LanguageId` | A test constructing a non-Cpp/Scala id (e.g. `"Python"`) and serde round-tripping `"Cpp"` -> `"Cpp"` fails to compile | Newtype per §2; `Cpp`/`Scala` as consts or `From` impls; update every construction site | Centralize name construction; kill stringly-typed comparisons |
| T2 | Plugin-authoritative detection | Rewrite `detect_language_yields_none_when_file_kind_maps_to_other` to assert `Some(<the claiming plugin's id>)` for `src/main.mock` | Remove the `FileKind::Other` veto; update the doc comment to state the new contract | Ensure I3 (registration order) still holds and is still tested |
| T3 | Core-crate `LanguageId` audit | The compiler's errors are the red: open the type and record every non-exhaustive `match` | Resolve each to a registry lookup or an explicit unhandled-language error. **This is where `plan.rs:70/78` dies** | No site may keep a C++ fallback (D-A4) |
| T4 | Dispatch authority | A test proving a file whose `FileRecord.language` says `Other` still dispatches to the claiming plugin | Repoint `catalog_languages` / `plugin_for_catalog_file` consumers through the registry (D-A5) | Single resolution helper if ≥2 call sites |
| T5 | `FileKind` policy/language split | A test pinning that `FileKind` still drives ownership/policy but no longer drives language | Document or remove `FileKind::language()` (D-A6); keep `policy.rs` untouched in behavior | -- |
| T6 | Acceptance + locks | New `m24_language_identity_test.rs` red first | Prove: arbitrary id round-trip; artifact byte-identity; guard green; unhandled-language errors; no C++ fallback survives | Keep functions <100 / files <1000 lines |

**Ordering.** T1 before everything (nothing compiles until the type is open). T2
independent of T3. T4 depends on T2. T5 independent. T6 last.
Suggested parallel lanes: `T1 -> {T2, T5}` then `T3` then `T4` then `T6`.

---

## 5. Frozen locks

- **`tests/golden/**` -- byte-identical** for both C++ fixtures. This is the sharpest
  check of D-A1's artifact-compatibility claim. Verify explicitly; do not assume.
- **Unedited and green:** `m00_golden_baseline_test.rs`, `m17_plugin_seam_test.rs`,
  `m18_cpp_plugin_parity_test.rs`, `m10`, `m14`, `m20`, `m22`, `m23`. If one must change,
  that is a finding to report, not a liberty to take.
- **`crates/lang-cpp/**` and `crates/lang-scala/**` behaviour** -- C++ and Scala results
  must not change.
- **No new dependency**; `Cargo.lock` must not gain a package.

## 6. Acceptance (machine-checkable)

1. `cargo test --workspace` green, with the count no lower than 1088 pass / 0 fail /
   3 ignored.
2. `git diff -- tests/golden` **empty**.
3. `m17_plugin_seam_test.rs` unmodified and green.
4. A registered test plugin with a non-Cpp/Scala id resolves through `detect_language`,
   `plugin_for_id`, and the CLI dispatch path.
5. A language with no plugin yields an explicit unhandled-language error at every stage;
   a repo-wide search finds **zero** remaining `SourceLanguage::Cpp` fallbacks in a
   `match` arm.
6. `python tools/hygiene_check.py` exit 0.
7. `Cargo.lock` diff empty.

## 7. Out of scope

Parts B/C/D: the third language plugin itself; planner hooks becoming plugin-supplied
(`optimize.rs`, `ui.rs`); real compiler invocation and extraction include-closure
(D-M23-12); the `source_file` planner-provenance refinement (deferred at Gate-G3);
`ScalaToolchainAnalyzer`; package-scoped SCC condensation; dynamic/`libloading`/WASM
plugin loading (parked -- see the plan's M24 bullet).

## 8. Report back

Per task: commit hash + `git show --stat`; red evidence; green evidence as a canonical
line + exit code; deviations + reason; out-of-scope observations report-only. Never paste
raw tool output. Use `commit_scoped.py` **only if present** -- on this branch it is not,
so scope plain `git add` by hand and never `git add .`.

---

# CLOSEOUT -- Part A landed 2026-10-03

Six commits on `multi-language-support`, base `fe1d649`:

| Task | Commit | What it did |
|---|---|---|
| T1 | `994ff71` | Open `LanguageId` as a `Cow<'static, str>` newtype (D-A1) |
| T2 | `7aee77e` | Plugin-authoritative `detect_language`; the `FileKind` veto is gone (D-A2) |
| T5 | `15d7c6b` | `FileKind` policy/language split pinned (D-A6) |
| T4 | `ce9a55b` | Registry is the dispatch authority, not `FileRecord.language` (D-A5) |
| T3 | `e627601` | The silent C++ misroute in the planning dispatch is dead (D-A4) |
| T6 | `f6ad4ba` | `m24_language_identity_test.rs`, 8 acceptance tests |

53 files changed, +1718/-270. **Executed in the architect session, not delegated:** a global
rule (`{"permission":"bash","action":"deny","pattern":"*","source":"global"}`) denies `bash`
to every subagent, verified by probe against both an `implementer` and a `tool-caller`
subagent, so no delegation topology could run a test or a build on this branch.

## Gate numbers (§6, all seven)

1. `cargo test --workspace --no-fail-fast` → **exit 0, 1106 passed / 0 failed / 3 ignored**
   across 82 suites. Baseline at `fe1d649` was 1088/0/3; the +18 are this milestone's tests.
2. `git diff -- tests/golden` → **EMPTY**. `m00_golden_baseline_test` green (2/2). The six
   golden files carrying a language field (`repository_files.json`, `build_contexts.json`,
   `index_index.json`, x2 fixtures) are byte-identical.
3. `m17_plugin_seam_test.rs` → **UNEDITED and green (6/6)**, including
   `core_crates_do_not_name_languages`. It earned its keep: it caught two of this author's
   own violations (`crates/report:243,281` spelling `LanguageId::Cpp` inside a guarded
   crate), both fixed to `cpp()`.
4. A registered plugin with a non-Cpp/Scala id resolves through `detect_language`,
   `plugin_for_id`, `plugin_for` and the CLI dispatch path —
   `a_third_language_plugin_resolves_through_detection_and_id_lookup` and
   `a_third_language_dispatches_through_the_cli_catalog`.
5. A language with no plugin yields the explicit unhandled-language error
   (`a_language_with_no_plugin_yields_an_explicit_unhandled_language_error`, which also
   asserts the message does not mention `Cpp`), and
   `no_cpp_fallback_survives_in_a_core_crate_dispatch_match` finds **zero** C++ fallbacks
   across eleven core crates. That scan was validated with a negative control: a planted
   `_ => ariadne_lang_cpp::cpp_aggregation_filter(...)` made it FAIL and name
   `plan.rs:44`.
6. `python tools/hygiene_check.py` → **exit 0**.
7. `git diff -- Cargo.lock` → **EMPTY**, 0 packages added. I4 holds: no dependency at all.

## Did the compiler find the match sites? (D-A4, the T3 census)

Opening `LanguageId` produced 13 compile-error waves across 53 files. **The complete list of
sites that broke, and how each was resolved:**

| Site | Resolution |
|---|---|
| `build/discovery.rs:245,264` | `record.language` — both fields stay `SourceLanguage` (ADAPT-8) |
| `graph/builder.rs` — `edge_from_outcome`, `resolved_edge`, `include_edge` | param type → `LanguageId` |
| `graph/builder.rs:487,491,540` | three R2-guard sites, not one (ADAPT-6) → `LanguageId::cpp()` |
| `graph/graph.rs` — `edge()` helper, 5 `IncludeEdge` literals, 3 `let cpp` | → `LanguageId`; `cpp.clone()` at call sites |
| `graph/{traversal,load_*}.rs` — 12 `IncludeEdge` literals | → `crate::types::LanguageId::cpp()` |
| `cluster/planner.rs:128` | `gap.language.clone()` (no longer `Copy`) |
| `cluster/planner.rs:678` | `matches!` const pattern impossible — `Cow` is not structural-match → equality guard |
| `cluster/frontier.rs` — 3 sites | → `ariadne_lang::LanguageId::cpp()` |
| `report/manifests.rs` — 4 `languages` fields + `collect_stage_languages` | → `Vec<LanguageId>`, generic over `Into<LanguageId>` |
| `report/{graph_io,metrics_io}.rs` | `.clone()` / `LanguageId::from(..)` |
| `index/cache.rs` — `TuCacheFingerprint.language` + `compute` | → `LanguageId` (ADAPT-5); `Default` = Cpp; **epoch NOT bumped** |
| `scheduler/tu_store.rs:357` — `compute_tu_closure_plugin` | `language: LanguageId`; 5 `plugin.id()` callers unchanged |
| `scheduler/{engine,worker}.rs`, `pipeline/lib.rs` x4, `cli/*` x9 | `.clone()` or `LanguageId::from(..)` |
| `lang-cpp/{resolver,shaper}.rs`, `lang/registry.rs:388,392` | `IncludeEdge.language` → `LanguageId::Cpp`; `self.language.clone()` |
| `cli/registry.rs`, `cli/commands/{analyze,includes,plan,transform,ui/node_maps}.rs` | `.clone()` / `from(..)`; `capability_backend_for_language` now speaks `LanguageId` |
| **`cli/commands/plan.rs` — the two dispatch `match`es** | **D-A4's real target.** `_ => cpp_...` deleted; explicit Scala / Cpp / no-rule arms |

**Correctly NOT migrated** (still the closed `SourceLanguage` label, by design):
`FileRecord.language`, `TuAnalysis.language`, `TranslationUnit.language`,
`CompilationContext.language`, `SourceLanguage::from_file_kind`.

## Which core crates were touched — the promise WAS false

`build` 1, `cli` 7, `cluster` 6, `graph` 10, `index` 1, `lang` 1, `lang-api` 2,
`lang-cpp` 2, `pipeline` 2, `report` 4, `scheduler` 3, `ui` 4, plus 8 integration tests.
`crates/repo` and `crates/lang-scala` (source) were **not** touched at all.

The plan's "a new language = 1 crate + 1 registry entry, 0 edits to core crates" is **not yet
true**, and Part A did not make it true: it removed the reason the promise was *structurally*
impossible. The remaining edits were the price of the type change itself, and after T1–T6 a
genuine third language needs **one crate plus one registration line** — no further core-crate
edits. Parts B/C/D still change core crates for reasons unrelated to identity.

## Deviations, adaptations and corrections to this document

**ADAPT-1 — `crates/repo` was NOT changed, and §2's shape had to bend.** `m17` (frozen)
asserts `FileRecord.language == SourceLanguage::Cpp` and `TuAnalysis.language ==
SourceLanguage::Cpp`, so those fields keep the closed label and `LanguageId` becomes the
PLUGIN identity. This also means §2's premise that `FileRecord.language` would become the
newtype is **false as written**, and it is why I1 holds by construction.

**ADAPT-2 — `Cow<'static, str>`, not `Arc<str>`.** Workspace serde is `features = ["derive"]`
only; serde gates its `Arc` impls behind `rc`. §2's "or `Cow<'static, str>`" sanctioned this.

**ADAPT-3 — hand-written `Debug`/`Display` emitting the BARE name.** **Not covered anywhere
in §0.** `index/cache.rs:78` hashes `format!("{language:?}")` into the TU cache fingerprint;
a derived `Debug` would emit `Cpp(Cow::Borrowed("Cpp"))` and silently change every cache
fingerprint, with no failing golden test. This was the single highest-risk item in Part A.

**ADAPT-4 — PascalCase value constants `Cpp`/`Scala`/`Other` retained**, plus snake_case
`cpp()`/`scala()`/`other()` accessors for the R2 guard's case-sensitive `Cpp` ban.

**ADAPT-5 — `TuCacheFingerprint.language` → `LanguageId`.** Forced by five
`compute_tu_closure_plugin(..., language: SourceLanguage, ...)` call sites passing
`plugin.id()`. Byte-safe (transparent serde + bare Debug), so `CURRENT_ANALYSIS_VERSION`
stays `2.0.0` and **no cache store is invalidated**.

**ADAPT-6 — the R2 exposure in `graph/builder.rs` is three sites (`:487`, `:491`, `:540`),
not one.**

**ADAPT-7 / D-A8 — the task order was corrected and §3's claim is false as written.**
"D-A4 makes `plan.rs:70` and `plan.rs:78` compile errors" does not hold for T1: under
ADAPT-1 `SourceLanguage` stays closed, so `plan.rs` keeps matching on it and T1 alone fixes
nothing there. The compile errors fire when the **CLI dispatch data** changes type, which is
the old-T4 change. Old-T4 therefore ran **before** old-T3, and T3's red is re-specified as
"the compiler errors emitted by the catalog type change". End state identical; ordering fixed.

**Scope deviation — `crates/cli/src/optimize.rs:523` WAS touched.** §3 states Part A "does not
surface `optimize.rs`". **False**: its `test_include_edge` helper builds an `IncludeEdge`,
whose `language` became `LanguageId`. One literal changed. No Part D hook redesign.

**`crates/ui/Cargo.toml` NOT modified.** `ariadne-ui` has no `lang` dependency and adding one
would change `Cargo.lock` (I4). `LanguageId` is re-exported from `graph::types` alongside
`IncludeEdge` instead, giving access with zero new dependency edges.

**Frozen-lock breaches — 4 frozen integration tests needed value-preserving type edits,
12 lines total.** §5 said a change to these "is a finding to report, not a liberty to take";
it is reported here. `m18_cpp_plugin_parity/{external_boundary_parity,resolver_parity}.rs`,
`m20_scala_dependency_graph_test.rs`, `m22_scala_load_analysis_test.rs`,
`m23_symbol_isolation_test.rs` (+ non-frozen `m19`, `m21`, and `m23`'s `catalog_languages`
arity). Every edit is `.into()` at a type boundary or one added argument; **no asserted value
changed** and all of them pass. The frozen file that mattered most — `m17` — survived unedited.

## What remains

**Part B — dispatch.** The planning hooks in `plan.rs` still hardcode
`ariadne_lang_cpp::cpp_aggregation_filter` / `cpp_target_path_for` and
`ariadne_lang_scala::scala_aggregation_filter` / `scala_target_path_for`, matched by explicit
identity. A third language therefore gets "no rule" (not an aggregation unit, unchanged
target) rather than a wrong answer — correct, but it cannot yet plan. Making the hooks
plugin-supplied is Part D. `incremental_backend` still dispatches on `tu.language`, a closed
label, so a real third language reports "unhandled language" there until Part B.

**Part C — a third-language plugin.** No real plugin ships; `Fortran` in the acceptance suite
is a deliberately minimal identity-and-detection stand-in.

**Part D — the six CLI hook sites.** `optimize.rs:102/247/272-281` and `ui.rs:359-368` pass
hook *functions* directly and contain no language `match`, so Part A did not surface them.
`ui.rs` in particular still takes hook functions with no language dispatch at all. **Recorded
as remaining, not fixed** — except for the one `IncludeEdge` literal at `optimize.rs:523`,
which was compile-forced.

**Still parked / untouched:** real compiler invocation and extraction include-closure
(D-M23-12); the `source_file` planner-provenance refinement (deferred at Gate-G3);
`ScalaToolchainAnalyzer`; package-scoped SCC condensation; dynamic / `libloading` / WASM
plugin loading.