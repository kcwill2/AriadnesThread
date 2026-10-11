//! Brand-asset locks for the served brand icon (track branding-assets,
//! handoff T-A). The canonical brand art (source of truth
//! `media/AT-svg.svg`) ships as a first-class UI asset: the copy stays
//! byte-identical to the source of truth (I3), the page head carries the
//! favicon link plus an `<img>` brand mark and nothing else of the sort
//! (I6/I7 markup half), and the standalone export inlines the art while
//! closing every icon route reference against the `file://` closed world
//! (I6). The route contract itself lives in
//! `test_icon_route_serves_brand_art_with_no_store` below, mirroring the
//! raw-request style of `d27_node_provenance.rs`.

use crate::common;

#[tokio::test]
async fn test_icon_route_serves_brand_art_with_no_store() {
    // I1: the shared ui_asset() builder carries the no-store promise (incident
    // E3); the served body is the embedded asset byte-for-byte.
    let app = ariadne_ui::build_router(common::loading_state());
    let (status, headers, body) = common::request_raw_body(app, "/icon.svg").await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(headers.get("Cache-Control").unwrap(), "no-store");
    assert_eq!(headers.get("Content-Type").unwrap(), "image/svg+xml");
    assert_eq!(
        body,
        ariadne_ui::ICON_SVG.as_bytes(),
        "the route must serve the exact embedded brand asset"
    );
}

#[test]
fn test_icon_asset_matches_source_of_truth() {
    // D2: the served path uses the d27 precedent join form ("assets/..."
    // under the manifest dir); the handoff's "../assets/..." expression
    // resolves outside crates/ui and can never exist.
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let served = manifest_dir.join("assets").join("icon.svg");
    assert!(
        served.exists(),
        "crates/ui/assets/icon.svg (the brand asset copied from media/AT-svg.svg) must exist"
    );
    let source_of_truth = manifest_dir.join("../../media/AT-svg.svg");
    assert_eq!(
        std::fs::read(served).unwrap(),
        std::fs::read(source_of_truth).unwrap(),
        "the served asset must stay byte-identical to the source of truth"
    );
}

#[test]
fn test_page_head_marks_favicon_and_brand_img() {
    let page = ariadne_ui::INDEX_HTML;
    assert!(
        page.contains("<link rel=\"icon\" type=\"image/svg+xml\" href=\"/icon.svg\">"),
        "the page head must link the served brand favicon"
    );
    assert!(
        page.contains(
            "<img class=\"brand-mark\" src=\"/icon.svg\" alt=\"Ariadne's Thread logo\">"
        ),
        "the header mark must be the brand icon carried as an <img>, never inlined"
    );
    assert!(
        !page.contains("d=\"M13 10V3L4 14h7v7l9-11h-7z\""),
        "the retired bolt path must be gone from the page markup"
    );
    assert!(
        !page.contains("neonGradient") && !page.contains("neonGlow"),
        "the brand svg ids live only in the served asset, keeping the page id-collision-free"
    );
    let styles = ariadne_ui::STYLES_CSS;
    assert!(
        styles.contains(".logo-badge img"),
        "the badge rule must style the img mark"
    );
    assert!(
        !styles.contains(".logo-badge svg"),
        "the retired bolt-only rule must be gone from the stylesheet"
    );
}

#[test]
fn test_standalone_export_closes_icon_references() {
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("icon-standalone.html");
    ariadne_ui::export_standalone_html(&common::empty_export_payload(), "", &out_path).unwrap();
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(
        content.contains("neonGradient"),
        "the standalone export must inline the brand art (file:// has no asset routes)"
    );
    // The surviving closed-world locks, re-asserted here so the whole set
    // lives in this new file; the originals in static_html.rs stay UNEDITED.
    for closed_world_reference in [
        "/icon.svg",
        "<img class=\"brand-mark\"",
        "<link rel=\"icon\"",
        "src=\"/app/",
        "href=\"/styles.css\"",
        "href=\"/themes.css\"",
    ] {
        assert!(
            !content.contains(closed_world_reference),
            "the standalone export must not carry a {closed_world_reference} reference"
        );
    }
}
