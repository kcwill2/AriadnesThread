pub fn format_progress(current: usize, total: usize, prefix: &str, unit: &str) -> String {
    let width = 20;
    let percentage = if total > 0 {
        (current as f64 / total as f64 * 100.0) as usize
    } else {
        100
    };
    let filled = if total > 0 {
        (current * width) / total
    } else {
        width
    };
    let empty = width.saturating_sub(filled);

    let bar = format!("{}{}", "█".repeat(filled), " ".repeat(empty));
    format!(
        "{} [{}] {}% ({}/{} {})",
        prefix, bar, percentage, current, total, unit
    )
}

pub fn print_progress(current: usize, total: usize, prefix: &str, unit: &str) {
    use std::io::{self, Write};
    let s = format_progress(current, total, prefix, unit);
    print!("\r{}", s);
    if current == total || total == 0 {
        println!();
    }
    let _ = io::stdout().flush();
}

pub const STEP_BAR_WIDTH: usize = 20;

/// Remove a leading `[k/n] ` stage label from a phase string so renderers can
/// prepend their own canonical `[{stage}/{total}]` label without duplication.
pub fn strip_stage_prefix(phase: &str) -> &str {
    if !phase.starts_with('[') {
        return phase;
    }
    let Some(close) = phase.find(']') else {
        return phase;
    };
    if close >= 2 && phase[1..close].contains('/') {
        let rest = &phase[close + 1..];
        return rest.strip_prefix(' ').unwrap_or(rest);
    }
    phase
}

/// One per-step progress frame: `[stage/total] title [bar] percent  message`,
/// where the bar shows the step's OWN 0→100, not a global percent.
pub fn format_step_bar(
    stage: usize,
    total_steps: usize,
    phase: &str,
    step_percent: usize,
    message: &str,
) -> String {
    let percent = step_percent.min(100);
    let filled = STEP_BAR_WIDTH * percent / 100;
    let empty = STEP_BAR_WIDTH - filled;
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
    format!(
        "[{}/{}] {} [{}] {:>3}%  {}",
        stage,
        total_steps,
        strip_stage_prefix(phase),
        bar,
        percent,
        message
    )
}

/// A rendered frame plus the terminal-control hints the caller needs.
pub struct StepFrame {
    pub line: String,
    pub needs_newline: bool,
    pub finalize: bool,
}

/// Remembers which step currently owns the open terminal line so callers know
/// when a `\n` must be committed before starting the next step's line, and
/// whether the run's final step already finalized its line.
#[derive(Default)]
pub struct StepBarRenderer {
    open_line_stage: Option<usize>,
    finalized: bool,
}

impl StepBarRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// True while a live `\r` line is still unterminated and owned by a step.
    pub fn has_open_line(&self) -> bool {
        self.open_line_stage.is_some()
    }

    /// Pure frame computation: does the open line need committing first, and
    /// does this frame terminate the whole run's last step? A 100% frame on
    /// any earlier step only redraws; its line is committed by the `\n` that
    /// precedes the next step's first frame, so mid-run and milestone frames
    /// at 100% can never duplicate the committed line.
    pub fn frame(
        &mut self,
        stage: usize,
        total_steps: usize,
        phase: &str,
        step_percent: usize,
        message: &str,
    ) -> StepFrame {
        let line = format_step_bar(stage, total_steps, phase, step_percent, message);
        if self.finalized {
            return StepFrame {
                line,
                needs_newline: false,
                finalize: false,
            };
        }
        let needs_newline = matches!(self.open_line_stage, Some(open) if open != stage);
        let finalize = step_percent >= 100 && stage == total_steps;
        if finalize {
            self.finalized = true;
            self.open_line_stage = None;
        } else {
            self.open_line_stage = Some(stage);
        }
        StepFrame {
            line,
            needs_newline,
            finalize,
        }
    }
}

/// Owns the renderer plus the byte sink it writes to. Dropping it commits an
/// unterminated live bar with `\n` so an aborted build cannot splice its
/// error text onto a half-drawn line.
pub struct TerminalStepBars {
    renderer: std::sync::Mutex<StepBarRenderer>,
    sink: std::sync::Arc<std::sync::Mutex<dyn std::io::Write + Send>>,
}

impl Default for TerminalStepBars {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalStepBars {
    pub fn new() -> Self {
        Self::with_sink(std::sync::Arc::new(std::sync::Mutex::new(std::io::stdout())))
    }

    pub fn with_sink(
        sink: std::sync::Arc<std::sync::Mutex<dyn std::io::Write + Send>>,
    ) -> Self {
        Self {
            renderer: std::sync::Mutex::new(StepBarRenderer::new()),
            sink,
        }
    }

    pub fn render(
        &self,
        stage: usize,
        total_steps: usize,
        phase: &str,
        step_percent: usize,
        message: &str,
    ) {
        let frame = {
            let mut renderer = self
                .renderer
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            renderer.frame(stage, total_steps, phase, step_percent, message)
        };
        let mut sink = self
            .sink
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if frame.needs_newline {
            let _ = sink.write_all(b"\n");
        }
        let _ = sink.write_all(b"\r");
        let _ = sink.write_all(frame.line.as_bytes());
        if frame.finalize {
            let _ = sink.write_all(b"\n");
        }
        let _ = sink.flush();
    }
}

impl Drop for TerminalStepBars {
    /// Commit a still-open bar line so terminal output after an aborted build
    /// (e.g. an anyhow error printed by main) starts on a fresh line.
    /// io errors are swallowed exactly like `render`'s writes.
    fn drop(&mut self) {
        let has_open_line = {
            let renderer = self
                .renderer
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            renderer.has_open_line()
        };
        if !has_open_line {
            return;
        }
        let mut sink = self
            .sink
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _ = sink.write_all(b"\n");
        let _ = sink.flush();
    }
}

#[cfg(test)]
mod step_bar_tests {
    use super::{StepBarRenderer, TerminalStepBars, STEP_BAR_WIDTH, format_step_bar};

    #[test]
    fn test_step_bar_fill_tracks_step_percent() {
        let line = format_step_bar(2, 5, "Indexing translation units...", 50, "12/24 TUs");
        assert_eq!(
            line.matches('█').count(),
            50 * STEP_BAR_WIDTH / 100,
            "bar fill must equal step_percent/5"
        );
        assert_eq!(line.matches('░').count(), STEP_BAR_WIDTH - 10);
        assert!(line.starts_with("[2/5] Indexing translation units..."));
        assert!(line.ends_with("12/24 TUs"));
        assert!(line.contains(" 50%"));
    }

    #[test]
    fn test_step_bar_empty_and_full_extremes() {
        let empty = format_step_bar(1, 5, "Loading artifacts...", 0, "Hashing sources");
        assert_eq!(empty.matches('█').count(), 0);
        assert_eq!(empty.matches('░').count(), STEP_BAR_WIDTH);
        let full = format_step_bar(1, 5, "Loading artifacts...", 100, "Include graph ready");
        assert_eq!(full.matches('█').count(), STEP_BAR_WIDTH);
        assert_eq!(full.matches('░').count(), 0);
    }

    #[test]
    fn test_step_bar_does_not_duplicate_stage_label() {
        let line = format_step_bar(3, 5, "[3/5] Reducing semantic model...", 20, "Deduplicating");
        assert_eq!(line.matches("[3/5]").count(), 1);
        assert!(line.starts_with("[3/5] Reducing semantic model... ["));
    }

    #[test]
    fn test_only_final_step_one_hundred_finalizes_the_line() {
        let mut renderer = StepBarRenderer::new();
        renderer.frame(2, 5, "Indexing translation units...", 90, "22/24 TUs");
        let done_event = renderer.frame(2, 5, "Indexing translation units...", 100, "24/24 TUs");
        assert!(
            !done_event.finalize,
            "a 100% event on a middle step must only redraw, not commit"
        );
        assert!(
            !renderer
                .frame(2, 5, "Indexing translation units...", 100, "Index complete")
                .needs_newline,
            "the 100% milestone re-renders the same open line, never a second one"
        );
        let next_step = renderer.frame(3, 5, "Reducing semantic model...", 0, "Deduplicating");
        assert!(
            next_step.needs_newline,
            "the step-2 line is committed exactly once, when step 3 starts"
        );
        assert!(
            !renderer
                .frame(3, 5, "Reducing semantic model...", 20, "more")
                .needs_newline
        );
    }

    #[test]
    fn test_final_step_finalizes_once_without_duplicate_commits() {
        let mut renderer = StepBarRenderer::new();
        assert!(
            !renderer
                .frame(5, 5, "Analyzing collateral...", 95, "Collateral mapped")
                .finalize
        );
        assert!(
            renderer
                .frame(5, 5, "Analyzing collateral...", 100, "Payload assembled")
                .finalize
        );
        assert!(
            !renderer
                .frame(5, 5, "Analyzing collateral...", 100, "Payload assembled")
                .finalize,
            "repeated 100% frames of the finished run must not commit again"
        );
        assert!(
            !renderer
                .frame(5, 5, "Analyzing collateral...", 100, "Payload assembled")
                .needs_newline
        );
    }

    #[test]
    fn test_abandoning_an_open_line_commits_it_once() {
        let mut renderer = StepBarRenderer::new();
        assert!(!renderer.frame(1, 5, "a", 0, "first").needs_newline);
        assert!(
            renderer.frame(2, 5, "b", 0, "switch").needs_newline,
            "abandoning step 1's live line requires committing a newline"
        );
        assert!(
            !renderer.frame(2, 5, "b", 35, "same").needs_newline,
            "stay-on-line redraws within one step need no newline"
        );
    }

    /// Test sink: the renderer writes into a shared Vec the test can read
    /// back after the guard (and its dyn handle) is gone.
    #[derive(Clone)]
    struct MemorySink {
        buffer: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    }

    impl MemorySink {
        fn new() -> Self {
            Self {
                buffer: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            }
        }

        fn handle(&self) -> std::sync::Arc<std::sync::Mutex<dyn std::io::Write + Send>> {
            self.buffer.clone()
        }

        fn text(&self) -> String {
            String::from_utf8(self.buffer.lock().unwrap().clone()).unwrap()
        }
    }

    #[test]
    fn test_drop_commits_an_unterminated_bar() {
        let sink = MemorySink::new();
        {
            let bars = TerminalStepBars::with_sink(sink.handle());
            bars.render(2, 5, "[2/5] Indexing translation units...", 100, "24/24 TUs");
            let live = sink.text();
            assert!(live.contains("\r[2/5]"), "frame must render in place");
            assert!(
                !live.contains('\n'),
                "step 2 at 100% is not lifecycle end: the line stays open"
            );
        }
        let committed = sink.text();
        assert_eq!(
            committed.matches('\n').count(),
            1,
            "dropping the renderer must commit the open bar line exactly once"
        );
        assert!(committed.ends_with('\n'));
    }

    #[test]
    fn test_drop_after_finalize_writes_no_extra_newline() {
        let sink = MemorySink::new();
        {
            let bars = TerminalStepBars::with_sink(sink.handle());
            bars.render(5, 5, "[5/5] Analyzing collateral...", 100, "Payload assembled");
        }
        let committed = sink.text();
        assert!(committed.contains("Payload assembled"));
        assert_eq!(
            committed.matches('\n').count(),
            1,
            "the finalize newline is the only commit; drop must not add one"
        );
    }

    #[test]
    fn test_drop_without_any_frame_writes_nothing() {
        let sink = MemorySink::new();
        drop(TerminalStepBars::with_sink(sink.handle()));
        assert!(sink.text().is_empty(), "a never-used renderer must stay silent");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_progress() {
        let out = format_progress(5, 10, "Indexing", "TUs");
        assert_eq!(out, "Indexing [██████████          ] 50% (5/10 TUs)");

        let out2 = format_progress(10, 10, "Indexing", "TUs");
        assert_eq!(out2, "Indexing [████████████████████] 100% (10/10 TUs)");

        let out3 = format_progress(0, 10, "Indexing", "TUs");
        assert_eq!(out3, "Indexing [                    ] 0% (0/10 TUs)");
    }
}
