use ariadne_build::TranslationUnit;
use std::collections::BinaryHeap;
use std::path::Path;

use crate::tu_store::SharedTuStoreContext;

// AR-20a compat shim: TuTask moved verbatim to `ariadne_worker_core::task`
// (the task unit travels with the engine). Re-exported here so
// `crate::queue::TuTask` and the flat name keep resolving unchanged.
pub use ariadne_worker_core::task::TuTask;

#[derive(Debug, Clone)]
pub struct PrioritizedTuQueue {
    tasks: BinaryHeap<TuTask>,
    total_enqueued: usize,
}

impl PrioritizedTuQueue {
    pub fn new(tasks: Vec<TuTask>) -> Self {
        let total_enqueued = tasks.len();
        Self {
            tasks: BinaryHeap::from(tasks),
            total_enqueued,
        }
    }

    pub fn from_translation_units(repo_root: &Path, tus: Vec<TranslationUnit>) -> Self {
        Self::from_translation_units_with_context(repo_root, tus, None)
    }

    pub fn from_translation_units_with_context(
        repo_root: &Path,
        tus: Vec<TranslationUnit>,
        context: Option<SharedTuStoreContext>,
    ) -> Self {
        let tasks: Vec<TuTask> = tus
            .into_iter()
            .enumerate()
            .map(|(idx, tu)| {
                let cost = estimate_tu_cost(repo_root, &tu);
                TuTask {
                    task_id: idx + 1,
                    estimated_cost: cost,
                    tu,
                    context: context.clone(),
                }
            })
            .collect();
        Self::new(tasks)
    }

    pub fn pop_next(&mut self) -> Option<TuTask> {
        self.tasks.pop()
    }

    pub fn requeue(&mut self, task: TuTask) {
        self.tasks.push(task);
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn total_tasks(&self) -> usize {
        self.total_enqueued
    }
}

pub fn estimate_tu_cost(repo_root: &Path, tu: &TranslationUnit) -> usize {
    let full_path = repo_root.join(&tu.source_path);
    match std::fs::metadata(&full_path) {
        Ok(meta) => {
            let len = meta.len() as usize;
            if len == 0 {
                1
            } else {
                len
            }
        }
        Err(_) => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_build::CompileCommand;
    use ariadne_repo::FileId;
    use std::path::PathBuf;

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
            language: Default::default(),
        }
    }

    #[test]
    fn test_queue_ordering() {
        let tasks = vec![
            TuTask {
                task_id: 1,
                tu: make_tu("b.rs"),
                estimated_cost: 10,
                context: None,
            },
            TuTask {
                task_id: 2,
                tu: make_tu("a.rs"),
                estimated_cost: 20,
                context: None,
            },
            TuTask {
                task_id: 3,
                tu: make_tu("c.rs"),
                estimated_cost: 10,
                context: None,
            },
        ];

        let mut queue = PrioritizedTuQueue::new(tasks);

        let first = queue.pop_next().unwrap();
        assert_eq!(first.task_id, 2);

        let second = queue.pop_next().unwrap();
        assert_eq!(second.task_id, 1);

        let third = queue.pop_next().unwrap();
        assert_eq!(third.task_id, 3);

        assert!(queue.is_empty());
    }

    #[test]
    fn test_requeue() {
        let tasks = vec![
            TuTask {
                task_id: 1,
                tu: make_tu("a.rs"),
                estimated_cost: 10,
                context: None,
            },
            TuTask {
                task_id: 2,
                tu: make_tu("b.rs"),
                estimated_cost: 20,
                context: None,
            },
        ];
        let mut queue = PrioritizedTuQueue::new(tasks);

        let task = queue.pop_next().unwrap();
        queue.requeue(task);

        let first = queue.pop_next().unwrap();
        assert_eq!(first.task_id, 2);
    }

    #[test]
    fn test_estimate_tu_cost() {
        use std::io::Write;
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"1234567890").unwrap();

        let tu = make_tu("test.rs");
        let cost = estimate_tu_cost(temp_dir.path(), &tu);
        assert_eq!(cost, 10);

        let missing_tu = make_tu("missing.rs");
        let cost_missing = estimate_tu_cost(temp_dir.path(), &missing_tu);
        assert_eq!(cost_missing, 1);
    }

    #[test]
    fn test_from_translation_units() {
        let repo_root = PathBuf::from(".");
        let tus = vec![make_tu("a.rs"), make_tu("b.rs")];
        let queue = PrioritizedTuQueue::from_translation_units(&repo_root, tus);
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.total_tasks(), 2);
    }

    // ---- UTA-T5 item 8: cost estimation edges + context propagation --------

    #[test]
    fn estimate_tu_cost_empty_file_floors_at_one_and_directories_use_metadata_size() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();
        std::fs::write(root.join("empty.cpp"), b"").unwrap();
        assert_eq!(estimate_tu_cost(root, &make_tu("empty.cpp")), 1);

        // A directory path takes the SAME metadata branch (Ok) as files:
        // cost is the reported size floored at 1, never the missing-file
        // Err branch. (Windows reports 0 for directories, hence the floor.)
        let directory_cost = estimate_tu_cost(root, &make_tu(""));
        let reported_len = std::fs::metadata(root).unwrap().len() as usize;
        assert_eq!(directory_cost, reported_len.max(1));
    }

    #[test]
    fn from_translation_units_with_context_propagates_measured_costs_and_context() {
        use std::collections::HashMap;

        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();
        std::fs::write(root.join("small.cpp"), b"12345").unwrap();
        std::fs::write(root.join("middle.cpp"), b"1234567890").unwrap();
        std::fs::write(root.join("large.cpp"), vec![b'0'; 30]).unwrap();

        let empty_discovery = ariadne_repo::RepositoryDiscoveryResult {
            root: root.to_path_buf(),
            files: vec![],
            ownership_summary: HashMap::new(),
            policy_summary: HashMap::new(),
        };
        let store_ctx = Some(crate::tu_store::TuStoreContext::new(
            &empty_discovery,
            vec![],
        ));

        let tus = vec![
            make_tu("small.cpp"),
            make_tu("middle.cpp"),
            make_tu("large.cpp"),
        ];
        let mut queue =
            PrioritizedTuQueue::from_translation_units_with_context(root, tus, store_ctx);
        assert_eq!(queue.total_tasks(), 3);

        // Real temp-file sizes drive real priorities: 30 > 10 > 5 pops first.
        let mut costs = Vec::new();
        let mut contexts_attached = Vec::new();
        while let Some(task) = queue.pop_next() {
            costs.push(task.estimated_cost);
            contexts_attached.push(task.context.is_some());
        }
        assert_eq!(costs, vec![30, 10, 5]);
        assert_eq!(contexts_attached, vec![true, true, true]);
    }
}
