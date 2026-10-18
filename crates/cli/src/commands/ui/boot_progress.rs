//! Boot-step snapshot producer for the web progress state.
//!
//! The terminal renderer in `crate::commands::progress` keeps the per-step
//! `[k/N]` bar frames inside its own closure. `BootStepSnapshot` mirrors
//! exactly those frames into `ServerProgress.steps` rows so the browser
//! receives the same per-step picture the terminal commits. The
//! I2 terminal-parity oracle in `boot_progress_tests.rs` freezes this
//! equivalence against the UNCHANGED renderer (milestone gate G2).
//!
//! A-1 (extraction-auto-apply): the boot-progress family below moved
//! VERBATIM out of `../ui.rs` to discharge the deferred W-2a F2
//! line-ceiling watch (ui.rs closed the wizard at 986/1000 lines); A-4's
//! apply install lands later in `ui.rs` and needs the headroom. `ui.rs`
//! re-exports the family so the frozen test imports and the production
//! callers there resolve UNEDITED. Items are `pub` at the module
//! boundary; the module itself is private to `commands::ui` (the
//! `stage_spans` move precedent).

use super::{
    build_ui_payload, step_percent, swap_into_state, write_state, ProgressFn, ProgressReport,
    Result, RUN_PHASES, STAGES_PER_RUN, UiBuildResult, UiState,
};
use crate::commands::progress::{strip_stage_prefix, TerminalStepBars};
use ariadne_index::StageFingerprints;
use ariadne_ui::{ProgressStepStatus, ProgressStepView};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Live per-step terminal bar renderer. TerminalStepBars is Send + Sync, so
/// the closure satisfies ProgressFn's Send + Sync bound and may move into
/// spawn_blocking; dropping the closure at the end of the build also drops
/// the guard, which commits any bar line left open by a failed build.
pub fn make_bar_progress() -> ProgressFn {
    // Fresh stdout handle per build, exactly what `TerminalStepBars::new`
    // builds internally.
    make_bar_progress_with_sink(Arc::new(Mutex::new(std::io::stdout())))
}

/// Bar renderer onto a caller-owned byte sink: the test seam over the
/// frozen `TerminalStepBars::with_sink` (boot-progress-visibility V1).
pub(crate) fn make_bar_progress_with_sink(sink: Arc<Mutex<dyn Write + Send>>) -> ProgressFn {
    let bars = TerminalStepBars::with_sink(sink);
    Arc::new(move |report: ProgressReport| {
        let step = step_percent(report.stage, report.percent);
        bars.render(
            report.stage,
            STAGES_PER_RUN,
            report.phase,
            step,
            &report.message,
        );
    })
}

/// Boot progress for the interactive server: mirrors the global percent into
/// web `UiState` (browser bar unchanged) and drives the live per-step bar.
pub fn make_boot_progress(state: Arc<UiState>) -> ProgressFn {
    make_boot_progress_with_sink(state, Arc::new(Mutex::new(std::io::stdout())))
}

/// Boot progress whose bar renders onto a caller-owned sink: the test seam
/// over `make_boot_progress` (boot-progress-visibility V1). Each report
/// additionally advances the per-step row snapshot written into
/// `progress.steps` (ui-progress-parity T2): `apply` is pure, so it runs
/// OUTSIDE the state write lock and the rows join the existing four global
/// writes in the SAME single write pass (I4: globals written exactly as
/// before; the error arm and `swap_into_state` never touch rows, so rows
/// freeze by construction - I6).
pub(crate) fn make_boot_progress_with_sink(
    state: Arc<UiState>,
    sink: Arc<Mutex<dyn Write + Send>>,
) -> ProgressFn {
    let bar = make_bar_progress_with_sink(sink);
    let snapshot = Mutex::new(BootStepSnapshot::seeded(STAGES_PER_RUN, &RUN_PHASES));
    Arc::new(move |report: ProgressReport| {
        let rows = snapshot
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .apply(&report);
        {
            let mut p = write_state(&state.progress);
            p.state = "loading".to_string();
            p.phase = report.phase.to_string();
            p.message = report.message.clone();
            p.percent = report.percent;
            p.steps = rows;
            p.total_stages = STAGES_PER_RUN;
        }
        bar(report);
    })
}

/// Best-effort panic payload text: `panic!` carries `&str` or `String`;
/// anything else degrades to a fixed marker, never silence (R8).
pub fn panic_payload_text(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

/// Write the terminal error state for a failed or panicked boot (I11): the
/// web UI leaves "loading" carrying the failure text. Committing any open
/// bar line is the TerminalStepBars guard-drop's job, not this write's.
pub fn surface_boot_failure(state: &UiState, text: String) {
    let mut progress = write_state(&state.progress);
    progress.state = "error".to_string();
    progress.message = text;
}

/// Run the boot build to its terminal UI state, mirroring the
/// `spawn_watcher` catch precedent so a dead boot is LOUD (I11, evidence
/// E3: a swallowed panic left the UI serving a frozen loading row while
/// the truth went to an unwatched console stderr). `Err` results and
/// escaped panics both write web state `"error"` with the failure text and
/// send `None` fingerprints, so the watcher parks instead of guarding a
/// dead generation. The boot surface logs `error!` where the watcher warns
/// (R8: never silent).
pub fn drive_boot_build(
    state: &Arc<UiState>,
    fps_tx: tokio::sync::oneshot::Sender<Option<StageFingerprints>>,
    build_fn: impl FnOnce() -> Result<UiBuildResult>,
) {
    let load_started = std::time::Instant::now();
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(build_fn)) {
        Ok(Ok(out)) => {
            crate::commands::progress::print_load_complete(load_started.elapsed());
            let fps = swap_into_state(state, out);
            let _ = fps_tx.send(Some(fps));
        }
        Ok(Err(e)) => {
            // The progress Arc was consumed, so its guard has already
            // committed any half-drawn bar line: errors print on a fresh one.
            surface_boot_failure(state, e.to_string());
            let _ = fps_tx.send(None);
        }
        Err(payload) => {
            // `&*payload`, not `&payload`: a bare reference unsizes to a
            // `dyn Any` view OF the box, not its contents (probe 2026-10-05).
            let text = panic_payload_text(&*payload);
            surface_boot_failure(state, text.clone());
            tracing::error!("UI boot task panicked: {text}");
            let _ = fps_tx.send(None);
        }
    }
}

/// The generation boot swaps in: a plain payload build stamped with the
/// `UiState` epoch boot's swap consumes. `new_loading` starts at epoch 0,
/// so boot's swap installs generation epoch 1 (the wizard reader carries
/// it; the watcher counts up from 2). R1 extraction from `run_ui`.
pub fn build_boot_generation(repo_path: &PathBuf, progress: ProgressFn) -> Result<UiBuildResult> {
    let mut built = build_ui_payload(repo_path, progress)?;
    built.install_epoch = 1;
    Ok(built)
}

/// One in-memory boot snapshot: `total_stages` plus one `ProgressStepView`
/// row per stage, in stage order. `seeded` materializes every row at boot
/// (DP-1); `apply` advances the rows by replaying one `ProgressReport`.
/// Pure value type - no locks inside; callers own the synchronization and
/// the `apply` call site stays outside the state write lock.
pub struct BootStepSnapshot {
    total_stages: usize,
    rows: Vec<ProgressStepView>,
    /// Highest stage seen so far: frames for an earlier stage are stale
    /// (the producer stream is stage-monotone).
    active_stage: Option<usize>,
    /// Mirrors `StepBarRenderer::finalized`: once the final step's 100%
    /// frame is seen, later frames can no longer commit or redraw it.
    finalized: bool,
}

impl BootStepSnapshot {
    /// All rows `Pending` at percent 0 with an empty message; labels are
    /// stored with the `[k/N] ` prefix stripped, so call sites pass the
    /// `*_PHASE` constants verbatim (`ARTIFACTS_PHASE .. COLLATERAL_PHASE`).
    pub fn seeded(total_stages: usize, labels: &[&'static str]) -> Self {
        let rows = labels
            .iter()
            .enumerate()
            .map(|(index, phase)| ProgressStepView {
                stage: index + 1,
                label: strip_stage_prefix(phase).to_string(),
                percent: 0,
                message: String::new(),
                status: ProgressStepStatus::Pending,
            })
            .collect();
        Self {
            total_stages,
            rows,
            active_stage: None,
            finalized: false,
        }
    }

    /// Snapshot of the current rows (row order = stage order). Used by the
    /// `run_ui` boot seed to write the all-`Pending` DP-1 rows without
    /// faking a report through `apply`.
    pub fn rows(&self) -> Vec<ProgressStepView> {
        self.rows.clone()
    }

    /// Apply one report and return the full row list (row order = stage
    /// order). Frames older than the active stage are ignored; frames
    /// after the finalize frame are ignored too, mirroring
    /// `StepBarRenderer::frame`'s finalized short-circuit.
    pub fn apply(&mut self, report: &ProgressReport) -> Vec<ProgressStepView> {
        let stale_or_final = self.finalized || self.stale_frame(report.stage);
        if !stale_or_final && self.in_range(report.stage) {
            self.commit_frames_before(report.stage);
            self.update_current_row(report);
            self.active_stage = Some(self.active_stage.unwrap_or(0).max(report.stage));
        }
        self.rows.clone()
    }

    /// True when the frame belongs to a stage strictly before the one
    /// currently owning the open line.
    fn stale_frame(&self, stage: usize) -> bool {
        self.active_stage.is_some_and(|active| stage < active)
    }

    fn in_range(&self, stage: usize) -> bool {
        (1..=self.rows.len()).contains(&stage)
    }

    /// Every stage that has already passed becomes `Done` at 100, keeping
    /// the message of that stage's last observed frame (a stage whose final
    /// frame already read 100 keeps that message, e.g. "Include graph
    /// ready"). Already-`Done` rows are left untouched.
    fn commit_frames_before(&mut self, stage: usize) {
        for row_index in 0..stage - 1 {
            let row = &mut self.rows[row_index];
            if row.status != ProgressStepStatus::Done {
                row.status = ProgressStepStatus::Done;
                row.percent = 100;
            }
        }
    }

    /// The row owning the open line: `Done` at 100 when the finalize rule
    /// from `StepBarRenderer::frame` fires (last stage at step-percent
    /// >= 100), otherwise `Active` at the step's own projected percent.
    fn update_current_row(&mut self, report: &ProgressReport) {
        let row = &mut self.rows[report.stage - 1];
        let percent = step_percent(report.stage, report.percent);
        row.message = report.message.clone();
        if report.stage == self.total_stages && percent >= 100 {
            row.status = ProgressStepStatus::Done;
            row.percent = 100;
            self.finalized = true;
        } else {
            row.status = ProgressStepStatus::Active;
            row.percent = percent;
        }
    }
}

#[cfg(test)]
mod tests {
    //! A-1 (extraction-auto-apply) move locks: the boot-progress family
    //! must live in THIS module, be gone from `../ui.rs`, and `ui.rs` must
    //! stay under the extraction ceiling that discharges the deferred W-2a
    //! F2 line-count watch. Every needle is built via format! so this test
    //! module's own source cannot match itself (`progress_contract_tests`
    //! line 73 precedent).

    /// The eight family names in family order: the first missing (or
    /// lingering) name is the one the failing assert reports.
    fn family_names() -> [&'static str; 8] {
        [
            "make_bar_progress",
            "make_bar_progress_with_sink",
            "make_boot_progress",
            "make_boot_progress_with_sink",
            "panic_payload_text",
            "surface_boot_failure",
            "drive_boot_build",
            "build_boot_generation",
        ]
    }

    #[test]
    fn test_boot_progress_family_lives_in_this_module() {
        let source = include_str!("boot_progress.rs");
        for name in family_names() {
            let needle = format!("fn {name}");
            assert!(
                source.contains(needle.as_str()),
                "boot_progress.rs must define fn {name}"
            );
        }
    }

    #[test]
    fn test_boot_progress_family_is_gone_from_ui_rs() {
        let source = include_str!("../ui.rs");
        for name in family_names() {
            let needle = format!("fn {name}");
            assert!(
                !source.contains(needle.as_str()),
                "ui.rs must no longer define fn {name}; the family belongs \
                 in boot_progress.rs"
            );
        }
    }

    #[test]
    fn test_ui_rs_is_under_the_extraction_ceiling() {
        let line_count = include_str!("../ui.rs").lines().count();
        assert!(
            line_count <= 880,
            "ui.rs must drop under the extraction ceiling, got {line_count} \
             lines (the W-2a F2 deferred row)"
        );
    }
}
