use crate::common;
use axum::http::StatusCode;

#[test]
fn test_index_html_content() {
    let html = common::page_content();
    assert!(html.contains("/*__ARIADNE_DATA__*/"));
    assert!(html.contains("Include Graph"));
    assert!(html.contains("Symbol Dependency Graph"));
    assert!(html.contains("Call Graph"));

    // M16 Unit 3: Interactive Frontend Traversal elements
    assert!(html.contains("id=\"ariadneThread\"") || html.contains("class=\"thread-bar\""));
    assert!(html.contains("id=\"entrypointsModal\"") || html.contains("id=\"entrypointsPanel\""));
    assert!(html.contains("id=\"pathfinderModal\""));
    assert!(html.contains("clientTraverseNeighborhood"));
    assert!(html.contains("clientShortestPath"));
    assert!(html.contains("clientGetEntrypoints"));
}

#[test]
fn test_ui_layout_and_interactive_traversal() {
    let html = common::page_content();

    // 1. Initial active button is hopAllCap (formerly hopAll)
    assert!(
        html.contains("id=\"hopAllCap\" class=\"active\"")
            || html.contains("class=\"active\" id=\"hopAllCap\"")
            || html.contains("id=\"hopAllCap\" class=\"active\"")
    );
    assert!(!html.contains("id=\"degMinus\" class=\"active\""));

    // 2. Layout wrapping logic
    assert!(html.contains("maxRowsPerCol"));
    assert!(html.contains("numSubCols"));

    // 3. findPath and showEntrypoints have non-empty implementations
    assert!(html.contains("fetch(`/api/graph/path"));
    assert!(html.contains("fetch(`/api/graph/entrypoints"));
}

#[test]
fn test_selected_node_edge_highlighting() {
    let html = common::page_content();

    // Check edge detection relative to selectedNodeId
    assert!(html.contains("e.target === selectedNodeId"));
    assert!(html.contains("e.source === selectedNodeId"));

    // Check highlighted classes
    assert!(html.contains("highlighted"));
    assert!(html.contains("entering") || html.contains("edge-entering"));
    assert!(html.contains("leaving") || html.contains("edge-leaving"));

    // Check marker assignment
    assert!(html.contains("url(#arrow-selected)"));
    assert!(html.contains("url(#arrow-entering)"));
    assert!(html.contains("url(#arrow-leaving)"));

    // Check CSS rules
    assert!(html.contains(".edge-path.highlighted"));
}

#[test]
fn test_ui_filter_and_degree_stepper_elements() {
    let html = common::page_content();

    // Degree stepper controls
    assert!(html.contains("id=\"degMinus\""));
    assert!(html.contains("id=\"degPlus\""));
    assert!(html.contains("id=\"degValue\""));

    // ALL-WITH-CAP toggle
    assert!(html.contains("id=\"hopAllCap\""));

    // Node Cap selector
    assert!(html.contains("id=\"nodeCapSelect\""));
    assert!(html.contains("value=\"25\""));
    assert!(html.contains("value=\"50\""));
    assert!(html.contains("value=\"100\""));
    assert!(html.contains("value=\"200\""));
    assert!(html.contains("value=\"500\""));

    // Search bar enhancements
    assert!(html.contains("id=\"searchClearBtn\""));
    assert!(html.contains("id=\"searchMatchCount\""));

    // Cap warning badge/indicator
    assert!(html.contains("id=\"capWarningBadge\""));

    // Clear filter button
    assert!(html.contains("id=\"btnClearFilter\""));
}

#[test]
fn test_ui_subgraph_filter_and_expansion_engine() {
    let html = common::page_content();

    // Core engine functions
    assert!(html.contains("function compileMatcher"));
    assert!(html.contains("function matchesFilter"));
    assert!(html.contains("function filterAndExpandSubgraph"));

    // Degree stepper logic implementations
    assert!(html.contains("currentDegree = Math.max"));
    assert!(html.contains("textContent = currentDegree"));
    assert!(html.contains("isAllCapped = "));
    assert!(html.contains("nodeCap = cap"));

    // BFS checks
    assert!(html.contains("dirSelect"));
    assert!(html.contains("activeNodes"));
    assert!(html.contains("activeEdges"));
    assert!(html.contains("capWarningBadge').style.display"));

    // Matcher logic check
    assert!(html.contains("path:"));
    assert!(html.contains("kind:"));
}

#[test]
fn test_ui_dependency_tree_direction_preserving() {
    let html = common::page_content();

    // Check dirSelect default is 'out' (Dependencies)
    assert!(html.contains("<option value=\"out\" selected>"));

    // Check BFS uses branchDir logic
    assert!(html.contains("let branchDir = 'both'"));
    assert!(html.contains("branchDir: nextDir"));
    assert!(html.contains("branchDir === 'both'"));
    assert!(html.contains("branchDir === 'out'"));
    assert!(html.contains("branchDir === 'in'"));
}

#[test]
fn test_ui_selected_node_focal_priority() {
    let html = common::page_content();

    // selectedNodeId should take precedence over q when determining seeds
    assert!(html.contains("if (selectedNodeId) {"));
    assert!(html.contains("} else if (q) {"));

    // selectNode should trigger filterAndExpandSubgraph regardless of search input
    assert!(html.contains("if (!skipFilter) {"));

    // btnShowAllMatches should exist
    assert!(html.contains("id=\"btnShowAllMatches\""));
}

#[test]
fn test_ui_noticeable_fading_elements() {
    let html = common::page_content();

    assert!(html.contains(".node-group.faded") && html.contains("--fade-opacity, 0.12")); // ramp var form; 0.12 stays the floor (T1 2026-10-09)
    assert!(html.contains(".edge-path.faded") && html.contains("--edge-fade-opacity, 0.06")); // ramp var form; 0.06 stays the floor (T1 2026-10-09)
    assert!(html.contains(".node-group.heat-root"));

    // Assert INDEX_HTML contains logic applying faded to non-dependencies when a node is selected in the include graph,
    // and clearing faded via the Clear Selection button or Esc.
    assert!(html.contains("currentTab === 'include'") && html.contains("selectedNodeId"));
    assert!(html.contains("reachableDependencies.add") || html.contains("reachableDependencies"));
    assert!(html.contains("faded"));
}

#[test]
fn test_ui_heat_map_coloring() {
    let html = common::page_content();

    // Heat-critical CSS class for drop-shadow glow
    assert!(
        html.contains(".node-group.heat-critical"),
        "missing .node-group.heat-critical CSS"
    );

    // getHeatColor function must exist and derive color from collateral ratio
    assert!(
        html.contains("getHeatColor"),
        "missing getHeatColor function"
    );

    // Heat tier thresholds: 5-tier (0, 0.15, 0.35, 0.60, 0.85)
    assert!(
        html.contains("0.15")
            && html.contains("0.35")
            && html.contains("0.60")
            && html.contains("0.85"),
        "missing heat tier thresholds"
    );

    // graphData.file_collateral used to retrieve collateral counts
    assert!(
        html.contains("file_collateral"),
        "missing file_collateral lookup"
    );

    // Dynamic legend switch to heat mode when selectedNodeId is active in include graph
    assert!(
        html.contains("updateHeatLegend") || html.contains("Heat Map"),
        "missing dynamic legend heat mode"
    );

    // Heat badges on node cards ([ROOT] pill and +XX col badge)
    assert!(
        html.contains("[ROOT]") && html.contains("col") && html.contains("heat-badge"),
        "missing heat badges on node cards"
    );
}

#[test]
fn test_ui_live_mode_fetches_file_collateral() {
    let html = common::page_content();

    // Live server mode must fetch the full payload (incl. file_collateral) from /api/export;
    // regression guard: without it, heat map and Collateral Breakdown show 0 for every file.
    assert!(
        html.contains("fetch('/api/export')"),
        "live mode must fetch /api/export for file_collateral"
    );
    assert!(
        html.contains("graphData.file_collateral ="),
        "live mode must assign graphData.file_collateral"
    );
}

#[test]
fn test_ui_heat_map_tri_state_justification() {
    let page = common::page_content();

    // graph.js must consult the per-root justification map on include-tab selection.
    assert!(
        ariadne_ui::GRAPH_JS.contains("file_justification"),
        "graph.js must read graphData.file_justification"
    );

    // Classification must be a named predicate helper (plan refactor requirement).
    assert!(
        ariadne_ui::MAIN_JS.contains("function classifyDependency("),
        "missing classifyDependency predicate helper (main.js after T-R move)"
    );

    // Tri-state verdicts render as the left verdict spine on reachable
    // dependencies (plan 1790573049166; replaces the old outline overrides).
    assert!(
        ariadne_ui::MAIN_JS.contains("appendVerdictSpine("),
        "missing verdict spine class application (main.js after T-R move)"
    );

    // CSS rules for the spine verdict colors exist in the page.
    assert!(
        page.contains(".verdict-spine.justified"),
        "missing .verdict-spine.justified CSS rule"
    );
    assert!(
        page.contains(".verdict-spine.mixed") && page.contains(".verdict-spine.unused"),
        "missing .verdict-spine.mixed/.unused CSS rules"
    );

    // Legend gains the green verdict row driven by the justification entries.
    assert!(
        ariadne_ui::MAIN_JS.contains("Needed by selected file"),
        "missing Needed-by-selected-file legend row"
    );
    assert!(
        ariadne_ui::MAIN_JS.contains("justified_count"),
        "legend must detect justified entries via justified_count"
    );

    // Live mode must pass file_justification through from /api/export,
    // mirroring the file_collateral guard above.
    assert!(
        ariadne_ui::API_JS.contains("graphData.file_justification ="),
        "live mode must assign graphData.file_justification"
    );

    // Collateral heat badges stay unchanged.
    assert!(
        page.contains("+${c} col"),
        "collateral +N col badge must remain intact"
    );
}

#[test]
fn test_ui_justification_fallback_defaults_empty() {
    // I4 negative lock (string-level, asset test): the file_justification lookup
    // must default with `|| {}` exactly like file_collateral, so a payload without
    // the field renders identical to the pre-justification heat map.
    let graph = ariadne_ui::GRAPH_JS;
    assert!(
        graph.contains("graphData.file_collateral || {}"),
        "file_collateral lookup must keep its || {{}} default"
    );
    assert!(
        graph.contains("graphData.file_justification || {}"),
        "file_justification lookup must default with || {{}} (I4)"
    );
}

#[test]
fn test_ui_inspector_justification_breakdown() {
    let page = common::page_content();

    // Live hydrate: on-demand unbounded detail via /api/justification, cached per
    // node (second selection must not refetch), skipped in the standalone report
    // via the same __ARIADNE_DATA__ mechanism init() already uses.
    assert!(
        ariadne_ui::API_JS.contains("/api/justification?file="),
        "live mode must fetch /api/justification detail for the selected node"
    );
    assert!(
        ariadne_ui::API_JS.contains("justificationCache[nodeId]"),
        "justification detail must be cached keyed by nodeId"
    );
    assert!(
        ariadne_ui::API_JS.contains("isStandaloneReport"),
        "the justification fetch must be gated by the standalone/live check"
    );

    // Named breakdown renderer with a standalone payload path: it renders from
    // the capped graphData.file_justification map without any fetch (I5).
    assert!(
        page.contains("function renderJustificationBreakdown("),
        "missing named renderJustificationBreakdown renderer"
    );
    assert!(
        ariadne_ui::API_JS.contains("payloadJustificationRows"),
        "breakdown must build rows from the capped payload when no live detail"
    );
    assert!(
        ariadne_ui::API_JS.contains("graphData.file_justification"),
        "standalone payload path must read graphData.file_justification"
    );

    // New inspector containers: section, row list, and justified/collateral totals.
    assert!(
        page.contains("id=\"inspJustificationSection\""),
        "missing inspJustificationSection container"
    );
    assert!(
        page.contains("id=\"inspJustificationList\""),
        "missing inspJustificationList row container"
    );
    assert!(
        page.contains("id=\"inspJustificationCount\""),
        "missing inspJustificationCount totals element"
    );

    // Tri-state row reading: justified names, required headers, collateral names,
    // with a '+N more' overflow marker for truncated name lists.
    assert!(
        page.contains("just-justified") && page.contains("just-needs") && page.contains("just-collateral"),
        "missing for/needs/collateral tri-state row parts"
    );
    assert!(
        page.contains("just-more"),
        "missing +N more truncation overflow marker"
    );

    // I4 fallback: the platitude is kept but must be guarded so nodes WITH
    // justification data render the breakdown instead. The breakdown call must
    // precede the insight fallback inside selectNode.
    let graph = ariadne_ui::GRAPH_JS;
    let sel_start = graph
        .find("function selectNode(")
        .expect("selectNode function");
    let body_end = graph
        .find("function clearInspector(")
        .expect("clearInspector function");
    assert!(sel_start < body_end);
    let body = &graph[sel_start..body_end];
    assert!(
        body.contains("renderJustificationBreakdown(id"),
        "selectNode must call the breakdown renderer for the selected node"
    );
    assert!(
        body.contains("&& !justificationShown"),
        "the high-fan-out platitude branch must be guarded by justification data"
    );
    let platitude = graph
        .find("Consider header extraction")
        .expect("platitude must remain in the fallback path");
    assert!(
        platitude > sel_start && platitude < body_end,
        "platitude must stay inside the selectNode insight fallback"
    );
    assert!(
        graph.find("renderJustificationBreakdown(id").unwrap() < platitude,
        "breakdown render must run before the insight fallback in selectNode order"
    );

    // Help text must describe the inspector breakdown.
    assert!(
        page.contains("Include Justification breakdown"),
        "legend help text must mention the inspector Include Justification breakdown"
    );
}

#[test]
fn test_ui_justification_cache_reset_on_live_refresh() {
    // Reviewer remediation: /api/justification detail is cached per node, but the
    // live refresh swaps the graphData payload fields in place. The cache must be
    // reset with the payload, otherwise reselecting a previously-viewed node
    // renders pre-rebuild breakdowns after a rebuild.
    let api = ariadne_ui::API_JS;
    let live = api
        .find("function startLiveRefresh(")
        .expect("startLiveRefresh function");
    let standalone = api
        .find("function isStandaloneReport(")
        .expect("isStandaloneReport function");
    assert!(live < standalone);
    let body = &api[live..standalone];
    let payload = body
        .find("graphData.extraction_proposals =")
        .expect("live refresh must reassign the export payload");
    let reset = body
        .find("justificationCache = {};")
        .expect("live refresh must reset the per-node justification cache");
    let render = body
        .find("applyLoadedData()")
        .expect("live refresh must re-render after the payload swap");
    assert!(
        reset > payload,
        "cache reset must follow the payload reassignment"
    );
    assert!(
        reset < render,
        "cache reset must precede the re-render after the payload swap"
    );
}

#[test]
fn test_ui_inspector_collateral_breakdown_and_path_illumination() {
    let html = common::page_content();

    // Inspector collateral breakdown list container
    assert!(
        html.contains("inspCollateralList"),
        "missing inspCollateralList container in HTML"
    );

    // Sorting by highest collateral first
    assert!(
        html.contains(".col - a.col") || html.contains("b.col - a.col"),
        "missing descending collateral sort in inspector"
    );

    // Collateral summary update in inspector insight
    assert!(
        html.contains("Total Collateral:") || html.contains("Top contributor:"),
        "missing collateral summary in inspector insight"
    );

    // Path illumination helper to trace path from selected root to dependency
    assert!(
        html.contains("illuminatePath") || html.contains("findIncludePath"),
        "missing path illumination logic"
    );
}

#[test]
fn test_onboarding_and_help_content() {
    let html = common::page_content();

    assert!(html.contains("id=\"tourOverlay\""));
    assert!(html.contains("id=\"tourCard\""));
    assert!(html.contains("id=\"welcomeModal\""));
    assert!(html.contains("id=\"helpCenterModal\""));
    assert!(html.contains("id=\"btnHelp\""));

    // Check for tour/help center content covering required features
    assert!(html.contains("Collateral Heat Map"));
    assert!(html.contains("Path Illumination"));
    assert!(html.contains("Degree"));
    assert!(html.contains("Breadcrumbs") || html.contains("Ariadne's Thread"));
    assert!(html.contains("Direction"));
    assert!(html.contains("ALL-WITH-CAP"));
    assert!(html.contains("Cold/Teal")); // Heat map legend colors
    assert!(html.contains("Finding & Decoupling Header Bloat"));
    assert!(html.contains("Isolating Focused Subgraphs"));
    assert!(html.contains("Breaking Circular Dependencies"));

    assert!(html.contains("function startTour"));
    assert!(html.contains("function triggerFirstVisitExperience"));
    assert!(html.contains("function openHelpCenter"));
}

#[test]
fn test_tour_overlay_dismissal_and_positioning() {
    let html = common::page_content();

    // Verify positionTourCard function exists and clamps
    assert!(html.contains("function positionTourCard("));
    assert!(html.contains("Math.max("));
    assert!(html.contains("Math.min(winH"));
    assert!(html.contains("Math.min(winW"));

    // Verify tourOverlay has an onclick dismissal
    assert!(html.contains("onclick=\"endTour()\"") && html.contains("id=\"tourOverlay\""));

    // Verify .tour-highlighted-element CSS
    assert!(html.contains(".tour-highlighted-element"));
    assert!(html.contains("z-index: 1002 !important"));
    assert!(html.contains("outline: 2px solid var(--cyan) !important"));

    // Verify tourCard smooth transition
    assert!(html.contains("transition: top 0.25s cubic-bezier"));

    // Verify btnTourPrev hidden on step 0
}

/// Slice of `styles.css` from the first appended step-row rule to the end
/// of the stylesheet. The T3 step-row block is appended last, so this slice
/// is exactly that block (used for the I8 token-only assertion).
fn step_row_css_block(css: &str) -> &str {
    let start = css
        .find(".step-row {")
        .expect("styles.css must define the .step-row rule");
    &css[start..]
}

/// Source slice from `marker` (a function definition in the bundle) to the
/// next function definition - the body region source-lock asserts run on.
fn body_slice<'a>(html: &'a str, marker: &str) -> &'a str {
    let start = html.find(marker).expect(marker);
    let tail = start + marker.len();
    let end = tail + html[tail..].find("function ").expect("a function follows the marker");
    &html[start..end]
}

#[test]
fn test_ui_loading_overlay_and_progress() {
    let html = common::page_content();

    assert!(html.contains("id=\"loadingOverlay\""));
    assert!(html.contains("id=\"loadingProgress\""));
    assert!(html.contains("id=\"loadingText\""));

    assert!(html.contains("function showProgress"));
    assert!(html.contains("function updateProgress"));
    assert!(html.contains("function hideProgress"));

    // Check integration in init()
    assert!(html.contains("pollStatus"));
    assert!(html.contains("status.progress.state === 'loading'"));
    assert!(html.contains("status.progress.state === 'ready'"));

    // T3 (ui-progress-parity): one snapshot renderer over step rows.
    assert!(html.contains("id=\"loadingSteps\""));
    assert!(html.contains("function renderProgressSnapshot"));
    assert!(html.contains("renderProgressSnapshot(status.progress)"));

    // I7 source lock: the bar fill mirrors format_step_bar's integer math.
    assert!(html.contains("Math.floor(20 * percent / 100)"));
    assert!(html.contains("total_stages"));

    // Clean-track revision: solid \u2588 escapes only (dithered \u2591 never
    // returns); the remainder run rides a builder-made step-bar-remainder span.
    assert!(html.contains("\\u2588") && !html.contains("\\u2591"));
    let bar_builder = body_slice(html.as_str(), "function buildStepBarNodes");
    assert!(bar_builder.contains("step-bar-remainder")
        && bar_builder.contains("createElement('span')"));

    // I8: the appended step-row CSS block paints with var() tokens only.
    let step_css = step_row_css_block(ariadne_ui::STYLES_CSS);
    assert!(!step_css.contains('#'));
    assert!(step_css.contains("var(--"));

    // Legacy fallback trio (shape.md:182-184): hidden while rows render;
    // updateProgress (hop-3 N1/N2 Variant A - the frozen error branch and
    // the ready phase report through it), hideProgress and the empty-steps
    // legacy branch restore it, so 'Error: ...' detail is never invisible.
    assert!(html.contains("function setLegacyFallbackVisible"));
    assert!(html.contains("setLegacyFallbackVisible(false)"));
    assert!(html.matches("setLegacyFallbackVisible(true)").count() >= 2);
    assert!(body_slice(html.as_str(), "function hideProgress").contains("setLegacyFallbackVisible(true)"));
    assert!(body_slice(html.as_str(), "function updateProgress").contains("setLegacyFallbackVisible(true)"));
}

fn assert_hotspot_panel_markup_and_rendering(html: &str) {
    // Panel, left-edge restore tab, and metric selector markup
    assert!(
        html.contains("id=\"hotspotPanel\""),
        "missing hotspotPanel container"
    );
    let tab_at = html.find("id=\"hotspotRestoreTab\"").expect("missing hotspotRestoreTab");
    let tag_start = html[..tab_at].rfind("<button").expect("tab must sit on a button");
    let tag_end = tab_at + html[tab_at..].find('>').expect("tab tag must close");
    let tab_tag = &html[tag_start..tag_end];
    assert!(
        tab_tag.contains("panel-restore-tab") && tab_tag.contains("onclick=\"openHotspots()\""),
        "hotspotRestoreTab must share restore-tab styling and call openHotspots()"
    );
    assert!(!html.contains("id=\"btnHotspots\""), "btnHotspots toolbar button must be gone");
    assert!(!html.contains("id=\"btnInspector\""), "btnInspector toolbar button must be gone");
    assert!(html.contains("id=\"hotspotMetric\""), "missing hotspotMetric selector");
    assert!(
        html.contains("function openHotspots") && !html.contains("toggleHotspots"),
        "openHotspots owns the reopen path and dead toggleHotspots must be gone"
    );

    // Ranking + rendering logic
    assert!(
        html.contains("function computeHotspots"),
        "missing computeHotspots ranking"
    );
    assert!(
        html.contains("function renderHotspots"),
        "missing renderHotspots renderer"
    );
}

fn assert_hotspot_burden_and_list_rules(html: &str) {
    // Collateral burden aggregation must read the full file_collateral map and cache it
    assert!(
        html.contains("function getBurdenMaps"),
        "missing collateral burden aggregation"
    );
    assert!(
        html.contains("graphData.file_collateral") && html.contains("hotspotBurdenSource"),
        "burden aggregation must read graphData.file_collateral with cache invalidation"
    );

    // List capped at top 25 rows; rows flow through selectNode so heat/fade/inspector apply
    assert!(
        html.contains("slice(0, 25)"),
        "hotspot list must cap at 25 rows"
    );
    assert!(
        html.contains("selectHotspotRow") && html.contains("selectNode(id)"),
        "hotspot row click must route through selectNode"
    );
}

fn assert_hotspot_metric_options(html: &str) {
    // All 5 metrics present, collateral burden is default
    assert!(
        html.contains(">Collateral burden<")
            && html.contains(">Fan-in<")
            && html.contains(">Fan-out<")
            && html.contains(">Closure burden<")
            && html.contains(">Cycle participation<"),
        "missing hotspot metrics"
    );
    assert!(
        html.contains("value=\"burden\" selected"),
        "collateral burden must be default metric"
    );
}

fn assert_hotspot_tour_and_canvas_isolation(html: &str) {
    // First-visit tour must demo the hotspot view and restore state on leave
    assert!(
        html.contains("id: '#hotspotPanel'") && html.contains("Hotspots Triage"),
        "guided tour must include a Hotspots step"
    );
    assert!(
        html.contains("tourOpenedHotspots"),
        "tour must only close hotspots if the tour step opened them"
    );
    assert!(
        !html.contains(".hotspot-panel.tour-highlighted-element"),
        "tour highlight must not pin the hotspot panel out of the workspace layout"
    );

    // Older exports without file_collateral degrade to Fan-in with an inline hint
    assert!(
        html.contains("showing Fan-in instead"),
        "missing Fan-in fallback hint"
    );

    // Include-tab-only gating
    assert!(
        html.contains("currentTab !== 'include'"),
        "hotspots must gate on the include tab"
    );

    // Bar normalized to top score
    assert!(
        html.contains("hotspot-bar-fill") && html.contains("e.value / top"),
        "missing normalized hotspot bar"
    );

    // Panel no longer lives inside the canvas, so canvas handlers must drop the obsolete guards.
    assert!(
        !ariadne_ui::GRAPH_JS.contains("closest('.hotspot-panel')"),
        "canvas mousedown/wheel handlers must no longer special-case a panel that is outside the canvas"
    );

    // Wheel fallback is dead once the panel owns its own column (native scrolling takes over).
    assert!(
        !ariadne_ui::GRAPH_JS.contains("hsList"),
        "the hsList wheel fallback is dead once the panel owns its own column"
    );
}

#[test]
fn test_ui_hotspots_panel() {
    let html = common::page_content();

    assert_hotspot_panel_markup_and_rendering(html.as_str());
    assert_hotspot_burden_and_list_rules(html.as_str());
    assert_hotspot_metric_options(html.as_str());
    assert_hotspot_tour_and_canvas_isolation(html.as_str());
}

#[test]
fn test_breadcrumbs_render_in_thread_bar() {
    let html = common::page_content();

    assert!(
        html.contains("id=\"ariadneThread\""),
        "thread bar container missing"
    );
    assert!(
        html.contains("let threadHistory = []"),
        "breadcrumb state missing"
    );
    assert!(
        html.contains("function recordCrumb"),
        "breadcrumb recording missing"
    );
    assert!(
        html.contains("function renderThreadBar"),
        "breadcrumb rendering missing"
    );
    assert!(
        html.contains("function jumpToCrumb"),
        "breadcrumb navigation missing"
    );
    assert!(
        html.contains("recordCrumb(id, node.label)"),
        "selectNode must record a crumb"
    );
    assert!(
        !html.contains("<!-- Breadcrumbs go here -->"),
        "breadcrumb bar must not be an empty placeholder"
    );

    // Auto-selected fallback nodes must not pollute the breadcrumb trail
    assert!(
        html.contains("selectNode(activeNodes[0].id, true, true)"),
        "filter fallback auto-select must skip crumb recording"
    );

    // Clearing the selection re-renders the bar (Clear Selection button, Escape, search)
    assert!(
        html.matches("renderThreadBar()").count() >= 5,
        "selection-clear paths must refresh the breadcrumb bar"
    );
}

// `test_index_markup_has_no_inline_style_or_script_bodies` (with its
// `tag_start_is_asset` helper) moved to theme_contract.rs: since theming
// landed, its contract is the anti-flash bootstrap exception and the eight
// asset scripts theming extended, not general index-markup structure. It also
// keeps this file under the R2 1000-line ceiling after the merge.

#[test]
fn test_script_tag_order_preserves_original_execution_order() {
    let html = ariadne_ui::INDEX_HTML;
    // Classic scripts share one global scope; this order locks the original
    // top-level execution order of the pre-split inline script.
    let order = [
        "/app/api.js",
        "/app/graph.js",
        "/app/file-view.js",
        "/app/hotspots.js",
        "/app/preview.js",
        "/app/provenance.js",
        "/app/main.js",
        // theme.js is new (T6) but its position is just as load-bearing: it
        // re-renders through renderSvg(), so it must sit after main.js.
        "/app/theme.js",
    ];
    let mut pos = 0usize;
    for src in order {
        let found = html[pos..]
            .find(src)
            .unwrap_or_else(|| panic!("missing <script src=\"{src}\"> in index markup"));
        pos += found + src.len();
    }

    // The embedded-data placeholder defines window.__ARIADNE_DATA__ that
    // init() (in main.js, loaded last) reads, so it must precede the assets.
    let placeholder = html.find("/*__ARIADNE_DATA__*/").unwrap();
    let first_asset = html.find("<script src=").unwrap();
    assert!(placeholder < first_asset);
}

#[tokio::test]
async fn test_ui_asset_routes_serve_bundled_content_with_no_store() {
    let assets: [(&str, &str, &str); 9] = [
        (
            "/styles.css",
            ariadne_ui::STYLES_CSS,
            "text/css; charset=utf-8",
        ),
        (
            "/app/api.js",
            ariadne_ui::API_JS,
            "text/javascript; charset=utf-8",
        ),
        (
            "/app/graph.js",
            ariadne_ui::GRAPH_JS,
            "text/javascript; charset=utf-8",
        ),
        (
            "/app/traversal.js",
            ariadne_ui::TRAVERSAL_JS,
            "text/javascript; charset=utf-8",
        ),
        (
            "/app/file-view.js",
            ariadne_ui::FILE_VIEW_JS,
            "text/javascript; charset=utf-8",
        ),
        (
            "/app/hotspots.js",
            ariadne_ui::HOTSPOTS_JS,
            "text/javascript; charset=utf-8",
        ),
        ("/app/preview.js", ariadne_ui::PREVIEW_JS, "text/javascript; charset=utf-8"),
        (
            "/app/provenance.js",
            ariadne_ui::PROVENANCE_JS,
            "text/javascript; charset=utf-8",
        ),
        (
            "/app/main.js",
            ariadne_ui::MAIN_JS,
            "text/javascript; charset=utf-8",
        ),
    ];
    for (path, bundled, mime) in assets {
        let app = ariadne_ui::build_router(common::loading_state());
        let (status, headers, body) = common::request_raw_body(app, path).await;
        assert_eq!(status, StatusCode::OK, "{path} must be served");
        assert_eq!(
            headers.get("Cache-Control").unwrap(),
            "no-store",
            "{path} must never be cached, same rule as the index route"
        );
        assert_eq!(headers.get("Content-Type").unwrap(), mime, "{path}");
        assert!(!body.is_empty(), "{path} must serve a non-empty body");
        assert_eq!(
            body,
            bundled.as_bytes(),
            "{path} must serve the exact bundled asset"
        );
    }
}

#[test]
fn test_ui_extraction_proposal_cards() {
    let page = common::page_content();

    // One shared card builder serves both surfaces (plan D2 refactor rule).
    // It must NOT live in graph.js (R2 headroom constraint).
    assert!(
        page.contains("function renderExtractionProposalCard("),
        "missing shared renderExtractionProposalCard builder"
    );
    assert!(
        !ariadne_ui::GRAPH_JS.contains("renderExtractionProposalCard"),
        "graph.js must stay untouched: the proposal renderer belongs to hotspots/api"
    );

    // Hotspots panel section: sibling AFTER the ranking list.
    assert!(
        page.contains("id=\"proposedExtractionsSection\""),
        "missing proposedExtractionsSection container"
    );
    assert!(
        page.contains("id=\"proposedExtractionsList\""),
        "missing proposedExtractionsList container"
    );
    let ranking = ariadne_ui::INDEX_HTML
        .find("id=\"hotspotList\"")
        .expect("hotspot ranking list");
    let proposals = ariadne_ui::INDEX_HTML
        .find("id=\"proposedExtractionsSection\"")
        .expect("proposed extractions section");
    assert!(
        ranking < proposals,
        "Proposed extractions section must sit after the hotspot ranking list"
    );

    // Inspector tie-in section + card slot.
    assert!(
        page.contains("id=\"inspProposalSection\""),
        "missing inspProposalSection container"
    );
    assert!(
        page.contains("id=\"inspProposalCard\""),
        "missing inspProposalCard slot"
    );

    // Live mode wires proposals from /api/export exactly like file_justification (C1),
    // and the inspector resolves the selected node via target_file_id.
    assert!(
        ariadne_ui::API_JS.contains("graphData.extraction_proposals ="),
        "live mode must assign graphData.extraction_proposals"
    );
    assert!(
        ariadne_ui::API_JS.contains("target_file_id"),
        "inspector tie-in must match proposals via target_file_id"
    );

    // Card content: before/after collateral delta, TU count, needs row, +N more overflow.
    assert!(
        ariadne_ui::HOTSPOTS_JS.contains("collateral_load_before")
            && ariadne_ui::HOTSPOTS_JS.contains("collateral_load_after"),
        "card must render the before/after collateral delta"
    );
    assert!(
        ariadne_ui::HOTSPOTS_JS.contains("consumer_tu_count"),
        "card must render the consumer TU count"
    );
    assert!(
        ariadne_ui::HOTSPOTS_JS.contains("extraction-row") && ariadne_ui::HOTSPOTS_JS.contains("extracted symbols need"),
        "card must render the required-headers needs row"
    );
    assert!(
        ariadne_ui::HOTSPOTS_JS.contains("just-more"),
        "card must render the +N more symbol overflow marker"
    );

    // Card click selects the target file through the hotspot-row entry point.
    assert!(
        ariadne_ui::HOTSPOTS_JS.contains("selectHotspotRow('"),
        "card click must reuse the hotspot row selection entry point"
    );

    // I4 fallback (string-level): defaulted access + zero-length hide.
    assert!(
        ariadne_ui::HOTSPOTS_JS.contains("graphData.extraction_proposals || []"),
        "proposals access must default with || [] (I4)"
    );
    assert!(
        ariadne_ui::HOTSPOTS_JS.contains("proposals.length === 0"),
        "empty proposals must hide the section entirely (I4)"
    );

    // Help text must mention the new section.
    assert!(
        ariadne_ui::INDEX_HTML.contains("Proposed extractions section"),
        "Hotspots help text must mention the Proposed extractions section"
    );
}

// `test_standalone_export_inlines_all_split_assets` moved verbatim to
// `static_html_standalone_export.rs` (R2 ceiling; journal:
// tasks/uta2-r1-journal.md). The suite runs both modules; no assertion
// changed.
