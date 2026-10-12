# AR-27 Shape Ruling - SourceLanguage -> LanguageId (A2, HIGH-risk)

Issued by: architect (binding). Supersedes design improvisation for the AR-27
successor hops. Basis: `tasks/ar-27-handoff.md`, `tasks/ar-27-handoff-journal.md`
(sections 4-7), and code-level re-verification at the dispatch HEAD performed
during this ruling (evidence files `target/ar27-evidence-*.txt` are gone with
the ephemeral `target/`; the journal quotes them verbatim and every claim was
re-derived from source below).

## 0. Verdict

OPTION B is ADOPTED, as amended by sections 2-5 of this ruling (dual-format
serde: transparent human-readable arm + pinned positional arm, with one
field-level exception). The byte-stability claim is VERIFIED at code level with
the corrections in section 6.

OPTION A's 2.2.0 -> 2.3.0 epoch bump is NOT NEEDED and is REJECTED as the
primary path: under B, zero committed or warm bytes move, so there is no
invalidation a bump could justify, and A would drag a DP-6/DP-2-class USER
decision event onto the critical path plus out-of-set edits to
`crates/index/src/cache.rs` and `crates/build/src/discovery.rs`. A survives
only as contingency if a pre-mutation probe (T-a step 1) discovers a violated
byte-freeze: STOP-AND-REPORT, then a ruling; do not self-authorize a bump.

OPTION C is REJECTED: exceeds the ~800-line bound, touches the most gate
locks, and exceeds the single-token edit class the reviewer lane accepted in
AR-24 D2 / AR-25 D1 (it changes a gate helper's PARAMETER TYPE).

## 1. Frozen locks inherited (with proving commands)

- ADAPT-8 (`crates/build/src/discovery.rs:245-252`): `CompilationContext.language`
  and `TranslationUnit.language` keep type-and-bytes. Proof: `context_hash`
  digest locks in `crates/index/src/cache.rs` (`honest_context_digest...`, the
  bincode-hash test at cache.rs:501) green UNEDITED.
- Epoch pin `CURRENT_ANALYSIS_VERSION == "2.2.0"`
  (`cache.rs:30`, pin test cache.rs:398-403) - UNEDITED, no bump.
- DP epoch rule: any bump is a user decision event. This ruling requires none.
- UNEDITED gate authority (measured green at dispatch HEAD, journal section 2):
  m23 bincode lock 5/0/0/1, m23-symbol 7/0/0/1, m00 4/0/0/2, m24 8/0/0/1,
  m24a 2/0/0/1, m18 23/0/0/1, ui 272/0/0/3, repo 15/0/0/2,
  lang-api 41/0/0/4, cli 118/0/8/7.
- Census anchor: `1400/0/13/105` (passed/failed/ignored/suites). The chain's
  "1400/0/0/13-ignored/105" spelling is WRONG; write it as `1400/0/13/105`.
- Hygiene: R1/R2 zero-blocking; functions <100 lines, files <1000, no source
  line >=2000 chars; pure ASCII; `encoding_guard --all`; commit_scoped.

## 2. Positional encoding contract (binding, verbatim for the implementer)

Workspace bincode is bincode 1.3.3 (Cargo.lock:488-489) with DEFAULT options
(fixint, little-endian; enum variant = u32 LE; String = u64 LE length + UTF-8
bytes). The `is_human_readable()` discriminator is valid here and is already
the in-tree convention (`field_conditional_on_format`, lang-api/src/types.rs
:13-22): the m23 lock family passes today ONLY because bincode reports false
(otherwise `TuAnalysis::default()` would omit empty `extends_edges` from
bincode and `bincode_round_trips_non_empty_analysis_vectors` would fail).

POSITIONAL (non-human-readable) arm of `LanguageId`:

1. Shape is driven by a private (never `pub`) helper
   `#[derive(Serialize, Deserialize)] enum WireLanguageForm { Cpp, Scala, Other, Open(String) }`
   so serde's enum encoding does the framing. Arm order is FROZEN FOREVER:
   Cpp=0, Scala=1, Other=2, Open=3. No reordering, no variant inserted before
   `Open`, ever.
2. Serialize: id "Cpp" -> `00 00 00 00` (u32 LE 0); "Scala" -> `01 00 00 00`;
   "Other" -> `02 00 00 00`; every OTHER id -> `03 00 00 00` (u32 LE 3) then a
   bincode String (u64 LE byte length + UTF-8 bytes). Legacy ids are ALWAYS
   written as their index, even by the new code (mixed-version warm caches stay
   readable both ways for legacy values).
3. Deserialize (positional): read u32 tag; 0 -> "Cpp", 1 -> "Scala", 2 ->
   "Other", 3 -> read String -> `LanguageId::new`; any other tag -> serde
   invalid-value error (no panic).
4. Human-readable arm: `#[serde(transparent)]` bare-name string for ALL ids,
   unchanged from today. An open id stays `"Fortran"` in JSON - there is NO
   externally-tagged `{"Open":...}` shape in any JSON.
5. Collision proof: `SourceLanguage` today has exactly three variants
   (`crates/repo/src/types.rs:142-147`: Cpp, Scala, Other), so the legacy tag
   set is {0,1,2}; tag 3 is unreachable in old bytes (journal section 5
   measured Cpp=[0,0,0,0], Scala=[1,0,0,0], Other=[2,0,0,0]).
6. Round-trip targets: old enum-era blobs (u32 index) load; new escape blobs
   load; JSON identical pre/post for every id.

STRING-FRAME EXCEPTION (field-level, not global):

7. `IncludeEdge.language` (lang-api/src/types.rs:308) is persisted in
   `includes/edges.bin` (`crates/artifact/src/manager/graph_io.rs:35` as
   `IncludeGraph.edges: Vec<IncludeEdge>`, graph.rs:34) and its warm bytes are
   transparent STRING-FRAMED since M24 Part A at the CURRENT epoch (2.2.0
   bump, DP-2). The global positional arm would move those bytes -> old blobs
   hard-error at load. Therefore `IncludeEdge.language` gets
   `#[serde(with = "...")]` (module `language_transparent_frame`, lives in
   `ariadne-repo` next to the newtype) that reproduces today's transparent
   behavior in BOTH formats byte-for-byte (serialize: bare name string;
   deserialize: String -> `LanguageId::new`), keeping `#[serde(default)]`.
8. T-a probe lock: a test enumerates every `Serialize` struct with a
   `LanguageId`-typed field that is bincode-persisted; the asserted set is
   EXACTLY {IncludeEdge.language}. Verified clean during this ruling:
   TuUsage (reduced.rs:72, no language), SemanticEdge, SymbolRecord, CallSite,
   OptimizationPlan (plans.bin; only PlanningGap carries a language and has no
   serde derive), LoadReport/report.bin ("carries no language data",
   metrics_io.rs:214), MigrationPreview (placement.rs:50), tu-usage / file-
   symbols / extends / sealed / callgraph / closures bins, TuCacheFingerprint
   (JSON-only cache.json), LanguageModeling (JSON-only). A probe surprise =
   STOP-AND-REPORT (second exception field, same module, ruling addendum).

Byte-freeze table vs HEAD (ALL zero drift): tu_analyses.bin (language stays
u32 index), repository/files.bin (kind indices untouched in T-a; language
stays u32 index), contexts/translation-units.bin, `SHA256(bincode(ctx))`
context_hash (ADAPT-8 honored), includes/edges.bin (exception module), every
JSON surface (manifest, m00 goldens, ui payloads, cache.json, plans.json,
report.json, load-manifest `languages`).

## 3. Type-level rulings (repo home + coherence)

- Newtype home MOVES to `crates/repo` (new module `crates/repo/src/identity.rs`)
  because `lang-api` already depends on `repo` (identity.rs:7) and the m24 gate
  forces the alias direction; `crates/repo/src/types.rs` gains
  `pub type SourceLanguage = LanguageId;` and `pub use` for the newtype;
  `crates/lang-api/src/identity.rs` becomes a re-export shim + keeps
  `language_of_file_kind` + moved unit tests. All existing import paths must
  keep resolving (`ariadne_repo::SourceLanguage`, `ariadne_lang_api::identity::LanguageId`,
  `ariadne_lang::LanguageId`). The R2 guard walk list is the 8 core crates
  (m17_plugin_seam_test.rs:335-344) - `repo` and `lang-api` are OUTSIDE it, so
  the `Cpp` literals in the newtype are legal in their new home.
- `Default` = Cpp (already true, cache.rs:48 JSON `serde(default)` depends on
  it). Hand-written Debug/Display stay the bare name (fingerprint digest
  identity.rs:36-39; the `{language:?}` error wording at cli/registry.rs:29
  stays byte-stable because enum Debug and the newtype Debug are both "Cpp").
- Ord is CUSTOM, not derived: legacy ids order by legacy variant index
  (Cpp < Scala < Other), preserving the old enum's derived-Ord semantics that
  `build_load_manifest` dedupe+sort consumes (`artifact/manager/metrics_io.rs:
  241` + `collect_stage_languages`) and report grouping; open ids sort AFTER
  all legacy ids, lexicographically among themselves. Derived lexicographic Ord
  would flip Scala-vs-Other sorts (byte drift risk in manifest/report bytes).
  No UNEDITED gate asserts LanguageId ordering (m24 grep-verified); the
  identity test `Cpp < python` stays true.
- Coherence: DELETE `impl From<SourceLanguage> for LanguageId`
  (identity.rs:117-125) - under the alias it is `From<T> for T` -> E0119; the
  std reflexive blanket covers every value site (registry.rs:28,
  stage_index.rs:213, m17:78/92, m19:115, m22:93/101). KEEP
  `impl From<&SourceLanguage> for LanguageId` (identity.rs:127-131 - legal as
  From<&T> for T): `artifact/manager/metrics_io.rs:241` calls
  `LanguageId::from(&SourceLanguage)` over an iterator and compiles UNEDITED
  ONLY because of this kept impl.
- No `Copy` (Cow inner cannot be Copy). Copy loss is handled by the rider
  protocol below - NOT by signature surgery.

## 4. Clone-rider protocol (named touch-set EXTENSION - the only extension)

The journal missed one forced out-of-set consumer:
`crates/pipeline/src/stage_index.rs` reads `self.language_filter:
Option<SourceLanguage>` by value from borrowed `self` (:103/:110) - legal
today only via the enum's `Copy`.

- `crates/pipeline/src/stage_index.rs`: BLESSED for compiler-forced
  clone-insertions only (no logic edits). Suite UNEDITED at its dispatch-
  measured baseline (successor re-brief must pin `-p ariadne-pipeline` counts
  pre-mutation).
- `crates/cli/src/registry.rs:191`: in-set clone, trivial.
- `crates/worker-core/src/tu_store.rs:210`: expected NO-OP (partial move of an
  owned local is legal Rust); blessed as clone-rider only if the compiler says
  otherwise; same measured-baseline discipline (`-p ariadne-worker-core`).
- General rule: any other out-of-set file needing a pure copy-semantics
  `.clone()` insertion is pre-blessed up to 6 sites TOTAL out-of-set; any
  out-of-set edit that is not a copy-semantics clone => STOP-AND-REPORT.
  (No new lock obligations beyond those two rider suites' exact counts.)

## 5. Split ruling, seam, TDD order, sizes

T-a - serde migration + byte locks (~450-520 lines, within the ~800 bound):
1. RED-FREE GREEN-FIRST (AR-08/AR-31 precedent): regenerate and COMMIT
   old-bytes fixtures (the deleted `target/ar27-evidence-*.txt` content is
   regenerated as tracked fixtures): `Vec<TuAnalysis>` (empty and non-empty
   extends/modifiers variants), `Vec<FileRecord>`, `CompilationContext`
   bincode bytes + digest hex, `Vec<IncludeEdge>` (bincode + JSON), the three
   ids' JSON. Committed green against the UNTOUCHED types; the same assertions
   stay green post-mutation. This is the oracle for every byte claim.
2. Home move + alias + From-coherence edits + custom Ord + Debug/Display/
   Default carried over.
3. Dual-format serde per section 2 + `language_transparent_frame` module +
   `IncludeEdge` attachment + probe-enumeration lock.
4. `parse_language_arg` KEEPS today's closed behavior byte-for-byte, Err
   wording included ("unknown language '{raw}'; expected cpp, scala, or other
   (serde names: Cpp, Scala, Other)"): the global transparent JSON reader now
   accepts any string, so the closed set is enforced by an explicit match
   against the three legacy ids after the existing normalization. The existing
   cli lock `parse_language_arg("Fortran") -> Err` MUST still hold after T-a
   (T-b's RED depends on it).
5. Riders per section 4; STRICT COMPILE-GREEN workspace; all section-1 counts
   UNEDITED green (in-set suite counts reconcile UP by the journal-recorded
   test delta; "exact" means pre-count + delta).

T-b - open `--lang` + FileKind roles + E2E (~300-380 lines):
6. FileKind role refactor is RULED INTO T-B (the journal proposed T-a;
   adjusted). Rationale: T-a's value is pure byte-stability and FileKind is
   inert there; the refactor's justification and its E2E lock are T-b's; the
   files.bin variant-index shift is contained by T-b's byte-guard: pre-grep ALL
   tracked fixtures, binary included, for committed FileKind bytes
   ("ScalaSource"/"ScalaScript"/"BuildDefinition") - the journal proved zero
   for committed JSON; repeat for `.bin` goldens + keep m00/m19/m22/m24a green.
   - Guard clean: rename to language-agnostic roles (Header, Source, Script,
     BuildDefinition, Other) + associated-const bridges (`FileKind::ScalaSource
     = Source`, `FileKind::ScalaScript = Script`) so m19:114, m23:145,
     lang-scala shaper:45 compile UNEDITED + `#[serde(alias = "ScalaSource")]`
     / `#[serde(alias = "ScalaScript")]` for warm JSON manifest reads (JSON
     only - aliases are self-describing-format mechanisms; the journal's own
     finding). New manifests write the role names for Scala files - ruled
     consequence, zero committed-byte impact (grep-proven class).
   - Guard dirty (any committed byte golden found): FALLBACK SHAPE, ruled now
     so no one improvises: FileKind byte shape frozen (5 variants stay),
     `FileKind::role()` added, per-language variants treated as spine
     taxonomy data, open-set language authority stays
     `registry.detect_language` (D-A6, identity.rs:133-141). Same E2E lock,
     zero byte movement.
   - Touch-set extension (named, in-crate): `crates/repo/src/discovery.rs`
     (:89) tags language from PATH taxonomy (m17 carve-out makes the spine
     legal); NO repo->registry dependency may be introduced. repo suite
     rebaselines in the T-b re-brief.
7. `--lang` open set: candidates from `registry.languages()`
   (lang/src/registry.rs:123, registration order). At the DEFAULT registry the
   rendered text stays byte-identical to today's "(cpp|scala|other)"; newly
   registered plugin ids append in registration order (new lock: third
   language appears when a mock plugin registers it). Closed-set authority
   becomes `ensure_language_registered` (already bails with
   `language plugin not registered: {language:?}`; wording may name the
   registry candidates in T-b, in-set). RED-FIRST E2E lock: mock third
   language flows detect -> tag (registry path) -> persist (bincode escape
   round-trip + JSON bare name) -> `--lang <id>` select with ZERO core edits;
   RED today = the `parse_language_arg("Fortran")` Err plus absence of an
   open-id persist path.

Dispatch discipline: one hop at a time, sole cargo-heavy lane (STRICT
COMPILE-GREEN requirement), journal per hop.

## 6. Corrections to the journal's Option B description

(i) human-readable arm transparent for OPEN ids too - never `{"Open":...}`
JSON anywhere; (ii) `IncludeEdge.language` string-frame exception is MANDATORY
(its warm string-framed bincode bytes at epoch 2.2.0 were not accounted for);
(iii) custom legacy-index Ord is mandatory (derived lexicographic flips
Scala/Other sorts feeding `build_load_manifest`); (iv) From-coherence edits
(delete value From, keep &-From - the artifact crate compiles only via the
latter); (v) `crates/pipeline/src/stage_index.rs` clone rider is required
(missed by the journal's copy-loss forecast); (vi) FileKind role refactor
moves from T-a to T-b with the byte-guard fallback; (vii) census anchor
spelling: `1400/0/13/105`.

## 7. Golden/lock amendment policy

ZERO. This ruling sanctions no edit to any tracked golden, lock, or byte-
authority file. The FileKind serde aliases touch no committed bytes (journal
grep proof; T-b re-proves including binary fixtures). The epoch pin stays
2.2.0. `m24:549`'s stale comment stays unedited (journal section 7).

## 8. Risks -> oracles

- Discriminator assumption: new pin test (bincode `is_human_readable()==false`,
  serde_json `true`) + the m23 family (already an empirical proof).
- Unenumerated warm LanguageId-in-bincode field: T-a probe lock section 2.8;
  surprise = STOP.
- Sort/manifest byte drift: m22 manifest + m00 goldens UNEDITED (custom Ord).
- Rider sprawl: 6-site cap out-of-set, then STOP (re-split = C-class ruling).
- T-b FileKind byte shift: pre-grep guard + ruled fallback shape.
- Help/help-lock drift: default-registry rendering byte-identical.

## 9. Handoff disposition

`tasks/ar-27-handoff.md` needs a planner rewrite: T-a and T-b become two new
handoff files (planner, via handoff-spec, embedding this ruling as the frozen
shape section); the old contract is bannered SUPERSEDED pointing here (planner
lands the banner; implementers do not edit contracts). The journal remains as
evidence. The re-brief must pin: repo 15/0/0/2 measured, m23-bincode 5/0/0/1
vs m23-symbol 7/0/0/1 distinction (journal banked finding), and rider-crate
baselines measured before mutation.

---

# RE-RULING (T-a copy-loss clamp) - binding, appended 2026-10-12

Input: `tasks/ar-27-ta-handoff-journal.md` (STOP at dispatch HEAD `85f40d5`,
zero code landed, probe reverted byte-clean). Section 2's positional encoding
contract STANDS UNCHANGED - the probe never touched serde/field order/framing
and contradicted no byte claim. What failed is section 4's copy-loss forecast:
measured 24 production out-of-set sites in 9 files (cap 6), 6 sites in
MUST-NOT-touch `crates/build`/`crates/artifact`, and >=6 token sites in
byte-frozen gate-lock TEST files - a LOWER bound while the integration
package cannot compile. This section supersedes section 4 and amends section
7; all other sections stand.

## R1. Chosen option: B (T-a0 consumer-adaptation hop FIRST), with A's
enumerated precision embedded as T-a0's ledger.

Land every copy-semantics token FIRST, in `tasks/ar-27-ta0-handoff.md`,
WHILE `SourceLanguage` is still the derived-`Copy` enum. Justification:
- Provably-inert riders: while the type is `Copy`, `.clone()` IS the copy
  (definitionally value-identical). The reviewer adjudicates a mechanical
  token-only commit instead of reasoning per-site "this clone restores prior
  semantics" inside a ~500-line semantic move. This is the AR-31
  characterization-first / AR-16 inert-landing precedent.
- The ledger lower-bound closes BEFORE the move lands: T-a0's completeness
  probe (R3 protocol below) surfaces the hidden integration/frozen-test
  remainder in a throwaway Copy-strip world where they compile. Under option
  A those sites surface MID-T-a (the integration package only compiles once
  the cli lib is fixed - i.e. after the move), guaranteeing mid-hop STOPs.
- Planning-Law sizing (axis = gate count x output volume): T-a0 is ~35-45
  single-token edits + ~9 tiny gates (worst under budget); T-a sheds its
  rider section and returns to a pure move+serde+oracles unit; T-b untouched.
  Option A stayed within the 1200-line bound too, but it fused two review
  objects (semantic move + frozen-territory token class) into one commit set,
  and it kept the unbounded ledger live during the semantic move.

## R2. FROZEN-TEST-TOKEN CLASS RULING (the sharp question), binding class law:

Compiler-forced copy-semantics TOKEN insertions into byte-frozen gate-lock
test files are COMPILE-ADAPTATION RIDERS, NOT lock amendments. The "LOCK
AMENDMENTS: NONE" clause governs what a lock ASSERTS (bytes, literals,
expectations, ordering, wording, counts); it is not violated by a value-
FETCH token that cannot change the fetched value - provable, because the
tokens land while the type is derived-`Copy` (clone-on-`Copy` is identity).
This generalizes the accepted AR-24-D2 (mock-signature token) / AR-25-D1
(render token) precedents from precedent to class.

Hard constraints (all reviewer-blockable):
1. Token forms ONLY: `x` -> `x.clone()`; `*x` -> `x.clone()`;
   `x.into()` -> `x.clone().into()`; `From::from(x)` -> `From::from(x.clone())`
   (likewise `LanguageId::from(x)`). NO `&`-iteration rewrites, NO signature
   or pattern surgery, NO reordering, NO logic. Uniform clone form -
   per-site `&`-cleverness is explicitly ruled OUT for reviewability.
2. Zero edits to any literal, assertion, expectation, fixture, count, or
   ordering in any file; suite test counts identical before/after.
3. Sites must appear in the contract's site ledger; probe-surfaced additions
   are appended to the ledger in the journal BEFORE commit, same class.
4. Reviewer verification recipe (token-diff class check): for each named
   frozen file, `git show <T-a0-commit> -- <file>` - every changed +/- pair
   must be token-normalizable: after deleting inserted `.clone()`s and
   restoring `*x` for `x.clone()` rewrites, the pair is byte-equal. Any pair
   that is not = REJECT the hop. Quote per-file outputs in the verdict.
5. Affected suites run UNEDITED-assertions at identical counts: pipeline
   `40/0/4/9` (contains `for_incremental_parity_test` `5/0/0/1`),
   m23-symbol `7/0/0/1`, the m22 suite, cli `118/0/8/7` - plus
   `-p ariadne-build` and `-p ariadne-artifact` counts MEASURED pre-mutation
   and pinned by the reviewer at T-a0 (never pinned before).

Amended section 7 (restated): still ZERO edits to committed BYTES and ZERO
changes to what any lock asserts; the only sanctioned tracked-file edits are
the ledger-enumerated value-invariant copy tokens of constraint 1, per R2.

## R3. T-a0 contract shape (planner authors `tasks/ar-27-ta0-handoff.md`):

Objective: pre-land every copy-semantics site the AR-27a alias will force,
inertly, before any type change. Sole cargo-heavy lane; STRICT COMPILE-GREEN.

Touch-set = EXACTLY this ledger (12 files; token-class only) - this list
SUPERSEDES the 6-site cap of section 4 and is the named out-of-set boundary:
- `crates/build/src/discovery.rs` :253 `ctx.language = record.language;` ->
  `record.language.clone()`; :272 `language: record.language,` ->
  `record.language.clone()`. (Verified expression-level reads; the ADAPT-8
  comment block :245-252 stays BYTE-UNEDITED and its claim remains true
  through the alias - the clone moves neither type nor bytes.)
- `crates/artifact/src/manager/discovery_io.rs` :51, :130, :131 ->
  `.map(|f| f.language.clone())` / `.map(|c| c.language.clone())` /
  `.map(|tu| tu.language.clone())` (clone form, NOT `&`-rewrite - R2.1).
- `crates/artifact/src/manager/m22_manifest_languages_tests.rs` :39, :154,
  :169 -> `*lang` -> `lang.clone()` (src test-mod in a MUST-NOT crate; enters
  only as a named T-a0 token file).
- `crates/pipeline/src/stage_index.rs` :110, :122 ->
  `self.language_filter.clone()`.
- `crates/pipeline/src/stage_load.rs` :183 -> `language.clone()`.
- `crates/pipeline/src/for_incremental.rs` :66 `LanguageId::from(language
  .clone())` (unblocks the :67 `{language:?}`), :85 clone inside the retain
  closure, :121 `ensure_language_registered(&registry, language.clone())`,
  plus one clone among the remaining uses (:125/:141/:143) so every use
  except the last clones; journal the exact token set (compiler-verified in
  the probe).
- `crates/pipeline/src/lib.rs` :681 -> clone.
- `crates/cli/src/registry.rs` :28 (`from(language.clone())`, fixes the :29
  bail E0382), :174-175, :192, :448 `*language` -> `language.clone()`, :520
  - exact tokens per the T-a journal ledger items 2-6.
- `crates/cli/src/commands/analyze.rs` :93, :143, :148, :154, :169, :262;
  `crates/cli/src/commands/index.rs` :77; `crates/cli/src/commands/ui/
  node_maps.rs` :59 `file.language.clone()` (entry key), :62 `*language` ->
  `language.clone()`.
- FROZEN existing test files, R2-class tokens:
  `crates/pipeline/tests/for_incremental_parity_test.rs` :98, :114, :153,
  :163; `tests/integration/tests/m22_scala_load_analysis_test.rs` :280
  `record.language.clone().into()`; `tests/integration/tests/
  m23_symbol_isolation_test.rs` :406 `LanguageId::from(record.language
  .clone())`.
- Probe-surfaced remainder (journal's lower-bound delta): same class, same
  forms, appended to the ledger.
- NOT touched in T-a0: `crates/repo/**`, `crates/lang-api/**`
  (identity.rs's `&`-From body is part of T-a's move/rewrite),
  `crates/worker-core/**` (tu_store:210 no-op CONFIRMED by probe - ruling
  claim upheld), every other file, all goldens.

Method (three-step, the journal's own reverted-probe protocol as the oracle):
1. Land the ledger tokens (they compile inertly at today's Copy type).
2. COMPLETENESS PROBE: strip `Copy` from the `SourceLanguage` derive list
   (throwaway, NEVER committed; throwaway tokens allowed ONLY in
   repo/lang-api/cli-registry-owned files that T-a will own - those sites are
   committed by T-a, not T-a0); run `cargo check --workspace --all-targets`;
   iterate until probe-world reports ZERO copy-semantics errors; revert the
   probe byte-clean (`git status --porcelain -- crates` empty apart from the
   committed tokens). This converts the ledger from lower-bound to complete.
3. Real-world gate set (<=9 invocations): all-targets workspace check green;
   affected-crate suites at exact counts (pipeline 40/0/4/9, cli 118/0/8/7,
   build/artifact MEASURED+pin, index 15/0/0/2 UNEDITED digest proof,
   m22+m23+for_incremental_parity identical counts); encoding_guard
   --all/--staged; hygiene zero-blocking; the R2.4 token-diff outputs for
   the three frozen test files quoted in the journal. Single scoped commit
   (commit_scoped, explicit --scope per file).

Sequencing: T-a0 APPROVED -> T-a (move+serde; MUST compile-green with ZERO
rider edits; newly surfaced SAME-class copy site = clone-token addition +
journal, no re-ruling; any site needing non-token adaptation = STOP-AND-
REPORT) -> T-b unchanged. Clippy note: pre-landed clones add transient
`clone_on_copy` warnings while the enum is Copy; CI does not gate warnings
(ci.yml:28-30, the `-D warnings` flip is a tracked future milestone) and
AR-28 owns the clone-on-Copy sweep; warnings vanish for these sites when the
alias lands.

## R4. Option sanity-checks (reaffirmed):

- Option C (Copy-able/interned identity) REJECTION STANDS, now at measured
  cost ~28-45 inert tokens: an interned/Copy representation needs a process-
  global interner, changes `LanguageId::new(impl Into<Cow<'static, str>>)`
  semantics that T-b's `--lang` user-token depends on, and fights the
  load-bearing owned-parse lock (`language_id_round_trips_..._json` parses
  `"Fortran"` to an owned id). Redesign-at-last-mile is strictly riskier than
  inert tokens; do not re-litigate.
- Option D (epoch bump) remains NOT AN OPTION: copy-loss is a compile-time
  class; bytes and epochs are untouched by this problem.

## R5. Erratum into the record (T-a journal section 6, upheld):

Section 4 of this ruling cited stage_index ":103/:110" as the by-value
borrow sites. Corrected: the E0507 sites are `:110` AND `:122`; `:103`
(match on `self.language_filter` with discriminant-only `Some(_)` pattern)
does NOT move and needs no token. Blessing a non-site would have made the
reviewer's token-diff check fail. (`worker-core tu_store:210` no-op and
`metrics_io:241` riding the kept `From<&T>` were both CONFIRMED by the
probe - those ruling claims stand.)

## R6. Planner change list (authoritative):

1. NEW `tasks/ar-27-ta0-handoff.md` = hop 0 per R3 (ledger verbatim;
   method; <=9 gates; token-diff evidence clause; baselines measured incl.
   `-p ariadne-build` / `-p ariadne-artifact`; single scoped commit;
   reviewer-gate reference includes R2.4 recipe).
2. `tasks/ar-27-ta-handoff.md`: dispatch-order table gains hop 0 (serial:
   T-a0 -> T-a -> T-b); DELETE the Clone-rider protocol section, the
   `stage_index.rs` touch-set bullet, and the `tu_store.rs` rider bullet;
   replace with the R3 sequencing clause (zero riders; same-class additions
   journal-logged; non-token site = STOP); drop `crates/pipeline/src/
   stage_index.rs` from the implementation-commit `--scope` list; move the
   rider-crate pre-mutation pins to T-a0 (keep a re-measure line); note that
   registry.rs clone tokens PRE-EXIST from T-a0 and survive the parse rework
   untouched; correct the `:103/:110` citation to `:110/:122`.
3. `tasks/ar-27-tb-handoff.md`: NO semantic change; optional one-line note
   that T-b's own new code must not assume `Copy` on `LanguageId`.
4. This RE-RULING is embedded VERBATIM (R1-R5) into the T-a0 contract's
   authority section by the planner; the encoding contract (section 2) is
   quoted into T-a unchanged.

Revised produced-line estimates: T-a0 ~90-140 (ledger tokens ~35-45 single-
line edits + journal); T-a ~430-500 (was ~450-520, riders removed); T-b
~300-380 unchanged. All hops inside the ~1200-line budget and the
Planning-Law gate budgets.
