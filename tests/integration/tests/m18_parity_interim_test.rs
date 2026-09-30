//! M18.3 interim byte-identical proof (plan §6.1 + M18 table): the
//! registry-driven index (`handle_index_with_registry`, the engine behind
//! `build_source_index`) must reproduce the pre-M18.3 registry-less path
//! byte-for-byte on BOTH fixtures — serialized `IndexResult` JSON, shard
//! cache envelopes, and `index` stage artifacts (identical
//! `TuCacheFingerprint` values; the fingerprint content change with
//! plugin_fingerprint/language fields lands M18.10).
//!
//! Kept until the M18.11 final parity suite replaces it (together with the
//! `#[deprecated]` legacy shim it drives). Runs on throwaway copies of the
//! fixtures, like `m00_golden_baseline_test`, so both sides are cold runs.

mod common;

use anyhow::{Context, Result};
use ariadne_cli::{build_source_index_filtered, handle_index_with_registry};
use ariadne_lang::LanguageRegistry;
use ariadne_repo::SourceLanguage;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct IsolatedRepo {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

#[test]
#[allow(deprecated)]
fn registry_index_matches_legacy_on_basic() -> Result<()> {
    index_parity("basic")
}

#[test]
#[allow(deprecated)]
fn registry_index_matches_legacy_on_expensive() -> Result<()> {
    index_parity("expensive")
}

#[test]
fn unregistered_language_filter_is_a_clear_error() -> Result<()> {
    // `--lang scala` today: ScalaPlugin lands M19, so the CLI must reject the
    // selection up front (plan §3.3 fact 3), never half-index as C++.
    let repo = isolated_repo("basic", "scala")?;
    let err = build_source_index_filtered(&repo.root, Some(SourceLanguage::Scala)).unwrap_err();
    assert!(
        err.to_string().contains("language plugin not registered"),
        "expected a registration error, got: {err}"
    );
    assert!(
        !repo.root.join("artifacts").join("index").join("index.json").exists(),
        "rejected --lang run must not produce index artifacts"
    );
    Ok(())
}

#[test]
fn registry_without_plugins_indexes_nothing_not_cpp() -> Result<()> {
    // §3.3 fact 3's silent-C++ guard from the other side: with no plugins,
    // discovery has no shaper and the walk has no plugin — an empty honest
    // result, never silently C++-parsed analyses.
    let repo = isolated_repo("basic", "empty")?;
    let result = handle_index_with_registry(&repo.root, Arc::new(LanguageRegistry::new()))?;
    assert!(
        result.tu_analyses.is_empty(),
        "unregistered languages must not be parsed as C++: {:?}",
        result.tu_analyses.iter().map(|a| a.tu_file_id).collect::<Vec<_>>()
    );
    Ok(())
}

#[allow(deprecated)]
fn index_parity(fixture: &str) -> Result<()> {
    let legacy_repo = isolated_repo(fixture, "legacy")?;
    let registry_repo = isolated_repo(fixture, "registry")?;
    let lang_repo = isolated_repo(fixture, "langcpp")?;

    let legacy = ariadne_cli::build_source_index_legacy_for_parity(&legacy_repo.root)?;
    let registry = Arc::new(ariadne_cli::registry::default_registry());
    let modern = handle_index_with_registry(&registry_repo.root, Arc::clone(&registry))?;
    let filtered = build_source_index_filtered(&lang_repo.root, Some(SourceLanguage::Cpp))?;

    let roots = vec![
        legacy_repo.root.clone(),
        registry_repo.root.clone(),
        lang_repo.root.clone(),
    ];
    let legacy_json = normalized_json(&serde_json::to_value(&legacy)?, &roots);
    let modern_json = normalized_json(&serde_json::to_value(&modern)?, &roots);
    let filtered_json = normalized_json(&serde_json::to_value(&filtered)?, &roots);

    assert_eq!(
        legacy_json.to_string(),
        modern_json.to_string(),
        "IndexResult JSON diverged between legacy and registry index for {fixture}"
    );
    assert_eq!(
        modern_json.to_string(),
        filtered_json.to_string(),
        "`--lang cpp` (matching all units) diverged from the unfiltered registry index for {fixture}"
    );

    // Cache manifest: shard envelopes (fingerprint + merged analysis) equal.
    let legacy_shards = shard_store(&legacy_repo.root)?;
    let modern_shards = shard_store(&registry_repo.root)?;
    let filtered_shards = shard_store(&lang_repo.root)?;
    assert_eq!(
        legacy_shards, modern_shards,
        "shard cache diverged for {fixture}"
    );
    assert_eq!(
        modern_shards, filtered_shards,
        "shard cache diverged under `--lang cpp` for {fixture}"
    );

    // Persisted index-stage artifacts are byte-identical (normalized only for
    // the per-run temp root paths).
    for (stage_dir, file) in [("index", "manifest.json"), ("index", "index.json")] {
        let legacy_artifact =
            normalized_json(&read_json(&legacy_repo.root.join("artifacts").join(stage_dir).join(file))?, &roots);
        let modern_artifact = normalized_json(
            &read_json(&registry_repo.root.join("artifacts").join(stage_dir).join(file))?,
            &roots,
        );
        assert_eq!(
            legacy_artifact.to_string(),
            modern_artifact.to_string(),
            "artifacts/{stage_dir}/{file} diverged for {fixture}"
        );
    }
    Ok(())
}

fn normalized_json(value: &Value, roots: &[PathBuf]) -> Value {
    common::normalize_json_with_roots(value.clone(), roots)
}

fn read_json(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read {:?}", path))?;
    serde_json::from_str(&content).with_context(|| format!("failed to parse JSON in {:?}", path))
}

/// Every shard envelope keyed by file name, contents normalized for the run
/// roots (paths never appear in `ShardCheckpoint`, normalize anyway).
fn shard_store(repo_root: &Path) -> Result<BTreeMap<String, String>> {
    let dir = repo_root.join("artifacts").join("shards");
    let mut shards = BTreeMap::new();
    for entry in fs::read_dir(&dir)
        .with_context(|| format!("shard store missing: {:?}", dir))?
    {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let value: Value = read_json(&entry.path())?;
        shards.insert(name, common::normalize_json_with_roots(value, &[]).to_string());
    }
    Ok(shards)
}

/// Copies `tests/fixtures/<name>` into a fresh temp dir (skipping regenerable
/// caches), so every run is cold and isolated — same contract as the m00
/// golden baseline.
fn isolated_repo(fixture: &str, label: &str) -> Result<IsolatedRepo> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join(format!("{fixture}-{label}"));
    copy_dir_filtered(&common::fixture_path(fixture), &root)
        .with_context(|| format!("failed to copy fixture {fixture} for {label}"))?;
    Ok(IsolatedRepo { _temp: temp, root })
}

fn copy_dir_filtered(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_owned();
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
