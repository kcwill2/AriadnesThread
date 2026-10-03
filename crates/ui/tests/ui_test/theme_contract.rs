// Theme token contract (plan 1790656207440 T2/T3): themes.css is the single
// definition site, it is linked/served/inlined BEFORE styles.css everywhere,
// and the standalone export carries the tokens because file:// has no routes.
// Phase 3 (T6/T7/T8) added here: the theme.js registry, the toolbar select,
// the anti-flash bootstrap, and the 3-way (N-way) token parity test.

use crate::common;
use crate::common::{strip_css_comments, strip_html_comments};

#[test]
fn test_theme_stylesheet_is_linked_before_rule_stylesheet() {
    // Tokens must be defined before the rules that consume them; a swapped
    // link order is invisible in the default theme and breaks themed loads.
    let html = ariadne_ui::INDEX_HTML;
    let themes = html
        .find("<link rel=\"stylesheet\" href=\"/themes.css\">")
        .expect("index.html must link themes.css");
    let styles = html
        .find("<link rel=\"stylesheet\" href=\"/styles.css\">")
        .expect("index.html must link styles.css");
    assert!(
        themes < styles,
        "themes.css (tokens) must be linked before styles.css (rules)"
    );
}

#[test]
fn test_token_definitions_live_only_in_themes_css() {
    // styles.css consumed to 1422 lines carrying its own :root; the move to
    // themes.css must not leave a second definition site behind, or a theme
    // switch silently keeps the old slate values.
    assert!(
        !ariadne_ui::STYLES_CSS.contains(":root"),
        "styles.css must no longer define :root tokens (single definition site)"
    );
    let themes = ariadne_ui::THEMES_CSS;
    let default_block = themes
        .find(":root:not([data-theme])")
        .expect("themes.css must carry a :root:not([data-theme]) default block");
    let light = themes
        .find("[data-theme=\"light\"]")
        .expect("themes.css must carry a light theme block");
    let steampunk = themes
        .find("[data-theme=\"dark-steampunk\"]")
        .expect("themes.css must carry a dark-steampunk theme block");
    assert!(
        default_block < light && light < steampunk,
        "theme blocks must be authored default-first, then each [data-theme] override"
    );
}

#[tokio::test]
async fn test_server_serves_theme_tokens() {
    // Same no-store asset idiom as /styles.css; the route must ship the real
    // token contract, not just any CSS.
    let app = ariadne_ui::build_router(common::loading_state());
    let (status, headers, body) = common::request_raw_body(app, "/themes.css").await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(
        headers.get(axum::http::header::CONTENT_TYPE).unwrap(),
        "text/css; charset=utf-8"
    );
    assert_eq!(
        headers.get(axum::http::header::CACHE_CONTROL).unwrap(),
        "no-store"
    );
    assert_eq!(body, ariadne_ui::THEMES_CSS.as_bytes());
}

#[tokio::test]
async fn test_server_serves_theme_script_asset() {
    // theme.js joins the no-store asset contract (same idiom as the CSS
    // route): a cached theme bootstrap can pin a stale registry after an
    // upgrade, so the route must serve the exact bundled bytes uncached.
    let app = ariadne_ui::build_router(common::loading_state());
    let (status, headers, body) = common::request_raw_body(app, "/app/theme.js").await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(
        headers.get(axum::http::header::CONTENT_TYPE).unwrap(),
        "text/javascript; charset=utf-8"
    );
    assert_eq!(
        headers.get(axum::http::header::CACHE_CONTROL).unwrap(),
        "no-store"
    );
    assert_eq!(body, ariadne_ui::THEME_JS.as_bytes());
}

#[test]
fn test_standalone_export_inlines_theme_tokens() {
    // file:// has no /themes.css: without inlining, the exported report boots
    // with zero token definitions and no error. Reuses the export pattern
    // from static_html.rs; themes-only tokens prove the theme block arrived,
    // and the ordering check proves tokens still precede the rules.
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("standalone.html");
    ariadne_ui::export_standalone_html(&common::empty_export_payload(), "", &out_path).unwrap();

    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(
        content.contains("--heat-4-text:"),
        "export must inline the theme token contract: file:// has no /themes.css"
    );
    assert!(
        content.contains("[data-theme=\"dark-steampunk\"]"),
        "export must inline all three theme blocks, not just the default"
    );
    assert!(
        !content.contains("href=\"/themes.css\""),
        "export must not reference the external themes.css route"
    );
    let tokens_at = content
        .find("--heat-4-text:")
        .expect("theme tokens must precede the rules that use them");
    let rules_at = content
        .find("scrollbar-width")
        .expect("styles.css rules must still be present in the export");
    assert!(
        tokens_at < rules_at,
        "inlined themes.css must still precede inlined styles.css in the export"
    );
}

/// Find a CSS hex color: '#' followed by exactly 3/4/6/8 hex digits (the legal
/// CSS color lengths), skipping HTML numeric character references — a '#' right
/// after '&' is the `&#128293;` entity syntax, never a color. This is the only
/// carve-out the sweep needs: every '#' + hexdigits run in index.html that is
/// not a color is an entity, and id selectors never open with 3+ hex digits.
fn first_hex_color(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    for i in 0..chars.len() {
        if chars[i] != '#' || (i > 0 && chars[i - 1] == '&') {
            continue;
        }
        let mut end = i + 1;
        while end < chars.len() && chars[end].is_ascii_hexdigit() {
            end += 1;
        }
        if matches!(end - (i + 1), 3 | 4 | 6 | 8) {
            return Some(chars[i..end].iter().collect());
        }
    }
    None
}

#[test]
fn test_styles_css_carries_no_color_literals() {
    // Phase 2a (T4) completion gate: themes.css is the ONLY definition site,
    // so a hex or rgba()/rgb() literal surviving in styles.css means a rule
    // that will not repaint on theme switch. Comments are exempt (they may
    // quote historical values); live declarations are not.
    let css = strip_css_comments(ariadne_ui::STYLES_CSS);
    assert!(
        !css.contains("rgba(") && !css.contains("rgb("),
        "styles.css must not declare rgb()/rgba() literals outside comments (themes.css owns every color)"
    );
    if let Some(found) = first_hex_color(&css) {
        panic!("styles.css must not declare hex colors outside comments, found {found}");
    }
}

#[test]
fn test_index_html_styling_carries_no_color_literals() {
    // Same gate for the styling surface of index.html (SVG marker fills and
    // inline styles): these render before any JS runs, so a literal there is
    // a first-paint theme leak. HTML comments are exempt; numeric character
    // references (&#128293;) are entities, not colors.
    let html = strip_html_comments(ariadne_ui::INDEX_HTML);
    assert!(
        !html.contains("rgba(") && !html.contains("rgb("),
        "index.html styles/markers must not declare rgb()/rgba() literals (themes.css owns every color)"
    );
    if let Some(found) = first_hex_color(&html) {
        panic!("index.html styles/markers must not declare hex colors, found {found}");
    }
}

/// Parse `const <NAME> = ['a', 'b', …]` from theme.js. The array is the
/// single source of truth every other site (select options, CSS blocks,
/// the inline bootstrap) must mirror — so the tests parse it instead of
/// hardcoding a fourth copy of the ids.
fn parse_js_string_array(js: &str, name: &str) -> Vec<String> {
    let marker = format!("const {name} = [");
    let at = js.find(&marker).unwrap_or_else(|| {
        panic!("theme.js must declare `const {name} = [...]` as the single source of truth")
    });
    let rest = &js[at + marker.len()..];
    let close = rest
        .find(']')
        .unwrap_or_else(|| panic!("`const {name}` array must close"));
    rest[..close]
        .split(',')
        .map(|item| item.trim().trim_matches('\'').trim_matches('"').to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn theme_registry() -> Vec<String> {
    let themes = parse_js_string_array(ariadne_ui::THEME_JS, "THEMES");
    assert!(!themes.is_empty(), "THEMES must list at least one theme");
    themes
}

/// Body text (inside the braces) of the first block whose selector contains
/// `selector`. CSS comments must already be stripped by the caller: a
/// commented-out selector would otherwise hijack the search.
fn css_block_body<'a>(css: &'a str, selector: &str) -> &'a str {
    let at = css
        .find(selector)
        .unwrap_or_else(|| panic!("themes.css must contain a block for {selector}"));
    let open = at + css[at..].find('{').expect("block must open with {");
    let close = open + css[open..].find('}').expect("block must close with }");
    &css[open + 1..close]
}

fn declared_tokens(block: &str) -> std::collections::HashSet<String> {
    block
        .lines()
        .filter_map(|line| {
            let (name, _) = line.trim().split_once(':')?;
            let name = name.trim();
            name.starts_with("--").then(|| name.to_string())
        })
        .collect()
}

/// All theme ids appearing in a `[data-theme="…"]` selector attribute match.
/// Comment-stripped CSS: the header prose mentions the selector form and a
/// commented fake id must never register as a real block.
fn data_theme_ids(css: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut cursor = 0;
    while let Some(at) = css[cursor..].find("[data-theme=\"") {
        let start = cursor + at + "[data-theme=\"".len();
        let end = start
            + css[start..]
                .find('"')
                .expect("data-theme value must close its quote");
        ids.push(css[start..end].to_string());
        cursor = end;
    }
    ids
}

#[test]
fn test_theme_select_options_mirror_the_theme_registry() {
    // T7: the #themeSelect <option value>s must equal the THEMES array in
    // order — human labels may drift, the ids may not, or a registry entry
    // silently becomes unselectable.
    let themes = theme_registry();
    let html = ariadne_ui::INDEX_HTML;
    let select_at = html
        .find("id=\"themeSelect\"")
        .expect("the toolbar must carry #themeSelect");
    let select_open = html[..select_at]
        .rfind("<select")
        .expect("#themeSelect must sit on a <select> element");
    let select_close = select_at
        + html[select_at..]
            .find("</select>")
            .expect("#themeSelect must close");
    let region = &html[select_open..select_close];
    let mut values = Vec::new();
    let mut cursor = 0;
    while let Some(at) = region[cursor..].find("<option value=\"") {
        let start = cursor + at + "<option value=\"".len();
        let end = start
            + region[start..]
                .find('"')
                .expect("option value must close its quote");
        values.push(region[start..end].to_string());
        cursor = end;
    }
    assert_eq!(
        values, themes,
        "#themeSelect option values must equal the THEMES registry in order"
    );
    assert!(
        region.contains("<option value=\"dark\" selected>"),
        "dark must carry the selected attribute so static markup and the \
         default palette agree on first paint"
    );
}

#[test]
fn test_css_theme_ids_and_registry_agree_both_ways() {
    // T8 guard against half-added themes: every [data-theme="…"] block in
    // themes.css must be selectable via THEMES, and every THEMES id must
    // have a palette block — a block with no option is dead CSS, an option
    // with no block renders undefined tokens.
    let registry: std::collections::HashSet<String> = theme_registry().into_iter().collect();
    let blocks: std::collections::HashSet<String> =
        data_theme_ids(&strip_css_comments(ariadne_ui::THEMES_CSS))
            .into_iter()
            .collect();
    assert_eq!(
        blocks, registry,
        "themes.css [data-theme] ids and the theme.js THEMES registry must be the same set"
    );
}

/// All theme ids the anti-flash bootstrap compares the stored value against,
/// parsed from the inline <head> script located via the ariadne_theme storage
/// key. The snippet runs before any asset loads, so it cannot import THEMES
/// and hand-mirrors the accepted ids in `!== '…'` comparisons — this parses
/// those literals instead of hardcoding a fourth copy.
fn bootstrap_theme_ids(html: &str) -> Vec<String> {
    let marker = html
        .find("ariadne_theme")
        .expect("index.html head must carry the anti-flash theme bootstrap (storage key)");
    let script_open = html[..marker]
        .rfind("<script>")
        .expect("the bootstrap storage key must sit inside the inline <head> script");
    let script_close = marker
        + html[marker..]
            .find("</script>")
            .expect("the bootstrap script must close");
    let script = &html[script_open..script_close];
    let mut ids = Vec::new();
    let mut cursor = 0;
    while let Some(at) = script[cursor..].find("!== '") {
        let start = cursor + at + "!== '".len();
        let end = start
            + script[start..]
                .find('\'')
                .expect("a bootstrap id comparison must close its quote");
        ids.push(script[start..end].to_string());
        cursor = end;
    }
    assert!(
        !ids.is_empty(),
        "the bootstrap must validate the stored value against quoted theme ids"
    );
    ids
}

#[test]
fn test_bootstrap_theme_ids_mirror_the_theme_registry() {
    // The bootstrap is a deliberate duplicate of theme.js's validation (see
    // its header comment): an id accepted there but not here silently falls
    // back to the default on first paint, and vice versa yields a data-theme
    // matching no themes.css block — so the id sets must agree.
    let registry: std::collections::HashSet<String> = theme_registry().into_iter().collect();
    let bootstrap: std::collections::HashSet<String> =
        bootstrap_theme_ids(ariadne_ui::INDEX_HTML).into_iter().collect();
    assert_eq!(
        bootstrap, registry,
        "the anti-flash bootstrap's accepted ids must equal the theme.js THEMES registry"
    );
}

#[test]
fn test_every_theme_block_declares_the_full_token_set() {
    // T8 3-way parity: iterate the blocks found in themes.css (not a
    // hardcoded two-theme list, so a fourth theme is forced into parity too)
    // and require its declared token names to equal the default block's.
    // The parity test enforces themes.css stays a complete per-theme contract,
    // which is what lets the JS sweep keep strict-zero color literals.
    let css = strip_css_comments(ariadne_ui::THEMES_CSS);
    let default_tokens = declared_tokens(css_block_body(&css, ":root:not([data-theme])"));
    assert!(
        default_tokens.len() > 50,
        "the default block must carry the full token contract"
    );
    let mut incomplete = Vec::new();
    for id in data_theme_ids(&css) {
        let tokens = declared_tokens(css_block_body(&css, &format!("[data-theme=\"{id}\"]")));
        if tokens != default_tokens {
            incomplete.push(id);
        }
    }
    assert!(
        incomplete.is_empty(),
        "theme blocks missing/adding tokens vs the default contract: {incomplete:?} \
         — every theme must define every token (3-way parity)"
    );
}

#[test]
fn test_export_inlines_theme_script_and_bootstrap_precedes_tokens() {
    // Two halves of the T6 wiring: (1) file:// has no /app/theme.js route, so
    // the export must carry the bootstrap code itself — a missed export.rs
    // tuple entry loses the theme control silently; (2) the anti-flash inline
    // snippet must run BEFORE the themes.css link, or the themed CSS paints
    // once under the default palette before the stored theme applies.
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("standalone.html");
    ariadne_ui::export_standalone_html(&common::empty_export_payload(), "", &out_path).unwrap();
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(
        content.contains(ariadne_ui::THEME_JS.trim_end()),
        "the standalone export must inline theme.js verbatim: file:// has no /app/theme.js"
    );
    assert!(
        !content.contains("src=\"/app/theme.js\""),
        "the export must not reference the external theme.js route"
    );
    let html = ariadne_ui::INDEX_HTML;
    let bootstrap = html
        .find("ariadne_theme")
        .expect("index.html head must carry the anti-flash theme bootstrap (storage key)");
    let tokens_link = html
        .find("href=\"/themes.css\"")
        .expect("themes.css must be linked");
    assert!(
        bootstrap < tokens_link,
        "the anti-flash bootstrap must run before the themed CSS paints"
    );
}

/// Moved verbatim from static_html.rs when the three-theme merge pushed that
/// file past the R2 1000-line ceiling. The test's contract is now the theme
/// system's: the bootstrap is the one sanctioned inline <script> body besides
/// the data placeholder, and theme.js is one of the asset scripts below.
#[test]
fn test_index_markup_has_no_inline_style_or_script_bodies() {
    let html = ariadne_ui::INDEX_HTML;
    assert!(
        !html.contains("<style"),
        "markup must not inline CSS: it belongs in /styles.css"
    );
    assert!(
        !html.contains("function "),
        "no JS function bodies may remain inline in index markup"
    );

    // Inspect every <script> tag: src-tagged assets are fine; the only inline
    // bodies allowed are the data placeholder consumed by the export and the
    // anti-flash theme bootstrap in <head> (it cannot be an external asset:
    // it must run synchronously before the themed CSS paints).
    let mut rest = html;
    let mut inline_script_count = 0;
    while let Some(open) = rest.find("<script") {
        let after_open = &rest[open..];
        let tag_end = after_open.find('>').expect("unterminated <script> tag");
        let close = after_open
            .find("</script>")
            .expect("unterminated <script> body");
        let tag = &after_open[..tag_end];
        if tag.contains("src=") {
            assert!(
                tag_start_is_asset(tag),
                "unexpected script tag with src: {tag}"
            );
        } else {
            inline_script_count += 1;
            let body = after_open[tag_end + 1..close].trim();
            assert!(
                body == "/*__ARIADNE_DATA__*/" || body.contains("ariadne_theme"),
                "inline script bodies are limited to the data placeholder and \
                 the anti-flash theme bootstrap (identified by its storage key)"
            );
        }
        rest = &after_open[close + "</script>".len()..];
    }
    assert_eq!(
        inline_script_count, 2,
        "exactly two inline script bodies: data placeholder + theme bootstrap"
    );
    assert_eq!(
        html.matches("<script src=").count(),
        8,
        "exactly eight classic asset scripts"
    );
}

fn tag_start_is_asset(tag: &str) -> bool {
    [
        "/app/api.js",
        "/app/graph.js",
        "/app/file-view.js",
        "/app/hotspots.js",
        "/app/preview.js",
        "/app/provenance.js",
        "/app/main.js",
        "/app/theme.js",
    ]
    .iter()
    .any(|src| tag.contains(src))
}

