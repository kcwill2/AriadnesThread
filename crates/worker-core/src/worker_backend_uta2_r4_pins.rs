//! UTA2-R4 pin (docs/specs/unit-test-audit.md Round 2, handoff item 1): the
//! `WorkerBackend for Arc<T>` blanket impl is a pure delegating shim - the
//! `Arc` never inspects, buffers, or rewrites the task, and the wrapped
//! backend sees the arguments verbatim and answers exactly once per call.
//! Child module of `worker_backend` (the round-1 pin-file model).

use super::*;
use ariadne_build::{CompileCommand, TranslationUnit};
use ariadne_repo::FileId;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Delegating spy: records every `analyze_task` call (task id + repo root)
/// and answers with a sentinel analysis keyed on the task it was given, so
/// pass-through and single-dispatch are observable at once.
struct RecordingBackend {
    calls: AtomicUsize,
    seen_calls: Mutex<Vec<(usize, PathBuf)>>,
}

impl RecordingBackend {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            seen_calls: Mutex::new(Vec::new()),
        }
    }

    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn recorded_calls(&self) -> Vec<(usize, PathBuf)> {
        self.seen_calls.lock().unwrap().clone()
    }
}

impl WorkerBackend for RecordingBackend {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.seen_calls
            .lock()
            .unwrap()
            .push((task.task_id, repo_root.to_path_buf()));
        Ok(TuAnalysis {
            tu_file_id: task.tu.file_id,
            context_name: "spy-inner-sentinel".to_string(),
            ..Default::default()
        })
    }
}

fn spy_task(task_id: usize) -> TuTask {
    let rel = "src/spy.cpp";
    TuTask {
        task_id,
        tu: TranslationUnit {
            file_id: FileId::from_relative_path(rel),
            source_path: PathBuf::from(rel),
            context_name: "spy-context".into(),
            compile_command: CompileCommand {
                directory: PathBuf::new(),
                file: PathBuf::from(rel),
                command: None,
                arguments: vec![],
                context_name: "spy-context".into(),
                output: None,
            },
            language: Default::default(),
        },
        estimated_cost: 1,
        context: None,
    }
}

#[test]
fn pin_arc_backend_delegates_to_inner_once_with_verbatim_arguments() {
    // Shape 1 (WorkerPool storage): `Arc<B>` with a concrete backend. The
    // blanket impl must reach the inner impl EXACTLY once, forwarding the
    // task and repo root unchanged, and hand the inner's answer back
    // byte-identical.
    let backend = Arc::new(RecordingBackend::new());
    let task = spy_task(77);
    let repo_root = PathBuf::from("spy-root-1");

    let analysis = <Arc<RecordingBackend> as WorkerBackend>::analyze_task(&backend, &task, &repo_root)
        .expect("delegation must not alter the Ok result");

    assert_eq!(analysis.context_name, "spy-inner-sentinel");
    assert_eq!(analysis.tu_file_id, task.tu.file_id);
    assert_eq!(backend.call_count(), 1, "one outer call, one inner call");
    assert_eq!(
        backend.recorded_calls(),
        vec![(77, repo_root)],
        "the inner backend must see the verbatim task id and repo root"
    );

    // Shape 2 (M18.3 registry wiring): `Arc<dyn WorkerBackend>`. The same
    // blanket impl dispatches through the trait object's vtable to the
    // concrete spy - still exactly once, still verbatim.
    let witness = Arc::new(RecordingBackend::new());
    let concrete = Arc::clone(&witness);
    let erased: Arc<dyn WorkerBackend> = concrete;
    let dyn_task = spy_task(78);
    let dyn_root = PathBuf::from("spy-root-2");

    let analysis = <Arc<dyn WorkerBackend> as WorkerBackend>::analyze_task(
        &erased, &dyn_task, &dyn_root,
    )
    .expect("the erased shape must delegate as well");

    assert_eq!(analysis.context_name, "spy-inner-sentinel");
    assert_eq!(witness.call_count(), 1);
    assert_eq!(witness.recorded_calls(), vec![(78, dyn_root)]);
}
