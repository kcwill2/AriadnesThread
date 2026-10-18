//! UTA-T6 item 8 (pin-only) locks for the build crate.
//!
//! - `BuildDiscoveryResult::normalize` dedups contexts BY NAME ONLY: a
//!   second same-named context is dropped WHOLE (its compiler, defines and
//!   dirs vanish) - the adjudicated smell at build/src/types.rs:97.
//! - `relative_path_for_source` branches (plain relative entry returned
//!   verbatim; verbatim-canonical root stripped through the string fallback)
//!   pinned through the public `discover()` path they actually serve.
//! - compile_commands.json discovery precedence root > build > out.
//! - the number formatter inside `MeasurementComparison::format_report` at
//!   its boundaries 0 / 1_000 / 100_000 / 1_000_000.

use crate::ClangCompileCommandEntry;
use crate::{
    BuildDiscoveryResult, BuildDiscoverer, BuildMeasurement, CompilationContext,
    MeasurementComparison,
};
use ariadne_repo::FileId;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::tempdir;

// ---------------------------------------------------------------------------
// normalize: whole-context drop on duplicate names
// ---------------------------------------------------------------------------

#[test]
fn pin_normalize_drops_duplicate_named_context() {
    // ADJUDICATED SMELL (types.rs:97 `dedup_by(|a, b| a.name == b.name)`):
    // contexts are sorted by name only, then deduped by name only. Two
    // DIFFERENT contexts sharing a name are collapsed to the first one in
    // input order - the loser is dropped whole, including its distinct
    // compiler, defines, and include dirs. Pinned, not endorsed: an owner
    // decision on merge-vs-reject semantics is pending.
    let mut kept = CompilationContext::new("Debug|x64", "cl.exe");
    kept.defines = vec!["FIRST_WINS".to_string()];
    kept.include_dirs = vec![PathBuf::from("only/in/first")];
    let mut dropped = CompilationContext::new("Debug|x64", "clang.exe");
    dropped.defines = vec!["SECOND_LOSES".to_string()];
    dropped.include_dirs = vec![PathBuf::from("only/in/second")];

    let mut result = BuildDiscoveryResult {
        repo_root: PathBuf::from("/repo"),
        contexts: vec![kept, dropped],
        translation_units: Vec::new(),
    };
    result.normalize();

    assert_eq!(
        result.contexts.len(),
        1,
        "pin: same-name contexts collapse to one regardless of content"
    );
    let surviving = &result.contexts[0];
    assert_eq!(surviving.compiler, "cl.exe");
    assert_eq!(surviving.defines, vec!["FIRST_WINS"]);
    assert_eq!(surviving.include_dirs, vec![PathBuf::from("only/in/first")]);
    assert!(
        !surviving.defines.iter().any(|d| d == "SECOND_LOSES"),
        "the loser's data must be gone entirely - no field merge happens"
    );
}

// ---------------------------------------------------------------------------
// relative_path_for_source branches + discovery precedence
// ---------------------------------------------------------------------------

fn write_compile_commands(dir: &Path, marker: &str) {
    let entries = vec![ClangCompileCommandEntry {
        directory: dir.to_path_buf(),
        file: PathBuf::from(format!("src/{marker}")),
        command: None,
        arguments: vec!["cl.exe".to_string(), "/c".to_string()],
        output: None,
    }];
    fs::write(
        dir.join("compile_commands.json"),
        serde_json::to_vec(&entries).unwrap(),
    )
    .unwrap();
}

#[test]
fn discover_prefers_root_then_build_then_out_database() {
    // Candidate order (discovery.rs:72-76): root, build, out. With all three
    // present the ROOT database wins; each deletion exposes the next tier.
    let dir = tempdir().unwrap();
    let root = dir.path();
    write_compile_commands(root, "from-root.cpp");
    fs::create_dir_all(root.join("build")).unwrap();
    write_compile_commands(&root.join("build"), "from-build.cpp");
    fs::create_dir_all(root.join("out")).unwrap();
    write_compile_commands(&root.join("out"), "from-out.cpp");

    let first = BuildDiscoverer::new(root).discover().unwrap();
    assert_eq!(first.translation_units.len(), 1);
    assert_eq!(
        first.translation_units[0].file_id,
        FileId::from_relative_path("src/from-root.cpp"),
        "pin: the root database takes precedence over build/ and out/"
    );

    fs::remove_file(root.join("compile_commands.json")).unwrap();
    let second = BuildDiscoverer::new(root).discover().unwrap();
    assert_eq!(
        second.translation_units[0].file_id,
        FileId::from_relative_path("src/from-build.cpp"),
        "pin: without the root file, build/ wins"
    );

    fs::remove_file(root.join("build").join("compile_commands.json")).unwrap();
    let third = BuildDiscoverer::new(root).discover().unwrap();
    assert_eq!(
        third.translation_units[0].file_id,
        FileId::from_relative_path("src/from-out.cpp"),
        "pin: without root and build, out/ wins"
    );
}

#[test]
fn pin_relative_entry_id_ignores_the_directory_field() {
    // Plain-file branch (discovery.rs:284-286): a relative `file` is handed
    // back VERBATIM as the repo-relative path - the entry's `directory`
    // never enters the id, even when it points at a different root.
    let dir = tempdir().unwrap();
    let entries = vec![ClangCompileCommandEntry {
        directory: PathBuf::from("Z:\\unrelated\\build\\dir"),
        file: PathBuf::from("src/keepme.cpp"),
        command: None,
        arguments: vec!["cl.exe".to_string()],
        output: None,
    }];
    fs::write(
        dir.path().join("compile_commands.json"),
        serde_json::to_vec(&entries).unwrap(),
    )
    .unwrap();

    let result = BuildDiscoverer::new(dir.path()).discover().unwrap();
    assert_eq!(result.translation_units.len(), 1);
    assert_eq!(
        result.translation_units[0].file_id,
        FileId::from_relative_path("src/keepme.cpp"),
        "pin: the relative entry path (slashes normalized) is the whole id - \
         the unrelated directory does not relocate it"
    );
}

#[cfg(windows)]
#[test]
fn pin_verbatim_canonical_root_strips_through_the_string_fallback() {
    // Verbatim-root branch: `discover` canonicalizes, so on Windows the
    // comparison root IS `\\?\...`. An entry naming the same location in
    // PLAIN form cannot be `strip_prefix`-ed structurally; the string
    // fallback (norm slashes + `//?/` trim) must still produce the
    // repo-relative path. Pinned as the FileId the fallback yields - the
    // counterfactual (fallback failure) would derive the id from the whole
    // plain source path instead.
    let dir = tempdir().unwrap();
    let canonical = dir.path().canonicalize().unwrap();
    assert!(
        canonical.to_string_lossy().starts_with("\\\\?\\"),
        "test premise: canonicalize yields a verbatim root, got {canonical:?}"
    );
    let plain_root = PathBuf::from(
        canonical
            .to_string_lossy()
            .trim_start_matches("\\\\?\\")
            .to_string(),
    );

    let entries = vec![ClangCompileCommandEntry {
        directory: plain_root.clone(),
        file: plain_root.join("src").join("verbatim.cpp"),
        command: None,
        arguments: vec!["cl.exe".to_string()],
        output: None,
    }];
    fs::write(
        dir.path().join("compile_commands.json"),
        serde_json::to_vec(&entries).unwrap(),
    )
    .unwrap();

    let result = BuildDiscoverer::new(dir.path()).discover().unwrap();
    assert_eq!(result.translation_units.len(), 1);
    let unit = &result.translation_units[0];
    assert_eq!(
        unit.file_id,
        FileId::from_relative_path("src/verbatim.cpp"),
        "pin: the string fallback strips the verbatim canonical root and the \
         id is repo-relative"
    );
    let full_plain = unit.source_path.to_string_lossy().replace('\\', "/");
    assert_ne!(
        unit.file_id,
        FileId::from_relative_path(&full_plain),
        "guard: the id must not be the full-path fallback answer"
    );
}

// ---------------------------------------------------------------------------
// format_report number formatting boundaries
// ---------------------------------------------------------------------------

fn measurement_with_collateral(load: usize) -> BuildMeasurement {
    BuildMeasurement {
        repo_root: PathBuf::from("/repo"),
        timestamp: 0,
        full_build_wall_time: Duration::from_millis(10),
        total_frontend_cpu_time: Duration::from_millis(10),
        total_preprocessing_time: Duration::from_millis(1),
        peak_memory_bytes: 0,
        tu_measurements: Vec::new(),
        collateral_load: Some(load),
        affected_tus_count: None,
    }
}

/// The rendered collateral pair as (before, after). The separator in the
/// report is U+2192 RIGHTWARDS ARROW; referenced by code point so this file
/// stays pure ASCII (repo UTF-8 rule).
fn collateral_pair(rendered: &str) -> (String, String) {
    let arrow = char::from_u32(0x2192).expect("U+2192 is an assigned code point");
    let line = rendered
        .lines()
        .find(|line| line.contains(arrow))
        .expect("format_report must render the collateral pair line");
    let mut sides = line.split(arrow);
    let before = sides.next().unwrap_or("").trim().to_string();
    let after = sides
        .next()
        .unwrap_or_else(|| panic!("the collateral line must carry both sides: {line}"))
        .trim()
        .to_string();
    (before, after)
}

#[test]
fn format_report_comma_boundaries_zero_one_k_one_hundred_k_one_million() {
    // The nested `format_num` (measure.rs:78-94) at its claimed boundaries.
    // NOTE (journal): `crates/cluster/src/plan.rs` carries its OWN digit
    // grouping implementation - divergence recorded, deliberately NOT
    // unified (out of this hand's scope; owner decision).
    let cases: Vec<(usize, usize, &str, &str)> = vec![
        (0, 0, "0", "0"),
        (0, 1_000, "0", "1,000"),
        (100_000, 1_000, "100,000", "1,000"),
        (100_000, 1_000_000, "100,000", "1,000,000"),
    ];
    for (before, after, expected_before, expected_after) in cases {
        let comparison = MeasurementComparison::compare(
            measurement_with_collateral(before),
            measurement_with_collateral(after),
        );
        let (rendered_before, rendered_after) = collateral_pair(&comparison.format_report());
        assert_eq!(
            rendered_before, expected_before,
            "before-side of ({before}, {after})"
        );
        assert_eq!(
            rendered_after, expected_after,
            "after-side of ({before}, {after})"
        );
    }
}
