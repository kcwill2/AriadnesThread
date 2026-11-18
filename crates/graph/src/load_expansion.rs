//! Shared physical-expansion entries of the load analyzer (2026-11-18
//! boost collateral-pass fixes 2 and 3), split from `load.rs` to keep it
//! under the R2 line limit. A child module of `load`, so it reaches the
//! analyzer's private fields and helpers (`build_metric`,
//! `symbol_record_and_def_file`) directly.

use super::{LoadAnalyzer, LoadMetric};
use anyhow::{anyhow, Result};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use std::collections::HashSet;
use std::sync::Arc;

/// Physical expansion of ONE defining file (the union of `file_symbols` over
/// the file's transitive includes, the file included), tagged with that file.
/// Built by `LoadAnalyzer::physical_expansion`; the tag lets
/// `analyze_symbol_load_with_expansion` reject a symbol from another file.
#[derive(Debug, Clone)]
pub struct DefFileExpansion {
    def_file: FileId,
    symbols: Arc<HashSet<SymbolId>>,
}

impl DefFileExpansion {
    /// The defining file this expansion was built for.
    pub fn def_file(&self) -> FileId {
        self.def_file
    }
}

impl<'a> LoadAnalyzer<'a> {
    /// `symbol_record_and_def_file`, plus the refusal of an expansion built
    /// for a different defining file than the symbol's.
    fn record_matching_expansion(
        &self,
        symbol_id: &SymbolId,
        expansion: &DefFileExpansion,
    ) -> Result<(&'a crate::ReducedSymbolRecord, FileId)> {
        let (record, def_file) = self.symbol_record_and_def_file(symbol_id)?;
        if def_file != expansion.def_file {
            return Err(anyhow!(
                "Expansion is for a different defining file than symbol: {}",
                symbol_id.0
            ));
        }
        Ok((record, def_file))
    }

    /// Counts-only symbol load (2026-11-18 boost memory fix): equals
    /// `analyze_symbol_load_with_expansion(..).summary_only()` without
    /// building the detail vectors. At boost scale those vectors held ~546M
    /// cloned ids (~26 GB) that nothing reads - the persisted report keeps
    /// only these scalars. `semantic_load` is |expansion intersect closure|, counted
    /// by probing the larger set while walking the smaller one: no clones.
    pub fn analyze_symbol_load_counts_with_expansion(
        &self,
        symbol_id: &SymbolId,
        expansion: &DefFileExpansion,
    ) -> Result<LoadMetric> {
        let (record, def_file) = self.record_matching_expansion(symbol_id, expansion)?;
        let semantic_closure = self
            .memo
            .cached_single_seed_closure(self.model, symbol_id);
        let physical_symbols: &HashSet<SymbolId> = &expansion.symbols;
        let (smaller, larger) = if semantic_closure.len() <= physical_symbols.len() {
            (&*semantic_closure, physical_symbols)
        } else {
            (physical_symbols, &*semantic_closure)
        };
        let semantic_load = smaller.iter().filter(|sym| larger.contains(*sym)).count();
        let include_load = physical_symbols.len();
        Ok(LoadMetric {
            target_name: record.name.clone(),
            target_file: self.include_graph.file_paths.get(&def_file).cloned(),
            semantic_load,
            include_load,
            collateral_load: include_load - semantic_load,
            total_load: include_load,
            semantic_symbols: Vec::new(),
            collateral_symbols: Vec::new(),
            collateral_sources: Vec::new(),
            largest_collateral_source: None,
            semantic_sources: Vec::new(),
        })
    }

    /// Physical expansion of one defining file, tagged with that file so
    /// `analyze_symbol_load_with_expansion` can refuse a mismatched symbol.
    /// Served through the memo (an admitted entry is a refcount bump), so a
    /// caller that groups symbols by defining file expands each file once
    /// even after the capped boot memo is full.
    pub fn physical_expansion(&self, def_file: FileId) -> DefFileExpansion {
        DefFileExpansion {
            def_file,
            symbols: self
                .memo
                .cached_physical_expansion(self.include_graph, self.model, def_file),
        }
    }

    /// Symbol load over a caller-held expansion of the symbol's defining
    /// file: identical metric to `analyze_symbol_load_by_id`, without
    /// re-expanding the file per symbol. Errors on the by-id miss paths,
    /// and when `expansion` belongs to a different defining file.
    pub fn analyze_symbol_load_with_expansion(
        &self,
        symbol_id: &SymbolId,
        expansion: &DefFileExpansion,
    ) -> Result<LoadMetric> {
        let (record, def_file) = self.record_matching_expansion(symbol_id, expansion)?;
        let target_file_path = self.include_graph.file_paths.get(&def_file).cloned();
        // Borrow the memoized `Arc` closure directly: the per-symbol pass
        // calls this once per symbol, so cloning the set is the cost to avoid.
        let semantic_closure = self
            .memo
            .cached_single_seed_closure(self.model, symbol_id);
        Ok(self.build_metric(
            record.name.clone(),
            target_file_path,
            &expansion.symbols,
            &semantic_closure,
        ))
    }
}
