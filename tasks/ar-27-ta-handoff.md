# AR-27a Handoff - LanguageId home move + dual-format serde + byte oracles (A2, hop 1/2)

## DISPATCH ORDER - AR-27 re-shape (read first)

Two SERIAL hops. T-a lands (reviewer APPROVED) before T-b dispatches; T-b's
RED depends on T-a's landed closed-set `parse_language_arg` and on the escape
arm existing. The hops are NOT parallel-safe: both touch
`crates/repo/src/types.rs`, `crates/repo/src/identity.rs`,
`crates/lang-api/src/identity.rs`, and `crates/cli/src/registry.rs`.
T-a is a STRICT COMPILE-GREEN hop: it must run as the SOLE cargo-heavy lane
(ruling section 5, dispatch discipline) - no other lane may hold in-flight
edits in `repo`/`lang-api`/`cli` while T-a mutates.

| hop | file | scope | produced lines | gates |
|---|---|---|---|---|
| 1 (this file) | `tasks/ar-27-ta-handoff.md` | byte-oracle fixtures + home move + alias + dual-format serde + string-frame exception + closed parse + clone riders | ~450-520 | 10 |
| 2 | `tasks/ar-27-tb-handoff.md` | registry-derived `--lang` + FileKind role refactor + third-language E2E | ~300-380 | 9 |

Superseded origin: `tasks/ar-27-handoff.md` (bannered). STOP evidence and
successor notes: `tasks/ar-27-handoff-journal.md` (sections 2, 5, 7). The
binding shape is `tasks/ar-27-shape-ruling.md` (Option B as amended) -
this contract embeds its sections 2, 3, 4, 7 VERBATIM below; do not improvise
around them.

Binding inputs (read in this order, targeted reads only):
1. This file (complete contract).
2. `tasks/ar-27-shape-ruling.md` sections 1-5 (authority).
3. `tasks/ar-27-handoff-journal.md` sections 2 and 5 (pre-change baselines;
   the measured old-bytes evidence whose values your fixtures regenerate).
4. `crates/lang-api/src/identity.rs` (the type you move: newtype :42-44,
   consts :14-20/:54-63, `cpp()`/`scala()`/`other()` :75-93, Default :99-103,
   Debug/Display :105-115, value From :117-125, `&`-From :127-131, unit
   tests :144-249).
5. `crates/repo/src/types.rs` (the enum being replaced: :141-159;
   `FileRecord.language` :192; FileKind :31-72 UNCHANGED this hop),
   `crates/repo/src/lib.rs` (:9-12 re-export list).
6. `crates/lang-api/src/types.rs` (`field_conditional_on_format` :13-22,
   `TuAnalysis` :198-239, `IncludeEdge.language` :302-308,
   `include_kind_bincode_indices_are_pinned` precedent).
7. `crates/index/src/cache.rs` (epoch pin :30 + pin test ~:398-403,
   `context_hash_of` :92-112, `TuCacheFingerprint::compute` :116-126 -
   `.context_hash` is the digest seam your fixture test calls).
8. `crates/build/src/types.rs` (`CompilationContext` :6-22,
   `TranslationUnit` :71-81 - ADAPT-8 frozen, read-only).
9. `crates/cli/src/registry.rs` (`parse_language_arg` :37-54,
   `ensure_language_registered` :24-32, clone site :186-196).
10. `crates/pipeline/src/stage_index.rs` :102-119 (named rider).

## Objective

Give `LanguageId` a dual-format serde (positional = pinned u32 tag + string
escape; human-readable = transparent bare name), move the newtype's home to
`crates/repo`, and make `pub type SourceLanguage = LanguageId;` - while
moving ZERO committed bytes and editing ZERO authority locks. Invariants this
hop must preserve:
- Every old-enum-era bincode warm blob loads at epoch `2.2.0` (no bump -
  epoch rule: a bump is a user decision event; this ruling requires none).
- `SHA256(bincode(ctx))` context_hash byte-identical (ADAPT-8).
- Every JSON surface byte-identical for every id, including open ids.
- `includes/edges.bin` warm bytes (string-framed since M24 Part A)
  unchanged via the field-level exception module.
- All authority locks UNEDITED and green (Baselines); workspace STRICT
  COMPILE-GREEN at every commit.

## Baselines (dispatch-HEAD; re-measure and journal BEFORE any mutation -
contracts must not quote stale numbers, and the dead AR-27 session's numbers
are banked in its journal section 2)

- Census anchor `1400/0/13/105` (passed/failed/ignored/suites; ruling
  correction vii - the "1400/0/0/13-ignored/105" spelling is WRONG). The
  workspace census is the REVIEWER's acceptance gate (Planning Law), NOT an
  implementer gate; do not re-measure it.
- Authority locks (ruling section 1; ALL UNEDITED green at dispatch, journal
  section 2; pin the split): m23 bincode lock file
  `crates/lang-api/tests/m23_bincode_serde_locks.rs` `5/0/0/1` vs
  m23 symbol-isolation `m23_symbol_isolation_test` `7/0/0/1`;
  m00 golden + facts-path `4/0/0/2`; m24 identity `8/0/0/1`;
  m24a Scala plan path `2/0/0/1`; m18 cpp plugin parity `23/0/0/1`;
  ui `272/0/0/3`; repo `15/0/0/2` (MEASURED at journal dispatch - never
  pinned before); cli `118/0/8/7`; lang-api `41/0/0/4`.
- `python tools/distill.py -- python tools/ui_smoke.py` -> `ui_smoke: PASS`
  REQUIRED both hops (payload serde surfaces).
- Rider-crate baselines measured and journalled PRE-mutation (ruling
  section 4): `-p ariadne-pipeline`, `-p ariadne-worker-core`. Their suites
  stay UNEDITED at their dispatch-measured counts; reconciled counts mean
  pre-count + journal-recorded test delta.
- `python tools/hygiene_check.py` ZERO blocking rows (never quote an R3
  count); `encoding_guard --all` clean.

## Touch-set (hard; any edit outside = blocker, report don't fix)

- `tasks/ar-27-ta-handoff-journal.md` (checkpoint journal; always in-set)
- `crates/repo/src/identity.rs` (NEW - the newtype's home: move the type,
  consts, accessors, Default/Debug/Display, `from_file_kind` (from
  repo/types.rs :150-158), the newtype unit tests; ADD custom
  `PartialOrd`/`Ord`, dual-format `Serialize`/`Deserialize` impls, private
  `WireLanguageForm`, and the `language_transparent_frame` module - all
  per the ruling sections quoted below)
- `crates/repo/src/types.rs` (DELETE enum `SourceLanguage` :141-159;
  ADD `pub type SourceLanguage = LanguageId;` + `pub use` per ruling
  section 3; FileKind :31-72 UNTOUCHED this hop)
- `crates/repo/src/lib.rs` (+`pub mod identity;`; extend the `pub use
  types::{...}` / new re-exports so `ariadne_repo::SourceLanguage`,
  `ariadne_repo::LanguageId` (alias-visible), and
  `ariadne_lang_api::identity::LanguageId` all keep resolving - ruling
  section 3)
- `crates/repo/Cargo.toml` (+1 `[dev-dependencies]` line
  `serde_json.workspace = true` - the MOVED unit test at identity.rs:160
  uses serde_json; repo has no tests today so the dev-dep is new; journal it)
- `crates/lang-api/src/identity.rs` (becomes the re-export shim + KEEPS
  `language_of_file_kind` :133-141 and its exhaustive unit test :219-236;
  the newtype unit tests MOVE to repo/src/identity.rs - conservation
  equation in the journal: `lang-api unit delta = -moved tests`,
  `repo unit delta = +moved tests`)
- `crates/lang-api/src/types.rs` (`IncludeEdge.language` :302-308: attach
  `#[serde(with = "...language_transparent_frame")]` KEEPING
  `#[serde(default)]`, doc-comment updated; NOTHING else - `TuAnalysis`
  field types stay `SourceLanguage`, which now resolves through the alias
  UNEDITED)
- `crates/cli/src/registry.rs` (`parse_language_arg` :37-54 closed-set
  rework per step 4; clone at :192 per ruling section 4; the `{language:?}`
  bail wording :29 stays byte-stable - Debug is already the bare name)
- `crates/pipeline/src/stage_index.rs` (BLESSED clone-rider, copy-semantics
  `.clone()` insertions ONLY - ruling section 4; `:110` moving
  `self.language_filter` by value out of borrowed `self` is the expected
  E0507 site)
- `crates/worker-core/src/tu_store.rs` (expected NO-OP: `:210` assigns from
  an OWNED local (`file_analysis.language`) - partial move of an owned
  local is legal Rust; touch ONLY if the compiler says otherwise, then
  clone-only)
- NEW test files only (existing files in these dirs are byte-frozen):
  - `crates/repo/tests/ar27_old_bytes_lock_test.rs`
  - `crates/lang-api/tests/ar27_dual_format_lock_test.rs`
  - `crates/index/tests/ar27_context_bytes_lock_test.rs`
  - `tests/integration/tests/ar27_field_probe_test.rs`

You MUST NOT touch: `crates/build/**`, `crates/index/src/**` (the index test
is NEW-FILE-ONLY; `cache.rs` UNEDITED including the epoch pin :30, its pin
test, and the digest locks), `crates/artifact/**` (its `metrics_io.rs:241`
compiles ONLY via the kept `From<&SourceLanguage>` - do not "help" it),
`crates/report/**`, `crates/ui/**`, `crates/lang/**`, `crates/lang-scala/**`,
`crates/worker-core/**` (unless the no-op turns out false - clone-only),
EVERY existing file under `tests/**` and `crates/*/tests/**` (suites and
goldens UNEDITED), `tasks/ar-27-handoff.md`, `tasks/ar-27-shape-ruling.md`,
`tasks/ar-27-handoff-journal.md`. No `git stash`, no branch change.

## The positional encoding contract (ruling section 2, VERBATIM - binding)

> Workspace bincode is bincode 1.3.3 (Cargo.lock:488-489) with DEFAULT options
> (fixint, little-endian; enum variant = u32 LE; String = u64 LE length + UTF-8
> bytes). The `is_human_readable()` discriminator is valid here and is already
> the in-tree convention (`field_conditional_on_format`, lang-api/src/types.rs
> :13-22): the m23 lock family passes today ONLY because bincode reports false
> (otherwise `TuAnalysis::default()` would omit empty `extends_edges` from
> bincode and `bincode_round_trips_non_empty_analysis_vectors` would fail).
>
> POSITIONAL (non-human-readable) arm of `LanguageId`:
>
> 1. Shape is driven by a private (never `pub`) helper
>    `#[derive(Serialize, Deserialize)] enum WireLanguageForm { Cpp, Scala, Other, Open(String) }`
>    so serde's enum encoding does the framing. Arm order is FROZEN FOREVER:
>    Cpp=0, Scala=1, Other=2, Open=3. No reordering, no variant inserted before
>    `Open`, ever.
> 2. Serialize: id "Cpp" -> `00 00 00 00` (u32 LE 0); "Scala" -> `01 00 00 00`;
>    "Other" -> `02 00 00 00`; every OTHER id -> `03 00 00 00` (u32 LE 3) then a
>    bincode String (u64 LE byte length + UTF-8 bytes). Legacy ids are ALWAYS
>    written as their index, even by the new code (mixed-version warm caches stay
>    readable both ways for legacy values).
> 3. Deserialize (positional): read u32 tag; 0 -> "Cpp", 1 -> "Scala", 2 ->
>    "Other", 3 -> read String -> `LanguageId::new`; any other tag -> serde
>    invalid-value error (no panic).
> 4. Human-readable arm: `#[serde(transparent)]` bare-name string for ALL ids,
>    unchanged from today. An open id stays `"Fortran"` in JSON - there is NO
>    externally-tagged `{"Open":...}` shape in any JSON.
> 5. Collision proof: `SourceLanguage` today has exactly three variants
>    (`crates/repo/src/types.rs:142-147`: Cpp, Scala, Other), so the legacy tag
>    set is {0,1,2}; tag 3 is unreachable in old bytes (journal section 5
>    measured Cpp=[0,0,0,0], Scala=[1,0,0,0], Other=[2,0,0,0]).
> 6. Round-trip targets: old enum-era blobs (u32 index) load; new escape blobs
>    load; JSON identical pre/post for every id.
>
> STRING-FRAME EXCEPTION (field-level, not global):
>
> 7. `IncludeEdge.language` (lang-api/src/types.rs:308) is persisted in
>    `includes/edges.bin` (`crates/artifact/src/manager/graph_io.rs:35` as
>    `IncludeGraph.edges: Vec<IncludeEdge>`, graph.rs:34) and its warm bytes are
>    transparent STRING-FRAMED since M24 Part A at the CURRENT epoch (2.2.0
>    bump, DP-2). The global positional arm would move those bytes -> old blobs
>    hard-error at load. Therefore `IncludeEdge.language` gets
>    `#[serde(with = "...")]` (module `language_transparent_frame`, lives in
>    `ariadne-repo` next to the newtype) that reproduces today's transparent
>    behavior in BOTH formats byte-for-byte (serialize: bare name string;
>    deserialize: String -> `LanguageId::new`), keeping `#[serde(default)]`.
> 8. T-a probe lock: a test enumerates every `Serialize` struct with a
>    `LanguageId`-typed field that is bincode-persisted; the asserted set is
>    EXACTLY {IncludeEdge.language}. Verified clean during this ruling:
>    TuUsage (reduced.rs:72, no language), SemanticEdge, SymbolRecord, CallSite,
>    OptimizationPlan (plans.bin; only PlanningGap carries a language and has no
>    serde derive), LoadReport/report.bin ("carries no language data",
>    metrics_io.rs:214), MigrationPreview (placement.rs:50), tu-usage / file-
>    symbols / extends / sealed / callgraph / closures bins, TuCacheFingerprint
>    (JSON-only cache.json), LanguageModeling (JSON-only). A probe surprise =
>    STOP-AND-REPORT (second exception field, same module, ruling addendum).

Byte-freeze table vs HEAD (ALL zero drift): `tu_analyses.bin`, repository
`files.bin` (FileKind indices untouched this hop; language stays u32 index),
`contexts/translation-units.bin`, `SHA256(bincode(ctx))` context_hash
(ADAPT-8), `includes/edges.bin` (exception module), every JSON surface.

## Type-level rulings (ruling section 3, VERBATIM - binding)

> - Newtype home MOVES to `crates/repo` (new module `crates/repo/src/identity.rs`)
>   because `lang-api` already depends on `repo` (identity.rs:7) and the m24 gate
>   forces the alias direction; `crates/repo/src/types.rs` gains
>   `pub type SourceLanguage = LanguageId;` and `pub use` for the newtype;
>   `crates/lang-api/src/identity.rs` becomes a re-export shim + keeps
>   `language_of_file_kind` + moved unit tests. All existing import paths must
>   keep resolving (`ariadne_repo::SourceLanguage`, `ariadne_lang_api::identity::LanguageId`,
>   `ariadne_lang::LanguageId`). The R2 guard walk list is the 8 core crates
>   (m17_plugin_seam_test.rs:335-344) - `repo` and `lang-api` are OUTSIDE it, so
>   the `Cpp` literals in the newtype are legal in their new home.
> - `Default` = Cpp (already true, cache.rs:48 JSON `serde(default)` depends on
>   it). Hand-written Debug/Display stay the bare name (fingerprint digest
>   identity.rs:36-39; the `{language:?}` error wording at cli/registry.rs:29
>   stays byte-stable because enum Debug and the newtype Debug are both "Cpp").
> - Ord is CUSTOM, not derived: legacy ids order by legacy variant index
>   (Cpp < Scala < Other), preserving the old enum's derived-Ord semantics that
>   `build_load_manifest` dedupe+sort consumes (`artifact/manager/metrics_io.rs:
>   241` + `collect_stage_languages`) and report grouping; open ids sort AFTER
>   all legacy ids, lexicographically among themselves. Derived lexicographic Ord
>   would flip Scala-vs-Other sorts (byte drift risk in manifest/report bytes).
>   No UNEDITED gate asserts LanguageId ordering (m24 grep-verified); the
>   identity test `Cpp < python` stays true.
> - Coherence: DELETE `impl From<SourceLanguage> for LanguageId`
>   (identity.rs:117-125) - under the alias it is `From<T> for T` -> E0119; the
>   std reflexive blanket covers every value site (registry.rs:28,
>   stage_index.rs:213, m17:78/92, m19:115, m22:93/101). KEEP
>   `impl From<&SourceLanguage> for LanguageId` (identity.rs:127-131 - legal as
>   From<&T> for T): `artifact/manager/metrics_io.rs:241` calls
>   `LanguageId::from(&SourceLanguage)` over an iterator and compiles UNEDITED
>   ONLY because of this kept impl.
> - No `Copy` (Cow inner cannot be Copy). Copy loss is handled by the rider
>   protocol below - NOT by signature surgery.

Implementer notes for the coherence edits (mechanical consequences, not
redesign): after the alias, the kept `&`-From body's `match source {
SourceLanguage::Cpp => ... }` is no longer valid syntax (struct associated
constants are not patterns) - reduce the body to `source.clone()`. The moved
From unit test (`identity.rs:197-209`) compiles and passes UNCHANGED through
the std reflexive blanket plus the kept `&`-impl. The `#[derive]` line loses
`Serialize, Deserialize` and `PartialOrd, Ord` (hand-written); keeps
`Clone, PartialEq, Eq, Hash`.

## Clone-rider protocol (ruling section 4, VERBATIM - binding)

> The journal missed one forced out-of-set consumer:
> `crates/pipeline/src/stage_index.rs` reads `self.language_filter:
> Option<SourceLanguage>` by value from borrowed `self` (:103/:110) - legal
> today only via the enum's `Copy`.
>
> - `crates/pipeline/src/stage_index.rs`: BLESSED for compiler-forced
>   clone-insertions only (no logic edits). Suite UNEDITED at its dispatch-
>   measured baseline (successor re-brief must pin `-p ariadne-pipeline` counts
>   pre-mutation).
> - `crates/cli/src/registry.rs:191`: in-set clone, trivial.
> - `crates/worker-core/src/tu_store.rs:210`: expected NO-OP (partial move of an
>   owned local is legal Rust); blessed as clone-rider only if the compiler says
>   otherwise; same measured-baseline discipline (`-p ariadne-worker-core`).
> - General rule: any other out-of-set file needing a pure copy-semantics
>   `.clone()` insertion is pre-blessed up to 6 sites TOTAL out-of-set; any
>   out-of-set edit that is not a copy-semantics clone => STOP-AND-REPORT.
>   (No new lock obligations beyond those two rider suites' exact counts.)

## RGB protocol (hard ordering; step 1 is the load-bearing discipline)

RED-FREE GREEN-FIRST (AR-08/AR-31 precedent, ruling section 5 step 1): the
byte-oracle fixtures are regenerated from the journal's deleted
`target/ar27-evidence-*.txt` evidence and COMMITTED GREEN AGAINST THE
UNTOUCHED TYPES, in their own first commit, BEFORE any type move. The same
assertions must stay green post-mutation - they are the oracle for every
byte claim in this contract. An RGB step whose "red" is a compile error is
not Red; these locks are characterization locks, and weakening an assertion
to pass is a reviewer blocker.

1. Bootstrap + journal: dispatch HEAD, all Baselines lines, rider-crate
   pre-mutation counts.
2. ORACLE COMMIT (types untouched): author the four new test files with
   inline hex fixtures. Authoring recipe: write a temporary
   `#[ignore]`d printer test once against the untouched types to print the
   bytes (or transcribe journal section 5's values and re-derive by running
   the real types), paste inline hex into the lock tests, delete the
   printer, run the four suites GREEN, commit via commit_scoped. Content:
   - repo file: `Vec<FileRecord>` bincode (empty; two records covering
     `#[serde(default)]`-absent and explicit `Scala`), FileRecord JSON,
     the three ids' JSON.
   - lang-api file: `Vec<TuAnalysis>` bincode (empty-extends,
     non-empty-extends, and Scala-language variants per ruling step 1),
     the three ids' JSON. Post-mutation additions land with the
     implementation commit: the escape-arm positional bytes
     (`03 00 00 00` + u64 LE len + UTF-8 for `Fortran`), the open-id JSON
     bare-name lock, and the DISCRIMINATOR PIN TEST (ruling section 8:
     bincode `is_human_readable() == false`, serde_json `true`).
   - index file: `CompilationContext` bincode bytes + digest hex - load
     the fixture bytes into `CompilationContext`, assert
     `bincode::serialize(&ctx)` equals the bytes and
     `TuCacheFingerprint::compute(...).context_hash` equals the pinned hex
     (digest seam; `context_hash_of` itself is private).
   - integration probe file: `Vec<IncludeEdge>` bincode (STRING-FRAMED -
     today's warm shape) + JSON; the ruling section 2.8 declared-enumeration
     lock as an inline asserted set EXACTLY {IncludeEdge.language} with
     the ruling's clean-verified list cited in a comment. A probe surprise
     = STOP-AND-REPORT (board + journal; second exception field = same
     module, ruling addendum).
3. Green: HOME MOVE (repo identity.rs + alias + lib.rs exports +
   lang-api shim + moved unit tests) - workspace compiles with the enum
   still serde-derived? NO: steps 3-4 land as one compile-green unit (the
   alias and the dual-format impls are mutually forced). Land 3+4+5
   together, STRICT COMPILE-GREEN at the commit.
4. Dual-format serde + `language_transparent_frame` + `IncludeEdge`
   attachment + custom Ord, per the quoted ruling sections.
5. `parse_language_arg` KEEPS today's closed behavior byte-for-byte, Err
   wording included (`"unknown language '{raw}'; expected cpp, scala, or
   other (serde names: Cpp, Scala, Other)"`). Mechanism: the existing
   normalization loop stays, but the serde-based acceptance is REPLACED by
   an explicit match against the three legacy ids (the transparent JSON
   reader now accepts any string - the closed set must be explicit). The
   existing cli lock `parse_language_arg("Fortran") -> Err` MUST still hold
   after T-a (T-b's RED depends on it).
6. Riders per the quoted section 4.
7. Full gate set; implementation+journal commit.

## Hygiene (review-enforced)

Functions < 100 lines, files < 1000 lines, no banned names, honest doc
comments on the dual-format impls (state WHERE the tag order is frozen and
WHY - a reordered arm silently corrupts every warm cache). `repo/src/identity.rs`
absorbs the newtype (~250 lines incl. moved tests) - well under R2.
`python tools/hygiene_check.py` exit 0, ZERO blocking rows.

## Frozen locks (LOCK AMENDMENTS: NONE - ruling section 7, VERBATIM)

> ZERO. This ruling sanctions no edit to any tracked golden, lock, or byte-
> authority file. The FileKind serde aliases touch no committed bytes (journal
> grep proof; T-b re-proves including binary fixtures). The epoch pin stays
> 2.2.0. `m24:549`'s stale comment stays unedited (journal section 7).

Machine checks (quote outputs in the journal):
- `python tools/distill.py -- git diff <dispatch-HEAD> -- tests` EMPTY except
  the ONE new file under `tests/integration/tests/` (new files are additions,
  not edits: also run `git status --porcelain -- tests` and confirm only
  `?? tasks/...`-style new-file lines exist for your own probe file).
- `python tools/distill.py -- git status --porcelain -- tests/golden` EMPTY.
- Epoch pin + `context_hash` digest locks: `crates/index/src/cache.rs` is
  outside the touch-set - covered by the `git diff` check above plus gate 4.
- Until `golden_guard.py` lands: manual byte-identical checks of the above -
  do NOT touch these paths.

## Gates (exact commands, timeout 600000 ms each; scoped only - workspace-wide
final gates belong to the reviewer's acceptance pass, including the
`1400/0/13/105`-anchor census)

1. `python tools/distill.py -- cargo check --workspace --all-targets`
   (baseline journal BEFORE mutation; then STRICT COMPILE-GREEN at every
   commit).
2. `python tools/testrun.py --distill -p ariadne-repo` - baseline `15/0/0/2`
   -> `15+D/0/0/3` with conservation equation (unit delta = +moved tests,
   +D fixture tests; suites +1 new file).
3. `python tools/testrun.py --distill -p ariadne-lang-api` - baseline
   `41/0/0/4` -> `41-D/0/0/5` (moved units OUT, +D dual-format lock tests,
   +1 suite; m23-bincode `5/0/0/1` UNEDITED inside it).
4. `python tools/testrun.py --distill -p ariadne-index` - pre-existing digest
   locks UNEDITED; +1 suite (probe fixture green pre and post).
5. `python tools/testrun.py --distill -p ariadne-cli` - `118/0/8/7` EXACT,
   suite UNEDITED (closed-wording locks live here).
6. `python tools/testrun.py --distill -p ariadne-integration-tests --test
   m23_symbol_isolation_test --test m00_golden_baseline_test --test
   m00_facts_path_parity_test --test ar27_field_probe_test` ->
   `7/0/0/1` + `4/0/0/2` UNEDITED green + your probe suite (resolve the m24
   `8/0/0/1`, m24a `2/0/0/1`, m18 `23/0/0/1` suite names by globbing
   `tests/integration/tests/` and run them UNEDITED green in the same or a
   second invocation - journal the exact suite names you ran).
7. `python tools/testrun.py --distill -p ariadne-ui` - `272/0/0/3` UNEDITED
   (payload serde surfaces) AND
   `python tools/distill.py -- python tools/ui_smoke.py` -> `ui_smoke: PASS`
   (REQUIRED this hop).
8. `python tools/hygiene_check.py` - exit 0, zero blocking rows.
9. `python tools/encoding_guard.py --staged` - exit 0 before each commit.
10. `python tools/encoding_guard.py --all` - exit 0, journal the run.

Commits (both via commit_scoped, never `git add .`):
- Oracle commit:
  `python tools/commit_scoped.py --scope "crates/repo/tests/**" --scope "crates/lang-api/tests/ar27_dual_format_lock_test.rs" --scope "crates/index/tests/**" --scope "tests/integration/tests/ar27_field_probe_test.rs" --scope "crates/repo/Cargo.toml" --scope "tasks/ar-27-ta-handoff-journal.md" -m "test(ar27): byte-oracle fixtures green PRE-mutation (old-bytes locks, home of AR-27a)"`
- Implementation commit:
  `python tools/commit_scoped.py --scope "crates/repo/**" --scope "crates/lang-api/src/**" --scope "crates/cli/src/registry.rs" --scope "crates/pipeline/src/stage_index.rs" --scope "crates/lang-api/tests/ar27_dual_format_lock_test.rs" --scope "tasks/ar-27-ta-handoff-journal.md" -m "refactor(repo,lang-api,cli): LanguageId home move + dual-format serde; SourceLanguage alias; clone riders (AR-27a)"`
  (add `--scope "crates/worker-core/src/tu_store.rs"` ONLY if the no-op
  turned out false; journal why).

## Rename-completeness proof clause (AR-20a D7 - conditional)

No `git mv` is planned (the move is delete-old-impl/new-file). IF you use any
`git mv` for any path: immediately after the commit run, via distiller,
`git show --stat -M HEAD` + `git ls-files <old path>`; the stat must list old
path deleted / new path added (or R100 rename) with no other paths;
`git ls-files` must return NOTHING for the old path. A stray staged delete
falls out of scoped commits - land an immediate second scoped commit for the
stray deletion ONLY and journal it.

## Discipline clauses

- UTF-8: never Set-Content/Out-File/>/Get-Content-replace; Write/Edit tools
  only; authored content pure ASCII (AGENTS.md File-Edit Discipline).
- Touch only the touch-set; any out-of-set edit that is not a blessed
  copy-semantics clone (max 6 out-of-set sites, ruling section 4) =
  STOP-AND-REPORT. Probe surprise = STOP-AND-REPORT. Any contemplated epoch
  bump = NEVER self-authorize (DP event; ruling section 0 contingency:
  STOP, then a ruling).
- No raw high-volume commands in your context: everything through the
  distilling entry points (`docs/specs/tool-output-distillation.md`); quote
  canonical summary lines from OWN stdout, never from shared capture files
  (capture-race ruling).
- Refused-command substitutions on this lane: raw `git status`/`git
  rev-parse`/`python -c`/chained-echo are allowlist-refused; substitute
  `python tools/distill.py -- git status --porcelain ...` /
  `-- git rev-parse HEAD`, Edit-tool for text transforms, and adjudicate
  exit codes from printed contracts (journal AR-27 section 10 + AR-02
  finding 3 precedent). `testrun.py` has no `-n`/`--help` (usage exit 2) -
  use `--test <suite-name>`; read its docstring.
- Journal-incremental SMALL steps: after EACH gate, append 1-3 lines to
  `tasks/ar-27-ta-handoff-journal.md` (gate + canonical line + commit hash
  + any deviation/finding). If context exhaustion looms: STOP, return a
  disk-state inventory; a dead session is NEVER resumed - successor runs
  `python tools/checkpoint_guard.py resume --spec tasks/ar-27-ta-handoff.md
  --scope ... --cmd ...`.

## Report back (final message)

1. Both commit hashes + `git show --stat` (distilled). 2. Oracle evidence:
   the four suites GREEN at the oracle commit with the dispatch baselines
   beside them (this is the pre-mutation proof), + the discriminator pin +
   escape-arm lines post-impl. 3. Green evidence: gates 2-7 canonical lines
   verbatim from own stdout + capture paths. 4. Rider ledger: every clone
   inserted, in-set or out-of-set (cap 6), with the E0507/E0382 that forced
   it. 5. Any deviation + reason. 6. Out-of-scope observations (report-only).
   Never paste raw tool output.

Reviewer-gate reference: acceptance is adjudicated via the `review-gate`
skill (fresh full verification, per-commit `git show` oldest-first,
commit-vs-touch-set enforcement against the list above; the workspace census
vs the `1400/0/13/105` anchor + delta is the reviewer's acceptance pass).
