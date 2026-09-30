//! Shared cold-run harness for the M18.11 final parity suite.
//!
//! Every section runs on a throwaway copy of `tests/fixtures/<name>` (like
//! the golden baseline and the M18 interim suites), so shard stores and
//! artifacts never bleed between runs.

pub mod isolation_output_parity;
pub mod provenance_lock;
pub mod resolver_parity;
pub mod worker_backend_parity;

use anyhow::{Context, Result};
use ariadne_build::{BuildDiscoverer, BuildDiscoveryResult};
use ariadne_repo::{FileRecord, Repository, RepositoryDiscoveryResult};
use std::fs;
use std::path::{Path, PathBuf};

pub struct IsolatedRepo {
    _temp: tempfile::TempDir,
    pub root: PathBuf,
}

/// Copies `tests/fixtures/<name>` into a fresh temp dir (skipping regenerable
/// caches), so every run is cold and isolated — same contract as the m00
/// golden baseline and the interim parity suites.
pub fn isolated_repo(fixture: &str, label: &str) -> Result<IsolatedRepo> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join(format!("{fixture}-{label}"));
    copy_dir_filtered(&crate::common::fixture_path(fixture), &root)
        .with_context(|| format!("failed to copy fixture {fixture} for {label}"))?;
    Ok(IsolatedRepo { _temp: temp, root })
}

fn copy_dir_filtered(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "artifacts" || name_str == ".ariadne" || name_str.starts_with('.') {
            continue;
        }
        let target = dst.join(name);
        if entry.file_type()?.is_dir() {
            copy_dir_filtered(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Repository discovery + build discovery with the registry-selected unit
/// shaper (plan M18.3/M18.4: the single registered plugin with a shaper owns
/// unit shaping) — the exact inputs the permanent registry pipeline feeds its
/// closure walks and workers.
pub fn repo_inputs(root: &Path) -> Result<(RepositoryDiscoveryResult, BuildDiscoveryResult)> {
    let repository = Repository::locate(root)?;
    let discovery = repository.discover()?;
    let registry = ariadne_cli::registry::default_registry();
    let build = BuildDiscoverer::new(&repository.root)
        .with_repo_discovery(&discovery)
        .with_unit_shaper(registry.unit_shaper_for())
        .discover()?;
    Ok((discovery, build))
}

/// Analyzable discovery records (sources and headers), sorted by
/// repository-relative path with forward slashes, so every sweep in this
/// suite has one deterministic order.
pub fn analyzable_records(discovery: &RepositoryDiscoveryResult) -> Vec<&FileRecord> {
    let mut records: Vec<&FileRecord> = discovery
        .files
        .iter()
        .filter(|record| record.kind.is_analyzable())
        .collect();
    records.sort_by(|a, b| normalized_rel(&a.path).cmp(&normalized_rel(&b.path)));
    records
}

/// Repository-relative path in the fixture-neutral forward-slash form used
/// for comparisons and embedded captures on every platform.
pub fn normalized_rel(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// Normalizes an embedded capture string the same way the golden convention
/// does (every `\` becomes `/`), so captures taken on Windows stay valid on
/// other platforms. Production bytes on Windows legitimately carry `\` inside
/// rendered dependency paths; the capture locks the bytes modulo that single
/// platform separator.
pub fn lockable(text: &str) -> String {
    text.replace('\\', "/")
}
