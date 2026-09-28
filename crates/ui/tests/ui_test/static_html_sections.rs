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

    // graph.js gained the clear-on-rerender hook (E-I6) plus the verdict-spine
    // machinery (plan 1790573049166); the cap keeps real headroom under the
    // 1000-line hygiene ceiling so future features still fit.
    assert!(
        ariadne_ui::GRAPH_JS.contains("ariadnePreview.clearPreview()"),
        "graph.js must clear the preview when the selection re-renders"
    );
    assert!(
        ariadne_ui::GRAPH_JS.lines().count() <= 990,
        "graph.js must stay within the spine-plan line budget (cap 990)"
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
fn test_ui_heat_tier_outlines_are_flat() {
    // Plan 1790573049166: every collateral tier (0..=4) thickens its tier stroke
    // but stays FLAT — colored glows were deleted and glow is reserved for
    // selection/root (cyan). The verdict channel is the spine, not the outline.
    for tier in 0..=4 {
        let body = css_rule_body(ariadne_ui::STYLES_CSS, &format!(".node-group.heat-tier-{tier} rect"));
        assert!(
            body.contains("stroke-width"),
            ".node-group.heat-tier-{tier} rect must thicken the tier stroke"
        );
        assert!(
            !body.contains("filter"),
            ".node-group.heat-tier-{tier} rect must carry no glow filter"
        );
    }

    // The class must be applied at render time, and I5: heat-critical stays.
    assert!(
        ariadne_ui::GRAPH_JS.contains("heat-tier-"),
        "graph.js must apply the heat-tier-<tier> class on collateral nodes"
    );
    assert!(
        ariadne_ui::GRAPH_JS.contains("heat-critical"),
        "heat-critical must keep being applied (I5)"
    );

    // Cascade ownership: .heat-tier-4 is the effective tier-4 renderer. The legacy
    // .heat-critical rect rule keeps the selector (structural lock + graph.js class)
    // but must not declare a competing filter, and must sit BEFORE the tier-4 rule.
    let critical_at = ariadne_ui::STYLES_CSS
        .find(".node-group.heat-critical rect")
        .expect("heat-critical rule must remain in styles.css");
    let tier4_at = ariadne_ui::STYLES_CSS
        .find(".node-group.heat-tier-4 rect")
        .expect("heat-tier-4 rule must exist");
    assert!(
        critical_at < tier4_at,
        ".node-group.heat-tier-4 rect must come after .node-group.heat-critical rect so it wins the equal-specificity cascade"
    );
    let critical = css_rule_body(ariadne_ui::STYLES_CSS, ".node-group.heat-critical rect");
    assert!(
        !critical.contains("filter"),
        ".node-group.heat-critical rect must carry no filter declaration; .heat-tier-4 owns tier-4 rendering"
    );

    // The selected ring must win over any co-applied heat tier by specificity of
    // declaration, not by source order luck.
    let selected = css_rule_body(ariadne_ui::STYLES_CSS, ".node-group.selected rect");
    assert!(
        selected.contains("filter: drop-shadow(0 0 14px var(--cyan-glow)) !important"),
        ".node-group.selected rect must guarantee the cyan ring with !important regardless of co-applied heat tiers"
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
        body.contains("width: 400px") && body.contains("flex-shrink: 0"),
        ".hotspot-panel must reserve a fixed 400px workspace column"
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

#[test]
fn test_inspector_panel_is_wider_and_collapsible() {
    let css = ariadne_ui::STYLES_CSS;
    let html = ariadne_ui::INDEX_HTML;
    let main = ariadne_ui::MAIN_JS;

    let inspector = css_rule_body(css, ".inspector");
    assert!(
        inspector.contains("width: 440px") && inspector.contains("flex-shrink: 0"),
        ".inspector must reserve a fixed 440px workspace column"
    );

    assert!(
        html.contains("id=\"inspectorPanel\""),
        "the inspector aside must carry id=\"inspectorPanel\" so it can be shown and hidden"
    );
    assert!(
        html.contains("id=\"btnInspector\""),
        "the toolbar must carry a persistent btnInspector toggle"
    );
    assert!(
        html.contains("onclick=\"toggleInspector()\""),
        "the inspector collapse/toggle controls must call toggleInspector()"
    );

    // The ✕ collapse control must live in the inspector head itself, wired to
    // toggleInspector() — not be satisfiable by the toolbar button alone.
    let head_at = html
        .find("class=\"inspector-head\"")
        .expect("inspector-head region must exist in index.html");
    let head_end = head_at
        + html[head_at..]
            .find("</div>")
            .expect("inspector-head must close before the inspector body");
    let head_region = &html[head_at..head_end];
    let collapse_at = head_region
        .find("class=\"inspector-collapse\"")
        .expect("inspector-head must contain a .inspector-collapse control");
    let collapse_btn_at = head_region[..collapse_at]
        .rfind("<button")
        .expect("the .inspector-collapse class must be declared on a <button> element");
    let collapse_tag_end = head_region[collapse_btn_at..]
        .find('>')
        .expect("the collapse button tag must close");
    let collapse_btn_tag = &head_region[collapse_btn_at..collapse_btn_at + collapse_tag_end];
    assert!(
        collapse_btn_tag.contains("onclick=\"toggleInspector()\""),
        "the .inspector-collapse button in inspector-head must be wired to toggleInspector()"
    );

    // The inspector starts expanded, so the toolbar toggle must start active.
    let btn_at = html
        .find("id=\"btnInspector\"")
        .expect("btnInspector toolbar toggle must exist");
    let btn_tag_start = html[..btn_at]
        .rfind("<button")
        .expect("the btnInspector id must sit on a <button> element");
    let btn_tag_end = btn_at
        + html[btn_at..]
            .find('>')
            .expect("the btnInspector tag must close");
    let btn_tag = &html[btn_tag_start..btn_tag_end];
    assert!(
        btn_tag.contains("class=\"btn-secondary active\""),
        "#btnInspector must carry the active class initially because the inspector starts expanded"
    );

    for marker in [
        "function toggleInspector(",
        "function collapseInspector(",
        "function expandInspector(",
    ] {
        assert!(
            main.contains(marker),
            "main.js must declare the inspector toggle helper {marker}"
        );
    }

    // The tour spotlights '.inspector' by measuring it, so the highlight path
    // must expand a collapsed inspector before resolving the target element.
    let guard_at = main
        .find("step.id === '.inspector'")
        .expect("the tour highlight path must guard the '.inspector' step target");
    let guard_window = &main[guard_at..guard_at + 240];
    assert!(
        guard_window.contains("expandInspector("),
        "the '.inspector' tour guard must call expandInspector() before measuring a hidden panel"
    );

    let collapse = css_rule_body(css, ".inspector-collapse");
    assert!(
        collapse.contains("cursor: pointer"),
        ".inspector-collapse must advertise itself as clickable (cursor: pointer)"
    );
}

fn js_fn_body(src: &str, signature: &str) -> String {
    let start = src
        .find(signature)
        .unwrap_or_else(|| panic!("missing JS function declaration: {signature}"));
    let open = src[start..]
        .find('{')
        .unwrap_or_else(|| panic!("unterminated body for {signature}"));
    let mut depth = 0usize;
    let chars: Vec<char> = src[start + open..].chars().collect();
    let mut end = chars.len();
    for (index, ch) in chars.iter().enumerate() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = index + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    chars[..end].iter().collect()
}

#[test]
fn test_collapsed_panels_offer_edge_restore_tabs() {
    let css = ariadne_ui::STYLES_CSS;
    let html = ariadne_ui::INDEX_HTML;
    let main = ariadne_ui::MAIN_JS;
    let file_view = ariadne_ui::FILE_VIEW_JS;

    // Both restore tabs must live inside the canvas container, hidden while
    // their panel is open, each wired to its own panel's restore entry point.
    let canvas_at = html
        .find("id=\"canvasContainer\"")
        .expect("canvas container must exist");
    let aside_at = html
        .find("<aside class=\"inspector\"")
        .expect("the inspector aside must sit after the canvas container");
    let canvas_region = &html[canvas_at..aside_at];
    for (tab_id, wiring) in [
        ("inspectorRestoreTab", "onclick=\"expandInspector()\""),
        ("sourceRestoreTab", "onclick=\"restoreSourceDrawer()\""),
    ] {
        let id_at = canvas_region
            .find(&format!("id=\"{tab_id}\""))
            .unwrap_or_else(|| panic!("{tab_id} must live inside the canvas container"));
        let tag_start = canvas_region[..id_at]
            .rfind("<button")
            .unwrap_or_else(|| panic!("{tab_id} must sit on a <button> element"));
        let tag_end = id_at
            + canvas_region[id_at..]
                .find('>')
                .unwrap_or_else(|| panic!("the {tab_id} tag must close"));
        let tab_tag = &canvas_region[tag_start..tag_end];
        assert!(
            tab_tag.contains("class=\"panel-restore-tab\""),
            "{tab_id} must share the shared .panel-restore-tab styling"
        );
        assert!(
            tab_tag.contains(wiring),
            "{tab_id} must be wired to {wiring} in the markup itself"
        );
        assert!(
            tab_tag.contains("hidden"),
            "{tab_id} must start hidden while its panel is open"
        );
    }

    // The tabs dock to the canvas' right edge; the canvas owns the frame.
    let tab_rule = css_rule_body(css, ".panel-restore-tab");
    assert!(
        tab_rule.contains("position: absolute") && tab_rule.contains("right: 0"),
        ".panel-restore-tab must dock to the canvas' right edge"
    );
    assert!(
        tab_rule.contains("writing-mode: vertical-rl"),
        ".panel-restore-tab must read vertically so it stays a slim edge tab"
    );
    assert!(
        tab_rule.contains("cursor: pointer"),
        ".panel-restore-tab must advertise itself as clickable"
    );
    let canvas_rule = css_rule_body(css, ".canvas-container");
    assert!(
        canvas_rule.contains("position: relative"),
        ".canvas-container must be position: relative so the restore tabs dock to it"
    );

    // Reveal/hide behavior lives inside the four panel functions themselves.
    let collapse_inspector = js_fn_body(main, "function collapseInspector(");
    assert!(
        collapse_inspector.contains("inspectorRestoreTab")
            && collapse_inspector.contains("hidden = false"),
        "collapseInspector must reveal #inspectorRestoreTab"
    );
    let expand_inspector = js_fn_body(main, "function expandInspector(");
    assert!(
        expand_inspector.contains("inspectorRestoreTab")
            && expand_inspector.contains("hidden = true"),
        "expandInspector must hide #inspectorRestoreTab"
    );
    assert!(
        file_view.contains("function restoreSourceDrawer("),
        "file-view.js must declare restoreSourceDrawer for the source restore tab"
    );
    let close_drawer = js_fn_body(file_view, "function closeSourceDrawer(");
    assert!(
        close_drawer.contains("sourceRestoreTab") && close_drawer.contains("hidden = false"),
        "closeSourceDrawer must reveal #sourceRestoreTab"
    );
    let restore_drawer = js_fn_body(file_view, "function restoreSourceDrawer(");
    assert!(
        restore_drawer.contains("sourceRestoreTab") && restore_drawer.contains("hidden = true"),
        "restoreSourceDrawer must hide #sourceRestoreTab"
    );
    assert!(
        restore_drawer.contains("classList.add('open')"),
        "restoreSourceDrawer must reopen the drawer in place without reloading its content"
    );
}

#[test]
fn test_clear_selection_button_in_toolbar() {
    let html = ariadne_ui::INDEX_HTML;

    let btn_at = html
        .find("id=\"btnClearSelection\"")
        .expect("the toolbar must carry an explicit btnClearSelection button");
    assert!(
        html.contains("onclick=\"clearSelection()\""),
        "the clear selection button must be wired to clearSelection()"
    );

    // It sits immediately after #btnInspector in the same toolbar-left group,
    // with no other button wedged between them.
    let inspector_at = html
        .find("id=\"btnInspector\"")
        .expect("btnInspector toolbar toggle must exist");
    let layout_at = html
        .find("id=\"layoutSelect\"")
        .expect("the layout select must exist after the toolbar buttons");
    assert!(
        inspector_at < btn_at && btn_at < layout_at,
        "btnClearSelection must live between btnInspector and the layout control in toolbar-left"
    );
    let inspector_tag_end = inspector_at
        + html[inspector_at..]
            .find('>')
            .expect("the btnInspector tag must close");
    let clear_tag_start = html[..btn_at]
        .rfind("<button")
        .expect("the btnClearSelection id must sit on a <button> element");
    assert!(
        !html[inspector_tag_end..clear_tag_start].contains("<button"),
        "btnClearSelection must be the button immediately after btnInspector"
    );
}

#[test]
fn test_clear_selection_button_starts_hidden() {
    let html = ariadne_ui::INDEX_HTML;

    let btn_at = html
        .find("id=\"btnClearSelection\"")
        .expect("btnClearSelection must exist in the toolbar");
    let tag_start = html[..btn_at]
        .rfind("<button")
        .expect("the btnClearSelection id must sit on a <button> element");
    let tag_end = btn_at
        + html[btn_at..]
            .find('>')
            .expect("the btnClearSelection tag must close");
    let btn_tag = &html[tag_start..tag_end];
    assert!(
        btn_tag.contains("class=\"btn-secondary\""),
        "btnClearSelection must share the secondary toolbar button styling"
    );
    assert!(
        btn_tag.contains("display:none"),
        "btnClearSelection must start hidden because the app boots with no selection"
    );
}

#[test]
fn test_clear_selection_resets_state_and_rerenders() {
    let graph = ariadne_ui::GRAPH_JS;

    let body = js_fn_body(graph, "function clearSelection(");
    assert!(
        body.contains("selectedNodeId = null"),
        "clearSelection must clear selectedNodeId"
    );
    assert!(
        body.contains("clearInspector()"),
        "clearSelection must fully clear the inspector on clear"
    );
    assert!(
        body.contains("renderSvg()"),
        "clearSelection must re-render the SVG so fading and selection visuals reset"
    );
    assert!(
        body.contains("renderThreadBar()"),
        "clearSelection must refresh the breadcrumb bar"
    );
    assert!(
        body.contains("renderHotspots()"),
        "clearSelection must re-render hotspots without re-filtering"
    );
    // Regression guard: filterAndExpandSubgraph's tail falls back to
    // selectNode(activeNodes[0]) when nothing is selected, so routing the
    // clear through it would instantly re-select a seed node and undo the clear.
    // Clear must mirror the Esc path: preserve the current subgraph view.
    assert!(
        !body.contains("filterAndExpandSubgraph"),
        "clearSelection must not call filterAndExpandSubgraph: its auto-select \
         fallback re-picks a seed node and defeats the clear"
    );
}

#[test]
fn test_canvas_background_click_no_longer_clears_selection() {
    let graph = ariadne_ui::GRAPH_JS;

    // Accidental clears (a mis-drag that lands as a click on empty canvas) are
    // replaced by the explicit Clear Selection button and Esc. Pan/zoom stay.
    let panzoom = js_fn_body(graph, "function setupCanvasPanZoom(");
    assert!(
        !panzoom.contains("selectedNodeId = null"),
        "the canvas container must no longer clear the selection on background click"
    );
    assert!(
        !panzoom.contains("addEventListener('click'"),
        "setupCanvasPanZoom must carry no click listener once background clearing is gone"
    );
    assert!(
        panzoom.contains("addEventListener('mousedown'") && panzoom.contains("addEventListener('wheel'"),
        "pan (mousedown) and zoom (wheel) must survive the click-listener removal"
    );
}

#[test]
fn test_tour_canvas_step_advises_clear_button() {
    let main = ariadne_ui::MAIN_JS;

    let step_at = main
        .find("id: '#canvasContainer'")
        .expect("the tour must keep its canvas step");
    let step_end = step_at
        + main[step_at..]
            .find('\n')
            .expect("the canvas tour step must end at a line break");
    let step = &main[step_at..step_end];
    assert!(
        !step.contains("Click the empty background"),
        "the canvas tour step must not teach the removed background-click clear"
    );
    assert!(
        step.contains("Clear Selection") && step.contains("Esc"),
        "the canvas tour step must point to the Clear Selection button and Esc"
    );
}

#[test]
fn test_render_svg_syncs_clear_selection_button() {
    let graph = ariadne_ui::GRAPH_JS;

    // Visibility rides the render path itself, not selectNode/clear callers,
    // so every path that changes selection keeps the button honest.
    let render = js_fn_body(graph, "function renderSvg(");
    assert!(
        render.contains("btnClearSelection"),
        "renderSvg must toggle #btnClearSelection visibility from selectedNodeId"
    );
}
