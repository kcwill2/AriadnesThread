use std::hash::Hasher;

pub struct HashPartitioner {
    pub num_partitions: usize,
}

impl HashPartitioner {
    pub fn new(num_partitions: usize) -> Self {
        Self { num_partitions: num_partitions.max(1) }
    }
    pub fn partition_key<H: std::hash::Hash>(&self, key: &H) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.num_partitions
    }
}

pub struct ShardedReducer {
    pub num_reducers: usize,
}

impl ShardedReducer {
    pub fn new(num_reducers: usize) -> Self {
        Self { num_reducers: num_reducers.max(1) }
    }

    pub async fn reduce(&self, repo_root: std::path::PathBuf, analyses: &[ariadne_index::TuAnalysis]) -> ariadne_graph::ReducedSemanticModel {
        use std::collections::{BTreeMap, BTreeSet};
        use ariadne_graph::{ReducedSemanticModel, ReducedSymbolRecord, ReducedSemanticEdge};
        use ariadne_index::types::{SymbolId, DependencyKind};
        use ariadne_repo::FileId;

        let num_reducers = self.num_reducers;
        let partitioner = HashPartitioner::new(num_reducers);

        struct ReducerInput {
            symbols: Vec<(ariadne_index::types::SymbolRecord, FileId)>,
            edges: Vec<(ariadne_index::types::SemanticEdge, FileId)>,
        }

        let mut partitions: Vec<ReducerInput> = (0..num_reducers)
            .map(|_| ReducerInput { symbols: Vec::new(), edges: Vec::new() })
            .collect();

        for tu in analyses {
            let tu_file = tu.tu_file_id.clone();
            for sym in &tu.symbols {
                let p = partitioner.partition_key(&sym.id);
                partitions[p].symbols.push((sym.clone(), tu_file.clone()));
            }
            for edge in &tu.edges {
                let p = partitioner.partition_key(&edge.src_symbol);
                partitions[p].edges.push((edge.clone(), tu_file.clone()));
            }
        }

        let mut handles = Vec::new();
        for input in partitions {
            let handle = tokio::spawn(async move {
                let mut symbols: BTreeMap<SymbolId, ReducedSymbolRecord> = BTreeMap::new();
                let mut raw_edges: BTreeMap<(SymbolId, SymbolId), DependencyKind> = BTreeMap::new();
                let mut tu_usage: BTreeMap<SymbolId, BTreeSet<FileId>> = BTreeMap::new();
                let mut file_symbols: BTreeMap<FileId, BTreeSet<SymbolId>> = BTreeMap::new();

                for (sym, tu_file) in input.symbols {
                    let entry = symbols.entry(sym.id.clone()).or_insert_with(|| ReducedSymbolRecord {
                        id: sym.id.clone(),
                        name: sym.name.clone(),
                        kind: sym.kind,
                        defining_file: None,
                        declarations: Vec::new(),
                        definition: None,
                    });

                    for decl in sym.declarations {
                        if !entry.declarations.contains(&decl) {
                            entry.declarations.push(decl);
                        }
                    }
                    entry.declarations.sort();

                    if let Some(def) = sym.definition {
                        if entry.definition.is_none() {
                            entry.definition = Some(def);
                            entry.defining_file = Some(sym.owning_file.clone());
                        }
                    }

                    tu_usage.entry(sym.id.clone()).or_default().insert(tu_file);
                    file_symbols.entry(sym.owning_file).or_default().insert(sym.id);
                }

                for (edge, tu_file) in input.edges {
                    tu_usage.entry(edge.src_symbol.clone()).or_default().insert(tu_file.clone());
                    tu_usage.entry(edge.dst_symbol.clone()).or_default().insert(tu_file);

                    let key = (edge.src_symbol, edge.dst_symbol);
                    match raw_edges.get(&key) {
                        Some(&DependencyKind::Complete) => {}
                        _ => {
                            raw_edges.insert(key, edge.kind);
                        }
                    }
                }

                (symbols, raw_edges, tu_usage, file_symbols)
            });
            handles.push(handle);
        }

        let mut final_symbols = BTreeMap::new();
        let mut final_raw_edges = BTreeMap::new();
        let mut final_tu_usage: BTreeMap<SymbolId, BTreeSet<FileId>> = BTreeMap::new();
        let mut final_file_symbols: BTreeMap<FileId, BTreeSet<SymbolId>> = BTreeMap::new();

        for handle in handles {
            let (syms, edges, tu_u, file_s) = handle.await.unwrap();
            final_symbols.extend(syms);
            for (k, v) in edges {
                match final_raw_edges.get(&k) {
                    Some(&DependencyKind::Complete) => {}
                    _ => {
                        final_raw_edges.insert(k, v);
                    }
                }
            }
            for (k, v) in tu_u {
                final_tu_usage.entry(k).or_default().extend(v);
            }
            for (k, v) in file_s {
                final_file_symbols.entry(k).or_default().extend(v);
            }
        }

        let mut final_edges: Vec<ReducedSemanticEdge> = final_raw_edges
            .into_iter()
            .map(|((src, dst), kind)| ReducedSemanticEdge {
                src_symbol: src,
                dst_symbol: dst,
                kind,
            })
            .collect();
        final_edges.sort_by(|a, b| a.src_symbol.cmp(&b.src_symbol).then_with(|| a.dst_symbol.cmp(&b.dst_symbol)));

        let tu_usage: BTreeMap<SymbolId, Vec<FileId>> = final_tu_usage
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();

        let file_symbols: BTreeMap<FileId, Vec<SymbolId>> = final_file_symbols
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();

        ReducedSemanticModel {
            repo_root,
            symbols: final_symbols,
            edges: final_edges,
            tu_usage,
            file_symbols,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use ariadne_index::types::{TuAnalysis, IndexResult, SymbolRecord, SymbolKind, SemanticEdge, SymbolId, DeclId, DependencyKind};
    use ariadne_graph::ReducedSemanticModel;
    use ariadne_repo::FileId;

    #[test]
    fn test_hash_partitioner_determinism() {
        let partitioner = HashPartitioner::new(4);
        let key = "some_node_id";
        let p1 = partitioner.partition_key(&key);
        let p2 = partitioner.partition_key(&key);
        assert_eq!(p1, p2);
    }

    #[tokio::test]
    async fn test_sharded_reducer_equivalence() {
        let repo_root = PathBuf::from("/dummy/repo");

        let f1 = FileId::from_relative_path("src/file1.cpp");
        let f2 = FileId::from_relative_path("src/file2.cpp");

        let sym1 = SymbolRecord {
            id: SymbolId("SymA".into()),
            name: "SymA".into(),
            kind: SymbolKind::Class,
            owning_file: f1.clone(),
            declarations: vec![DeclId("decl_a".into())],
            definition: Some(DeclId("def_a".into())),
        };

        let sym2 = SymbolRecord {
            id: SymbolId("SymB".into()),
            name: "SymB".into(),
            kind: SymbolKind::Function,
            owning_file: f2.clone(),
            declarations: vec![DeclId("decl_b".into())],
            definition: Some(DeclId("def_b".into())),
        };

        let edge = SemanticEdge {
            src_symbol: SymbolId("SymA".into()),
            dst_symbol: SymbolId("SymB".into()),
            kind: DependencyKind::Complete,
            src_file: f1.clone(),
            dst_file: f2.clone(),
        };

        let tu1 = TuAnalysis {
            tu_file_id: f1.clone(),
            context_name: "default".into(),
            symbols: vec![sym1.clone()],
            declarations: vec![],
            edges: vec![edge.clone()],
            references: vec![],
            call_sites: vec![],
        };

        let tu2 = TuAnalysis {
            tu_file_id: f2.clone(),
            context_name: "default".into(),
            symbols: vec![sym2.clone()],
            declarations: vec![],
            edges: vec![],
            references: vec![],
            call_sites: vec![],
        };

        let analyses = vec![tu1, tu2];

        // Direct reduction
        let index_res = IndexResult {
            tu_analyses: analyses.clone(),
            repo_root: repo_root.clone(),
            total_symbols: 2,
            total_edges: 1,
            complete_edges: 1,
            incomplete_edges: 0,
        };
        let direct_model = ReducedSemanticModel::reduce(&index_res);

        // Sharded reduction with 1 reducer
        let sharded_model_1 = ShardedReducer::new(1).reduce(repo_root.clone(), &analyses).await;

        // Sharded reduction with 4 reducers
        let sharded_model_4 = ShardedReducer::new(4).reduce(repo_root, &analyses).await;

        assert_eq!(direct_model.symbols, sharded_model_1.symbols);
        assert_eq!(direct_model.edges, sharded_model_1.edges);
        assert_eq!(direct_model.tu_usage, sharded_model_1.tu_usage);
        assert_eq!(direct_model.file_symbols, sharded_model_1.file_symbols);

        assert_eq!(direct_model.symbols, sharded_model_4.symbols);
        assert_eq!(direct_model.edges, sharded_model_4.edges);
        assert_eq!(direct_model.tu_usage, sharded_model_4.tu_usage);
        assert_eq!(direct_model.file_symbols, sharded_model_4.file_symbols);
    }
}
