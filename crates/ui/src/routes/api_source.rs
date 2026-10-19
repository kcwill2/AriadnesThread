//! AR-13 extraction (verbatim from server.rs): source-file access family:
//! `/api/source` (sandboxed file reads) and `/api/repo-info` (path mapping
//! for cross-environment navigation).

use crate::state::UiState;
use axum::extract::{Query, State};
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub(crate) struct SourceQuery {
    file: String,
    /// Optional local path prefix to map server paths to local paths.
    /// If provided, the server strips its own `repo_root` prefix and
    /// prepends `local_prefix` before reading the file.
    local_prefix: Option<String>,
}

pub(crate) async fn api_source(
    State(state): State<Arc<UiState>>,
    Query(query): Query<SourceQuery>,
) -> Result<String, axum::http::StatusCode> {
    let candidate = if let Some(prefix) = &query.local_prefix {
        // Cross-environment navigation: strip server repo_root, apply local
        // prefix. The containment below sandboxes the result INTO the repo:
        // an escaping prefix is rejected, never followed.
        let server_root = &state.repo_root;
        let requested = std::path::Path::new(&query.file);
        let relative = requested.strip_prefix(server_root).unwrap_or(requested);
        let local_path = std::path::Path::new(prefix)
            .join(relative.to_string_lossy().trim_start_matches('/'));
        local_path
    } else {
        state.repo_root.join(&query.file)
    };
    // S1 containment (AR-01): canonicalize BOTH ends and require
    // `starts_with`, so absolute `file=` values, `..` traversal, escaping
    // `local_prefix=`, and in-repo symlinks that resolve outside all land
    // outside the canonical root and are rejected. Canonicalization failure
    // means a missing path: the same 404 the read produced before. Contained
    // reads keep the exact pre-fix bytes (locked by source_containment).
    let canonical_root = tokio::fs::canonicalize(&state.repo_root)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let contained_path = tokio::fs::canonicalize(&candidate)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)?;
    if !contained_path.starts_with(&canonical_root) {
        return Err(axum::http::StatusCode::FORBIDDEN);
    }
    tokio::fs::read_to_string(contained_path)
        .await
        .map_err(|_| axum::http::StatusCode::NOT_FOUND)
}

/// Repository metadata endpoint: exposes the server's `repo_root` and
/// file-path mapping so extension hosts can reconcile server-side paths
/// with their own local source tree (cross-environment navigation).
#[derive(serde::Serialize)]
pub struct RepoInfoResponse {
    /// Absolute path on the server where the analysis was generated.
    pub repo_root: String,
    /// File paths relative to `repo_root` (as the server sees them).
    pub file_paths: Vec<String>,
}

pub(crate) async fn api_repo_info(
    State(state): State<Arc<UiState>>,
) -> Json<RepoInfoResponse> {
    let analysis = state.analysis();
    // UTB-3 (UTA2-R1, journal: tasks/uta2-r1-journal.md): `file_paths` must
    // carry the documented repo-relative PATH values - the endpoint exists
    // so extension hosts can reconcile server paths with their own tree;
    // numeric FileId keys meant nothing outside the server. Sorted for
    // emission determinism (the same nondeterminism class as UTB-2). A path
    // outside `repo_root` keeps its full server-side string. Consumer
    // greps (extension host, ui assets, ui_test, integration, tools):
    // nothing reads the old numeric values.
    let file_paths = analysis
        .as_ref()
        .map(|gen| {
            let mut paths: Vec<String> = gen
                .include_graph
                .file_paths
                .values()
                .map(|path| {
                    path.strip_prefix(&state.repo_root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .into_owned()
                })
                .collect();
            paths.sort();
            paths
        })
        .unwrap_or_default();

    Json(RepoInfoResponse {
        repo_root: state.repo_root.to_string_lossy().into_owned(),
        file_paths,
    })
}
