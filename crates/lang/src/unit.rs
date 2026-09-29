//! [`UnitShaper`] — the build-stage plugin hook (plan §3.3).
//!
//! A language plugin exposes a shaper through `LanguagePlugin::shaper()`;
//! `None` means the pipeline keeps its built-in CMake/VS/compile_commands
//! shaper. `UnitDescriptor` itself lives in `ariadne-lang-api`.

use ariadne_lang_api::unit::UnitDescriptor;
use ariadne_repo::FileRecord;
use std::path::Path;

/// Decides which discovered files become analysis units, and in which
/// build context — the language-specific half of build discovery.
pub trait UnitShaper: Send + Sync {
    /// Build/manifest files this language recognises:
    /// `["CMakeLists.txt", ...]` or `["build.sbt", "build.sc", "pom.xml"]`.
    fn build_markers(&self) -> Vec<&'static str>;

    /// Compiler binary name used for context/measurements; may legitimately
    /// be absent (`ToolchainStatus::Missing`).
    fn default_compiler(&self) -> &str;

    /// Language-specific: which discovered files become analysis units, and
    /// their context.
    fn shape_units(&self, repo_root: &Path, files: &[FileRecord]) -> Vec<UnitDescriptor>;
}
