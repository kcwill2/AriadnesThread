# Implementation Plan: Collateral Import Traversal View

## Overview
Build a new "Traversal" view that shows unnecessary package/file traversal — the collateral imports pulled in when a file imports a dependency for specific symbols, versus what would be needed if those symbols were isolated. This leverages the existing `extraction_impact` infrastructure in the graph crate.

## Architecture Decisions

1. **New tab in existing UI**: Add "Traversal" tab alongside Include/Symbol/Call tabs (follows existing `switchTab` pattern)
2. **New API endpoint**: `/api/graph/traversal` — takes `root_file_id`, `dep_file_id`, optional `symbol_ids[]`
3. **Reuse extraction_impact**: The backend already computes `required_headers`, `dropped_from_root`, `still_reachable` — we just need to expose it per dependency edge
4. **Frontend detail panel**: Three-column or tabbed view showing Necessary / Actual / Collateral Delta subtrees
5. **Symbol selection**: When multiple symbols are used from a dependency, allow multi-select to analyze specific symbols

## Task List

### Phase 1: Backend API
- [ ] Task 1: Add `TraversalQuery` and `TraversalResponse` DTOs in `crates/ui/src/dto.rs`
- [ ] Task 2: Implement `/api/graph/traversal` endpoint in `crates/ui/src/server.rs`
- [ ] Task 3: Add unit tests for the new endpoint in `crates/ui/tests/`

### Phase 2: Frontend - Tab & Navigation
- [ ] Task 4: Add "Traversal" tab to HTML (index.html embedded in ui crate)
- [ ] Task 5: Add tab handling in `crates/ui/assets/graph.js` `switchTab` function
- [ ] Task 6: Add traversal legend configuration in `crates/ui/assets/main.js`

### Phase 3: Frontend - Data Fetching & Rendering
- [ ] Task 7: Add traversal data fetching in `crates/ui/assets/api.js`
- [ ] Task 8: Create traversal renderer in `crates/ui/assets/graph.js` (new `renderTraversalView` function)
- [ ] Task 9: Build detail panel with three subtrees (Necessary / Actual / Collateral Delta)
- [ ] Task 10: Add symbol selector UI for multi-symbol dependencies

### Phase 4: Integration & Polish
- [ ] Task 11: Wire up hotspots/consumer clicks to open Traversal tab with context
- [ ] Task 12: Add traversal to extraction proposal cards (show what extraction achieves)
- [ ] Task 13: Style the traversal view (CSS in `crates/ui/assets/styles.css`)

### Phase 5: Verification
- [ ] Task 14: Integration test - full flow from hotspot click to traversal view
- [ ] Task 15: Run `cargo test` and `python tools/ui_smoke.py` 

## Checkpoints

### Checkpoint: After Phase 1 (Backend)
- [ ] `cargo test` passes
- [ ] New endpoint returns correct JSON for test fixture

### Checkpoint: After Phase 2 (Tab & Navigation)
- [ ] Tab appears in UI
- [ ] Tab switching works without errors

### Checkpoint: After Phase 3 (Rendering)
- [ ] Traversal view renders for a test case
- [ ] Three subtrees display correctly

### Checkpoint: Complete
- [ ] All acceptance criteria met
- [ ] `cargo test` passes
- [ ] `python tools/ui_smoke.py` passes
- [ ] Ready for review

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `extraction_impact` requires justified symbols per edge — may need to fetch from justification data | Medium | Use existing `/api/justification` or payload's `file_justification` to get per-edge symbols |
| Large subtrees may exceed node cap | Medium | Apply existing `nodeCap` filtering; show summary counts for overflow |
| Symbol selection UX complexity | Medium | Start with "all symbols from this edge" as default, add multi-select as enhancement |

## Open Questions
1. Should the traversal view be a separate tab or a mode within the Include tab? (Current plan: separate tab for clear mental model)
2. How to handle the case where root uses multiple symbols from the same dependency? (Plan: default to all, allow narrowing)
3. Should we show the "still_reachable" decoy headers as a separate category? (Yes — they're the "false collateral" that arrives anyway)