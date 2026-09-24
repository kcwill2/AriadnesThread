//! Task scheduler for parallel analysis and indexing.

pub mod incremental;
pub use incremental::*;

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
