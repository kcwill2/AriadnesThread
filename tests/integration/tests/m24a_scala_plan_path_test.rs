//! AR-19 (A1 part 2) acceptance lock: the Scala PLAN PATH through the
//! registry-dispatched planning hooks.
//!
//! The bug this file demonstrates (silent-empty/incorrect plans for Scala):
//! the two planning consumers outside `commands/plan.rs` -- the
//! `tool optimize` lane (`crates/cli/src/optimize.rs`) and the UI extraction
//! proposals (`crates/cli/src/commands/ui.rs::build_extraction_proposals`) --
//! passed the FREE C++ functions `cpp_aggregation_filter`/`cpp_target_path_for`
//! positionally into the cluster planner. The C++ aggregation filter is
//! extension-only, so no `.scala` file ever became an aggregation unit on
//! those lanes: a Scala repo silently yielded ZERO collateral opportunities,
//! ZERO safe plans, and NO extraction proposals, even though the very same
//! repo plans correctly through `plan_extractions` (m23's witness).
//!
//! After AR-19 both lanes resolve the plugin per aggregation unit through the
//! registry and consume the AR-18 trait hooks
//! (`aggregation_filter`/`target_path_for`), so both witnesses below are
//! GREEN: non-empty AND Scala-shaped (`{Symbol}.scala` targets beside the
//! package-object file, never the C++ `{symbol}_extracted.h` naming).
//!
//! Isolation: every test owns a tempdir copy of the `scala-isolation` fixture
//! (the m22/m23 harness pattern: exclude `artifacts/` and dot-dirs so no
//! stale stage output can ride in). The C++ byte-identity side of the switch
//! is NOT this file's job -- it stays locked UNEDITED by m00 (payload/plans
//! goldens), m18 (plugin parity), and m24 (language identity).

use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE: &str = "scala-isolation";

/// The shared package-object file -- the only Scala aggregation unit. The
/// proposal DTO renders the target as this basename (proposal_basename).
const SHARED_BASENAME: &str = "legacy.scala";
/// Where the Scala namer (not the C++ one) must land extracted symbols.
const LEGACY_REPORT_TARGET: &str = "src/main/scala/com/example/legacy/LegacyReport.scala";

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

/// Throwaway copy of the fixture WITHOUT `artifacts/` or dot-dirs (m22/m23
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

// ---------------------------------------------------------------------------
// 1. The `tool optimize` lane: the production EndToEndOptimizer must surface
//    the Scala aggregation unit's collateral and propose at least one safe
//    plan. Before AR-19 both were silent zero (the cpp free functions never
//    claim a `.scala` path), while `plan_extractions` on the same fixture
//    proposed the extraction -- the silent-empty split the handoff names.
// ---------------------------------------------------------------------------

#[test]
fn optimize_lane_proposes_the_scala_plan_path() -> Result<()> {
    let repo = isolated_scala_repo("optimize")?;
    let optimizer = ariadne_cli::optimize::EndToEndOptimizer::new(repo.root.clone());
    let report = optimizer.optimize_repository()?;

    let opportunity_names: Vec<String> = report
        .collateral_opportunities
        .iter()
        .map(|opportunity| opportunity.header_name.clone())
        .collect();
    assert!(
        report
            .collateral_opportunities
            .iter()
            .any(|opportunity| opportunity.header_name == "legacy.scala"
                && opportunity.collateral_symbols > 0),
        "A1 part 2: the optimize lane must resolve the aggregation filter \
         from the Scala plugin, so the package-object file's collateral \
         surfaces (the cpp filter extension-rejects `.scala`: silent-empty \
         witness). Opportunities seen: {opportunity_names:?}"
    );
    assert!(
        report.safe_plans_available >= 1,
        "A1 part 2: the optimize lane's planner must receive the Scala \
         target namer through the registry, so the same repo that plans \
         through `plan_extractions` (m23) plans here too -- zero safe plans \
         on a repo with a real Scala extraction is the bug"
    );

    let persisted = read_json(&repo.root.join("artifacts/optimize/report.json"))?;
    assert_eq!(
        persisted["safe_plans_available"].as_u64(),
        Some(report.safe_plans_available as u64),
        "the persisted optimize report agrees with the returned report"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. The UI extraction-proposals lane: the payload served at /api/export
//    must carry the Scala proposal -- non-empty AND Scala-named. Before the
//    switch the key was absent entirely (proposals skipped-when-empty).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ui_payload_carries_scala_extraction_proposals() -> Result<()> {
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
        // S6: keep the 127.0.0.1 bind default (no --expose), m22 pattern.
        let result =
            ariadne_cli::run_ui(Some(repo.root.clone()), port, true, None, 2, true, None).await;
        assert!(result.is_ok(), "run_ui server errored: {:?}", result.err());
    });

    let poll = poll_ready(&client, &base_url, &server).await;
    let outcome = match poll {
        Ok(()) => assert_scala_proposals(&client, &base_url).await,
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

async fn assert_scala_proposals(client: &reqwest::Client, base_url: &str) -> Result<()> {
    let response = client
        .get(format!("{base_url}/api/export"))
        .send()
        .await?;
    assert_eq!(response.status().as_u16(), 200, "/api/export must serve ready");
    let payload: Value = response.json().await?;

    let proposals = payload["extraction_proposals"]
        .as_array()
        .filter(|proposals| !proposals.is_empty())
        .unwrap_or_else(|| {
            panic!(
                "A1 part 2: the UI lane must plan through the registry, so a \
                 Scala repo with a real extraction gets proposals -- the cpp \
                 free functions never claim a `.scala` aggregation unit, so \
                 the key was silently ABSENT before the switch. Payload keys: \
                 {:?}",
                payload.as_object().map(|map| map.keys().collect::<Vec<_>>())
            )
        });

    let proposal_targets: Vec<String> = proposals
        .iter()
        .map(|proposal| proposal["target_file"].as_str().unwrap_or_default().to_string())
        .collect();
    // Display contract (proposal_basename): the DTO renders the aggregation
    // target as its BASENAME; the repo-relative full paths live in the
    // `target_files` rows asserted below.
    let shared = proposals
        .iter()
        .find(|proposal| proposal["target_file"].as_str() == Some(SHARED_BASENAME))
        .unwrap_or_else(|| {
            panic!(
                "the shared package-object file is the proposal target \
                 (proposal targets seen: {proposal_targets:?})"
            )
        });

    let target_paths: Vec<String> = shared["target_files"]
        .as_array()
        .expect("target-file rows (playbook shape C2)")
        .iter()
        .map(|row| normalized_rel(Path::new(row["path"].as_str().unwrap_or_default())))
        .collect();
    assert!(
        target_paths.iter().any(|path| path == LEGACY_REPORT_TARGET),
        "D-M23-5 naming must reach the UI lane: the Scala target namer puts \
         LegacyReport beside the shared file (targets seen: {target_paths:?})"
    );
    let cpp_naming: Vec<&String> = target_paths
        .iter()
        .filter(|path| path.contains("_extracted.h"))
        .collect();
    assert!(
        cpp_naming.is_empty(),
        "the C++ `{{symbol}}_extracted.h` namer must not be applied to Scala \
         candidates (misroute shape the registry dispatch retires): {cpp_naming:?}"
    );
    assert!(
        !shared["symbol_names"]
            .as_array()
            .expect("symbol names are present")
            .is_empty(),
        "the proposal names its extracted symbols"
    );
    Ok(())
}
