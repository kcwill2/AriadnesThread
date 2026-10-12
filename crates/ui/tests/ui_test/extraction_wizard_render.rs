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
use ariadne_ui::{build_router, EXTRACTION_WIZARD_JS, HOTSPOTS_JS, INDEX_HTML};
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
