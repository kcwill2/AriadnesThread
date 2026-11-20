//! Dense stage-5 file pass: the compact file load over `DenseLoadIndex`.
//!
//! Equal to `analyze_file_load_compact_by_id`, computed without string sets:
//!
//! 1. set one bit per symbol of the file's physical expansion,
//! 2. stamp every component the file's own symbols reach (one walk for the
//!    whole file),
//! 3. read the bits back in ascending order - which is ascending id order,
//!    so the used side comes out sorted and each collateral source meets its
//!    smallest ids first - and split each symbol on its component's stamp.
//!
//! Strings are cloned only for what the metric stores: the used symbols and
//! the sample names of the collateral sources.

use super::compact::keep_names_of_shown_sources;
use super::dense::{DenseLoadIndex, VisitScratch, NO_SOURCE};
use super::{CompactShape, ContributingSource, LoadMetric};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use std::path::PathBuf;

/// Per-thread arrays of the file pass, all sized by the index.
pub(super) struct FileScratch {
    visit: VisitScratch,
    /// One bit per symbol; all zero between files.
    physical: Vec<u64>,
    /// Collateral symbols seen per source; all zero between files.
    source_counts: Vec<u32>,
}

/// A file's physical expansion split on use, as symbol and source numbers.
struct PhysicalSplit {
    include_load: usize,
    collateral_load: usize,
    /// Used symbols, ascending.
    semantic: Vec<u32>,
    /// `(source, collateral symbols credited to it)`.
    collateral_counts: Vec<(u32, u32)>,
    /// `(source, symbol)`: each source's smallest collateral symbols.
    collateral_samples: Vec<(u32, u32)>,
}

/// Count descending, then path: the order of the full metric's sources.
fn sort_sources(sources: &mut [ContributingSource]) {
    sources.sort_by(|left, right| {
        right
            .symbol_count
            .cmp(&left.symbol_count)
            .then_with(|| left.path.cmp(&right.path))
    });
}

impl<'a> DenseLoadIndex<'a> {
    /// The compact load metric of `file_id`: equal to
    /// `LoadAnalyzer::analyze_file_load_compact_by_id(file_id, shape)`.
    pub fn file_load_compact(&self, file_id: FileId, shape: CompactShape) -> LoadMetric {
        let mut scratch = self.file_buffers.take_or(|| FileScratch {
            visit: VisitScratch::new(self.component_uses.node_count()),
            physical: vec![0; self.ids.len().div_ceil(64)],
            source_counts: vec![0; self.source_files.len()],
        });
        for listed in self.physical_symbol_lists(file_id) {
            for &symbol in listed {
                scratch.physical[symbol as usize / 64] |= 1u64 << (symbol % 64);
            }
        }
        let used = self.stamp_used_components(file_id, &mut scratch.visit);
        let split = self.split_physical(&mut scratch, used, shape.names_per_source);
        self.compact_metric(file_id, split, shape.named_sources)
    }

    /// Stamp every component reachable from the file's own symbols; returns
    /// the stamp value marking "used by this file".
    fn stamp_used_components(&self, file_id: FileId, visit: &mut VisitScratch) -> u32 {
        let used = visit.begin_walk();
        let seeds = self.file_symbols.get(&file_id).map(Vec::as_slice).unwrap_or_default();
        for &seed in seeds {
            let component = self.component_of[seed as usize];
            if visit.stamps[component as usize] != used {
                visit.stamps[component as usize] = used;
                visit.stack.push(component);
            }
        }
        while let Some(component) = visit.stack.pop() {
            for &next in self.component_uses.successors(component) {
                if visit.stamps[next as usize] != used {
                    visit.stamps[next as usize] = used;
                    visit.stack.push(next);
                }
            }
        }
        used
    }

    /// Read the physical bits back in ascending symbol order, clearing them,
    /// and split every symbol on whether its component carries `used`.
    fn split_physical(&self, scratch: &mut FileScratch, used: u32, names_per_source: usize) -> PhysicalSplit {
        let mut split = PhysicalSplit {
            include_load: 0,
            collateral_load: 0,
            semantic: Vec::new(),
            collateral_counts: Vec::new(),
            collateral_samples: Vec::new(),
        };
        let mut touched_sources: Vec<u32> = Vec::new();
        for (word_index, word) in scratch.physical.iter_mut().enumerate() {
            let mut bits = std::mem::take(word);
            while bits != 0 {
                let symbol = (word_index * 64) as u32 + bits.trailing_zeros();
                bits &= bits - 1;
                split.include_load += 1;
                if scratch.visit.stamps[self.component_of[symbol as usize] as usize] == used {
                    split.semantic.push(symbol);
                    continue;
                }
                split.collateral_load += 1;
                let source = self.source_of[symbol as usize];
                if source == NO_SOURCE {
                    continue;
                }
                let count = &mut scratch.source_counts[source as usize];
                if *count == 0 {
                    touched_sources.push(source);
                }
                *count += 1;
                if *count as usize <= names_per_source {
                    split.collateral_samples.push((source, symbol));
                }
            }
        }
        touched_sources.sort_unstable();
        split.collateral_counts = touched_sources
            .into_iter()
            .map(|source| (source, std::mem::take(&mut scratch.source_counts[source as usize])))
            .collect();
        split
    }

    fn compact_metric(&self, file_id: FileId, split: PhysicalSplit, named_sources: usize) -> LoadMetric {
        let target_file = self.include_graph.file_paths.get(&file_id).cloned();
        let target_name = target_file
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let semantic_sources = self.semantic_sources(&split.semantic);
        let mut collateral_sources = self.collateral_sources(&split.collateral_counts, split.collateral_samples);
        keep_names_of_shown_sources(&mut collateral_sources, &semantic_sources, named_sources);
        LoadMetric {
            target_name,
            target_file,
            semantic_load: split.semantic.len(),
            include_load: split.include_load,
            collateral_load: split.collateral_load,
            total_load: split.include_load,
            semantic_symbols: self.symbol_ids(split.semantic.iter().copied()),
            collateral_symbols: Vec::new(),
            largest_collateral_source: collateral_sources.first().cloned(),
            collateral_sources,
            semantic_sources,
        }
    }

    fn symbol_ids(&self, symbols: impl Iterator<Item = u32>) -> Vec<SymbolId> {
        symbols.map(|symbol| self.ids[symbol as usize].clone()).collect()
    }

    fn source(&self, source: u32, symbol_count: usize, symbols: Vec<SymbolId>) -> ContributingSource {
        let file_id = self.source_files[source as usize];
        ContributingSource {
            path: self
                .include_graph
                .file_paths
                .get(&file_id)
                .cloned()
                .unwrap_or_else(|| PathBuf::from("unknown")),
            file_id,
            symbol_count,
            symbols,
        }
    }

    /// The used symbols grouped by credited file, each group ascending.
    fn semantic_sources(&self, semantic: &[u32]) -> Vec<ContributingSource> {
        let mut credited: Vec<(u32, u32)> = semantic
            .iter()
            .map(|&symbol| (self.source_of[symbol as usize], symbol))
            .filter(|(source, _)| *source != NO_SOURCE)
            .collect();
        credited.sort_unstable();
        let mut sources: Vec<ContributingSource> = credited
            .chunk_by(|left, right| left.0 == right.0)
            .map(|group| self.source(group[0].0, group.len(), self.symbol_ids(group.iter().map(|pair| pair.1))))
            .collect();
        sort_sources(&mut sources);
        sources
    }

    /// The collateral sources with their full counts and sample names.
    /// `counts` is ascending by source.
    fn collateral_sources(&self, counts: &[(u32, u32)], mut samples: Vec<(u32, u32)>) -> Vec<ContributingSource> {
        samples.sort_unstable();
        let mut remaining = samples.as_slice();
        let mut sources: Vec<ContributingSource> = Vec::with_capacity(counts.len());
        for &(source, count) in counts {
            let named = remaining.iter().take_while(|sample| sample.0 == source).count();
            let (own, rest) = remaining.split_at(named);
            remaining = rest;
            sources.push(self.source(source, count as usize, self.symbol_ids(own.iter().map(|pair| pair.1))));
        }
        sort_sources(&mut sources);
        sources
    }
}
