//! Shared value types moved verbatim from `crates/index/src/types.rs`
//! (plan §3.1): same field order and serde attributes, so artifact JSON stays
//! byte-compatible. Only the additive fields/variants documented in plan §3.2
//! were introduced (`TuAnalysis::language`, `SymbolKind::{Trait, Object,
//! Module}` — the latter are never emitted by the C++ parser).

use ariadne_repo::{FileId, SourceLanguage};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SymbolId(pub String);

impl SymbolId {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self(name.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeclId(pub String);

impl DeclId {
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self(id.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Class,
    Struct,
    Union,
    Enum,
    Function,
    Variable,
    TypeAlias,
    Field,
    Concept,
    Other,
    /// Single-file languages (Scala `trait`, …). Never emitted by the C++ parser.
    Trait,
    /// Single-file languages (Scala `object`, …). Never emitted by the C++ parser.
    Object,
    /// Namespace/module symbols. Never emitted by the C++ parser.
    Module,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DependencyKind {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileLocation {
    pub file_id: FileId,
    pub line: usize,
    pub column: usize,
}

impl Default for FileLocation {
    fn default() -> Self {
        Self {
            file_id: FileId(0),
            line: 0,
            column: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolDecl {
    pub id: DeclId,
    pub symbol_id: SymbolId,
    pub location: FileLocation,
    pub is_definition: bool,
}

impl Default for SymbolDecl {
    fn default() -> Self {
        Self {
            id: DeclId(String::new()),
            symbol_id: SymbolId(String::new()),
            location: FileLocation::default(),
            is_definition: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub owning_file: FileId,
    pub declarations: Vec<DeclId>,
    pub definition: Option<DeclId>,
}

impl Default for SymbolRecord {
    fn default() -> Self {
        Self {
            id: SymbolId(String::new()),
            name: String::new(),
            kind: SymbolKind::Other,
            owning_file: FileId(0),
            declarations: Vec::new(),
            definition: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticEdge {
    pub src_symbol: SymbolId,
    pub dst_symbol: SymbolId,
    pub kind: DependencyKind,
    pub src_file: FileId,
    pub dst_file: FileId,
}

impl Default for SemanticEdge {
    fn default() -> Self {
        Self {
            src_symbol: SymbolId(String::new()),
            dst_symbol: SymbolId(String::new()),
            kind: DependencyKind::Incomplete,
            src_file: FileId(0),
            dst_file: FileId(0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallSite {
    pub caller_symbol: SymbolId,
    pub callee_name: String,
    pub location: FileLocation,
}

impl Default for CallSite {
    fn default() -> Self {
        Self {
            caller_symbol: SymbolId(String::new()),
            callee_name: String::new(),
            location: FileLocation::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolReference {
    pub symbol_id: SymbolId,
    pub file_id: FileId,
    pub location: FileLocation,
    pub is_write: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuAnalysis {
    pub tu_file_id: FileId,
    pub context_name: String,
    pub symbols: Vec<SymbolRecord>,
    pub declarations: Vec<SymbolDecl>,
    pub edges: Vec<SemanticEdge>,
    pub references: Vec<SymbolReference>,
    pub call_sites: Vec<CallSite>,
    /// Language this analysis was produced for. Additive (plan §3.5):
    /// `#[serde(default)]` ⇒ pre-existing artifacts read back as `Cpp`.
    #[serde(default)]
    pub language: SourceLanguage,
}

impl Default for TuAnalysis {
    fn default() -> Self {
        Self {
            tu_file_id: FileId(0),
            context_name: String::new(),
            symbols: Vec::new(),
            declarations: Vec::new(),
            edges: Vec::new(),
            references: Vec::new(),
            call_sites: Vec::new(),
            language: SourceLanguage::default(),
        }
    }
}

// ── Include-graph DTOs moved from `crates/graph/src/types.rs` (plan M18.1) ──
// The transitional `ariadne-lang-cpp` resolver produces `IncludeEdge` values
// and a plugin crate may not depend on `ariadne-graph`, so these pure data
// types live with the rest of the shared artifact vocabulary; `graph`
// re-exports them at their original paths (serde/paths unchanged).

/// Classification of an included header file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum IncludeKind {
    /// Local header found via relative search or local `-I` directory.
    Local,
    /// System header found via `<...>` or system include paths.
    System,
    /// External third-party library header.
    External,
    /// Resolved module-system dependency directive (M20 D10). Appended LAST:
    /// the bincode serialized indices of the variants above are frozen
    /// (0/1/2) — see `include_kind_bincode_indices_are_pinned`.
    Import,
}

/// Source location of an include directive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
}

impl SourceLocation {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

/// Directed include edge representing a dependency directive from source to
/// destination file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct IncludeEdge {
    pub source: FileId,
    pub destination: Option<FileId>,
    pub context_name: String,
    pub source_location: SourceLocation,
    pub kind: IncludeKind,
    pub raw_path: String,
    pub resolved_path: Option<PathBuf>,
    /// Language the edge was produced for. Additive (plan §3.5, M18.5):
    /// `#[serde(default)]` ⇒ pre-existing artifacts read back as `Cpp`.
    #[serde(default)]
    pub language: SourceLanguage,
}

#[cfg(test)]
mod include_dto_tests {
    use super::*;

    #[test]
    fn test_include_edge_serialization() {
        let edge = IncludeEdge {
            source: FileId(10),
            destination: Some(FileId(20)),
            context_name: "Debug|x64".to_string(),
            source_location: SourceLocation::new(4, 1),
            kind: IncludeKind::Local,
            raw_path: "thing_types.h".to_string(),
            resolved_path: Some(PathBuf::from("include/thing_types.h")),
            language: SourceLanguage::Cpp,
        };

        let json = serde_json::to_string(&edge).unwrap();
        let decoded: IncludeEdge = serde_json::from_str(&json).unwrap();
        assert_eq!(edge, decoded);

        let bin = bincode::serialize(&edge).unwrap();
        let decoded_bin: IncludeEdge = bincode::deserialize(&bin).unwrap();
        assert_eq!(edge, decoded_bin);

        // Additive-field contract (plan §3.5): artifacts written before the
        // `language` key existed must keep deserializing, reading as Cpp.
        let json_no_lang = json.replace(",\"language\":\"Cpp\"", "");
        let pre: IncludeEdge = serde_json::from_str(&json_no_lang).unwrap();
        assert_eq!(pre.language, SourceLanguage::Cpp);
        assert_eq!(pre.source, edge.source);
    }

    #[test]
    fn include_kind_bincode_indices_are_pinned() {
        // IV-2/IV-6 bincode index discipline: bincode 1.x writes a fieldless
        // enum variant as its discriminant index (u32, little-endian). The
        // pre-M20 variants are decoded from hand-written bytes representing
        // the original three-variant understanding, proving the append of
        // `Import` last did not shift any serialized index.
        let index_bytes = |variant: IncludeKind| -> Vec<u8> {
            bincode::serialize(&variant).unwrap()
        };
        assert_eq!(index_bytes(IncludeKind::Local), [0, 0, 0, 0]);
        assert_eq!(index_bytes(IncludeKind::System), [1, 0, 0, 0]);
        assert_eq!(index_bytes(IncludeKind::External), [2, 0, 0, 0]);

        assert_eq!(
            bincode::deserialize::<IncludeKind>(&[0, 0, 0, 0]).unwrap(),
            IncludeKind::Local
        );
        assert_eq!(
            bincode::deserialize::<IncludeKind>(&[1, 0, 0, 0]).unwrap(),
            IncludeKind::System
        );
        assert_eq!(
            bincode::deserialize::<IncludeKind>(&[2, 0, 0, 0]).unwrap(),
            IncludeKind::External
        );

        // IV-2: `Import` is appended LAST, so it takes the next free index.
        assert_eq!(IncludeKind::Import as u64, 3);
        assert_eq!(index_bytes(IncludeKind::Import), [3, 0, 0, 0]);
        assert_eq!(
            bincode::deserialize::<IncludeKind>(&[3, 0, 0, 0]).unwrap(),
            IncludeKind::Import
        );
    }

    #[test]
    fn import_edge_with_module_language_round_trips() {
        // IV-6: the additive `Import` kind travels with the module-system
        // language without disturbing any pre-existing serialized shape.
        let edge = IncludeEdge {
            source: FileId(11),
            destination: Some(FileId(22)),
            context_name: "Default".to_string(),
            source_location: SourceLocation::new(2, 1),
            kind: IncludeKind::Import,
            raw_path: "com.example.models.Order".to_string(),
            resolved_path: Some(PathBuf::from("src/main/pkg/models/Order.src")),
            language: SourceLanguage::Scala,
        };

        let json = serde_json::to_string(&edge).unwrap();
        let decoded: IncludeEdge = serde_json::from_str(&json).unwrap();
        assert_eq!(edge, decoded);

        let bin = bincode::serialize(&edge).unwrap();
        let decoded_bin: IncludeEdge = bincode::deserialize(&bin).unwrap();
        assert_eq!(decoded_bin.kind, IncludeKind::Import);
        assert_eq!(decoded_bin.language, SourceLanguage::Scala);
        assert_eq!(edge, decoded_bin);
    }
}
