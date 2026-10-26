//! rust-language R-4 discovery lock (milestone-named per the guide Step-5
//! rule - the track key `rust_language_`, NOT an mNN name). Model suite:
//! `m19_scala_discovery_test.rs` (fixture -> TempDir -> registry/handler
//! calls -> model + manifest asserts). The identity-byte claims mirror
//! `m24_language_identity_test.rs` claims 1+2 for the REAL plugin id - m24
//! is cited, never edited (its Fortran mock rehearses N=3; this hop makes
//! it real around it). The positional byte-framing of the escape arm is
//! byte-frozen UNEDITED in `crates/lang-api/tests/ar27_dual_format_lock_test
//! .rs` (`discriminator_selects_the_wire_arm_by_format`,
//! `positional_form_names_an_open_id_through_the_escape_arm`) - this package
//! has no bincode dependency (placement note in `ar27_field_probe_test.rs`),
//! so this suite proves the escape through the PRODUCTION writer instead.
//!
//! What this suite locks at the registration hop:
//! INV-6 fixture discovery/shaping truth on `tests/fixtures/rust-basic/`;
//! INV-9 `.rs` claimed case-exactly, `plan_backend` witnesses live;
//! INV-1 enumeration `[Cpp, Scala, Rust]` from the integration side;
//! INV-2 first-wins probes for `.cpp`/`.scala` unchanged;
//! INV-5 capability literal (discovery ONLY);
//! INV-4 identity bytes for the real plugin id (JSON bare name + dispatch +
//! live positional escape bytes in a production `files.bin` blob).

use anyhow::Result;
use ariadne_build::BuildDiscoverer;
use ariadne_lang::{LanguageCapabilities, LanguageId, LanguagePlugin};
use ariadne_repo::{
    FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, Repository, SourceLanguage,
};
use ariadne_report::ArtifactManager;
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE: &str = "rust-basic";

/// The five `src/**.rs` sources the shaper must turn into units.
const SRC_UNITS: [&str; 5] = [
    "src/greeting.rs",
    "src/lib.rs",
    "src/macro_specimen.rs",
    "src/models/mod.rs",
    "src/models/user.rs",
];

/// The test-side file: recorded by discovery, excluded from units (the
/// `tests` exclusion tag, m19's test-root-spec idiom).
const TEST_SIDE_FILE: &str = "tests/cli_smoke.rs";

/// The external/frozen side: recorded ThirdParty/Frozen, never a unit of
/// context (no `src` component). `target/` cannot be committed, so the
/// external marker rides `vendor/` per shape sec. 4.
const VENDOR_FILE: &str = "vendor/legacy/legacy.rs";

struct IsolatedRustRepo {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fixtures")
        .join(name)
}

/// Throwaway copy of the fixture (m19 idiom: the frozen fixture set is read
/// only, never reused as a harness).
fn isolated_rust_repo(label: &str) -> Result<IsolatedRustRepo> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join(label);
    copy_tree(&fixture_path(FIXTURE), &root)?;
    Ok(IsolatedRustRepo { _temp: temp, root })
}

fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn normalized_rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn sorted_file_ids(paths: &[&str]) -> Vec<u64> {
    let mut ids: Vec<u64> = paths
        .iter()
        .map(|path| FileId::from_relative_path(path).0)
        .collect();
    ids.sort();
    ids
}

/// INV-6 + shape sec. 4: discovery records every `.rs` of the fixture and
/// nothing else - `Cargo.toml` (the build marker) is `FileKind::Other` on
/// the spine table (extension `toml` is shared, shape sec. 3.5) and is
/// never recorded, never analyzable, never a unit.
#[test]
fn discovery_records_every_rust_source_and_never_the_manifest() -> Result<()> {
    let repo_copy = isolated_rust_repo("discover")?;
    let repository = Repository::locate(&repo_copy.root)?;
    let discovery = repository.discover()?;

    let mut actual: Vec<String> = discovery
        .files
        .iter()
        .map(|record| normalized_rel(&record.path))
        .collect();
    actual.sort();
    let mut expected: Vec<String> = SRC_UNITS.iter().map(|path| path.to_string()).collect();
    expected.push(TEST_SIDE_FILE.to_string());
    expected.push(VENDOR_FILE.to_string());
    expected.sort();
    assert_eq!(
        actual, expected,
        "exactly the seven .rs files are recorded; the manifest is not"
    );

    for record in &discovery.files {
        assert_eq!(record.kind, FileKind::RustSource);
        assert_eq!(record.language, SourceLanguage::Rust);
    }
    for record in &discovery.files {
        let rel = normalized_rel(&record.path);
        if rel == VENDOR_FILE {
            assert_eq!(record.ownership, FileOwnership::ThirdParty);
            assert_eq!(record.optimization_policy, OptimizationPolicy::Frozen);
        } else {
            assert_eq!(record.ownership, FileOwnership::Repository);
            assert_eq!(record.optimization_policy, OptimizationPolicy::Optimizable);
        }
    }
    Ok(())
}

/// INV-6 unit truth through the registry-selected shaper (D22 marker
/// selection: only `Cargo.toml` sits at the fixture root, so the Rust
/// shaper wins - registration order still had C++/Scala first, they simply
/// own no marker here). One unit per `src/**.rs`, one root context named by
/// the nearest `Cargo.toml` ancestor, `rustc`, no C++/Scala flavor.
#[test]
fn shaper_selection_and_unit_truth_on_the_fixture() -> Result<()> {
    let repo_copy = isolated_rust_repo("shaper")?;
    let registry = ariadne_cli::registry::default_registry();
    let repository = Repository::locate(&repo_copy.root)?;
    let discovery = repository.discover()?;
    let shaper = registry
        .unit_shaper_for_repo(&repository.root)
        .expect("the cargo marker at the fixture root selects a registered shaper");
    assert_eq!(
        shaper.build_markers(),
        vec!["Cargo.toml"],
        "D22 marker selection reached the Rust shaper (the C++/Scala markers \
         are absent at this root; first-registration order still answers first \
         wherever THEIR markers are)"
    );

    let build = BuildDiscoverer::new(&repository.root)
        .with_repo_discovery(&discovery)
        .with_unit_shaper(Some(shaper))
        .discover()?;

    assert_eq!(build.contexts.len(), 1, "one cargo root context");
    let context = &build.contexts[0];
    assert_eq!(context.name, "root", "the only Cargo.toml is at the root");
    assert_eq!(context.compiler, "rustc");
    assert_eq!(context.language, SourceLanguage::Rust);

    assert_eq!(
        build.translation_units.len(),
        SRC_UNITS.len(),
        "one unit per src-rooted .rs; the tests/ and vendor/ files are not units"
    );
    let mut unit_ids: Vec<u64> = build
        .translation_units
        .iter()
        .map(|unit| unit.file_id.0)
        .collect();
    unit_ids.sort();
    assert_eq!(unit_ids, sorted_file_ids(&SRC_UNITS));
    for unit in &build.translation_units {
        assert_eq!(unit.language, SourceLanguage::Rust);
        let rel = normalized_rel(&unit.source_path);
        assert!(
            !rel.contains("vendor") && !rel.contains("tests/") && !rel.contains("Cargo.toml"),
            "excluded-side path leaked into the unit set: {rel}"
        );
    }

    let foreign_flavored = build.contexts.iter().any(|ctx| {
        ctx.compiler == "cl.exe"
            || ctx.compiler == "scalac"
            || ctx.language == SourceLanguage::Cpp
            || ctx.language == SourceLanguage::Scala
    });
    assert!(
        !foreign_flavored,
        "no C++/Scala-flavored context appears (D8-lock idiom from m19)"
    );
    Ok(())
}

/// INV-1 + INV-2 from the integration side: enumeration is
/// `[Cpp, Scala, Rust]` and the shipped detectors still answer before
/// Rust's claim for their own extensions (first registration order held).
#[test]
fn enumeration_and_first_wins_probes() {
    let registry = ariadne_cli::registry::default_registry();
    assert_eq!(
        registry.languages(),
        vec![LanguageId::Cpp, LanguageId::Scala, LanguageId::Rust],
        "registration appends Rust LAST (shape sec. 3.2, INV-1/INV-2)"
    );
    assert!(registry.plugin_for_id(LanguageId::Rust).is_some());
    assert_eq!(
        registry.detect_language(Path::new("src/a.cpp")),
        Some(LanguageId::Cpp),
        "a .cpp probe still resolves Cpp - the shipped detector answers first"
    );
    assert_eq!(
        registry.detect_language(Path::new("src/Main.scala")),
        Some(LanguageId::Scala),
        "a .scala probe still resolves Scala"
    );
}

/// INV-9 `.rs` claimed, consistently: the exact-set parse pins, the
/// detection row, and the A3/A4 discipline witnesses live - a `plan_backend`
/// on `src/unclaimed.rs` now RESOLVES (this suite does NOT pin it as a
/// rejection anymore; the cli A3/A4 tests moved to `.f90` witnesses).
#[test]
fn rs_claimed_case_exact_and_plan_backend_resolves() {
    let registry = ariadne_cli::registry::default_registry();
    assert_eq!(
        ariadne_cli::registry::parse_language_arg("rust").unwrap(),
        SourceLanguage::Rust
    );
    assert_eq!(
        ariadne_cli::registry::parse_language_arg("Rust").unwrap(),
        SourceLanguage::Rust,
        "the exact PascalCase serde name parses (exact-set discipline)"
    );
    assert!(
        ariadne_cli::registry::parse_language_arg("RUST").is_err(),
        "RUST keeps its case and matches no serde name - same class as CPP/SCALA"
    );
    assert_eq!(
        registry.detect_language(Path::new("src/lib.rs")),
        Some(LanguageId::Rust)
    );

    // The witness inverts: `src/unclaimed.rs` now resolves for an enabled
    // capability...
    assert!(
        ariadne_cli::registry::plan_backend(
            &registry,
            Path::new("src/unclaimed.rs"),
            |caps| caps.discovery,
            "discovery",
        )
        .is_ok(),
        "post-registration a .rs plan target RESOLVES through the Rust plugin"
    );
    // ...and where the probed capability is still OFF (R-5 lands the
    // analyzer), the honest error is the CAPABILITY error, never the
    // unclaimed-target error.
    let capability_off = ariadne_cli::registry::plan_backend(
        &registry,
        Path::new("src/unclaimed.rs"),
        |caps| caps.extraction,
        "extraction",
    )
    .err()
    .expect("Rust keeps extraction false at this hop (INV-5)");
    let capability_text = capability_off.to_string();
    assert!(
        !capability_text.contains("no language plugin claims")
            && capability_text.contains("does not enable"),
        "the .rs target must get the capability-off error, never the \
         unclaimed-target error: {capability_text}"
    );
}

/// INV-5 capability lock, integration side, m19 idiom: an INDEPENDENT
/// literal - R-4 flipped `discovery`, R-5 added `semantic_index`, R-6's
/// resolver hop adds `dependency_graph` (m23 Scala precedent: superseded
/// flip-locks are amended with the flip, never weakened).
#[test]
fn capabilities_lock_the_registration_flip_literal() {
    let registry = ariadne_cli::registry::default_registry();
    let rust = registry
        .plugin_for_id(LanguageId::Rust)
        .expect("Rust registered in the CLI default registry (R-4)");
    assert_eq!(
        rust.capabilities(),
        LanguageCapabilities {
            discovery: true,
            dependency_graph: true,
            semantic_index: true,
            load_metric: false,
            planning: false,
            extraction: false,
            rewriting: false,
            validation: false,
            requires_toolchain: false,
        },
        "R-4 turned discovery on, R-5 added semantic_index, R-6 adds \
         dependency_graph and ONLY that (INV-5 monotonicity); \
         requires_toolchain stays false (A3/D-2)"
    );
}

/// INV-4 identity bytes, mirroring m24 claim 1 (`serde_json` transparent
/// bare name + Debug/Display) and claim 2 (resolution through detection,
/// id lookup AND the CLI catalog dispatch) - for the REAL plugin id taken
/// from the registered plugin, not the m24 Fortran mock. m24 is cited,
/// never edited.
#[test]
fn identity_bytes_mirror_m24_claims_for_the_real_plugin_id() {
    let registry = ariadne_cli::registry::default_registry();
    let plugin = registry
        .plugin_for_id(LanguageId::Rust)
        .expect("the registered Rust plugin is the real identity source");
    let real_id = plugin.id();
    assert_eq!(real_id, LanguageId::Rust);
    assert_eq!(real_id.as_str(), "Rust");

    // m24 claim 1 mechanism: the newtype serializes as the bare name, so
    // artifacts round-trip byte-identically in both directions.
    let json = serde_json::to_string(&real_id).expect("serialize a language id");
    assert_eq!(json, "\"Rust\"", "a language id is a bare string");
    let back: LanguageId = serde_json::from_str(&json).expect("deserialize it");
    assert_eq!(back, real_id);
    // Debug/Display stay bare (TU cache digest hashes `{language:?}`).
    assert_eq!(format!("{:?}", real_id), "Rust");
    assert_eq!(format!("{}", real_id), "Rust");

    // m24 claim 2 mechanism: the real identity resolves through detection,
    // id lookup, AND the CLI catalog dispatch - with the record carrying
    // the deliberately WRONG `Other` label (D-A5: the registry decides).
    assert_eq!(
        registry.detect_language(Path::new("src/lib.rs")),
        Some(real_id.clone()),
        "plugin detection is authoritative for .rs"
    );
    let mislabeled = FileRecord {
        id: FileId::from_relative_path("src/lib.rs"),
        path: PathBuf::from("src/lib.rs"),
        canonical_path: PathBuf::from("src/lib.rs"),
        kind: FileKind::Other,
        size: 0,
        content_hash: String::new(),
        ownership: FileOwnership::Repository,
        optimization_policy: OptimizationPolicy::Optimizable,
        language: SourceLanguage::Other,
    };
    let discovery = ariadne_repo::RepositoryDiscoveryResult {
        root: PathBuf::from("."),
        files: vec![mislabeled],
        ownership_summary: Default::default(),
        policy_summary: Default::default(),
    };
    let catalog = ariadne_cli::registry::catalog_languages(&registry, &discovery);
    assert_eq!(catalog.get(&FileId::from_relative_path("src/lib.rs")), Some(&real_id));
    assert_eq!(
        ariadne_cli::registry::plugin_for_catalog_file(
            &registry,
            &catalog,
            FileId::from_relative_path("src/lib.rs"),
        )
        .expect("a mislabeled record still dispatches through the registry")
        .id(),
        real_id
    );
}

/// INV-4 positional bytes, LIVE-witness form: the PRODUCTION writer
/// (`ArtifactManager::save_repository_artifacts`, the exact call the
/// discover command makes) bincode-serializes the fixture's Rust-labelled
/// records into `repository/files.bin`. An open id under a positional
/// writer takes the escape arm tag 3 + u64 LE length + UTF-8; here that is
/// `[3,0,0,0] [4,0,0,0,0,0,0,0] b"Rust"` for the REAL plugin id, and the
/// same blob decodes back to `LanguageId::Rust`. The stored trio's u32 LE
/// indices 0/1/2 are byte-frozen UNEDITED by
/// `crates/repo/tests/ar27_old_bytes_lock_test.rs` and
/// `crates/lang-api/tests/ar27_dual_format_lock_test.rs` (green in the
/// census); this package carries no bincode dependency by placement note,
/// so the framing proof rides those locks and this test proves the REAL id
/// rides the escape arm through the writer that produces the warm blobs.
#[test]
fn warm_files_bin_blob_carries_the_open_escape_bytes_for_rust() -> Result<()> {
    let repo_copy = isolated_rust_repo("escape")?;
    let repository = Repository::locate(&repo_copy.root)?;
    let discovery = repository.discover()?;

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    let repository_dir = artifacts.save_repository_artifacts(&discovery, "rust-language-r4-witness")?;
    let blob = fs::read(repository_dir.join("files.bin"))?;

    let escape_arm: [u8; 16] = [
        3, 0, 0, 0, // variant index 3 (Open) as u32 LE
        4, 0, 0, 0, 0, 0, 0, 0, // string length as u64 LE
        b'R', b'u', b's', b't', // b"Rust"
    ];
    assert!(
        blob.windows(escape_arm.len())
            .any(|window| window == escape_arm.as_slice()),
        "the production positional writer must spell the real plugin id \
         through the tag-3 Open escape (tag 3 + u64 LE length + the four \
         UTF-8 bytes)"
    );

    let decoded = artifacts.load_repository_artifacts()?;
    assert_eq!(decoded.len(), discovery.files.len());
    assert!(
        decoded.iter().all(|record| record.language == LanguageId::Rust),
        "the escape-arm bytes decode back to the real plugin id at the \
         frozen epoch - never a panic, never a silent trio substitution"
    );
    Ok(())
}
