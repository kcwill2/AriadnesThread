//! [`ScalaDetector`] - classifies Scala-ecosystem files (plan M19.1).
//!
//! Mirrors `ariadne_repo::FileKind::from_path` case-insensitivity: the
//! extension test ignores ASCII case. Basename rules (build definitions)
//! are checked BEFORE extension rules, because `build.sc` shares the `.sc`
//! extension with Ammonite scripts yet is a build definition.

use ariadne_lang::{LanguageDetector, SourceFileKind};
use std::path::Path;

/// Classifies Scala sources, Ammonite scripts, and sbt/Maven build
/// definitions into [`SourceFileKind`] roles.
pub struct ScalaDetector;

impl LanguageDetector for ScalaDetector {
    fn name(&self) -> &str {
        "scala"
    }

    fn detect(&self, path: &Path) -> SourceFileKind {
        if is_build_definition(path) {
            return SourceFileKind::BuildDefinition;
        }
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("scala") => SourceFileKind::Source,
            Some(ext) if ext.eq_ignore_ascii_case("sc") => SourceFileKind::Script,
            _ => SourceFileKind::Other,
        }
    }
}

/// Basename rules run before the extension rules: `build.sc` shares the
/// `.sc` extension with Ammonite scripts yet is an Mill build definition.
fn is_build_definition(path: &Path) -> bool {
    let Some(basename) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    basename.eq_ignore_ascii_case("build.sbt")
        || basename.eq_ignore_ascii_case("build.sc")
        || basename.eq_ignore_ascii_case("pom.xml")
        || (basename.eq_ignore_ascii_case("build.properties")
            && parent_dir_is_project(path))
}

/// sbt's own build reads `project/build.properties` (the launcher
/// version); a `build.properties` anywhere else carries no build meaning.
fn parent_dir_is_project(path: &Path) -> bool {
    path.parent()
        .and_then(|parent| parent.file_name())
        .and_then(|dir| dir.to_str())
        .is_some_and(|dir| dir.eq_ignore_ascii_case("project"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detector() -> ScalaDetector {
        ScalaDetector
    }

    #[test]
    fn detector_name_is_scala() {
        assert_eq!(detector().name(), "scala");
    }

    #[test]
    fn scala_extensions_become_sources() {
        assert_eq!(
            detector().detect(Path::new("src/main/scala/com/example/Main.scala")),
            SourceFileKind::Source
        );
        assert_eq!(
            detector().detect(Path::new("dir/MAIN.SCALA")),
            SourceFileKind::Source,
            "extension case is ignored, mirroring FileKind::from_path"
        );
    }

    #[test]
    fn sc_extension_becomes_script() {
        assert_eq!(
            detector().detect(Path::new("scratch/notes.sc")),
            SourceFileKind::Script
        );
    }

    #[test]
    fn build_definition_basenames_beat_the_script_rule() {
        assert_eq!(
            detector().detect(Path::new("build.sc")),
            SourceFileKind::BuildDefinition,
            "build.sc must be checked before the .sc extension rule"
        );
        assert_eq!(
            detector().detect(Path::new("sub/build.sbt")),
            SourceFileKind::BuildDefinition
        );
        assert_eq!(
            detector().detect(Path::new("pom.xml")),
            SourceFileKind::BuildDefinition
        );
    }

    #[test]
    fn build_properties_needs_the_project_parent() {
        assert_eq!(
            detector().detect(Path::new("project/build.properties")),
            SourceFileKind::BuildDefinition
        );
        assert_eq!(
            detector().detect(Path::new("sub/build.properties")),
            SourceFileKind::Other,
            "only the sbt `project/` directory's build.properties is a build definition"
        );
    }

    #[test]
    fn foreign_and_unknown_files_are_other() {
        for path in ["Main.java", "main.kt", "main.cpp", "matrix.h", "notes.txt"] {
            assert_eq!(
                detector().detect(Path::new(path)),
                SourceFileKind::Other,
                "{path} is not a Scala-ecosystem file"
            );
        }
    }
}
