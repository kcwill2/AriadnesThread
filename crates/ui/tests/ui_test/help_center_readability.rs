use ariadne_ui::{INDEX_HTML, STYLES_CSS};

/// Local copy of the css_rule_body idiom (gui_scrollbar_style.rs) so this
/// suite can judge rule bodies without reaching into another test module.
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

/// Region of index markup between two unique markers, opening tag excluded.
fn tab_slice(html: &str, start_marker: &str, end_marker: &str) -> String {
    let marker_at = html.find(start_marker).unwrap_or_else(|| {
        panic!("index.html must contain {start_marker}");
    });
    let gt = html[marker_at..].find('>').unwrap_or_else(|| {
        panic!("unterminated opening tag at {start_marker}");
    });
    let start = marker_at + gt + 1;
    let end = html[start..].find(end_marker).unwrap_or_else(|| {
        panic!("index.html must close the region after {start_marker} with {end_marker}");
    });
    html[start..start + end].to_string()
}

fn strip_tags(fragment: &str) -> String {
    let mut kept = String::new();
    let mut inside_tag = false;
    for ch in fragment.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            other if !inside_tag => kept.push(other),
            _ => {}
        }
    }
    kept.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn next_boundary(region: &str, boundaries: &[&str]) -> Option<(usize, usize)> {
    boundaries
        .iter()
        .filter_map(|marker| region.find(marker).map(|idx| (idx, marker.len())))
        .min_by_key(|(idx, _)| *idx)
}

/// Text blocks delimited by the given closing tags, tags stripped and
/// whitespace collapsed. The readability contract judges each block.
fn prose_blocks(region: &str) -> Vec<String> {
    const BOUNDARIES: [&str; 4] = ["</p>", "</li>", "</h4>", "</h5>"];
    let mut blocks = Vec::new();
    let mut rest = region;
    while let Some((idx, len)) = next_boundary(rest, &BOUNDARIES) {
        blocks.push(strip_tags(&rest[..idx]));
        rest = &rest[idx + len..];
    }
    blocks.push(strip_tags(rest));
    blocks
}

#[test]
fn test_help_body_scrolls() {
    let body = css_rule_body(STYLES_CSS, ".help-body {");
    assert!(
        body.contains("overflow-y: auto"),
        "the help body must scroll its own overflow"
    );
    assert!(
        body.contains("max-height"),
        "the help body must cap its height so tall tabs stop clipping off-viewport"
    );
    let center = css_rule_body(STYLES_CSS, ".help-center {");
    assert!(
        center.contains("max-height"),
        "the help modal must cap the container height"
    );
    assert!(
        INDEX_HTML.contains("class=\"modal help-center\""),
        "the help modal must carry the .help-center class"
    );
    assert!(
        INDEX_HTML.contains("class=\"help-body\""),
        "the help content pane must carry the .help-body class"
    );
}

#[test]
fn test_legend_tab_has_no_wall_of_prose() {
    let legend = tab_slice(INDEX_HTML, "id=\"helpTab-legend\"", "id=\"helpTab-recipes\"");
    let heading_count = legend.matches("<h5").count();
    assert!(
        heading_count >= 5,
        "the legend tab must be split into at least 5 <h5 sections, found {heading_count}"
    );
    let oversized: Vec<String> = prose_blocks(&legend)
        .into_iter()
        .filter(|block| block.chars().count() > 400)
        .collect();
    let first_oversized = oversized.first().map(|block| block.chars().take(90).collect::<String>());
    assert!(
        oversized.is_empty(),
        "legend text blocks must stay under 400 chars, {} oversized, first starts: {first_oversized:?}",
        oversized.len()
    );
}

#[test]
fn test_legend_copy_locks_survive() {
    for lock in [
        "Collateral Heat Map",
        "Cold/Teal",
        "Include Justification breakdown",
        "Preview after extraction",
        "red dashed",
        "0/N used",
        "system/stdlib headers are not analyzed",
        "Path Illumination",
        "beside the open list",
        "Proposed extractions section",
        "ALL-WITH-CAP",
        "Direction",
        "Finding & Decoupling Header Bloat",
        "Isolating Focused Subgraphs",
    ] {
        assert!(
            INDEX_HTML.contains(lock),
            "hard copy lock missing from index markup: {lock}"
        );
    }
}

#[test]
fn test_active_tab_state_is_styled() {
    let active = css_rule_body(STYLES_CSS, ".help-tab-btn.active {");
    assert!(
        active.contains("var(--cyan)"),
        "the active help tab must be visibly marked with the cyan accent token"
    );
}

#[test]
fn test_kbd_and_shortcut_rows_are_styled() {
    assert!(
        STYLES_CSS.contains("kbd {"),
        "kbd elements must carry a visible keycap style"
    );
    assert!(
        STYLES_CSS.contains(".help-shortcut-row"),
        "shortcut rows must have a grid-row class"
    );
    let shortcuts = tab_slice(INDEX_HTML, "id=\"helpTab-shortcuts\"", "id=\"helpTab-legend\"");
    let key_count = shortcuts.matches("<kbd").count();
    assert!(
        key_count >= 12,
        "every shortcut must keep its <kbd> key markup, found {key_count}"
    );
}

#[test]
fn test_recipes_are_step_lists() {
    let recipes = tab_slice(INDEX_HTML, "id=\"helpTab-recipes\"", "quickOpenPanel");
    let ordered_lists = recipes.matches("<ol").count();
    assert!(
        ordered_lists >= 4,
        "each recipe must be an <ol> of steps, found {ordered_lists}"
    );
    for title in [
        "Finding & Decoupling Header Bloat",
        "Triage Worst Offenders",
        "Isolating Focused Subgraphs",
        "Breaking Circular Dependencies",
    ] {
        assert!(
            recipes.contains(title),
            "recipe title must survive verbatim: {title}"
        );
    }
}
