// Extraction-section contrast oracle (milestone ui-extraction-card-clarity,
// hand T-A, shape contract C4): WCAG floors on the token pairs the
// extraction cards paint, evaluated for EVERY theme in the theme.js
// registry. I13: the pair table loops the parsed registry, so a fifth
// theme is forced into the same rows automatically and a registry id
// without a themes.css block panics here (missing row = test bug by
// construction). The private parsing helpers are COPIED from
// theme_dark_grey_palette.rs per house idiom (no imports across test
// modules); only strip_css_comments is shared via crate::common.
//
// COMPOSITE CONVENTION (C4): a translucent token declaration composites
// over --bg-surface (the hotspot panel the cards sit on) per channel as
// `a * color + (1 - a) * underlay`; opaque declarations pass through
// unchanged. Every token in the pair table is opaque hex today - the
// composite arm is the documented fallback for a future translucent
// declaration, never a silent rescue for a current one.

use crate::common::strip_css_comments;

/// One C4 pair row: `token` must clear `floor` against every surface in
/// `against`. WCAG contrast ratio is symmetric, so the mass/border
/// separation rows share this shape with the ink rows.
struct ContrastRow {
    group: &'static str,
    token: &'static str,
    against: &'static [&'static str],
    floor: f64,
}

/// The C4 pair table verbatim (nine rows): body ink 7.0, muted ink 4.5
/// on all four grounds, off-card UI ink 3.0, verdict hues 4.5 on the
/// card, surface separation 1.12, border separation 1.35 / 1.50.
const CONTRAST_ROWS: &[ContrastRow] = &[
    ContrastRow {
        group: "body",
        token: "--text-main",
        against: &["--bg-card", "--bg-sub"],
        floor: 7.0,
    },
    ContrastRow {
        group: "body",
        token: "--text-muted",
        against: &["--bg-card", "--bg-sub", "--bg-surface", "--bg-main"],
        floor: 4.5,
    },
    ContrastRow {
        group: "offcard",
        token: "--text-sub",
        against: &["--bg-main"],
        floor: 3.0,
    },
    ContrastRow {
        group: "verdict",
        token: "--amber",
        against: &["--bg-card"],
        floor: 4.5,
    },
    ContrastRow {
        group: "verdict",
        token: "--green",
        against: &["--bg-card"],
        floor: 4.5,
    },
    ContrastRow {
        group: "verdict",
        token: "--red-text",
        against: &["--bg-card"],
        floor: 4.5,
    },
    ContrastRow {
        group: "mass",
        token: "--bg-card",
        against: &["--bg-surface"],
        floor: 1.12,
    },
    ContrastRow {
        group: "border",
        token: "--border-light",
        against: &["--bg-card"],
        floor: 1.35,
    },
    ContrastRow {
        group: "border",
        token: "--border-light",
        against: &["--bg-surface"],
        floor: 1.50,
    },
];

/// Parse `const <NAME> = ['a', 'b', ...]` from theme.js. Copied from
/// theme_dark_grey_palette.rs: the registry is parsed, never hardcoded,
/// so this oracle cannot drift from the surface it locks.
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

/// The themes.css selector that carries a registry id's block.
fn theme_selector(registry_id: &str) -> String {
    format!("[data-theme=\"{registry_id}\"]")
}

/// Body text (inside the braces) of the first block whose selector
/// contains `selector`. Copied from theme_dark_grey_palette.rs; comments
/// must already be stripped by the caller so a commented fake selector
/// cannot hijack the search.
fn css_block_body<'a>(css: &'a str, selector: &str) -> &'a str {
    let at = css
        .find(selector)
        .unwrap_or_else(|| panic!("themes.css must contain a block for {selector}"));
    let open = at + css[at..].find('{').expect("block must open with {");
    let close = open + css[open..].find('}').expect("block must close with }");
    &css[open + 1..close]
}

/// The declared value of `token` in a block body, trimmed of whitespace
/// and the trailing semicolon. Copied from theme_dark_grey_palette.rs.
fn declared_value(block: &str, token: &str) -> String {
    for line in block.lines() {
        if let Some((name, value)) = line.trim().split_once(':') {
            if name.trim() == token {
                return value.trim().trim_end_matches(';').trim().to_string();
            }
        }
    }
    panic!("token {token} must be declared in the theme block")
}

fn nibble_pair(high: char, low: char) -> u8 {
    let high_digit = high.to_digit(16).expect("first hex digit");
    let low_digit = low.to_digit(16).expect("second hex digit");
    (high_digit * 16 + low_digit) as u8
}

/// One rgba channel divided by its scale (255 for rgb, 1 for alpha).
fn parse_rgba_channel(part: Option<&str>, name: &str, scale: f64) -> f64 {
    let raw = part.unwrap_or_else(|| panic!("rgba() must declare a {name} channel"));
    raw.parse::<f64>()
        .unwrap_or_else(|_| panic!("rgba {name} channel \"{raw}\" must be numeric"))
        / scale
}

/// `#...` at the start of `run` as (r, g, b, alpha) channels scaled
/// 0..1. Short forms expand; 4/8-digit forms carry a trailing 8-bit
/// alpha, mirroring the grey-palette helper's reading of hex runs.
fn parse_hex_run(run: &str) -> (f64, f64, f64, f64) {
    let digits: Vec<char> = run[1..].chars().take_while(|c| c.is_ascii_hexdigit()).collect();
    let byte = |high: char, low: char| nibble_pair(high, low) as f64 / 255.0;
    let expanded = |nibble: char| byte(nibble, nibble);
    match digits.len() {
        3 => (
            expanded(digits[0]),
            expanded(digits[1]),
            expanded(digits[2]),
            1.0,
        ),
        4 => (
            expanded(digits[0]),
            expanded(digits[1]),
            expanded(digits[2]),
            expanded(digits[3]),
        ),
        6 => (
            byte(digits[0], digits[1]),
            byte(digits[2], digits[3]),
            byte(digits[4], digits[5]),
            1.0,
        ),
        8 => (
            byte(digits[0], digits[1]),
            byte(digits[2], digits[3]),
            byte(digits[4], digits[5]),
            byte(digits[6], digits[7]),
        ),
        _ => panic!("hex run \"{run}\" must have 3, 4, 6 or 8 hex digits"),
    }
}

/// `rgba(r, g, b, a)` at the start of `run`, channels scaled to 0..1.
fn parse_rgba_run(run: &str) -> (f64, f64, f64, f64) {
    let open = "rgba(".len();
    let close = open + run[open..].find(')').expect("rgba( must close with )");
    let mut parts = run[open..close].split(',').map(str::trim);
    let red = parse_rgba_channel(parts.next(), "red", 255.0);
    let green = parse_rgba_channel(parts.next(), "green", 255.0);
    let blue = parse_rgba_channel(parts.next(), "blue", 255.0);
    let alpha = parse_rgba_channel(parts.next(), "alpha", 1.0);
    (red, green, blue, alpha)
}

/// The first color declared in a token value as (r, g, b, alpha) scaled
/// 0..1 - the earliest of a `#` hex run or an `rgba(` run.
fn first_color_run(value: &str) -> (f64, f64, f64, f64) {
    let hex_at = value.find('#');
    let rgba_at = value.find("rgba(");
    let use_rgba = match (hex_at, rgba_at) {
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (Some(hex), Some(rgba)) => rgba < hex,
        (None, None) => panic!("token value \"{value}\" must declare a hex or rgba color"),
    };
    if use_rgba {
        parse_rgba_run(&value[rgba_at.unwrap()..])
    } else {
        parse_hex_run(&value[hex_at.unwrap()..])
    }
}

/// The declared token color composited over `underlay` per the C4
/// convention (doc comment above): opaque passes through, translucent
/// blends per channel `a * color + (1 - a) * underlay`.
fn effective_color(block: &str, token: &str, underlay: (f64, f64, f64)) -> (f64, f64, f64) {
    let (red, green, blue, alpha) = first_color_run(&declared_value(block, token));
    if alpha >= 1.0 {
        return (red, green, blue);
    }
    (
        alpha * red + (1.0 - alpha) * underlay.0,
        alpha * green + (1.0 - alpha) * underlay.1,
        alpha * blue + (1.0 - alpha) * underlay.2,
    )
}

/// An underlay must itself be opaque - a translucent underlay of itself
/// is undefined. --bg-surface is opaque in every shipped theme.
fn opaque_color(block: &str, token: &str) -> (f64, f64, f64) {
    let (red, green, blue, alpha) = first_color_run(&declared_value(block, token));
    assert!(
        alpha >= 1.0,
        "underlay token {token} must be opaque, found alpha {alpha}"
    );
    (red, green, blue)
}

/// WCAG relative luminance over channels scaled 0..1: linearize each
/// sRGB channel, then weight (same formula as the grey-palette oracle).
fn relative_luminance(color: (f64, f64, f64)) -> f64 {
    let linearize = |channel: f64| -> f64 {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linearize(color.0) + 0.7152 * linearize(color.1) + 0.0722 * linearize(color.2)
}

/// WCAG contrast ratio between two colors, lighter first by
/// construction so callers may pass either order.
fn contrast_ratio(first: (f64, f64, f64), second: (f64, f64, f64)) -> f64 {
    let first_lum = relative_luminance(first);
    let second_lum = relative_luminance(second);
    let (brighter, dimmer) = if first_lum >= second_lum {
        (first_lum, second_lum)
    } else {
        (second_lum, first_lum)
    };
    (brighter + 0.05) / (dimmer + 0.05)
}

/// Evaluate every pair row of `group` across the whole registry; each
/// failure names theme, pair, measured ratio, and floor.
fn assert_group_floors(group: &str) {
    let css = strip_css_comments(ariadne_ui::THEMES_CSS);
    let registry = theme_registry();
    let mut failures: Vec<String> = Vec::new();
    for theme_id in &registry {
        let block = css_block_body(&css, &theme_selector(theme_id));
        let underlay = opaque_color(block, "--bg-surface");
        for row in CONTRAST_ROWS.iter().filter(|row| row.group == group) {
            let subject = effective_color(block, row.token, underlay);
            for against in row.against {
                let background = effective_color(block, against, underlay);
                let measured = contrast_ratio(subject, background);
                if measured < row.floor {
                    failures.push(format!(
                        "{theme_id}: {} vs {against} measured {measured:.2}:1, below the {:.2} floor",
                        row.token, row.floor
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "[{group}] contrast floors violated:\n{}",
        failures.join("\n")
    );
}

#[test]
fn test_registry_ids_all_have_theme_blocks() {
    // I13: every registry id must carry a themes.css block; the copied
    // css_block_body PANICS on a missing selector, so a theme added to
    // theme.js without a block fails loudly here, not silently skipped.
    let css = strip_css_comments(ariadne_ui::THEMES_CSS);
    for theme_id in theme_registry() {
        let block = css_block_body(&css, &theme_selector(&theme_id));
        assert!(
            !block.trim().is_empty(),
            "themes.css block for {theme_id} must declare tokens"
        );
    }
}

#[test]
fn test_body_ink_floors() {
    // I2: main ink >= 7.0 and muted ink >= 4.5 on every card-class
    // ground, in every theme.
    assert_group_floors("body");
}

#[test]
fn test_offcard_ink_floor() {
    // C4 off-card row: --text-sub stays a legible UI ink on --bg-main.
    assert_group_floors("offcard");
}

#[test]
fn test_verdict_hues_on_card() {
    // Verdict hues are information: --amber/--green/--red-text must
    // clear 4.5 on the card surface in every theme (D6 allows value
    // deepening for contrast only - never a floor weakening).
    assert_group_floors("verdict");
}

#[test]
fn test_mass_separation() {
    // I4: card must not collapse into the panel surface (the "one gray
    // mass" falsifier row, floor 1.12).
    assert_group_floors("mass");
}

#[test]
fn test_border_separation() {
    // I4: the border token must read as a seam on both the card and the
    // panel (floors 1.35 / 1.50).
    assert_group_floors("border");
}

#[test]
fn test_pair_table_loops_parsed_registry() {
    // I13 / C4: the nine rows are applied per PARSED registry id. A
    // fifth theme joins automatically; a hardcoded theme list would
    // show up as a count mismatch on this line.
    assert_eq!(
        CONTRAST_ROWS.len(),
        9,
        "the C4 contract defines exactly nine pair rows"
    );
    let css = strip_css_comments(ariadne_ui::THEMES_CSS);
    let registry = theme_registry();
    let mut evaluated_rows = 0usize;
    for theme_id in &registry {
        let block = css_block_body(&css, &theme_selector(theme_id));
        let underlay = opaque_color(block, "--bg-surface");
        for row in CONTRAST_ROWS {
            let _subject_declared = effective_color(block, row.token, underlay);
            evaluated_rows += 1;
        }
    }
    assert_eq!(
        evaluated_rows,
        registry.len() * CONTRAST_ROWS.len(),
        "every pair row must be evaluated for every registry id \
         (table rows {} x registry {})",
        CONTRAST_ROWS.len(),
        registry.len()
    );
}
