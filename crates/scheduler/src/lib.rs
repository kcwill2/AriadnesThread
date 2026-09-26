//! Task scheduler for parallel analysis and indexing.

pub mod incremental;
pub use incremental::*;

pub mod queue;
pub use queue::*;

pub mod worker;
pub use worker::*;

pub mod shard;
pub use shard::*;

pub mod tu_store;
pub use tu_store::*;

pub mod engine;
pub use engine::*;

pub struct TaskScheduler {
    pub concurrency: usize,
}

impl TaskScheduler {
    pub fn new(concurrency: usize) -> Self {
        Self { concurrency }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_init() {
        let sched = TaskScheduler::new(4);
        assert_eq!(sched.concurrency, 4);
    }
}
