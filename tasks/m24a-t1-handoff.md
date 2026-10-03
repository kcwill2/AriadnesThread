# M24-Part-A T1 Handoff — Open `LanguageId` as an open-ended string newtype

Base: branch `multi-language-support` @ `fe1d649`. Worktree `.kilo/worktrees/linen-mice`.
Binding spec: `plans/m24-part-a-language-identity-kickoff.md` (read §1 invariants, §2 contracts,
§3 decisions D-A1/D-A3/D-A7, §5 frozen locks, §6 acceptance).

**This file records four ADAPTATIONS the tree forces on the kickoff's §2 contract. Read
§"Forced adaptations" before writing code. They are not optional and not your judgment call —
they are the reason this handoff exists.** Each was verified against the tree at `fe1d649`.

---

## 1. Baseline (verified by the architect at `fe1d649`, do not re-derive)

- `cargo test --workspace --no-fail-fast` → **exit 0, 1088 passed / 0 failed / 3 ignored**, 81 suites, 0 errors.
- `python tools/hygiene_check.py` → **exit 0** (prints nothing on success).
- `git status --porcelain` clean. `git diff -- tests/golden` empty.
- Integration tests live in `tests/integration/tests/`, NOT `tests/`.
- `tools/` holds only `hygiene_check.py` + `hygiene_check_tests.py`. **`testrun.py`,
  `distill.py`, `encoding_guard.py`, `golden_guard.py`, `commit_scoped.py`,
  `checkpoint_guard.py` DO NOT EXIST HERE.** Substitute the commands in §7. Never run bare
  `cargo test`/`build`/`clippy` without distilling first (redirect to a capture file, then
  grep the summary lines).

---

## 2. Objective

Replace the closed-enum language identity with an **open-ended** one, so a third-language
plugin can name itself. Today `crates/lang-api/src/identity.rs:6` is
`pub use ariadne_repo::SourceLanguage as LanguageId;` and `SourceLanguage` is
`{Cpp, Scala, Other}` — so `LanguagePlugin::id()` physically cannot return a name outside
those three.

**Behavior of `LanguageId` is the only thing that changes.** This task changes no dispatch
logic, no registry semantics, no golden bytes.

---

## 3. Forced adaptations (READ CAREFULLY)

### ADAPT-1 — `crates/repo` is NOT touched. `SourceLanguage` stays a closed enum.

The kickoff's §2 comment implies `FileRecord.language` might become the newtype. **It must
not.** Proof — `tests/integration/tests/m17_plugin_seam_test.rs` is FROZEN unedited (§5 lock
I2) and asserts:

- line 78: `assert_eq!(tu.language, ariadne_repo::SourceLanguage::Cpp);`
- line 92: `assert_eq!(record.language, ariadne_repo::SourceLanguage::Cpp);`

If `TuAnalysis.language` / `FileRecord.language` became `LanguageId`, both lines fail to
compile. **Therefore: `SourceLanguage` keeps its three variants, `FileRecord.language` stays
`SourceLanguage`, `TuAnalysis.language` stays `SourceLanguage`, `TranslationUnit.language`
stays `SourceLanguage`, and `crates/repo` gains zero edits.** This also means I1 (golden
byte-identity) holds by construction for `repository_files.json`, `index_index.json`,
`build_contexts.json` — all three carry a `SourceLanguage` field, untouched.

`LanguageId` becomes the **plugin-identity** type. `crates/lang-api` depends on
`crates/repo` (verified: `crates/lang-api/Cargo.toml` → `ariadne-repo`), so `LanguageId`
stays defined in `crates/lang-api/src/identity.rs` exactly as §2 says. No new dependency edge.

### ADAPT-2 — `Cow<'static, str>`, not `Arc<str>`.

Workspace `Cargo.toml:31` is `serde = { version = "1.0", features = ["derive"] }` — the
`rc` feature is **NOT** enabled, and serde's `Arc<T>` impls are gated behind it. `Arc<str>`
would require a feature change to the manifest. `Cow<'static, str>` is natively supported by
serde both ways and is strictly better here: the three built-in ids are `Cow::Borrowed`,
i.e. **zero heap allocation** in the hot paths (`index/src/cache.rs:78` digest,
`graph`/`reduced`/`cache` per-node language). §2 explicitly sanctions `Cow<'static, str>`.

### ADAPT-3 — hand-written `Debug` (and `Display`) emitting the BARE name.

**This is the most dangerous item in the task and the kickoff does not mention it.**
`crates/index/src/cache.rs:78` does:

```rust
hasher.update(format!("{language:?}").as_bytes());
```

That string feeds the **cache fingerprint digest**. Today `{:?}` of `SourceLanguage::Cpp`
is `Cpp`. A *derived* `Debug` on a newtype would emit `Cpp(Cow::Borrowed("Cpp"))` and
**silently change every cache fingerprint** — a store-format break with no compile error and
no failing golden test. So: implement `Debug` manually as `f.write_str(self.as_str())`, and
same for `Display`. §0.5 of the kickoff reasons about serde only; this is the `Debug` twin of
that hazard.

Sites that depend on this staying byte-identical: `crates/index/src/cache.rs:78`,
`crates/scheduler/src/worker.rs:88`, `crates/scheduler/src/engine.rs:143`,
`crates/scheduler/src/shard.rs`, `crates/cli/src/registry.rs:29,91,119,257,262`,
`crates/cli/src/commands/analyze.rs:206`, `crates/cluster/src/plan.rs:153`
(`RejectionReason` derives `Debug`, so `PlanningUnsupported(Cpp)` must still print that way).

### ADAPT-5 — `TuCacheFingerprint.language` becomes `LanguageId` (cache-epoch MUST NOT bump).

The tree forces a change at five sites that pass `plugin.id()` into
`crates/scheduler/src/tu_store.rs:351-357 compute_tu_closure_plugin(..., language: SourceLanguage, ...)`
(`tu_store.rs:252`, `tu_store.rs:464`, `engine.rs:113`, `incremental.rs:89`,
`pipeline/src/lib.rs:804`). `From<LanguageId> for SourceLanguage` is **forbidden** — a third
language has no `SourceLanguage`. The choice is (a) pass `tu.language`, or (b) retarget
`TuCacheFingerprint.language` to `LanguageId`.

**Decision: (b).** `crates/index/src/cache.rs:24-27` documents that field as "owning language,
so one store cannot poison entries across languages". Under (a) the cache namespace key stays
the file-kind label, which D-A5 explicitly rejects, and a Part-C third language would collapse
onto `Other`. (b) is type-correct and preserves that invariant.

**It is byte-safe, which is why `CURRENT_ANALYSIS_VERSION` must stay `"2.0.0"`:**
- JSON is identical — `#[serde(transparent)]` + bare Debug keep `"language": "Cpp"`.
- `#[serde(default)]` (cache.rs:26) requires `LanguageId: Default`; **it must return `Cpp`**
  (D-A3's serde-default half), so v1 `cache.json` files still read back as C++.
- The digest at `cache.rs:78` is `format!("{language:?}")`; with ADAPT-3's bare `Debug` the
  bytes are unchanged, so **every cache fingerprint stays identical** and the epoch does not
  move. Do NOT bump `CURRENT_ANALYSIS_VERSION` — that would invalidate every store.

With (b) the five `plugin.id()` call sites need **zero** edits; only `cache.rs` changes.
Note `crates/index` IS an R2-guarded crate (m17:334-342) and its own tests at
`cache.rs:219,229,248,259,276,296,329-332` currently spell `SourceLanguage::Cpp` (legal via the
exact-string strip). They must become `LanguageId::cpp()` — the `Cpp` spelling is banned there.
`crates/index/src/types.rs:101` is `TuAnalysis.language`, stays `SourceLanguage`, untouched.

### ADAPT-6 — R2 guard sites are three, not one.

`crates/graph/src/builder.rs` has **three** `SourceLanguage::Cpp` sites, not one:
**`:487`** (`fn id`), **`:491`** (`LanguageModeling.language`), **`:540`** (`UnitDescriptor.language`).
All three become `LanguageId` fields, so all three fall under the `contains("Cpp")` ban and all
three must become `ariadne_lang::LanguageId::cpp()`.

### ADAPT-4 — keep PascalCase **value** constants `Cpp` / `Scala` / `Other`, plus guard-legal `cpp()` / `scala()` / `other()` fns.

§2's literal shape exposes only `pub const CPP: &'static str`. That would delete the path
`LanguageId::Cpp`. **But `m17_plugin_seam_test.rs` is frozen and uses `LanguageId::Scala` as
a value at lines 200, 204, 242, 244, 254, 261.** So you MUST keep value constants named
`Cpp`, `Scala`, `Other`.

Keeping them also collapses the diff: ~40 existing `LanguageId::Cpp` sites in
`lang-cpp`, `lang-scala`, `lang`, `cli` compile unchanged. **Prefer that — do not churn
call sites that need no change.**

Additionally the R2 guard (`m17_plugin_seam_test.rs:343-351`) forbids the substring `"Cpp"`
in any non-comment line of `crates/{index,graph,cluster,transform,validate,report,pipeline}/src/**`
(matching is case-SENSITIVE `contains`). So `const CPP` is safe in guarded crates, but the
`Cpp` spelling is not. Add idiomatic snake_case accessor fns so core crates can name a
language without the banned token:

```rust
impl LanguageId {
    pub const CPP:   &'static str = "Cpp";
    pub const SCALA: &'static str = "Scala";
    pub const OTHER: &'static str = "Other";

    #[allow(non_upper_case_globals)] pub const Cpp:   Self = Self(Cow::Borrowed(Self::CPP));
    #[allow(non_upper_case_globals)] pub const Scala: Self = Self(Cow::Borrowed(Self::SCALA));
    #[allow(non_upper_case_globals)] pub const Other: Self = Self(Cow::Borrowed(Self::OTHER));

    pub fn new(name: impl Into<Cow<'static, str>>) -> Self { Self(name.into()) }
    pub fn as_str(&self) -> &str { &self.0 }
    pub fn cpp()   -> Self { Self::Cpp }
    pub fn scala() -> Self { Self::Scala }
    pub fn other() -> Self { Self::Other }
}

impl fmt::Debug for LanguageId { /* f.write_str(self.as_str()) */ }
impl fmt::Display for LanguageId { /* f.write_str(self.as_str()) */ }
impl From<SourceLanguage> for LanguageId { /* Cpp=>Cpp, Scala=>Scala, Other=>Other */ }
impl From<&SourceLanguage> for LanguageId { /* delegates */ }
```

**Known guard site that must change:** `crates/graph/src/builder.rs:486-488` is inside
`crates/graph/src/` (walked by the guard, `#[cfg(test)]` mods included) and currently returns
`SourceLanguage::Cpp`, legal only because `GUARD_NEUTRAL_VOCAB` strips that exact string
(`m17_plugin_seam_test.rs:353`). After this task it must return a `LanguageId` **without
spelling `Cpp`** — use `ariadne_lang::LanguageId::cpp()` (or `LanguageId::new(LanguageId::CPP)`).

---

## 4. Touch-set (hard; anything outside = blocker, report it, do not fix)

Expected in-set (verify with `git status` before committing; the compiler will tell you the
real list):

- `crates/lang-api/src/identity.rs` — the core change.
- `crates/lang-api/src/modeling.rs`, `crates/lang-api/src/unit.rs` — only if a construction
  site or the round-trip test needs adjusting.
- `crates/index/src/cache.rs` — **ADAPT-5**: `TuCacheFingerprint.language` and
  `TuCacheFingerprint::compute`'s `language` parameter become `LanguageId`; its tests move to
  `LanguageId::cpp()`. Do NOT touch `crates/index/src/types.rs` (`TuAnalysis.language` stays
  `SourceLanguage`).
- `crates/scheduler/src/tu_store.rs` — **ADAPT-5**: only the
  `compute_tu_closure_plugin` `language: SourceLanguage` parameter (line 357) becomes
  `LanguageId`. Its five `plugin.id()` call sites stay byte-for-byte unchanged.
- `crates/lang/src/registry.rs` — `plugin_for_id`/`languages` signatures + their tests.
- `crates/lang-cpp/src/{plugin,shaper}.rs`, `crates/lang-scala/src/{plugin,shaper}.rs` —
  only where a value now needs `.clone()` or a `SourceLanguage`→`LanguageId` conversion.
- `crates/graph/src/builder.rs` — the THREE guard sites above (ADAPT-6).
- `crates/cluster/src/{plan,planner}.rs`, `crates/cli/src/registry.rs`,
  `crates/cli/src/commands/analyze.rs` — only where the type split forces it.
- `tasks/m24a-t1-handoff.md`, `tasks/m24a-t1-journal.md` (journal always in-set).

**NEVER touch:** `crates/repo/**`, `tests/golden/**`, `tests/fixtures/**`,
`tests/integration/tests/**`, `Cargo.toml`, `Cargo.lock`,
`crates/{lang-cpp,lang-scala}/**` beyond compile-forced edits, `crates/cli/src/optimize.rs`,
`crates/cli/src/commands/ui.rs` (Part D — do not touch; if the compiler seems to demand it,
STOP and report).

**No new dependency. `Cargo.lock` diff must be empty.**

---

## 5. Red — author these tests FIRST, watch them fail, then implement

Add to `crates/lang-api/src/identity.rs` (`mod tests`). All four must exist and fail before
you write the newtype.

1. `language_id_can_name_a_language_that_is_not_cpp_or_scala`
   `LanguageId::new("Python")` constructs; `as_str() == "Python"`; it is not equal to any of
   the three built-ins. **Today: does not compile** (`LanguageId::new` does not exist).
2. `language_id_round_trips_the_three_legacy_names_through_json`
   For each of `Cpp`/`Scala`/`Other`: `serde_json::to_string(&id) == "\"Cpp\""` (etc.) and
   `serde_json::from_str::<LanguageId>` returns it. This is the §0.5 artifact-compatibility
   claim, proven rather than assumed — **the kickoff explicitly demands this test.**
3. `language_id_debug_and_display_are_the_bare_name`
   `format!("{:?}", LanguageId::Cpp) == "Cpp"`,
   `format!("{:?}", LanguageId::new("Python")) == "Python"`,
   `format!("{}", LanguageId::Scala) == "Scala"`. **This is the ADAPT-3 cache-digest lock.**
   Also assert `format!("{language:?}", language = LanguageId::Cpp) == "Cpp"` using the exact
   `format!` shape from `crates/index/src/cache.rs:78`.
4. `legacy_source_language_converts_into_a_language_id`
   `LanguageId::from(SourceLanguage::Cpp) == LanguageId::Cpp` for all three variants, and the
   reverse direction is documented as intentionally absent (`SourceLanguage` is a file-kind
   label; it cannot name a third language).

Also keep green (do not delete): the existing `default_language_is_cpp_for_legacy_artifacts`
intent must survive in whatever form the newtype allows — `SourceLanguage::default()` is
still `Cpp` (untouched), and if you give `LanguageId` a `Default`, it must be `Cpp`.

---

## 6. Green — implementation bounds

1. Make `LanguageId` the newtype per §3 above, in `crates/lang-api/src/identity.rs`.
2. Delete the now-false module doc claim at `identity.rs:1-2` ("the neutral `LanguageId` is
   the **existing** `ariadne_repo::SourceLanguage` enum") and replace it with the real
   post-change statement: `SourceLanguage` is the closed **file-kind** label;
   `LanguageId` is the open **plugin** identity.
3. `language_of_file_kind(kind: FileKind) -> LanguageId` keeps its signature; its body becomes
   `SourceLanguage::from_file_kind(kind).into()`.
4. **Non-`Copy`:** `LanguageId` is `Clone`, not `Copy`. Every site that relied on `Copy`
   needs `.clone()` or a borrow. Audit at minimum: `LanguageRegistry::plugin_for_id`,
   `languages()`, `crates/cli/src/registry.rs`, the `lang-cpp`/`lang-scala` `id()` bodies,
   and the `#[cfg(test)]` modules of `crates/lang/src/registry.rs`.
   Consider `plugin_for_id(&self, id: &LanguageId)` if by-value causes wide churn — but if you
   change its signature, `m17_plugin_seam_test.rs:244` (`plugin_for_id(LanguageId::Scala)`)
   would fail to compile, and it is FROZEN. **So: keep `plugin_for_id(id: LanguageId)` by
   value.** Const arguments work fine.
5. Derives required: `Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize`,
   `#[serde(transparent)]`. **Do NOT derive `Debug` or `Display`** (ADAPT-3). Do not derive
   `Copy` or `Default` unless a site forces it; if `Default` is added it must be `Cpp`.
6. Fix every compile error the compiler reports. That list IS the discovery mechanism
   (D-A4) — record it in your journal; T3 consumes it.

### Deliberately NOT in this task
- Do **not** touch `detect_language`'s `FileKind::Other` veto (`registry.rs:109-114`) — that
  is T2. Do not touch `crates/cli/src/commands/plan.rs` — that is T3. Do not repoint
  `catalog_languages` — that is T4.

### ADAPT-7 (sequencing) — D-A4's compile-error discovery fires in T4, not T1.

The kickoff's §3 asserts "D-A4 makes `plan.rs:70` and `plan.rs:78` compile errors". Under
ADAPT-1 that is **false as stated for T1**: `SourceLanguage` stays a closed enum, so
`plan.rs:70-81` still matches on `SourceLanguage` and survives T1 untouched. The silent
`_ => Cpp` misroute is therefore NOT fixed by T1.

It IS fixed, and the compiler DOES surface it — but only when the **CLI dispatch data**
changes type, which is `catalog_languages`/`language_of_path` yielding `LanguageId` instead of
`SourceLanguage` (the old T4). At that instant `plan.rs:70` and `plan.rs:78` become genuine
non-exhaustive/type errors.

**Therefore the task order is corrected: old-T4 (registry-authoritative dispatch) runs BEFORE
old-T3 (the CLI match audit), and T3's red is re-specified as "the compiler errors emitted by
the catalog type change", not "by opening `LanguageId`".** The end state is identical to the
kickoff's; only the ordering of the red is corrected. Recorded as **D-A8** in the closeout.

---

## 7. Refactor / hygiene

- Files < 1000 lines, functions < 100 lines, intention-revealing names. No `tmp`/`res`/`data`.
- Centralize construction: no raw string literals at call sites — use the consts or `new`.
- Run `python tools/hygiene_check.py` → must exit 0.

### Gate commands (substituted — the distilling tools do not exist on this branch)

```powershell
# build, distilled (never bare):
cargo build --workspace 2>&1 | Out-File -Encoding utf8 .m24a\t1-build.txt
# test, distilled:
cargo test --workspace --no-fail-fast 2>&1 | Out-File -Encoding utf8 .m24a\t1-test.txt
# then summarise (this is the distillation step):
Select-String -Path .m24a\t1-test.txt -Pattern '^test result:' |
  ForEach-Object { $_.Line }
# total the counts:
$r = Select-String -Path .m24a\t1-test.txt -Pattern '^test result:' |
  ForEach-Object { $_.Line -replace '.*?ok\. (\d+) passed; (\d+) failed; (\d+) ignored.*','$1 $2 $3' }
# hygiene
python tools/hygiene_check.py
# golden + lock proofs
git diff -- tests/golden        # MUST be empty
git diff -- Cargo.lock          # MUST be empty
# frozen-test proof
git diff --name-only -- tests/integration/tests   # MUST be empty
```

**File-edit rule:** use the Write/Edit tools only. Never `Set-Content`, `Out-File`, `>`, or
`Get-Content -Raw | -replace` on tracked files — PowerShell 5.1 corrupts UTF-8 that way.
Console mojibake on UTF-8 output is a **display artifact, not corruption**; do not "fix" it.

**Commit:** scoped `git add <explicit paths>` only. **Never `git add .`.** Do not use
`commit_scoped.py` (absent). Stage only `crates/**` and `tasks/m24a-t1-*`. Do NOT stage
`.m24a/`. Commit message must carry `Landed:` provenance and record the ADAPT-1..4 reasons.

---

## 8. Acceptance for T1 (all must hold before you report done)

- [ ] `cargo build --workspace` exit 0.
- [ ] `cargo test --workspace --no-fail-fast` exit 0, **passed >= 1088, failed 0, ignored 3**.
- [ ] `git diff -- tests/golden` **empty**.
- [ ] `git diff -- Cargo.lock` **empty**.
- [ ] `git diff --name-only -- tests/integration/tests crates/repo` **empty**.
- [ ] `python tools/hygiene_check.py` exit 0.
- [ ] The 4 red tests from §5 exist and pass.
- [ ] `crates/repo` untouched.

## 9. Report back (digest only — never paste raw tool output)

- commit hash + `git show --stat` summary.
- **Red evidence:** which of the 4 tests failed and how (compile error vs assertion).
- **Green evidence:** one canonical line (e.g. `TOTAL passed=1092 failed=0 ignored=3`) + exit code.
- **Compiler-error census:** the list of sites the type split broke and how each was resolved
  (T3 needs this).
- Deviations from this handoff + reason.
- Any place the kickoff's §2 shape had to bend further than ADAPT-1..4.