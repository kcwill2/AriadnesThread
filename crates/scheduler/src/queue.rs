use ariadne_build::TranslationUnit;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::path::Path;

use crate::tu_store::SharedTuStoreContext;

#[derive(Debug, Clone)]
pub struct TuTask {
    pub task_id: usize,
    pub tu: TranslationUnit,
    pub estimated_cost: usize,
    /// Shared store context (discovery + contexts) enabling closure-merge
    /// analysis inside workers. `None` keeps body-only analysis.
    pub context: Option<SharedTuStoreContext>,
}

impl PartialEq for TuTask {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
            && self.estimated_cost == other.estimated_cost
            && self.tu == other.tu
    }
}

impl Eq for TuTask {}

impl PartialOrd for TuTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TuTask {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.estimated_cost.cmp(&other.estimated_cost) {
            Ordering::Equal => match other.tu.source_path.cmp(&self.tu.source_path) {
                Ordering::Equal => match other.tu.file_id.cmp(&self.tu.file_id) {
                    Ordering::Equal => other.task_id.cmp(&self.task_id),
                    ord => ord,
                },
                ord => ord,
            },
            ord => ord,
        }
    }
}

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
}
