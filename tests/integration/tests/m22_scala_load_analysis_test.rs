//! M22.3 acceptance suite (kickoff acceptance #1–#5, plan:428): the Scala
//! load/explain/plan surface on a temp copy of the evolved scala-basic
//! fixture (D27 wildcard import). The headline witness is
//! `load_reports_collateral_from_package_object` — the Scala analogue of
//! the C++ "expensive" collateral story: `OrderService`'s closure carries
//! the package-object file `models/all.scala` as its collateral.
//!
//! Isolation: every test owns a tempdir copy; the copy EXCLUDES any
//! `artifacts/` directory and dot-dirs (the m21 kickoff flagged that the
//! plain copy_tree could inherit stale stage artifacts — this lane builds
//! everything itself).
//!
//! Wording note: the CLI handlers print stdout, which an in-process test
//! cannot capture; every wording assertion runs the EXACT formatter the
//! handler calls (`format_summary_with` / `format_explanation_with` fed by
//! the registry vocabulary) plus the targeted-handler Ok lane. The caveat
//! LINE format is locked by the cli `run_vocabulary` unit tests (M22.2b).

use anyhow::Result;
use ariadne_cli::registry::default_registry;
use ariadne_cli::{analyze_dependency_load, plan_extractions};
use ariadne_cluster::planner::{OptimizationPlanner, PlanningGap};
use ariadne_cluster::plan::RejectionReason;
use ariadne_graph::{build_dependency_graph, LoadAnalyzer};
use ariadne_lang::{CyclePolicy, DependencyVocabulary, LanguageId};
use ariadne_pipeline::Pipeline;
use ariadne_repo::SourceLanguage;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE: &str = "scala-basic";

const ORDER_SERVICE_REL: &str = "src/main/scala/com/example/services/OrderService.scala";

const ORDER_SERVICE_ID: &str = "com.example.services.OrderService";
const PACKAGE_OBJECT_ID: &str = "com.example.models";
const USER_ID: &str = "com.example.models.User";
const ORDER_ID: &str = "com.example.models.Order";

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

/// Throwaway copy of the fixture WITHOUT `artifacts/` or dot-dirs (see the
/// module note — this is the m21 kickoff's copy_tree caveat, fixed here).
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

// ---------------------------------------------------------------------------
// Registry vocabulary helpers — the exact data the CLI handlers feed their
// *_with formatters.
// ---------------------------------------------------------------------------

fn scala_vocabulary() -> DependencyVocabulary {
    let registry = default_registry();
    let plugin = registry
        .plugin_for_id(SourceLanguage::Scala)
        .expect("Scala plugin is registered");
    plugin.modeling().dependency_vocabulary
}

fn scala_caveat() -> String {
    let registry = default_registry();
    registry
        .plugin_for_id(SourceLanguage::Scala)
        .expect("Scala plugin is registered")
        .modeling()
        .analysis_caveat
        .expect("D26: the Scala analysis output is labelled heuristic")
}

fn read_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)?;
    let parsed = serde_json::from_str(&text)?;
    Ok(parsed)
}

// ---------------------------------------------------------------------------
// 1. THE acceptance witness (plan:428, kickoff acceptance #1): collateral
//    load on the plain D22 auto-shaped lane, largest source models/all.scala.
// ---------------------------------------------------------------------------

#[test]
fn load_reports_collateral_from_package_object() -> Result<()> {
    let repo = isolated_scala_repo("collateral")?;

    // Plain Pipeline::new — no --lang, no injected shaper (D22 makes the
    // sbt markers select the Scala shaper).
    let mut pipeline = Pipeline::new(&repo.root, None)?;
    let (graph, closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;

    let analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);
    let metric = analyzer.analyze_symbol_load(ORDER_SERVICE_ID)?;

    assert!(
        metric.collateral_load > 0,
        "plan:428: the wildcard import drags the package object into OrderService's \
         closure, so collateral load must be non-zero (metric: {:?})",
        metric.collateral_sources.iter().map(|s| (normalized_rel(&s.path), s.symbol_count)).collect::<Vec<_>>()
    );

    let largest = metric
        .largest_collateral_source
        .expect("a collateral source exists above");
    assert_eq!(
        normalized_rel(&largest.path),
        "src/main/scala/com/example/models/all.scala",
        "kickoff acceptance #1: the largest collateral source is the package-object file"
    );
    assert_eq!(
        largest.symbol_count, 1,
        "all.scala contributes exactly the package object"
    );
    let collateral_ids: Vec<String> = metric
        .collateral_symbols
        .iter()
        .map(|id| id.0.clone())
        .collect();
    assert!(
        collateral_ids.iter().any(|id| id == PACKAGE_OBJECT_ID),
        "the collateral set contains the package object, got {collateral_ids:?}"
    );

    // The used symbols must stay SEMANTIC — collateral is the unused side.
    for semantic_id in [USER_ID, ORDER_ID] {
        assert!(
            metric
                .semantic_symbols
                .iter()
                .any(|id| id.0 == semantic_id),
            "{semantic_id} is used by OrderService and must not be collateral"
        );
    }

    // The handler lane itself (targeted => never persists, m6's guard) must
    // run clean through the auto-shaped pipeline.
    analyze_dependency_load(&repo.root, Some(ORDER_SERVICE_ID), None)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. explain renders the import chain with D26 vocabulary + caveat (plan:428
//    acceptance #2).
// ---------------------------------------------------------------------------

#[test]
fn explain_renders_import_chain() -> Result<()> {
    let repo = isolated_scala_repo("explain")?;
    let mut pipeline = Pipeline::new(&repo.root, None)?;
    let (graph, closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;

    let analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);
    let metric = analyzer.analyze_symbol_load(ORDER_SERVICE_ID)?;
    let explanation = analyzer.explain_symbol_load(ORDER_SERVICE_ID, None)?;

    let vocabulary = scala_vocabulary();
    let summary = metric.format_summary_with(&vocabulary);
    let formatted = explanation.format_explanation_with(&vocabulary);

    assert!(summary.contains("Import load:"), "D26 load wording: {summary}");
    assert!(!summary.contains("Include load"), "C++ wording absent: {summary}");

    assert!(formatted.contains("↓"), "chain arrows rendered: {formatted}");
    assert!(
        formatted.contains("all.scala"),
        "the explained chain names the package-object file: {formatted}"
    );
    assert!(
        formatted.contains("is imported by"),
        "import-flavored provenance verb: {formatted}"
    );
    assert!(
        !formatted.contains("included"),
        "C++-flavored wording absent from the Scala render: {formatted}"
    );
    assert!(
        !formatted.contains("header"),
        "no header wording in the Scala render: {formatted}"
    );

    // D26 caveat: the registry hands the CLI the heuristic label every Scala
    // load/explain print leads with (line format locked by cli unit tests).
    let caveat = scala_caveat();
    assert!(caveat.contains("heuristic"), "caveat is honest: {caveat}");
    assert!(
        caveat.contains("Scala toolchain"),
        "caveat points at the exact-types remedy: {caveat}"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. plan prints the capability-gated rejection and invents nothing
//    (plan:428 acceptance #3, D23).
// ---------------------------------------------------------------------------

#[test]
fn plan_prints_capability_rejection() -> Result<()> {
    let repo = isolated_scala_repo("plan")?;

    // The CLI lane: exit Ok, no plans, and the persisted stage is empty —
    // no fake header is invented for Scala.
    let plans = plan_extractions(&repo.root)?;
    assert!(plans.is_empty(), "D4/D23: Scala never gets fabricated plans");
    let plans_json = read_json(&repo.root.join("artifacts/plans/plans.json"))?;
    assert_eq!(
        plans_json,
        Value::Array(vec![]),
        "the persisted plan stage is an empty array, not a fabricated entry"
    );
    let manifest = read_json(&repo.root.join("artifacts/plans/manifest.json"))?;
    assert_eq!(
        manifest["total_plans"].as_u64(),
        Some(0),
        "the plan manifest counts zero plans: {manifest}"
    );

    // The rejection stdout line is `Planning rejected: {explanation}` from
    // this same planner lane (stdout is uncapturable in-process, so the
    // wording is asserted at its source — the CLI adds only the prefix).
    let mut pipeline = Pipeline::new(&repo.root, None)?;
    let (graph, _closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;
    let tu_file_ids: Vec<_> = pipeline
        .build
        .translation_units
        .iter()
        .map(|tu| tu.file_id)
        .collect();
    let registry = default_registry();
    let mut gaps: Vec<PlanningGap> = Vec::new();
    for record in &pipeline.discovery.files {
        if gaps.iter().any(|gap| gap.language == record.language) {
            continue;
        }
        let Some(plugin) = registry.plugin_for_id(record.language) else {
            continue;
        };
        if plugin.capabilities().planning {
            continue;
        }
        gaps.push(PlanningGap {
            language: record.language,
            sample_path: record.path.clone(),
        });
    }
    let scala_gap = gaps
        .iter()
        .find(|gap| gap.language == SourceLanguage::Scala)
        .expect("the discovered Scala records form one planning gap");
    assert!(
        normalized_rel(&scala_gap.sample_path).ends_with(".scala"),
        "the gap carries an honest discovered Scala path as its target_file sample: {:?}",
        scala_gap.sample_path
    );

    let mut planner = OptimizationPlanner::new(
        &graph,
        &model,
        &pipeline.discovery.files,
        &tu_file_ids,
        &ariadne_lang_cpp::cpp_aggregation_filter,
        &ariadne_lang_cpp::cpp_target_path_for,
        None,
        ariadne_lang::CyclePolicy::Reject,
    );
    planner.with_planning_gaps(gaps);
    let (planner_plans, rejections) = planner.plan_with_rejections();
    assert!(planner_plans.is_empty());
    assert_eq!(
        rejections.len(),
        1,
        "exactly one explanatory rejection for the repository-present Scala"
    );
    assert_eq!(
        rejections[0].reason,
        RejectionReason::PlanningUnsupported(LanguageId::Scala),
        "D23: the reason names the language by data"
    );
    assert!(
        rejections[0].explanation.contains("Scala"),
        "the printed explanation names Scala: {}",
        rejections[0].explanation
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. D27 UI surface (kickoff D27, optional smoke): live `run_ui` server on
//    the Scala copy — file nodes carry language/unit_kind, the symbol
//    node's defining_file is a resolved repo-relative path, not a numeric
//    FileId string (the §3.6.1 defect).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ui_nodes_carry_language_unit_kind_and_resolved_defining_file() -> Result<()> {
    use std::net::TcpListener;

    let repo = isolated_scala_repo("ui")?;
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
        let port = listener.local_addr().expect("read ephemeral port").port();
        drop(listener);
        port
    };
    let base_url = format!("http://127.0.0.1:{port}");
    let client = reqwest::Client::new();

    let server = tokio::spawn(async move {
        let result = ariadne_cli::run_ui(Some(repo.root.clone()), port, true, None, 2, true).await;
        assert!(result.is_ok(), "run_ui server errored: {:?}", result.err());
        // `repo` (tempdir) lives as long as this future: aborting the task
        // drops it, keeping the tree alive exactly while the server runs.
    });

    let poll = poll_ready(&client, &base_url, &server).await;
    let outcome = match poll {
        Ok(()) => assert_ui_d27_surface(&client, &base_url).await,
        Err(error) => {
            server.abort();
            return Err(error);
        }
    };
    server.abort();
    outcome
}

async fn poll_ready(
    client: &reqwest::Client,
    base_url: &str,
    server: &tokio::task::JoinHandle<()>,
) -> Result<()> {
    for _ in 0..300 {
        if let Ok(snapshot) = client.get(format!("{base_url}/api/status")).send().await {
            if let Ok(value) = snapshot.json::<Value>().await {
                if value["progress"]["state"].as_str() == Some("ready") {
                    return Ok(());
                }
            }
        }
        if server.is_finished() {
            return Err(anyhow::anyhow!(
                "run_ui server task exited before reporting ready"
            ));
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    Err(anyhow::anyhow!("run_ui did not reach ready within 30s"))
}

async fn fetch_elements(client: &reqwest::Client, route: &str) -> Result<Vec<Value>> {
    let response = client.get(route).send().await?;
    assert_eq!(response.status().as_u16(), 200, "{route} must serve once ready");
    let elements: Vec<Value> = response.json().await?;
    Ok(elements)
}

fn find_node<'a>(elements: &'a [Value], key: &str, value: &str) -> Option<&'a Value> {
    elements.iter().find(|element| {
        element
            .get("data")
            .and_then(|data| data.get(key))
            .and_then(|field| field.as_str())
            == Some(value)
    })
}

async fn assert_ui_d27_surface(client: &reqwest::Client, base_url: &str) -> Result<()> {
    let files = fetch_elements(client, &format!("{base_url}/api/graph/include")).await?;
    let service_file = find_node(&files, "path", ORDER_SERVICE_REL)
        .expect("the OrderService.scala file node is present");
    assert_eq!(
        service_file["data"]["language"].as_str(),
        Some("Scala"),
        "D27: file nodes carry the language"
    );
    assert_eq!(
        service_file["data"]["unit_kind"].as_str(),
        Some("SourceFile"),
        "D27: file nodes carry the plugin modeling unit_kind"
    );

    let symbols = fetch_elements(client, &format!("{base_url}/api/graph/symbol")).await?;
    let service_symbol = find_node(&symbols, "id", ORDER_SERVICE_ID)
        .expect("the OrderService symbol node is present");
    let defining = service_symbol["data"]["defining_file"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert_eq!(
        normalized_rel(Path::new(&defining)),
        ORDER_SERVICE_REL,
        "§3.6.1: defining_file is the resolved repo-relative path"
    );
    assert!(
        !defining.chars().all(|c| c.is_ascii_digit()),
        "the numeric-FileId-string defect must stay fixed, got {defining}"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 5. T-D witness: the load manifest written by the CLI analyze lane carries
//    the stage languages (Scala positive; the empty/C++ byte-identity side
//    is witnessed by the m00 goldens at the workspace gate).
// ---------------------------------------------------------------------------

#[test]
fn load_manifest_carries_scala_language() -> Result<()> {
    let repo = isolated_scala_repo("manifest")?;

    analyze_dependency_load(&repo.root, None, None)?;

    let manifest = read_json(&repo.root.join("artifacts/load/manifest.json"))?;
    assert_eq!(
        manifest["languages"],
        Value::Array(vec![Value::String("Scala".to_string())]),
        "D25 + T-D: a Scala run's load manifest names its language (C++ runs \
         keep the key absent — m00 goldens)"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 6. Replacement for the dropped cpp_regression_witnesses slot: the models
//    <-> services cycle survives the PLAIN D22 auto-shaped graph lane and
//    stays informational under the M20 tolerate lane (m20 locked this on the
//    injected-shaper seam; this locks the auto-selected twin).
// ---------------------------------------------------------------------------

#[test]
fn auto_shaped_graph_keeps_the_cycle_informational() -> Result<()> {
    let repo = isolated_scala_repo("cycle")?;
    let registry = default_registry();
    let graph = build_dependency_graph(&repo.root, &registry)?;

    let cycles = graph.detect_cycles();
    assert!(
        !cycles.is_empty(),
        "the deliberate fixture cycle is present on the auto-shaped lane"
    );
    let tolerated =
        graph.generate_report_with_cycle_policies(&|_file_id| CyclePolicy::TolerateWithinPackage);
    assert_eq!(
        tolerated.informational_cycle_count, tolerated.cycle_count,
        "M20 lane: an all-tolerate stance keeps every cycle informational"
    );
    assert!(tolerated.informational_cycle_count > 0);
    Ok(())
}
