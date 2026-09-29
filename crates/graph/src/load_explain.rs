//! Explanation surface of `LoadAnalyzer` (`LoadExplanation`, the two
//! `explain_*` entry points, and the shortest-include-path BFS they share),
//! relocated verbatim from `load.rs` so both files stay well under the R2
//! 1000-line hygiene limit. Pure code move: the characterization suite
//! locks these outputs byte-for-byte across the split.

use super::*;

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

impl<'a> LoadAnalyzer<'a> {
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
