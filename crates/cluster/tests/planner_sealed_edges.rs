//! M23 T7 S4 (D-M23-8): the planner must feed each candidate file's typed
//! extends channel and the `sealed` modifiers into
//! `ClusteringConstraints::colocated_groups_with_edges`, so a sealed trait
//! and its direct sub-types land in ONE batch while an unsealed sibling
//! stays separable. Neutral fixture (`.agg`/`_unit`, zero language names).

use ariadne_cluster::plan::OptimizationPlan;
use ariadne_cluster::planner::OptimizationPlanner;
use ariadne_graph::reduced::{ReducedSemanticModel, ReducedSymbolRecord};
use ariadne_graph::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_graph::IncludeGraph;
use ariadne_index::types::{SymbolId, SymbolKind};
use ariadne_lang::{ClusteringConstraints, CyclePolicy, SymbolRecord};
use ariadne_repo::{
    FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Mirrors the sealed rule (modifier + extends edge) and records what the
/// planner actually fed it, so the wiring itself - not just the rule - is
/// under lock.
struct SealedFeedConstraints {
    seen_sealed_ids: Mutex<Vec<String>>,
    seen_edges: Mutex<Vec<(String, String)>>,
}

impl SealedFeedConstraints {
    fn new() -> Self {
        Self {
            seen_sealed_ids: Mutex::new(Vec::new()),
            seen_edges: Mutex::new(Vec::new()),
        }
    }
}

impl ClusteringConstraints for SealedFeedConstraints {
    fn colocated_groups(&self, _symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
        Vec::new()
    }

    fn colocated_groups_with_edges(
        &self,
        records: &[SymbolRecord],
        edges: &[(SymbolId, SymbolId)],
    ) -> Vec<Vec<SymbolId>> {
        let sealed: Vec<&SymbolRecord> = records
            .iter()
            .filter(|record| record.modifiers.iter().any(|m| m == "sealed"))
            .collect();
        *self.seen_sealed_ids.lock().unwrap() =
            sealed.iter().map(|r| r.id.0.clone()).collect();
        *self.seen_edges.lock().unwrap() = edges
            .iter()
            .map(|(sub, sup)| (sub.0.clone(), sup.0.clone()))
            .collect();
        let mut groups = Vec::new();
        for sup in sealed {
            let members: Vec<SymbolId> = edges
                .iter()
                .filter(|(_, parent)| *parent == sup.id)
                .map(|(child, _)| child.clone())
                .collect();
            if !members.is_empty() {
                let mut group = vec![sup.id.clone()];
                group.extend(members);
                groups.push(group);
            }
        }
        groups
    }

    fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> HashSet<SymbolId> {
        HashSet::new()
    }
}

fn accept_every(path: &Path) -> bool {
    path.file_name().is_some()
}

fn unit_template(path: &Path, symbol: &str) -> PathBuf {
    path.with_file_name(format!("{symbol}_unit"))
}

fn file_record(id: FileId, name: &str, ownership: FileOwnership, policy: OptimizationPolicy) -> FileRecord {
    FileRecord {
        id,
        path: PathBuf::from(name),
        canonical_path: PathBuf::from(name),
        kind: FileKind::Header,
        size: 0,
        content_hash: String::new(),
        ownership,
        optimization_policy: policy,
        language: SourceLanguage::Other,
    }
}

/// `pkg.agg` (optimizable candidate, sealed Shape + Circle + Square + Base +
/// Helper) includes `col.agg` (frozen collateral source), mirroring
/// `planner_injection.rs::collateral_pair`.
fn sealed_fixture(model: &mut ReducedSemanticModel) -> (IncludeGraph, Vec<FileRecord>, Vec<FileId>) {
    let mut graph = IncludeGraph::default();
    graph.register_file(FileId(1), PathBuf::from("pkg.agg"));
    graph.register_file(FileId(2), PathBuf::from("col.agg"));
    graph.add_edge(IncludeEdge {
        source: FileId(1),
        destination: Some(FileId(2)),
        context_name: "Default".into(),
        source_location: SourceLocation::new(1, 1),
        kind: IncludeKind::Local,
        raw_path: "col.agg".to_string(),
        resolved_path: Some(PathBuf::from("col.agg")),
        language: SourceLanguage::Other,
    });
    let records = vec![
        file_record(
            FileId(1),
            "pkg.agg",
            FileOwnership::Repository,
            OptimizationPolicy::Optimizable,
        ),
        file_record(
            FileId(2),
            "col.agg",
            FileOwnership::ThirdParty,
            OptimizationPolicy::Frozen,
        ),
    ];
    let mut define = |model: &mut ReducedSemanticModel, file: FileId, name: &str| {
        let sym = SymbolId::new(name);
        model.symbols.insert(
            sym.clone(),
            ReducedSymbolRecord {
                id: sym.clone(),
                name: name.rsplit('.').next().unwrap_or(name).to_string(),
                kind: SymbolKind::Class,
                defining_file: Some(file),
                declarations: vec![],
                definition: None,
                declaring_files: vec![],
            },
        );
        model
            .file_symbols
            .entry(file)
            .or_default()
            .push(sym);
    };
    define(model, FileId(1), "p.Shape");
    define(model, FileId(1), "p.Circle");
    define(model, FileId(1), "p.Square");
    define(model, FileId(1), "p.Base");
    define(model, FileId(1), "p.Helper");
    define(model, FileId(2), "p.Big");
    (graph, records, vec![FileId(1), FileId(2)])
}

/// The full snapshot: candidate file with the sealed trio + unsealed pair,
/// frozen collateral source, typed channel, and sealed modifier feed.
fn sealed_env() -> (ReducedSemanticModel, IncludeGraph, Vec<FileRecord>, Vec<FileId>) {
    let mut model = ReducedSemanticModel::default();
    let (graph, records, tu_file_ids) = sealed_fixture(&mut model);
    model.extends_edges = vec![
        (SymbolId::new("p.Circle"), SymbolId::new("p.Shape")),
        (SymbolId::new("p.Square"), SymbolId::new("p.Shape")),
        (SymbolId::new("p.Helper"), SymbolId::new("p.Base")),
    ];
    model.sealed_symbols = vec![SymbolId::new("p.Shape")];
    (model, graph, records, tu_file_ids)
}

fn plan_with(env: &(ReducedSemanticModel, IncludeGraph, Vec<FileRecord>, Vec<FileId>), constraints: &dyn ClusteringConstraints) -> Vec<OptimizationPlan> {
    let (model, graph, records, tu_file_ids) = env;
    OptimizationPlanner::new(
        graph,
        model,
        records,
        tu_file_ids,
        &accept_every,
        &unit_template,
        Some(constraints),
        CyclePolicy::Reject,
    )
    .plan()
}

fn target_of(plan: &OptimizationPlan, symbol_id: &str) -> PathBuf {
    plan.proposed_extractions
        .iter()
        .find(|p| p.symbol_id.0 == symbol_id)
        .unwrap_or_else(|| panic!("{symbol_id} is proposed"))
        .target_header
        .clone()
}

#[test]
fn sealed_trait_and_subtypes_land_in_one_batch_and_unsealed_sibling_stays_separable() {
    let env = sealed_env();
    let constraints = SealedFeedConstraints::new();
    let plans = plan_with(&env, &constraints);
    assert_eq!(plans.len(), 1, "pkg.agg is the only aggregation candidate");
    let plan = &plans[0];
    let trio_target = target_of(plan, "p.Shape");
    assert_eq!(
        trio_target,
        target_of(plan, "p.Circle"),
        "sealed Shape and its subtype Circle share one batch"
    );
    assert_eq!(
        trio_target,
        target_of(plan, "p.Square"),
        "sealed Shape and its subtype Square share one batch"
    );
    assert_ne!(
        trio_target,
        target_of(plan, "p.Helper"),
        "the unsealed Helper keeps its own target - it stays separable"
    );
}

#[test]
fn planner_feeds_sealed_modifiers_and_file_scoped_extends_edges() {
    let env = sealed_env();
    let constraints = SealedFeedConstraints::new();
    let plans = plan_with(&env, &constraints);
    assert_eq!(plans.len(), 1, "candidate planned");
    assert_eq!(
        *constraints.seen_sealed_ids.lock().unwrap(),
        vec!["p.Shape".to_string()],
        "D-M23-8: model.sealed_symbols flows into the constraint records as modifiers"
    );
    let mut fed = constraints.seen_edges.lock().unwrap().clone();
    fed.sort();
    assert_eq!(
        fed,
        vec![
            ("p.Circle".to_string(), "p.Shape".to_string()),
            ("p.Helper".to_string(), "p.Base".to_string()),
            ("p.Square".to_string(), "p.Shape".to_string()),
        ],
        "the candidate file's extends channel reaches the hook (both endpoints in-file)"
    );
}
