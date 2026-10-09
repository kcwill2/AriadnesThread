// Edge Verdict Story (plan 1790557615891): the root's outgoing include arrows
// carry the verdict via tri-color, u/t used badges, tooltips, legend rows, and
// help copy. UI-only: string-level asset assertions in static_html.rs style.

use crate::common;

#[test]
fn test_edge_verdict_marker_and_css_cascade() {
    let html = ariadne_ui::INDEX_HTML;
    assert!(
        html.contains("id=\"arrow-unused\""),
        "missing arrow-unused marker def in index markup"
    );

    let css = ariadne_ui::STYLES_CSS;
    let leaving = css
        .find(".edge-path.edge-leaving")
        .expect("edge-leaving rule");
    let justified = css
        .find(".edge-path.edge-justified")
        .expect("missing .edge-path.edge-justified rule");
    let unused = css
        .find(".edge-path.edge-unused")
        .expect("missing .edge-path.edge-unused rule");
    assert!(
        justified > leaving && unused > leaving,
        "verdict edge rules must sit after .edge-path.edge-leaving"
    );
    let preview = css
        .find(".edge-path.preview-slim-edge")
        .expect("preview-slim-edge rule");
    assert!(
        justified < preview && unused < preview,
        "verdict edge rules must stay above the E3 preview block so preview keeps winning"
    );

    let unused_body = &css[unused..unused + css[unused..].find('}').expect("unused rule close")];
    common::assert_token_or_hex(unused_body, "red", "#ef4444", "edge-unused must be red");
    assert!(
        unused_body.contains("stroke-dasharray: 6 4"),
        "edge-unused must be dashed 6 4"
    );
    assert!(
        unused_body.contains("stroke-width: 2px"),
        "edge-unused must be 2px"
    );

    let justified_body =
        &css[justified..justified + css[justified..].find('}').expect("justified rule close")];
    assert!(
        justified_body.contains("stroke"),
        "edge-justified must set a stroke"
    );
}

#[test]
fn test_edge_verdict_helpers_live_in_main_js() {
    let main = ariadne_ui::MAIN_JS;
    assert!(
        main.contains("function classifyEdgeVerdict("),
        "missing classifyEdgeVerdict helper"
    );
    assert!(
        main.contains("function verdictBadgeText("),
        "missing verdictBadgeText helper"
    );
    assert!(
        main.contains("function verdictTooltipText("),
        "missing verdictTooltipText helper"
    );

    // Self-explaining badge and tooltip copy anchors.
    assert!(
        main.contains("0/${c} used"),
        "collateral-only badge must render 0/N used"
    );
    assert!(
        main.contains("droppable or slimmable"),
        "unused tooltip must state the include is droppable or slimmable"
    );
    assert!(
        main.contains("system/stdlib headers are not analyzed"),
        "unused tooltip must carry the system/stdlib scope caveat"
    );
}

#[test]
fn test_edge_verdict_graph_wiring() {
    let graph = ariadne_ui::GRAPH_JS;
    assert!(
        graph.contains("classifyEdgeVerdict("),
        "graph.js must classify the root's direct edges"
    );
    assert!(
        graph.contains("verdictBadgeText("),
        "graph.js must build badges via verdictBadgeText"
    );
    assert!(
        graph.contains("edge-justified") && graph.contains("edge-unused"),
        "missing edge-justified/edge-unused class application"
    );
    assert!(
        graph.contains("edge-partial"),
        "missing edge-partial class application"
    );
    assert!(
        graph.contains("url(#arrow-unused)"),
        "unused edges must use the arrow-unused marker"
    );
    assert!(
        graph.contains("appendVerdictTip("),
        "graph.js must attach verdict tooltips via appendVerdictTip"
    );
    assert!(
        ariadne_ui::MAIN_JS.contains("'title'"),
        "the SVG title tooltip element must be created in main.js (R2 headroom)"
    );
    assert!(
        graph.contains("cycle-edge"),
        "tri-coloring must skip cycle edges"
    );

    // Existing behavior locks: neutral badge unchanged; node verdict styling is
    // now the left verdict spine (plan 1790573049166), outline overrides gone.
    assert!(
        graph.contains("+${c} col"),
        "neutral +N col badge must remain intact"
    );
    assert!(
        ariadne_ui::MAIN_JS.contains("appendVerdictSpine("), // relocated with appendVerdictSpine to main.js (T-R 2026-10-09)
        "node verdicts must be applied via the verdict spine"
    );
}

#[test]
fn test_edge_verdict_legend_and_help_copy() {
    let main = ariadne_ui::MAIN_JS;
    assert!(
        main.contains("Needed by selected file"),
        "legend must gain the green Needed-by row"
    );
    assert!(
        main.contains("Unused include (red dashed) — this arrow can go"),
        "legend must name the red dashed arrow as an unused include"
    );
    assert!(
        main.contains("edge-style-unused"),
        "red verdict row must use the dashed legend-line swatch"
    );
    assert!(
        ariadne_ui::STYLES_CSS.contains(".legend-line.edge-style-unused")
            && ariadne_ui::STYLES_CSS.contains("2px dashed"),
        "styles.css must define the dashed legend swatch"
    );
    assert!(
        main.contains("Verdicts relative to selected file"),
        "heat legend needs the root-relative footnote"
    );
    assert!(
        main.contains("system headers not analyzed"),
        "footnote must state the system-header scope limit"
    );
    assert!(
        main.contains("Partly needed (J/T used) or no verdict data"),
        "legend must explain the dominant amber arrow"
    );
    assert!(
        main.contains("edge-style-partial")
            && ariadne_ui::STYLES_CSS.contains(".legend-line.edge-style-partial"),
        "amber verdict row needs its solid swatch class and rule"
    );
    assert!(
        main.contains("Transitive dependency \u{2014} no verdict vs selection"),
        "legend must name grey arrows as transitive dependencies with no root-relative verdict"
    );
    assert!(
        main.contains("edge-style-context")
            && ariadne_ui::STYLES_CSS.contains(".legend-line.edge-style-context"),
        "context row needs its grey swatch class and rule"
    );
    assert!(
        main.contains("hasJustifiedVerdict") && main.contains("hasUnusedVerdict"),
        "green/red legend rows must be gated on actual payload tiers"
    );
    assert!(
        main.contains("greenRow + amberRow + redRow + contextRow"),
        "legend rows must render in needed-to-context order"
    );

    let html = ariadne_ui::INDEX_HTML;
    assert!(
        html.contains("system/stdlib headers are not analyzed"),
        "help copy must carry the stdlib-scope caveat"
    );
    assert!(
        html.contains("red dashed") && html.contains("0/N used"),
        "help copy must describe the arrow vocabulary"
    );
}
