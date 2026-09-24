use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use ariadne_repo::FileId;
use ariadne_index::types::{SymbolId, DeclId, SymbolKind, DependencyKind, IndexResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReducedSymbolRecord {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub defining_file: Option<FileId>,
    pub declarations: Vec<DeclId>,
    pub definition: Option<DeclId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReducedSemanticEdge {
    pub src_symbol: SymbolId,
    pub dst_symbol: SymbolId,
    pub kind: DependencyKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuUsage {
    pub symbol_id: SymbolId,
    pub tu_file_ids: Vec<FileId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSymbols {
    pub file_id: FileId,
    pub symbols: Vec<SymbolId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReducedSemanticModel {
    pub repo_root: PathBuf,
    pub symbols: BTreeMap<SymbolId, ReducedSymbolRecord>,
    pub edges: Vec<ReducedSemanticEdge>,
    pub tu_usage: BTreeMap<SymbolId, Vec<FileId>>,
    pub file_symbols: BTreeMap<FileId, Vec<SymbolId>>,
}

impl ReducedSemanticModel {
    pub fn reduce(index_result: &IndexResult) -> Self {
        let mut symbols: BTreeMap<SymbolId, ReducedSymbolRecord> = BTreeMap::new();
        let mut edges: Vec<ReducedSemanticEdge> = Vec::new();
        let mut tu_usage: BTreeMap<SymbolId, BTreeSet<FileId>> = BTreeMap::new();
        let mut file_symbols: BTreeMap<FileId, BTreeSet<SymbolId>> = BTreeMap::new();
        let mut raw_edges: BTreeMap<(SymbolId, SymbolId), DependencyKind> = BTreeMap::new();

        for tu in &index_result.tu_analyses {
            let tu_file = tu.tu_file_id;

            // Symbols
            for sym in &tu.symbols {
                let entry = symbols.entry(sym.id.clone()).or_insert_with(|| ReducedSymbolRecord {
                    id: sym.id.clone(),
                    name: sym.name.clone(),
                    kind: sym.kind,
                    defining_file: None,
                    declarations: Vec::new(),
                    definition: None,
                });

                for decl in &sym.declarations {
                    if !entry.declarations.contains(decl) {
                        entry.declarations.push(decl.clone());
                    }
                }
                // Sort to maintain determinism
                entry.declarations.sort();

                if let Some(def) = &sym.definition {
                    if entry.definition.is_none() {
                        entry.definition = Some(def.clone());
                        entry.defining_file = Some(sym.owning_file); // Owning file is the defining file usually
                    }
                }

                tu_usage.entry(sym.id.clone()).or_default().insert(tu_file);
                file_symbols.entry(sym.owning_file).or_default().insert(sym.id.clone());
            }

            // Edges
            for edge in &tu.edges {
                tu_usage.entry(edge.src_symbol.clone()).or_default().insert(tu_file);
                tu_usage.entry(edge.dst_symbol.clone()).or_default().insert(tu_file);

                let key = (edge.src_symbol.clone(), edge.dst_symbol.clone());
                match raw_edges.get(&key) {
                    Some(&DependencyKind::Complete) => {
                        // Complete wins, do nothing
                    }
                    _ => {
                        raw_edges.insert(key, edge.kind);
                    }
                }
            }

            // References
            for ref_item in &tu.references {
                tu_usage.entry(ref_item.symbol_id.clone()).or_default().insert(tu_file);
            }
        }

        // Convert raw_edges to deterministic sorted vec
        for ((src, dst), kind) in raw_edges {
            edges.push(ReducedSemanticEdge {
                src_symbol: src,
                dst_symbol: dst,
                kind,
            });
        }
        
        let mut tu_usage_vec = BTreeMap::new();
        for (sym, files) in tu_usage {
            let mut file_vec: Vec<FileId> = files.into_iter().collect();
            file_vec.sort();
            tu_usage_vec.insert(sym, file_vec);
        }

        let mut file_symbols_vec = BTreeMap::new();
        for (file, syms) in file_symbols {
            let mut sym_vec: Vec<SymbolId> = syms.into_iter().collect();
            sym_vec.sort();
            file_symbols_vec.insert(file, sym_vec);
        }

        Self {
            repo_root: index_result.repo_root.clone(),
            symbols,
            edges,
            tu_usage: tu_usage_vec,
            file_symbols: file_symbols_vec,
        }
    }

    pub fn find_symbol(&self, name_or_id: &str) -> Option<&ReducedSymbolRecord> {
        if let Some(record) = self.symbols.get(&SymbolId::new(name_or_id)) {
            return Some(record);
        }
        self.symbols.values().find(|record| record.name == name_or_id || record.id.0 == name_or_id)
    }

    pub fn get_dependencies(&self, sym_id: &SymbolId) -> Vec<&ReducedSemanticEdge> {
        self.edges.iter().filter(|edge| edge.src_symbol == *sym_id).collect()
    }

    pub fn get_users(&self, sym_id: &SymbolId) -> Vec<FileId> {
        self.tu_usage.get(sym_id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_index::types::{FileLocation, TuAnalysis, SymbolRecord, SemanticEdge, SymbolReference};

    fn create_dummy_index() -> IndexResult {
        let f1 = FileId::from_relative_path("a.cpp");
        let f2 = FileId::from_relative_path("b.cpp");
        let f3 = FileId::from_relative_path("c.h");

        let sym_a = SymbolId::new("A");
        let sym_b = SymbolId::new("B");
        let sym_c = SymbolId::new("C");

        let decl_a1 = DeclId::new("decl_a1");
        let decl_a2 = DeclId::new("decl_a2");

        let tu1 = TuAnalysis {
            tu_file_id: f1.clone(),
            context_name: "test".into(),
            symbols: vec![
                SymbolRecord {
                    id: sym_a.clone(),
                    name: "A".into(),
                    kind: SymbolKind::Class,
                    owning_file: f3.clone(),
                    declarations: vec![decl_a1.clone()],
                    definition: None,
                },
                SymbolRecord {
                    id: sym_c.clone(),
                    name: "C".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: Some(DeclId::new("decl_c")),
                }
            ],
            declarations: vec![],
            edges: vec![
                SemanticEdge {
                    src_symbol: sym_a.clone(),
                    dst_symbol: sym_b.clone(),
                    kind: DependencyKind::Incomplete,
                    src_file: f1.clone(),
                    dst_file: f3.clone(),
                }
            ],
            references: vec![],
        };

        let tu2 = TuAnalysis {
            tu_file_id: f2.clone(),
            context_name: "test".into(),
            symbols: vec![
                SymbolRecord {
                    id: sym_a.clone(),
                    name: "A".into(),
                    kind: SymbolKind::Class,
                    owning_file: f3.clone(),
                    declarations: vec![decl_a2.clone()],
                    definition: Some(decl_a2.clone()),
                }
            ],
            declarations: vec![],
            edges: vec![
                SemanticEdge {
                    src_symbol: sym_a.clone(),
                    dst_symbol: sym_b.clone(),
                    kind: DependencyKind::Complete,
                    src_file: f2.clone(),
                    dst_file: f3.clone(),
                }
            ],
            references: vec![
                SymbolReference {
                    symbol_id: sym_c.clone(),
                    file_id: f2.clone(),
                    location: FileLocation { file_id: f2.clone(), line: 1, column: 1 },
                    is_write: false,
                }
            ],
        };

        IndexResult::new(PathBuf::from("/test"), vec![tu1, tu2])
    }

    #[test]
    fn test_symbol_identity_resolution_and_aggregation() {
        let index = create_dummy_index();
        let reduced = ReducedSemanticModel::reduce(&index);

        let sym_a = reduced.symbols.get(&SymbolId::new("A")).unwrap();
        assert_eq!(sym_a.declarations.len(), 2);
        assert!(sym_a.declarations.contains(&DeclId::new("decl_a1")));
        assert!(sym_a.declarations.contains(&DeclId::new("decl_a2")));
        assert_eq!(sym_a.definition, Some(DeclId::new("decl_a2")));
        assert_eq!(sym_a.defining_file, Some(FileId::from_relative_path("c.h"))); // owning_file is c.h

        let sym_c = reduced.symbols.get(&SymbolId::new("C")).unwrap();
        assert_eq!(sym_c.definition, Some(DeclId::new("decl_c")));
        assert_eq!(sym_c.defining_file, Some(FileId::from_relative_path("a.cpp")));
    }

    #[test]
    fn test_dependency_deduplication_and_precedence() {
        let index = create_dummy_index();
        let reduced = ReducedSemanticModel::reduce(&index);

        assert_eq!(reduced.edges.len(), 1);
        let edge = &reduced.edges[0];
        assert_eq!(edge.src_symbol, SymbolId::new("A"));
        assert_eq!(edge.dst_symbol, SymbolId::new("B"));
        assert_eq!(edge.kind, DependencyKind::Complete); // Complete wins over Incomplete
    }

    #[test]
    fn test_tu_usage_aggregation() {
        let index = create_dummy_index();
        let reduced = ReducedSemanticModel::reduce(&index);

        let f1 = FileId::from_relative_path("a.cpp");
        let f2 = FileId::from_relative_path("b.cpp");

        let usages_a = reduced.tu_usage.get(&SymbolId::new("A")).unwrap();
        assert!(usages_a.contains(&f1));
        assert!(usages_a.contains(&f2));

        let usages_b = reduced.tu_usage.get(&SymbolId::new("B")).unwrap();
        assert!(usages_b.contains(&f1));
        assert!(usages_b.contains(&f2));

        let usages_c = reduced.tu_usage.get(&SymbolId::new("C")).unwrap();
        assert!(usages_c.contains(&f1)); // declared in f1
        assert!(usages_c.contains(&f2)); // referenced in f2
    }

    #[test]
    fn test_model_determinism() {
        let index = create_dummy_index();
        let reduced1 = ReducedSemanticModel::reduce(&index);
        let reduced2 = ReducedSemanticModel::reduce(&index);
        assert_eq!(reduced1, reduced2);
    }

    #[test]
    fn test_query_helpers() {
        let index = create_dummy_index();
        let reduced = ReducedSemanticModel::reduce(&index);

        let sym_a = reduced.find_symbol("A").unwrap();
        assert_eq!(sym_a.name, "A");
        
        let sym_c_by_name = reduced.find_symbol("C").unwrap();
        assert_eq!(sym_c_by_name.name, "C");

        let deps_a = reduced.get_dependencies(&SymbolId::new("A"));
        assert_eq!(deps_a.len(), 1);
        assert_eq!(deps_a[0].dst_symbol, SymbolId::new("B"));

        let users_a = reduced.get_users(&SymbolId::new("A"));
        assert_eq!(users_a.len(), 2);
    }
}
