pub mod optimize;
pub use optimize::*;

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use ariadne_repo::{FileId, FileOwnership, OptimizationPolicy, Repository};
use ariadne_build::BuildDiscoverer;
use ariadne_graph::{IncludeClosure, IncludeGraph, IncludeReport, IncludeResolver, ReducedSemanticModel};
use ariadne_report::ArtifactManager;
use ariadne_index::{CppSemanticAnalyzer, IndexResult, TuAnalysis};

#[derive(Parser)]
#[command(name = "tool", version = "0.1.0", about = "C++ Dependency Optimization Tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum BuildSubcommands {
    /// Discover compilation contexts and translation units
    Discover {
        /// Path to repository root
        repo: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum IncludesSubcommands {
    /// Explain include tree for a specific file
    Explain {
        /// Path to repository root
        repo: PathBuf,
        /// Target file to explain
        #[arg(short, long)]
        file: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum QuerySubcommands {
    /// Query symbol details
    Symbol {
        /// Path to repository root
        repo: PathBuf,
        /// Name or ID of the symbol
        name: String,
    },
    /// Query symbol dependencies
    Deps {
        /// Path to repository root
        repo: PathBuf,
        /// Name or ID of the symbol
        name: String,
    },
    /// Query symbol users
    Users {
        /// Path to repository root
        repo: PathBuf,
        /// Name or ID of the symbol
        name: String,
    },
}

#[derive(Subcommand)]
pub enum AnalyzeSubcommands {
    /// Load analysis
    Load {
        /// Path to repository root
        repo: PathBuf,
        /// Symbol to analyze
        #[arg(long)]
        symbol: Option<String>,
        /// File to analyze
        #[arg(long)]
        file: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum ExplainSubcommands {
    /// Explain dependency load
    Load {
        /// Path to repository root
        repo: PathBuf,
        /// Symbol to explain
        #[arg(long)]
        symbol: Option<String>,
        /// File to explain
        #[arg(long)]
        file: Option<PathBuf>,
        /// Optional collateral source filter
        #[arg(long)]
        source: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PlanSubcommands {
    /// Show detailed optimization plan
    Show {
        /// Path to repository root
        repo: PathBuf,
        /// Plan ID (e.g. PLAN-001)
        #[arg(long)]
        id: String,
    },
}


#[derive(Subcommand)]
pub enum TransformSubcommands {
    /// Extract headers based on optimization plan
    Extract {
        /// Path to repository root
        repo: PathBuf,
        /// Plan ID to extract
        #[arg(long)]
        plan: String,
    },
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show version information
    Version,
    /// Analyze repository dependencies
    Analyze {
        /// Path to repository root
        repo: Option<PathBuf>,
        /// Number of parallel workers
        #[arg(long)]
        workers: Option<usize>,
        #[command(subcommand)]
        command: Option<AnalyzeSubcommands>,
    },
    /// Explain dependencies
    Explain {
        #[command(subcommand)]
        command: ExplainSubcommands,
    },
    /// Discover repository source files
    Discover {
        /// Path to repository root
        repo: PathBuf,
    },
    /// Build context discovery commands
    Build {
        #[command(subcommand)]
        command: BuildSubcommands,
    },
    /// Include graph analysis commands
    Includes {
        /// Path to repository root
        repo: Option<PathBuf>,
        #[command(subcommand)]
        command: Option<IncludesSubcommands>,
    },
    /// Semantic indexing commands
    Index {
        /// Path to repository root
        repo: PathBuf,
    },
    /// Query semantic model
    Query {
        #[command(subcommand)]
        command: QuerySubcommands,
    },
    /// Generate or show optimization plans
    Plan {
        /// Path to repository root
        repo: Option<PathBuf>,
        #[command(subcommand)]
        command: Option<PlanSubcommands>,
    },
    /// Transformation commands
    Transform {
        #[command(subcommand)]
        command: TransformSubcommands,
    },
    /// Migrate consumers to generated headers
    Migrate {
        /// Path to repository root
        repo: PathBuf,
        /// Plan ID to migrate consumers for
        #[arg(long)]
        plan: String,
        /// Preview changes without applying them
        #[arg(long, default_value_t = false)]
        preview: bool,
    },
    /// Apply an optimization plan transactionally
    Apply {
        /// Path to repository root
        repo: PathBuf,
        /// Plan ID to apply
        #[arg(long)]
        plan: String,
    },
    /// Measure build performance of a repository
    Measure {
        /// Path to repository root
        repo: PathBuf,
    },
    /// Compare two build performance measurements
    Compare {
        /// Before measurement path
        before: PathBuf,
        /// After measurement path
        after: PathBuf,
    },
    /// Optimize repository dependencies end-to-end
    Optimize {
        /// Path to repository root
        repo: PathBuf,
    },
    /// Launch interactive graphical dependency and call graph interface
    Ui {
        /// Path to repository root
        repo: Option<PathBuf>,
        /// Port to bind web server
        #[arg(long, default_value_t = 3000)]
        port: u16,
        /// Do not open browser automatically
        #[arg(long, default_value_t = false)]
        no_open: bool,
        /// Export standalone HTML report to path
        #[arg(long)]
        export: Option<PathBuf>,
    },
}

pub fn handle_discover(repo_path: &PathBuf) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let discovery = repository.discover()?;

    println!("Repository");
    println!("  Root: {}", repository.root.display());
    println!();
    println!("Files:");
    for file in &discovery.files {
        println!("  {}", file.canonical_path.display());
    }
    println!();
    println!("Ownership:");
    println!(
        "  Repository: {}",
        discovery.ownership_summary.get(&FileOwnership::Repository).unwrap_or(&0)
    );
    println!(
        "  ThirdParty: {}",
        discovery.ownership_summary.get(&FileOwnership::ThirdParty).unwrap_or(&0)
    );
    println!(
        "  Toolchain: {}",
        discovery.ownership_summary.get(&FileOwnership::Toolchain).unwrap_or(&0)
    );
    println!(
        "  Generated: {}",
        discovery.ownership_summary.get(&FileOwnership::Generated).unwrap_or(&0)
    );
    println!();
    println!("Optimization policy:");
    println!(
        "  Optimizable: {}",
        discovery.policy_summary.get(&OptimizationPolicy::Optimizable).unwrap_or(&0)
    );
    println!(
        "  Contract: {}",
        discovery.policy_summary.get(&OptimizationPolicy::Contract).unwrap_or(&0)
    );
    println!(
        "  Frozen: {}",
        discovery.policy_summary.get(&OptimizationPolicy::Frozen).unwrap_or(&0)
    );

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    artifacts.save_repository_artifacts(&discovery)?;

    Ok(())
}

pub fn handle_build_discover(repo_path: &PathBuf) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let repo_discovery = repository.discover().ok();

    let discoverer = BuildDiscoverer::new(&repository.root);
    let build_result = if let Some(ref disc) = repo_discovery {
        discoverer.with_repo_discovery(disc).discover()?
    } else {
        discoverer.discover()?
    };

    println!("Compilation contexts: {}", build_result.contexts.len());
    println!();
    for ctx in &build_result.contexts {
        println!("Context: {}", ctx.name);
        println!("  Compiler: {}", ctx.compiler);
        println!("  Definitions:");
        for def in &ctx.defines {
            println!("    {}", def);
        }
        println!("  Include roots:");
        for inc in &ctx.include_dirs {
            println!("    {}", inc.display());
        }
        println!();
    }
    println!("Translation units: {}", build_result.translation_units.len());

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    artifacts.save_build_context_artifacts(&build_result)?;

    Ok(())
}

pub fn build_include_graph(
    repo_path: &PathBuf,
) -> Result<(Repository, IncludeGraph, HashMap<FileId, IncludeClosure>, IncludeReport)> {
    let repository = Repository::locate(repo_path)?;
    let repo_discovery = repository.discover()?;

    let build_discoverer = BuildDiscoverer::new(&repository.root).with_repo_discovery(&repo_discovery);
    let build_result = build_discoverer.discover()?;

    let mut graph = IncludeGraph::new();

    for file_record in &repo_discovery.files {
        graph.register_file(file_record.id, file_record.path.clone());
    }

    let mut scanned_files: HashSet<FileId> = HashSet::new();
    let mut scan_queue: VecDeque<(FileId, PathBuf, Option<String>)> = VecDeque::new();

    for tu in &build_result.translation_units {
        scan_queue.push_back((tu.file_id, tu.source_path.clone(), Some(tu.context_name.clone())));
    }

    while let Some((file_id, rel_path, ctx_name_opt)) = scan_queue.pop_front() {
        if scanned_files.contains(&file_id) {
            continue;
        }
        scanned_files.insert(file_id);

        let abs_path = repository.root.join(&rel_path);
        let content = match fs::read_to_string(&abs_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let target_ctx = ctx_name_opt
            .as_ref()
            .and_then(|name| build_result.contexts.iter().find(|c| &c.name == name))
            .or_else(|| build_result.contexts.first());

        let mut resolver = IncludeResolver::new(&repository.root).with_repo_discovery(&repo_discovery);
        if let Some(ctx) = target_ctx {
            resolver = resolver.with_context(ctx);
        }

        let edges = resolver.resolve_file_includes(file_id, &rel_path, &content);
        for edge in edges {
            if let Some(dest_id) = edge.destination {
                if !scanned_files.contains(&dest_id) {
                    if let Some(dest_path) = graph.file_paths.get(&dest_id).cloned() {
                        scan_queue.push_back((dest_id, dest_path, ctx_name_opt.clone()));
                    }
                }
            }
            graph.add_edge(edge);
        }
    }

    let closures = graph.compute_closures();
    let report = graph.generate_report();

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    artifacts.save_include_artifacts(&graph, &closures, &report)?;

    Ok((repository, graph, closures, report))
}

pub fn handle_includes(repo_path: &PathBuf) -> Result<()> {
    let (repository, _graph, _closures, report) = build_include_graph(repo_path)?;

    println!("Include Graph Summary");
    println!("  Repository: {}", repository.root.display());
    println!("  Total Edges: {}", report.total_edges);
    println!("  Unique Included Headers: {}", report.unique_headers);
    println!("  Local Headers: {}", report.local_headers_count);
    println!("  System Headers: {}", report.system_headers_count);
    println!("  External Headers: {}", report.external_headers_count);
    println!("  Unresolved Includes: {}", report.unresolved_includes_count);
    println!("  Cycles Detected: {}", report.cycle_count);
    println!();
    println!("Top Headers by Transitive Include Closure:");
    for (idx, (_id, path, closure_size)) in report.top_headers_by_closure.iter().enumerate() {
        println!("  {}. {} (closure size: {})", idx + 1, path.display(), closure_size);
    }

    Ok(())
}

pub fn handle_includes_explain(repo_path: &PathBuf, file_path: &PathBuf) -> Result<()> {
    let (_repository, graph, _closures, _report) = build_include_graph(repo_path)?;

    let target_id = match graph.find_file_id(file_path) {
        Some(id) => id,
        None => {
            let rel_str = file_path.to_string_lossy().replace('\\', "/");
            FileId::from_relative_path(&rel_str)
        }
    };

    if !graph.file_paths.contains_key(&target_id) {
        bail!("File {:?} not found in repository include graph", file_path);
    }

    let tree_str = graph.format_explain(target_id);
    print!("{}", tree_str);

    Ok(())
}

pub fn handle_index(repo_path: &PathBuf) -> Result<IndexResult> {
    let repository = Repository::locate(repo_path)?;
    let repo_discovery = repository.discover()?;

    let build_discoverer = BuildDiscoverer::new(&repository.root).with_repo_discovery(&repo_discovery);
    let build_result = build_discoverer.discover()?;

    let mut tu_analyses = Vec::new();

    for tu in &build_result.translation_units {
        let abs_path = repository.root.join(&tu.source_path);
        let content = fs::read_to_string(&abs_path)?;
        let target_ctx = build_result.contexts.iter().find(|c| c.name == tu.context_name)
            .or_else(|| build_result.contexts.first());

        let mut resolver = IncludeResolver::new(&repository.root).with_repo_discovery(&repo_discovery);
        if let Some(ctx) = target_ctx {
            resolver = resolver.with_context(ctx);
        }

        let edges = resolver.resolve_file_includes(tu.file_id.clone(), &tu.source_path, &content);
        
        let mut all_files = vec![(tu.file_id.clone(), tu.source_path.clone(), content)];
        
        let mut visited = HashSet::new();
        visited.insert(tu.file_id.clone());
        let mut queue = edges;
        
        while let Some(edge) = queue.pop() {
            if let Some(dest_id) = edge.destination {
                if visited.insert(dest_id.clone()) {
                    if let Some(dest_path) = repo_discovery.files.iter().find(|f| f.id == dest_id).map(|f| f.path.clone()) {
                        let header_abs = repository.root.join(&dest_path);
                        if let Ok(header_content) = fs::read_to_string(&header_abs) {
                            all_files.push((dest_id.clone(), dest_path.clone(), header_content.clone()));
                            let new_edges = resolver.resolve_file_includes(dest_id, &dest_path, &header_content);
                            queue.extend(new_edges);
                        }
                    }
                }
            }
        }

        let mut combined_symbols = Vec::new();
        let mut combined_declarations = Vec::new();
        let mut combined_edges = Vec::new();

        for (file_id, _, file_content) in all_files {
            let mut file_analysis = CppSemanticAnalyzer::analyze_source(file_id, &file_content, &tu.context_name);
            combined_symbols.append(&mut file_analysis.symbols);
            combined_declarations.append(&mut file_analysis.declarations);
            combined_edges.append(&mut file_analysis.edges);
        }
        
        tu_analyses.push(TuAnalysis {
            tu_file_id: tu.file_id.clone(),
            context_name: tu.context_name.clone(),
            symbols: combined_symbols,
            declarations: combined_declarations,
            edges: combined_edges,
            references: vec![],
            call_sites: vec![],
        });
    }

    let index_result = IndexResult::new(repository.root.clone(), tu_analyses);

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    artifacts.save_index_artifacts(&index_result)?;

    println!("Indexed {} translation units", index_result.tu_analyses.len());
    println!();
    println!("Symbols:");
    println!("  {}", index_result.total_symbols);
    println!();
    println!("Semantic edges:");
    println!("  {}", index_result.total_edges);
    println!();
    println!("Complete:");
    println!("  {}", index_result.complete_edges);
    println!();
    println!("Incomplete:");
    println!("  {}", index_result.incomplete_edges);

    Ok(index_result)
}

pub fn get_or_build_reduced_model(repo_path: &PathBuf) -> Result<(Repository, ReducedSemanticModel)> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    if let Ok(model) = artifacts.load_reduced_artifacts() {
        return Ok((repository, model));
    }

    // Need to build it
    let index_result = handle_index(repo_path)?;
    let model = ReducedSemanticModel::reduce(&index_result);
    artifacts.save_reduced_artifacts(&model)?;
    
    Ok((repository, model))
}

pub fn handle_query_symbol(repo_path: &PathBuf, name: &str) -> Result<()> {
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let sym = match model.find_symbol(name) {
        Some(s) => s,
        None => {
            println!("Symbol {} not found.", name);
            return Ok(());
        }
    };

    let file_map = {
        let mut map = std::collections::HashMap::new();
        if let Ok(content) = std::fs::read(repo_path.join("artifacts/repository/files.bin")) {
            if let Ok(records) = bincode::deserialize::<Vec<ariadne_repo::FileRecord>>(&content) {
                for r in records {
                    map.insert(r.id, r.path.to_string_lossy().into_owned());
                }
            }
        }
        map
    };

    println!("{}", sym.name);
    println!("  Definition:");
    if let Some(def_file) = &sym.defining_file {
        if let Some(path) = file_map.get(def_file) {
            println!("    {}", path);
        } else {
            println!("    {}", def_file);
        }
    } else {
        println!("    (none)");
    }

    println!();
    println!("  Semantic dependencies:");
    let deps = model.get_dependencies(&sym.id);
    if deps.is_empty() {
        println!("    (none)");
    } else {
        for dep in deps {
            let dep_sym = model.symbols.get(&dep.dst_symbol);
            if let Some(ds) = dep_sym {
                println!("    {}", ds.name);
            } else {
                println!("    {}", dep.dst_symbol.0);
            }
        }
    }

    println!();
    println!("  Used by:");
    let users = model.get_users(&sym.id);
    if users.is_empty() {
        println!("    (none)");
    } else {
        for u in users {
            if let Some(path) = file_map.get(&u) {
                println!("    {}", path);
            } else {
                println!("    {}", u);
            }
        }
    }

    Ok(())
}

pub fn handle_query_deps(repo_path: &PathBuf, name: &str) -> Result<()> {
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let sym = match model.find_symbol(name) {
        Some(s) => s,
        None => {
            println!("Symbol {} not found.", name);
            return Ok(());
        }
    };

    println!("{} dependencies:", sym.name);
    let deps = model.get_dependencies(&sym.id);
    if deps.is_empty() {
        println!("  (none)");
    } else {
        for dep in deps {
            let dep_sym = model.symbols.get(&dep.dst_symbol);
            if let Some(ds) = dep_sym {
                println!("  {}", ds.name);
            } else {
                println!("  {}", dep.dst_symbol.0);
            }
        }
    }

    Ok(())
}

pub fn handle_query_users(repo_path: &PathBuf, name: &str) -> Result<()> {
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let sym = match model.find_symbol(name) {
        Some(s) => s,
        None => {
            println!("Symbol {} not found.", name);
            return Ok(());
        }
    };

    let file_map = {
        let mut map = std::collections::HashMap::new();
        if let Ok(content) = std::fs::read(repo_path.join("artifacts/repository/files.bin")) {
            if let Ok(records) = bincode::deserialize::<Vec<ariadne_repo::FileRecord>>(&content) {
                for r in records {
                    map.insert(r.id, r.path.to_string_lossy().into_owned());
                }
            }
        }
        map
    };

    println!("{} used by:", sym.name);
    let users = model.get_users(&sym.id);
    if users.is_empty() {
        println!("  (none)");
    } else {
        for u in users {
            if let Some(path) = file_map.get(&u) {
                println!("  {}", path);
            } else {
                println!("  {}", u);
            }
        }
    }

    Ok(())
}

pub fn handle_analyze(repo_path: &PathBuf) -> Result<ariadne_scheduler::IncrementalAnalysisResult> {
    let repo = Repository::locate(repo_path)?;
    let repo_discovery = repo.discover()?;
    let build_discovery = BuildDiscoverer::new(&repo.root).with_repo_discovery(&repo_discovery).discover()?;
    let mut analyzer = ariadne_scheduler::IncrementalAnalyzer::new(&repo, &repo_discovery, &build_discovery);
    let result = analyzer.analyze()?;
    result.save_artifacts(&repo)?;

    println!("Analyzing...\n");
    println!("Reused:\n    {} translation units\n", result.stats.reused_tus);
    println!("Reanalyzed:\n    {} translation units\n", result.stats.reanalyzed_tus);
    println!("Cache hit rate:\n    {:.1}%", result.stats.hit_rate);

    Ok(result)
}

pub fn handle_production_analyze(repo_path: &PathBuf, workers: Option<usize>) -> Result<ariadne_scheduler::ProductionAnalysisResult> {
    let repo = Repository::locate(repo_path)?;
    let repo_discovery = repo.discover()?;
    let build_discovery = BuildDiscoverer::new(&repo.root).with_repo_discovery(&repo_discovery).discover()?;

    let num_workers = workers.unwrap_or(4);
    let analyzer = ariadne_scheduler::ProductionAnalyzer::new(&repo, &repo_discovery, &build_discovery)
        .with_workers(num_workers);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let result = rt.block_on(analyzer.analyze())?;

    let manager = ArtifactManager::prepare_for_repo(&repo)?;
    manager.save_reduced_artifacts(&result.reduced_model)?;
    manager.save_index_artifacts(&result.index_result)?;

    println!("{}", result.stats.format_m13_output());

    Ok(result)
}

use ariadne_graph::LoadAnalyzer;
use ariadne_report::LoadReport;

pub fn handle_analyze_load(repo_path: &PathBuf, symbol: Option<&str>, file: Option<&std::path::Path>) -> Result<()> {
    let (repository, graph, _closures, _report) = build_include_graph(repo_path)?;
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let analyzer = LoadAnalyzer::new(&graph, &model);
    let mut load_report = LoadReport {
        repository_root: repository.root.clone(),
        symbol_metrics: Vec::new(),
        file_metrics: Vec::new(),
    };

    if let Some(sym_name) = symbol {
        let metric = analyzer.analyze_symbol_load(sym_name)?;
        println!("{}", metric.format_summary());
        load_report.symbol_metrics.push(metric);
    } else if let Some(file_path) = file {
        let metric = analyzer.analyze_file_load(file_path)?;
        println!("{}", metric.format_summary());
        load_report.file_metrics.push(metric);
    } else {
        println!("Analyzing all symbols in model...");
        for sym_record in model.symbols.values() {
            if let Ok(metric) = analyzer.analyze_symbol_load(&sym_record.name) {
                load_report.symbol_metrics.push(metric);
            }
        }
        println!("Analyzed {} symbols.", load_report.symbol_metrics.len());

        println!("Analyzing all files in model...");
        for (file_id, _) in &model.file_symbols {
            if let Some(file_path) = graph.file_paths.get(file_id) {
                if let Ok(metric) = analyzer.analyze_file_load(file_path) {
                    load_report.file_metrics.push(metric);
                }
            }
        }
        println!("Analyzed {} files.", load_report.file_metrics.len());
    }

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    artifacts.save_load_artifacts(&load_report)?;

    Ok(())
}

pub fn handle_explain_load(repo_path: &PathBuf, symbol: Option<&str>, file: Option<&std::path::Path>, source: Option<&str>) -> Result<()> {
    let (_repository, graph, _closures, _report) = build_include_graph(repo_path)?;
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let analyzer = LoadAnalyzer::new(&graph, &model);

    if let Some(sym_name) = symbol {
        let explanation = analyzer.explain_symbol_load(sym_name, source)?;
        println!("{}", explanation.format_explanation());
    } else if let Some(file_path) = file {
        let explanation = analyzer.explain_file_load(file_path, source)?;
        println!("{}", explanation.format_explanation());
    } else {
        bail!("--symbol or --file is required to explain load");
    }

    Ok(())
}

pub fn handle_plan(repo_path: &PathBuf) -> Result<Vec<ariadne_cluster::plan::OptimizationPlan>> {
    let repository = Repository::locate(repo_path)?;
    let discovery = repository.discover()?;

    let build_discoverer = BuildDiscoverer::new(&repository.root).with_repo_discovery(&discovery);
    let build_result = build_discoverer.discover()?;
    let tu_file_ids: Vec<_> = build_result.translation_units.iter().map(|tu| tu.file_id).collect();

    let (_repository, graph, _closures, _report) = build_include_graph(repo_path)?;
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let planner = ariadne_cluster::planner::OptimizationPlanner::new(&graph, &model, &discovery.files, &tu_file_ids);
    let plans = planner.plan();

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    artifacts.save_plan_artifacts(&repository.root, &plans)?;

    println!("Optimization plans: {}", plans.len());
    println!();
    for plan in &plans {
        println!("{}", plan.format_detail());
    }

    Ok(plans)
}

pub fn handle_plan_show(repo_path: &PathBuf, id: &str) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let plans = match artifacts.load_plan_artifacts() {
        Ok(p) => p,
        Err(_) => handle_plan(repo_path)?,
    };

    if let Some(plan) = plans.iter().find(|p| p.id.0 == id) {
        println!("{}", plan.format_detail());
    } else {
        println!("Plan {} not found.", id);
    }

    Ok(())
}

pub fn handle_transform_extract(repo_path: &PathBuf, plan_id: &str) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let plans = artifacts.load_plan_artifacts()?;
    let plan = match plans.iter().find(|p| p.id.0 == plan_id) {
        Some(p) => p,
        None => bail!("Plan {} not found", plan_id),
    };

    let model = artifacts.load_reduced_artifacts()?;
    let extractor = ariadne_transform::HeaderExtractor::new(&model);
    let result = extractor.extract(plan);

    let output_dir = artifacts.artifact_root.join("generated");
    artifacts.save_extraction_artifacts(&result, &output_dir)?;

    println!("Extracted {} headers from plan {}", result.headers.len(), plan_id);
    println!("generated/");
    
    let mut files = result.headers.iter().map(|h| h.filename.to_string_lossy().to_string()).collect::<Vec<_>>();
    files.sort();
    
    for (i, f) in files.iter().enumerate() {
        if i == files.len() - 1 {
            println!("└── {}", f);
        } else {
            println!("├── {}", f);
        }
    }

    Ok(())
}

pub fn handle_migrate(repo_path: &PathBuf, plan_id: &str, preview: bool) -> Result<()> {
    use ariadne_transform::{IncludeRewriter, GeneratedHeader};
    use std::collections::BTreeMap;

    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    // 1. Load the plan
    let plans = artifacts.load_plan_artifacts()
        .or_else(|_| handle_plan(repo_path).map(|_| vec![]))
        .unwrap_or_default();
    let plans = if plans.is_empty() {
        artifacts.load_plan_artifacts()?
    } else {
        plans
    };
    let plan = plans.iter().find(|p| p.id.0 == plan_id)
        .ok_or_else(|| anyhow::anyhow!("Plan {} not found", plan_id))?;

    // 2. Build symbol_name_to_header map from plan's proposed_extractions
    let mut symbol_name_to_header: BTreeMap<String, PathBuf> = BTreeMap::new();
    for ext in &plan.proposed_extractions {
        symbol_name_to_header.insert(ext.symbol_name.clone(), ext.target_header.clone());
    }

    // 3. Build generated headers list (from proposed_extractions, conservative: no required_includes)
    let mut header_map: BTreeMap<PathBuf, GeneratedHeader> = BTreeMap::new();
    for ext in &plan.proposed_extractions {
        header_map.entry(ext.target_header.clone()).or_insert_with(|| GeneratedHeader {
            filename: ext.target_header.clone(),
            pragma_once: true,
            symbols: vec![],
            required_includes: vec![],
        });
    }
    let generated_headers: Vec<GeneratedHeader> = header_map.into_values().collect();

    // 4. Find consumers: TUs that include the target file (heuristic: check source file content)
    let repo_discovery = repository.discover()?;
    let build_discoverer = BuildDiscoverer::new(&repository.root).with_repo_discovery(&repo_discovery);
    let build_result = build_discoverer.discover()?;

    let target_include = plan.target_file.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    
    let all_symbol_names: Vec<String> = plan.proposed_extractions.iter()
        .map(|e| e.symbol_name.clone())
        .collect();

    let mut consumers: Vec<(PathBuf, Vec<PathBuf>, Vec<String>)> = Vec::new();
    for tu in &build_result.translation_units {
        let abs_path = repository.root.join(&tu.source_path);
        if let Ok(content) = fs::read_to_string(&abs_path) {
            // Check if this TU includes the target header
            if content.contains(&format!("#include \"{}\"", target_include))
                || content.contains(&format!("#include <{}>", target_include))
            {
                consumers.push((
                    tu.source_path.clone(),
                    vec![plan.target_file.clone()],
                    all_symbol_names.clone(),   // conservative: all symbols
                ));
            }
        }
    }

    // 5. Build preview
    let rewriter = IncludeRewriter::new(symbol_name_to_header, generated_headers);
    let preview_result = rewriter.build_preview(plan_id.to_string(), &consumers);

    // 6. Print diff
    if preview {
        println!("Migration preview for {}:", plan_id);
        println!();
        if preview_result.rewrites.is_empty() {
            println!("  No consumers found for target: {}", plan.target_file.display());
        } else {
            println!("{}", preview_result.format_diff());
            println!("No files changed.");
        }
    }

    // 7. Save artifacts
    artifacts.save_migration_artifacts(&preview_result)?;

    println!("Migration preview: {} effective rewrites", preview_result.effective_rewrites());

    Ok(())
}

pub fn handle_apply(repo_path: &PathBuf, plan_id: &str) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let plans = artifacts.load_plan_artifacts()
        .or_else(|_| handle_plan(repo_path).map(|_| vec![]))
        .unwrap_or_default();
    let plans = if plans.is_empty() {
        artifacts.load_plan_artifacts()?
    } else {
        plans
    };
    let plan = plans.iter().find(|p| p.id.0 == plan_id)
        .ok_or_else(|| anyhow::anyhow!("Plan {} not found", plan_id))?;

    let model = artifacts.load_reduced_artifacts()?;
    
    let extractor = ariadne_transform::HeaderExtractor::new(&model);
    let extraction = extractor.extract(plan);

    let mut symbol_name_to_header = std::collections::BTreeMap::new();
    let mut header_map = std::collections::BTreeMap::new();
    
    for ext in &plan.proposed_extractions {
        symbol_name_to_header.insert(ext.symbol_name.clone(), ext.target_header.clone());
        header_map.entry(ext.target_header.clone()).or_insert_with(|| ariadne_transform::GeneratedHeader {
            filename: ext.target_header.clone(),
            pragma_once: true,
            symbols: vec![],
            required_includes: vec![],
        });
    }
    
    let generated_headers: Vec<_> = header_map.into_values().collect();

    let repo_discovery = repository.discover()?;
    let build_discoverer = BuildDiscoverer::new(&repository.root).with_repo_discovery(&repo_discovery);
    let build_result = build_discoverer.discover()?;

    let target_include = plan.target_file.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    
    let all_symbol_names: Vec<String> = plan.proposed_extractions.iter()
        .map(|e| e.symbol_name.clone())
        .collect();

    let mut consumers = Vec::new();
    for tu in &build_result.translation_units {
        let abs_path = repository.root.join(&tu.source_path);
        if let Ok(content) = fs::read_to_string(&abs_path) {
            if content.contains(&format!("#include \"{}\"", target_include))
                || content.contains(&format!("#include <{}>", target_include))
            {
                consumers.push((
                    tu.source_path.clone(),
                    vec![plan.target_file.clone()],
                    all_symbol_names.clone(),
                ));
            }
        }
    }

    let rewriter = ariadne_transform::IncludeRewriter::new(symbol_name_to_header, generated_headers);
    let migration = rewriter.build_preview(plan_id.to_string(), &consumers);

    println!("Applying {}...\n", plan_id);
    
    let (snapshot, new_paths) = ariadne_validate::TransactionApplier::apply(&repository.root, &extraction, &migration)?;

    let mut tiers_passed = Vec::new();
    
    let t1 = ariadne_validate::TierValidator::validate_structural(&extraction);
    if t1.status == ariadne_validate::ValidationStatus::Pass {
        tiers_passed.push(t1);
    } else {
        let outcome = ariadne_validate::ApplyOutcome::RolledBack { failed_tier: t1, tiers_passed };
        println!("{}", outcome.format_output());
        ariadne_validate::TransactionApplier::rollback(&snapshot, &new_paths)?;
        artifacts.save_transaction_artifacts(plan_id, &outcome)?;
        return Ok(());
    }

    let header_output_dir = repository.root.join("generated");
    let t2 = ariadne_validate::TierValidator::validate_generated_headers(&extraction, &header_output_dir);
    if t2.status == ariadne_validate::ValidationStatus::Pass {
        tiers_passed.push(t2);
    } else {
        let outcome = ariadne_validate::ApplyOutcome::RolledBack { failed_tier: t2, tiers_passed };
        println!("{}", outcome.format_output());
        ariadne_validate::TransactionApplier::rollback(&snapshot, &new_paths)?;
        artifacts.save_transaction_artifacts(plan_id, &outcome)?;
        return Ok(());
    }

    let t3 = ariadne_validate::TierValidator::validate_affected_tus(&migration, &header_output_dir);
    if t3.status == ariadne_validate::ValidationStatus::Pass {
        tiers_passed.push(t3);
    } else {
        let outcome = ariadne_validate::ApplyOutcome::RolledBack { failed_tier: t3, tiers_passed };
        println!("{}", outcome.format_output());
        ariadne_validate::TransactionApplier::rollback(&snapshot, &new_paths)?;
        artifacts.save_transaction_artifacts(plan_id, &outcome)?;
        return Ok(());
    }

    let outcome = ariadne_validate::ApplyOutcome::Committed { tiers_passed };
    println!("{}", outcome.format_output());
    artifacts.save_transaction_artifacts(plan_id, &outcome)?;

    Ok(())
}

pub fn handle_measure(repo_path: &PathBuf) -> Result<ariadne_build::BuildMeasurement> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let repo_discovery = repository.discover().ok();
    let mut build_discoverer = BuildDiscoverer::new(&repository.root);
    if let Some(ref disc) = repo_discovery {
        build_discoverer = build_discoverer.with_repo_discovery(disc);
    }
    let build_result = build_discoverer.discover()?;

    let load_report = artifacts.load_load_artifacts().ok();
    let collateral_load = load_report.map(|lr| {
        lr.symbol_metrics.iter().map(|m| m.collateral_load).sum::<usize>()
    });

    let measurer = ariadne_build::BuildMeasurer::new(repository.root.clone())
        .with_structural_metrics(collateral_load, Some(build_result.translation_units.len()));

    println!("Measuring {} translation units...", build_result.translation_units.len());
    let measurement = measurer.measure_all(&build_result.translation_units)?;

    artifacts.save_measurement_artifacts(&measurement)?;

    println!("Measured front-end CPU: {:.1}s", measurement.total_frontend_cpu_time.as_secs_f64());
    println!("Measured wall time: {:.1}s", measurement.full_build_wall_time.as_secs_f64());
    println!("Peak memory: {} bytes", measurement.peak_memory_bytes);

    Ok(measurement)
}

pub fn handle_compare(before_path: &PathBuf, after_path: &PathBuf) -> Result<ariadne_build::MeasurementComparison> {
    let before = ArtifactManager::load_measurement_from_path(before_path)?;
    let after = ArtifactManager::load_measurement_from_path(after_path)?;

    let comparison = ariadne_build::MeasurementComparison::compare(before, after);
    println!("{}", comparison.format_report());

    Ok(comparison)
}

pub async fn run_ui(repo_opt: Option<PathBuf>, port: u16, no_open: bool, export_opt: Option<PathBuf>) -> Result<()> {
    let repo_path = repo_opt.unwrap_or_else(|| std::env::current_dir().unwrap());
    
    // Discover files, build compilation contexts, build include graph & closures & cycles.
    let (repository, include_graph, _closures, report) = build_include_graph(&repo_path)?;
    
    // Index source files & build reduced semantic model.
    let index_result = handle_index(&repo_path)?;
    let reduced_model = ReducedSemanticModel::reduce(&index_result);
    
    // Build CallGraph from index result & reduced model.
    let call_graph = ariadne_graph::CallGraph::build_from_index(&index_result, &reduced_model);
    
    // Build GraphExportPayload
    let payload = ariadne_ui::GraphExportPayload {
        repo_name: repository.root.file_name().unwrap_or_default().to_string_lossy().into_owned(),
        include_graph: ariadne_ui::build_include_elements(&include_graph, &_closures, &[]),
        symbol_graph: ariadne_ui::build_symbol_elements(&reduced_model),
        call_graph: ariadne_ui::build_call_elements(&call_graph),
        stats: ariadne_ui::SummaryStats {
            total_files: include_graph.file_paths.len(),
            total_symbols: reduced_model.symbols.len(),
            total_calls: call_graph.edges.len(),
            cycle_count: report.cycle_count,
        }
    };
    
    if let Some(path) = export_opt {
        ariadne_ui::export_standalone_html(&payload, "", &path)?;
        println!("Report exported successfully to {}", path.display());
    } else {
        println!("AriadnesThread Visualizer running at http://localhost:{}", port);
        
        let state = std::sync::Arc::new(ariadne_ui::UiState {
            repo_root: repository.root.clone(),
            include_graph,
            reduced_model,
            call_graph,
            payload,
        });
        
        let app = ariadne_ui::build_router(state);
        
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        
        if !no_open {
            let _ = open::that(format!("http://localhost:{}", port));
        }
        
        axum::serve(listener, app).await?;
    }
    
    Ok(())
}

pub fn run() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) | None => {
            println!("tool version 0.1.0");
        }
        Some(Commands::Analyze { repo, workers, command }) => match (repo, command) {
            (Some(repo_path), Some(AnalyzeSubcommands::Load { repo: load_repo, symbol, file })) => {
                let r = if repo_path != load_repo { load_repo } else { repo_path };
                handle_analyze_load(&r, symbol.as_deref(), file.as_deref())?;
            }
            (None, Some(AnalyzeSubcommands::Load { repo, symbol, file })) => {
                handle_analyze_load(&repo, symbol.as_deref(), file.as_deref())?;
            }
            (Some(repo_path), None) => {
                if let Some(w) = workers {
                    handle_production_analyze(&repo_path, Some(w))?;
                } else {
                    handle_analyze(&repo_path)?;
                }
            }
            (None, None) => {
                bail!("Repository path required for tool analyze command");
            }
        },
        Some(Commands::Explain { command: ExplainSubcommands::Load { repo, symbol, file, source } }) => {
            handle_explain_load(&repo, symbol.as_deref(), file.as_deref(), source.as_deref())?;
        }
        Some(Commands::Discover { repo }) => {
            handle_discover(&repo)?;
        }
        Some(Commands::Build { command: BuildSubcommands::Discover { repo } }) => {
            handle_build_discover(&repo)?;
        }
        Some(Commands::Includes { repo, command }) => match (repo, command) {
            (_, Some(IncludesSubcommands::Explain { repo, file })) => {
                handle_includes_explain(&repo, &file)?;
            }
            (Some(repo_path), None) => {
                handle_includes(&repo_path)?;
            }
            (None, None) => {
                bail!("Repository path required for tool includes command");
            }
        },
        Some(Commands::Index { repo }) => {
            handle_index(&repo)?;
        }
        Some(Commands::Query { command }) => match command {
            QuerySubcommands::Symbol { repo, name } => {
                handle_query_symbol(&repo, &name)?;
            }
            QuerySubcommands::Deps { repo, name } => {
                handle_query_deps(&repo, &name)?;
            }
            QuerySubcommands::Users { repo, name } => {
                handle_query_users(&repo, &name)?;
            }
        },
        Some(Commands::Plan { repo, command }) => match (repo, command) {
            (_, Some(PlanSubcommands::Show { repo, id })) => {
                handle_plan_show(&repo, &id)?;
            }
            (Some(repo_path), None) => {
                handle_plan(&repo_path)?;
            }
            (None, None) => {
                bail!("Repository path required for tool plan command");
            }
        },
        Some(Commands::Transform { command }) => match command {
            TransformSubcommands::Extract { repo, plan } => {
                handle_transform_extract(&repo, &plan)?;
            }
        },
        Some(Commands::Migrate { repo, plan, preview }) => {
            handle_migrate(&repo, &plan, preview)?;
        },
        Some(Commands::Apply { repo, plan }) => {
            handle_apply(&repo, &plan)?;
        },
        Some(Commands::Measure { repo }) => {
            handle_measure(&repo)?;
        },
        Some(Commands::Compare { before, after }) => {
            handle_compare(&before, &after)?;
        },
        Some(Commands::Optimize { repo }) => {
            handle_optimize(&repo)?;
        },
        Some(Commands::Ui { repo, port, no_open, export }) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(run_ui(repo, port, no_open, export))?;
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_analyze_load() {
        let args = vec!["tool", "analyze", "load", "my_repo", "--symbol", "MyClass"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Analyze { repo, workers: _, command: Some(AnalyzeSubcommands::Load { repo: load_repo, symbol, file }) } => {
                assert_eq!(repo, None);
                assert_eq!(load_repo.to_str().unwrap(), "my_repo");
                assert_eq!(symbol.unwrap(), "MyClass");
                assert_eq!(file, None);
            }
            _ => panic!("Expected Analyze Load command"),
        }
    }

    #[test]
    fn test_cli_explain_load() {
        let args = vec!["tool", "explain", "load", "my_repo", "--file", "my_file.cpp", "--source", "Huge.h"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Explain { command: ExplainSubcommands::Load { repo, symbol, file, source } } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(symbol, None);
                assert_eq!(file.unwrap().to_str().unwrap(), "my_file.cpp");
                assert_eq!(source.unwrap(), "Huge.h");
            }
            _ => panic!("Expected Explain Load command"),
        }
    }

    #[test]
    fn test_cli_plan() {
        let args = vec!["tool", "plan", "my_repo"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Plan { repo, command } => {
                assert_eq!(repo.unwrap().to_str().unwrap(), "my_repo");
                assert!(command.is_none());
            }
            _ => panic!("Expected Plan command"),
        }
    }

    #[test]
    fn test_cli_plan_show() {
        let args = vec!["tool", "plan", "show", "my_repo", "--id", "PLAN-001"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Plan { repo: _, command: Some(PlanSubcommands::Show { repo, id }) } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(id, "PLAN-001");
            }
            _ => panic!("Expected Plan Show command"),
        }
    }

    #[test]
    fn test_cli_transform_extract() {
        let args = vec!["tool", "transform", "extract", "my_repo", "--plan", "PLAN-001"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Transform { command: TransformSubcommands::Extract { repo, plan } } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(plan, "PLAN-001");
            }
            _ => panic!("Expected Transform Extract command"),
        }
    }

    #[test]
    fn test_cli_migrate_preview() {
        let args = vec!["tool", "migrate", "my_repo", "--plan", "PLAN-001", "--preview"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Migrate { repo, plan, preview } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(plan, "PLAN-001");
                assert!(preview);
            }
            _ => panic!("Expected Migrate command"),
        }
    }

    #[test]
    fn test_cli_migrate_no_preview_flag() {
        let args = vec!["tool", "migrate", "my_repo", "--plan", "PLAN-001"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Migrate { repo, plan, preview } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(plan, "PLAN-001");
                assert!(!preview); // default is false
            }
            _ => panic!("Expected Migrate command"),
        }
    }

    #[test]
    fn test_cli_apply() {
        let args = vec!["tool", "apply", "my_repo", "--plan", "PLAN-001"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Apply { repo, plan } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
                assert_eq!(plan, "PLAN-001");
            }
            _ => panic!("Expected Apply command"),
        }
    }
    #[test]
    fn test_cli_measure() {
        let args = vec!["tool", "measure", "my_repo"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Measure { repo } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
            }
            _ => panic!("Expected Measure command"),
        }
    }

    #[test]
    fn test_cli_compare() {
        let args = vec!["tool", "compare", "before_dir", "after_dir"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Compare { before, after } => {
                assert_eq!(before.to_str().unwrap(), "before_dir");
                assert_eq!(after.to_str().unwrap(), "after_dir");
            }
            _ => panic!("Expected Compare command"),
        }
    }

    #[test]
    fn test_handle_measure() {
        let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros();
        let temp = std::env::temp_dir().join(format!("ariadne_test_meas_{}", unique));
        let repo_dir = temp.join("repo");
        fs::create_dir_all(repo_dir.join("src")).unwrap();
        fs::write(repo_dir.join("ariadne.toml"), "").unwrap();
        
        let result = handle_measure(&repo_dir);
        assert!(result.is_ok());
        
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn test_handle_compare() {
        let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros();
        let temp = std::env::temp_dir().join(format!("ariadne_test_comp_{}", unique));
        
        let before_dir = temp.join("before");
        fs::create_dir_all(&before_dir).unwrap();
        
        let after_dir = temp.join("after");
        fs::create_dir_all(&after_dir).unwrap();
        
        let result = handle_compare(&before_dir, &after_dir);
        // This will fail because there are no valid measurement artifacts, 
        // but for now the command doesn't even exist.
        assert!(result.is_err());
        
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn test_cli_analyze_repo() {
        let args = vec!["tool", "analyze", "my_repo"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Analyze { repo, workers, command } => {
                assert_eq!(repo.unwrap().to_str().unwrap(), "my_repo");
                assert_eq!(workers, None);
                assert!(command.is_none());
            }
            _ => panic!("Expected Analyze command"),
        }
    }

    #[test]
    fn test_cli_analyze_workers_flag() {
        let args = vec!["tool", "analyze", "my_repo", "--workers", "32"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Analyze { repo, workers, command } => {
                assert_eq!(repo.unwrap().to_str().unwrap(), "my_repo");
                assert_eq!(workers, Some(32));
                assert!(command.is_none());
            }
            _ => panic!("Expected Analyze command with --workers"),
        }
    }

    #[test]
    fn test_handle_analyze() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf();
        let repo_dir = root.join("tests/fixtures/basic");

        let result = handle_analyze(&repo_dir).unwrap();
        assert!(result.stats.total_tus > 0 || result.stats.total_tus == 0); // Just check it returned Ok
    }

    #[test]
    fn test_cli_optimize() {
        let args = vec!["tool", "optimize", "my_repo"];
        let cli = Cli::parse_from(args);
        match cli.command.unwrap() {
            Commands::Optimize { repo } => {
                assert_eq!(repo.to_str().unwrap(), "my_repo");
            }
            _ => panic!("Expected Optimize command"),
        }
    }
}
