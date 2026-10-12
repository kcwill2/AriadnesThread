//! A6 (AR-22) parity lock: `Pipeline::for_incremental` must configure a
//! pipeline IDENTICALLY to the sequence `crates/cli/src/commands/analyze.rs`
//! mirrors by hand today (`run_incremental_analysis_filtered` lines 32-56
//! plus its `restrict_and_validate_units` / `artifacts_are_authoritative`
//! helpers, and the `ensure_language_registered` guard from
//! `crates/cli/src/registry.rs`). The mirrored sequence is REPLICATED here
//! (the CLI file itself stays untouched - AR-23 owns the switch) and both
//! constructions are compared surface by surface on the same temp fixture:
//! repo root, discovery records, the (restricted) build result, the stage
//! fingerprint chain, the artifact root, and the recorded
//! restriction/authority state. See the journal for the parity surface and
//! the unobservable fields.
//!
//! The fixture shapes make the restriction mutation-sensitive: one real
//! narrowing on each side (Cpp filter over a Scala-shaped unit set, Scala
//! filter over a C++-shaped one), so skipping the ctor's restrict step
//! turns this suite red rather than leaving it vacuously green.

use std::fs;
use std::path::{Path, PathBuf};

use ariadne_build::{BuildDiscoverer, BuildDiscoveryResult};
use ariadne_index::StageFingerprints;
use ariadne_lang::LanguageRegistry;
use ariadne_pipeline::*;
use ariadne_repo::{FileRecord, Repository, SourceLanguage};
use ariadne_report::ArtifactManager;
use tempfile::TempDir;

/// Scala build marker present, every C++ marker absent: registry marker
/// selection routes shaping to the Scala shaper, so the main-root `.scala`
/// becomes the only unit while the `.cpp` stays discovery-only. A Cpp
/// filter therefore really narrows the unit set.
fn scala_marker_repo() -> TempDir {
    let dir = TempDir::new().expect("temp repo root");
    let root = dir.path();
    fs::create_dir_all(root.join("src/main/scala")).expect("scala source dir");
    fs::write(root.join("build.sbt"), "name := \"parity-fixture\"\n").expect("write marker");
    fs::write(
        root.join("src/main/scala/App.scala"),
        "object App { def main(args: Array[String]): Unit = () }\n",
    )
    .expect("write scala source");
    fs::write(
        root.join("src/legacy.cpp"),
        "int legacy() { return 1; }\n",
    )
    .expect("write cpp source");
    dir
}

/// No root build markers at all: marker selection falls back to
/// registration order (C++ first), so `src/tu.cpp` shapes into the only
/// unit while the `.scala` stays discovery-only. A Scala filter therefore
/// really narrows the unit set.
fn cpp_repo() -> TempDir {
    let dir = TempDir::new().expect("temp repo root");
    let root = dir.path();
    fs::create_dir_all(root.join("include")).expect("include dir");
    fs::create_dir_all(root.join("src")).expect("src dir");
    fs::write(root.join("include/common.h"), "struct Common { int id; };\n").expect("write header");
    fs::write(root.join("src/tu.cpp"), "#include \"common.h\"\nint tu() { return 2; }\n")
        .expect("write cpp source");
    dir
}

/// Everything the mirrored analyze.rs sequence determines, observable here
/// without running any stage or persisting any stage artifact.
struct MirroredState {
    repo_root: PathBuf,
    discovery_root: PathBuf,
    discovery_files: Vec<FileRecord>,
    build: BuildDiscoveryResult,
    fps: StageFingerprints,
    artifact_root: PathBuf,
    language_filter: Option<SourceLanguage>,
    discovered_units: usize,
    retained_units: usize,
}

impl MirroredState {
    /// Replication of analyze.rs `artifacts_are_authoritative`: only an
    /// unfiltered run whose unit set was never narrowed is authoritative.
    fn artifacts_are_authoritative(&self) -> bool {
        self.language_filter.is_none() && self.retained_units >= self.discovered_units
    }
}

/// Replication of `ariadne_cli::registry::ensure_language_registered`.
fn ensure_language_registered(
    registry: &LanguageRegistry,
    language: SourceLanguage,
) -> Result<(), String> {
    if registry
        .plugin_for_id(ariadne_lang::LanguageId::from(language.clone()))
        .is_none()
    {
        return Err(format!("language plugin not registered: {language:?}"));
    }
    Ok(())
}

/// Replication of analyze.rs `restrict_and_validate_units` (retain by the
/// registry-detected identity, then require every retained unit to be
/// claimed by some registered plugin - §3.3 fact 3).
fn restrict_and_validate_units(
    registry: &LanguageRegistry,
    language: Option<SourceLanguage>,
    build: &mut BuildDiscoveryResult,
) -> Result<(), String> {
    if let Some(language) = language {
        build.translation_units.retain(|tu| {
            registry.detect_language(&tu.source_path)
                == Some(ariadne_lang::LanguageId::from(language.clone()))
        });
    }
    for tu in &build.translation_units {
        if registry.detect_language(&tu.source_path).is_none() {
            return Err(format!(
                "unhandled language for {} (no registered plugin claims it)",
                tu.source_path.display()
            ));
        }
    }
    Ok(())
}

/// The analyze.rs construction sequence, line for line, as a test-local
/// replication. Returns the error TEXT (not anyhow) so the unregistered
/// language case can be compared verbatim against the constructor.
fn mirror_construct(
    repo_path: &Path,
    language: Option<SourceLanguage>,
) -> Result<MirroredState, String> {
    let registry = build_default_registry();
    if let Some(language) = language.clone() {
        ensure_language_registered(&registry, language)?;
    }
    let repo = Repository::locate(repo_path).map_err(|err| err.to_string())?;
    let repo_discovery = repo.discover().map_err(|err| err.to_string())?;
    let mut build_discovery = BuildDiscoverer::new(&repo.root)
        .with_repo_discovery(&repo_discovery)
        .with_unit_shaper(registry.unit_shaper_for_repo(&repo.root))
        .discover()
        .map_err(|err| err.to_string())?;
    let fps = StageFingerprints::from_inputs(
        &repo.root,
        &repo_discovery,
        &build_discovery.contexts,
        &build_discovery.translation_units,
    );
    let discovered_units = build_discovery.translation_units.len();
    restrict_and_validate_units(&registry, language.clone(), &mut build_discovery)?;
    let manager = ArtifactManager::init(&repo.root).map_err(|err| err.to_string())?;
    let retained_units = build_discovery.translation_units.len();
    Ok(MirroredState {
        repo_root: repo.root,
        discovery_root: repo_discovery.root,
        discovery_files: repo_discovery.files,
        build: build_discovery,
        fps,
        artifact_root: manager.artifact_root,
        language_filter: language,
        discovered_units,
        retained_units,
    })
}

/// Compare every inspectable configuration surface.
fn assert_parity(state: &MirroredState, pipeline: &Pipeline) {
    assert_eq!(
        pipeline.repo.root, state.repo_root,
        "parity surface: located repository root"
    );
    assert_eq!(
        pipeline.discovery.root, state.discovery_root,
        "parity surface: discovery root"
    );
    assert_eq!(
        pipeline.discovery.files, state.discovery_files,
        "parity surface: discovery records"
    );
    assert_eq!(
        pipeline.build, state.build,
        "parity surface: build discovery result (contexts + restricted unit set)"
    );
    assert_eq!(
        pipeline.fps, state.fps,
        "parity surface: stage fingerprint chain"
    );
    assert_eq!(
        pipeline.artifact_root(), state.artifact_root,
        "parity surface: artifact root"
    );
    let view = pipeline
        .incremental_view
        .as_ref()
        .expect("for_incremental must record its unit-restriction/authority state");
    assert_eq!(
        view.language_filter, state.language_filter,
        "parity surface: recorded language restriction"
    );
    assert_eq!(
        view.discovered_units, state.discovered_units,
        "parity surface: discovered unit count"
    );
    assert_eq!(
        view.retained_units, state.retained_units,
        "parity surface: retained unit count"
    );
    assert_eq!(
        view.artifacts_are_authoritative(),
        state.artifacts_are_authoritative(),
        "parity surface: artifact authority rule"
    );
    assert_eq!(
        view.retained_units,
        pipeline.build.translation_units.len(),
        "recorded state must agree with the pipeline's own restricted unit set"
    );
}

#[test]
fn unrestricted_run_on_scala_marker_repo_matches_mirrored_sequence() {
    let dir = scala_marker_repo();
    let state = mirror_construct(dir.path(), None).expect("mirror constructs");
    let pipeline =
        Pipeline::for_incremental(dir.path(), None).expect("constructor constructs");
    assert!(
        state.artifacts_are_authoritative(),
        "fixture guard: the unfiltered run must be authoritative on both sides"
    );
    assert_parity(&state, &pipeline);
}

#[test]
fn cpp_filter_on_scala_marker_repo_matches_mirrored_sequence() {
    let dir = scala_marker_repo();
    let state = mirror_construct(dir.path(), Some(SourceLanguage::Cpp)).expect("mirror constructs");
    let pipeline =
        Pipeline::for_incremental(dir.path(), Some(SourceLanguage::Cpp))
            .expect("constructor constructs");
    assert_eq!(
        state.retained_units, 0,
        "fixture guard: the Cpp filter must truly narrow the Scala-shaped set \
         (mutation-sensitivity of this lock)"
    );
    assert!(
        !state.artifacts_are_authoritative(),
        "fixture guard: an active filter is never authoritative"
    );
    assert_parity(&state, &pipeline);
}

#[test]
fn scala_filter_on_cpp_repo_matches_mirrored_sequence() {
    let dir = cpp_repo();
    let state = mirror_construct(dir.path(), Some(SourceLanguage::Scala)).expect("mirror constructs");
    let pipeline =
        Pipeline::for_incremental(dir.path(), Some(SourceLanguage::Scala))
            .expect("constructor constructs");
    assert_eq!(
        state.retained_units, 0,
        "fixture guard: the Scala filter must truly narrow the C++-shaped set \
         (mutation-sensitivity of this lock)"
    );
    assert_parity(&state, &pipeline);
}

#[test]
fn cpp_filter_on_cpp_repo_matches_mirrored_sequence() {
    let dir = cpp_repo();
    let state = mirror_construct(dir.path(), Some(SourceLanguage::Cpp)).expect("mirror constructs");
    let pipeline =
        Pipeline::for_incremental(dir.path(), Some(SourceLanguage::Cpp))
            .expect("constructor constructs");
    assert_eq!(
        state.retained_units, state.discovered_units,
        "fixture guard: the Cpp filter keeps every unit of the C++-shaped set"
    );
    assert!(
        !state.artifacts_are_authoritative(),
        "an active filter stays non-authoritative even when nothing was narrowed"
    );
    assert_parity(&state, &pipeline);
}

#[test]
fn unregistered_language_errors_identically() {
    let dir = cpp_repo();
    let mirror_error = mirror_construct(dir.path(), Some(SourceLanguage::Other))
        .err()
        .expect("the mirrored registration guard must reject an unregistered language");
    let ctor_error = Pipeline::for_incremental(dir.path(), Some(SourceLanguage::Other))
        .err()
        .expect("for_incremental must reject the unregistered language too")
        .to_string();
    assert_eq!(
        ctor_error, mirror_error,
        "parity surface: unregistered-language error text"
    );
}
