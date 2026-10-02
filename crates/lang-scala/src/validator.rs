//! M23.4 (T4, D-M23-6): the Scala toolchain probe validator.
//!
//! Replaces `StubValidator` (M19.1, retired in this task) with
//! [`ScalaToolchainValidator`]. The probe mirrors the C++ sidecar shape
//! ([`crate`] has no dependency on it, but
//! `crates/lang-cpp/src/plugin.rs::probe_compiler_status` and
//! `tools/clang-worker/src/main.rs::check_clang_available` are the spawn
//! `--version` models): spawn a candidate, read its version banner. The
//! Scala probe adds the exit-success requirement locked by the tests below —
//! a candidate is usable only when the process could be spawned **and** the
//! `--version` run exited successfully.
//!
//! Probe contract (locked in [`validator::tests`], D-M23-6):
//! - candidates are probed in the exact order
//!   [`TOOLCHAIN_PROBE_ORDER`] = `scalac`, `scala-cli`, `sbt`;
//! - first usable candidate ⇒ [`ToolchainStatus::Available`] carrying the
//!   first non-empty stdout line (the binary name when the banner is empty —
//!   `ToolchainStatus::Available(String)` from `ariadne-lang-api` is the
//!   version carrier);
//! - no usable candidate ⇒ [`ToolchainStatus::Missing`] — the offline
//!   contract on this machine and in CI (I7), feeding the existing neutral
//!   `Missing ⇒ Skipped` tier-3 mapping in `crates/validate/src/lib.rs`.
//!
//! `validate_compile` per D-M23-12 (real invocation deferred for both
//! languages): with no usable toolchain it returns `Err` naming that
//! decision and never spawns anything beyond the `--version` probes; with a
//! usable toolchain it invokes `<hit> <compiler_args…> <source_files…>` —
//! the scalac options-before-sources CLI convention — and maps a failing or
//! vanished process to `Err` with the captured output. `TierValidator` never
//! calls `validate_compile`; this behavior matters only once T7 flips
//! `validation: true` and a toolchain actually exists.

use ariadne_lang::{CompilerValidator, ToolchainStatus};
use std::path::PathBuf;

/// Probe candidates, probed left to right until one is usable (D-M23-6).
pub const TOOLCHAIN_PROBE_ORDER: [&str; 3] = ["scalac", "scala-cli", "sbt"];

/// Arguments of every probe invocation, mirroring the C++ side's
/// `probe_compiler_status` `--version` shape.
pub(crate) const VERSION_PROBE_ARGS: &[&str] = &["--version"];

/// The single honest deferral message when no toolchain is usable
/// (D-M23-12: real compile invocation deferred out of M23).
const COMPILE_DEFERRAL: &str = "Scala compile deferred (D-M23-12): no usable Scala toolchain \
    (scalac/scala-cli/sbt) is present; offline path stays Missing and tier 3 reports Skipped";

/// One completed command run captured through [`CommandRunner`].
#[derive(Debug, Clone)]
pub struct CommandOutcome {
    /// Whether the process exited with success.
    pub success: bool,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
}

/// Seam over `std::process::Command` keeping the probe and the compile
/// invocation unit-testable without spawning real processes.
pub trait CommandRunner: Send + Sync {
    /// Spawn `binary` with `args`, capturing output. `None` only when the
    /// process could not be started (binary not resolvable); a process that
    /// started but exited unsuccessfully is `Some(CommandOutcome {
    /// success: false, .. })`.
    fn run(&self, binary: &str, args: &[&str]) -> Option<CommandOutcome>;
}

/// Production runner: `Command::new(binary).args(args).output()`.
pub struct ProcessCommandRunner;

impl CommandRunner for ProcessCommandRunner {
    fn run(&self, binary: &str, args: &[&str]) -> Option<CommandOutcome> {
        let output = std::process::Command::new(binary).args(args).output().ok()?;
        Some(CommandOutcome {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// Probe-backed [`CompilerValidator`] for Scala (D-M23-6).
pub struct ScalaToolchainValidator {
    runner: Box<dyn CommandRunner>,
}

impl ScalaToolchainValidator {
    /// Production validator backed by real process spawning.
    pub fn new() -> Self {
        Self::with_runner(Box::new(ProcessCommandRunner))
    }

    /// Injection seam for the T4 tests (scripted runners, no real spawns).
    pub fn with_runner(runner: Box<dyn CommandRunner>) -> Self {
        Self { runner }
    }

    /// First usable candidate and its version banner line. T4 red skeleton:
    /// behavior lands in the green step.
    fn first_usable_toolchain(&self) -> Option<(&'static str, String)> {
        let _ = &self.runner;
        None
    }
}

impl Default for ScalaToolchainValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerValidator for ScalaToolchainValidator {
    /// D-M23-12: without a usable toolchain this must be an honest `Err`
    /// naming the decision with no compile spawn; with one it invokes
    /// `<binary> <compiler_args…> <source_files…>` and maps failure to
    /// `Err` carrying the compiler output. T4 red skeleton: always defers.
    fn validate_compile(
        &self,
        _source_files: &[PathBuf],
        _compiler_args: &[String],
    ) -> Result<(), String> {
        Err(COMPILE_DEFERRAL.to_string())
    }

    /// Probes `scalac`, `scala-cli`, `sbt` in order (D-M23-6); `Missing`
    /// when none is usable — the offline contract on this machine and CI
    /// (I7). T4 red skeleton: trait-default `Unknown` still in place.
    fn toolchain_status(&self) -> ToolchainStatus {
        ToolchainStatus::Unknown
    }
}

#[cfg(test)]
mod tests;
