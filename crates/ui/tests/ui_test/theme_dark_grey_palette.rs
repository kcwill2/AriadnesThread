// Dark-grey palette oracles (shape 1791105717628, invariants I5-I8): the
// fourth theme's mirror order, semantic-layer byte-equality with the default
// block, achromatic neutral chrome, and the repo's first numeric WCAG
// contrast floors (grey-only to start). Shared parsing helpers and the WCAG
// math come from crate::common (T-A review rider findings 2-3); the
// NEUTRALITY scanners stay here on purpose: hex_channel_runs SKIPS
// malformed runs and treats 4/8-digit alphas as exempt, and
// rgba_channel_runs ignores alpha - the achromatic bound must judge live
// declarations leniently, the opposite contract from the extraction
// oracle's panic-on-malformed first_color_run, so the two parsers stay
// per-module.

use crate::common::{
    channels_scaled, contrast_ratio, css_block_body, declared_value, nibble_pair,
    strip_css_comments, theme_registry,
};

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

/// WCAG luminance/contrast math is the shared canonical one in
/// crate::common (rider finding 2); 8-bit colors reach it through
/// channels_scaled, which reproduces the byte/255.0 scaling the removed
/// local copy applied, so every floor outcome is unchanged.

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
        let ratio = contrast_ratio(
            channels_scaled(text_main),
            channels_scaled(first_hex(grey, background)),
        );
        assert!(
            ratio >= 7.0,
            "--text-main on {background} is {ratio:.2}:1, below the 7:1 floor"
        );
    }
    let muted_ratio = contrast_ratio(
        channels_scaled(first_hex(grey, "--text-muted")),
        channels_scaled(first_hex(grey, "--bg-main")),
    );
    assert!(
        muted_ratio >= 4.5,
        "--text-muted on --bg-main is {muted_ratio:.2}:1, below the 4.5:1 floor"
    );
    let sub_ratio = contrast_ratio(
        channels_scaled(first_hex(grey, "--text-sub")),
        channels_scaled(first_hex(grey, "--bg-main")),
    );
    assert!(
        sub_ratio >= 3.0,
        "--text-sub on --bg-main is {sub_ratio:.2}:1, below the 3:1 floor"
    );
}
