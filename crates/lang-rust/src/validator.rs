//! [`RustToolchainValidator`] - the probe-only Rust toolchain validator
//! (rust-language R-9b, shape sec. 3.5, handoff item 2).
//!
//! Replaces the R-3 honest never-dispatched stub (retired with this hop,
//! amendment-1 sec. 9). The shape mirrors
//! `crates/lang-scala/src/validator.rs` 1-for-1 INCLUDING its pub surface
//! ([`CommandOutcome`], [`CommandRunner`], [`ProcessCommandRunner`],
//! [`RustToolchainValidator::with_runner`]) so the R-9b2 suite can construct
//! a scripted runner from OUTSIDE the crate (F6 tier-3 `Skipped` chain) and
//! the in-src probe tests need no real spawns.
//!
//! Probe contract (locked in [`tests`], mirroring D-M23-6):
//! - candidates are probed in the exact order [`TOOLCHAIN_PROBE_ORDER`] =
//!   `cargo`, `rustc` (mission wording order);
//! - first usable candidate => [`ToolchainStatus::Available`] carrying the
//!   first non-empty stdout line (the binary name when the banner is empty);
//! - no usable candidate => [`ToolchainStatus::Missing`] - feeding the
//!   existing neutral `Missing => Skipped` tier-3 mapping in
//!   `crates/validate/src/lib.rs` (consumed, never re-implemented).
//!
//! `validate_compile` per the D-M23-12 class deferral: with no usable
//! toolchain it returns `Err` naming that deferral and spawns nothing beyond
//! the `--version` probe walk (INV-10c); with one it invokes
//! `rustc <compiler_args...> <source_files...>` - `rustc` specifically, not
//! the probe hit, options-before-sources. Single-input honesty: `rustc`
//! takes ONE main input file, so a multi-file invocation is not a crate-wide
//! compile and full include-closure compilation stays deferred (the
//! D-M23-12 class); `TierValidator` never reaches `validate_compile` while
//! the status is `Missing` (INV-10), and `requires_toolchain: false` (A3)
//! means the pipeline never invokes a Rust compile at all.

use ariadne_lang::{CompilerValidator, PluginError, ToolchainStatus};
use std::path::PathBuf;

/// Probe candidates, probed left to right until one is usable (shape
/// sec. 3.5: mission wording order `cargo`, `rustc`).
pub const TOOLCHAIN_PROBE_ORDER: [&str; 2] = ["cargo", "rustc"];

/// Arguments of every probe invocation, mirroring the Scala validator's
/// `--version` shape (INV-10b: probes only, never compile-shaped spawns).
pub(crate) const VERSION_PROBE_ARGS: &[&str] = &["--version"];

/// The compile always runs through `rustc` specifically - the probe order
/// answers toolchain PRESENCE, not which binary compiles (shape sec. 3.5).
const COMPILE_BINARY: &str = "rustc";

/// The single honest deferral message when no toolchain is usable
/// (D-M23-12 class: real compile invocation deferred, same posture as the
/// Scala validator's offline contract).
const COMPILE_DEFERRAL: &str = "Rust compile deferred (D-M23-12 class): no usable Rust toolchain \
    (cargo/rustc) is present; offline path stays Missing and tier 3 reports Skipped";

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
/// invocation unit-testable without spawning real processes (mirrors the
/// Scala seam 1-for-1, pub surface included).
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

/// Probe-backed [`CompilerValidator`] for Rust (shape sec. 3.5).
pub struct RustToolchainValidator {
    runner: Box<dyn CommandRunner>,
}

impl RustToolchainValidator {
    /// Production validator backed by real process spawning.
    pub fn new() -> Self {
        Self::with_runner(Box::new(ProcessCommandRunner))
    }

    /// Injection seam: the in-src probe tests script the walk, and the
    /// R-9b2 suite composes the F6 tier-3 `Skipped` chain from outside
    /// the crate through this exact entry point.
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

impl Default for RustToolchainValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerValidator for RustToolchainValidator {
    /// D-M23-12 class: without a usable toolchain this is an honest `Err`
    /// naming the deferral (kind [`PluginError::ToolchainUnavailable`]) with
    /// only the `--version` probe walk spawned (INV-10c); with one it
    /// invokes `rustc <compiler_args...> <source_files...>` (options-before-
    /// sources; `rustc` specifically per the single-input honesty note in
    /// the module docs) and maps failure to [`PluginError::CompileFailure`]
    /// carrying the compiler output.
    fn validate_compile(
        &self,
        source_files: &[PathBuf],
        compiler_args: &[String],
    ) -> Result<(), PluginError> {
        if self.first_usable_toolchain().is_none() {
            return Err(PluginError::ToolchainUnavailable {
                message: COMPILE_DEFERRAL.to_string(),
            });
        }
        let arguments = compile_arguments(compiler_args, source_files);
        let args: Vec<&str> = arguments.iter().map(String::as_str).collect();
        match self.runner.run(COMPILE_BINARY, &args) {
            Some(outcome) if outcome.success => Ok(()),
            Some(outcome) => Err(failure_error(&outcome)),
            None => Err(PluginError::ToolchainUnavailable {
                message: format!("{COMPILE_BINARY} became unrunnable between probe and compile"),
            }),
        }
    }

    /// Probes `cargo`, `rustc` in order (shape sec. 3.5); `Missing` when
    /// none is usable - never `Unknown` (the INV-10 class: `Missing` is
    /// what the neutral tier-3 mapper turns into `Skipped`).
    fn toolchain_status(&self) -> ToolchainStatus {
        match self.first_usable_toolchain() {
            Some((_, version)) => ToolchainStatus::Available(version),
            None => ToolchainStatus::Missing,
        }
    }
}

/// The first non-empty line of the version output, falling back to the
/// probed binary name - the Scala probe's banner shape.
fn version_banner(stdout: &str, binary: &str) -> String {
    stdout
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(binary)
        .to_string()
}

/// The documented invocation shape: `[compiler_args..., source_files...]`.
fn compile_arguments(compiler_args: &[String], source_files: &[PathBuf]) -> Vec<String> {
    let mut arguments: Vec<String> = compiler_args.to_vec();
    arguments.extend(source_files.iter().map(|file| file.display().to_string()));
    arguments
}

/// A failed compile maps to [`PluginError::CompileFailure`] carrying the
/// binary and the compiler's own output (stderr first, then stdout) in the
/// Scala validator's pre-typed-seam wording.
fn failure_error(outcome: &CommandOutcome) -> PluginError {
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
        message: format!("{COMPILE_BINARY} exited unsuccessfully: {detail}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CommandOutcome, CommandRunner, RustToolchainValidator, TOOLCHAIN_PROBE_ORDER,
    };
    use ariadne_lang::{CompilerValidator, PluginError, ToolchainStatus};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    /// Scripted runner mirroring the Scala validator's test seam: probe
    /// answers per binary plus a compile answer, with every call recorded.
    #[derive(Clone)]
    struct ScriptedRunner {
        probes: Arc<HashMap<String, Option<CommandOutcome>>>,
        compile: Arc<Option<CommandOutcome>>,
        calls: Arc<Mutex<Vec<(String, Vec<String>)>>>,
    }

    impl ScriptedRunner {
        fn new() -> Self {
            Self {
                probes: Arc::new(HashMap::new()),
                compile: Arc::new(None),
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn with_probe(self, binary: &str, outcome: Option<CommandOutcome>) -> Self {
            let mut probes = (*self.probes).clone();
            probes.insert(binary.to_string(), outcome);
            Self {
                probes: Arc::new(probes),
                ..self
            }
        }

        fn into_validator(self) -> RustToolchainValidator {
            RustToolchainValidator::with_runner(Box::new(self))
        }

        fn calls(&self) -> Vec<(String, Vec<String>)> {
            self.calls.lock().unwrap().clone()
        }
    }

    fn is_version_probe(args: &[&str]) -> bool {
        args.len() == 1 && args[0] == "--version"
    }

    impl CommandRunner for ScriptedRunner {
        fn run(&self, binary: &str, args: &[&str]) -> Option<CommandOutcome> {
            self.calls.lock().unwrap().push((
                binary.to_string(),
                args.iter().map(|arg| arg.to_string()).collect(),
            ));
            if is_version_probe(args) {
                self.probes.get(binary).cloned().flatten()
            } else {
                (*self.compile).clone()
            }
        }
    }

    fn usable(banner: &str) -> CommandOutcome {
        CommandOutcome {
            success: true,
            stdout: banner.to_string(),
            stderr: String::new(),
        }
    }

    fn recorded(binary: &str, args: &[&str]) -> (String, Vec<String>) {
        (
            binary.to_string(),
            args.iter().map(|arg| arg.to_string()).collect(),
        )
    }

    #[test]
    fn probe_order_is_cargo_then_rustc() {
        assert_eq!(
            TOOLCHAIN_PROBE_ORDER,
            ["cargo", "rustc"],
            "shape sec. 3.5 probe order literal (mission wording order)"
        );
        let runner = ScriptedRunner::new();
        let observer = runner.clone();
        assert_eq!(
            runner.into_validator().toolchain_status(),
            ToolchainStatus::Missing
        );
        assert_eq!(
            observer.calls(),
            vec![
                recorded("cargo", &["--version"]),
                recorded("rustc", &["--version"]),
            ],
            "every unresolvable candidate is probed, in order, exactly once each"
        );
    }

    /// Positive replacement for the retired stub pin
    /// `toolchain_status_is_missing_never_unknown`: the answer now comes
    /// FROM the probe walk (probe-backed, never unconditional), and the
    /// INV-10 class stands - `Missing`, never the trait's `Unknown` default.
    #[test]
    fn probe_reports_missing_when_no_probe_is_usable() {
        let validator = ScriptedRunner::new().into_validator();
        assert_eq!(
            validator.toolchain_status(),
            ToolchainStatus::Missing,
            "INV-10 class: an all-None probe walk answers Missing - TierValidator \
             maps Missing to a Skipped tier, never a spurious Fail"
        );
    }

    /// INV-10b: every spawn the probe walk performs is a probe-order entry
    /// with the `--version` flag - never a compile-shaped spawn.
    #[test]
    fn every_probe_spawn_is_a_probe_order_entry_with_version_flag() {
        let runner = ScriptedRunner::new();
        let observer = runner.clone();
        runner.into_validator().toolchain_status();
        let calls = observer.calls();
        assert_eq!(
            calls.len(),
            TOOLCHAIN_PROBE_ORDER.len(),
            "exactly one spawn per probe-order entry, nothing else"
        );
        for (index, (binary, args)) in calls.iter().enumerate() {
            assert_eq!(
                binary, TOOLCHAIN_PROBE_ORDER[index],
                "spawn {index} is the {index}-th probe-order entry"
            );
            assert!(
                is_version_probe(&args.iter().map(String::as_str).collect::<Vec<_>>()),
                "INV-10b: every probe spawn carries the --version flag, got: {args:?}"
            );
        }
    }

    /// Positive replacement for the retired stub pin
    /// `validate_compile_errors_naming_the_deferral`: the deferral-message
    /// truth survives HERE (offline-scripted), now probe-backed - the walk
    /// ran and still nothing beyond it was spawned (INV-10c).
    #[test]
    fn validate_compile_offline_errors_naming_the_deferral_spawning_nothing_beyond_the_probe_walk()
    {
        let runner = ScriptedRunner::new();
        let observer = runner.clone();
        let result = runner
            .into_validator()
            .validate_compile(&[PathBuf::from("src/lib.rs")], &["--edition".into(), "2021".into()]);
        let error = result.expect_err("offline validate_compile must never report a vacuous Ok");
        let message = error.to_string();
        assert!(
            message.contains("D-M23-12") && message.contains("cargo") && message.contains("rustc"),
            "the deferral names its class and its probe subjects: {message}"
        );
        assert!(
            matches!(error, PluginError::ToolchainUnavailable { .. }),
            "offline is an absence (Skipped-mappable), not a Fail kind: {error:?}"
        );
        let calls = observer.calls();
        assert_eq!(
            calls,
            vec![
                recorded("cargo", &["--version"]),
                recorded("rustc", &["--version"]),
            ],
            "the probe walk ran (the deferral is probe-backed) and NOTHING else was spawned (INV-10c)"
        );
    }

    #[test]
    fn probe_reports_available_with_version_banner_of_first_usable_probe() {
        let runner = ScriptedRunner::new()
            .with_probe("cargo", Some(usable("cargo 1.80.0 (d0576ea11 2024-07-03)")));
        let observer = runner.clone();
        assert_eq!(
            runner.into_validator().toolchain_status(),
            ToolchainStatus::Available("cargo 1.80.0 (d0576ea11 2024-07-03)".to_string()),
            "the first usable probe answers and its banner's first non-empty line is the carrier"
        );
        assert_eq!(
            observer.calls(),
            vec![recorded("cargo", &["--version"])],
            "the first hit short-circuits the order - rustc is never probed"
        );
    }
}
