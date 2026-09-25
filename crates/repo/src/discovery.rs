//! Filesystem traversal and hashing for repository discovery.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::policy::classify_ownership_and_policy;
use crate::types::{
    FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, RepositoryDiscoveryResult,
};

/// Computes the SHA-256 hex digest of a file's contents.
pub fn compute_file_hash(path: &Path) -> Result<String> {
    let bytes = fs::read(path)
        .with_context(|| format!("Failed to read file for hashing: {:?}", path))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Recursively discovers source and header files in the given root directory.
pub fn discover_repository_files(root: &Path) -> Result<Vec<FileRecord>> {
    let mut files = Vec::new();
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
                stack.push(path);
            } else if path.is_file() {
                let kind = FileKind::from_path(&path);
                if kind.is_source_or_header() {
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
                    });
                }
            }
        }
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

/// Constructs a complete RepositoryDiscoveryResult with summary counts.
pub fn build_discovery_result(
    root: PathBuf,
    files: Vec<FileRecord>,
) -> RepositoryDiscoveryResult {
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
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");

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
}

