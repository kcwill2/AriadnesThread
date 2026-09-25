use std::time::Duration;
use serde::{Deserialize, Serialize};

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

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;

use ariadne_build::{BuildDiscoverer, BuildMeasurer};
use ariadne_cluster::planner::OptimizationPlanner;
use ariadne_cluster::plan::RiskLevel;
use ariadne_graph::LoadAnalyzer;
use ariadne_repo::Repository;
use ariadne_report::ArtifactManager;
use ariadne_scheduler::ProductionAnalyzer;
use ariadne_transform::{HeaderExtractor, IncludeRewriter};
use ariadne_validate::{TierValidator, TransactionApplier, ValidationStatus};

pub struct EndToEndOptimizer {
    pub repo_path: PathBuf,
}

impl EndToEndOptimizer {
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
        }
    }

    pub fn optimize(&self) -> Result<OptimizationReport> {
        let repo = Repository::locate(&self.repo_path)?;
        let repo_discovery = repo.discover()?;
        let build_discoverer = BuildDiscoverer::new(&repo.root).with_repo_discovery(&repo_discovery);
        let build_discovery = build_discoverer.discover()?;

        let total_files = repo_discovery.files.len();
        let total_tus = build_discovery.translation_units.len();

        // 1. Initial measurement (before)
        let measurer = BuildMeasurer::new(repo.root.clone());
        let before_measurement = measurer.measure_all(&build_discovery.translation_units)?;

        // 2. Include analysis
        let (_repo_copy, graph, closures, _report) = crate::build_include_graph(&self.repo_path)?;

        // 3. Semantic analysis & reduction
        let prod_analyzer = ProductionAnalyzer::new(&repo, &repo_discovery, &build_discovery)
            .with_workers(4);
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
        let prod_result = rt.block_on(prod_analyzer.analyze())?;
        let model = prod_result.reduced_model;

        let total_symbols = model.symbols.len();
        let total_semantic_edges = model.edges.len();

        // 4. Load analysis
        let load_analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);
        let mut header_collateral_map: HashMap<String, usize> = HashMap::new();

        for sym in model.symbols.values() {
            let lm = load_analyzer.analyze_symbol_load(&sym.name).unwrap();
            for src in &lm.collateral_sources {
                if let Some(f) = repo_discovery.files.iter().find(|rf| &rf.id == &src.file_id) {
                    let name = f.path.file_name().and_then(|s: &std::ffi::OsStr| s.to_str()).unwrap_or("unknown").to_string();
                    *header_collateral_map.entry(name).or_insert(0) += src.symbol_count;
                }
            }
        }

        let mut collateral_opportunities: Vec<CollateralOpportunity> = header_collateral_map
            .into_iter()
            .map(|(header_name, collateral_symbols)| CollateralOpportunity {
                header_name,
                collateral_symbols,
            })
            .collect();
        collateral_opportunities.sort_by(|a, b| b.collateral_symbols.cmp(&a.collateral_symbols).then_with(|| a.header_name.cmp(&b.header_name)));

        // 5. Optimization planning
        let tu_file_ids: Vec<_> = build_discovery.translation_units.iter().map(|tu| tu.file_id).collect();
        let planner = OptimizationPlanner::new(&graph, &model, &repo_discovery.files, &tu_file_ids);
        let plans = planner.plan();

        let safe_plans: Vec<_> = plans
            .into_iter()
            .filter(|p| matches!(p.risk, RiskLevel::Conservative))
            .collect();

        let safe_plans_available = safe_plans.len();

        // 6. Preview, apply, and validate safe plans
        let mut affected_consumer_files_set = HashSet::new();
        let mut committed_transformations = 0;
        let mut validation_passed_tus = 0;
        let mut validation_total_tus = 0;
        let mut build_passed = true;

        for plan in &safe_plans {
            let plan_id = &plan.id;

            // Header extraction
            let extractor = HeaderExtractor::new(&model);
            let extraction = extractor.extract(plan);

            let mut symbol_name_to_header = std::collections::BTreeMap::new();
            let mut generated_headers = Vec::new();
            for header in &extraction.headers {
                let hpath = PathBuf::from("generated").join(&header.filename);
                let mut new_header = header.clone();
                new_header.filename = hpath.clone();
                new_header.required_includes = header.required_includes.iter().map(|p| PathBuf::from("generated").join(p)).collect();
                generated_headers.push(new_header);
                for sym in &header.symbols {
                    symbol_name_to_header.insert(sym.name.clone(), hpath.clone());
                }
            }

            let target_include = plan.target_file.file_name().and_then(|n: &std::ffi::OsStr| n.to_str()).unwrap_or("");
            let all_symbol_names: Vec<String> = plan.proposed_extractions.iter().map(|e| e.symbol_name.clone()).collect();

            let mut consumers = Vec::new();
            for tu in &build_discovery.translation_units {
                let abs_path = repo.root.join(&tu.source_path);
                if let Ok(content) = fs::read_to_string(&abs_path) {
                    if content.contains(&format!("#include \"{}\"", target_include))
                        || content.contains(&format!("#include <{}>", target_include))
                    {
                        consumers.push((
                            tu.source_path.clone(),
                            vec![plan.target_file.clone()],
                            all_symbol_names.clone(),
                        ));
                        affected_consumer_files_set.insert(tu.source_path.clone());
                    }
                }
            }

            let rewriter = IncludeRewriter::new(symbol_name_to_header, generated_headers);
            let migration = rewriter.build_preview(plan_id.to_string(), &consumers);

            validation_total_tus += consumers.len();

            let (snapshot, new_paths) = TransactionApplier::apply(&repo.root, &extraction, &migration)?;

            let t1 = TierValidator::validate_structural(&extraction);
            let header_output_dir = repo.root.join("generated");
            let t2 = TierValidator::validate_generated_headers(&extraction, &header_output_dir);
            let t3 = TierValidator::validate_affected_tus(&migration, &header_output_dir);

            if t1.status == ValidationStatus::Pass
                && t2.status == ValidationStatus::Pass
                && t3.status == ValidationStatus::Pass
            {
                committed_transformations += 1;
                validation_passed_tus += consumers.len();
            } else {
                build_passed = false;
                TransactionApplier::rollback(&snapshot, &new_paths)?;
            }
        }

        // 7. Post-measurement (after)
        let after_measurement = measurer.measure_all(&build_discovery.translation_units)?;

        // Residual collateral load
        let initial_collateral_sum: usize = collateral_opportunities.iter().map(|o| o.collateral_symbols).sum();
        let reduced_collateral: usize = safe_plans
            .iter()
            .take(committed_transformations)
            .map(|p| p.collateral_load_before.saturating_sub(p.collateral_load_after))
            .sum();
        let residual_collateral_load = initial_collateral_sum.saturating_sub(reduced_collateral);

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

        // Save optimization artifacts
        let _artifacts = ArtifactManager::prepare_for_repo(&repo)?;
        let opt_dir = repo.root.join("artifacts").join("optimize");
        fs::create_dir_all(&opt_dir)?;
        if let Ok(json) = serde_json::to_string_pretty(&report) {
            let _ = fs::write(opt_dir.join("report.json"), json);
        }

        Ok(report)
    }
}

pub fn handle_optimize(repo_path: &Path) -> Result<OptimizationReport> {
    let optimizer = EndToEndOptimizer::new(repo_path);
    let report = optimizer.optimize()?;
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
}
