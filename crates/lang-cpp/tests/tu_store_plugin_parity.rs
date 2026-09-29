//! M18.2 incremental-injection parity (plan §M18, `incremental.rs` row):
//! the registry-driven `tu_store` variants consumed through
//! `IncrementalAnalyzer::with_analyzer`/`with_resolver` must reproduce the
//! transitional hardcoded path — identical closure membership, identical
//! cache fingerprint, and identical merged analysis. Third-party-owned
//! headers are the one documented divergence (`ResolveOutcome::External`
//! drops the boundary file from the plugin closure, plan §3.4); the fixture
//! below uses only repository-owned files.

use ariadne_build::{CompilationContext, CompileCommand, TranslationUnit};
use ariadne_lang_cpp::{analyzer::CppSemanticAnalyzer, CppDependencyResolver};
use ariadne_repo::FileId;
use ariadne_scheduler::{
    analyze_closure, analyze_closure_with, compute_tu_closure, compute_tu_closure_plugin,
    TuStoreContext,
};
use std::path::{Path, PathBuf};

const VERSION: &str = "parity-version";

fn write_file(root: &Path, rel: &str, content: &str) {
    let full = root.join(rel);
    std::fs::create_dir_all(full.parent().expect("parent dir")).unwrap();
    std::fs::write(full, content).unwrap();
}

/// Two-chain closure (`tu1 -> common.h -> base.h`) plus a TU whose single
/// include resolves nowhere, exercising the miss path.
fn sample_repo(root: &Path) {
    write_file(root, "include/base.h", "struct Base { int b; };");
    write_file(
        root,
        "include/common.h",
        "#include \"base.h\"\nstruct Common { Base base; };",
    );
    write_file(
        root,
        "src/tu1.cpp",
        "#include \"../include/common.h\"\nvoid use(Common c) { (void)c; }",
    );
    write_file(
        root,
        "src/tu2.cpp",
        "#include <nowhere_to_be_found.h>\nvoid alone() {}",
    );
}

fn parity_context() -> CompilationContext {
    let mut context = CompilationContext::new("parity", "clang++");
    context.include_dirs = vec![PathBuf::from("include")];
    context
}

fn translation_unit(rel: &str) -> TranslationUnit {
    TranslationUnit {
        file_id: FileId::from_relative_path(rel),
        source_path: PathBuf::from(rel),
        context_name: "parity".to_string(),
        compile_command: CompileCommand {
            directory: PathBuf::new(),
            file: PathBuf::from(rel),
            command: None,
            arguments: vec![],
            context_name: "parity".to_string(),
            output: None,
        },
    }
}

fn given_repo(root: &Path) -> ariadne_repo::RepositoryDiscoveryResult {
    let files = ariadne_repo::discover_repository_files(root).unwrap();
    ariadne_repo::build_discovery_result(root.to_path_buf(), files)
}

fn sorted_files(
    closure: &ariadne_scheduler::TuClosure,
) -> Vec<(FileId, PathBuf, String)> {
    let mut files = closure.files.clone();
    files.sort();
    files
}

#[test]
fn plugin_closure_matches_legacy_closure_and_fingerprint() {
    let repo = tempfile::tempdir().unwrap();
    let root = repo.path();
    sample_repo(root);
    let discovery = given_repo(root);
    let context = parity_context();
    let store_ctx = TuStoreContext::new(&discovery, vec![context.clone()]);
    let plugin_resolver = CppDependencyResolver::new(root)
        .with_repo_discovery(&discovery)
        .with_context(&context);

    for rel in ["src/tu1.cpp", "src/tu2.cpp"] {
        let tu = translation_unit(rel);
        let legacy = compute_tu_closure(&tu, root, &store_ctx, VERSION).unwrap();
        let plugin =
            compute_tu_closure_plugin(&tu, root, &store_ctx, VERSION, &plugin_resolver)
                .unwrap();

        assert_eq!(
            legacy.fingerprint.combined_hash, plugin.fingerprint.combined_hash,
            "cache fingerprint must be identical for {rel}"
        );
        assert_eq!(
            sorted_files(&legacy),
            sorted_files(&plugin),
            "closure file membership and content must be identical for {rel}"
        );
    }
}

#[test]
fn injected_analyzer_merge_matches_hardcoded_merge() {
    let repo = tempfile::tempdir().unwrap();
    let root = repo.path();
    sample_repo(root);
    let discovery = given_repo(root);
    let context = parity_context();
    let store_ctx = TuStoreContext::new(&discovery, vec![context.clone()]);

    let tu = translation_unit("src/tu1.cpp");
    let closure = compute_tu_closure(&tu, root, &store_ctx, VERSION).unwrap();

    let legacy = analyze_closure(&tu, &closure);
    let injected = analyze_closure_with(&tu, &closure, &CppSemanticAnalyzer);
    assert_eq!(
        serde_json::to_string(&legacy).unwrap(),
        serde_json::to_string(&injected).unwrap(),
        "injected `&dyn SemanticAnalyzer` merge must be byte-identical"
    );
}
