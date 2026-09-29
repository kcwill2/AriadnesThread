//! Shared value types moved verbatim from `crates/index/src/types.rs`
//! (plan §3.1): same field order and serde attributes, so artifact JSON stays
//! byte-compatible. Only the additive fields/variants documented in plan §3.2
//! were introduced (`TuAnalysis::language`, `SymbolKind::{Trait, Object,
//! Module}` — the latter are never emitted by the C++ parser).

use ariadne_repo::{FileId, SourceLanguage};
use serde::{Deserialize, Serialize};

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
