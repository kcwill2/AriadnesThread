//! D27 (M22.2c): language/unit_kind on file nodes + declared-vs-defined on
//! symbol nodes. The numeric-FileId `defining_file`/`path` strings are a
//! documented defect (plan §3.6.1 intent): both must become resolved
//! repo-relative paths, with `declaring_files` as the additive declared-in set.

use ariadne_graph::{IncludeGraph, ReducedSemanticModel, ReducedSymbolRecord};
use ariadne_index::SymbolId;
use ariadne_repo::FileId;
use ariadne_ui::{build_include_elements, build_symbol_elements};
use std::collections::{BTreeMap, HashMap};

use crate::common;

fn symbol_model(defining: Option<FileId>, declaring: Vec<FileId>) -> ReducedSemanticModel {
    let id = SymbolId("com.example.OrderService".to_string());
    ReducedSemanticModel {
        repo_root: std::path::PathBuf::from("."),
        symbols: BTreeMap::from([(
            id.clone(),
            ReducedSymbolRecord {
                id,
                name: "OrderService".to_string(),
                kind: ariadne_index::SymbolKind::Class,
                defining_file: defining,
                declarations: vec![],
                definition: None,
                declaring_files: declaring,
            },
        )]),
        edges: vec![],
        tu_usage: BTreeMap::new(),
        file_symbols: BTreeMap::new(),
    }
}

fn path_map(entries: [(u64, &str); 2]) -> HashMap<u64, String> {
    entries
        .into_iter()
        .map(|(id, path)| (id, path.to_string()))
        .collect()
}

fn single_symbol_node(model: &ReducedSemanticModel, paths: &HashMap<u64, String>) -> serde_json::Value {
    let nodes = build_symbol_elements(model, paths);
    nodes
        .iter()
        .find(|e| e.data["id"] == "com.example.OrderService")
        .expect("symbol node")
        .data
        .clone()
}

#[test]
fn d27_symbol_defining_file_is_resolved_path_not_numeric_id() {
    let paths = path_map([
        (2, "src/main/scala/models/Order.scala"),
        (3, "src/main/scala/services/OrderService.scala"),
    ]);
    let data = single_symbol_node(
        &symbol_model(Some(FileId(3)), vec![FileId(2), FileId(3)]),
        &paths,
    );
    assert_eq!(data["defining_file"], "src/main/scala/services/OrderService.scala");
    assert_ne!(data["defining_file"], "3", "D27 defect fix: numeric id string must be gone");
    assert_eq!(
        data["declaring_files"],
        serde_json::json!(["src/main/scala/models/Order.scala", "src/main/scala/services/OrderService.scala"]),
        "declaring files keep id order and resolve to paths"
    );
    assert_eq!(data["path"], "src/main/scala/services/OrderService.scala");
}

#[test]
fn d27_symbol_path_resolution_enables_source_drawer_contract() {
    // file-view.js guards on a truthy node.path; the resolved defining path
    // is what turns a Scala symbol node into a source-drawer-capable node.
    let paths = path_map([(1, "include/widget.hpp"), (9, "src/widget.cpp")]);
    let data = single_symbol_node(&symbol_model(Some(FileId(9)), vec![FileId(1), FileId(9)]), &paths);
    assert_eq!(data["path"], "src/widget.cpp");
    assert!(
        !serde_json::to_string(&data).unwrap().contains("\"9\""),
        "no numeric FileId string may survive in the node data"
    );
}

#[test]
fn d27_unresolvable_ids_degrade_to_null_and_empty_without_panic() {
    let paths = path_map([(1, "a.h"), (2, "b.h")]);
    let data = single_symbol_node(&symbol_model(Some(FileId(404)), vec![FileId(403), FileId(404)]), &paths);
    assert!(data["defining_file"].is_null(), "unresolvable defining id ⇒ null");
    assert_eq!(data["declaring_files"], serde_json::json!([]), "unresolvable ids drop out");
    assert_eq!(data["path"], "", "unresolvable path ⇒ empty string (JS guard contract)");

    let no_defining = single_symbol_node(&symbol_model(None, vec![]), &paths);
    assert!(no_defining["defining_file"].is_null());
    assert_eq!(no_defining["declaring_files"], serde_json::json!([]));
}

#[test]
fn d27_declaring_files_are_deduplicated_after_resolution() {
    let paths = path_map([(5, "pkg/all.scala"), (6, "pkg/Other.scala")]);
    let data = single_symbol_node(
        &symbol_model(Some(FileId(5)), vec![FileId(5), FileId(6), FileId(5)]),
        &paths,
    );
    assert_eq!(
        data["declaring_files"],
        serde_json::json!(["pkg/all.scala", "pkg/Other.scala"])
    );
}

fn one_file_graph() -> IncludeGraph {
    let mut graph = IncludeGraph::new();
    graph
        .file_paths
        .insert(FileId(4), std::path::PathBuf::from("src/main/scala/Main.scala"));
    graph
}

#[test]
fn d27_file_nodes_emit_language_and_unit_kind_from_injected_maps() {
    let languages: HashMap<u64, String> =
        [(4u64, "Scala".to_string())].into_iter().collect();
    let unit_kinds: HashMap<u64, String> =
        [(4u64, "SourceFile".to_string())].into_iter().collect();
    let elements = build_include_elements(&one_file_graph(), &HashMap::new(), &[], &languages, &unit_kinds);
    let file_node = &elements[0];
    assert_eq!(file_node.data["language"], "Scala");
    assert_eq!(file_node.data["unit_kind"], "SourceFile");
    assert_eq!(file_node.data["kind"], "SOURCE", "existing extension-based kind keeps working");
}

#[test]
fn d27_file_node_decoration_is_omitted_when_map_entry_absent() {
    // Locked default for missing entries: the key is OMITTED (honest absence),
    // never a fabricated "UNKNOWN" on files discovery never classified.
    let elements = build_include_elements(
        &one_file_graph(),
        &HashMap::new(),
        &[],
        &HashMap::new(),
        &HashMap::new(),
    );
    assert!(elements[0].data.get("language").is_none());
    assert!(elements[0].data.get("unit_kind").is_none());
}

// ---------------------------------------------------------------------------
// Asset surface (D27 inspector): red via disk reads + live route before the
// provenance asset exists, mirroring the preview.js (E-I6) precedent.
// ---------------------------------------------------------------------------

fn provenance_asset_text() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/provenance.js");
    std::fs::read_to_string(path).unwrap_or_default()
}

#[test]
fn d27_provenance_asset_renders_declared_vs_defined_rows() {
    let asset = provenance_asset_text();
    assert!(
        asset.contains("function renderNodeProvenance("),
        "provenance asset must own a named inspector renderer"
    );
    assert!(
        asset.contains("Defined in:") && asset.contains("Declared in:"),
        "symbol rows: Defined in … / Declared in … ({{n}})"
    );
    assert!(
        asset.contains("Language:"),
        "file rows show the language when the payload carries one"
    );
    assert!(
        asset.contains("declaring_files") && asset.contains("defining_file"),
        "rows must read the D27 payload fields"
    );
    assert!(
        asset.contains("escapeHtml("),
        "path values must be HTML-escaped like every other inspector consumer"
    );
    assert!(
        asset.contains("MutationObserver"),
        "the renderer mounts on an inspector-title mutation so graph.js stays frozen (R2 cap 990)"
    );
}

#[test]
fn d27_index_markup_mounts_provenance_section_and_script() {
    let html = ariadne_ui::INDEX_HTML;
    assert!(
        html.contains("id=\"inspProvenanceSection\""),
        "inspector must carry the provenance section container"
    );
    assert!(
        html.contains("<script src=\"/app/provenance.js\"></script>"),
        "provenance asset must load as a classic script tag"
    );
}

#[tokio::test]
async fn d27_provenance_route_serves_the_asset() {
    let app = ariadne_ui::build_router(common::loading_state());
    let (status, headers, body) =
        common::request_raw_body(app, "/app/provenance.js").await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(headers.get("Cache-Control").unwrap(), "no-store");
    assert_eq!(
        headers.get("Content-Type").unwrap(),
        "text/javascript; charset=utf-8"
    );
    assert!(
        String::from_utf8_lossy(&body).contains("function renderNodeProvenance("),
        "the route must serve the exact bundled asset"
    );
}

#[test]
fn d27_standalone_export_inlines_provenance() {
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("d27-standalone.html");
    ariadne_ui::export_standalone_html(&common::empty_export_payload(), "", &out_path).unwrap();
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(
        content.contains("function renderNodeProvenance("),
        "the standalone report must inline the provenance renderer (I5 both-surface rule)"
    );
    assert!(
        !content.contains("src=\"/app/provenance.js\""),
        "the standalone report must not reference the asset route"
    );
}
