# Collateral Import Traversal View - Intent

## Purpose
Create a new analysis view that shows unnecessary package/file traversal — the collateral imports pulled in when a file imports a dependency for a specific symbol, versus what would be needed if that symbol were isolated.

## Problem Statement
Current views show:
- **Consumer view** (hotspots): what collateral symbols a file imports from its dependencies
- **Justification view**: per-edge symbol breakdown (justified vs collateral)

What's missing: a view that answers "If file A imports file B for symbol X, and B imports C,D,E,F — which of those are actually needed for X, and which are collateral traversal that could be eliminated?"

## User Scenario
- File A imports File B because it needs symbol X
- Symbol X is defined in File C (so C is the necessary import)
- But File B also imports files C, D, E, F
- D, E, F are **collateral imports** — they're transitively pulled in but not needed for symbol X
- Isolating symbol X into its own header would chop off the D/E/F branch of the import tree for ANY file importing X

## Acceptance Criteria
1. **New tab/view**: "Traversal" tab alongside Include/Symbol/Call tabs
2. **Entry point**: Select a root file → shows its direct dependencies
3. **Per-dependency drill**: Click a dependency → shows:
   - **Necessary subtree**: files actually required for the symbols the root uses from this dependency
   - **Actual subtree**: all files the dependency transitively imports
   - **Collateral delta**: the difference (files in actual but not necessary)
4. **Symbol selector**: When multiple symbols are used from a dependency, allow selecting which symbol(s) to analyze
5. **Visual distinction**: Clear visual encoding for necessary vs collateral vs shared files
6. **Impact summary**: Quantify the collateral load (symbol count, file count) that could be eliminated

## Technical Approach
- Leverage existing `LoadAnalyzer::extraction_impact` which computes:
  - `required_headers`: what's actually needed for the justified symbols
  - `dropped_from_root`: what would be eliminated from root's closure
  - `still_reachable`: what arrives via other paths anyway (decoy headers)
- New API endpoint: `/api/graph/traversal` taking `root_file_id` and `dep_file_id` and optional `symbol_ids`
- Frontend: new tab with dependency list + detail panel showing the three subtrees

## Success Criteria
- Build engineers can identify "fat headers" that pull in unnecessary transitive dependencies
- Quantifies the build-speed impact of header isolation
- Integrates with existing extraction proposals (shows what extraction would actually achieve)