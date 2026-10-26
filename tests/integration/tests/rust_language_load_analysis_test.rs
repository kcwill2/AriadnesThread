//! rust-language R-7 load-metric lock (milestone-named per the guide Step-5
//! rule - the track key `rust_language_`, never an mNN name). Model suite:
//! `m22_scala_load_analysis_test.rs` (the read-only zero-diff idiom:
//! fixture -> TempDir -> real pipeline load lane -> manifest/vocabulary/UI
//! DATA asserts; no stdout scraping anywhere).
//!
//! What this suite locks at the load_metric hop, all through existing data
//! paths with `crates/ui`, `crates/report`, `crates/artifact` byte-empty
//! (INV-14, proven by the journal's name-only diff):
//! - INV-5 / shape sec. 3.4: the integration-side byte-exact flip lock
//!   (`discovery + semantic_index + dependency_graph + load_metric` true and
//!   ONLY those four; m19 idiom, authored WITH the flip commit).
//! - the load lane over the `rust-basic` units: every metric is non-empty
//!   where the unit wrote symbols, finite (bounded by the model's symbol
//!   count), and keeps the LoadMetric shape class m22 asserts for Scala
//!   (include == total == semantic + collateral partition, disjoint sides,
//!   ordered contributing sources, largest == first row); the targeted
//!   handler lane runs clean (m22's witness).
//! - the Rust collateral story: a single-file-placement `use` drags the
//!   referenced file's written surface into the consumer's closure as
//!   collateral - the shape class of m22's package-object witness, with the
//!   consumer's semantic side honestly empty (a top-level `use` names no
//!   enclosing symbol, so the analyzer leaves the semantic channel empty
//!   rather than inventing edges).
//! - INV-14 manifest proof: the run's `artifacts/load/manifest.json` carries
//!   `"Rust"` in its additive `languages` field through the existing
//!   `collect_stage_languages` path (crates/artifact zero-diff).
//! - D26 vocabulary proof: the run's single non-default language selects the
//!   Rust vocabulary through the registry (the exact data the CLI handlers
//!   feed the *_with formatters - m22 idiom); "Use load:" renders through
//!   `format_summary_with`, the passive renders "is used by" through
//!   `format_explanation_with`, and the aggregation noun arrives as data.
//! - UI node-label proof: the node-language label derives from the
//!   hand-written bare-name `Debug` (`crates/cli/src/commands/ui/node_maps.rs`
//!   `format!("{:?}", file.language)` - read-only); asserted on THIS run's
//!   discovery values, so `Rust` appears with zero edits to any UI crate.

use anyhow::Result;
use ariadne_cli::analyze_dependency_load;
use ariadne_cli::registry::default_registry;
use ariadne_graph::{IncludeClosure, IncludeGraph, LoadAnalyzer, LoadMetric, ReducedSemanticModel};
use ariadne_lang::{DependencyVocabulary, LanguageCapabilities, LanguageId};
use ariadne_pipeline::Pipeline;
use ariadne_repo::{FileId, SourceLanguage};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE: &str = "rust-basic";
const LIB_REL: &str = "src/lib.rs";
const GREETING_REL: &str = "src/greeting.rs";
const MODELS_REL: &str = "src/models/mod.rs";
const USER_REL: &str = "src/models/user.rs";
const MACRO_REL: &str = "src/macro_specimen.rs";

/// The five `src/**.rs` units the shaper turns into translation units
/// (R-4/R-5 suites carry the same const set - cited, not edited).
const SRC_UNITS: [&str; 5] = [LIB_REL, GREETING_REL, MODELS_REL, USER_REL, MACRO_REL];

/// Temp-copy consumer added exactly like the R-6 suite's harness files
/// (fixture stays READ-ONLY). A single `use` of the frozen
/// `src/models/user.rs` surface - the exact edge spelling class R-6's
/// `use_edges_over_fixture_paths_point_at_files` locks to USER_REL.
const CONSUMER_REL: &str = "src/consumer.rs";
const CONSUMER_SOURCE: &str = "use crate::models::user::default_user;\n";

struct IsolatedRustRepo {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fixtures")
        .join(name)
}

/// Throwaway copy of the frozen fixture (m19/m22 isolation - every
/// synthetic file and artifact lands only inside the copy).
fn isolated_rust_repo(label: &str) -> Result<IsolatedRustRepo> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join(label);
    copy_tree(&fixture_path(FIXTURE), &root)?;
    Ok(IsolatedRustRepo { _temp: temp, root })
}

fn copy_tree(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let target = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn normalized_rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn read_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)?;
    let parsed = serde_json::from_str(&text)?;
    Ok(parsed)
}

/// The production load lane's stage inputs: boot through the CLI default
/// registry (marker-based shaper selection picks the Rust shaper on the
/// fixture's `Cargo.toml`, R-4-locked), graph from the R-6 resolver, symbols
/// from the R-5 analyzer - exactly the components the flipped bit's backing
/// load stage already consumes (handoff item 3).
fn load_lane_inputs(
    root: &Path,
) -> Result<(
    Pipeline,
    IncludeGraph,
    ReducedSemanticModel,
    HashMap<FileId, IncludeClosure>,
)> {
    let mut pipeline = Pipeline::new(root, None)?;
    let (graph, closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;
    Ok((pipeline, graph, model, closures))
}

/// The vocabulary selection the CLI handler performs for this run's
/// languages (single non-default language -> that plugin's vocabulary, the
/// `crates/cli/src/commands/analyze.rs` `run_vocabulary` rule reproduced over
/// THIS run's discovery data - rule R3-legal, no language match).
fn run_vocabulary(pipeline: &Pipeline) -> DependencyVocabulary {
    let registry = default_registry();
    let mut distinct: Vec<SourceLanguage> = Vec::new();
    for record in &pipeline.discovery.files {
        if !distinct.contains(&record.language) {
            distinct.push(record.language.clone());
        }
    }
    let [language] = distinct.as_slice() else {
        panic!("the rust-basic fixture discovers exactly one language, got {distinct:?}");
    };
    registry
        .plugin_for_id(LanguageId::from(language.clone()))
        .expect("the run's language has a registered plugin")
        .modeling()
        .dependency_vocabulary
}

/// The LoadMetric shape class m22 asserts for Scala, expressed as invariants
/// of the returned model (non-empty is asserted where the written unit says
/// so; finiteness by the model-size bound): partition identity, disjoint
/// sides, ordered contributing sources, largest == first ordered row.
fn assert_metric_shape_class(metric: &LoadMetric, model_symbol_count: usize) {
    assert!(
        metric.total_load <= model_symbol_count,
        "finite: {}'s total_load {} is bounded by the model's {} symbols",
        metric.target_name,
        metric.total_load,
        model_symbol_count
    );
    assert_eq!(
        metric.include_load, metric.total_load,
        "shape class: include == total for {}",
        metric.target_name
    );
    assert_eq!(
        metric.semantic_load + metric.collateral_load,
        metric.total_load,
        "shape class: the physical expansion partitions for {}",
        metric.target_name
    );
    for semantic in &metric.semantic_symbols {
        assert!(
            !metric.collateral_symbols.contains(semantic),
            "{}: {} cannot be both semantic and collateral",
            metric.target_name,
            semantic.0
        );
    }
    let mut ordered = metric.collateral_sources.clone();
    ordered.sort_by(|a, b| {
        b.symbol_count
            .cmp(&a.symbol_count)
            .then_with(|| a.path.cmp(&b.path))
    });
    assert_eq!(
        metric.collateral_sources, ordered,
        "{}: contributing sources keep the (count desc, path asc) order",
        metric.target_name
    );
    let expected_largest = metric.collateral_sources.first().cloned();
    assert_eq!(
        metric.largest_collateral_source, expected_largest,
        "{}: the largest collateral source is the first ordered row",
        metric.target_name
    );
}

// ---------------------------------------------------------------------------
// 1. INV-5 / shape sec. 3.4 - the integration-side byte-exact flip lock
//    (m19 idiom, authored WITH the flip commit; the handoff's predicted Red
//    for the PRE-hand run is this pin failing
//    `left: load_metric: false, right: true` - R-5/R-6 D5 precedent).
// ---------------------------------------------------------------------------

#[test]
fn capabilities_lock_the_load_metric_flip_literal() {
    let registry = default_registry();
    let rust = registry
        .plugin_for_id(LanguageId::Rust)
        .expect("Rust registered in the CLI default registry (R-4)");
    assert_eq!(
        rust.capabilities(),
        LanguageCapabilities {
            discovery: true,
            dependency_graph: true,
            semantic_index: true,
            load_metric: true,
            planning: false,
            extraction: false,
            rewriting: false,
            validation: false,
            requires_toolchain: false,
        },
        "the load-metric hop turns load_metric on alongside the three landed \
         bits and ONLY those four (INV-5 monotonicity); requires_toolchain \
         stays false at every hop (A3/D-2)"
    );
    let caps = rust.capabilities();
    let enabled = [
        caps.discovery,
        caps.dependency_graph,
        caps.semantic_index,
        caps.load_metric,
        caps.planning,
        caps.extraction,
        caps.rewriting,
        caps.validation,
    ];
    assert_eq!(
        enabled.iter().filter(|bit| **bit).count(),
        4,
        "exactly four stage bits are true after this flip"
    );
}

// ---------------------------------------------------------------------------
// 2. The load lane over the rust-basic units: non-empty, finite, shape
//    class, the targeted handler lane, and the Rust collateral story.
// ---------------------------------------------------------------------------

#[test]
fn load_metric_answers_over_the_rust_basic_units() -> Result<()> {
    let repo = isolated_rust_repo("units")?;
    fs::write(repo.root.join("src").join("consumer.rs"), CONSUMER_SOURCE)?;
    let (_pipeline, graph, model, closures) = load_lane_inputs(&repo.root)?;
    let analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);
    let model_symbol_count = model.symbols.len();

    let mut unit_metrics = Vec::new();
    for rel in SRC_UNITS {
        let metric = analyzer.analyze_file_load(Path::new(rel))?;
        let target_rel = metric
            .target_file
            .as_deref()
            .map(normalized_rel)
            .unwrap_or_else(|| rel.to_string());
        assert_eq!(target_rel, rel, "{rel}: the metric names its own unit");
        assert!(
            metric.semantic_load >= 1,
            "{rel}: every src unit wrote at least one symbol, so its \
             seed-inclusive semantic closure is non-empty"
        );
        assert_metric_shape_class(&metric, model_symbol_count);
        unit_metrics.push(metric);
    }

    // Written-surface witnesses through the real lane (unique-owner ids -
    // `crate::greeting` stays untouched: its mod-record/fn-record id-sharing
    // class is R-9's parked adjudication, R-5 D5 relay).
    let lib_metric = unit_metrics
        .iter()
        .find(|metric| {
            metric
                .target_file
                .as_deref()
                .is_some_and(|path| normalized_rel(path) == LIB_REL)
        })
        .expect("lib.rs answered");
    assert!(
        lib_metric
            .semantic_symbols
            .iter()
            .any(|id| id.0 == "crate::fixture_marker"),
        "the crate root's own written fn is semantic on its own unit: {:?}",
        lib_metric
            .semantic_symbols
            .iter()
            .map(|id| id.0.clone())
            .collect::<Vec<_>>()
    );
    let macro_metric = unit_metrics
        .iter()
        .find(|metric| {
            metric
                .target_file
                .as_deref()
                .is_some_and(|path| normalized_rel(path) == MACRO_REL)
        })
        .expect("macro_specimen.rs answered");
    assert!(
        macro_metric
            .semantic_symbols
            .iter()
            .any(|id| id.0 == "crate::written_code_only"),
        "the D-2 specimen's written fn answers through the load lane too"
    );

    // The Rust collateral story (same shape class as m22's package-object
    // witness) over the temp-copy consumer.
    assert_consumer_collateral(&analyzer, model_symbol_count)?;

    // The handler lane itself (targeted => never persists, m6's guard, m22's
    // witness): the load stage must run clean over the Rust pipeline.
    analyze_dependency_load(&repo.root, Some("crate::fixture_marker"), None)?;
    Ok(())
}

/// The Rust collateral story (same shape class as m22's package-object
/// witness): the consumer's single-file-placement `use` drags the whole
/// written surface of src/models/user.rs into its closure as collateral.
fn assert_consumer_collateral(
    analyzer: &LoadAnalyzer<'_>,
    model_symbol_count: usize,
) -> Result<()> {
    let consumer_metric = analyzer.analyze_file_load(Path::new(CONSUMER_REL))?;
    assert_eq!(
        consumer_metric.collateral_load, 2,
        "user.rs's two written items arrive as the consumer's collateral \
         (the consumer itself wrote no symbols - a top-level `use` leaves \
         the semantic channel honestly empty, nothing invented)"
    );
    let largest = consumer_metric
        .largest_collateral_source
        .as_ref()
        .expect("the use-lane collateral exists");
    assert_eq!(
        normalized_rel(&largest.path),
        USER_REL,
        "the largest collateral source is the used file"
    );
    assert_eq!(largest.symbol_count, 2);
    let collateral_ids: Vec<String> = consumer_metric
        .collateral_symbols
        .iter()
        .map(|id| id.0.clone())
        .collect();
    for written_id in ["crate::User", "crate::default_user"] {
        assert!(
            collateral_ids.iter().any(|id| id == written_id),
            "{written_id} is dragged in by the single `use`: {collateral_ids:?}"
        );
    }
    assert_metric_shape_class(&consumer_metric, model_symbol_count);
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. INV-14 manifest proof (m22's load_manifest_carries_scala_language
//    mirror): the existing collect_stage_languages path stamps "Rust" into
//    the additive languages field - zero artifact/report/UI bytes.
// ---------------------------------------------------------------------------

#[test]
fn load_manifest_carries_rust_language() -> Result<()> {
    let repo = isolated_rust_repo("manifest")?;

    analyze_dependency_load(&repo.root, None, None)?;

    let manifest = read_json(&repo.root.join("artifacts/load/manifest.json"))?;
    assert_eq!(
        manifest["languages"],
        Value::Array(vec![Value::String("Rust".to_string())]),
        "INV-14: a Rust run's load manifest names its language through the \
         existing additive `languages` field (C++ runs keep the key absent - \
         m00 goldens)"
    );
    assert!(
        manifest["total_analyzed_symbols"]
            .as_u64()
            .unwrap_or(0)
            >= 1,
        "the full-stage lane produced a non-empty symbol pass: {manifest}"
    );
    assert!(
        manifest["total_analyzed_files"]
            .as_u64()
            .unwrap_or(0)
            >= 1,
        "the full-stage lane produced a non-empty file pass: {manifest}"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. D26 vocabulary proof (m22's formatter-fed idiom): the run's
//    registry-selected vocabulary renders through the EXACT *_with
//    formatters the CLI handlers call; data asserts, never a stdout scrape.
// ---------------------------------------------------------------------------

#[test]
fn d26_vocabulary_surfaces_through_the_graph_render_lane() -> Result<()> {
    let repo = isolated_rust_repo("vocabulary")?;
    fs::write(repo.root.join("src").join("consumer.rs"), CONSUMER_SOURCE)?;
    let (pipeline, graph, model, closures) = load_lane_inputs(&repo.root)?;
    let analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);

    let vocabulary = run_vocabulary(&pipeline);
    assert_eq!(vocabulary.directive_label, "Use");
    assert_eq!(vocabulary.directive_passive_verb, "used");
    assert_eq!(vocabulary.aggregation_noun, "mod aggregation");

    let consumer_metric = analyzer.analyze_file_load(Path::new(CONSUMER_REL))?;
    let summary = consumer_metric.format_summary_with(&vocabulary);
    assert!(
        summary.contains("Use load:"),
        "D26 use-flavored load wording: {summary}"
    );
    assert!(
        !summary.contains("Include load:") && !summary.contains("Import load:"),
        "other languages' wording is absent from the Rust render: {summary}"
    );

    let explanation = analyzer.explain_file_load(Path::new(CONSUMER_REL), None)?;
    let formatted = explanation.format_explanation_with(&vocabulary);
    assert!(
        formatted.contains("user.rs is used by consumer.rs."),
        "the passive vocabulary field renders through the real formatter: {formatted}"
    );
    assert!(
        !formatted.contains("included") && !formatted.contains("imported"),
        "no other language's passive verb appears: {formatted}"
    );
    assert!(!formatted.contains("  by "), "no double-space artifact: {formatted}");

    // D-2 honesty travels with the load answers (m22's caveat-as-data form).
    let registry = default_registry();
    let caveat = registry
        .plugin_for_id(LanguageId::Rust)
        .expect("Rust registered")
        .modeling()
        .analysis_caveat
        .expect("the macro limit is declared, never silent");
    assert!(caveat.contains("macro"), "caveat is honest: {caveat}");
    Ok(())
}

// ---------------------------------------------------------------------------
// 5. UI node-label proof: the node-language label is the hand-written
//    bare-name Debug of the run's language values - the exact expression at
//    `crates/cli/src/commands/ui/node_maps.rs` `language_map` (read-only:
//    the UI/report/artifact crates have zero bytes in this chain,
//    journal-proven).
// ---------------------------------------------------------------------------

#[test]
fn node_language_labels_derive_from_the_bare_debug_name() -> Result<()> {
    let repo = isolated_rust_repo("node-labels")?;
    let (pipeline, _graph, _model, _closures) = load_lane_inputs(&repo.root)?;
    let registry = default_registry();

    assert!(
        !pipeline.discovery.files.is_empty(),
        "the run discovered the fixture's units"
    );
    for record in &pipeline.discovery.files {
        let label = format!("{:?}", record.language);
        assert_eq!(
            label,
            "Rust",
            "the Debug bare name of the run's language values prints `Rust` \
             with zero UI edits ({})",
            normalized_rel(&record.path)
        );
        let unit_label = registry
            .plugin_for_id(LanguageId::from(record.language.clone()))
            .map(|plugin| format!("{:?}", plugin.modeling().unit));
        assert_eq!(
            unit_label.as_deref(),
            Some("SourceFile"),
            "the unit-kind label follows the modeling data on the same idiom"
        );
    }
    Ok(())
}
