use anyhow::{Context, Result};
use ariadne_index::CURRENT_ANALYSIS_VERSION;
use ariadne_repo::Repository;
use std::fs;
use std::path::{Path, PathBuf};

mod apply_io;
mod bincode_options;
mod discovery_io;
mod graph_io;
mod index_io;
mod metrics_io;
mod transform_io;

#[cfg(test)]
mod m22_manifest_languages_tests;
#[cfg(test)]
mod payload_reshape_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod uta_t6_pin_tests;

/// An artifact that cannot be trusted: missing manifest, mismatched input
/// fingerprint, mismatched repository root, or a different analysis version.
/// Callers treat this as "rebuild the stage", never as a hard failure.
#[derive(Debug, Clone)]
pub struct StaleArtifact {
    pub stage: String,
    pub reason: String,
}

impl std::fmt::Display for StaleArtifact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "stale artifact for stage '{}': {}",
            self.stage, self.reason
        )
    }
}

impl std::error::Error for StaleArtifact {}

pub fn is_stale(err: &anyhow::Error) -> bool {
    err.downcast_ref::<StaleArtifact>().is_some()
}

pub struct ArtifactManager {
    pub artifact_root: PathBuf,
}

fn stale(stage: &str, reason: impl Into<String>) -> anyhow::Error {
    anyhow::Error::from(StaleArtifact {
        stage: stage.to_string(),
        reason: reason.into(),
    })
}

/// Process-unique temp name so concurrent writers of the same artifact never
/// rename away each other's staging file.
fn unique_tmp(path: &Path) -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    path.with_file_name(format!(
        ".{}.{}.{}.tmp",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("artifact"),
        std::process::id(),
        id
    ))
}

/// Write bytes to `path` atomically: write a unique temp file next to the
/// target, then rename over the target so a crash mid-write leaves the
/// previous artifact intact.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let staged_temp_path = unique_tmp(path);
    fs::write(&staged_temp_path, bytes)
        .with_context(|| format!("Failed to write temp file at {:?}", staged_temp_path))?;
    fs::rename(&staged_temp_path, path)
        .with_context(|| format!("Failed to rename {:?} to {:?}", staged_temp_path, path))?;
    Ok(())
}

fn write_json_atomic<T: serde::Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)
        .with_context(|| format!("Failed to serialize {:?}", path))?;
    write_atomic(path, json.as_bytes())
}

/// Compare paths ignoring Windows verbatim (`\\?\`) prefixes and casing.
#[cfg(windows)]
fn same_path(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| -> String {
        p.to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .replace('\\', "/")
            .to_lowercase()
            .trim_end_matches('/')
            .to_string()
    };
    norm(a) == norm(b)
}

#[cfg(not(windows))]
fn same_path(a: &Path, b: &Path) -> bool {
    a == b
}

impl ArtifactManager {
    pub fn init<P: AsRef<Path>>(root: P) -> Result<Self> {
        let artifact_root = root.as_ref().join("artifacts");
        fs::create_dir_all(&artifact_root).with_context(|| {
            format!("Failed to create artifact directory at {:?}", artifact_root)
        })?;
        Ok(Self { artifact_root })
    }

    pub fn prepare_for_repo(repo: &Repository) -> Result<Self> {
        Self::init(&repo.root)
    }

    pub fn repo_root(&self) -> PathBuf {
        self.artifact_root
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.artifact_root.clone())
    }

    /// Verify a stage manifest declares the expected input fingerprint,
    /// repository root, and analysis version. Any unreadable/corrupt manifest
    /// counts as stale so the stage is rebuilt rather than trusted.
    pub fn stage_is_fresh(&self, stage: &str, expected_fingerprint: &str) -> bool {
        self.check_manifest(stage, expected_fingerprint).is_ok()
    }

    /// Verify a stage manifest declares the expected input fingerprint,
    fn check_manifest(&self, stage: &str, expected_fingerprint: &str) -> Result<()> {
        let manifest_path = self.artifact_root.join(stage).join("manifest.json");
        let content = fs::read_to_string(&manifest_path)
            .map_err(|_| stale(stage, format!("no manifest.json at {:?}", manifest_path)))?;
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| stale(stage, format!("unreadable manifest.json: {}", e)))?;

        let stored_fp = value
            .get("input_fingerprint")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if stored_fp.is_empty() {
            return Err(stale(stage, "artifact has no input fingerprint"));
        }
        if stored_fp != expected_fingerprint {
            return Err(stale(stage, "input fingerprint mismatch"));
        }

        let stored_version = value
            .get("analysis_version")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if stored_version != CURRENT_ANALYSIS_VERSION {
            return Err(stale(
                stage,
                format!(
                    "analysis version mismatch: {} != {}",
                    stored_version, CURRENT_ANALYSIS_VERSION
                ),
            ));
        }

        if let Some(root) = value.get("repository_root").and_then(|v| v.as_str()) {
            if !same_path(Path::new(root), &self.repo_root()) {
                return Err(stale(
                    stage,
                    format!(
                        "repository root mismatch: {} != {}",
                        root,
                        self.repo_root().display()
                    ),
                ));
            }
        }

        Ok(())
    }
}
