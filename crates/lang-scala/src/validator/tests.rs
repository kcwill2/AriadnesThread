//! T4 locks (kickoff §5-T4, D-M23-6 + D-M23-12): probe order,
//! Missing/Available semantics, and the deferred-compile contract — all
//! through the injected [`CommandRunner`] seam, never spawning real
//! processes. The one real-runner test only proves the unspawnable-binary
//! path (`None`), which spawns nothing.

use super::{
    CommandOutcome, CommandRunner, ProcessCommandRunner, ScalaToolchainValidator,
    TOOLCHAIN_PROBE_ORDER,
};
use ariadne_lang::CompilerValidator;
use ariadne_lang::ToolchainStatus;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
struct RecordedCall {
    binary: String,
    args: Vec<String>,
}

impl RecordedCall {
    fn is_version_probe(&self) -> bool {
        self.args == ["--version"]
    }
}

#[derive(Clone)]
struct ScriptedRunner {
    probes: Arc<HashMap<String, Option<CommandOutcome>>>,
    compile: Arc<Option<CommandOutcome>>,
    calls: Arc<Mutex<Vec<RecordedCall>>>,
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

    fn with_compile(self, outcome: Option<CommandOutcome>) -> Self {
        Self {
            compile: Arc::new(outcome),
            ..self
        }
    }

    fn into_validator(self) -> ScalaToolchainValidator {
        ScalaToolchainValidator::with_runner(Box::new(self))
    }

    fn calls(&self) -> Vec<RecordedCall> {
        self.calls.lock().unwrap().clone()
    }

    fn compile_calls(&self) -> Vec<RecordedCall> {
        self.calls()
            .into_iter()
            .filter(|call| !call.is_version_probe())
            .collect()
    }
}

fn is_version_probe(args: &[&str]) -> bool {
    args.len() == 1 && args[0] == "--version"
}

impl CommandRunner for ScriptedRunner {
    fn run(&self, binary: &str, args: &[&str]) -> Option<CommandOutcome> {
        self.calls.lock().unwrap().push(RecordedCall {
            binary: binary.to_string(),
            args: args.iter().map(|arg| arg.to_string()).collect(),
        });
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

fn exited_unsuccessfully() -> CommandOutcome {
    CommandOutcome {
        success: false,
        stdout: String::new(),
        stderr: String::new(),
    }
}

fn recorded(binary: &str, args: &[&str]) -> RecordedCall {
    RecordedCall {
        binary: binary.to_string(),
        args: args.iter().map(|arg| arg.to_string()).collect(),
    }
}

#[test]
fn probe_reports_missing_when_no_candidate_resolves() {
    let validator = ScriptedRunner::new().into_validator();
    assert_eq!(validator.toolchain_status(), ToolchainStatus::Missing);
}

#[test]
fn probe_reports_available_with_version_banner_of_first_candidate() {
    let runner = ScriptedRunner::new().with_probe("scalac", Some(usable("scalac 2.13.14\nScala compiler")));
    let observer = runner.clone();
    let status = runner.into_validator().toolchain_status();
    assert_eq!(
        status,
        ToolchainStatus::Available("scalac 2.13.14".to_string()),
        "the first usable candidate answers and its banner's first non-empty line is the carrier"
    );
    assert_eq!(
        observer.calls(),
        vec![recorded("scalac", &["--version"])],
        "the first hit short-circuits the order — scala-cli/sbt are never probed"
    );
}

#[test]
fn probe_order_is_scalac_then_scala_cli_then_sbt() {
    assert_eq!(
        TOOLCHAIN_PROBE_ORDER,
        ["scalac", "scala-cli", "sbt"],
        "D-M23-6 probe order literal"
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
            recorded("scalac", &["--version"]),
            recorded("scala-cli", &["--version"]),
            recorded("sbt", &["--version"]),
        ],
        "every unresolvable candidate is probed, in order, exactly once each"
    );
}

#[test]
fn probe_falls_through_to_scala_cli_when_scalac_is_unresolvable() {
    let runner = ScriptedRunner::new()
        .with_probe("scalac", None)
        .with_probe("scala-cli", Some(usable("Scala CLI version 1.3.8")));
    let observer = runner.clone();
    assert_eq!(
        runner.into_validator().toolchain_status(),
        ToolchainStatus::Available("Scala CLI version 1.3.8".to_string())
    );
    assert_eq!(
        observer.calls(),
        vec![
            recorded("scalac", &["--version"]),
            recorded("scala-cli", &["--version"]),
        ],
        "an unresolvable scalac is skipped, a usable scala-cli ends the walk"
    );
}

#[test]
fn probe_reaches_sbt_when_sbt_is_the_only_usable_candidate() {
    let validator = ScriptedRunner::new()
        .with_probe("scala-cli", None)
        .with_probe("sbt", Some(usable("sbt script version: 1.9.9")))
        .into_validator();
    assert_eq!(
        validator.toolchain_status(),
        ToolchainStatus::Available("sbt script version: 1.9.9".to_string())
    );
}

#[test]
fn probe_reports_missing_when_candidates_run_but_version_probe_fails() {
    let validator = ScriptedRunner::new()
        .with_probe("scalac", Some(exited_unsuccessfully()))
        .with_probe("scala-cli", Some(exited_unsuccessfully()))
        .with_probe("sbt", Some(exited_unsuccessfully()))
        .into_validator();
    assert_eq!(
        validator.toolchain_status(),
        ToolchainStatus::Missing,
        "the process must start AND exit successfully — a spawned-but-failing \
         version probe is not a usable toolchain"
    );
}

#[test]
fn probe_available_falls_back_to_binary_name_on_empty_version_banner() {
    let validator = ScriptedRunner::new()
        .with_probe("scalac", None)
        .with_probe("scala-cli", Some(usable("   \n\n")))
        .into_validator();
    assert_eq!(
        validator.toolchain_status(),
        ToolchainStatus::Available("scala-cli".to_string()),
        "empty banner ⇒ binary-name carrier (probe_compiler_status fallback shape)"
    );
}

#[test]
fn validate_compile_defers_without_toolchain_and_never_compiles() {
    let runner = ScriptedRunner::new();
    let observer = runner.clone();
    let result = runner
        .into_validator()
        .validate_compile(&[PathBuf::from("A.scala")], &["-deprecation".into()]);
    let error = result.expect_err("offline validate_compile must never report a vacuous Ok");
    assert!(
        error.contains("D-M23-12") && error.contains("scalac"),
        "deferral message names the decision and the probe subject: {error}"
    );
    let calls = observer.calls();
    assert_eq!(
        calls
            .iter()
            .filter(|call| call.is_version_probe())
            .count(),
        3,
        "the probe walk ran — the deferral is probe-backed, not unconditional"
    );
    assert!(
        observer.compile_calls().is_empty(),
        "Missing ⇒ no compile process is ever spawned (I7)"
    );
}

#[test]
fn validate_compile_invokes_the_probe_hit_compiler_with_args_then_files() {
    let runner = ScriptedRunner::new()
        .with_probe("scalac", Some(usable("scalac 2.13.14")))
        .with_compile(Some(usable("")));
    let observer = runner.clone();
    let result = runner.into_validator().validate_compile(
        &[
            PathBuf::from("src/main/scala/A.scala"),
            PathBuf::from("B.scala"),
        ],
        &["-deprecation".into()],
    );
    assert_eq!(result, Ok(()));
    assert_eq!(
        observer.compile_calls(),
        vec![recorded(
            "scalac",
            &["-deprecation", "src/main/scala/A.scala", "B.scala"]
        )],
        "documented args shape: <probe-hit binary> <compiler_args…> <source_files…> \
         (scalac options-before-sources convention)"
    );
}

#[test]
fn validate_compile_maps_compiler_failure_output_to_err() {
    let validator = ScriptedRunner::new()
        .with_probe("scalac", Some(usable("scalac 2.13.14")))
        .with_compile(Some(CommandOutcome {
            success: false,
            stdout: String::new(),
            stderr: "error: not found: type Thing".to_string(),
        }))
        .into_validator();
    let error = validator
        .validate_compile(&[PathBuf::from("A.scala")], &[])
        .expect_err("a failing compile must map to Err, never a vacuous Ok");
    assert!(
        error.contains("scalac") && error.contains("not found: type Thing"),
        "Err carries the binary and the compiler's own failure text: {error}"
    );
}

#[test]
fn validate_compile_maps_a_vanished_compiler_to_err() {
    let validator = ScriptedRunner::new()
        .with_probe("scalac", Some(usable("scalac 2.13.14")))
        .with_compile(None)
        .into_validator();
    let error = validator
        .validate_compile(&[PathBuf::from("A.scala")], &[])
        .expect_err("a compiler that vanished between probe and compile must map to Err");
    assert!(
        error.contains("scalac") && !error.contains("D-M23-12"),
        "this is a vanished toolchain, not the offline deferral: {error}"
    );
}

#[test]
fn real_runner_reports_none_for_unspawnable_binary() {
    assert_eq!(
        ProcessCommandRunner
            .run("ariadne-definitely-not-on-path-xyz", &["--version"])
            .is_none(),
        true,
        "the production runner maps spawn failure to None — the Missing half of the probe"
    );
}
