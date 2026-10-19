//! UTA-T6 item 9 (pin-only) locks for the pipeline crate.
//!
//! - `try_reuse_persisted_load`: a corrupt `report.bin` behind a FRESH
//!   manifest is swallowed by `.ok()` and the whole stage cold-rebuilds,
//!   surfacing Ok (the pre-existing rebuild-on-doubt contract, named at
//!   stage_load.rs:162-172).
//! - `--lang` filtered index: units of the filter language are retained and
//!   NO full-fingerprint index manifest is written (a filtered view never
//!   poisons the authoritative artifact).
//! - `for_incremental` unit validation: a unit no registered plugin claims
//!   bails the construction with a named error.
//! - `RunStats::record` replace semantics + `format_summary` both branches.
//! - `cycle_policy_by_file`: unclaimed language falls back to `Reject`.
//!
//! Fixture discipline mirrors `stage_load_tests.rs`: directive text assembled
//! from a non-contiguous marker, generic `cc` compile commands, no plugin
//! names (R2 guard walk).

use super::*;
use ariadne_repo::{FileKind, FileOwnership, FileRecord, OptimizationPolicy};
use std::fs;
use tempfile::TempDir;

/// Two-unit C++-fixture repo (same sanctioned shape as
/// `stage_load_tests::setup_repo`); `extra_unit` optionally appends one
/// compile-database entry for an extension no plugin claims.
fn setup_repo(extra_unit: Option<&str>) -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("include/common.h"), "struct Common { int id; };").unwrap();
    fs::write(
        root.join("include/header_a.h"),
        concat!("#inc", "lude \"common.h\"\nstruct ThingA { Common c; };"),
    )
    .unwrap();
    fs::write(root.join("src/tu1.cpp"), concat!("#inc", "lude \"header_a.h\"\nvoid use_a(ThingA a) {}")).unwrap();
    fs::write(root.join("src/tu2.cpp"), "void use_b(struct Common b) { (void)b; }").unwrap();

    let root_str = root.to_string_lossy().replace('\\', "/");
    let extra = extra_unit.map_or(String::new(), |unit| {
        format!(
            ",\n    {{ \"directory\": \"{root_str}\", \"command\": \"cc -c {unit}\", \"file\": \"{unit}\" }}"
        )
    });
    let compile_commands = format!(
        r#"[
    {{ "directory": "{root_str}", "command": "cc -Iinclude -c src/tu1.cpp", "file": "src/tu1.cpp" }},
    {{ "directory": "{root_str}", "command": "cc -Iinclude -c src/tu2.cpp", "file": "src/tu2.cpp" }}{extra}
]"#,
        root_str = root_str
    );
    fs::write(root.join("compile_commands.json"), compile_commands).unwrap();
    dir
}

// ---------------------------------------------------------------------------
// Item 9a - corrupt report.bin behind a fresh manifest
// ---------------------------------------------------------------------------

#[test]
fn pin_corrupt_report_bin_with_fresh_manifest_rebuilds_after_the_ok_swallow() {
    // The `.ok()` on stage_load.rs:171 erases WHY the verified load failed:
    // fresh manifest + corrupt blob is indistinguishable (from the caller's
    // seat) from a stale stage. Pinned contract: `ensure_load` still
    // answers Ok, the stage records Rebuilt (never Loaded), and the rebuild
    // rewrites a decodable blob.
    let dir = setup_repo(None);
    {
        let mut cold = Pipeline::new(dir.path(), None).unwrap();
        cold.ensure_load().unwrap();
    }
    let report_bin = dir
        .path()
        .join("artifacts")
        .join("load")
        .join("report.bin");
    fs::write(&report_bin, b"corrupted-payload-not-bincode").unwrap();

    let mut warm = Pipeline::new(dir.path(), None).unwrap();
    assert!(
        warm.manager.stage_is_fresh(STAGE_LOAD, &warm.fps.load),
        "pin premise: the manifest must still read fresh after blob corruption"
    );
    assert!(
        warm.manager.load_load_checked(&warm.fps.load).is_err(),
        "pin premise: the corrupt blob must fail to decode through the checked path"
    );

    warm.ensure_load()
        .expect("pin: the swallowed decode Err must route into a cold rebuild, not a hard failure");
    assert_eq!(
        warm.stats.stage_reuse(STAGE_LOAD),
        Some(StageReuse::Rebuilt),
        "pin: the corrupt-blob leg records Rebuilt - the swallow hides the cause, the outcome is a rebuild"
    );
    warm.manager
        .load_load_checked(&warm.fps.load)
        .expect("pin: the rebuild must re-persist a decodable report.bin");
}

// ---------------------------------------------------------------------------
// Item 9b - `--lang` filtered index path
// ---------------------------------------------------------------------------

#[test]
fn pin_lang_filtered_index_keeps_units_and_writes_no_full_fingerprint_manifest() {
    // stage_index.rs:74-80: a filtered run keeps its units in the cache but
    // never persists them under the FULL index fingerprint; a following
    // unfiltered run must therefore still be a Rebuilt (nothing was left
    // that could poison it).
    let dir = setup_repo(None);
    let mut filtered = Pipeline::new(dir.path(), None)
        .unwrap()
        .language_filter(Some(SourceLanguage::Cpp));
    filtered.ensure_index().unwrap();

    let index = filtered
        .index_cache
        .as_ref()
        .expect("filtered run must have built an index");
    assert_eq!(
        index.tu_analyses.len(),
        2,
        "pin: the language filter retains both discoverable units"
    );
    assert!(
        !filtered
            .manager
            .artifact_root
            .join("index")
            .join("manifest.json")
            .exists(),
        "pin: a filtered run must not write the full-fingerprint index manifest"
    );
    assert!(!filtered.manager.stage_is_fresh(STAGE_INDEX, &filtered.fps.index));
    assert_eq!(
        filtered.stats.stage_reuse(STAGE_INDEX),
        Some(StageReuse::Rebuilt)
    );
    drop(filtered);

    let mut full = Pipeline::new(dir.path(), None).unwrap();
    full.ensure_index().unwrap();
    assert_eq!(
        full.stats.stage_reuse(STAGE_INDEX),
        Some(StageReuse::Rebuilt),
        "pin: after a filtered-only run the first full run still rebuilds - \
         the filtered run poisoned nothing"
    );
}

// ---------------------------------------------------------------------------
// Item 9c - for_incremental unhandled-language bail
// ---------------------------------------------------------------------------

#[test]
fn pin_for_incremental_bails_on_an_unhandled_language_unit() {
    // Control: the all-claimed fixture constructs with a fully retained view.
    let plain = setup_repo(None);
    let ok = Pipeline::for_incremental(plain.path(), None).unwrap();
    let view = ok
        .incremental_view
        .as_ref()
        .expect("for_incremental always records the view");
    assert_eq!(view.discovered_units, 2);
    assert_eq!(view.retained_units, 2);
    assert!(view.artifacts_are_authoritative());
    drop(ok);

    // With an entry for an extension NO registered plugin claims (no
    // `--lang`, so nothing filters it out first), the validation leg bails
    // construction, naming the unit.
    let mixed = setup_repo(Some("src/weird.xyz"));
    let err = match Pipeline::for_incremental(mixed.path(), None) {
        Ok(_unbailed) => panic!("pin: an unclaimed unit must bail the construction, never ride along silently"),
        Err(err) => err,
    };
    let message = err.to_string();
    assert!(
        message.contains("unhandled language") && message.contains("weird.xyz"),
        "error must name the condition and the offending unit, got: {message}"
    );
}

// ---------------------------------------------------------------------------
// Item 9d - RunStats record + format_summary
// ---------------------------------------------------------------------------

#[test]
fn run_stats_record_replaces_per_stage() {
    let mut stats = RunStats::default();
    stats.record("alpha", StageReuse::Loaded);
    stats.record("alpha", StageReuse::Rebuilt);
    stats.record("beta", StageReuse::Loaded);

    assert_eq!(
        stats.stages.len(),
        2,
        "pin: record REPLACES the same-stage entry (retain + push), never appends a duplicate"
    );
    assert_eq!(stats.stage_reuse("alpha"), Some(StageReuse::Rebuilt));
    assert_eq!(stats.stage_reuse("beta"), Some(StageReuse::Loaded));
    assert_eq!(stats.stage_reuse("gamma"), None);
}

#[test]
fn run_stats_format_summary_covers_both_empty_and_filled_branches() {
    // Each side of the summary renders "none" only when its own list is
    // empty; the joined side is comma-space separated in stage order.
    let empty = RunStats::default();
    assert_eq!(
        empty.format_summary(),
        "0 TUs reused, 0 reanalyzed; stages loaded: none; rebuilt: none"
    );

    let mut filled = RunStats::default();
    filled.reused_tus = 3;
    filled.reanalyzed_tus = 1;
    filled.record("a", StageReuse::Loaded);
    filled.record("b", StageReuse::Loaded);
    filled.record("c", StageReuse::Rebuilt);
    assert_eq!(
        filled.format_summary(),
        "3 TUs reused, 1 reanalyzed; stages loaded: a, b; rebuilt: c"
    );
}

// ---------------------------------------------------------------------------
// Item 9e - cycle_policy_by_file unknown-plugin fallback
// ---------------------------------------------------------------------------

fn discovery_record(id: u64, rel: &str, language: SourceLanguage) -> FileRecord {
    FileRecord {
        id: FileId(id),
        path: PathBuf::from(rel),
        canonical_path: PathBuf::from(rel),
        kind: FileKind::Source,
        size: 0,
        content_hash: String::new(),
        ownership: FileOwnership::Repository,
        optimization_policy: OptimizationPolicy::Optimizable,
        language,
    }
}

#[test]
fn pin_cycle_policy_unknown_plugin_falls_back_to_reject() {
    let registry = default_registry();
    let discovery = RepositoryDiscoveryResult {
        root: PathBuf::from("/fixture"),
        files: vec![
            discovery_record(1, "src/known.cpp", SourceLanguage::Cpp),
            discovery_record(2, "src/foreign.src", SourceLanguage::Other),
        ],
        ownership_summary: HashMap::new(),
        policy_summary: HashMap::new(),
    };

    let policies = cycle_policy_by_file(&registry, &discovery);
    let cpp_plugin = registry
        .plugin_for_id(LanguageId::from(SourceLanguage::Cpp))
        .expect("the default registry claims C++");
    assert_eq!(
        policies[&FileId(1)],
        cpp_plugin.modeling().cycle_policy,
        "a claimed language answers its plugin's stance"
    );
    assert_eq!(
        policies[&FileId(2)],
        CyclePolicy::Reject,
        "pin: a language no registered plugin claims falls back to Reject"
    );
}
