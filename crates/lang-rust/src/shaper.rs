//! [`RustUnitShaper`] - the Rust half of build discovery (rust-language R-3).
//!
//! Units are exactly the discovered `FileKind::RustSource` files whose path
//! carries a lowercase `src` component (the cargo source root, any crate of
//! a workspace). `tests`, `examples`, and `benches` path components are the
//! exclusion tags: cargo builds those roots separately from the crate's
//! library/binary code, so they never become analysis units (the
//! exclusion-tag idiom, mirroring Scala's `src/test/scala` precedent - a
//! pathological path carrying both roots is excluded, the exclusion wins).
//!
//! The context name is the nearest ancestor directory holding a
//! `Cargo.toml` (walk from the file's dir toward `repo_root`); the workspace
//! root itself, or no manifest at all, is named `root` - Scala's
//! `ROOT_CONTEXT` precedent, keeping assertions independent of the
//! machine-specific tempdir name.
//!
//! Cargo invocation text is out of scope at this hop: `build_spec` stays
//! empty and `default_compiler()` hands `"rustc"` to `build`'s shaper
//! plumbing (`crates/build/src/discovery.rs`), which fills
//! `CompilationContext.compiler` from it.

use ariadne_lang::{LanguageId, UnitDescriptor, UnitKind, UnitShaper};
use ariadne_repo::{FileKind, FileRecord};
use std::path::Path;

/// Lowercase path component of the cargo source root.
const SOURCE_ROOT_COMPONENT: &str = "src";

/// Path components that exclude a source file from unit shaping: cargo
/// compiles these roots separately from the crate's library/binary code.
const EXCLUDED_ROOT_COMPONENTS: [&str; 3] = ["tests", "examples", "benches"];

/// Root-context name used when no non-root `Cargo.toml` ancestor exists:
/// `"root"` keeps test assertions independent of the machine-specific
/// tempdir name (Scala's `ROOT_CONTEXT` precedent).
const ROOT_CONTEXT: &str = "root";

/// Path components lowercased, so root matching is case-insensitive on
/// mixed-case directories (Scala's `path_contains_components` precedent).
fn lowercase_components(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
        .collect()
}

fn is_rust_source_unit(record: &FileRecord) -> bool {
    if record.kind != FileKind::RustSource {
        return false;
    }
    let components = lowercase_components(&record.path);
    !components
        .iter()
        .any(|component| EXCLUDED_ROOT_COMPONENTS.contains(&component.as_str()))
        && components
            .iter()
            .any(|component| component == SOURCE_ROOT_COMPONENT)
}

fn has_cargo_manifest(dir: &Path) -> bool {
    dir.join("Cargo.toml").is_file()
}

/// Walks from the file's directory toward the repo root; the first NON-ROOT
/// directory holding a `Cargo.toml` names the context (every cargo crate and
/// workspace root carries one, and each crate's own manifest is nearer than
/// the workspace's). The workspace root itself, or no manifest at all, is
/// named [`ROOT_CONTEXT`].
fn cargo_context_name(file_dir: &Path, repo_root: &Path) -> String {
    let mut dir = file_dir;
    while dir.starts_with(repo_root) {
        if dir != repo_root && has_cargo_manifest(dir) {
            if let Some(name) = dir.file_name().and_then(|name| name.to_str()) {
                return name.to_string();
            }
        }
        dir = match dir.parent() {
            Some(parent) => parent,
            None => break,
        };
    }
    ROOT_CONTEXT.to_string()
}

fn descriptor_for(record: &FileRecord, context_name: String) -> UnitDescriptor {
    UnitDescriptor {
        file_id: record.id,
        language: LanguageId::Rust,
        unit_kind: UnitKind::SourceFile,
        context_name,
        aggregation_target: None,
        build_spec: vec![],
        defines: vec![],
        include_dirs: vec![],
    }
}

/// Shapes every discovered Rust source under a `src` root into one
/// source-file unit of its nearest-crate context.
pub struct RustUnitShaper;

impl UnitShaper for RustUnitShaper {
    /// Shape sec. 3.5 verbatim: the file that marks a Rust project root.
    fn build_markers(&self) -> Vec<&'static str> {
        vec!["Cargo.toml"]
    }

    /// Recorded for `build`'s shaper plumbing, which fills
    /// `CompilationContext.compiler` from it; never invoked while the crate
    /// is unregistered.
    fn default_compiler(&self) -> &str {
        "rustc"
    }

    /// One unit per `src`-rooted Rust source, in input order; the crate
    /// whose `Cargo.toml` is the nearest non-root ancestor names the context.
    fn shape_units(&self, repo_root: &Path, files: &[FileRecord]) -> Vec<UnitDescriptor> {
        files
            .iter()
            .filter(|record| is_rust_source_unit(record))
            .map(|record| {
                let file_dir = record.canonical_path.parent().unwrap_or(repo_root);
                descriptor_for(record, cargo_context_name(file_dir, repo_root))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_repo::{FileId, FileOwnership, OptimizationPolicy, SourceLanguage};
    use std::fs;
    use std::path::PathBuf;

    fn record(root: &Path, rel: &str, kind: FileKind) -> FileRecord {
        let rel_path = PathBuf::from(rel);
        FileRecord {
            id: FileId::from_relative_path(rel),
            path: rel_path.clone(),
            canonical_path: root.join(&rel_path),
            kind,
            size: 0,
            content_hash: String::new(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
            language: SourceLanguage::Rust,
        }
    }

    fn rust_record(root: &Path, rel: &str) -> FileRecord {
        record(root, rel, FileKind::RustSource)
    }

    fn create_tree(root: &Path, rels: &[&str]) {
        for rel in rels {
            let full = root.join(rel);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, "").unwrap();
        }
    }

    #[test]
    fn build_markers_match_the_plan_literal() {
        assert_eq!(RustUnitShaper.build_markers(), vec!["Cargo.toml"]);
    }

    #[test]
    fn default_compiler_is_rustc() {
        assert_eq!(RustUnitShaper.default_compiler(), "rustc");
    }

    #[test]
    fn shape_units_yields_one_unit_per_src_file_in_the_workspace_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(
            root,
            &[
                "Cargo.toml",
                "src/lib.rs",
                "src/models/greeting.rs",
                "README.md",
            ],
        );
        let files = vec![
            rust_record(root, "src/lib.rs"),
            rust_record(root, "src/models/greeting.rs"),
            record(root, "README.md", FileKind::Other),
        ];

        let units = RustUnitShaper.shape_units(root, &files);

        assert_eq!(units.len(), 2, "one unit per src-rooted Rust source");
        assert_eq!(units[0].file_id, FileId::from_relative_path("src/lib.rs"));
        assert_eq!(
            units[1].file_id,
            FileId::from_relative_path("src/models/greeting.rs"),
            "discovery yields path-sorted records; shapers must stay order-preserving"
        );
        for unit in &units {
            assert_eq!(unit.language, LanguageId::Rust);
            assert_eq!(unit.unit_kind, UnitKind::SourceFile);
            assert_eq!(unit.context_name, "root", "manifest only at the workspace root");
            assert_eq!(unit.aggregation_target, None);
            assert!(
                unit.build_spec.is_empty()
                    && unit.defines.is_empty()
                    && unit.include_dirs.is_empty(),
                "cargo invocation text is out of scope at this hop"
            );
        }
    }

    #[test]
    fn shape_units_never_shapes_the_tests_examples_or_benches_roots() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(
            root,
            &[
                "Cargo.toml",
                "tests/integration.rs",
                "examples/demo.rs",
                "benches/throughput.rs",
                "src/tests/helper.rs",
            ],
        );
        let files = vec![
            rust_record(root, "tests/integration.rs"),
            rust_record(root, "examples/demo.rs"),
            rust_record(root, "benches/throughput.rs"),
            rust_record(root, "src/tests/helper.rs"),
        ];

        assert!(
            RustUnitShaper.shape_units(root, &files).is_empty(),
            "tests/examples/benches are exclusion tags; the exclusion wins even \
             for the pathological src/tests/ path carrying both roots"
        );
    }

    #[test]
    fn shape_units_names_the_crate_context_from_its_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(
            root,
            &["Cargo.toml", "client/Cargo.toml", "client/src/lib.rs"],
        );
        let files = vec![rust_record(root, "client/src/lib.rs")];

        let units = RustUnitShaper.shape_units(root, &files);

        assert_eq!(units.len(), 1);
        assert_eq!(units[0].context_name, "client");
        assert_eq!(
            units[0].file_id,
            FileId::from_relative_path("client/src/lib.rs")
        );
    }

    #[test]
    fn shape_units_without_any_cargo_toml_still_names_the_root_context() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(root, &["src/app.rs"]);
        let files = vec![rust_record(root, "src/app.rs")];

        let units = RustUnitShaper.shape_units(root, &files);

        assert_eq!(units.len(), 1);
        assert_eq!(units[0].context_name, "root");
    }

    #[test]
    fn shape_units_of_empty_input_yields_nothing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(RustUnitShaper.shape_units(dir.path(), &[]).is_empty());
    }
}
