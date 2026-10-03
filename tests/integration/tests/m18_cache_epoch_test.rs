//! M18.10 cache-epoch gate (plan §3.5 cache epoch row + §7 `symbols.bin`
//! risk row): stage manifests carrying a stale `analysis_version` must
//! trigger a fresh rebuild — never a serde/bincode error on the old blobs.
//!
//! Scope honesty: a byte-authentic v1 `symbols.bin` cannot be constructed
//! from this milestone's structs (the v1→v2 record layout is the parallel
//! M18.9 change), so the pipeline-level tests here rewrite a CURRENT-epoch
//! manifest to `"1.0.0"` and — in one case — replace the stage's bincode
//! blob with undecodable bytes, locking the ORDER: the manifest gate fires
//! first and the stage regenerates cleanly.

use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn isolated_repo() -> Result<(TempDir, PathBuf)> {
    let guard = tempfile::tempdir()?;
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/basic");
    let repo_root = guard.path().join("basic");
    copy_dir_all(&fixture_root, &repo_root)?;
    Ok((guard, repo_root))
}

fn index_manifest(root: &Path) -> Value {
    serde_json::from_str(
        &fs::read_to_string(root.join("artifacts").join("index").join("manifest.json")).unwrap(),
    )
    .unwrap()
}

fn force_stale_epoch(root: &Path) {
    let manifest_path = root.join("artifacts").join("index").join("manifest.json");
    let mut manifest = index_manifest(root);
    assert_eq!(
        manifest["analysis_version"].as_str().unwrap(),
        ariadne_index::CURRENT_ANALYSIS_VERSION,
        "precondition: the fresh run records the current epoch"
    );
    manifest["analysis_version"] = Value::String("1.0.0".to_string());
    fs::write(&manifest_path, serde_json::to_string(&manifest).unwrap()).unwrap();
}

#[test]
fn stale_index_epoch_triggers_reindex_not_deserialize_error() -> Result<()> {
    let (_guard, root) = isolated_repo()?;
    ariadne_cli::build_source_index(&root)?;
    force_stale_epoch(&root);

    let rebuilt = ariadne_cli::build_source_index(&root);
    assert!(
        rebuilt.is_ok(),
        "a stale-epoch cache must re-index cleanly, got: {:?}",
        rebuilt.err()
    );
    assert_eq!(
        index_manifest(&root)["analysis_version"].as_str(),
        Some(ariadne_index::CURRENT_ANALYSIS_VERSION),
        "the rebuild must re-stamp the current epoch"
    );
    Ok(())
}

#[test]
fn stale_epoch_with_undecodable_blob_still_reindexes() -> Result<()> {
    // The §7 direction: even when the stage blob is no longer readable by the
    // current structs, the manifest gate short-circuits the load before any
    // bincode read, so the pipeline regenerates instead of erroring.
    let (_guard, root) = isolated_repo()?;
    ariadne_cli::build_source_index(&root)?;
    fs::write(
        root.join("artifacts").join("index").join("tu_analyses.bin"),
        b"v1-shaped-or-corrupt-blob-bytes",
    )?;
    force_stale_epoch(&root);

    let rebuilt = ariadne_cli::build_source_index(&root);
    assert!(
        rebuilt.is_ok(),
        "stale epoch must outrank blob readability: {:?}",
        rebuilt.err()
    );
    assert!(index_manifest(&root)["analysis_version"]
        .as_str()
        .is_some_and(|v| v == ariadne_index::CURRENT_ANALYSIS_VERSION));
    Ok(())
}
