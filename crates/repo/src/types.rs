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
    // Rust - APPENDED LAST (INV-12: no pre-existing variant's serde index moves)
    RustSource,
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
            Some("rs") => FileKind::RustSource,
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
            FileKind::Header | FileKind::Source | FileKind::ScalaSource | FileKind::RustSource
        )
    }
}

#[cfg(test)]
mod file_kind_matrix_tests {
    use super::{is_includable_header, FileKind};
    use std::path::Path;

    fn kind_of(name: &str) -> FileKind {
        FileKind::from_path(Path::new(name))
    }

    /// UTA2-R3 item 3: lowercase extension matrix through `from_path`,
    /// including the no-extension, dotfile, and multi-dot shapes.
    #[test]
    fn pin_file_kind_lowercase_extension_matrix() {
        let cases: Vec<(&str, FileKind)> = vec![
            ("a.cpp", FileKind::Source),
            ("a.cc", FileKind::Source),
            ("a.cxx", FileKind::Source),
            ("a.c", FileKind::Source),
            ("a.h", FileKind::Header),
            ("a.hpp", FileKind::Header),
            ("a.hxx", FileKind::Header),
            ("a.hh", FileKind::Header),
            ("a.inl", FileKind::Header),
            ("a.ipp", FileKind::Header),
            ("a.scala", FileKind::ScalaSource),
            ("a.sc", FileKind::ScalaScript),
            ("a.sbt", FileKind::BuildDefinition),
            ("a.mill", FileKind::BuildDefinition),
            ("a.txt", FileKind::Other),
            ("a.rs", FileKind::RustSource),
            ("Makefile", FileKind::Other),
            ("a", FileKind::Other),
            // A leading-dot name carries no extension for Path: dotfiles
            // are Other, never Header, even when the whole name looks like
            // one (".h" alone is ALSO Other through from_path - note this
            // CONTRASTS with is_includable_header, which reads the file NAME
            // and calls ".h" a header; see the divergence pin below).
            (".h", FileKind::Other),
            (".gitignore", FileKind::Other),
            ("a.b.cpp", FileKind::Source),
            ("a.h.rs", FileKind::RustSource),
        ];
        for (name, expected) in cases {
            assert_eq!(kind_of(name), expected, "FileKind::from_path({name:?})");
        }
    }

    /// UTA2-R3 item 3: `from_path` case-FOLDS the extension before matching,
    /// so every upper/mixed-case form keeps its lowercase kind - including
    /// `.H`, which becomes Header here while staying NON-includable.
    #[test]
    fn pin_file_kind_case_matrix_is_folded_insensitive() {
        let cases: Vec<(&str, FileKind)> = vec![
            ("a.CPP", FileKind::Source),
            ("a.CC", FileKind::Source),
            ("a.CXX", FileKind::Source),
            ("a.C", FileKind::Source),
            ("a.H", FileKind::Header),
            ("a.HPP", FileKind::Header),
            ("a.HXX", FileKind::Header),
            ("a.HH", FileKind::Header),
            ("a.INL", FileKind::Header),
            ("a.IPP", FileKind::Header),
            ("a.SCALA", FileKind::ScalaSource),
            ("a.SC", FileKind::ScalaScript),
            ("a.SBT", FileKind::BuildDefinition),
            ("a.MILL", FileKind::BuildDefinition),
            ("a.Sc", FileKind::ScalaScript),
            ("a.Mill", FileKind::BuildDefinition),
            ("a.Cc", FileKind::Source),
            ("a.RS", FileKind::RustSource),
        ];
        for (name, expected) in cases {
            assert_eq!(kind_of(name), expected, "FileKind::from_path({name:?})");
        }
    }

    /// UTA2-R3 item 3 (REPORT): case-sensitivity DIVERGENCE between the two
    /// header vocabularies. `from_path` folds case; `is_includable_header`
    /// is a case-SENSITIVE ends_with on `.h`/`.hpp`/`.hxx`. Result:
    /// `.H`, `.HH`, `.HPP`, `.HXX`, `.INL`, `.IPP` are Header (analyzable)
    /// yet NOT includable, and `.hh`/`.inl`/`.ipp` are Header even in
    /// lowercase but never includable (deliberate E-I9 scope). Pinned
    /// exactly here; unifying the two vocabularies is an owner decision.
    #[test]
    fn pin_kind_vs_includable_header_case_sensitivity_divergence() {
        let matrix: Vec<(&str, FileKind, bool)> = vec![
            ("a.h", FileKind::Header, true),
            ("a.hpp", FileKind::Header, true),
            ("a.hxx", FileKind::Header, true),
            ("a.H", FileKind::Header, false),
            ("a.HPP", FileKind::Header, false),
            ("a.HXX", FileKind::Header, false),
            ("a.hh", FileKind::Header, false),
            ("a.HH", FileKind::Header, false),
            ("a.inl", FileKind::Header, false),
            ("a.INL", FileKind::Header, false),
            ("a.ipp", FileKind::Header, false),
            ("a.cc", FileKind::Source, false),
            // The dotfile contrast: Other for the kind system, header for
            // the includable predicate (ends_with on the bare name).
            (".h", FileKind::Other, true),
        ];
        for (name, expected_kind, expected_includable) in matrix {
            assert_eq!(kind_of(name), expected_kind, "FileKind::from_path({name:?})");
            assert_eq!(
                is_includable_header(Path::new(name)),
                expected_includable,
                "is_includable_header({name:?})"
            );
        }
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
/// AR-27a home move: the closed enum is retired; this is now a `pub type`
/// alias of the open-ended [`LanguageId`] newtype homed in
/// [`crate::identity`]. Every expression-spelled site (`SourceLanguage::Cpp`,
/// `SourceLanguage::default()`, `SourceLanguage::from_file_kind(..)`, the
/// `.language` fields) compiles unchanged through the alias; the serde shape
/// contract (positional u32 index under bincode, bare name under JSON) is
/// carried by the newtype's dual-format impls and byte-locked by the
/// `ar27_*` oracle tests.
pub type SourceLanguage = crate::identity::LanguageId;

pub use crate::identity::LanguageId;

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
