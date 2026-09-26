use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use anyhow::{anyhow, Result};
use tokio::sync::Mutex;

use ariadne_index::{CppSemanticAnalyzer, TuAnalysis};
use crate::queue::{PrioritizedTuQueue, TuTask};

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
            let (analysis, _fingerprint) = crate::tu_store::analyze_tu_full(&task.tu, repo_root, ctx)?;
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

        let mut child_res = Err(std::io::Error::new(std::io::ErrorKind::NotFound, "not started"));
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

        if child_res.is_err() {
            // Try cargo run as fallback for dev environments
            child_res = std::process::Command::new("cargo")
                .args(&["run", "-q", "-p", "clang-worker", "--bin", "clang-worker", "--", "--file"])
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
                                    if let Ok(analysis) = serde_json::from_value::<TuAnalysis>(analysis_json.clone()) {
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

    pub async fn run(&self, repo_root: &Path, queue: &mut PrioritizedTuQueue) -> Result<Vec<TuAnalysis>> {
        use tokio::sync::mpsc;

        let num_workers = self.config.num_workers.max(1);
        let queue_mutex = Arc::new(Mutex::new(std::mem::replace(queue, PrioritizedTuQueue::new(vec![]))));
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
                                let err_msg = format!("Task {} failed after {} retries: {}", task.task_id, max_retries, e);
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
                    self.peak_memory_bytes.fetch_max(approx_mem, Ordering::Relaxed);
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
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicUsize;
    use ariadne_build::{CompileCommand, TranslationUnit};
    use ariadne_repo::FileId;

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
            TuTask { task_id: 1, tu: make_tu("a.cpp"), estimated_cost: 10, context: None },
            TuTask { task_id: 2, tu: make_tu("b.cpp"), estimated_cost: 20, context: None },
            TuTask { task_id: 3, tu: make_tu("c.cpp"), estimated_cost: 15, context: None },
        ];
        let mut queue = PrioritizedTuQueue::new(tus);
        let pool = WorkerPool::new(DefaultWorkerBackend, WorkerPoolConfig { num_workers: 2, ..Default::default() });
        let temp_dir = tempfile::tempdir().unwrap();
        let results = pool.run(temp_dir.path(), &mut queue).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_worker_pool_failure_recovery() {
        let tus = vec![
            TuTask { task_id: 1, tu: make_tu("a.cpp"), estimated_cost: 10, context: None },
            TuTask { task_id: 2, tu: make_tu("b.cpp"), estimated_cost: 20, context: None },
        ];
        let mut queue = PrioritizedTuQueue::new(tus);
        let pool = WorkerPool::new(FlakyBackend { attempts: AtomicUsize::new(0) }, WorkerPoolConfig { num_workers: 2, max_retries: 2, ..Default::default() });
        let temp_dir = tempfile::tempdir().unwrap();
        let results = pool.run(temp_dir.path(), &mut queue).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_worker_pool_max_retries_exceeded() {
        let tus = vec![
            TuTask { task_id: 1, tu: make_tu("a.cpp"), estimated_cost: 10, context: None },
        ];
        let mut queue = PrioritizedTuQueue::new(tus);
        let pool = WorkerPool::new(FailingBackend, WorkerPoolConfig { num_workers: 1, max_retries: 2, ..Default::default() });
        let temp_dir = tempfile::tempdir().unwrap();
        let res = pool.run(temp_dir.path(), &mut queue).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_worker_pool_memory_tracking() {
        let tus = vec![
            TuTask { task_id: 1, tu: make_tu("a.cpp"), estimated_cost: 10, context: None },
        ];
        let mut queue = PrioritizedTuQueue::new(tus);
        let pool = WorkerPool::new(DefaultWorkerBackend, WorkerPoolConfig::default());
        let temp_dir = tempfile::tempdir().unwrap();
        let _ = pool.run(temp_dir.path(), &mut queue).await.unwrap();
        assert!(pool.peak_memory_bytes() > 0);
    }
}
