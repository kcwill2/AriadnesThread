//! AR-01 security locks (S1 CRIT path containment, S3 HIGH / S4-partial
//! export script escape).
//!
//! `/api/source` must never read outside the canonical repo root: absolute
//! `file=` values, `..` traversal, and escaping `local_prefix=` are rejected,
//! while contained (legitimate) reads stay byte-identical. The standalone
//! export must never embed raw `<`, U+2028, or U+2029 in the
//! `window.__ARIADNE_DATA__` script: that shareable artifact would otherwise
//! break out of its `<script>` tag (or the JS line, for the separators) in
//! every reader's browser.

use crate::common;
use ariadne_ui::build_router;
use axum::http::StatusCode;

const OUTSIDE_SECRET: &str = "OUTSIDE-REPO-SECRET";
const PREFIX_SECRET: &str = "LOCAL-PREFIX-SECRET";
const WIDGET_BODY: &[u8] = b"// widget source\n";
const LINE_SEPARATOR_BYTES: [u8; 3] = [0xE2, 0x80, 0xA8];
const PARAGRAPH_SEPARATOR_BYTES: [u8; 3] = [0xE2, 0x80, 0xA9];

/// S1 fixture: the served `repo/` contains `src/widget.cc`; the sibling
/// `outside/` tree holds the payloads an escape attempt would reach --
/// `outside/secret.txt` via absolute `file=` or `..` traversal, and
/// `outside/widget.cc` via an escaping `local_prefix=`.
fn source_containment_app() -> (tempfile::TempDir, axum::Router) {
    let sandbox = tempfile::tempdir().expect("sandbox temp dir");
    let repo = sandbox.path().join("repo");
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::write(repo.join("src/widget.cc"), WIDGET_BODY).unwrap();
    std::fs::create_dir_all(sandbox.path().join("outside")).unwrap();
    std::fs::write(
        sandbox.path().join("outside/secret.txt"),
        format!("{OUTSIDE_SECRET}\n").as_bytes(),
    )
    .unwrap();
    std::fs::write(
        sandbox.path().join("outside/widget.cc"),
        format!("{PREFIX_SECRET}\n").as_bytes(),
    )
    .unwrap();

    let state = common::ready_state(
        repo,
        ariadne_graph::IncludeGraph::new(),
        ariadne_graph::CallGraph::default(),
        common::empty_export_payload(),
    );
    (sandbox, build_router(state))
}

/// Percent-encode every reserved byte so any raw path (Windows separators,
/// `..`, `:`) reaches the router verbatim after decoding, on every platform.
fn encode_query_component(raw: &str) -> String {
    let mut encoded = String::new();
    for byte in raw.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(*byte as char)
            }
            reserved => encoded.push_str(&format!("%{reserved:02X}")),
        }
    }
    encoded
}

fn assert_no_outside_content_leaked(body: &[u8]) {
    let rendered = String::from_utf8_lossy(body).into_owned();
    assert!(
        !rendered.contains(OUTSIDE_SECRET) && !rendered.contains(PREFIX_SECRET),
        "a rejected read must not echo outside-repo content, got {rendered:?}"
    );
}

#[tokio::test]
async fn test_source_route_serves_contained_path_byte_identical() {
    let (_sandbox, app) = source_containment_app();
    let (status, _headers, body) =
        common::request_raw_body(app, "/api/source?file=src/widget.cc").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        WIDGET_BODY.to_vec(),
        "contained (legitimate) reads stay byte-identical"
    );
}

#[tokio::test]
async fn test_source_route_rejects_absolute_path_outside_repo() {
    let (sandbox, app) = source_containment_app();
    let escape_target = sandbox.path().join("outside/secret.txt");
    let uri = format!(
        "/api/source?file={}",
        encode_query_component(&escape_target.to_string_lossy())
    );
    let (status, _headers, body) = common::request_raw_body(app, &uri).await;
    assert_ne!(
        status,
        StatusCode::OK,
        "an absolute file= outside the repo root must be rejected (S1)"
    );
    assert_no_outside_content_leaked(&body);
}

#[tokio::test]
async fn test_source_route_rejects_parent_traversal_outside_repo() {
    let (_sandbox, app) = source_containment_app();
    let uri = format!(
        "/api/source?file={}",
        encode_query_component("../outside/secret.txt")
    );
    let (status, _headers, body) = common::request_raw_body(app, &uri).await;
    assert_ne!(
        status,
        StatusCode::OK,
        "a `..` traversal escaping the repo root must be rejected (S1)"
    );
    assert_no_outside_content_leaked(&body);
}

#[tokio::test]
async fn test_source_route_rejects_local_prefix_escaping_repo() {
    let (sandbox, app) = source_containment_app();
    let uri = format!(
        "/api/source?file={}&local_prefix={}",
        encode_query_component("widget.cc"),
        encode_query_component(&sandbox.path().join("outside").to_string_lossy())
    );
    let (status, _headers, body) = common::request_raw_body(app, &uri).await;
    assert_ne!(
        status,
        StatusCode::OK,
        "a local_prefix that resolves outside the repo root must be rejected (S1)"
    );
    assert_no_outside_content_leaked(&body);
}

#[tokio::test]
async fn test_source_route_local_prefix_equal_to_root_still_maps() {
    let (sandbox, app) = source_containment_app();
    let repo = sandbox.path().join("repo");
    let uri = format!(
        "/api/source?file={}&local_prefix={}",
        encode_query_component(&repo.join("src/widget.cc").to_string_lossy()),
        encode_query_component(&repo.to_string_lossy())
    );
    let (status, _headers, body) = common::request_raw_body(app, &uri).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "cross-environment mapping (local_prefix == server root) must keep working"
    );
    assert_eq!(body, WIDGET_BODY.to_vec());
}

// ---------------------------------------------------------------------------
// S3 HIGH / S4-partial: embedded export JSON must carry no raw script-
// breakout characters, and clean payloads must not shift a single byte.
// ---------------------------------------------------------------------------

#[test]
fn test_export_embeds_json_without_raw_script_breakout_chars() {
    let mut payload = common::empty_export_payload();
    // Hostile fixture built from Unicode escapes only (pure-ASCII source rule).
    let hostile = format!("<svg onload=alert(1)>\u{2028}\u{2029}");
    payload.repo_name = hostile.clone();

    let export_dir = tempfile::tempdir().unwrap();
    let export_path = export_dir.path().join("hostile-export.html");
    ariadne_ui::export_standalone_html(&payload, "/*__ARIADNE_DATA__*/", &export_path)
        .expect("export writes the standalone page");

    let written = std::fs::read(&export_path).unwrap();
    assert!(
        !written.contains(&b'<'),
        "the embedded payload must carry no raw '<' (script tag breakout, S3)"
    );
    for separator in [LINE_SEPARATOR_BYTES, PARAGRAPH_SEPARATOR_BYTES] {
        assert!(
            !written.windows(3).any(|window| window == separator),
            "the embedded payload must carry no raw U+2028/U+2029 (JS line terminators, S3)"
        );
    }

    let text = String::from_utf8(written).expect("export is UTF-8");
    let json_slice = text
        .strip_prefix("window.__ARIADNE_DATA__ = ")
        .and_then(|rest| rest.strip_suffix(";"))
        .expect("template placeholder was replaced by the data script");
    let round_trip: serde_json::Value =
        serde_json::from_str(json_slice).expect("escaped JSON still parses");
    assert_eq!(
        round_trip["repo_name"].as_str(),
        Some(hostile.as_str()),
        "escaping is encoding-only: the parsed payload is unchanged"
    );
}

#[test]
fn test_export_leaves_clean_json_byte_identical() {
    let payload = common::empty_export_payload();
    let export_dir = tempfile::tempdir().unwrap();
    let export_path = export_dir.path().join("clean-export.html");
    ariadne_ui::export_standalone_html(&payload, "/*__ARIADNE_DATA__*/", &export_path).unwrap();

    let text = std::fs::read_to_string(&export_path).unwrap();
    let expected = format!(
        "window.__ARIADNE_DATA__ = {};",
        serde_json::to_string(&payload).unwrap()
    );
    assert_eq!(
        text, expected,
        "a clean payload must serialize exactly as before the escape (rider lock)"
    );
}

#[test]
fn test_hotspots_asset_ships_attribute_safe_escaper() {
    assert!(
        ariadne_ui::HOTSPOTS_JS.contains("function escapeAttribute"),
        "the AR-01 attribute-safe escaper must ship in the shared renderer (S4-partial)"
    );
}
