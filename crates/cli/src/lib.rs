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

pub fn run() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) | None => {
            println!("tool version 0.1.0");
        }
        Some(Commands::Analyze { repo, command }) => match (repo, command) {
            (Some(repo_path), Some(AnalyzeSubcommands::Load { repo: load_repo, symbol, file })) => {
                let r = if repo_path != load_repo { load_repo } else { repo_path };
                handle_analyze_load(&r, symbol.as_deref(), file.as_deref())?;
            }
            (None, Some(AnalyzeSubcommands::Load { repo, symbol, file })) => {
                handle_analyze_load(&repo, symbol.as_deref(), file.as_deref())?;
            }
            (Some(repo_path), None) => {
                let repository = Repository::locate(&repo_path)?;
                let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
                println!("Analyzed repository at: {:?}", repository.root);
                println!("Artifacts directory initialized at: {:?}", artifacts.artifact_root);
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
            Commands::Analyze { repo, command: Some(AnalyzeSubcommands::Load { repo: load_repo, symbol, file }) } => {
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
}
