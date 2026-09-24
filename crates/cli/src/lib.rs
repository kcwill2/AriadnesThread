use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand};
use ariadne_repo::{FileOwnership, OptimizationPolicy, Repository};
use ariadne_report::ArtifactManager;

#[derive(Parser)]
#[command(name = "tool", version = "0.1.0", about = "C++ Dependency Optimization Tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
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
    }

    Ok(())
}
