//! Unit usage through shared files (store-once indexing).
//!
//! A store-once index keeps each file's records in ONE unit's analysis; every
//! other unit whose closure reaches that file names it in
//! `TuAnalysis::shared_files`. "Which units use symbol S" must still count
//! those units, exactly as when every unit carried a full copy. Each record
//! is stamped with the file it was analyzed from (symbol `owning_file`, edge
//! `src_file`, reference `file_id`), so the symbols a file touches can be
//! collected once and credited to every unit that shares the file.

use std::collections::{BTreeMap, BTreeSet};

use ariadne_index::{IndexResult, SymbolId, TuAnalysis};
use ariadne_repo::FileId;

/// Credit every unit with the symbols touched by the files it shares.
/// A no-op for indexes without shared files (full-closure producers).
pub(crate) fn credit_shared_file_usage(
    index_result: &IndexResult,
    tu_usage: &mut BTreeMap<SymbolId, BTreeSet<FileId>>,
) {
    let shared: BTreeSet<FileId> = index_result
        .tu_analyses
        .iter()
        .flat_map(|tu| tu.shared_files.iter().copied())
        .collect();
    if shared.is_empty() {
        return;
    }
    let touched = symbols_touched_by_files(&index_result.tu_analyses, &shared);
    for tu in &index_result.tu_analyses {
        for file in &tu.shared_files {
            for symbol in touched.get(file).into_iter().flatten() {
                tu_usage.entry(symbol.clone()).or_default().insert(tu.tu_file_id);
            }
        }
    }
}

/// Symbols each file in `files` declares, links, or references, gathered
/// from whichever unit stores that file's records.
fn symbols_touched_by_files(
    analyses: &[TuAnalysis],
    files: &BTreeSet<FileId>,
) -> BTreeMap<FileId, BTreeSet<SymbolId>> {
    let mut touched: BTreeMap<FileId, BTreeSet<SymbolId>> = BTreeMap::new();
    let mut credit = |file: FileId, symbol: &SymbolId| {
        if files.contains(&file) {
            touched.entry(file).or_default().insert(symbol.clone());
        }
    };
    for tu in analyses {
        for symbol in &tu.symbols {
            credit(symbol.owning_file, &symbol.id);
        }
        for edge in &tu.edges {
            credit(edge.src_file, &edge.src_symbol);
            credit(edge.src_file, &edge.dst_symbol);
        }
        for reference in &tu.references {
            credit(reference.file_id, &reference.symbol_id);
        }
    }
    touched
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_index::{DependencyKind, SemanticEdge, SymbolRecord};
    use std::path::PathBuf;

    fn symbol(name: &str, file: u64) -> SymbolRecord {
        SymbolRecord {
            id: SymbolId::new(name),
            name: name.to_string(),
            owning_file: FileId(file),
            ..SymbolRecord::default()
        }
    }

    fn unit(file: u64, symbols: Vec<SymbolRecord>, shared: Vec<u64>) -> TuAnalysis {
        TuAnalysis {
            tu_file_id: FileId(file),
            symbols,
            shared_files: shared.into_iter().map(FileId).collect(),
            ..TuAnalysis::default()
        }
    }

    #[test]
    fn units_sharing_a_header_are_credited_with_its_symbols_and_edges() {
        // Unit 1 stores header 10 (symbol Api, edge Api -> Base); unit 2
        // only names header 10 as shared.
        let mut owner = unit(1, vec![symbol("Main1", 1), symbol("Api", 10)], vec![]);
        owner.edges.push(SemanticEdge {
            src_symbol: SymbolId::new("Api"),
            dst_symbol: SymbolId::new("Base"),
            kind: DependencyKind::Complete,
            src_file: FileId(10),
            dst_file: FileId(11),
        });
        let sharer = unit(2, vec![symbol("Main2", 2)], vec![10]);
        let index = IndexResult::new(PathBuf::from("/repo"), vec![owner, sharer]);

        let mut usage: BTreeMap<SymbolId, BTreeSet<FileId>> = BTreeMap::new();
        credit_shared_file_usage(&index, &mut usage);

        let users = |name: &str| usage.get(&SymbolId::new(name)).cloned().unwrap_or_default();
        assert_eq!(users("Api"), BTreeSet::from([FileId(2)]));
        assert_eq!(users("Base"), BTreeSet::from([FileId(2)]));
        assert!(users("Main1").is_empty(), "unit 1's own file is not shared");
    }

    #[test]
    fn indexes_without_shared_files_are_left_alone() {
        let index = IndexResult::new(
            PathBuf::from("/repo"),
            vec![unit(1, vec![symbol("A", 1)], vec![])],
        );
        let mut usage: BTreeMap<SymbolId, BTreeSet<FileId>> = BTreeMap::new();
        credit_shared_file_usage(&index, &mut usage);
        assert!(usage.is_empty());
    }
}
