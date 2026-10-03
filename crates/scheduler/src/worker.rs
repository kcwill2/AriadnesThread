use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

use crate::queue::TuTask;
use ariadne_index::TuAnalysis;
use ariadne_lang::LanguageRegistry;

/// Registry-backed backends must outlive the tokio worker tasks that hold
/// them (`WorkerPool` requires `B: 'static`), so the shared-ownership variant
/// of [`PluginWorkerBackend`] carries the registry as an [`Arc`] (plan M18.3
/// cli wiring). Dispatch, byte-parity, and the unhandled-language error are
/// identical to the borrowed variant — it just delegates.
pub struct SharedPluginWorkerBackend {
    registry: Arc<LanguageRegistry>,
}

impl SharedPluginWorkerBackend {
    pub fn new(registry: Arc<LanguageRegistry>) -> Self {
        Self { registry }
    }

    pub fn registry(&self) -> &LanguageRegistry {
        &self.registry
    }
}

impl WorkerBackend for SharedPluginWorkerBackend {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        PluginWorkerBackend::new(&self.registry).analyze_task(task, repo_root)
    }
}

/// `WorkerPool` stores backends as `Arc<B>`, and the registry wiring hands it
/// an `Arc<dyn WorkerBackend>` (M18.3): dispatch through the `Arc` to the
/// wrapped backend.
impl<T: WorkerBackend + ?Sized> WorkerBackend for Arc<T> {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis> {
        T::analyze_task(self, task, repo_root)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerStatus {
    Idle,
    Busy(usize),
    Failed(String),
}

pub trait WorkerBackend: Send + Sync {
    fn analyze_task(&self, task: &TuTask, repo_root: &Path) -> Result<TuAnalysis>;
}

/// Registry-dispatching worker backend (plan M18.2; since M18.11 the ONLY
/// in-process backend): the analyzer and the closure walk are resolved from
/// [`LanguageRegistry`] by the unit's language. Dispatch mechanism:
/// `registry.detect_language(&task.tu.source_path)` — the extension →
/// `FileKind` mapping lives in `ariadne-repo` (no extension literals here)
/// and each plugin's detector must claim the path.
///
/// Byte-for-byte equal to the pre-M18 hardcoded-C++ in-process backend for
/// tasks whose language maps to a registered plugin (locked by
/// `m18_cpp_plugin_parity_test`): the body-only path calls the plugin's
/// `SemanticAnalyzer` (for C++ the three-line adapter over the same
/// analyzer), and a task carrying a store context runs the canonical
/// `tu_store` closure merge through the plugin's resolver/analyzer. An
/// unhandled language is reported as an error instead of being silently
/// parsed (plan §3.3 fact 3).
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
            .ok_or_else(|| anyhow::anyhow!("unhandled language for {:?}", task.tu.source_path))?;
        let plugin = self
            .registry
            .plugin_for_id(language.clone())
            .ok_or_else(|| anyhow::anyhow!("no plugin registered for language {language:?}"))?;
        if let Some(ctx) = &task.context {
            // Canonical closure merge through the unit's own plugin — the
            // C++ bytes match the pre-M18 hardcoded merge exactly.
            let (analysis, _fingerprint) =
                crate::tu_store::analyze_tu_full_with_plugin(&task.tu, repo_root, ctx, plugin)?;
            return Ok(analysis);
        }
        let full_path = repo_root.join(&task.tu.source_path);
        let source_code = std::fs::read_to_string(&full_path).unwrap_or_default();
        Ok(plugin
            .analyzer()
            .analyze(task.tu.file_id.clone(), &source_code, &task.tu.context_name))
    }
}

// The pre-M18 clang-worker sidecar backend relocated to
// `ariadne-lang-cpp` (`ariadne_lang_cpp::ClangWorkerBackend`) in M18.11, as
// sanctioned by the M18.1–M18.2 execution note (b): the relocation needs
// `lang-cpp -> scheduler` as a normal dependency, which only becomes acyclic
// once the `index`/`graph -> lang-cpp` transitional shim deps are removed.
