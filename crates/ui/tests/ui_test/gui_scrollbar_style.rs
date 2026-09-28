use ariadne_ui::STYLES_CSS;

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
fn test_scrollbars_are_themed() {
    // Native light scrollbars on the dark theme are the user-visible defect:
    // every scroll region (panels, lists, drawer, thread bar) must carry
    // themed thin scrollbars instead of the platform default.
    for marker in [
        "scrollbar-width",
        "scrollbar-color",
        "::-webkit-scrollbar",
        "::-webkit-scrollbar-thumb:hover",
    ] {
        assert!(
            STYLES_CSS.contains(marker),
            "styles.css must define themed scrollbars ({marker}) so scroll regions match the dark chrome"
        );
    }
}

#[test]
fn test_scrollbar_thumb_uses_theme_tokens() {
    // The thumb must read on every panel surface, so it is token-driven
    // (border-light at rest, text-sub on hover) rather than a raw hex.
    let thumb = css_rule_body(STYLES_CSS, "::-webkit-scrollbar-thumb {");
    assert!(
        thumb.contains("background: var(--border-light)"),
        "the scrollbar thumb must use var(--border-light) so it reads on every panel surface"
    );

    let hover = css_rule_body(STYLES_CSS, "::-webkit-scrollbar-thumb:hover {");
    assert!(
        hover.contains("var(--text-sub)"),
        "the hovered scrollbar thumb must brighten to var(--text-sub)"
    );

    let firefox = css_rule_body(STYLES_CSS, "* {");
    assert!(
        firefox.contains("scrollbar-width: thin")
            && firefox.contains("scrollbar-color: var(--border-light) transparent"),
        "Firefox scrollbars must be thin with a transparent track via scrollbar-width/scrollbar-color"
    );
}

#[test]
fn test_source_drawer_uses_slate_theme_tokens() {
    // The drawer carried a hardcoded GitHub palette that clashed with the
    // slate theme; it must paint theme tokens like every other panel.
    assert!(
        !STYLES_CSS.contains("#0d1117") && !STYLES_CSS.contains("#161b22"),
        "styles.css must not hardcode the GitHub drawer palette (#0d1117 / #161b22) against the slate theme"
    );

    let drawer = css_rule_body(STYLES_CSS, ".drawer {");
    assert!(
        drawer.contains("background: var(--bg-canvas)"),
        ".drawer must paint the app canvas surface"
    );

    let header = css_rule_body(STYLES_CSS, ".drawer-header {");
    assert!(
        header.contains("background: var(--bg-sub)"),
        ".drawer-header must paint the sub-toolbar surface like the toolbar"
    );

    let body = css_rule_body(STYLES_CSS, ".drawer-body {");
    assert!(
        body.contains("background: var(--bg-canvas)"),
        ".drawer-body must paint the app canvas surface"
    );
}

#[test]
fn test_select_dark_matches_search_input_focus_treatment() {
    // Form controls must behave consistently: the layout select needs the same
    // hover/focus affordance the search input already has.
    let select = css_rule_body(STYLES_CSS, ".select-dark {");
    assert!(
        select.contains("transition: border-color 0.15s ease"),
        ".select-dark must transition border-color like .search-input"
    );

    let hover = css_rule_body(STYLES_CSS, ".select-dark:hover {");
    assert!(
        hover.contains("border-color: var(--border-light)"),
        ".select-dark:hover must lighten its border like other controls"
    );

    let focus = css_rule_body(STYLES_CSS, ".select-dark:focus {");
    assert!(
        focus.contains("border-color: var(--cyan)")
            && focus.contains("box-shadow: 0 0 0 1px var(--cyan)"),
        ".select-dark:focus must mirror the cyan ring of .search-input:focus"
    );
}

#[test]
fn test_graph_chrome_hex_rules_untouched_by_token_sweep() {
    // The token sweep is GUI-chrome-only: the graph legend card and the SVG
    // expand badge keep their literal hexes (they render on the canvas).
    let legend_card = css_rule_body(STYLES_CSS, ".legend-card {");
    assert!(
        legend_card.contains("background: #1e293b"),
        ".legend-card must keep its literal canvas chip background; graph rules are out of the token sweep"
    );

    let badge = css_rule_body(STYLES_CSS, ".badge-expand {");
    assert!(
        badge.contains("fill: #334155"),
        ".badge-expand must keep its literal SVG fill; graph rules are out of the token sweep"
    );
}
