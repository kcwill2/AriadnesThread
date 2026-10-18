use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use ariadne_index::types::{
    DeclId, DependencyKind, IndexResult, SemanticEdge, SymbolId, SymbolKind, SymbolRecord,
};
use ariadne_repo::FileId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReducedSymbolRecord {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    /// File hosting the accepted definition declaration (plan §3.6.1). Falls
    /// back to the legacy `SymbolRecord::owning_file` ONLY when the defining
    /// (symbol, decl) pair is absent from its TU's declaration registry,
    /// carries only the `FileId(0)` sentinel (`FileLocation::default()`)
    /// location, or resolves ambiguously to multiple files (cross-file
    /// `decl_<counter>` id collisions).
    pub defining_file: Option<FileId>,
    pub declarations: Vec<DeclId>,
    pub definition: Option<DeclId>,
    /// Files hosting ANY merged declaration with a usable location (plan
    /// §3.6.1), deduplicated and sorted.
    pub declaring_files: Vec<FileId>,
}

/// TU-scoped registry: files hosting each (symbol, decl) pair declared in
/// the TU analysis; sentinel `FileId(0)` locations are filtered at build.
type DeclLocations<'a> = BTreeMap<(&'a SymbolId, &'a DeclId), Vec<FileId>>;

/// Files hosting the given (symbol, decl) pair in this TU's declaration
/// registry; sentinel `FileId(0)` locations are filtered at registry build.
fn decl_files_for<'x, 'i>(
    symbol_id: &'i SymbolId,
    decl: &'i DeclId,
    decl_locations: &'x DeclLocations<'i>,
) -> &'x [FileId] {
    decl_locations
        .get(&(symbol_id, decl))
        .map(Vec::as_slice)
        .unwrap_or(&[])
}

/// Provenance rule (plan §3.6.1): the defining file is the file of the
/// definition declaration's own location. `owning_file` is the documented
/// fallback (M18.9) when the (symbol, decl) pair is unknown to the TU's
/// declaration registry, carries only the `FileId(0)` sentinel location, or
/// resolves ambiguously (the parser's per-file `decl_<counter>` ids collide
/// across files, so a pair can map to several files after TU merge).
fn defining_file_for<'i>(
    symbol_id: &'i SymbolId,
    def: &'i DeclId,
    decl_locations: &DeclLocations<'i>,
    owning_file: FileId,
) -> FileId {
    match decl_files_for(symbol_id, def, decl_locations) {
        [single] => *single,
        _ => owning_file,
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReducedSemanticModel {
    pub repo_root: PathBuf,
    pub symbols: BTreeMap<SymbolId, ReducedSymbolRecord>,
    pub edges: Vec<ReducedSemanticEdge>,
    pub tu_usage: BTreeMap<SymbolId, Vec<FileId>>,
    pub file_symbols: BTreeMap<FileId, Vec<SymbolId>>,
    /// Deduplicated sorted supertype channel folded from the per-TU typed
    /// `TuAnalysis::extends_edges` (plan M23, D-M23-8): persisted separately
    /// from the four frozen blobs (`extends-edges.bin`), so their bincode
    /// layouts and the epoch stay untouched. C++ folds none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extends_edges: Vec<(SymbolId, SymbolId)>,
    /// Symbols whose analysis records carried the `sealed` modifier
    /// (plan M23, D-M23-8). A side channel for the same reason as
    /// `extends_edges`: the `symbols.bin` layout is frozen, so
    /// `ReducedSymbolRecord` cannot grow `modifiers`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sealed_symbols: Vec<SymbolId>,
}

impl ReducedSemanticModel {
    pub fn reduce(index_result: &IndexResult) -> Self {
        let mut symbols: BTreeMap<SymbolId, ReducedSymbolRecord> = BTreeMap::new();
        let mut edges: Vec<ReducedSemanticEdge> = Vec::new();
        let mut tu_usage: BTreeMap<SymbolId, BTreeSet<FileId>> = BTreeMap::new();
        let mut file_symbols: BTreeMap<FileId, BTreeSet<SymbolId>> = BTreeMap::new();
        let mut raw_edges: BTreeMap<(SymbolId, SymbolId), DependencyKind> = BTreeMap::new();
        let mut extends: BTreeSet<(SymbolId, SymbolId)> = BTreeSet::new();
        let mut sealed: BTreeSet<SymbolId> = BTreeSet::new();

        for tu in &index_result.tu_analyses {
            let tu_file = tu.tu_file_id;

            // DeclIds are per-file parser counters (`decl_<counter>`) that
            // collide across files once analyses are merged into one
            // TuAnalysis, so locations are keyed by (symbol, decl) and
            // multiple files per pair are kept for the ambiguous case.
            let mut decl_locations: BTreeMap<(&SymbolId, &DeclId), Vec<FileId>> = BTreeMap::new();
            for decl in &tu.declarations {
                if decl.location.file_id.0 == 0 {
                    continue;
                }
                let files = decl_locations
                    .entry((&decl.symbol_id, &decl.id))
                    .or_default();
                if !files.contains(&decl.location.file_id) {
                    files.push(decl.location.file_id);
                }
            }

            Self::merge_tu_symbols(
                &mut symbols,
                &mut tu_usage,
                &mut file_symbols,
                tu_file,
                &tu.symbols,
                &decl_locations,
            );

            Self::merge_tu_edges(&mut tu_usage, &mut raw_edges, tu_file, &tu.edges);

            // D-M23-8 typed side channels: fold extends pairs and sealed
            // modifiers (C++ analyses contribute nothing to either).
            extends.extend(tu.extends_edges.iter().cloned());
            for sym in &tu.symbols {
                if sym.modifiers.iter().any(|modifier| modifier == "sealed") {
                    sealed.insert(sym.id.clone());
                }
            }

            // References
            for ref_item in &tu.references {
                tu_usage
                    .entry(ref_item.symbol_id.clone())
                    .or_default()
                    .insert(tu_file);
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
            extends_edges: extends.into_iter().collect(),
            sealed_symbols: sealed.into_iter().collect(),
        }
    }

    /// Symbols
    fn merge_tu_symbols<'i>(
        symbols: &mut BTreeMap<SymbolId, ReducedSymbolRecord>,
        tu_usage: &mut BTreeMap<SymbolId, BTreeSet<FileId>>,
        file_symbols: &mut BTreeMap<FileId, BTreeSet<SymbolId>>,
        tu_file: FileId,
        records: &'i [SymbolRecord],
        decl_locations: &DeclLocations<'i>,
    ) {
        for sym in records {
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

            for decl in &sym.declarations {
                if !entry.declarations.contains(decl) {
                    entry.declarations.push(decl.clone());
                }
                for file in decl_files_for(&sym.id, decl, decl_locations) {
                    if !entry.declaring_files.contains(file) {
                        entry.declaring_files.push(*file);
                    }
                }
            }
            // Sort to maintain determinism
            entry.declarations.sort();
            entry.declaring_files.sort();

            if let Some(def) = &sym.definition {
                if entry.definition.is_none() {
                    entry.definition = Some(def.clone());
                    entry.defining_file = Some(defining_file_for(
                        &sym.id,
                        def,
                        decl_locations,
                        sym.owning_file,
                    ));
                } else if entry.kind != SymbolKind::Class
                    && entry.kind != SymbolKind::Struct
                    && (sym.kind == SymbolKind::Class || sym.kind == SymbolKind::Struct)
                {
                    // Class/Struct definitions take precedence over function definitions (like constructors)
                    entry.definition = Some(def.clone());
                    entry.defining_file = Some(defining_file_for(
                        &sym.id,
                        def,
                        decl_locations,
                        sym.owning_file,
                    ));
                    entry.kind = sym.kind;
                }
            }

            tu_usage.entry(sym.id.clone()).or_default().insert(tu_file);
            file_symbols
                .entry(sym.owning_file)
                .or_default()
                .insert(sym.id.clone());
        }
    }

    /// Edges
    fn merge_tu_edges(
        tu_usage: &mut BTreeMap<SymbolId, BTreeSet<FileId>>,
        raw_edges: &mut BTreeMap<(SymbolId, SymbolId), DependencyKind>,
        tu_file: FileId,
        edges: &[SemanticEdge],
    ) {
        for edge in edges {
            tu_usage
                .entry(edge.src_symbol.clone())
                .or_default()
                .insert(tu_file);
            tu_usage
                .entry(edge.dst_symbol.clone())
                .or_default()
                .insert(tu_file);

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
    }

    pub fn find_symbol(&self, name_or_id: &str) -> Option<&ReducedSymbolRecord> {
        if let Some(record) = self.symbols.get(&SymbolId::new(name_or_id)) {
            return Some(record);
        }
        self.symbols
            .values()
            .find(|record| record.name == name_or_id || record.id.0 == name_or_id)
    }

    pub fn get_dependencies(&self, sym_id: &SymbolId) -> Vec<&ReducedSemanticEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.src_symbol == *sym_id)
            .collect()
    }

    pub fn get_users(&self, sym_id: &SymbolId) -> Vec<FileId> {
        self.tu_usage.get(sym_id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_index::types::{
        FileLocation, SemanticEdge, SymbolDecl, SymbolRecord, SymbolReference, TuAnalysis,
    };

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
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![
                SymbolRecord {
                    id: sym_a.clone(),
                    name: "A".into(),
                    kind: SymbolKind::Class,
                    owning_file: f3.clone(),
                    declarations: vec![decl_a1.clone()],
                    definition: None,

                    modifiers: Vec::new(),
                },
                SymbolRecord {
                    id: sym_c.clone(),
                    name: "C".into(),
                    kind: SymbolKind::Function,
                    owning_file: f1.clone(),
                    declarations: vec![],
                    definition: Some(DeclId::new("decl_c")),

                    modifiers: Vec::new(),
                },
            ],
            declarations: vec![],
            edges: vec![SemanticEdge {
                src_symbol: sym_a.clone(),
                dst_symbol: sym_b.clone(),
                kind: DependencyKind::Incomplete,
                src_file: f1.clone(),
                dst_file: f3.clone(),
            }],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
        };

        let tu2 = TuAnalysis {
            tu_file_id: f2.clone(),
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![SymbolRecord {
                id: sym_a.clone(),
                name: "A".into(),
                kind: SymbolKind::Class,
                owning_file: f3.clone(),
                declarations: vec![decl_a2.clone()],
                definition: Some(decl_a2.clone()),

                modifiers: Vec::new(),
            }],
            declarations: vec![],
            edges: vec![SemanticEdge {
                src_symbol: sym_a.clone(),
                dst_symbol: sym_b.clone(),
                kind: DependencyKind::Complete,
                src_file: f2.clone(),
                dst_file: f3.clone(),
            }],
            references: vec![SymbolReference {
                symbol_id: sym_c.clone(),
                file_id: f2.clone(),
                location: FileLocation {
                    file_id: f2.clone(),
                    line: 1,
                    column: 1,
                },
                is_write: false,
            }],
            call_sites: vec![],

            extends_edges: Vec::new(),
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
        assert_eq!(
            sym_c.defining_file,
            Some(FileId::from_relative_path("a.cpp"))
        );
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

    fn symbol_decl(
        id: &str,
        symbol: &str,
        file_id: FileId,
        line: usize,
        is_definition: bool,
    ) -> SymbolDecl {
        SymbolDecl {
            id: DeclId::new(id),
            symbol_id: SymbolId::new(symbol),
            location: FileLocation {
                file_id,
                line,
                column: 1,
            },
            is_definition,
        }
    }

    #[test]
    fn test_defining_file_from_definition_decl_location() {
        let analyzed = FileId::from_relative_path("src/a.cpp");
        let header = FileId::from_relative_path("headers/x.h");
        let decl_def = DeclId::new("decl_7");

        let tu = TuAnalysis {
            tu_file_id: analyzed,
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![SymbolRecord {
                id: SymbolId::new("X"),
                name: "X".into(),
                kind: SymbolKind::Function,
                owning_file: analyzed,
                declarations: vec![decl_def.clone()],
                definition: Some(decl_def.clone()),

                modifiers: Vec::new(),
            }],
            declarations: vec![symbol_decl("decl_7", "X", header, 3, true)],
            edges: vec![],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
        };

        let index = IndexResult::new(PathBuf::from("/test"), vec![tu]);
        let reduced = ReducedSemanticModel::reduce(&index);

        let x = reduced.symbols.get(&SymbolId::new("X")).unwrap();
        assert_eq!(
            x.defining_file,
            Some(header),
            "defining_file must be the definition declaration's file, not the owning file"
        );
    }

    #[test]
    fn test_defining_file_falls_back_to_owning_file_for_unusable_decl_location() {
        let analyzed = FileId::from_relative_path("src/a.cpp");
        let unknown_def = DeclId::new("decl_missing");
        let sentinel_def = DeclId::new("decl_sentinel");

        let tu = TuAnalysis {
            tu_file_id: analyzed,
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![
                SymbolRecord {
                    id: SymbolId::new("Unknown"),
                    name: "Unknown".into(),
                    kind: SymbolKind::Function,
                    owning_file: analyzed,
                    declarations: vec![unknown_def.clone()],
                    definition: Some(unknown_def.clone()),

                    modifiers: Vec::new(),
                },
                SymbolRecord {
                    id: SymbolId::new("Sentinel"),
                    name: "Sentinel".into(),
                    kind: SymbolKind::Function,
                    owning_file: analyzed,
                    declarations: vec![sentinel_def.clone()],
                    definition: Some(sentinel_def.clone()),

                    modifiers: Vec::new(),
                },
            ],
            declarations: vec![symbol_decl("decl_sentinel", "Sentinel", FileId(0), 0, true)],
            edges: vec![],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
        };

        let index = IndexResult::new(PathBuf::from("/test"), vec![tu]);
        let reduced = ReducedSemanticModel::reduce(&index);

        let unknown = reduced.symbols.get(&SymbolId::new("Unknown")).unwrap();
        assert_eq!(
            unknown.defining_file,
            Some(analyzed),
            "definition decl absent from the TU declaration registry must fall back to owning_file"
        );
        let sentinel = reduced.symbols.get(&SymbolId::new("Sentinel")).unwrap();
        assert_eq!(
            sentinel.defining_file,
            Some(analyzed),
            "definition decl located at the FileId(0) sentinel must fall back to owning_file"
        );
    }

    #[test]
    fn test_defining_file_falls_back_to_owning_file_for_ambiguous_decl_id() {
        let analyzed = FileId::from_relative_path("src/a.cpp");
        let host_one = FileId::from_relative_path("host_one.h");
        let host_two = FileId::from_relative_path("host_two.h");

        let tu = TuAnalysis {
            tu_file_id: analyzed,
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![SymbolRecord {
                id: SymbolId::new("Dup"),
                name: "Dup".into(),
                kind: SymbolKind::Class,
                owning_file: analyzed,
                declarations: vec![DeclId::new("decl_1")],
                definition: Some(DeclId::new("decl_1")),

                modifiers: Vec::new(),
            }],
            declarations: vec![
                symbol_decl("decl_1", "Dup", host_one, 1, true),
                symbol_decl("decl_1", "Dup", host_two, 2, true),
            ],
            edges: vec![],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
        };

        let index = IndexResult::new(PathBuf::from("/test"), vec![tu]);
        let reduced = ReducedSemanticModel::reduce(&index);

        let dup = reduced.symbols.get(&SymbolId::new("Dup")).unwrap();
        assert_eq!(
            dup.defining_file,
            Some(analyzed),
            "a (symbol, decl) pair resolving to multiple files must fall back to owning_file"
        );
        assert_eq!(
            dup.declaring_files,
            {
                let mut files = vec![host_one, host_two];
                files.sort();
                files
            },
            "ambiguous pairs must still contribute every hosting file to declaring_files"
        );
    }

    #[test]
    fn test_class_upgrade_repoints_defining_file_to_class_decl_file() {
        let impl_file = FileId::from_relative_path("src/impl.cpp");
        let stale_file = FileId::from_relative_path("src/stale.cpp");
        let header = FileId::from_relative_path("include/thing.h");

        let tu_fn = TuAnalysis {
            tu_file_id: impl_file,
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![SymbolRecord {
                id: SymbolId::new("Thing"),
                name: "Thing".into(),
                kind: SymbolKind::Function,
                owning_file: impl_file,
                declarations: vec![DeclId::new("decl_fn")],
                definition: Some(DeclId::new("decl_fn")),

                modifiers: Vec::new(),
            }],
            declarations: vec![symbol_decl("decl_fn", "Thing", impl_file, 5, true)],
            edges: vec![],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
        };

        let tu_class = TuAnalysis {
            tu_file_id: stale_file,
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![SymbolRecord {
                id: SymbolId::new("Thing"),
                name: "Thing".into(),
                kind: SymbolKind::Class,
                owning_file: stale_file,
                declarations: vec![DeclId::new("decl_class")],
                definition: Some(DeclId::new("decl_class")),

                modifiers: Vec::new(),
            }],
            declarations: vec![symbol_decl("decl_class", "Thing", header, 2, true)],
            edges: vec![],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
        };

        let index = IndexResult::new(PathBuf::from("/test"), vec![tu_fn, tu_class]);
        let reduced = ReducedSemanticModel::reduce(&index);

        let thing = reduced.symbols.get(&SymbolId::new("Thing")).unwrap();
        assert_eq!(thing.kind, SymbolKind::Class);
        assert_eq!(thing.definition, Some(DeclId::new("decl_class")));
        assert_eq!(
            thing.defining_file,
            Some(header),
            "Class-over-function upgrade must re-point defining_file to the class definition decl's file"
        );
    }

    #[test]
    fn test_declaring_files_deduped_and_sorted_across_tus() {
        let a = FileId::from_relative_path("zeta.h");
        let b = FileId::from_relative_path("alpha.h");

        let tu1 = TuAnalysis {
            tu_file_id: a,
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![SymbolRecord {
                id: SymbolId::new("S"),
                name: "S".into(),
                kind: SymbolKind::Class,
                owning_file: a,
                declarations: vec![DeclId::new("decl_1")],
                definition: None,

                modifiers: Vec::new(),
            }],
            declarations: vec![symbol_decl("decl_1", "S", a, 1, false)],
            edges: vec![],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
        };

        let tu2 = TuAnalysis {
            tu_file_id: b,
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![SymbolRecord {
                id: SymbolId::new("S"),
                name: "S".into(),
                kind: SymbolKind::Class,
                owning_file: b,
                declarations: vec![DeclId::new("decl_1"), DeclId::new("decl_2")],
                definition: None,

                modifiers: Vec::new(),
            }],
            declarations: vec![
                symbol_decl("decl_1", "S", b, 9, false),
                symbol_decl("decl_2", "S", b, 10, false),
            ],
            edges: vec![],
            references: vec![],
            call_sites: vec![],

            extends_edges: Vec::new(),
        };

        let index = IndexResult::new(PathBuf::from("/test"), vec![tu1, tu2]);
        let reduced = ReducedSemanticModel::reduce(&index);

        let s = reduced.symbols.get(&SymbolId::new("S")).unwrap();
        assert_eq!(
            s.declaring_files,
            vec![b, a],
            "declaring_files must be deduplicated and sorted across TUs"
        );
    }

    // The fixture-based `defining_file`-vs-legacy projection lock moved to
    // the permanent M18.11 suite (`m18_cpp_plugin_parity::provenance_lock`),
    // which drives the real analyzer through `ariadne-lang-cpp` — graph must
    // not depend on a language plugin even in tests (plan §6.3 R1 spirit).

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

    // UTA-T1 items 11 and 15 (hand H1): D-M23-8 side channels, edge-kind
    // precedence in both orders, and the find_symbol duplicate-name pin.
    fn plain_record(id: &str, kind: SymbolKind, modifiers: Vec<String>) -> SymbolRecord {
        SymbolRecord {
            id: SymbolId::new(id),
            name: id.to_string(),
            kind,
            owning_file: FileId::from_relative_path("src/main.scala"),
            declarations: vec![],
            definition: None,
            modifiers,
        }
    }

    #[test]
    fn reduce_folds_extends_edges_and_sealed_symbols_side_channels() {
        let tu_a = TuAnalysis {
            tu_file_id: FileId::from_relative_path("src/sub.scala"),
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Scala,
            symbols: vec![
                plain_record(
                    "sub",
                    SymbolKind::Trait,
                    vec!["sealed".to_string()],
                ),
                plain_record("sup", SymbolKind::Trait, Vec::new()),
            ],
            declarations: vec![],
            edges: vec![],
            references: vec![],
            call_sites: vec![],
            extends_edges: vec![(SymbolId::new("sub"), SymbolId::new("sup"))],
        };
        let tu_b = TuAnalysis {
            tu_file_id: FileId::from_relative_path("src/other.scala"),
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Scala,
            symbols: vec![
                plain_record(
                    "sub",
                    SymbolKind::Trait,
                    vec!["sealed".to_string()],
                ),
                plain_record("other", SymbolKind::Trait, Vec::new()),
            ],
            declarations: vec![],
            edges: vec![],
            references: vec![],
            call_sites: vec![],
            // Duplicate (sub, sup) across TUs plus a second pair: the
            // channel must come out deduplicated and ascending.
            extends_edges: vec![
                (SymbolId::new("sub"), SymbolId::new("sup")),
                (SymbolId::new("other"), SymbolId::new("sup")),
            ],
        };

        let index = IndexResult::new(
            PathBuf::from("/test"),
            vec![tu_a, tu_b],
        );
        let reduced = ReducedSemanticModel::reduce(&index);

        assert_eq!(
            reduced.extends_edges,
            vec![
                (SymbolId::new("other"), SymbolId::new("sup")),
                (SymbolId::new("sub"), SymbolId::new("sup")),
            ],
            "extends_edges is a deduplicated, ascending (subtype, supertype) channel"
        );
        assert_eq!(
            reduced.sealed_symbols,
            vec![SymbolId::new("sub")],
            "sealed modifiers fold into a deduplicated ascending side channel, \
             never into the frozen symbol records"
        );
    }

    #[test]
    fn merge_tu_edges_keeps_complete_kind_when_seen_first() {
        // The existing suite pins Incomplete-then-Complete (upgrade wins);
        // this locks the opposite arrival order: a later Incomplete edge
        // must NOT demote an already-Complete pair.
        let f1 = FileId::from_relative_path("a.cpp");
        let f2 = FileId::from_relative_path("b.cpp");
        let edge = |kind: DependencyKind| SemanticEdge {
            src_symbol: SymbolId::new("A"),
            dst_symbol: SymbolId::new("B"),
            kind,
            src_file: f1,
            dst_file: f2,
        };
        let tu_complete_first = TuAnalysis {
            tu_file_id: f1,
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![],
            declarations: vec![],
            edges: vec![edge(DependencyKind::Complete)],
            references: vec![],
            call_sites: vec![],
            extends_edges: Vec::new(),
        };
        let tu_incomplete_second = TuAnalysis {
            tu_file_id: f2,
            context_name: "test".into(),
            language: ariadne_repo::SourceLanguage::Cpp,
            symbols: vec![],
            declarations: vec![],
            edges: vec![edge(DependencyKind::Incomplete)],
            references: vec![],
            call_sites: vec![],
            extends_edges: Vec::new(),
        };

        let index = IndexResult::new(
            PathBuf::from("/test"),
            vec![tu_complete_first, tu_incomplete_second],
        );
        let reduced = ReducedSemanticModel::reduce(&index);

        assert_eq!(reduced.edges.len(), 1, "one merged pair");
        assert_eq!(
            reduced.edges[0].kind,
            DependencyKind::Complete,
            "Complete must win regardless of arrival order"
        );
    }

    #[test]
    fn find_symbol_with_duplicate_name_returns_the_first_btreemap_key() {
        // Two distinct ids share the NAME "dup". The id-key lookup misses,
        // the value scan walks BTreeMap key order ascending - pin that the
        // FIRST key wins deterministically, and that an exact id query
        // still routes straight to its own record.
        let mut model = ReducedSemanticModel::default();
        for id in ["b_dup", "a_dup"] {
            model.symbols.insert(
                SymbolId::new(id),
                ReducedSymbolRecord {
                    id: SymbolId::new(id),
                    name: "dup".to_string(),
                    kind: SymbolKind::Function,
                    defining_file: None,
                    declarations: Vec::new(),
                    definition: None,
                    declaring_files: Vec::new(),
                },
            );
        }

        let found = model
            .find_symbol("dup")
            .expect("a name-duplicate record is found");
        assert_eq!(
            found.id,
            SymbolId::new("a_dup"),
            "name lookup over duplicate names must pin to the ascending \
             BTreeMap's first key"
        );
        assert_eq!(
            model
                .find_symbol("b_dup")
                .expect("exact id-key lookup is unaffected")
                .id,
            SymbolId::new("b_dup")
        );
    }
}
