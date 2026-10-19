//! UTA2-R4 pin (docs/specs/unit-test-audit.md Round 2, handoff item 5):
//! the eight `load_*_checked` variants gate through ONE shared helper -
//! `check_manifest` - and share its exact fingerprint-mismatch verdict:
//! a `StaleArtifact` whose Display is `stale artifact for stage '<stage>':
//! input fingerprint mismatch`. Confirmed by grep (journal carries the
//! grep-lock note: every variant's first statement is
//! `self.check_manifest("<stage>", input_fingerprint)?`) and pinned here
//! per variant, which simultaneously locks each variant's STAGE directory
//! name (the string check_manifest reports) and the manifest-gates-first
//! order (a stage with NO payload blobs still answers fingerprint-mismatch,
//! never an IO error).

use super::tests::manager_in_fresh_tempdir;
use super::{is_stale, ArtifactManager};
use serde_json::json;
use std::fs;
use tempfile::tempdir;

const STORED_FP: &str = "stored-manifest-fp";
const EXPECTED_FP: &str = "caller-expected-fp";

/// Drop a fingerprint-bearing manifest.json into a synthetic stage dir;
/// `check_manifest` reads only that file, so the variants need no payload
/// blobs to exercise the gate (the round-1 t6 plant shape).
fn plant_manifest(mgr: &ArtifactManager, stage: &str) {
    let stage_dir = mgr.artifact_root.join(stage);
    fs::create_dir_all(&stage_dir).unwrap();
    fs::write(
        stage_dir.join("manifest.json"),
        json!({
            "input_fingerprint": STORED_FP,
            "analysis_version": ariadne_index::CURRENT_ANALYSIS_VERSION,
        })
        .to_string(),
    )
    .unwrap();
}

#[test]
fn pin_load_checked_variants_share_the_fingerprint_mismatch_verdict() {
    let dir = tempdir().unwrap();
    let mgr = manager_in_fresh_tempdir(&dir);
    let stages = [
        "repository",
        "contexts",
        "index",
        "includes",
        "reduced",
        "callgraph",
        "load",
        "plans",
    ];
    for stage in stages {
        plant_manifest(&mgr, stage);
    }

    // `includes` is the only variant needing extra caller input (the current
    // discovery); an empty one suffices - the manifest gate must answer
    // before any payload read could need it.
    let discovery = ariadne_repo::RepositoryDiscoveryResult {
        root: mgr.repo_root(),
        files: vec![],
        ownership_summary: Default::default(),
        policy_summary: Default::default(),
    };

    let drivers: Vec<(&str, Box<dyn Fn(&ArtifactManager) -> anyhow::Result<()>>)> = vec![
        (
            "repository",
            Box::new(|m| m.load_repository_checked(EXPECTED_FP).map(|_| ())),
        ),
        (
            "contexts",
            Box::new(|m| m.load_build_context_checked(EXPECTED_FP).map(|_| ())),
        ),
        (
            "index",
            Box::new(|m| m.load_index_checked(EXPECTED_FP).map(|_| ())),
        ),
        (
            "includes",
            Box::new(|m| {
                m.load_include_checked(EXPECTED_FP, &discovery)
                    .map(|_| ())
            }),
        ),
        (
            "reduced",
            Box::new(|m| m.load_reduced_checked(EXPECTED_FP).map(|_| ())),
        ),
        (
            "callgraph",
            Box::new(|m| m.load_callgraph_checked(EXPECTED_FP).map(|_| ())),
        ),
        (
            "load",
            Box::new(|m| m.load_load_checked(EXPECTED_FP).map(|_| ())),
        ),
        (
            "plans",
            Box::new(|m| m.load_plan_checked(EXPECTED_FP).map(|_| ())),
        ),
    ];
    assert_eq!(drivers.len(), stages.len(), "all eight variants covered");

    for (stage, drive) in drivers {
        let err = drive(&mgr).expect_err("a mismatched fingerprint must never load");
        assert!(
            is_stale(&err),
            "variant {stage}: mismatch must be a StaleArtifact (rebuild), not a hard error: {err}"
        );
        assert_eq!(
            err.to_string(),
            format!("stale artifact for stage '{stage}': input fingerprint mismatch"),
            "variant {stage}: exact shared-helper verdict, including the stage name"
        );
    }
}
