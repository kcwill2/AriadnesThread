use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollateralOpportunity {
    pub header_name: String,
    pub collateral_symbols: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizationReport {
    pub total_files: usize,
    pub total_tus: usize,
    pub total_symbols: usize,
    pub total_semantic_edges: usize,
    pub collateral_opportunities: Vec<CollateralOpportunity>,
    pub safe_plans_available: usize,
    pub affected_consumer_files: usize,
    pub validation_passed_tus: usize,
    pub validation_total_tus: usize,
    pub build_passed: bool,
    pub frontend_cpu_before: Duration,
    pub frontend_cpu_after: Duration,
    pub committed_transformations: usize,
    pub residual_collateral_load: usize,
}

impl OptimizationReport {
    pub fn format_m14_output(&self) -> String {
        let mut opportunities_str = String::new();
        for (i, opp) in self.collateral_opportunities.iter().enumerate() {
            opportunities_str.push_str(&format!(
                "    {}. {}\n       {} collateral symbols\n\n",
                i + 1,
                opp.header_name,
                opp.collateral_symbols
            ));
        }
        let opp_trimmed = opportunities_str.trim_end();

        format!(
            r#"Repository:
    {} files
    {} translation units

Analysis:
    {} symbols
    {} semantic edges

Largest collateral-load opportunities:

{}

Plan:
    {} safe transformations available

Preview:
    {} consumer files affected

Validation:
    {} / {} affected TUs passed

Build:
    {}

Measurement:
    Front-end CPU: {:.1}s → {:.1}s

Committed:
    {} transformations

Residual collateral load:
    {} symbols"#,
            self.total_files,
            self.total_tus,
            self.total_symbols,
            self.total_semantic_edges,
            opp_trimmed,
            self.safe_plans_available,
            self.affected_consumer_files,
            self.validation_passed_tus,
            self.validation_total_tus,
            if self.build_passed { "PASS" } else { "FAIL" },
            self.frontend_cpu_before.as_secs_f64(),
            self.frontend_cpu_after.as_secs_f64(),
            self.committed_transformations,
            self.residual_collateral_load,
        )
    }
}

use anyhow::Result;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use ariadne_build::{BuildDiscoveryResult, BuildMeasurer};
use ariadne_cluster::plan::{OptimizationPlan, RiskLevel};
use ariadne_cluster::planner::OptimizationPlanner;
use ariadne_graph::{IncludeClosure, IncludeGraph, LoadAnalyzer, ReducedSemanticModel};
use ariadne_lang::CyclePolicy;
use ariadne_lang_cpp::{cpp_aggregation_filter, cpp_target_path_for};
use ariadne_pipeline::Pipeline;
use ariadne_repo::{Repository, RepositoryDiscoveryResult};
use ariadne_transform::{ExtractionResult, GeneratedUnit, IncludeRewriter, IsolationEngine};
use ariadne_validate::{TierValidator, TransactionApplier};

pub struct EndToEndOptimizer {
    pub repo_path: PathBuf,
}

impl EndToEndOptimizer {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
        }
    }

    pub fn optimize_repository(&self) -> Result<OptimizationReport> {
        // M18.3: production path routes the index stage through the plugin
        // registry (byte-identical for C++); the worker pool backend is
        // registry-dispatching.
        let registry = std::sync::Arc::new(crate::registry::default_registry());
        let mut pipeline = Pipeline::with_registry(&self.repo_path, Some(4), registry)?;
        let repo = pipeline.repo.clone();
        let repo_discovery = pipeline.discovery.clone();
        let build_discovery = pipeline.build.clone();

        let total_files = repo_discovery.files.len();
        let total_tus = build_discovery.translation_units.len();

        // 1. Initial measurement (before)
        let measurer = BuildMeasurer::new(repo.root.clone());
        let before_measurement = measurer.measure_all(&build_discovery.translation_units)?;

        // 2. Include analysis (fingerprint-checked, loaded when fresh)
        let (graph, closures) = pipeline.get_or_build_include_graph()?;

        // 3. Semantic analysis & reduction (fingerprint-checked, TU-level reuse)
        let model = pipeline.get_or_build_reduced()?;

        let total_symbols = model.symbols.len();
        let total_semantic_edges = model.edges.len();

        // 4. Load analysis
        let collateral_opportunities =
            analyze_collateral_opportunities(&graph, &model, closures, &repo_discovery);

        // 5. Optimization planning
        let safe_plans = plan_safe_extractions(&graph, &model, &repo_discovery, &build_discovery);
        let safe_plans_available = safe_plans.len();

        // 6. Preview, apply, and validate safe plans
        let mut affected_consumer_files_set = HashSet::new();
        let mut committed_transformations = 0;
        let mut validation_passed_tus = 0;
        let mut validation_total_tus = 0;
        let mut build_passed = true;

        for plan in &safe_plans {
            let outcome = apply_safe_plan(&repo, &repo_discovery, &model, &build_discovery, plan)?;
            affected_consumer_files_set.extend(outcome.affected_source_paths);
            validation_total_tus += outcome.consumer_tus;
            if outcome.validation_passed {
                committed_transformations += 1;
                validation_passed_tus += outcome.consumer_tus;
            } else {
                build_passed = false;
            }
        }

        // 7. Post-measurement (after)
        let after_measurement = measurer.measure_all(&build_discovery.translation_units)?;

        let residual_collateral_load = residual_collateral_load(
            &collateral_opportunities,
            &safe_plans,
            committed_transformations,
        );

        let report = OptimizationReport {
            total_files,
            total_tus,
            total_symbols,
            total_semantic_edges,
            collateral_opportunities,
            safe_plans_available,
            affected_consumer_files: affected_consumer_files_set.len(),
            validation_passed_tus,
            validation_total_tus,
            build_passed,
            frontend_cpu_before: before_measurement.total_frontend_cpu_time,
            frontend_cpu_after: after_measurement.total_frontend_cpu_time,
            committed_transformations,
            residual_collateral_load,
        };

        save_optimization_report(&repo.root, &report)?;

        println!("{}", pipeline.stats.format_summary());

        Ok(report)
    }
}

struct SafePlanOutcome {
    consumer_tus: usize,
    affected_source_paths: HashSet<PathBuf>,
    validation_passed: bool,
}

fn analyze_collateral_opportunities(
    graph: &IncludeGraph,
    model: &ReducedSemanticModel,
    closures: HashMap<ariadne_repo::FileId, IncludeClosure>,
    repo_discovery: &RepositoryDiscoveryResult,
) -> Vec<CollateralOpportunity> {
    let load_analyzer = LoadAnalyzer::with_closures(graph, model, closures);
    let mut header_collateral_map: HashMap<String, usize> = HashMap::new();

    for sym in model.symbols.values() {
        // Sibling convention (commands/analyze.rs sweeps): symbols whose load
        // metric cannot be resolved (e.g. no defining file) are skipped in
        // aggregate scans instead of aborting the whole report.
        let Ok(lm) = load_analyzer.analyze_symbol_load(&sym.name) else {
            continue;
        };
        for src in &lm.collateral_sources {
            if let Some(f) = repo_discovery
                .files
                .iter()
                .find(|rf| &rf.id == &src.file_id)
            {
                let name = f
                    .path
                    .file_name()
                    .and_then(|s: &std::ffi::OsStr| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                *header_collateral_map.entry(name).or_insert(0) += src.symbol_count;
            }
        }
    }

    let mut collateral_opportunities: Vec<CollateralOpportunity> = header_collateral_map
        .into_iter()
        .filter(|(header_name, _)| cpp_aggregation_filter(std::path::Path::new(header_name)))
        .map(|(header_name, collateral_symbols)| CollateralOpportunity {
            header_name,
            collateral_symbols,
        })
        .collect();
    collateral_opportunities.sort_by(|a, b| {
        b.collateral_symbols
            .cmp(&a.collateral_symbols)
            .then_with(|| a.header_name.cmp(&b.header_name))
    });
    collateral_opportunities
}

fn plan_safe_extractions(
    graph: &IncludeGraph,
    model: &ReducedSemanticModel,
    repo_discovery: &RepositoryDiscoveryResult,
    build_discovery: &BuildDiscoveryResult,
) -> Vec<OptimizationPlan> {
    let tu_file_ids: Vec<_> = build_discovery
        .translation_units
        .iter()
        .map(|tu| tu.file_id)
        .collect();
    let planner = OptimizationPlanner::new(
        graph,
        model,
        &repo_discovery.files,
        &tu_file_ids,
        &cpp_aggregation_filter,
        &cpp_target_path_for,
        None,
        CyclePolicy::Reject,
    );
    planner
        .plan()
        .into_iter()
        .filter(|p| matches!(p.risk, RiskLevel::Conservative))
        .collect()
}

fn relocate_extraction_to_generated(
    extraction: &ExtractionResult,
) -> (BTreeMap<String, PathBuf>, Vec<GeneratedUnit>) {
    let mut symbol_name_to_header = BTreeMap::new();
    let mut generated_units = Vec::new();
    for unit in &extraction.headers {
        let upath = PathBuf::from("generated").join(&unit.path);
        let mut new_unit = unit.clone();
        new_unit.path = upath.clone();
        new_unit.required_deps = unit
            .required_deps
            .iter()
            .map(|p| PathBuf::from("generated").join(p))
            .collect();
        generated_units.push(new_unit);
        for sym in &unit.symbols {
            symbol_name_to_header.insert(sym.name.clone(), upath.clone());
        }
    }
    (symbol_name_to_header, generated_units)
}

fn find_including_translation_units(
    registry: &ariadne_lang::LanguageRegistry,
    repo_discovery: &RepositoryDiscoveryResult,
    build_discovery: &BuildDiscoveryResult,
    plan: &OptimizationPlan,
) -> Vec<(PathBuf, Vec<PathBuf>, Vec<String>)> {
    // D-M23-7 iii (T7 S2): language-routed consumer discovery — the C++
    // plugin reproduces this function's former raw substring scan exactly
    // (set-equal consumers, I1); see commands/transform.rs
    // `plan_target_consumers` for the shared rule.
    let target_include = plan
        .target_file
        .file_name()
        .and_then(|n: &std::ffi::OsStr| n.to_str())
        .unwrap_or("");
    let all_symbol_names: Vec<String> = plan
        .proposed_extractions
        .iter()
        .map(|e| e.symbol_name.clone())
        .collect();

    crate::commands::transform::plan_target_consumers(
        registry,
        &crate::registry::catalog_languages(repo_discovery),
        &build_discovery.translation_units,
        &repo_discovery.root,
        target_include,
        &plan.target_file,
        &all_symbol_names,
    )
}

fn apply_safe_plan(
    repo: &Repository,
    repo_discovery: &RepositoryDiscoveryResult,
    model: &ReducedSemanticModel,
    build_discovery: &BuildDiscoveryResult,
    plan: &OptimizationPlan,
) -> Result<SafePlanOutcome> {
    // Header extraction (M23 T1/T7, D-M23-3 + D-M23-7 ii: source-file text
    // from the catalog, verbatim bodies located by the PLAN-LANGUAGE
    // extraction backend; unreadable files keep the engine's placeholder
    // fallback per I4).
    let extractor = IsolationEngine::new(model);
    let source_reader = crate::commands::transform::repository_source_reader(repo)?;
    let plugin_registry = crate::registry::default_registry();
    let text_extractor = crate::registry::plan_backend(
        &plugin_registry,
        &plan.target_file,
        |caps| caps.extraction,
        "extraction",
    )?
    .extractor();
    let extraction = extractor.extract(plan, &source_reader, text_extractor);

    let (symbol_name_to_header, generated_headers) = relocate_extraction_to_generated(&extraction);

    let consumers =
        find_including_translation_units(&plugin_registry, repo_discovery, build_discovery, plan);

    let rewriter = IncludeRewriter::new(symbol_name_to_header, generated_headers);
    let migration = rewriter.build_preview(plan.id.to_string(), &consumers);

    // M23 T7 (D-M23-7 ii): directive text (guard block + include replacement)
    // is owned by the plan-language rewriting backend; validate orchestrates
    // the rest.
    let text_rewriter = crate::registry::plan_backend(
        &plugin_registry,
        &plan.target_file,
        |caps| caps.rewriting,
        "rewriting",
    )?
    .rewriter();
    let (snapshot, new_paths) = TransactionApplier::apply_with(
        &repo.root,
        &extraction,
        &migration,
        text_rewriter,
        text_extractor,
    )?;

    // M18.10: the tier-3 skip decision uses the injected plugin validator's
    // toolchain status; M23 T7 (D-M23-7 ii) routes it to the plan's language.
    let compiler_validator = crate::registry::plan_backend(
        &plugin_registry,
        &plan.target_file,
        |caps| caps.validation,
        "validation",
    )?
    .validator();
    let t1 = TierValidator::validate_structural(&extraction);
    let header_output_dir = repo.root.join("generated");
    let t2 = TierValidator::validate_generated_headers(&extraction, &header_output_dir);
    let t3 = TierValidator::validate_affected_tus(
        &migration,
        &header_output_dir,
        Some(compiler_validator),
    );

    let validation_passed = !t1.status.is_fail() && !t2.status.is_fail() && !t3.status.is_fail();

    if !validation_passed {
        TransactionApplier::rollback(&snapshot, &new_paths)?;
    }

    Ok(SafePlanOutcome {
        consumer_tus: consumers.len(),
        affected_source_paths: consumers
            .iter()
            .map(|(source_path, _, _)| source_path.clone())
            .collect(),
        validation_passed,
    })
}

fn residual_collateral_load(
    collateral_opportunities: &[CollateralOpportunity],
    safe_plans: &[OptimizationPlan],
    committed_transformations: usize,
) -> usize {
    let initial_collateral_sum: usize = collateral_opportunities
        .iter()
        .map(|o| o.collateral_symbols)
        .sum();
    let reduced_collateral: usize = safe_plans
        .iter()
        .take(committed_transformations)
        .map(|p| {
            p.collateral_load_before
                .saturating_sub(p.collateral_load_after)
        })
        .sum();
    initial_collateral_sum.saturating_sub(reduced_collateral)
}

fn save_optimization_report(repo_root: &Path, report: &OptimizationReport) -> Result<()> {
    let opt_dir = repo_root.join("artifacts").join("optimize");
    fs::create_dir_all(&opt_dir)?;
    if let Ok(json) = serde_json::to_string_pretty(report) {
        let _ = fs::write(opt_dir.join("report.json"), json);
    }
    Ok(())
}

pub fn run_end_to_end_optimize(repo_path: &Path) -> Result<OptimizationReport> {
    let optimizer = EndToEndOptimizer::new(repo_path);
    let report = optimizer.optimize_repository()?;
    println!("{}", report.format_m14_output());
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_m14_output() {
        let report = OptimizationReport {
            total_files: 14821,
            total_tus: 3412,
            total_symbols: 182441,
            total_semantic_edges: 4100000,
            collateral_opportunities: vec![
                CollateralOpportunity {
                    header_name: "graphics.h".into(),
                    collateral_symbols: 4382,
                },
                CollateralOpportunity {
                    header_name: "networking.h".into(),
                    collateral_symbols: 2941,
                },
                CollateralOpportunity {
                    header_name: "serialization.h".into(),
                    collateral_symbols: 1882,
                },
            ],
            safe_plans_available: 7,
            affected_consumer_files: 183,
            validation_passed_tus: 183,
            validation_total_tus: 183,
            build_passed: true,
            frontend_cpu_before: Duration::from_secs_f64(91.2),
            frontend_cpu_after: Duration::from_secs_f64(54.7),
            committed_transformations: 7,
            residual_collateral_load: 8241,
        };

        let formatted = report.format_m14_output();
        assert!(formatted.contains("Repository:\n    14821 files\n    3412 translation units"));
        assert!(formatted.contains("Analysis:\n    182441 symbols\n    4100000 semantic edges"));
        assert!(formatted.contains("Largest collateral-load opportunities:"));
        assert!(formatted.contains("1. graphics.h\n       4382 collateral symbols"));
        assert!(formatted.contains("2. networking.h\n       2941 collateral symbols"));
        assert!(formatted.contains("3. serialization.h\n       1882 collateral symbols"));
        assert!(formatted.contains("Plan:\n    7 safe transformations available"));
        assert!(formatted.contains("Preview:\n    183 consumer files affected"));
        assert!(formatted.contains("Validation:\n    183 / 183 affected TUs passed"));
        assert!(formatted.contains("Build:\n    PASS"));
        assert!(formatted.contains("Measurement:\n    Front-end CPU: 91.2s → 54.7s"));
        assert!(formatted.contains("Committed:\n    7 transformations"));
        assert!(formatted.contains("Residual collateral load:\n    8241 symbols"));
    }

    #[test]
    fn test_collateral_opportunities_skip_symbols_without_defining_file() {
        // Regression lock for 161f028: analyze_symbol_load legitimately errors
        // for symbols with defining_file: None; the scan must skip them and
        // keep aggregating the rest instead of panicking on unwrap.
        use ariadne_graph::{IncludeEdge, IncludeKind, ReducedSymbolRecord, SourceLocation};
        use ariadne_index::types::{SymbolId, SymbolKind};
        use ariadne_repo::{
            FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy,
            RepositoryDiscoveryResult,
        };

        let mut graph = IncludeGraph::new();
        graph.register_file(FileId(1), PathBuf::from("Thing.h"));
        graph.register_file(FileId(2), PathBuf::from("graphics.h"));
        graph.register_file(FileId(3), PathBuf::from("HugeThirdParty.h"));
        let link = |source: FileId, destination: FileId, raw_path: &str| IncludeEdge {
            source,
            destination: Some(destination),
            context_name: String::new(),
            source_location: SourceLocation::new(1, 1),
            kind: IncludeKind::Local,
            raw_path: raw_path.to_string(),
            resolved_path: Some(PathBuf::from(raw_path)),
            language: ariadne_repo::SourceLanguage::Cpp,
        };
        graph.add_edge(link(FileId(1), FileId(2), "graphics.h"));
        graph.add_edge(link(FileId(2), FileId(3), "HugeThirdParty.h"));

        let mut model = ReducedSemanticModel {
            repo_root: PathBuf::from("/"),
            symbols: BTreeMap::new(),
            edges: vec![],
            tu_usage: BTreeMap::new(),
            file_symbols: BTreeMap::new(),

            extends_edges: Vec::new(),
            sealed_symbols: Vec::new(),
        };
        let add_symbol = |model: &mut ReducedSemanticModel,
                          name: &str,
                          kind: SymbolKind,
                          defining_file: Option<FileId>| {
            let id = SymbolId::new(name);
            model.symbols.insert(
                id.clone(),
                ReducedSymbolRecord {
                    id: id.clone(),
                    name: name.to_string(),
                    kind,
                    defining_file,
                    declarations: vec![],
                    definition: None,
                    declaring_files: vec![],
                },
            );
            if let Some(file) = defining_file {
                model.file_symbols.entry(file).or_default().push(id);
            }
        };
        // Ghost sorts first in the BTreeMap, so the loop reaches the Err arm
        // before any well-formed symbol is aggregated.
        add_symbol(&mut model, "Ghost", SymbolKind::Function, None);
        add_symbol(&mut model, "HugeThing", SymbolKind::Class, Some(FileId(3)));
        add_symbol(&mut model, "Thing", SymbolKind::Class, Some(FileId(1)));

        let file_record = |id: FileId, name: &str| FileRecord {
            id,
            path: PathBuf::from(name),
            canonical_path: PathBuf::from(name),
            kind: FileKind::Header,
            size: 1,
            content_hash: String::new(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
            language: ariadne_repo::SourceLanguage::Cpp,
        };
        let repo_discovery = RepositoryDiscoveryResult {
            root: PathBuf::from("/"),
            files: vec![
                file_record(FileId(1), "Thing.h"),
                file_record(FileId(2), "graphics.h"),
                file_record(FileId(3), "HugeThirdParty.h"),
            ],
            ownership_summary: HashMap::new(),
            policy_summary: HashMap::new(),
        };

        let opportunities = analyze_collateral_opportunities(
            &graph,
            &model,
            graph.compute_closures(),
            &repo_discovery,
        );

        assert_eq!(
            opportunities,
            vec![CollateralOpportunity {
                header_name: "HugeThirdParty.h".to_string(),
                collateral_symbols: 1,
            }],
            "Ghost (defining_file: None) must contribute nothing while Thing's collateral aggregation is unaffected"
        );
    }
}
