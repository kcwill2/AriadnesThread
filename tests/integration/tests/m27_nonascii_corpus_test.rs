//! M27 T0b approved non-ASCII specimen corpus (frozen-set sibling dir
//! `tests/fixtures/m27-nonascii-underflow/`, E3 shape per the M27 shape plan).
//!
//! Two locks land with the fixture:
//!
//! 1. `test_specimen_bytes_are_single_encoded_utf8` - locks the committed
//!    byte shape (word run carrying U+00E9, punct run carrying U+00AB, no
//!    double-encoding signature, pure-ASCII authored content). GREEN at
//!    landing and after the T2 fix.
//! 2. `test_m27_nonascii_corpus_cold_index_completes` - the carried RED
//!    completion lock for T2 (invariant I12: the corpus that panicked now
//!    finishes). Lands `#[ignore]`d; the T2 fix commit removes the ignore
//!    line as its single-line rider.
//!
//! This file is pure ASCII by policy: every multi-byte fixture character is
//! named here by its stored UTF-8 BYTE VALUES (or a `\u{...}` escape), never
//! by a literal, per the repo UTF-8 rule (AGENTS.md). The fixture data files
//! themselves carry the real single-encoded bytes - that is their point.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

const WORD_RUN: &str = "src/word_run.cpp";
const PUNCT_RUN: &str = "src/punct_run.cpp";

/// Every specimen source the shard-JSON line/column scan may reference.
const SPECIMEN_SOURCE_PATHS: [&str; 3] = [WORD_RUN, PUNCT_RUN, "include/runs.h"];

/// U+00E9 (word-run character) as stored: lead C3, continuation A9.
const E_ACUTE_BYTES: [u8; 2] = [0xC3, 0xA9];
/// U+00AB (punct-run character) as stored: lead C2, continuation AB.
const GUILLEMET_BYTES: [u8; 2] = [0xC2, 0xAB];
/// U+20AC (punct-run PANIC DRIVER, handoff contingency swap) as stored:
/// E2 82 AC at column 1 of its line - the three-byte width is what pushes
/// `flush_punct`'s column+1 subtraction below zero at tokenizer.rs:331.
const EURO_DRIVER_BYTES: [u8; 3] = [0xE2, 0x82, 0xAC];
/// U+00C3 "A with tilde" - lead character of classic double-encoded UTF-8
/// mojibake. Its absence proves the specimen is single-encoded, not re-saved
/// corruption (repo incident E2 class).
const DOUBLE_ENCODING_SIGNATURE: char = '\u{00C3}';

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|tests_dir| tests_dir.parent())
        .expect("integration tests must live under <repo>/tests/integration")
        .join("tests")
        .join("fixtures")
        .join("m27-nonascii-underflow")
}

fn read_fixture_bytes(relative: &str) -> Vec<u8> {
    let path = fixture_root().join(relative);
    fs::read(&path).unwrap_or_else(|err| panic!("fixture file {:?} must exist: {}", path, err))
}

fn is_ascii_alphanumeric_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

/// True when `sequence` occurs in `bytes` flanked on BOTH sides by bytes
/// accepted by the two predicates (requires real neighbors, not file edges).
fn sequence_flanked_by(
    bytes: &[u8],
    sequence: &[u8],
    left_neighbor: fn(u8) -> bool,
    right_neighbor: fn(u8) -> bool,
) -> bool {
    let span = sequence.len();
    if bytes.len() < span + 2 {
        return false;
    }
    bytes.windows(span + 2).any(|window| {
        window[1..=span] == *sequence
            && left_neighbor(window[0])
            && right_neighbor(window[span + 1])
    })
}

fn flanked_by_word_chars(bytes: &[u8], sequence: &[u8]) -> bool {
    sequence_flanked_by(bytes, sequence, is_ascii_alphanumeric_byte, is_ascii_alphanumeric_byte)
}

fn flanked_by_non_word_chars(bytes: &[u8], sequence: &[u8]) -> bool {
    sequence_flanked_by(
        bytes,
        sequence,
        |byte| !is_ascii_alphanumeric_byte(byte),
        |byte| !is_ascii_alphanumeric_byte(byte),
    )
}

#[test]
fn test_specimen_bytes_are_single_encoded_utf8() {
    let word_bytes = read_fixture_bytes(WORD_RUN);
    let punct_bytes = read_fixture_bytes(PUNCT_RUN);

    for (name, bytes) in [(WORD_RUN, &word_bytes), (PUNCT_RUN, &punct_bytes)] {
        let text = std::str::from_utf8(bytes)
            .unwrap_or_else(|err| panic!("{name}: specimen must be valid UTF-8: {err}"));
        assert!(
            !text.contains(DOUBLE_ENCODING_SIGNATURE),
            "{name}: U+00C3 double-encoding signature found - the specimen must stay single-encoded UTF-8"
        );
    }

    assert!(
        flanked_by_word_chars(&word_bytes, &E_ACUTE_BYTES),
        "{WORD_RUN}: a C3 A9 sequence must sit between ASCII-alphanumeric bytes (word-run shape)"
    );
    assert!(
        flanked_by_non_word_chars(&punct_bytes, &GUILLEMET_BYTES),
        "{PUNCT_RUN}: a C2 AB sequence must sit between non-alphanumeric bytes (punct-run shape)"
    );
    assert!(
        punct_bytes.windows(EURO_DRIVER_BYTES.len()).any(|window| window == EURO_DRIVER_BYTES),
        "{PUNCT_RUN}: the E2 82 AC panic-driver sequence must be present (contingency swap)"
    );

    let self_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("m27_nonascii_corpus_test.rs");
    let authored_files = [
        ("README.md", read_fixture_bytes("README.md")),
        (
            "m27_nonascii_corpus_test.rs",
            fs::read(&self_path).expect("this test file must be readable"),
        ),
    ];
    for (name, bytes) in authored_files {
        let non_ascii_count = bytes.iter().filter(|byte| **byte >= 0x80).count();
        assert_eq!(
            non_ascii_count, 0,
            "{name}: authored content must be pure ASCII, found {non_ascii_count} non-ASCII bytes"
        );
    }
}

#[test]
fn test_m27_nonascii_corpus_cold_index_completes() {
    let (_fixture_temp, repo_path) = setup_isolated_repo();
    write_synthetic_compile_db(&repo_path);

    let mut pipeline = ariadne_pipeline::Pipeline::new(&repo_path, None)
        .expect("pipeline must construct on the temp specimen copy");
    pipeline
        .ensure_index()
        .expect("cold index must COMPLETE on the specimen corpus (I12)");
    assert_eq!(
        pipeline.stats.reanalyzed_tus, 2,
        "both specimen TUs must be analyzed - a silently-empty corpus must never pass this lock"
    );

    let source_lines = load_specimen_line_index(&repo_path);
    let shard_dir = repo_path.join("artifacts").join("shards");
    let shard_paths = shard_json_paths(&shard_dir);
    assert!(
        !shard_paths.is_empty(),
        "cold index must write shards under {:?}",
        shard_dir
    );
    for shard_path in shard_paths {
        let shard = read_json(&shard_path);
        assert_no_wrapped_columns(&shard, &source_lines, &shard_path);
    }
}

/// Copies the specimen fixture into a temp dir (m00 throwaway-copy idiom,
/// skipping regenerable caches) so the pipeline always runs cold and
/// isolated from the tracked fixture.
fn setup_isolated_repo() -> (tempfile::TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("temp dir for the isolated specimen copy");
    let repo_path = temp_dir.path().join("m27-nonascii-underflow");
    copy_tree_filtered(&fixture_root(), &repo_path)
        .unwrap_or_else(|err| panic!("failed to copy specimen fixture to {:?}: {}", repo_path, err));
    (temp_dir, repo_path)
}

fn copy_tree_filtered(src: &Path, dst: &Path) -> std::io::Result<()> {
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
            copy_tree_filtered(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Marker-free repositories shape zero C++ units (M22 D22), so the temp copy
/// carries a synthetic compile db (m18 idiom). The committed fixture stays
/// basic-style with no compile db.
fn write_synthetic_compile_db(repo_path: &Path) {
    let root_str = repo_path.to_string_lossy().replace('\\', "/");
    let compile_commands = format!(
        r#"[
    {{ "directory": "{root_str}", "command": "clang++ -Iinclude -c src/word_run.cpp", "file": "src/word_run.cpp" }},
    {{ "directory": "{root_str}", "command": "clang++ -Iinclude -c src/punct_run.cpp", "file": "src/punct_run.cpp" }}
]"#
    );
    fs::write(repo_path.join("compile_commands.json"), compile_commands)
        .expect("synthetic compile db must be written into the temp copy");
}

/// file_id -> source lines, for every specimen source. FileId is the
/// documented deterministic hash of the normalized relative path, so the
/// mapping is reproducible without any discovery artifact.
fn load_specimen_line_index(repo_path: &Path) -> HashMap<u64, Vec<String>> {
    SPECIMEN_SOURCE_PATHS
        .iter()
        .map(|relative_path| {
            let file_id = ariadne_repo::FileId::from_relative_path(relative_path).0;
            let text = fs::read_to_string(repo_path.join(relative_path))
                .unwrap_or_else(|err| panic!("specimen source {relative_path} must exist: {err}"));
            (*relative_path, file_id, text.lines().map(str::to_owned).collect())
        })
        .map(|(_relative_path, file_id, lines)| (file_id, lines))
        .collect()
}

fn shard_json_paths(shard_dir: &Path) -> Vec<PathBuf> {
    if !shard_dir.exists() {
        return Vec::new();
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(shard_dir)
        .expect("shard dir must be readable")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    paths.sort();
    paths
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read shard {:?}: {}", path, err));
    serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("failed to parse shard JSON {:?}: {}", path, err))
}

/// I14-shaped release-wrap detector: the release-profile twin of the E3
/// underflow WRAPS SILENTLY to huge columns, so every stored column must
/// stay within its source line's char count + 1. In DEBUG the corpus never
/// gets this far - it panics in the tokenizer first (the Red evidence).
fn assert_no_wrapped_columns(
    value: &Value,
    line_index: &HashMap<u64, Vec<String>>,
    shard_path: &Path,
) {
    match value {
        Value::Object(entries) => {
            if let Some(column) = entries.get("column").and_then(Value::as_u64) {
                let bound = column_bound(entries, line_index);
                assert!(
                    column <= bound,
                    "{:?}: stored column {} exceeds its bound {} (source line char count + 1) - \
                     silent release wrap of the E3 underflow class",
                    shard_path,
                    column,
                    bound
                );
            }
            for child in entries.values() {
                assert_no_wrapped_columns(child, line_index, shard_path);
            }
        }
        Value::Array(items) => {
            for item in items {
                assert_no_wrapped_columns(item, line_index, shard_path);
            }
        }
        _ => {}
    }
}

fn column_bound(entries: &Map<String, Value>, line_index: &HashMap<u64, Vec<String>>) -> u64 {
    let global_max = line_index
        .values()
        .flat_map(|lines| lines.iter().map(|line| line.chars().count() as u64))
        .max()
        .unwrap_or(0)
        + 1;
    match (
        entries.get("file_id").and_then(Value::as_u64),
        entries.get("line").and_then(Value::as_u64),
    ) {
        (Some(file_id), Some(line)) if line >= 1 => {
            let lines = line_index.get(&file_id).unwrap_or_else(|| {
                panic!("shard JSON references file_id {file_id} outside the specimen corpus")
            });
            let chars = lines
                .get((line - 1) as usize)
                .map(|line| line.chars().count() as u64)
                .unwrap_or_else(|| {
                    panic!("shard JSON references line {line} outside the specimen sources")
                });
            chars + 1
        }
        // Locations with no usable position (e.g. FileLocation::default())
        // still fall to the coarse bound - a wrapped column passes neither.
        _ => global_max,
    }
}
