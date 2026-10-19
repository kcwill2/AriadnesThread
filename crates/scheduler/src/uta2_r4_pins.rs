//! UTA2-R4 characterization pins for the `incremental` IO failure paths
//! (docs/specs/unit-test-audit.md Round 2, handoff item 2): `save_artifacts`
//! must surface a stage-write IO failure as an Err WITHOUT writing any
//! manifest, and `analyze` must surface a mid-loop shard-save failure as an
//! Err in a PARTIAL-PERSISTENCE state (earlier shards already on disk, no
//! rollback). Child module of `incremental` (the round-1 pin-file model), so
//! the fixtures can build results through the private `store_ctx`-carrying
//! constructors like the inline suites do.

use super::*;
use ariadne_repo::{
    FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, Repository,
    RepositoryDiscoveryResult,
};
use std::collections::HashMap;
use tempfile::TempDir;

fn r4_record(root: &std::path::Path, rel: &str, kind: FileKind, content_hash: &str) -> FileRecord {
    FileRecord {
        id: FileId::from_relative_path(rel),
        path: root.join(rel),
        canonical_path: root.join(rel),
        kind,
        size: 10,
        content_hash: content_hash.to_string(),
        ownership: FileOwnership::Repository,
        optimization_policy: OptimizationPolicy::Optimizable,
        language: ariadne_repo::SourceLanguage::Cpp,
    }
}

fn r4_unit(root: &std::path::Path, rel: &str, context_name: &str) -> ariadne_build::TranslationUnit {
    ariadne_build::TranslationUnit {
        file_id: FileId::from_relative_path(rel),
        source_path: root.join(rel),
        context_name: context_name.to_string(),
        compile_command: ariadne_build::CompileCommand {
            directory: root.to_path_buf(),
            file: root.join(rel),
            command: Some(format!("clang++ -c {}", rel)),
            arguments: vec!["clang++".to_string(), "-c".to_string(), rel.to_string()],
            context_name: context_name.to_string(),
            output: None,
        },
        language: Default::default(),
    }
}

/// The inline-suite fixture shape (files on disk, hand-built discovery, two
/// TUs in [main.cpp, utils.cpp] order), kept local so the round-1 inline
/// suites stay byte-untouched.
fn r4_setup() -> (
    Repository,
    RepositoryDiscoveryResult,
    ariadne_build::BuildDiscoveryResult,
    TempDir,
) {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().canonicalize().unwrap();
    std::fs::write(
        root.join("main.cpp"),
        "#include \"header.h\"\nint main() { return 0; }",
    )
    .unwrap();
    std::fs::write(root.join("header.h"), "void foo();").unwrap();
    std::fs::write(root.join("utils.cpp"), "void bar() {}").unwrap();

    let repo = Repository { root: root.clone() };
    let repo_discovery = RepositoryDiscoveryResult {
        root: root.clone(),
        files: vec![
            r4_record(&root, "main.cpp", FileKind::Source, "hash_main1"),
            r4_record(&root, "header.h", FileKind::Header, "hash_header1"),
            r4_record(&root, "utils.cpp", FileKind::Source, "hash_utils1"),
        ],
        ownership_summary: HashMap::new(),
        policy_summary: HashMap::new(),
    };
    let context = ariadne_build::CompilationContext::new("test_ctx", "clang++");
    let build_result = ariadne_build::BuildDiscoveryResult {
        repo_root: root.clone(),
        contexts: vec![context.clone()],
        translation_units: vec![
            r4_unit(&root, "main.cpp", &context.name),
            r4_unit(&root, "utils.cpp", &context.name),
        ],
    };
    (repo, repo_discovery, build_result, temp_dir)
}

#[test]
fn pin_save_artifacts_surfaces_stage_io_failure_with_no_manifest_written() {
    // A plain file planted on the FIRST stage's directory slot fails its
    // create_dir_all (the t5 `save_shard_surfaces_create_dir_all_failure_
    // as_error` injection class - deterministic on this host, where
    // read-only dir attributes are advisory for our own writes). The Err
    // must surface and NO stage manifest may be written: the failing first
    // stage gets none, and the later stages are never attempted.
    let (repo, repo_discovery, build_result, _temp) = r4_setup();
    let plugin = ariadne_lang_cpp::CppPlugin::new();
    let mut analyzer =
        IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result, &plugin);
    let result = analyzer.analyze().expect("cold analyze must succeed");

    let blocked_slot = repo.root.join("artifacts").join("repository");
    std::fs::write(&blocked_slot, b"not-a-directory").unwrap();

    let err = result
        .save_artifacts(&repo)
        .expect_err("an uncreatable stage dir must surface as an Err");
    let chain = format!("{err:#}");
    assert!(
        chain.contains("Failed to create repository artifact directory"),
        "the error must name the failing stage's directory creation: {chain}"
    );
    assert!(
        blocked_slot.is_file(),
        "the blocker file must be untouched by the failed stage"
    );
    for manifest in [
        blocked_slot.join("manifest.json"),
        repo.root.join("artifacts").join("index").join("manifest.json"),
        repo.root.join("artifacts").join("reduced").join("manifest.json"),
    ] {
        assert!(
            !manifest.exists(),
            "no manifest may exist at {:?} when the stage failed to start",
            manifest
        );
    }
}

#[test]
fn pin_analyze_surfaces_shard_save_failure_keeping_earlier_shards() {
    // Partial-save state (handoff item 2): the SECOND TU's shard final path
    // is planted as a DIRECTORY, so the first shard saves cleanly, the
    // second's staging rename fails on every platform (rename over a
    // directory is rejected by both), and analyze surfaces the Err before
    // any stats/index work. No rollback exists: the first shard stays on
    // disk, and the failed save's staging tmp is left behind.
    let (repo, repo_discovery, build_result, _temp) = r4_setup();
    let store = shard_dir(&repo.root);
    std::fs::create_dir_all(&store).unwrap();
    let second_id = build_result.translation_units[1].file_id;
    let first_id = build_result.translation_units[0].file_id;
    std::fs::create_dir(store.join(format!("{}.json", second_id.0))).unwrap();

    let plugin = ariadne_lang_cpp::CppPlugin::new();
    let mut analyzer =
        IncrementalAnalyzer::new(&repo, &repo_discovery, &build_result, &plugin);
    match analyzer.analyze() {
        Ok(_) => panic!("a shard save that cannot rename must surface an Err"),
        Err(_) => {}
    }

    let persisted = load_shard(&store, first_id)
        .expect("pin: the earlier shard must survive the later save failure");
    assert_eq!(persisted.analysis.tu_file_id, first_id);
    assert!(
        !persisted.fingerprint.is_empty(),
        "the surviving shard is the complete envelope, not a partial write"
    );
    assert!(
        store.join(format!("{}.json", second_id.0)).is_dir(),
        "pin: the failed rename leaves the planted target untouched"
    );

    let staging_residue: Vec<String> = std::fs::read_dir(&store)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp"))
        .collect();
    assert_eq!(
        staging_residue.len(),
        1,
        "pin: the failed save's staged tmp is left behind (no cleanup, no \
         rollback): {staging_residue:?}"
    );
}
