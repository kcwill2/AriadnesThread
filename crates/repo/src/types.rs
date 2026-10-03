//! Core types for repository discovery and file modeling.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

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

/// Category of file based on its language role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FileKind {
    // C / C++
    Header,
    Source,
    // Scala
    ScalaSource,
    ScalaScript,
    BuildDefinition,
    // Catch-all
    Other,
}

impl FileKind {
    pub fn from_path(path: &Path) -> Self {
        match path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
            .as_deref()
        {
            Some("h" | "hpp" | "hxx" | "hh" | "inl" | "ipp") => FileKind::Header,
            Some("cpp" | "cc" | "cxx" | "c") => FileKind::Source,
            Some("scala") => FileKind::ScalaSource,
            Some("sc") => FileKind::ScalaScript,
            Some("sbt" | "mill") => FileKind::BuildDefinition,
            _ => FileKind::Other,
        }
    }

    pub fn is_source_or_header(&self) -> bool {
        matches!(self, FileKind::Header | FileKind::Source)
    }

    /// Returns `true` for any file kind that should be semantically analysed.
    pub fn is_analyzable(&self) -> bool {
        matches!(
            self,
            FileKind::Header | FileKind::Source | FileKind::ScalaSource
        )
    }
}

/// E-I9 (plan 1790521470000): needs-sets carry includable headers only —
/// `.h`/`.hpp`/`.hxx`, case-sensitive, mirroring the ui node-kind detection
/// in `dto.rs`. `.ipp`/`.inl` template-impl files are deliberately OUT for
/// now: whether they belong in the includable set is a future question, not
/// today's. Paths without a final component (e.g. `..`, empty) are never
/// includable.
///
/// The file-name taxonomy lives in `ariadne-repo` (plan §6.3 rule R2 keeps
/// suffix knowledge out of the core pipeline crates); `graph` re-exports
/// this predicate for its load-justification consumers.
pub fn is_includable_header(path: &Path) -> bool {
    match path.file_name() {
        Some(name) => {
            let name = name.to_string_lossy();
            name.ends_with(".h") || name.ends_with(".hpp") || name.ends_with(".hxx")
        }
        None => false,
    }
}

#[cfg(test)]
mod includable_header_tests {
    use super::is_includable_header;
    use std::path::Path;

    #[test]
    fn test_is_includable_header_predicate() {
        let cases: Vec<(&str, bool)> = vec![
            ("a.h", true),
            ("a.hpp", true),
            ("a.hxx", true),
            ("inc/util.h", true),
            // Final component ends in .h, so this IS a header regardless of the
            // embedded .cpp: e.g. a hand-written header living beside worker.cpp.
            ("x.cpp.h", true),
            // Decision, documented here: a file literally named ".h" is treated
            // as a header — the predicate is ends_with on the file name (same
            // rule as the dto.rs node-kind check), and ".h" satisfies it.
            (".h", true),
            ("a.cc", false),
            ("a.cpp", false),
            ("a.c", false),
            // Template-impl files are deliberately OUT for now — future question.
            ("a.ipp", false),
            ("a.inl", false),
            // Case-sensitive, matching dto.rs kind detection.
            ("a.H", false),
            ("a.txt", false),
            // No final component at all: not includable.
            ("..", false),
            ("", false),
        ];
        for (name, expected) in cases {
            assert_eq!(
                is_includable_header(Path::new(name)),
                expected,
                "is_includable_header({name:?})"
            );
        }
    }
}

/// The programming language associated with a file.
///
/// Derived `Ord` gives a stable order for report grouping; `#[default] Cpp`
/// makes every pre-existing JSON artifact (which lacks the field) read back
/// as `Cpp`. Stored PascalCase variant names are unchanged (no `rename_all`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub enum SourceLanguage {
    #[default]
    Cpp,
    Scala,
    Other,
}

impl SourceLanguage {
    pub fn from_file_kind(kind: FileKind) -> Self {
        match kind {
            FileKind::Header | FileKind::Source => SourceLanguage::Cpp,
            FileKind::ScalaSource | FileKind::ScalaScript | FileKind::BuildDefinition => {
                SourceLanguage::Scala
            }
            FileKind::Other => SourceLanguage::Other,
        }
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
    /// Language of the file. Additive (plan §3.5): `#[serde(default)]` ⇒
    /// pre-existing manifests read back as `Cpp`.
    #[serde(default)]
    pub language: SourceLanguage,
}

/// Aggregate result of repository discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryDiscoveryResult {
    pub root: PathBuf,
    pub files: Vec<FileRecord>,
    pub ownership_summary: HashMap<FileOwnership, usize>,
    pub policy_summary: HashMap<OptimizationPolicy, usize>,
}
