//! Bazel package layout for the Scala unit shaper.
//!
//! In a Bazel repo the build truth is the BUILD package, not any leftover
//! `*.sbt` file, and sources rarely sit under `src/main/scala`. The rules:
//!
//! - A Bazel package is a directory holding `BUILD.bazel`, or holding
//!   `BUILD` when the repo root carries a workspace marker (`WORKSPACE`,
//!   `WORKSPACE.bazel`, `MODULE.bazel`). Plain `BUILD` alone is ambiguous
//!   (Pants and others use it too), so it needs the workspace marker;
//!   `BUILD.bazel` is unambiguous and counts even when the analyzed root is
//!   a subtree with no workspace file.
//! - A Scala file's context is its nearest ancestor package, named by the
//!   package's repo-relative path (`features/billing`) so names stay unique
//!   across a monorepo; the root package is named `root`.
//! - Test sources are not units (conservative read of typical
//!   `scala_library` / `scala_test` layouts): any path segment named
//!   `test` or `tests`, or a file stem ending in `Test`, `Tests`, `Spec`
//!   or `Suite`.

use std::path::Path;

const WORKSPACE_MARKERS: [&str; 3] = ["WORKSPACE", "WORKSPACE.bazel", "MODULE.bazel"];
const TEST_DIR_NAMES: [&str; 2] = ["test", "tests"];
const TEST_STEM_SUFFIXES: [&str; 4] = ["Test", "Tests", "Spec", "Suite"];

/// True when the repo root carries a Bazel workspace marker file.
pub(crate) fn has_workspace_marker(repo_root: &Path) -> bool {
    WORKSPACE_MARKERS
        .iter()
        .any(|marker| repo_root.join(marker).is_file())
}

fn is_bazel_package(dir: &Path, workspace_present: bool) -> bool {
    dir.join("BUILD.bazel").is_file() || (workspace_present && dir.join("BUILD").is_file())
}

/// Name of the nearest ancestor Bazel package of `file_dir` (inclusive),
/// or `None` when no package between it and `repo_root` owns the file.
pub(crate) fn package_context_name(
    file_dir: &Path,
    repo_root: &Path,
    workspace_present: bool,
) -> Option<String> {
    let mut dir = file_dir;
    while dir.starts_with(repo_root) {
        if is_bazel_package(dir, workspace_present) {
            return Some(relative_package_name(dir, repo_root));
        }
        dir = dir.parent()?;
    }
    None
}

fn relative_package_name(package_dir: &Path, repo_root: &Path) -> String {
    let relative = package_dir.strip_prefix(repo_root).unwrap_or(package_dir);
    let segments: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    if segments.is_empty() {
        super::shaper::ROOT_CONTEXT.to_string()
    } else {
        segments.join("/")
    }
}

/// True when the repo-relative path looks like a Bazel test source.
pub(crate) fn is_test_source(relative_path: &Path) -> bool {
    let in_test_dir = relative_path.parent().is_some_and(|parent| {
        parent.components().any(|component| {
            let name = component.as_os_str().to_string_lossy().to_lowercase();
            TEST_DIR_NAMES.contains(&name.as_str())
        })
    });
    let test_stem = relative_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .is_some_and(|stem| TEST_STEM_SUFFIXES.iter().any(|suffix| stem.ends_with(suffix)));
    in_test_dir || test_stem
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_rule_matches_dirs_and_stems_but_not_lookalikes() {
        assert!(is_test_source(Path::new("lib/test/Helper.scala")));
        assert!(is_test_source(Path::new("lib/Tests/Helper.scala")));
        assert!(is_test_source(Path::new("lib/src/test/scala/Fixture.scala")));
        assert!(is_test_source(Path::new("lib/FooSpec.scala")));
        assert!(is_test_source(Path::new("lib/FooSuite.scala")));
        assert!(!is_test_source(Path::new("lib/testing/Harness.scala")));
        assert!(!is_test_source(Path::new("lib/Contest.scala")));
        assert!(!is_test_source(Path::new("test.scala")));
    }
}
