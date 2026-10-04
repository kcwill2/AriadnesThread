//! Boot-step snapshot producer for the web progress state.
//!
//! The terminal renderer in `crate::commands::progress` keeps the per-step
//! `[k/N]` bar frames inside its own closure. `BootStepSnapshot` mirrors
//! exactly those frames into `ServerProgress.steps` rows so the browser
//! receives the same per-step picture the terminal commits. The
//! I2 terminal-parity oracle in `boot_progress_tests.rs` freezes this
//! equivalence against the UNCHANGED renderer (milestone gate G2).

use super::{step_percent, ProgressReport};
use crate::commands::progress::strip_stage_prefix;
use ariadne_ui::{ProgressStepStatus, ProgressStepView};

/// One in-memory boot snapshot: `total_stages` plus one `ProgressStepView`
/// row per stage, in stage order. `seeded` materializes every row at boot
/// (DP-1); `apply` advances the rows by replaying one `ProgressReport`.
/// Pure value type - no locks inside; callers own the synchronization and
/// the `apply` call site stays outside the state write lock.
pub struct BootStepSnapshot {
    total_stages: usize,
    rows: Vec<ProgressStepView>,
    /// Highest stage seen so far: frames for an earlier stage are stale
    /// (defensive I3 guard; the producer stream is stage-monotone).
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
