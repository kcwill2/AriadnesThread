//! Render locks for the extraction wizard (ui-extraction-wizard W-3a): the
//! asset registration chain (script tag, route, const, served bytes,
//! ui_smoke row), the modal-shell stepper markup (shape C1 chrome), the C7
//! wizard-state reset, the S0/S1 renderers over the boot payload, and the
//! ASCII/escape discipline of the new asset.
//!
//! Model: the house string-lock idiom of extraction_card_story.rs plus the
//! route-serve checks of asset_injection_hardening.rs. The step order is a
//! string-order lock over the asset (shape C1 row order, W-3a required test
//! 4); payload caps honesty reads the boot flag `target_files_truncated` and
//! the `+K more` chip idiom (shape C6, `just-more` precedent).

use crate::common;
use ariadne_graph::{CallGraph, IncludeGraph};
use ariadne_ui::{build_router, EXTRACTION_WIZARD_JS, HOTSPOTS_JS, INDEX_HTML, STYLES_CSS};
use axum::http::{header, StatusCode};

const WIZARD_SCRIPT_TAG: &str = "<script src=\"/app/extraction-wizard.js\"></script>";
const WIZARD_ROUTE_PATH: &str = "/app/extraction-wizard.js";
const SERVER_SOURCE: &str = include_str!("../../src/server.rs");
const ASSET_HANDLER_SOURCE: &str = include_str!("../../src/routes/api_payloads.rs");
const LIB_SOURCE: &str = include_str!("../../src/lib.rs");
const UI_SMOKE_SOURCE: &str = include_str!("../../../../tools/ui_smoke.py");

/// Shape C1 step labels, verbatim order S0..S6.
const WIZARD_STEP_LABELS: [&str; 7] = [
    "Overview",
    "Files to create",
    "Per-file manifest",
    "Impact and risk",
    "Dry run",
    "Execute",
    "After apply",
];

// ---- registration chain (required tests 1-2; green at Red by the
// documented chain-first order) ----

#[test]
fn test_index_loads_wizard_asset_in_pinned_position() {
    assert!(
        INDEX_HTML.contains(WIZARD_SCRIPT_TAG),
        "index.html must load /app/extraction-wizard.js"
    );
    let wizard_tag = INDEX_HTML
        .find(WIZARD_SCRIPT_TAG)
        .expect("wizard script tag present");
    let main_tag = INDEX_HTML
        .find("<script src=\"/app/main.js\"></script>")
        .expect("main.js script tag present");
    let theme_tag = INDEX_HTML
        .find("<script src=\"/app/theme.js\"></script>")
        .expect("theme.js script tag present");
    assert!(
        main_tag < wizard_tag && wizard_tag < theme_tag,
        "the wizard tag is pinned immediately before /app/theme.js (after /app/main.js)"
    );
}

#[tokio::test]
async fn test_wizard_route_serves_the_registered_const() {
    let route_lines: Vec<&str> = SERVER_SOURCE
        .lines()
        .filter(|line| line.contains(WIZARD_ROUTE_PATH))
        .collect();
    assert!(
        !route_lines.is_empty(),
        "server.rs must register the {WIZARD_ROUTE_PATH} route"
    );
    for line in route_lines {
        assert!(
            line.contains("get("),
            "route line `{line}` must use get(...)"
        );
    }
    assert!(
        LIB_SOURCE.contains("pub const EXTRACTION_WIZARD_JS: &str = include_str!(\"../assets/extraction-wizard.js\");"),
        "lib.rs must declare EXTRACTION_WIZARD_JS from the asset path"
    );
    assert!(
        ASSET_HANDLER_SOURCE.contains("asset_extraction_wizard_js")
            && ASSET_HANDLER_SOURCE.contains("crate::EXTRACTION_WIZARD_JS"),
        "routes/api_payloads.rs must serve ariadne_ui::EXTRACTION_WIZARD_JS"
    );

    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        IncludeGraph::new(),
        CallGraph::default(),
        common::empty_export_payload(),
    ));
    let (status, headers, body) = common::request_raw_body(app, WIZARD_ROUTE_PATH).await;
    assert_eq!(status, StatusCode::OK, "the wizard asset route must serve");
    assert_eq!(
        body,
        EXTRACTION_WIZARD_JS.as_bytes(),
        "the route serves the EXTRACTION_WIZARD_JS bytes verbatim"
    );
    assert_eq!(
        headers
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/javascript; charset=utf-8"),
        "the wizard asset must serve as JavaScript"
    );
    assert_eq!(
        headers
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("no-store"),
        "every UI asset route serves no-store (ui_asset contract)"
    );
}

// ---- modal shell + stepper chrome (required tests 3 and 5) ----

#[test]
fn test_wizard_modal_shell_declares_the_stepper() {
    assert!(
        INDEX_HTML.contains("id=\"wizardModal\""),
        "index.html must contain the wizard modal shell"
    );
    assert!(
        INDEX_HTML.contains("class=\"modal wizard-modal\""),
        "the wizard must be a .modal panel reusing the house modal infrastructure"
    );
    assert!(
        INDEX_HTML.contains("wizard-step-list"),
        "the shell must carry the ordered step list"
    );
    for label in WIZARD_STEP_LABELS {
        assert!(
            INDEX_HTML.contains(label),
            "the shell step list must name the {label} step"
        );
    }
    assert!(
        INDEX_HTML.contains("aria-current=\"step\""),
        "the stepper chrome must mark the current step with aria-current"
    );
    assert!(
        INDEX_HTML.contains("<button type=\"button\" class=\"btn-secondary\" id=\"wizardBack\"")
            && INDEX_HTML.contains(">Back</button>"),
        "Back must be a real <button> in the stepper chrome"
    );
    assert!(
        INDEX_HTML.contains("<button type=\"button\" class=\"btn-primary\" id=\"wizardNext\"")
            && INDEX_HTML.contains(">Next</button>"),
        "Next must be a real <button> in the stepper chrome"
    );
    assert!(
        INDEX_HTML.contains("id=\"wizardStep0\">"),
        "S0 must be the visible step section of the shell"
    );
    for step_index in 1..=6 {
        let hidden_section = format!("id=\"wizardStep{step_index}\" hidden>");
        assert!(
            INDEX_HTML.contains(hidden_section.as_str()),
            "step S{step_index} must switch via the hidden attribute (string-lockable)"
        );
    }
}

// ---- C7 wizard state (required test 6's reset half, declared separately so
// Red names it as its own failing lock) ----

#[test]
fn test_wizard_state_reset_declared() {
    assert!(
        EXTRACTION_WIZARD_JS.contains("function resetWizardState("),
        "extraction-wizard.js must declare the wizard state reset"
    );
    for field in [
        "planId:", "step:", "selectedFile:", "manifest:", "dryRun:", "error:",
    ] {
        assert!(
            EXTRACTION_WIZARD_JS.contains(field),
            "the wizard state object must carry the {field} slot (shape C7)"
        );
    }
    assert!(
        EXTRACTION_WIZARD_JS.contains("wizardState.planId !== planId")
            && EXTRACTION_WIZARD_JS.contains("resetWizardState(planId)"),
        "opening the wizard for a different plan must reset the wizard state (shape C7)"
    );
}

// ---- step order (required test 4) ----

#[test]
fn test_wizard_step_list_order() {
    let mut previous_label: Option<(&str, usize)> = None;
    for label in WIZARD_STEP_LABELS {
        let label_at = EXTRACTION_WIZARD_JS.find(label).unwrap_or_else(|| {
            panic!("step {label} must be declared in the wizard step list order")
        });
        if let Some((earlier_label, earlier_at)) = previous_label {
            assert!(
                earlier_at < label_at,
                "step {label} must come after step {earlier_label} (shape C1 order)"
            );
        }
        previous_label = Some((label, label_at));
    }
}

// ---- card entry (required test 6) ----

#[test]
fn test_card_entry_button_opens_wizard() {
    assert!(
        HOTSPOTS_JS.contains("openExtractionWizard("),
        "the proposal card must wire the Open-wizard handler"
    );
    assert!(
        HOTSPOTS_JS.contains("class=\"wizard-open-btn\""),
        "the proposal card must carry the Open wizard button"
    );
    assert!(
        HOTSPOTS_JS.contains("data-wizard-plan="),
        "the Open wizard button must ride the plan id inert (no inline payload)"
    );
    assert!(
        EXTRACTION_WIZARD_JS.contains("wizardState.planId !== planId"),
        "the entry must reset the wizard state for a different plan (shape C7)"
    );
}

// ---- caps honesty (required test 7) ----

#[test]
fn test_wizard_step_files_honors_payload_caps() {
    assert!(
        EXTRACTION_WIZARD_JS.contains(".target_files_truncated"),
        "the S1 step must read the payload truncation flag for caps honesty"
    );
    assert!(
        EXTRACTION_WIZARD_JS.contains("class=\"just-more\""),
        "the truncation note must reuse the just-more chip idiom (shape C6)"
    );
    assert!(
        EXTRACTION_WIZARD_JS.contains("+${hiddenSymbols} more"),
        "the per-file symbol overflow must render the +K more chip"
    );
    assert!(
        EXTRACTION_WIZARD_JS.contains("escapeHtml(fileRow.path)"),
        "the destination path must pass through escapeHtml"
    );
    assert!(
        EXTRACTION_WIZARD_JS.contains("escapeAttribute(fileRow.path)"),
        "the selectable row's data attribute must pass through escapeAttribute"
    );
}

// ---- asset discipline (required test 8) ----

#[test]
fn test_wizard_asset_is_ascii_and_escaped() {
    assert!(
        EXTRACTION_WIZARD_JS.is_ascii(),
        "extraction-wizard.js must be pure ASCII (no byte above 0x7F)"
    );
    assert!(
        !EXTRACTION_WIZARD_JS.contains("function escapeHtml("),
        "the wizard must not re-declare escapeHtml (exactly-once lock, asset_injection_hardening)"
    );
    assert!(
        !EXTRACTION_WIZARD_JS.contains("function escapeAttribute("),
        "the wizard must not re-declare escapeAttribute (exactly-once lock, asset_injection_hardening)"
    );
    for raw_interpolation in ["${proposal.", "${fileRow."] {
        assert!(
            !EXTRACTION_WIZARD_JS.contains(raw_interpolation),
            "every interpolated payload value must pass through an escaper, found raw {raw_interpolation}"
        );
    }
}

// ---- ui_smoke mirror (required test 9) ----

#[test]
fn test_ui_smoke_route_table_carries_wizard_row() {
    assert!(
        UI_SMOKE_SOURCE.contains("(\"/app/extraction-wizard.js\", \"text/javascript\", True)"),
        "tools/ui_smoke.py ROUTE_ASSERTIONS must carry the wizard route row"
    );
}

// ==== W-3b: steps S2-S6, the reset paths, the recipe, and the help copy ====
//
// Shape C1 rows S2-S6, C3 error handling, C6 caps, C7 resets, W3/W4/W5/W6/W9
// invariants. Local helper copies follow the house idiom (the card suites
// each carry their own css_rule_body/tab_slice copies).

/// Body of a named top-level extraction-wizard.js function, delimited by its
/// opening brace and the asset's column-0 closing-brace line (the asset is
/// not wrapped in an IIFE, unlike hotspots.js).
fn wizard_function_body(js: &str, name: &str) -> String {
    let head = format!("function {name}(");
    let at = js
        .find(&head)
        .unwrap_or_else(|| panic!("extraction-wizard.js must define {head}"));
    let open = js[at..]
        .find('{')
        .unwrap_or_else(|| panic!("unterminated body for {name}"));
    let close = js[at + open..]
        .find("\n}")
        .unwrap_or_else(|| panic!("unterminated body block for {name}"));
    js[at + open + 1..at + open + close].to_string()
}

/// Region of index markup between two unique markers, opening tag excluded
/// (local copy of the tab_slice idiom from help_center_readability.rs).
fn wizard_tab_slice(html: &str, start_marker: &str, end_marker: &str) -> String {
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

/// W4 copy-vs-move sentence, verbatim contract text (shape section 2 W4: it
/// must state that the backend writes the new file and that the source
/// declarations stay in place). The asset carries this constant verbatim.
const COPY_VS_MOVE_SENTENCE: &str = "The C++ backend writes the new file and leaves the source declarations in place; nothing is moved out of the source files.";

// Required test 1: S2 fetches the manifest route per step, never at boot.
#[test]
fn test_wizard_step_manifest_fetches_per_step() {
    assert!(
        EXTRACTION_WIZARD_JS.contains("'/api/extraction/'"),
        "extraction-wizard.js must fetch the manifest for the selected file"
    );
    assert!(
        EXTRACTION_WIZARD_JS.contains("'/manifest'"),
        "extraction-wizard.js must build the manifest URL from '/api/extraction/' + plan id + '/manifest' (shape C2)"
    );
    // extraction-auto-apply A-5 (journal deviation D1): the apply POST
    // rides its own fetch site by handoff mandate - its 409 carries two
    // meanings the blanket-409 GET fetcher cannot tell apart (shape C7) -
    // so the bound is now the GET fetcher plus exactly one sanctioned
    // apply site. The GET-side assertions below keep the original lock.
    assert!(
        EXTRACTION_WIZARD_JS.matches("fetch(").count() <= 2,
        "the wizard must keep its fetch handling to the GET fetcher plus the one apply POST site (fetch handling stays in named places only)"
    );
    let fetcher = wizard_function_body(EXTRACTION_WIZARD_JS, "fetchWizardRoute");
    assert!(
        fetcher.contains("await fetch("),
        "the single fetch call site must live in fetchWizardRoute"
    );
    let opener = wizard_function_body(EXTRACTION_WIZARD_JS, "openExtractionWizard");
    assert!(
        !opener.contains("fetch(") && !opener.contains("ensureWizard"),
        "the boot/open path must fetch nothing (shape C6: per-step fetching, never at boot)"
    );
    let entry = wizard_function_body(EXTRACTION_WIZARD_JS, "enterWizardStep");
    assert!(
        entry.contains("ensureWizardManifest") && entry.contains("ensureWizardDryRun"),
        "step entry must drive the per-step manifest and dry-run fetches (shape C6)"
    );
    assert!(
        wizard_function_body(EXTRACTION_WIZARD_JS, "wizardStepNext").contains("enterWizardStep("),
        "step navigation must enter the new step so its data is fetched per step"
    );
}

// Required test 2: the honest includes fallback (W3), never a fabricated path.
#[test]
fn test_wizard_includes_fallback_when_unavailable() {
    let manifest_step = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepManifest");
    assert!(
        manifest_step.contains("includes_unavailable"),
        "the S2 renderer must read includes_unavailable as the degradation signal (W3)"
    );
    assert!(
        manifest_step.contains("Includes unavailable: the engine could not resolve the dependencies for this new file"),
        "the S2 renderer must print the honest fallback line when includes_unavailable (W3)"
    );
    for fabricated in ["#include", "extracted.h"] {
        assert!(
            !EXTRACTION_WIZARD_JS.contains(fabricated),
            "no include path may be hardcoded in the wizard asset, found {fabricated} (W3)"
        );
    }
}

// Required test 3: the exact copy-vs-move sentence, keyed off the COMPUTED
// move_removals path with includes_unavailable as the degradation signal.
// The degraded honest_absent_dry_run answers copy_only: false alongside
// move_removals: 0, so the renderer must never read the bare copy_only flag
// (W-2a verdict relay): the computed path plus the degradation gate carries
// the truth.
#[test]
fn test_copy_vs_move_sentence_is_exact_and_keyed_off_the_computed_path() {
    assert!(
        EXTRACTION_WIZARD_JS.contains(COPY_VS_MOVE_SENTENCE),
        "extraction-wizard.js must carry the exact locked copy-vs-move sentence (W4)"
    );
    let impact = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepImpact");
    assert!(
        impact.contains("COPY_VS_MOVE_SENTENCE"),
        "step S3 must carry the locked copy-vs-move sentence (W4)"
    );
    let execute = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepExecute");
    assert!(
        execute.contains("COPY_VS_MOVE_SENTENCE"),
        "step S5 must carry the locked copy-vs-move sentence (W4)"
    );
    let copy_only_gate = wizard_function_body(EXTRACTION_WIZARD_JS, "wizardCopyOnly");
    assert!(
        copy_only_gate.contains("includes_unavailable"),
        "the copy-only gate must treat includes_unavailable as the degradation signal (W-2a verdict relay)"
    );
    assert!(
        copy_only_gate.contains("move_removals === 0"),
        "the copy-only sentence must be keyed off the computed move_removals path, never the bare copy_only flag"
    );
    assert!(
        !EXTRACTION_WIZARD_JS.contains(".copy_only"),
        "the renderer must never read the copy_only field directly (degraded answers set it false by force)"
    );
}

// Required test 4: S4 fetches the dry-run route and renders rewrite rows
// with the &rarr; entity between the old and new include sets (shape C4).
#[test]
fn test_wizard_dry_run_rewrites_render() {
    assert!(
        EXTRACTION_WIZARD_JS.contains("'/dry-run'"),
        "extraction-wizard.js must fetch the dry-run route '/api/extraction/' + plan id + '/dry-run' (shape C2)"
    );
    let dry_run = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepDryRun");
    assert!(
        dry_run.contains("&rarr;"),
        "rewrite rows must separate old and new include sets with the &rarr; entity"
    );
    let old_at = dry_run.find("wizard-diff-old").expect("rewrite rows must mark the old include set");
    let arrow_at = dry_run.find("&rarr;").expect("rewrite rows must carry the &rarr; entity");
    let new_at = dry_run
        .find("wizard-diff-new")
        .expect("rewrite rows must mark the new include set");
    assert!(
        old_at < arrow_at && arrow_at < new_at,
        "the &rarr; entity must sit between the old and the new include set in every rewrite row"
    );
    assert!(
        dry_run.contains("side_effect_preservations"),
        "rewrite rows must render the preserved side-effect includes (shape C4)"
    );
    assert!(
        dry_run.contains("The dry run changes nothing"),
        "the dry-run panel must honestly state that it changes nothing (shape C4)"
    );
}

// Required test 5 (first half, W6/C7): the 409 stale-generation handler
// resets the wizard state from the fetch error path.
#[test]
fn test_wizard_resets_on_stale_generation() {
    let fetcher = wizard_function_body(EXTRACTION_WIZARD_JS, "fetchWizardRoute");
    assert!(
        fetcher.contains("response.status === 409"),
        "extraction-wizard.js must handle the 409 stale-generation error shape (shape C3/W6)"
    );
    assert!(
        fetcher.contains("resetWizardState(wizardState.planId)"),
        "a 409 stale-generation response must reset the wizard step state from the fetch error path (C7)"
    );
}

// Required test 5 (second half, C7): the payload-swap re-render path resets
// the wizard state; the reset call stays string-lockable.
#[test]
fn test_wizard_resets_on_payload_swap() {
    assert!(
        EXTRACTION_WIZARD_JS.contains("function resetWizardOnPayloadSwap("),
        "extraction-wizard.js must declare the payload-swap wizard reset (shape C7)"
    );
    assert!(
        EXTRACTION_WIZARD_JS.contains("window.applyLoadedData = "),
        "the payload-swap reset must hook the existing re-render path (applyLoadedData, shape C7)"
    );
    let swap = wizard_function_body(EXTRACTION_WIZARD_JS, "resetWizardOnPayloadSwap");
    assert!(
        swap.contains("resetWizardState(planId)"),
        "the payload swap must reset the wizard state and keep it string-lockable (C7)"
    );
}

// Required test 6: caps, symbol_count truth, and the truncation copy (C6).
#[test]
fn test_wizard_step_caps_and_truncation_are_honest() {
    assert!(
        EXTRACTION_WIZARD_JS.contains("EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE = 200"),
        "the renderer must mirror the documented per-file cap EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE = 200 (C6)"
    );
    assert!(
        EXTRACTION_WIZARD_JS.contains("EXTRACTION_DRY_RUN_MAX_REWRITES = 200"),
        "the renderer must mirror EXTRACTION_DRY_RUN_MAX_REWRITES = 200 (C6)"
    );
    assert!(
        EXTRACTION_WIZARD_JS.contains("EXTRACTION_MANIFEST_MAX_FILES = 64"),
        "the renderer must mirror EXTRACTION_MANIFEST_MAX_FILES = 64 (C6)"
    );
    let manifest_step = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepManifest");
    assert!(
        manifest_step.contains("EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE"),
        "the per-file list must render at most EXTRACTION_MANIFEST_MAX_SYMBOLS_PER_FILE rows"
    );
    assert!(
        manifest_step.contains("symbol_count"),
        "symbol_count must render beside the capped rows as the count truth (C6)"
    );
    assert!(
        manifest_step.contains("wizardCapChip"),
        "the per-file overflow must use the shared +K more chip helper (shape C6)"
    );
    assert!(
        manifest_step.contains("files_truncated"),
        "files_truncated must drive the honest file-list cap copy (C6)"
    );
    let chip = wizard_function_body(EXTRACTION_WIZARD_JS, "wizardCapChip");
    assert!(
        chip.contains("class=\"just-more\">+${hiddenCount} more"),
        "the chip must reuse the just-more idiom with the +K more form (shape C6)"
    );
    let dry_run = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepDryRun");
    assert!(
        dry_run.contains("EXTRACTION_DRY_RUN_MAX_REWRITES"),
        "the rewrites list must honor EXTRACTION_DRY_RUN_MAX_REWRITES"
    );
    assert!(
        dry_run.contains("rewrites_truncated") && dry_run.contains("truncated"),
        "rewrites_truncated must drive the honest truncation copy (C6)"
    );
}

// Required test 7: the recipe verb list is exactly the three W5 verbs with
// the plan id interpolated; no fourth verb string exists in the asset.
#[test]
fn test_wizard_recipe_names_exactly_the_three_w5_verbs() {
    let execute = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepExecute");
    assert!(
        execute.contains("tool transform extract &lt;repo-root&gt; --plan "),
        "the recipe lock reports the missing verb: 'tool transform extract <repo-root> --plan <plan_id>' (W5)"
    );
    assert!(
        execute.contains("tool migrate &lt;repo-root&gt; --plan ") && execute.contains("--preview"),
        "the recipe must show the --preview-first step 'tool migrate <repo-root> --plan <plan_id> --preview' (W5)"
    );
    assert!(
        execute.contains("tool apply &lt;repo-root&gt; --plan "),
        "the recipe must show 'tool apply <repo-root> --plan <plan_id>' (W5)"
    );
    let preview_first = execute
        .find("tool migrate")
        .expect("migrate verb present in the recipe");
    let extract_at = execute
        .find("tool transform extract")
        .expect("transform extract verb present in the recipe");
    let apply_at = execute.find("tool apply").expect("apply verb present in the recipe");
    assert!(
        preview_first < extract_at && extract_at < apply_at,
        "the recipe order must be preview first, then transform extract, then the full apply"
    );
    assert!(
        execute.contains("escapeHtml(planId || '')"),
        "the plan id interpolated into every recipe verb must pass through escapeHtml"
    );
    let named_verbs = EXTRACTION_WIZARD_JS.matches("tool ").count();
    let honest_verbs = EXTRACTION_WIZARD_JS.matches("tool apply").count()
        + EXTRACTION_WIZARD_JS.matches("tool migrate").count()
        + EXTRACTION_WIZARD_JS.matches("tool transform extract").count();
    assert_eq!(
        named_verbs, honest_verbs,
        "W5: only tool apply, tool migrate and tool transform extract may be named in the wizard asset"
    );
}

// Required test 8: the help copy names the execution boundary and the
// follow-on milestone key; the W-3b renderers keep the escape discipline.
#[test]
fn test_wizard_help_copy_names_the_execution_boundary() {
    let legend = wizard_tab_slice(INDEX_HTML, "id=\"helpTab-legend\"", "id=\"helpTab-recipes\"");
    // C8 amendment 3 (extraction-auto-apply A-5): the never-mutates
    // sentence is the copy this milestone flips; the claim of record
    // replaces it and the extraction-auto-apply key below stays as
    // provenance.
    assert!(
        legend.contains("the UI applies with a dry-run first, validation in three tiers and automatic rollback"),
        "the Hotspots help copy must state that the UI applies with a dry-run first, validation in three tiers and automatic rollback (extraction-auto-apply C8 amendment 3)"
    );
    assert!(
        legend.contains("extraction-auto-apply"),
        "the Hotspots help copy must name the follow-on milestone key extraction-auto-apply as the execution boundary (shape section 4)"
    );
    for raw_interpolation in [
        "${proposal.",
        "${manifest.",
        "${selected.",
        "${dryRun.",
        "${rewrite.",
        "${fileRow.",
    ] {
        assert!(
            !EXTRACTION_WIZARD_JS.contains(raw_interpolation),
            "every W-3b interpolated payload value must pass through an escaper, found raw {raw_interpolation}"
        );
    }
}

// ==== extraction-auto-apply A-5: the S5 approval surface, the S6 live ====
// ==== outcome, the apply response mapping, and the style block.        ====

/// A7 dry-run-first instruction, verbatim contract copy: the S5 renderer
/// carries it whenever no approval token is in hand.
const DRY_RUN_FIRST_INSTRUCTION: &str =
    "Open the dry run first: the apply unlocks once the dry run returns its single-use approval token.";

// Required A-5 test 1 (A7): the apply control is enabled only with the
// dry run's approval token, and the C7 state slots exist and clear at the
// single reset point (which every existing reset path already calls).
#[test]
fn test_execute_step_apply_control_requires_an_approval_token() {
    let execute = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepExecute");
    assert!(
        execute.contains("wizardState.dryRun.apply_token") && execute.contains("disabled"),
        "renderWizardStepExecute must gate its apply control on the dry-run apply_token"
    );
    assert!(
        execute.contains("approvalMissing") && execute.contains("applyDisabled"),
        "the apply control's enabled state must be keyed off the token through a named predicate"
    );
    assert!(
        EXTRACTION_WIZARD_JS.contains("applying: false")
            && EXTRACTION_WIZARD_JS.contains("outcome: null"),
        "the wizard state must carry the applying and outcome slots (shape C7)"
    );
    let reset = wizard_function_body(EXTRACTION_WIZARD_JS, "resetWizardState");
    assert!(
        reset.contains("wizardState.applying = false")
            && reset.contains("wizardState.outcome = null"),
        "the single reset point must clear applying and outcome so every existing reset path clears both (shape C7)"
    );
}

// Required A-5 test 2 (A7): without a token the step renders the
// dry-run-first instruction.
#[test]
fn test_execute_step_shows_the_dry_run_first_instruction_without_a_token() {
    assert!(
        EXTRACTION_WIZARD_JS.contains(DRY_RUN_FIRST_INSTRUCTION),
        "the execute step must instruct the user to open the dry run first when no token is present"
    );
    let execute = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepExecute");
    assert!(
        execute.contains("approvalMissing") && execute.contains("DRY_RUN_FIRST_INSTRUCTION"),
        "the dry-run-first instruction must be the branch taken when the token is absent"
    );
}

// Required A-5 test 3 (W5 survival after the S5 edit): the three verbs,
// their order, the escaped plan id and the W4 sentence all survive.
#[test]
fn test_execute_step_keeps_the_three_verb_recipe() {
    let execute = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepExecute");
    let preview_first = execute
        .find("tool migrate")
        .expect("migrate verb present in the recipe");
    let extract_at = execute
        .find("tool transform extract")
        .expect("transform extract verb present in the recipe");
    let apply_at = execute.find("tool apply").expect("apply verb present in the recipe");
    assert!(
        preview_first < extract_at && extract_at < apply_at,
        "the S5 edit must keep the recipe order: preview first, then transform extract, then the full apply"
    );
    assert!(
        execute.contains("escapeHtml(planId || '')"),
        "the S5 edit must keep the plan id interpolated through escapeHtml"
    );
    assert!(
        execute.contains("COPY_VS_MOVE_SENTENCE"),
        "the S5 edit must keep the locked W4 copy-vs-move sentence"
    );
}

// Required A-5 test 4 (A5/S6): the live outcome rows - tiers, verdict,
// created files, verbatim report - all escaped, and the descriptive copy
// stays for the pre-apply entry.
#[test]
fn test_after_apply_step_renders_the_live_outcome_rows() {
    let after_apply = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepAfterApply");
    assert!(
        after_apply.contains("renderWizardStepOutcome("),
        "renderWizardStepAfterApply must render the outcome tiers, verdict and output_text"
    );
    assert!(
        after_apply.contains("Rollback fires only on a failed tier"),
        "the step keeps its descriptive copy when no outcome exists (it stays enterable before an apply)"
    );
    let outcome = wizard_function_body(EXTRACTION_WIZARD_JS, "renderWizardStepOutcome");
    assert!(outcome.contains("tiers"), "the outcome renderer must read the tier rows");
    for escaped in [
        "escapeHtml(tier.tier)",
        "escapeHtml(statusText)",
        "escapeHtml(tier.detail)",
        "escapeHtml(filePath)",
        "escapeHtml(outcome.output_text",
    ] {
        assert!(
            outcome.contains(escaped),
            "the outcome renderer must interpolate {escaped} through the escaper (A9/A11 escape discipline)"
        );
    }
    assert!(
        outcome.contains("Committed") && outcome.contains("Rolled back"),
        "the verdict line must state committed vs rolled back"
    );
    assert!(
        outcome.contains("artifacts/transaction"),
        "the S6 copy must state the outcome record lands under artifacts/transaction (shape C6)"
    );
}

// Required A-5 test 5 (A9/A11): the ban - no raw outcome or applying
// interpolation may ever enter the asset.
#[test]
fn test_every_outcome_interpolation_is_escaped() {
    for raw_form in ["${outcome.", "${applying"] {
        assert!(
            !EXTRACTION_WIZARD_JS.contains(raw_form),
            "the outcome path must interpolate no value raw, found {raw_form} (A9/A11)"
        );
    }
}

// Required A-5 test 6 (shape C7 mapping): the apply request is one named
// function with its own fetch site, and every refusal code has its own
// copy; the 200 path stores the outcome and advances to S6.
#[test]
fn test_apply_response_mapping_covers_every_refusal() {
    assert!(
        EXTRACTION_WIZARD_JS.contains("async function applyExtractionPlan("),
        "the apply request must be one named function applyExtractionPlan (shape section 9 binding name)"
    );
    let apply = wizard_function_body(EXTRACTION_WIZARD_JS, "applyExtractionPlan");
    assert!(
        apply.contains("extractionRouteUrl('/apply')")
            && apply.contains("method: 'POST'")
            && apply.contains("apply_token"),
        "the apply must POST to the plan's /apply route with the dry run's token as the body (shape C3)"
    );
    assert!(
        !apply.contains("fetchWizardRoute"),
        "the apply must not ride the blanket-409 GET fetcher: its 409 has two meanings (handoff item 3)"
    );
    assert!(
        apply.contains("stale_generation")
            && apply.contains("resetWizardState(wizardState.planId)"),
        "the apply's 409 stale_generation must ride the existing stale-generation reset (shape C7)"
    );
    assert!(
        apply.contains("apply_in_progress")
            && apply.contains("another apply is running; try again in a moment"),
        "the apply_in_progress refusal must show its own sentence (shape C7)"
    );
    assert!(
        apply.contains("status === 428")
            && apply.contains("re-open the dry run to refresh the approval"),
        "the 428 dry_run_required refusal must show the approval-refresh copy (shape C7)"
    );
    assert!(
        apply.contains("status === 500") && apply.contains("refusal.detail"),
        "the 500 apply_failed refusal must show the refusal's detail (journal deviation D3)"
    );
    assert!(
        apply.contains("status === 503") && apply.contains("analysis is reloading"),
        "the 503 analysis_loading refusal must state the analysis is reloading"
    );
    assert!(
        apply.contains("status === 404"),
        "the 404 unknown_plan refusal must have its own branch and copy (journal deviation D4)"
    );
    assert!(
        apply.contains("is unreachable"),
        "a network failure must reuse the existing unreachable-copy idiom"
    );
    assert!(
        apply.contains("wizardState.outcome =")
            && apply.contains("showWizardStep(6)")
            && apply.contains("enterWizardStep(6)"),
        "the 200 path stores the outcome and advances to S6 (shape C7)"
    );
}

// Required A-5 test 7: the asset stays ASCII-only through the new apply
// and outcome code (the existing idiom, restated at this hand).
#[test]
fn test_wizard_asset_stays_ascii_after_the_apply_surface() {
    assert!(
        EXTRACTION_WIZARD_JS.is_ascii(),
        "extraction-wizard.js must stay pure ASCII (no byte above 0x7F) after the A-5 apply and outcome code"
    );
}

// Required A-5 test 8: the contiguous A-5 style block exists, styles the
// apply control, its disabled and in-flight states and the outcome rows,
// adds zero hex and takes its colors from existing theme tokens (the
// theme locks' assertion style; theme_token_check --check-added is the
// milestone-level added-hex oracle).
#[test]
fn test_apply_outcome_style_block_adds_no_hex() {
    let marker = "=== EXTRACTION AUTO-APPLY (extraction-auto-apply A-5)";
    let at = STYLES_CSS.find(marker).unwrap_or_else(|| {
        panic!("styles.css must carry the contiguous A-5 apply/outcome block behind its marker comment")
    });
    let block = &STYLES_CSS[at..];
    assert!(
        !block.contains('#'),
        "the A-5 apply/outcome block must add zero hex literals (existing theme tokens only)"
    );
    for rule in [
        ".wizard-apply-row {",
        ".wizard-apply-button[disabled] {",
        ".wizard-apply-instruction {",
        ".wizard-apply-inflight {",
        ".wizard-outcome-tier {",
        ".wizard-tier-pass {",
        ".wizard-tier-fail {",
        ".wizard-tier-skipped {",
        ".wizard-outcome-verdict {",
        ".wizard-outcome-text {",
    ] {
        assert!(
            block.contains(rule),
            "the A-5 block must style {rule} (apply control, disabled state, in-flight state, outcome rows)"
        );
    }
    assert!(
        block.contains("var(--"),
        "the A-5 block must take its colors from existing theme tokens"
    );
}
