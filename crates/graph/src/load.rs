use crate::{IncludeClosure, IncludeGraph, ReducedSemanticModel};
use anyhow::{anyhow, Result};
use ariadne_index::types::SymbolId;
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollateralSource {
    pub file_id: FileId,
    pub path: PathBuf,
    pub symbol_count: usize,
    pub symbols: Vec<SymbolId>,
}

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

        let mut collateral_symbols = Vec::new();
        let mut col_by_file: HashMap<FileId, Vec<SymbolId>> = HashMap::new();

        for sym in &physical_symbols {
            if !semantic_closure.contains(sym) {
                collateral_symbols.push(sym.clone());
                if let Some(record) = self.model.symbols.get(sym) {
                    if let Some(f) = record.defining_file {
                        col_by_file.entry(f).or_default().push(sym.clone());
                    }
                }
            }
        }

        let mut collateral_sources = Vec::new();
        for (f_id, syms) in col_by_file {
            let path = self
                .include_graph
                .file_paths
                .get(&f_id)
                .cloned()
                .unwrap_or_else(|| PathBuf::from("unknown"));
            collateral_sources.push(CollateralSource {
                file_id: f_id,
                path,
                symbol_count: syms.len(),
                symbols: syms,
            });
        }

        collateral_sources.sort_by(|a, b| {
            b.symbol_count
                .cmp(&a.symbol_count)
                .then_with(|| a.path.cmp(&b.path))
        });

        let largest_collateral_source = collateral_sources.first().cloned();

        let mut semantic_sym_vec: Vec<SymbolId> = physical_symbols
            .iter()
            .filter(|s| semantic_closure.contains(s))
            .cloned()
            .collect();

        semantic_sym_vec.sort();
        collateral_symbols.sort();

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
        })
    }

    pub fn analyze_symbol_load(&self, symbol_name_or_id: &str) -> Result<LoadMetric> {
        let sym_id = self
            .find_symbol_id(symbol_name_or_id)
            .ok_or_else(|| anyhow!("Symbol not found: {}", symbol_name_or_id))?;

        let record = self.model.symbols.get(&sym_id).unwrap();
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
            let current = *path.last().unwrap();
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

        let record = self
            .model
            .symbols
            .get(&self.find_symbol_id(symbol_name_or_id).unwrap())
            .unwrap();
        let target_file_id = record.defining_file.unwrap();

        let path_ids = self
            .find_shortest_include_path(target_file_id, col_source.file_id)
            .ok_or_else(|| anyhow!("No include path found to collateral source"))?;

        let mut path_strs = Vec::new();
        path_strs.push(metric.target_name.clone());
        for id in &path_ids {
            let p = self.include_graph.file_paths.get(id).unwrap();
            path_strs.push(p.file_name().unwrap().to_string_lossy().to_string());
        }

        let parent_file_id = if path_ids.len() >= 2 {
            path_ids[path_ids.len() - 2]
        } else {
            path_ids[0]
        };
        let parent_name = self
            .include_graph
            .file_paths
            .get(&parent_file_id)
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let col_name = col_source
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let reason = format!(
            "{} is included by {}.\nNo semantic dependency from {} reaches symbols owned by {}.",
            col_name, parent_name, metric.target_name, col_name
        );

        Ok(LoadExplanation {
            target_name: metric.target_name,
            target_file: metric.target_file.unwrap(),
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
        let file_id = self.include_graph.find_file_id(file_path_or_id).unwrap();

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
            let p = self.include_graph.file_paths.get(id).unwrap();
            path_strs.push(p.file_name().unwrap().to_string_lossy().to_string());
        }

        let parent_file_id = if path_ids.len() >= 2 {
            path_ids[path_ids.len() - 2]
        } else {
            path_ids[0]
        };
        let parent_name = self
            .include_graph
            .file_paths
            .get(&parent_file_id)
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let col_name = col_source
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let reason = format!(
            "{} is included by {}.\nNo semantic dependency from {} reaches symbols owned by {}.",
            col_name, parent_name, metric.target_name, col_name
        );

        Ok(LoadExplanation {
            target_name: metric.target_name,
            target_file: metric.target_file.unwrap(),
            path: path_strs,
            collateral_source: col_name,
            collateral_symbols_count: col_source.symbol_count,
            reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reduced::ReducedSymbolRecord;
    use crate::types::{IncludeEdge, IncludeKind, SourceLocation};
    use ariadne_index::types::{SymbolId, SymbolKind};
    use ariadne_repo::FileId;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn setup_mock_data() -> (IncludeGraph, ReducedSemanticModel) {
        let mut ig = IncludeGraph::new();
        ig.register_file(FileId(1), PathBuf::from("Thing.h"));
        ig.register_file(FileId(2), PathBuf::from("graphics.h"));
        ig.register_file(FileId(3), PathBuf::from("HugeThirdParty.h"));

        ig.add_edge(IncludeEdge {
            source: FileId(1),
            destination: Some(FileId(2)),
            context_name: "".to_string(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "graphics.h".to_string(),
            resolved_path: Some(PathBuf::from("graphics.h")),
        });
        ig.add_edge(IncludeEdge {
            source: FileId(2),
            destination: Some(FileId(3)),
            context_name: "".to_string(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: "HugeThirdParty.h".to_string(),
            resolved_path: Some(PathBuf::from("HugeThirdParty.h")),
        });

        let mut model = ReducedSemanticModel {
            repo_root: PathBuf::from("/"),
            symbols: BTreeMap::new(),
            edges: vec![],
            tu_usage: BTreeMap::new(),
            file_symbols: BTreeMap::new(),
        };

        let sym_thing = SymbolId::new("Thing");
        model.symbols.insert(
            sym_thing.clone(),
            ReducedSymbolRecord {
                id: sym_thing.clone(),
                name: "Thing".to_string(),
                kind: SymbolKind::Class,
                defining_file: Some(FileId(1)),
                declarations: vec![],
                definition: None,
            },
        );
        model
            .file_symbols
            .insert(FileId(1), vec![sym_thing.clone()]);

        let sym_huge = SymbolId::new("HugeThing");
        model.symbols.insert(
            sym_huge.clone(),
            ReducedSymbolRecord {
                id: sym_huge.clone(),
                name: "HugeThing".to_string(),
                kind: SymbolKind::Class,
                defining_file: Some(FileId(3)),
                declarations: vec![],
                definition: None,
            },
        );
        model.file_symbols.insert(FileId(3), vec![sym_huge.clone()]);

        (ig, model)
    }

    #[test]
    fn test_analyze_symbol_load() {
        let (ig, model) = setup_mock_data();
        let analyzer = LoadAnalyzer::new(&ig, &model);

        let metric = analyzer
            .analyze_symbol_load("Thing")
            .expect("analyze_symbol_load failed");
        assert_eq!(metric.target_name, "Thing");
        assert_eq!(metric.semantic_load, 1);
        assert_eq!(metric.include_load, 2);
        assert_eq!(metric.collateral_load, 1);
        assert_eq!(metric.total_load, 2);
        assert_eq!(
            metric.largest_collateral_source.as_ref().unwrap().path,
            PathBuf::from("HugeThirdParty.h")
        );
    }

    #[test]
    fn test_explain_symbol_load() {
        let (ig, model) = setup_mock_data();
        let analyzer = LoadAnalyzer::new(&ig, &model);

        let explanation = analyzer
            .explain_symbol_load("Thing", None)
            .expect("explain_symbol_load failed");
        assert_eq!(explanation.target_name, "Thing");
        assert_eq!(explanation.collateral_source, "HugeThirdParty.h");
        assert_eq!(
            explanation.path,
            vec!["Thing", "Thing.h", "graphics.h", "HugeThirdParty.h"]
        );

        let formatted = explanation.format_explanation();
        assert!(formatted.contains("HugeThirdParty.h is included by graphics.h."));
        assert!(formatted.contains(
            "No semantic dependency from Thing reaches symbols owned by HugeThirdParty.h."
        ));
    }

    #[test]
    fn test_analyze_file_load() {
        let (ig, model) = setup_mock_data();
        let analyzer = LoadAnalyzer::new(&ig, &model);

        let metric = analyzer
            .analyze_file_load(std::path::Path::new("Thing.h"))
            .expect("analyze_file_load failed");
        assert_eq!(metric.target_name, "Thing.h");
        assert_eq!(metric.semantic_load, 1);
        assert_eq!(metric.include_load, 2);
        assert_eq!(metric.collateral_load, 1);
        assert_eq!(metric.total_load, 2);
        assert_eq!(
            metric.largest_collateral_source.as_ref().unwrap().path,
            PathBuf::from("HugeThirdParty.h")
        );

        // Error case
        assert!(analyzer
            .analyze_file_load(std::path::Path::new("NonExistent.h"))
            .is_err());
    }

    #[test]
    fn test_explain_file_load() {
        let (ig, model) = setup_mock_data();
        let analyzer = LoadAnalyzer::new(&ig, &model);

        // Automatic largest collateral source
        let explanation = analyzer
            .explain_file_load(std::path::Path::new("Thing.h"), None)
            .expect("explain_file_load failed");
        assert_eq!(explanation.target_name, "Thing.h");
        assert_eq!(explanation.collateral_source, "HugeThirdParty.h");
        assert_eq!(
            explanation.path,
            vec!["Thing.h", "graphics.h", "HugeThirdParty.h"]
        );

        // Explicit collateral source
        let explanation_explicit = analyzer
            .explain_file_load(std::path::Path::new("Thing.h"), Some("HugeThirdParty"))
            .expect("explain_file_load failed");
        assert_eq!(explanation_explicit.collateral_source, "HugeThirdParty.h");

        // Error cases
        assert!(analyzer
            .explain_file_load(std::path::Path::new("NonExistent.h"), None)
            .is_err());
        assert!(analyzer
            .explain_file_load(std::path::Path::new("HugeThirdParty.h"), None)
            .is_err());
    }

    #[test]
    fn test_format_summary() {
        let (ig, model) = setup_mock_data();
        let analyzer = LoadAnalyzer::new(&ig, &model);

        // With largest collateral source
        let metric = analyzer
            .analyze_file_load(std::path::Path::new("Thing.h"))
            .unwrap();
        let summary = metric.format_summary();
        assert!(summary.contains("Thing.h"));
        assert!(summary.contains("Semantic load:"));
        assert!(summary.contains("1 symbols"));
        assert!(summary.contains("Largest collateral source:"));
        assert!(summary.contains("HugeThirdParty.h"));

        // Without collateral source
        let metric_empty = analyzer
            .analyze_file_load(std::path::Path::new("HugeThirdParty.h"))
            .unwrap();
        let summary_empty = metric_empty.format_summary();
        assert!(summary_empty.contains("None"));
        assert!(summary_empty.contains("0 symbols"));
    }
}
