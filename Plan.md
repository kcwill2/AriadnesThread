# Implementation Plan & Milestones

## 1. Goal

Build the C++ dependency optimization tool incrementally so that every milestone produces a usable, testable capability.

Each milestone must answer:

1. **What exists after this milestone?**
2. **What can a developer do with it?**
3. **How do we test it?**
4. **What does it prove?**
5. **What does it deliberately not do yet?**

The implementation should proceed from:

```text
Repository
    ↓
Discovery
    ↓
Compilation contexts
    ↓
Include graph
    ↓
Semantic graph
    ↓
Dependency load
    ↓
Optimization plan
    ↓
Transformation
    ↓
Validation
    ↓
Measurement
```

Each layer should become independently observable before the next layer depends on it.

---

# 2. Milestone Overview

| Milestone | Capability                      | User-visible result                          |
| --------- | ------------------------------- | -------------------------------------------- |
| M0        | Project skeleton & test harness | `cargo test` and CLI work                    |
| M1        | Repository discovery            | Understands the repository and its files     |
| M2        | Build/compilation discovery     | Produces deterministic compilation contexts  |
| M3        | Include graph                   | Can explain physical include load            |
| M4        | Semantic indexing               | Can explain actual symbol dependencies       |
| M5        | Reduced dependency model        | Can query a project-wide semantic model      |
| M6        | Load analysis                   | Can quantify semantic vs collateral load     |
| M7        | Optimization planning           | Produces explainable optimization plans      |
| M8        | Header extraction               | Generates candidate sub-headers              |
| M9        | Consumer rewriting              | Generates safe include migrations            |
| M10       | Validation & transactions       | Can apply, validate, and roll back changes   |
| M11       | Measurement                     | Demonstrates actual build impact             |
| M12       | Incremental analysis            | Avoids redoing unchanged work                |
| M13       | Production-scale execution      | Sharded, parallel, high-throughput operation |
| M14       | End-to-end optimization         | Complete automated workflow                  |
| M15       | Graph UI                        | Browse symbol, include and call graphs      |
| M16       | Clang worker and traversal      | Out-of-process analysis plus graph traversal |
| M17       | Language API seam               | Core crates speak a neutral vocabulary      |
| M18       | C++ as a plugin                 | C++ analysis is dispatched by the registry  |
| M19       | Scala plugin: discovery         | Scala repositories are discovered and shaped |
| M20       | Scala dependency graph          | Scala imports become typed graph edges      |
| M21       | Scala semantic index            | Offline Scala symbol index with warm reuse  |
| M22       | Scala analysis outputs          | Load, explain, plan and Graph UI cover Scala |
| M23       | Symbol isolation (C++ + Scala)  | Extraction emits real code, not placeholders |

The first meaningful product should exist around **M6**.

The first genuinely transformative version should exist around **M10–M11**.

---

# 3. M0 — Project Skeleton & Test Harness

## Objective

Establish the Rust workspace, CLI, test infrastructure, artifact conventions, and a tiny synthetic C++ fixture repository.

No real analysis is required yet.

## Deliverables

```text
crates/
├── cli/
├── repo/
├── build/
├── scheduler/
├── index/
├── graph/
├── cluster/
├── transform/
├── validate/
└── report/

tools/
└── clang-worker/

tests/
├── fixtures/
│   └── basic/
└── integration/
```

CLI:

```text
tool --help
tool version
tool analyze <repo>
```

Artifact root:

```text
artifacts/
```

## Test fixture

Create a deliberately small C++ repository containing:

```text
fixture/
├── include/
│   ├── thing.h
│   ├── thing_types.h
│   └── heavy.h
├── src/
│   └── thing.cpp
└── tests/
    └── consumer.cpp
```

The fixture should contain at least one intentional collateral dependency.

## Test

```bash
cargo test --workspace
tool --help
tool version
```

Expected result:

* workspace builds
* all tests pass
* CLI starts
* fixture repository can be located
* artifact directory can be created

## Proves

The development environment and testing model are operational.

## Does not prove

Anything about C++ analysis.

---

# 4. M1 — Repository Discovery

## Objective

Discover the source tree independently of MSBuild project files.

The source tree is the authoritative inventory of source files.

## Capabilities

The tool should:

* enumerate source files
* classify file types
* canonicalize paths
* identify generated directories
* assign ownership
* assign optimization policy
* calculate content hashes
* produce stable `FileId`s

## CLI

```bash
tool discover <repo>
```

Example:

```text
Repository
  Root: C:\src\project

Files:
  C:\src\project\include\thing.h
  C:\src\project\include\heavy.h
  C:\src\project\src\thing.cpp

Ownership:
  Repository: 3
  ThirdParty: 0
  Toolchain: 0
  Generated: 0

Optimization policy:
  Optimizable: 3
  Contract: 0
  Frozen: 0
```

## Artifact

```text
artifacts/repository/
├── files.bin
├── files.json
└── manifest.json
```

JSON is useful during early development even if production artifacts later become binary.

## Tests

Unit tests:

* path canonicalization
* stable IDs
* ownership classification
* generated-directory detection
* policy precedence

Integration test:

```bash
tool discover tests/fixtures/basic
```

Compare output against a golden file.

## Proves

The tool has a reliable understanding of what source code belongs to the repository.

## Does not prove

Which files actually participate in compilation.

---

# 5. M2 — Build & Compilation Discovery

## Objective

Understand how the repository actually compiles.

The tool should recover compilation commands rather than attempting to reconstruct MSBuild semantics itself.

## Capabilities

Discover:

* `cl.exe`
* compiler/toolset
* architecture
* language standard
* include paths
* system include paths
* forced includes
* preprocessor definitions
* configuration
* translation units

Where necessary, use captured compiler invocations such as those produced by Catter.

## CLI

```bash
tool build discover <repo>
```

Output:

```text
Compilation contexts: 2

Context: Debug|x64
  Compiler: cl.exe
  Definitions:
    DEBUG
  Include roots:
    ...

Context: Release|x64
  Compiler: cl.exe
  Definitions:
    NDEBUG
  Include roots:
    ...

Translation units: 842
```

## Artifact

```text
artifacts/contexts/
├── contexts.json
├── commands.json
└── manifest.json
```

## Tests

Fixture with:

```text
DEBUG
NDEBUG
```

Verify that:

* Debug produces `DEBUG`
* Release produces `NDEBUG`
* include paths are preserved
* compiler options are preserved
* translation units are deterministic

## Acceptance criterion

Running discovery twice against the same build configuration produces equivalent normalized compilation contexts.

## Proves

We know what compiler environments actually exist.

## Does not prove

What those compilation commands actually include.

---

# 6. M3 — Include Graph

## Objective

Build the physical file dependency graph.

```text
File → File
```

## Capabilities

For each translation unit:

* resolve `#include`
* distinguish local/system/external headers
* record include edges
* calculate include closure
* associate headers with consuming TUs

Use `clang-scan-deps` where appropriate, supplemented by repository-level textual analysis when needed.

## CLI

```bash
tool includes <repo>
tool includes explain <repo> --file thing.h
```

Example:

```text
thing.h
  ├── thing_types.h
  ├── graphics.h
  │   └── HugeSDK.h
  └── utility.h
```

## Artifact

```text
artifacts/includes/
├── edges.bin
├── closures.bin
└── report.json
```

## Tests

Fixture:

```text
thing.h
  -> graphics.h
       -> heavy.h
```

Assert exact graph:

```text
thing.h → graphics.h
graphics.h → heavy.h
```

Test:

* duplicate includes
* include guards / pragma once
* system headers
* missing includes
* cycles
* canonical paths

## Proves

We understand the physical include structure.

## Does not prove

That every included symbol is semantically required.

---

# 7. M4 — Semantic Indexing

## Objective

Introduce the Clang/Clice semantic backend.

This is the first major technical spike.

## Capabilities

For each translation unit, identify:

* declarations
* definitions
* canonical symbols
* symbol ownership
* references
* semantic dependencies
* Complete vs Incomplete dependency kind
* TU usage

## Semantic worker boundary

Rust:

```text
CompileCommand
      ↓
semantic worker
      ↓
TU analysis
      ↓
versioned binary result
```

The Rust side should not need to understand Clang's internal memory model.

## CLI

```bash
tool index <repo>
```

Example:

```text
Indexed 842 translation units

Symbols:
  41,203

Semantic edges:
  182,441

Complete:
  74,212

Incomplete:
  108,229
```

## Critical tests

Construct tiny fixtures for:

### Forward declaration

```cpp
class B;

class A {
    B* value;
};
```

Expected:

```text
A → B : Incomplete
```

### By-value member

```cpp
class B {};

class A {
    B value;
};
```

Expected:

```text
A → B : Complete
```

### Inheritance

```cpp
class B {};
class A : public B {};
```

Expected:

```text
A → B : Complete
```

### `sizeof`

```cpp
sizeof(B);
```

Expected:

```text
A → B : Complete
```

### Function declaration

```cpp
B make_b();
```

Expected:

```text
A → B : Incomplete
```

These become permanent semantic regression tests.

## Proves

The system can distinguish physical inclusion from semantic dependency.

## Does not prove

That the project-wide graph has been correctly reduced.

---

# 8. M5 — Reduced Semantic Model

## Objective

Turn independent TU analysis results into one deterministic project-wide semantic model.

## Capabilities

Perform:

* symbol identity resolution
* definition aggregation
* dependency deduplication
* TU usage aggregation
* file/symbol indexing
* graph construction

## Query interface

```bash
tool query symbol <repo> Thing
tool query deps <repo> Thing
tool query users <repo> Thing
```

Example:

```text
Thing
  Definition:
    include/thing.h

  Semantic dependencies:
    ThingOptions
    GraphicsAdapter

  Used by:
    src/foo.cpp
    src/bar.cpp
```

## Artifact

```text
artifacts/reduced/
├── symbols.bin
├── edges.bin
├── tu-usage.bin
├── file-symbols.bin
└── manifest.json
```

## Tests

Given multiple TUs referencing the same symbol:

* symbol exists exactly once
* references aggregate correctly
* dependency edges deduplicate
* TU usage bitmap is correct

## Determinism test

Run indexing twice.

The reduced artifact hashes must match.

## Proves

The tool has a stable project-wide semantic model.

---

# 9. M6 — Dependency Load Analysis

## Objective

Turn the graph into the core product concept: **load**.

## Capabilities

For any symbol or file, calculate:

```text
Semantic load
Include load
Collateral load
Total load
```

Preserve provenance for collateral dependencies.

## CLI

```bash
tool analyze load <repo> --symbol Thing
```

Example:

```text
Thing

Semantic load:
    17 symbols

Include load:
    4,399 symbols

Collateral load:
    4,382 symbols

Largest collateral source:
    HugeThirdParty.h
    4,121 symbols
```

## Explain command

```bash
tool explain load <repo> --symbol Thing
```

Output:

```text
Thing
  ↓
thing.h
  ↓
graphics.h
  ↓
HugeThirdParty.h

Reason:
  HugeThirdParty.h is included by graphics.h.
  No semantic dependency from Thing reaches symbols owned by
  HugeThirdParty.h.
```

## Tests

Synthetic fixture with known numbers.

The test should assert exact:

```text
semantic load
collateral load
total load
```

Also test a dependency that is both:

* physically included
* semantically required

It must **not** be classified as collateral.

## Proves

The tool can identify the actual optimization target.

This is the first milestone where the product starts doing what it exists to do.

---

# 10. M7 — Optimization Planning

## Objective

Identify safe places where collateral load can be separated.

## Capabilities

The planner should consider:

* SCCs
* dependency direction
* ownership
* optimization policy
* movability
* complete/incomplete requirements
* templates
* macros
* header side effects
* context correctness
* self-containment
* cluster constraints

The output is a **plan**, not yet a mutation.

## CLI

```bash
tool plan <repo>
tool plan show <repo> --id PLAN-001
```

Example:

```text
PLAN-001

Target:
    thing.h

Problem:
    4,382 symbols of collateral load

Proposed extraction:
    ThingOptions → thing_types.h
    ThingBuilder → thing_builder.h

Optimization frontier:
    graphics.h

Expected result:
    4,382 → 1,140 collateral symbols

Risk:
    Conservative

Validation required:
    23 translation units
```

## Important property

Plans must be explainable.

A developer should be able to answer:

> Why did the tool propose this?

without reading the implementation.

## Tests

Golden-plan fixtures.

Given the same repository:

```text
tool plan fixture
```

must produce the same plan.

Test rejection cases:

* cycle creation
* frozen file
* contract violation
* non-movable declaration
* unresolved semantic dependency

## Proves

The system can turn analysis into an actionable recommendation without modifying the repository.

---

# 11. M8 — Header Extraction

## Objective

Generate the proposed headers without changing consumers.

## Capabilities

Generate:

```text
thing_fwd.h
thing_types.h
thing_core.h
```

with:

* correct declarations
* correct definitions
* required predecessor includes
* include guards / `#pragma once`
* deterministic formatting/order

## CLI

```bash
tool transform extract <repo> --plan PLAN-001
```

Output:

```text
generated/
├── thing_fwd.h
├── thing_types.h
└── thing_core.h
```

The original repository is not modified yet.

## Tests

### Structural

Verify:

* every moved symbol appears exactly once
* no symbol is lost
* generated headers compile independently where required

### Self-containment

For every generated header:

```cpp
#include "thing_types.h"
int main() {}
```

must compile.

### Graph validation

Generated include graph must be acyclic.

## Proves

The planner can produce actual compilable structural changes.

---

# 12. M9 — Consumer Include Rewriting

## Objective

Migrate consumers from the monolithic header to the smallest appropriate generated header.

## Capabilities

For each consumer:

1. determine referenced symbols
2. map symbols to generated headers
3. determine required predecessor headers
4. inspect side effects
5. generate replacement
6. validate the affected TU

## CLI

Preview:

```bash
tool migrate <repo> --plan PLAN-001 --preview
```

Output:

```text
src/foo.cpp
    - #include "thing.h"
    + #include "thing_types.h"

src/bar.cpp
    - #include "thing.h"
    + #include "thing_core.h"
```

No files changed.

## Tests

For each rewrite:

* old TU compiles
* rewritten TU compiles
* referenced symbols remain available
* unrelated side effects are not silently removed

Critical negative test:

If a header has no obvious direct symbol usage but has a known macro/registration side effect, the tool must **not** automatically remove it.

## Proves

The system can migrate real consumers rather than merely generating headers.

---

# 13. M10 — Transactional Transformation & Validation

## Objective

Make transformations safe to apply to a real repository.

## Lifecycle

```text
Plan
  ↓
Preview
  ↓
Apply
  ↓
Validate
  ↓
Commit / Rollback
```

## CLI

```bash
tool apply <repo> --plan PLAN-001
```

Possible result:

```text
Applying PLAN-001...

Structural validation: PASS
Generated header validation: PASS
Affected TU validation: PASS
Full build: PASS

Committed.
```

Failure:

```text
Applying PLAN-001...

Structural validation: PASS
Generated header validation: PASS
Affected TU validation: FAIL

Rolling back...

Repository restored.
```

## Validation tiers

### Tier 1 — Structural

Graph and file consistency.

### Tier 2 — Generated headers

Self-containment and compilation.

### Tier 3 — Affected TUs

Only consumers affected by the transformation.

### Tier 4 — Full build

Full repository compilation.

### Tier 5 — Behavioral tests

Repository test suite where available.

## Tests

The most important test is intentionally broken transformation:

```text
Apply
→ validation failure
→ rollback
→ repository hash identical to pre-apply state
```

## Proves

The tool can safely modify a real repository.

---

# 14. M11 — Build Performance Measurement

## Objective

Connect dependency optimization to the actual reason we are doing it: build performance.

## Capabilities

Measure before and after:

* preprocessing time
* compiler front-end CPU
* wall-clock time
* memory
* affected TU times
* full build time

Also report structural changes:

```text
Collateral load:
    4,382 → 1,140

Affected TUs:
    183

Measured front-end CPU:
    91.2s → 37.5s
```

## CLI

```bash
tool measure <repo>
```

And:

```bash
tool compare <before> <after>
```

## Tests

Use a controlled fixture with intentionally expensive headers.

The exact absolute timings should not be treated as deterministic unit-test assertions.

Instead test:

* measurement collection succeeds
* metrics are attributed to the correct TUs
* before/after comparison is calculated correctly
* structural and measured metrics are correlated in the report

## Proves

The tool's optimization objective can be evaluated in the real compiler.

---

# 15. M12 — Incremental Analysis

## Objective

Avoid reanalyzing unchanged code.

## Capabilities

Cache based on:

* file content
* compilation context
* compiler/toolchain version
* analysis version
* relevant configuration

A change to one source file should invalidate only the necessary analysis.

## CLI

```bash
tool analyze <repo>
```

Second run:

```text
Analyzing...

Reused:
    817 translation units

Reanalyzed:
    25 translation units

Cache hit rate:
    97.0%
```

## Tests

Modify one fixture header.

Assert:

* dependent analysis is invalidated
* unrelated TUs remain cached
* changing compiler context invalidates appropriate artifacts
* changing analysis version invalidates incompatible artifacts

## Proves

The tool can be used repeatedly during development rather than only as an occasional batch process.

---

# 16. M13 — Production-Scale Execution

## Objective

Scale the proven analysis pipeline to large repositories.

Only now should the high-throughput architecture become the primary concern.

## Capabilities

Introduce:

* process-level worker isolation
* TU scheduling
* shard generation
* hash-partitioned shuffle
* parallel reducers
* memory-bounded execution
* progress reporting
* resumability
* deterministic artifacts

## CLI

```bash
tool analyze <repo> --workers 32
```

Output:

```text
Translation units: 8,421

Workers:
    32

Progress:
    8,421 / 8,421

Semantic edges:
    12,381,442

Peak memory:
    31.4 GB

Elapsed:
    8m 42s
```

## Tests

### Determinism

Run:

```bash
tool analyze repo --workers 8
tool analyze repo --workers 32
```

Reduced semantic output must be equivalent.

### Failure recovery

Kill a worker.

The scheduler should recover or resume without corrupting the final model.

### Memory

Run against a large fixture and verify bounded worker memory.

### Straggler scheduling

Verify that large TUs do not all run at the end of the schedule.

## Proves

The architecture scales beyond the toy implementation.

---

# 17. M14 — End-to-End Optimization

## Objective

The complete workflow should be usable by an engineer without manually orchestrating individual subsystems.

## CLI

```bash
tool optimize <repo>
```

The tool performs:

```text
Discover
   ↓
Build discovery
   ↓
Include analysis
   ↓
Semantic analysis
   ↓
Load analysis
   ↓
Optimization planning
   ↓
Preview
   ↓
Transformation
   ↓
Validation
   ↓
Measurement
   ↓
Report
```

## Example

```text
$ tool optimize C:\src\project

Repository:
    14,821 files
    3,412 translation units

Analysis:
    182,441 symbols
    4.1M semantic edges

Largest collateral-load opportunities:

    1. graphics.h
       4,382 collateral symbols

    2. networking.h
       2,941 collateral symbols

    3. serialization.h
       1,882 collateral symbols

Plan:
    7 safe transformations available

Preview:
    183 consumer files affected

Validation:
    183 / 183 affected TUs passed

Build:
    PASS

Measurement:
    Front-end CPU: 91.2s → 54.7s

Committed:
    7 transformations

Residual collateral load:
    8,241 symbols
```

## Proves

The complete product loop works.

---

# 18. Milestone Quality Gates

Every milestone must satisfy four gates.

## Gate A — Functional

The capability works on its intended fixture.

## Gate B — Regression

Existing tests continue to pass.

## Gate C — Deterministic

Equivalent input produces equivalent artifacts/results.

## Gate D — Observable

A developer can inspect what happened.

A milestone is not complete merely because the underlying code exists.

---

# 19. Test Repository Strategy

We should maintain several levels of test repositories.

## Fixture 1 — Minimal

Tests individual semantic rules.

```text
tests/fixtures/semantic/
```

Examples:

* forward declarations
* by-value members
* inheritance
* templates
* inline methods
* `sizeof`
* aliases

## Fixture 2 — Optimization

A deliberately badly structured header.

```text
tests/fixtures/optimization/
```

It should contain:

```text
Small useful symbols
        +
heavy unrelated dependency
        +
third-party boundary
```

This becomes the canonical end-to-end test.

## Fixture 3 — Context

Tests:

```text
DEBUG
NDEBUG
```

and differences between compilation contexts.

## Fixture 4 — Side Effects

Tests:

* macros
* registration
* initialization
* pragmas
* order-sensitive behavior

## Fixture 5 — Large Synthetic Repository

Used for:

* scheduler tests
* memory tests
* shard tests
* reducer tests
* incremental tests

## Fixture 6 — Real Repository

Eventually maintain a representative internal codebase as the primary integration benchmark.

---

# 20. What We Should Be Able to Demo at Each Stage

The milestones should tell a coherent story.

### After M1

> "Point it at a repository and it tells me what source code exists and what we're allowed to change."

### After M2

> "It understands how this repository actually builds."

### After M3

> "It can show me why a header causes another header to be processed."

### After M4

> "It understands what the C++ code actually depends on."

### After M5

> "It has a project-wide semantic dependency model."

### After M6

> "It can tell me how much unnecessary load a symbol carries and where that load comes from."

### After M7

> "It can tell me what I could safely change and why."

### After M8

> "It can generate the proposed smaller headers."

### After M9

> "It can migrate consumers to those headers."

### After M10

> "It can safely make the change and undo it if validation fails."

### After M11

> "It can prove whether the change actually improves builds."

### After M12

> "I can run it repeatedly without paying the full analysis cost every time."

### After M13

> "It can handle a large real-world repository efficiently."

### After M14

> "I can point it at a repository and have it find, explain, safely perform, validate, and measure dependency optimizations."

---

# 21. The First Vertical Slice

We should **not** begin by implementing the complete architecture.

The first real target should be:

```text
M0
 ↓
M1
 ↓
M2
 ↓
M3
 ↓
M4
 ↓
M5
 ↓
M6
```

At M6 we should stop and evaluate the product.

The first compelling demonstration should be:

```text
tool analyze load fixture --symbol Thing
```

producing an explanation such as:

```text
Thing

Required:
    17 symbols

Collateral:
    4,382 symbols

Primary source:
    HugeThirdParty.h

Why:
    Thing.h includes Graphics.h.
    Graphics.h includes HugeThirdParty.h.
    Thing has no semantic dependency on symbols from HugeThirdParty.h.

Potential frontier:
    Graphics.h

Estimated opportunity:
    4,382 symbols
```

If we cannot produce a result like this accurately and convincingly, optimizing the execution engine or clustering algorithm is premature.

---

# 22. Implementation Priority

The priority order is therefore:

```text
                    PRODUCT VALUE
                         ↑
                         │
                 ┌───────┴───────┐
                 │ Load Analysis │  ← first major proof
                 └───────┬───────┘
                         │
              Semantic Dependency Model
                         │
                   Include Graph
                         │
              Compilation Discovery
                         │
               Repository Discovery
                         │
                  Test Infrastructure
                         │
                         ↓
                 IMPLEMENTATION COST
```

Only after this vertical slice works should we invest heavily in:

* distributed execution
* sophisticated clustering
* persistent high-performance indexes
* advanced visualization
* automatic policy inference

---

# 23. Definition of Done for the Product

The project is ready for serious use when an engineer can run:

```bash
tool optimize <repository>
```

and receive:

1. a deterministic analysis of the repository
2. a model of its real compilation contexts
3. a physical include graph
4. a semantic dependency graph
5. quantified collateral load
6. explanations of major sources of collateral load
7. identified optimization frontiers
8. safe transformation plans
9. generated headers
10. consumer migration plans
11. validation results
12. rollback on failure
13. measured build impact
14. residual-load reporting
15. reproducible artifacts

The final product should make the following workflow routine:

```text
Find expensive collateral load
            ↓
Understand why it exists
            ↓
Identify where we can intervene
            ↓
Construct the smallest safe change
            ↓
Preview it
            ↓
Apply it transactionally
            ↓
Validate it
            ↓
Measure it
            ↓
Commit it
            ↓
Find the next largest remaining opportunity
```

That loop is the product.
The indexing, sharding, graph algorithms, clustering, and transformation machinery exist to make that loop accurate, safe, fast, and scalable.

---

# Part II -- Multi-Language and Plugin Milestones (M15-M23)

Part I (M0-M14) built the C++ dependency-optimization tool. Part II records the
milestones that made it language-neutral and added Scala as the first additional
language through the plugin mechanism. The governing spec is
`plans/multi-language-support-implementation_plan.md`; per-milestone kickoffs and
gate verdicts live in `plans/m19..m23-*-kickoff.md`.

---

## M15 -- Graph UI

### Objective

A browser-based graph interface over an analyzed repository exists, so the
include, symbol, and call graphs can be inspected without reading JSON
artifacts.

### Capabilities

- Serves `/api/graph/include`, `/api/graph/symbol`, and `/api/graph/call`, plus
  `/api/status`.
- Exports the whole view as a single standalone HTML file that embeds the graph
  payload as `window.__ARIADNE_DATA__`.
- Reports startup through `/api/status`: the suite waits for `progress.state` to
  reach `ready` and checks that `last_refresh` is set.

### CLI

`tool ui <repo> [--port 7742] [--no-open] [--export <path>] [--watch-interval 2] [--no-watch]`

### Artifacts

- The `--export` HTML file: a standalone document carrying the graph payload.

### Tests

- `tests/integration/tests/m15_graph_ui_test.rs` covers both the standalone
  export and the running HTTP server against `tests/fixtures/basic`.

### Proves

- A repository can be inspected graphically end to end, and the server answers
  without browser automation.

### Does not prove

- The suite asserts payload size and the embedded marker, not the rendered
  graph, so visual correctness is not locked.
- Traversal, search, and pathfinding are not claimed here; those are M16.
- This is a read-only view. Nothing transforms a file in this milestone.

Landed: 1ae948f (implements the graphical interface for symbol, include, and call graphs).

---

## M16 -- Clang worker and graph traversal

### Objective

C++ translation units can be analysed by an out-of-process worker when one is
available, and the graph UI gains interactive traversal and pathfinding.

### Capabilities

- `ClangWorkerBackend` spawns `tools/clang-worker` with the unit's `-D`/`-I`
  flags and degrades to the in-process C++ path when the worker, its response,
  or Clang itself is unavailable.
- Interactive traversal: neighbour stepping, entering/leaving edge highlight,
  filtering, a degree stepper, node capping, and dependency-tree isolation.
- Pathfinding, entry-point, and search endpoints over the same graphs.

### CLI

`tool ui <repo>` -- traversal is a capability of the existing surface, not a new command.

### Artifacts

- `crates/lang-cpp/src/backend.rs` holds the relocated `ClangWorkerBackend`.

### Tests

- `tests/integration/tests/m16_clang_worker_test.rs` locks graceful degradation
  and still-valid `TuAnalysis` output from a `WorkerPool`.
- `tests/integration/tests/m16_traversal_test.rs` drives the traversal endpoints
  against a live fixture server.
- `crates/ui/tests/ui_test/static_html.rs` carries the crate-local traversal
  element locks.

### Proves

- A missing worker never turns into an error; the pool still yields analyses.
- Traversal and pathfinding answer on a real fixture over HTTP.

### Does not prove

- `tools/clang-worker` ships in the tree and the backend knows how to find it,
  but with no C++ toolchain on PATH the real sidecar IPC path is never
  exercised: a test-only seam suppresses it, so the suite drives the in-process
  fallback instead.
- Worker coverage is C++-only; no plugin other than C++ has a worker backend.
- Traversal is exercised on `tests/fixtures/basic` only, not at production scale.

Landed: a42ab52 (implements interactive graph traversal and pathfinding); the worker IPC fix it rides on is 0b37b1a.

---

## M17 -- Language API seam

### Objective

The plugin vocabulary exists and is wired, while C++ still runs exactly its
previous code paths.

### Capabilities

- New `ariadne-lang-api` crate: the value types moved verbatim out of
  `crates/index/src/types.rs`, plus modeling, capability, catalog, and unit
  vocabulary.
- `LanguagePlugin` gains `id`, `modeling`, `capabilities`,
  `analysis_fingerprint`, and `shaper`; the registry gains `plugin_for_id`,
  `languages`, and `detect_language`.
- `DependencyResolver::resolve_target`, `FileCatalog` with `ResolveOutcome`,
  `CompilerValidator::toolchain_status`, and `UnitShaper` complete the seam.
- The `lang -> index` dependency is gone; `index` re-exports the moved types.

### CLI

not user-visible

### Artifacts

- `TuAnalysis` gains a `#[serde(default)] language` field and `SymbolKind` gains
  `Trait`/`Object`/`Module`, so pre-existing artifacts still decode as C++.

### Tests

- `tests/integration/tests/m17_plugin_seam_test.rs`, including the architecture
  guards that core crates name no language and plugin crates avoid the pipeline.

### Proves

- The seam is real and the guards hold: neutral vocabulary is available and
  unused by core crates.
- C++ behaviour is unchanged: the golden capture taken as M17.0 is byte-identical
  afterwards.

### Does not prove

- The spec's assumption A2 (no `libloading`, no WASM host) means there is no
  dynamic plugin loading. Plugins are linked at compile time only, which is why
  the out-of-process plugin host stayed parked as M24.
- A pure refactor proves nothing about a second language. M19 is the first.

Landed: 57cc995 (moves index value types into ariadne-lang-api, adds the neutral vocabulary, extends LanguagePlugin and the registry); preceded by b8a9de7 (the M17.0 golden capture).

---

## M18 -- C++ becomes a plugin

### Objective

The C++ implementation is relocated into `ariadne-lang-cpp` and every core stage
dispatches through the registry instead of naming C++.

### Capabilities

- New `ariadne-lang-cpp` crate holding `CppDependencyResolver`,
  `CppSemanticAnalyzer`, `CppUnitShaper`, `CppSourceExtractor`,
  `CppDependencyRewriter`, and `ClangWorkerBackend`.
- `default_registry()` becomes the one place plugins are enumerated; graph,
  index, build, scheduler, cluster, transform, and validate dispatch through it.
- The isolation engine goes neutral (`GeneratedHeader` becomes `GeneratedUnit`)
  and delegates all generated text to the plugin.
- `ValidationStatus::Skipped`, a language plus plugin-fingerprint cache key, and
  the analysis epoch move `1.0.0` to `2.0.0`.

### CLI

`tool analyze index <repo> --lang cpp|scala|other`

### Artifacts

- The reduced model's `defining_file` becomes the defining declaration's file and
  gains `declaring_files`; the epoch bump invalidates `symbols.bin`.

### Tests

- `tests/integration/tests/m18_cpp_plugin_parity_test.rs`

### Proves

- C++ behaviour is preserved on both fixtures: resolver output, worker dispatch,
  isolation-engine bytes, `MigrationPreview` JSON and diff text, and
  reduced-model values all match the pre-M18 output.

### Does not prove

- The isolation engine still emits `// Symbol: <name> (<kind>)` placeholder
  bodies, byte-locked on purpose. Real declaration text lands in M23.
- The neutrality guard had two admitted gaps at M18.11: core-crate test code
  still named plugin types, and `c++20` literals passed the forbidden-token list.
  M23 closes both.
- The `compile_commands` argument parser in `build` still falls back to the
  `cl.exe` literal when no compiler token precedes the flags.
- M18.11 found `pipeline` and `cli` each enumerating plugins. The CLI now
  delegates to the single pipeline composition root.

Landed: e8422ef (relocates the C++ engine into ariadne-lang-cpp); reviewer approval recorded in df57491 over 3b06e54.

---

## M19 -- Scala plugin: discovery and build context

### Objective

A Scala repository is discovered, classified by ownership, and shaped into
analysis units with no Scala toolchain present.

### Capabilities

- New `ariadne-lang-scala` crate with `ScalaPlugin`, `ScalaDetector`, and
  `ScalaUnitShaper`.
- `.scala` and `.sc` sources plus `build.sbt`, `build.sc`, and `pom.xml` are
  recognized; `lib/` and `target/` become the frozen external boundary.
- Registering the plugin in `default_registry()` is the only pipeline edit needed
  to add the language.

### CLI

`tool analyze discover <repo>`

### Artifacts

- `tests/fixtures/scala-basic/**` is the new fixture. It ships `lib/legacy.jar`
  so the frozen external boundary is exercised by a committed file.

### Tests

- `tests/integration/tests/m19_scala_discovery_test.rs`

### Proves

- Discovery lists the `.scala` files, shaping yields one unit per
  `src/main/scala` file with `compiler: "scalac"` and `language: Scala`, and no
  C++ diagnostics are produced.

### Does not prove

- Planning, extraction, rewriting, and validation stayed flagged `false`; the
  Scala dependency graph, semantic index, and analysis outputs are M20 to M22.
- `scalac` is a string literal only. Nothing is executed (offline assumption A3).
- The fixture's six-record discovery lock was deliberately left unedited by M20
  and M21.

Landed: d0c6499 (creates crates/lang-scala with ScalaPlugin, ScalaDetector, and ScalaUnitShaper); gate notes at 715/0/3 in plans/m19-scala-discovery-kickoff.md.

---

## M20 -- Scala dependency graph: imports become edges

### Objective

Scala `package` and `import` directives become typed dependency edges in the
existing graph.

### Capabilities

- `ScalaImportResolver` parses the `package` clause, `import a.b.C`, grouped
  imports, wildcards, and Scala 3 `export`, mapping targets through `FileCatalog`.
- `IncludeKind::Import` is a new additive variant with its bincode index pinned.
- `CyclePolicy::TolerateWithinPackage` reports an intra-package cycle as
  informational instead of a blocking defect.

### CLI

`tool analyze includes <repo>`

### Artifacts

- `tests/fixtures/scala-basic` gains a deliberate `models` <-> `services` cycle.

### Tests

- `tests/integration/tests/m20_scala_dependency_graph_test.rs`

### Proves

- The graph emits `OrderService.scala` to `models/Order.scala` and
  `models/User.scala` with `kind: "Import"`, `import scala.collection.mutable._`
  becomes `External`, and the C++ cycle suite still asserts real cycle
  reporting.

### Does not prove

- The cycle substrate rework (SCC groups, iterative Tarjan) was deliberately
  deferred out of M20 into a standalone follow-up.
- Marker-based shaper auto-selection was still deferred; the suite injects
  `ScalaUnitShaper` explicitly.
- These edges are syntactic. No symbol identity exists yet.

Landed: 77bd0a7 (execution notes recording completion); reviewer gates APPROVED at 75abb8e with the final gate 767/0/3.

---

## M21 -- Scala semantic index (offline, deterministic)

### Objective

A deterministic offline Scala parser produces a language-scoped semantic index
with working warm reuse.

### Capabilities

- `ScalaSourceParser` extracts packages, top-level declarations, `extends` and
  `with` edges, references, and call sites using a hand-written scanner, with no
  new dependency.
- `ScalaAnalyzer` is dispatched per unit. M21 set the plugin fingerprint to
  `scala-parser-1`; M23 bumped it to `scala-parser-2`.
- The index stage selects the plugin's unit shaper when a language filter is
  given, so a Scala repository is indexed through the Scala path only.

### CLI

`tool analyze index <repo> --lang scala`

### Artifacts

- No artifact schema change: `cache.rs` was untouched. The only cache action was
  the Scala plugin fingerprint bump, which is what invalidates cached Scala
  shards when the parser changes.

### Tests

- `tests/integration/tests/m21_scala_semantic_index_test.rs`

### Proves

- The cold run analyses every unit, the second run reports a full hit rate,
  editing `models/Order.scala` reanalyzes exactly {Order, OrderService}, and C++
  cache entries are not invalidated by the Scala fingerprint.

### Does not prove

- Same-package sibling resolution is in-file only. The offline per-file parser
  cannot see cross-file siblings, so the semantic graph is shallower than a real
  compiler's.
- `ScalaToolchainAnalyzer` was deferred so CI stays offline-deterministic.
- Unfiltered Scala shaping, that is marker auto-selection, was still open.

Landed: 33e9950 (interactive walkthrough and closeout notes); four reviewer gates APPROVED at acbb1a9 with the final gate 842/0/3.

---

## M22 -- Scala analysis outputs

### Objective

Load analysis, explanation, planning, reports, and the Graph UI speak Scala as
well as C++.

### Capabilities

- A Scala aggregation unit is a file declaring a top-level `package object`; a
  package without one gets no aggregation and no invented header.
- A capability-gated `PlanRejection` explains a disabled stage instead of
  silently returning nothing.
- Report manifests gain additive `languages` fields, and analyze/explain wording
  comes from `LanguageModeling` with an explicit offline-parser caveat.
- The Graph UI distinguishes C++ from Scala file nodes and renders
  `defining_file` plus `declaring_files` on symbol nodes.

### CLI

`tool load --symbol <symbol> <repo>`, `tool explain load <repo>`, `tool plan <repo>`

### Artifacts

- The repository, build-context, includes, and load manifests gain
  skip-when-empty `languages` fields, so C++ output stays byte-identical.

### Tests

- `tests/integration/tests/m22_scala_load_analysis_test.rs`

### Proves

- `tool load --symbol com.example.services.OrderService` reports non-zero
  collateral load attributed to `models/all.scala`, `explain` renders the import
  chain, `plan` prints the capability-gated rejection, and the C++ report and UI
  goldens are unchanged.

### Does not prove

- Scala `planning` was still `false`, so production never ran the Scala planner.
  The wiring was proven by unit tests and the planner-injection lane only.
- Sealed-trait grouping was honestly empty at this point: `SymbolRecord` carried
  no modifier bit. M23 adds the sealed modifier.
- The plan's LoadMetric-struct `language` fields were deferred; only the manifest
  fields landed.

Landed: 76f619c (M22 closeout); Gate-C ACCEPTED at 09530af with the final gate 916/0/3.

---

## M23 -- Symbol isolation made real for C++ and Scala

### Objective

The isolation primitive produces real code for both languages: verbatim symbol
bodies through one plugin extractor, and a rollback-capable move for Scala.

### Capabilities

- `locate_symbol_bodies` supplies verbatim declaration text to the extractor, and
  `SourceExtractor::extract` becomes the single body emitter for both the engine
  and the applier.
- Scala gains a real extractor, rewriter, and probe validator; its
  planning, extraction, rewriting, and validation capabilities flip to `true`.
- Scala isolation is a move, not a copy: the moved region is removed from the
  source file inside the existing snapshot/rollback transaction, and sealed
  traits stay with their subtypes.
- Consumer discovery, rewrite specs, and in-place placement move behind neutral
  `DependencyRewriter` and `SourceExtractor` hooks whose C++ defaults are
  byte-identical.

### CLI

`tool transform extract <repo> --plan <id>`, `tool transform migrate`, `tool transform apply`

### Artifacts

- New fixture `tests/fixtures/scala-isolation`; `ProposedExtraction` gains a
  `#[serde(default)] source_file`.

### Tests

- `tests/integration/tests/m23_symbol_isolation_test.rs`

### Proves

- Generated C++ extracted files now carry real declaration text. A Scala symbol
  moves to its own `{symbol}.scala`, the source region is gone, and a whole-tree
  rollback restores the exact prior bytes. C++ preview and diff locks stayed
  byte-identical, and the `m00` goldens differ only by additive `source_file`
  lines.

### Does not prove

- Real compiler invocation was deliberately NOT wired (amendment D-M23-12, after
  reviewer finding F3: extracted headers are not self-contained). Tier 3 stays
  `Skipped` for both languages. Extraction include-closure is a follow-up.
- The `AuditTrail` `source_file` provenance fix was deferred. Both Gate-G3 lanes
  ruled it incomplete rather than incorrect, and the planner rework churns the
  C++ plan goldens, so it was out of milestone scope.
- The placeholder body is still the total fallback when a body cannot be located.
- `ScalaToolchainAnalyzer` and package-scoped SCC condensation were deferred to a
  follow-up milestone.

Landed: 36b182a (Gate-G3 closure: both lanes APPROVED, M23 closed); the final full gate 1088/0/3 was verified at f9bcf69.
