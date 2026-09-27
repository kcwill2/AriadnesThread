use crate::{IncludeClosure, IncludeGraph, ReducedSemanticModel};
use anyhow::{anyhow, Result};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Renders a path's final component, falling back to the full path for names
/// without one (e.g. `..`), mirroring `format_summary`'s file_name fallback.
fn path_basename(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

/// E-I9 (plan 1790521470000): needs-sets carry includable headers only —
/// `.h`/`.hpp`/`.hxx`, case-sensitive, mirroring the ui node-kind detection
/// in `dto.rs`. `.ipp`/`.inl` template-impl files are deliberately OUT for
/// now: whether they belong in the includable set is a future question, not
/// today's. Paths without a final component (e.g. `..`, empty) are never
/// includable.
pub fn is_includable_header(path: &Path) -> bool {
    match path.file_name() {
        Some(name) => {
            let name = name.to_string_lossy();
            name.ends_with(".h") || name.ends_with(".hpp") || name.ends_with(".hxx")
        }
        None => false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContributingSource {
    pub file_id: FileId,
    pub path: PathBuf,
    pub symbol_count: usize,
    pub symbols: Vec<SymbolId>,
}

/// Collateral sources are contributing sources on the unused side of the
/// include-closure split; the alias keeps existing consumers unchanged.
pub type CollateralSource = ContributingSource;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadMetric {
    pub target_name: String,
    pub target_file: Option<PathBuf>,
    pub semantic_load: usize,
    pub include_load: usize,
    pub collateral_load: usize,
    pub total_load: usize,
    pub semantic_symbols: Vec<SymbolId>,
    pub collateral_symbols: Vec<SymbolId>,
    pub collateral_sources: Vec<CollateralSource>,
    pub largest_collateral_source: Option<CollateralSource>,
    #[serde(default)]
    pub semantic_sources: Vec<ContributingSource>,
}

impl LoadMetric {
    pub fn format_summary(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.target_name);
        out.push_str("\n\nSemantic load:\n");
        out.push_str(&format!("    {} symbols\n\n", self.semantic_load));

        out.push_str("Include load:\n");
        out.push_str(&format!("    {} symbols\n\n", self.include_load));

        out.push_str("Collateral load:\n");
        out.push_str(&format!("    {} symbols\n\n", self.collateral_load));

        out.push_str("Largest collateral source:\n");
        if let Some(ref lcs) = self.largest_collateral_source {
            if let Some(name) = lcs.path.file_name().and_then(|s| s.to_str()) {
                out.push_str(&format!("    {}\n", name));
            } else {
                out.push_str(&format!("    {}\n", lcs.path.display()));
            }
            out.push_str(&format!("    {} symbols\n", lcs.symbol_count));
        } else {
            out.push_str("    None\n    0 symbols\n");
        }

        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadExplanation {
    pub target_name: String,
    pub target_file: PathBuf,
    pub path: Vec<String>,
    pub collateral_source: String,
    pub collateral_symbols_count: usize,
    pub reason: String,
}

impl LoadExplanation {
    pub fn format_explanation(&self) -> String {
        let mut out = String::new();
        let path_len = self.path.len();
        for (i, node) in self.path.iter().enumerate() {
            out.push_str(node);
            out.push('\n');
            if i < path_len - 1 {
                out.push_str("  ↓\n");
            }
        }

        out.push_str("\nReason:\n");
        let indented_reason = self
            .reason
            .split('\n')
            .map(|line| format!("  {}", line))
            .collect::<Vec<_>>()
            .join("\n");
        out.push_str(&indented_reason);
        out.push('\n');

        out
    }
}

pub struct LoadAnalyzer<'a> {
    pub include_graph: &'a IncludeGraph,
    pub model: &'a ReducedSemanticModel,
    closures: RefCell<Option<HashMap<FileId, IncludeClosure>>>,
    symbol_adjacency: RefCell<Option<HashMap<SymbolId, Vec<SymbolId>>>>,
}

impl<'a> LoadAnalyzer<'a> {
    pub fn new(include_graph: &'a IncludeGraph, model: &'a ReducedSemanticModel) -> Self {
        Self {
            include_graph,
            model,
            closures: RefCell::new(None),
            symbol_adjacency: RefCell::new(None),
        }
    }

    /// Reuse include closures already computed elsewhere (e.g. by `build_include_graph`)
    /// instead of recomputing them lazily on the first query.
    pub fn with_closures(
        include_graph: &'a IncludeGraph,
        model: &'a ReducedSemanticModel,
        closures: HashMap<FileId, IncludeClosure>,
    ) -> Self {
        Self {
            include_graph,
            model,
            closures: RefCell::new(Some(closures)),
            symbol_adjacency: RefCell::new(None),
        }
    }

    fn find_symbol_id(&self, symbol_name_or_id: &str) -> Option<SymbolId> {
        if let Some((k, _)) = self
            .model
            .symbols
            .iter()
            .find(|(k, v)| k.0 == symbol_name_or_id || v.name == symbol_name_or_id)
        {
            return Some(k.clone());
        }
        None
    }

    /// Renders the file name for a graph file id, falling back to the
    /// `"unknown"` placeholder `compute_load` uses when the id is referenced
    /// only by edges and never registered in `file_paths`.
    fn file_basename(&self, id: FileId) -> String {
        match self.include_graph.file_paths.get(&id) {
            Some(path) => path_basename(path),
            None => "unknown".to_string(),
        }
    }

    fn get_transitive_includes(&self, file_id: FileId) -> HashSet<FileId> {
        let mut guard = self.closures.borrow_mut();
        let closures = guard.get_or_insert_with(|| self.include_graph.compute_closures());
        let mut all = HashSet::new();
        all.insert(file_id);
        if let Some(closure) = closures.get(&file_id) {
            all.extend(closure.transitive_includes.iter().copied());
        }
        all
    }

    fn get_semantic_closure_of_symbols(&self, start_symbols: &[SymbolId]) -> HashSet<SymbolId> {
        let mut guard = self.symbol_adjacency.borrow_mut();
        let adjacency = guard.get_or_insert_with(|| {
            let mut adj: HashMap<SymbolId, Vec<SymbolId>> = HashMap::new();
            for edge in &self.model.edges {
                adj.entry(edge.src_symbol.clone())
                    .or_default()
                    .push(edge.dst_symbol.clone());
            }
            adj
        });

        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        for s in start_symbols {
            stack.push(s.clone());
            visited.insert(s.clone());
        }

        while let Some(current) = stack.pop() {
            if let Some(nexts) = adjacency.get(&current) {
                for next in nexts {
                    if !visited.contains(next) {
                        visited.insert(next.clone());
                        stack.push(next.clone());
                    }
                }
            }
        }
        visited
    }

    /// Groups symbols by defining file into contributor entries, applying the
    /// shared `(symbol_count desc, path asc)` ordering contract.
    fn contributing_sources(&self, symbols: &[SymbolId]) -> Vec<ContributingSource> {
        let mut by_file: HashMap<FileId, Vec<SymbolId>> = HashMap::new();
        for sym in symbols {
            if let Some(record) = self.model.symbols.get(sym) {
                if let Some(f) = record.defining_file {
                    by_file.entry(f).or_default().push(sym.clone());
                }
            }
        }

        let mut sources: Vec<ContributingSource> = by_file
            .into_iter()
            .map(|(file_id, syms)| ContributingSource {
                path: self
                    .include_graph
                    .file_paths
                    .get(&file_id)
                    .cloned()
                    .unwrap_or_else(|| PathBuf::from("unknown")),
                file_id,
                symbol_count: syms.len(),
                symbols: syms,
            })
            .collect();

        sources.sort_by(|a, b| {
            b.symbol_count
                .cmp(&a.symbol_count)
                .then_with(|| a.path.cmp(&b.path))
        });
        sources
    }

    /// Required headers for a set of symbols: the defining files of the
    /// semantic closure of `symbol_ids` (with contributing closure-symbol
    /// counts), EXCLUDING the seeds' own defining files — a header does not
    /// "require" the file the seed itself is defined in. The exclusion is
    /// per seed defining-file, not per symbol: a non-seed closure symbol is
    /// dropped too whenever its defining file equals ANY seed defining file.
    /// Rows are includable headers only (E-I9): a closure symbol defined in
    /// a translation unit (`.cc`/`.cpp`/...) contributes no include
    /// requirement — it is satisfied by a declaration elsewhere or by
    /// linking — and defining files unregistered in `file_paths` cannot be
    /// shown to be headers, so they are dropped as well. Filtering happens
    /// before grouping, so counts never fold in source files. Unknown
    /// symbol ids are ignored. Ordered `(count desc, path asc)` per I6,
    /// sharing the grouping contract of `contributing_sources`.
    pub fn required_headers_for_symbols(
        &self,
        symbol_ids: &[SymbolId],
    ) -> Vec<(FileId, PathBuf, usize)> {
        let seed_files: HashSet<FileId> = symbol_ids
            .iter()
            .filter_map(|s| self.model.symbols.get(s).and_then(|r| r.defining_file))
            .collect();
        let closure = self.get_semantic_closure_of_symbols(symbol_ids);
        let required: Vec<SymbolId> = closure
            .into_iter()
            .filter(|sym| {
                self.model
                    .symbols
                    .get(sym)
                    .and_then(|r| r.defining_file)
                    .is_some_and(|f| {
                        !seed_files.contains(&f)
                            && self
                                .include_graph
                                .file_paths
                                .get(&f)
                                .is_some_and(|p| is_includable_header(p))
                    })
            })
            .collect();
        self.contributing_sources(&required)
            .into_iter()
            .map(|src| (src.file_id, src.path, src.symbol_count))
            .collect()
    }

    fn compute_load(
        &self,
        target_name: String,
        target_file: Option<PathBuf>,
        def_files: HashSet<FileId>,
        semantic_closure: HashSet<SymbolId>,
    ) -> Result<LoadMetric> {
        let mut physical_include_files = HashSet::new();
        for f in def_files {
            physical_include_files.extend(self.get_transitive_includes(f));
        }

        let mut physical_symbols = HashSet::new();
        for f in &physical_include_files {
            if let Some(syms) = self.model.file_symbols.get(f) {
                physical_symbols.extend(syms.iter().cloned());
            }
        }

        let mut collateral_symbols: Vec<SymbolId> = physical_symbols
            .iter()
            .filter(|s| !semantic_closure.contains(s))
            .cloned()
            .collect();
        let mut semantic_sym_vec: Vec<SymbolId> = physical_symbols
            .iter()
            .filter(|s| semantic_closure.contains(s))
            .cloned()
            .collect();

        collateral_symbols.sort();
        semantic_sym_vec.sort();

        let collateral_sources = self.contributing_sources(&collateral_symbols);
        let largest_collateral_source = collateral_sources.first().cloned();
        let semantic_sources = self.contributing_sources(&semantic_sym_vec);

        Ok(LoadMetric {
            target_name,
            target_file,
            semantic_load: semantic_sym_vec.len(),
            include_load: physical_symbols.len(),
            collateral_load: collateral_symbols.len(),
            total_load: physical_symbols.len(),
            semantic_symbols: semantic_sym_vec,
            collateral_symbols,
            collateral_sources,
            largest_collateral_source,
            semantic_sources,
        })
    }

    pub fn analyze_symbol_load(&self, symbol_name_or_id: &str) -> Result<LoadMetric> {
        let sym_id = self
            .find_symbol_id(symbol_name_or_id)
            .ok_or_else(|| anyhow!("Symbol not found: {}", symbol_name_or_id))?;

        let record = self
            .model
            .symbols
            .get(&sym_id)
            .ok_or_else(|| anyhow!("Symbol record not found for: {}", symbol_name_or_id))?;
        let def_file = record
            .defining_file
            .ok_or_else(|| anyhow!("No defining file for symbol"))?;
        let target_file_path = self.include_graph.file_paths.get(&def_file).cloned();

        let semantic_closure = self.get_semantic_closure_of_symbols(&[sym_id]);

        let mut def_files = HashSet::new();
        def_files.insert(def_file);

        self.compute_load(
            record.name.clone(),
            target_file_path,
            def_files,
            semantic_closure,
        )
    }

    pub fn analyze_file_load(&self, file_path_or_id: &Path) -> Result<LoadMetric> {
        let file_id = self
            .include_graph
            .find_file_id(file_path_or_id)
            .ok_or_else(|| anyhow!("File not found in include graph"))?;

        let target_file = self.include_graph.file_paths.get(&file_id).cloned();
        let target_name = target_file
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let syms = self
            .model
            .file_symbols
            .get(&file_id)
            .cloned()
            .unwrap_or_default();
        let semantic_closure = self.get_semantic_closure_of_symbols(&syms);

        let mut def_files = HashSet::new();
        def_files.insert(file_id);

        self.compute_load(target_name, target_file, def_files, semantic_closure)
    }

    fn find_shortest_include_path(&self, start: FileId, end: FileId) -> Option<Vec<FileId>> {
        if start == end {
            return Some(vec![start]);
        }

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(vec![start]);
        let mut visited = HashSet::new();
        visited.insert(start);

        while let Some(path) = queue.pop_front() {
            // Queue entries are seeded with `[start]` and grow one hop at a
            // time, so `last()` is a defensive guard, not a reachable None.
            let current = match path.last() {
                Some(&id) => id,
                None => continue,
            };
            for dest in self.include_graph.direct_includes(current) {
                if dest == end {
                    let mut new_path = path.clone();
                    new_path.push(dest);
                    return Some(new_path);
                }
                if !visited.contains(&dest) {
                    visited.insert(dest);
                    let mut new_path = path.clone();
                    new_path.push(dest);
                    queue.push_back(new_path);
                }
            }
        }
        None
    }

    pub fn explain_symbol_load(
        &self,
        symbol_name_or_id: &str,
        source_filter: Option<&str>,
    ) -> Result<LoadExplanation> {
        let metric = self.analyze_symbol_load(symbol_name_or_id)?;

        let col_source = if let Some(filter) = source_filter {
            metric
                .collateral_sources
                .iter()
                .find(|cs| cs.path.to_string_lossy().contains(filter))
                .cloned()
        } else {
            metric.largest_collateral_source.clone()
        };

        let col_source =
            col_source.ok_or_else(|| anyhow!("No matching collateral source found"))?;

        let sym_id = self
            .find_symbol_id(symbol_name_or_id)
            .ok_or_else(|| anyhow!("Symbol not found: {}", symbol_name_or_id))?;
        let record = self
            .model
            .symbols
            .get(&sym_id)
            .ok_or_else(|| anyhow!("Symbol record not found for: {}", symbol_name_or_id))?;
        let target_file_id = record
            .defining_file
            .ok_or_else(|| anyhow!("No defining file for symbol"))?;

        let path_ids = self
            .find_shortest_include_path(target_file_id, col_source.file_id)
            .ok_or_else(|| anyhow!("No include path found to collateral source"))?;

        let mut path_strs = Vec::new();
        path_strs.push(metric.target_name.clone());
        for id in &path_ids {
            path_strs.push(self.file_basename(*id));
        }

        let parent_file_id = if path_ids.len() >= 2 {
            path_ids[path_ids.len() - 2]
        } else {
            path_ids[0]
        };
        let parent_name = self.file_basename(parent_file_id);
        let col_name = path_basename(&col_source.path);

        let reason = format!(
            "{} is included by {}.\nNo semantic dependency from {} reaches symbols owned by {}.",
            col_name, parent_name, metric.target_name, col_name
        );

        Ok(LoadExplanation {
            target_name: metric.target_name,
            target_file: metric
                .target_file
                .unwrap_or_else(|| PathBuf::from("unknown")),
            path: path_strs,
            collateral_source: col_name,
            collateral_symbols_count: col_source.symbol_count,
            reason,
        })
    }

    pub fn explain_file_load(
        &self,
        file_path_or_id: &Path,
        source_filter: Option<&str>,
    ) -> Result<LoadExplanation> {
        let metric = self.analyze_file_load(file_path_or_id)?;
        let file_id = self
            .include_graph
            .find_file_id(file_path_or_id)
            .ok_or_else(|| anyhow!("File not found in include graph"))?;

        let col_source = if let Some(filter) = source_filter {
            metric
                .collateral_sources
                .iter()
                .find(|cs| cs.path.to_string_lossy().contains(filter))
                .cloned()
        } else {
            metric.largest_collateral_source.clone()
        };

        let col_source =
            col_source.ok_or_else(|| anyhow!("No matching collateral source found"))?;

        let path_ids = self
            .find_shortest_include_path(file_id, col_source.file_id)
            .ok_or_else(|| anyhow!("No include path found to collateral source"))?;

        let mut path_strs = Vec::new();
        for id in &path_ids {
            path_strs.push(self.file_basename(*id));
        }

        let parent_file_id = if path_ids.len() >= 2 {
            path_ids[path_ids.len() - 2]
        } else {
            path_ids[0]
        };
        let parent_name = self.file_basename(parent_file_id);
        let col_name = path_basename(&col_source.path);

        let reason = format!(
            "{} is included by {}.\nNo semantic dependency from {} reaches symbols owned by {}.",
            col_name, parent_name, metric.target_name, col_name
        );

        Ok(LoadExplanation {
            target_name: metric.target_name,
            target_file: metric
                .target_file
                .unwrap_or_else(|| PathBuf::from("unknown")),
            path: path_strs,
            collateral_source: col_name,
            collateral_symbols_count: col_source.symbol_count,
            reason,
        })
    }
}

/// Characterization suite: locks CURRENT behavior of the load analyzer
/// (explain_symbol_load outputs incl. error paths) via mock fixtures;
/// moved to a sibling module following the discovery.rs pattern.
#[cfg(test)]
#[path = "load_characterization_tests.rs"]
mod characterization;

/// Justification suite: locks the semantic (used-symbol) side of the
/// include-closure split as grouped per-source contributions.
#[cfg(test)]
#[path = "load_justification_tests.rs"]
mod justification;

/// Extraction-impact suite: locks the drop-set semantics of
/// `extraction_impact` (Pass E, plan 1790521470000).
#[cfg(test)]
#[path = "load_impact_tests.rs"]
mod impact;
