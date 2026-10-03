// Phase 2b (plan 1790656207440 T5): the app JS (main.js, graph.js, preview.js)
// must paint exclusively with themes.css tokens. themes.css loads in <head>
// before any of these scripts run, so var(--token) resolves in SVG
// presentation attributes and inline styles exactly like it does in CSS.
// Phase 3 reconciliation: the three insight-card alphas Phase 2b carved out
// now have by-value tokens (--red-wash-soft, --cyan-border, --cyan-wash-soft),
// so the assertion is strict zero — no exception table, count-verified or
// otherwise. themes.css is the single definition site, full stop.

use crate::common;

const JS_FILES: &[(&str, &str)] = &[
    ("main.js", ariadne_ui::MAIN_JS),
    ("graph.js", ariadne_ui::GRAPH_JS),
    ("preview.js", ariadne_ui::PREVIEW_JS),
    // theme.js themes by attribute only — it must never grow a color literal.
    ("theme.js", ariadne_ui::THEME_JS),
];

fn line_of(src: &str, offset: usize) -> usize {
    src[..offset].matches('\n').count() + 1
}

/// Finds a raw hex color: `#` followed by a maximal run of exactly 3 or 6
/// hex digits. Rejects entities like `&#8250;` (4-digit run) and SVG fragment
/// refs like `url(#arrow-cycle)` (non-hex run).
fn first_hex_literal(src: &str) -> Option<(usize, String)> {
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'#' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_hexdigit() {
                j += 1;
            }
            let run = j - (i + 1);
            if run == 3 || run == 6 {
                return Some((i, src[i..j].to_string()));
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    None
}

#[test]
fn test_app_js_carries_no_raw_hex_colors() {
    for (name, src) in JS_FILES {
        if let Some((offset, literal)) = first_hex_literal(src) {
            panic!(
                "{name}:{} contains raw hex color {literal} — replace it with the \
                 matching themes.css token var(--…) (Phase 2b token contract)",
                line_of(src, offset)
            );
        }
    }
}

#[test]
fn test_app_js_carries_no_rgba_literals() {
    for (name, src) in JS_FILES {
        assert!(
            !src.contains("rgba("),
            "{name} must not contain raw rgba() colors — every color is a \
             themes.css token var(--x) (Phase 3 strict-zero token contract)"
        );
    }
}

fn default_theme_block(themes: &str) -> &str {
    let selector = themes
        .find(":root:not([data-theme])")
        .expect("themes.css must carry the :root:not([data-theme]) default block");
    let open = selector + themes[selector..].find('{').expect("default block must open with {");
    let close = open + themes[open..].find('}').expect("default block must close with }");
    &themes[open + 1..close]
}

fn declared_tokens(block: &str) -> std::collections::HashSet<&str> {
    block
        .lines()
        .filter_map(|line| {
            let (name, _) = line.trim().split_once(':')?;
            let name = name.trim();
            name.starts_with("--").then_some(name)
        })
        .collect()
}

fn used_var_tokens(src: &str) -> Vec<String> {
    let mut used = Vec::new();
    let mut cursor = 0;
    while let Some(at) = src[cursor..].find("var(--") {
        let start = cursor + at + "var(".len();
        let end = start
            + src[start..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(src.len() - start);
        used.push(src[start..end].to_string());
        cursor = end;
    }
    used
}

#[test]
fn test_every_var_reference_in_app_js_is_declared_in_default_theme() {
    let declared = declared_tokens(default_theme_block(ariadne_ui::THEMES_CSS));
    assert!(declared.len() > 50, "default block must carry the full token set");
    for (name, src) in JS_FILES {
        let missing: Vec<String> = used_var_tokens(src)
            .into_iter()
            .filter(|token| !declared.contains(token.as_str()))
            .collect();
        assert!(
            missing.is_empty(),
            "{name} references undeclared tokens {missing:?} — themes.css is the \
             single definition site; every var(--x) in JS must exist in its default block"
        );
    }
}

#[test]
fn test_every_var_reference_in_static_surfaces_is_declared_in_default_theme() {
    // Same declared-vs-used lock, widened to the two static surfaces
    // (styles.css rules and index.html inline styles/markers). Comments are
    // stripped first so prose inside /* … */ or <!-- … --> cannot register
    // as a live reference — the same carve-out the hex sweeps apply.
    let declared = declared_tokens(default_theme_block(ariadne_ui::THEMES_CSS));
    let sources = [
        ("styles.css", common::strip_css_comments(ariadne_ui::STYLES_CSS)),
        (
            "index.html",
            common::strip_html_comments(ariadne_ui::INDEX_HTML),
        ),
    ];
    for (name, src) in sources {
        let missing: Vec<String> = declared_needing_tokens(&src)
            .into_iter()
            .filter(|token| !declared.contains(token.as_str()))
            .collect();
        assert!(
            missing.is_empty(),
            "{name} references undeclared tokens {missing:?} — themes.css is the \
             single definition site; every var(--x) in the static surfaces must \
             exist in its default block"
        );
    }
}

/// var(--…) references that carry no inline fallback, and so can only
/// resolve through a themes.css declaration. `var(--x, fallback)` is exempt:
/// the inline default makes it safe without a declaration — the same escape
/// hatch the literal sweeps accept (common::assert_var_or) — for genuinely
/// non-token variables like the JS-set layout width --source-panel-w in
/// .drawer.open.
fn declared_needing_tokens(src: &str) -> Vec<String> {
    let mut used = Vec::new();
    let mut cursor = 0;
    while let Some(at) = src[cursor..].find("var(--") {
        let start = cursor + at + "var(".len();
        let end = start
            + src[start..]
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
                .unwrap_or(src.len() - start);
        if !src[end..].starts_with(',') {
            used.push(src[start..end].to_string());
        }
        cursor = end;
    }
    used
}
