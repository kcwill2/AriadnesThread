//! M23 T7b (kickoff §8 item 2): the defining-file filter. An extraction
//! proposal whose target path resolves back to the file that defines the
//! symbol is a no-op move and is never planned. The filter is a neutral
//! path equality between the candidate path and the injected
//! `target_path_for` result — shape-impossible for C++ `{name}_extracted.h`
//! targets (locked byte-identically by the m00 goldens and the untouched
//! planner characterization suites), provably C++-no-op.

use ariadne_cluster::plan::OptimizationPlan;
use ariadne_cluster::planner::OptimizationPlanner;
use ariadne_graph::reduced::{ReducedSemanticModel, ReducedSymbolRecord};
use ariadne_graph::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_graph::IncludeGraph;
use ariadne_index::types::{SymbolId, SymbolKind};
use ariadne_lang::{CyclePolicy, LanguageId};
use ariadne_repo::{
    FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const CANDIDATE: &str = "pkg/legacy.agg";

#[derive(Default)]
struct Fixture {
    graph: IncludeGraph,
    symbols: BTreeMap<SymbolId, ReducedSymbolRecord>,
    file_symbols: BTreeMap<FileId, Vec<SymbolId>>,
    records: Vec<FileRecord>,
}

impl Fixture {
    fn file(mut self, id: FileId, name: &str, optimizable: bool) -> Self {
        self.graph.register_file(id, PathBuf::from(name));
        self.records.push(FileRecord {
            id,
            path: PathBuf::from(name),
            canonical_path: PathBuf::from(name),
            kind: FileKind::Header,
            size: 0,
            content_hash: String::new(),
            ownership: if optimizable {
                FileOwnership::Repository
            } else {
                FileOwnership::ThirdParty
            },
            optimization_policy: if optimizable {
                OptimizationPolicy::Optimizable
            } else {
                OptimizationPolicy::Frozen
            },
            language: SourceLanguage::Other,
        });
        self
    }

    fn includes(mut self, source: FileId, dest: FileId) -> Self {
        let raw_path = self
            .graph
            .file_paths
            .get(&dest)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        self.graph.add_edge(IncludeEdge {
            source,
            destination: Some(dest),
            context_name: "Default".into(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: raw_path.clone(),
            resolved_path: Some(PathBuf::from(raw_path)),
            language: LanguageId::Other,
        });
        self
    }

    fn defines(mut self, file: FileId, name: &str) -> Self {
        let sym = SymbolId::new(name);
        self.symbols.insert(
            sym.clone(),
            ReducedSymbolRecord {
                id: sym.clone(),
                name: name.to_string(),
                kind: SymbolKind::Class,
                defining_file: Some(file),
                declarations: vec![],
                definition: None,
                declaring_files: vec![],
            },
        );
        self.file_symbols.entry(file).or_default().push(sym);
        self
    }

    fn plan_with(&self, target_path_for: &dyn Fn(&Path, &str) -> PathBuf) -> Vec<OptimizationPlan> {
        let model = ReducedSemanticModel {
            repo_root: PathBuf::from("/"),
            symbols: self.symbols.clone(),
            edges: vec![],
            tu_usage: BTreeMap::new(),
            file_symbols: self.file_symbols.clone(),
            extends_edges: Vec::new(),
            sealed_symbols: Vec::new(),
        };
        let accept_every = |path: &Path| path.file_name().is_some();
        OptimizationPlanner::new(
            &self.graph,
            &model,
            &self.records,
            &[],
            &accept_every,
            target_path_for,
            None,
            CyclePolicy::Reject,
        )
        .plan()
    }
}

/// Optimizable candidate `pkg/legacy.agg` (defines Alpha + Legacy) with
/// collateral load from a frozen source — the minimal shape that gets
/// proposed.
fn collateral_candidate() -> Fixture {
    Fixture::default()
        .file(FileId(1), CANDIDATE, true)
        .file(FileId(2), "col.agg", false)
        .includes(FileId(1), FileId(2))
        .defines(FileId(1), "Alpha")
        .defines(FileId(1), "Legacy")
        .defines(FileId(2), "Big")
}

#[test]
fn proposal_targeting_the_defining_file_is_dropped_others_survive() {
    let self_targeting = |path: &Path, symbol: &str| -> PathBuf {
        if symbol == "Legacy" {
            path.to_path_buf()
        } else {
            path.with_file_name(format!("{symbol}_unit"))
        }
    };
    let plans = collateral_candidate().plan_with(&self_targeting);

    assert_eq!(plans.len(), 1, "the candidate still plans via Alpha");
    let proposals = &plans[0].proposed_extractions;
    assert_eq!(
        proposals.iter().map(|e| e.symbol_name.as_str()).collect::<Vec<_>>(),
        vec!["Alpha"],
        "the self-targeted symbol is never proposed; the other survives"
    );
    assert!(
        proposals
            .iter()
            .all(|e| e.target_header != PathBuf::from(CANDIDATE)),
        "no proposal may target the file that defines its symbol: {proposals:?}"
    );
}

#[test]
fn candidate_whose_only_batches_self_target_produces_no_plan() {
    let always_self = |path: &Path, _symbol: &str| -> PathBuf { path.to_path_buf() };
    let plans = collateral_candidate().plan_with(&always_self);
    assert!(
        plans.is_empty(),
        "a candidate whose every proposal is a no-op self-move yields no plan"
    );
}

#[test]
fn extracted_header_style_targets_never_trip_the_filter() {
    let cpp_style = |path: &Path, symbol: &str| -> PathBuf {
        path.with_file_name(format!("{symbol}_extracted.h"))
    };
    let plans = collateral_candidate().plan_with(&cpp_style);

    assert_eq!(plans.len(), 1);
    assert_eq!(
        plans[0]
            .proposed_extractions
            .iter()
            .map(|e| e.target_header.clone())
            .collect::<Vec<_>>(),
        vec![
            PathBuf::from("pkg/Alpha_extracted.h"),
            PathBuf::from("pkg/Legacy_extracted.h"),
        ],
        "C++ '{{name}}_extracted.h' targets are never equal to the candidate — filter is a no-op"
    );
}
