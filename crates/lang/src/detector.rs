//! File-kind detection trait for language plugins.

use std::path::Path;

/// The role a file plays within its language's build model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceFileKind {
    /// A compilable source unit (`.cpp`, `.scala`, `.rs`, …).
    Source,
    /// A header or interface file included by other sources (`.h`, `.hpp`, …).
    Header,
    /// A build definition file (`build.sbt`, `CMakeLists.txt`, …).
    BuildDefinition,
    /// A script or REPL file (`.sc`, `.sh`, …).
    Script,
    /// Anything the plugin does not recognise.
    Other,
}

impl SourceFileKind {
    /// Returns `true` for file kinds that should be semantically analysed.
    pub fn is_analyzable(self) -> bool {
        matches!(self, Self::Source | Self::Header)
    }
}

/// Classifies files by extension into [`SourceFileKind`] variants.
///
/// Implement this trait for each language plugin. The registry calls
/// [`detect`] on every registered plugin and uses the first non-`Other` result.
pub trait LanguageDetector: Send + Sync {
    /// Returns the plugin's human-readable language name (e.g. `"c++"`, `"scala"`).
    fn name(&self) -> &str;

    /// Classifies `path` by its extension (and optionally its content).
    fn detect(&self, path: &Path) -> SourceFileKind;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_file_kind_is_analyzable() {
        assert!(SourceFileKind::Source.is_analyzable());
        assert!(SourceFileKind::Header.is_analyzable());
        assert!(!SourceFileKind::BuildDefinition.is_analyzable());
        assert!(!SourceFileKind::Script.is_analyzable());
        assert!(!SourceFileKind::Other.is_analyzable());
    }
}
