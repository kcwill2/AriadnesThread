use crate::common;

fn css_rule_body(css: &str, selector: &str) -> String {
    let at = css.find(selector).unwrap_or_else(|| {
        panic!("styles.css must contain a rule for {selector}");
    });
    let open = css[at..]
        .find('{')
        .unwrap_or_else(|| panic!("unterminated rule block for {selector}"));
    let close = css[at + open..]
        .find('}')
        .unwrap_or_else(|| panic!("unterminated rule block for {selector}"));
    css[at + open + 1..at + open + close].to_string()
}

// Slice a top-level preview.js function body out of the asset text, following
// the same indentation-based boundary idiom as static_html_sections.rs.
fn preview_fn_source(preview: &str, signature: &str) -> String {
    let start = preview
        .find(signature)
        .unwrap_or_else(|| panic!("preview.js must define {signature}"));
    let rest = &preview[start..];
    let end = ["\n    function ", "\n    async function ", "\n    window."]
        .iter()
        .filter_map(|boundary| rest[1..].find(boundary).map(|offset| offset + 1))
        .min()
        .unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn test_hypothetical_banner_lives_in_canvas_container() {
    let html = ariadne_ui::INDEX_HTML;

    let canvas = html
        .find("id=\"canvasContainer\"")
        .expect("canvas container must exist");
    let svg_close_at = html[canvas..]
        .find("</svg>")
        .expect("graph svg must close inside the canvas container")
        + canvas;
    let banner = html
        .find("id=\"previewBanner\"")
        .expect("hypothetical preview banner element must exist");
    let legend = html
        .find("id=\"legendBox\"")
        .expect("legend box must exist");
    assert!(
        banner > svg_close_at && banner < legend,
        "preview banner must sit after the svg and before the legend, inside the canvas container"
    );

    let tag_start = html[..banner]
        .rfind("<div")
        .expect("banner id must sit inside a div tag");
    let tag_end = html[banner..]
        .find('>')
        .expect("unterminated preview banner tag");
    let banner_tag = &html[tag_start..banner + tag_end];
    assert!(
        banner_tag.contains("preview-banner"),
        "banner must carry the preview-banner class"
    );
    assert!(
        banner_tag.contains("hidden"),
        "banner must default to hidden — it only appears while a preview is active"
    );
    assert!(
        html.contains("Hypothetical preview"),
        "banner must state that the picture is a hypothetical after-state"
    );
    assert!(
        html.contains("preview-banner-glyph"),
        "banner must carry a warning glyph"
    );

    // Absolute positioning inside the canvas requires a positioned ancestor.
    let container = css_rule_body(ariadne_ui::STYLES_CSS, ".canvas-container");
    assert!(
        container.contains("position: relative"),
        ".canvas-container must be positioned so the banner anchors inside it"
    );
}

#[test]
fn test_hypothetical_preview_wiring_and_teardown() {
    let preview_src = ariadne_ui::PREVIEW_JS;

    for marker in [
        "previewShowHypothetical",
        "previewBanner",
        "preview-hypothetical",
        "just-preview-btn-active",
        "previewActivateButton",
    ] {
      assert!(
        preview_src.contains(marker),
            "preview overlay must wire the hypothetical affordance marker {marker}"
        );
    }

    // The legend header must warn that "After extraction" is not the current view.
    assert!(
        preview_src.contains("\\u26A0 After extraction"),
        "legend view label must warn the after-extraction view is hypothetical"
    );

    let clear_body = preview_fn_source(preview_src, "function clearPreview");
    for marker in [
        "previewBanner",
        "preview-hypothetical",
        "previewActivateButton",
    ] {
        assert!(
            clear_body.contains(marker),
            "clearPreview must tear down {marker} state (I2)"
        );
    }

    // I3: the banner must never swallow canvas interaction.
    let banner = css_rule_body(ariadne_ui::STYLES_CSS, ".preview-banner");
    assert!(
        banner.contains("pointer-events: none"),
        ".preview-banner must not intercept pan/zoom or node hit-testing"
    );

    let activate = preview_fn_source(preview_src, "function previewActivateButton");
    assert!(
        activate.contains(".just-preview-btn"),
        "only one preview button may hold the active state at a time"
    );
}

#[test]
fn test_hypothetical_affordance_css() {
    let css = ariadne_ui::STYLES_CSS;

    let banner = css_rule_body(css, ".preview-banner");
    assert!(
        banner.contains("position: absolute"),
        ".preview-banner must be absolutely positioned in the canvas"
    );
    let banner_z = banner
        .find("z-index:")
        .map(|at| banner[at + "z-index:".len()..].trim_start().chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
        .unwrap_or_default();
    let banner_z: u32 = banner_z.parse().expect(".preview-banner must set a numeric z-index");
    let legend_z: u32 = css_rule_body(css, ".legend-box")
        .split("z-index:")
        .nth(1)
        .unwrap_or_default()
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .expect(".legend-box must set a numeric z-index");
    assert!(
        banner_z > legend_z,
        ".preview-banner must stack above the legend box ({banner_z} > {legend_z})"
    );
    assert!(
        banner.contains("dashed"),
        ".preview-banner must carry the amber dashed hypothetical theme"
    );
    common::assert_token_or_hex(
        &banner,
        "preview-amber-edge",
        "#f59e0b",
        ".preview-banner must stroke the hypothetical amber",
    );
    common::assert_var_or(
        &banner,
        &["var(--preview-hyp-bg)", "rgba(69, 26, 3", "rgba(69,26,3"],
        ".preview-banner needs the translucent dark amber backdrop",
    );
    assert!(
        banner.contains("monospace"),
        ".preview-banner must use the monospace canvas voice"
    );

    let frame = css_rule_body(css, "#canvasContainer.preview-hypothetical");
    assert!(
        frame.contains("outline: 1px dashed") && frame.contains("outline-offset: -3px"),
        "the whole canvas must be framed while a hypothetical preview is active"
    );

    let active = css_rule_body(css, ".just-preview-btn-active");
    common::assert_var_or(
        &active,
        &[
            "var(--preview-amber-edge)",
            "var(--preview-amber-text)",
            "#f59e0b",
            "#fbbf24",
        ],
        "the active preview button must be clearly amber, distinct from hover",
    );
}

#[test]
fn test_proposed_header_node_construction() {
    let preview = ariadne_ui::PREVIEW_JS;

    assert!(
        preview.contains("function previewProposedFilename"),
        "preview.js must derive the placeholder header name"
    );
    let naming = preview_fn_source(preview, "function previewProposedFilename");
    assert!(
        naming.contains("_extracted.h"),
        "the placeholder name must follow the planner _extracted.h stem convention"
    );
    assert!(
        naming.contains("lastIndexOf"),
        "multi-dot stems must split at the LAST dot"
    );

    let inject = preview_fn_source(preview, "function previewInjectProposedNode");
    for marker in [
        "preview-proposed-node",
        "data-preview-synthetic",
        "tabAllNodes",
        "createElementNS",
        "Proposed header (hypothetical placeholder name)",
        "PROPOSED",
    ] {
        assert!(
            inject.contains(marker),
            "previewInjectProposedNode must build the synthetic card marker {marker}"
        );
    }

    // I3: the synthetic card is chrome, never a selectable graph node.
    let proposed_block = preview
        .find("function previewInjectProposedNode")
        .map(|start| &preview[start..])
        .unwrap_or("");
    let synthetic_guard = proposed_block
        .find("function previewInjectProposedEdges")
        .unwrap_or_else(|| panic!("previewInjectProposedEdges must follow the node builder"));
    assert!(
        !proposed_block[..synthetic_guard].contains("dataset.nodeId"),
        "the proposed node must never carry data-node-id (I3)"
    );

    let apply = preview_fn_source(preview, "async function applyPreview");
    assert!(
        apply.contains("previewInjectProposedNode("),
        "applyPreview must inject the proposed node"
    );

    // SF-1: the badge shares no row with the label (y=12 vs baseline y=25), so
    // a length threshold only makes it a dead branch — it must always render.
    assert!(
        !inject.contains("label.length"),
        "the PROPOSED badge must render unconditionally, not gated on label length (SF-1)"
    );
    let badge_x_at = inject
        .find("badge.setAttribute('x', '")
        .expect("the badge must set an explicit x inside the 170px card (N3)");
    let badge_x: i32 = inject[badge_x_at + "badge.setAttribute('x', '".len()..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .expect("badge x must be numeric");
    // 8 chars at ~4.2px each in 7px monospace ≈ 34px; right edge must sit at
    // least 6px inside the card border: x <= 170 - 6 - 34 = 130.
    assert!(
        badge_x <= 170 - 6 - 34,
        "the PROPOSED badge must end at least 6px inside the card (x={badge_x} <= 130, N3)"
    );
    assert!(
        !ariadne_ui::STYLES_CSS.contains("letter-spacing: 0.5"),
        "badge letter-spacing must be dropped so PROPOSED fits the card (N3)"
    );

    assert!(
        preview.contains("Proposed header (placeholder name)"),
        "the legend must explain the proposed header row"
    );
    // N4: the swatch row already names the proposed header; the view label is
    // the short hypothetical warning, nothing more.
    assert!(
        preview.contains("\\u26A0 After extraction (hypothetical)"),
        "the legend view label must be the short hypothetical warning (N4)"
    );
    assert!(
        !preview.contains("proposed edges'),"),
        "the view label must not re-list the proposed edges (N4)"
    );
    common::assert_var_or(
        preview,
        &[
            "background:transparent;border:1px dashed var(--preview-proposed-stroke)",
            "background:transparent;border:1px dashed #a78bfa",
        ],
        "the legend proposal swatch must be a violet dashed outline, not a solid chip",
    );
}

#[test]
fn test_proposed_edges_and_slim_edge_wiring() {
    let preview = ariadne_ui::PREVIEW_JS;
    let html = ariadne_ui::INDEX_HTML;

    assert!(
        html.contains("id=\"arrow-proposed\""),
        "defs must carry a violet proposed-edge arrow marker"
    );
    let arrow = &html[html.find("id=\"arrow-proposed\"").unwrap()..];
    let arrow_path_end = arrow.find("</marker>").expect("unterminated arrow-proposed marker");
    common::assert_token_or_hex(
        &arrow[..arrow_path_end],
        "preview-proposed-stroke",
        "#a78bfa",
        "arrow-proposed must be violet like the proposed edges",
    );

    let edges = preview_fn_source(preview, "function previewInjectProposedEdges");
    for marker in [
        "preview-proposed-edge",
        "url(#arrow-proposed)",
        "data-preview-synthetic",
        "required_headers",
        "data-edge-source",
        "preview-slim-edge",
    ] {
        assert!(
            edges.contains(marker),
            "previewInjectProposedEdges must wire {marker}"
        );
    }

    // E-I4 boundary: required-header proposal edges need the full impact payload.
    assert!(
        edges.contains("full") && edges.contains("16"),
        "required-header edges are full-mode-only and capped"
    );

    let clear_body = preview_fn_source(preview, "function clearPreview");
    for marker in ["preview-slim-edge", "data-preview-synthetic"] {
        assert!(
            clear_body.contains(marker),
            "clearPreview must tear down {marker} (I2)"
        );
    }
}

#[test]
fn test_proposed_visual_css() {
    let css = ariadne_ui::STYLES_CSS;

    let card = css_rule_body(css, ".node-group.preview-proposed-node rect");
    common::assert_token_or_hex(
        &card,
        "preview-proposed-stroke",
        "#a78bfa",
        "the proposed card must read as a violet dashed hypothetical header",
    );
    assert!(
        card.contains("stroke-dasharray"),
        "the proposed card must read as a violet dashed hypothetical header"
    );

    let group = css_rule_body(css, ".node-group.preview-proposed-node {");
    assert!(
        group.contains("pointer-events: none"),
        "the proposed card must never take part in hit-testing (I3)"
    );

    let edge = css_rule_body(css, ".edge-path.preview-proposed-edge {");
    common::assert_token_or_hex(
        &edge,
        "preview-proposed-stroke",
        "#a78bfa",
        ".preview-proposed-edge must be a violet dashed bezier",
    );
    assert!(
        edge.contains("stroke-dasharray") && edge.contains("fill: none"),
        ".preview-proposed-edge must be a violet dashed bezier"
    );
    assert!(
        edge.contains("pointer-events: none"),
        "proposed edges must not intercept canvas interaction"
    );

    let slim = css_rule_body(css, ".edge-path.preview-slim-edge {");
    common::assert_token_or_hex(
        &slim,
        "preview-amber-text",
        "#fbbf24",
        ".preview-slim-edge must be the amber slimmed-consumer treatment",
    );
    assert!(
        slim.contains("stroke-dasharray"),
        ".preview-slim-edge must be the amber slimmed-consumer treatment"
    );
    assert!(
        slim.contains("opacity: 1 !important"),
        ".preview-slim-edge must stay visible under edges-has-selection dimming"
    );
}

#[test]
fn test_help_text_describes_the_proposed_preview_affordances() {
    let html = ariadne_ui::INDEX_HTML;
    let help_at = html
        .find("Collateral Heat Map")
        .expect("heat map help paragraph must exist");
    let help_end = html[help_at..]
        .find("</p>")
        .expect("unterminated help paragraph")
        + help_at;
    let help = &html[help_at..help_end];

    for phrase in [
        "PROPOSED",
        "placeholder name",
        "hypothetical",
        "slimmed consumer edge",
    ] {
        assert!(
            help.contains(phrase),
            "the Collateral Heat Map help text must explain the proposed-header preview affordance ({phrase})"
        );
    }
}

#[test]
fn test_proposed_card_placements_below_source_column() {
    let preview = ariadne_ui::PREVIEW_JS;

    // Midpoint placement is structurally impossible in the layered layout
    // (170px cards, ~70px gutters) - the collision-descending below-column
    // placement replaces it entirely.
    assert!(
        !preview.contains("anchorsClose") && !preview.contains("/ 2 - 85")
            && !preview.contains("Math.hypot"),
        "preview.js must not keep any midpoint/fallback-midpoint placement path"
    );

    let collides = preview_fn_source(preview, "function previewProposedRectCollides");
    assert!(
        collides.contains("170") && collides.contains("42"),
        "the collision test must be a plain AABB over the rendered 170x42 card rects"
    );

    let place = preview_fn_source(preview, "function previewResolveProposedPosition");
    assert!(
        place.contains("source.x"),
        "the proposed card must start in the source's own column (same x)"
    );
    assert!(
        place.contains("source.y + 72") || place.contains("+ 72;"),
        "the proposed card must start one 72px row pitch below the source"
    );
    assert!(
        place.contains("previewProposedRectCollides(") && place.contains("< 8"),
        "placement must descend 72px steps while colliding, capped at 8 steps"
    );
    assert!(
        place.contains("Math.max") && place.contains("+ 72"),
        "after the descent budget placement must fall back to max(rendered y) + 72 \
         at the source column - a band below the whole layout"
    );

    let inject = preview_fn_source(preview, "function previewInjectProposedNode");
    assert!(
        inject.contains("previewResolveProposedPosition(source, activeNodes)"),
        "collision tests must run against the RENDERED cards (activeNodes), not \
         tabAllNodes with stale coordinates"
    );
    assert!(
        inject.contains("return null") && inject.contains("!source"),
        "previewInjectProposedNode must skip injection when the source is unresolvable"
    );
}
