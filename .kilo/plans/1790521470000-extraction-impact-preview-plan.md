# Pass E — Extraction Impact Preview: Intent Specification & TDD Plan

Status: DRAFT for delegation. Repo @ 81de36d (post include-justification pass, FINAL ACCEPTED).
Canonical gates: 430 workspace tests, clippy warm-replay 125 / fresh 126, `hygiene_check.py` exit 0.

## 0. User story being closed

After reading a justification row — "graph.h includes jobserver.h for X, Y, Z · needs: a.h, b.h" — the user must be able to answer:

> **What is this extraction actually worth, and what does the world look like after it?**

Concretely: (a) which headers `graph.h` would stop transitively seeing, (b) which of those would *still* arrive via other include paths (so are not real savings), (c) the resulting dependency tree with kept vs eliminated nodes marked, (d) closure size before → after and symbols no longer parsed. The raw collateral symbol count stays as a heat proxy but is demoted from "the value metric".

## 1. Intent statement

Add an extraction-impact computation over include closures (per consumer edge: root F × source S × justified symbol set), surface it as a bounded payload summary + unbounded route field, and render an after-state overlay on the heat map with a preview toggle. The math is pure set algebra on `IncludeGraph` closures plus `required_headers_for_symbols` (A2) — no new analysis, no planner changes.

The *why*: collateral counts measure waste; eliminated-include sets measure the prize. Only the prize ranks extractions honestly — one huge vendored header beats 40 symbols from a cheap one.

## 2. Invariants

- **E-I1 — Two drop sets are distinct and both reported.** `dropped_directly = closure(S) \ closure({S} ∪ R)` (what the slim edge sheds) vs `dropped_from_root = closure(F) \ closure_after(F)` (what F actually stops seeing). `still_reachable = dropped_directly \ dropped_from_root` — headers arriving via sibling paths. `dropped_directly = dropped_from_root ⊎ still_reachable`. Only `dropped_from_root` is labeled "value".
- **E-I2 — closure_after is modeled, not speculative.** F's edges with F→S replaced by F→S_slim where S_slim includes exactly R (the required set) and contains S's own content; everything else in F's graph unchanged. `S ∈ closure_after` (the slim header still arrives). Determinism: all header lists sorted `(path asc)`, sizes are `usize`.
- **E-I3 — Self-edge guard.** Impact is defined only for `root ≠ source` and `justified_symbols` non-empty; otherwise the API returns `None`/absent and the UI shows no preview control.
- **E-I4 — Payload bound.** Payload impact entries are SUMMARIES only: counts + ≤ 8 header names per list (dropped sample), present only on justification entries where `justified_count > 0 && collateral_count > 0`, max 8 such entries per root file. Route response carries the FULL unbounded lists. `truncated` flags on each sample list. (Client-side recomputation is NOT used for correctness — capped samples mark a sample; live mode marks everything from the route.)
- **E-I5 — One build function.** Payload path stays inside `build_ui_payload`; route stays `LoadAnalyzer`-state based. Live hydrates from `/api/justification` (impact rides the existing response); standalone renders from capped summaries.
- **E-I6 — graph.js line budget.** `graph.js` is 947/1000; new preview logic ships as a NEW asset `crates/ui/assets/preview.js` (registered in `lib.rs` `include_str!` + `export.rs` inline list + inline-assets test, following the split-asset precedent). graph.js gains at most 8 lines (hook + reset).
- **E-I7 — Hygiene/gates.** fns ≤ 100 lines, files ≤ 1000, `hygiene_check.py` exit 0, clippy introduces ZERO new warnings in touched files, all 430 existing tests byte-green (new tests only add).
- **E-I8 — Cluster/transform untouched.** Proposal-card planner numbers unchanged; impact math lives in `crates/graph`; per-consumer framing never mutates or reads plan files.
- **E-I9 — Needs-sets are includable headers only.** `required_headers_for_symbols` (A2) and everything derived from it (`needs:` display rows, payload/route `required_headers`, `ExtractionImpact.required_headers`, and therefore `closure_after`) filter defining files to the header predicate `is_includable_header` (extension `.h` / `.hpp` / `.hxx`, matching dto.rs:122's kind detection, owned as a pub fn in `crates/graph`). A closure symbol defined in `.cc`/`.cpp` contributes NO include requirement — it is satisfied through its declaration elsewhere or by linking, not by inclusion. Justified symbol lists are NOT filtered (a symbol defined in a .cpp is still a legitimate "for X" entry).

## 3. Contract & API signatures

### 3.1 `crates/graph` (E1)

```rust
// load.rs — new public types + method on LoadAnalyzer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImpactHeader { pub file_id: FileId, pub path: PathBuf, pub symbol_count: usize }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionImpact {
    pub root_file_id: FileId,
    pub source_file_id: FileId,
    pub required_headers: Vec<ImpactHeader>,      // R, excluding S itself (A2 rule); headers-only per E-I9
    pub dropped_directly: Vec<ImpactHeader>,      // closure(S) \ closure({S} ∪ R)
    pub dropped_from_root: Vec<ImpactHeader>,     // closure(F) \ closure_after(F)  ← "the prize"
    pub still_reachable: Vec<ImpactHeader>,       // dropped_directly \ dropped_from_root
    pub root_closure_before: usize,
    pub root_closure_after: usize,
    pub symbols_no_longer_parsed: usize,          // Σ symbol_count over dropped_from_root
}

pub fn extraction_impact(
    &self,
    root: FileId,
    source: FileId,
    justified_symbols: &[SymbolId],
) -> Option<ExtractionImpact>;
// None per E-I3 (root == source, empty justified, unknown file ids).
// required_headers reuses required_headers_for_symbols(justified).
// closure_after: BFS from root where `source` expands to {source} ∪ R instead of
// its real includes (override expansion; no graph cloning).
// symbol_count = self.model.file_symbols[file].len() (0 when absent).
```

### 3.2 `crates/ui` (E2)

```rust
// dto.rs — rides existing structures, no new top-level payload field:
pub struct ImpactSummary {
    pub dropped_from_root_count: usize, pub still_reachable_count: usize,
    pub dropped_sample: Vec<String>,        // basenames, ≤ 8, sorted, truncated flag below
    pub dropped_sample_truncated: bool,
    pub root_closure_before: usize, pub root_closure_after: usize,
    pub symbols_no_longer_parsed: usize,
}
pub struct JustificationEntry { /* existing fields */ pub impact: Option<ImpactSummary> }
// FileJustificationResponse entry: full ExtractionImpact embedded as
//   SourceJustification.impact: Option<ExtractionImpact>   (serde, unbounded per E-I4)
```

Builder: inside `build_file_justification` (and the route detail builder) call `extraction_impact` for eligible entries (≤ 8 per root per E-I4 for the payload route path; route is unbounded), map to `ImpactSummary` via a shared conversion helper. `IncludeGraph::compute_closures` is computed ONCE per build/route-request and threaded through (no per-entry recomputation) — signature growth allowed.

### 3.3 `crates/ui` assets (E3)

- `preview.js` (new): `window.ariadnePreview = { applyPreview(rootId, sourceId), clearPreview(), hasImpact(rootId, sourceId) }` + data source `justificationCache` entries (live, full `ExtractionImpact`) or capped payload summaries (standalone — marks sample + banner count). Node marking classes: `preview-kept` (solid green, heat off), `preview-eliminated` (red dashed stroke, struck label), `preview-still-reachable` (amber ring). Legend: `After extraction: closure 128 → 94 · 34 headers dropped · 5 still reach via other paths`.
- Trigger controls: a `Preview after extraction` button on each expanded justification row that has `impact` (api.js renderer) and on proposal cards (proposal card preview runs against the CURRENTLY selected consumer root, not the planner's global target — label says so). Heat map selection flow calls `ariadnePreview.clearPreview()` on any new selection or Esc (the ≤ 8 graph.js lines; verify exact hook placement, keep renderSvg untouched otherwise).
- Proposal card gets: `drops N headers from selected file's closure · closure B → A` line when impact for the selected root exists, else unchanged card.

## 4. TDD task breakdown

Red before green, one red commit + one green commit per task; Reviewer at Pass E close + final gate. All cargo with `$env:CARGO_TARGET_DIR="C:\Users\Kurt\AppData\Local\Temp\kilo\target-fresh"`.

### Phase 0 — E0: needs-sets are headers-only (E-I9, behavior change to A2, lands FIRST so E1 inherits correct R)
- **Green surface of prior work:** `required_headers_for_symbols` today groups by defining file regardless of extension, so a justified symbol whose definition lives in `worker.cpp` pollutes `needs:` with a non-includable file, and (post-E1) would model `closure_after` through a `.cpp` — wrong.
- **Red:** extend `crates/graph/src/load_justification_tests.rs` — new fixture where the semantic closure reaches BOTH `b.h` and `worker.cpp` → assert result contains ONLY `b.h` (worker.cpp absent); pure-source closure (only .cpp defining files reached) → empty vec; extensions `.h/.hpp/.hxx` all included, `.cc/.cpp/.c/.ipp/.inl` excluded; seed-file exclusion still applies to headers. Public predicate `is_includable_header(&Path) -> bool` gets its own table-driven test. Expected failure: worker.cpp currently present.
- **Green:** add `pub fn is_includable_header(path: &Path) -> bool` in graph crate (`.h`, `.hpp`, `.hxx` — same set as dto.rs:122's kind check; `.ipp/.inl` stay OUT with a code comment that template-impl files are a future question, not today); filter inside `required_headers_for_symbols` grouping. **Intentional semantic change:** update any B1/B2 locked expectations that asserted a source file inside `required_headers` (grep test fixtures for `.cpp`/`.cc` strings in required_headers asserts) — update those expectations IN THE RED COMMIT with a comment marking them as E-I9 amendments, so the red commit shows the deliberate behavior flip, not a silent test edit.
- **Surfacing:** `needs:` rows in api.js renderer and `ImpactSummary`/route payloads inherit the fix automatically (single source). Add ONE cheap static_html marker only if the row builder currently post-filters (it must NOT — graph layer owns it).
- **Gate note for reviewer:** this changes `/api/justification.required_headers` and payload `required_headers` for source-defining symbols — old payloads/JSON round-trip fine (no schema change), so no serde/version churn.


### Phase 1 — E1: graph crate impact math (foundation)
- **Red** `crates/graph/src/load_impact_tests.rs` (new sibling module): fixture with root `main.cpp → fat.h`, `fat.h → {slim_base.h, c.h → e.h, d.h}`, justified symbols defined in fat.h whose closure needs only `slim_base.h`. Assert: `required_headers == [slim_base.h]`; `dropped_directly == [c.h, d.h, e.h]` sorted path-asc; `dropped_from_root` equals the same when no sibling path exists; `still_reachable == []`; `symbols_no_longer_parsed == Σ counts`; E-I1 partition sum. **Trap tests (the point of the pass):** (1) add sibling edge `main.cpp → d.h` → `dropped_directly` still has d.h and e.h? (d.h directly dropped, but d.h remains in closure_after; e.h arrives via d.h) → assert `still_reachable == [d.h, e.h]`, `dropped_from_root == []` — closure_after unchanged for those; (2) E-I3: root==source → None; empty justified → None; unknown ids → None; (3) `dropped_from_root ⊆ dropped_directly` in a mixed fixture; (4) determinism: twice → equal; (5) `root_closure_after ≥ 1` (S still arrives, E-I2). Expected failure: E0599 no method.
- **Green:** types + `extraction_impact` with an override-expansion BFS helper (private, ≤ 50 lines) reusing the existing `closures` RefCell via `compute_closures` or `get_transitive_includes` where possible; partition asserts internal `debug_assert!`. Keep `load.rs ≤ 1000 lines` (553 + ~120 est — fine; report actual).
- **Refactor:** share `ImpactHeader` construction; no behavior delta.

### Phase 2 — E2: data to surfaces (E2a payload+route, single task)
- **Red** (ui tests): route test — `/api/justification` entries carry full `impact` (13-name elimination list NOT capped in route, `dropped_sample` absent in route shape; key-lock test updated); payload builder tests — impact present ONLY on eligible entries (justified>0 ∧ collateral>0), ≤ 8 entries per root carry impact (17 eligible → 8 have Some, 9 None; selection order = existing rank order), sample ≤ 8 + truncated true at 9, `root ≠ source` entries only (self row None, locks E-I3 in surface layer), serde key names locked, absent-from-JSON test unaffected (entries without impact serialize `"impact": null` — decide explicitly: `skip_serializing_if Option::is_none` on the field; lock whichever chosen).
- **Green:** shared `impact_summary_from(impact: &ExtractionImpact) -> ImpactSummary` in dto.rs; closures computed once per builder call and threaded; route embeds full `ExtractionImpact`; all 430 byte-green.
- **Refactor:** closure-threading may widen signatures — keep each fn ≤ 100 lines; if `build_file_justification` grows past 90 lines, extract per-entry assembly fn.

### Phase 3 — E3: preview overlay (single asset task, no graph.js growth beyond ≤ 8 lines)
- **Red** `static_html.rs`: preview.js registered (page_content contains `ariadnePreview`, `preview-eliminated`, `preview-kept`, `preview-still-reachable`, legend marker `After extraction:`); api.js contains the per-row `Preview after extraction` control marker + proposal-card impact line marker; graph.js contains the clear-on-select hook and `git show --stat` style assertion that graph.js edits are minimal is impossible in-test — instead assert graph.js ≤ 960 lines (hard numeric cap, forces future discipline); export embed test: standalone HTML with capped-impact payload contains `ariadnePreview` + sample banner marker.
- **Green:** new `preview.js` wired in `lib.rs`, `export.rs` inline list; api.js row/card controls feeding it (live: full lists from justificationCache/impact; standalone: sample + `+N more` banner, honest per E-I4); CSS in styles.css (precedent); clearPreview wired into selection/Esc paths.
- **Refactor:** node-class application reuses C1's `classifyDependency` results — preview classes compose with (not replace) justified/heat classes as documented in code comments.

### Phase 4 — Reviewer gate (E1–E3 range)
Reviewer measures: E-I1 partition + subset relations from code, eligibility/selection order in payload, route/payload shape consistency (live vs standalone field access — the C2 shape-match lesson), one-closure-computation claim (no per-entry BFS storm: count `compute_closures` call sites), zero clippy attribution to touched files, 430+n tests ×3, hygiene exit 0, graph.js line count, and adversarially: a fixture-derived repo where a third-party header (system include) appears in dropped lists — system headers ARE in IncludeGraph edges only when local; verify behavior for edges to unregistered FileIds (file_paths miss → path "unknown" precedent) doesn't panic.

## 5. Delegation map

| Order | Task | Depends on | Notes |
|---|---|---|---|
| 1 | E0 | 81de36d (430-test baseline recorded in plan) | A2 header-only filter; graph crate only |
| 2 | E1 | E0 | impact math inherits header-only R |
| 3 | E2 | E1 | dto+server+cli, one tree |
| 4 | E3 | E2 | assets only; graph.js ≤ 8-line hook |
| 5 | Reviewer | all | one gate (range small enough); findings → remediation task |

## 6. Explicit non-goals

- No global/multi-consumer impact aggregation (per-consumer only; planner card keeps its global numbers).
- No speculative slim-header file generation, no write-to-disk, no `tool optimize` changes (I8).
- No hotspot-metric rework (a future "extractable savings" metric option can read the same `ImpactSummary`).
- No client-side full-recomputation of impact in standalone (capped sample is the contract).
