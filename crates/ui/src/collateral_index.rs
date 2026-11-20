//! Compact per-file collateral counts behind the on-demand routes.
//!
//! The include tab needs, for a selected file, how many unused symbols each
//! header of its closure contributes (a row), for a selected header, how
//! much each including file pays for it (a column), and per header the
//! totals over all files (burden). Shipping the whole file x header map in
//! `/api/export` grew with files times closure size: 59 MB for 2500 files,
//! gigabytes on large repos, past what a browser parses as one JSON text.
//! The live server keeps it here as numeric pairs and answers one row or
//! column per request; the offline export still embeds the nested map.

use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

/// One header's totals over every file that pays for it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct CollateralBurden {
    pub burden: usize,
    pub affected: usize,
}

/// Root file id -> (source file id, collateral symbol count), zeros dropped.
#[derive(Debug, Default)]
pub struct CollateralIndex {
    rows: HashMap<u64, Vec<(u64, usize)>>,
}

impl CollateralIndex {
    /// From stage-5 file metrics: a metric whose target is not a graph file
    /// is skipped, zero-count sources are dropped, and a root with no
    /// contributing source has no row (the `file_collateral` rules).
    pub fn from_metrics(
        path_to_id: &HashMap<PathBuf, u64>,
        file_metrics: &[ariadne_graph::LoadMetric],
    ) -> Self {
        let mut rows = HashMap::new();
        for metric in file_metrics {
            let Some(root) = metric.target_file.as_ref().and_then(|path| path_to_id.get(path)) else {
                continue;
            };
            let row: Vec<(u64, usize)> = metric
                .collateral_sources
                .iter()
                .filter(|source| source.symbol_count > 0)
                .map(|source| (source.file_id.0, source.symbol_count))
                .collect();
            if !row.is_empty() {
                rows.insert(*root, row);
            }
        }
        Self { rows }
    }

    /// From the payload's nested string-keyed map (non-numeric ids skipped).
    pub fn from_nested_map(map: &HashMap<String, HashMap<String, usize>>) -> Self {
        let mut rows = HashMap::new();
        for (root, sources) in map {
            let Ok(root) = root.parse::<u64>() else { continue };
            let row: Vec<(u64, usize)> = sources
                .iter()
                .filter(|(_, count)| **count > 0)
                .filter_map(|(source, count)| source.parse::<u64>().ok().map(|id| (id, *count)))
                .collect();
            if !row.is_empty() {
                rows.insert(root, row);
            }
        }
        Self { rows }
    }

    /// The nested map the offline export embeds as `file_collateral`.
    pub fn to_nested_map(&self) -> HashMap<String, HashMap<String, usize>> {
        self.rows
            .iter()
            .map(|(root, row)| {
                let inner = row.iter().map(|(source, count)| (source.to_string(), *count)).collect();
                (root.to_string(), inner)
            })
            .collect()
    }

    /// What `root` pays per header, keyed by header id.
    pub fn row(&self, root: u64) -> BTreeMap<String, usize> {
        self.rows
            .get(&root)
            .map(|row| row.iter().map(|(source, count)| (source.to_string(), *count)).collect())
            .unwrap_or_default()
    }

    /// What each file pays for `source`, keyed by root id (one scan).
    pub fn column(&self, source: u64) -> BTreeMap<String, usize> {
        let mut column = BTreeMap::new();
        for (root, row) in &self.rows {
            if let Some((_, count)) = row.iter().find(|(id, _)| *id == source) {
                column.insert(root.to_string(), *count);
            }
        }
        column
    }

    /// Per header: total collateral paid and how many files pay it.
    pub fn burden(&self) -> BTreeMap<String, CollateralBurden> {
        let mut totals: HashMap<u64, CollateralBurden> = HashMap::new();
        for row in self.rows.values() {
            for (source, count) in row {
                let entry = totals.entry(*source).or_default();
                entry.burden += count;
                entry.affected += 1;
            }
        }
        totals.into_iter().map(|(source, total)| (source.to_string(), total)).collect()
    }
}
