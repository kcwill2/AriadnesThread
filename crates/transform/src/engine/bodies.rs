//! Verbatim body resolution for one generated unit (plan M23, D-M23-1/2/3).
//!
//! The unit's source files are deduplicated, the plugin locator is asked for
//! declaration blocks per readable file, and the first readable text is
//! carried as the unit's primary source for the namespace seam (D-M23-2).
//! Anything unreadable or unresolved keeps the engine's placeholder fallback
//! (invariant I4).

use ariadne_cluster::plan::ProposedExtraction;
use ariadne_graph::reduced::ReducedSemanticModel;
use ariadne_index::SymbolKind;
use ariadne_lang::SourceExtractor;
use ariadne_repo::FileId;
use std::collections::BTreeMap;

/// Ask the plugin to locate verbatim declaration bodies in the unit's source
/// files; returns the located bodies by symbol name plus the unit's primary
/// source text (first readable file, `None` when no source was readable).
pub(super) fn locate_bodies_for_unit(
    model: &ReducedSemanticModel,
    extractions: &[&ProposedExtraction],
    sources: &dyn Fn(FileId) -> Option<String>,
    extractor: &dyn SourceExtractor,
) -> (BTreeMap<String, String>, Option<String>) {
    let mut wanted_per_source: BTreeMap<FileId, Vec<(String, SymbolKind)>> = BTreeMap::new();
    for ext in extractions {
        if let Some(source_file) = ext.source_file {
            let kind = model
                .find_symbol(&ext.symbol_id.0)
                .map(|r| r.kind)
                .unwrap_or(SymbolKind::Other);
            wanted_per_source
                .entry(source_file)
                .or_default()
                .push((ext.symbol_name.clone(), kind));
        }
    }

    let mut located: BTreeMap<String, String> = BTreeMap::new();
    let mut primary_source_text: Option<String> = None;
    for (source_file, wanted) in wanted_per_source {
        let Some(source_text) = sources(source_file) else {
            continue;
        };
        if primary_source_text.is_none() {
            primary_source_text = Some(source_text.clone());
        }
        for body in extractor.locate_symbol_bodies(&source_text, &wanted) {
            located.entry(body.name).or_insert(body.body);
        }
    }
    (located, primary_source_text)
}
