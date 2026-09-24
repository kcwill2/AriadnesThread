# Architecture & Design Specification: High-Throughput C++ Header Optimization & Dependency Decoupling Tool

---

## 1. Overview & Problem Statement

Large-scale C++ codebases suffering from long build times often contain bloated "god headers." When headers define numerous loosely coupled types alongside heavy internal or third-party dependencies, downstream translation units (TUs) endure severe transitive compilation overhead, even when consuming only a tiny fraction of the definitions.

The fundamental problem is not merely header size. It is **unnecessary compilation load caused by dependency structure**.

A symbol may require only a small semantic dependency closure while its current header organization causes thousands of additional symbols and dependencies to be processed.

This tool analyzes the actual C++ dependency graph, identifies collateral compilation load, finds safe optimization frontiers, proposes structural changes, and validates and measures the resulting improvements.

The product loop is:

```text
Analyze
   ↓
Report
   ↓
Review opportunity
   ↓
 ┌───────────────────────┐
 │                       │
Auto-update          Manual update
 │                       │
 └──────────┬────────────┘
            ↓
         Validate
            ↓
         Measure
            ↓
        Re-analyze
            ↓
          Repeat
```

The implementation uses Clang-based semantic analysis and an out-of-core, sharded execution architecture designed to scale from developer workstations to large repositories.

---

# 2. Product Model

The product is an **optimization advisor with an optional automated refactoring capability**.

The system must distinguish between:

* analysis
* recommendations
* automatically applicable transformations
* manually implemented transformations
* validation
* measurement

The default analysis workflow is non-destructive.

The primary user interaction is:

```text
tool analyze
tool report
tool explain <opportunity>
```

An automatically applicable opportunity can then be applied through:

```text
tool update <opportunity>
```

A manually implemented opportunity is supported through:

```text
tool implement <opportunity>
```

followed by:

```text
tool validate <opportunity>
tool measure
tool analyze
```

The tool must never require the user to accept a transformation merely because the optimizer found it.

---

# 3. User Workflow

## 3.1 Analyze

```bash
tool analyze
```

Analysis is read-only.

It discovers the repository, compilation contexts, include structure, semantic dependencies, dependency load, and optimization opportunities.

The result is an immutable analysis snapshot.

Example:

```text
Analysis complete.

Translation units:     3,412
Symbols:              182,441
Semantic edges:     4,103,221

Optimization opportunities:
    OPT-001   graphics.h
    OPT-002   networking.h
    OPT-003   serialization.h
```

---

## 3.2 Report

```bash
tool report
```

The report presents optimization opportunities in terms of expected compilation-load reduction and explainability.

Example:

```text
OPT-001  graphics.h

Collateral load:       12,482 symbols
Affected TUs:              384

Primary collateral source:
    third_party/GraphicsSDK.h

Proposed intervention:
    Extract GraphicsOptions and GraphicsConfig

Automation:
    Available

Confidence:
    High
```

Reports must be available in both human-readable Markdown and machine-readable structured form.

---

## 3.3 Explain an Opportunity

Every optimization opportunity has a stable ID.

```bash
tool explain OPT-001
```

The explanation must answer:

* What is expensive?
* What is semantically required?
* What is collateral?
* Where does the collateral load originate?
* What is the optimization frontier?
* What symbols/files would change?
* Which consumers are affected?
* Why is the transformation considered safe?
* What validation will be performed?
* What impact is expected?

The system must make its recommendations explainable without requiring the user to inspect internal graph artifacts.

---

## 3.4 Automatic Update

For an opportunity that can be safely automated:

```bash
tool update OPT-001 --preview
```

produces the proposed changes without modifying the repository.

Example:

```text
OPT-001 proposed changes:

    + include/graphics_options.h
    + include/graphics_fwd.h
    ~ include/graphics.h
    ~ src/foo.cpp
    ~ src/bar.cpp

Estimated collateral load:
    12,482 → 1,561

Affected translation units:
    384
```

The user can then approve the update:

```bash
tool update OPT-001
```

Approval must be explicit before repository mutation.

The update is transactional and must be validated before being committed.

---

## 3.5 Manual Update

Some opportunities cannot safely be automated.

For these:

```bash
tool implement OPT-002
```

produces a manual implementation guide.

The guide identifies:

* intended architectural change
* relevant files
* symbols involved
* dependency constraints
* expected include changes
* known risks
* validation requirements

The tool does not pretend that a manual recommendation has been implemented.

After the developer makes the change:

```bash
tool validate OPT-002
```

determines whether the resulting repository satisfies the intended transformation.

---

## 3.6 Validate

Validation occurs after either an automatic or manual update.

```bash
tool validate OPT-001
```

Validation proceeds through applicable tiers:

```text
Structural validation
        ↓
Generated-header validation
        ↓
Affected-TU validation
        ↓
Full build
        ↓
Behavioral tests
```

A failed automatic transformation must be rolled back.

A failed manual transformation must produce actionable diagnostics without modifying the user's work.

---

## 3.7 Measure

After validation:

```bash
tool measure
```

compares structural and compilation metrics.

Example:

```text
OPT-001

Collateral load:
    12,482 → 1,561

Affected TUs:
    384

Front-end CPU:
    91.2s → 54.7s

Build wall time:
    42.1s → 31.4s
```

Measured results become part of the opportunity history.

---

## 3.8 Re-analyze

After changes:

```bash
tool analyze
```

produces a new analysis snapshot.

The system should preserve the relationship between analyses so that completed and remaining opportunities can be tracked.

Example:

```text
Previous analysis:
    2026-09-23-a81f

Completed:
    OPT-001

Residual collateral load:
    1,561 symbols

New opportunities:
    OPT-004
    OPT-005
```

The optimization loop continues until the user chooses to stop.

---

# 4. Optimization Opportunity Model

An optimization opportunity is the primary unit of work presented to the user.

Conceptually:

```rust
struct OptimizationOpportunity {
    id: OpportunityId,
    target: Target,
    problem: LoadExplanation,
    proposed_change: TransformationPlan,
    automation: AutomationLevel,
    confidence: Confidence,
    affected_files: Vec<FileId>,
    affected_tus: RoaringBitmap,
    estimated_impact: LoadImpact,
    validation_plan: ValidationPlan,
}
```

An opportunity moves through a lifecycle:

```text
Discovered
    ↓
Reviewed
    ↓
Approved
    ↓
Implemented
    ↓
Validated
    ↓
Measured
    ↓
Completed
```

Alternative states include:

```text
Rejected
Deferred
Blocked
Failed
```

The opportunity model provides a stable bridge between analysis and repository transformation.

---

# 5. System Architecture

The system is divided into a product workflow and an analysis/transformation pipeline.

```text
                         USER WORKFLOW

    ┌──────────┐
    │ Analyze  │
    └────┬─────┘
         ↓
    ┌──────────┐
    │  Report  │
    └────┬─────┘
         ↓
    ┌──────────┐
    │ Explain  │
    └────┬─────┘
         ↓
   ┌─────┴─────┐
   ↓           ↓
Auto Update   Manual Update
   │           │
   └─────┬─────┘
         ↓
    ┌──────────┐
    │ Validate │
    └────┬─────┘
         ↓
    ┌──────────┐
    │ Measure  │
    └────┬─────┘
         ↓
    ┌──────────┐
    │ Analyze  │
    └──────────┘


                       ANALYSIS PIPELINE

Phase 0  Repository & Build Discovery
              ↓
Phase 1  Compilation Context & Include Analysis
              ↓
Phase 2  Parallel Semantic Indexing
              ↓
Phase 3  Shuffle & Reduction
              ↓
Phase 4  Dependency Analysis & Optimization Planning
              ↓
Phase 5  Transformation, Validation & Measurement
```

The product workflow determines **when** capabilities are invoked.

The analysis pipeline determines **how** those capabilities are produced.

---

# 6. Phase 0: Repository & Build Discovery

Phase 0 establishes the repository model.

Responsibilities:

* discover source files
* discover build configuration
* discover compiler/toolchain
* establish ownership
* establish optimization policy
* establish repository identity
* identify available compilation commands

The source tree is authoritative for source discovery.

MSBuild files are treated primarily as build configuration inputs rather than as the authoritative inventory of source files.

Where practical, actual compiler invocations should be captured and normalized into `compile_commands.json`.

Catter may be used to capture MSBuild compiler commands.

---

# 7. Compiler Environment & MSVC Discovery

The target environment is C++ compiled with Visual Studio/MSVC.

The tool must discover the actual compiler environment rather than hard-coding installation paths.

```rust
struct CompilerEnvironment {
    compiler_path: PathBuf,
    toolset_version: String,
    standard_include_roots: Vec<PathBuf>,
    sdk_include_roots: Vec<PathBuf>,
    external_include_roots: Vec<PathBuf>,
}
```

Discovery should identify:

* `cl.exe`
* MSVC toolset version
* MSVC standard library include roots
* Windows SDK roots
* external include roots

Ownership should be determined using resolved canonical paths rather than filename patterns.

---

# 8. Ownership & Optimization Policy

Ownership and optimization policy are separate concepts.

```rust
enum FileOwnership {
    Repository,
    ThirdParty,
    Toolchain,
    Generated,
}

enum OptimizationPolicy {
    Optimizable,
    Contract,
    Frozen,
}
```

Ownership answers:

> Who controls or produces this file?

Policy answers:

> Are we permitted to change it?

Default policy:

```text
Repository → Optimizable
ThirdParty  → Frozen
Toolchain   → Frozen
Generated   → Frozen
```

Repository configuration may override these defaults.

Example:

```yaml
optimization:
  default: optimizable

  paths:
    "include/public/**": contract
    "third_party/**": frozen

  directory_names:
    generated: frozen
```

Policy precedence:

1. explicit file override
2. most-specific path/glob
3. directory-name rule
4. inherited parent-directory policy
5. repository default

Generated code should normally be:

```text
Analyze: YES
Optimize: NO
Rewrite: NO
Report: YES
```

---

# 9. Phase 1: Compilation Context & Include Analysis

Phase 1 establishes the actual compilation contexts and physical include graph.

A compilation context includes:

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

The tool analyzes the actual compilation contexts used by the repository.

It does not enumerate arbitrary hypothetical preprocessor states.

For the target codebase, the known configuration distinction includes definitions such as `DEBUG` and `NDEBUG`.

### PCH Scope

Precompiled headers are explicitly out of scope for the initial implementation.

The target codebase does not use precompiled headers, so the optimizer models translation-unit-local preprocessing and compilation directly.

If PCH support is required for another codebase in the future, it should be introduced as an additional compilation-cost model rather than changing the core semantic dependency model.

---

# 10. Data Model & Wire Formats

The system uses immutable versioned artifacts between major pipeline stages.

The semantic worker and Rust orchestration layer communicate through a versioned process boundary.

A packed edge record may use:

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

Auxiliary artifacts include:

* string interning tables
* TU usage bitmaps
* symbol records
* declaration/definition records
* file/symbol indexes

Roaring Bitmaps represent TU usage:

```text
TUs(s) = { tu_id_1, tu_id_2, ... }
```

---

# 11. Phase 2: Parallel Semantic Indexing

Phase 2 invokes a Clang-based semantic backend.

The architecture should abstract the backend:

```rust
trait CppSemanticBackend {
    fn analyze_tu(
        &self,
        command: &CompileCommand,
    ) -> Result<TuAnalysis>;
}
```

Clice is the first implementation candidate.

The architecture should not hard-code the system to Clice-specific internals.

The semantic worker identifies:

* canonical symbols
* declarations
* definitions
* symbol ownership
* references
* semantic dependencies
* Complete versus Incomplete dependencies
* TU usage

---

# 12. Semantic Graph & Dependency Classification

The semantic graph is:

```text
Symbol → Symbol
```

The include graph is:

```text
File → File
```

The two graphs must remain distinct.

### Complete Dependencies

Examples include:

* base classes
* by-value members
* `sizeof`
* `alignof`
* `typeid`
* direct member access requiring the target definition
* explicit down/cross-casts
* inline method bodies
* template definitions and instantiations where the definition is required

### Incomplete Dependencies

Examples include:

* pointers
* references
* function parameter types in declarations
* return types in declarations
* forward-declarable relationships
* type aliases where only the declaration is required

The exact classification is semantic and must be determined from the AST rather than textual heuristics alone.

---

# 13. Template & ODR Model

The optimizer must distinguish:

* template declarations
* template definitions
* explicit specializations
* partial specializations
* implicit instantiations
* explicit instantiations

It must also account for:

* inline definitions
* internal linkage
* anonymous namespaces
* explicit specializations
* ODR constraints
* constexpr variables
* registration
* initialization
* source-order dependencies

These properties affect whether a declaration can be moved between generated headers.

---

# 14. Macro & Header Side-Effect Model

Macros and header side effects can make apparently unused includes semantically relevant.

The model includes:

```text
Macro → Declaration
Macro → Use
Macro → ConditionalRegion
```

Headers may be classified as:

```text
PureDeclarative
MacroProducing
PragmaProducing
RegistrationBearing
InitializationBearing
OrderSensitive
Unknown
```

The optimizer must not remove an include merely because no direct symbol reference was found.

---

# 15. Phase 3: Shuffle & Parallel Reduction

Semantic analysis produces independent TU-level results.

Phase 3 produces the project-wide reduced semantic model.

Workers route records by hash:

```text
src_usr_hash % K
```

Each reducer owns a bucket and can therefore:

* deduplicate edges
* aggregate symbol definitions
* aggregate TU usage
* construct canonical symbol records
* build project-wide indexes

No synchronization is required between independent reducers for a given bucket.

---

# 16. Canonical Symbol & File Model

A canonical symbol has:

```rust
struct Symbol {
    id: SymbolId,
    kind: SymbolKind,
    declarations: Vec<DeclId>,
    definition: Option<DeclId>,
    owning_file: FileId,
}
```

The reduced model also maintains:

```text
File → Symbols
Symbol → Owning File
Symbol → Semantic Dependencies
Symbol → Consuming TUs
```

This model is the basis for load analysis and transformation planning.

---

# 17. Dependency Load Model

Every symbol has a dependency load.

The system distinguishes:

### Semantic Dependency Load

Dependencies genuinely required by C++ semantics.

### Include Load

The set of symbols/files processed because of the current include structure.

### Collateral Load

Include load not attributable to the symbol's semantic dependency closure.

### Residual Load

Collateral load that remains after an optimization/refactoring step.

Residual load is not a separate dependency type.

The optimization loop is:

```text
Measure
  ↓
Identify collateral load
  ↓
Find optimization frontier
  ↓
Refactor
  ↓
Measure residual load
  ↓
Repeat
```

---

# 18. Formal Load Metrics

The system should maintain multiple levels of load measurement.

## Structural Load

Graph-derived quantities such as:

* semantic dependency count
* include closure size
* collateral symbol count
* affected TU count

## Estimated Compilation Load

Potential estimates include:

* preprocessed token count
* declaration count
* AST node count
* template instantiation count

## Measured Compilation Cost

Observed compiler behavior:

* preprocessing CPU
* front-end CPU
* wall-clock time
* peak memory
* affected TU compile time
* full build time

The product should not assume that structural load and measured compilation cost are identical.

They are separate metrics that can be compared.

---

# 19. Optimization Frontiers

An optimization frontier is the point in the dependency graph where the optimizer can intervene before reaching a dependency that is frozen, contractual, third-party, toolchain-owned, generated, or otherwise outside the permitted optimization scope.

Example:

```text
App.h
  ↓
GraphicsAdapter.h
  ↓
HugeSDK.h

HugeSDK.h:
    Ownership: ThirdParty
    Policy: Frozen

Optimization frontier:
    GraphicsAdapter.h
```

The frontier is a graph concept.

`OptimizationPolicy` determines whether a file may be changed.

---

# 20. Confidence Model

Analysis results carry confidence:

```rust
enum Confidence {
    Exact,
    Conservative,
    Incomplete,
    Unknown,
}
```

Confidence should influence automation.

A low-confidence recommendation may still be reported as an opportunity but should generally require manual review rather than automatic mutation.

---

# 21. Incremental Analysis & Caching

Analysis artifacts should be reusable when their inputs have not changed.

Cache keys must account for relevant:

* file content
* compilation context
* compiler/toolchain version
* analysis version
* configuration

A source change should invalidate only the artifacts whose results depend upon it.

The system should report cache reuse:

```text
Translation units:
    3,412

Reused:
    3,217

Reanalyzed:
      195

Cache hit rate:
    94.3%
```

---

# 22. Phase 4: Dependency Analysis & Optimization Planning

Phase 4 converts the reduced semantic model into optimization opportunities.

The planner performs:

* load analysis
* collateral-load attribution
* SCC analysis
* optimization-frontier detection
* cluster analysis
* movability analysis
* side-effect analysis
* context analysis
* transformation planning

The output is a set of stable `OptimizationOpportunity` records.

---

# 23. Constrained DAG Clustering

Splitting a header is modeled as a graph partitioning problem under a DAG constraint.

### SCC Condensation

Run Tarjan's SCC algorithm on the symbol graph.

Mutually dependent symbols are collapsed into indivisible super-nodes.

### Constrained Agglomerative Merging

Initialize each super-node as an independent cluster.

Candidate clusters are merged according to affinity and optimization objectives.

A merge is rejected if it violates:

* semantic correctness
* dependency direction
* optimization policy
* movability
* context correctness
* side-effect constraints
* maximum cluster size
* DAG constraints

The clustering objective is subordinate to the actual product objective:

> reduce unnecessary collateral compilation load while preserving correctness.

---

# 24. Transformation Planning

A transformation plan describes what would change without modifying the repository.

```rust
struct TransformationPlan {
    opportunity_id: OpportunityId,
    generated_files: Vec<PathBuf>,
    modified_files: Vec<PathBuf>,
    removed_dependencies: Vec<Dependency>,
    added_dependencies: Vec<Dependency>,
    consumer_edits: Vec<Edit>,
    validation_plan: ValidationPlan,
    estimated_impact: LoadImpact,
}
```

The plan must be deterministic and explainable.

A plan is not an applied transformation.

---

# 25. Header Extraction

The tool may extract symbols into smaller headers such as:

```text
<name>_fwd.h
<name>_types.h
<name>_core.h
```

Generated headers must:

* preserve required semantic dependencies
* include predecessor headers as necessary
* remain self-contained where required
* preserve relevant declarations and definitions
* obey the cluster DAG
* respect macros and side effects

---

# 26. Header Self-Containment

Every generated header intended for independent inclusion must be validated independently.

For example:

```cpp
#include "thing_types.h"

int main() {}
```

must compile when `thing_types.h` is expected to be self-contained.

Self-containment failures are transformation failures, not merely documentation issues.

---

# 27. Safe Consumer Include Rewriting

Consumer rewriting follows:

1. identify symbols referenced by the consumer
2. map symbols to generated headers
3. determine minimum required header set
4. include transitive predecessor requirements
5. inspect macro/pragma/registration effects
6. determine whether the original include has non-symbol effects
7. propose replacement
8. validate the affected TU
9. commit only after validation

Critical invariant:

> Never remove an include merely because no direct symbol reference was found.

An include may be retained because it has semantic or preprocessing side effects.

---

# 28. Backward-Compatible Shim

Where appropriate, the original header remains as a compatibility shim.

Example:

```cpp
#pragma once

#include "thing_types.h"
#include "thing_core.h"
```

The shim allows existing consumers to continue compiling while direct consumers migrate to smaller headers.

The optimizer should not assume that every original include must immediately be removed.

---

# 29. Manual vs Automatic Transformation

Every opportunity receives an automation classification.

Conceptually:

```text
Automatic
    Safe for tool-generated transformation

Assisted
    Tool can generate most of the change but requires review

Manual
    Tool provides analysis and implementation guidance only

Blocked
    Transformation is not currently safe or permitted
```

Automatic updates require explicit user approval.

Manual opportunities remain part of the optimization report and can be validated after the developer implements them.

---

# 30. Transactional Transformation

Automatic transformation lifecycle:

```text
Plan
  ↓
Preview
  ↓
Approve
  ↓
Apply
  ↓
Validate
  ↓
Measure
  ↓
Commit
```

If validation fails:

```text
Apply
  ↓
Validate
  ↓
FAIL
  ↓
Rollback
```

The original repository state must be recoverable.

The transaction boundary must cover:

* generated files
* modified headers
* consumer rewrites
* metadata
* associated artifacts

---

# 31. Tiered Validation

Validation occurs at increasing cost.

### Tier 1 — Structural

Verify:

* graph consistency
* symbol ownership
* no lost symbols
* no invalid dependency edges
* DAG constraints

### Tier 2 — Generated Header

Verify:

* self-containment
* compilation
* include correctness

### Tier 3 — Affected Translation Units

Compile every affected TU.

### Tier 4 — Full Build

Compile the repository.

### Tier 5 — Behavioral Tests

Run the repository's available test suite.

A transformation must not be considered successful merely because Tier 1 succeeds.

---

# 32. Transformation Validation Across Contexts

A transformation must be valid across the relevant compilation contexts in which the affected files are compiled.

For example:

```text
Debug
Release
```

must be considered independently when their semantic models differ.

The optimizer must not generate a transformation that is valid only under one relevant preprocessor configuration when the same source participates in another.

---

# 33. Performance Measurement

Performance measurement compares before and after states.

The system should report:

* structural load
* estimated compilation load
* measured compilation cost
* affected TU population
* full-build impact where available

Example:

```text
Collateral load:
    12,482 → 1,561

Affected TUs:
    384

Front-end CPU:
    91.2s → 54.7s

Build wall time:
    42.1s → 31.4s
```

Measurements must identify the population and context being compared.

---

# 34. Generated Code Provenance

Generated files must retain provenance sufficient to determine:

* which opportunity produced them
* which analysis produced the opportunity
* which source symbols were extracted
* which transformation plan generated the files
* which tool version produced them

Generated code must not silently become an authoritative source of truth.

Where possible, provenance should identify the original source declarations.

---

# 35. Reporting & Presentation

Reporting is a first-class product capability.

Reports should include:

```text
Analysis
  ├── Repository summary
  ├── Compilation contexts
  ├── Dependency analysis
  ├── Load analysis
  ├── Optimization opportunities
  ├── Transformation plans
  ├── Validation results
  ├── Measurements
  └── Residual load
```

Reports should be available as Markdown and structured machine-readable data.

Mermaid output should provide visualizations such as:

* header graph
* symbol graph
* cluster DAG
* before/after dependency graph
* collateral-load provenance

Example report structure:

```text
report/
├── overview.md
├── dependency-graph.mmd
├── clusters.mmd
├── opportunities/
│   ├── OPT-001.md
│   └── OPT-002.md
├── headers/
│   ├── thing.md
│   └── graphics.md
└── migration.md
```

Every report should identify the analysis ID, tool version, and relevant configuration.

---

# 36. Opportunity Reporting

The report should prioritize opportunities based on measurable potential rather than merely header size.

Each opportunity should expose:

```text
Opportunity ID
Target
Collateral load
Affected TUs
Estimated impact
Optimization frontier
Confidence
Automation level
Proposed transformation
Validation requirements
```

Example:

```text
OPT-001

Target:
    graphics.h

Collateral load:
    12,482 symbols

Affected TUs:
    384

Frontier:
    GraphicsAdapter.h

Automation:
    Automatic

Confidence:
    High

Estimated reduction:
    10,921 symbols
```

The report must not hide opportunities simply because they cannot currently be automated.

---

# 37. Migration Reporting

Migration reports describe both automated and manual work.

For automatic work:

```text
OPT-001
    17 files modified
    384 TUs affected
    Validation: PASS
```

For manual work:

```text
OPT-002

Required implementation:
    Extract NetworkOptions from NetworkClient.

Affected files:
    12

Validation command:
    tool validate OPT-002
```

The report should make manual work actionable without falsely claiming completion.

---

# 38. Rust Implementation

The majority of the system should be implemented in Rust.

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
│   ├── validate/
│   └── report/
├── tools/
│   └── clang-worker/
└── tests/
```

Rust owns:

* CLI
* repository discovery
* MSBuild/build orchestration
* compilation contexts
* scheduling
* subprocess management
* sharding
* binary serialization
* shuffle/reduce
* graph algorithms
* clustering
* transformation planning
* transformation application
* validation
* caching
* reporting
* diagnostics

Clang/Clice owns C++ semantic analysis.

---

# 39. Process Boundary

The initial semantic architecture should use a process boundary rather than Rust FFI.

```text
Rust
  │
  │ versioned binary protocol
  ↓
Clang/Clice worker
  │
  ↓
TU semantic analysis
  │
  ↓
immutable artifact
```

This isolates Clang memory behavior and allows worker processes to terminate and release memory independently.

The protocol must be versioned.

---

# 40. Rust Technology Stack

Suggested stack:

* stable Rust
* `clap`
* Tokio
* Rayon where CPU-parallel Rust work benefits from it
* standard filesystem/path APIs
* `ignore` / ripgrep engine for repository scanning
* Catter for MSBuild compiler-command capture where needed
* Clice/LLVM-Clang semantic backend
* `memmap2`
* `rkyv` or custom versioned binary formats
* `roaring`
* `xxhash-rust`
* `zstd`
* custom graph structures
* `tracing`
* `thiserror`
* `anyhow`
* Rust unit/integration tests
* golden tests
* fuzz testing for parsers and artifact readers

A database and RPC framework are not required initially.

The artifact model should serve as the inter-process protocol and provide a future path to distributed execution.

---

# 41. Incremental Artifact Model

Artifacts are organized by analysis and pipeline stage.

```text
artifacts/
├── repository/
├── contexts/
├── includes/
├── ast-shards/
├── shuffle/
├── reduced/
├── graphs/
├── opportunities/
├── plans/
├── transformations/
├── validation/
└── reports/
```

Artifacts should be:

* versioned
* content-addressable where practical
* deterministic
* independently inspectable
* safe to discard and regenerate

An analysis snapshot should reference the exact artifact versions used to produce its opportunities.

---

# 42. High-Throughput Execution Strategy

## Process-Level Isolation

Independent worker processes prevent Clang memory accumulation from affecting the entire analysis.

Large translation units should be scheduled early to minimize stragglers.

Scheduling may consider:

* include depth
* token complexity
* historical execution time
* dependency characteristics

## Zero-Contention Shuffle

Workers route records to hash buckets.

Reducers process buckets independently.

This permits high parallelism without a central synchronized graph builder.

The high-throughput architecture is an implementation detail in service of the product workflow; it must not change the semantic results.

---

# 43. Key Invariants

The implementation must preserve the following invariants.

### Semantic correctness

A generated structure must preserve required C++ semantics.

### Dependency correctness

Semantic dependencies and physical include dependencies remain distinct.

### DAG correctness

Generated header dependencies must remain acyclic where the model requires a DAG.

### Policy correctness

Frozen, contract, third-party, toolchain, and generated files must not be modified without explicit policy allowing it.

### Context correctness

Transformations must remain valid across relevant compilation contexts.

### Side-effect correctness

Macros, pragmas, registration, initialization, and order-sensitive behavior must not be silently removed.

### Determinism

Equivalent input must produce equivalent analysis and transformation artifacts.

### Transaction safety

Failed transformations must not leave the repository in a partially applied state.

### Explainability

Every automatic transformation must have an inspectable explanation connecting the transformation to the load analysis that motivated it.

### Measurement integrity

Before/after measurements must identify the relevant repository state and compilation context.

---

# 44. End-to-End Pipeline

The complete system is:

```text
                    USER

                     │
                     ▼

              ┌─────────────┐
              │   analyze   │
              └──────┬──────┘
                     │
                     ▼
              ┌─────────────┐
              │   report    │
              └──────┬──────┘
                     │
                     ▼
              ┌─────────────┐
              │   explain   │
              └──────┬──────┘
                     │
               ┌─────┴─────┐
               ▼           ▼
        ┌────────────┐ ┌────────────┐
        │ auto update│ │manual impl │
        └─────┬──────┘ └─────┬──────┘
              │              │
              └──────┬───────┘
                     ▼
              ┌─────────────┐
              │   validate  │
              └──────┬──────┘
                     │
                     ▼
              ┌─────────────┐
              │   measure   │
              └──────┬──────┘
                     │
                     ▼
              ┌─────────────┐
              │   analyze   │
              └─────────────┘


              ANALYSIS ENGINE

Repository & Build Discovery
             ↓
Compilation Contexts
             ↓
Include Graph
             ↓
Semantic Indexing
             ↓
Shuffle & Reduction
             ↓
Reduced Semantic Model
             ↓
Load Analysis
             ↓
Optimization Opportunities
             ↓
Transformation Plans
             ↓
Validation
             ↓
Measurement
```

---

# 45. Initial Implementation Priorities

The implementation should proceed through vertical product milestones.

## P0 — Product Foundation

1. Repository discovery
2. Compilation-context discovery
3. Include graph
4. Semantic indexing
5. Reduced semantic model
6. Dependency load analysis

The first meaningful product checkpoint is:

```bash
tool analyze load <repo> --symbol <symbol>
```

which should explain:

* semantic load
* include load
* collateral load
* collateral provenance
* optimization frontier

## P1 — Optimization Workflow

7. Optimization opportunity model
8. Explainable optimization plans
9. Header extraction
10. Consumer rewriting
11. Transactional transformations
12. Tiered validation
13. Manual implementation workflow

## P2 — Measurement & Iteration

14. Build-performance measurement
15. Residual-load analysis
16. Before/after reporting
17. Re-analysis and opportunity history

## P3 — Scale

18. Incremental caching
19. Process-level parallelism
20. Sharded artifacts
21. Shuffle/reduce
22. Resumability
23. Large-repository performance

## P4 — Future

* distributed execution
* sophisticated visualization
* advanced policy inference
* persistent high-performance indexes
* automatic generator-source discovery
* additional semantic backends

---

# 46. Product-Level Model

The product is fundamentally an optimization loop:

```text
Find expensive collateral load
            ↓
Explain its provenance
            ↓
Identify the optimization frontier
            ↓
Construct the smallest safe intervention
            ↓
Present the opportunity
            ↓
Approve automatic update
        OR
Provide manual implementation guidance
            ↓
Validate
            ↓
Measure
            ↓
Report residual collateral load
            ↓
Find the next opportunity
```

The central product concepts are therefore:

```text
Repository
Compilation Context
Semantic Dependency
Include Dependency
Dependency Load
Collateral Load
Residual Load
Optimization Frontier
Optimization Opportunity
Transformation Plan
Validation
Measurement
```

The system exists to make this workflow accurate, safe, explainable, and scalable.

The ultimate objective is:

> **Analyze the real C++ dependency graph, explain where compilation load comes from, safely separate unnecessary collateral dependencies, and prove that the resulting code is both correct and cheaper to build.**
