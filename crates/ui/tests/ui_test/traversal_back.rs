//! Return-to-origin locks for the traversal panel Back control (change
//! track ui-traversal-back-button, handoff T1). The traversal head carries a
//! Back button, not a Close button: the markup contract replaces
//! `traversal-close`/`closeTraversal` entirely (I1), both traversal entry
//! points capture the origin tab through the shared opening function so Back
//! lands where the user came from (I2), and the restyled rule stays
//! token-only with zero hex (I6). Source-level string locks in the house
//! idiom of hotspot_direction.rs / static_html_sections.rs; the pre-existing
//! ariadne-ui suite is the untouched oracle for every runtime path.

use ariadne_ui::{HOTSPOTS_JS, INDEX_HTML, STYLES_CSS, TRAVERSAL_JS};

/// Body of a named CSS rule, from its opening brace to the first closing
/// brace (local copy of the house css_rule_body idiom so this suite can
/// judge rule bodies without reaching into another test module).
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
fn test_back_control_markup() {
    // I1: the traversal head ships a Back control and the close contract is
    // dead in every asset that carried it.
    assert!(
        INDEX_HTML.contains("class=\"traversal-back\""),
        "I1: the traversal head must carry the traversal-back control class"
    );
    assert!(
        INDEX_HTML.contains("onclick=\"backFromTraversal()\""),
        "I1: the Back control must route clicks to backFromTraversal()"
    );
    assert!(
        INDEX_HTML.contains("title=\"Back to"),
        "I1: the Back control tooltip must describe returning, not closing"
    );
    assert!(
        !INDEX_HTML.contains("traversal-close"),
        "I1: INDEX_HTML must not keep the traversal-close class"
    );
    assert!(
        !INDEX_HTML.contains("closeTraversal"),
        "I1: INDEX_HTML must not keep any closeTraversal reference"
    );
    assert!(
        !TRAVERSAL_JS.contains("closeTraversal"),
        "I1: traversal.js must not keep the closeTraversal contract"
    );
    assert!(
        !STYLES_CSS.contains(".traversal-close"),
        "I1: styles.css must not keep a .traversal-close rule"
    );
}

#[test]
fn test_return_capture_wiring() {
    // I2: the origin tab is captured at traversal entry (guarded so a
    // traversal-to-traversal re-entry never overwrites it) and consumed by
    // Back; exactly the two shipped entry points route through the capture.
    assert!(
        TRAVERSAL_JS.contains("let traversalReturnTab = 'include'"),
        "I2: traversal.js must default the return tab to 'include'"
    );
    assert!(
        TRAVERSAL_JS.contains("function openTraversalTab()"),
        "I2: traversal.js must define the capturing entry function"
    );
    assert!(
        TRAVERSAL_JS.contains("currentTab !== 'traversal'"),
        "I2: the capture must be guarded against overwriting on re-entry"
    );
    assert!(
        TRAVERSAL_JS.contains("function backFromTraversal()"),
        "I2: traversal.js must define the Back handler"
    );
    assert!(
        TRAVERSAL_JS.contains("switchTab(traversalReturnTab)"),
        "I2: Back must switch to the captured origin tab"
    );
    assert!(
        INDEX_HTML.contains("onclick=\"openTraversalTab()\""),
        "I2: the #tabTraversal button must enter traversal via openTraversalTab()"
    );
    assert!(
        !INDEX_HTML.contains("switchTab('traversal')"),
        "I2: INDEX_HTML must not enter traversal with a bare switchTab"
    );
    assert!(
        HOTSPOTS_JS.contains("openTraversalTab()"),
        "I2: the hotspot-card entry must go through openTraversalTab()"
    );
    assert!(
        !HOTSPOTS_JS.contains("switchTab('traversal')"),
        "I2: hotspots.js must not enter traversal with a bare switchTab"
    );
}

#[test]
fn test_back_css_token_only() {
    // I6: the renamed rule resolves through existing var() tokens only -
    // no hex literal may appear in its body (theme safety across all
    // themes). Mirrors the step_row_css_block token-only pattern.
    assert!(
        STYLES_CSS.contains(".traversal-back"),
        "I6: styles.css must define the .traversal-back rule"
    );
    let rule_body = css_rule_body(STYLES_CSS, ".traversal-back");
    assert!(
        rule_body.contains("var(--"),
        "I6: the .traversal-back rule body must declare theme tokens"
    );
    assert!(
        !rule_body.contains('#'),
        "I6: the .traversal-back rule body must stay free of hex literals"
    );
    assert!(
        !STYLES_CSS.contains(".traversal-close"),
        "I6: the .traversal-close rule must be fully renamed"
    );
}
