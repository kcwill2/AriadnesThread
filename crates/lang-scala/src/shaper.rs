//! [`ScalaUnitShaper`] - the Scala half of build discovery (plan M19.2).
//!
//! Units are exactly the discovered `FileKind::ScalaSource` files that sit
//! under a consecutive `src/main/scala` root (any sbt module). The sbt test
//! root (`src/test/scala`) is the exclusion tag: a file's path never has
//! both roots, so the main-root requirement already drops it - the
//! explicit test-root check documents that intent and guards against
//! pathological paths containing both segments.
//!
//! Bazel repos override that rule: a file owned by a Bazel package is
//! named after the package's repo-relative path and leftover `*.sbt` files
//! are ignored (rules and test-source exclusion in [`crate::bazel`]).
//!
//! sbt invocation text is M23 toolchain work: `build_spec` stays empty and
//! `default_compiler()` hands `"scalac"` to `build`'s shaper plumbing
//! (`crates/build/src/discovery.rs`), which fills
//! `CompilationContext.compiler` from it.

use crate::bazel;
use ariadne_lang::{LanguageId, UnitDescriptor, UnitKind, UnitShaper};
use ariadne_repo::{FileKind, FileRecord};
use std::fs;
use std::path::Path;

/// Consecutive lowercase path components of the sbt main source root.
const MAIN_SOURCE_ROOT: [&str; 3] = ["src", "main", "scala"];

/// Consecutive lowercase path components of the sbt test source root.
const TEST_SOURCE_ROOT: [&str; 3] = ["src", "test", "scala"];

/// Root-context name used for the sbt root project (and as the fallback
/// when no module `*.sbt` ancestor exists): `"root"` keeps test assertions
/// independent of the machine-specific tempdir name.
pub(crate) const ROOT_CONTEXT: &str = "root";

fn path_contains_components(path: &Path, pattern: &[&str]) -> bool {
    let components: Vec<String> = path
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
        .collect();
    components.windows(pattern.len()).any(|window| {
        window
            .iter()
            .enumerate()
            .all(|(index, component)| *component == pattern[index])
    })
}

fn is_scala_main_source(record: &FileRecord) -> bool {
    record.kind == FileKind::ScalaSource
        && !path_contains_components(&record.path, &TEST_SOURCE_ROOT)
        && path_contains_components(&record.path, &MAIN_SOURCE_ROOT)
}

fn has_sbt_file(dir: &Path) -> bool {
    fs::read_dir(dir).map_or(false, |entries| {
        entries.filter_map(|entry| entry.ok()).any(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("sbt"))
        })
    })
}

/// Walks from the file's directory toward the repo root; the first
/// directory holding any `*.sbt` file names the context. The sbt root
/// project (marker found at the repo root, or no marker at all) is named
/// [`ROOT_CONTEXT`].
fn sbt_context_name(file_dir: &Path, repo_root: &Path) -> String {
    let mut dir = file_dir;
    while dir.starts_with(repo_root) {
        if dir != repo_root && has_sbt_file(dir) {
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
        language: LanguageId::Scala,
        unit_kind: UnitKind::SourceFile,
        context_name,
        aggregation_target: None,
        build_spec: vec![],
        defines: vec![],
        include_dirs: vec![],
    }
}

/// Shapes every discovered Scala main-root source into one source-file
/// unit of its sbt module context.
pub struct ScalaUnitShaper;

impl UnitShaper for ScalaUnitShaper {
    /// Plan:383 verbatim: the files that mark a Scala project root.
    fn build_markers(&self) -> Vec<&'static str> {
        vec!["build.sbt", "build.sc", "pom.xml"]
    }

    /// Recorded for `build`'s shaper plumbing, which fills
    /// `CompilationContext.compiler` from it; never invoked at kickoff.
    fn default_compiler(&self) -> &str {
        "scalac"
    }

    /// One unit per Scala source, in input order. A file owned by a Bazel
    /// package (see [`crate::bazel`]) is named after that package and test
    /// sources are dropped; in a Bazel workspace a file no package owns is
    /// not built, so it is no unit. Everything else keeps the sbt rule:
    /// main-root sources only, named by the nearest `*.sbt` ancestor.
    fn shape_units(&self, repo_root: &Path, files: &[FileRecord]) -> Vec<UnitDescriptor> {
        let workspace_present = bazel::has_workspace_marker(repo_root);
        files
            .iter()
            .filter(|record| record.kind == FileKind::ScalaSource)
            .filter_map(|record| {
                let file_dir = record.canonical_path.parent().unwrap_or(repo_root);
                let context = context_for(record, file_dir, repo_root, workspace_present)?;
                Some(descriptor_for(record, context))
            })
            .collect()
    }
}

/// Context name for one Scala source, or `None` when it is not a unit.
fn context_for(
    record: &FileRecord,
    file_dir: &Path,
    repo_root: &Path,
    workspace_present: bool,
) -> Option<String> {
    if let Some(package) = bazel::package_context_name(file_dir, repo_root, workspace_present) {
        return (!bazel::is_test_source(&record.path)).then_some(package);
    }
    if workspace_present || !is_scala_main_source(record) {
        return None;
    }
    Some(sbt_context_name(file_dir, repo_root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_repo::{FileId, FileOwnership, OptimizationPolicy, SourceLanguage};
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
            language: SourceLanguage::Scala,
        }
    }

    fn scala_record(root: &Path, rel: &str) -> FileRecord {
        record(root, rel, FileKind::ScalaSource)
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
        assert_eq!(
            ScalaUnitShaper.build_markers(),
            vec!["build.sbt", "build.sc", "pom.xml"]
        );
    }

    #[test]
    fn default_compiler_is_scalac() {
        assert_eq!(ScalaUnitShaper.default_compiler(), "scalac");
    }

    #[test]
    fn shape_units_yields_one_unit_per_main_scala_file_in_the_root_project() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(
            root,
            &[
                "build.sbt",
                "src/main/scala/com/example/Main.scala",
                "src/main/scala/com/example/models/User.scala",
                "src/test/scala/Spec.scala",
                "notes.txt",
            ],
        );
        let files = vec![
            scala_record(root, "src/main/scala/com/example/Main.scala"),
            scala_record(root, "src/main/scala/com/example/models/User.scala"),
            scala_record(root, "src/test/scala/Spec.scala"),
            record(root, "notes.txt", FileKind::Other),
        ];

        let units = ScalaUnitShaper.shape_units(root, &files);

        assert_eq!(units.len(), 2, "one unit per src/main/scala file");
        assert_eq!(
            units[0].file_id,
            FileId::from_relative_path("src/main/scala/com/example/Main.scala")
        );
        assert_eq!(
            units[1].file_id,
            FileId::from_relative_path("src/main/scala/com/example/models/User.scala")
        );
        for unit in &units {
            assert_eq!(unit.language, LanguageId::Scala);
            assert_eq!(unit.unit_kind, UnitKind::SourceFile);
            assert_eq!(unit.context_name, "root", "sbt root project");
            assert_eq!(unit.aggregation_target, None);
            assert!(
                unit.build_spec.is_empty()
                    && unit.defines.is_empty()
                    && unit.include_dirs.is_empty(),
                "sbt invocation text is M23 toolchain work"
            );
        }
    }

    #[test]
    fn shape_units_names_the_submodule_context_from_its_build_sbt() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(
            root,
            &[
                "build.sbt",
                "client/build.sbt",
                "client/src/main/scala/Client.scala",
            ],
        );
        let files = vec![scala_record(root, "client/src/main/scala/Client.scala")];

        let units = ScalaUnitShaper.shape_units(root, &files);

        assert_eq!(units.len(), 1);
        assert_eq!(units[0].context_name, "client");
        assert_eq!(
            units[0].file_id,
            FileId::from_relative_path("client/src/main/scala/Client.scala")
        );
    }

    #[test]
    fn shape_units_without_any_sbt_file_still_names_the_root_context() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(root, &["src/main/scala/App.scala"]);
        let files = vec![scala_record(root, "src/main/scala/App.scala")];

        let units = ScalaUnitShaper.shape_units(root, &files);

        assert_eq!(units.len(), 1);
        assert_eq!(units[0].context_name, "root");
    }

    #[test]
    fn shape_units_never_shapes_the_test_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(
            root,
            &["build.sbt", "src/test/scala/Spec.scala", "src/test/scala/deep/NestedSpec.scala"],
        );
        let files = vec![
            scala_record(root, "src/test/scala/Spec.scala"),
            scala_record(root, "src/test/scala/deep/NestedSpec.scala"),
        ];

        assert!(
            ScalaUnitShaper.shape_units(root, &files).is_empty(),
            "src/test/scala is the test-root exclusion tag"
        );
    }

    #[test]
    fn shape_units_of_empty_input_yields_nothing() {
        let dir = tempfile::tempdir().unwrap();
        assert!(ScalaUnitShaper.shape_units(dir.path(), &[]).is_empty());
    }

    #[test]
    fn shape_units_preserves_input_order() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(
            root,
            &[
                "build.sbt",
                "src/main/scala/com/example/Main.scala",
                "src/main/scala/com/example/models/User.scala",
            ],
        );
        let files = vec![
            scala_record(root, "src/main/scala/com/example/models/User.scala"),
            scala_record(root, "src/main/scala/com/example/Main.scala"),
        ];

        let units = ScalaUnitShaper.shape_units(root, &files);

        assert_eq!(units.len(), 2);
        assert_eq!(
            units[0].file_id,
            FileId::from_relative_path("src/main/scala/com/example/models/User.scala"),
            "discovery yields path-sorted records; shapers must stay order-preserving"
        );
        assert_eq!(
            units[1].file_id,
            FileId::from_relative_path("src/main/scala/com/example/Main.scala")
        );
    }

    #[test]
    fn bazel_repo_names_contexts_after_nearest_build_package_and_ignores_sbt() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(
            root,
            &[
                "WORKSPACE",
                "build.sbt",
                "features/build.sbt",
                "features/billing/BUILD.bazel",
                "features/billing/Invoice.scala",
                "features/billing/model/Line.scala",
                "baselibs/core/BUILD",
                "baselibs/core/src/main/scala/Core.scala",
                "baselibs/core/util/BUILD",
                "baselibs/core/util/Strings.scala",
            ],
        );
        let files = vec![
            scala_record(root, "features/billing/Invoice.scala"),
            scala_record(root, "features/billing/model/Line.scala"),
            scala_record(root, "baselibs/core/src/main/scala/Core.scala"),
            scala_record(root, "baselibs/core/util/Strings.scala"),
        ];

        let names: Vec<String> = ScalaUnitShaper
            .shape_units(root, &files)
            .into_iter()
            .map(|unit| unit.context_name)
            .collect();

        assert_eq!(
            names,
            vec![
                "features/billing",
                "features/billing",
                "baselibs/core",
                "baselibs/core/util",
            ],
            "repo-relative package path names the context; sbt files are ignored"
        );
    }

    #[test]
    fn bazel_repo_excludes_test_sources() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(
            root,
            &[
                "MODULE.bazel",
                "lib/BUILD.bazel",
                "lib/Lib.scala",
                "lib/LibTest.scala",
                "lib/LibSpec.scala",
                "lib/LibSuite.scala",
                "lib/test/BUILD",
                "lib/test/Helper.scala",
                "lib/src/test/scala/Fixture.scala",
                "lib/tests/Other.scala",
            ],
        );
        let files: Vec<FileRecord> = [
            "lib/Lib.scala",
            "lib/LibTest.scala",
            "lib/LibSpec.scala",
            "lib/LibSuite.scala",
            "lib/test/Helper.scala",
            "lib/src/test/scala/Fixture.scala",
            "lib/tests/Other.scala",
        ]
        .iter()
        .map(|rel| scala_record(root, rel))
        .collect();

        let units = ScalaUnitShaper.shape_units(root, &files);

        assert_eq!(units.len(), 1, "only the library source is a unit");
        assert_eq!(units[0].file_id, FileId::from_relative_path("lib/Lib.scala"));
    }

    #[test]
    fn bazel_repo_drops_sources_outside_any_package_and_names_root_package_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(
            root,
            &["WORKSPACE.bazel", "BUILD", "Top.scala", "loose/Orphan.scala"],
        );
        let files = vec![
            scala_record(root, "Top.scala"),
            scala_record(root, "loose/Orphan.scala"),
        ];

        let units = ScalaUnitShaper.shape_units(root, &files);

        assert_eq!(units.len(), 2, "the root BUILD package owns both files");
        assert!(units.iter().all(|unit| unit.context_name == "root"));

        fs::remove_file(root.join("BUILD")).unwrap();
        assert!(
            ScalaUnitShaper.shape_units(root, &files).is_empty(),
            "a .scala file no Bazel package owns is not built, so not a unit"
        );
    }

    #[test]
    fn build_bazel_names_packages_even_without_a_workspace_marker() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        create_tree(
            root,
            &[
                "build.sbt",
                "billing/BUILD.bazel",
                "billing/Invoice.scala",
                "pants/BUILD",
                "pants/src/main/scala/Legacy.scala",
            ],
        );
        let files = vec![
            scala_record(root, "billing/Invoice.scala"),
            scala_record(root, "pants/src/main/scala/Legacy.scala"),
        ];

        let names: Vec<String> = ScalaUnitShaper
            .shape_units(root, &files)
            .into_iter()
            .map(|unit| unit.context_name)
            .collect();

        assert_eq!(
            names,
            vec!["billing", "root"],
            "BUILD.bazel is unambiguous; plain BUILD needs a workspace marker, \
             so the second file keeps the sbt rule"
        );
    }

    #[test]
    fn pin_path_contains_components_is_case_insensitive_on_mixed_case_roots() {
        assert!(
            path_contains_components(Path::new("Src/MAIN/scala/App.scala"), &MAIN_SOURCE_ROOT),
            "components are lowercased before the window match, so a mixed-case \
             `Src/MAIN/scala` root still reads as the sbt main root"
        );
        assert!(
            path_contains_components(Path::new("SRC/test/Scala/Spec.scala"), &TEST_SOURCE_ROOT),
            "the test-root exclusion tag matches case-insensitively too"
        );
        assert!(
            !path_contains_components(Path::new("src/main/scala"), &TEST_SOURCE_ROOT),
            "a shorter-than-pattern path can never match the window"
        );
    }
}
