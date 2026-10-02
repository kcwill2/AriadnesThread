use ariadne_graph::{ReducedSemanticEdge, ReducedSemanticModel, ReducedSymbolRecord};
use ariadne_index::types::{DependencyKind, SemanticEdge, SymbolId, SymbolRecord};
use ariadne_repo::FileId;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hasher;

pub struct HashPartitioner {
    pub num_partitions: usize,
}

impl HashPartitioner {
    pub fn new(num_partitions: usize) -> Self {
        Self {
            num_partitions: num_partitions.max(1),
        }
    }
    pub fn partition_key<H: std::hash::Hash>(&self, key: &H) -> usize {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.num_partitions
    }
}

struct ReducerInput {
    symbols: Vec<(SymbolRecord, FileId)>,
    edges: Vec<(SemanticEdge, FileId)>,
}

struct PartitionAggregate {
    symbols: BTreeMap<SymbolId, ReducedSymbolRecord>,
    raw_edges: BTreeMap<(SymbolId, SymbolId), DependencyKind>,
    tu_usage: BTreeMap<SymbolId, BTreeSet<FileId>>,
    file_symbols: BTreeMap<FileId, BTreeSet<SymbolId>>,
}

pub struct ShardedReducer {
    pub num_reducers: usize,
}

impl ShardedReducer {
    pub fn new(num_reducers: usize) -> Self {
        Self {
            num_reducers: num_reducers.max(1),
        }
    }

    fn partition_analyses(&self, analyses: &[ariadne_index::TuAnalysis]) -> Vec<ReducerInput> {
        let partitioner = HashPartitioner::new(self.num_reducers);

        let mut partitions: Vec<ReducerInput> = (0..self.num_reducers)
            .map(|_| ReducerInput {
                symbols: Vec::new(),
                edges: Vec::new(),
            })
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

        partitions
    }

    async fn reduce_partition(input: ReducerInput) -> PartitionAggregate {
        let mut symbols: BTreeMap<SymbolId, ReducedSymbolRecord> = BTreeMap::new();
        let mut raw_edges: BTreeMap<(SymbolId, SymbolId), DependencyKind> = BTreeMap::new();
        let mut tu_usage: BTreeMap<SymbolId, BTreeSet<FileId>> = BTreeMap::new();
        let mut file_symbols: BTreeMap<FileId, BTreeSet<SymbolId>> = BTreeMap::new();

        for (sym, tu_file) in input.symbols {
            let entry = symbols
                .entry(sym.id.clone())
                .or_insert_with(|| ReducedSymbolRecord {
                    id: sym.id.clone(),
                    name: sym.name.clone(),
                    kind: sym.kind,
                    defining_file: None,
                    declarations: Vec::new(),
                    definition: None,
                    declaring_files: Vec::new(),
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
            file_symbols
                .entry(sym.owning_file)
                .or_default()
                .insert(sym.id);
        }

        for (edge, tu_file) in input.edges {
            tu_usage
                .entry(edge.src_symbol.clone())
                .or_default()
                .insert(tu_file.clone());
            tu_usage
                .entry(edge.dst_symbol.clone())
                .or_default()
                .insert(tu_file);

            let key = (edge.src_symbol, edge.dst_symbol);
            match raw_edges.get(&key) {
                Some(&DependencyKind::Complete) => {}
                _ => {
                    raw_edges.insert(key, edge.kind);
                }
            }
        }

        PartitionAggregate {
            symbols,
            raw_edges,
            tu_usage,
            file_symbols,
        }
    }

    async fn merge_partition_aggregates(
        handles: Vec<tokio::task::JoinHandle<PartitionAggregate>>,
    ) -> PartitionAggregate {
        let mut merged = PartitionAggregate {
            symbols: BTreeMap::new(),
            raw_edges: BTreeMap::new(),
            tu_usage: BTreeMap::new(),
            file_symbols: BTreeMap::new(),
        };

        for handle in handles {
            let part = handle
                .await
                .expect("a panicked/cancelled partition task leaves no complete shard to merge");
            merged.symbols.extend(part.symbols);
            for (k, v) in part.raw_edges {
                match merged.raw_edges.get(&k) {
                    Some(&DependencyKind::Complete) => {}
                    _ => {
                        merged.raw_edges.insert(k, v);
                    }
                }
            }
            for (k, v) in part.tu_usage {
                merged.tu_usage.entry(k).or_default().extend(v);
            }
            for (k, v) in part.file_symbols {
                merged.file_symbols.entry(k).or_default().extend(v);
            }
        }

        merged
    }

    pub async fn reduce(
        &self,
        repo_root: std::path::PathBuf,
        analyses: &[ariadne_index::TuAnalysis],
    ) -> ariadne_graph::ReducedSemanticModel {
        let handles: Vec<tokio::task::JoinHandle<PartitionAggregate>> = self
            .partition_analyses(analyses)
            .into_iter()
            .map(|input| tokio::spawn(Self::reduce_partition(input)))
            .collect();

        let merged = Self::merge_partition_aggregates(handles).await;

        let mut final_edges: Vec<ReducedSemanticEdge> = merged
            .raw_edges
            .into_iter()
            .map(|((src, dst), kind)| ReducedSemanticEdge {
                src_symbol: src,
                dst_symbol: dst,
                kind,
            })
            .collect();
        final_edges.sort_by(|a, b| {
            a.src_symbol
                .cmp(&b.src_symbol)
                .then_with(|| a.dst_symbol.cmp(&b.dst_symbol))
        });

        let tu_usage: BTreeMap<SymbolId, Vec<FileId>> = merged
            .tu_usage
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();

        let file_symbols: BTreeMap<FileId, Vec<SymbolId>> = merged
            .file_symbols
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().collect()))
            .collect();

        ReducedSemanticModel {
            repo_root,
            symbols: merged.symbols,
            edges: final_edges,
            tu_usage,
            file_symbols,

            extends_edges: Vec::new(),
            sealed_symbols: Vec::new(),
        }
    }
}

#[cfg(test)]
mod reduce_characterization {
    //! Characterization: locks the exact merged output of ShardedReducer::reduce for a
    //! multi-partition input, captured from current behavior before decomposition.
    //! The sharded path deliberately differs from ReducedSemanticModel::reduce and this
    //! test locks those differences as-is (do NOT "fix" them here):
    //!   - definitions are first-wins by analysis order (no Class/Struct precedence rule);
    //!   - kind/name of a multi-TU symbol come from the first record only;
    //!   - tu.references contribute nothing to tu_usage (the direct path adds them).
    use super::*;
    use ariadne_graph::{ReducedSemanticEdge, ReducedSemanticModel, ReducedSymbolRecord};
    use ariadne_index::types::{
        DeclId, DependencyKind, FileLocation, SemanticEdge, SymbolId, SymbolKind, SymbolRecord,
        SymbolReference, TuAnalysis,
    };
    use ariadne_repo::FileId;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn ch_file(rel: &str) -> FileId {
        FileId::from_relative_path(rel)
    }

    fn ch_sorted(mut ids: Vec<u64>) -> Vec<FileId> {
        ids.sort();
        ids.into_iter().map(FileId).collect()
    }

    fn char_tu_alpha() -> TuAnalysis {
        let hdr = ch_file("include/shard_char.h");
        let alpha = ch_file("src/shard_char_alpha.cpp");
        let beta = ch_file("src/shard_char_beta.cpp");
        let gamma = ch_file("src/shard_char_gamma.cpp");
        let shared = SymbolId::new("ChSymShared");
        let alpha_sym = SymbolId::new("ChSymAlpha");
        let beta_sym = SymbolId::new("ChSymBeta");
        let edge_target = SymbolId::new("ChSymEdgeTarget");

        TuAnalysis {
            tu_file_id: alpha,
            context_name: "char".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![
                SymbolRecord {
                    id: shared.clone(),
                    name: "SharedRecord".into(),
                    kind: SymbolKind::Function,
                    owning_file: hdr,
                    declarations: vec![DeclId::new("ch_decl_3"), DeclId::new("ch_decl_1")],
                    definition: Some(DeclId::new("ch_def_shared_first")),

                    modifiers: Vec::new(),
                },
                SymbolRecord {
                    id: alpha_sym.clone(),
                    name: "AlphaRec".into(),
                    kind: SymbolKind::Class,
                    owning_file: hdr,
                    declarations: vec![DeclId::new("ch_decl_a1")],
                    definition: None,

                    modifiers: Vec::new(),
                },
                SymbolRecord {
                    id: beta_sym.clone(),
                    name: "BetaRec".into(),
                    kind: SymbolKind::Function,
                    owning_file: gamma,
                    declarations: vec![],
                    definition: Some(DeclId::new("ch_def_beta")),

                    modifiers: Vec::new(),
                },
            ],
            declarations: vec![],
            edges: vec![
                SemanticEdge {
                    src_symbol: alpha_sym,
                    dst_symbol: edge_target.clone(),
                    kind: DependencyKind::Incomplete,
                    src_file: alpha,
                    dst_file: beta,
                },
                SemanticEdge {
                    src_symbol: beta_sym,
                    dst_symbol: edge_target,
                    kind: DependencyKind::Complete,
                    src_file: alpha,
                    dst_file: beta,
                },
            ],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
        }
    }

    fn char_tu_beta() -> TuAnalysis {
        let hdr = ch_file("include/shard_char.h");
        let beta = ch_file("src/shard_char_beta.cpp");
        let shared = SymbolId::new("ChSymShared");
        let alpha_sym = SymbolId::new("ChSymAlpha");
        let gamma_sym = SymbolId::new("ChSymGamma");
        let edge_target = SymbolId::new("ChSymEdgeTarget");

        TuAnalysis {
            tu_file_id: beta,
            context_name: "char".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![
                // Second record for the shared symbol: Class kind and a different
                // definition arrive AFTER the Function record from tu alpha.
                SymbolRecord {
                    id: shared.clone(),
                    name: "SharedRecordLate".into(),
                    kind: SymbolKind::Class,
                    owning_file: hdr,
                    declarations: vec![DeclId::new("ch_decl_1"), DeclId::new("ch_decl_2")],
                    definition: Some(DeclId::new("ch_def_shared_second")),

                    modifiers: Vec::new(),
                },
                SymbolRecord {
                    id: gamma_sym.clone(),
                    name: "GammaRec".into(),
                    kind: SymbolKind::Function,
                    owning_file: beta,
                    declarations: vec![DeclId::new("ch_decl_g1")],
                    definition: None,

                    modifiers: Vec::new(),
                },
            ],
            declarations: vec![],
            edges: vec![
                SemanticEdge {
                    src_symbol: alpha_sym,
                    dst_symbol: edge_target.clone(),
                    kind: DependencyKind::Complete,
                    src_file: beta,
                    dst_file: beta,
                },
                SemanticEdge {
                    src_symbol: gamma_sym,
                    dst_symbol: edge_target,
                    kind: DependencyKind::Incomplete,
                    src_file: beta,
                    dst_file: beta,
                },
            ],
            references: vec![SymbolReference {
                symbol_id: SymbolId::new("ChSymReferenced"),
                file_id: beta,
                location: FileLocation {
                    file_id: beta,
                    line: 1,
                    column: 1,
                },
                is_write: false,
            }],
            call_sites: vec![],

            extends_edges: Vec::new(),
        }
    }

    fn char_tu_gamma() -> TuAnalysis {
        let beta = ch_file("src/shard_char_beta.cpp");
        let gamma = ch_file("src/shard_char_gamma.cpp");

        TuAnalysis {
            tu_file_id: gamma,
            context_name: "char".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![],
            declarations: vec![],
            edges: vec![SemanticEdge {
                src_symbol: SymbolId::new("ChSymBeta"),
                dst_symbol: SymbolId::new("ChSymEdgeTarget"),
                kind: DependencyKind::Incomplete,
                src_file: gamma,
                dst_file: beta,
            }],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
        }
    }

    fn char_input() -> Vec<TuAnalysis> {
        vec![char_tu_alpha(), char_tu_beta(), char_tu_gamma()]
    }

    fn char_expected_symbols() -> BTreeMap<SymbolId, ReducedSymbolRecord> {
        let hdr = ch_file("include/shard_char.h");
        let gamma = ch_file("src/shard_char_gamma.cpp");
        let shared = SymbolId::new("ChSymShared");
        let alpha_sym = SymbolId::new("ChSymAlpha");
        let beta_sym = SymbolId::new("ChSymBeta");
        let gamma_sym = SymbolId::new("ChSymGamma");

        let mut symbols = BTreeMap::new();
        symbols.insert(
            alpha_sym.clone(),
            ReducedSymbolRecord {
                id: alpha_sym.clone(),
                name: "AlphaRec".into(),
                kind: SymbolKind::Class,
                defining_file: None,
                declarations: vec![DeclId::new("ch_decl_a1")],
                definition: None,
                declaring_files: vec![],
            },
        );
        symbols.insert(
            beta_sym.clone(),
            ReducedSymbolRecord {
                id: beta_sym.clone(),
                name: "BetaRec".into(),
                kind: SymbolKind::Function,
                defining_file: Some(gamma),
                declarations: vec![],
                definition: Some(DeclId::new("ch_def_beta")),
                declaring_files: vec![],
            },
        );
        symbols.insert(
            gamma_sym.clone(),
            ReducedSymbolRecord {
                id: gamma_sym,
                name: "GammaRec".into(),
                kind: SymbolKind::Function,
                defining_file: None,
                declarations: vec![DeclId::new("ch_decl_g1")],
                definition: None,
                declaring_files: vec![],
            },
        );
        // First-wins merge: Function kind/name/definition from the first TU record,
        // declarations deduplicated and sorted across both TUs.
        symbols.insert(
            shared.clone(),
            ReducedSymbolRecord {
                id: shared,
                name: "SharedRecord".into(),
                kind: SymbolKind::Function,
                defining_file: Some(hdr),
                declarations: vec![
                    DeclId::new("ch_decl_1"),
                    DeclId::new("ch_decl_2"),
                    DeclId::new("ch_decl_3"),
                ],
                definition: Some(DeclId::new("ch_def_shared_first")),
                declaring_files: vec![],
            },
        );
        symbols
    }

    fn char_expected_tu_usage() -> BTreeMap<SymbolId, Vec<FileId>> {
        let alpha = ch_file("src/shard_char_alpha.cpp");
        let beta = ch_file("src/shard_char_beta.cpp");
        let gamma = ch_file("src/shard_char_gamma.cpp");
        let shared = SymbolId::new("ChSymShared");
        let alpha_sym = SymbolId::new("ChSymAlpha");
        let beta_sym = SymbolId::new("ChSymBeta");
        let gamma_sym = SymbolId::new("ChSymGamma");
        let edge_target = SymbolId::new("ChSymEdgeTarget");

        let mut tu_usage = BTreeMap::new();
        tu_usage.insert(alpha_sym, ch_sorted(vec![alpha.0, beta.0]));
        tu_usage.insert(beta_sym, ch_sorted(vec![alpha.0, gamma.0]));
        tu_usage.insert(gamma_sym, ch_sorted(vec![beta.0]));
        // Edge target has no SymbolRecord anywhere, yet still accumulates usage
        // from every TU whose edges reference it as dst.
        tu_usage.insert(edge_target, ch_sorted(vec![alpha.0, beta.0, gamma.0]));
        tu_usage.insert(shared, ch_sorted(vec![alpha.0, beta.0]));
        // ChSymReferenced appears only in tu.references and is absent from
        // tu_usage: the sharded path ignores references.
        tu_usage
    }

    fn char_expected() -> ReducedSemanticModel {
        let hdr = ch_file("include/shard_char.h");
        let beta = ch_file("src/shard_char_beta.cpp");
        let gamma = ch_file("src/shard_char_gamma.cpp");
        let shared = SymbolId::new("ChSymShared");
        let alpha_sym = SymbolId::new("ChSymAlpha");
        let beta_sym = SymbolId::new("ChSymBeta");
        let gamma_sym = SymbolId::new("ChSymGamma");
        let edge_target = SymbolId::new("ChSymEdgeTarget");

        let mut file_symbols = BTreeMap::new();
        file_symbols.insert(beta, vec![SymbolId::new("ChSymGamma")]);
        file_symbols.insert(gamma, vec![SymbolId::new("ChSymBeta")]);
        file_symbols.insert(hdr, vec![alpha_sym.clone(), shared]);

        ReducedSemanticModel {
            repo_root: PathBuf::from("/char/repo"),
            symbols: char_expected_symbols(),
            edges: vec![
                ReducedSemanticEdge {
                    src_symbol: alpha_sym,
                    dst_symbol: edge_target.clone(),
                    kind: DependencyKind::Complete,
                },
                ReducedSemanticEdge {
                    src_symbol: beta_sym,
                    dst_symbol: edge_target.clone(),
                    kind: DependencyKind::Complete,
                },
                ReducedSemanticEdge {
                    src_symbol: gamma_sym,
                    dst_symbol: edge_target,
                    kind: DependencyKind::Incomplete,
                },
            ],
            tu_usage: char_expected_tu_usage(),
            file_symbols,

            extends_edges: Vec::new(),
            sealed_symbols: Vec::new(),
        }
    }

    #[tokio::test]
    async fn reduce_locks_full_merged_model_across_partition_counts() {
        for num_reducers in [1usize, 3, 8] {
            let model = ShardedReducer::new(num_reducers)
                .reduce(PathBuf::from("/char/repo"), &char_input())
                .await;
            assert_eq!(
                model,
                char_expected(),
                "sharded reduce output must match the locked model for {num_reducers} reducers"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_graph::ReducedSemanticModel;
    use ariadne_index::types::{
        DeclId, DependencyKind, IndexResult, SemanticEdge, SymbolId, SymbolKind, SymbolRecord,
        TuAnalysis,
    };
    use ariadne_repo::FileId;
    use std::path::PathBuf;

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

            modifiers: Vec::new(),
        };

        let sym2 = SymbolRecord {
            id: SymbolId("SymB".into()),
            name: "SymB".into(),
            kind: SymbolKind::Function,
            owning_file: f2.clone(),
            declarations: vec![DeclId("decl_b".into())],
            definition: Some(DeclId("def_b".into())),

            modifiers: Vec::new(),
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
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![sym1.clone()],
            declarations: vec![],
            edges: vec![edge.clone()],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
        };

        let tu2 = TuAnalysis {
            tu_file_id: f2.clone(),
            context_name: "default".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![sym2.clone()],
            declarations: vec![],
            edges: vec![],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
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
        let sharded_model_1 = ShardedReducer::new(1)
            .reduce(repo_root.clone(), &analyses)
            .await;

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
