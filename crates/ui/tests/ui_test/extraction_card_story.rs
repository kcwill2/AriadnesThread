//! Story locks for the Proposed Extractions card (milestone
//! ui-extraction-card-story). The shared builder in hotspots.js must tell a
//! five-beat story top to bottom: why -> what moves -> what it costs -> who
//! is affected -> preview. These are source-level locks on the builder
//! markup, its CSS tier rules, and the Hotspots help copy, following the
//! house string-assert idiom of static_html.rs / help_center_readability.rs.

use ariadne_ui::{EXTRACTION_WIZARD_JS, HOTSPOTS_JS, INDEX_HTML, STYLES_CSS};

/// Local copy of the css_rule_body idiom (help_center_readability.rs) so this
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

/// Region of index markup between two unique markers, opening tag excluded
/// (local copy of the tab_slice idiom from help_center_readability.rs).
fn tab_slice(html: &str, start_marker: &str, end_marker: &str) -> String {
    let marker_at = html.find(start_marker).unwrap_or_else(|| {
        panic!("index.html must contain {start_marker}");
    });
    let gt = html[marker_at..]
        .find('>')
        .unwrap_or_else(|| panic!("unterminated opening tag at {start_marker}"));
    let start = marker_at + gt + 1;
    let end = html[start..].find(end_marker).unwrap_or_else(|| {
        panic!("index.html must close the region after {start_marker} with {end_marker}");
    });
    html[start..start + end].to_string()
}

#[test]
fn test_why_beat_visible_in_body() {
    // S1: the only why-text moves from the hover tooltip into the card body.
    assert!(
        HOTSPOTS_JS.contains("extraction-problem"),
        "the why beat must render an .extraction-problem body line"
    );
    assert!(
        HOTSPOTS_JS.contains("escapeHtml(p.problem"),
        "the problem sentence must reach the body escaped"
    );
    assert!(
        !HOTSPOTS_JS.contains("title=\"${escapeHtml(p.problem"),
        "the problem sentence must no longer hide in the card title tooltip"
    );
}

#[test]
fn test_plan_id_identity_chip() {
    // S2: the payload's stable identity becomes visible as a header chip.
    assert!(
        HOTSPOTS_JS.contains("extraction-plan-id"),
        "the head beat must render an .extraction-plan-id identity chip"
    );
    assert!(
        HOTSPOTS_JS.contains("escapeHtml(p.plan_id"),
        "the plan id must reach the chip escaped"
    );
}

#[test]
fn test_saving_computed_with_zero_guard() {
    // S3: the saving is derived display math behind a named helper, guarded
    // so before == 0 never divides and a non-shrinking load never shows a
    // negative percent.
    assert!(
        HOTSPOTS_JS.contains("extractionSavingText"),
        "the saving must be computed by the named extractionSavingText helper"
    );
    assert!(
        HOTSPOTS_JS.contains("before > 0"),
        "the saving must be guarded on a positive before-load"
    );
    assert!(
        HOTSPOTS_JS.contains("Math.round"),
        "the saving percent must be rounded to a whole percent"
    );
    assert!(
        HOTSPOTS_JS.contains("100 / before"),
        "the percent must divide only by the guarded before value"
    );
}

#[test]
fn test_no_runon_join_blobs() {
    // S4: names and headers render as chips, never as comma run-ons. The
    // no-argument join('') used for markup assembly is a different substring.
    assert!(
        !HOTSPOTS_JS.contains(".join(', ')"),
        "hotspots.js must not rebuild comma blob lists; chips replace them"
    );
}

#[test]
fn test_names_and_headers_render_as_chips() {
    // S4/S10 (helper half): one escaping chips helper feeds both chip rows.
    assert!(
        HOTSPOTS_JS.contains("extractionChips"),
        "the card needs the shared extractionChips helper"
    );
    assert!(
        HOTSPOTS_JS.contains("extraction-chip"),
        "moved symbols and required headers must render as .extraction-chip spans"
    );
    assert!(
        HOTSPOTS_JS.contains("map(escapeHtml"),
        "every chip label must pass through escapeHtml"
    );
}

#[test]
fn test_risk_tier_badge_sanitized() {
    // S5: the tier class suffix is sanitizer-derived (unknown values degrade
    // to the neutral base class), and CSS gives each tier a real color.
    assert!(
        HOTSPOTS_JS.contains("extraction-risk"),
        "the risk badge must carry the .extraction-risk class pair"
    );
    assert!(
        HOTSPOTS_JS.contains("toLowerCase") && HOTSPOTS_JS.contains("[^a-z]"),
        "the tier class suffix must be lowercased and filtered to [a-z] only"
    );
    let conservative = css_rule_body(STYLES_CSS, ".extraction-risk-conservative {");
    assert!(
        conservative.contains("var(--green)"),
        "the conservative tier must read green"
    );
    let moderate = css_rule_body(STYLES_CSS, ".extraction-risk-moderate {");
    assert!(
        moderate.contains("var(--amber)"),
        "the moderate tier must read amber"
    );
    let aggressive = css_rule_body(STYLES_CSS, ".extraction-risk-aggressive {");
    assert!(
        aggressive.contains("var(--red)"),
        "the aggressive tier must read red"
    );
    let base = css_rule_body(STYLES_CSS, ".extraction-risk {");
    // C5 amendment 2 (shape C5, inherited from clarity T-B): the base rule
    // painted the card surface with --text-sub, measured 2.97:1 in dark at
    // HEAD 97dd836 - below the UI-ink floor. The swap to --text-muted moves
    // the degraded tier onto a token the four-theme contrast oracle locks at
    // >= 4.5:1 on every ground (extraction_contrast.rs, unedited).
    assert!(
        base.contains("var(--text-muted)"),
        "unknown tiers must degrade to the neutral base rule on the --text-muted token (C5: 2.97:1 in dark was too low)"
    );
}

#[test]
fn test_order_and_preview_locks() {
    // S6: payload order is rendered as-is; the UI never re-sorts proposals.
    // This lock is green before and after the rewrite (no-regression guard).
    assert!(
        HOTSPOTS_JS.contains("proposals.map(renderExtractionProposalCard)"),
        "the section must map payload order straight into the shared builder"
    );
    assert!(
        !HOTSPOTS_JS.contains("extraction_proposals.sort"),
        "the UI must never re-sort extraction proposals"
    );
}

#[test]
fn test_beats_appear_in_story_order() {
    // C5 amendment 1 (shape C5, inherited from clarity T-B): the narrative
    // these five beats carried moved into the wizard, so the beat list
    // becomes the wizard-step order. The lock survives unchanged in KIND -
    // an order lock over the asset that tells the story - now over the S0..S6
    // step renderers in shape C1 order. The card builder itself is untouched.
    let beats = [
        "function renderWizardStepOverview(",
        "function renderWizardStepFiles(",
        "function renderWizardStepManifest(",
        "function renderWizardStepImpact(",
        "function renderWizardStepDryRun(",
        "function renderWizardStepExecute(",
        "function renderWizardStepAfterApply(",
    ];
    let mut previous_beat: Option<(&str, usize)> = None;
    for beat in beats {
        let beat_at = EXTRACTION_WIZARD_JS
            .find(beat)
            .unwrap_or_else(|| panic!("wizard asset must define the {beat} step renderer in step order"));
        if let Some((earlier_beat, earlier_at)) = previous_beat {
            assert!(
                earlier_at < beat_at,
                "step renderer {beat} must appear after {earlier_beat} in the wizard-step order (shape C1)"
            );
        }
        previous_beat = Some((beat, beat_at));
    }
}

#[test]
fn test_tu_sentence_and_help_copy() {
    // C6/S11 additions: the TU count reads as a re-validation sentence on the
    // card, and the legend tab defines collateral and re-validation in
    // reader terms. Char caps and copy locks stay with the existing suite.
    assert!(
        HOTSPOTS_JS.contains("consumer TU(s) need re-validation"),
        "the consumer TU count must read as a re-validation sentence"
    );
    let legend = tab_slice(INDEX_HTML, "id=\"helpTab-legend\"", "id=\"helpTab-recipes\"");
    assert!(
        legend.contains("symbols every consumer drags"),
        "the help tab must define collateral as the symbols every consumer drags along"
    );
    assert!(
        legend.contains("re-validation"),
        "the help tab must explain the consumer TU count as re-validation"
    );
}
