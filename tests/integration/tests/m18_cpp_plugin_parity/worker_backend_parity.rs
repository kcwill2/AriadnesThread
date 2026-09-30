//! Section 2 — worker backend parity (plan §5 M18 acceptance, second clause).
//!
//! `PluginWorkerBackend` on body-only tasks must reproduce the deleted
//! transitional in-process C++ backend byte-for-byte. The oracle is a
//! verbatim copy of that backend's body-only `analyze_task` branch from
//! `git show ceac55f:crates/scheduler/src/worker.rs` (lines 61–80, the
//! `context: None` path), calling `ariadne_lang_cpp::CppSemanticAnalyzer`
//! — the same analyzer the relocated plugin adapter uses, public and
//! non-deprecated after M18.11-B. The full `TuAnalysis` serde JSON is
//! compared for every analyzable file of both fixtures.

use super::{analyzable_records, isolated_repo, normalized_rel, repo_inputs};
use anyhow::{Context, Result};
use ariadne_build::{CompileCommand, TranslationUnit};
use ariadne_index::TuAnalysis;
use ariadne_lang::LanguageRegistry;
use ariadne_repo::FileRecord;
use ariadne_scheduler::{PluginWorkerBackend, TuTask, WorkerBackend};
use std::path::Path;

/// The names the vendored oracle must surface in the fixture analyses —
/// a cheap presence lock so a degenerate (empty) analysis on both sides
/// cannot pass section 2 vacuously.
const EXPECTED_SYMBOL_PRESENT: &[(&str, &str, &str)] = &[
    ("basic", "include/thing.h", "Thing"),
    ("basic", "include/thing_types.h", "ThingConfig"),
    ("basic", "include/heavy.h", "HeavyData"),
    ("expensive", "include/thing.h", "Thing"),
    ("expensive", "include/heavy.h", "HeavyTemplate"),
];

#[test]
fn plugin_worker_backend_matches_vendored_inprocess_backend_basic() -> Result<()> {
    worker_backend_parity("basic")
}

#[test]
fn plugin_worker_backend_matches_vendored_inprocess_backend_expensive() -> Result<()> {
    worker_backend_parity("expensive")
}

#[test]
fn empty_registry_backend_reports_unhandled_language_never_cpp() -> Result<()> {
    // §3.3 fact 3 from the worker side, surviving the shim deletions: a
    // registry with no plugins must error, never silently analyze as C++.
    let repo = isolated_repo("basic", "worker-empty")?;
    let (discovery, _build) = repo_inputs(&repo.root)?;
    let record = analyzable_records(&discovery)
        .into_iter()
        .find(|record| normalized_rel(&record.path) == "src/thing.cpp")
        .expect("basic fixture has src/thing.cpp");
    let task = body_only_task(record, 0);
    let err = PluginWorkerBackend::new(&LanguageRegistry::new())
        .analyze_task(&task, &repo.root)
        .expect_err("empty registry must not parse C++ silently");
    assert!(
        err.to_string().contains("unhandled language"),
        "expected an unhandled-language error, got: {err}"
    );
    Ok(())
}

fn worker_backend_parity(fixture: &str) -> Result<()> {
    let repo = isolated_repo(fixture, "worker")?;
    let (discovery, _build) = repo_inputs(&repo.root)?;
    let registry = ariadne_cli::registry::default_registry();
    let backend = PluginWorkerBackend::new(&registry);
    let records = analyzable_records(&discovery);
    assert!(!records.is_empty(), "{fixture}: no analyzable files");
    for (index, record) in records.iter().enumerate() {
        let rel = normalized_rel(&record.path);
        let task = body_only_task(record, index);
        let modern = backend
            .analyze_task(&task, &repo.root)
            .with_context(|| format!("{fixture}:{rel}"))?;
        let vendored = vendored_inprocess_backend(&task, &repo.root);
        assert_eq!(
            serde_json::to_string(&vendored)?,
            serde_json::to_string(&modern)?,
            "{fixture}:{rel}: PluginWorkerBackend diverged from the vendored \
             pre-M18.11 body-only backend"
        );
    }
    assert_expected_symbols(&repo.root, &records, fixture)?;
    Ok(())
}

/// Verbatim body-only branch of the transitional in-process backend from
/// `git show ceac55f:crates/scheduler/src/worker.rs` (the `context: None`
/// path of the pre-M18.11 hardcoded-C++ `WorkerBackend` impl, class name
/// elided here so this suite keeps its zero-deprecated-reference grep).
/// The only change is the analyzer import path: `ariadne_lang_cpp` is where
/// the analyzer permanently lives.
fn vendored_inprocess_backend(task: &TuTask, repo_root: &Path) -> TuAnalysis {
    let full_path = repo_root.join(&task.tu.source_path);
    let source_code = std::fs::read_to_string(&full_path).unwrap_or_default();
    ariadne_lang_cpp::CppSemanticAnalyzer::analyze_source(
        task.tu.file_id.clone(),
        &source_code,
        &task.tu.context_name,
    )
}

/// Body-only (`context: None`) worker task for one discovery record — the
/// exact task shape that takes the vendored backend's body-only branch.
fn body_only_task(record: &FileRecord, task_id: usize) -> TuTask {
    let context_name = "m18-parity";
    TuTask {
        task_id,
        tu: TranslationUnit {
            file_id: record.id,
            source_path: record.path.clone(),
            context_name: context_name.to_string(),
            compile_command: CompileCommand {
                directory: Default::default(),
                file: record.path.clone(),
                command: None,
                arguments: Vec::new(),
                context_name: context_name.to_string(),
                output: None,
            },
            language: Default::default(),
        },
        estimated_cost: 1,
        context: None,
    }
}

fn assert_expected_symbols(
    root: &Path,
    records: &[&FileRecord],
    fixture: &str,
) -> Result<()> {
    for (expect_fixture, rel, symbol_name) in EXPECTED_SYMBOL_PRESENT {
        if *expect_fixture != fixture {
            continue;
        }
        let record = records
            .iter()
            .find(|record| normalized_rel(&record.path) == *rel)
            .unwrap_or_else(|| panic!("{fixture}:{rel} missing from discovery"));
        let analysis = vendored_inprocess_backend(&body_only_task(record, 999), root);
        assert!(
            analysis.symbols.iter().any(|s| s.name == *symbol_name),
            "{fixture}:{rel} must define {symbol_name}: {:?}",
            analysis
                .symbols
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>()
        );
    }
    Ok(())
}
