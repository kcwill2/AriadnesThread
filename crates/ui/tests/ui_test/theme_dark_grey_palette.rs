// Dark-grey palette oracles (shape 1791105717628, invariants I5-I8): the
// fourth theme's mirror order, semantic-layer byte-equality with the default
// block, achromatic neutral chrome, and the repo's first numeric WCAG
// contrast floors (grey-only to start). The private parsing helpers are
// COPIED from theme_contract.rs on purpose: that file is held byte-identical
// apart from its sanctioned export-assertion extension (handoff touch-set).

use crate::common::strip_css_comments;

const GREY_SELECTOR: &str = "[data-theme=\"dark-grey\"]";
const DEFAULT_SELECTOR: &str = ":root:not([data-theme])";

/// I6: these hues ARE the information (verdicts, heat ramp, cycle red,
/// preview verdicts) - the grey theme retints the table, not the food, so
/// every value must stay byte-equal to the default block's declaration.
const SEMANTIC_TOKENS: &[&str] = &[
    "--green", "--amber", "--red", "--green-glow", "--red-wash",
    "--red-wash-soft", "--red-wash-hover", "--red-glow", "--red-glow-strong",
    "--red-text", "--amber-wash", "--amber-border", "--amber-glow",
    "--amber-text", "--heat-ramp-a", "--heat-0-bg", "--heat-0-border",
    "--heat-0-text", "--heat-1-bg", "--heat-1-border", "--heat-1-text",
    "--heat-2-bg", "--heat-2-border", "--heat-2-text", "--heat-3-bg",
    "--heat-3-border", "--heat-3-text", "--heat-4-bg", "--heat-4-border",
    "--heat-4-text", "--node-cycle-bg", "--node-cycle-text",
    "--node-cycle-border", "--preview-eliminated-bg",
    "--preview-eliminated-text", "--preview-eliminated-glow",
    "--preview-kept-bg", "--preview-kept-text", "--preview-kept-glow",
    "--preview-still-bg", "--preview-still-text", "--preview-amber-edge",
    "--preview-amber-text", "--preview-amber-glow", "--preview-amber-outline",
];

/// I7: the chrome layer is the only thing this theme may retint, and it must
/// stay achromatic. Decorative accents (--cyan*, --brand*, --primary-btn*,
/// --tag-pill*, --tour-glow) and node-kind / proposed identity tokens are
/// exempt: they are allowed to carry hue.
const CHROME_TOKENS: &[&str] = &[
    "--bg-main", "--bg-canvas", "--bg-surface", "--bg-sub", "--bg-card",
    "--bg-card-hover", "--border", "--border-light", "--text-main",
    "--text-muted", "--text-sub", "--text-strong", "--on-accent",
    "--grid-dot", "--shadow-soft", "--shadow-node", "--shadow-ambient",
    "--shadow-lift", "--overlay-scrim", "--tour-scrim", "--overlay-panel-bg",
    "--tile-bg", "--progress-track", "--edge-neutral", "--edge-incomplete",
    "--code-text",
];

/// Parse `const <NAME> = ['a', 'b', ...]` from theme.js. Copied from
/// theme_contract.rs: the registry is parsed, never hardcoded, so this file
/// cannot drift from the surface it locks.
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
/// `selector`. Copied from theme_contract.rs; comments must already be
/// stripped by the caller so a commented fake selector cannot hijack the
/// search.
fn css_block_body<'a>(css: &'a str, selector: &str) -> &'a str {
    let at = css
        .find(selector)
        .unwrap_or_else(|| panic!("themes.css must contain a block for {selector}"));
    let open = at + css[at..].find('{').expect("block must open with {");
    let close = open + css[open..].find('}').expect("block must close with }");
    &css[open + 1..close]
}

/// The declared value of `token` in a block body, trimmed of whitespace and
/// the trailing semicolon: the byte string compared by the I6 oracle.
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

/// Every '#' run of exactly 3/4/6/8 hex digits in a value, reduced to its
/// r,g,b channels (short forms expand; 4/8-digit alphas are trailing and
/// exempt from the neutrality bound).
fn hex_channel_runs(value: &str) -> Vec<(u8, u8, u8)> {
    let chars: Vec<char> = value.chars().collect();
    let mut runs = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != '#' {
            index += 1;
            continue;
        }
        let mut end = index + 1;
        while end < chars.len() && chars[end].is_ascii_hexdigit() {
            end += 1;
        }
        let digits = &chars[index + 1..end];
        let channels = match digits.len() {
            3 | 4 => (
                nibble_pair(digits[0], digits[0]),
                nibble_pair(digits[1], digits[1]),
                nibble_pair(digits[2], digits[2]),
            ),
            6 | 8 => (
                nibble_pair(digits[0], digits[1]),
                nibble_pair(digits[2], digits[3]),
                nibble_pair(digits[4], digits[5]),
            ),
            _ => {
                index = end;
                continue;
            }
        };
        runs.push(channels);
        index = end;
    }
    runs
}

/// Every `rgba(...)` run in a value, reduced to its r,g,b channels (the
/// alpha channel is exempt from the neutrality bound).
fn rgba_channel_runs(value: &str) -> Vec<(i64, i64, i64)> {
    let mut runs = Vec::new();
    let mut search_from = 0;
    while let Some(at) = value[search_from..].find("rgba(") {
        let open = search_from + at + "rgba(".len();
        let close = open + value[open..].find(')').expect("rgba( must close with )");
        let mut channels = value[open..close].split(',').map(str::trim);
        let red: i64 = channels.next().expect("rgba needs r").parse().expect("r must be numeric");
        let green: i64 = channels.next().expect("rgba needs g").parse().expect("g must be numeric");
        let blue: i64 = channels.next().expect("rgba needs b").parse().expect("b must be numeric");
        runs.push((red, green, blue));
        search_from = close;
    }
    runs
}

/// WCAG relative luminance: linearize each sRGB channel, then weight.
fn relative_luminance(channels: (u8, u8, u8)) -> f64 {
    let linearize = |byte: u8| -> f64 {
        let scaled = byte as f64 / 255.0;
        if scaled <= 0.04045 {
            scaled / 12.92
        } else {
            ((scaled + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linearize(channels.0) + 0.7152 * linearize(channels.1) + 0.0722 * linearize(channels.2)
}

/// WCAG contrast ratio between two opaque colors, lighter first by
/// construction so callers may pass either order.
fn contrast_ratio(first: (u8, u8, u8), second: (u8, u8, u8)) -> f64 {
    let first_lum = relative_luminance(first);
    let second_lum = relative_luminance(second);
    let (brighter, dimmer) = if first_lum >= second_lum {
        (first_lum, second_lum)
    } else {
        (second_lum, first_lum)
    };
    (brighter + 0.05) / (dimmer + 0.05)
}

/// The first hex color a token declares (all contrast-relevant tokens are
/// opaque hexes, never rgba).
fn first_hex(block: &str, token: &str) -> (u8, u8, u8) {
    hex_channel_runs(&declared_value(block, token))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{token} must declare an opaque hex color"))
}

#[test]
fn test_dark_grey_block_sits_in_mirror_order() {
    // I5: source order mirrors the registry (light, dark-grey, steampunk
    // adjacent to dark), so reading themes.css tells the same story as the
    // dropdown - the order the user confirmed (Q4).
    let css = strip_css_comments(ariadne_ui::THEMES_CSS);
    let offset_of = |selector: &str| -> usize {
        css.find(selector)
            .unwrap_or_else(|| panic!("themes.css must contain a block for {selector}"))
    };
    let light = offset_of("[data-theme=\"light\"]");
    let grey = offset_of(GREY_SELECTOR);
    let steampunk = offset_of("[data-theme=\"dark-steampunk\"]");
    assert!(
        light < grey && grey < steampunk,
        "the dark-grey block must sit between light and dark-steampunk: \
         light {light} < grey {grey} < steampunk {steampunk}"
    );
    let registry = theme_registry();
    let third = registry.get(2).expect("THEMES must list at least three ids");
    assert_eq!(
        third, "dark-grey",
        "the registry must list dark-grey third, adjacent to dark"
    );
}

#[test]
fn test_dark_grey_semantic_layer_equals_default_block() {
    // I6: byte-string compare of every semantically locked token against
    // the default block (trimmed value). Stronger than steampunk's
    // guarantee, which is a header comment only. Byte compare is exact:
    // default writes 0.1 where steampunk writes 0.10 - grey must copy
    // default verbatim.
    let css = strip_css_comments(ariadne_ui::THEMES_CSS);
    let grey = css_block_body(&css, GREY_SELECTOR);
    let default = css_block_body(&css, DEFAULT_SELECTOR);
    let mut mismatched = Vec::new();
    for token in SEMANTIC_TOKENS {
        let grey_value = declared_value(grey, token);
        let default_value = declared_value(default, token);
        if grey_value != default_value {
            mismatched.push(format!("{token}: grey \"{grey_value}\" vs default \"{default_value}\""));
        }
    }
    assert!(
        mismatched.is_empty(),
        "the dark-grey semantic layer must be byte-equal to the default block \
         (verdicts/heat/cycle/preview hues are information): {mismatched:?}"
    );
}

#[test]
fn test_dark_grey_chrome_is_achromatic() {
    // I7: the whole point of dark-grey is a NEUTRAL dark chrome; a hex may
    // drift at most 4 units channel-to-channel and an rgba must be grey by
    // construction, or we have re-shipped slate.
    let css = strip_css_comments(ariadne_ui::THEMES_CSS);
    let grey = css_block_body(&css, GREY_SELECTOR);
    for token in CHROME_TOKENS {
        let value = declared_value(grey, token);
        for (red, green, blue) in hex_channel_runs(&value) {
            let spread =
                red.max(green).max(blue) - red.min(green).min(blue);
            assert!(
                spread <= 4,
                "{token} value \"{value}\" is not achromatic: hex channel spread {spread} > 4"
            );
        }
        for (red, green, blue) in rgba_channel_runs(&value) {
            assert!(
                red == green && green == blue,
                "{token} value \"{value}\" is not achromatic: rgba channels differ"
            );
        }
    }
    // Distinguishability corollary (characterization lock): the DEFAULT
    // block's --bg-main carries the slate tint this theme exists to differ
    // from - blue channel at least 8 above red (today 0x17 - 0x0b = 12).
    let default = css_block_body(&css, DEFAULT_SELECTOR);
    let (slate_red, _slate_green, slate_blue) = first_hex(default, "--bg-main");
    assert!(
        slate_blue as i16 - slate_red as i16 >= 8,
        "the default --bg-main must stay slate-tinted (b - r >= 8), found \
         b={slate_blue} r={slate_red}: this characterization backs the grey theme's reason to exist"
    );
}

#[test]
fn test_dark_grey_contrast_floors() {
    // I8: first numeric legibility lock in the repo. Computed from the
    // declared hexes with WCAG relative luminance, floors 7:1 for main
    // text on every dark surface, 4.5:1 muted, 3:1 sub.
    let css = strip_css_comments(ariadne_ui::THEMES_CSS);
    let grey = css_block_body(&css, GREY_SELECTOR);
    let text_main = first_hex(grey, "--text-main");
    for background in ["--bg-main", "--bg-canvas", "--bg-surface", "--bg-card"] {
        let ratio = contrast_ratio(text_main, first_hex(grey, background));
        assert!(
            ratio >= 7.0,
            "--text-main on {background} is {ratio:.2}:1, below the 7:1 floor"
        );
    }
    let muted_ratio = contrast_ratio(first_hex(grey, "--text-muted"), first_hex(grey, "--bg-main"));
    assert!(
        muted_ratio >= 4.5,
        "--text-muted on --bg-main is {muted_ratio:.2}:1, below the 4.5:1 floor"
    );
    let sub_ratio = contrast_ratio(first_hex(grey, "--text-sub"), first_hex(grey, "--bg-main"));
    assert!(
        sub_ratio >= 3.0,
        "--text-sub on --bg-main is {sub_ratio:.2}:1, below the 3:1 floor"
    );
}
