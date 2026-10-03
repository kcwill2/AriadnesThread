//! Isolation engine: grouping planned extractions into generated units
//! (plan §3.6). The engine owns grouping, inter-symbol dependency edges,
//! the unit-cycle check, and the disk write; the new-file *text* is built by
//! a language plugin through the neutral `SourceExtractor` hook.
//!
//! Module map: `bodies` resolves verbatim declaration bodies through the
//! plugin locator (D-M23-1), `write` renders and writes generated units,
//! `tests` holds the orchestration/body-resolution characterization suite.

mod bodies;
mod write;

use crate::{ExtractedSymbol, ExtractionResult, GeneratedUnit};
use ariadne_cluster::plan::OptimizationPlan;
use ariadne_graph::reduced::ReducedSemanticModel;
use ariadne_index::SymbolKind;
use ariadne_lang::SourceExtractor;
use ariadne_repo::FileId;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub struct IsolationEngine<'a> {
    model: &'a ReducedSemanticModel,
}

impl<'a> IsolationEngine<'a> {
    pub fn new(model: &'a ReducedSemanticModel) -> Self {
        Self { model }
    }

    /// Group the plan's extractions into generated units.
    ///
    /// - `sources`: reads the text of a planned extraction's `source_file`
    ///   (callers read disk; `|_| None` keeps every body on the placeholder
    ///   fallback, invariant I4).
    /// - `extractor`: the language plugin asked to locate verbatim declaration
    ///   bodies in those source texts (D-M23-1). The engine itself names only
    ///   this neutral trait.
    pub fn extract(
        &self,
        plan: &OptimizationPlan,
        sources: &dyn Fn(FileId) -> Option<String>,
        extractor: &dyn SourceExtractor,
    ) -> ExtractionResult {
        let mut groups: BTreeMap<PathBuf, Vec<&ariadne_cluster::plan::ProposedExtraction>> =
            BTreeMap::new();
        let mut symbol_to_unit: BTreeMap<ariadne_index::types::SymbolId, PathBuf> =
            BTreeMap::new();

        for ext in &plan.proposed_extractions {
            groups
                .entry(ext.target_header.clone())
                .or_default()
                .push(ext);
            symbol_to_unit.insert(ext.symbol_id.clone(), ext.target_header.clone());
        }

        let mut units = Vec::new();

        for (target_path, extractions) in groups {
            units.push(self.build_generated_unit(
                target_path,
                extractions,
                &symbol_to_unit,
                sources,
                extractor,
            ));
        }

        let mut errors = vec![];

        if Self::detect_cycles_in_units(&units) {
            errors.push("Cycle detected in generated headers".to_string());
        }

        ExtractionResult {
            plan_id: plan.id.0.clone(),
            headers: units,
            errors,
        }
    }

    fn build_generated_unit(
        &self,
        target_path: PathBuf,
        extractions: Vec<&ariadne_cluster::plan::ProposedExtraction>,
        symbol_to_unit: &BTreeMap<ariadne_index::types::SymbolId, PathBuf>,
        sources: &dyn Fn(FileId) -> Option<String>,
        extractor: &dyn SourceExtractor,
    ) -> GeneratedUnit {
        let (located_bodies, primary_source_text) =
            bodies::locate_bodies_for_unit(self.model, &extractions, sources, extractor);

        let mut symbols = Vec::new();
        let mut required_deps = BTreeSet::new();

        for ext in extractions {
            let kind = self
                .model
                .find_symbol(&ext.symbol_id.0)
                .map(|r| r.kind)
                .unwrap_or(SymbolKind::Other);

            let body = match located_bodies.get(&ext.symbol_name) {
                Some(verbatim) => verbatim.clone(),
                None => format!("// Symbol: {} ({:?})", ext.symbol_name, kind),
            };

            symbols.push(ExtractedSymbol {
                name: ext.symbol_name.clone(),
                qualified_id: ext.symbol_id.0.clone(),
                kind,
                body,
            });

            for edge in self.model.get_dependencies(&ext.symbol_id) {
                if let Some(dep_unit) = symbol_to_unit.get(&edge.dst_symbol) {
                    if dep_unit != &target_path {
                        required_deps.insert(dep_unit.clone());
                    }
                }
            }
        }

        GeneratedUnit {
            path: target_path,
            symbols,
            required_deps: required_deps.into_iter().collect(),
            primary_source_text,
        }
    }

    fn detect_cycles_in_units(units: &[GeneratedUnit]) -> bool {
        let mut adj: BTreeMap<PathBuf, BTreeSet<PathBuf>> = BTreeMap::new();
        for u in units {
            adj.insert(
                u.path.clone(),
                u.required_deps.iter().cloned().collect(),
            );
        }

        fn has_cycle(
            node: &PathBuf,
            adj: &BTreeMap<PathBuf, BTreeSet<PathBuf>>,
            visited: &mut BTreeSet<PathBuf>,
            stack: &mut BTreeSet<PathBuf>,
        ) -> bool {
            visited.insert(node.clone());
            stack.insert(node.clone());

            if let Some(neighbors) = adj.get(node) {
                for n in neighbors {
                    if !visited.contains(n) {
                        if has_cycle(n, adj, visited, stack) {
                            return true;
                        }
                    } else if stack.contains(n) {
                        return true;
                    }
                }
            }
            stack.remove(node);
            false
        }

        let mut visited = BTreeSet::new();
        let mut stack = BTreeSet::new();
        let mut cycle_found = false;

        for u in units {
            if !visited.contains(&u.path) {
                if has_cycle(&u.path, &adj, &mut visited, &mut stack) {
                    cycle_found = true;
                    break;
                }
            }
        }

        cycle_found
    }
}

#[cfg(test)]
mod tests;
