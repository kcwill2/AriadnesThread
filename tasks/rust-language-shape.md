# rust-language - Change Shape (2026-10-25)

Companion to `tasks/rust-language-kickoff.md` (the authority for decisions
D-1..D-3, acceptance commands, frozen locks L1-L12, amendments A1-A8, and
the graduation audit). This doc is the change SHAPE only: invariants,
contracts, the modeling decision table, the module map, the handoff-arc
sketch, risks with named oracles, hygiene notes. The planner turns this
into sized handoffs; nothing here sequences them beyond hard edges.

Start state: branch `balsam-basil`, anchor `1853/0/13/119 @ 522a770`,
HEAD (docs) `f235c8a`+kickoff commit. All line numbers verified by read
this session unless marked STALE.

## 1. Intent

Add Rust as the third language plugin at full parity with Scala (D-1),
parsing real Rust syntax with `syn` (D-2), through the seams every prior
language used: open `LanguageId` (no vocabulary enum edit), one registration
line, plugin-supplied modeling data, registry-dispatched planning hooks.
The spine edit is exactly the `FileKind` table (`crates/repo` is out of the
neutrality-guard walk by design - the spine MUST name languages), plus one
appended planning-vocabulary variant in `lang-api`.

## 2. Invariants (numbered, each independently falsifiable)

INV-1 Enumeration. `default_registry().languages() == [Cpp, Scala, Rust]`
in that order, cli and pipeline agreeing. Lock: amended
`default_registry_enums_cpp_then_scala` (A1) +
`registries_enumerate_identical_languages` (stays green automatically,
verified) + amended m19 `plugin_contract_in_default_registry` (A7).

INV-2 First-wins detection is unchanged for every path the C++/Scala
detectors claim. For every path in the existing scala/cpp fixture corpora,
`detect_language` answers before and after identically. Lock: m19 scala +
m18 cpp discovery/parity suites + m00 ALL green UNEDITED; `.rs` is claimed
by neither shipped detector (read-verified this session: ScalaDetector owns
`.scala`/`.sc`/`build.sbt`/`build.sc`/`pom.xml`/`project/build.properties`;
the C++ side owns header/source extensions).

INV-3 Plugin zero-diff. `git diff <start>..HEAD -- crates/lang-cpp
crates/lang-scala` is EMPTY at closeout (locks L7/L8).

INV-4 Byte-freeze. The `ar27_*` suites, `tests/golden/**`, and m00 stay
byte-clean UNEDITED. `"Rust"` serializes as the bare name under JSON and as
the tag-3 Open escape under positional formats - proven for the REAL plugin
id (not the m24 Fortran mock) by a new identity test in the milestone's
discovery suite, mirroring `m24_language_identity_test.rs` claims 1+2.
`WireLanguageForm`, `legacy_rank`, `language_transparent_frame` unedited
(legacy_rank: `"Rust"` is rank 3 like any open id - reports group it after
the stored trio; no edit, verified by the ar27 trio's own tests).

INV-5 Capability monotonicity. At every commit on the arc, no capability
bit is `true` before the component behind it exists in the crate. The final
state is the byte-exact all-true literal lock (D-23-6-style):
`capabilities == LanguageCapabilities { discovery: true, dependency_graph:
true, semantic_index: true, load_metric: true, planning: true, extraction:
true, rewriting: true, validation: true, requires_toolchain: false }`,
locked by a lang-rust in-src test named in Scala's pattern
(`capabilities_match_the_*_flip_literal`). Lock: that test + m19-style
capability lock in the new integration suite.

INV-6 Discovery fixture truth. On `tests/fixtures/rust-basic/`: every
`src/**.rs` yields exactly one unit of context per nearest `Cargo.toml`
ancestor; paths carrying `tests`/`examples`/`benches` components are
excluded; `Cargo.toml` itself never becomes a unit; no C++/Scala-flavored
context appears (D8-lock idiom from m19's test). Lock: the milestone's
discovery suite.

INV-7 Span truth (D-2's correctness guarantee made falsifiable). For named
fixture items, analyzer records carry the EXACT expected line and column
(1-based) - e.g. the `use` of a fixture `mod`/item pinned to its literal
line - proving `proc-macro2` `span-locations` is enabled. Without the
feature the accessors do not exist at all (compile-forced), so the
secondary oracle is the crate compiling with spans consumed.

INV-8 Fingerprint. `analysis_fingerprint() == "rust-1"` (literal-locked in
lang-rust; locked the hop the real parser lands); C++/Scala fingerprint
literals unedited (INV-3 covers). Cache digests for C++/Scala fixtures do
not move (m21-style witness unedited).

INV-9 `.rs` claimed, consistently. Post-registration:
`parse_language_arg("rust") == Ok`, `"Rust"` accepted case-folded per the
pin-parse exact-set test (amended A2), AND the string `unclaimed.rs`
occurs NOWHERE under `crates/` (A3/A4 moved those witnesses to `.f90`).
Grep-count zero is the oracle.

INV-10 Probe-only validation (A3 assumption). `toolchain_status()` is never
`Unknown` (probe validator landed); scripted-runner tests prove: (a) no
usable probe -> `Missing`; (b) every spawn through the injected runner is
`(TOOLCHAIN_PROBE_ORDER entry, ["--version"])`; (c) `validate_compile`
offline errors naming the deferral without spawning anything beyond the
probe walk. Mirror of the Scala validator's locked shape. Tier 3 `Skipped`
mapping is `crates/validate`'s existing neutral behavior - no edit.

INV-11 Guard coverage extended. BOTH `PLUGIN_CRATES` lists in
`m17_plugin_seam_test.rs` (:271 manifest guard, :486 transitive-tree
guard) contain `"crates/lang-rust"` as a 3rd element, and both guards run
green UNEDITED from that hop on. Appending the element and adding the
manifest land in the SAME commit (both guards self-skip a missing
manifest - append-early would silently pass, append-late is caught by the
test-diff review).

INV-12 Enum append-only. `FileKind::RustSource` is the LAST variant (after
`Other`); the new `AggregationKind` variant is the LAST variant. No
pre-existing variant's serde index moves, so every C++/Scala warm byte and
manifest byte is unchanged. Oracle: ar27/golden/m00 green UNEDITED + the
`pin_file_kind_lowercase_extension_matrix` amendment (A5) + JSON round-trip
tests.

INV-13 Census arithmetic. Every hop reports the canonical `testrun.py
--distill` line; passed >= the previous anchor, failed 0, every delta
attributed by name; ignored 13 unchanged unless a named hand carries it.

INV-14 Language is data (no UI/report edits). ZERO bytes under
`crates/ui/**`, `crates/report/**`, `crates/artifact/**` change. A Rust
run's manifests carry `"Rust"` in their additive `languages` fields through
the existing `collect_stage_languages` data path (verified data-driven),
and the UI node language map shows `Rust` through the hand-written bare-name
`Debug` (`crates/cli/src/commands/ui/node_maps.rs` `format!("{:?}", ...)`).
Oracle: the R-4/R-6 integration suites assert manifest/UI-data presence of
`"Rust"` with those crates' diffs EMPTY.

INV-15 Hygiene/encoding. `hygiene_check.py` exit 0 and
`encoding_guard --all` exit 0 every hop; all authored bytes ASCII.

## 3. Contracts (exact literals and signatures)

### 3.1 Core-spine edits (guard-legal; `crates/repo` by design, `lang-api`
is plugin vocabulary)

```rust
// crates/repo/src/types.rs - enum APPENDS the new variant LAST (INV-12):
pub enum FileKind { Header, Source, ScalaSource, ScalaScript,
                    BuildDefinition, Other, RustSource }
// from_path table gains:  Some("rs") => FileKind::RustSource,
// is_analyzable():        matches!(..., Header | Source | ScalaSource | RustSource)
// Matrix test :103 amended (A5): ("a.rs", FileKind::RustSource)
//   (+ case-matrix row "a.RS"; precedent: extension case already folds)

// crates/repo/src/identity.rs - const pattern mirrors Cpp/Scala:
pub const RUST: &str = "Rust";
#[allow(non_upper_case_globals)]
pub const Rust: Self = Self(Cow::Borrowed(RUST));
pub fn rust() -> Self { Self::Rust }   // guard-consistent snake spelling
// from_file_kind arm:  FileKind::RustSource => Self::Rust,
// (its exhaustive test arm compiles by the same edit; lang-api advisory
//  test row added - A6)
```

`SourceLanguage` is the alias (`types.rs:259`, verified) - no other spine
type moves. `WireLanguageForm` / `legacy_rank` / transparent-frame NOT
touched (L1).

```rust
// crates/lang-api/src/modeling.rs - APPEND after None (INV-12):
pub enum AggregationKind { Header, PackageObject, None, ModDeclaration }
```

No production `match` on `AggregationKind` exists anywhere (grep-verified
this session; the scheduler/graph occurrences are test-module
constructions). If any new non-exhaustive site appears at compile time it
is a STOP-and-report, not a silent arm.

### 3.2 Registration (single line, appended LAST - order is a contract)

```rust
// crates/pipeline/src/registry.rs build_default_registry:
.register(Box::new(ariadne_lang_rust::RustPlugin::new()))
```

CLI delegates (`crates/cli/src/registry.rs:16`, verified) - no CLI
registration edit. Root `Cargo.toml`: `"crates/lang-rust"` appended after
`"crates/lang-scala"` in members; `[workspace.dependencies]` gains
(house style, pinned by workspace):

```toml
syn = { version = "2", features = ["full"] }
proc-macro2 = { version = "1", features = ["span-locations"] }
unicode-ident = "1"
```

`Cargo.lock` regenerates in that same commit (touch-set, reviewer rule).
`span-locations` on `proc-macro2` is load-bearing (D-2, INV-7).

### 3.3 Plugin identity and modeling (decision table)

| field | Rust value | Scala precedent | C++ precedent | reason |
|---|---|---|---|---|
| id() | `LanguageId::Rust` ("Rust") | Scala | Cpp | open Cow newtype; no vocabulary enum edit (verified :79 consts pattern) |
| unit | `UnitKind::SourceFile` | SourceFile | TranslationUnit | one `.rs` file is a module - Rust's unit granularity is the file |
| aggregation | `AggregationKind::ModDeclaration` (NEW variant) | PackageObject | Header | `mod x;` + `mod.rs`/`x.rs` sibling is Rust's aggregation mechanism; PackageObject naming would lie for Rust, `None` would understate a real mechanism (kickoff L11 append-only) |
| guard | `GuardKind::None` | None | PragmaOnce | Rust has no duplicate-definition guard idiom |
| symbol_placement | `SymbolPlacement::SingleFile` | SingleFile | SplitDeclaration | decl+def live in one file; declared record only, no dispatch (guide trap noted) |
| aggregation_reexports | `false` | false | true | mod declaration does not re-aggregate code; `pub use` is a separate explicit idiom, not a property of the mechanism |
| cycle_policy | `CyclePolicy::TolerateWithinPackage` | TolerateWithinPackage | Reject | `use`-edge cycles inside one crate's module tree are legal Rust (cross-crate file cycles cannot arise from a Cargo-legal workspace); planner treats the stance as informational (m18.7 lock `cycle_policy_variants_share_the_scc_coproposal_path_for_now`, read-verified at plan.rs:199-203), report side consumes it via the existing registry lookup `cycle_policy_by_file` (pipeline/src/lib.rs:672-687, verified) - ZERO core edits |
| external_path_markers | `["target", "vendor", ".cargo"]` | lib/target/.ivy2/.cache (single-sourced const) | third_party/vendor | `target` build output + `vendor` (C++-convention vendored deps) + `.cargo` repo-local dir; all three are discovery-skipped or rarely present; declared for boundary-data honesty, mirroring Scala's defensive `target` single-source pattern (`boundary.rs` const mirrored in lang-rust) |
| vocabulary | `directive_label "Use"`, `directive_passive_verb "used"`, `aggregation_noun "mod aggregation"` | Import/imported/package aggregation | Include/included/header | D26 data path - CLI/graph render consumes the fields; "Use" matches the `use` keyword |
| analysis_caveat | `Some(...)` naming the macro limit; literal must contain "macro" (test predicate idiom mirrors Scala's `contains("heuristic")`) | Some(heuristic disclaimer) | None | D-2 honest limit: `include!` / `macro_rules!`-generated `mod` structure is not expanded; syn is exact about written code, not about macro output |

### 3.4 Capability literals

Final state (R-5): the all-true + `requires_toolchain: false` struct of
INV-5, byte-exact locked. Intermediate states per the monotonicity
invariant; suggested per-hand schedule (planner owns): R-1
`discovery:true` only -> R-2 `+semantic_index` (analyzer) -> R-3
`+dependency_graph` (resolver+graph acceptance) -> R-4 `+load_metric` ->
R-5 `+planning/extraction/rewriting/validation` and the final literal lock.
`requires_toolchain: false` at every hop (A3/D-2).

### 3.5 Detector / shaper / hooks / validator contracts

```rust
// detector: name() == "rust";
// detect(): basename "Cargo.toml" (case-folded) -> BuildDefinition  [before ext rule]
//           ext "rs" (case-folded)              -> Source
//           everything else                      -> Other
```

Note: the spine `FileKind` does NOT gain a Cargo.toml mapping (extension
`toml` is shared - mapping it would leak into Other's neighbors); the
plugin detector names it, which is exactly what Scala does for `pom.xml`
(base-name rule read-verified). `detect_language` remains
plugin-authoritative (D-A2/D-A6 verified), so the advisory table staying
silent on Cargo.toml is honest.

```rust
// shaper: build_markers() == ["Cargo.toml"]; default_compiler() == "rustc";
// shape_units: FileKind::RustSource records whose path contains a
//   consecutive lowercase ["src"] component (cargo source root, any crate
//   of a workspace), EXCLUDING paths carrying a "tests"/"examples"/
//   "benches" component (the exclusion-tag idiom, mirroring Scala's
//   src/test/scala tag); context name = nearest ancestor dir holding a
//   Cargo.toml (walk file dir toward repo_root), fallback "root".
```

```rust
// aggregation_filter override (planning hooks - ADR-0002 Decision 3 pair;
// it is the plugin override, not a CLI edit, that enrolls Rust -
// planning_hooks registry dispatch read-verified at cli/src/registry.rs:276):
//   Some: file content parses (syn) and declares >= 1 non-inline `mod x;`
//   item (disk-level aggregation edge). Inline `mod x { .. }` blocks are
//   NOT aggregation children. Unreadable/unparseable -> Some(false)
//   (the filter invents nothing - Scala parity).
// target_path_for override:
//   Some: `{sanitized_symbol}.rs` beside the aggregation file
//   (sanitize discipline mirrors scala_target_path_for).
```

```rust
// validator: TOOLCHAIN_PROBE_ORDER == ["cargo", "rustc"]  (mission
//   wording order); toolchain_status -> Available(first banner) | Missing;
//   validate_compile uses `rustc` specifically with options-before-sources
//   (a single-input honesty note in its doc: rustc takes one main input,
//   multi-file calls are out of scope - TierValidator never calls it while
//   offline (INV-10), D-M23-12 class). CommandRunner injection seam
//   mirrors the Scala file 1-for-1 so the probe tests need no real spawns.
```

### 3.6 CLI `--lang` closed set

`closed_set_language_by_serde_name` gains the RUST branch; error text
becomes `unknown language '{raw}'; expected cpp, scala, rust, or other
(serde names: Cpp, Scala, Rust, Other)`; tests A1/A2 amended in the
registration commit. `ensure_language_registered` needs no edit (registry
lookup). The three `CyclePolicy::Reject` planner literals
(`commands/plan.rs:203`, `optimize.rs:305`, `commands/ui.rs:419`,
current refs verified this session) DO NOT change for Rust: all three
constructors already consume registry-dispatched `planning_hooks`
(post-AR-19 state read-verified), the policy field is informational to the
planner (INV-7-of-m18.7 lock), and per-language report-side policy arrives
through `cycle_policy_by_file`. Rust enrolls in planning purely by
overriding the two trait hooks - the guide's "expect to edit every site"
is a pre-AR-19 statement; its dated rider (R-6) records the truth.

### 3.7 Analyzer/resolver/extractor/rewriter surface (shape level)

- analyzer (syn): items -> `SymbolRecord`s (fn/struct/enum/trait/impl/type/
  const/static/mod with qualified `crate::<path>::<Name>` ids), `use`
  statements -> usage edges (kind mirrors the Scala Incomplete stance -
  path targets resolve through the resolver), span-locations line/col on
  every record (INV-7). File with macro items: parse what syn accepts;
  unexpanded macros recorded honestly (caveat, no invented symbols).
- resolver: `mod x;` -> the child file (`x.rs` or `x/mod.rs`); `use` paths
  -> repo-discovered targets via `ResolutionContext` (external/registry
  paths fall to the external-boundary markers; out-of-repo `::serde::...`
  resolves to nothing - mirrors C++ system-header/Scala-lib stance);
  `RawDepEdge` line/col carried.
- constraints: `colocated_groups` == Rust's trivial groups (none by
  default - no companion-object idiom; an honest empty today),
  `frozen_symbols` empty (IV3: policy owns freezing - Scala parity).
- extractor: emits valid Rust source from the ORIGINAL item text
  (`proc_macro2::TokenStream` to_string via syn printing, preserving
  comment loss honestly - the ASCII rule applies to every generated byte).
- rewriter: `use`-path rewrite (the consumer-rewrite hook analog; the
  `RewriteSpec` old_deps/new_deps mapping over `use x.y::Z;` spellings),
  plus the RISK-MODTREE duty (sec. 6): keeping the extracted module
  declared in its parent.

## 4. Module map

EDITED (with current line counts -> margin):
- `Cargo.toml` (53 -> members + workspace deps), `Cargo.lock` (regenerated)
- `crates/repo/src/types.rs` (304) - enum/table/predicate + matrix tests
- `crates/repo/src/identity.rs` (401) - consts + `from_file_kind` arm
- `crates/lang-api/src/modeling.rs` (204) - appended variant
- `crates/lang-api/src/identity.rs` (52) - advisory test row
- `crates/pipeline/src/registry.rs` (39) - one `.register` line (+doc)
- `crates/cli/src/registry.rs` (761) - closed set, error text, tests A1-A4
- `tests/integration/tests/m17_plugin_seam_test.rs` (740) - two list
  elements (A8)
- `tests/integration/tests/m19_scala_discovery_test.rs` (283) - one vec
  literal (A7)

NEW:
- `crates/lang-rust/**` - mirror Scala's file inward-shape: `lib.rs`,
  `plugin.rs`, `detector.rs`, `shaper.rs`, `boundary.rs` (single-sourced
  markers), `analyzer.rs` (syn walker; split `parser_items.rs` /
  `parser_uses.rs` if growth demands), `resolver.rs`, `constraints.rs`,
  `planning.rs` (hooks + sanitizing), `extractor.rs`,
  `dependency_rewriter.rs`, `validator.rs` (+ `tests` submodule)
- `tests/fixtures/rust-basic/**` - one workspace root `Cargo.toml` + one
  crate (guide Step-4 properties: build-manifest the shaper recognizes;
  sources under `src/`; a test-side file under `tests/` that must be
  excluded; an external/generated directory - `target/` cannot be committed
  (discovery-skipped anyway), so carry the external side as a `vendor/`
  directory on the frozen side; module-tree specimen: `src/lib.rs` +
  `src/models/mod.rs`-style AND `src/greeting.rs`-sibling forms so both
  module layouts are exercised; one macro specimen that exercises the
  caveat honestly). Added via the documented frozen-set approval path
  (kickoff L3).
- `tests/integration/tests/rust_language_*.rs` - named for THIS milestone
  (guide's mNN-trap): discovery, dependency graph, semantic index, load
  analysis, symbol isolation, full-parity acceptance
  (`rust_language_parity_test.rs`). The planner names the exact roster.

MUST NOT EDIT: `crates/lang-cpp/**`, `crates/lang-scala/**`,
`crates/{index,graph,cluster,transform,validate,report,artifact,scheduler,ui}/**`
production or tests, `tools/**`, `tests/golden/**`, existing
`tests/fixtures/**`, theme assets, `docs/adr/**` (the guide rider in R-6
touches ONLY `docs/adding-a-language.md`), `m24_language_identity_test.rs`
(its Fortran mock already rehearses N=3; Rust's landing makes it real
around it - cite, never edit), `m20/m21/m22/m23` Scala suites.

## 5. Handoff-arc sketch (planner resizes; each sketch honors the law)

Serial. Hard edge: nothing precedes registration (R-1); the final parity
suite (R-6) needs everything. Enumeration amendments A1-A8 live in the
hand whose flip makes them red - ALL of them trip at R-1 (registration +
`.rs` flip), so they belong there (this relocates the brief's R-4 slot for
"enumeration-suite amendments" - recorded deviation from the mission
sketch, reason: a hand may not land its own red).

- R-1 (~600-850 lines, ~9-10 gates): spine edits (types/identity/
  modeling variant), root Cargo.toml + lock, crate skeleton (`lib.rs`,
  `plugin.rs` with discovery-only capabilities per INV-5, `detector.rs`,
  `shaper.rs`, `boundary.rs`, `planning.rs` naming helpers NOT yet hooked
  - hooks stay default-None until R-5 so planning capability stays false),
  registration line, fixture `rust-basic`, amendments A1-A8, discovery
  suite (INV-6) + identity byte test (INV-4) + grep-zero witness (INV-9).
  Single crate dir + the two integration test files + spine files; within
  law. If the planner finds it over the line ceiling, the natural split is
  spine+registration (1a) then fixture+suite (1b) - 1a alone must keep the
  workspace green (the amendments ride 1a because registration rides 1a).
- R-2 (~600-800 lines, ~8 gates): syn analyzer + symbol surface +
  fingerprint "rust-1" lock + semantic_index flip + span-literal tests
  (INV-7) + warm/cold cache witness.
- R-3 (~400-600, ~8): resolver (mod tree + use paths), graph-edge
  acceptance, dependency_graph flip, graph-shape tests.
- R-4 (~250-400, ~7): load_metric flip, `languages`-manifest assertion +
  CLI vocabulary print proof (INV-14 data path), UI map shows "Rust"
  (data-driven assertion, no edit).
- R-5 (~700-1000, ~9-10): planning hook overrides + extractor + rewriter
  (incl. RISK-MODTREE mechanism) + probe validator + final all-true
  capability literal lock (INV-5) + plan/preview/apply/rollback fixture
  suite (Scala m23-shape mirror). If over law: split rewriter+extractor
  (5a) from hooks+validator+flip (5b), 5b after 5a.
- R-6 (~300-500, ~8): `rust_language_parity_test.rs` full-parity
  acceptance + `docs/adding-a-language.md` dated riders (sec. 8) +
  closeout-facing ledger evidence packaging.

## 6. Risk register (each with its oracle)

RISK-SYN-SPAN - line/col exist only with `proc-macro2` `span-locations`; a
dep-table edit could drop it later. Mitigation: workspace-level feature
line + INV-7 exact-literal test (fails if spans vanish) + feature-gate
compile forcing. Oracle: INV-7 suite.

RISK-DETPRECEDENCE - registration could perturb first-wins detection.
Verified `.rs` unclaimed by both shipped detectors and Rust appends last;
mitigation is the order contract itself. Oracle: INV-2 (UNEDITED legacy
suites) + `detect_language_first_registered_claiming_plugin_wins` (exists,
unedited).

RISK-SELFSCAN (D-3) - the `.rs` flip makes own-tree scans index ~this
repo's Rust sources. Test surface stays fixture-only; bench/gate corpora
are fixture paths and `tools/**` is zero-diff, so no bench baseline moves
and no gate silently resizes its input. If any future load-perf decision
points at the own tree it must be deliberate (kickoff sec. 2/4). Oracle:
INV-13 + `bench compare: 3 run(s), no regressions past 1.25x baseline`
fresh at src-touching hops + fixture-path grep evidence in R-6.

RISK-MODTREE - an extracted Rust file is invisible to the crate until its
parent declares `mod <file>;`; Scala's `target_path_for` + rewriter idiom
alone produces unreachable code. Mitigation: the R-5 rewriter (or
extractor) must land the parent's `mod` declaration insert as part of the
transaction, asserted in the R-5 apply suite (parent file is a MODIFIED
file in the dry-run set - the grown-oracle discipline SM-4 introduced).
If the mechanism exceeds one hand: STOP-discipline precedent, planner
splits; if it proves un-shippable, the honest fallback is
`analysis_caveat`-adjacent documentation + keeping extraction scope to
items whose target file's parent already declares the module (planner
decision, not silent). Oracle: R-5 apply suite asserting parent-file
modification bytes + re-plan convergence.

RISK-AGGKIND - new lang-api variant could meet an exhaustive match in
core. Read-verified: no production `match` exists (construction sites
only; scheduler/graph occurrences are `#[cfg(test)]`). Appended at END, so
serde indices are preserved. Oracle: INV-12 + L1/L2/L4 green UNEDITED; any
surprise arm is a compile-time discovery = STOP-and-report.

RISK-CYCLE - `TolerateWithinPackage` for Rust reads as
"TolerateWithinCrate"; the vocabulary predates Rust (the enum lives in the
neutral seam; inventing a Rust-specific variant would leak a language into
shared vocabulary - the exact thing the neutrality guard exists for).
Stance is planner-informational + report-side only. Oracle: existing
SCC-sharing lock UNEDITED + a report-classification assertion in the
R-3/R-6 suites.

RISK-GUIDE-DEPS - the guide's "three deps and no more" snippet
(docs/adding-a-language.md:28-30, STALE ref note: its dispatch-site line
numbers at :23-25 are also stale - current sites verified at
plan.rs:203/optimize.rs:305/ui.rs:419, hooks already registry-dispatched
by AR-19) conflicts with D-2's parser deps. Resolution: deliberate,
user-ordered deviation; the m17 guards forbid only
`ariadne-graph/cluster/transform/validate` (manifest guard read-verified)
and scheduler/report/cluster/graph/transform/validate (transitive guard),
which third-party parser deps cannot reach; a dated rider in R-6
documents the deviation beside the snippet (sec. 8 rider 1).

RISK-HONEST-LIMITS-STALE - the guide's honest-limits section ("guard has
only ever seen two plugins"; "R1 has never been falsified by a language
that did not already exist here") is falsified-at-landing by this very
milestone. Rider (sec. 8 rider 3) updates it with a dated note - third
plugin exists, still this team, R1 exercised on a foreign language for the
first time; dynamic loading stays parked.

RISK-LOCKFILE - `Cargo.lock` regenerates at R-1; must be in that hand's
touch-set or review-gate's commit-vs-touch-set check trips (known
reviewer-rule trap, named here, not discovered mid-hand).

RISK-EMPTY-CLAIMANT - none: capability resolution is language-scoped
(`capability_backend_for_language`, read-verified), `incremental_backend`'s
multi-language error is unchanged behavior, and mixed cpp+rust repos keep
the honest M21 error - the milestone does not pretend to fix dispatch it
never claimed. If a handoff brief claims "rust + cpp in one repo plans
end-to-end", it is FALSE per current tree; the fixture is single-language.

## 7. Hygiene notes (planned, not discovered late)

No touched file is near a ceiling: largest edited is
`cli/src/registry.rs` at 761 (+~30, margin ~210); `identity.rs` 401
(+~40); `m17_plugin_seam_test.rs` 740 (+4); `types.rs` 304; the rest
comfortable. New `lang-rust` files: mirror Scala's granularity (its
largest production files stay well under 1000); the syn walker is the one
file plausibly trending long - pre-split it (`parser_items.rs` /
`parser_uses.rs`) the moment `analyzer.rs` crosses ~600. All functions
< 100 lines; no `tmp`/`res`/single-letter locals (R3 evidence, kickoff
sec. 7); authored content pure ASCII; the fixture's macro specimen stays
ASCII.

## 8. Docs riders (land at R-6, `docs/adding-a-language.md` only)

1. Crate-manifest snippet: dated rider recording lang-rust's deliberate
   parser-deps extension (syn/proc-macro2[span-locations]/unicode-ident,
   workspace-pinned), why (D-2 correctness guarantee), and what the m17
   guards still forbid (the four-name list + transitive list) - the
   trio-for-three rule stays the lang-scala baseline, not a universal cap.
2. Dispatch-sites paragraph: line refs refreshed to 203/305/419, the
   AR-19 hook-dispatch state, and the informational CyclePolicy stance
   ("a language enrolls by overriding the trait hooks; these literals
   stayed").
3. Honest-limits: dated third-plugin update (RISK-HONEST-LIMITS-STALE).
4. Guard wording: "PLUGIN_CRATES is a two-element list" -> three (the
   append-discipline from INV-11 named as the pattern).
5. Step 5: name this milestone's suites (`rust_language_*`) as the
   milestone-named example of the mNN-trap rule.

## 9. Existing evidence this seam handles N=3

`tests/integration/tests/m24_language_identity_test.rs` (Fortran mock
third plugin, "unhandled language" proof - read-verified; UNEDITED).
Rust's landing converts the rehearsal into a shipped third language; the
milestone adds its own real-plugin identity assertion (INV-4) so the
promise is carried by shipped code, not only by the mock.
