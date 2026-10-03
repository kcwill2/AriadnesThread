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

    /// Build-context name inferred from a compilation database's defines
    /// (plan M18.4: the `DEBUG`/`NDEBUG` naming semantics are language
    /// toolchain knowledge; `build` keeps only neutral plumbing). The
    /// default returns a plain, language-neutral name.
    fn context_name_for_defines(&self, defines: &[String]) -> String {
        let _ = defines;
        "Default".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubShaper;
    impl UnitShaper for StubShaper {
        fn build_markers(&self) -> Vec<&'static str> {
            vec![]
        }
        fn default_compiler(&self) -> &str {
            "stubc"
        }
        fn shape_units(&self, _repo_root: &Path, _files: &[FileRecord]) -> Vec<UnitDescriptor> {
            vec![]
        }
    }

    #[test]
    fn context_name_for_defines_default_is_language_neutral() {
        // M18.4 default: no language vocabulary leaks into the neutral trait;
        // a plugin (CppUnitShaper) overrides with its own naming semantics.
        let shaper = StubShaper;
        assert_eq!(shaper.context_name_for_defines(&[]), "Default");
        assert_eq!(
            shaper.context_name_for_defines(&["DEBUG".into(), "NDEBUG".into()]),
            "Default"
        );
    }
}
