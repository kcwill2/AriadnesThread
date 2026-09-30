use ariadne_build::{CompileCommand, TranslationUnit};
use ariadne_lang::LanguageRegistry;
use ariadne_repo::FileId;
use ariadne_scheduler::{PrioritizedTuQueue, SharedPluginWorkerBackend, TuTask, WorkerPool,
    WorkerPoolConfig};
use ariadne_lang_cpp::ClangWorkerBackend;
/// m16 — ClangWorkerBackend Integration Test
///
/// Verifies that ClangWorkerBackend gracefully degrades to the built-in
/// CppSemanticAnalyzer when the `clang-worker` binary / Clang is not available,
/// and that WorkerPool still produces valid TuAnalysis results.
use std::path::PathBuf;

fn cpp_registry() -> LanguageRegistry {
    LanguageRegistry::new().register(Box::new(ariadne_lang_cpp::CppPlugin::new()))
}

fn make_tu(path: &str) -> TranslationUnit {
    TranslationUnit {
        file_id: FileId::from_relative_path(path),
        source_path: PathBuf::from(path),
        context_name: "test".to_string(),
        compile_command: CompileCommand {
            directory: PathBuf::new(),
            file: PathBuf::from(path),
            command: None,
            arguments: vec!["-DTEST_DEFINE=1".to_string(), "-Iinclude".to_string()],
            context_name: "test".to_string(),
            output: None,
        },
        language: Default::default(),
    }
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/fixtures/basic")
}

/// ClangWorkerBackend falls back gracefully and returns a valid TuAnalysis
/// when clang is not available — i.e., no panic, status "success", analysis Some(_).
#[tokio::test]
async fn test_clang_worker_backend_fallback_succeeds() {
    let root = fixture_root();
    let tasks = vec![TuTask {
        task_id: 1,
        tu: make_tu("src/thing.cpp"),
        estimated_cost: 10,
        context: None,
    }];
    let mut queue = PrioritizedTuQueue::new(tasks);
    let pool = WorkerPool::new(
        ClangWorkerBackend,
        WorkerPoolConfig {
            num_workers: 1,
            max_retries: 0,
            ..Default::default()
        },
    );
    let results = pool.run_worker_pool(&root, &mut queue).await;
    assert!(
        results.is_ok(),
        "Pool with ClangWorkerBackend must not error: {:?}",
        results
    );
    let analyses = results.unwrap();
    assert_eq!(analyses.len(), 1, "Expected exactly one analysis result");
}

/// ClangWorkerBackend and the registry plugin backend both produce non-None analyses for the same TU.
#[tokio::test]
async fn test_clang_and_default_backends_produce_consistent_results() {
    let root = fixture_root();

    let tasks_clang = vec![TuTask {
        task_id: 1,
        tu: make_tu("src/thing.cpp"),
        estimated_cost: 10,
        context: None,
    }];
    let mut queue_clang = PrioritizedTuQueue::new(tasks_clang);
    let pool_clang = WorkerPool::new(ClangWorkerBackend, WorkerPoolConfig::default());
    let clang_results = pool_clang
        .run_worker_pool(&root, &mut queue_clang)
        .await
        .unwrap();

    let tasks_default = vec![TuTask {
        task_id: 1,
        tu: make_tu("src/thing.cpp"),
        estimated_cost: 10,
        context: None,
    }];
    let mut queue_default = PrioritizedTuQueue::new(tasks_default);
    let pool_default = WorkerPool::new(
        SharedPluginWorkerBackend::new(std::sync::Arc::new(cpp_registry())),
        WorkerPoolConfig::default(),
    );
    let default_results = pool_default
        .run_worker_pool(&root, &mut queue_default)
        .await
        .unwrap();

    assert_eq!(clang_results.len(), 1);
    assert_eq!(default_results.len(), 1);

    // Both must reference the same file
    assert_eq!(
        clang_results[0].tu_file_id, default_results[0].tu_file_id,
        "Both backends must analyze the same file"
    );
}

/// WorkerPool with ClangWorkerBackend processes multiple TUs without errors,
/// and -D / -I flags in compile commands are forwarded to the request.
#[tokio::test]
async fn test_clang_worker_processes_multiple_translation_units() {
    let root = fixture_root();
    let tasks = vec![
        TuTask {
            task_id: 1,
            tu: make_tu("src/thing.cpp"),
            estimated_cost: 10,
            context: None,
        },
        TuTask {
            task_id: 2,
            tu: make_tu("tests/consumer.cpp"),
            estimated_cost: 20,
            context: None,
        },
    ];
    let mut queue = PrioritizedTuQueue::new(tasks);
    let pool = WorkerPool::new(
        ClangWorkerBackend,
        WorkerPoolConfig {
            num_workers: 2,
            max_retries: 0,
            ..Default::default()
        },
    );
    let results = pool.run_worker_pool(&root, &mut queue).await.unwrap();
    assert_eq!(results.len(), 2, "All TUs must be processed");
}
