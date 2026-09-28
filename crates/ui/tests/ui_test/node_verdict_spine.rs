// Node Verdict Spine (plan 1790573049166): dependency cards carry the verdict
// as a left accent spine; fill stays pure collateral heat (flat, every tier,
// justified included); colored tier glows are deleted and glow is reserved
// for selection/root. String-level asset assertions, edge_verdict_story.rs style.

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
fn test_graph_js_builds_spine_and_flat_tiers() {
    let graph = ariadne_ui::GRAPH_JS;
    assert!(
        graph.contains("appendVerdictSpine("),
        "graph.js must build the verdict spine via appendVerdictSpine"
    );
    assert!(
        graph.contains("verdict-spine"),
        "graph.js must emit the verdict-spine class"
    );
    assert!(
        graph.contains("heat-tier-"),
        "graph.js must apply heat-tier-<tier> classes"
    );
    assert!(
        !graph.contains("heat-glow-"),
        "the heat-glow class emission must be renamed away"
    );
    assert!(
        !graph.contains("dep-justified") && !graph.contains("dep-mixed"),
        "verdict outline-override classes must no longer be added"
    );
    assert!(
        graph.contains("verdictBadgeColor("),
        "badge colors must flow through the verdict badge color helper"
    );
}

#[test]
fn test_styles_spine_verdict_colors_and_flat_tiers() {
    let css = ariadne_ui::STYLES_CSS;

    let justified = css_rule_body(css, ".verdict-spine.justified");
    assert!(
        justified.contains("var(--green)") || justified.contains("#4ade80"),
        ".verdict-spine.justified must be green like .edge-path.edge-justified"
    );
    let mixed = css_rule_body(css, ".verdict-spine.mixed");
    assert!(
        mixed.contains("var(--amber)") || mixed.contains("#fb923c"),
        ".verdict-spine.mixed must be amber like .edge-path.edge-leaving"
    );
    let unused = css_rule_body(css, ".verdict-spine.unused");
    assert!(
        unused.contains("#ef4444"),
        ".verdict-spine.unused must be red like .edge-path.edge-unused"
    );

    assert!(
        !css.contains("dep-justified") && !css.contains("dep-mixed"),
        "no dep-justified/dep-mixed selector may remain in styles.css"
    );

    for tier in 0..=4 {
        let body = css_rule_body(css, &format!(".node-group.heat-tier-{tier} rect"));
        assert!(
            body.contains("stroke-width"),
            ".node-group.heat-tier-{tier} rect must thicken the tier stroke"
        );
        assert!(
            !body.contains("filter"),
            ".node-group.heat-tier-{tier} rect must stay flat (glow reserved for selection/root)"
        );
    }

    let hide = css_rule_body(css, ".preview-hypothetical .verdict-spine");
    assert!(
        hide.contains("display: none"),
        "preview-after-extraction must hide the verdict spines"
    );
}

#[test]
fn test_legend_teaches_card_spine_vocabulary() {
    let main = ariadne_ui::MAIN_JS;
    assert!(
        main.contains("legend-card"),
        "legend must gain card-shaped spine swatches"
    );
    assert!(
        main.contains("spine-justified")
            && main.contains("spine-mixed")
            && main.contains("spine-unused"),
        "legend card swatches must cover the three spine verdicts"
    );
    assert!(
        main.contains("Card: every symbol used \u{2014} needed"),
        "legend must teach the needed card"
    );
    assert!(
        main.contains("Card: partly needed"),
        "legend must teach the partly-needed card"
    );
    assert!(
        main.contains("Card: supplies nothing \u{2014} this include can go"),
        "legend must teach the unused card"
    );
    assert!(
        main.contains("No spine \u{2014} no verdict data"),
        "legend must teach the no-spine case"
    );
    assert!(
        main.contains("Card fill = collateral weight"),
        "legend must title the heat pill block"
    );
    assert!(
        main.contains("greenRow + amberRow + redRow + contextRow"),
        "existing arrow-row gating/order contract must be preserved"
    );
    assert!(
        main.contains("verdictCardLegendRows(hasJustifiedVerdict, hasUnusedVerdict)"),
        "card rows must be gated on the same payload tiers as the arrow rows"
    );

    let css = ariadne_ui::STYLES_CSS;
    assert!(
        css.contains(".legend-card") && css.contains(".legend-card.spine-unused"),
        "styles.css must define the legend card swatch and its spine variants"
    );
}

#[test]
fn test_help_and_tour_copy_teach_spine() {
    let html = ariadne_ui::INDEX_HTML;
    assert!(
        html.contains("green spine")
            && html.contains("amber spine")
            && html.contains("red spine"),
        "help legend must name the spine verdict colors"
    );
    assert!(
        !html.contains("green-outlined"),
        "the stale green-outlined claim must be gone"
    );

    let main = ariadne_ui::MAIN_JS;
    assert!(
        main.contains("left spine"),
        "tour step copy must point at the spine channel"
    );
}

#[test]
fn test_verdict_tooltips_name_spine_colors() {
    let main = ariadne_ui::MAIN_JS;
    assert!(
        main.contains("green spine"),
        "justified tooltip must name the green spine"
    );
    assert!(
        main.contains("amber spine"),
        "mixed tooltip must name the amber spine"
    );
    assert!(
        main.contains("red spine"),
        "unused tooltip must name the red spine"
    );
    assert!(
        main.contains("droppable or slimmable"),
        "unused tooltip must keep the droppable/slimmable guidance"
    );
    assert!(
        main.contains("system/stdlib headers are not analyzed"),
        "unused tooltip must keep the system/stdlib scope caveat"
    );
}
