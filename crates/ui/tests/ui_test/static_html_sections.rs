use crate::common;

#[test]
fn test_inspector_link_sections_are_collapsible() {
    let html = ariadne_ui::INDEX_HTML;

    for (list_id, title_id, count_id) in [
        ("inspUpstreamList", "inspUpstreamTitle", "inspUpstreamCount"),
        (
            "inspDownstreamList",
            "inspDownstreamTitle",
            "inspDownstreamCount",
        ),
    ] {
        let list = html
            .find(&format!("id=\"{list_id}\""))
            .unwrap_or_else(|| panic!("missing {list_id} container"));
        let open = html[..list]
            .rfind("<details")
            .unwrap_or_else(|| panic!("{list_id} must be wrapped in a <details> element"));
        let wrapper = &html[open..list];
        let tag_end = wrapper.find('>').expect("unterminated <details> tag");
        assert!(
            wrapper[..tag_end].contains("open"),
            "{list_id} section must default to expanded (open attribute)"
        );
        assert!(
            !wrapper.contains("</details>"),
            "{list_id} must sit directly inside its enclosing <details>"
        );
        let summary = wrapper
            .find("<summary")
            .unwrap_or_else(|| panic!("{list_id} section needs a <summary> before the list"));
        let summary_end = wrapper[summary..]
            .find("</summary>")
            .unwrap_or_else(|| panic!("{list_id} summary must close before the list"));
        let summary_body = &wrapper[summary..summary + summary_end];
        assert!(
            summary_body.contains(&format!("id=\"{title_id}\""))
                && summary_body.contains(&format!("id=\"{count_id}\"")),
            "{list_id} summary must keep the title ({title_id}) and count ({count_id}) visible when collapsed"
        );
        assert!(
            html[list..].contains("</details>"),
            "{list_id} section wrapper must close after the list"
        );
    }
}

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

#[test]
fn test_justification_section_fills_vertical_space() {
    let css = ariadne_ui::STYLES_CSS;
    let html = ariadne_ui::INDEX_HTML;
    let api = ariadne_ui::API_JS;

    let body = css_rule_body(css, ".inspector-body");
    assert!(
        body.contains("flex-direction: column"),
        ".inspector-body must lay its sections out as a flex column"
    );

    let section = css_rule_body(css, "#inspJustificationSection");
    assert!(
        section.contains("flex: 1") && section.contains("flex-direction: column"),
        "#inspJustificationSection must grow (flex: 1 ...) as a flex column to fill free vertical space"
    );
    assert!(
        section.contains("overflow: clip"),
        "#inspJustificationSection needs overflow: clip so its inner list can never paint over later siblings"
    );

    let list = css_rule_body(css, "#inspJustificationList");
    assert!(
        list.contains("flex: 1") && list.contains("overflow-y: auto"),
        "#inspJustificationList must flex-grow and scroll inside the grown section area"
    );

    assert!(
        !html.contains("max-height:220px"),
        "index.html must not cap the justification list at 220px; the CSS flex story owns its height"
    );

    let fn_start = api
        .find("function renderJustificationBreakdown")
        .expect("renderJustificationBreakdown missing from api.js");
    let fn_end = api[fn_start..]
        .find("\n    function ")
        .map(|offset| fn_start + offset)
        .unwrap_or(api.len());
    let render_fn = &api[fn_start..fn_end];
    assert!(
        render_fn.contains("style.display = 'flex'"),
        "the justification section show toggle must use display 'flex' so the CSS column layout applies"
    );
    assert!(
        !render_fn.contains("style.display = 'block'"),
        "the justification section must not fall back to display 'block' when shown"
    );
}

#[test]
fn test_justification_section_cannot_shrink_below_content() {
    let css = ariadne_ui::STYLES_CSS;
    let section = css_rule_body(css, "#inspJustificationSection");

    let has_zero_min_height = section
        .split(';')
        .map(str::trim)
        .filter(|decl| decl.starts_with("min-height"))
        .any(|decl| {
            decl.split(':')
                .nth(1)
                .map(str::trim)
                .is_some_and(|value| value == "0" || value == "0px")
        });
    assert!(
        !has_zero_min_height,
        "#inspJustificationSection must not set min-height: 0; min-height: auto floors it at min-content \
         (title + list floor) so flex-shrink can never collapse it below its own content"
    );
    assert!(
        section.contains("overflow: clip"),
        "#inspJustificationSection must use a NON-scrollable overflow (visible/clip); the min-content \
         floor exists only while the flexbox automatic minimum stays content-based. overflow: hidden, \
         auto, or scroll turns the section into a scroll container, which zeroes that automatic \
         minimum (the c08d1e6 regression mechanism) and lets it collapse below its content again. \
         clip additionally guarantees nothing paints over later siblings."
    );
}

// E3 lands preview.js as a NEW split asset. Read it straight from disk so this
// test compiles before the lib.rs constant exists: the red run must fail on
// missing markers/registration, never on a compile error.
fn preview_asset_text() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/preview.js");
    std::fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn test_extraction_impact_preview_overlay() {
    let preview = preview_asset_text();
    let page = common::page_content() + preview.as_str();

    // Public overlay API (E-I6: a new asset, not more graph.js).
    for marker in ["window.ariadnePreview", "applyPreview", "clearPreview"] {
        assert!(
            page.contains(marker),
            "preview overlay asset must expose {marker} (E-I6)"
        );
    }

    // Node-marking classes: applied in preview.js, styled in styles.css.
    for class in ["preview-eliminated", "preview-kept", "preview-still-reachable"] {
        assert!(
            page.contains(class),
            "preview overlay must apply the {class} class"
        );
        assert!(
            ariadne_ui::STYLES_CSS.contains(&format!(".{class}")),
            "styles.css must define a rule for .{class}"
        );
    }

    // Legend replaces the heat legend with the after-state summary.
    assert!(
        page.contains("After extraction:"),
        "preview legend must carry the after-extraction summary line"
    );

    // E-I4 proof of the standalone path: the asset reads the capped sample
    // fields and admits the sample honestly with a "+N more" note.
    assert!(
        preview.contains("dropped_sample") && preview.contains("more eliminated"),
        "preview.js must drive the standalone capped-sample path (E-I4)"
    );

    // Row control lives in the api.js justification renderer.
    assert!(
        ariadne_ui::API_JS.contains("Preview after extraction"),
        "justification rows with impact need the Preview after extraction control"
    );
    assert!(
        ariadne_ui::API_JS.contains("just-preview-btn"),
        "the preview control must carry the just-preview-btn class"
    );

    // Proposal-card control rides the shared card builder (hotspots.js) and
    // names the currently selected consumer as the preview root.
    assert!(
        ariadne_ui::HOTSPOTS_JS.contains("Preview drop from selected file"),
        "proposal cards need the selection-aware preview control"
    );

    // graph.js gains only the clear-on-rerender hook (E-I6).
    assert!(
        ariadne_ui::GRAPH_JS.contains("ariadnePreview.clearPreview()"),
        "graph.js must clear the preview when the selection re-renders"
    );
    assert!(
        ariadne_ui::GRAPH_JS.lines().count() <= 960,
        "graph.js must stay within the E-I6 line budget (cap 960)"
    );

    // Help text describes the overlay in the heat map legend region.
    assert!(
        ariadne_ui::INDEX_HTML.contains("Preview after extraction"),
        "heat map help text must mention the Preview after extraction overlay"
    );
}

#[test]
fn test_preview_marks_are_solid_color_coded() {
    // The legend card promises color-coded after-state chips; the graph itself
    // must deliver solid tinted cards, not thin outlines on a muted node.
    let eliminated = css_rule_body(ariadne_ui::STYLES_CSS, ".node-group.preview-eliminated rect");
    assert!(
        eliminated.contains("fill: #450a0a"),
        ".preview-eliminated rect must be tinted solid dark red"
    );
    assert!(
        eliminated.contains("stroke-dasharray: 4 3"),
        ".preview-eliminated rect must keep the dashed red outline"
    );
    assert!(
        eliminated.contains("drop-shadow(0 0 12px rgba(239, 68, 68"),
        ".preview-eliminated rect must glow red like the legend chip"
    );
    assert!(
        !eliminated.contains("opacity: 0.55"),
        ".preview-eliminated must not mute the card; the solid fill is the mark"
    );

    let still = css_rule_body(ariadne_ui::STYLES_CSS, ".node-group.preview-still-reachable rect");
    assert!(
        still.contains("fill: #422006"),
        ".preview-still-reachable rect must be tinted solid dark amber"
    );
    assert!(
        still.contains("stroke-dasharray: 2 3"),
        ".preview-still-reachable rect must keep the tight amber dash"
    );
    assert!(
        still.contains("drop-shadow(0 0 10px rgba(245, 158, 11"),
        ".preview-still-reachable rect must glow amber like the legend chip"
    );

    let kept = css_rule_body(ariadne_ui::STYLES_CSS, ".node-group.preview-kept rect");
    assert!(
        kept.contains("fill: #052e16"),
        ".preview-kept rect must be tinted solid dark green"
    );
    assert!(
        kept.contains("drop-shadow(0 0 10px rgba(74, 222, 128"),
        ".preview-kept rect must glow green like the legend chip"
    );
}

#[test]
fn test_ui_heat_tier_outlines_glow() {
    // Every collateral tier (0..=4), not just tier 4, must paint a visible colored glow.
    let tiers = [
        (0, "rgba(56, 189, 248"),
        (1, "rgba(34, 197, 94"),
        (2, "rgba(234, 179, 8"),
        (3, "rgba(249, 115, 22"),
        (4, "rgba(239, 68, 68"),
    ];
    for (tier, rgb) in tiers {
        let body = css_rule_body(ariadne_ui::STYLES_CSS, &format!(".node-group.heat-glow-{tier} rect"));
        assert!(
            body.contains("drop-shadow") && body.contains("stroke-width"),
            ".node-group.heat-glow-{tier} rect must glow with a thickened outline"
        );
        assert!(
            body.contains(rgb),
            ".node-group.heat-glow-{tier} rect must glow in the tier color {rgb}"
        );
    }

    // The class must be applied at render time, and I5: heat-critical stays.
    assert!(
        ariadne_ui::GRAPH_JS.contains("heat-glow-"),
        "graph.js must apply the heat-glow-<tier> class on collateral nodes"
    );
    assert!(
        ariadne_ui::GRAPH_JS.contains("heat-critical"),
        "heat-critical must keep being applied (I5)"
    );

    // The justified ring gets a stronger, locked glow.
    let justified = css_rule_body(ariadne_ui::STYLES_CSS, ".node-group.dep-justified rect");
    assert!(
        justified.contains("drop-shadow(0 0 10px rgba(74, 222, 128, 0.75)"),
        ".node-group.dep-justified rect must use the strengthened 10px green glow"
    );

    // Cascade ownership: .heat-glow-4 is the effective tier-4 renderer. The legacy
    // .heat-critical rect rule keeps the selector (structural lock + graph.js class)
    // but must not declare a competing filter, and must sit BEFORE the glow-4 rule.
    let critical_at = ariadne_ui::STYLES_CSS
        .find(".node-group.heat-critical rect")
        .expect("heat-critical rule must remain in styles.css");
    let glow4_at = ariadne_ui::STYLES_CSS
        .find(".node-group.heat-glow-4 rect")
        .expect("heat-glow-4 rule must exist");
    assert!(
        critical_at < glow4_at,
        ".node-group.heat-glow-4 rect must come after .node-group.heat-critical rect so it wins the equal-specificity cascade"
    );
    let critical = css_rule_body(ariadne_ui::STYLES_CSS, ".node-group.heat-critical rect");
    assert!(
        !critical.contains("filter"),
        ".node-group.heat-critical rect must carry no filter declaration; .heat-glow-4 owns tier-4 rendering"
    );

    // The selected ring must win over any co-applied heat glow by specificity of
    // declaration, not by source order luck.
    let selected = css_rule_body(ariadne_ui::STYLES_CSS, ".node-group.selected rect");
    assert!(
        selected.contains("filter: drop-shadow(0 0 14px var(--cyan-glow)) !important"),
        ".node-group.selected rect must guarantee the cyan ring with !important regardless of co-applied heat glows"
    );
}

#[test]
fn test_hotspots_panel_is_collapsible_layout_column() {
    let html = ariadne_ui::INDEX_HTML;

    // The panel is a direct workspace child placed before the canvas, like the
    // inspector — never a floating overlay that can cover the legend box.
    let workspace = html.find("class=\"workspace\"").expect("workspace container");
    let panel = html.find("id=\"hotspotPanel\"").expect("hotspot panel");
    let canvas = html.find("id=\"canvasContainer\"").expect("canvas container");
    assert!(
        panel > workspace && panel < canvas,
        "hotspot panel must sit inside .workspace before the graph canvas"
    );

    let body = css_rule_body(ariadne_ui::STYLES_CSS, ".hotspot-panel");
    assert!(
        body.contains("position: static"),
        ".hotspot-panel must be a static flex column, not an absolute overlay"
    );
    assert!(
        body.contains("width: 300px") && body.contains("flex-shrink: 0"),
        ".hotspot-panel must reserve a fixed 300px workspace column"
    );
    assert!(
        body.contains("background: var(--bg-surface)"),
        ".hotspot-panel must paint an opaque workspace surface"
    );
    assert!(
        !body.contains("max-height"),
        ".hotspot-panel must not cap its height; the column owns its height"
    );

    let list = css_rule_body(ariadne_ui::STYLES_CSS, ".hotspot-list");
    assert!(
        list.contains("min-height: 0") && list.contains("flex: 1"),
        ".hotspot-list must grow and scroll inside the column instead of the panel growing past the viewport"
    );

    let page = common::page_content();
    assert!(
        page.contains("beside the open list"),
        "help text must say the heat map renders beside the open list, not behind it"
    );
}
