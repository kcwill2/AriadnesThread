use anyhow::Result;
use std::path::Path;

use crate::queue::TuTask;
use ariadne_index::{CppSemanticAnalyzer, TuAnalysis};
use ariadne_lang::LanguageRegistry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerStatus {
    Idle,
    Busy(usize),
    Failed(String),
}

pub trait WorkerBackend: Send + Sync {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis>;
}

/// Transitional (`#[deprecated]`): the pre-M18 hardcoded C++ backend, kept
/// byte-identical so `engine.rs`, `pool` tests and the m13/m16 suites keep
/// passing unchanged until M18.11. New call sites must use
/// [`PluginWorkerBackend`], which resolves the analyzer through the registry.
#[deprecated(
    note = "superseded by PluginWorkerBackend (registry dispatch); removed in M18.11"
)]
pub struct DefaultWorkerBackend;

impl WorkerBackend for DefaultWorkerBackend {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        if let Some(ctx) = &task.context {
            // Canonical closure-merge analysis: identical content to the
            // single-threaded index path so all shard store entries agree.
            let (analysis, _fingerprint) =
                crate::tu_store::analyze_tu_full(&task.tu, repo_root, ctx)?;
            return Ok(analysis);
        }
        let full_path = repo_root.join(&task.tu.source_path);
        let source_code = std::fs::read_to_string(&full_path).unwrap_or_default();
        let analysis = CppSemanticAnalyzer::analyze_source(
            task.tu.file_id.clone(),
            &source_code,
            &task.tu.context_name,
        );
        Ok(analysis)
    }
}

/// Registry-dispatching worker backend (plan M18.2, replaces
/// `DefaultWorkerBackend`): the analyzer is resolved from
/// [`LanguageRegistry`] by the unit's language. Dispatch mechanism:
/// `registry.detect_language(&task.tu.source_path)` — the extension →
/// `FileKind` mapping lives in `ariadne-repo` (no extension literals here)
/// and each plugin's detector must claim the path.
///
/// Byte-for-byte equal to the deprecated `DefaultWorkerBackend` for tasks
/// whose language maps to a registered plugin: the body-only path calls the
/// plugin's `SemanticAnalyzer` (for C++ the three-line adapter over the same
/// `CppSemanticAnalyzer::analyze_source`), and the closure path runs the
/// same canonical `tu_store` merge (plugin-driven closure invalidation is
/// contributed in M18.10 per plan §M18). An unhandled language is reported
/// as an error instead of being silently parsed as C++ (plan §3.3 fact 3).
pub struct PluginWorkerBackend<'a> {
    pub registry: &'a LanguageRegistry,
}

impl<'a> PluginWorkerBackend<'a> {
    pub fn new(registry: &'a LanguageRegistry) -> Self {
        Self { registry }
    }
}

impl WorkerBackend for PluginWorkerBackend<'_> {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        let language = self
            .registry
            .detect_language(&task.tu.source_path)
            .ok_or_else(|| {
                anyhow::anyhow!("unhandled language for {:?}", task.tu.source_path)
            })?;
        if let Some(ctx) = &task.context {
            let (analysis, _fingerprint) =
                crate::tu_store::analyze_tu_full(&task.tu, repo_root, ctx)?;
            return Ok(analysis);
        }
        let plugin = self.registry.plugin_for_id(language).ok_or_else(|| {
            anyhow::anyhow!("no plugin registered for language {language:?}")
        })?;
        let full_path = repo_root.join(&task.tu.source_path);
        let source_code = std::fs::read_to_string(&full_path).unwrap_or_default();
        Ok(plugin.analyzer().analyze(
            task.tu.file_id.clone(),
            &source_code,
            &task.tu.context_name,
        ))
    }
}

/// C++ clang-worker sidecar backend.
///
/// TODO(M18.11): relocate verbatim to `ariadne-lang-cpp` (`src/backend.rs`).
/// The move is blocked today by the transitional shim edges:
/// `lang-cpp -> scheduler -> index -> lang-cpp` would be a hard cargo cycle
/// (`ClangWorkerBackend` needs `scheduler::TuTask`/`WorkerBackend` as real
/// deps, and `scheduler -> index` is permanent). Scheduler must not depend on
/// lang-cpp, so this stays until the `index -> lang-cpp` shim dep is removed.
pub struct ClangWorkerBackend;

impl ClangWorkerBackend {
    fn collect_compiler_flags(arguments: &[String]) -> (Vec<String>, Vec<String>) {
        let mut defines = Vec::new();
        let mut include_dirs = Vec::new();
        for arg in arguments {
            if arg.starts_with("-D") {
                defines.push(arg[2..].to_string());
            } else if arg.starts_with("-I") {
                include_dirs.push(arg[2..].to_string());
            }
        }
        (defines, include_dirs)
    }

    fn build_analyze_request(task: &TuTask, full_path: &Path) -> serde_json::Value {
        let (defines, include_dirs) =
            Self::collect_compiler_flags(&task.tu.compile_command.arguments);
        serde_json::json!({
            "action": "analyze",
            "source_file": full_path.to_string_lossy().to_string(),
            "file_id": task.tu.file_id,
            "context_name": task.tu.context_name,
            "compiler": task.tu.compile_command.command.clone(),
            "defines": defines,
            "include_dirs": include_dirs,
            "language_standard": None::<String>
        })
    }

    fn discover_worker_binaries() -> Vec<std::path::PathBuf> {
        let mut candidate_paths = vec![
            std::path::PathBuf::from("clang-worker"),
            std::path::PathBuf::from("target/debug/clang-worker.exe"),
            std::path::PathBuf::from("target/debug/clang-worker"),
        ];
        if let Ok(cur_exe) = std::env::current_exe() {
            if let Some(target_dir) = cur_exe.parent().and_then(|p| p.parent()) {
                candidate_paths.push(target_dir.join("clang-worker.exe"));
                candidate_paths.push(target_dir.join("clang-worker"));
            }
        }
        candidate_paths
    }

    fn spawn_worker_process(req_path: &Path) -> std::io::Result<std::process::Child> {
        // TEST-ONLY SEAM (Pass 1.3 characterization): when a fake worker command is
        // installed in FAKE_WORKER_CMD, it replaces the candidate-binary discovery and
        // the `--file` arg convention, and the `cargo run` fallback is suppressed so
        // tests never compile or execute the real clang-worker. All items in this
        // seam are `#[cfg(test)]`-gated: non-test builds compile exactly as before.
        #[cfg(test)]
        let fake_worker_cmd = fake_worker_cmd_override();

        #[cfg(test)]
        let candidate_paths = if fake_worker_cmd.is_some() {
            Vec::new()
        } else {
            Self::discover_worker_binaries()
        };
        #[cfg(not(test))]
        let candidate_paths = Self::discover_worker_binaries();

        let mut child_res = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "not started",
        ));

        #[cfg(test)]
        if let Some((fake_program, fake_args)) = &fake_worker_cmd {
            child_res = std::process::Command::new(fake_program)
                .args(fake_args)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();
        }

        for bin in candidate_paths {
            let worker_command_output = std::process::Command::new(&bin)
                .arg("--file")
                .arg(req_path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();
            if worker_command_output.is_ok() {
                child_res = worker_command_output;
                break;
            }
        }

        #[cfg(not(test))]
        let cargo_fallback_suppressed = false;
        #[cfg(test)]
        let cargo_fallback_suppressed = fake_worker_cmd.is_some();

        if child_res.is_err() && !cargo_fallback_suppressed {
            // Try cargo run as fallback for dev environments
            child_res = std::process::Command::new("cargo")
                .args(&[
                    "run",
                    "-q",
                    "-p",
                    "clang-worker",
                    "--bin",
                    "clang-worker",
                    "--",
                    "--file",
                ])
                .arg(req_path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();
        }

        child_res
    }

    fn extract_analysis_from_output(output: &std::process::Output) -> Option<TuAnalysis> {
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        for line in stdout_str.lines() {
            if let Ok(resp) = serde_json::from_str::<serde_json::Value>(line) {
                if resp.get("status").and_then(|s| s.as_str()) == Some("success") {
                    if let Some(analysis_json) = resp.get("analysis") {
                        if let Ok(analysis) =
                            serde_json::from_value::<TuAnalysis>(analysis_json.clone())
                        {
                            return Some(analysis);
                        }
                    }
                }
            }
        }
        None
    }
}

impl WorkerBackend for ClangWorkerBackend {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        let full_path = repo_root.join(&task.tu.source_path);
        let req = Self::build_analyze_request(task, &full_path);

        // Use `--file` argument to pass a temp file containing the request to avoid stdin issues with `cargo run`
        let temp_dir = std::env::temp_dir();
        let req_path = temp_dir.join(format!("worker_req_{}.json", task.task_id));
        let _ = std::fs::write(&req_path, req.to_string());

        let analysis = match Self::spawn_worker_process(&req_path) {
            Ok(child) => match child.wait_with_output() {
                Ok(out) if out.status.success() => Self::extract_analysis_from_output(&out),
                _ => None,
            },
            Err(_) => None,
        };

        let _ = std::fs::remove_file(&req_path);

        match analysis {
            Some(analysis) => Ok(analysis),
            // Fallback if execution failed or response invalid
            None => DefaultWorkerBackend.analyze_task(task, repo_root),
        }
    }
}

#[cfg(test)]
static FAKE_WORKER_CMD: std::sync::Mutex<Option<(std::path::PathBuf, Vec<std::ffi::OsString>)>> =
    std::sync::Mutex::new(None);

#[cfg(test)]
fn fake_worker_cmd_override() -> Option<(std::path::PathBuf, Vec<std::ffi::OsString>)> {
    FAKE_WORKER_CMD
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

#[cfg(test)]
mod fake_process_tests {
    //! Characterization: ClangWorkerBackend::execute driven by a fake worker process
    //! (cmd.exe /c type <response file>).
    use super::*;
    use ariadne_build::{CompileCommand, TranslationUnit};
    use ariadne_repo::FileId;
    use std::ffi::OsString;
    use std::sync::{Mutex, MutexGuard};

    const WORKER_MALFORMED_RESPONSE: &str = "{{{ this is not valid json";
    const WORKER_ERROR_STATUS_RESPONSE: &str = r#"{"status":"error","message":"worker boom"}"#;
    const WORKER_SUCCESS_WITHOUT_ANALYSIS_RESPONSE: &str = r#"{"status":"success"}"#;
    const PROBE_SOURCE_CONTENT: &str = "class DiskProbeWorker { public: int probe_field; };";

    /// Serializes fake-worker tests (the override is a process-wide static) and owns
    /// the response tempdir; clearing the override on Drop.
    struct FakeWorkerGuard {
        _serialization_lock: MutexGuard<'static, ()>,
        _response_dir: Option<tempfile::TempDir>,
    }

    static FAKE_WORKER_SERIAL: Mutex<()> = Mutex::new(());

    fn clear_fake_worker_cmd() {
        *FAKE_WORKER_CMD
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }

    impl FakeWorkerGuard {
        fn install(program: std::path::PathBuf, args: Vec<OsString>) -> Self {
            let serialization_lock = FAKE_WORKER_SERIAL
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *FAKE_WORKER_CMD
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some((program, args));
            Self {
                _serialization_lock: serialization_lock,
                _response_dir: None,
            }
        }

        fn with_response_dir(mut self, response_dir: tempfile::TempDir) -> Self {
            self._response_dir = Some(response_dir);
            self
        }
    }

    impl Drop for FakeWorkerGuard {
        fn drop(&mut self) {
            clear_fake_worker_cmd();
        }
    }

    fn comspec_program() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var_os("COMSPEC").unwrap_or_else(|| OsString::from("cmd.exe")),
        )
    }

    fn install_fake_worker_printing_file(response_body: &str) -> FakeWorkerGuard {
        let response_dir = tempfile::tempdir().expect("tempdir for fake worker response file");
        let response_path = response_dir.path().join("fake_worker_response.jsonl");
        std::fs::write(&response_path, response_body).expect("write fake worker response file");
        FakeWorkerGuard::install(
            comspec_program(),
            vec![
                OsString::from("/c"),
                OsString::from("type"),
                response_path.as_os_str().to_os_string(),
            ],
        )
        .with_response_dir(response_dir)
    }

    fn install_fake_worker_exiting_with(exit_code: &str) -> FakeWorkerGuard {
        FakeWorkerGuard::install(
            comspec_program(),
            vec![
                OsString::from("/c"),
                OsString::from("exit"),
                OsString::from("/b"),
                OsString::from(exit_code),
            ],
        )
    }

    fn install_fake_worker_missing_binary() -> FakeWorkerGuard {
        FakeWorkerGuard::install(
            std::path::PathBuf::from("definitely-missing-worker-binary-9f3a2e.exe"),
            Vec::new(),
        )
    }

    fn canned_success_response(analysis: &TuAnalysis) -> String {
        format!(
            "{}\n",
            serde_json::json!({
                "status": "success",
                "analysis": serde_json::to_value(analysis).expect("serializable canned analysis"),
            })
        )
    }

    fn probe_task(task_id: usize, source_rel: &str) -> TuTask {
        TuTask {
            task_id,
            tu: TranslationUnit {
                file_id: FileId::from_relative_path(source_rel),
                source_path: std::path::PathBuf::from(source_rel),
                context_name: "fake-probe".to_string(),
                compile_command: CompileCommand {
                    directory: std::path::PathBuf::new(),
                    file: std::path::PathBuf::from(source_rel),
                    command: None,
                    arguments: Vec::new(),
                    context_name: "fake-probe".to_string(),
                    output: None,
                },
            },
            estimated_cost: 10,
            context: None,
        }
    }

    fn repo_with_probe_source(repo_dir: &tempfile::TempDir, source_rel: &str) {
        let full_source = repo_dir.path().join(source_rel);
        std::fs::create_dir_all(full_source.parent().expect("source parent dir"))
            .expect("create probe source dir");
        std::fs::write(&full_source, PROBE_SOURCE_CONTENT).expect("write probe source");
    }

    #[test]
    fn execute_returns_analysis_from_well_formed_worker_success_line() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_ok.cpp");
        let canned_analysis = TuAnalysis {
            tu_file_id: FileId::from_relative_path("probe_ok.cpp"),
            context_name: "canned-context".to_string(),
            ..Default::default()
        };
        let _fake_worker =
            install_fake_worker_printing_file(&canned_success_response(&canned_analysis));

        let analysis = ClangWorkerBackend
            .analyze_task(&probe_task(9001, "probe_ok.cpp"), repo.path())
            .expect("well-formed success line must decode to Ok, not fall back");
        assert_eq!(analysis, canned_analysis);
    }

    #[test]
    fn execute_falls_back_to_default_analysis_on_malformed_json_line() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_malformed.cpp");
        let _fake_worker =
            install_fake_worker_printing_file(&format!("{WORKER_MALFORMED_RESPONSE}\n"));

        let task = probe_task(9002, "probe_malformed.cpp");
        let analysis = ClangWorkerBackend
            .analyze_task(&task, repo.path())
            .expect("malformed worker JSON must degrade to Ok via in-process fallback");
        assert_eq!(
            analysis,
            DefaultWorkerBackend
                .analyze_task(&task, repo.path())
                .expect("default backend fallback must succeed")
        );
    }

    #[test]
    fn execute_falls_back_when_worker_reports_error_status() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_error.cpp");
        let _fake_worker =
            install_fake_worker_printing_file(&format!("{WORKER_ERROR_STATUS_RESPONSE}\n"));

        let task = probe_task(9003, "probe_error.cpp");
        let analysis = ClangWorkerBackend
            .analyze_task(&task, repo.path())
            .expect("error-status worker JSON must degrade to Ok via in-process fallback");
        assert_eq!(
            analysis,
            DefaultWorkerBackend
                .analyze_task(&task, repo.path())
                .expect("default backend fallback must succeed")
        );
    }

    #[test]
    fn execute_falls_back_when_success_line_omits_analysis_field() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_noanalysis.cpp");
        let _fake_worker = install_fake_worker_printing_file(&format!(
            "{WORKER_SUCCESS_WITHOUT_ANALYSIS_RESPONSE}\n"
        ));

        let task = probe_task(9004, "probe_noanalysis.cpp");
        let analysis = ClangWorkerBackend
            .analyze_task(&task, repo.path())
            .expect("success line without analysis must degrade to Ok via fallback");
        assert_eq!(
            analysis,
            DefaultWorkerBackend
                .analyze_task(&task, repo.path())
                .expect("default backend fallback must succeed")
        );
    }

    #[test]
    fn execute_skips_noise_lines_and_uses_later_success_line() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_mixed.cpp");
        let canned_analysis = TuAnalysis {
            tu_file_id: FileId::from_relative_path("probe_mixed.cpp"),
            context_name: "canned-mixed-context".to_string(),
            ..Default::default()
        };
        let noisy_body = format!(
            "{WORKER_MALFORMED_RESPONSE}\n{WORKER_ERROR_STATUS_RESPONSE}\n{}",
            canned_success_response(&canned_analysis)
        );
        let _fake_worker = install_fake_worker_printing_file(&noisy_body);

        let analysis = ClangWorkerBackend
            .analyze_task(&probe_task(9005, "probe_mixed.cpp"), repo.path())
            .expect("success line after noise must decode to Ok, not fall back");
        assert_eq!(analysis, canned_analysis);
    }

    #[test]
    fn execute_falls_back_when_worker_exits_nonzero() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_exit3.cpp");
        let _fake_worker = install_fake_worker_exiting_with("3");

        let task = probe_task(9006, "probe_exit3.cpp");
        let analysis = ClangWorkerBackend
            .analyze_task(&task, repo.path())
            .expect("nonzero worker exit must degrade to Ok via in-process fallback");
        assert_eq!(
            analysis,
            DefaultWorkerBackend
                .analyze_task(&task, repo.path())
                .expect("default backend fallback must succeed")
        );
    }

    #[test]
    fn execute_falls_back_when_worker_binary_cannot_spawn() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_spawnfail.cpp");
        // Seam note: production would additionally try the `cargo run` fallback here;
        // the cfg(test) seam suppresses it so tests never build/run the real worker.
        let _fake_worker = install_fake_worker_missing_binary();

        let task = probe_task(9007, "probe_spawnfail.cpp");
        let analysis = ClangWorkerBackend
            .analyze_task(&task, repo.path())
            .expect("spawn failure must degrade to Ok via in-process fallback, never Err");
        assert_eq!(
            analysis,
            DefaultWorkerBackend
                .analyze_task(&task, repo.path())
                .expect("default backend fallback must succeed")
        );
    }

    #[test]
    fn execute_removes_request_file_after_completion() {
        let repo = tempfile::tempdir().expect("repo tempdir");
        repo_with_probe_source(&repo, "probe_reqcleanup.cpp");
        let _fake_worker = install_fake_worker_exiting_with("0");

        ClangWorkerBackend
            .analyze_task(&probe_task(9008, "probe_reqcleanup.cpp"), repo.path())
            .expect("zero-exit worker with empty stdout must degrade to Ok via fallback");
        let request_path = std::env::temp_dir().join("worker_req_9008.json");
        assert!(
            !request_path.exists(),
            "worker request file must be removed after execute: {request_path:?}"
        );
    }
}
