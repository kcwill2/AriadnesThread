//! Selecting a node rebuilds its neighborhood: the view centers on the
//! selection, a busy badge shows while it works, and the rebuild reads the
//! cached tab index instead of rescanning every edge (on large repos those
//! scans were the bulk of the click-to-graph lag).

use ariadne_ui::{GRAPH_JS, INDEX_HTML, MAIN_JS, STYLES_CSS};

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).unwrap_or_else(|| panic!("missing {signature}"));
    let rest = &source[start + signature.len()..];
    let end = rest.find("\n    function ").unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn the_rebuild_centers_on_the_selected_node() {
    let filter = function_body(GRAPH_JS, "function filterAndExpandSubgraph(");
    let layout_at = filter.find("applyLayout();").expect("the rebuild lays out");
    let center_at = filter.find("centerOnNode(selectedNodeId)").expect("the rebuild centers");
    assert!(layout_at < center_at, "center only after the layout has placed the node");
    let center = function_body(MAIN_JS, "function centerOnNode(");
    assert!(center.contains("clientWidth / 2") && center.contains("zoomScale"));
}

#[test]
fn a_selection_shows_a_busy_badge_until_it_is_drawn() {
    assert!(INDEX_HTML.contains("id=\"selectionBusyBadge\""));
    assert!(STYLES_CSS.contains(".canvas-container.selection-busy"));
    let select = function_body(GRAPH_JS, "function selectNode(");
    assert!(select.contains("setSelectionBusy(true)"));
    assert!(
        select.contains("finally { setSelectionBusy(false); }"),
        "the badge clears even when the rebuild throws"
    );
    assert!(select.contains("setTimeout(run, 0)"), "yield so the badge paints first");
}

#[test]
fn the_rebuild_and_inspector_read_the_cached_index() {
    let filter = function_body(GRAPH_JS, "function filterAndExpandSubgraph(");
    assert!(!filter.contains("tabAllEdges.forEach"), "no per-click adjacency rebuild");
    assert!(!filter.contains("tabAllEdges.filter"), "active edges come from the index");
    assert!(!filter.contains("queue.shift()"), "shift() is linear per pop");
    let inspector = function_body(GRAPH_JS, "function selectNodeNow(");
    assert!(inspector.contains("tabIndex().inEdges") && inspector.contains("tabIndex().outEdges"));
    let index = function_body(MAIN_JS, "function tabIndex(");
    assert!(index.contains("outEdges, inEdges"));
}
