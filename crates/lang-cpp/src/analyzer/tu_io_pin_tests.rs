// UTA-T2 pin lock for analyzer IO (handoff item 16): analyze_tu on a
// nonexistent path. Current behavior is Err (the read error propagates with
// `?`), so this is a behavior lock, not a swallow pin - contrast with
// backend/flag_and_source_pins.rs::pin_backend_missing_source_analyzes_empty.

use super::*;
use ariadne_build::{CompileCommand, TranslationUnit};

fn missing_source_tu() -> TranslationUnit {
    TranslationUnit {
        file_id: FileId::from_relative_path("uta_t2_absent_tu.cpp"),
        source_path: std::path::PathBuf::from("uta_t2_absent_tu.cpp"),
        context_name: "uta-t2-pin".to_string(),
        compile_command: CompileCommand {
            directory: std::path::PathBuf::new(),
            file: std::path::PathBuf::from("uta_t2_absent_tu.cpp"),
            command: None,
            arguments: Vec::new(),
            context_name: "uta-t2-pin".to_string(),
            output: None,
        },
        language: Default::default(),
    }
}

#[test]
fn pin_analyze_tu_on_missing_path_is_err() {
    let repo = tempfile::tempdir().expect("repo tempdir");
    let result = CppSemanticAnalyzer.analyze_tu(&missing_source_tu(), repo.path());
    assert!(
        result.is_err(),
        "analyze_tu propagates the read failure (Ok-empty would diverge from the trait contract)"
    );
}
