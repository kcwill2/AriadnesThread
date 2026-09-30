use anyhow::Result;
use ariadne_build::BuildDiscoverer;
use ariadne_index::{contexts_fingerprint, contexts_material, Fingerprint};
use ariadne_repo::{FileOwnership, OptimizationPolicy, Repository};
use ariadne_report::ArtifactManager;
use std::path::PathBuf;

pub fn discover_repository_files(repo_path: &PathBuf) -> Result<()> {
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
        discovery
            .ownership_summary
            .get(&FileOwnership::Repository)
            .unwrap_or(&0)
    );
    println!(
        "  ThirdParty: {}",
        discovery
            .ownership_summary
            .get(&FileOwnership::ThirdParty)
            .unwrap_or(&0)
    );
    println!(
        "  Toolchain: {}",
        discovery
            .ownership_summary
            .get(&FileOwnership::Toolchain)
            .unwrap_or(&0)
    );
    println!(
        "  Generated: {}",
        discovery
            .ownership_summary
            .get(&FileOwnership::Generated)
            .unwrap_or(&0)
    );
    println!();
    println!("Optimization policy:");
    println!(
        "  Optimizable: {}",
        discovery
            .policy_summary
            .get(&OptimizationPolicy::Optimizable)
            .unwrap_or(&0)
    );
    println!(
        "  Contract: {}",
        discovery
            .policy_summary
            .get(&OptimizationPolicy::Contract)
            .unwrap_or(&0)
    );
    println!(
        "  Frozen: {}",
        discovery
            .policy_summary
            .get(&OptimizationPolicy::Frozen)
            .unwrap_or(&0)
    );

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    let hashes: Vec<_> = discovery
        .files
        .iter()
        .map(|f| (f.id, f.content_hash.clone()))
        .collect();
    let repo_fp = Fingerprint::inputs(&repository.root, &hashes);
    artifacts.save_repository_artifacts(&discovery, &repo_fp)?;

    Ok(())
}

pub fn discover_build_contexts(repo_path: &PathBuf) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let repo_discovery = repository.discover().ok();

    // M18.3: the unit shaper is selected through the registry (single
    // registered plugin with a shaper ⇒ that shaper).
    let registry = crate::registry::default_registry();
    let discoverer = BuildDiscoverer::new(&repository.root).with_unit_shaper(registry.unit_shaper_for());
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
    println!(
        "Translation units: {}",
        build_result.translation_units.len()
    );

    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;
    let hashes: Vec<_> = repo_discovery
        .as_ref()
        .map(|d| {
            d.files
                .iter()
                .map(|f| (f.id, f.content_hash.clone()))
                .collect()
        })
        .unwrap_or_default();
    let repo_fp = Fingerprint::inputs(&repository.root, &hashes);
    let contexts_fp = contexts_fingerprint(
        &repo_fp,
        &contexts_material(&build_result.contexts, &build_result.translation_units),
    );
    artifacts.save_build_context_artifacts(&build_result, &contexts_fp)?;

    Ok(())
}
