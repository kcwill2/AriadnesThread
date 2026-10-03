# T-C Handoff — CyclePolicy honored in cycle reporting (M20.2 cycle policy)

Kickoff: `plans/m20-scala-graph-kickoff.md` (D11). Base: T-B's commit. Bounded session: touch-set, scoped commands, commit on green.

## 1. Intent & Invariants

**Objective:** make `generate_report` able to classify detected cycles as **informational** (all members tolerate) vs **blocking** (any member rejects), WITHOUT replacing `detect_cycles()` and WITHOUT changing the default report. A `TolerateWithinPackage` language's cycle counts in `cycle_count` (m3 stays green) but also surfaces in a new `informational_cycle_count` that is **absent from serde when zero** — so every existing artifact is byte-identical.

**Hard boundary:** do NOT implement `detect_cycle_groups`, iterative Tarjan, or the `CycleGroup` contract — that is M23.5 (user decision D3, spec `.kilo/plans/1790821308681-cycle-substrate-scc-m23_5.md`). Reuse the current `detect_cycles() -> Vec<Vec<FileId>>` output verbatim; this task only *classifies* each returned cycle.

**Invariants:**
- IV-1 `IncludeReport` gains `informational_cycle_count: usize` with `#[serde(default, skip_serializing_if = "IsZero::is_zero")]` (or an equivalent local predicate). When the count is 0 (every C++ case), `report.json` is byte-identical to today.
- IV-2 `include_cycle_count`/`cycle_count` semantics UNCHANGED: still `self.detect_cycles().len()`, counting all cycles regardless of policy. m3's `cycle_count > 0` assertion holds untouched.
- IV-3 New `IncludeGraph::generate_report_with_cycle_policies(&self, policy_of: &dyn Fn(FileId) -> CyclePolicy) -> IncludeReport`: same as `generate_report` except `informational_cycle_count` = number of cycles where **every** member file's `policy_of(file)` is non-`Reject` (`Tolerate` or `TolerateWithinPackage`). A mixed-member cycle (≥1 `Reject`) is NOT informational.
- IV-4 `generate_report(&self)` keeps its exact signature and delegates: `self.generate_report_with_cycle_policies(&|_id| CyclePolicy::Reject)`. Because all-Reject ⇒ zero informational ⇒ field skipped ⇒ today's bytes. All existing callers are unchanged.
- IV-5 `pipeline/src/lib.rs` (~339-343, the sole production `generate_report()` site) switches to `generate_report_with_cycle_policies(&|file_id| <policy from record.language via registry>)`: resolve `CyclePolicy` = `record.language` → `registry.plugin_for_id(lang).map(|p| p.modeling().cycle_policy).unwrap_or(CyclePolicy::Reject)`. Use the existing discovery records the pipeline already holds. This is an R3-legal registry lookup — no `match LanguageId` in graph/pipeline core.
- IV-6 `crates/cluster/src/planner.rs`: behavior UNCHANGED. Update the `cycle_policy` field doc (~36-41) to state what shipped in M20 (report-side informational classification for tolerate-languages; planner SCC co-proposal path shared across policies) and that per-package cycle precision is superseded by the M23.5 group-condensation milestone. Do not read the field into a new branch; `planner_injection.rs:315` must stay green.
- IV-7 No language name in `graph.rs` core (R2). The policy arrives purely through the injected `policy_of` closure; `graph.rs` imports `CyclePolicy` from `ariadne-lang-api` (already a dependency) and names only `Reject`/`Tolerate*`.

## 2. API contracts

```rust
// crates/lang-api/src/types.rs — IncludeReport additive field
#[serde(default, skip_serializing_if = "…zero…")]
pub informational_cycle_count: usize,

// crates/graph/src/graph.rs
pub fn generate_report_with_cycle_policies(&self, policy_of: &dyn Fn(FileId) -> CyclePolicy) -> IncludeReport;
pub fn generate_report(&self) -> IncludeReport { self.generate_report_with_cycle_policies(&|_| CyclePolicy::Reject) }

// crates/pipeline/src/lib.rs — call site switched to the policy variant (see IV-5)
```

`CyclePolicy` is already `#[derive(...)]`-equipped in `lang-api/src/modeling.rs`; it needs no edit beyond what already exists. Confirm it is re-exported where `graph.rs` can name it (it is used by `planner.rs` already via `ariadne_lang`).

## 3. Plan-Task (RGB)

- **Red:**
  - `graph.rs` unit tests: (a) a 2-file mutual cycle with both members tolerating ⇒ `informational_cycle_count == 1` AND `cycle_count == 1`; (b) same graph, one member `Reject` ⇒ `informational_cycle_count == 0`, `cycle_count == 1`; (c) `generate_report()` (no policy fn) on a cycle graph ⇒ `informational_cycle_count == 0` and, critically, **serde output has no `informational_cycle_count` key** (assert the serialized string does not contain the key).
  - serde byte-lock: serialize a default `IncludeReport` (zero informational) and assert the JSON has no `informational_cycle_count` key (proves C++ artifact identity at the unit level; m00 proves it end-to-end).
- **Green:** add the field + the policy method + the delegating `generate_report`; refactor the existing `generate_report` body into the policy variant so there is ONE cycle-counting path (no duplication). Pipeline call site per IV-5.
- **Refactor / Hygiene:** classification loop reads as `cycles.iter().filter(|cycle| all tolerate).count()`; helper `fn is_informational(cycle: &[FileId], policy_of: &dyn Fn(FileId) -> CyclePolicy) -> bool` named honestly; `python tools/hygiene_check.py` exit 0.

## 4. Touch-set

`crates/lang-api/src/types.rs` (`IncludeReport` field only) · `crates/graph/src/graph.rs` (report method split + classification) · `crates/pipeline/src/lib.rs` (call-site switch) · `crates/cluster/src/planner.rs` (**doc-comment only**) · **no** builder/resolver/fixture/test-fixture edits (T-D owns the m20 suite).

## 5. Scoped commands

```powershell
cargo test -p ariadne-graph
cargo test -p ariadne-pipeline
cargo test -p ariadne-integration-tests --test m3_include_graph_test
cargo test -p ariadne-integration-tests --test m00_golden_baseline_test
cargo test -p ariadne-integration-tests --test m15_graph_ui_test
cargo test -p ariadne-cluster
python tools/hygiene_check.py
```

## 6. Clauses

- **m00 golden drift = STOP.** The skip-when-zero field is what keeps `report.json` byte-identical; any drift means the field is not truly skipped or `cycle_count` semantics changed — halt and report.
- `m15_graph_ui_test` must stay green (UI consumes `detect_cycles()` raw, which is untouched).
- Do NOT introduce `detect_cycle_groups` or change `detect_cycles`. Targeted diffs; commit on green (`feat(m20.2): CyclePolicy honored in cycle report`).
