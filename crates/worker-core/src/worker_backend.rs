//! The worker backend contract.
//!
//! `WorkerBackend` moved verbatim from `ariadne-scheduler::worker` in
//! AR-20a, together with its `Arc` blanket impl: the orphan rule forces
//! that blanket impl to live in the trait's defining crate, so it cannot
//! stay in the scheduler. Implementor-side backends (local types such as
//! `SharedPluginWorkerBackend` / `PluginWorkerBackend`) stay wherever
//! their types live.

use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use crate::task::TuTask;
use ariadne_index::TuAnalysis;

pub trait WorkerBackend: Send + Sync {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis>;
}

/// `WorkerPool` stores backends as `Arc<B>`, and the registry wiring hands it
/// an `Arc<dyn WorkerBackend>` (M18.3): dispatch through the `Arc` to the
/// wrapped backend.
impl<T: WorkerBackend + ?Sized> WorkerBackend for Arc<T> {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        T::analyze_task(self, task, repo_root)
    }
}
