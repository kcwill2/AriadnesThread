# ADR-0002: Symbol placement and the isolation primitive

- Status: Accepted
- Date: 2026-10-03
- Applies to: `crates/lang-api` (`SymbolPlacement`, `LanguageModeling`), `crates/graph` (reduced model), `crates/transform` (isolation engine), `crates/cluster` (planner), both plugin crates
- Supersedes: nothing
- Spec source: `plans/multi-language-support-implementation_plan.md` sections 0.1, 3.6, 3.6.1; decision points D6 and D7

## Context

C++ is special among the target languages: a symbol's *declaration* lives in an aggregation
artifact while its *definition* may live in a different source file. Most modern languages
(Scala, Go, TS, Python) are single-file placement -- one file per top-level symbol, declaration
and definition inseparable. The original model papered over this by assuming the two were the
same, "usually", which is a C++ accident dressed as a universal rule.

The product's core move is therefore not "extract a C++ header". It is *move symbol S out of
shared file F into its own file F_S, so consumers that need only S stop depending on F*. For C++
this shrinks the include closure; for single-file languages it shrinks import granularity -- and
in both cases it makes the symbol-to-file relation 1:1, which is what makes the per-file load
metric actionable at all. A metric that measures "what does this file cost me" is only
actionable if the thing being costed is addressable on its own.

## Decision

### 1. Symbol placement is modeled, not assumed

`LanguageModeling` (`crates/lang-api/src/modeling.rs`) carries a `symbol_placement` field whose
value is one of exactly two `SymbolPlacement` variants. The variant names, as declared in that
file, are:

- `SymbolPlacement::SplitDeclaration` -- the declaration and the definition live in different
  files. `crates/lang-cpp/src/plugin.rs` declares this.
- `SymbolPlacement::SingleFile` -- declaration and definition are inseparable. Declared by
  `crates/lang-scala/src/plugin.rs`.

Both plugins' declarations are locked by an in-crate assertion, and `LanguageModeling` round-trips
through serde with a dedicated test for the two placement values, so a plugin cannot quietly
disagree with the vocabulary it is stored under.

**One honest limit, stated because it changes what the field currently buys you:** no production
code path branches on `LanguageModeling::symbol_placement`. Every occurrence in the workspace is
either a plugin declaring its own value, a test asserting that value, or a re-export. This is a
deliberate M23 ruling, not an oversight: kickoff decision D-M23-7 (iv) records that placement
semantics "are realized by each language's filter/target-pair", and that `SymbolPlacement` "needs
no new planner hook". The field is therefore the *declared, tested record* of what each language
means by placement, and the behaviour is carried by the plugin hook pair described in Decision 3.
A later reader should treat the field as the contract and the hooks as the mechanism, and should
not assume a runtime dispatch on the field is waiting to be used.

**On the "no extension check" claim, scoped honestly.** The blanket statement that no extension
test or `.h` suffix check appears anywhere in the core crates is **not** true, and this record
does not make it. Three narrower statements are each verified:

- The literal `".h"` does not appear in the `src/` tree of any of the seven core crates the
  neutrality guard walks (`crates/index`, `crates/graph`, `crates/cluster`, `crates/transform`,
  `crates/validate`, `crates/report`, `crates/pipeline`). That is mechanically enforced: the
  token list in `tests/integration/tests/m17_plugin_seam_test.rs` includes `".h"` alongside
  `#include`, `Cpp`, and the plugin crate names, and the guard fails the build on any hit.
- No production path in the isolation pipeline chooses the isolation unit by inspecting a path
  suffix. The unit comes from plugin-supplied hooks (Decision 3), and the provenance that
  identifies a symbol's files comes from recorded declaration locations (Decision 4), not from a
  filename shape.
- Extension classification does exist, in two places a reader should know about rather than
  discover. `crates/repo` is the repository spine and legitimately maps a discovered file's
  `FileKind` to a `SourceLanguage`; it matches on the `FileKind` enum, not on a path suffix, and
  it is deliberately outside the neutrality guard's walk scope because the spine must be able to
  name languages. Separately, `crates/graph/src/builder.rs` contains lowercase `"cpp"` extension
  checks inside its `#[cfg(test)]` test-seam module, where a stub resolver and a stub detector
  impersonate a language to exercise the seam. Those are test fakes in a guarded crate, not
  production logic, and the guard's token list does not match lowercase `"cpp"`.

The distinction that matters: classifying *which language a file belongs to* is a repository-level
concern and is allowed to exist. Inferring *where a symbol's declaration and definition live* from
a filename is the accident this decision removes.

### 2. Isolation means "move the symbol into its own file"

The primitive is language-neutral in the only sense that survives contact with a second language:
the *plan* is one plan, the *engine* is one engine, and the *verification* is one verification.
The spec's division-of-labor table is transposed below to past tense, recording where each piece
lives now, post-landing. The spec remains the historical record of the intent; where the two
differ, this table wins, because it describes shipped code.

| Piece | Where it lives now | Notes |
|---|---|---|
| Isolation engine: group plan extractions per target file, compute inter-symbol dependency edges over the reduced model, cycle-check the target graph, write files | `crates/transform`, as `IsolationEngine` (stays neutral) | Owns grouping, the unit-cycle check, and the disk write. Names only the neutral `SourceExtractor` hook; the module's own doc comment states that the new-file *text* is built by a language plugin |
| New-file text rendering (guard plus directives for the new unit) | plugin: `SourceExtractor` (`crates/lang/src/extractor.rs`) | The trait carries `extract` plus the hooks that grew around it: `unit_namespace`, `locate_symbol_bodies`, `unit_imports`, and `places_generated_units_in_place`. C++ defaults are byte-identical to pre-M23 output |
| Consumer redirect: which old dependencies a consumer still needs, and the directive *text* rewriting | engine computes the neutral rewrite decision; the text lives behind the plugin's `DependencyRewriter` | `DependencyRewriter` also owns consumer discovery and the `remove_top_level_symbols` hook that makes a single-file language's isolation a move rather than a copy |
| Target file naming | plugin hook, injected into the planner | `crates/cluster` takes a `target_path_for` function pointer; `crates/lang-cpp` supplies one producing `{symbol}_extracted.h`, `crates/lang-scala` supplies one producing `{Symbol}.scala` beside the aggregation file |
| Validation tiers | already neutral; tier 3 is a probe-backed skip | Tier 3 never invokes a compiler. See Consequences, and amendment D-M23-12 |

**The invariant that makes this safe:** a core crate must not learn a language's file-naming
convention or directive syntax in order to do any of the above. The engine receives a plan and a
set of trait objects; the plugin decides what a generated file is called and what its text says.
This is the same boundary ADR-0001 fixes from the plugin side, and it is deliberately restated
here as a load-bearing condition for the isolation primitive specifically, not restated as ADR
history.

### 3. The isolation unit differs per language

This is the counter-intuitive part, so it is stated as a table rather than a paragraph:

| Language | Placement | What gets isolated | Why | Definitions |
|---|---|---|---|---|
| C++ | `SplitDeclaration` | the **declaration site** -- extract the declaration out of the aggregation artifact | that declaration is what consumers actually pay for; it is the thing in the include closure that drags in collateral symbols | stay in their source files; C++ isolation is copy-only |
| Single-file languages (Scala today) | `SingleFile` | the **whole top-level symbol** -- the file becomes its own file | declaration and definition are inseparable, so there is nothing else to extract; the win is import granularity, and the moved region leaves the source file | move with the symbol, inside the snapshot/rollback transaction |

The consequence is the point of the whole record: **the same plan, the same engine, and the same
verification produce the right isolation unit for each language, because the plugin supplies the
answer.** No core crate contains a branch of the form "if the language is X, isolate the
declaration instead of the symbol". The two answers differ in kind -- one is a slice of a shared
file, the other is a whole file plus a deletion -- and that difference is carried entirely by the
plugin's aggregation filter and target-path hook, plus, for single-file languages, the rewriter's
removal hook.

Because C++ isolation stays copy-only and single-file isolation is a move, the two languages also
differ in what a rollback has to restore. That asymmetry is a property of placement, not an
inconsistency in the engine: the C++ path never removes anything, so it has nothing to undo in the
source tree, while the single-file path removes the moved region and relies on the existing
file-snapshot transaction to put it back.

### 4. Symbol-to-file provenance is recorded, not inferred

`ReducedSymbolRecord` in `crates/graph/src/reduced.rs` no longer equates a symbol's declaring file
with its defining file. It carries two distinct fields:

- `defining_file` -- the file hosting the accepted *definition* declaration. It is resolved from
  the defining declaration's own recorded file location, replacing the old shortcut that simply
  used the owning file. When the defining declaration's file location cannot be used, it falls
  back to the owning file. The code is precise about when, and all three conditions are real:
  the `(symbol, decl)` pair is absent from the translation unit's declaration registry; the pair
  carries only the `FileId(0)` sentinel location that means "no location recorded"; or the pair
  resolves ambiguously to several files, which happens because the parser's per-file declaration
  ids collide across files once a translation unit is merged. All three are covered by named unit
  tests in that file.
- `declaring_files` -- every file hosting any merged declaration of the symbol, **deduplicated and
  sorted**. Sorting is not cosmetic: it is what makes the stored bytes deterministic across runs,
  and a cross-TU dedup-and-sort test locks it.

**The artifact trap, stated because it will bite the next person.** The reduced symbol blob is
**bincode, not JSON** -- `artifacts/reduced/symbols.bin` is written with `bincode::serialize` in
`crates/report/src/manager/graph_io.rs`. So `#[serde(default)]` does **not** rescue a stale blob.
Serde defaults work by name, and a bincode blob carries no field names; a record whose layout grew
a field deserializes as shifted garbage or fails outright, and a default cannot fill in a field
the reader never reaches. Adding `declaring_files` was safe *only because* the cache epoch was
bumped: `CURRENT_ANALYSIS_VERSION` went from `1.0.0` to `"2.0.0"` in `crates/index/src/cache.rs`,
and `check_manifest` in `crates/report/src/manager.rs` rejects a stale epoch before any blob is
deserialized, forcing regeneration. The constant's own doc comment says so.

The general rule this establishes: **a change to a stored bincode struct's field set requires a
cache-epoch bump in the same change.** `#[serde(default)]` is not a substitute and must not be
cited as one in a review of such a change. Contrast this with `ProposedExtraction::source_file` in
`crates/cluster/src/plan.rs`, which *is* `#[serde(default)]` and *is* safe -- because plan
artifacts are JSON. The distinction is the format, not the field.

## Consequences

**Positive.**

- The per-file load metric becomes trustworthy, because the symbol-to-file relation it reasons
  about is recorded rather than guessed. "Defined in the `.cpp`, declared in the header" is a fact
  the model now carries, and a single-file language honestly reports one file.
- C++ behaviour was preserved through the neutrality work rather than reimplemented. M23 could then
  feed real symbol bodies for both languages without re-litigating C++ semantics: verbatim
  declaration text reaches the plugin extractor, and the C++ preview and diff byte-locks stayed
  identical.
- A third language reaches isolation without a core edit. It writes a plugin, supplies a
  `LanguageModeling` with a `symbol_placement` value, and supplies the hook pair that turns that
  declaration into behaviour.
- Provenance survives the artifact round trip, and its dedup-and-sort makes the stored bytes
  reproducible rather than order-dependent.

**Accepted costs and known limits, taken from the M23 record.** These are the honest boundaries of
what the primitive is proven to do, and they are recorded here so no later reader mistakes absence
for a ruling.

- **Extracted units are not self-contained.** They can reference types outside their own
  dependency set. This is reviewer finding F3 from M23's Gate-G1, and it is the reason the next
  bullet is a decision rather than an omission.
- **Real compiler invocation was deliberately NOT wired, for both languages.** The spec's planned
  division-of-labor table says "C++ tier 3 = real compile"; amendment **D-M23-12** in
  `plans/m23-symbol-isolation-kickoff.md` supersedes that, after F3 established that a real
  syntax-only pass over generated units would deterministically fail on the current fixtures. The
  only honest fix is include-closure computation at extraction time, which is a workstream of its
  own. **Tier-3 validation therefore stays `Skipped` for both languages offline and does not run
  a compiler.** What ships instead is probe-backed honesty: when a plugin validator's
  `toolchain_status` reports `Missing`, the tier reports `Skipped` rather than a `Pass` it cannot
  justify, and the Scala plugin supplies a real probe so that offline outcome is *reported*
  honestly rather than assumed. The mapping lives in `crates/validate/src/lib.rs`, and the
  doc comment on that function cites D-M23-12 rather than repeating a plan line that no longer
  holds. An `Unknown` or absent validator still yields `Pass`, because absence is unproven; only a
  proven `Missing` downgrades.
- **The `source_file` provenance refinement for the planner is deferred.** Both Gate-G3 lanes
  ruled it *incomplete, not incorrect*, and it was deferred because the planner-side rework churns
  the C++ plan goldens, which is out of milestone scope by the milestone's own intent rule. The
  observable effect: for one symbol on the Scala fixture, the proposed extraction's `source_file`
  is mislabeled at the candidate rather than at the member's defining file, that unit extracts as
  a placeholder body, and the per-unit move strip skips it. The moved region and the defining site
  are still safe -- a placeholder unit is an inert comment, there is no duplicate definition, and
  the all-or-nothing removal protects the defining site. The consumer-import narrowing over
  placeholder-unit ids rides the same follow-up.
- **`SymbolPlacement` is declared but not dispatched on** in production, as recorded in Decision
  1. The contract is live and tested; the branching deliberately does not exist.
- **The placeholder body is still the total fallback** when a body cannot be located. That is a
  deliberate, visible, deterministic fallback rather than a silent omission.

## What this ADR does not decide

The following are follow-up work, not settled decisions. Absence below is a deferral with an owner
named in the M23 backlog, not a ruling, and none of it should be read as having been considered
and rejected.

- **Real compiler invocation and the extraction include-closure computation that would make it
  possible.** Together these are what would let tier 3 report a genuine `Pass` instead of
  `Skipped`, and they carry a golden shift when `Skipped` stops being the offline outcome. The
  condition that must be met first is the one above: extracted units are not yet self-contained.
- **A toolchain-backed Scala analyzer** for exact type resolution. The Scala analyzer that ships is
  an offline heuristic parser and carries an explicit analysis caveat saying so; a toolchain-backed
  analyzer is deferred to a follow-up milestone because there is nothing to exercise offline.
- **Package-scoped cycle precision**, including strongly-connected-component condensation. The
  shipped Scala cycle policy tolerates cycles within a package; condensing them is follow-up work.
- **Dynamic or out-of-process plugin loading.** Not decided here and not restated -- see ADR-0001,
  which carries the full question, the explicit gate for un-parking it, and the reason it needs its
  own record: it changes how the boundary is *enforced*, not merely how plugins are loaded.
- **Whether the isolation primitive extends to a third language.** The pattern is evidenced for
  C++ and Scala only.
