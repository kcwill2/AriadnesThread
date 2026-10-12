//! AR-03 (S5 MED) regression: an `artifacts/*.bin` blob must never be able
//! to drive the process into a declared-length allocation. A corrupt or
//! attacker-authored blob has to surface as a normal load `Err` (the same
//! cache-miss path any unreadable artifact takes), not as a capacity-overflow
//! panic or allocation abort. Honest reads must stay byte-identical.

use ariadne_report::ArtifactManager;
use ariadne_repo::types::{
    FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage,
};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

/// bincode 1.3 legacy encoding (the one `bincode::serialize` writes): fixed
/// 8-byte little-endian lengths.
fn fixint_length(value: u64) -> [u8; 8] {
    value.to_le_bytes()
}

fn write_stage_blob(temp_root: &Path, stage: &str, file_name: &str, bytes: &[u8]) -> PathBuf {
    let stage_dir = temp_root.join("artifacts").join(stage);
    std::fs::create_dir_all(&stage_dir).expect("create stage dir");
    let blob_path = stage_dir.join(file_name);
    std::fs::write(&blob_path, bytes).expect("write blob");
    blob_path
}

fn manager_at(temp_root: &Path) -> ArtifactManager {
    ArtifactManager {
        artifact_root: temp_root.join("artifacts"),
    }
}

/// The streaming read (`load/report.bin`) is the site where a declared
/// length reached the allocator directly: bincode's `IoReader` resized its
/// temp buffer to the blob-declared string length BEFORE reading, so an
/// 8-byte blob declaring `u64::MAX` aborted the process. This is the S5
/// regression proper: expect a clean `Err`, never a panic/abort.
#[test]
fn load_report_blob_declaring_huge_string_length_errors_instead_of_aborting() {
    let dir = tempdir().unwrap();
    // PersistedLoadReport field 1 is `repository_root: PathBuf`, i.e. a
    // length-prefixed string read first, before any honest payload bytes.
    write_stage_blob(
        dir.path(),
        "load",
        "report.bin",
        &fixint_length(u64::MAX),
    );

    let outcome = manager_at(dir.path()).load_load_artifacts();

    assert!(
        outcome.is_err(),
        "declared-length blob must error as a normal load failure, got Ok"
    );
}

/// Slice-read site (`repository/files.bin`): a blob declaring a collection
/// length far beyond its own bytes must fail, not reserve it.
#[test]
fn files_blob_declaring_huge_seq_length_errors_instead_of_aborting() {
    let dir = tempdir().unwrap();
    let mut blob = fixint_length(u64::MAX).to_vec();
    blob.extend_from_slice(b"junk");
    write_stage_blob(dir.path(), "repository", "files.bin", &blob);

    let outcome = manager_at(dir.path()).load_repository_artifacts();

    assert!(
        outcome.is_err(),
        "huge seq-length blob must error, got Ok"
    );
}

/// Slice-read site with the huge length inside the first element's string
/// field: the declared string length (not the seq count) is oversized.
#[test]
fn files_blob_declaring_huge_string_length_errors_instead_of_aborting() {
    let dir = tempdir().unwrap();
    let mut blob = fixint_length(1).to_vec();
    blob.extend_from_slice(&7u64.to_le_bytes()); // FileId(7)
    blob.extend_from_slice(&fixint_length(u64::MAX)); // path string length
    write_stage_blob(dir.path(), "repository", "files.bin", &blob);

    let outcome = manager_at(dir.path()).load_repository_artifacts();

    assert!(
        outcome.is_err(),
        "huge string-length blob must error, got Ok"
    );
}

fn honest_file_records() -> Vec<FileRecord> {
    let first = FileRecord {
        id: FileId(1),
        path: PathBuf::from("include/worker.h"),
        canonical_path: PathBuf::from("/tmp/repo/include/worker.h"),
        kind: FileKind::Header,
        size: 2048,
        content_hash: "aaaa".to_string(),
        ownership: FileOwnership::Repository,
        optimization_policy: OptimizationPolicy::Optimizable,
        language: SourceLanguage::Cpp,
    };
    let second = FileRecord {
        id: FileId(2),
        path: PathBuf::from("src/worker.cpp"),
        canonical_path: PathBuf::from("/tmp/repo/src/worker.cpp"),
        kind: FileKind::Source,
        size: 4096,
        content_hash: "bbbb".to_string(),
        ownership: FileOwnership::Repository,
        optimization_policy: OptimizationPolicy::Contract,
        language: SourceLanguage::Cpp,
    };
    vec![first, second]
}

/// Honest reads unchanged: a blob written by the real writer still loads
/// with identical values through the bounded decode path.
#[test]
fn honest_files_blob_roundtrips_through_bounded_decode() {
    let dir = tempdir().unwrap();
    let records = honest_file_records();
    let bytes = bincode::serialize(&records).expect("honest serialize");
    write_stage_blob(dir.path(), "repository", "files.bin", &bytes);

    let loaded = manager_at(dir.path())
        .load_repository_artifacts()
        .expect("honest files.bin must decode unchanged");

    assert_eq!(loaded, records);
}

/// Honest streaming read unchanged: `load/report.bin` with the real layout
/// (root string, scalar rows, file_metrics) decodes through the bounded
/// path with identical values. bincode encodes a struct as its fields in
/// derive order, so the hand-built bytes below match `PersistedLoadReport`.
#[test]
fn honest_load_report_blob_roundtrips_through_bounded_decode() {
    let dir = tempdir().unwrap();
    let root = "fixture/repo";
    let mut blob = fixint_length(root.len() as u64).to_vec();
    blob.extend_from_slice(root.as_bytes());
    blob.extend_from_slice(&fixint_length(0)); // symbol_summary_rows: empty
    blob.extend_from_slice(&fixint_length(0)); // file_metrics: empty
    write_stage_blob(dir.path(), "load", "report.bin", &blob);

    let loaded = manager_at(dir.path())
        .load_load_artifacts()
        .expect("honest report.bin must decode unchanged");

    assert_eq!(loaded.repository_root, PathBuf::from(root));
    assert!(loaded.symbol_metrics.is_empty());
    assert!(loaded.file_metrics.is_empty());
}

/// Size bookkeeping for the decode limit (recorded in
/// `tasks/ar-03-handoff-journal.md`): fixture-scale honest blobs are tiny;
/// the worst blob scale EVER measured on this project is the pre-DP-2
/// stage-5 save-span transient at ~4096 MiB for both report forms combined
/// (tasks/stage5-collateral-memory-ceiling-t0a-evidence.md sec. 4), and the
/// warm `report.bin` measured ~0.9 MiB at N=1280 files (sec. 6). The decode
/// limit (16 GiB) keeps an order of magnitude above those. This assertion
/// only pins that honest fixture blobs stay small enough to be trivially
/// under any sane limit.
#[test]
fn honest_fixture_blobs_are_far_below_the_decode_limit() {
    let honest_files_bytes =
        bincode::serialize(&honest_file_records()).expect("honest serialize");
    assert!(honest_files_bytes.len() < 4096);

    let report_blob = fixint_length(0).to_vec();
    assert!(report_blob.len() == 8);
}
