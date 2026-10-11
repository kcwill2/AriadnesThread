//! Task scheduler for parallel analysis and indexing.

pub mod incremental;
pub use incremental::*;

pub mod queue;
pub use queue::*;

pub mod worker;
pub use worker::*;

pub mod pool;
pub use pool::*;

pub mod shard;
pub use shard::*;

// AR-20a compat shim: the engine lives in `ariadne-worker-core` (neutral
// crate), every name stays reachable here unchanged. The module alias
// preserves module-qualified `scheduler::tu_store::...` paths (locked by
// `m18_persistence_incremental_test.rs:226`); the glob preserves the flat
// names; in-crate `crate::tu_store::` uses keep resolving. Do not repoint
// consumers around these shims (shape 4.2/4.4 - the compat contract).
pub use ariadne_worker_core::tu_store;
pub use ariadne_worker_core::tu_store::*;

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
