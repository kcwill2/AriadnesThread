//! The registry as one [`UnitShaper`] over every registered plugin.
//!
//! Repositories routinely mix languages (a Visual Studio C++ tree beside
//! Scala services, built by Bazel with stale sbt files lying around).
//! Picking ONE shaper from root markers dropped every other language's
//! units entirely, so the registry now shapes with all of them: each
//! plugin's shaper claims only its own files, and the descriptors are
//! concatenated in registration order. A single-language repo shapes
//! exactly as before because the other shapers claim nothing.

use super::LanguageRegistry;
use crate::unit::UnitShaper;
use ariadne_lang_api::identity::LanguageId;
use ariadne_lang_api::unit::UnitDescriptor;
use ariadne_repo::FileRecord;
use std::path::Path;

impl LanguageRegistry {
    /// Shapers of the registered plugins that provide one, in
    /// registration order.
    fn shapers(&self) -> impl Iterator<Item = &dyn UnitShaper> {
        self.plugins.iter().filter_map(|plugin| plugin.shaper())
    }

    fn first_shaper(&self) -> Option<&dyn UnitShaper> {
        self.shapers().next()
    }
}

impl UnitShaper for LanguageRegistry {
    fn build_markers(&self) -> Vec<&'static str> {
        self.shapers()
            .flat_map(|shaper| shaper.build_markers())
            .collect()
    }

    /// The first registered shaper's compiler: the fallback for contexts
    /// whose language no shaper-providing plugin owns.
    fn default_compiler(&self) -> &str {
        self.first_shaper()
            .map_or("", |shaper| shaper.default_compiler())
    }

    fn compiler_for(&self, language: &LanguageId) -> &str {
        self.plugins
            .iter()
            .filter(|plugin| &plugin.id() == language)
            .find_map(|plugin| plugin.shaper())
            .map_or_else(
                || self.default_compiler(),
                |shaper| shaper.compiler_for(language),
            )
    }

    fn shape_units(&self, repo_root: &Path, files: &[FileRecord]) -> Vec<UnitDescriptor> {
        self.shapers()
            .flat_map(|shaper| shaper.shape_units(repo_root, files))
            .collect()
    }

    fn discovery_notes(&self, repo_root: &Path, files: &[FileRecord]) -> Vec<String> {
        self.shapers()
            .flat_map(|shaper| shaper.discovery_notes(repo_root, files))
            .collect()
    }

    /// Compilation databases are C/C++ artifacts named by the first
    /// registered shaper, exactly as the former first-wins fallback did.
    fn context_name_for_defines(&self, defines: &[String]) -> String {
        match self.first_shaper() {
            Some(shaper) => shaper.context_name_for_defines(defines),
            None => "Default".to_string(),
        }
    }
}
