# Architecture & Design Specification: High-Throughput C++ Header Optimization & Dependency Decoupling Tool

---

## 1. Overview & Problem Statement

Large-scale C++ codebases suffering from long build times often contain bloated "god headers." When headers define numerous loosely coupled types alongside heavy internal or third-party dependencies, downstream translation units (TUs) endure severe transitive compilation overhead, even when consuming only a tiny fraction of the definitions.

This tool automates the detection, analysis, clustering, extraction, and migration of symbols into lightweight, decoupled sub-headers.

The system is fundamentally a **dependency-load optimization system**, rather than merely a header-clustering algorithm.

Its purpose is to:

1. Discover the repository and actual compiler configuration.
2. Build a semantic model of symbols and their dependencies.
3. Build a separate model of textual include relationships.
4. Identify semantic versus collateral compilation load.
5. Identify optimization policies and frontiers.
6. Construct safe refactoring plans that reduce collateral load.
7. Apply transformations transactionally.
8. Validate semantic correctness.
9. Measure the actual compilation benefit.
10. Report both the optimization and the remaining residual collateral load.

The implementation is primarily Rust, with a Clang-based semantic backend providing C++ semantic analysis.

---

# 2. System Architecture

The system consists of six distinct phases:

```text
Phase 0: Repository & Build Discovery
                ↓
Phase 1: Compilation Context & Include Analysis
                ↓
Phase 2: Parallel Semantic Indexing
                ↓
Phase 3: Hash-Partitioned Shuffle & Reduction
                ↓
Phase 4: Dependency Analysis & Optimization Planning
                ↓
Phase 5: Transformation, Validation & Measurement
```

Detailed flow:

```text
┌──────────────────────────────────────────────────────────────┐
│                  Phase 0: Discovery                          │
│                                                              │
│ Source Tree   MSBuild   cl.exe   MSVC STL   Windows SDK      │
│ Ownership     Policies  Compile Commands                     │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│            Phase 1: Context & Include Analysis               │
│                                                              │
│ Compilation Contexts   Include Graph   TU Schedule           │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│             Phase 2: Parallel Semantic Indexing              │
│                                                              │
│  [Worker 0]       [Worker 1]       [Worker N]                │
│   TU → Shard       TU → Shard       TU → Shard               │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│             Phase 3: Shuffle & Parallel Reduction            │
│                                                              │
│ [Bucket 0]        [Bucket 1]        [Bucket K]               │
│ [Reducer 0]       [Reducer 1]       [Reducer K]              │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│          Phase 4: Dependency Analysis & Planning             │
│                                                              │
│ Semantic Graph  Include Graph  Load  Policies  Clusters      │
│ SCCs            Frontiers     Plans                          │
└──────────────────────────────┬───────────────────────────────┘
                               │
                               ▼
┌──────────────────────────────────────────────────────────────┐
│       Phase 5: Transformation / Validation / Measurement     │
│                                                              │
│ Plan → Preview → Apply → Validate → Measure → Commit/Rollback│
└──────────────────────────────────────────────────────────────┘
```

---

# 3. Phase 0: Repository & Build Discovery

Phase 0 establishes the universe of files, build configurations, compiler environment, ownership, and optimization policy.

## 3.1 Source Discovery

The repository filesystem is authoritative for source discovery. `.sln`, `.vcxproj`, `.props`, and `.targets` are build-configuration inputs, not the canonical source inventory.

Useful primitives include:

* filesystem traversal
* `rg`
* extension filtering
* ignore rules
* textual `#include` discovery
* generated-file detection

Typical source extensions:

```text
.h
.hpp
.hxx
.cpp
.cc
.cxx
.inl
.ipp
```

Textual analysis is useful for discovery and optimization, but semantic truth comes from the compiler-backed semantic backend.

## 3.2 Build Discovery

The tool should preferably operate from actual compiler commands captured from MSBuild.

A tool such as Catter may be used to observe `cl.exe` invocations and produce:

```text
compile_commands.json
```

This avoids requiring the optimizer to implement a complete MSBuild evaluator.

## 3.3 File Model

```rust
struct FileRecord {
    id: FileId,
    path: PathBuf,
    canonical_path: PathBuf,
    kind: FileKind,
    size: u64,
    content_hash: Hash,
    ownership: FileOwnership,
    optimization_policy: OptimizationPolicy,
}
```

---

# 4. Compiler Environment & MSVC Discovery

The tool must discover the actual compiler environment rather than hardcoding Visual Studio paths.

```rust
struct CompilerEnvironment {
    compiler_path: PathBuf,
    toolset_version: String,
    standard_include_roots: Vec<PathBuf>,
    sdk_include_roots: Vec<PathBuf>,
    external_include_roots: Vec<PathBuf>,
}
```

The environment should be derived from actual compiler configuration whenever possible.

This is important for:

* MSVC STL headers
* Windows SDK headers
* compiler intrinsic headers
* toolset-specific headers
* architecture-specific headers

Compiler/toolchain files are analyzable for dependency accounting but are normally not transformation targets.

Ownership is determined from resolved canonical paths rather than filename patterns.

---

# 5. Ownership & Optimization Policy

Ownership and optimization policy are deliberately separate concepts.

## 5.1 File Ownership

Ownership answers:

> Who controls or produces this file?

```rust
enum FileOwnership {
    Repository,
    ThirdParty,
    Toolchain,
    Generated,
}
```

* `Repository` — source controlled by the project/repository.
* `ThirdParty` — vendored or externally maintained dependencies.
* `Toolchain` — MSVC STL, Windows SDK, compiler-provided headers, and similar toolchain content.
* `Generated` — produced by a generator rather than directly maintained source.

## 5.2 Optimization Policy

Optimization policy answers:

> What is the optimizer permitted to change?

```rust
enum OptimizationPolicy {
    Optimizable,
    Contract,
    Frozen,
}
```

A repository-owned public API header can therefore be:

```text
Ownership: Repository
Policy: Contract
```

while a third-party header is normally:

```text
Ownership: ThirdParty
Policy: Frozen
```

This separation permits combinations such as `Repository + Frozen` or future `Generated + Optimizable` policies without conflating ownership with transformation permission.

## 5.3 Default Policies

Recommended defaults:

```text
Repository   → Optimizable
ThirdParty   → Frozen
Toolchain    → Frozen
Generated    → Frozen
```

Repository-owned files can be overridden explicitly.

## 5.4 Directory Policies

Whole directories may define inherited optimization policy.

```yaml
optimization:
  default: optimizable

  paths:
    "include/public/**": contract
    "third_party/**": frozen

  directory_names:
    generated: frozen
```

A directory-name rule applies to any directory with that name anywhere in the repository:

```text
project/
├── generated/                    FROZEN
├── src/
│   ├── generated/                FROZEN
│   └── foo/
│       └── generated/            FROZEN
├── modules/
│   └── graphics/
│       └── generated/            FROZEN
└── third_party/                  FROZEN
```

Policy precedence:

1. Explicit file override
2. Most-specific path/glob
3. Directory-name rule
4. Inherited parent-directory policy
5. Repository default

## 5.5 Optimization Frontier

An optimization frontier is not an ownership category or policy. It is the point in the dependency graph where an optimization can intervene without crossing a frozen or contract region.

For example:

```text
App.h
  ↓
GraphicsAdapter.h
  ↓
HugeSDK.h
```

If `HugeSDK.h` is:

```text
Ownership: ThirdParty
Policy: Frozen
```

the optimizer may identify `GraphicsAdapter.h` as the optimization frontier.

The policy explains why the dependency cannot be modified; the frontier identifies where intervention can occur.

---

# 6. Phase 1: Compilation Context & Include Analysis

## 6.1 Compilation Contexts

C++ meaning depends on compiler configuration and preprocessing state. The system therefore models compilation contexts explicitly.

```rust
struct CompilationContext {
    id: ContextId,

    compiler_path: PathBuf,
    toolset_version: String,

    target_triple: String,
    architecture: String,

    language_standard: String,

    include_paths: Vec<PathBuf>,
    system_include_paths: Vec<PathBuf>,
    forced_includes: Vec<PathBuf>,

    preprocessor_definitions: Vec<(String, Option<String>)>,

    configuration: String,
}
```

## 6.2 Explicit Preprocessor Inputs

The optimizer does not need to model arbitrary hypothetical preprocessor states.

The relevant preprocessor definitions are inputs from the actual compiler commands. This makes preprocessing deterministic for each compilation context.

For the initial target codebase, the principal configuration variation is expected to be:

```text
DEBUG
NDEBUG
```

For example:

```text
Debug:
    DEBUG=...
    NDEBUG absent

Release:
    NDEBUG=...
    DEBUG absent
```

Their exact presence and values come from the captured compile commands.

Other definitions can be added if they materially affect the codebase.

If different contexts produce different semantic models, those differences are preserved. A transformation spanning multiple contexts must be valid for every relevant context.

## 6.3 Include Graph

The system maintains a separate file-level include graph:

```text
IncludeGraph:
    File → File
```

An edge records that preprocessing the source file causes the destination file to be included under the relevant context.

```rust
struct IncludeEdge {
    source: FileId,
    destination: FileId,
    context: ContextId,
    source_location: SourceLocation,
}
```

Conditional behavior is represented by the contexts in which an edge is observed, rather than by exploring arbitrary macro combinations.

## 6.4 PCH Model
PCH is explicitly out of scope for the initial implementation.

The target codebase does not use precompiled headers, so the optimizer models
translation-unit-local preprocessing and compilation directly.

If PCH support is required for another codebase in the future, it should be
introduced as an additional compilation-cost model rather than changing the
core semantic dependency model.

## 6.5 TU Context Resolution

For every translation unit:

```text
TU
 ├── CompileCommand
 ├── CompilationContext
 ├── IncludePaths
 ├── ForcedIncludes
```

This tuple becomes the canonical semantic input identity.

---

# 7. Data Model & Wire Formats

To minimize memory contention and support zero-copy deserialization and efficient disk/network serialization, AST workers emit immutable, versioned binary artifacts.

## 7.1 Binary Edge Schema

```cpp
#pragma pack(push, 1)
struct DiskEdge {
    uint64_t src_usr_hash;
    uint64_t dst_usr_hash;
    uint32_t src_file_id;
    uint32_t dst_file_id;
    uint8_t  dep_kind;     // 0 = Incomplete, 1 = Complete
};
#pragma pack(pop)
```

Production artifacts must carry a schema version and, where required, context and provenance information.

## 7.2 Auxiliary Lookup Tables

* **String Interning Delta:** append-only dictionary mapping hashes to USR/path strings, deduplicated during reduction.
* **TU Usage Bitmaps:** for each indexed symbol, downstream TU usage is stored as a Roaring Bitmap.

```text
TUs(symbol) = { tu_id1, tu_id2, ... }
```

## 7.3 Canonical Symbol Model

USR-based identity is the foundation for cross-TU symbol identity, but logical symbols must be distinguished from concrete declarations and definitions.

```rust
struct Symbol {
    id: SymbolId,
    kind: SymbolKind,

    declarations: Vec<DeclId>,
    definition: Option<DeclId>,

    owning_file: FileId,
}
```

The model must distinguish:

```text
logical symbol
declaration
definition
redeclaration
specialization
instantiation
```

---

# 8. Phase 2: Parallel Semantic Indexing

The semantic backend analyzes each TU under its resolved compilation context.

The backend must emit:

* canonical symbol identities
* declarations and definitions
* symbol-to-symbol dependencies
* dependency kind
* source locations/provenance
* template relationships
* macro dependencies where relevant
* TU usage
* file/symbol relationships
* side-effect information where detectable

The implementation should expose a backend abstraction:

```rust
trait CppSemanticBackend {
    fn analyze_tu(
        &self,
        command: &CompileCommand,
    ) -> Result<TuAnalysis>;
}
```

Clice is the initial implementation candidate, but the architecture refers to the abstraction as a **Clang-based semantic backend**.

---

# 9. Semantic Graph & Dependency Classification

The semantic graph is distinct from the include graph:

```text
SemanticGraph:
    Symbol → Symbol
```

A semantic edge means that the source symbol requires the target according to C++ semantics.

Each edge records dependency strength:

```text
Incomplete
Complete
```

## 9.1 Complete Dependency

Examples:

* base classes/inheritance
* member variables stored by value
* `sizeof(T)`
* `alignof(T)`
* `typeid(T)`
* direct member access requiring the target definition
* explicit down-casts/cross-casts requiring completeness
* inline method bodies
* template definitions/instantiations requiring completeness

## 9.2 Incomplete Dependency

Examples:

* pointers
* references
* function parameter types in declarations
* return types in declarations
* other forward-declarable uses

Dependency classification describes semantic requirements. It does **not** by itself determine whether a symbol can safely be moved.

---

# 10. Template & ODR Model

## 10.1 Template Relationships

Templates require explicit treatment of:

```text
Template Declaration
Template Definition
Explicit Specialization
Partial Specialization
Implicit Instantiation
Explicit Instantiation
```

The optimizer must determine whether an extracted header requires a declaration, complete definition, specialization, or explicit instantiation construct.

## 10.2 Symbol Movability

A separate movability classification is required:

```rust
enum Movability {
    FreelyMovable,
    DependencyConstrained,
    ContextSensitive,
    NonMovable,
}
```

Factors include:

* inline definitions
* templates
* explicit specializations
* internal/static linkage
* anonymous namespaces
* constexpr variables
* registration mechanisms
* initialization behavior
* macro dependence
* source-order requirements
* ODR constraints

A semantically valid dependency graph does not imply that a symbol is safe to extract.

---

# 11. Macro & Header Side-Effect Model

Header semantics are not purely a symbol graph.

## 11.1 Macro Dependencies

At minimum model:

```text
Macro → Declaration
Macro → Use
Macro → ConditionalRegion
```

The actual macro environment comes from the explicit compilation context. The initial implementation therefore analyzes the concrete environments actually used by the build rather than attempting to enumerate hypothetical macro combinations.

## 11.2 Header Side Effects

Detect and classify, where possible:

* `#define` / `#undef`
* pragmas
* `#pragma comment`
* static registration objects
* global initialization
* compiler-specific directives
* intentional inclusion-order behavior

Suggested classification:

```text
PureDeclarative
MacroProducing
PragmaProducing
RegistrationBearing
InitializationBearing
OrderSensitive
Unknown
```

Side-effect-bearing headers constrain extraction and consumer include rewriting.

---

# 12. Phase 3: Shuffle & Parallel Reduction

## 12.1 Process-Level Isolation

Independent worker processes provide:

* memory reclamation
* failure isolation
* straggler containment
* predictable resource limits

Translation units should be scheduled using estimated complexity so large TUs begin early.

## 12.2 Hash-Partitioned Shuffle

Workers route records using a deterministic hash such as:

```text
src_symbol_hash % K
```

Each reducer owns one bucket.

This permits local deduplication, symbol aggregation, and Roaring Bitmap construction without a shared mutable graph.

## 12.3 Reduced Semantic Model

The reduction stage produces a canonical model containing at least:

```text
Symbol Graph
Include Graph
File/Symbol Index
Declaration/Definition Index
TU Usage
Compilation Contexts
Macro/Side-Effect Metadata
```

---

# 13. File/Symbol Index

Maintain:

```text
File → Symbols
Symbol → Owning File
```

This enables the optimizer to distinguish symbols semantically required by a consumer from symbols made available only because of the current file/include structure.

---

# 14. Dependency Load Model

Every symbol has a dependency load consisting of:

1. The symbol itself.
2. Its valid semantic dependency closure.
3. Collateral dependencies introduced by the current include structure.

The conceptual model is:

```text
Symbol
  ↓
FileOf(Symbol)
  ↓
IncludeClosure
  ↓
File Symbols
  ├── Semantic Closure
  └── Collateral
```

## 14.1 Semantic Dependency Load

Dependencies required by C++ semantics.

## 14.2 Include Load

Files/symbols made available by the current textual include structure.

## 14.3 Collateral Load

Include load that is not part of the symbol's semantic dependency closure.

## 14.4 Residual Load

Residual load is **collateral load that remains after an optimization/refactoring step**.

It is not a fourth dependency type.

Example:

```text
Before:

Semantic load:       120
Collateral load:   4,880
Total load:        5,000


After:

Semantic load:       300
Collateral load:   1,200
Total load:        1,500
```

Residual collateral load is therefore:

```text
1,200
```

Residual load may be intentional or acceptable. It simply represents collateral load remaining after the optimization.

---

# 15. Formal Load Metrics

"Load" must be measurable at several levels.

## 15.1 Structural Load

Track:

```text
reachable files
reachable symbols
include closure size
semantic dependency closure size
collateral file count
collateral symbol count
```

## 15.2 Estimated Compilation Load

Where available, estimate:

```text
preprocessed token count
AST node count
unique declarations processed
template instantiation count
```

## 15.3 Measured Compilation Cost

Measure before/after:

```text
front-end CPU time
front-end wall time
peak memory
preprocessing time
full compilation time
```

Conceptually:

```text
Structural Load
      ↓
Estimated Compilation Load
      ↓
Measured Compilation Cost
```

Structural metrics guide optimization. Measured compilation cost determines whether the optimization produced an actual build benefit.

---

# 16. PCH-Aware Load Analysis
PCH is explicitly out of scope for the initial implementation.

The target codebase does not use precompiled headers, so the optimizer models
translation-unit-local preprocessing and compilation directly.

If PCH support is required for another codebase in the future, it should be
introduced as an additional compilation-cost model rather than changing the
core semantic dependency model.

---

# 17. Optimization Frontiers

For each significant collateral dependency, the system should be able to explain:

* why the dependency is present
* why it is collateral
* why the target cannot be modified
* where intervention is possible
* estimated potential reduction

Example:

```text
HugeSDK.h

Why here?
    GraphicsAdapter.h → HugeSDK.h

Why collateral?
    No semantic dependency

Can we remove it?
    No — ThirdParty + Frozen

Optimization frontier:
    GraphicsAdapter.h

Potential structural reduction:
    8,412 symbols
```

The frontier is a graph concept, not a file classification.

---

# 18. Confidence Model

Not all analysis results have identical certainty.

```text
Exact
Conservative
Incomplete
Unknown
```

Potential causes of reduced confidence include:

* incomplete compilation database
* unsupported compiler extensions
* complex macro behavior
* generated code
* unknown include search behavior
* semantic backend limitations

Transformations require sufficient confidence.

Reports must expose uncertainty rather than hiding it.

---

# 19. Incremental Analysis & Caching

Large repositories cannot be reanalyzed wholesale after every change.

Semantic cache keys should incorporate at least:

```text
source content hash
compile command hash
compiler/toolset version
semantic backend version
analysis configuration
```

Conceptually:

```text
AnalysisKey =
    Hash(
        source,
        compile_command,
        compiler,
        toolset,
        backend,
        configuration
    )
```

Cache layers should include:

```text
Repository discovery
Include scan
AST/TU analysis
Reduced graph
Optimization plan
Validation
```

Changing a source or semantic input invalidates affected downstream artifacts according to their dependency relationships.

---

# 20. Phase 4: Dependency Analysis & Optimization Planning

Phase 4 transforms the reduced semantic model into candidate optimization plans.

It does **not** directly mutate source files.

## 20.1 SCC Condensation

Run Tarjan's SCC algorithm on the semantic graph.

Mutually dependent symbols become indivisible super-nodes, preventing cyclic semantic relationships from being split.

## 20.2 Multi-Objective Affinity

For symbols `u` and `v` within a header, affinity may include:

$$
W(u,v)
=
\alpha \cdot Structural(u,v)
+
\beta \cdot J(u,v)
+
\gamma \cdot E(u,v)
$$

where:

```text
Structural
    direct semantic relationship

J
    TU co-usage using Roaring Bitmap intersection/union

E
    external dependency overlap
```

Default starting weights may be:

```text
α = 0.50
β = 0.35
γ = 0.15
```

These are heuristic inputs, not the final optimization objective.

## 20.3 Load-Based Optimization Objective

The actual objective is to maximize expected compilation-load reduction subject to:

```text
semantic correctness
ODR correctness
context correctness
optimization policies
movability constraints
DAG constraints
header self-containment
acceptable cluster size
side-effect constraints
```

Thus:

```text
Affinity
   ↓
Candidate clustering
   ↓
Estimated load reduction
   ↓
Constraint validation
   ↓
Optimization plan
```

## 20.4 Constrained Agglomerative Merging

1. Initialize each SCC as an independent cluster.
2. Evaluate candidate merges using affinity and expected load reduction.
3. Reject merges violating semantic, context, movability, policy, side-effect, or DAG constraints.
4. Stop when the objective no longer improves sufficiently or cluster limits are reached.

The cluster dependency graph must remain acyclic.

---

# 21. Transformation Planning

Every optimization is represented as an explicit transaction-ready plan.

```rust
struct TransformationPlan {
    source_revision: RevisionId,

    files_create: Vec<PathBuf>,
    files_modify: Vec<PathBuf>,
    files_delete: Vec<PathBuf>,

    symbol_moves: Vec<SymbolMove>,
    include_edits: Vec<IncludeEdit>,

    validation_requirements: ValidationRequirements,
}
```

Lifecycle:

```text
Plan
  ↓
Preview
  ↓
Apply
  ↓
Validate
  ↓
Measure
  ↓
Commit
```

Failure must support rollback without leaving a partially applied optimization.

---

# 22. Header Extraction

Candidate clusters become generated sub-headers.

Possible names include:

```text
<name>_fwd.h
<name>_types.h
<name>_core.h
<name>_detail.h
```

Naming is configurable.

Generated headers include predecessor clusters according to the cluster DAG's topological ordering.

Generated headers must not cross frozen or contract regions in ways that violate their policies.

---

# 23. Header Self-Containment

Every generated header must compile independently under every relevant compilation context.

For example:

```cpp
#include "thing_types.h"
```

must compile in a minimal validation TU.

This catches accidental reliance on:

* include order
* transitive includes
* unrelated macros
* missing forward declarations
* hidden definitions

Self-containment is a mandatory validation tier for generated headers.

---

# 24. Safe Consumer Include Rewriting

Consumer rewriting must not rely solely on direct symbol references.

The algorithm is:

```text
1. Identify symbols referenced by the consumer.
2. Determine generated headers containing those symbols.
3. Determine the minimum required generated-header set.
4. Add required transitive predecessor headers.
5. Analyze macro/pragma/registration/initialization effects.
6. Determine whether the original include has non-symbol semantic effects.
7. Propose the replacement.
8. Validate the affected TU under all relevant contexts.
9. Commit only after validation succeeds.
```

Critical invariant:

> An include must never be removed merely because no direct symbol reference was found.

An include may provide:

* macros
* specializations
* registration
* initialization
* pragmas
* overloads
* template definitions
* intentional inclusion-order behavior

Consumer rewriting must therefore be semantics-aware.

---

# 25. Backward-Compatible Shim

The original header may remain as a compatibility shim:

```cpp
#pragma once

#include "thing_fwd.h"
#include "thing_types.h"
#include "thing_core.h"
```

This allows incremental consumer migration.

The shim should preserve any required side effects and ordering semantics. It must not be assumed to be safely reducible to a simple list of generated headers without validation.

---

# 26. Tiered Validation

Validation is staged to avoid requiring a full build for every candidate.

## Tier 1 — Structural Validation

Verify:

* generated files exist
* references resolve
* cluster DAG is valid
* symbols are neither duplicated nor missing
* policies are respected

## Tier 2 — Generated Header Self-Containment

Compile generated headers in isolation under relevant contexts.

## Tier 3 — Affected TU Validation

Recompile TUs whose include or semantic dependencies changed.

## Tier 4 — Full Build

Run the complete relevant build configurations.

## Tier 5 — Behavioral Tests

Run relevant automated tests.

---

# 27. Transformation Validation Across Contexts

Every affected compilation context must be considered.

For example:

```text
Debug
Release
```

may differ because of `DEBUG` / `NDEBUG`.

Generated headers and rewritten consumers must compile correctly under every relevant context.

A transformation that succeeds in one context but changes semantics or fails in another is not valid for a multi-context optimization target.

---

# 28. Performance Measurement

The optimizer should capture before/after measurements associated with:

```text
repository revision
compiler/toolset
compilation context
optimization plan
```

At minimum:

```text
preprocessed tokens
front-end time
peak memory
wall-clock build time
```

Where available:

```text
AST node count
template instantiation count
unique declarations processed
```

Reports distinguish:

```text
Predicted structural reduction
Actual structural reduction
Estimated compilation-load reduction
Measured compilation-cost reduction
```

---

# 29. Generated Code Provenance

Generated files are analyzed but normally not modified.

The system should retain provenance where available:

```text
Generated file
    ↓
Generator
    ↓
Generator input/source
```

Future versions may identify generator source automatically, but initial versions need only preserve enough provenance to explain why a generated file is frozen.

---

# 30. Phase 5: Transformation, Validation & Measurement

Phase 5 executes the plan produced by Phase 4.

```text
Transformation Plan
        ↓
Preview
        ↓
Apply
        ↓
Structural Validation
        ↓
Self-Containment Validation
        ↓
Affected TU Validation
        ↓
Full Build / Tests
        ↓
Performance Measurement
        ↓
Commit or Rollback
```

This phase is the only phase that mutates repository source files.

---

# 31. Reporting & Presentation

Reporting is a first-class consumer of the canonical reduced model, transformation plan, validation results, and measurements.

```text
Reduced Semantic Model
  ├── Analysis Results
  ├── Optimization Plan
  ├── Validation Results
  ├── Performance Measurements
  └── Presentation Output
        ├── Markdown
        └── Mermaid
```

Example:

```text
report/
├── overview.md
├── load-analysis.md
├── dependency-graph.mmd
├── clusters.mmd
├── validation.md
├── headers/
│   ├── thing.md
│   └── ...
└── migration.md
```

Mermaid views may include:

```text
Header graph
Symbol graph
Cluster DAG
Before/after dependency graph
Collateral-load graph
Optimization frontiers
Migration graph
```

---

# 32. Load Analysis Reporting

Example:

```text
Thing

Semantic dependency load:
    17 symbols

Collateral dependency load:
    4,382 symbols

Total structural load:
    4,399 symbols

Largest collateral source:
    HugeThirdParty.h
    4,121 symbols

Optimization frontier:
    Thing.h

Boundary:
    ThirdParty + Frozen

Potential structural reduction:
    4,121 symbols
```

After optimization:

```text
Thing

Semantic dependency load:
    24 symbols

Collateral dependency load:
    812 symbols

Residual collateral load:
    812 symbols

Structural reduction:
    3,570 symbols
```

The report must clearly identify residual load as remaining collateral load.

---

# 33. Migration Reporting

For every transformed consumer, report:

```text
Original include:
    thing.h

Replacement:
    thing_types.h

Referenced symbols:
    Thing
    ThingId

Reason:
    Consumer requires only symbols contained in thing_types.h

Validation:
    Passed

Contexts:
    Debug
    Release
```

If an include is retained because of side effects:

```text
Original include:
    registration.h

Rewrite:
    Not performed

Reason:
    Header contains registration side effects

Policy:
    Frozen or OrderSensitive
```

---

# 34. Rust Implementation

The system should be implemented primarily in Rust.

Suggested workspace:

```text
project/
├── crates/
│   ├── cli/
│   ├── repo/
│   ├── build/
│   ├── scheduler/
│   ├── index/
│   ├── graph/
│   ├── cluster/
│   ├── transform/
│   ├── artifacts/
│   ├── validate/
│   └── report/
│
├── tools/
│   └── clang-worker/
│
└── tests/
```

Rust owns:

* CLI
* repository discovery
* MSBuild/build orchestration
* compiler environment discovery
* compilation contexts
* scheduling
* subprocess management
* sharding
* binary serialization
* shuffle/reduce
* Roaring Bitmaps
* graph algorithms
* SCC analysis
* clustering
* optimization planning
* transformation
* validation
* caching
* reporting
* diagnostics

Clang/Clice owns:

* parsing
* preprocessing
* AST construction
* semantic identity
* references
* declarations
* definitions
* type relationships
* template semantics

---

# 35. Process Boundary

The first implementation should prefer a process boundary between Rust and the C++ semantic engine:

```text
Rust optimizer
      │
      │ versioned binary protocol
      ▼
Clang/Clice worker
```

Advantages include:

* memory isolation
* crash isolation
* independent versioning
* easy worker parallelism
* future distributed execution

---

# 36. Suggested Rust Technology Stack

```text
Rust
clap
Tokio
Rayon
ignore / ripgrep engine
roaring
xxhash-rust
zstd
memmap2
custom versioned binary artifacts
rkyv or equivalent where useful
tracing
thiserror
anyhow
```

C++/Clang worker:

```text
Clang
Clice
CMake
```

Avoid introducing a database or RPC system until workload evidence demonstrates a need.

Shard artifacts should initially serve as the inter-process and future distributed protocol.

---

# 37. Incremental Artifact Model

The pipeline should produce persistent intermediate artifacts:

```text
artifacts/
├── repository/
├── contexts/
├── includes/
├── ast-shards/
├── shuffle/
├── reduced/
├── graphs/
├── plans/
├── validation/
└── reports/
```

Every artifact should include:

```text
schema version
analysis ID
repository revision
configuration hash
compiler/toolchain identity
```

This makes analysis resumable and reproducible.

---

# 38. Key Invariants

### Invariant 1 — Semantic correctness

Every generated dependency must be semantically justified.

### Invariant 2 — Include graph separation

Semantic and textual include dependencies are never conflated.

### Invariant 3 — Context correctness

A transformation must remain valid under every relevant compilation context.

### Invariant 4 — Ownership and policy

Ownership describes who controls a file; optimization policy describes what the optimizer may change.

Frozen and contract content is not modified unless explicitly permitted.

### Invariant 5 — DAG correctness

Generated cluster dependencies must remain acyclic.

### Invariant 6 — Header self-containment

Every generated header must compile independently under its supported contexts.

### Invariant 7 — Transactional transformation

A failed validation must never leave a partially applied optimization.

### Invariant 8 — Cache correctness

Cached analysis must never be reused when any semantic input affecting the result has changed.

### Invariant 9 — Include safety

An include cannot be removed solely because direct symbol usage was not observed.

### Invariant 10 — Measurement provenance

Performance measurements must identify the exact compiler, context, repository revision, and optimization plan that produced them.

---

# 39. End-to-End Pipeline

```text
Repository
    │
    ▼
Phase 0: Repository & Build Discovery
    │
    ├── source files
    ├── MSBuild configuration
    ├── compiler/toolchain
    ├── MSVC STL
    ├── Windows SDK
    ├── ownership
    └── optimization policy
    │
    ▼
Phase 1: Compilation Context & Include Analysis
    │
    ├── explicit preprocessor definitions
    ├── Debug / Release contexts
    ├── include graph
    └── TU schedule
    │
    ▼
Phase 2: Parallel Semantic Indexing
    │
    ├── symbols
    ├── declarations
    ├── definitions
    ├── semantic edges
    ├── macro dependencies
    ├── template relationships
    └── TU usage
    │
    ▼
Phase 3: Shuffle & Reduction
    │
    ├── canonical symbols
    ├── reduced semantic graph
    ├── include graph
    ├── file-symbol index
    └── context
    │
    ▼
Phase 4: Dependency Analysis & Optimization Planning
    │
    ├── semantic load
    ├── include load
    ├── collateral load
    ├── residual-load baseline
    ├── optimization frontiers
    ├── SCC condensation
    ├── clustering
    └── transformation plans
    │
    ▼
Phase 5: Transformation, Validation & Measurement
    │
    ├── preview
    ├── apply
    ├── self-contained headers
    ├── affected TUs
    ├── full build
    ├── tests
    ├── performance measurement
    └── commit / rollback
    │
    ▼
Reports
    ├── Markdown
    ├── Mermaid
    ├── migration plan
    └── measured load reduction
```

---

# 40. Initial Implementation Priorities

## P0 — Required for serious implementation

1. Repository and compiler discovery
2. Explicit compilation-context model
3. MSVC STL / Windows SDK discovery
4. Include graph
6. Canonical symbol/declaration model
7. Semantic dependency extraction
8. Formal structural load metrics
9. Semantic vs collateral load model
10. Template dependency model
11. Movability constraints
12. Safe include-rewrite algorithm
13. Incremental analysis cache
14. Transactional transformation model
15. Tiered validation
16. Header self-containment validation

## P1 — Required for robust production use

17. Macro dependency graph
18. Header side-effect analysis
19. Confidence model
20. Actual compilation-performance measurement
21. Load-based clustering objective
22. Optimization-frontier diagnostics
23. Generated-code provenance
24. compilation-cost estimation
25. Rich Markdown/Mermaid reporting

## P2 — Can follow initial production implementation

26. Distributed execution
27. Sophisticated visualization
28. Automatic optimization-policy inference
29. Advanced persistent indexes
30. Automatic generator-source discovery

---

# 41. Product-Level Model

The optimizer should ultimately answer five questions:

```text
1. What is expensive?
2. Why is it expensive?
3. Which portion is actually necessary?
4. Where are we allowed to intervene?
5. What did the intervention actually save?
```

The core product loop is therefore:

```text
Find expensive collateral load
          ↓
Explain its provenance
          ↓
Identify optimization frontier
          ↓
Construct smallest safe intervention
          ↓
Validate semantic correctness
          ↓
Measure actual improvement
          ↓
Report residual collateral load
```

Clustering is one mechanism inside this loop.

The broader system is a **dependency-load optimizer for large C++ codebases**.
