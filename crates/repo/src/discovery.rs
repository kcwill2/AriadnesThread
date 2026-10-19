//! Filesystem traversal and hashing for repository discovery.

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::policy::classify_ownership_and_policy;
use crate::types::{
    FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, RepositoryDiscoveryResult,
    SourceLanguage,
};

/// Computes the SHA-256 hex digest of a file's contents.
pub fn compute_file_hash(path: &Path) -> Result<String> {
    let bytes =
        fs::read(path).with_context(|| format!("Failed to read file for hashing: {:?}", path))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Recursively discovers source and header files in the given root directory.
pub fn discover_repository_files(root: &Path) -> Result<Vec<FileRecord>> {
    discover_repository_files_with_progress(root, &mut |_hashed, _discovered| {})
}

/// Walk and hash analyzable files, firing `on_hashed` after each
/// successful `compute_file_hash` with `(hashed_so_far, discovered_so_far)`.
/// On this single-pass walk both counts are equal; the pair is spelled as
/// two arguments so a future 2-pass walk can split them. The hook fires in
/// WALK order; the returned records keep the path-sorted order.
pub(crate) fn discover_repository_files_with_progress(
    root: &Path,
    on_hashed: &mut dyn FnMut(usize, usize),
) -> Result<Vec<FileRecord>> {
    let mut files = Vec::new();
    let mut hashed_so_far = 0usize;
    // UTB-8: a junction/symlink back into an ancestor makes the stack walk
    // unbounded. A directory is admitted only when its canonical form has
    // not been walked yet; canonicalize failure falls back to the raw path,
    // mirroring the file-side precedent below (discovery.rs file branch).
    let mut walked_dirs: HashSet<PathBuf> = HashSet::new();
    walked_dirs.insert(root.canonicalize().unwrap_or_else(|_| root.to_path_buf()));
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            if name_str.starts_with('.')
                || name_str == "target"
                || name_str == "artifacts"
                || name_str == "node_modules"
            {
                continue;
            }

            if path.is_dir() {
                let walked_key = path.canonicalize().unwrap_or_else(|_| path.clone());
                if walked_dirs.insert(walked_key) {
                    stack.push(path);
                }
            } else if path.is_file() {
                let kind = FileKind::from_path(&path);
                // M19.3 (ruling 3): neutral analyzability predicate - includes
                // ScalaSource while excluding ScalaScript/BuildDefinition,
                // exactly plan:387's ".scala files listed".
                if kind.is_analyzable() {
                    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
                    let rel_path = path.strip_prefix(root).unwrap_or(&path);
                    let norm_rel_str = rel_path.to_string_lossy().replace('\\', "/");
                    let file_id = FileId::from_relative_path(&norm_rel_str);

                    let metadata = fs::metadata(&path)?;
                    let size = metadata.len();
                    let content_hash = compute_file_hash(&path)?;
                    let (ownership, policy) = classify_ownership_and_policy(rel_path, false);

                    files.push(FileRecord {
                        id: file_id,
                        path: PathBuf::from(&norm_rel_str),
                        canonical_path,
                        kind,
                        size,
                        content_hash,
                        ownership,
                        optimization_policy: policy,
                        language: SourceLanguage::from_file_kind(kind),
                    });
                    hashed_so_far += 1;
                    on_hashed(hashed_so_far, files.len());
                }
            }
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

/// Constructs a complete RepositoryDiscoveryResult with summary counts.
pub fn build_discovery_result(root: PathBuf, files: Vec<FileRecord>) -> RepositoryDiscoveryResult {
    let mut ownership_summary = HashMap::new();
    ownership_summary.insert(FileOwnership::Repository, 0);
    ownership_summary.insert(FileOwnership::ThirdParty, 0);
    ownership_summary.insert(FileOwnership::Toolchain, 0);
    ownership_summary.insert(FileOwnership::Generated, 0);

    let mut policy_summary = HashMap::new();
    policy_summary.insert(OptimizationPolicy::Optimizable, 0);
    policy_summary.insert(OptimizationPolicy::Contract, 0);
    policy_summary.insert(OptimizationPolicy::Frozen, 0);

    for file in &files {
        *ownership_summary.entry(file.ownership).or_insert(0) += 1;
        *policy_summary.entry(file.optimization_policy).or_insert(0) += 1;
    }

    RepositoryDiscoveryResult {
        root,
        files,
        ownership_summary,
        policy_summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_compute_file_hash() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, b"hello world").unwrap();

        let hash = compute_file_hash(&file_path).unwrap();
        // SHA-256 of "hello world"
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );

        let missing = dir.path().join("missing.txt");
        assert!(compute_file_hash(&missing).is_err());
    }

    #[test]
    fn test_discover_repository_files() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Create some source files
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("src/main.cpp"), b"int main() {}").unwrap();
        fs::write(root.join("src/utils.h"), b"void foo();").unwrap();
        fs::write(root.join("src/data.txt"), b"some data").unwrap();

        // Create ignored dirs
        fs::create_dir(root.join(".hidden")).unwrap();
        fs::write(root.join(".hidden/secret.cpp"), b"secret").unwrap();

        fs::create_dir(root.join("target")).unwrap();
        fs::write(root.join("target/out.cpp"), b"out").unwrap();

        fs::create_dir(root.join("artifacts")).unwrap();
        fs::write(root.join("artifacts/test.c"), b"test").unwrap();

        fs::create_dir(root.join("node_modules")).unwrap();
        fs::write(root.join("node_modules/lib.cpp"), b"lib").unwrap();

        let files = discover_repository_files(root).unwrap();

        assert_eq!(files.len(), 2, "Should only contain main.cpp and utils.h");

        // Results should be alphabetically sorted
        assert_eq!(files[0].path.to_str().unwrap(), "src/main.cpp");
        assert_eq!(files[1].path.to_str().unwrap(), "src/utils.h");
    }

    #[test]
    fn test_build_discovery_result() {
        let file1 = FileRecord {
            id: crate::types::FileId::from_relative_path("src/main.cpp"),
            path: PathBuf::from("src/main.cpp"),
            canonical_path: PathBuf::from("/mock/src/main.cpp"),
            kind: crate::types::FileKind::Source,
            size: 10,
            content_hash: "hash1".to_string(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
            language: crate::types::SourceLanguage::Cpp,
        };
        let file2 = FileRecord {
            id: crate::types::FileId::from_relative_path("third_party/lib.cpp"),
            path: PathBuf::from("third_party/lib.cpp"),
            canonical_path: PathBuf::from("/mock/third_party/lib.cpp"),
            kind: crate::types::FileKind::Source,
            size: 10,
            content_hash: "hash2".to_string(),
            ownership: FileOwnership::ThirdParty,
            optimization_policy: OptimizationPolicy::Frozen,
            language: crate::types::SourceLanguage::Cpp,
        };

        let result = build_discovery_result(PathBuf::from("/mock"), vec![file1, file2]);

        assert_eq!(result.ownership_summary[&FileOwnership::Repository], 1);
        assert_eq!(result.ownership_summary[&FileOwnership::ThirdParty], 1);
        assert_eq!(result.ownership_summary[&FileOwnership::Toolchain], 0);
        assert_eq!(result.ownership_summary[&FileOwnership::Generated], 0);

        assert_eq!(result.policy_summary[&OptimizationPolicy::Optimizable], 1);
        assert_eq!(result.policy_summary[&OptimizationPolicy::Frozen], 1);
        assert_eq!(result.policy_summary[&OptimizationPolicy::Contract], 0);
    }

    #[test]
    fn test_analyzable_filter_records_scala_sources() {
        // M19.3 (T-B ruling 3): the recording filter is the neutral
        // is_analyzable - ScalaSource in, ScalaScript/BuildDefinition out.
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::write(root.join("a.cpp"), b"int main() {}").unwrap();
        fs::write(root.join("b.h"), b"void foo();").unwrap();
        fs::write(root.join("c.scala"), b"object C {}").unwrap();
        fs::write(root.join("d.sc"), b"println(1)").unwrap();
        fs::write(root.join("build.sbt"), b"name := \"x\"").unwrap();

        let files = discover_repository_files(root).unwrap();
        let names: Vec<String> = files
            .iter()
            .map(|record| record.path.to_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["a.cpp", "b.h", "c.scala"]);

        let scala = names
            .iter()
            .position(|name| name == "c.scala")
            .map(|index| &files[index]);
        let scala = scala.expect("c.scala is recorded as an analyzable ScalaSource");
        assert_eq!(scala.kind, crate::types::FileKind::ScalaSource);
        assert_eq!(scala.language, crate::types::SourceLanguage::Scala);
    }
}

#[cfg(test)]
mod progress_hook_tests {
    use super::*;
    use crate::Repository;
    use std::fs;
    use tempfile::TempDir;

    /// Creates `rel` under `root`, building parent dirs as needed.
    fn create_tree_file(root: &Path, rel: &str, body: &[u8]) {
        let full = root.join(rel);
        fs::create_dir_all(full.parent().expect("relative path has a parent")).unwrap();
        fs::write(&full, body).unwrap();
    }

    /// The contract lock: with_progress yields exactly what discover()
    /// yields - same records, same ids/paths/hashes, same order, same
    /// summaries.
    #[test]
    fn test_discover_with_progress_result_identical_to_discover() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        create_tree_file(root, "include/thing.h", b"#pragma once\nvoid foo();");
        create_tree_file(root, "src/main.cpp", b"int main() { return 0; }");
        create_tree_file(root, "src/nested/deep.scala", b"object Deep {}");
        create_tree_file(root, "target/out.cpp", b"out");
        create_tree_file(root, "notes.md", b"# notes");

        let repo = Repository::locate(root).unwrap();
        let plain_result = repo.discover().unwrap();
        let hooked_result = repo
            .discover_with_progress(&mut |_hashed, _discovered| {})
            .unwrap();

        assert!(
            !plain_result.files.is_empty(),
            "fixture must actually record files"
        );
        assert_eq!(plain_result.root, hooked_result.root);
        assert_eq!(
            plain_result.files, hooked_result.files,
            "records (ids, paths, hashes, order) must be identical"
        );
        assert_eq!(
            plain_result.ownership_summary,
            hooked_result.ownership_summary
        );
        assert_eq!(plain_result.policy_summary, hooked_result.policy_summary);
    }

    /// Hook fires exactly once per analyzable file; the first argument
    /// counts 1,2,3 in call order and equals the second argument at every
    /// call (single-pass walk per the contract).
    #[test]
    fn test_hook_fires_once_per_analyzable_file_with_monotone_counts() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        create_tree_file(root, "src/first.cpp", b"int first() {}");
        create_tree_file(root, "src/second.h", b"void second();");
        create_tree_file(root, "src/third.scala", b"object Third {}");
        create_tree_file(root, "node_modules/skipped.cpp", b"skipped");
        create_tree_file(root, "readme.md", b"# readme");

        let repo = Repository::locate(root).unwrap();
        let mut hook_calls: Vec<(usize, usize)> = Vec::new();
        let discovery_result = repo
            .discover_with_progress(&mut |hashed, discovered| {
                hook_calls.push((hashed, discovered));
            })
            .unwrap();

        assert_eq!(
            hook_calls.len(),
            3,
            "hook fires once per analyzable file"
        );
        let hashed_counts: Vec<usize> = hook_calls.iter().map(|call| call.0).collect();
        assert_eq!(hashed_counts, vec![1, 2, 3], "first argument runs 1,2,3");
        for call in &hook_calls {
            assert_eq!(
                call.0, call.1,
                "single-pass walk: hashed_so_far == discovered_so_far"
            );
        }
        assert_eq!(discovery_result.files.len(), 3);
    }

    /// Hook call count equals the recorded-file count on a tree with five
    /// analyzable files across three dirs plus the whole skip list present.
    #[test]
    fn test_hook_count_matches_result_len_on_mixed_tree() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        create_tree_file(root, "src/one.cpp", b"int one() {}");
        create_tree_file(root, "src/two.h", b"void two();");
        create_tree_file(root, "lib/three.cpp", b"int three() {}");
        create_tree_file(root, "lib/four.c", b"int four() {}");
        create_tree_file(root, "apps/five.scala", b"object Five {}");
        create_tree_file(root, "target/skip.cpp", b"skip");
        create_tree_file(root, "artifacts/skip.c", b"skip");
        create_tree_file(root, "node_modules/skip.cpp", b"skip");
        create_tree_file(root, ".hidden/skip.h", b"skip");

        let repo = Repository::locate(root).unwrap();
        let mut hook_call_count = 0usize;
        let discovery_result = repo
            .discover_with_progress(&mut |_hashed, _discovered| {
                hook_call_count += 1;
            })
            .unwrap();

        assert_eq!(
            hook_call_count,
            discovery_result.files.len(),
            "hook count must match recorded files"
        );
        assert_eq!(
            hook_call_count, 5,
            "five analyzable files across three dirs, skip list inert"
        );
    }

    /// Shipped as the Windows fallback for
    /// test_hook_surfaces_hash_error_propagation (unreadable files are not
    /// cheaply expressible via std::fs on Windows; journal records this).
    /// Locks: the hook fires while walking (once per analyzable file) while
    /// the returned records stay path-sorted - hook order is walk order,
    /// never asserted to equal path order.
    #[test]
    fn test_hook_order_independent_of_path_sort() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        create_tree_file(root, "aardvark.cpp", b"int a() {}");
        create_tree_file(root, "sub/zulu.h", b"void z();");
        create_tree_file(root, "sub/yankee.h", b"void y();");
        create_tree_file(root, "zzz.cpp", b"int z() {}");

        let repo = Repository::locate(root).unwrap();
        let mut hook_call_count = 0usize;
        let discovery_result = repo
            .discover_with_progress(&mut |_hashed, _discovered| {
                hook_call_count += 1;
            })
            .unwrap();

        assert_eq!(
            hook_call_count, 4,
            "hook fires while walking, once per analyzable file"
        );
        assert_eq!(
            hook_call_count,
            discovery_result.files.len(),
            "hook count matches the sorted result length"
        );
        let result_paths: Vec<String> = discovery_result
            .files
            .iter()
            .map(|record| record.path.to_string_lossy().to_string())
            .collect();
        let mut sorted_paths = result_paths.clone();
        sorted_paths.sort();
        assert_eq!(
            result_paths, sorted_paths,
            "returned records stay path-sorted regardless of hook timing"
        );
    }
}
