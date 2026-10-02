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
use ariadne_cli::registry::default_registry;
use ariadne_cli::plan_extractions;
use ariadne_graph::{build_dependency_graph, LoadAnalyzer};
use ariadne_index::DependencyKind;
use ariadne_lang_scala::ScalaUnitShaper;
use ariadne_pipeline::Pipeline;
use ariadne_repo::{FileId, FileKind, FileOwnership, OptimizationPolicy, Repository, SourceLanguage};
use serde_json::Value;
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
    // TODO(M23 S4-landing): assert `SymbolRecord.modifiers` carries `sealed`
    // on LegacyShape. NOT assertable at this gate HEAD (b3a9958): the field
    // does not exist in ariadne-lang-api yet; the T7-S4 successor (typed
    // extends channel + modifiers, orchestrator ruling board_cef7c44)
    // enables this assertion without touching the rest of this suite.
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
        .any(|record| match registry.plugin_for_id(record.language) {
            Some(plugin) => !plugin.capabilities().planning,
            None => false,
        });
    assert!(
        !any_planning_gap,
        "D-M23-6: no PlanningUnsupported(Scala) rejection can exist post-flip"
    );

    assert_persisted_plan_stage(&repo.root, plans.len())?;
    // TODO(M23 S4-landing): assert the sealed pair (LegacyShape+LegacyCircle)
    // lands in ONE batch/target once the typed extends channel feeds
    // colocated_groups_with_edges; today (HEAD b3a9958) they are separate
    // singleton batches, which the assertions above deliberately do not lock.
    Ok(())
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
