use anyhow::Result;
use ariadne_cli::{
    apply_transform_plan, build_migration_preview, build_source_index, plan_extractions,
};
use ariadne_cluster::plan::OptimizationPlan;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn provision_indexed_repo_with_plans() -> Result<(TempDir, PathBuf, Vec<OptimizationPlan>)> {
    let guard = tempfile::tempdir()?;
    let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/basic");
    let repo_root = guard.path().join("basic");
    copy_dir_all(&fixture_root, &repo_root)?;
    build_source_index(&repo_root)?;
    let plans = plan_extractions(&repo_root)?;
    Ok((guard, repo_root, plans))
}

fn plan_id_targeting_thing_header(plans: &[OptimizationPlan]) -> String {
    plans
        .iter()
        .find(|plan| {
            plan.target_file
                .to_string_lossy()
                .replace('\\', "/")
                .ends_with("include/thing.h")
        })
        .expect("fresh plan set must contain the include/thing.h plan")
        .id
        .0
        .clone()
}

fn list_relative_files(root: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                names.push(
                    path.strip_prefix(root)?
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
    }
    names.sort();
    Ok(names)
}

fn expected_thing_plan_new_includes() -> Vec<PathBuf> {
    [
        "include\\ThingConfig_extracted.h",
        "include\\Thing_Thing_extracted.h",
        "include\\Thing_extracted.h",
        "include\\Thing_get_id_extracted.h",
    ]
    .iter()
    .map(PathBuf::from)
    .collect()
}

#[test]
fn migrate_preview_writes_migration_artifacts_without_disk_changes() -> Result<()> {
    let (_repo_guard, repo_root, plans) = provision_indexed_repo_with_plans()?;
    let plan_id = plan_id_targeting_thing_header(&plans);
    assert_eq!(
        plan_id, "PLAN-001",
        "thing.h plan id is stable across fresh runs"
    );

    let consumer_before = fs::read_to_string(repo_root.join("tests").join("consumer.cpp"))?;
    let thing_impl_before = fs::read_to_string(repo_root.join("src").join("thing.cpp"))?;

    build_migration_preview(&repo_root, &plan_id, true)?;

    let migration_dir = repo_root.join("artifacts").join("migration");
    assert!(migration_dir.join("preview.bin").exists());
    assert!(migration_dir.join("manifest.json").exists());

    let preview_json = fs::read_to_string(migration_dir.join("preview.json"))?;
    let preview: ariadne_transform::MigrationPreview = serde_json::from_str(&preview_json)?;
    assert_eq!(preview.plan_id, plan_id);
    assert_eq!(preview.rewrites.len(), 2);
    assert_eq!(preview.effective_rewrites(), 2);
    for rewrite in &preview.rewrites {
        assert!(
            rewrite.consumer_file.is_absolute(),
            "current behavior: persisted preview stores absolute consumer paths \
             (\\??-prefixed after canonicalize) — machine-specific, characterization only"
        );
        assert_eq!(rewrite.old_includes, vec![PathBuf::from("include/thing.h")]);
        assert_eq!(rewrite.new_includes, expected_thing_plan_new_includes());
        assert!(rewrite.side_effect_preservations.is_empty());
    }

    let consumer_after = fs::read_to_string(repo_root.join("tests").join("consumer.cpp"))?;
    let thing_impl_after = fs::read_to_string(repo_root.join("src").join("thing.cpp"))?;
    assert_eq!(
        consumer_before, consumer_after,
        "migrate must not modify consumers"
    );
    assert_eq!(
        thing_impl_before, thing_impl_after,
        "migrate must not modify consumers"
    );
    assert!(
        !repo_root.join("generated").exists(),
        "migrate must not write generated headers"
    );
    Ok(())
}

#[test]
fn migrate_unknown_plan_id_reports_not_found() -> Result<()> {
    let (_repo_guard, repo_root, _plans) = provision_indexed_repo_with_plans()?;

    let error = build_migration_preview(&repo_root, "PLAN-999", true)
        .expect_err("migrating an unknown plan id must fail");
    let message = format!("{error:#}");
    assert!(
        message.contains("Plan PLAN-999 not found"),
        "unexpected error text: {message}"
    );
    Ok(())
}

#[test]
fn migrate_regenerates_missing_plan_artifacts_and_succeeds() -> Result<()> {
    let (_repo_guard, repo_root, plans) = provision_indexed_repo_with_plans()?;
    let plan_id = plan_id_targeting_thing_header(&plans);

    fs::remove_dir_all(repo_root.join("artifacts").join("plans"))?;

    build_migration_preview(&repo_root, &plan_id, false)?;

    assert!(repo_root
        .join("artifacts")
        .join("migration")
        .join("preview.json")
        .exists());
    assert!(
        repo_root
            .join("artifacts")
            .join("plans")
            .join("plans.bin")
            .exists(),
        "the in-command fallback must regenerate plan artifacts when they are missing"
    );
    assert!(
        !repo_root.join("generated").exists(),
        "migrate must never write generated headers"
    );
    Ok(())
}

#[test]
fn apply_thing_plan_locks_transaction_outcome_and_disk_effects() -> Result<()> {
    let (_repo_guard, repo_root, plans) = provision_indexed_repo_with_plans()?;
    let plan_id = plan_id_targeting_thing_header(&plans);

    let consumer_before = fs::read_to_string(repo_root.join("tests").join("consumer.cpp"))?;
    let thing_impl_before = fs::read_to_string(repo_root.join("src").join("thing.cpp"))?;
    let thing_header_before = fs::read_to_string(repo_root.join("include").join("thing.h"))?;
    assert!(
        !repo_root.join("generated").exists(),
        "precondition: no generated dir"
    );

    apply_transform_plan(&repo_root, &plan_id)?;

    let transaction_dir = repo_root.join("artifacts").join("transaction");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(transaction_dir.join("manifest.json"))?)?;
    assert_eq!(manifest["plan_id"], Value::String(plan_id));
    assert_eq!(manifest["outcome"], "Committed");
    assert_eq!(manifest["tiers_run"], 3);
    assert_eq!(manifest["tiers_passed"], 3);

    let outcome: Value =
        serde_json::from_str(&fs::read_to_string(transaction_dir.join("outcome.json"))?)?;
    let tiers: Vec<String> = outcome["Committed"]["tiers_passed"]
        .as_array()
        .expect("thing.h plan apply must reach the Committed variant")
        .iter()
        .map(|tier| tier["tier"].as_str().unwrap_or_default().to_string())
        .collect();
    assert_eq!(tiers, vec!["Structural", "GeneratedHeaders", "AffectedTUs"]);

    let generated_files = list_relative_files(&repo_root.join("generated"))?;
    assert_eq!(
        generated_files,
        vec![
            "include/ThingConfig_extracted.h",
            "include/Thing_Thing_extracted.h",
            "include/Thing_extracted.h",
            "include/Thing_get_id_extracted.h",
        ],
        "apply writes exactly the plan's extraction headers under generated/<target dir>/"
    );

    let consumer_after = fs::read_to_string(repo_root.join("tests").join("consumer.cpp"))?;
    let thing_impl_after = fs::read_to_string(repo_root.join("src").join("thing.cpp"))?;
    // Fixed truth (extraction-consumer-rewrite CW-1): the C++ plugin now
    // implements consumer_rewrite_spec with the bare-file-name rule, so the
    // committed apply rewrites the bare `#include "thing.h"` into the four
    // bare extracted includes, in the same BTreeSet order the generated-file
    // list above asserts. Each replace-clone pins old-gone, four-in-place,
    // order, and all-other-bytes-unchanged, and stays byte-safe under either
    // checkout EOL (no hardcoded full-file bytes).
    let expected_block = "#include \"ThingConfig_extracted.h\"\n#include \"Thing_Thing_extracted.h\"\n#include \"Thing_extracted.h\"\n#include \"Thing_get_id_extracted.h\"";
    assert_eq!(
        consumer_after,
        consumer_before.replace("#include \"thing.h\"", expected_block),
        "extraction-consumer-rewrite: apply rewrites tests/consumer.cpp - the bare old \
         include is replaced in place by the four extracted includes"
    );
    assert_eq!(
        thing_impl_after,
        thing_impl_before.replace("#include \"thing.h\"", expected_block),
        "extraction-consumer-rewrite: apply rewrites src/thing.cpp the same way"
    );
    assert!(
        !consumer_after.contains("#include \"thing.h\""),
        "the old bare include directive is gone from the rewritten consumer"
    );

    let thing_header_after = fs::read_to_string(repo_root.join("include").join("thing.h"))?;
    fn uncomment_marked_regions(text: &str) -> String {
        let mut restored = String::new();
        let mut inside_region = false;
        for line in text.split_inclusive('\n') {
            let line_body = line.trim_end_matches(['\r', '\n']);
            if line_body.starts_with("// ARIADNE-EXTRACT-OUT BEGIN ") { inside_region = true; continue; }
            if line_body.starts_with("// ARIADNE-EXTRACT-OUT END ") { inside_region = false; continue; }
            restored.push_str(if inside_region { line.strip_prefix("//").unwrap_or(line) } else { line });
        }
        restored
    }
    let begin_markers: Vec<&str> = thing_header_after.lines().filter(|line| line.starts_with("// ARIADNE-EXTRACT-OUT BEGIN ")).collect();
    let end_markers: Vec<&str> = thing_header_after.lines().filter(|line| line.starts_with("// ARIADNE-EXTRACT-OUT END ")).collect();
    assert_ne!(thing_header_after, thing_header_before, "the original file moved: apply must retire the moved declarations in include/thing.h, not leave it byte-identical (extraction-symbol-move)");
    assert_eq!(begin_markers.len(), end_markers.len(), "the marked regions are balanced BEGIN/END pairs");
    assert_eq!(begin_markers, vec!["// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-001 symbols=Thing"], "exactly ONE BEGIN line: the class record marks as one merged region (ctor/method locate inside it and refuse, ThingConfig is unlocatable - planner fixture verdict, not the shape's relayed 4-pair count); it names the plan id AND the moved symbols forwarded through move_out_top_level_symbols");
    assert_eq!(end_markers, vec!["// ARIADNE-EXTRACT-OUT END plan=PLAN-001"], "the END line binds the same plan id provenance");
    assert_eq!(uncomment_marked_regions(&thing_header_after), thing_header_before, "kickoff section 1 layer-2 reversibility: uncommenting the marked region reconstructs the original file byte-exactly");
    assert!(!thing_header_after.contains("\nclass Thing {"), "no live declaration of a moved name survives: the region line carries the // prefix");
    assert!(
        thing_impl_after.contains("Thing::Thing(ThingConfig config)"),
        "extraction-symbol-move: the consumer's definition bytes survive; it is the \
         ORIGINAL file include/thing.h where declarations are commented out in place \
         under the marked ARIADNE-EXTRACT-OUT region"
    );
    Ok(())
}

#[test]
fn apply_unknown_plan_id_reports_not_found() -> Result<()> {
    let (_repo_guard, repo_root, _plans) = provision_indexed_repo_with_plans()?;

    let error = apply_transform_plan(&repo_root, "PLAN-999")
        .expect_err("applying an unknown plan id must fail");
    let message = format!("{error:#}");
    assert!(
        message.contains("Plan PLAN-999 not found"),
        "unexpected error text: {message}"
    );
    Ok(())
}

#[test]
fn apply_and_migrate_missing_repo_path_report_locate_failure() -> Result<()> {
    let missing = std::env::temp_dir().join("ariadne-m20-does-not-exist-7c1d4e");

    let apply_error =
        apply_transform_plan(&missing, "PLAN-001").expect_err("apply on a missing path must fail");
    let apply_message = format!("{apply_error:#}");
    assert!(
        apply_message.contains("Failed to locate repository path"),
        "unexpected apply error: {apply_message}"
    );

    let migrate_error = build_migration_preview(&missing, "PLAN-001", true)
        .expect_err("migrate on a missing path must fail");
    let migrate_message = format!("{migrate_error:#}");
    assert!(
        migrate_message.contains("Failed to locate repository path"),
        "unexpected migrate error: {migrate_message}"
    );
    Ok(())
}
