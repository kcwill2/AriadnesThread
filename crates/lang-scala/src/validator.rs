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

use ariadne_lang::{CompilerValidator, PluginError, ToolchainStatus};
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

    /// First usable candidate and its version banner line. A candidate is
    /// usable when the probe process could be spawned **and** exited
    /// successfully; unresolvable or failing candidates fall through to the
    /// next entry of [`TOOLCHAIN_PROBE_ORDER`].
    fn first_usable_toolchain(&self) -> Option<(&'static str, String)> {
        for candidate in TOOLCHAIN_PROBE_ORDER {
            if let Some(outcome) = self.runner.run(candidate, VERSION_PROBE_ARGS) {
                if outcome.success {
                    return Some((candidate, version_banner(&outcome.stdout, candidate)));
                }
            }
        }
        None
    }
}

impl Default for ScalaToolchainValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerValidator for ScalaToolchainValidator {
    /// D-M23-12: without a usable toolchain this is an honest `Err` naming
    /// the decision (kind [`PluginError::ToolchainUnavailable`]), with only
    /// the `--version` probe walk spawned; with one it invokes
    /// `<binary> <compiler_args...> <source_files...>` (the scalac
    /// options-before-sources convention) and maps failure to
    /// [`PluginError::CompileFailure`] carrying the compiler output -- every
    /// rendering byte-stable (AR-24 locks in [`validator::tests`]).
    fn validate_compile(
        &self,
        source_files: &[PathBuf],
        compiler_args: &[String],
    ) -> Result<(), PluginError> {
        let Some((binary, _)) = self.first_usable_toolchain() else {
            return Err(PluginError::ToolchainUnavailable {
                message: COMPILE_DEFERRAL.to_string(),
            });
        };
        let arguments = compile_arguments(compiler_args, source_files);
        let args: Vec<&str> = arguments.iter().map(String::as_str).collect();
        match self.runner.run(binary, &args) {
            Some(outcome) if outcome.success => Ok(()),
            Some(outcome) => Err(failure_error(binary, &outcome)),
            None => Err(PluginError::ToolchainUnavailable {
                message: format!("{binary} became unrunnable between probe and compile"),
            }),
        }
    }

    /// Probes `scalac`, `scala-cli`, `sbt` in order (D-M23-6); `Missing`
    /// when none is usable — the offline contract on this machine and CI
    /// (I7), feeding the existing neutral `Missing ⇒ Skipped` tier-3 mapping.
    fn toolchain_status(&self) -> ToolchainStatus {
        match self.first_usable_toolchain() {
            Some((_, version)) => ToolchainStatus::Available(version),
            None => ToolchainStatus::Missing,
        }
    }
}

/// The first non-empty line of the version output, falling back to the
/// probed binary name — the `probe_compiler_status` banner shape.
fn version_banner(stdout: &str, binary: &str) -> String {
    stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(binary)
        .to_string()
}

/// The documented invocation shape: `[compiler_args…, source_files…]`.
fn compile_arguments(compiler_args: &[String], source_files: &[PathBuf]) -> Vec<String> {
    let mut arguments: Vec<String> = compiler_args.to_vec();
    arguments.extend(source_files.iter().map(|file| file.display().to_string()));
    arguments
}

/// A failed compile maps to [`PluginError::CompileFailure`] carrying the
/// binary and the compiler's own output (stderr first, then stdout) in the
/// exact pre-typed-seam wording (byte-locked in [`validator::tests`]).
fn failure_error(binary: &str, outcome: &CommandOutcome) -> PluginError {
    let stderr = outcome.stderr.trim();
    let stdout = outcome.stdout.trim();
    let detail = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        "no compiler output"
    };
    PluginError::CompileFailure {
        message: format!("{binary} exited unsuccessfully: {detail}"),
    }
}

#[cfg(test)]
mod tests;
