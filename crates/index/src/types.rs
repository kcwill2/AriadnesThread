use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use ariadne_repo::FileId;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolDecl {
    pub id: DeclId,
    pub symbol_id: SymbolId,
    pub location: FileLocation,
    pub is_definition: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticEdge {
    pub src_symbol: SymbolId,
    pub dst_symbol: SymbolId,
    pub kind: DependencyKind,
    pub src_file: FileId,
    pub dst_file: FileId,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexResult {
    pub repo_root: PathBuf,
    pub tu_analyses: Vec<TuAnalysis>,
    pub total_symbols: usize,
    pub total_edges: usize,
    pub complete_edges: usize,
    pub incomplete_edges: usize,
}

impl IndexResult {
    pub fn new(repo_root: PathBuf, tu_analyses: Vec<TuAnalysis>) -> Self {
        let mut total_symbols = 0;
        let mut total_edges = 0;
        let mut complete_edges = 0;
        let mut incomplete_edges = 0;

        let mut seen_symbols = std::collections::HashSet::new();

        for tu in &tu_analyses {
            for sym in &tu.symbols {
                if seen_symbols.insert(sym.id.clone()) {
                    total_symbols += 1;
                }
            }
            total_edges += tu.edges.len();
            for edge in &tu.edges {
                match edge.kind {
                    DependencyKind::Complete => complete_edges += 1,
                    DependencyKind::Incomplete => incomplete_edges += 1,
                }
            }
        }

        Self {
            repo_root,
            tu_analyses,
            total_symbols,
            total_edges,
            complete_edges,
            incomplete_edges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_types_and_index_result_aggregation() {
        let file1 = FileId::from_relative_path("src/foo.cpp");
        let file2 = FileId::from_relative_path("include/foo.h");

        let sym_a = SymbolId::new("A");
        let sym_b = SymbolId::new("B");

        let decl_a = DeclId::new("decl_a_1");

        let symbol_rec_a = SymbolRecord {
            id: sym_a.clone(),
            name: "A".into(),
            kind: SymbolKind::Class,
            owning_file: file2.clone(),
            declarations: vec![decl_a.clone()],
            definition: Some(decl_a),
        };

        let edge = SemanticEdge {
            src_symbol: sym_a,
            dst_symbol: sym_b,
            kind: DependencyKind::Complete,
            src_file: file1.clone(),
            dst_file: file2,
        };

        let tu = TuAnalysis {
            tu_file_id: file1,
            context_name: "Debug|x64".into(),
            symbols: vec![symbol_rec_a],
            declarations: vec![],
            edges: vec![edge],
            references: vec![],
        };

        let result = IndexResult::new(PathBuf::from("/test"), vec![tu]);
        assert_eq!(result.total_symbols, 1);
        assert_eq!(result.total_edges, 1);
        assert_eq!(result.complete_edges, 1);
        assert_eq!(result.incomplete_edges, 0);
    }
}
