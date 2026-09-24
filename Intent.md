# Intent

## 1. Purpose

We are building a tool that makes large C++ codebases cheaper to compile by **reducing unnecessary dependency load**.

The primary problem is not simply that headers are large. The deeper problem is that a consumer of a small piece of functionality is often forced to process a much larger dependency graph than the functionality actually requires.

A single header may contain many loosely related symbols, and those symbols may collectively pull in substantial internal, third-party, or toolchain dependencies. As a result, a translation unit can pay the compilation cost of thousands of symbols when it only needs a small subset.

The purpose of this tool is to discover that unnecessary load and safely restructure the code so that consumers can depend on smaller, more appropriate units.

---

## 2. The Core Idea

The tool should answer four questions:

1. **What does each symbol actually depend on?**
2. **What additional code is being pulled in only because of the current header structure?**
3. **Where can that unnecessary load be safely separated?**
4. **Does the resulting change actually reduce compilation cost without changing program behavior?**

The tool should move from:

> "This header is huge."

to:

> "This symbol requires these dependencies, but its current header structure additionally causes these other dependencies to be processed. These dependencies can be separated at this boundary, and the resulting change was validated."

---

## 3. The Mental Model

The system should reason about three distinct graphs.

### Semantic Graph

**Symbol → Symbol**

This represents dependencies required by C++ semantics.

For example:

* inheritance
* by-value members
* expressions requiring complete types
* inline implementations
* template definitions and instantiations
* other relationships that genuinely require another symbol

The semantic graph answers:

> "What does this symbol actually need?"

### Include Graph

**File → File**

This represents the physical organization of the source tree and the files pulled in through `#include`.

The include graph answers:

> "What does this file cause the compiler to process?"

### File/Symbol Mapping

**File → Symbols**

This connects the physical organization of the source tree to the semantic model.

Together these models allow us to distinguish:

```text
Symbol
  ↓
Semantic Dependencies
  ↓
Files containing those dependencies
  ↓
Include Closure
  ↓
Collateral Load
```

These concepts must remain separate. A file being included does not automatically mean that every symbol in that file is semantically required.

---

## 4. Dependency Load

Every symbol has a compilation **load**.

Its load consists of:

* the symbol itself
* its required semantic dependencies
* the files required to obtain those dependencies
* additional dependencies pulled in by the current organization of the headers

The important distinction is between **required load** and **collateral load**.

### Semantic Load

The code that is genuinely required for the symbol to be understood and compiled correctly.

### Collateral Load

Code that is processed because of the current include structure but is not part of the symbol's required semantic dependency closure.

This collateral load is the primary optimization target.

### Residual Load

Residual load is **collateral load that remains after an optimization or refactoring step**.

It is not a separate kind of dependency.

The optimization loop is therefore:

```text
Measure
   ↓
Identify collateral load
   ↓
Find a safe optimization boundary
   ↓
Refactor
   ↓
Measure residual load
   ↓
Repeat
```

The goal is not to eliminate every dependency.

The goal is to eliminate **unnecessary collateral load while preserving correctness and useful architectural boundaries**.

---

## 5. The Optimization Frontier

The optimizer cannot change everything it encounters.

Some code is:

* owned by the repository and safe to restructure
* third-party code
* compiler/toolchain code
* generated code
* part of an externally exposed contract
* explicitly frozen

The tool therefore needs to understand where it is allowed to intervene.

An **optimization frontier** is the point in the dependency graph where the optimizer can make a change before encountering a dependency that it cannot or should not modify.

For example:

```text
Application
    ↓
GraphicsAdapter.h        ← optimization frontier
    ↓
HugeSDK.h                 ← third-party / frozen
```

The goal is not to modify `HugeSDK.h`.

The goal is to restructure the repository-owned code immediately before that boundary so that consumers do not unnecessarily inherit the full cost of the SDK dependency.

---

## 6. What We Want the Tool to Discover

The tool should be able to explain expensive dependencies in terms that are useful to an engineer.

For example:

```text
Thing

Required semantic load:
    17 symbols

Collateral load:
    4,382 symbols

Largest collateral source:
    HugeThirdParty.h
    4,121 symbols

Reason:
    Thing.h includes Graphics.h
    Graphics.h includes HugeThirdParty.h
    Thing does not semantically depend on HugeThirdParty.h

Optimization frontier:
    Graphics.h
```

The important output is not merely a number.

The tool should explain:

**what is expensive, why it is expensive, where that cost originates, and where it can safely be separated.**

---

## 7. What the Tool Should Change

The tool should be able to transform a monolithic header into smaller, coherent headers when doing so reduces dependency load.

For example:

```text
Before:

thing.h
├── Thing
├── ThingBuilder
├── ThingOptions
├── ThingInternals
└── heavy dependencies
```

into something closer to:

```text
After:

thing_fwd.h
thing_types.h
thing_core.h
thing_builder.h
```

with dependencies between the resulting headers following a valid DAG.

Consumers should then be able to include the smallest appropriate header rather than inheriting the complete dependency structure of the original monolithic header.

---

## 8. Compatibility Matters

The tool must support incremental adoption.

Existing include paths should not need to break immediately.

Where appropriate, the original header can become a compatibility shim:

```cpp
#pragma once

#include "thing_types.h"
#include "thing_core.h"
#include "thing_builder.h"
```

This allows the repository to migrate gradually while consumers are rewritten to depend directly on the new smaller headers.

The tool should therefore support both:

* **structural transformation**, and
* **consumer migration**.

---

## 9. Correctness Comes Before Optimization

The tool is a refactoring system, not merely a graph partitioner.

A proposed optimization is only useful if it preserves the semantics of the program.

The system must account for C++ constructs that affect whether a declaration can be moved or separated, including:

* complete versus incomplete type requirements
* templates and instantiations
* inheritance
* inline definitions
* explicit specializations
* linkage and ODR constraints
* macros
* initialization and registration side effects
* source-order dependencies
* context-dependent compilation
* generated code
* public contracts

When uncertainty exists, the tool should prefer a conservative result over an unsafe transformation.

---

## 10. Compilation Context Matters

C++ source does not have one universal interpretation independent of its build context.

The analysis must respect the actual compilation environments used by the repository, including relevant:

* compiler and toolchain
* architecture
* language standard
* include paths
* forced includes
* preprocessor definitions
* build configurations

The tool should analyze the real compilation contexts rather than inventing arbitrary combinations of preprocessor states.

For the target codebase, precompiled headers are explicitly out of scope.

---

## 11. Measurement Is Part of the Product

Structural improvement is not sufficient.

The tool ultimately exists to improve build performance.

Therefore every meaningful transformation should be measurable.

The system should distinguish between:

### Structural Load

The dependency relationships implied by the source graph.

### Estimated Compilation Load

Approximate compiler work such as:

* preprocessed tokens
* declarations
* AST nodes
* template instantiations

### Measured Compilation Cost

Observed build behavior such as:

* front-end CPU time
* wall-clock time
* preprocessing cost
* memory consumption
* affected translation-unit build time

The tool should eventually be able to demonstrate:

```text
Before:
    Total load:       5,000 symbols
    Collateral load:  4,880 symbols

After:
    Total load:       1,500 symbols
    Collateral load:  1,200 symbols

Residual collateral load:
    1,200 symbols
```

The ultimate validation is whether those structural improvements correspond to real build improvements.

---

## 12. The Tool Should Explain Its Work

A developer should be able to inspect a recommendation and understand:

* what symbols were analyzed
* what dependencies were discovered
* which dependencies are semantic
* which dependencies are collateral
* where collateral load originates
* which files form the optimization frontier
* what transformation is proposed
* why the transformation is considered safe
* which consumers will change
* what validation was performed
* how much load remains afterward

The output should therefore be both **machine-actionable and human-readable**.

---

## 13. The Transformation Loop

The fundamental workflow is:

```text
Discover
   ↓
Analyze
   ↓
Explain
   ↓
Identify collateral load
   ↓
Find optimization frontier
   ↓
Plan transformation
   ↓
Preview
   ↓
Apply
   ↓
Validate
   ↓
Measure
   ↓
Commit or Roll Back
```

The system should never treat a successful graph transformation as sufficient evidence of correctness.

A transformation becomes successful only after the relevant validation has passed.

---

## 14. What Success Looks Like

Success is not:

> "We split a large header into several files."

Success is:

> "We identified unnecessary compilation load, found a safe architectural boundary, separated the dependency structure, migrated consumers to the smaller interfaces, preserved correctness, and demonstrated reduced compilation cost."

The resulting repository should have:

* smaller dependency surfaces
* less collateral compilation
* clearer ownership of dependencies
* more focused headers
* more direct consumer dependencies
* preserved compatibility where required
* measurable build-time improvements

---

## 15. Guiding Principles

### 1. Optimize load, not line count

A smaller header is not necessarily a cheaper header.

### 2. Understand semantics before changing structure

The tool must know why a dependency exists before attempting to remove it.

### 3. Separate semantic dependencies from include dependencies

Physical inclusion and semantic necessity are different concepts.

### 4. Optimize at safe boundaries

Third-party, toolchain, generated, contract, and frozen code establish boundaries rather than targets for arbitrary modification.

### 5. Prefer small, explainable transformations

The system should make changes that an engineer can understand and review.

### 6. Preserve compatibility where practical

Large repositories need incremental migration rather than an all-at-once rewrite.

### 7. Be conservative under uncertainty

An optimization opportunity that cannot be proven safe should remain a recommendation rather than an automatic transformation.

### 8. Validate every transformation

Structural correctness, compilation correctness, and ultimately behavioral correctness all matter.

### 9. Measure the result

The objective is reduced compilation cost, not merely a prettier dependency graph.

### 10. Iterate on residual load

After every successful optimization, inspect what remains and identify the next meaningful source of collateral load.

---

## 16. The Product in One Sentence

**Analyze the real C++ dependency graph, explain where compilation load comes from, safely separate unnecessary collateral dependencies, and prove that the resulting code is both correct and cheaper to build.**

---

## 17. Relationship to the Technical Specification

`Intent.md` defines **why the system exists and what it must accomplish**.

`Spec.md` defines **the architecture and mechanisms used to accomplish it**.

The implementation should be judged against both:

```text
Intent
  │
  │ defines purpose, constraints, and success
  ▼
Specification
  │
  │ defines architecture and algorithms
  ▼
Implementation
  │
  │ produces
  ▼
Analysis → Transformation → Validation → Measurement
```

The implementation is successful only when the mechanisms described in the specification serve the goals established here.
