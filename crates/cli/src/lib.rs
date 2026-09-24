use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::PathBuf;
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use ariadne_repo::{FileId, FileOwnership, OptimizationPolicy, Repository};
use ariadne_build::BuildDiscoverer;
use ariadne_graph::{IncludeClosure, IncludeGraph, IncludeReport, IncludeResolver};
use ariadne_report::ArtifactManager;

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
pub enum Commands {
    /// Show version information
    Version,
    /// Analyze repository dependencies
    Analyze {
        /// Path to repository root
        repo: PathBuf,
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

pub fn run() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Version) | None => {
            println!("tool version 0.1.0");
        }
        Some(Commands::Analyze { repo }) => {
            let repository = Repository::locate(&repo)?;
            let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
            println!("Analyzed repository at: {:?}", repository.root);
            println!("Artifacts directory initialized at: {:?}", artifacts.artifact_root);
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
    }

    Ok(())
}
