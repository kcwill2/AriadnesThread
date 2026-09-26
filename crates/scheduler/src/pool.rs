use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::queue::PrioritizedTuQueue;
use crate::worker::WorkerBackend;
use ariadne_index::TuAnalysis;

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
mod fake_process_tests {
    //! Characterization: WorkerPool::run dispatch semantics driven by fake backends.
    use super::*;
    use crate::queue::TuTask;
    use ariadne_build::{CompileCommand, TranslationUnit};
    use ariadne_repo::FileId;
    use std::sync::atomic::AtomicUsize;

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
    use crate::queue::TuTask;
    use crate::worker::DefaultWorkerBackend;
    use ariadne_build::{CompileCommand, TranslationUnit};
    use ariadne_index::CppSemanticAnalyzer;
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
