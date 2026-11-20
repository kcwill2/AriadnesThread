//! Time-remaining estimates for the counted stage-5 sub-steps.
//!
//! The stage-5 bar is cut into fixed percent spans calibrated on one
//! repository, so on another repository a span can take far longer than its
//! share of the bar suggests. A counted sub-step ("1200/9000 files") knows
//! its own rate, so its frames carry an estimate measured on the repository
//! being loaded: `1200/9000 files, ~2m 10s left`.
//!
//! The estimate is linear (elapsed x remaining / done) and appears only once
//! the sub-step has run for `MINIMUM_ELAPSED`: early rates are noise, and
//! short loads keep their messages unchanged.

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// A sub-step must run this long before its frames carry an estimate.
const MINIMUM_ELAPSED: Duration = Duration::from_secs(3);

/// `~45s left`, `~2m 10s left`, `~1h 05m left`.
fn format_duration_left(remaining: Duration) -> String {
    let seconds = remaining.as_secs();
    match seconds {
        0..=59 => format!("~{}s left", seconds.max(1)),
        60..=3599 => format!("~{}m {:02}s left", seconds / 60, seconds % 60),
        _ => format!("~{}h {:02}m left", seconds / 3600, seconds % 3600 / 60),
    }
}

/// The remaining-time text for a sub-step `done` of `total` items in after
/// `elapsed`, or `None` while there is nothing sound to say: nothing done
/// yet, already finished, or too early to trust the rate.
pub fn remaining_text(elapsed: Duration, done: usize, total: usize) -> Option<String> {
    if done == 0 || done >= total || elapsed < MINIMUM_ELAPSED {
        return None;
    }
    let remaining = elapsed.mul_f64((total - done) as f64 / done as f64);
    Some(format_duration_left(remaining))
}

/// Clock of the sub-step currently reporting counts: it restarts whenever
/// the sub-step name changes.
#[derive(Default)]
pub struct SubstepEta {
    running: Mutex<Option<(&'static str, Instant)>>,
}

impl SubstepEta {
    /// The text to append to a `done/total` frame of `substep`: empty, or
    /// `, ~2m 10s left`.
    pub fn suffix(&self, substep: &'static str, done: usize, total: usize) -> String {
        let started = {
            let mut running = self.running.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            match *running {
                Some((name, started)) if name == substep => started,
                _ => {
                    let now = Instant::now();
                    *running = Some((substep, now));
                    now
                }
            }
        };
        remaining_text(started.elapsed(), done, total).map_or_else(String::new, |text| format!(", {text}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seconds(count: u64) -> Duration {
        Duration::from_secs(count)
    }

    #[test]
    fn estimate_scales_elapsed_by_the_work_left() {
        // A quarter done in 10 s: three quarters left take 30 s.
        assert_eq!(remaining_text(seconds(10), 250, 1000).as_deref(), Some("~30s left"));
        assert_eq!(remaining_text(seconds(65), 500, 1500).as_deref(), Some("~2m 10s left"));
        assert_eq!(remaining_text(seconds(600), 100, 750).as_deref(), Some("~1h 05m left"));
    }

    #[test]
    fn a_nearly_done_step_never_reads_zero_seconds() {
        assert_eq!(remaining_text(seconds(100), 999, 1000).as_deref(), Some("~1s left"));
    }

    #[test]
    fn no_estimate_without_progress_at_the_end_or_too_early() {
        assert_eq!(remaining_text(seconds(60), 0, 100), None);
        assert_eq!(remaining_text(seconds(60), 100, 100), None);
        assert_eq!(remaining_text(seconds(60), 0, 0), None);
        assert_eq!(remaining_text(Duration::from_millis(2999), 50, 100), None);
    }

    #[test]
    fn a_fresh_substep_adds_nothing_to_the_message() {
        let eta = SubstepEta::default();
        assert_eq!(eta.suffix("files", 5, 10), "");
        // A new sub-step restarts the clock, so it is fresh again.
        assert_eq!(eta.suffix("symbols", 5, 10), "");
    }
}
