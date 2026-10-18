use anyhow::{bail, Result};
use ariadne_graph::IncludeGraph;
use ariadne_lang::DependencyVocabulary;
use ariadne_pipeline::Pipeline;
use ariadne_repo::FileId;
use std::path::PathBuf;

/// Title-cases each space-separated word ("package aggregation" ->
/// "Package Aggregation") for composed header-style labels (D26).
fn title_case(wording: &str) -> String {
    wording
        .split(' ')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// One noun-bearing label: today's literal without a vocabulary, composed
/// (`Template {Noun}s ...`) with one — the C++ vocabulary renders the exact
/// pre-D26 bytes (locked by tests below).
fn aggregated_label(template: &str, noun: &str) -> String {
    format!("{template}{}s", title_case(noun))
}

fn graph_summary_title(vocabulary: Option<&DependencyVocabulary>) -> String {
    match vocabulary {
        Some(vocabulary) => format!("{} Graph Summary", title_case(&vocabulary.directive_label)),
        None => "Include Graph Summary".to_string(),
    }
}

fn unique_aggregates_label(vocabulary: Option<&DependencyVocabulary>) -> String {
    match vocabulary {
        Some(vocabulary) => format!(
            "Unique {} {}",
            title_case(&vocabulary.directive_passive_verb),
            aggregated_label("", &vocabulary.aggregation_noun)
        ),
        None => "Unique Included Headers".to_string(),
    }
}

fn unresolved_label(vocabulary: Option<&DependencyVocabulary>) -> String {
    match vocabulary {
        Some(vocabulary) => {
            aggregated_label("Unresolved ", &vocabulary.directive_label)
        }
        None => "Unresolved Includes".to_string(),
    }
}

fn top_closure_heading(vocabulary: Option<&DependencyVocabulary>) -> String {
    match vocabulary {
        Some(vocabulary) => format!(
            "Top {}s by Transitive {} Closure:",
            title_case(&vocabulary.aggregation_noun),
            title_case(&vocabulary.directive_label)
        ),
        None => "Top Headers by Transitive Include Closure:".to_string(),
    }
}

fn aggregate_kind_label(
    qualifier: &str,
    vocabulary: Option<&DependencyVocabulary>,
) -> String {
    match vocabulary {
        Some(vocabulary) => format!("{qualifier} {}", aggregated_label("", &vocabulary.aggregation_noun)),
        None => format!("{qualifier} Headers"),
    }
}

/// Summary label for the cycle line. Takes the vocabulary for signature
/// symmetry with the sibling label helpers only - this line names a graph
/// property, not a language aggregate, so no vocabulary word ever composes
/// into it (deliberate, not an oversight).
fn cycle_groups_label(_vocabulary: Option<&DependencyVocabulary>) -> &'static str {
    "  Cycle Groups Detected"
}

pub fn analyze_include_graph(repo_path: &PathBuf) -> Result<()> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let (graph, _closures) = pipeline.get_or_build_include_graph()?;
    let report = graph.generate_report();
    let (vocabulary, caveats) = crate::commands::analyze::discovered_run_vocabulary(&pipeline);
    crate::commands::analyze::print_caveats(&caveats);

    println!("{}", graph_summary_title(vocabulary.as_ref()));
    println!("  Repository: {}", pipeline.repo.root.display());
    println!("  Total Edges: {}", report.total_edges);
    println!(
        "  {}: {}",
        unique_aggregates_label(vocabulary.as_ref()),
        report.unique_headers
    );
    println!(
        "  {}: {}",
        aggregate_kind_label("Local", vocabulary.as_ref()),
        report.local_headers_count
    );
    println!(
        "  {}: {}",
        aggregate_kind_label("System", vocabulary.as_ref()),
        report.system_headers_count
    );
    println!(
        "  {}: {}",
        aggregate_kind_label("External", vocabulary.as_ref()),
        report.external_headers_count
    );
    println!(
        "  {}: {}",
        unresolved_label(vocabulary.as_ref()),
        report.unresolved_includes_count
    );
    println!("{}: {}", cycle_groups_label(vocabulary.as_ref()), report.cycle_count);
    println!();
    println!("{}", pipeline.stats.format_summary());
    println!();
    println!("{}", top_closure_heading(vocabulary.as_ref()));
    for (idx, (_id, path, closure_size)) in report.top_headers_by_closure.iter().enumerate() {
        println!(
            "  {}. {} (closure size: {})",
            idx + 1,
            path.display(),
            closure_size
        );
    }

    Ok(())
}

/// Normalizes an `explain` target into the hash input the graph keys use
/// (backslashes folded to forward slashes, lossy UTF-8). Extracted as a pure
/// seam (UTA-T7b row 8) so the fallback id's identity is pinned without a
/// live graph.
fn explain_fallback_file_id(file_path: &PathBuf) -> FileId {
    let rel_str = file_path.to_string_lossy().replace('\\', "/");
    FileId::from_relative_path(&rel_str)
}

/// The three outcomes of `explain` target resolution: a registered id found
/// by the graph's two-pass lookup, the hash-fallback id when it survives
/// registration, and the honest bail. Extracted (UTA-T7b row 8) so all
/// three arms are testable without booting the pipeline; the emission order
/// (lookup first, registration check second) is unchanged.
fn resolve_explain_target(graph: &IncludeGraph, file_path: &PathBuf) -> Result<FileId> {
    let target_id = match graph.find_file_id(file_path) {
        Some(id) => id,
        None => explain_fallback_file_id(file_path),
    };

    if !graph.file_paths.contains_key(&target_id) {
        bail!("File {:?} not found in repository include graph", file_path);
    }

    Ok(target_id)
}

pub fn explain_include_graph_for_file(repo_path: &PathBuf, file_path: &PathBuf) -> Result<()> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let (graph, _closures) = pipeline.get_or_build_include_graph()?;

    let target_id = resolve_explain_target(&graph, file_path)?;

    let tree_str = graph.format_explain(target_id);
    print!("{}", tree_str);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vocabulary_for(language: ariadne_repo::SourceLanguage) -> DependencyVocabulary {
        let registry = crate::registry::default_registry();
        registry
            .plugin_for_id(ariadne_lang::LanguageId::from(language))
            .expect("plugin registered")
            .modeling()
            .dependency_vocabulary
    }

    /// D26 stretch lock: composing with the real C++ vocabulary must render
    /// the exact pre-D26 label bytes (no witness exercised these strings, so
    /// the lock lives here).
    #[test]
    fn cpp_vocabulary_composes_todays_exact_labels() {
        let cpp_vocab = vocabulary_for(ariadne_repo::SourceLanguage::Cpp);
        let cpp = Some(&cpp_vocab);
        assert_eq!(graph_summary_title(cpp), "Include Graph Summary");
        assert_eq!(
            unique_aggregates_label(cpp),
            "Unique Included Headers"
        );
        assert_eq!(unresolved_label(cpp), "Unresolved Includes");
        assert_eq!(
            top_closure_heading(cpp),
            "Top Headers by Transitive Include Closure:"
        );
        assert_eq!(aggregate_kind_label("Local", cpp), "Local Headers");
        assert_eq!(aggregate_kind_label("System", cpp), "System Headers");
        assert_eq!(aggregate_kind_label("External", cpp), "External Headers");
    }

    /// No vocabulary (C++-only / mixed runs) keeps the literal call path.
    #[test]
    fn default_labels_match_the_cpp_vocabulary_composition() {
        let cpp_vocab = vocabulary_for(ariadne_repo::SourceLanguage::Cpp);
        let cpp = Some(&cpp_vocab);
        assert_eq!(graph_summary_title(None), graph_summary_title(cpp));
        assert_eq!(unique_aggregates_label(None), unique_aggregates_label(cpp));
        assert_eq!(unresolved_label(None), unresolved_label(cpp));
        assert_eq!(
            top_closure_heading(None),
            top_closure_heading(cpp)
        );
        assert_eq!(
            aggregate_kind_label("Local", None),
            aggregate_kind_label("Local", cpp)
        );
    }

    /// M23.5 plan sec 1.3: the summary line names the group count, so the
    /// label must read "Cycle Groups Detected" (with the two leading spaces
    /// every neighbour line carries), not the old path-enumeration wording.
    #[test]
    fn includes_summary_labels_report_cycle_groups_not_cycle_paths() {
        let cpp_vocab = vocabulary_for(ariadne_repo::SourceLanguage::Cpp);
        let cpp = Some(&cpp_vocab);
        assert_eq!(cycle_groups_label(cpp), "  Cycle Groups Detected");
        assert_eq!(cycle_groups_label(None), "  Cycle Groups Detected");
    }

    #[test]
    fn scala_vocabulary_composes_import_flavored_labels() {
        let scala_vocab = vocabulary_for(ariadne_repo::SourceLanguage::Scala);
        let scala = Some(&scala_vocab);
        assert_eq!(graph_summary_title(scala), "Import Graph Summary");
        assert_eq!(
            unique_aggregates_label(scala),
            "Unique Imported Package Aggregations"
        );
        assert_eq!(unresolved_label(scala), "Unresolved Imports");
        assert_eq!(
            top_closure_heading(scala),
            "Top Package Aggregations by Transitive Import Closure:"
        );
        assert_eq!(
            aggregate_kind_label("Local", scala),
            "Local Package Aggregations"
        );
    }

    // UTA-T7b row 8: explain target-resolution seam pins.

    #[test]
    fn pin_explain_fallback_file_id_hashes_the_normalized_path() {
        // Backslash spellings must hash as the forward-slash repo-relative
        // form (the registration key space), and provably NOT as the raw
        // backslash string - the replace is load-bearing.
        let windows_spelling = PathBuf::from("src\\nested\\thing.h");
        assert_eq!(
            explain_fallback_file_id(&windows_spelling),
            FileId::from_relative_path("src/nested/thing.h")
        );
        assert_ne!(
            explain_fallback_file_id(&windows_spelling),
            FileId::from_relative_path("src\\nested\\thing.h"),
            "without the normalization the Windows spelling would hash to a \
             different (never-registered) id"
        );
    }

    #[test]
    fn pin_resolve_explain_target_bails_verbatim_for_an_unregistered_target() {
        // The honest bail: Debug-quoted path, fixed suffix, and the target's
        // fallback id is NOT fabricated into a success (an empty graph can
        // never contain it).
        let graph = IncludeGraph::default();
        let missing = PathBuf::from("src/ghost.h");
        let error = resolve_explain_target(&graph, &missing)
            .err()
            .expect("unregistered target must not resolve");
        assert_eq!(
            error.to_string(),
            format!(
                "File {:?} not found in repository include graph",
                missing
            )
        );
    }

    #[test]
    fn pin_resolve_explain_target_uses_lookup_first_fallback_id_second() {
        // Arm 1: a registered path resolves through find_file_id (exact and
        // basename spellings both reach the same id).
        let registered_id = FileId::from_relative_path("src/real.h");
        let mut graph = IncludeGraph::default();
        graph.register_file(registered_id, PathBuf::from("src/real.h"));
        assert_eq!(
            resolve_explain_target(&graph, &PathBuf::from("src/real.h")).unwrap(),
            registered_id
        );
        assert_eq!(
            resolve_explain_target(&graph, &PathBuf::from("real.h")).unwrap(),
            registered_id,
            "basename lookup resolves without ever reaching the fallback"
        );

        // Arm 2: find_file_id misses (different registered path value), yet
        // the fallback hash id IS registered - resolution succeeds with the
        // fallback id. This is the only way the fallback arm can succeed,
        // pinned deliberately.
        let fallback_id = FileId::from_relative_path("a/b.h");
        let mut crossed = IncludeGraph::default();
        crossed.register_file(fallback_id, PathBuf::from("x/y/z.h"));
        assert_eq!(
            resolve_explain_target(&crossed, &PathBuf::from("a/b.h")).unwrap(),
            fallback_id,
            "the fallback id wins when find_file_id misses but the id is \
             registered under a different path value"
        );
    }
}
