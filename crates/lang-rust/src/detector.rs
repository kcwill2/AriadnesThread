//! [`RustDetector`] - classifies Rust-ecosystem files (rust-language R-3).
//!
//! Mirrors `ariadne_repo::FileKind::from_path` case-insensitivity: both the
//! basename test and the extension test ignore ASCII case. The basename rule
//! (Cargo.toml) is checked BEFORE the extension rule, mirroring Scala's
//! `build.sc`-before-`.sc` ordering. The spine `FileKind` gains no `toml`
//! mapping (the extension is shared; mapping it would leak into Other's
//! neighbors); the plugin detector names Cargo.toml, which is exactly what
//! Scala's detector does for `pom.xml`, and `detect_language` remains
//! plugin-authoritative (M24 D-A2).

use ariadne_lang::{LanguageDetector, SourceFileKind};
use std::path::Path;

/// Classifies Rust sources and Cargo build definitions into
/// [`SourceFileKind`] roles.
pub struct RustDetector;

impl LanguageDetector for RustDetector {
    fn name(&self) -> &str {
        "rust"
    }

    fn detect(&self, path: &Path) -> SourceFileKind {
        if is_build_definition(path) {
            return SourceFileKind::BuildDefinition;
        }
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) if ext.eq_ignore_ascii_case("rs") => SourceFileKind::Source,
            _ => SourceFileKind::Other,
        }
    }
}

/// Basename rule runs before the extension rules: every cargo crate and
/// workspace root carries a `Cargo.toml` build manifest (case-folded).
fn is_build_definition(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|basename| basename.eq_ignore_ascii_case("Cargo.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detector() -> RustDetector {
        RustDetector
    }

    #[test]
    fn detector_name_is_rust() {
        assert_eq!(detector().name(), "rust");
    }

    #[test]
    fn rs_extension_becomes_source() {
        assert_eq!(
            detector().detect(Path::new("src/lib.rs")),
            SourceFileKind::Source
        );
        assert_eq!(
            detector().detect(Path::new("dir/MAIN.RS")),
            SourceFileKind::Source,
            "extension case is ignored, mirroring FileKind::from_path"
        );
    }

    #[test]
    fn cargo_toml_basenames_beat_the_extension_rule() {
        assert_eq!(
            detector().detect(Path::new("Cargo.toml")),
            SourceFileKind::BuildDefinition
        );
        assert_eq!(
            detector().detect(Path::new("sub/Cargo.toml")),
            SourceFileKind::BuildDefinition
        );
        assert_eq!(
            detector().detect(Path::new("cargo.toml")),
            SourceFileKind::BuildDefinition,
            "the basename test is case-folded too"
        );
    }

    #[test]
    fn foreign_and_unknown_files_are_other() {
        for path in ["pom.xml", "README.txt", "build.sbt", "main.java"] {
            assert_eq!(
                detector().detect(Path::new(path)),
                SourceFileKind::Other,
                "{path} is not a Rust-ecosystem file"
            );
        }
    }
}
