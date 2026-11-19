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
///
/// Works one symbol at a time: the units to credit are the concatenated
/// sharer lists of the files touching it, merged into its set in one sorted
/// pass. Crediting (unit, file, symbol) triples one by one cloned a symbol
/// name per triple and dominated the whole reduce on large repos.
pub(crate) fn credit_shared_file_usage(
    index_result: &IndexResult,
    tu_usage: &mut BTreeMap<SymbolId, BTreeSet<FileId>>,
) {
    let sharers = units_sharing_each_file(&index_result.tu_analyses);
    if sharers.is_empty() {
        return;
    }
    let files_by_symbol = files_touching_each_symbol(&index_result.tu_analyses, &sharers);
    for (symbol, files) in files_by_symbol {
        let entry = tu_usage.entry(symbol.clone()).or_default();
        let mut users: Vec<FileId> = files
            .iter()
            .flat_map(|file| sharers[file].iter().copied())
            .chain(entry.iter().copied())
            .collect();
        users.sort_unstable();
        users.dedup();
        *entry = users.into_iter().collect();
    }
}

/// For each shared file, the units that name it in `shared_files`.
fn units_sharing_each_file(analyses: &[TuAnalysis]) -> BTreeMap<FileId, Vec<FileId>> {
    let mut sharers: BTreeMap<FileId, Vec<FileId>> = BTreeMap::new();
    for tu in analyses {
        for file in &tu.shared_files {
            sharers.entry(*file).or_default().push(tu.tu_file_id);
        }
    }
    sharers
}

/// For each symbol, the shared files that declare, link, or reference it,
/// gathered from whichever unit stores that file's records.
fn files_touching_each_symbol<'a>(
    analyses: &'a [TuAnalysis],
    shared: &BTreeMap<FileId, Vec<FileId>>,
) -> BTreeMap<&'a SymbolId, BTreeSet<FileId>> {
    let mut touching: BTreeMap<&'a SymbolId, BTreeSet<FileId>> = BTreeMap::new();
    let mut credit = |file: FileId, symbol: &'a SymbolId| {
        if shared.contains_key(&file) {
            touching.entry(symbol).or_default().insert(file);
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
    touching
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
