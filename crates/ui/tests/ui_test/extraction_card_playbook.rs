//! Playbook locks for the Proposed Extractions card (milestone
//! ui-extraction-card-playbook, hand T-C). The shared builder in hotspots.js
//! must present a collapsed summary with a keyboard-accessible disclosure,
//! and the detail must carry the honest destination-file manifest plus the
//! exact apply recipe. Source-level string locks following the house idiom of
//! static_html.rs / help_center_readability.rs; the CSS rule bodies reuse the
//! local css_rule_body and tab_slice copies.

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

/// Body of a named top-level hotspots.js function, delimited by its opening
/// brace and the house closing-brace line. Lets each lock judge exactly the
/// helper it names instead of the whole file.
fn function_body(js: &str, name: &str) -> String {
    let head = format!("function {name}(");
    let at = js
        .find(&head)
        .unwrap_or_else(|| panic!("hotspots.js must define {head}"));
    let open = js[at..]
        .find('{')
        .unwrap_or_else(|| panic!("unterminated body for {name}"));
    let close = js[at + open..]
        .find("\n    }")
        .unwrap_or_else(|| panic!("unterminated body block for {name}"));
    js[at + open + 1..at + open + close].to_string()
}

#[test]
fn test_details_collapsed_by_default() {
    // P4: the mechanism of "collapsed" is the hidden attribute in the detail
    // template plus an aria-expanded=false initial button state.
    assert!(
        HOTSPOTS_JS.contains("class=\"extraction-details\" hidden>"),
        "P4: the detail section must ship collapsed via the hidden attribute"
    );
    assert!(
        HOTSPOTS_JS.contains("aria-expanded=\"false\""),
        "P4: the disclosure button must start collapsed (aria-expanded false)"
    );
}

#[test]
fn test_disclosure_button_contract() {
    // P5/C5: a real button, dispatched through the named toggle, DOM-walking
    // within the enclosing card (no element ids: the card renders in two
    // surfaces at once), stopping propagation so the card click still selects.
    assert!(
        HOTSPOTS_JS.contains("<button type=\"button\" class=\"extraction-disclosure\""),
        "P5: the toggle must be a real type=button carrying .extraction-disclosure"
    );
    assert!(
        HOTSPOTS_JS.contains("toggleExtractionCard(event, this)"),
        "P5: the button must dispatch through toggleExtractionCard(event, this)"
    );
    let toggle = function_body(HOTSPOTS_JS, "toggleExtractionCard");
    assert!(
        toggle.contains("stopPropagation"),
        "P5: the toggle must stopPropagation so the card click still selects the hotspot"
    );
    assert!(
        toggle.contains("hidden = !"),
        "P5: the toggle must flip the hidden property of the details"
    );
    assert!(
        toggle.contains("aria-expanded"),
        "P5: the toggle must mirror the disclosure state into aria-expanded"
    );
    assert!(
        toggle.contains("closest") && toggle.contains("querySelector"),
        "P5/C5: the toggle must locate the details by DOM walk inside the same card"
    );
    assert!(
        !toggle.contains("getElementById"),
        "P5/C5: no element ids; the toggle must walk the DOM instead"
    );
}

#[test]
fn test_summary_meta_and_risk_visible() {
    // Shape C3: the always-visible summary carries the symbol count, the same
    // guarded saving text the cost beat uses, and the risk badge.
    let summary = function_body(HOTSPOTS_JS, "extractionSummaryMeta");
    assert!(
        summary.contains("extraction-summary-meta"),
        "C3: the summary must carry the .extraction-summary-meta count"
    );
    assert!(
        summary.contains(" symbols</span>"),
        "C3: the summary count must read as a symbol count"
    );
    assert!(
        summary.contains("extractionSavingText"),
        "C3: the summary saving must be reused from the guarded extractionSavingText helper"
    );
    assert!(
        HOTSPOTS_JS.contains("extraction-summary-saving"),
        "C3: the summary saving must render inside .extraction-summary-saving"
    );
    assert!(
        HOTSPOTS_JS.contains("${renderExtractionRiskTag(p.risk)}"),
        "C3: the summary must carry the renderExtractionRiskTag badge"
    );
}

#[test]
fn test_file_plan_rows_markup() {
    // P1/C3: one row per suggested destination file with its symbol manifest,
    // an honest truncation marker, and an honest fallback for legacy payloads.
    let plan = function_body(HOTSPOTS_JS, "renderExtractionFilePlan");
    assert!(
        plan.contains("Files to create"),
        "C3: the manifest rows must be introduced by a Files to create label"
    );
    // C5 amendment 3 (shape C5): the wizard restructure owns the full
    // destination-file block now (its .extraction-file-block in step S2);
    // the card keeps its row but the block selector moved with the block
    // into the wizard asset. The two selectors cannot coexist as the single
    // source of truth for the file block, so the expectation moves with it.
    assert!(
        EXTRACTION_WIZARD_JS.contains("class=\"extraction-file-block\""),
        "C3 amended (C5): the destination-file block moved into the wizard asset and renders .extraction-file-block"
    );
    assert!(
        plan.contains("class=\"extraction-file-path\""),
        "C3: the real destination path must render in .extraction-file-path"
    );
    assert!(
        plan.contains("more files"),
        "P1: a truncated file list must honestly mark that more files exist"
    );
    assert!(
        plan.contains("<em>no file plan in payload</em>"),
        "P1: legacy payloads must render the honest fallback, never a fabricated path"
    );
    assert!(
        HOTSPOTS_JS.contains("p.target_files_truncated"),
        "P1: the builder must read the truncation flag from the payload"
    );
}

#[test]
fn test_apply_recipe_is_honest() {
    // P9: exactly the two real CLI verbs with the positional <repo-root>
    // placeholder and the plan id interpolated; no other command is named.
    // C5 amendment 5 (shape C5): the verb list grows by tool transform
    // extract ONLY - it is a real CLI verb (crates/cli/src/lib.rs) and the
    // wizard's S5 recipe names it per W5 - and the "UI never applies"
    // sentence is reworded to name the follow-on boundary (kickoff frozen
    // lock: the help-copy flip belongs to extraction-auto-apply).
    let hint = function_body(HOTSPOTS_JS, "renderExtractionApplyHint");
    assert!(
        hint.contains("tool apply &lt;repo-root&gt; --plan ${"),
        "P9: the card must show 'tool apply <repo-root> --plan <plan_id>' with the plan id interpolated"
    );
    assert!(
        hint.contains("tool migrate &lt;repo-root&gt; --plan ${"),
        "P9: the note must show 'tool migrate <repo-root> --plan <plan_id>' with the plan id interpolated"
    );
    assert!(
        hint.contains("--preview"),
        "P9: the migrate dry run must keep its --preview flag"
    );
    assert!(
        hint.contains("Validates and rolls back"),
        "P9: the note must state the validate/rollback guarantee honestly"
    );
    assert!(
        EXTRACTION_WIZARD_JS.contains("tool transform extract &lt;repo-root&gt; --plan "),
        "P9 amended (C5): the wizard recipe names the third real verb 'tool transform extract <repo-root> --plan <plan_id>' (W5)"
    );
    let named_verbs = HOTSPOTS_JS.matches("tool ").count()
        + EXTRACTION_WIZARD_JS.matches("tool ").count();
    let honest_verbs = HOTSPOTS_JS.matches("tool apply").count()
        + HOTSPOTS_JS.matches("tool migrate").count()
        + EXTRACTION_WIZARD_JS.matches("tool apply").count()
        + EXTRACTION_WIZARD_JS.matches("tool migrate").count()
        + EXTRACTION_WIZARD_JS.matches("tool transform extract").count();
    assert_eq!(
        named_verbs, honest_verbs,
        "P9 amended: only tool apply, tool migrate and tool transform extract may be named as commands"
    );
    let legend = tab_slice(INDEX_HTML, "id=\"helpTab-legend\"", "id=\"helpTab-recipes\"");
    // C8 amendment 1 (extraction-auto-apply A-5): the sentence this lock
    // pinned is the copy the milestone deletes; the claim of record
    // replaces it, mechanically opposed to the mandated copy flip.
    assert!(
        legend.contains("the UI applies with a dry-run first, validation in three tiers and automatic rollback"),
        "A-5 C8 amendment 1: the reworded sentence must state the UI applies with a dry-run first, validation in three tiers and automatic rollback"
    );
    assert!(
        legend.contains("extraction-auto-apply"),
        "C5 amended: the reworded never-applies sentence must name the follow-on boundary extraction-auto-apply"
    );
}

#[test]
fn test_playbook_escape_discipline() {
    // P10: every new interpolation escapes; paths inline, symbol names via
    // the shared escaping chips helper, the plan id before reuse.
    let plan = function_body(HOTSPOTS_JS, "renderExtractionFilePlan");
    assert!(
        plan.contains("escapeHtml(t.path"),
        "P10: every destination path must pass through escapeHtml"
    );
    assert!(
        plan.contains("extractionChips(t.symbol_names)"),
        "P10: symbol names may reach the row only through the escaping extractionChips helper"
    );
    let hint = function_body(HOTSPOTS_JS, "renderExtractionApplyHint");
    assert!(
        hint.contains("escapeHtml(planId"),
        "P10: the plan id interpolated into the commands must be escaped"
    );
}

#[test]
fn test_playbook_styles_declared_tokens() {
    // C6: the new playbook rules take every color from the existing
    // three-theme token set; belt over the hex drift gate.
    // C5 amendment 4 (shape C5): the selector list is extended with the
    // wizard rules - the playbook's visual language continued into the
    // modal stepper (W-3a shell + W-3b blocks), and those rules carry the
    // same token discipline.
    for selector in [
        ".extraction-disclosure {",
        ".extraction-details {",
        ".extraction-file-path {",
        ".extraction-command {",
        ".extraction-apply-note {",
        ".extraction-file-block {",
        ".wizard-file-row {",
        ".wizard-file-path {",
        ".wizard-rewrite-row {",
    ] {
        let body = css_rule_body(STYLES_CSS, selector);
        assert!(
            body.contains("var(--"),
            "C6: {selector} must take its colors from theme tokens"
        );
        assert!(
            !body.contains('#'),
            "C6: {selector} must carry no raw hex literal"
        );
    }
}

#[test]
fn test_no_new_data_sources() {
    // P8: the renderer reads payload fields only; green before and after the
    // rewrite as a no-regression guard.
    assert!(
        !HOTSPOTS_JS.contains("fetch("),
        "P8: the renderer must never fetch"
    );
    assert!(
        !HOTSPOTS_JS.contains("XMLHttpRequest"),
        "P8: the renderer must never open an XMLHttpRequest"
    );
    assert!(
        !HOTSPOTS_JS.contains("EventSource"),
        "P8: the renderer must never open an EventSource"
    );
}

#[test]
fn test_help_playbook_copy() {
    // C7: the Hotspots help explains the collapsed card, the never-mutates
    // boundary, and the validated/rollback nature of apply; the locked
    // phrases survive in the same legend tab.
    let legend = tab_slice(INDEX_HTML, "id=\"helpTab-legend\"", "id=\"helpTab-recipes\"");
    assert!(
        legend.contains("collapsed summary") && legend.contains("click"),
        "C7: help must explain the collapsed summary card and how to open it"
    );
    // C8 amendment 2 (extraction-auto-apply A-5): the same flipped claim
    // replaces the never-applies lock; the flip is the milestone's
    // headline behavior and every other locked phrase below stays.
    assert!(
        legend.contains("the UI applies with a dry-run first, validation in three tiers and automatic rollback"),
        "A-5 C8 amendment 2: help must state the UI applies with a dry-run first, validation in three tiers and automatic rollback"
    );
    assert!(
        legend.contains("three tiers") && legend.contains("rollback"),
        "C7: help must state apply validates in three tiers with automatic rollback"
    );
    assert!(
        legend.contains("Proposed extractions section"),
        "C7: the locked 'Proposed extractions section' phrase must survive"
    );
    assert!(
        legend.contains("Preview after extraction"),
        "C7: the locked 'Preview after extraction' phrase must survive"
    );
}
