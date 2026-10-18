//! Explanation surface of `LoadAnalyzer` (`LoadExplanation`, the two
//! `explain_*` entry points, and the shortest-include-path BFS they share),
//! relocated verbatim from `load.rs` so both files stay well under the R2
//! 1000-line hygiene limit. Pure code move: the characterization suite
//! locks these outputs byte-for-byte across the split.

use super::*;
use ariadne_lang_api::modeling::DependencyVocabulary;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadExplanation {
    pub target_name: String,
    pub target_file: PathBuf,
    pub path: Vec<String>,
    pub collateral_source: String,
    pub collateral_symbols_count: usize,
    pub reason: String,
    /// M22 T-E: the true parent FILE basename chosen from the BFS `path_ids`
    /// at the two `explain_*` sites (identical input to the stored reason's
    /// composition). `format_explanation_with` renders with this instead of
    /// recomposing from the display path, which for `explain_symbol_load`
    /// prepends the target NAME — a len==1 display path would otherwise put
    /// the symbol name in the parent slot ("X is imported by X").
    /// Asymmetry recorded deliberately: the stored `reason` template carries
    /// the same len==1 degeneracy (path_ids[0] == the collateral file makes
    /// it its own parent) and is m6-locked byte-for-byte; the *_with lane
    /// mirrors that true parent rather than re-deriving from `path`.
    #[serde(skip, default)]
    parent: String,
}

impl LoadExplanation {
    pub fn format_explanation(&self) -> String {
        render_explanation(&self.path, &self.reason)
    }

    /// D26 (plan:425): renders the reason line with the vocabulary's passive
    /// verb ("… is imported by …") instead of the stored C++ default wording,
    /// composed through `format!` so no double-space artifact can appear.
    /// The reason components are exactly the ones the `explain_*` builders
    /// template (collateral basename, second-to-last path node = parent
    /// basename, target name), so the default vocab renders today's bytes.
    pub fn format_explanation_with(&self, vocabulary: &DependencyVocabulary) -> String {
        // M22 T-E: render with the true `path_ids` parent carried from the
        // explain site. The display-path composition below only runs for
        // values built elsewhere (no site ran), where it matches the
        // file-only display shape.
        let parent = if self.parent.is_empty() {
            self.path
                .get(self.path.len().saturating_sub(2))
                .cloned()
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            self.parent.clone()
        };
        let reason = format!(
            "{} is {} by {}.\nNo semantic dependency from {} reaches symbols owned by {}.",
            self.collateral_source, vocabulary.directive_passive_verb, parent, self.target_name,
            self.collateral_source
        );
        render_explanation(&self.path, &reason)
    }
}

/// Shared output body of both formatters (moved verbatim): path tree with
/// `↓` connectors plus the indented `Reason:` block.
fn render_explanation(path: &[String], reason: &str) -> String {
    let mut out = String::new();
    let path_len = path.len();
    for (i, node) in path.iter().enumerate() {
        out.push_str(node);
        out.push('\n');
        if i < path_len - 1 {
            out.push_str("  ↓\n");
        }
    }

    out.push_str("\nReason:\n");
    let indented_reason = reason
        .split('\n')
        .map(|line| format!("  {}", line))
        .collect::<Vec<_>>()
        .join("\n");
    out.push_str(&indented_reason);
    out.push('\n');

    out
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
            parent: parent_name,
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
            parent: parent_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scala_vocabulary() -> DependencyVocabulary {
        DependencyVocabulary {
            directive_label: "Import".into(),
            directive_passive_verb: "imported".into(),
            aggregation_noun: "package aggregation".into(),
        }
    }

    /// Mirrors what `explain_symbol_load` builds for the C++ `basic`
    /// fixture's Thing/heavy.h collateral chain (m6's witness shape).
    fn explanation() -> LoadExplanation {
        LoadExplanation {
            target_name: "Thing".into(),
            target_file: PathBuf::from("include/thing.h"),
            path: vec!["Thing".into(), "thing.h".into(), "heavy.h".into()],
            collateral_source: "heavy.h".into(),
            collateral_symbols_count: 2,
            reason: "heavy.h is included by thing.h.\nNo semantic dependency from Thing \
                     reaches symbols owned by heavy.h."
                .into(),
            parent: "thing.h".into(),
        }
    }

    #[test]
    fn default_formatting_is_todays_exact_bytes() {
        assert_eq!(
            explanation().format_explanation(),
            "Thing\n  ↓\nthing.h\n  ↓\nheavy.h\n\nReason:\n  \
             heavy.h is included by thing.h.\n  \
             No semantic dependency from Thing reaches symbols owned by heavy.h.\n"
        );
    }

    #[test]
    fn scala_vocabulary_renders_is_imported_by() {
        let formatted = explanation().format_explanation_with(&scala_vocabulary());
        assert!(
            formatted.contains("heavy.h is imported by thing.h."),
            "expected import-flavored reason, got: {formatted}"
        );
        assert!(formatted.contains("No semantic dependency from Thing reaches symbols owned by heavy.h."));
        assert!(!formatted.contains("included"));
    }

    /// M22 T-E len==1 shape: `explain_symbol_load` when the collateral file
    /// IS the symbol's defining file — `path_ids` is a single node, the true
    /// parent the stored template uses is that file itself, and the display
    /// path is `[target_name, file]`. The vocabulary lane must render the
    /// true file parent (mirroring the stored reason's verb swap), never the
    /// target SYMBOL NAME as parent ("X is imported by X" with X = symbol).
    #[test]
    fn len1_symbol_path_renders_true_parent_not_target_name() {
        let explanation = LoadExplanation {
            target_name: "com.example.services.OrderService".into(),
            target_file: PathBuf::from("src/main/scala/com/example/services/OrderService.scala"),
            path: vec![
                "com.example.services.OrderService".into(),
                "OrderService.scala".into(),
            ],
            collateral_source: "OrderService.scala".into(),
            collateral_symbols_count: 1,
            reason: "OrderService.scala is included by OrderService.scala.\nNo semantic \
                     dependency from com.example.services.OrderService reaches symbols \
                     owned by OrderService.scala."
                .into(),
            parent: "OrderService.scala".into(),
        };
        let formatted = explanation.format_explanation_with(&scala_vocabulary());
        assert!(
            formatted.contains("OrderService.scala is imported by OrderService.scala."),
            "the len==1 lane mirrors the stored reason's true file parent: {formatted}"
        );
        assert!(
            !formatted.contains("OrderService is imported by"),
            "the target symbol name must never be rendered as the parent: {formatted}"
        );
    }

    #[test]
    fn scala_vocabulary_verb_composes_without_double_spaces() {
        let formatted = explanation().format_explanation_with(&scala_vocabulary());
        assert!(!formatted.contains("  by "), "verb substitution must not leave a double-space artifact: {formatted}");
        assert!(!formatted.contains("imported  by"));
    }
}

/// UTA-T1 item 9 (hand H1): behavior pins for the two `explain_*` entry
/// points around `source_filter` - a hit picks the matching collateral
/// source, a miss errors with `No matching collateral source found`, and a
/// collateral source with no include path from the target errors with
/// `No include path found to collateral source`.
#[cfg(test)]
mod source_filter_pins {
    use super::*;
    use crate::reduced::ReducedSymbolRecord;
    use ariadne_index::types::SymbolKind;
    use std::collections::BTreeMap;

    fn record(name: &str, defining_file: FileId) -> ReducedSymbolRecord {
        ReducedSymbolRecord {
            id: SymbolId::new(name),
            name: name.to_string(),
            kind: SymbolKind::Function,
            defining_file: Some(defining_file),
            declarations: Vec::new(),
            definition: None,
            declaring_files: Vec::new(),
        }
    }

    /// root.h -> mid.h carries `Root` (semantic), `Mid` and `Stray`.
    /// `Stray`'s defining file is elsewhere.h - REGISTERED but with no
    /// include path from either graph file - the reachable/unreachable
    /// collateral pair the filter and both error strings are pinned on.
    fn reachable_and_stray_fixture() -> (IncludeGraph, ReducedSemanticModel) {
        let mut ig = IncludeGraph::new();
        ig.register_file(FileId(1), PathBuf::from("root.h"));
        ig.register_file(FileId(2), PathBuf::from("mid.h"));
        ig.register_file(FileId(9), PathBuf::from("elsewhere.h"));
        ig.add_edge(crate::types::IncludeEdge {
            source: FileId(1),
            destination: Some(FileId(2)),
            context_name: String::new(),
            source_location: crate::types::SourceLocation::new(1, 1),
            kind: crate::types::IncludeKind::Local,
            raw_path: "mid.h".to_string(),
            resolved_path: None,
            language: crate::types::LanguageId::cpp(),
        });

        let mut model = ReducedSemanticModel {
            repo_root: PathBuf::from("/"),
            symbols: BTreeMap::new(),
            edges: vec![],
            tu_usage: BTreeMap::new(),
            file_symbols: BTreeMap::new(),
            extends_edges: Vec::new(),
            sealed_symbols: Vec::new(),
        };
        model
            .symbols
            .insert(SymbolId::new("Root"), record("Root", FileId(1)));
        model
            .symbols
            .insert(SymbolId::new("Mid"), record("Mid", FileId(2)));
        model
            .symbols
            .insert(SymbolId::new("Stray"), record("Stray", FileId(9)));
        model.file_symbols.insert(FileId(1), vec![SymbolId::new("Root")]);
        model.file_symbols.insert(
            FileId(2),
            vec![SymbolId::new("Mid"), SymbolId::new("Stray")],
        );
        (ig, model)
    }

    #[test]
    fn file_lane_filter_hit_renders_the_reachable_collateral() {
        let (ig, model) = reachable_and_stray_fixture();
        let analyzer = LoadAnalyzer::new(&ig, &model);

        let explanation = analyzer
            .explain_file_load(Path::new("root.h"), Some("mid"))
            .expect("mid.h is a collateral source reachable by include path");
        assert_eq!(explanation.collateral_source, "mid.h");
        assert_eq!(explanation.path, vec!["root.h", "mid.h"]);
        assert_eq!(explanation.collateral_symbols_count, 1);
        assert!(explanation.reason.starts_with("mid.h is included by root.h."));
    }

    #[test]
    fn file_lane_filter_miss_locks_matching_error_string() {
        let (ig, model) = reachable_and_stray_fixture();
        let analyzer = LoadAnalyzer::new(&ig, &model);

        let error = analyzer
            .explain_file_load(Path::new("root.h"), Some("no-such-source"))
            .expect_err("a filter matching no collateral path must error");
        assert_eq!(error.to_string(), "No matching collateral source found");
    }

    #[test]
    fn file_lane_locks_no_include_path_error_string() {
        // No filter: `largest_collateral_source` is elsewhere.h (the
        // path-ascending tie-break puts it before mid.h), and no include
        // path reaches it - the second "No ..." error string.
        let (ig, model) = reachable_and_stray_fixture();
        let analyzer = LoadAnalyzer::new(&ig, &model);

        let error = analyzer
            .explain_file_load(Path::new("root.h"), None)
            .expect_err("the largest collateral is unreachable");
        assert_eq!(
            error.to_string(),
            "No include path found to collateral source"
        );
    }

    #[test]
    fn symbol_lane_filter_hit_prefixes_the_display_path_with_the_target_name() {
        let (ig, model) = reachable_and_stray_fixture();
        let analyzer = LoadAnalyzer::new(&ig, &model);

        let explanation = analyzer
            .explain_symbol_load("Root", Some("mid"))
            .expect("mid.h is a collateral source reachable by include path");
        assert_eq!(explanation.collateral_source, "mid.h");
        assert_eq!(explanation.path, vec!["Root", "root.h", "mid.h"]);
        assert!(explanation.reason.starts_with("mid.h is included by root.h."));
    }

    #[test]
    fn symbol_lane_locks_both_error_strings() {
        let (ig, model) = reachable_and_stray_fixture();
        let analyzer = LoadAnalyzer::new(&ig, &model);

        let miss = analyzer
            .explain_symbol_load("Root", Some("no-such-source"))
            .expect_err("filter miss must error");
        assert_eq!(miss.to_string(), "No matching collateral source found");

        let unreachable = analyzer
            .explain_symbol_load("Root", Some("elsewhere"))
            .expect_err("unreachable collateral must error");
        assert_eq!(
            unreachable.to_string(),
            "No include path found to collateral source"
        );
    }
}
