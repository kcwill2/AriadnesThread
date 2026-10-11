//! AR-15 locks (P5 + P6 MED): the `/api/export` payload is served from a
//! precomputed snapshot (`Bytes` + `ETag`) instead of a per-request deep
//! clone. This file captures the payload BYTES of today's serialization
//! first (characterization), then locks the ETag semantics computed over
//! exactly those bytes.
//!
//! Wire-format authority: axum 0.7's `Json` renders with compact
//! `serde_json` into the body and sets `content-type: application/json`,
//! so `serde_json::to_vec(&payload)` plus that content type is exactly
//! what the pre-change deep-clone path served. The first test locks the
//! route to it BEFORE and AFTER the snapshot change (constraint: payload
//! bytes identical to today's serialization).

use crate::common;
use ariadne_ui::{
    build_router, ExtractionProposalDto, GraphExportPayload, JustificationEntry, SummaryStats,
    UiState,
};
use axum::body::Body;
use axum::http::{header, HeaderMap, Request, StatusCode};
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceExt;

/// Reference FNV-1a/64 (offset basis 0xcbf29ce484222325, prime
/// 0x100000001b3). This test-local copy is the SPEC the served `ETag`
/// must match: xor-then-multiply per byte, lowercase 16-hex, wrapped in
/// RFC-7232 strong-validator quotes. Multiplication by the odd prime is
/// injective, so two inputs differing in one byte can never collide.
fn fnv1a_64_reference(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn etag_of(bytes: &[u8]) -> String {
    format!("\"{:016x}\"", fnv1a_64_reference(bytes))
}

/// Deterministic fixture payload: sorted-emission maps (dto's
/// serialize_sorted_nested_maps), a non-empty proposal, and include-graph
/// elements built from the shared fixture graph. Every field order is
/// construction-determined, so the serialized bytes are byte-stable
/// across runs and processes.
fn ar15_export_payload() -> GraphExportPayload {
    let (_fixture_graph, mut payload) = common::fixture_include_graph();
    payload.repo_name = "ar15-fixture-repo".to_string();
    payload.stats = SummaryStats {
        total_files: 4,
        total_symbols: 7,
        total_calls: 3,
        cycle_count: 1,
    };

    let mut burden = HashMap::new();
    burden.insert("2".to_string(), 5usize);
    burden.insert("3".to_string(), 11usize);
    payload.file_collateral.insert("1".to_string(), burden.clone());
    payload.file_collateral.insert("10".to_string(), burden);

    let entry = JustificationEntry {
        justified_symbols: vec!["main".to_string()],
        collateral_symbols: vec!["helper".to_string()],
        justified_count: 1,
        collateral_count: 1,
        justified_truncated: false,
        collateral_truncated: false,
        required_headers: vec!["b.h".to_string()],
        impact: None,
    };
    let mut per_source = HashMap::new();
    per_source.insert("2".to_string(), entry);
    payload.file_justification.insert("1".to_string(), per_source);

    payload.extraction_proposals = vec![ExtractionProposalDto {
        plan_id: "ar15-plan-1".to_string(),
        target_file: "extracted.h".to_string(),
        target_file_id: "2".to_string(),
        problem: "helper drags collateral into 3 TUs".to_string(),
        collateral_load_before: 16,
        collateral_load_after: 2,
        symbol_names: vec!["helper".to_string()],
        total_symbols: 1,
        required_headers: vec!["b.h".to_string()],
        consumer_tu_count: 3,
        risk: "low".to_string(),
        target_files: vec![],
        target_files_truncated: false,
    }];
    payload
}

async fn get_export(app: axum::Router, if_none_match: Option<&str>) -> (StatusCode, HeaderMap, Vec<u8>) {
    let mut builder = Request::builder().uri("/api/export");
    if let Some(tag) = if_none_match {
        builder = builder.header(header::IF_NONE_MATCH, tag);
    }
    let response = app.oneshot(builder.body(Body::empty()).unwrap()).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    (status, headers, body)
}

/// Characterization, captured BEFORE the snapshot change: the served
/// export body is byte-identical to `serde_json::to_vec(&payload)` (the
/// deep-clone wire), and its FNV-1a/64 hash is the locked literal.
#[tokio::test]
async fn test_export_payload_bytes_match_locked_serialization_and_hash() {
    let payload = ar15_export_payload();
    let serialized = serde_json::to_vec(&payload).expect("fixture payload must serialize");
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        ariadne_graph::IncludeGraph::new(),
        ariadne_graph::CallGraph::default(),
        payload,
    ));

    let (status, headers, body) = get_export(app, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/json"),
        "export content-type must stay exactly what the Json path served"
    );
    assert_eq!(
        body, serialized,
        "export body bytes must stay identical to today's serialization"
    );
    // LOCK: hash of the exact bytes the pre-change path served. Captured
    // from the route (not recomputed) so a wire-format drift trips here.
    let computed = format!("{:016x}", fnv1a_64_reference(&body));
    assert_eq!(
        computed, "993e43a23e1c6fe5",
        "export payload-bytes characterization hash drifted"
    );
}

/// The ETag is computed over exactly the served bytes, in the spec format
/// the test carries as its own reference implementation.
#[tokio::test]
async fn test_export_etag_header_locks_the_served_bytes() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        ariadne_graph::IncludeGraph::new(),
        ariadne_graph::CallGraph::default(),
        ar15_export_payload(),
    ));

    let (status, headers, body) = get_export(app.clone(), None).await;
    assert_eq!(status, StatusCode::OK);
    let etag = headers
        .get(header::ETAG)
        .and_then(|v| v.to_str().ok())
        .expect("export must carry an ETag validator")
        .to_string();
    assert_eq!(etag, etag_of(&body), "ETag must hash the served bytes");
}

/// Unchanged payload -> 304: a repeat poll carrying the exact ETag back
/// gets an empty-body NOT MODIFIED with the same validator.
#[tokio::test]
async fn test_export_unchanged_payload_serves_304_for_exact_if_none_match() {
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        ariadne_graph::IncludeGraph::new(),
        ariadne_graph::CallGraph::default(),
        ar15_export_payload(),
    ));

    let (_status, headers, body) = get_export(app.clone(), None).await;
    let etag = headers.get(header::ETAG).unwrap().to_str().unwrap().to_string();

    let (status, headers_304, body_304) = get_export(app, Some(&etag)).await;
    assert_eq!(
        status,
        StatusCode::NOT_MODIFIED,
        "exact If-None-Match on an unchanged payload must yield 304"
    );
    assert!(body_304.is_empty(), "304 must carry no body, got {body_304:?}");
    assert_eq!(
        headers_304.get(header::ETAG).and_then(|v| v.to_str().ok()),
        Some(etag.as_str()),
        "304 must repeat the validator (RFC 7232)"
    );
    let _ = body;
}

/// A stale/mismatched/`*` validator never earns a 304: the full first-
/// request path (status, bytes, content type) stays byte-identical.
#[tokio::test]
async fn test_export_if_none_match_mismatch_serves_full_body() {
    let payload = ar15_export_payload();
    let serialized = serde_json::to_vec(&payload).expect("fixture payload must serialize");
    let app = build_router(common::ready_state(
        std::path::PathBuf::from("."),
        ariadne_graph::IncludeGraph::new(),
        ariadne_graph::CallGraph::default(),
        payload,
    ));

    for stale_tag in ["\"0000000000000000\"", "*"] {
        let (status, _headers, body) = get_export(app.clone(), Some(stale_tag)).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "If-None-Match {stale_tag} is not an exact match and must serve 200"
        );
        assert_eq!(body, serialized, "mismatched validator must serve full bytes");
    }
}

/// Mutation non-vacuity (contract: bump one byte -> ETag changes -> 304
/// not served): a rebuild whose payload differs in exactly one byte of
/// repo_name must produce a fresh 200 with a different validator against
/// the previous ETag. The rebuild is simulated the way the standing
/// server_api lock simulates it (payload write + install_analysis).
#[tokio::test]
async fn test_export_etag_changes_when_one_byte_changes_and_304_is_not_served() {
    let state: Arc<UiState> = common::ready_state(
        std::path::PathBuf::from("."),
        ariadne_graph::IncludeGraph::new(),
        ariadne_graph::CallGraph::default(),
        ar15_export_payload(),
    );
    let app = build_router(state.clone());

    let (_status, headers, _body) = get_export(app.clone(), None).await;
    let old_tag = headers
        .get(header::ETAG)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // One-byte payload mutation + generation install (watcher-swap shape).
    let mut mutated = ar15_export_payload();
    mutated.repo_name = "ar15-fixture-rep0".to_string(); // one byte differs
    {
        *state.payload.write().unwrap() = Some(mutated);
        state.install_analysis(
            ariadne_graph::IncludeGraph::new(),
            common::empty_reduced_model(),
            Arc::new(ariadne_graph::LoadMemo::new()),
        );
    }

    let (status, headers_after, body_after) = get_export(app, Some(&old_tag)).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "a changed payload must never answer 304 to the previous ETag"
    );
    let new_tag = headers_after
        .get(header::ETAG)
        .and_then(|v| v.to_str().ok())
        .expect("changed export must carry an ETag")
        .to_string();
    assert_ne!(new_tag, old_tag, "one-byte payload change must move the ETag");
    assert_eq!(
        new_tag,
        etag_of(&body_after),
        "the fresh ETag must hash the fresh served bytes"
    );
}

/// Spec-level mutation lock for the hash itself: the FNV reference is
/// single-byte sensitive (the injectivity argument), so no payload edit
/// can keep the validator while changing the bytes.
#[test]
fn test_etag_hash_is_single_byte_mutation_sensitive() {
    let body = serde_json::to_vec(&ar15_export_payload()).unwrap();
    let mut bumped = body.clone();
    let last = bumped.len() - 1;
    bumped[last] ^= 0x01;
    assert_ne!(
        fnv1a_64_reference(&body),
        fnv1a_64_reference(&bumped),
        "one bumped byte must move the FNV-1a/64 value"
    );
    assert_eq!(
        fnv1a_64_reference(&body),
        fnv1a_64_reference(&body.clone()),
        "identical bytes must hash identically"
    );
}

/// No-deep-clone lock (P5): steady requests are served from the ONE
/// published snapshot - the serialize counter (the only remaining
/// per-generation serialization site) moves exactly once per generation,
/// never once per request. Every body served while the counter is flat
/// therefore comes from the same Arc'd snapshot bytes, so the pre-change
/// per-request `payload.clone()` peak is gone by construction.
#[tokio::test]
async fn test_steady_export_requests_serve_snapshot_clones_not_reserialization() {
    let state: Arc<UiState> = common::ready_state(
        std::path::PathBuf::from("."),
        ariadne_graph::IncludeGraph::new(),
        ariadne_graph::CallGraph::default(),
        ar15_export_payload(),
    );
    let app = build_router(state.clone());

    let (_status, headers, _body) = get_export(app.clone(), None).await;
    assert_eq!(
        state.export_serialization_count(),
        1,
        "the first request serializes the generation exactly once"
    );
    let etag = headers.get(header::ETAG).unwrap().to_str().unwrap().to_string();

    // A poll-tick mix: full fetches and 304-conditional fetches alike.
    for _ in 0..3 {
        get_export(app.clone(), None).await;
        get_export(app.clone(), Some(&etag)).await;
    }
    assert_eq!(
        state.export_serialization_count(),
        1,
        "steady polls must serve snapshot clones, not a fresh serialization"
    );

    // One simulated rebuild (payload write + install, watcher shape): the
    // next request performs exactly ONE more serialization, then holds.
    let mut mutated = ar15_export_payload();
    mutated.repo_name = "ar15-fixture-rep1".to_string();
    {
        *state.payload.write().unwrap() = Some(mutated);
        state.install_analysis(
            ariadne_graph::IncludeGraph::new(),
            common::empty_reduced_model(),
            Arc::new(ariadne_graph::LoadMemo::new()),
        );
    }
    get_export(app.clone(), None).await;
    assert_eq!(
        state.export_serialization_count(),
        2,
        "one rebuild must cost exactly one re-serialization"
    );
    for _ in 0..3 {
        get_export(app.clone(), None).await;
    }
    assert_eq!(
        state.export_serialization_count(),
        2,
        "the rebuilt generation must stay on its published snapshot"
    );
}
