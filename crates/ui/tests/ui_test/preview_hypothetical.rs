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
        banner.contains("dashed") && banner.contains("#f59e0b"),
        ".preview-banner must carry the amber dashed hypothetical theme"
    );
    assert!(
        banner.contains("rgba(69, 26, 3") || banner.contains("rgba(69,26,3"),
        ".preview-banner needs the translucent dark amber backdrop"
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
    assert!(
        active.contains("#f59e0b") || active.contains("#fbbf24"),
        "the active preview button must be clearly amber, distinct from hover"
    );
}
