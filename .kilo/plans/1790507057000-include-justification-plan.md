# Include Justification & Extraction Proposals — Intent Specification & TDD Plan

Status: DRAFT for delegation. Repo state at time of writing: main @ 6e1a521+ (post Pass 5, hygiene gate live).
Target user story (verbatim intent):

> From a hotspot or heat-map selection of `graph.h`, the user sees:
> 1. `graph.h` includes `jobserver.h` **for symbols X, Y, Z** (justification);
> 2. X, Y, Z **require** transitive includes `a.h` and `b.h` (required set);
> 3. `jobserver.h` also drags **collateral** `c.h, d.h, e.h` (unused payload);
> 4. and is told: *extract X, Y, Z into a slim header including only a.h, b.h to remove N collateral symbols across M TUs* (proposal, shown in **both** live UI and standalone report.html).

## 1. Intent Statement

Today `file_collateral` ships only `rootFileId → {depFileId: unused_count}` and the heat map renders `+XX col` badges. The semantic ("used") side of the include-closure intersection is computed inside `LoadAnalyzer::compute_load` (`crates/graph/src/load.rs:202–280`) but collapsed into a flat `semantic_symbols` list and then discarded from the UI payload. The user cannot attribute an include edge to the symbols that justify it, cannot see which headers those symbols require, and cannot see the extraction proposals that `OptimizationPlanner` (`crates/cluster/src/planner.rs`) already produces from `tool optimize`. This change makes the used-vs-collateral split a first-class, symbol-attributed, per-source-header output and surfaces it in the heat map, inspector, hotspots, and standalone report.

The *why*: the heat map currently proves "this file is heavy" but not "why this edge exists and what to cut." The justification split is the difference between a warning and an action.

## 2. Invariants

- **I1 — Justification is an intersection, never a re-parse.** For root file F: `semantic_sources(F)` groups `LoadMetric.semantic_symbols` by defining file; `collateral_sources(F)` is unchanged. A symbol appears in exactly one side. `Σ semantic_sources[].symbol_count + Σ collateral_sources[].symbol_count == include_load`.
- **I2 — Artifact back-compat.** `LoadMetric` gains `#[serde(default)]` fields only. Old on-disk load artifacts must still deserialize (`artifacts.load_load_artifacts()` green with pre-change fixtures).
- **I3 — Payload stays bounded.** Standalone payload must not grow super-linearly. Caps: top 16 contributor entries per root file, top 12 symbol names per entry per side, with `justified_truncated` / `collateral_truncated` booleans. Full detail is on-demand via `/api/justification` (live mode only).
- **I4 — Empty-map tolerance.** New payload fields use `skip_serializing_if` empty; UI code defaults `|| {}` exactly as `file_collateral` does today (api.js:55,109; server_api.rs:267 locks "clients must default it").
- **I5 — One data path, two surfaces.** `report.html` is the same payload + same inlined assets via `export_standalone_html` (`crates/ui/src/export.rs`). No report-only code paths.
- **I6 — Determinism.** Contributor ordering by `(symbol_count desc, path asc)` — the existing `collateral_sources` sort contract (load.rs:251–255). Symbol-name lists sorted by SymbolId string before capping.
- **I7 — Hygiene gate.** Every new/edited function ≤ 100 physical lines, files ≤ 1000 lines, `tools/hygiene_check.py` R1/R2 green; clippy warning count must not exceed the canonical 126 fresh-target baseline (see memory `hygiene.clippy_count_method`); workspace tests currently 380/380 — all existing tests stay green, new tests only add.
- **I8 — Planner proposals in payload are read-only display.** No `tool optimize` behavior changes; `RiskLevel::Conservative` filter from `plan_safe_extractions` is preserved for the payload (Moderate/Aggressive not advertised in UI in this change).

## 3. Contract & API Signatures

### 3.1 `crates/graph` (Pass A)

```rust
// load.rs — new shared shape (keep CollateralSource untouched for compatibility;
// introduce the general name and alias):
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContributingSource {           // same fields as CollateralSource
    pub file_id: FileId,
    pub path: PathBuf,
    pub symbol_count: usize,
    pub symbols: Vec<SymbolId>,
}
pub type CollateralSource = ContributingSource; // serde/type alias keeps old paths compiling

// LoadMetric gains (serde(default) per I2):
pub struct LoadMetric {
    // ... existing fields ...
    #[serde(default)]
    pub semantic_sources: Vec<ContributingSource>,   // grouped like collateral_sources
}

// LoadAnalyzer — new public API (used by Pass B route + planner later):
pub fn required_headers_for_symbols(&self, symbol_ids: &[SymbolId]) -> Vec<(FileId, PathBuf, usize)>;
// Returns defining files (with contributing symbol counts) of the semantic closure of
// `symbol_ids`, EXCLUDING the seed symbols' own defining files. Sorted per I6.
```

`compute_load` populates `semantic_sources` by grouping `semantic_sym_vec` through `model.symbols[..].defining_file` — symmetric to the existing `col_by_file` loop.

### 3.2 `crates/ui` payload + route (Pass B)

```rust
// dto.rs — GraphExportPayload gains:
#[serde(default, skip_serializing_if = "HashMap::is_empty")]
pub file_justification: HashMap<String, HashMap<String, JustificationEntry>>,
// key1 = root FileId string, key2 = source FileId string (same keying as file_collateral)

#[derive(Serialize, Deserialize, Clone)]
pub struct JustificationEntry {           // serde: camelCase not used in repo; snake_case kept
    pub justified_symbols: Vec<String>,   // ≤ 12 names, truncated per I3
    pub collateral_symbols: Vec<String>,  // ≤ 12 names
    pub justified_count: usize,
    pub collateral_count: usize,          // == file_collateral value; kept so heat badges unaffected
    pub justified_truncated: bool,
    pub collateral_truncated: bool,
    pub required_headers: Vec<String>,    // basenames, the 3.1 required_headers_for_symbols set for the justified symbols
}
```

```rust
// server.rs — live-mode detail route:
GET /api/justification?file=<fileId>  -> FileJustificationResponse  { entries: Vec<..unbounded..> }
```

Payload builder lives in `crates/cli/src/commands/ui.rs` next to `build_file_collateral` (`build_file_justification`), symbol names resolved via `reduced_model.symbols[].name`.

### 3.3 Proposals in payload (Pass D1)

```rust
// dto.rs — GraphExportPayload gains:
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub extraction_proposals: Vec<ExtractionProposalDto>,   // capped top 12 by (before-after) desc

pub struct ExtractionProposalDto {
    pub plan_id: String,
    pub target_file: String,               // basename
    pub problem: String,
    pub collateral_load_before: usize,
    pub collateral_load_after: usize,
    pub symbol_names: Vec<String>,          // proposed_extractions, ≤ 12, truncated flag implied by len < total
    pub total_symbols: usize,
    pub required_headers: Vec<String>,      // optimizer_frontier basenames
    pub consumer_tu_count: usize,           // validation_required_tus.len()
    pub risk: String,                        // "Conservative"
}
```

Built in `commands/ui.rs` using `OptimizationPlanner::new(&include_graph, &reduced_model, &pipeline.discovery.files, &tu_ids)` (fields already public: `pipeline.discovery`, `pipeline.build`, `Pipeline` lib.rs:119–123), filtering `RiskLevel::Conservative` exactly like `plan_safe_extractions` (optimize.rs:254–271). No change to `tool optimize` output.

## 4. TDD Task Breakdown

Every task: Red first (failing test committed and run before implementation), Green minimal, Refactor under I7. Every pass closes with a Reviewer handoff (diff + test run + hygiene + clippy before/after).

### Phase 0 — Baseline (1 Implementer, gated)
- **T0:** `cargo test --workspace` (expect 380/380), fresh-target `cargo clippy --workspace --all-targets` (recount method from `hygiene.clippy_count_method`), `python tools/hygiene_check.py` — record all three in the handoff. No code moves.

### Phase 1 — Pass A: graph-crate justification data (A1, A2 parallel-safe: distinct functions)
- **A1 — `semantic_sources` in `LoadMetric`**
  - Red: new `crates/graph/src/load_justification_tests.rs` (sibling-module pattern of `load_characterization_tests.rs`): fixture where Thing.h semantically needs Helper symbols and ignores HugeThirdParty symbols → assert `semantic_sources` grouped by defining file, counts, I1 partition sum, deterministic order, and old-artifact JSON deserializes with `semantic_sources` absent (I2). Expected failure: field absent / empty.
  - Green: group loop in `compute_load`; type alias + serde default.
  - Refactor: share one grouping helper between the collateral and semantic loops (kills the duplicated `col_by_file` code); keep `format_summary` output byte-identical (characterization locks).
- **A2 — `required_headers_for_symbols`**
  - Red: tests in the same new file: two-symbol seed whose closure reaches a.h and b.h, seed's own file excluded, empty input → empty, unknown symbols → empty, I6 order. Expected failure: method missing.
  - Green: reuse `get_semantic_closure_of_symbols` + `defining_file` map; function ≤ 40 lines by delegating counting.
  - Refactor: no duplication of adjacency building — reuse the `symbol_adjacency` RefCell cache.

### Phase 2 — Pass B: payload & route (B1 → B2)
- **B1 — `file_justification` payload**
  - Red: `crates/ui/tests/ui_test/dto_builders.rs` + new `justification_builders.rs`: builder maps fixture LoadMetrics + reduced model → entries with capped names (13th name absent, truncation flag true), counts equal `file_collateral` entries (I3/I4), empty map skipped in serialization (mirrors server_api.rs:267). CLI-side: `build_file_justification` unit test in `commands/ui.rs`'s test module or `tests/` sibling.
  - Green: DTO types + builder in cli/ui.rs; names from `reduced_model.symbols`.
  - Refactor: `build_file_collateral` and `build_file_justification` share path→id map + contributor cap helper.
- **B2 — `/api/justification` route**
  - Red: `server_api.rs` test: payload with a root → route returns unbounded entries for that file incl. `required_headers` via A2; malformed `file` param → empty response, not error (matches `/api/collateral` tolerance server.rs:587).
  - Green: route reusing `LoadAnalyzer` state; wire in router next to `api_graph_collateral`.
  - Refactor: entry-conversion helper shared with B1 builder.

### Phase 3 — Pass C: UI tri-state heat map + inspector (C1, C2 sequential; may run parallel with D1)
- **C1 — heat map justification coloring**
  - Red: `static_html.rs` asset assertions (existing pattern, e.g. static_html.rs:191–197): graph.js must consult `file_justification`, mark `justified` dependencies with a distinct tier (needed, never faded), keep `+N col` collateral badges unchanged. Expected failure: string absent.
  - Green: `crates/ui/assets/graph.js` selection block (321–374): dep classes become justified / collateral-only / mixed; legend (main.js `updateHeatLegend`) gains "Justified (needed)" row.
  - Refactor: class computation extracted to a named predicate helper (`classifyDependency(...)` — boolean/predicate naming per hygiene rule).
- **C2 — inspector edge breakdown**
  - Red: `static_html.rs`: inspector must render justified symbol names, collateral symbol names, and `required_headers` for the selected node's top contributor; index.html help text updated (index.html:392 region).
  - Green: `graph.js` `selectNode` block (394–460): replace the generic "consider header extraction" platitude (422–425) with the data-driven breakdown; live mode hydrates from `/api/justification`, standalone uses capped payload names (I5).
  - Refactor: DOM list rendering reuses existing `link-item` builder; no inline onclick string bloat.

### Phase 4 — Pass D: extraction proposals in both surfaces (D1 ∥ C-tasks; D2 after D1)
- **D1 — proposals into payload**
  - Red: ui-test: non-empty `extraction_proposals` serialized, capped to 12, sorted by collateral delta, `skip_serializing_if` empty round-trip (server_api.rs pattern); planner-fixture test that Conservative-only plans are included (I8).
  - Green: `build_extraction_proposals` in commands/ui.rs from public pipeline fields; payload field wired for both `ui` and `ui --export` paths (same UiBuildResult — verify both).
  - Refactor: cap/sort helpers shared with B1 helpers; planner call isolated so the ui.rs build function does not cross 100 lines.
- **D2 — proposal cards in hotspots + inspector; standalone proof**
  - Red: `static_html.rs`: assets reference `extraction_proposals` render path; **and** an export test (`export_standalone_html`) asserting the generated HTML embeds non-empty `extraction_proposals` data (locks "both surfaces", I5).
  - Green: `hotspots.js` panel gains "💡 Proposed extractions" section (name list, before→after collateral, required headers, TU count); inspector shows matching card when the selected file is a proposal target.
  - Refactor: card renderer < 60 lines; hotspot cache invalidation follows the `hotspotBurdenSource` identity-cache convention (hotspots.js:181–184).

### Phase 5 — Reviewer gate (after each pass, and one final)
Reviewer receives: diff range, 3× `cargo test --workspace` outputs, clippy before/after (must be ≤ 126), `hygiene_check.py` output, and re-measures the I1 partition sum and I3 caps against actual code (Pass 5 precedent: measure from git blobs, not the ledger).

## 5. Delegation Map

| Order | Task | Depends on | Parallel with |
|---|---|---|---|
| 1 | T0 | — | — |
| 2 | A1, A2 | T0 | each other |
| 3 | B1 | A1 | — |
| 4 | B2, D1 | B1 (+A2 for B2) | each other |
| 5 | C1 | B1 | D2 |
| 6 | C2 | C1 | D2 |
| 7 | D2 | D1, C2 | — |
| 8 | Final review | all | — |

## 6. Explicit Non-Goals

- No `tool optimize` / planner algorithm changes; no extraction execution from the UI.
- No Moderate/Aggressive plan exposure.
- No per-symbol node in the include graph view (symbol tab stays as-is; the bridge is `required_headers` text + route data).
- No changes to cycle detection, closure math, or heat-tier thresholds.
