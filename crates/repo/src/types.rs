//! Core types for repository discovery and file modeling.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Unique, stable identifier for a file within a repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FileId(pub u64);

impl FileId {
    /// Derives a deterministic FileId from a normalized relative path string.
    pub fn from_relative_path(rel_path: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(rel_path.as_bytes());
        let result = hasher.finalize();
        let bytes: [u8; 8] = result[0..8].try_into().expect("Slice is 8 bytes");
        FileId(u64::from_be_bytes(bytes))
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Category of file based on C++ role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileKind {
    Header,
    Source,
    Other,
}

impl FileKind {
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()).map(|s| s.to_lowercase()).as_deref() {
            Some("h" | "hpp" | "hxx" | "hh" | "inl" | "ipp") => FileKind::Header,
            Some("cpp" | "cc" | "cxx" | "c") => FileKind::Source,
            _ => FileKind::Other,
        }
    }

    pub fn is_source_or_header(&self) -> bool {
        matches!(self, FileKind::Header | FileKind::Source)
    }
}

/// Entity controlling or producing the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileOwnership {
    Repository,
    ThirdParty,
    Toolchain,
    Generated,
}

/// Permission for the dependency optimizer to alter the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OptimizationPolicy {
    Optimizable,
    Contract,
    Frozen,
}

/// Comprehensive record describing a discovered file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: FileId,
    pub path: PathBuf,
    pub canonical_path: PathBuf,
    pub kind: FileKind,
    pub size: u64,
    pub content_hash: String,
    pub ownership: FileOwnership,
    pub optimization_policy: OptimizationPolicy,
}

/// Aggregate result of repository discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryDiscoveryResult {
    pub root: PathBuf,
    pub files: Vec<FileRecord>,
    pub ownership_summary: HashMap<FileOwnership, usize>,
    pub policy_summary: HashMap<OptimizationPolicy, usize>,
}
