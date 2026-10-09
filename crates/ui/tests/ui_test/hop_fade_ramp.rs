//! Hop-distance opacity ramp locks for the heat-overlay fade (change track
//! ui-degree-opacity-ramp, handoff T1). Faded graph nodes and edges carry a
//! per-element inline custom property keyed to the undirected BFS hop
//! distance from the selected node (I3); the two frozen tables are the whole
//! tuning surface, with today's flat values as their exact floors (I1) and a
//! monotone descent under the kept-edge dim (I2). The faded SET of every
//! heat pass stays byte-identical - the ramp only adds one property line per
//! faded branch (I4/I5) - and illuminatePath keeps its flat binary fade,
//! protected by a removeProperty leak guard (I6). Source-level shape locks
//! in the house idiom; the pre-existing ariadne-ui suite is the untouched
//! equivalence oracle. No JS is executed.

use ariadne_ui::{GRAPH_JS, HOTSPOTS_JS, MAIN_JS, STYLES_CSS};
use regex::Regex;

/// Body of a named top-level asset function, delimited by its opening brace
/// and the house closing-brace line. Local copy of the
/// hotspot_direction.rs function_body idiom (modules do not import across
/// each other); the panic text is asset-neutral because this suite judges
/// graph.js, hotspots.js and main.js alike.
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

/// Parse one frozen ramp table out of its owner-asset declaration. The
/// literal IS the contract (shape Q2), so a missing declaration is a hard
/// panic, not a silent default.
fn ramp_table(js: &str, name: &str) -> Vec<f64> {
    let re = Regex::new(&format!(r"const {name} = \[([^\]]*)\];")).unwrap();
    let entries = re
        .captures(js)
        .unwrap_or_else(|| panic!("the ramp owner asset must define {name}"))
        .get(1)
        .map(|matched| matched.as_str())
        .unwrap_or("");
    entries
        .split(',')
        .map(|entry| {
            entry
                .trim()
                .parse::<f64>()
                .unwrap_or_else(|_| panic!("{name} must hold decimal entries"))
        })
        .collect()
}

/// I1 floor-exactness: a faded element with no finite hop renders exactly
/// today's flat values - both table floors and both CSS fallbacks.
#[test]
fn test_floor_exactness() {
    let node_table = ramp_table(GRAPH_JS, "HEAT_RAMP_NODE_OPACITIES");
    let edge_table = ramp_table(GRAPH_JS, "HEAT_RAMP_EDGE_OPACITIES");
    assert_eq!(
        node_table[node_table.len() - 1],
        0.12,
        "I1: the node ramp floor must equal the pre-ramp flat value 0.12"
    );
    assert_eq!(
        edge_table[edge_table.len() - 1],
        0.06,
        "I1: the edge ramp floor must equal the pre-ramp flat value 0.06"
    );
    assert!(
        STYLES_CSS.contains("var(--fade-opacity, 0.12)"),
        "I1: the node .faded rule must read the ramp var with the 0.12 fallback"
    );
    assert!(
        STYLES_CSS.contains("var(--edge-fade-opacity, 0.06)"),
        "I1: the edge .faded rule must read the ramp var with the 0.06 fallback"
    );
}

/// I2 monotonicity: the parsed tables must equal the frozen contract
/// literally; the ordering guards restate the descent so an edit to any
/// entry fails with a reason naming the violated property.
#[test]
fn test_monotone_ramp_tables() {
    let frozen_node_table: [f64; 6] = [0.85, 0.55, 0.36, 0.23, 0.15, 0.12];
    let frozen_edge_table: [f64; 6] = [0.30, 0.28, 0.18, 0.12, 0.08, 0.06];
    let node_table = ramp_table(GRAPH_JS, "HEAT_RAMP_NODE_OPACITIES");
    let edge_table = ramp_table(GRAPH_JS, "HEAT_RAMP_EDGE_OPACITIES");
    assert_eq!(
        node_table,
        frozen_node_table.to_vec(),
        "I2: the node table IS the frozen contract"
    );
    assert_eq!(
        edge_table,
        frozen_edge_table.to_vec(),
        "I2: the edge table IS the frozen contract"
    );
    assert_eq!(node_table[0], 0.85, "I2: hop-1 nodes start near-opaque at 0.85");
    for pair in node_table.windows(2) {
        assert!(pair[0] > pair[1], "I2: the node ramp must strictly decrease");
    }
    for pair in edge_table.windows(2) {
        assert!(
            pair[0] >= pair[1],
            "I2: the edge ramp must be non-increasing"
        );
    }
    assert!(
        edge_table[0] <= 0.30,
        "I2/R4: a hop-1 faded edge stays strictly under the 0.35 kept-edge dim"
    );
}

/// I3 distance independence: the walk reads only the full tab dataset; the
/// degree/cap/seeds/filter machinery is structurally excluded, so neither
/// the open degree-setting bug nor any toolbar state can influence the ramp.
#[test]
fn test_distance_independence() {
    let body = function_body(GRAPH_JS, "computeHopDistances");
    assert!(
        body.contains("tabAllEdges"),
        "I3: hop distance must come from the full tabAllEdges dataset"
    );
    for foreign_input in [
        "activeNodes",
        "activeEdges",
        "nodeCap",
        "currentDegree",
        "isAllCapped",
        "seeds",
        "isFiltering",
    ] {
        assert!(
            !body.contains(foreign_input),
            "I3: computeHopDistances must never read {foreign_input}"
        );
    }
}

/// I4/I5 faded-set equivalence: every .faded condition literal stays
/// byte-identical at all four heat sites, and each pass owns exactly one
/// ramp setter per property name (a setter in a reachable branch would push
/// these counts past one).
#[test]
fn test_faded_sites_and_property_counts() {
    assert!(
        GRAPH_JS.contains("if (!reachableDependencies.has(src) && src !== selectedNodeId) {"),
        "I4: the outbound edge fade condition must stay byte-identical"
    );
    assert!(
        GRAPH_JS.contains("} else if (reachableDependencies.has(nid)) {"),
        "I4: the outbound node reachable branch must stay byte-identical"
    );
    assert!(
        HOTSPOTS_JS.contains("} else if (payers.has(nid)) {"),
        "I4: the inbound node payer branch must stay byte-identical"
    );
    assert!(
        HOTSPOTS_JS.contains("if (dst !== selectedNodeId && !payers.has(dst)) {"),
        "I4: the inbound edge fade condition must stay byte-identical"
    );
    assert_eq!(
        GRAPH_JS.matches("setProperty('--fade-opacity'").count(),
        1,
        "I4: exactly one node ramp setter in graph.js"
    );
    assert_eq!(
        HOTSPOTS_JS.matches("setProperty('--fade-opacity'").count(),
        1,
        "I4: exactly one node ramp setter in hotspots.js"
    );
    assert_eq!(
        GRAPH_JS.matches("setProperty('--edge-fade-opacity'").count(),
        1,
        "I4: exactly one edge ramp setter in graph.js"
    );
    assert_eq!(
        HOTSPOTS_JS.matches("setProperty('--edge-fade-opacity'").count(),
        1,
        "I4: exactly one edge ramp setter in hotspots.js"
    );
}

/// I6: illuminatePath stays a flat binary fade - it clears stale inline ramp
/// properties from the post-heat DOM instead of adopting them.
#[test]
fn test_illuminate_path_stays_flat() {
    let body = function_body(MAIN_JS, "illuminatePath");
    assert!(
        body.contains("removeProperty('--fade-opacity')"),
        "I6: the path pass must clear stale node ramp properties"
    );
    assert!(
        body.contains("removeProperty('--edge-fade-opacity')"),
        "I6: the path pass must clear stale edge ramp properties"
    );
    assert!(
        !body.contains("setProperty("),
        "I6: the path pass must not set any ramp property"
    );
    assert!(
        !body.contains("HEAT_RAMP"),
        "I6: the path pass must not reference the ramp tables"
    );
    assert!(
        body.contains("if (pathNodes.has(nid)) {"),
        "I6: the path node condition literal must stay unedited"
    );
    assert!(
        body.contains("if (pathEdgeKeys.has("),
        "I6: the path edge condition literal must stay unedited"
    );
}
