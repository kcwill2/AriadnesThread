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