//! M23 T8a plan-witness suite (kickoff §5-T8, D-M23-5/D-M23-9): the
//! `scala-isolation` fixture proves the WIRED CLI route (`plan_extractions`
//! with the default registry — no test-injected hooks) proposes a real Scala
//! extraction: a `{Symbol}.scala` target beside the shared package-object
//! file, `source_file` provenance on the proposal (D-M23-3), D17 companion
//! co-targeting, and zero `PlanningUnsupported(Scala)` rejections (the
//! D-M23-6 flip).
//!
//! T7b/T8b will add: preview, apply+move-removal transaction, rollback,
//! tiers [Pass,Pass,Skipped], byte-preservation.
//!
//! Isolation: every test owns a tempdir copy of the fixture; the copy
//! excludes `artifacts/` and dot-dirs (the m22 harness pattern this suite
//! copies, so a stale stage directory can never ride in).

use anyhow::Result;
use ariadne_build::BuildDiscoverer;
use ariadne_cli::commands::index::build_source_index_filtered;
use ariadne_cli::commands::transform::{
    plan_rewrite_targets, plan_target_consumers, repository_source_reader,
};
use ariadne_cli::registry::{catalog_languages, default_registry, plan_backend};
use ariadne_cli::{apply_transform_plan, build_migration_preview, plan_extractions};
use ariadne_graph::{build_dependency_graph, LoadAnalyzer};
use ariadne_index::DependencyKind;
use ariadne_lang_scala::ScalaUnitShaper;
use ariadne_pipeline::Pipeline;
use ariadne_repo::{FileId, FileKind, FileOwnership, OptimizationPolicy, Repository, SourceLanguage};
use ariadne_transform::{IncludeRewriter, IsolationEngine};
use ariadne_validate::TransactionApplier;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE: &str = "scala-isolation";

const SHARED_REL: &str = "src/main/scala/com/example/legacy/legacy.scala";
const AUDIT_TRAIL_REL: &str = "src/main/scala/com/example/legacy/records/AuditTrail.scala";
const AUDIT_SEAL_REL: &str = "src/main/scala/com/example/legacy/records/AuditSeal.scala";
const CONSUMER_REL: &str = "src/main/scala/com/example/apps/Dashboard.scala";

const ALL_SCALA_RELS: [&str; 4] = [SHARED_REL, AUDIT_TRAIL_REL, AUDIT_SEAL_REL, CONSUMER_REL];

const SHARED_DIR: &str = "src/main/scala/com/example/legacy";
const DASHBOARD_ID: &str = "com.example.apps.Dashboard";
const LEGACY_REPORT_ID: &str = "com.example.legacy.LegacyReport";
const LEGACY_IMPORT_ID: &str = "com.example.legacy.LegacyImport";
const LEGACY_SHAPE_ID: &str = "com.example.legacy.LegacyShape";
const LEGACY_CIRCLE_ID: &str = "com.example.legacy.LegacyCircle";
const LEGACY_LEDGER_ID: &str = "com.example.legacy.LegacyLedger";
const LEGACY_LEDGER_OBJECT_ID: &str = "com.example.legacy.LegacyLedger$";
const PACKAGE_OBJECT_ID: &str = "com.example.legacy.legacy";
const AUDIT_TRAIL_ID: &str = "com.example.legacy.records.AuditTrail";
const AUDIT_SEAL_ID: &str = "com.example.legacy.records.AuditSeal";

struct IsolatedScalaRepo {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fixtures")
        .join(name)
}

/// Throwaway copy of the fixture WITHOUT `artifacts/` or dot-dirs (m22
/// pattern: every test builds its own stage outputs from a clean copy).
fn isolated_scala_repo(label: &str) -> Result<IsolatedScalaRepo> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join(label);
    copy_tree_excluding_build_output(&fixture_path(FIXTURE), &root)?;
    Ok(IsolatedScalaRepo { _temp: temp, root })
}

fn copy_tree_excluding_build_output(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "artifacts" || name.starts_with('.') {
            continue;
        }
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree_excluding_build_output(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn normalized_rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn read_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)?;
    let parsed = serde_json::from_str(&text)?;
    Ok(parsed)
}

/// T7b defining-file filter lock: no proposed extraction may target the
/// file that defines its symbol (a self-move is a no-op the planner must
/// never plan).
fn assert_no_self_targeted_proposals(
    extractions: &[ariadne_cluster::plan::ProposedExtraction],
    defining_rel: &str,
) {
    let self_targeted: Vec<String> = extractions
        .iter()
        .filter(|ext| normalized_rel(&ext.target_header) == defining_rel)
        .map(|ext| ext.symbol_id.0.clone())
        .collect();
    assert!(
        self_targeted.is_empty(),
        "T7b: no proposal may target the file that defines its symbol (self-targeted: {self_targeted:?})"
    );
}

// ---------------------------------------------------------------------------
// 1. Fixture discovery + shaping (D-M23-9 sbt shape).
// ---------------------------------------------------------------------------

#[test]
fn discovery_records_the_four_scala_sources() -> Result<()> {
    let repo = isolated_scala_repo("discover")?;
    let repository = Repository::locate(&repo.root)?;
    let discovery = repository.discover()?;

    let mut actual: Vec<String> = discovery
        .files
        .iter()
        .map(|record| normalized_rel(&record.path))
        .collect();
    actual.sort();
    let mut expected: Vec<String> = ALL_SCALA_RELS.iter().map(|path| path.to_string()).collect();
    expected.sort();
    assert_eq!(actual, expected, "exactly the four .scala files are recorded");

    for record in &discovery.files {
        assert_eq!(record.kind, FileKind::ScalaSource);
        assert_eq!(record.language, SourceLanguage::Scala);
        assert_eq!(record.ownership, FileOwnership::Repository);
        assert_eq!(record.optimization_policy, OptimizationPolicy::Optimizable);
    }
    Ok(())
}

#[test]
fn shaper_shapes_four_units_from_the_sbt_layout() -> Result<()> {
    let repo = isolated_scala_repo("shaper")?;
    let repository = Repository::locate(&repo.root)?;
    let discovery = repository.discover()?;
    let build = BuildDiscoverer::new(&repository.root)
        .with_repo_discovery(&discovery)
        .with_unit_shaper(Some(&ScalaUnitShaper))
        .discover()?;

    assert_eq!(build.contexts.len(), 1, "one sbt root-project context");
    assert_eq!(build.contexts[0].language, SourceLanguage::Scala);

    assert_eq!(
        build.translation_units.len(),
        ALL_SCALA_RELS.len(),
        "one unit per src/main/scala file (the fixture keeps no src/test tree)"
    );
    let mut unit_ids: Vec<u64> = build
        .translation_units
        .iter()
        .map(|unit| unit.file_id.0)
        .collect();
    unit_ids.sort();
    let mut expected_ids: Vec<u64> = ALL_SCALA_RELS
        .iter()
        .map(|path| FileId::from_relative_path(path).0)
        .collect();
    expected_ids.sort();
    assert_eq!(unit_ids, expected_ids, "the shaped units are exactly the four sources");
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. Registry-aware graph + production index on the fixture copy.
// ---------------------------------------------------------------------------

#[test]
fn graph_and_index_are_green_on_the_fixture() -> Result<()> {
    let repo = isolated_scala_repo("graph-index")?;
    let registry = default_registry();
    let graph = build_dependency_graph(&repo.root, &registry)?;

    for rel in ALL_SCALA_RELS {
        let registered = graph
            .file_paths
            .values()
            .any(|path| normalized_rel(path) == rel);
        assert!(registered, "{rel} is registered in the include graph");
    }
    let report =
        graph.generate_report_with_cycle_policies(&|_file_id| ariadne_lang::CyclePolicy::Reject);
    assert_eq!(report.cycle_count, 0, "the fixture is acyclic by design");

    let index = build_source_index_filtered(
        &repo.root.to_path_buf(),
        Some(SourceLanguage::Scala),
    )?;
    assert_eq!(
        index.tu_analyses.len(),
        ALL_SCALA_RELS.len(),
        "every shaped unit is indexed"
    );

    for id in [
        PACKAGE_OBJECT_ID,
        LEGACY_REPORT_ID,
        LEGACY_IMPORT_ID,
        LEGACY_SHAPE_ID,
        LEGACY_CIRCLE_ID,
        LEGACY_LEDGER_ID,
        LEGACY_LEDGER_OBJECT_ID,
    ] {
        assert!(
            symbol_in_index(&index, id).is_some(),
            "the shared file's symbol {id} is in the index"
        );
    }
    assert!(
        has_edge(&index, LEGACY_CIRCLE_ID, LEGACY_SHAPE_ID, DependencyKind::Complete),
        "the in-file `extends` edge is Complete (D16)"
    );
    assert!(
        has_edge(&index, LEGACY_REPORT_ID, AUDIT_TRAIL_ID, DependencyKind::Complete),
        "the grouped-named import reference `AuditTrail` resolves Complete"
    );
    let shape = symbol_in_index(&index, LEGACY_SHAPE_ID).expect("LegacyShape is in the index");
    assert!(
        shape.modifiers.iter().any(|modifier| modifier == "sealed"),
        "D-M23-8: SymbolRecord.modifiers carries `sealed` on the sealed trait, got {:?}",
        shape.modifiers
    );
    Ok(())
}

fn symbol_in_index<'a>(
    result: &'a ariadne_index::IndexResult,
    symbol_id: &str,
) -> Option<&'a ariadne_index::SymbolRecord> {
    result
        .tu_analyses
        .iter()
        .flat_map(|tu| tu.symbols.iter())
        .find(|symbol| symbol.id.0 == symbol_id)
}

fn has_edge(
    result: &ariadne_index::IndexResult,
    src: &str,
    dst: &str,
    kind: DependencyKind,
) -> bool {
    result.tu_analyses.iter().flat_map(|tu| tu.edges.iter()).any(|edge| {
        edge.kind == kind && edge.src_symbol.0 == src && edge.dst_symbol.0 == dst
    })
}

// ---------------------------------------------------------------------------
// 3. The load story the extraction is meant to fix: the consumer's wildcard
//    rides in the WHOLE shared file but justifies only LegacyReport.
// ---------------------------------------------------------------------------

#[test]
fn consumer_load_blames_the_shared_file() -> Result<()> {
    let repo = isolated_scala_repo("consumer-load")?;
    let mut pipeline = Pipeline::new(&repo.root, None)?;
    let (graph, closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;

    let analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);
    let metric = analyzer.analyze_symbol_load(DASHBOARD_ID)?;

    assert!(
        metric
            .semantic_symbols
            .iter()
            .any(|id| id.0 == LEGACY_REPORT_ID),
        "LegacyReport is the justified symbol the consumer actually uses (wildcard \
         attribution D16 makes the guess `com.example.legacy.LegacyReport` match the id)"
    );

    let collateral_ids: Vec<String> = metric
        .collateral_symbols
        .iter()
        .map(|id| id.0.clone())
        .collect();
    for collateral_id in [PACKAGE_OBJECT_ID, LEGACY_IMPORT_ID, LEGACY_LEDGER_ID, AUDIT_SEAL_ID] {
        assert!(
            collateral_ids.iter().any(|id| id == collateral_id),
            "{collateral_id} is collateral to the consumer, got {collateral_ids:?}"
        );
    }

    let largest = metric
        .largest_collateral_source
        .expect("the shared file is the largest collateral source");
    assert_eq!(
        normalized_rel(&largest.path),
        SHARED_REL,
        "the shared package-object file carries the collateral"
    );
    assert_eq!(
        largest.symbol_count, 6,
        "six of the shared file's seven symbols arrive unused (only LegacyReport is used)"
    );
    assert_eq!(
        metric.collateral_load, 7,
        "6 shared-file symbols + the unreferenced AuditSeal riding the grouped import"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. THE key witness: the wired CLI plan route proposes the Scala
//    extraction (D-M23-5 naming through the CLI dispatch, NOT injected).
// ---------------------------------------------------------------------------

#[test]
fn wired_plan_proposes_scala_extraction() -> Result<()> {
    let repo = isolated_scala_repo("wired-plan")?;
    let plans = plan_extractions(&repo.root)?;

    assert!(
        !plans.is_empty(),
        "the wired planner must propose at least one Scala plan (D-M23-9 sizing)"
    );
    let plan = plans
        .iter()
        .find(|plan| normalized_rel(&plan.target_file) == SHARED_REL)
        .expect("the shared package-object file is the plan target");
    assert!(
        plan.collateral_load_before >= 1,
        "the candidate screen required non-zero collateral (AuditSeal): {}",
        plan.collateral_load_before
    );

    let report = plan
        .proposed_extractions
        .iter()
        .find(|ext| ext.symbol_id.0 == LEGACY_REPORT_ID)
        .expect("LegacyReport is proposed for extraction");
    assert_eq!(
        normalized_rel(&report.target_header),
        format!("{SHARED_DIR}/LegacyReport.scala"),
        "D-M23-5: `{{Symbol}}.scala` lands beside the shared file via scala_target_path_for \
         through the CLI dispatch"
    );
    assert_eq!(
        report.source_file,
        Some(FileId::from_relative_path(SHARED_REL)),
        "T1 provenance field: the proposal carries the shared-file FileId (D-M23-3)"
    );

    let ledger_targets: Vec<String> = plan
        .proposed_extractions
        .iter()
        .filter(|ext| ext.symbol_id.0 == LEGACY_LEDGER_ID || ext.symbol_id.0 == LEGACY_LEDGER_OBJECT_ID)
        .map(|ext| normalized_rel(&ext.target_header))
        .collect();
    assert_eq!(
        ledger_targets,
        vec![
            format!("{SHARED_DIR}/LegacyLedger.scala"),
            format!("{SHARED_DIR}/LegacyLedger.scala")
        ],
        "D17 companion pairing co-targets the class and its `$` companion object into one file"
    );

    // Every Scala proposal sources from the shared file (the only candidate).
    for ext in &plan.proposed_extractions {
        assert_eq!(
            ext.source_file,
            Some(FileId::from_relative_path(SHARED_REL)),
            "proposal {} proves source_file provenance",
            ext.symbol_id.0
        );
    }

    // T7b defining-file filter: `{Symbol}.scala` naming resolves the package
    // object `com.example.legacy.legacy` back to `legacy.scala` — extracting
    // a symbol into its own defining file is a no-op move and must never be
    // planned.
    assert_no_self_targeted_proposals(&plan.proposed_extractions, SHARED_REL);

    // The flip retired the gap at its source: no discovered language lacks
    // planning, so plan_with_rejections has no PlanningUnsupported(Scala) —
    // asserted from the registry rule the CLI applies (m22 pattern).
    let registry = default_registry();
    let repository = Repository::locate(&repo.root)?;
    let any_planning_gap = repository
        .discover()?
        .files
        .iter()
        .any(|record| match registry.plugin_for_id(ariadne_lang::LanguageId::from(record.language)) {
            Some(plugin) => !plugin.capabilities().planning,
            None => false,
        });
    assert!(
        !any_planning_gap,
        "D-M23-6: no PlanningUnsupported(Scala) rejection can exist post-flip"
    );

    assert_persisted_plan_stage(&repo.root, plans.len())?;
    assert_sealed_pair_single_batch(&plan.proposed_extractions);
    Ok(())
}

/// D-M23-8 (T8b TODO retirement): the typed extends channel makes the sealed
/// pair one colocated batch, so both members co-target the anchor's file —
/// never the candidate itself.
fn assert_sealed_pair_single_batch(
    extractions: &[ariadne_cluster::plan::ProposedExtraction],
) {
    let target_of = |symbol_id: &str| {
        extractions
            .iter()
            .find(|ext| ext.symbol_id.0 == symbol_id)
            .map(|ext| normalized_rel(&ext.target_header))
    };
    let shape_target = target_of(LEGACY_SHAPE_ID).expect("LegacyShape is proposed");
    let circle_target = target_of(LEGACY_CIRCLE_ID).expect("LegacyCircle is proposed");
    assert_eq!(
        shape_target, circle_target,
        "D-M23-8: the sealed pair (LegacyShape+LegacyCircle) lands in ONE batch, \
         so both members co-target the batch anchor's file"
    );
    assert_ne!(
        shape_target,
        SHARED_REL.to_string(),
        "the sealed batch moves to a new file, not the candidate itself"
    );
}

fn assert_persisted_plan_stage(root: &Path, plan_count: usize) -> Result<()> {
    let persisted = read_json(&root.join("artifacts/plans/plans.json"))?;
    assert_eq!(
        persisted.as_array().map(Vec::len),
        Some(plan_count),
        "the persisted plan stage mirrors the returned plans"
    );
    let manifest = read_json(&root.join("artifacts/plans/manifest.json"))?;
    assert_eq!(
        manifest["total_plans"].as_u64(),
        Some(plan_count as u64),
        "the plan manifest counts the proposed plans: {manifest}"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 5. T8 acceptance (T8b fence lift): preview -> apply -> load win -> rollback.
//    Style of m22: every test owns a temp copy; the flow is the wired CLI
//    route (no injected hooks). Kickoff §5-T8.
// ---------------------------------------------------------------------------

/// The consumer's whole closure (semantic ∪ collateral, sorted ids) plus the
/// collateral load, fresh-analyzed from the tree as it currently stands.
fn closure_ids(root: &Path) -> Result<(Vec<String>, usize)> {
    let mut pipeline = Pipeline::new(&root.to_path_buf(), None)?;
    let (graph, closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;
    let analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);
    let metric = analyzer.analyze_symbol_load(DASHBOARD_ID)?;
    let mut ids: Vec<String> = metric
        .semantic_symbols
        .iter()
        .chain(metric.collateral_symbols.iter())
        .map(|id| id.0.clone())
        .collect();
    ids.sort();
    ids.dedup();
    Ok((ids, metric.collateral_load))
}

/// The exact bytes between two markers (end-exclusive), trimmed of trailing
/// whitespace — the verbatim-region oracle (D-M23-1).
fn region_between(text: &str, start_marker: &str, end_marker: &str) -> String {
    let start = text.find(start_marker).expect(start_marker);
    let end = text[start..]
        .find(end_marker)
        .map(|offset| start + offset)
        .expect(end_marker);
    text[start..end].trim_end().to_string()
}

/// Top-level declaration lines (a declaration-introducing keyword at the
/// line's first non-whitespace position) — the move-proof oracle (I5).
fn declaration_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| {
            let head = line.trim_start();
            ["class ", "object ", "trait ", "sealed ", "final ", "case class ", "abstract ", "package object "]
                .iter()
                .any(|prefix| head.starts_with(prefix))
        })
        .collect()
}

/// sha256(relpath ‖ bytes) per file, whole tree, sorted — the rollback
/// dir-hash identity digest.
fn tree_digest(root: &Path) -> Result<Vec<(String, String)>> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) -> Result<()> {
        let mut entries: Vec<PathBuf> = fs::read_dir(dir)?.map(|entry| entry.unwrap().path()).collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(&path, root, out)?;
            } else {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                let mut hasher = Sha256::new();
                hasher.update(rel.as_bytes());
                hasher.update(fs::read(&path)?);
                out.push((rel, format!("{:x}", hasher.finalize())));
            }
        }
        Ok(())
    }
    let mut digest = Vec::new();
    walk(root, root, &mut digest)?;
    Ok(digest)
}

/// The serde label of a `ValidationStatus` value ("Pass" | "Fail" | "Skipped").
fn status_label(value: &Value) -> String {
    if let Some(text) = value.as_str() {
        return text.to_string();
    }
    value
        .as_object()
        .and_then(|map| map.keys().next().cloned())
        .unwrap_or_default()
}

fn plan_json_string_array(value: &Value, field: &str) -> Vec<String> {
    value[field]
        .as_array()
        .unwrap_or_else(|| panic!("expected array field {field} in {value}"))
        .iter()
        .map(|entry| normalized_rel(Path::new(entry.as_str().unwrap_or_default())))
        .collect()
}

#[test]
fn apply_isolates_the_shared_file_and_sheds_the_consumer_collateral() -> Result<()> {
    let repo = isolated_scala_repo("full-apply")?;
    let shared_before = fs::read_to_string(repo.root.join(SHARED_REL))?;
    let (ids_before, load_before) = closure_ids(&repo.root)?;
    assert!(
        ids_before.iter().any(|id| id == PACKAGE_OBJECT_ID),
        "precondition: the package object rides in the consumer closure today"
    );

    let plans = plan_extractions(&repo.root)?;
    let plan = plans
        .iter()
        .find(|plan| normalized_rel(&plan.target_file) == SHARED_REL)
        .expect("the shared package-object file is the plan target");
    let plan_id = plan.id.0.clone();
    let shared_file_id = FileId::from_relative_path(SHARED_REL);
    let candidate_set: Vec<&_> = plan
        .proposed_extractions
        .iter()
        .filter(|ext| ext.source_file == Some(shared_file_id))
        .collect();
    assert!(
        !candidate_set.is_empty(),
        "the plan extracts the shared file's own symbols (D-M23-3 source_file provenance)"
    );

    build_migration_preview(&repo.root, &plan_id, false)?;
    assert_dashboard_preview_consumer(&repo.root)?;
    apply_transform_plan(&repo.root, &plan_id)?;
    assert_committed_offline_tiers(&repo.root)?;
    assert_moved_units_and_stripped_source(&repo.root, &shared_before, plan, &candidate_set)?;
    assert_consumer_narrowed_and_untouched_bytes(&repo.root)?;
    assert_the_load_win(&repo.root, &ids_before, load_before)?;
    Ok(())
}

/// T8b fence lift, preview stage: the import-scan consumer discovery reaches
/// Dashboard through the CLI route and redirects it at the moved units.
fn assert_dashboard_preview_consumer(root: &Path) -> Result<()> {
    let preview = read_json(&root.join("artifacts/migration/preview.json"))?;
    let consumer_rewrite = preview["rewrites"]
        .as_array()
        .expect("preview rewrites array")
        .iter()
        .find(|rw| {
            normalized_rel(Path::new(rw["consumer_file"].as_str().unwrap_or_default()))
                .ends_with(CONSUMER_REL)
        })
        .expect("T8b fence lift: the Scala import scan finds Dashboard as a consumer");
    let new_includes = plan_json_string_array(consumer_rewrite, "new_includes");
    assert!(
        new_includes
            .iter()
            .any(|path| path == &format!("{SHARED_DIR}/LegacyReport.scala")),
        "the rewrite redirects the consumer at the moved unit (new_includes: {new_includes:?})"
    );
    Ok(())
}

/// Kickoff §5-T8: offline apply commits with tiers [Pass, Pass, Skipped].
fn assert_committed_offline_tiers(root: &Path) -> Result<()> {
    let manifest = read_json(&root.join("artifacts/transaction/manifest.json"))?;
    assert_eq!(manifest["outcome"], "Committed", "offline apply commits (I7)");
    let outcome = read_json(&root.join("artifacts/transaction/outcome.json"))?;
    let statuses: Vec<String> = outcome["Committed"]["tiers_passed"]
        .as_array()
        .expect("committed tiers")
        .iter()
        .map(|tier| status_label(&tier["status"]))
        .collect();
    assert_eq!(
        statuses,
        vec!["Pass", "Pass", "Skipped"],
        "kickoff §5-T8: validation tiers [Pass, Pass, Skipped]"
    );
    Ok(())
}

/// Post-apply disk state: every planned unit exists at its in-place target
/// with package clause + symbol text (verbatim for the report, D-M23-1), the
/// sealed pair shares its batch unit, and the source file lost its moved
/// declarations while keeping the package object and its neighbours (I5).
fn assert_moved_units_and_stripped_source(
    root: &Path,
    shared_before: &str,
    plan: &ariadne_cluster::plan::OptimizationPlan,
    candidate_set: &[&ariadne_cluster::plan::ProposedExtraction],
) -> Result<()> {
    for ext in candidate_set {
        let extracted = fs::read_to_string(root.join(&ext.target_header))
            .unwrap_or_else(|_| panic!("extracted unit at {}", ext.target_header.display()));
        let simple = ext.symbol_name.strip_suffix('$').unwrap_or(&ext.symbol_name);
        assert!(
            extracted.contains(simple),
            "the extracted unit carries {}'s text",
            ext.symbol_name
        );
        assert!(
            extracted.contains("package com.example.legacy"),
            "the extracted unit keeps the package clause (D-M23-5)"
        );
    }
    let report_unit = fs::read_to_string(root.join(format!("{SHARED_DIR}/LegacyReport.scala")))?;
    let report_region = region_between(shared_before, "class LegacyReport {", "/** A heavyweight sibling");
    assert!(
        report_unit.contains(&report_region),
        "the extracted body is the VERBATIM source region (D-M23-1)"
    );
    assert!(
        report_unit.contains("com.example.legacy.records"),
        "the referenced import rides along (T5)"
    );
    assert!(
        !report_unit.contains("AuditSeal"),
        "referenced-only import selection (D-M23-13b)"
    );

    let sealed_target = plan
        .proposed_extractions
        .iter()
        .find(|ext| ext.symbol_id.0 == LEGACY_SHAPE_ID)
        .map(|ext| root.join(&ext.target_header))
        .expect("the sealed batch target");
    let sealed_unit = fs::read_to_string(&sealed_target)?;
    assert!(
        sealed_unit.contains("sealed trait LegacyShape") && sealed_unit.contains("LegacyCircle"),
        "D-M23-8: the sealed pair moved together into one unit"
    );

    let shared_after = fs::read_to_string(root.join(SHARED_REL))?;
    for ext in candidate_set {
        let simple = ext.symbol_name.strip_suffix('$').unwrap_or(&ext.symbol_name);
        assert!(
            !declaration_lines(&shared_after)
                .iter()
                .any(|line| line.contains(simple)),
            "I5: {simple} is no longer declared in the source file (move, not copy)"
        );
    }
    assert_eq!(
        declaration_lines(&shared_after),
        vec!["package object legacy {"],
        "the un-moved package object survives, byte-shaped as before"
    );
    assert!(
        shared_after.contains("package object legacy")
            && shared_after.contains("legacy isolation fixture"),
        "I5: neighbours of the moved regions stay intact"
    );
    Ok(())
}

/// The consumer's wildcard narrowed to the moved symbols (D-M23-5) while
/// its non-directive bytes and the untouched fixture files stay byte-exact.
fn assert_consumer_narrowed_and_untouched_bytes(root: &Path) -> Result<()> {
    let dashboard = fs::read_to_string(root.join(CONSUMER_REL))?;
    assert!(
        dashboard.contains("import com.example.legacy.LegacyReport"),
        "the wildcard narrowed to the moved symbols (D-M23-5)"
    );
    assert!(
        !dashboard.contains("import com.example.legacy._"),
        "the wildcard import is gone"
    );
    assert!(
        dashboard.contains("class Dashboard {")
            && dashboard.contains("def headline: String = new LegacyReport().title"),
        "non-directive consumer bytes stay byte-identical (I1 analogue)"
    );

    for rel in [
        "src/main/scala/com/example/legacy/records/AuditSeal.scala",
        "build.sbt",
        "project/build.properties",
    ] {
        assert_eq!(
            fs::read(root.join(rel))?,
            fs::read(fixture_path(FIXTURE).join(rel))?,
            "{rel} stays byte-identical"
        );
    }
    assert!(
        !root.join("generated").exists(),
        "D-M23-5: move targets land in the package dir, never a generated staging dir"
    );
    Ok(())
}

/// The point of the extraction: re-analyzing the applied tree takes the
/// non-extracted package object out of the consumer's closure.
fn assert_the_load_win(root: &Path, ids_before: &[String], load_before: usize) -> Result<()> {
    fs::remove_dir_all(root.join("artifacts"))?;
    let (ids_after, load_after) = closure_ids(root)?;
    assert!(
        !ids_after.iter().any(|id| id == PACKAGE_OBJECT_ID),
        "the load win: the non-extracted package object left the consumer closure"
    );
    assert!(
        ids_after.iter().any(|id| id == LEGACY_REPORT_ID),
        "the justified symbol survives the move"
    );
    assert!(
        load_after < load_before,
        "collateral shed: {load_before} -> {load_after}"
    );
    Ok(())
}

#[test]
fn transaction_rollback_restores_the_whole_fixture_tree() -> Result<()> {
    let repo = isolated_scala_repo("rollback")?;
    let plans = plan_extractions(&repo.root)?;
    let plan = plans
        .into_iter()
        .find(|plan| normalized_rel(&plan.target_file) == SHARED_REL)
        .expect("the shared package-object file is the plan target");

    let mut pipeline = Pipeline::new(&repo.root, None)?;
    let model = pipeline.get_or_build_reduced()?;
    let registry = default_registry();
    let source_reader = repository_source_reader(&pipeline.repo)?;
    let extractor =
        plan_backend(&registry, &plan.target_file, |caps| caps.extraction, "extraction")?
            .extractor();
    let rewriter =
        plan_backend(&registry, &plan.target_file, |caps| caps.rewriting, "rewriting")?
            .rewriter();
    let extraction = IsolationEngine::new(&model).extract(&plan, &source_reader, extractor);

    let moved_ids: Vec<String> = plan
        .proposed_extractions
        .iter()
        .map(|ext| ext.symbol_id.0.clone())
        .collect();
    let targets = rewriter.consumer_discovery_targets(&plan.target_file, &moved_ids);
    let all_names: Vec<String> = plan
        .proposed_extractions
        .iter()
        .map(|ext| ext.symbol_name.clone())
        .collect();
    let consumers = plan_target_consumers(
        &registry,
        &catalog_languages(&pipeline.discovery),
        &pipeline.build.translation_units,
        &repo.root,
        &targets,
        &plan.target_file,
        &all_names,
    );
    let (symbol_name_to_header, units) = plan_rewrite_targets(&plan);
    let migration =
        IncludeRewriter::new(symbol_name_to_header, units).build_preview(plan.id.0.clone(), &consumers);
    assert!(
        !migration.rewrites.is_empty(),
        "the neutral discovery finds consumers before apply"
    );

    let before = tree_digest(&repo.root)?;
    let (snapshot, new_paths) =
        TransactionApplier::apply_with(&repo.root, &extraction, &migration, rewriter, extractor)?;
    let during = tree_digest(&repo.root)?;
    assert_ne!(during, before, "apply mutates the tree (sanity)");

    TransactionApplier::rollback(&snapshot, &new_paths)?;
    assert_eq!(
        tree_digest(&repo.root)?,
        before,
        "rollback restores dir-hash identity across the WHOLE fixture tree"
    );
    assert!(TransactionApplier::verify_restored(&snapshot).is_ok());
    Ok(())
}
