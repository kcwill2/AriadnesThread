//! D26 (plan:425) suite: locks the vocabulary-parameterized summary
//! rendering — `format_summary()` keeps today's exact C++ bytes while
//! `format_summary_with(vocabulary)` re-renders only the directive label.

use super::*;
use ariadne_repo::FileId;
use std::path::PathBuf;

fn scala_vocabulary() -> DependencyVocabulary {
    DependencyVocabulary {
        directive_label: "Import".into(),
        directive_passive_verb: "imported".into(),
        aggregation_noun: "package aggregation".into(),
    }
}

fn metric() -> LoadMetric {
    LoadMetric {
        target_name: "com.example.services.OrderService".into(),
        target_file: Some(PathBuf::from("src/main/scala/com/example/services/OrderService.scala")),
        semantic_load: 3,
        include_load: 9,
        collateral_load: 6,
        total_load: 9,
        semantic_symbols: vec![],
        collateral_symbols: vec![],
        collateral_sources: vec![],
        largest_collateral_source: Some(CollateralSource {
            file_id: FileId::from_relative_path("src/main/scala/com/example/models/all.scala"),
            path: PathBuf::from("src/main/scala/com/example/models/all.scala"),
            symbol_count: 6,
            symbols: vec![],
        }),
        semantic_sources: vec![],
    }
}

/// Index of the directive-label line: target name, blank, "Semantic
/// load:", count, blank, directive label.
const DIRECTIVE_LABEL_LINE: usize = 5;

#[test]
fn default_summary_keeps_todays_exact_first_label() {
    let summary = metric().format_summary();
    let directive_label = summary
        .lines()
        .nth(DIRECTIVE_LABEL_LINE)
        .expect("directive label line");
    assert_eq!(directive_label, "Include load:");
    assert!(summary.contains("\nSemantic load:\n"));
    assert!(summary.contains("\nCollateral load:\n"));
    assert!(summary.contains("\nLargest collateral source:\n"));
}

#[test]
fn scala_vocabulary_renders_import_load_label() {
    let summary = metric().format_summary_with(&scala_vocabulary());
    let directive_label = summary
        .lines()
        .nth(DIRECTIVE_LABEL_LINE)
        .expect("directive label line");
    assert_eq!(directive_label, "Import load:");
    assert!(!summary.contains("Include load:"));
}

#[test]
fn only_the_directive_label_differs_between_the_two_paths() {
    let default_summary = metric().format_summary();
    let scala_summary = metric().format_summary_with(&scala_vocabulary());
    assert_eq!(
        scala_summary.replacen("Import load:", "Include load:", 1),
        default_summary,
        "every other label (Semantic/Collateral/Largest) has no language \
         vocabulary and must stay identical"
    );
}
