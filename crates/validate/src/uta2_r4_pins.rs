//! UTA2-R4 pins (docs/specs/unit-test-audit.md Round 2, handoff item 8) for
//! the two IO-error observations the UTA-T4 review shipped as journal notes
//! (verdict "Follow-up dispositions"): `FileSnapshot::capture` fails on a
//! pre-existing DIRECTORY target with a BARE std::io error - before the
//! AR-04 decorated rollback path, with no path context - and a rolled-back
//! `apply_with` restores FILES but leaves the intermediate directories it
//! created. Pinned exactly; unification/cleanup is an owner decision.
//! Child module of the crate root (the `orchestration_tests` model).

use super::*;
use ariadne_lang::{DependencyRewriter, GeneratedSourceFile, RewriteSpec, SourceExtractor};
use ariadne_transform::{ConsumerRewrite, GeneratedUnit};

/// Neutral no-text rewriter (the orchestration-suite double shape).
struct NoOpRewriter;

impl DependencyRewriter for NoOpRewriter {
    fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
        source.to_string()
    }
}

/// Static-bytes extractor; the namespace/imports hooks keep their defaults.
struct StaticExtractor;

impl SourceExtractor for StaticExtractor {
    fn extract(
        &self,
        _symbols: &[ariadne_lang::SymbolBody],
        _namespace: &str,
        _required_deps: &[String],
    ) -> GeneratedSourceFile {
        GeneratedSourceFile {
            path: PathBuf::from("unused.h"),
            content: "PIN-STATIC\n".to_string(),
        }
    }
}

#[test]
fn pin_capture_of_directory_target_surfaces_bare_io_error_before_any_mutation() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();
    fs::write(repo.join("keep.h"), b"content").unwrap();
    let dir_slot = repo.join("dirslot");
    fs::create_dir_all(&dir_slot).unwrap();

    // Direct capture: a directory target EXISTS, so `fs::read(p)?` runs and
    // fails. The error is the BARE std::io::Error (typed `std::io::Result`,
    // no anyhow context, no path naming); which io kind Windows (denied) vs
    // Unix (is-a-directory) reports is platform error text, not contract.
    let paths = vec![repo.join("keep.h"), dir_slot.clone()];
    let outcome: std::io::Result<FileSnapshot> = FileSnapshot::capture(&paths);
    match outcome {
        Ok(snapshot) => panic!(
            "pin: a directory target must fail capture, not record an entry \
             ({:?})",
            snapshot.entries.keys().collect::<Vec<_>>()
        ),
        Err(_bare_io_error) => {}
    }

    // The UTA-T4 observation end to end: the same failure through
    // `apply_with` aborts BEFORE any mutation and therefore never reaches
    // the AR-04 rollback decoration - the chain carries no "rolled back"
    // line and no staging dir was ever created.
    let extraction = ExtractionResult {
        plan_id: "P-CAP".into(),
        headers: vec![],
        errors: vec![],
    };
    let migration = MigrationPreview {
        plan_id: "P-CAP".into(),
        rewrites: vec![ConsumerRewrite {
            consumer_file: PathBuf::from("dirslot"),
            old_includes: vec![],
            new_includes: vec![],
            side_effect_preservations: vec![],
        }],
    };
    let outcome = TransactionApplier::apply_with(
        repo,
        &extraction,
        &migration,
        &NoOpRewriter,
        &StaticExtractor,
    );
    match outcome {
        Ok(_) => panic!("a directory consumer target must abort apply_with"),
        Err(err) => {
            let chain = format!("{err:#}");
            assert!(
                !chain.contains("rolled back"),
                "pin: capture failure precedes the AR-04 decorated path: {chain}"
            );
        }
    }
    assert!(
        !repo.join("generated").exists(),
        "pin: capture aborts before any staging/mutation"
    );
    assert!(dir_slot.is_dir(), "the directory target itself stays intact");
    assert_eq!(fs::read(repo.join("keep.h")).unwrap(), b"content");
}

#[test]
fn pin_rollback_restores_files_but_keeps_created_intermediate_dirs() {
    // The companion observation: a rolled-back batch removes the FILES it
    // created, but the intermediate directories `create_dir_all` staged for
    // a nested unit survive the rollback (empty dirs the applier never
    // tracks). Deterministic: unit two's parent slot is occupied by unit
    // one's freshly written file, so its staging mkdir fails.
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path();

    let unit_at = |rel: &str| GeneratedUnit {
        path: PathBuf::from(rel),
        symbols: vec![],
        required_deps: vec![],
        primary_source_text: None,
    };
    let extraction = ExtractionResult {
        plan_id: "P-DIR".into(),
        headers: vec![
            unit_at("deep/nested/a.h"),
            unit_at("deep/nested/a.h/x.h"),
        ],
        errors: vec![],
    };
    let migration = MigrationPreview {
        plan_id: "P-DIR".into(),
        rewrites: vec![],
    };

    let outcome = TransactionApplier::apply_with(
        repo,
        &extraction,
        &migration,
        &NoOpRewriter,
        &StaticExtractor,
    );
    match outcome {
        Ok(_) => panic!("mkdir through an occupied file slot must fail the batch"),
        Err(err) => {
            // Counted paths: 2 snapshot entries (both units) + 1 created
            // file (unit one); unit two's path was never staged.
            assert!(
                err.to_string().contains("rolled back 3 files"),
                "2 snapshot entries + 1 created file: {err:#}"
            );
        }
    }

    // Files: both unit paths are gone (unit one rolled back, unit two never
    // written). Directories: the staged intermediates REMAIN.
    assert!(!repo.join("generated/deep/nested/a.h").exists());
    assert!(!repo.join("generated/deep/nested/a.h/x.h").exists());
    assert!(
        repo.join("generated/deep/nested").is_dir(),
        "pin: rollback restores files, not the intermediate dirs it created"
    );
}
