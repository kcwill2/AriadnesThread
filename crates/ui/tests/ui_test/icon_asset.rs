//! Brand-asset locks for the served brand icon (track branding-assets,
//! handoff T-A). The canonical brand art (source of truth
//! `media/AT-svg.svg`) ships as a first-class UI asset: the copy stays
//! byte-identical to the source of truth (I3), the page head carries the
//! favicon link plus an `<img>` brand mark and nothing else of the sort
//! (I6/I7 markup half), and the standalone export inlines the art while
//! closing every icon route reference against the `file://` closed world
//! (I6). The route contract itself lives in
//! `test_icon_route_serves_brand_art_with_no_store` below, mirroring the
//! raw-request style of `d27_node_provenance.rs`. Splash branding rides the
//! same asset: the loading overlay IS the brand scene - a giant watermark
//! over a token-painted labyrinth grid with a version corner and nothing
//! else brand-shaped
//! (`test_splash_screen_paints_watermark_grid_and_version_corner`), its one
//! CSS animation respects reduced motion
//! (`test_splash_motion_respects_reduced_motion`), the build version is
//! injected into served and exported pages, and the standalone export drops
//! every splash route reference as dead under file://
//! (`test_standalone_export_closes_splash_brand_reference`).

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
fn test_splash_screen_paints_watermark_grid_and_version_corner() {
    // The watermark IS the brand: the overlay fills its space with the art
    // at 15% over a token-painted labyrinth grid and a radial pool of light,
    // plus a build-version corner line and nothing else brand-shaped (the
    // small lockup was retired - simplicity pass). The overlay's paint
    // lives in the stylesheet (an inline background would outrank it).
    let page = ariadne_ui::INDEX_HTML;
    let overlay_start = page
        .find("id=\"loadingOverlay\"")
        .expect("the loading overlay must exist");
    let watermark_tag = "<img class=\"splash-watermark\" src=\"/icon.svg\" alt=\"\">";
    let watermark = page
        .find(watermark_tag)
        .expect("the overlay must carry the giant watermark <img>");
    assert!(
        watermark > overlay_start,
        "the watermark must live inside the loading overlay"
    );
    let progress_row = page
        .find("id=\"loadingProgress\"")
        .expect("the progress bar must exist");
    assert!(
        watermark < progress_row,
        "the watermark must sit ahead of the progress rows in overlay order"
    );
    assert!(
        !page.contains("splash-mark")
            && !page.contains("splash-wordmark")
            && !page.contains("splash-brand"),
        "the small icon and wordmark must be gone - the watermark is the only brand"
    );
    let overlay_head = &page[overlay_start..watermark];
    assert!(
        !overlay_head.contains("background: var(--bg-surface)"),
        "the overlay background must move to the stylesheet to allow layered paint"
    );
    assert!(
        page.contains("class=\"splash-track\""),
        "the progress track row must carry the shimmer class"
    );
    assert!(
        page.contains("<div class=\"splash-version\">v__ARIADNE_VERSION__</div>"),
        "the overlay corner must carry the version placeholder"
    );
    assert!(
        !page.contains("neonGradient"),
        "the watermark must be served art, never inlined into the app page"
    );
    let styles = ariadne_ui::STYLES_CSS;
    for token in [
        "#loadingOverlay {",
        "radial-gradient(",
        "repeating-linear-gradient(",
        ".splash-watermark {",
        "opacity: 0.15;",
        "z-index: -1;",
        ".splash-version {",
    ] {
        assert!(
            styles.contains(token),
            "the stylesheet must carry the splash paint rule token: {token}"
        );
    }
    assert!(
        !styles.contains(".splash-mark")
            && !styles.contains(".splash-wordmark")
            && !styles.contains(".splash-brand"),
        "the retired lockup rules must be gone from the stylesheet"
    );
}

#[test]
fn test_splash_motion_respects_reduced_motion() {
    // Simplicity pass: the shimmer sweep is the splash's only motion (the
    // mark pulse died with the small lockup) and it bows to the user's
    // reduced-motion preference.
    let styles = ariadne_ui::STYLES_CSS;
    assert!(
        styles.contains("@keyframes splash-shimmer")
            && styles.contains("animation: splash-shimmer")
            && styles.contains(".splash-track::after"),
        "the track shimmer sweep must stay wired"
    );
    assert!(
        !styles.contains("splash-breathe"),
        "the retired mark-pulse keyframes must be gone"
    );
    assert!(
        styles.contains("prefers-reduced-motion"),
        "the shimmer must bow to the reduced-motion preference"
    );
}

#[tokio::test]
async fn test_served_index_injects_the_build_version() {
    let app = ariadne_ui::build_router(common::loading_state());
    let (status, _, body) = common::request_raw_body(app, "/").await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let page = String::from_utf8(body).expect("the served page is UTF-8");
    let versioned = format!("v{}", env!("CARGO_PKG_VERSION"));
    assert!(
        page.contains(&versioned),
        "the served splash must carry the crate version"
    );
    assert!(
        !page.contains("__ARIADNE_VERSION__"),
        "no version placeholder may survive into the served page"
    );
}

#[test]
fn test_standalone_export_injects_version_without_placeholders() {
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("versioned-standalone.html");
    ariadne_ui::export_standalone_html(&common::empty_export_payload(), "", &out_path).unwrap();
    let content = std::fs::read_to_string(&out_path).unwrap();
    assert!(
        content.contains(&format!("v{}", env!("CARGO_PKG_VERSION"))),
        "the export splash must carry the crate version too"
    );
    assert!(
        !content.contains("__ARIADNE_VERSION__"),
        "the export must not ship the raw placeholder"
    );
}

#[test]
fn test_standalone_export_closes_splash_brand_reference() {
    // file:// has no /icon.svg route and the overlay never shows there (no
    // /status feed): the export drops the splash mark instead of inlining a
    // second copy of the art (which would duplicate neonGradient/neonGlow).
    let temp_dir = tempfile::tempdir().unwrap();
    let out_path = temp_dir.path().join("splash-standalone.html");
    ariadne_ui::export_standalone_html(&common::empty_export_payload(), "", &out_path).unwrap();
    let content = std::fs::read_to_string(&out_path).unwrap();
    // The `.splash-mark` stylesheet rule legitimately survives inlining; the
    // closed-world hazard is only the route reference itself.
    assert!(
        !content.contains("src=\"/icon.svg\""),
        "the standalone export must not leave the splash mark as a dangling route reference"
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
