use crate::commands::plan::plan_extractions;
use anyhow::{bail, Result};
use ariadne_build::BuildDiscoverer;
use ariadne_repo::{FileId, Repository};
use ariadne_report::ArtifactManager;
use ariadne_transform::{GeneratedUnit, IncludeRewriter};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// Content provider for the isolation engine (plan M23, D-M23-3): resolves a
/// planned extraction's `source_file` FileId through the repository catalog
/// and reads the file from disk. An unreadable file yields `None` — the
/// engine then keeps its placeholder body (I4), never an error.
pub fn repository_source_reader(
    repository: &Repository,
) -> Result<impl Fn(FileId) -> Option<String>> {
    let discovery = repository.discover()?;
    let path_by_id: BTreeMap<FileId, PathBuf> = discovery
        .files
        .iter()
        .map(|record| (record.id, record.path.clone()))
        .collect();
    let repo_root = repository.root.clone();
    Ok(move |file_id| {
        let relative_path = path_by_id.get(&file_id)?;
        fs::read_to_string(repo_root.join(relative_path)).ok()
    })
}

pub fn extract_plan_headers(repo_path: &PathBuf, plan_id: &str) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let plans = artifacts.load_plan_artifacts()?;
    let plan = match plans.iter().find(|p| p.id.0 == plan_id) {
        Some(p) => p,
        None => bail!("Plan {} not found", plan_id),
    };

    let model = artifacts.load_reduced_artifacts()?;
    let extractor = ariadne_transform::IsolationEngine::new(&model);
    let source_reader = repository_source_reader(&repository)?;

    let output_dir = artifacts.artifact_root.join("generated");
    // M23 T7 (D-M23-7 ii): the extraction backend is routed by the PLAN's
    // language (target-file detector) — the flip made the pre-T7
    // exactly-one-claim selection ambiguous; C++ plans keep resolving to
    // the C++ plugin, Scala plans now get their own.
    let registry = crate::registry::default_registry();
    let text_extractor = crate::registry::plan_backend(&registry, &plan.target_file, |caps| caps.extraction, "extraction")?
        .extractor();
    let result = extractor.extract(plan, &source_reader, text_extractor);
    artifacts.save_extraction_artifacts(&result, &output_dir, text_extractor)?;

    println!(
        "Extracted {} headers from plan {}",
        result.headers.len(),
        plan_id
    );
    println!("generated/");

    let mut files = result
        .headers
        .iter()
        .map(|h| h.path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
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

pub fn build_migration_preview(repo_path: &PathBuf, plan_id: &str, preview: bool) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    // 1. Load the plan
    let plan = load_plan_for_action(&artifacts, repo_path, plan_id)?;

    // 2-3. Build symbol->header map and generated headers list from the plan
    let (symbol_name_to_header, generated_headers) = plan_rewrite_targets(&plan);

    // 4. Find consumers: TUs that include the target file (heuristic: check source file content)
    let consumers = find_plan_consumers(&repository, &plan)?;

    // 5. Build preview
    let rewriter = IncludeRewriter::new(symbol_name_to_header, generated_headers);
    let preview_result = rewriter.build_preview(plan_id.to_string(), &consumers);

    // 6. Print diff
    if preview {
        println!("Migration preview for {}:", plan_id);
        println!();
        if preview_result.rewrites.is_empty() {
            println!(
                "  No consumers found for target: {}",
                plan.target_file.display()
            );
        } else {
            // M23 T7 (D-M23-7 ii): directive diff text is rendered by the
            // plan-language rewriting backend.
            let registry = crate::registry::default_registry();
            let rewriter = crate::registry::plan_backend(&registry, &plan.target_file, |caps| caps.rewriting, "rewriting")?
                .rewriter();
            println!("{}", preview_result.format_diff_with(rewriter));
            println!("No files changed.");
        }
    }

    // 7. Save artifacts
    artifacts.save_migration_artifacts(&preview_result)?;

    println!(
        "Migration preview: {} effective rewrites",
        preview_result.effective_rewrites()
    );

    Ok(())
}

pub fn apply_transform_plan(repo_path: &PathBuf, plan_id: &str) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let plan = load_plan_for_action(&artifacts, repo_path, plan_id)?;

    let model = artifacts.load_reduced_artifacts()?;

    let extractor = ariadne_transform::IsolationEngine::new(&model);
    // M23 T1/T7 (D-M23-3, D-M23-7 ii): the engine reads each extraction's
    // source file (via the catalog) and asks the PLAN-LANGUAGE extraction
    // backend to locate verbatim declaration bodies; unreadable files fall
    // back to placeholders (I4).
    let source_reader = repository_source_reader(&repository)?;
    let registry = crate::registry::default_registry();
    let text_extractor = crate::registry::plan_backend(&registry, &plan.target_file, |caps| caps.extraction, "extraction")?
        .extractor();
    let extraction = extractor.extract(&plan, &source_reader, text_extractor);

    let (symbol_name_to_header, generated_headers) = plan_rewrite_targets(&plan);

    let consumers = find_plan_consumers(&repository, &plan)?;

    let rewriter =
        ariadne_transform::IncludeRewriter::new(symbol_name_to_header, generated_headers);
    let migration = rewriter.build_preview(plan_id.to_string(), &consumers);

    println!("Applying {}...\n", plan_id);

    // M23 T7 (D-M23-7 ii): apply mutates directive text through the plan-
    // language rewriting backend; validate only orchestrates snapshot/
    // rollback/tiers.
    let rewriter = crate::registry::plan_backend(&registry, &plan.target_file, |caps| caps.rewriting, "rewriting")?
        .rewriter();

    let (snapshot, new_paths) = ariadne_validate::TransactionApplier::apply_with(
        &repository.root,
        &extraction,
        &migration,
        rewriter,
        text_extractor,
    )?;

    commit_or_rollback(
        &artifacts,
        plan_id,
        &repository,
        &extraction,
        &migration,
        &snapshot,
        &new_paths,
        &registry,
        &plan.target_file,
    )
}

fn load_plan_for_action(
    artifacts: &ArtifactManager,
    repo_path: &PathBuf,
    plan_id: &str,
) -> Result<ariadne_cluster::plan::OptimizationPlan> {
    let plans = artifacts
        .load_plan_artifacts()
        .or_else(|_| plan_extractions(repo_path).map(|_| vec![]))
        .unwrap_or_default();
    let plans = if plans.is_empty() {
        artifacts.load_plan_artifacts()?
    } else {
        plans
    };
    plans
        .into_iter()
        .find(|p| p.id.0 == plan_id)
        .ok_or_else(|| anyhow::anyhow!("Plan {} not found", plan_id))
}

fn plan_rewrite_targets(
    plan: &ariadne_cluster::plan::OptimizationPlan,
) -> (BTreeMap<String, PathBuf>, Vec<GeneratedUnit>) {
    // Build symbol_name_to_header map from plan's proposed_extractions
    let mut symbol_name_to_header: BTreeMap<String, PathBuf> = BTreeMap::new();
    // Generated units list (from proposed_extractions, conservative: no required deps)
    let mut unit_map: BTreeMap<PathBuf, GeneratedUnit> = BTreeMap::new();
    for ext in &plan.proposed_extractions {
        symbol_name_to_header.insert(ext.symbol_name.clone(), ext.target_header.clone());
        unit_map
            .entry(ext.target_header.clone())
            .or_insert_with(|| GeneratedUnit {
                path: ext.target_header.clone(),
                symbols: vec![],
                required_deps: vec![],
                primary_source_text: None,
            });
    }
    (symbol_name_to_header, unit_map.into_values().collect())
}

/// Consumer rewrite input: (TU source path, old includes, symbol names).
pub(crate) type PlanConsumers = Vec<(PathBuf, Vec<PathBuf>, Vec<String>)>;

/// Neutral consumer discovery (plan M23, D-M23-7 iii, T7 S2): for every TU,
/// the plugin of that file's **catalog language** (never a C++ default)
/// scans its own directive syntax for `target_include`; a hit ⇔ consumer,
/// keeping today's conservative (path, old-include, all-symbols) payload.
/// `lang-cpp` reproduces the pre-neutralization raw substring scan
/// byte-for-byte, so C++ consumer sets are set-equal (I1 — locked by the
/// untouched m10/m14/m19/m20 characterization suites); Scala consumers are
/// located by import scan but stay rewrite-empty until the T7b move-removal
/// transaction (orchestrator scope fence — hits are membership-only).
pub(crate) fn plan_target_consumers(
    registry: &ariadne_lang::LanguageRegistry,
    catalog: &std::collections::BTreeMap<FileId, ariadne_repo::SourceLanguage>,
    translation_units: &[ariadne_build::TranslationUnit],
    repo_root: &std::path::Path,
    target_include: &str,
    target_file: &PathBuf,
    all_symbol_names: &[String],
) -> PlanConsumers {
    let mut consumers: PlanConsumers = Vec::new();
    for tu in translation_units {
        let abs_path = repo_root.join(&tu.source_path);
        let Ok(content) = fs::read_to_string(&abs_path) else {
            continue;
        };
        let Some(plugin) = crate::registry::plugin_for_catalog_file(registry, catalog, tu.file_id)
        else {
            continue;
        };
        if !plugin
            .rewriter()
            .find_dependency_targets(&content, target_include)
            .is_empty()
        {
            consumers.push((
                tu.source_path.clone(),
                vec![target_file.clone()],
                all_symbol_names.to_vec(), // conservative: all symbols
            ));
        }
    }
    consumers
}

fn find_plan_consumers(
    repository: &Repository,
    plan: &ariadne_cluster::plan::OptimizationPlan,
) -> Result<PlanConsumers> {
    let repo_discovery = repository.discover()?;
    // D22: registry unit-shaper selection by build markers at the repo root
    // (first-wins fallback preserved).
    let registry = crate::registry::default_registry();
    let build_discoverer = BuildDiscoverer::new(&repository.root)
        .with_repo_discovery(&repo_discovery)
        .with_unit_shaper(registry.unit_shaper_for_repo(&repository.root));
    let build_result = build_discoverer.discover()?;

    let target_include = plan
        .target_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let all_symbol_names: Vec<String> = plan
        .proposed_extractions
        .iter()
        .map(|e| e.symbol_name.clone())
        .collect();

    Ok(plan_target_consumers(
        &registry,
        &crate::registry::catalog_languages(&repo_discovery),
        &build_result.translation_units,
        &repository.root,
        target_include,
        &plan.target_file,
        &all_symbol_names,
    ))
}

fn commit_or_rollback(
    artifacts: &ArtifactManager,
    plan_id: &str,
    repository: &Repository,
    extraction: &ariadne_transform::ExtractionResult,
    migration: &ariadne_transform::MigrationPreview,
    snapshot: &ariadne_validate::FileSnapshot,
    new_paths: &[PathBuf],
    registry: &ariadne_lang::LanguageRegistry,
    plan_target_file: &std::path::Path,
) -> Result<()> {
    let mut tiers_passed = Vec::new();

    // M18.10 (§6.2 `validate::lib`): rollback fires ONLY on Fail — a Skipped
    // tier (e.g. no compiler toolchain) commits like a Pass. The tier-3
    // skip decision runs through the injected plugin validator's
    // `toolchain_status()`, never a probe inside `ariadne-validate`.
    // M23 T7 (D-M23-7 ii): the tier-3 toolchain belongs to the PLAN's
    // language (C++ plans clang, Scala plans the probe-only
    // ScalaToolchainValidator — offline Missing ⇒ Skipped, I7).
    let compiler_validator =
        crate::registry::plan_backend(registry, plan_target_file, |caps| caps.validation, "validation")?
            .validator();

    let t1 = ariadne_validate::TierValidator::validate_structural(extraction);
    if !t1.status.is_fail() {
        tiers_passed.push(t1);
    } else {
        return report_rollback(artifacts, plan_id, t1, tiers_passed, snapshot, new_paths);
    }

    let header_output_dir = repository.root.join("generated");
    let t2 =
        ariadne_validate::TierValidator::validate_generated_headers(extraction, &header_output_dir);
    if !t2.status.is_fail() {
        tiers_passed.push(t2);
    } else {
        return report_rollback(artifacts, plan_id, t2, tiers_passed, snapshot, new_paths);
    }

    let t3 = ariadne_validate::TierValidator::validate_affected_tus(
        migration,
        &header_output_dir,
        Some(compiler_validator),
    );
    if !t3.status.is_fail() {
        tiers_passed.push(t3);
    } else {
        return report_rollback(artifacts, plan_id, t3, tiers_passed, snapshot, new_paths);
    }

    let outcome = ariadne_validate::ApplyOutcome::Committed { tiers_passed };
    println!("{}", outcome.format_output());
    artifacts.save_transaction_artifacts(plan_id, &outcome)?;

    Ok(())
}

fn report_rollback(
    artifacts: &ArtifactManager,
    plan_id: &str,
    failed_tier: ariadne_validate::TierResult,
    tiers_passed: Vec<ariadne_validate::TierResult>,
    snapshot: &ariadne_validate::FileSnapshot,
    new_paths: &[PathBuf],
) -> Result<()> {
    let outcome = ariadne_validate::ApplyOutcome::RolledBack {
        failed_tier,
        tiers_passed,
    };
    println!("{}", outcome.format_output());
    ariadne_validate::TransactionApplier::rollback(snapshot, new_paths)?;
    artifacts.save_transaction_artifacts(plan_id, &outcome)?;
    Ok(())
}
