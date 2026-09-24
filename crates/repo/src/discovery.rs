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
