//! M18.6/M18.7 planner-injection tests: the engine must honour an injected
//! aggregation-file filter, an injected target-path template, and injected
//! `ClusteringConstraints` (colocated batches, frozen symbols), with
//! `CyclePolicy` carried through. Fixture names use neutral `.agg`/`_unit`
//! placeholders so the suite itself locks the engine to zero language names.

use ariadne_cluster::plan::OptimizationPlan;
use ariadne_cluster::planner::OptimizationPlanner;
use ariadne_graph::reduced::{
    ReducedSemanticEdge, ReducedSemanticModel, ReducedSymbolRecord,
};
use ariadne_graph::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_graph::IncludeGraph;
use ariadne_index::types::{DependencyKind, SymbolId, SymbolKind};
use ariadne_lang::{ClusteringConstraints, CyclePolicy, SymbolRecord};
use ariadne_repo::{
    FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage,
};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

/// Minimal fixture builder for injected-hook planner cases.
#[derive(Default)]
struct Fixture {
    graph: IncludeGraph,
    symbols: BTreeMap<SymbolId, ReducedSymbolRecord>,
    edges: Vec<ReducedSemanticEdge>,
    file_symbols: BTreeMap<FileId, Vec<SymbolId>>,
    records: Vec<FileRecord>,
    tu_file_ids: Vec<FileId>,
}

impl Fixture {
    fn file(
        mut self,
        id: FileId,
        name: &str,
        ownership: FileOwnership,
        policy: OptimizationPolicy,
    ) -> Self {
        self.graph.register_file(id, PathBuf::from(name));
        self.records.push(FileRecord {
            id,
            path: PathBuf::from(name),
            canonical_path: PathBuf::from(name),
            kind: FileKind::Header,
            size: 0,
            content_hash: String::new(),
            ownership,
            optimization_policy: policy,
            language: SourceLanguage::Other,
        });
        self
    }

    fn candidate(self, id: FileId, name: &str) -> Self {
        self.file(id, name, FileOwnership::Repository, OptimizationPolicy::Optimizable)
    }

    fn frozen(self, id: FileId, name: &str) -> Self {
        self.file(id, name, FileOwnership::ThirdParty, OptimizationPolicy::Frozen)
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
            language: SourceLanguage::Other,
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

    fn depends(mut self, src: &str, dst: &str) -> Self {
        self.edges.push(ReducedSemanticEdge {
            src_symbol: SymbolId::new(src),
            dst_symbol: SymbolId::new(dst),
            kind: DependencyKind::Complete,
        });
        self
    }

    fn plan_with(
        &self,
        aggregation_filter: &dyn Fn(&Path) -> bool,
        target_path_for: &dyn Fn(&Path, &str) -> PathBuf,
        constraints: Option<&dyn ClusteringConstraints>,
        cycle_policy: CyclePolicy,
    ) -> Vec<OptimizationPlan> {
        let model = ReducedSemanticModel {
            repo_root: PathBuf::from("/"),
            symbols: self.symbols.clone(),
            edges: self.edges.clone(),
            tu_usage: BTreeMap::new(),
            file_symbols: self.file_symbols.clone(),
        };
        OptimizationPlanner::new(
            &self.graph,
            &model,
            &self.records,
            &self.tu_file_ids,
            aggregation_filter,
            target_path_for,
            constraints,
            cycle_policy,
        )
        .plan()
    }
}

struct FakeConstraints {
    groups: Vec<Vec<SymbolId>>,
    frozen: HashSet<SymbolId>,
}

impl FakeConstraints {
    fn colocated(group_names: Vec<Vec<&str>>) -> Self {
        Self {
            groups: group_names
                .into_iter()
                .map(|group| group.into_iter().map(SymbolId::new).collect())
                .collect(),
            frozen: HashSet::new(),
        }
    }
}

impl ClusteringConstraints for FakeConstraints {
    fn colocated_groups(&self, _symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
        self.groups.clone()
    }

    fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> HashSet<SymbolId> {
        self.frozen.clone()
    }
}

fn accept_every(path: &Path) -> bool {
    path.file_name().is_some()
}

fn unit_template(path: &Path, symbol: &str) -> PathBuf {
    path.with_file_name(format!("{symbol}_unit"))
}

/// One optimizable candidate with collateral load from a frozen source.
fn collateral_pair(suffix: &str) -> Fixture {
    Fixture::default()
        .candidate(FileId(1), &format!("pkg.{suffix}"))
        .frozen(FileId(2), &format!("col.{suffix}"))
        .includes(FileId(1), FileId(2))
        .defines(FileId(1), "Alpha")
        .defines(FileId(1), "Delta")
        .defines(FileId(2), "Big")
}

#[test]
fn injected_aggregation_filter_limits_candidate_files() {
    let fixture = Fixture::default()
        .candidate(FileId(1), "alpha.agg")
        .candidate(FileId(2), "beta.agg")
        .frozen(FileId(3), "col.agg")
        .includes(FileId(1), FileId(3))
        .includes(FileId(2), FileId(3))
        .defines(FileId(1), "One")
        .defines(FileId(2), "Two")
        .defines(FileId(3), "Big");

    let unfiltered = fixture.plan_with(&accept_every, &unit_template, None, CyclePolicy::Reject);
    assert_eq!(
        unfiltered.len(),
        2,
        "both candidates plan when every file passes the filter"
    );

    let only_alpha = |path: &Path| path == Path::new("alpha.agg");
    let filtered = fixture.plan_with(&only_alpha, &unit_template, None, CyclePolicy::Reject);
    assert_eq!(
        filtered.len(),
        1,
        "only files passing the injected filter become candidates"
    );
    assert_eq!(filtered[0].target_file, PathBuf::from("alpha.agg"));
}

#[test]
fn injected_target_path_for_replaces_proposal_target_paths() {
    let marker = |_path: &Path, _symbol: &str| PathBuf::from("generated/marker.unit");
    let plans = collateral_pair("agg").plan_with(&accept_every, &marker, None, CyclePolicy::Reject);

    assert_eq!(plans.len(), 1);
    let targets: Vec<PathBuf> = plans[0]
        .proposed_extractions
        .iter()
        .map(|e| e.target_header.clone())
        .collect();
    assert_eq!(
        targets,
        vec![
            PathBuf::from("generated/marker.unit"),
            PathBuf::from("generated/marker.unit"),
        ],
        "every proposal carries the injected target path"
    );
}

#[test]
fn colocated_group_merges_separate_sccs_into_one_proposal_batch() {
    let fixture = collateral_pair("agg");

    let unmerged = fixture.plan_with(&accept_every, &unit_template, None, CyclePolicy::Reject);
    assert_eq!(unmerged.len(), 1);
    let unmerged_targets: Vec<PathBuf> = unmerged[0]
        .proposed_extractions
        .iter()
        .map(|e| e.target_header.clone())
        .collect();
    assert_eq!(
        unmerged_targets,
        vec![PathBuf::from("Alpha_unit"), PathBuf::from("Delta_unit")],
        "without constraints each symbol targets its own path"
    );

    let colocated = FakeConstraints::colocated(vec![vec!["Alpha", "Delta"]]);
    let merged = fixture.plan_with(
        &accept_every,
        &unit_template,
        Some(&colocated),
        CyclePolicy::Reject,
    );

    assert_eq!(merged.len(), 1);
    let names: Vec<String> = merged[0]
        .proposed_extractions
        .iter()
        .map(|e| e.symbol_name.clone())
        .collect();
    assert_eq!(names, vec!["Alpha".to_string(), "Delta".to_string()]);
    let targets: Vec<PathBuf> = merged[0]
        .proposed_extractions
        .iter()
        .map(|e| e.target_header.clone())
        .collect();
    assert_eq!(
        targets,
        vec![PathBuf::from("Alpha_unit"), PathBuf::from("Alpha_unit")],
        "a colocated batch is co-proposed to ONE shared target path"
    );
}

#[test]
fn frozen_symbols_are_skipped_from_proposals() {
    let fixture = collateral_pair("agg");
    let fake = FakeConstraints {
        groups: vec![],
        frozen: [SymbolId::new("Delta")].into_iter().collect(),
    };

    let plans =
        fixture.plan_with(&accept_every, &unit_template, Some(&fake), CyclePolicy::Reject);

    assert_eq!(plans.len(), 1);
    let names: Vec<String> = plans[0]
        .proposed_extractions
        .iter()
        .map(|e| e.symbol_name.clone())
        .collect();
    assert_eq!(names, vec!["Alpha".to_string()]);
}

#[test]
fn colocated_batch_with_a_frozen_member_is_skipped_entirely() {
    let fixture = collateral_pair("agg");
    let mut fake = FakeConstraints::colocated(vec![vec!["Alpha", "Delta"]]);
    fake.frozen.insert(SymbolId::new("Delta"));

    let plans =
        fixture.plan_with(&accept_every, &unit_template, Some(&fake), CyclePolicy::Reject);

    assert!(
        plans.is_empty(),
        "a co-located batch containing a frozen symbol cannot be extracted together"
    );
}

#[test]
fn cycle_policy_variants_share_the_scc_coproposal_path_for_now() {
    // M18.7 documents Tolerate/TolerateWithinPackage as reusing the SCC
    // co-proposal path (multi-member batches stay rejected by the checker)
    // until M20 adds package-scoped informational reporting.
    let fixture = collateral_pair("agg").depends("Alpha", "Delta").depends("Delta", "Alpha");
    for policy in [
        CyclePolicy::Reject,
        CyclePolicy::Tolerate,
        CyclePolicy::TolerateWithinPackage,
    ] {
        let plans =
            fixture.plan_with(&accept_every, &unit_template, None, policy);
        assert!(
            plans.is_empty(),
            "cycle members stay co-proposed-and-rejected under {policy:?}"
        );
    }
}
