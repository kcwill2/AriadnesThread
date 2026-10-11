//! AR-11 oracle (S2 HIGH / S4 MED): every repo-derived interpolation into
//! shipped markup either passes through the quote-safe escapers or rides an
//! inert data-* attribute consumed by delegated listeners. The locks are
//! single-pass string locks over the shipped asset sources, the served asset
//! bytes, and the standalone export - not a browser.
//!
//! The behavior-preservation contract: every removed inline handler keeps its
//! exact call (`selectNode`, `illuminatePath`, `selectHotspotRow`,
//! `openTraversalForRoot`, `closeModals`) via the mapping documented in the
//! delegation comments and in tasks/ar-11-handoff-journal.md.

use crate::common;
use ariadne_ui::{
    build_router, API_JS, FILE_VIEW_JS, GRAPH_JS, HOTSPOTS_JS, MAIN_JS, PREVIEW_JS,
    PROVENANCE_JS, THEME_JS, TRAVERSAL_JS,
};
use axum::http::StatusCode;

/// Hostile analyzed-repo content: a filename carrying a tag, a double quote
/// and a single quote (all ASCII; no fixture literals beyond ASCII needed).
const HOSTILE_FILENAME: &str = "<svg onload=x>\"quote'.h";
const HOSTILE_REPO_NAME: &str = "<svg onload=x>\"quote'.repo";

/// Local copy of the function_body idiom (modules do not import each other).
fn function_body(js: &str, name: &str) -> String {
    let marker = format!("function {name}");
    let at = js.find(&marker).unwrap_or_else(|| panic!("asset must define {name}"));
    let open = at + js[at..].find('{').expect("function must open with brace");
    let mut depth = 0usize;
    let mut index = open;
    while index < js.len() {
        match js.as_bytes()[index] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return js[open..=index].to_string();
                }
            }
            _ => {}
        }
        index += 1;
    }
    panic!("function {name} must close");
}

/// Every inline `onclick="..."` payload the shipped source still carries
/// (matched on the literal `onclick="` prefix inside template strings).
fn inline_onclick_payloads(source: &str) -> Vec<String> {
    let marker = "onclick=\"";
    let mut payloads = Vec::new();
    let mut cursor = 0usize;
    while let Some(found) = source[cursor..].find(marker) {
        let start = cursor + found + marker.len();
        let end = source[start..]
            .find('"')
            .map(|relative| start + relative)
            .unwrap_or(source.len());
        payloads.push(source[start..end].to_string());
        cursor = end;
    }
    payloads
}

/// The dangerous subset: inline handler bodies that still interpolate a
/// repo-derived expression into a JS payload (`${...}` inside the attribute).
fn interpolated_onclick_payloads(source: &str) -> Vec<String> {
    inline_onclick_payloads(source)
        .into_iter()
        .filter(|payload| payload.contains("${"))
        .collect()
}

fn all_js_assets() -> [(&'static str, &'static str); 9] {
    [
        ("api.js", API_JS),
        ("graph.js", GRAPH_JS),
        ("traversal.js", TRAVERSAL_JS),
        ("file-view.js", FILE_VIEW_JS),
        ("hotspots.js", HOTSPOTS_JS),
        ("preview.js", PREVIEW_JS),
        ("provenance.js", PROVENANCE_JS),
        ("main.js", MAIN_JS),
        ("theme.js", THEME_JS),
    ]
}

/// S2 oracle: no repo-derived interpolation survives inside an inline
/// handler payload. The two survivors are pinned with reasons:
/// - traversal.js `selectNode('${n.id}')`: server-minted numeric file ids
///   (`FileId.0.to_string()`), and the handoff forbids any traversal.js diff
///   beyond its two declared option sites (tasks/ar-11-handoff.md).
/// - main.js `jumpToCrumb(${i})`: numeric threadHistory index, not a
///   repo-derived value.
#[test]
fn test_no_repo_interpolation_survives_in_inline_onclick() {
    for (asset_name, source) in [
        ("index.html", ariadne_ui::INDEX_HTML),
        ("api.js", API_JS),
        ("graph.js", GRAPH_JS),
        ("file-view.js", FILE_VIEW_JS),
        ("hotspots.js", HOTSPOTS_JS),
        ("preview.js", PREVIEW_JS),
        ("provenance.js", PROVENANCE_JS),
        ("theme.js", THEME_JS),
    ] {
        assert!(
            interpolated_onclick_payloads(source).is_empty(),
            "{asset_name} must not interpolate repo-derived payloads into inline onclick"
        );
    }
    assert_eq!(
        interpolated_onclick_payloads(TRAVERSAL_JS),
        vec!["selectNode('${n.id}')".to_string()],
        "traversal.js keeps exactly its one known survivor (numeric ids, zero-diff constraint)"
    );
    assert_eq!(
        interpolated_onclick_payloads(MAIN_JS),
        vec!["jumpToCrumb(${i})".to_string()],
        "main.js keeps exactly its numeric thread-crumb index handler"
    );
}

/// S4 contract: the shared text escaper escapes all five markup-significant
/// characters (quote extension shipped), the attribute escaper stays
/// attribute-safe, and each ships exactly once across all JS assets so no
/// later script can re-declare a weaker `escapeHtml` over the shared one.
#[test]
fn test_shared_escapers_are_quote_safe_and_shipped_exactly_once() {
    let escape_html_body = function_body(HOTSPOTS_JS, "escapeHtml");
    for entity in ["&amp;", "&lt;", "&gt;", "&quot;", "&#39;"] {
        assert!(
            escape_html_body.contains(entity),
            "escapeHtml must escape to {entity} (quote extension for S4)"
        );
    }
    let escape_attribute_body = function_body(HOTSPOTS_JS, "escapeAttribute");
    for entity in ["&amp;", "&lt;", "&gt;", "&quot;", "&#39;"] {
        assert!(
            escape_attribute_body.contains(entity),
            "escapeAttribute must escape to {entity}"
        );
    }
    let html_definition_count = all_js_assets()
        .iter()
        .filter(|(_, source)| source.contains("function escapeHtml("))
        .count();
    let attribute_definition_count = all_js_assets()
        .iter()
        .filter(|(_, source)| source.contains("function escapeAttribute("))
        .count();
    assert_eq!(
        html_definition_count, 1,
        "exactly one escapeHtml definition across all shipped JS assets"
    );
    assert_eq!(
        attribute_definition_count, 1,
        "exactly one escapeAttribute definition across all shipped JS assets"
    );
}

/// Double-escape lock: no site may run a value through two escapers, so the
/// composed call shapes must never appear in shipped sources.
#[test]
fn test_no_nested_escaper_calls() {
    for (asset_name, source) in all_js_assets() {
        for nesting in [
            "escapeHtml(escapeHtml(",
            "escapeHtml(escapeAttribute(",
            "escapeAttribute(escapeHtml(",
            "escapeAttribute(escapeAttribute(",
        ] {
            assert!(
                !source.contains(nesting),
                "{asset_name} must not nest escapers (single-pass): found {nesting}"
            );
        }
    }
}

/// graph.js inspector sites (audit anchor :596-:640): ids move to inert
/// data-* attributes, labels escape through the escapers, and the inline
/// selectNode/illuminatePath payloads are gone.
#[test]
fn test_graphjs_inspector_sites_delegated_and_escaped() {
    assert!(
        GRAPH_JS.contains(r##"<div class="link-item" data-select-id="${escapeAttribute(e.source)}">"##),
        "upstream row must carry the source id as an inert escaped data attribute"
    );
    assert!(
        GRAPH_JS.contains(r##"<div class="link-item" data-select-id="${escapeAttribute(e.target)}">"##),
        "downstream row must carry the target id as an inert escaped data attribute"
    );
    assert!(
        GRAPH_JS
            .contains(r##"<div class="link-item" data-path-from="${escapeAttribute(id)}" data-path-to="${escapeAttribute(entry.id)}" title="Click to illuminate path from ${escapeAttribute(node.label)} to ${escapeAttribute(entry.label)}">"##),
        "collateral row must carry the path pair inert and escape both title labels"
    );
    assert!(
        GRAPH_JS.contains(r##"<span>${escapeHtml(name)}</span>"##),
        "upstream name must pass through escapeHtml"
    );
    assert!(
        GRAPH_JS.contains(r##"<span style="color:var(--cyan);">${escapeHtml(name)}</span>"##),
        "downstream name must pass through escapeHtml"
    );
    assert!(
        GRAPH_JS.contains(r##"<span>${escapeHtml(entry.label)}</span>"##),
        "collateral label must pass through escapeHtml"
    );
    assert!(!GRAPH_JS.contains(r##"onclick="selectNode("##), "inline selectNode payload must be gone");
    assert!(
        !GRAPH_JS.contains(r##"onclick="illuminatePath("##),
        "inline illuminatePath payload must be gone"
    );
}

/// api.js declared sites (:304 path nodes, :324 entrypoints) plus the
/// justification preview buttons: attributes switch from escapeHtml to the
/// attribute-safe escaper, text goes through escapeHtml, and the entrypoint
/// row id rides an inert data attribute.
#[test]
fn test_apijs_declared_sites_escaped_and_delegated() {
    assert!(
        API_JS.contains(r##"<span class="path-node">${escapeHtml(n)}</span>"##),
        "path-node text must pass through escapeHtml"
    );
    assert!(
        API_JS.contains(r##"data-select-id="${escapeAttribute(ep)}">${escapeHtml(ep)}</div>"##),
        "entrypoint row must carry the id inert and escape its text"
    );
    assert!(
        API_JS.contains(
            r##"data-preview-root="${escapeAttribute(String(nodeId))}" data-preview-source="${escapeAttribute(String(row.id))}""##
        ),
        "justification preview attributes must use the attribute-safe escaper"
    );
    assert!(
        !API_JS.contains(r##"data-preview-root="${escapeHtml("##),
        "attribute contexts must not use the plain text escaper"
    );
    assert!(
        !API_JS.contains(r##"onclick="selectNode("##),
        "inline selectNode payload must be gone from api.js"
    );
}

/// traversal.js: only the two declared option sites change (:44 roots, :131
/// hidden deps). The degree-direction behavior (known-open
/// `ui.traversal_view_degree_setting_bug`) must be byte-untouched.
#[test]
fn test_traversaljs_declared_option_sites_escaped_without_degree_regression() {
    assert!(
        TRAVERSAL_JS.contains(
            r##"<option value="${escapeAttribute(n.id)}">${escapeHtml(n.label)} (${escapeHtml(n.path)})</option>"##
        ),
        "traversal root option must escape id, label and path"
    );
    assert!(
        TRAVERSAL_JS.contains(
            r##"<option value="${escapeAttribute(d.id)}">${escapeHtml(d.label)} (${escapeHtml(d.path)})</option>"##
        ),
        "traversal dependency option must escape id, label and path"
    );
    assert!(
        TRAVERSAL_JS.contains(r##"data-node-id="${n.id}" onclick="selectNode('${n.id}')"##),
        "zero-diff constraint: the known survivor row stays byte-identical"
    );
    assert!(
        TRAVERSAL_JS.contains("dirSelect.value = 'out'"),
        "zero-diff constraint: the degree/direction handling stays untouched"
    );
}

/// hotspots.js delegation: row, traversal button and extraction card move to
/// data-* + one document listener; the mapping to the removed inline handlers
/// ships with the listener; the disclosure button keeps its inline handler
/// (no repo data in it) so its stopPropagation still suppresses card select.
#[test]
fn test_hotspotsjs_rows_and_cards_delegated_behaviour_preserved() {
    assert!(
        HOTSPOTS_JS.contains(r##"data-select-id="${escapeAttribute(e.node.id)}""##),
        "hotspot row id must ride an inert escaped data attribute"
    );
    assert!(
        HOTSPOTS_JS.contains(r##"data-traversal-root="${escapeAttribute(e.node.id)}""##),
        "traversal button root must ride an inert escaped data attribute"
    );
    assert!(
        HOTSPOTS_JS.contains(r##"data-select-id="${escapeAttribute(String(p.target_file_id))}""##),
        "extraction card id must ride an inert escaped data attribute"
    );
    assert!(
        HOTSPOTS_JS.contains("openTraversalForRoot(traversalBtn.dataset.traversalRoot)"),
        "delegated traversal button must call openTraversalForRoot like the removed handler"
    );
    assert!(
        HOTSPOTS_JS.contains("selectHotspotRow(card.dataset.selectId)"),
        "delegated card click must call selectHotspotRow like the removed handler"
    );
    assert!(
        HOTSPOTS_JS.contains("selectHotspotRow(row.dataset.selectId)"),
        "delegated row click must call selectHotspotRow like the removed handler"
    );
    assert!(
        HOTSPOTS_JS.contains(r##"selectHotspotRow('${e.node.id}')"##)
            && HOTSPOTS_JS.contains(r##"openTraversalForRoot('${e.node.id}')"##),
        "the delegation mapping to the removed inline handlers must ship with the listener"
    );
    assert!(
        HOTSPOTS_JS.contains(r##"onclick="toggleExtractionCard(event, this)""##),
        "disclosure button keeps its data-free inline handler (stopPropagation contract)"
    );
    assert!(
        !HOTSPOTS_JS.contains(r##"onclick="selectHotspotRow("##)
            && !HOTSPOTS_JS.contains(r##"onclick="event.stopPropagation()"##),
        "repo-derived inline payloads must be gone from hotspots.js"
    );
    assert!(
        PREVIEW_JS.contains("e.stopPropagation()"),
        "preview buttons keep suppressing the card click route"
    );
}

/// main.js delegation wiring: one listener per static container, calling the
/// same handlers and argument shapes the removed inline payloads used.
#[test]
fn test_mainjs_delegation_wiring_calls_the_legacy_handlers() {
    assert!(
        MAIN_JS.contains(
            r##"onListClick('inspUpstreamList', '.link-item[data-select-id]', row => selectNode(row.dataset.selectId));"##
        ),
        "upstream rows must delegate to selectNode with the row id"
    );
    assert!(
        MAIN_JS.contains(
            r##"onListClick('inspDownstreamList', '.link-item[data-select-id]', row => selectNode(row.dataset.selectId));"##
        ),
        "downstream rows must delegate to selectNode with the row id"
    );
    assert!(
        MAIN_JS.contains(
            r##"onListClick('inspCollateralList', '.link-item[data-path-from]', row => illuminatePath(row.dataset.pathFrom, row.dataset.pathTo));"##
        ),
        "collateral rows must delegate to illuminatePath(from, to)"
    );
    assert!(
        MAIN_JS.contains(r##"onListClick('entrypointsList', 'div[data-select-id]', row => {"##),
        "entrypoint rows must delegate to the selectNode + closeModals pair"
    );
}

/// Hostile fixture reaching the served surface and the standalone export:
/// the asset bytes carry the hardened markup, exactly one escaper ships in
/// the export, and the hostile filename never survives raw into the page.
#[tokio::test]
async fn test_served_and_exported_surfaces_carry_hardened_markup_for_hostile_payloads() {
    let (mut graph, mut payload) = common::fixture_include_graph();
    graph.file_paths.insert(
        ariadne_repo::FileId(11),
        std::path::PathBuf::from(HOSTILE_FILENAME),
    );
    graph.edges.push(ariadne_graph::IncludeEdge {
        source: ariadne_repo::FileId(11),
        destination: Some(ariadne_repo::FileId(1)),
        context_name: String::new(),
        source_location: ariadne_graph::SourceLocation { line: 1, column: 1 },
        kind: ariadne_graph::IncludeKind::Local,
        raw_path: String::new(),
        resolved_path: None,
        language: ariadne_graph::LanguageId::Cpp,
    });
    payload.include_graph = ariadne_ui::build_include_elements(
        &graph,
        &std::collections::HashMap::new(),
        &[],
        &std::collections::HashMap::new(),
        &std::collections::HashMap::new(),
    );
    payload.repo_name = HOSTILE_REPO_NAME.to_string();

    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        graph,
        ariadne_graph::CallGraph::default(),
        payload.clone(),
    ));
    for (route, constant) in [
        ("/app/api.js", API_JS),
        ("/app/graph.js", GRAPH_JS),
        ("/app/traversal.js", TRAVERSAL_JS),
        ("/app/file-view.js", FILE_VIEW_JS),
        ("/app/hotspots.js", HOTSPOTS_JS),
        ("/app/main.js", MAIN_JS),
    ] {
        let (status, _headers, body) = common::request_raw_body(app.clone(), route).await;
        assert_eq!(status, StatusCode::OK, "{route} must serve");
        assert_eq!(
            body,
            constant.as_bytes(),
            "{route} serves exactly the hardened asset bytes"
        );
    }
    let (_status, _headers, served_graph) =
        common::request_raw_body(app, "/app/graph.js").await;
    assert!(
        interpolated_onclick_payloads(&String::from_utf8(served_graph).unwrap()).is_empty(),
        "the served graph.js carries no inline payload to execute"
    );

    let export_dir = tempfile::tempdir().unwrap();
    let export_path = export_dir.path().join("hostile-export.html");
    ariadne_ui::export_standalone_html(&payload, "", &export_path).unwrap();
    let page = std::fs::read_to_string(&export_path).unwrap();
    assert!(
        page.contains(r##"data-select-id="${escapeAttribute(e.source)}""##),
        "the export inlines the hardened graph.js markup"
    );
    assert!(
        page.contains("function escapeAttribute("),
        "the export carries the shared escaper (single concatenated load order)"
    );
    assert_eq!(
        page.matches("function escapeHtml(").count(),
        1,
        "the export ships exactly one escapeHtml definition"
    );
    assert!(
        !page.contains(HOSTILE_FILENAME),
        "hostile filename must never appear raw in the standalone page"
    );
    let mut page_survivors = interpolated_onclick_payloads(&page);
    page_survivors.sort();
    assert_eq!(
        page_survivors,
        vec![
            "jumpToCrumb(${i})".to_string(),
            "selectNode('${n.id}')".to_string()
        ],
        "the export keeps exactly the two pinned numeric survivors"
    );
}
