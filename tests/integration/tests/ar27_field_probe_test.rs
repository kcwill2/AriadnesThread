//! AR-27a field-probe lock (shape ruling section 2.8): the declared
//! enumeration of every `Serialize`-struct field of type `LanguageId` in the
//! crates' sources, classified by persistence, with the asserted
//! bincode-persisted set EXACTLY `{IncludeEdge.language}`.
//!
//! Why the probe exists: the dual-format positional arm moves ANY
//! LanguageId-typed field's bincode shape; the string-frame exception is
//! ruled for ONE field (`IncludeEdge.language`, warm in
//! `includes/edges.bin` since M24 Part A). A second bincode-persisted
//! LanguageId field appearing anywhere is a PROBE SURPRISE: STOP-AND-REPORT
//! (ruling section 2.8 - second exception field requires a ruling addendum,
//! not improvisation).
//!
//! RED-FREE GREEN-FIRST: authored against the untouched types, green before
//! AND after the move. The JSON locks below were captured from the
//! current types pre-mutation (temp printer, since deleted).
//!
//! Bincode byte locks for the string-framed warm shape live in
//! `crates/lang-api/tests/ar27_dual_format_lock_test.rs`: this package has
//! no bincode dependency, and its Cargo.toml is outside the AR-27a
//! touch-set (journal D1 deviation with reason).

use ariadne_lang_api::identity::LanguageId;
use ariadne_lang_api::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_repo::FileId;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|tests_dir| tests_dir.parent())
        .expect("integration tests live under <repo>/tests/integration")
        .to_path_buf()
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Every `pub language: LanguageId,` declaration under `crates/*/src`,
/// repo-relative and sorted.
fn language_id_field_sites() -> Vec<String> {
    let crates_dir = repo_root().join("crates");
    let mut files = Vec::new();
    collect_rs_files(&crates_dir, &mut files);
    let mut sites: Vec<String> = files
        .iter()
        .filter(|file| {
            std::fs::read_to_string(file)
                .unwrap_or_default()
                .lines()
                .any(|line| line.contains("pub language: LanguageId"))
        })
        .map(|file| {
            file.strip_prefix(repo_root())
                .unwrap_or(file.as_path())
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    sites.sort();
    sites
}

/// The declared census from the shape ruling (section 2.8 "verified clean"
/// list), re-derived by this scan at the AR-27a dispatch HEAD. Each entry:
/// file + persistence class. A site NOT in this census is a probe surprise.
fn classify_site(file: &str) -> &'static str {
    match file {
        // Warm in includes/edges.bin (artifact/manager/graph_io.rs:35 as
        // IncludeGraph.edges) - THE string-frame exception field.
        "crates/lang-api/src/types.rs" => "BINCODE_PERSISTED (includes/edges.bin)",
        // cache.json is a JSON surface only (TuCacheEntry store); no bincode
        // blob carries a fingerprint (ruling section 2.8 list).
        "crates/index/src/cache.rs" => "JSON_ONLY (cache.json)",
        // LanguageModeling persists through plugin JSON surfaces only.
        "crates/lang-api/src/modeling.rs" => "JSON_ONLY",
        // UnitDescriptor: shaper -> build-discovery in-memory seam; the
        // serialized round-trip is in-src test only, no persisted blob.
        "crates/lang-api/src/unit.rs" => "TRANSIENT (shaper seam, not persisted)",
        // PlanningGap has NO serde derive (ruling section 2.8: only
        // PlanningGap carries a language in the plans path and plans.bin
        // never serializes it).
        "crates/cluster/src/planner.rs" => "NOT_SERIALIZED (no serde derive)",
        other => panic!(
            "PROBE SURPRISE (shape ruling section 2.8): LanguageId-typed field in \
             an unclassified file {other} - STOP-AND-REPORT, a second bincode-persisted \
             exception field requires a ruling addendum."
        ),
    }
}

#[test]
fn language_id_typed_src_fields_match_the_declared_enumeration() {
    let found = language_id_field_sites();
    let expected = [
        "crates/cluster/src/planner.rs",
        "crates/index/src/cache.rs",
        "crates/lang-api/src/modeling.rs",
        "crates/lang-api/src/types.rs",
        "crates/lang-api/src/unit.rs",
    ]
    .map(String::from)
    .to_vec();
    assert_eq!(
        found, expected,
        "the LanguageId-field census changed - probe surprise is STOP-AND-REPORT \
         (ruling section 2.8); new bincode-persisted fields may not be waved through"
    );
}

#[test]
fn bincode_persisted_language_id_field_set_is_exactly_include_edge() {
    let persisted: Vec<String> = language_id_field_sites()
        .iter()
        .filter(|file| classify_site(file).starts_with("BINCODE_PERSISTED"))
        .map(|file| format!("{file}: language"))
        .collect();
    assert_eq!(
        persisted,
        vec!["crates/lang-api/src/types.rs: language".to_string()],
        "asserted set EXACTLY {{IncludeEdge.language}} (ruling section 2.8)"
    );
}

fn edge(source: u64, destination: u64, raw_path: &str, language: LanguageId) -> IncludeEdge {
    IncludeEdge {
        source: FileId(source),
        destination: Some(FileId(destination)),
        context_name: "Debug|x64".to_string(),
        source_location: SourceLocation::new(4, 1),
        kind: IncludeKind::Local,
        raw_path: raw_path.to_string(),
        resolved_path: Some(PathBuf::from("include/thing_types.h")),
        language,
    }
}

fn warm_edges() -> Vec<IncludeEdge> {
    vec![
        edge(10, 20, "thing_types.h", LanguageId::Cpp),
        edge(11, 22, "Order.scala", LanguageId::Scala),
        edge(12, 23, "solver.f90", LanguageId::new("Fortran")),
    ]
}

#[test]
fn include_edge_json_locks_bare_names_including_the_open_id() {
    // Captured pre-mutation: the JSON surface spells every language a bare
    // name - legacy ids AND the open id - never an externally-tagged shape
    // (shape ruling section 2.4).
    let cpp_json = serde_json::to_string(&warm_edges()[0]).unwrap();
    assert_eq!(
        cpp_json,
        "{\"source\":10,\"destination\":20,\"context_name\":\"Debug|x64\",\
         \"source_location\":{\"line\":4,\"column\":1},\"kind\":\"Local\",\
         \"raw_path\":\"thing_types.h\",\"resolved_path\":\"include/thing_types.h\",\
         \"language\":\"Cpp\"}"
    );
    let scala_json = serde_json::to_string(&warm_edges()[1]).unwrap();
    assert!(scala_json.contains("\"language\":\"Scala\""), "{scala_json}");
    let open_json = serde_json::to_string(&warm_edges()[2]).unwrap();
    assert!(
        open_json.contains("\"language\":\"Fortran\""),
        "an open id stays a bare name in JSON: {open_json}"
    );
    assert!(!open_json.contains("{\"Open\""), "never an externally-tagged shape: {open_json}");

    let vec_json = serde_json::to_string(&warm_edges()).unwrap();
    assert_eq!(
        serde_json::from_str::<Vec<IncludeEdge>>(&vec_json).unwrap(),
        warm_edges()
    );

    // Additive-field contract (plan 3.5): edges written before the
    // `language` key existed keep reading back as Cpp.
    let without_language = cpp_json.replace(",\"language\":\"Cpp\"", "");
    let back: IncludeEdge = serde_json::from_str(&without_language).unwrap();
    assert_eq!(back, warm_edges()[0]);
    assert_eq!(back.language, LanguageId::Cpp);
}
