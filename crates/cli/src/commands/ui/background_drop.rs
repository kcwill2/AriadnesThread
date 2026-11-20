//! Free a build's temporaries off the boot's critical path.
//!
//! After "Payload assembled", `build_ui_payload` still owns the pipeline
//! (with its cached model and call-graph copies), the load report and the
//! node maps. Freeing them on the build thread delayed the "Ready" swap by
//! time that grows with repo size: on large repos the page sat on "Payload
//! assembled" while the allocator walked gigabytes. Nothing reads them
//! again, so a short-lived thread frees them while the swap proceeds.

/// Drop `value` on a detached thread and return at once. Falls back to an
/// inline drop if the thread cannot be spawned.
pub(super) fn drop_in_background<T: Send + 'static>(value: T) {
    let mut slot = Some(value);
    let spawned = std::thread::Builder::new()
        .name("ui-build-free".to_string())
        .spawn({
            let value = slot.take();
            move || drop(value)
        });
    if spawned.is_err() {
        drop(slot);
    }
}

#[cfg(test)]
mod tests {
    use super::drop_in_background;
    use std::sync::mpsc;
    use std::time::Duration;

    /// Signals its drop, after first waiting for the test's go-ahead.
    struct SlowDrop {
        release: mpsc::Receiver<()>,
        dropped: mpsc::Sender<std::thread::ThreadId>,
    }

    impl Drop for SlowDrop {
        fn drop(&mut self) {
            let _ = self.release.recv_timeout(Duration::from_secs(5));
            let _ = self.dropped.send(std::thread::current().id());
        }
    }

    #[test]
    fn returns_before_the_drop_finishes_and_drops_on_another_thread() {
        let (release_tx, release_rx) = mpsc::channel();
        let (dropped_tx, dropped_rx) = mpsc::channel();
        drop_in_background(SlowDrop { release: release_rx, dropped: dropped_tx });
        // Reaching here while the drop still waits proves the caller is free.
        assert!(dropped_rx.try_recv().is_err(), "drop must not have finished yet");
        release_tx.send(()).unwrap();
        let dropper = dropped_rx.recv_timeout(Duration::from_secs(5)).expect("value is dropped");
        assert_ne!(dropper, std::thread::current().id());
    }
}
