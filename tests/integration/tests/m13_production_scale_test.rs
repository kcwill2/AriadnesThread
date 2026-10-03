use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

use ariadne_build::TranslationUnit;
use ariadne_cli::run_production_scale_analysis;
use ariadne_lang::LanguageRegistry;
use ariadne_repo::FileId;
use ariadne_scheduler::{
    PrioritizedTuQueue, SharedPluginWorkerBackend, TuTask, WorkerBackend, WorkerPool,
    WorkerPoolConfig,
};

fn setup_production_fixture() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();

    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();

    fs::write(root.join("include/common.h"), "struct Common { int id; };").unwrap();
    fs::write(
        root.join("include/header_a.h"),
        "#include \"common.h\"\nstruct Alpha { Common c; };",
    )
    .unwrap();
    fs::write(
        root.join("include/header_b.h"),
        "struct Beta { double val; };",
    )
    .unwrap();

    fs::write(
        root.join("src/tu1.cpp"),
        "#include \"header_a.h\"\nvoid func_a(Alpha a) {}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/tu2.cpp"),
        "#include \"header_b.h\"\nvoid func_b(Beta b) {}\n",
    )
    .unwrap();
    fs::write(
        root.join("src/tu3.cpp"),
        "#include \"common.h\"\nvoid func_c(Common c) {}\n",
    )
    .unwrap();

    let root_str = root.to_string_lossy().replace("\\", "/");
    let compile_commands = format!(
        r#"[
    {{
        "directory": "{root_str}",
        "command": "clang++ -Iinclude -c src/tu1.cpp",
        "file": "src/tu1.cpp"
    }},
    {{
        "directory": "{root_str}",
        "command": "clang++ -Iinclude -c src/tu2.cpp",
        "file": "src/tu2.cpp"
    }},
    {{
        "directory": "{root_str}",
        "command": "clang++ -Iinclude -c src/tu3.cpp",
        "file": "src/tu3.cpp"
    }}
]"#,
        root_str = root_str
    );
    fs::write(root.join("compile_commands.json"), compile_commands).unwrap();

    temp_dir
}

#[test]
fn test_m13_determinism_workers_comparison() {
    let fixture1 = setup_production_fixture();
    let root1 = fixture1.path().to_path_buf();

    let fixture2 = setup_production_fixture();
    let root2 = fixture2.path().to_path_buf();

    // Run with 2 workers
    let res_workers_2 =
        run_production_scale_analysis(&root1, Some(2)).expect("Analyze with 2 workers");

    // Run with 8 workers
    let res_workers_8 =
        run_production_scale_analysis(&root2, Some(8)).expect("Analyze with 8 workers");

    // Assert determinism across different worker counts
    assert_eq!(
        res_workers_2.reduced_model.symbols, res_workers_8.reduced_model.symbols,
        "Symbols must be identical across worker counts"
    );
    assert_eq!(
        res_workers_2.reduced_model.edges, res_workers_8.reduced_model.edges,
        "Edges must be identical across worker counts"
    );
    assert_eq!(
        res_workers_2.reduced_model.tu_usage, res_workers_8.reduced_model.tu_usage,
        "TU usage must be identical across worker counts"
    );
    assert_eq!(
        res_workers_2.reduced_model.file_symbols, res_workers_8.reduced_model.file_symbols,
        "File symbols must be identical across worker counts"
    );
}

#[tokio::test]
async fn test_m13_failure_recovery() {
    struct FaultyWorkerBackend {
        attempts: Arc<AtomicUsize>,
        fallback: SharedPluginWorkerBackend,
    }

    impl WorkerBackend for FaultyWorkerBackend {
        fn analyze_task(
            &self,
            task: &TuTask,
            repo_root: &std::path::Path,
        ) -> Result<ariadne_index::TuAnalysis> {
            let attempt = self.attempts.fetch_add(1, Ordering::SeqCst);
            // Simulate crash/failure on first attempt of task 1
            if task.task_id == 1 && attempt == 0 {
                return Err(anyhow!("Simulated worker crash"));
            }
            self.fallback.analyze_task(task, repo_root)
        }
    }

    let fixture = setup_production_fixture();
    let root = fixture.path().to_path_buf();

    let t1 = TuTask {
        task_id: 1,
        tu: TranslationUnit {
            file_id: FileId::from_relative_path("src/tu1.cpp"),
            source_path: PathBuf::from("src/tu1.cpp"),
            context_name: "default".into(),
            compile_command: ariadne_build::CompileCommand {
                directory: root.clone(),
                file: PathBuf::from("src/tu1.cpp"),
                command: None,
                arguments: vec![],
                context_name: "default".into(),
                output: None,
            },
            language: Default::default(),
        },
        estimated_cost: 100,
        context: None,
    };

    let mut queue = PrioritizedTuQueue::new(vec![t1]);
    let cpp_registry = Arc::new(
        LanguageRegistry::new().register(Box::new(ariadne_lang_cpp::CppPlugin::new())),
    );
    let backend = FaultyWorkerBackend {
        attempts: Arc::new(AtomicUsize::new(0)),
        fallback: SharedPluginWorkerBackend::new(cpp_registry),
    };
    let pool = WorkerPool::new(
        backend,
        WorkerPoolConfig {
            num_workers: 2,
            max_retries: 2,
            memory_limit_bytes: None,
        },
    );

    let results = pool
        .run_worker_pool(&root, &mut queue)
        .await
        .expect("Pool should recover from failure");
    assert_eq!(
        results.len(),
        1,
        "Failed task should have been retried and completed"
    );
}

#[test]
fn test_m13_straggler_scheduling() {
    let t_light = TuTask {
        task_id: 1,
        tu: TranslationUnit {
            file_id: FileId::from_relative_path("light.cpp"),
            source_path: PathBuf::from("light.cpp"),
            context_name: "default".into(),
            compile_command: ariadne_build::CompileCommand {
                directory: PathBuf::new(),
                file: PathBuf::from("light.cpp"),
                command: None,
                arguments: vec![],
                context_name: "default".into(),
                output: None,
            },
            language: Default::default(),
        },
        estimated_cost: 50,
        context: None,
    };

    let t_heavy = TuTask {
        task_id: 2,
        tu: TranslationUnit {
            file_id: FileId::from_relative_path("heavy.cpp"),
            source_path: PathBuf::from("heavy.cpp"),
            context_name: "default".into(),
            compile_command: ariadne_build::CompileCommand {
                directory: PathBuf::new(),
                file: PathBuf::from("heavy.cpp"),
                command: None,
                arguments: vec![],
                context_name: "default".into(),
                output: None,
            },
            language: Default::default(),
        },
        estimated_cost: 10000,
        context: None,
    };

    let mut queue = PrioritizedTuQueue::new(vec![t_light, t_heavy]);
    let first = queue.pop_next().expect("Queue should not be empty");
    assert_eq!(
        first.task_id, 2,
        "Heaviest TU (straggler candidate) must be popped first (LPT heuristic)"
    );
}

#[test]
fn test_m13_memory_bounding_and_cli_output() {
    let fixture = setup_production_fixture();
    let root = fixture.path().to_path_buf();

    let result =
        run_production_scale_analysis(&root, Some(4)).expect("Production analyze should succeed");

    assert!(
        result.stats.peak_memory_bytes > 0,
        "Peak memory must be tracked"
    );
    let output = result.stats.format_m13_output();

    assert!(output.contains("Translation units: 3"));
    assert!(output.contains("Workers:\n    4"));
    assert!(output.contains("Progress:\n    3 / 3"));
    assert!(output.contains("Semantic edges:"));
    assert!(output.contains("Peak memory:"));
    assert!(output.contains("Elapsed:"));
}
