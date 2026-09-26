use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::queue::{PrioritizedTuQueue, TuTask};
use ariadne_index::{CppSemanticAnalyzer, TuAnalysis};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerStatus {
    Idle,
    Busy(usize),
    Failed(String),
}

pub trait WorkerBackend: Send + Sync {
    fn execute(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis>;
}

pub struct DefaultWorkerBackend;

impl WorkerBackend for DefaultWorkerBackend {
    fn execute(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        if let Some(ctx) = &task.context {
            // Canonical closure-merge analysis: identical content to the
            // single-threaded index path so all shard store entries agree.
            let (analysis, _fingerprint) =
                crate::tu_store::analyze_tu_full(&task.tu, repo_root, ctx)?;
            return Ok(analysis);
        }
        let full_path = repo_root.join(&task.tu.source_path);
        let source_code = std::fs::read_to_string(&full_path).unwrap_or_default();
        let analysis = CppSemanticAnalyzer::analyze_source(
            task.tu.file_id.clone(),
            &source_code,
            &task.tu.context_name,
        );
        Ok(analysis)
    }
}

pub struct ClangWorkerBackend;

impl WorkerBackend for ClangWorkerBackend {
    fn execute(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        let full_path = repo_root.join(&task.tu.source_path);

        let mut defines = Vec::new();
        let mut include_dirs = Vec::new();
        for arg in &task.tu.compile_command.arguments {
            if arg.starts_with("-D") {
                defines.push(arg[2..].to_string());
            } else if arg.starts_with("-I") {
                include_dirs.push(arg[2..].to_string());
            }
        }

        let req = serde_json::json!({
            "action": "analyze",
            "source_file": full_path.to_string_lossy().to_string(),
            "file_id": task.tu.file_id,
            "context_name": task.tu.context_name,
            "compiler": task.tu.compile_command.command.clone(),
            "defines": defines,
            "include_dirs": include_dirs,
            "language_standard": None::<String>
        });

        // Use `--file` argument to pass a temp file containing the request to avoid stdin issues with `cargo run`
        let temp_dir = std::env::temp_dir();
        let req_path = temp_dir.join(format!("worker_req_{}.json", task.task_id));
        let _ = std::fs::write(&req_path, req.to_string());

        let mut candidate_paths = vec![
            std::path::PathBuf::from("clang-worker"),
            std::path::PathBuf::from("target/debug/clang-worker.exe"),
            std::path::PathBuf::from("target/debug/clang-worker"),
        ];
        if let Ok(cur_exe) = std::env::current_exe() {
            if let Some(target_dir) = cur_exe.parent().and_then(|p| p.parent()) {
                candidate_paths.push(target_dir.join("clang-worker.exe"));
                candidate_paths.push(target_dir.join("clang-worker"));
            }
        }

        // TEST-ONLY SEAM (Pass 1.3 characterization): when a fake worker command is
        // installed in FAKE_WORKER_CMD, it replaces the candidate-binary discovery and
        // the `--file` arg convention, and the `cargo run` fallback is suppressed so
        // tests never compile or execute the real clang-worker. All items in this
        // seam are `#[cfg(test)]`-gated: non-test builds compile exactly as before.
        #[cfg(test)]
        let fake_worker_cmd = fake_worker_cmd_override();

        #[cfg(test)]
        if fake_worker_cmd.is_some() {
            candidate_paths = Vec::new();
        }

        let mut child_res = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not started",
        ));

        #[cfg(test)]
        if let Some((fake_program, fake_args)) = &fake_worker_cmd {
            child_res = std::process::Command::new(fake_program)
                .args(fake_args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();
        }

        for bin in candidate_paths {
            let res = std::process::Command::new(&bin)
                .arg("--file")
                .arg(&req_path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();
            if res.is_ok() {
                child_res = res;
                break;
            }
        }

        #[cfg(not(test))]
        let cargo_fallback_suppressed = false;
        #[cfg(test)]
        let cargo_fallback_suppressed = fake_worker_cmd.is_some();

        if child_res.is_err() && !cargo_fallback_suppressed {
            // Try cargo run as fallback for dev environments
            child_res = std::process::Command::new("cargo")
                .args(&[
                    "run",
                    "-q",
                    "-p",
                    "clang-worker",
                    "--bin",
                    "clang-worker",
                    "--",
                    "--file",
                ])
                .arg(&req_path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();
        }

        let mut success = false;
        let mut analysis_res = None;

        if let Ok(child) = child_res {
            if let Ok(out) = child.wait_with_output() {
                if out.status.success() {
                    let stdout_str = String::from_utf8_lossy(&out.stdout);
                    for line in stdout_str.lines() {
                        if let Ok(resp) = serde_json::from_str::<serde_json::Value>(line) {
                            if resp.get("status").and_then(|s| s.as_str()) == Some("success") {
                                if let Some(analysis_json) = resp.get("analysis") {
                                    if let Ok(analysis) =
                                        serde_json::from_value::<TuAnalysis>(analysis_json.clone())
                                    {
                                        success = true;
                                        analysis_res = Some(analysis);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let _ = std::fs::remove_file(&req_path);

        if success {
            if let Some(analysis) = analysis_res {
                return Ok(analysis);
            }
        }

        // Fallback if execution failed or response invalid
        DefaultWorkerBackend.execute(task, repo_root)
    }
}

#[derive(Debug, Clone)]
pub struct WorkerPoolConfig {
    pub num_workers: usize,
    pub max_retries: usize,
    pub memory_limit_bytes: Option<usize>,
}

impl Default for WorkerPoolConfig {
    fn default() -> Self {
        Self {
            num_workers: 4,
            max_retries: 3,
            memory_limit_bytes: None,
        }
    }
}

pub struct WorkerPool<B: WorkerBackend> {
    backend: Arc<B>,
    config: WorkerPoolConfig,
    peak_memory_bytes: AtomicU64,
}

impl<B: WorkerBackend + 'static> WorkerPool<B> {
    pub fn new(backend: B, config: WorkerPoolConfig) -> Self {
        Self {
            backend: Arc::new(backend),
            config,
            peak_memory_bytes: AtomicU64::new(0),
        }
    }

    pub fn peak_memory_bytes(&self) -> u64 {
        self.peak_memory_bytes.load(Ordering::Relaxed)
    }

    pub async fn run(
        &self,
        repo_root: &Path,
        queue: &mut PrioritizedTuQueue,
    ) -> Result<Vec<TuAnalysis>> {
        use tokio::sync::mpsc;

        let num_workers = self.config.num_workers.max(1);
        let queue_mutex = Arc::new(Mutex::new(std::mem::replace(
            queue,
            PrioritizedTuQueue::new(vec![]),
        )));
        let retries = Arc::new(Mutex::new(HashMap::<usize, usize>::new()));
        let (tx, mut rx) = mpsc::channel(num_workers * 2);

        let total_tasks = {
            let q = queue_mutex.lock().await;
            q.total_tasks()
        };

        if total_tasks == 0 {
            *queue = PrioritizedTuQueue::new(vec![]);
            return Ok(Vec::new());
        }

        // Spawn workers
        let mut handles = Vec::new();
        for _ in 0..num_workers {
            let q = Arc::clone(&queue_mutex);
            let retries_map = Arc::clone(&retries);
            let backend = Arc::clone(&self.backend);
            let root = repo_root.to_path_buf();
            let tx_worker = tx.clone();
            let max_retries = self.config.max_retries;

            let handle = tokio::spawn(async move {
                loop {
                    let task_opt = {
                        let mut locked_q = q.lock().await;
                        locked_q.pop_next()
                    };

                    let task = match task_opt {
                        Some(t) => t,
                        None => break,
                    };

                    let result = backend.execute(&task, &root);
                    match result {
                        Ok(analysis) => {
                            if tx_worker.send(Ok(analysis)).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            let mut locked_retries = retries_map.lock().await;
                            let count = locked_retries.entry(task.task_id).or_insert(0);
                            if *count < max_retries {
                                *count += 1;
                                drop(locked_retries);
                                let mut locked_q = q.lock().await;
                                locked_q.requeue(task);
                            } else {
                                let err_msg = format!(
                                    "Task {} failed after {} retries: {}",
                                    task.task_id, max_retries, e
                                );
                                let _ = tx_worker.send(Err(anyhow!(err_msg))).await;
                                break;
                            }
                        }
                    }
                }
            });
            handles.push(handle);
        }
        drop(tx); // drop main sender so rx terminates when workers are done

        let mut results = Vec::new();
        let mut fatal_error = None;
        let mut approx_mem: u64 = 1024 * 1024; // Base memory estimate: 1 MB

        while let Some(msg) = rx.recv().await {
            match msg {
                Ok(analysis) => {
                    // Estimate memory footprint per analysis
                    let sym_bytes = (analysis.symbols.len() * 128) as u64;
                    let edge_bytes = (analysis.edges.len() * 64) as u64;
                    approx_mem = approx_mem.saturating_add(sym_bytes + edge_bytes + 512);
                    self.peak_memory_bytes
                        .fetch_max(approx_mem, Ordering::Relaxed);
                    results.push(analysis);
                }
                Err(e) => {
                    if fatal_error.is_none() {
                        fatal_error = Some(e);
                    }
                }
            }
        }

        for h in handles {
            let _ = h.await;
        }

        // Restore any remaining queue items
        let remaining_q = queue_mutex.lock().await;
        *queue = (*remaining_q).clone();

        if let Some(err) = fatal_error {
            return Err(err);
        }

        Ok(results)
    }
}

#[cfg(test)]
static FAKE_WORKER_CMD: std::sync::Mutex<Option<(std::path::PathBuf, Vec<std::ffi::OsString>)>> =
    std::sync::Mutex::new(None);

#[cfg(test)]
fn fake_worker_cmd_override() -> Option<(std::path::PathBuf, Vec<std::ffi::OsString>)> {
    FAKE_WORKER_CMD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

#[cfg(test)]
mod fake_process_tests {
    //! Characterization: ClangWorkerBackend::execute driven by a fake worker process
    //! (cmd.exe /c type <response file>) and WorkerPool::run dispatch semantics.
    use super::*;
    use ariadne_build::{CompileCommand, TranslationUnit};
    use ariadne_repo::FileId;
    use std::ffi::OsString;
    use std::sync::atomic::AtomicUsize;
    use std::sync::{Mutex, MutexGuard};

    const WORKER_MALFORMED_RESPONSE: &str = "{{{ this is not valid json";
    const WORKER_ERROR_STATUS_RESPONSE: &str = r#"{"status":"error","message":"worker boom"}"#;
    const WORKER_SUCCESS_WITHOUT_ANALYSIS_RESPONSE: &str = r#"{"status":"success"}"#;
    const PROBE_SOURCE_CONTENT: &str = "class DiskProbeWorker { public: int probe_field; };";

    /// Serializes fake-worker tests (the override is a process-wide static) and owns
    /// the response tempdir; clearing the override on Drop.
    struct FakeWorkerGuard {
        _serialization_lock: MutexGuard<'static, ()>,
        _response_dir: Option<tempfile::TempDir>,
    }

    static FAKE_WORKER_SERIAL: Mutex<()> = Mutex::new(());

    fn clear_fake_worker_cmd() {
        *FAKE_WORKER_CMD
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }

    impl FakeWorkerGuard {
        fn install(program: std::path::PathBuf, args: Vec<OsString>) -> Self {
            let serialization_lock = FAKE_WORKER_SERIAL
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *FAKE_WORKER_CMD
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some((program, args));
            Self {
                _serialization_lock: serialization_lock,
                _response_dir: None,
            }
        }

        fn with_response_dir(mut self, response_dir: tempfile::TempDir) -> Self {
            self._response_dir = Some(response_dir);
            self
        }
    }

    impl Drop for FakeWorkerGuard {
        fn drop(&mut self) {
            clear_fake_worker_cmd();
        }
    }

    fn comspec_program() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var_os("COMSPEC").unwrap_or_else(|| OsString::from("cmd.exe")),
        )
    }

    fn install_fake_worker_printing_file(response_body: &str) -> FakeWorkerGuard {
        let response_dir = tempfile::tempdir().expect("tempdir for fake worker response file");
        let response_path = response_dir.path().join("fake_worker_response.jsonl");
        std::fs::write(&response_path, response_body).expect("write fake worker response file");
        FakeWorkerGuard::install(
            comspec_program(),
            vec![
                OsString::from("/c"),
                OsString::from("type"),
                response_path.as_os_str().to_os_string(),
            ],
        )
        .with_response_dir(response_dir)
    }

    fn install_fake_worker_exiting_with(exit_code: &str) -> FakeWorkerGuard {
        FakeWorkerGuard::install(
            comspec_program(),
            vec![
                OsString::from("/c"),
                OsString::from("exit"),
                OsString::from("/b"),
                OsString::from(exit_code),
            ],
        )
    }

    fn install_fake_worker_missing_binary() -> FakeWorkerGuard {
        FakeWorkerGuard::install(
            std::path::PathBuf::from("definitely-missing-worker-binary-9f3a2e.exe"),
            Vec::new(),
        )
    }

    fn canned_success_response(analysis: &TuAnalysis) -> String {
        format!(
            "{}\n",
            serde_json::json!({
                "status": "success",
                "analysis": serde_json::to_value(analysis).expect("serializable canned analysis"),
            })
        )
    }

    fn probe_task(task_id: usize, source_rel: &str) -> TuTask {
        TuTask {
            task_id,
            tu: TranslationUnit {
                file_id: FileId::from_relative_path(source_rel),
                source_path: std::path::PathBuf::from(source_rel),
                context_name: "fake-probe".to_string(),
                compile_command: CompileCommand {
                    directory: std::path::PathBuf::new(),
                    file: std::path::PathBuf::from(source_rel),
                    command: None,
                    arguments: Vec::new(),
                    context_name: "fake-probe".to_string(),
                    output: None,
                },
            },
            estimated_cost: 10,
            context: None,
        }
    }

    fn repo_with_probe_source(repo_dir: &tempfile::TempDir, source_rel: &str) {
        let full_source = repo_dir.path().join(source_rel);
        std::fs::create_dir_all(full_source.parent().expect("source parent dir"))
            .expect("create probe source dir");
        std::fs::write(&full_source, PROBE_SOURCE_CONTENT).expect("write probe source");
    }

    #[test]
    fn execute_returns_analysis_from_well_formed_worker_success_line() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_ok.cpp");
        let canned_analysis = TuAnalysis {
            tu_file_id: FileId::from_relative_path("probe_ok.cpp"),
            context_name: "canned-context".to_string(),
            ..Default::default()
        };
        let _fake_worker =
            install_fake_worker_printing_file(&canned_success_response(&canned_analysis));

        let analysis = ClangWorkerBackend
            .execute(&probe_task(9001, "probe_ok.cpp"), repo.path())
            .expect("well-formed success line must decode to Ok, not fall back");
        assert_eq!(analysis, canned_analysis);
    }

    #[test]
    fn execute_falls_back_to_default_analysis_on_malformed_json_line() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_malformed.cpp");
        let _fake_worker =
            install_fake_worker_printing_file(&format!("{WORKER_MALFORMED_RESPONSE}\n"));

        let task = probe_task(9002, "probe_malformed.cpp");
        let analysis = ClangWorkerBackend
            .execute(&task, repo.path())
            .expect("malformed worker JSON must degrade to Ok via in-process fallback");
        assert_eq!(
            analysis,
            DefaultWorkerBackend
                .execute(&task, repo.path())
                .expect("default backend fallback must succeed")
        );
    }

    #[test]
    fn execute_falls_back_when_worker_reports_error_status() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_error.cpp");
        let _fake_worker =
            install_fake_worker_printing_file(&format!("{WORKER_ERROR_STATUS_RESPONSE}\n"));

        let task = probe_task(9003, "probe_error.cpp");
        let analysis = ClangWorkerBackend
            .execute(&task, repo.path())
            .expect("error-status worker JSON must degrade to Ok via in-process fallback");
        assert_eq!(
            analysis,
            DefaultWorkerBackend
                .execute(&task, repo.path())
                .expect("default backend fallback must succeed")
        );
    }

    #[test]
    fn execute_falls_back_when_success_line_omits_analysis_field() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_noanalysis.cpp");
        let _fake_worker = install_fake_worker_printing_file(&format!(
            "{WORKER_SUCCESS_WITHOUT_ANALYSIS_RESPONSE}\n"
        ));

        let task = probe_task(9004, "probe_noanalysis.cpp");
        let analysis = ClangWorkerBackend
            .execute(&task, repo.path())
            .expect("success line without analysis must degrade to Ok via fallback");
        assert_eq!(
            analysis,
            DefaultWorkerBackend
                .execute(&task, repo.path())
                .expect("default backend fallback must succeed")
        );
    }

    #[test]
    fn execute_skips_noise_lines_and_uses_later_success_line() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_mixed.cpp");
        let canned_analysis = TuAnalysis {
            tu_file_id: FileId::from_relative_path("probe_mixed.cpp"),
            context_name: "canned-mixed-context".to_string(),
            ..Default::default()
        };
        let noisy_body = format!(
            "{WORKER_MALFORMED_RESPONSE}\n{WORKER_ERROR_STATUS_RESPONSE}\n{}",
            canned_success_response(&canned_analysis)
        );
        let _fake_worker = install_fake_worker_printing_file(&noisy_body);

        let analysis = ClangWorkerBackend
            .execute(&probe_task(9005, "probe_mixed.cpp"), repo.path())
            .expect("success line after noise must decode to Ok, not fall back");
        assert_eq!(analysis, canned_analysis);
    }

    #[test]
    fn execute_falls_back_when_worker_exits_nonzero() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_exit3.cpp");
        let _fake_worker = install_fake_worker_exiting_with("3");

        let task = probe_task(9006, "probe_exit3.cpp");
        let analysis = ClangWorkerBackend
            .execute(&task, repo.path())
            .expect("nonzero worker exit must degrade to Ok via in-process fallback");
        assert_eq!(
            analysis,
            DefaultWorkerBackend
                .execute(&task, repo.path())
                .expect("default backend fallback must succeed")
        );
    }

    #[test]
    fn execute_falls_back_when_worker_binary_cannot_spawn() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_spawnfail.cpp");
        // Seam note: production would additionally try the `cargo run` fallback here;
        // the cfg(test) seam suppresses it so tests never build/run the real worker.
        let _fake_worker = install_fake_worker_missing_binary();

        let task = probe_task(9007, "probe_spawnfail.cpp");
        let analysis = ClangWorkerBackend
            .execute(&task, repo.path())
            .expect("spawn failure must degrade to Ok via in-process fallback, never Err");
        assert_eq!(
            analysis,
            DefaultWorkerBackend
                .execute(&task, repo.path())
                .expect("default backend fallback must succeed")
        );
    }

    #[test]
    fn execute_removes_request_file_after_completion() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_reqcleanup.cpp");
        let _fake_worker = install_fake_worker_exiting_with("0");

        ClangWorkerBackend
            .execute(&probe_task(9008, "probe_reqcleanup.cpp"), repo.path())
            .expect("zero-exit worker with empty stdout must degrade to Ok via fallback");
        let request_path = std::env::temp_dir().join("worker_req_9008.json");
        assert!(
            !request_path.exists(),
            "worker request file must be removed after execute: {request_path:?}"
        );
    }

    struct TagBySourcePathBackend;

    impl WorkerBackend for TagBySourcePathBackend {
        fn execute(&self, task: &TuTask, _repo_root: &Path) -> Result<TuAnalysis> {
            Ok(TuAnalysis {
                tu_file_id: task.tu.file_id,
                context_name: task.tu.source_path.to_string_lossy().to_string(),
                ..Default::default()
            })
        }
    }

    struct SelectivelyFailingBackend {
        fatal_task_id: usize,
        successful_executions: Arc<AtomicUsize>,
    }

    impl WorkerBackend for SelectivelyFailingBackend {
        fn execute(&self, task: &TuTask, _repo_root: &Path) -> Result<TuAnalysis> {
            if task.task_id == self.fatal_task_id {
                return Err(anyhow!(
                    "simulated permanent crash for task {}",
                    task.task_id
                ));
            }
            self.successful_executions.fetch_add(1, Ordering::SeqCst);
            Ok(TuAnalysis {
                tu_file_id: task.tu.file_id,
                context_name: task.tu.context_name.clone(),
                ..Default::default()
            })
        }
    }

    struct CountingBackend {
        total_executions: Arc<AtomicUsize>,
    }

    impl WorkerBackend for CountingBackend {
        fn execute(&self, task: &TuTask, _repo_root: &Path) -> Result<TuAnalysis> {
            self.total_executions.fetch_add(1, Ordering::SeqCst);
            Ok(TuAnalysis {
                tu_file_id: task.tu.file_id,
                context_name: task.tu.context_name.clone(),
                ..Default::default()
            })
        }
    }

    #[tokio::test]
    async fn run_with_single_worker_orders_results_by_priority_pop_order() {
        let tus = vec![
            TuTask {
                task_id: 9101,
                tu: make_tagged_tu("order_high.cpp"),
                estimated_cost: 30,
                context: None,
            },
            TuTask {
                task_id: 9102,
                tu: make_tagged_tu("order_mid.cpp"),
                estimated_cost: 20,
                context: None,
            },
            TuTask {
                task_id: 9103,
                tu: make_tagged_tu("order_low.cpp"),
                estimated_cost: 10,
                context: None,
            },
        ];
        let mut queue = PrioritizedTuQueue::new(tus);
        let pool = WorkerPool::new(
            TagBySourcePathBackend,
            WorkerPoolConfig {
                num_workers: 1,
                max_retries: 0,
                ..Default::default()
            },
        );
        let repo = tempfile::tempdir().expect("repo tempdir");

        let analyses = pool
            .run(repo.path(), &mut queue)
            .await
            .expect("single-worker run must be Ok");
        let arrival_order: Vec<String> = analyses
            .into_iter()
            .map(|analysis| analysis.context_name)
            .collect();
        assert_eq!(
            arrival_order,
            vec!["order_high.cpp", "order_mid.cpp", "order_low.cpp"],
            "single-worker completion order must equal queue pop order (highest cost first)"
        );
    }

    fn make_tagged_tu(source_rel: &str) -> TranslationUnit {
        TranslationUnit {
            file_id: FileId::from_relative_path(source_rel),
            source_path: std::path::PathBuf::from(source_rel),
            context_name: source_rel.to_string(),
            compile_command: CompileCommand {
                directory: std::path::PathBuf::new(),
                file: std::path::PathBuf::from(source_rel),
                command: None,
                arguments: Vec::new(),
                context_name: source_rel.to_string(),
                output: None,
            },
        }
    }

    #[tokio::test]
    async fn run_continues_other_tasks_while_permanent_failure_exhausts_retries() {
        let tus = vec![
            TuTask {
                task_id: 9201,
                tu: make_tagged_tu("fatal.cpp"),
                estimated_cost: 30,
                context: None,
            },
            TuTask {
                task_id: 9202,
                tu: make_tagged_tu("good_one.cpp"),
                estimated_cost: 20,
                context: None,
            },
            TuTask {
                task_id: 9203,
                tu: make_tagged_tu("good_two.cpp"),
                estimated_cost: 10,
                context: None,
            },
        ];
        let mut queue = PrioritizedTuQueue::new(tus);
        let successful_executions = Arc::new(AtomicUsize::new(0));
        let backend = SelectivelyFailingBackend {
            fatal_task_id: 9201,
            successful_executions: Arc::clone(&successful_executions),
        };
        let pool = WorkerPool::new(
            backend,
            WorkerPoolConfig {
                num_workers: 2,
                max_retries: 1,
                ..Default::default()
            },
        );
        let repo = tempfile::tempdir().expect("repo tempdir");

        let outcome = pool.run(repo.path(), &mut queue).await;
        let err = outcome.expect_err("exhausted retries must surface a fatal pool error");
        let err_text = format!("{err:#}");
        assert!(
            err_text.contains("Task 9201"),
            "fatal error must name the task: {err_text}"
        );
        assert!(
            err_text.contains("failed after 1 retries"),
            "fatal error must state retry budget: {err_text}"
        );
        assert_eq!(
            successful_executions.load(Ordering::SeqCst),
            2,
            "other tasks must keep executing to completion while the failing task exhausts retries"
        );
        assert!(
            queue.is_empty(),
            "all tasks must be consumed from the queue before run returns"
        );
    }

    #[tokio::test]
    async fn run_dispatches_each_task_exactly_once_across_multiple_workers() {
        let tus: Vec<TuTask> = (1..=5usize)
            .map(|idx| TuTask {
                task_id: 9300 + idx,
                tu: make_tagged_tu(&format!("dispatch_{}.cpp", idx)),
                estimated_cost: idx * 10,
                context: None,
            })
            .collect();
        let mut queue = PrioritizedTuQueue::new(tus);
        let total_executions = Arc::new(AtomicUsize::new(0));
        let backend = CountingBackend {
            total_executions: Arc::clone(&total_executions),
        };
        let pool = WorkerPool::new(
            backend,
            WorkerPoolConfig {
                num_workers: 3,
                max_retries: 2,
                ..Default::default()
            },
        );
        let repo = tempfile::tempdir().expect("repo tempdir");

        let analyses = pool
            .run(repo.path(), &mut queue)
            .await
            .expect("clean multi-worker run must be Ok");
        assert_eq!(
            analyses.len(),
            5,
            "every queued task must yield one analysis"
        );
        assert_eq!(
            total_executions.load(Ordering::SeqCst),
            5,
            "each task must execute exactly once when no backend failure occurs"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_build::{CompileCommand, TranslationUnit};
    use ariadne_repo::FileId;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicUsize;

    fn make_tu(path: &str) -> TranslationUnit {
        TranslationUnit {
            file_id: FileId::from_relative_path(path),
            source_path: PathBuf::from(path),
            context_name: "test".to_string(),
            compile_command: CompileCommand {
                directory: PathBuf::new(),
                file: PathBuf::from(path),
                command: None,
                arguments: Vec::new(),
                context_name: "test".to_string(),
                output: None,
            },
        }
    }

    struct FlakyBackend {
        attempts: AtomicUsize,
    }

    impl WorkerBackend for FlakyBackend {
        fn execute(&self, task: &TuTask, _repo_root: &Path) -> Result<TuAnalysis> {
            let attempt = self.attempts.fetch_add(1, Ordering::SeqCst);
            if task.task_id == 1 && attempt == 0 {
                return Err(anyhow!("Transient worker failure"));
            }
            Ok(CppSemanticAnalyzer::analyze_source(
                task.tu.file_id.clone(),
                "class Foo {};",
                &task.tu.context_name,
            ))
        }
    }

    struct FailingBackend;

    impl WorkerBackend for FailingBackend {
        fn execute(&self, _task: &TuTask, _repo_root: &Path) -> Result<TuAnalysis> {
            Err(anyhow!("Permanent worker crash"))
        }
    }

    #[tokio::test]
    async fn test_worker_pool_success() {
        let tus = vec![
            TuTask {
                task_id: 1,
                tu: make_tu("a.cpp"),
                estimated_cost: 10,
                context: None,
            },
            TuTask {
                task_id: 2,
                tu: make_tu("b.cpp"),
                estimated_cost: 20,
                context: None,
            },
            TuTask {
                task_id: 3,
                tu: make_tu("c.cpp"),
                estimated_cost: 15,
                context: None,
            },
        ];
        let mut queue = PrioritizedTuQueue::new(tus);
        let pool = WorkerPool::new(
            DefaultWorkerBackend,
            WorkerPoolConfig {
                num_workers: 2,
                ..Default::default()
            },
        );
        let temp_dir = tempfile::tempdir().unwrap();
        let results = pool.run(temp_dir.path(), &mut queue).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_worker_pool_failure_recovery() {
        let tus = vec![
            TuTask {
                task_id: 1,
                tu: make_tu("a.cpp"),
                estimated_cost: 10,
                context: None,
            },
            TuTask {
                task_id: 2,
                tu: make_tu("b.cpp"),
                estimated_cost: 20,
                context: None,
            },
        ];
        let mut queue = PrioritizedTuQueue::new(tus);
        let pool = WorkerPool::new(
            FlakyBackend {
                attempts: AtomicUsize::new(0),
            },
            WorkerPoolConfig {
                num_workers: 2,
                max_retries: 2,
                ..Default::default()
            },
        );
        let temp_dir = tempfile::tempdir().unwrap();
        let results = pool.run(temp_dir.path(), &mut queue).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_worker_pool_max_retries_exceeded() {
        let tus = vec![TuTask {
            task_id: 1,
            tu: make_tu("a.cpp"),
            estimated_cost: 10,
            context: None,
        }];
        let mut queue = PrioritizedTuQueue::new(tus);
        let pool = WorkerPool::new(
            FailingBackend,
            WorkerPoolConfig {
                num_workers: 1,
                max_retries: 2,
                ..Default::default()
            },
        );
        let temp_dir = tempfile::tempdir().unwrap();
        let res = pool.run(temp_dir.path(), &mut queue).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_worker_pool_memory_tracking() {
        let tus = vec![TuTask {
            task_id: 1,
            tu: make_tu("a.cpp"),
            estimated_cost: 10,
            context: None,
        }];
        let mut queue = PrioritizedTuQueue::new(tus);
        let pool = WorkerPool::new(DefaultWorkerBackend, WorkerPoolConfig::default());
        let temp_dir = tempfile::tempdir().unwrap();
        let _ = pool.run(temp_dir.path(), &mut queue).await.unwrap();
        assert!(pool.peak_memory_bytes() > 0);
    }
}
