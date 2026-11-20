//! Compact file load for the stage-5 collateral pass.
//!
//! A full file [`LoadMetric`] lists every symbol of the file's include
//! closure: the used (semantic) ones and the unused (collateral) ones, and
//! again per contributing source. On large repos a file's closure holds tens
//! of thousands of symbols, so the stage-5 pass cloned, sorted, and stored
//! files x closure-symbols names; that dominated both its time and the size
//! of the load artifacts. Consumers of the stage-5 metrics only read the
//! collateral side's counts and a few sorted sample names per source.
//!
//! The compact metric equals the full one except on the collateral side:
//! `collateral_symbols` is empty, and each collateral source keeps its full
//! `symbol_count` but at most `names_per_source` of its smallest symbol ids.
//! Only sources a capped justification list can show keep names at all:
//! the includable headers ranked within the top `named_sources` by combined
//! used + collateral count, ties broken by file id as the UI breaks them. A file reaches about a
//! thousand collateral headers on a large repo, and naming them all was most
//! of the load artifacts' size. The used side, every load count, and the
//! source order are identical. Explaining one file in full still goes
//! through `analyze_file_load_by_id`.

use super::{ContributingSource, LoadAnalyzer, LoadMetric};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Sample names kept per named collateral source: at least what any
/// consumer shows (the UI justification rows show 12).
pub const COLLATERAL_SAMPLE_SIZE: usize = 16;

/// Collateral sources that keep sample names, by count: at least as many as
/// any capped consumer lists (the UI justification shows 16 sources).
pub const COLLATERAL_NAMED_SOURCES: usize = 16;

/// How much collateral detail a compact metric keeps.
#[derive(Debug, Clone, Copy)]
pub struct CompactShape {
    pub names_per_source: usize,
    pub named_sources: usize,
}

impl Default for CompactShape {
    fn default() -> Self {
        Self {
            names_per_source: COLLATERAL_SAMPLE_SIZE,
            named_sources: COLLATERAL_NAMED_SOURCES,
        }
    }
}

/// One collateral source's running count and its smallest symbol ids.
#[derive(Default)]
struct CollateralTally<'m> {
    count: usize,
    smallest: Vec<&'m SymbolId>,
}

impl<'m> CollateralTally<'m> {
    fn record(&mut self, symbol: &'m SymbolId, sample_size: usize) {
        self.count += 1;
        if self.smallest.len() == sample_size
            && self.smallest.last().is_some_and(|largest| symbol >= *largest)
        {
            return;
        }
        let position = self.smallest.partition_point(|kept| *kept < symbol);
        self.smallest.insert(position, symbol);
        self.smallest.truncate(sample_size);
    }
}

impl<'a> LoadAnalyzer<'a> {
    /// The compact load metric of `file_id` (see the module docs).
    pub fn analyze_file_load_compact_by_id(&self, file_id: FileId, shape: CompactShape) -> LoadMetric {
        let target_file = self.include_graph.file_paths.get(&file_id).cloned();
        let target_name = target_file
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let seeds = self.model.file_symbols.get(&file_id).cloned().unwrap_or_default();
        let semantic_closure = self.memo.get_semantic_closure_of_symbols(self.model, &seeds);

        let mut physical: HashSet<&'a SymbolId> = HashSet::new();
        for included in self.memo.get_transitive_includes(self.include_graph, file_id) {
            if let Some(symbols) = self.model.file_symbols.get(&included) {
                physical.extend(symbols.iter());
            }
        }

        let mut semantic_symbols: Vec<SymbolId> = Vec::new();
        let mut collateral_load = 0usize;
        let mut tallies: HashMap<FileId, CollateralTally<'a>> = HashMap::new();
        for symbol in &physical {
            if semantic_closure.contains(*symbol) {
                semantic_symbols.push((*symbol).clone());
                continue;
            }
            collateral_load += 1;
            let defining_file = self.model.symbols.get(*symbol).and_then(|record| self.attribution_file(record));
            if let Some(source) = defining_file {
                tallies.entry(source).or_default().record(symbol, shape.names_per_source);
            }
        }
        semantic_symbols.sort();

        let mut collateral_sources = self.sources_from_tallies(tallies);
        let semantic_sources = self.contributing_sources(&semantic_symbols);
        keep_names_of_shown_sources(&mut collateral_sources, &semantic_sources, shape.named_sources);
        LoadMetric {
            target_name,
            target_file,
            semantic_load: semantic_symbols.len(),
            include_load: physical.len(),
            collateral_load,
            total_load: physical.len(),
            semantic_symbols,
            collateral_symbols: Vec::new(),
            largest_collateral_source: collateral_sources.first().cloned(),
            collateral_sources,
            semantic_sources,
        }
    }

    /// Sources in the full metric's order: count descending, then path.
    fn sources_from_tallies(&self, tallies: HashMap<FileId, CollateralTally<'_>>) -> Vec<ContributingSource> {
        let mut sources: Vec<ContributingSource> = tallies
            .into_iter()
            .map(|(file_id, tally)| ContributingSource {
                path: self
                    .include_graph
                    .file_paths
                    .get(&file_id)
                    .cloned()
                    .unwrap_or_else(|| PathBuf::from("unknown")),
                file_id,
                symbol_count: tally.count,
                symbols: tally.smallest.into_iter().cloned().collect(),
            })
            .collect();
        sources.sort_by(|left, right| {
            right
                .symbol_count
                .cmp(&left.symbol_count)
                .then_with(|| left.path.cmp(&right.path))
        });
        sources
    }
}

/// Clears the sample names of collateral sources no capped list can show.
/// Mirrors the UI justification ranking (`ui::dto::rank_source_ids`): the
/// includable-header sources of both sides, by combined used + collateral
/// count descending, then file id; only the first `named_sources` keep names.
fn keep_names_of_shown_sources(
    collateral: &mut [ContributingSource],
    semantic: &[ContributingSource],
    named_sources: usize,
) {
    let mut combined: HashMap<FileId, usize> = HashMap::new();
    for source in semantic.iter().chain(collateral.iter()) {
        if source.symbol_count > 0 && super::is_includable_header(&source.path) {
            *combined.entry(source.file_id).or_default() += source.symbol_count;
        }
    }
    let mut ranked: Vec<(FileId, usize)> = combined.into_iter().collect();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then(left.0 .0.cmp(&right.0 .0)));
    let shown: HashSet<FileId> = ranked.into_iter().take(named_sources).map(|(id, _)| id).collect();
    for source in collateral.iter_mut() {
        if !shown.contains(&source.file_id) {
            source.symbols.clear();
        }
    }
}
