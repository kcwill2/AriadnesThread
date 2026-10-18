//! UTA-T6 pin-only contract locks for the artifact manager.
//!
//! Every test here pins CURRENT behavior of an adjudicated smell or locks an
//! error branch the audit found unexercised. Nothing in this module asks for
//! a behavior change; the adjudication lives in
//! `docs/specs/unit-test-audit.md` ("Adjudicated-as-pin-only"). Green-by-design
//! pins are the point - they make the smell's exact shape visible and locked.

use super::tests::{manager_in_fresh_tempdir, FP};
use super::{is_stale, ArtifactManager};
use ariadne_index::CURRENT_ANALYSIS_VERSION;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Drop a manifest.json into a synthetic stage dir; `check_manifest` reads
/// only that file, so a stage needs no payload blobs to exercise its gates.
fn plant_manifest(mgr: &ArtifactManager, stage: &str, content: &str) {
    let stage_dir = mgr.artifact_root.join(stage);
    fs::create_dir_all(&stage_dir).unwrap();
    fs::write(stage_dir.join("manifest.json"), content).unwrap();
}

/// A manifest that passes the fingerprint and epoch gates for `mgr`, with a
/// caller-chosen `repository_root` form.
fn manifest_with_root(root: &Value) -> String {
    json!({
        "repository_root": root,
        "input_fingerprint": FP,
        "analysis_version": CURRENT_ANALYSIS_VERSION,
    })
    .to_string()
}

fn manifest_json(dir: &Path) -> Value {
    let content = fs::read_to_string(dir.join("manifest.json")).unwrap();
    serde_json::from_str(&content).unwrap()
}

// ---------------------------------------------------------------------------
// Item 2 - check_manifest branches
// ---------------------------------------------------------------------------

#[test]
fn check_manifest_rejects_repository_root_mismatch() {
    // A manifest that passes fingerprint + epoch but names a different root
    // is stale (rebuild), naming the root gate in the reason.
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    plant_manifest(
        &mgr,
        "roottest",
        &manifest_with_root(&json!("Z:\\definitely-not-the-repo")),
    );

    let err = mgr
        .check_manifest("roottest", FP)
        .expect_err("a foreign repository_root must never pass");
    assert!(is_stale(&err), "root mismatch must be StaleArtifact: {err}");
    assert!(
        err.to_string().contains("repository root mismatch"),
        "error must name the root gate: {err}"
    );
    assert!(!mgr.stage_is_fresh("roottest", FP));
}

#[test]
fn pin_manifest_without_root_key_skips_root_check() {
    // ADJUDICATED SMELL (unit-test-audit.md, "root check skipped when
    // manifest key absent"): a manifest with no `repository_root` key at all
    // simply skips the root gate and passes. Pinned, not endorsed: a
    // relocated artifact tree that also refreshed its fingerprint would be
    // trusted. Owner decision pending on making the key mandatory.
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let without_root = json!({
        "input_fingerprint": FP,
        "analysis_version": CURRENT_ANALYSIS_VERSION,
    })
    .to_string();
    plant_manifest(&mgr, "norootkey", &without_root);

    assert!(
        mgr.check_manifest("norootkey", FP).is_ok(),
        "pin: key-absent manifest skips the root check and stays fresh"
    );
    assert!(mgr.stage_is_fresh("norootkey", FP));
}

#[test]
fn check_manifest_corrupt_json_is_stale() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    plant_manifest(&mgr, "corrupt", "{not json");

    let err = mgr
        .check_manifest("corrupt", FP)
        .expect_err("a corrupt manifest must never pass");
    assert!(is_stale(&err), "corrupt manifest must be StaleArtifact: {err}");
    assert!(
        err.to_string().contains("unreadable manifest.json"),
        "error must name the parse gate: {err}"
    );
}

#[cfg(windows)]
#[test]
fn pin_same_path_table_accepts_case_slash_trailing_and_verbatim_forms() {
    // ADJUDICATED ROW (unit-test-audit: same_path table). The table is
    // `same_path`'s own contract, pinned at the function: case, separator
    // direction, trailing slash, and the `\\?\` verbatim prefix are all
    // cosmetic for a plain root; a distinct sibling is not.
    use super::same_path;
    let plain = Path::new("C:\\repos\\demo");

    let cosmetic: Vec<(&str, &str)> = vec![
        ("uppercase", "C:\\REPOS\\DEMO"),
        ("forward slashes", "C:/repos/demo"),
        ("trailing backslash", "C:\\repos\\demo\\"),
        ("trailing forward slash", "C:/repos/demo/"),
        ("verbatim prefix", "\\\\?\\C:\\repos\\demo"),
    ];
    for (label, variant) in cosmetic {
        assert!(
            same_path(plain, Path::new(variant)),
            "pin: the {label} spelling must read as the same path"
        );
        assert!(
            same_path(Path::new(variant), plain),
            "pin: same_path must stay symmetric for the {label} form"
        );
    }
    assert!(
        !same_path(plain, Path::new("C:\\repos\\demo2")),
        "a distinct sibling path must stay a mismatch"
    );
}

#[cfg(windows)]
#[test]
fn pin_verbatim_manager_root_rejects_the_slash_rewritten_spelling() {
    // Edge discovered while pinning the table, and its real-world
    // consequence. `Repository::locate` canonicalizes, so on Windows the
    // manager-side root IS a `\\?\` verbatim path. same_path's verbatim
    // strip is a string-prefix operation, so a manifest root whose slashes
    // were rewritten (`//?/C:/...`) can no longer be recognized - the
    // separator-cosmetic rule from the table does NOT survive combination
    // with the verbatim prefix. Pinned at the manifest gate: cosmetic case
    // and trailing-slash forms of the verbatim root pass; the slash-rewritten
    // form mismatches. Reported for the owner decision, not fixed.
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let root = mgr.repo_root().to_string_lossy().to_string();
    assert!(
        root.starts_with("\\\\?\\"),
        "test premise: Repository::locate canonicalizes to a verbatim root, got {root}"
    );

    for (label, variant) in [
        ("uppercase", root.to_uppercase()),
        ("trailing backslash", format!("{root}\\")),
        ("trailing forward slash", format!("{root}/")),
    ] {
        plant_manifest(&mgr, "verbatim", &manifest_with_root(&json!(variant)));
        assert!(
            mgr.check_manifest("verbatim", FP).is_ok(),
            "pin: the {label} form of the verbatim manager root stays fresh"
        );
    }

    let rewritten = root.replace('\\', "/");
    plant_manifest(&mgr, "verbatim", &manifest_with_root(&json!(rewritten)));
    let err = mgr
        .check_manifest("verbatim", FP)
        .expect_err("pin: slash-rewriting a verbatim root loses the prefix strip");
    assert!(
        err.to_string().contains("repository root mismatch"),
        "error must name the root gate: {err}"
    );
}

// ---------------------------------------------------------------------------
// Item 3a - apply_io load_measurement_from_path fallback ladder
// ---------------------------------------------------------------------------

fn save_measurement(mgr: &ArtifactManager, repo_root: &Path) -> std::path::PathBuf {
    let measurement = ariadne_build::BuildMeasurement {
        repo_root: repo_root.to_path_buf(),
        timestamp: 4242,
        full_build_wall_time: std::time::Duration::from_millis(500),
        total_frontend_cpu_time: std::time::Duration::from_millis(1500),
        total_preprocessing_time: std::time::Duration::from_millis(200),
        peak_memory_bytes: 1024,
        tu_measurements: vec![],
        collateral_load: Some(10),
        affected_tus_count: Some(5),
    };
    mgr.save_measurement_artifacts(&measurement).unwrap()
}

#[test]
fn pin_corrupt_bin_falls_back_to_json() {
    // ADJUDICATED SMELL (unit-test-audit.md, "apply_io.rs:94-109 silent
    // stale-JSON fallback"): a corrupt measurements.bin is decoded, its
    // failure SILENTLY swallowed, and the json twin answered instead - no
    // warning that the primary binary artifact is damaged. Pinned: honest
    // json content still loads, but the bin corruption is invisible to the
    // caller. Owner decision pending on surfacing a warning.
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let measure_dir = save_measurement(&mgr, dir.path());
    fs::write(measure_dir.join("measurements.bin"), b"corrupted-bin").unwrap();

    let loaded = ArtifactManager::load_measurement_from_path(&measure_dir)
        .expect("pin: corrupt bin + valid json must still load via the json fallback");
    assert_eq!(loaded.timestamp, 4242);
}

#[test]
fn both_corrupt_measurement_forms_error_names_the_path() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let measure_dir = save_measurement(&mgr, dir.path());
    fs::write(measure_dir.join("measurements.bin"), b"corrupted-bin").unwrap();
    fs::write(measure_dir.join("measurements.json"), "{not json").unwrap();

    let err = ArtifactManager::load_measurement_from_path(&measure_dir)
        .expect_err("both forms corrupt must be a hard Err");
    assert!(
        err.to_string()
            .contains("Could not load measurement artifacts from path"),
        "error must name the searched path: {err}"
    );
}

#[test]
fn measurement_path_with_unknown_extension_is_rejected() {
    // Neither branch of the ladder fires for an extension that is neither
    // `.bin` nor `.json` (and the path is a file, not a dir): the file is
    // never even parsed, however honest its content.
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let measure_dir = save_measurement(&mgr, dir.path());
    let honest_json = fs::read_to_string(measure_dir.join("measurements.json")).unwrap();
    let odd_path = dir.path().join("measurements.txt");
    fs::write(&odd_path, honest_json).unwrap();

    let err = ArtifactManager::load_measurement_from_path(&odd_path)
        .expect_err("pin: a .txt measurement file is outside the extension ladder");
    assert!(
        err.to_string().contains("measurements.txt"),
        "error must name the rejected path: {err}"
    );
}

// ---------------------------------------------------------------------------
// Item 4 - index_io round-trip with reuse stats; transform_io fingerprint
// ---------------------------------------------------------------------------

fn fixture_tu_analysis() -> ariadne_index::TuAnalysis {
    use ariadne_index::{
        DeclId, DependencyKind, FileLocation, SemanticEdge, SymbolDecl, SymbolId, SymbolKind,
        SymbolRecord, TuAnalysis,
    };
    let tu_file = ariadne_repo::FileId::from_relative_path("src/alpha.cpp");
    let header_file = ariadne_repo::FileId::from_relative_path("include/beta.h");
    TuAnalysis {
        tu_file_id: tu_file,
        context_name: "Debug|x64".to_string(),
        symbols: vec![SymbolRecord {
            id: SymbolId::new("p.Alpha"),
            name: "Alpha".to_string(),
            kind: SymbolKind::Class,
            owning_file: tu_file,
            declarations: vec![DeclId::new("alpha-decl")],
            definition: Some(DeclId::new("alpha-decl")),
            modifiers: vec![],
        }],
        declarations: vec![SymbolDecl {
            id: DeclId::new("alpha-decl"),
            symbol_id: SymbolId::new("p.Alpha"),
            location: FileLocation {
                file_id: tu_file,
                line: 3,
                column: 1,
            },
            is_definition: true,
        }],
        edges: vec![
            SemanticEdge {
                src_symbol: SymbolId::new("p.Alpha"),
                dst_symbol: SymbolId::new("p.Beta"),
                kind: DependencyKind::Complete,
                src_file: tu_file,
                dst_file: header_file,
            },
            SemanticEdge {
                src_symbol: SymbolId::new("p.Alpha"),
                dst_symbol: SymbolId::new("p.Gamma"),
                kind: DependencyKind::Incomplete,
                src_file: tu_file,
                dst_file: header_file,
            },
        ],
        references: vec![],
        call_sites: vec![],
        language: Default::default(),
        extends_edges: vec![],
    }
}

#[test]
fn index_round_trip_with_reuse_stats_populates_manifest_and_reloads() {
    // The reuse-stats arm of save_index_artifacts (Some(IncrementalStats))
    // had no unit coverage: the manifest reuse fields and a non-empty TU
    // bincode round-trip are locked together here.
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let index_result =
        ariadne_index::IndexResult::new(mgr.repo_root(), vec![fixture_tu_analysis()]);
    let stats = ariadne_index::IncrementalStats {
        total_tus: 4,
        reused_tus: 3,
        reanalyzed_tus: 1,
        hit_rate: 75.0,
    };

    let index_dir = mgr
        .save_index_artifacts(&index_result, FP, Some(&stats))
        .unwrap();

    let manifest = manifest_json(&index_dir);
    assert_eq!(manifest["total_cached_tus"].as_u64(), Some(4));
    assert_eq!(manifest["reused_tus"].as_u64(), Some(3));
    assert_eq!(manifest["reanalyzed_tus"].as_u64(), Some(1));
    assert_eq!(manifest["hit_rate"].as_f64(), Some(75.0));
    assert_eq!(manifest["total_translation_units"].as_u64(), Some(1));
    assert_eq!(manifest["total_symbols"].as_u64(), Some(1));
    assert_eq!(manifest["total_edges"].as_u64(), Some(2));
    assert_eq!(manifest["complete_edges"].as_u64(), Some(1));
    assert_eq!(manifest["incomplete_edges"].as_u64(), Some(1));
    assert_eq!(manifest["input_fingerprint"].as_str(), Some(FP));
    assert_eq!(
        manifest["analysis_version"].as_str(),
        Some(CURRENT_ANALYSIS_VERSION)
    );

    let loaded = mgr.load_index_checked(FP).unwrap();
    assert_eq!(
        loaded.tu_analyses, index_result.tu_analyses,
        "tu_analyses.bin must round-trip the non-empty TU element-for-element"
    );
    assert_eq!(loaded.repo_root, mgr.repo_root());
    assert_eq!(loaded.total_symbols, 1);
    assert_eq!(loaded.total_edges, 2);
    assert_eq!(loaded.complete_edges, 1);
    assert_eq!(loaded.incomplete_edges, 1);
}

/// Text-hook double: the manifest is the subject, the header bytes are not.
struct VerbatimExtractor;

impl ariadne_lang::SourceExtractor for VerbatimExtractor {
    fn extract(
        &self,
        symbols: &[ariadne_lang::SymbolBody],
        _namespace: &str,
        _required_deps: &[String],
    ) -> ariadne_lang::GeneratedSourceFile {
        ariadne_lang::GeneratedSourceFile {
            path: std::path::PathBuf::new(),
            content: format!("// {} symbol(s)\n", symbols.len()),
        }
    }
}

#[test]
fn pin_extraction_manifest_fingerprint_empty() {
    // ADJUDICATED SMELL (extraction fingerprint policy): save_extraction_-
    // artifacts HARDCODES `input_fingerprint: String::new()`, so an
    // extraction manifest can never pass check_manifest's
    // no-empty-fingerprint gate - extraction artifacts are structurally
    // outside the reuse-check machinery. Pinned: the key is present and
    // empty on every save. Owner decision pending on filling it (the
    // extraction stage has no fingerprint plumbed in).
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let unit = ariadne_transform::GeneratedUnit {
        path: std::path::PathBuf::from("gen/pinned.h"),
        symbols: vec![ariadne_transform::ExtractedSymbol {
            name: "Pin".to_string(),
            qualified_id: "Pin".to_string(),
            kind: ariadne_index::SymbolKind::Function,
            body: "// Pin".to_string(),
        }],
        required_deps: vec![],
        primary_source_text: None,
    };
    let result = ariadne_transform::ExtractionResult {
        plan_id: "PIN-T6".to_string(),
        headers: vec![unit],
        errors: vec![],
    };

    let output_dir = dir.path().join("extraction");
    mgr.save_extraction_artifacts(&result, &output_dir, &VerbatimExtractor)
        .unwrap();

    let manifest = manifest_json(&output_dir);
    assert_eq!(manifest["plan_id"].as_str(), Some("PIN-T6"));
    assert_eq!(manifest["header_count"].as_u64(), Some(1));
    assert_eq!(manifest["error_count"].as_u64(), Some(0));
    assert_eq!(
        manifest["input_fingerprint"].as_str(),
        Some(""),
        "pin: the extraction manifest fingerprint is always the empty string"
    );
    assert!(
        manifest.get("input_fingerprint").is_some(),
        "the key must be present (empty), not absent - the skip-key escape \
         hatch (pin_manifest_without_root_key_skips_root_check) does not apply"
    );
    assert_eq!(
        manifest["analysis_version"].as_str(),
        Some(CURRENT_ANALYSIS_VERSION)
    );
}
