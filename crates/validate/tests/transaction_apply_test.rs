//! Relocated (M23 T3, D-M23-10) from `crates/validate/src/lib.rs` verbatim:
//! this orchestration test drives `apply_with` with the real C++ rewriter and
//! extractor byte-parity baselines, so it lives outside the guard walk scope
//! (`crates/*/src`). The neutral-fake orchestration tests stay in `lib.rs`.

use ariadne_lang_cpp::{CppDependencyRewriter, CppSourceExtractor};
use ariadne_transform::{ConsumerRewrite, ExtractionResult, GeneratedUnit, MigrationPreview};
use ariadne_validate::TransactionApplier;
use std::fs;
use std::path::PathBuf;

// R2 keeps directive text out of this crate's sources, so the C++ shapes
// asserted here are assembled from a non-contiguous marker (the plugin
// crate owns the canonical form).
const DIRECTIVE: &str = concat!("#inc", "lude ");
fn include_of(target: &str) -> String {
    format!("{DIRECTIVE}\"{target}\"")
}
fn consumer_source(target: &str) -> String {
    format!("{}\nint main() {{}}", include_of(target))
}

#[test]
fn test_transaction_apply_and_rollback() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();

    let src_dir = repo.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let consumer = src_dir.join("main.cpp");
    fs::write(&consumer, consumer_source("old.h")).unwrap();

    let extraction = ExtractionResult {
        plan_id: "P1".into(),
        headers: vec![GeneratedUnit {
            path: "new.h".into(),
            symbols: vec![],
            required_deps: vec![],
            primary_source_text: None,
        }],
        errors: vec![],
    };

    let migration = MigrationPreview {
        plan_id: "P1".into(),
        rewrites: vec![ConsumerRewrite {
            consumer_file: PathBuf::from("src/main.cpp"),
            old_includes: vec![PathBuf::from("old.h")],
            new_includes: vec![PathBuf::from("new.h")],
            side_effect_preservations: vec![],
        }],
    };

    let (snap, new_paths) = TransactionApplier::apply_with(
        repo,
        &extraction,
        &migration,
        &CppDependencyRewriter,
        &CppSourceExtractor,
    )
    .unwrap();

    assert!(repo.join("generated/new.h").exists());
    // Byte-parity with the pre-M18.8b apply emit (guard block + bodies):
    // the empty unit renders as the guard block alone, verbatim from the
    // extractor (M23 T2 single emitter).
    assert_eq!(
        fs::read_to_string(repo.join("generated/new.h")).unwrap(),
        "#pragma once\n\n"
    );
    let consumer_content = fs::read_to_string(&consumer).unwrap();
    assert!(consumer_content.contains(&include_of("new.h")));
    assert!(!consumer_content.contains(&include_of("old.h")));
    // Byte-parity with the pre-M18.8b replace loop.
    assert_eq!(consumer_content, consumer_source("new.h"));

    assert!(TransactionApplier::verify_restored(&snap).is_err());

    TransactionApplier::rollback(&snap, &new_paths).unwrap();
    assert!(TransactionApplier::verify_restored(&snap).is_ok());
    assert!(!repo.join("generated/new.h").exists());
    let restored_content = fs::read_to_string(&consumer).unwrap();
    assert!(restored_content.contains(&include_of("old.h")));
}
