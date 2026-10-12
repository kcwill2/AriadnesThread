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
