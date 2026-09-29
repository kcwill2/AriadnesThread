//! M18.2 worker-backend parity (plan §6.2/§M18): `PluginWorkerBackend`
//! (registry dispatch) must reproduce the deprecated `DefaultWorkerBackend`
//! byte-for-byte on C++ tasks, both body-only and closure-merged, and must
//! refuse unhandled languages instead of parsing them as C++ (plan §3.3
//! fact 3).

// Compiler-forced transitional allow: every deprecated use below is the
// deliberate legacy-vs-plugin comparison; removed with the shims in M18.11.
#![allow(deprecated)]

use ariadne_build::{CompileCommand, TranslationUnit};
use ariadne_lang::{LanguageId, LanguageRegistry};
use ariadne_lang_cpp::CppPlugin;
use ariadne_repo::FileId;
use ariadne_scheduler::{
    DefaultWorkerBackend, PluginWorkerBackend, SharedTuStoreContext, TuTask, WorkerBackend,
};
use std::path::{Path, PathBuf};

fn cpp_registry() -> LanguageRegistry {
    LanguageRegistry::new().register(Box::new(CppPlugin::new()))
}

fn make_task(task_id: usize, rel: &str, context: Option<SharedTuStoreContext>) -> TuTask {
    TuTask {
        task_id,
        tu: TranslationUnit {
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
        },
        estimated_cost: 10,
        context,
    }
}

fn write_file(root: &Path, rel: &str, content: &str) {
    let full = root.join(rel);
    std::fs::create_dir_all(full.parent().expect("parent dir")).unwrap();
    std::fs::write(full, content).unwrap();
}

fn sample_repo(root: &Path) {
    write_file(
        root,
        "include/common.h",
        "struct Common { int id; };",
    );
    write_file(
        root,
        "src/tu1.cpp",
        "#include \"../include/common.h\"\nvoid use_a(Common c) { (void)c; }",
    );
    write_file(root, "src/bare.cpp", "class Bare { int x; };");
}

fn store_context(root: &Path) -> SharedTuStoreContext {
    let files = ariadne_repo::discover_repository_files(root).unwrap();
    let discovery = ariadne_repo::build_discovery_result(root.to_path_buf(), files);
    ariadne_scheduler::TuStoreContext::new(&discovery, vec![])
}

#[test]
#[allow(deprecated)]
fn body_only_backend_analysis_is_byte_identical_to_legacy() {
    let repo = tempfile::tempdir().unwrap();
    sample_repo(repo.path());
    let registry = cpp_registry();

    for rel in ["src/bare.cpp", "src/tu1.cpp"] {
        let task = make_task(1, rel, None);
        let legacy = DefaultWorkerBackend
            .analyze_task(&task, repo.path())
            .expect("legacy backend succeeds");
        let plugin = PluginWorkerBackend::new(&registry)
            .analyze_task(&task, repo.path())
            .expect("plugin backend succeeds");
        assert_eq!(
            serde_json::to_string(&legacy).unwrap(),
            serde_json::to_string(&plugin).unwrap(),
            "body-only analyses must be byte-identical for {rel}"
        );
        assert_eq!(legacy.language, LanguageId::Cpp);
    }
}

#[test]
#[allow(deprecated)]
fn closure_merged_backend_analysis_is_byte_identical_to_legacy() {
    let repo = tempfile::tempdir().unwrap();
    sample_repo(repo.path());
    let registry = cpp_registry();
    let ctx = store_context(repo.path());

    let task = make_task(2, "src/tu1.cpp", Some(ctx.clone()));
    let legacy = DefaultWorkerBackend
        .analyze_task(&task, repo.path())
        .expect("legacy closure merge succeeds");
    let plugin = PluginWorkerBackend::new(&registry)
        .analyze_task(&task, repo.path())
        .expect("plugin closure merge succeeds");
    assert_eq!(
        serde_json::to_string(&legacy).unwrap(),
        serde_json::to_string(&plugin).unwrap(),
        "closure-merged analyses must be byte-identical"
    );
    assert!(
        legacy.symbols.iter().any(|s| s.name == "Common"),
        "closure merge must still pick up header symbols"
    );
}

#[test]
fn unhandled_language_is_reported_not_parsed_as_cpp() {
    let repo = tempfile::tempdir().unwrap();
    write_file(repo.path(), "notes.txt", "class Fake {};");
    let registry = cpp_registry();

    let task = make_task(3, "notes.txt", None);
    let result = PluginWorkerBackend::new(&registry).analyze_task(&task, repo.path());
    let err = result.expect_err("FileKind::Other must not be analysed");
    assert!(
        err.to_string().contains("unhandled language"),
        "expected unhandled-language error, got: {err}"
    );
}
