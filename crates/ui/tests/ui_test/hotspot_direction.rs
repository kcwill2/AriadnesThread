//! Direction-follow locks for hotspot row clicks (change track
//! ui-hotspot-canvas-direction, handoff T1). Clicking a hotspot row or an
//! extraction card sets the toolbar Direction to the metric's natural
//! direction (I1/I2) and arms an inbound heat lens keyed to selection
//! PROVENANCE, never to the toolbar value (I3); the inbound heat view is
//! consumer-first (I4), the outbound heat block stays byte-identical behind
//! the delegate branch (I5), and no new payload surface appears (I6). Source
//! -level shape locks in the house idiom; the pre-existing ariadne-ui suite
//! is the untouched equivalence oracle for every manual selection path.

use ariadne_ui::{GRAPH_JS, HOTSPOTS_JS, INDEX_HTML};
use regex::Regex;

/// Body of a named top-level asset function, delimited by its opening brace
/// and the house closing-brace line. Local copy of the
/// extraction_card_playbook.rs function_body idiom (modules do not import
/// across each other); the panic text is asset-neutral because this suite
/// judges both hotspots.js and graph.js.
fn function_body(js: &str, name: &str) -> String {
    let head = format!("function {name}(");
    let at = js
        .find(&head)
        .unwrap_or_else(|| panic!("the asset must define {head}"));
    let open = js[at..]
        .find('{')
        .unwrap_or_else(|| panic!("unterminated body for {name}"));
    let close = js[at + open..]
        .find("\n    }")
        .unwrap_or_else(|| panic!("unterminated body block for {name}"));
    js[at + open + 1..at + open + close].to_string()
}

#[test]
fn test_hotspot_metric_direction_mapping() {
    // I1: burden/fanin are inbound aggregates, fanout/closure are outbound,
    // cycle is a bidirectional rank that keeps the user's toolbar pick.
    let map = function_body(HOTSPOTS_JS, "hotspotMetricDirection");
    assert!(
        map.contains("metric === 'burden'")
            && map.contains("metric === 'fanin'")
            && map.contains("return 'in'"),
        "I1: burden and fanin must map to 'in'"
    );
    assert!(
        map.contains("metric === 'fanout'")
            && map.contains("metric === 'closure'")
            && map.contains("return 'out'"),
        "I1: fanout and closure must map to 'out'"
    );
    assert!(
        map.contains("return null"),
        "I1: cycle must map to null so the toolbar keeps the user's pick (D3)"
    );
}

#[test]
fn test_select_hotspot_row_follows_metric_direction() {
    // I2: the row/click entry point auto-sets the toolbar from the metric and
    // arms the lens via pendingHeatLens; the bare selectNode(id) literal must
    // survive because a pre-existing static_html.rs lock greps for it (R3).
    let body = function_body(HOTSPOTS_JS, "selectHotspotRow");
    assert!(
        body.contains("hotspotMetricDirection(document.getElementById('hotspotMetric').value)"),
        "I2: selectHotspotRow must consult hotspotMetricDirection for the live metric"
    );
    assert!(
        body.contains("document.getElementById('dirSelect').value = dir"),
        "I2: a mapped direction must be written to the dirSelect toolbar"
    );
    assert!(
        body.contains("pendingHeatLens = dir === 'in' ? 'in' : null"),
        "I2: only inbound metrics arm the pending heat lens"
    );
    assert!(
        body.contains("selectNode(id)"),
        "I2: the bare selectNode(id) literal must survive for the pre-existing lock"
    );
    assert!(
        !body.contains("selectNode(id,") && !body.contains("selectNode(id "),
        "I2: the lens travels via pendingHeatLens, never as a selectNode argument"
    );
}

#[test]
fn test_select_node_consumes_pending_lens() {
    // I3: consumption happens as the FIRST statements of selectNode, so no
    // other selection path (canvas click, breadcrumb, quick-open, inspector
    // link) can inherit a pending lens. Signature stays unchanged.
    let body = function_body(GRAPH_JS, "selectNode");
    let first_statements: Vec<String> = body
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .take(2)
        .map(|line| line.to_string())
        .collect();
    assert_eq!(
        first_statements,
        vec!["hotspotHeatLens = pendingHeatLens;", "pendingHeatLens = null;"],
        "I3: selectNode must consume the pending lens as its first two statements"
    );
}

#[test]
fn test_direction_change_clears_lens_flags() {
    // I3: a MANUAL Direction change is the equivalence boundary - both lens
    // flags must be nulled before the refilter so manual selections keep
    // today's byte-identical outbound behavior.
    const MARKER: &str = "document.getElementById('dirSelect').onchange";
    let at = HOTSPOTS_JS
        .find(MARKER)
        .expect("hotspots.js must wire the dirSelect onchange handler");
    let region = &HOTSPOTS_JS[at..];
    let handler = match region.find("\n    };") {
        Some(end) => &region[..end],
        None => region,
    };
    let cleared_lens = handler
        .find("hotspotHeatLens = null;")
        .expect("I3: the dirSelect onchange must null hotspotHeatLens");
    let cleared_pending = handler
        .find("pendingHeatLens = null;")
        .expect("I3: the dirSelect onchange must null pendingHeatLens");
    let refilter = handler
        .find("filterAndExpandSubgraph()")
        .expect("I3: the dirSelect onchange must keep calling filterAndExpandSubgraph");
    assert!(
        cleared_lens < refilter && cleared_pending < refilter,
        "I3: both lens flags must be nulled BEFORE the refilter call"
    );
}

#[test]
fn test_graph_delegates_inbound_heat_before_outbound_block() {
    // I5: renderSvg gains ONLY the leading provenance branch; the old
    // condition line becomes an `} else if` rewrite and its block body stays
    // byte-identical behind it.
    let body = function_body(GRAPH_JS, "renderSvg");
    let lead = "if (currentTab === 'include' && hotspotHeatLens === 'in' && selectedNodeId) {";
    let lead_at = body
        .find(lead)
        .unwrap_or_else(|| panic!("I5: renderSvg must lead with the inbound-lens branch: {lead}"));
    let outbound = "} else if (currentTab === 'include' && selectedNodeId) {";
    let outbound_at = body.find(outbound).unwrap_or_else(|| {
        panic!("I5: the outbound heat block must sit behind the literal {outbound}")
    });
    assert!(lead_at < outbound_at, "I5: the inbound branch must precede the outbound block");
    assert!(
        body[lead_at..outbound_at].contains("renderInboundHeatView(nodesLayer, edgesLayer);"),
        "I5: the leading branch must delegate to renderInboundHeatView(nodesLayer, edgesLayer)"
    );
}

#[test]
fn test_consumer_heat_view_reads_payer_side() {
    // I4: the payment badge is the CONSUMER's payment toward the selection,
    // never the root-side map; the view uses its own reverse BFS.
    let heat = function_body(HOTSPOTS_JS, "renderInboundHeatView");
    assert!(
        heat.contains("((graphData.file_collateral || {})[nid] || {})[selectedNodeId]"),
        "I4: paid(nid) must be the consumer-first lookup on the guarded idiom"
    );
    assert!(
        !heat.contains("computeFullDependencies"),
        "I4: the inbound view must use its own computeFullConsumers, not the forward BFS"
    );
}

#[test]
fn test_inbound_heat_reads_no_new_payload_fields() {
    // I6: the only graphData.<field> reads inside the inbound view are the
    // two already-shipped maps; a new field would be new payload surface.
    let heat = function_body(HOTSPOTS_JS, "renderInboundHeatView");
    let reads = Regex::new(r"graphData\.([A-Za-z_][A-Za-z0-9_]*)").unwrap();
    let mut seen_reads = 0usize;
    for capture in reads.captures_iter(&heat) {
        seen_reads += 1;
        let field = &capture[1];
        assert!(
            matches!(&*field, "file_collateral" | "file_justification"),
            "I6: renderInboundHeatView must not read new payload surface (found graphData.{field})"
        );
    }
    assert!(
        seen_reads > 0,
        "I6: the payer-side lookup must read at least the collateral map"
    );
}

#[test]
fn test_help_copy_documents_direction_follow() {
    // I8: one new Hotspots help item right after the row-click anchor,
    // naming the cycle exception (D3) so the kept behavior is documented.
    const ANCHOR: &str =
        "Clicking a row selects the file and renders its heat map live beside the open list.</li>";
    let anchor_at = INDEX_HTML
        .find(ANCHOR)
        .expect("I8: the existing row-click help item must stay");
    let start = anchor_at + ANCHOR.len();
    let end = INDEX_HTML[start..]
        .find("</li>")
        .expect("I8: a new help li must follow the row-click item");
    let item = &INDEX_HTML[start..start + end];
    assert!(
        item.contains("<li"),
        "I8: the direction-follow copy must be a list item right after the anchor"
    );
    assert!(
        item.contains("natural direction"),
        "I8: the help copy must document the metric's natural direction follow"
    );
    assert!(
        item.contains("Cycle participation keeps your current Direction"),
        "I8: the help copy must name the cycle exception (D3)"
    );
}
