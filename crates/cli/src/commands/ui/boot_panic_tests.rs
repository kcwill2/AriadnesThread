//! I11 locks (boot-progress-visibility V1): a dead boot is LOUD. The boot
//! task body is testable through `drive_boot_build` (watcher precedent
//! mirrored at the `spawn_watcher` catch); both a returned `Err` and an
//! escaped panic must land in web state `"error"` with the failure text,
//! send `None` fingerprints, and commit any open bar line exactly once.

use super::{
    drive_boot_build, make_boot_progress_with_sink, read_state, stage_report, UiBuildResult,
    TU_INDEX_PHASE,
};
use anyhow::Result;
use ariadne_ui::UiState;
use std::io::Write;
use std::sync::{Arc, Mutex};

/// Test sink: byte buffer behind the same `Arc<Mutex<dyn Write + Send>>`
/// seam `TerminalStepBars::with_sink` takes (imitates the `MemorySink`
/// precedent in `progress.rs`'s own tests).
struct MemorySink {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl MemorySink {
    fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn handle(&self) -> Arc<Mutex<dyn Write + Send>> {
        self.buffer.clone()
    }

    fn text(&self) -> String {
        String::from_utf8(self.buffer.lock().unwrap().clone()).expect("utf-8 sink")
    }
}

fn loading_state() -> Arc<UiState> {
    let repo_dir = tempfile::TempDir::new().expect("temp repo");
    Arc::new(UiState::new_loading(repo_dir.path().to_path_buf()))
}

#[test]
fn test_panic_in_boot_task_surfaces_error_state() {
    let state = loading_state();
    let (fps_tx, mut fps_rx) = tokio::sync::oneshot::channel();
    drive_boot_build(&state, fps_tx, || -> Result<UiBuildResult> {
        panic!("injected boot failure")
    });
    let progress = read_state(&state.progress);
    assert_eq!(
        progress.state, "error",
        "a panicked boot must not leave the UI stuck in 'loading'"
    );
    assert!(
        progress.message.contains("injected boot failure"),
        "the panic text must reach the web state, got {:?}",
        progress.message
    );
    drop(progress);
    assert!(
        matches!(fps_rx.try_recv(), Ok(None)),
        "a panicked boot must report no fingerprints (watcher stays parked)"
    );
}

#[test]
fn test_error_result_surfaces_error_state() {
    let state = loading_state();
    let (fps_tx, mut fps_rx) = tokio::sync::oneshot::channel();
    drive_boot_build(&state, fps_tx, || -> Result<UiBuildResult> {
        Err(anyhow::anyhow!("injected build error"))
    });
    let progress = read_state(&state.progress);
    assert_eq!(progress.state, "error");
    assert!(
        progress.message.contains("injected build error"),
        "the error text must reach the web state, got {:?}",
        progress.message
    );
    drop(progress);
    assert!(
        matches!(fps_rx.try_recv(), Ok(None)),
        "a failed build must report no fingerprints"
    );
}

#[test]
fn test_panicked_boot_commits_the_open_bar_line() {
    let sink = MemorySink::new();
    let state = loading_state();
    let progress = make_boot_progress_with_sink(state.clone(), sink.handle());
    progress(stage_report(2, 30, TU_INDEX_PHASE, "12/24 TUs"));
    let live = sink.text();
    assert!(
        live.contains("\r[2/5]"),
        "the stage-2 frame renders live on the sink"
    );
    assert!(
        !live.contains('\n'),
        "a mid-run stage-2 line stays open while the boot continues"
    );

    let (fps_tx, mut fps_rx) = tokio::sync::oneshot::channel();
    let boot = progress.clone();
    drive_boot_build(&state, fps_tx, move || -> Result<UiBuildResult> {
        boot(stage_report(2, 60, TU_INDEX_PHASE, "still indexing"));
        panic!("injected boot failure")
    });
    assert_eq!(read_state(&state.progress).state, "error");
    assert!(matches!(fps_rx.try_recv(), Ok(None)));

    // The build closure consumed its clone; releasing the test's own handle
    // is what drops the TerminalStepBars guard.
    drop(progress);
    let committed = sink.text();
    assert!(
        committed.contains("still indexing"),
        "the last live bar must be on the sink before the commit"
    );
    assert!(
        committed.ends_with('\n'),
        "guard-drop commits the open bar line so the error text starts fresh"
    );
    assert_eq!(
        committed.matches('\n').count(),
        1,
        "exactly one commit newline, at the end"
    );
}

/// UTA-T7b row 3: `panic_payload_text` arm matrix - the two payload shapes
/// `panic!` actually carries are extracted verbatim, anything else degrades
/// to the fixed marker (never silence, R8).
#[test]
fn pin_panic_payload_text_string_and_foreign_payload_arms() {
    use super::boot_progress::panic_payload_text;

    let owned: String = "owned panic message".to_string();
    assert_eq!(
        panic_payload_text(&owned),
        "owned panic message",
        "the String payload arm must hand back the owned text verbatim"
    );

    let literal: &str = "static panic message";
    assert_eq!(
        panic_payload_text(&literal),
        "static panic message",
        "the &str payload arm must hand back the literal verbatim"
    );

    let foreign = 42u32;
    assert_eq!(
        panic_payload_text(&foreign),
        "unknown panic payload",
        "a non-string payload degrades to the fixed marker, never silence"
    );
}
