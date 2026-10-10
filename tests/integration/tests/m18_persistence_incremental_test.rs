use std::fs;
use tempfile::TempDir;

use ariadne_pipeline::{Pipeline, StageReuse};
use ariadne_repo::FileId;

const STAGES: [&str; 5] = [
    ariadne_pipeline::STAGE_INCLUDE_GRAPH,
    ariadne_pipeline::STAGE_INDEX,
    ariadne_pipeline::STAGE_REDUCED,
    ariadne_pipeline::STAGE_CALLGRAPH,
    ariadne_pipeline::STAGE_LOAD,
];

fn setup_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("include")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("include/common.h"), "struct Common { int id; };").unwrap();
    fs::write(
        root.join("include/header_a.h"),
        "#include \"common.h\"\nstruct ThingA { Common c; };",
    )
    .unwrap();
    fs::write(
        root.join("include/header_b.h"),
        "struct ThingB { double x; };",
    )
    .unwrap();
    fs::write(
        root.join("src/tu1.cpp"),
        "#include \"header_a.h\"\nvoid use_a(ThingA a) {}",
    )
    .unwrap();
    fs::write(
        root.join("src/tu2.cpp"),
        "#include \"header_b.h\"\nvoid use_b(ThingB b) {}",
    )
    .unwrap();

    let root_str = root.to_string_lossy().replace('\\', "/");
    let compile_commands = format!(
        r#"[
    {{ "directory": "{root_str}", "command": "clang++ -Iinclude -c src/tu1.cpp", "file": "src/tu1.cpp" }},
    {{ "directory": "{root_str}", "command": "clang++ -Iinclude -c src/tu2.cpp", "file": "src/tu2.cpp" }}
]"#,
        root_str = root_str
    );
    fs::write(root.join("compile_commands.json"), compile_commands).unwrap();
    dir
}

fn ensure_all(pipeline: &mut Pipeline) {
    pipeline.ensure_index().unwrap();
    pipeline.ensure_include_graph().unwrap();
    pipeline.ensure_reduced().unwrap();
    pipeline.ensure_callgraph().unwrap();
    pipeline.ensure_load().unwrap();
}

#[test]
fn test_m18_cold_run_persists_every_stage() {
    let dir = setup_repo();
    let root = dir.path();

    let mut cold = Pipeline::new(root, None).unwrap();
    ensure_all(&mut cold);

    for stage in STAGES {
        assert_eq!(
            cold.stats.stage_reuse(stage),
            Some(StageReuse::Rebuilt),
            "stage {} should rebuild cold",
            stage
        );
    }
    assert_eq!(cold.stats.reanalyzed_tus, 2);
    assert_eq!(cold.stats.reused_tus, 0);

    let includes = root.join("artifacts").join("includes");
    assert!(includes.join("edges.bin").exists());
    assert!(includes.join("closures.bin").exists());
    assert!(includes.join("manifest.json").exists());
    assert!(root.join("artifacts/index/manifest.json").exists());
    assert!(root.join("artifacts/reduced/manifest.json").exists());
    assert!(root.join("artifacts/callgraph/manifest.json").exists());
    assert!(root.join("artifacts/load/manifest.json").exists());

    let shards = root.join("artifacts").join("shards");
    assert!(shards.exists(), "shard store should persist");
}

#[test]
fn test_m18_warm_rerun_loads_all_stages_with_zero_parses() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let mut cold = Pipeline::new(&root, None).unwrap();
    ensure_all(&mut cold);

    let mut warm = Pipeline::new(&root, None).unwrap();
    ensure_all(&mut warm);

    for stage in STAGES {
        assert_eq!(
            warm.stats.stage_reuse(stage),
            Some(StageReuse::Loaded),
            "stage {} should load warm",
            stage
        );
    }
    assert_eq!(warm.stats.reanalyzed_tus, 0, "zero re-parses when warm");
    assert_eq!(
        warm.stats.reused_tus, 0,
        "loaded index means no per-TU work"
    );
}

#[test]
fn test_m18_header_edit_reparses_only_dependent_tu() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let mut cold = Pipeline::new(&root, None).unwrap();
    ensure_all(&mut cold);

    // tu1 includes header_a.h -> common.h; tu2 does not.
    fs::write(
        root.join("include/common.h"),
        "struct Common { int id; int extra; };",
    )
    .unwrap();

    let mut next = Pipeline::new(&root, None).unwrap();
    ensure_all(&mut next);

    assert_eq!(next.stats.reanalyzed_tus, 1, "only tu1 should re-parse");
    assert_eq!(next.stats.reused_tus, 1, "tu2 shard stays valid");
    assert_eq!(
        next.stats.stage_reuse(ariadne_pipeline::STAGE_INDEX),
        Some(StageReuse::Rebuilt)
    );
    assert_eq!(
        next.stats
            .stage_reuse(ariadne_pipeline::STAGE_INCLUDE_GRAPH),
        Some(StageReuse::Rebuilt)
    );
    assert_eq!(
        next.stats.stage_reuse(ariadne_pipeline::STAGE_REDUCED),
        Some(StageReuse::Rebuilt)
    );

    // A further no-change run is warm again for every stage.
    let mut settled = Pipeline::new(&root, None).unwrap();
    ensure_all(&mut settled);
    for stage in STAGES {
        assert_eq!(settled.stats.stage_reuse(stage), Some(StageReuse::Loaded));
    }
}

#[test]
fn test_m18_tu_body_edit_reparses_one_tu() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let mut cold = Pipeline::new(&root, None).unwrap();
    ensure_all(&mut cold);

    fs::write(
        root.join("src/tu2.cpp"),
        "#include \"header_b.h\"\nvoid use_b(ThingB b) {}\nvoid extra(ThingB b) { (void)b; }",
    )
    .unwrap();

    let mut next = Pipeline::new(&root, None).unwrap();
    ensure_all(&mut next);
    assert_eq!(next.stats.reanalyzed_tus, 1);
    assert_eq!(next.stats.reused_tus, 1);
    assert_eq!(
        next.stats.stage_reuse(ariadne_pipeline::STAGE_INDEX),
        Some(StageReuse::Rebuilt)
    );
}

#[test]
fn test_m18_old_plain_shard_is_stale_and_rewritten() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let mut cold = Pipeline::new(&root, None).unwrap();
    ensure_all(&mut cold);
    let index = cold.get_or_build_index().unwrap();

    let tu1_id = FileId::from_relative_path("src/tu1.cpp");
    let analysis = index
        .tu_analyses
        .iter()
        .find(|a| a.tu_file_id == tu1_id)
        .expect("tu1 analysis present")
        .clone();
    fs::write(
        root.join("artifacts")
            .join("shards")
            .join(format!("{}.json", tu1_id.0)),
        serde_json::to_string(&analysis).unwrap(),
    )
    .unwrap();
    // Force the index stage itself stale so the rebuild consults the store.
    fs::remove_dir_all(root.join("artifacts").join("index")).unwrap();

    // Plain pre-envelope shards must deserialize-fail into a miss, not trust.
    let mut next = Pipeline::new(&root, None).unwrap();
    next.ensure_index().unwrap();
    assert_eq!(
        next.stats.stage_reuse(ariadne_pipeline::STAGE_INDEX),
        Some(StageReuse::Rebuilt)
    );
    assert_eq!(
        next.stats.reanalyzed_tus, 1,
        "only the corrupted TU re-parses"
    );
    assert_eq!(next.stats.reused_tus, 1, "intact shard still loads");

    // The reanalyzed TU was rewritten as a fingerprinted envelope.
    let envelope: ariadne_scheduler::tu_store::ShardCheckpoint = serde_json::from_str(
        &fs::read_to_string(
            root.join("artifacts")
                .join("shards")
                .join(format!("{}.json", tu1_id.0)),
        )
        .unwrap(),
    )
    .expect("shard must now be an envelope");
    assert!(!envelope.fingerprint.is_empty());
    let mut again = Pipeline::new(&root, None).unwrap();
    again.ensure_index().unwrap();
    assert_eq!(
        again.stats.stage_reuse(ariadne_pipeline::STAGE_INDEX),
        Some(StageReuse::Loaded)
    );
}

#[test]
fn test_m18_corrupt_manifest_fingerprint_forces_rebuild() {
    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    let mut cold = Pipeline::new(&root, None).unwrap();
    ensure_all(&mut cold);

    let manifest_path = root
        .join("artifacts")
        .join("includes")
        .join("manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    let mut corrupt = manifest.as_object().unwrap().clone();
    corrupt.insert(
        "input_fingerprint".into(),
        serde_json::Value::String("0".repeat(64)),
    );
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&corrupt).unwrap(),
    )
    .unwrap();

    let mut next = Pipeline::new(&root, None).unwrap();
    ensure_all(&mut next);
    assert_eq!(
        next.stats
            .stage_reuse(ariadne_pipeline::STAGE_INCLUDE_GRAPH),
        Some(StageReuse::Rebuilt),
        "stale-fingerprint graph must not be trusted"
    );
    assert_eq!(
        next.stats.stage_reuse(ariadne_pipeline::STAGE_INDEX),
        Some(StageReuse::Loaded),
        "unrelated stages stay fresh"
    );

    let mut warm = Pipeline::new(&root, None).unwrap();
    warm.ensure_include_graph().unwrap();
    assert_eq!(
        warm.stats
            .stage_reuse(ariadne_pipeline::STAGE_INCLUDE_GRAPH),
        Some(StageReuse::Loaded),
        "rebuilt artifact is trusted afterwards"
    );
}

#[tokio::test]
async fn test_m18_ui_warm_start_reports_all_stages_loaded() {
    use std::net::TcpListener;

    let dir = setup_repo();
    let root = dir.path().to_path_buf();

    // Pre-warm: the first pipeline builds (cold) and persists every stage
    // artifact, so the server boot can load them all.
    let mut prewarm = Pipeline::new(&root, None).unwrap();
    ensure_all(&mut prewarm);
    for stage in STAGES {
        assert_eq!(prewarm.stats.stage_reuse(stage), Some(StageReuse::Rebuilt));
    }

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let repo_dir = root.clone();
    let server_task = tokio::spawn(async move {
        // S6: keep the 127.0.0.1 bind default (no --expose).
        let _ = ariadne_cli::run_ui(Some(repo_dir), port, true, None, 2, true, None).await;
    });

    // Poll-first readiness (mirrors m21_run_ui_smoke poll_until_ready):
    // probe immediately, only sleep BETWEEN attempts. Same 120 x 250 ms
    // budget and the same terminal ready assertions below.
    let base = format!("http://127.0.0.1:{}", port);
    let mut status = serde_json::Value::Null;
    for _ in 0..120 {
        let Ok(resp) = reqwest::get(format!("{}/api/status", base)).await else {
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            continue;
        };
        if !resp.status().is_success() {
            tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
            continue;
        }
        status = resp.json().await.unwrap_or(serde_json::Value::Null);
        if status["progress"]["state"].as_str() == Some("ready") {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
    }

    assert_eq!(status["progress"]["state"].as_str(), Some("ready"));
    assert!(status["last_refresh"].as_u64().unwrap_or(0) > 0);

    let stages = status["stages"]
        .as_array()
        .expect("status should carry stages");
    assert_eq!(stages.len(), STAGES.len());
    for entry in stages {
        assert_eq!(
            entry["reuse"].as_str(),
            Some("loaded"),
            "warm UI start should load every stage: {:?}",
            entry
        );
    }

    server_task.abort();
}
