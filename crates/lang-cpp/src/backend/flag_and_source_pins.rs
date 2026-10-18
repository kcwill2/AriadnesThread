// UTA-T2 pin locks for the backend (handoff items 4, 11): the compiler-flag
// split table and the missing-source empty-Ok swallow at backend.rs:189.
// Pins assert CURRENT behavior; none drove a src change. The frozen
// fake_process_tests module stays untouched.

use super::*;
use ariadne_build::{CompileCommand, TranslationUnit};
use ariadne_repo::FileId;

fn probe_task(source_rel: &str) -> TuTask {
    TuTask {
        task_id: 4242,
        tu: TranslationUnit {
            file_id: FileId::from_relative_path(source_rel),
            source_path: std::path::PathBuf::from(source_rel),
            context_name: "uta-t2-pin".to_string(),
            compile_command: CompileCommand {
                directory: std::path::PathBuf::new(),
                file: std::path::PathBuf::from(source_rel),
                command: None,
                arguments: Vec::new(),
                context_name: "uta-t2-pin".to_string(),
                output: None,
            },
            language: Default::default(),
        },
        estimated_cost: 1,
        context: None,
    }
}

// - item 4: collect_compiler_flags table ---------------------------------------

#[test]
fn pin_collect_compiler_flags_splits_prefixed_bare_and_unmatched_tokens() {
    let arguments = vec![
        "-DA=1".to_string(),
        "-Iinc".to_string(),
        "/std:c++20".to_string(),
        "-D".to_string(),
        "-I".to_string(),
    ];
    let (defines, include_dirs) = ClangWorkerBackend::collect_compiler_flags(&arguments);
    assert_eq!(
        defines,
        vec!["A=1".to_string(), String::new()],
        "bare `-D` yields an empty define entry, not a skip"
    );
    assert_eq!(
        include_dirs,
        vec!["inc".to_string(), String::new()],
        "bare `-I` yields an empty include-dir entry; `/std:c++20` matches nothing"
    );
}

// - item 11: missing source analyzed as empty Ok --------------------------------

#[test]
fn pin_backend_missing_source_analyzes_empty() {
    // KNOWN SWALLOW (report-flagged, spec pin-only row): backend.rs:189
    // reads the TU source with `unwrap_or_default()`, so an absent file is
    // analyzed as empty text and returned Ok - never an error.
    let repo = tempfile::tempdir().expect("repo tempdir");
    let task = probe_task("uta_t2_absent_source.cpp");
    let analysis = inprocess_cpp_fallback(&task, repo.path())
        .expect("the pinned swallow: missing source still analyzes to Ok");
    assert_eq!(analysis.tu_file_id, task.tu.file_id);
    assert_eq!(analysis.context_name, "uta-t2-pin");
    assert!(
        analysis.symbols.is_empty()
            && analysis.declarations.is_empty()
            && analysis.edges.is_empty()
            && analysis.call_sites.is_empty(),
        "an absent source must analyze to a wholly empty TuAnalysis"
    );
}
