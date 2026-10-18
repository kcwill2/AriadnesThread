//! The prioritized-queue task unit.
//!
//! `TuTask` moved verbatim from `ariadne-scheduler::queue` in AR-20a so
//! worker implementors (plugin crates) can name the task type without a
//! scheduler edge. The queue machinery itself (`PrioritizedTuQueue`,
//! `estimate_tu_cost`) stays in the scheduler.

use ariadne_build::TranslationUnit;
use std::cmp::Ordering;

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

#[cfg(test)]
mod ordering_contract_tests {
    //! UTA-T5 item 5: locks the exact TuTask ordering contract the priority
    //! queue depends on: cost descending, then source_path ascending, then
    //! file_id ascending, then task_id ascending, plus the PartialEq field
    //! set and the estimated_cost=0 boundary.
    use super::*;
    use ariadne_build::CompileCommand;
    use ariadne_repo::FileId;
    use std::collections::BinaryHeap;
    use std::path::PathBuf;

    fn task_with(
        task_id: usize,
        rel: &str,
        estimated_cost: usize,
        file_id_override: Option<u64>,
    ) -> TuTask {
        let file_id = file_id_override
            .map(FileId)
            .unwrap_or_else(|| FileId::from_relative_path(rel));
        TuTask {
            task_id,
            tu: TranslationUnit {
                file_id,
                source_path: PathBuf::from(rel),
                context_name: "ordering-pin".into(),
                compile_command: CompileCommand {
                    directory: PathBuf::new(),
                    file: PathBuf::from(rel),
                    command: None,
                    arguments: vec![],
                    context_name: "ordering-pin".into(),
                    output: None,
                },
                language: Default::default(),
            },
            estimated_cost,
            context: None,
        }
    }

    #[test]
    fn equal_cost_tasks_descend_through_source_path_file_id_then_task_id() {
        // First tie-break: lower source_path ranks higher (pops first).
        assert_eq!(
            task_with(1, "a.cpp", 5, None).cmp(&task_with(2, "b.cpp", 5, None)),
            Ordering::Greater
        );
        // Second tie-break (same source_path): lower file_id ranks higher.
        assert_eq!(
            task_with(3, "x.cpp", 5, Some(10)).cmp(&task_with(4, "x.cpp", 5, Some(20))),
            Ordering::Greater
        );
        // Last tie-break (same path AND file_id): lower task_id ranks higher.
        assert_eq!(
            task_with(5, "y.cpp", 5, Some(99)).cmp(&task_with(6, "y.cpp", 5, Some(99))),
            Ordering::Greater
        );
    }

    #[test]
    fn partial_eq_covers_task_id_cost_and_whole_tu() {
        let left = task_with(1, "same.cpp", 5, Some(42));
        let same = task_with(1, "same.cpp", 5, Some(42));
        assert_eq!(left, same);

        // Same task_id + cost, different `tu` content: must be unequal
        // (PartialEq includes `tu`, locking that field into the contract).
        let different_path = task_with(1, "other.cpp", 5, Some(42));
        assert_ne!(left, different_path);
        let different_file_id = task_with(1, "same.cpp", 5, Some(43));
        assert_ne!(left, different_file_id);

        // Other fields are also load-bearing.
        assert_ne!(left, task_with(2, "same.cpp", 5, Some(42)));
        assert_ne!(left, task_with(1, "same.cpp", 6, Some(42)));
    }

    #[test]
    fn estimated_cost_zero_is_the_floor_of_the_priority_axis() {
        // cost 1 outranks cost 0 ...
        assert_eq!(
            task_with(1, "a.cpp", 1, None).cmp(&task_with(2, "a.cpp", 0, None)),
            Ordering::Greater
        );
        // ... while two zero-cost tasks still fall through to path descent.
        assert_eq!(
            task_with(1, "a.cpp", 0, None).cmp(&task_with(2, "b.cpp", 0, None)),
            Ordering::Greater
        );
        assert_eq!(
            task_with(1, "a.cpp", 0, None).cmp(&task_with(1, "a.cpp", 0, None)),
            Ordering::Equal
        );
    }

    #[test]
    fn heap_pop_order_is_input_permutation_invariant() {
        // Expected pop order under the locked contract: cost descending;
        // within cost 100, path "h1.cpp" before "h2.cpp"; within cost 50,
        // file_id 1 before file_id 2.
        let expected_ids: Vec<usize> = vec![7101, 7102, 7103, 7104, 7105];
        let tasks = vec![
            task_with(7101, "h1.cpp", 100, None),
            task_with(7102, "h2.cpp", 100, None),
            task_with(7103, "same50.cpp", 50, Some(1)),
            task_with(7104, "same50.cpp", 50, Some(2)),
            task_with(7105, "low.cpp", 1, None),
        ];

        // Every rotation plus the reversal must yield the same pop order.
        let mut rotations: Vec<Vec<TuTask>> = Vec::new();
        for rotation in 0..tasks.len() {
            let mut rotated: Vec<TuTask> = Vec::new();
            for offset in 0..tasks.len() {
                rotated.push(tasks[(rotation + offset) % tasks.len()].clone());
            }
            rotations.push(rotated);
        }
        let mut reversed = tasks.clone();
        reversed.reverse();
        rotations.push(reversed);

        for order in rotations {
            let mut heap: BinaryHeap<TuTask> = BinaryHeap::from(order);
            let mut popped: Vec<usize> = Vec::new();
            while let Some(task) = heap.pop() {
                popped.push(task.task_id);
            }
            assert_eq!(popped, expected_ids);
        }
    }
}
