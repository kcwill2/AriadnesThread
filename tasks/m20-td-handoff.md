# T-D Handoff — Scala fixture cycle + `m20_scala_dependency_graph_test` + final gate (M20 acceptance)

Kickoff: `plans/m20-scala-graph-kickoff.md` (D13, D14, acceptance). Base: T-C's commit. Final bounded session: fixture content edits, the plan:398 acceptance suite, then the **only** `--workspace` run of the milestone.

## 1. Intent & Invariants

**Objective:** prove M20 end-to-end on the `scala-basic` fixture through the real registry path (`build_dependency_graph_with_unit_shaper` + `default_registry`), per plan:398. Add the deliberate `models ↔ services` cycle the plan requires via **content-only** edits, and lock the full acceptance contract in a new integration suite.

**Invariants:**
- IV-1 `ariadne graph` (via `build_dependency_graph_with_unit_shaper(root, &default_registry(), &ScalaUnitShaper)`) on `scala-basic` yields edges `services/OrderService.scala → models/Order.scala` and `→ models/User.scala`, each `kind: IncludeKind::Import` and `language: Scala`, `destination: Some(<models file id>)`.
- IV-2 `import scala.collection.mutable._` (added to OrderService) resolves to `ResolveOutcome::External` ⇒ the edge is `kind: External`, `destination: None`. No edge points into `lib/` or `target/` (those paths are not even discovered — assert absence, mirroring `m19`'s `external_frozen_boundary_lock`).
- IV-3 The fixture has a real `models ↔ services` cycle: `Order.scala` imports `com.example.services.OrderService` (with a genuine usage) while `OrderService.scala` imports `com.example.models.{Order, User}`. `detect_cycles()` finds ≥1 cycle; `generate_report_with_cycle_policies(&scala_tolerate_policy)` sets `informational_cycle_count >= 1` (cycle present but **non-blocking**); `generate_report()` (all-Reject delegate) sets `informational_cycle_count == 0` while `cycle_count` still counts it.
- IV-4 Fixture edits are content-only inside existing files — the 6-record discovery set from `m19_scala_discovery_test` is unchanged; no file is added, removed, renamed, or moved. `tests/golden/**` must not change (scala-basic is not a golden fixture; m00 runs `basic`/`expensive` only).
- IV-5 C++ locks green **unedited**: `m3_include_graph_test`, `m00` (zero golden drift), `m18_cpp_plugin_parity` 22/22, `m20_apply_migrate_characterization` (the OLD-numbering guard), `m10`, `m14`, `m17` 5/5 (both architecture guards), `m19_scala_discovery` 5/5.
- IV-6 No Scala toolchain invoked (A3). `scalac`/`sbt` remain literals; the suite must pass with none on PATH.

## 2. Suite shape (mirrors existing style — see `m19_scala_discovery_test.rs`)

New file `tests/integration/tests/m20_scala_dependency_graph_test.rs` (package `ariadne-integration-tests`). Reuse the `isolated_repo` temp-dir copy pattern (`m18_cpp_plugin_parity/mod.rs` / `m19`'s `isolated_scala_repo`) and a `fixture_path` helper. `tests/integration/Cargo.toml` already dev-deps `ariadne-lang`, `ariadne-lang-scala`, `ariadne-lang-api` (M19 added these) — add `ariadne-graph` only if not already a regular dep (it is). Do NOT add language names to core crates to satisfy the test; drive through the public seams.

Tests to write (Red first — each fails until T-A/T-B/T-C seams are all present, which they will be at this point):
- `graph_emits_import_edges_for_order_service` — IV-1: assert the two `Import`/`Scala` edges by resolving each `IncludeEdge.destination` back to a rel path via the catalog / graph node table.
- `wildcard_package_import_is_external` — IV-2: `import scala.collection.mutable._` ⇒ an `External`-kind edge with `destination: None`.
- `fixture_cycle_is_informational_under_tolerate_policy` — IV-3: build the policy fn as `|_id| CyclePolicy::TolerateWithinPackage`; assert `informational_cycle_count >= 1` and `cycle_count >= 1`; then default `generate_report()` ⇒ `informational_cycle_count == 0`, `cycle_count` unchanged (same value).
- `no_edges_point_into_external_boundary` — IV-2: no edge destination resolves under `lib/` or `target/`.
- `cpp_locks_untouched_witness` (optional guard) — trivial pass-through asserting `m00`/`m3` ran; keep only if it adds signal, otherwise rely on the scoped commands.

## 3. Fixture edits (content-only, IV-4)

- `tests/fixtures/scala-basic/src/main/scala/com/example/models/Order.scala`: add `import com.example.services.OrderService` + a genuine use (e.g. a method `def describe(svc: OrderService): String = svc.summarize(...)` or any real reference so the import is not dead code). Keep the existing `package com.example.models` and `case class Order(...)`.
- `tests/fixtures/scala-basic/src/main/scala/com/example/services/OrderService.scala`: add `import scala.collection.mutable._` + a genuine use (e.g. `val buffer = ArrayBuffer[Int]()`) so the wildcard import is exercised, alongside its existing `import com.example.models.{Order, User}`.
- These are plain-text Scala comments/strings; the T-A directive parser must NOT have produced spurious edges from them — that is part of what this suite locks indirectly. UTF-8 discipline: content is ASCII; still use Read/Edit, verify `git diff --numstat`.

## 4. Plan-Task (RGB)

- **Red:** write the suite first against the T-C tip; each test fails for the *expected* reason (missing edge / wrong kind / missing policy field) — confirm the failure mode is an assertion, not a compile error, proving the seams are exercised from the outside.
- **Green:** the seams already exist (T-A/B/C); the suite should pass with only fixture content edits. If a test reveals a real gap in T-A/B/C, report it as a cross-task finding rather than silently widening this task's touch-set.
- **Final gate (only here):**
  ```powershell
  cargo test -p ariadne-integration-tests --test m20_scala_dependency_graph_test
  cargo test --workspace
  python tools/hygiene_check.py
  ```
  Then record the exact workspace pass/fail/skip triple and the per-lock suite counts in the milestone execution notes.

## 5. Touch-set

`tests/fixtures/scala-basic/**` (two `.scala` file contents) · `tests/integration/tests/m20_scala_dependency_graph_test.rs` (new) · `tests/integration/Cargo.toml` (only if a dev-dep is genuinely missing) · **nothing else.** No `crates/**` edits — if one seems necessary it is a T-A/B/C defect.

## 6. Clauses

- **m00 golden drift = STOP.** Report the `git diff --stat tests/golden` result in the commit body (expected: empty).
- m3, m18, m20_apply_migrate, m10, m14, m17, m19 suites must be green **unedited**; do not touch them to make this pass.
- `cargo test --workspace` timeout 600000 ms. Commit on green (`test(m20): Scala dependency-graph acceptance + fixture cycle`). If the workspace run reveals failures you cannot fix inside the touch-set, STOP and report — do not expand scope.
- Report back: workspace triple (P/F/S), each lock suite count, the two fixture diffs, and the `git diff --numstat tests/golden` (empty expected).
