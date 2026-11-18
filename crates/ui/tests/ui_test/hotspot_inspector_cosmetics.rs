//! Cosmetic locks (2026-11-18): hotspot rows keep the file name readable
//! (path on its own line, full path in the tooltip), the inspector link
//! lists get a text filter, and proposal targets are discoverable from the
//! hotspot list (badge + section count).

use ariadne_ui::{GRAPH_JS, HOTSPOTS_JS, INDEX_HTML, MAIN_JS, STYLES_CSS};

/// Body of a CSS rule block for an exact selector line (`selector {`).
fn css_rule(css: &str, selector: &str) -> String {
    let head = format!("{selector} {{");
    let at = css
        .find(&head)
        .unwrap_or_else(|| panic!("styles.css must define `{head}`"));
    let close = css[at..].find('}').expect("unterminated css rule");
    css[at + head.len()..at + close].to_string()
}

/// Body of a named asset function, delimited by the house closing-brace line.
fn function_body(js: &str, name: &str) -> String {
    let head = format!("function {name}(");
    let at = js
        .find(&head)
        .unwrap_or_else(|| panic!("the asset must define {head}"));
    let open = js[at..].find('{').expect("unterminated signature");
    let close = js[at + open..]
        .find("\n    }")
        .expect("unterminated body block");
    js[at + open + 1..at + open + close].to_string()
}

#[test]
fn test_hotspot_row_name_column_can_shrink_without_eating_the_name() {
    let row = css_rule(STYLES_CSS, ".hotspot-row");
    assert!(
        row.contains("grid-template-columns: 16px minmax(0, 1fr) auto auto;"),
        "the name column must be minmax(0, 1fr) so long content cannot widen it"
    );
}

#[test]
fn test_hotspot_path_sits_on_its_own_line_under_the_name() {
    let path = css_rule(STYLES_CSS, ".hotspot-path");
    assert!(path.contains("grid-column: 2;"), "path must sit under the name");
    assert!(path.contains("grid-row: 2;"), "path must take the second row");
    assert!(
        path.contains("overflow-wrap: anywhere;"),
        "long paths wrap instead of being cut off"
    );
    let name = css_rule(STYLES_CSS, ".hotspot-name");
    assert!(name.contains("grid-row: 1;"), "name must own the first row");
    let bar = css_rule(STYLES_CSS, ".hotspot-bar");
    assert!(bar.contains("grid-row: 3;"), "bar moves below the path");
}

#[test]
fn test_hotspot_row_tooltip_carries_the_full_path() {
    let body = function_body(HOTSPOTS_JS, "renderHotspots");
    assert!(
        body.contains("hotspotRowTooltip(e, metric)"),
        "row title must come from hotspotRowTooltip"
    );
    let tip = function_body(HOTSPOTS_JS, "hotspotRowTooltip");
    assert!(tip.contains("entry.node.path"), "tooltip must include the path");
}

#[test]
fn test_hotspot_rows_flag_extraction_proposal_targets() {
    let body = function_body(HOTSPOTS_JS, "renderHotspots");
    assert!(body.contains("graphData.extraction_proposals"));
    assert!(body.contains("hotspot-proposal-badge"));
    assert!(INDEX_HTML.contains("id=\"proposedExtractionsCount\""));
    let section = function_body(HOTSPOTS_JS, "renderProposalsSection");
    assert!(section.contains("proposedExtractionsCount"));
}

#[test]
fn test_inspector_link_lists_have_filter_inputs() {
    for id in ["inspUpstreamFilter", "inspDownstreamFilter"] {
        assert!(
            INDEX_HTML.contains(&format!("id=\"{id}\"")),
            "index.html must define the {id} input"
        );
        assert!(
            MAIN_JS.contains(&format!("bindLinkListFilter('{id}'")),
            "main.js must wire {id}"
        );
    }
    let filter = function_body(MAIN_JS, "applyLinkListFilter");
    assert!(filter.contains("toLowerCase()"), "filter is case-insensitive");
    assert!(filter.contains("dataset.filterText"), "filter matches name + path");
}

#[test]
fn test_inspector_rows_carry_filter_text_and_filters_reset_on_selection() {
    assert!(
        GRAPH_JS.matches("data-filter-text=").count() >= 2,
        "both incoming and outgoing rows carry data-filter-text"
    );
    assert!(GRAPH_JS.contains("resetLinkListFilter('inspUpstreamFilter'"));
    assert!(GRAPH_JS.contains("resetLinkListFilter('inspDownstreamFilter'"));
}
