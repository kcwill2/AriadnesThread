//! Neutral worker vocabulary for TU analysis: the content-keyed store
//! engine (`tu_store`), the queue task unit (`task`), and the worker
//! backend contract (`worker_backend`).
//!
//! Extracted verbatim from `ariadne-scheduler` in AR-20a (shape ruling
//! `tasks/ar-20-shape.md`, C-lite): the engine was never scheduler logic,
//! only scheduler-located, so plugin crates can call it as a legal
//! downward call once it lives below both scheduler and the plugins.
//! Scheduler keeps every moved name resolving via re-export shims, so
//! all pre-existing consumers compile unedited.

pub mod tu_store;
pub use tu_store::*;

pub mod task;
pub use task::*;

pub mod worker_backend;
pub use worker_backend::*;
