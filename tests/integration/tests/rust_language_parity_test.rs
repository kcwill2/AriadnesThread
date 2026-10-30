//! rust-language R-10 FULL-PARITY acceptance suite - the milestone's final
//! lock (milestone-named per the guide Step-5 rule). The mNN trap named
//! explicitly: an `mNN` prefix is NOT a plan milestone (`m17_dir_nav_test.rs`
//! is UI navigation, not milestone M17; `m21_run_ui_smoke_test.rs` is not the
//! Scala milestone), so this suite carries the TRACK key `rust_language_`,
//! never an mNN name - exactly as the five per-capability suites before it do.
//!
//! Parity shape mirrors `m18_cpp_plugin_parity_test.rs` (one permanent file
//! that binds the whole shipped arc against the real registry, at
//! Scala-parity breadth), and every claim re-binds truth a per-capability
//! suite already made at its owning hand: this file re-drives the shipped
//! data through the final-state plugin - it re-litigates nothing and edits
//! nothing. m18, m19, m24 and the five `rust_language_*` suites are CITED,
//! never edited (their files in NO hunk of this hop).
//!
//! What this suite locks at the final hand, each test naming its owner hand:
//! - INV-1/INV-4 enumeration `[Cpp, Scala, Rust]` final order and the
//!   real-plugin identity bytes (owner R-4 `rust_language_discovery_test`;
//!   m24's claims 1+2 cited for the real id, never the Fortran mock).
//! - INV-5 capability monotonicity FINAL witness: the all-true +
//!   requires_toolchain:false byte-exact struct plus the m19-style
//!   in-suite pin `plugin_contract_in_default_registry` shape (owner R-9b1).
//! - INV-6 discovery/shaping data re-drive (owner R-4); INV-7/INV-8 span and
//!   fingerprint re-drive (owner R-5); dependency-graph mod-tree re-drive
//!   (owner R-6); INV-14 load/manifest/vocabulary data re-drive (owner R-7).
//! - the decision-table modeling data + hook enrollment (owner R-4/R-9b1).
//! - plan/preview data re-drive incl. the R-9b3 AMENDMENT-3 probe re-pin
//!   (owner R-9b1/R-9b3); the apply/rollback transaction arc on the m23-
//!   mirror consumer bytes (owner R-8/R-9a/R-9b3).
//! - INV-10 probe contract re-drive through the granted dev-dep seam
//!   (owner R-9b1; the F6 tier mapping itself stays owned by the isolation
//!   suite's tier-three test, cited not duplicated).
//! - RISK-EMPTY-CLAIMANT honesty: mixed cpp+rust stays the CURRENT M21
//!   multi-language error - a behavior pin, NOT a feature claim (shape
//!   sec. 6: the milestone never claimed end-to-end mixed-language planning).
//! - RISK-SELFSCAN (D-3): every `rust_language_*` suite operates on
//!   `tests/fixtures/rust-basic` COPIES and carries no own-tree analysis root;
//!   the grep evidence is journalled verbatim (shape sec. 6).
//!
//! RGB posture: JUSTIFIED RED-SKIP (SM-1 / W-1 / CW-2 precedent trio, cited
//! in the R-10 journal): every assertion here binds truth that was already
//! RED at its owning hand and journal-recorded there; a suite that invents
//! no new claim cannot manufacture a fresh red. Falsifiability statement
//! (journalled verbatim): the suite's all-true caps bind and the
//! mixed-language-current-behavior bind fail on the R-9 tree MINUS any
//! single flip.
//!
//! Isolation discipline (D-3): the frozen fixture is READ-ONLY; every copy,
//! artifact, and synthetic file lands inside a `tempfile::TempDir` copy
//! (the m19/m22 idiom the per-capability suites established).

use anyhow::Result;
use ariadne_build::{BuildDiscoverer, BuildDiscoveryResult, CompileCommand, TranslationUnit};
use ariadne_cli::commands::transform::{
    plan_consumer_discovery_targets, plan_rewrite_targets, plan_target_consumers,
    repository_source_reader,
};
use ariadne_cli::registry::{
    catalog_languages, default_registry, incremental_backend, plan_backend,
};
use ariadne_cli::{analyze_dependency_load, plan_extractions};
use ariadne_cluster::plan::OptimizationPlan;
use ariadne_graph::{build_dependency_graph, IncludeKind, LoadAnalyzer};
use ariadne_lang::{
    AggregationKind, CompilerValidator, CyclePolicy, GuardKind, LanguageCapabilities, LanguageId,
    LanguagePlugin, LanguageRegistry, PluginError, SymbolKind, SymbolPlacement, ToolchainStatus,
    TuAnalysis, UnitKind,
};
use ariadne_lang_rust::validator::{CommandOutcome, CommandRunner};
use ariadne_lang_rust::RustToolchainValidator;
use ariadne_pipeline::Pipeline;
use ariadne_repo::{FileId, Repository, SourceLanguage};
use ariadne_transform::IncludeRewriter;
use ariadne_validate::TransactionApplier;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const FIXTURE: &str = "rust-basic";
const LIB_REL: &str = "src/lib.rs";
const GREETING_REL: &str = "src/greeting.rs";
const MODELS_REL: &str = "src/models/mod.rs";
const USER_REL: &str = "src/models/user.rs";
const MACRO_REL: &str = "src/macro_specimen.rs";
const TEST_SIDE_REL: &str = "tests/cli_smoke.rs";
const VENDOR_REL: &str = "vendor/legacy/legacy.rs";

/// The five `src/**.rs` units (the per-capability suites carry the same
/// const set - cited, not edited).
const SRC_UNITS: [&str; 5] = [LIB_REL, GREETING_REL, MODELS_REL, USER_REL, MACRO_REL];

/// The temp-copy consumer exactly as the isolation suite authors it (R-6
/// precedent, cited) - the OLD imports below are the bytes the m23-mirror
/// apply assertion watches retarget.
const CONSUMER_REL: &str = "src/consumer.rs";
const CONSUMER_SOURCE: &str = "use crate::fixture_marker;\nuse crate::models::models_marker;\n";

struct IsolatedRustRepo {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

/// The m19 fixture-path idiom, shared verbatim by all five per-capability
/// suites: the copy source is always `tests/fixtures/<name>`.
fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("fixtures").join(name)
}

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

/// The isolation-suite consumer authoring (cited, not edited): the consumer
/// file plus the `mod consumer;` line grown into the COPY of lib.rs.
fn add_temp_consumer(root: &Path) -> Result<()> {
    fs::write(root.join(CONSUMER_REL), CONSUMER_SOURCE)?;
    let lib_text = fs::read_to_string(root.join(LIB_REL))?;
    let grown = lib_text.replace("mod models;\n", "mod models;\nmod consumer;\n");
    assert_ne!(lib_text, grown, "the copy's lib.rs declares `mod models;`");
    fs::write(root.join(LIB_REL), grown)?;
    Ok(())
}

fn normalized_rel(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

fn registered_plugin<'a>(registry: &'a LanguageRegistry) -> &'a dyn LanguagePlugin {
    registry
        .plugin_for_id(LanguageId::Rust)
        .expect("Rust registered in the CLI default registry (R-4)")
}

fn count_line_matches(text: &str, wanted: &str) -> usize {
    text.lines().filter(|line| line.trim() == wanted).count()
}

/// The registered plugin's analyzer (the R-5 suite's seam, cited): analysis
/// over the FROZEN fixture bytes taken through the real registry dispatch.
fn analyze_registered(rel: &str) -> Result<TuAnalysis> {
    let source = fs::read_to_string(fixture_path(FIXTURE).join(rel))?;
    let registry = default_registry();
    let plugin = registered_plugin(&registry);
    Ok(plugin.analyzer().analyze(FileId::from_relative_path(rel), &source, FIXTURE))
}

fn declaration_spans(analysis: &TuAnalysis) -> Vec<(String, usize, usize)> {
    analysis
        .declarations
        .iter()
        .map(|decl| (decl.symbol_id.0.clone(), decl.location.line, decl.location.column))
        .collect()
}

// ---------------------------------------------------------------------------
// 1. INV-1 + INV-4 - enumeration and the real-plugin identity bytes, final
//    state (owner hand R-4 `rust_language_discovery_test`; m24 claims 1+2
//    cited for the mechanism, the Fortran mock never re-used).
// ---------------------------------------------------------------------------

#[test]
fn parity_default_registry_enumeration_and_identity_bytes() {
    let registry = default_registry();
    assert_eq!(
        registry.languages(),
        vec![LanguageId::Cpp, LanguageId::Scala, LanguageId::Rust],
        "INV-1 final state: registration appended Rust LAST and never moved \
         the pre-existing order (owner hand R-4; the m19 A7 lock stays green \
         UNEDITED inside the census)"
    );
    let real_id = registered_plugin(&registry).id();
    assert_eq!(real_id, LanguageId::Rust);
    assert_eq!(real_id.as_str(), "Rust");
    let json = serde_json::to_string(&real_id).expect("serialize a language id");
    assert_eq!(json, "\"Rust\"", "INV-4: the bare-name transparent form");
    let back: LanguageId = serde_json::from_str(&json).expect("deserialize it");
    assert_eq!(back, real_id, "byte-exact round trip (m24 claim 1, cited)");
    assert_eq!(format!("{:?}", real_id), "Rust", "the cache-digest Debug form");
    assert_eq!(
        registry.detect_language(Path::new("src/lib.rs")),
        Some(LanguageId::Rust),
        "INV-4/INV-9: detection is plugin-authoritative for .rs"
    );
    assert_eq!(
        registry.detect_language(Path::new("src/a.cpp")),
        Some(LanguageId::Cpp),
        "INV-2 first-wins unchanged for the shipped detectors (owner hand R-4)"
    );
}

// ---------------------------------------------------------------------------
// 2. INV-5 - the FINAL all-true capability witness, m19-style in-suite pin
//    (owner hand R-9b1; the in-src flip literal + the four per-cap locks
//    agree byte-for-byte, all green inside the census, cited not re-made).
// ---------------------------------------------------------------------------

#[test]
fn parity_capabilities_all_true_final_witness() {
    let registry = default_registry();
    // The m19 idiom `plugin_contract_in_default_registry` (cited, never
    // edited): enumeration + the plugin's capability contract in one pin.
    assert_eq!(
        registry.languages(),
        vec![LanguageId::Cpp, LanguageId::Scala, LanguageId::Rust]
    );
    let rust = registered_plugin(&registry);
    assert_eq!(
        rust.capabilities(),
        LanguageCapabilities {
            discovery: true,
            dependency_graph: true,
            semantic_index: true,
            load_metric: true,
            planning: true,
            extraction: true,
            rewriting: true,
            validation: true,
            requires_toolchain: false,
        },
        "INV-5 final literal (shape sec. 3.4, byte-exact): all eight stage \
         bits true, every one behind a shipped component; the D-23-6-style \
         struct (owner hand R-9b1 flip)"
    );
    let capabilities = rust.capabilities();
    let stage_bits = [
        capabilities.discovery,
        capabilities.dependency_graph,
        capabilities.semantic_index,
        capabilities.load_metric,
        capabilities.planning,
        capabilities.extraction,
        capabilities.rewriting,
        capabilities.validation,
    ];
    assert_eq!(
        stage_bits.iter().filter(|bit| **bit).count(),
        8,
        "the true-count pin of the all-true literal (no silent eighth bit)"
    );
    assert!(
        !capabilities.requires_toolchain,
        "requires_toolchain stays false at every hop AND at the final state \
         (A3/D-2 offline posture, owner hands R-1..R-9b1 carried it)"
    );
}

// ---------------------------------------------------------------------------
// 3. INV-6 - discovery + shaping data re-drive on a temp copy (owner hand
//    R-4 `rust_language_discovery_test`; cheap re-drive, not re-litigation).
// ---------------------------------------------------------------------------

#[test]
fn parity_fixture_discovery_and_unit_shaping() -> Result<()> {
    let repo = isolated_rust_repo("parity-discovery")?;
    let registry = default_registry();
    let repository = Repository::locate(&repo.root)?;
    let discovery = repository.discover()?;

    let mut recorded: Vec<String> = discovery
        .files
        .iter()
        .map(|record| normalized_rel(&record.path))
        .collect();
    recorded.sort();
    let mut expected: Vec<String> = SRC_UNITS.iter().map(|rel| rel.to_string()).collect();
    expected.push(TEST_SIDE_REL.to_string());
    expected.push(VENDOR_REL.to_string());
    expected.sort();
    assert_eq!(recorded, expected, "the seven .rs files, never the manifest");
    assert!(
        discovery
            .files
            .iter()
            .all(|record| record.language == SourceLanguage::Rust),
        "every record carries the Rust label through the spine table (R-1/R-4)"
    );

    let shaper = registry
        .unit_shaper_for_repo(&repository.root)
        .expect("the cargo marker at the copy root selects the Rust shaper");
    assert_eq!(shaper.build_markers(), vec!["Cargo.toml"]);
    let build = BuildDiscoverer::new(&repository.root)
        .with_repo_discovery(&discovery)
        .with_unit_shaper(Some(shaper))
        .discover()?;
    assert_eq!(build.contexts.len(), 1, "one cargo root context");
    assert_eq!(build.contexts[0].name, "root");
    assert_eq!(build.contexts[0].compiler, "rustc");
    assert_eq!(
        build.translation_units.len(),
        SRC_UNITS.len(),
        "one unit per src-rooted .rs; the tests/ side and vendor/ stay out \
         (the D8-lock class from m19, owner hand R-4)"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. INV-7 + INV-8 - span truth and the fingerprint re-driven through the
//    registered analyzer (owner hand R-5 `rust_language_semantic_index_test`).
// ---------------------------------------------------------------------------

#[test]
fn parity_semantic_index_spans_and_fingerprint() -> Result<()> {
    let registry = default_registry();
    assert_eq!(
        registered_plugin(&registry).analysis_fingerprint(),
        "rust-1",
        "INV-8: the fingerprint rides the analyzer's shipped home (owner \
         hand R-5); C++/Scala digests are separate unedited inputs"
    );
    let lib_analysis = analyze_registered(LIB_REL)?;
    assert_eq!(
        declaration_spans(&lib_analysis),
        vec![
            ("crate::greeting".to_string(), 6, 5),
            ("crate::models".to_string(), 7, 5),
            ("crate::fixture_marker".to_string(), 9, 8),
        ],
        "INV-7: exact 1-based name-token spans prove `span-locations` is \
         enabled (the literal set owner hand R-5 locked; if the feature \
         vanished these values could not reproduce)"
    );
    let macro_analysis = analyze_registered(MACRO_REL)?;
    let macro_surface: Vec<(String, SymbolKind)> = macro_analysis
        .symbols
        .iter()
        .map(|record| (record.id.0.clone(), record.kind))
        .collect();
    assert_eq!(
        macro_surface,
        vec![("crate::written_code_only".to_string(), SymbolKind::Function)],
        "D-2 macro honesty travels with the final plugin: written items \
         only, the expansion stays invisible (owner hand R-5)"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 5. Dependency-graph data re-drive over the production graph path (owner
//    hand R-6 `rust_language_dependency_graph_test`).
// ---------------------------------------------------------------------------

#[test]
fn parity_dependency_graph_mod_tree_edges() -> Result<()> {
    let repo = isolated_rust_repo("parity-graph")?;
    let registry = default_registry();
    let graph = build_dependency_graph(&repo.root, &registry)?;
    let rel_of = |id: FileId| graph.file_paths.get(&id).map(|path| normalized_rel(path));
    let resolved_pairs = |source: FileId| -> Vec<String> {
        graph
            .edges
            .iter()
            .filter(|edge| edge.source == source && edge.destination.is_some())
            .map(|edge| {
                rel_of(edge.destination.expect("filtered above")).expect("registered path")
            })
            .collect()
    };
    let lib = FileId::from_relative_path(LIB_REL);
    assert_eq!(
        resolved_pairs(lib),
        vec![GREETING_REL.to_string(), MODELS_REL.to_string()],
        "`mod greeting;` resolves to the flat sibling, `mod models;` to the \
         mod.rs sibling - BOTH layouts through the shipped resolver"
    );
    let models = FileId::from_relative_path(MODELS_REL);
    assert_eq!(
        resolved_pairs(models),
        vec![USER_REL.to_string()],
        "`pub mod user;` resolves inside its own directory (owner hand R-6)"
    );
    assert!(
        graph
            .edges
            .iter()
            .filter(|edge| edge.source == lib || edge.source == models)
            .all(|edge| edge.kind == IncludeKind::Import),
        "one Import relation per directive (the IV-1 mirror, owner hand R-6)"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 6. INV-14 - load metric + manifest + vocabulary DATA re-drive (owner hand
//    R-7 `rust_language_load_analysis_test`; no stdout scraping, no UI bytes).
// ---------------------------------------------------------------------------

#[test]
fn parity_load_metric_manifest_and_vocabulary_data() -> Result<()> {
    let repo = isolated_rust_repo("parity-load")?;
    let mut pipeline = Pipeline::new(&repo.root, None)?;
    let (graph, closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;
    let analyzer = LoadAnalyzer::with_closures(&graph, &model, closures);
    let lib_metric = analyzer.analyze_file_load(Path::new(LIB_REL))?;
    assert!(
        lib_metric
            .semantic_symbols
            .iter()
            .any(|id| id.0 == "crate::fixture_marker")
            && lib_metric.semantic_load >= 1,
        "the load lane answers over the Rust units through the shipped \
         closure machinery (owner hand R-7)"
    );

    analyze_dependency_load(&repo.root, None, None)?;
    let manifest = read_json(&repo.root.join("artifacts/load/manifest.json"))?;
    assert_eq!(
        manifest["languages"],
        Value::Array(vec![Value::String("Rust".to_string())]),
        "INV-14: the additive languages field names Rust through the \
         existing collect_stage_languages path - zero UI/report/artifact \
         bytes (owner hand R-7)"
    );

    let modeling = registered_plugin(&default_registry()).modeling();
    assert_eq!(modeling.dependency_vocabulary.directive_label, "Use");
    assert_eq!(modeling.dependency_vocabulary.directive_passive_verb, "used");
    assert_eq!(modeling.dependency_vocabulary.aggregation_noun, "mod aggregation");
    assert_eq!(
        format!("{:?}", SourceLanguage::Rust),
        "Rust",
        "the bare-name Debug is what the UI node map prints - data, not an \
         edit (the node_maps.rs expression, owner hand R-7)"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 7. The decision table as DATA + the hook pair enrolled (shape sec. 3.3/
//    3.5; owner hands R-4 modeling, R-9b1 hook enrollment - the guide's
//    trap: placement is a declared record, the hooks are what enroll).
// ---------------------------------------------------------------------------

#[test]
fn parity_modeling_data_and_hook_enrollment() -> Result<()> {
    let repo = isolated_rust_repo("parity-modeling")?;
    let registry = default_registry();
    let plugin = registered_plugin(&registry);
    let modeling = plugin.modeling();
    assert_eq!(modeling.language, LanguageId::Rust);
    assert_eq!(modeling.unit, UnitKind::SourceFile, "one .rs is a module");
    assert_eq!(
        modeling.aggregation,
        AggregationKind::ModDeclaration,
        "the appended-last variant (INV-12), never a renamed package object"
    );
    assert_eq!(modeling.guard, GuardKind::None, "Rust has no duplicate-definition guard idiom");
    assert_eq!(modeling.symbol_placement, SymbolPlacement::SingleFile);
    assert!(!modeling.aggregation_reexports, "a mod declaration does not re-aggregate code");
    assert_eq!(
        modeling.cycle_policy,
        CyclePolicy::TolerateWithinPackage,
        "planner-informational stance, consumed report-side only (RISK-CYCLE)"
    );
    let markers: Vec<&str> = modeling
        .external_path_markers
        .iter()
        .map(|marker| marker.as_str())
        .collect();
    assert_eq!(markers, vec!["target", "vendor", ".cargo"]);
    let caveat = modeling
        .analysis_caveat
        .expect("the macro limit is declared, never silent");
    assert!(caveat.contains("macro"), "caveat is honest: {caveat}");

    assert_eq!(
        plugin.aggregation_filter(&repo.root, Path::new(LIB_REL)),
        Some(true),
        "the hook pair is ENROLLED: lib.rs declares out-of-line mods (owner \
         hand R-9b1; the filter answers Some for every copy file)"
    );
    assert_eq!(
        normalized_rel(
            &plugin
                .target_path_for(Path::new(LIB_REL), "crate::fixture_marker")
                .expect("the target-naming hook is enrolled")
        ),
        "src/fixture_marker.rs",
        "the sanitized symbol name lands as its own .rs beside the \
         aggregation file, qualifier stripped"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 8. Plan + preview data re-drive, incl. the R-9b3 AMENDMENT-3 probe list
//    (owner hands R-9b1 planning enrollment, R-9b3 discovery truth).
// ---------------------------------------------------------------------------

#[test]
fn parity_plan_and_preview_rebind() -> Result<()> {
    let repo = isolated_rust_repo("parity-plan")?;
    add_temp_consumer(&repo.root)?;
    let plans = plan_extractions(&repo.root)?;
    let mut targets: Vec<String> = plans
        .iter()
        .map(|plan| normalized_rel(&plan.target_file))
        .collect();
    targets.sort();
    assert_eq!(
        targets,
        vec![LIB_REL.to_string(), MODELS_REL.to_string()],
        "the wired planner's candidate set is EXACTLY the two aggregation \
         files (owner hand R-9b1, mirrored from the isolation suite)"
    );
    let proposed_ids: Vec<String> = {
        let mut ids: Vec<String> = plans
            .iter()
            .flat_map(|plan| {
                plan.proposed_extractions
                    .iter()
                    .map(|proposal| proposal.symbol_id.0.clone())
            })
            .collect();
        ids.sort();
        ids
    };
    assert_eq!(
        proposed_ids,
        vec!["crate::fixture_marker".to_string(), "crate::models_marker".to_string()],
        "exactly the two fn proposals; the Module records and the \
         `crate::greeting` fold stay frozen (F2/F5, owner hand R-9b1)"
    );

    let registry = default_registry();
    let lib_probes = plan_consumer_discovery_targets(&registry, plan_for(&plans, LIB_REL))?;
    assert_eq!(
        lib_probes,
        vec!["crate::fixture_marker".to_string(), "lib".to_string()],
        "plan 1 probes: the moved id plus the sanitized stem (owner hand \
         R-8's discovery law, re-pinned by R-9b3's W3 as byte-identical)"
    );
    let models_probes = plan_consumer_discovery_targets(&registry, plan_for(&plans, MODELS_REL))?;
    assert_eq!(
        models_probes,
        vec![
            "crate::models_marker".to_string(),
            "crate::models::models_marker".to_string(),
            "mod".to_string(),
        ],
        "plan 2 probes carry the honest parent-qualified spelling - the \
         nested-module fix of owner hand R-9b3 (AMENDMENT-3 option (a)), \
         re-bound verbatim from the isolation suite's sanctioned re-pin"
    );
    Ok(())
}

fn plan_for<'a>(plans: &'a [OptimizationPlan], rel: &str) -> &'a OptimizationPlan {
    plans
        .iter()
        .find(|plan| normalized_rel(&plan.target_file) == rel)
        .unwrap_or_else(|| panic!("a plan targets {rel}"))
}

// ---------------------------------------------------------------------------
// 9. Apply/rollback transaction arc on the m23-mirror consumer bytes - the
//    extraction + rewriting legs of the all-eight dispatch (owner hands
//    R-8 extractor/rewriter, R-9a retention hook, R-9b3 consumer discovery;
//    the applier-core route keeps this host-toolchain-free).
// ---------------------------------------------------------------------------

#[test]
fn parity_apply_and_rollback_arc() -> Result<()> {
    let repo = isolated_rust_repo("parity-apply")?;
    add_temp_consumer(&repo.root)?;
    let plans = plan_extractions(&repo.root)?;
    let consumer_before = fs::read_to_string(repo.root.join(CONSUMER_REL))?;
    let lib_before = fs::read_to_string(repo.root.join(LIB_REL))?;

    let (snapshot, new_paths) = apply_one_plan_with_transaction(&repo.root, &plans, LIB_REL)?;

    let consumer_after = fs::read_to_string(repo.root.join(CONSUMER_REL))?;
    assert!(
        consumer_after.contains("use crate::fixture_marker::fixture_marker;"),
        "the consumer import retargets onto the moved unit (the m23-mirror \
         VERBATIM pair, owner hands R-9a/R-9b3)"
    );
    assert!(
        !consumer_after.contains("use crate::fixture_marker;"),
        "the old one-line import is GONE (NOT-OLD leg, VERBATIM)"
    );
    assert!(
        consumer_after.contains("use crate::models::models_marker;"),
        "the models-side import stays byte-untouched by this plan"
    );
    let lib_after = fs::read_to_string(repo.root.join(LIB_REL))?;
    assert!(!lib_after.contains("pub fn fixture_marker"), "move, not copy");
    assert_eq!(
        count_line_matches(&lib_after, "mod fixture_marker;"),
        1,
        "the parent gains its mod line exactly once (the RISK-MODTREE leg, \
         owner hand R-8's insertion mechanism)"
    );
    let moved_unit = fs::read_to_string(repo.root.join("src/fixture_marker.rs"))?;
    assert!(
        moved_unit.contains("extracted Rust unit") && moved_unit.contains("pub fn fixture_marker"),
        "the moved unit is the engine's generated file carrying the moved fn"
    );

    TransactionApplier::rollback(&snapshot, &new_paths)?;
    TransactionApplier::verify_restored(&snapshot)
        .map_err(|reason| anyhow::anyhow!("rollback verification failed: {reason}"))?;
    assert_eq!(
        fs::read_to_string(repo.root.join(CONSUMER_REL))?,
        consumer_before,
        "the consumer is byte-restored (the m23 rollback idiom)"
    );
    assert_eq!(
        fs::read_to_string(repo.root.join(LIB_REL))?,
        lib_before,
        "the parent is byte-restored"
    );
    assert!(
        !repo.root.join("src/fixture_marker.rs").exists(),
        "the rollback deletes the generated file it created"
    );
    Ok(())
}

/// Apply ONE plan through the applier core (no tiers/commit - the m23
/// rollback idiom the isolation suite uses verbatim, cited not edited).
fn apply_one_plan_with_transaction(
    root: &Path,
    plans: &[OptimizationPlan],
    rel: &str,
) -> Result<(ariadne_validate::FileSnapshot, Vec<PathBuf>)> {
    let registry = default_registry();
    let plan = plan_for(plans, rel);
    let migration = compose_plan_migration(root, plans, rel)?;
    let mut pipeline = Pipeline::new(root, None)?;
    let model = pipeline.get_or_build_reduced()?;
    let source_reader = repository_source_reader(&pipeline.repo)?;
    let extractor =
        plan_backend(&registry, &plan.target_file, |caps| caps.extraction, "extraction")?
            .extractor();
    let extraction =
        ariadne_transform::IsolationEngine::new(&model).extract(plan, &source_reader, extractor);
    let rewriter =
        plan_backend(&registry, &plan.target_file, |caps| caps.rewriting, "rewriting")?.rewriter();
    Ok(TransactionApplier::apply_with(
        root,
        &extraction,
        &migration,
        rewriter,
        extractor,
    )?)
}

fn compose_plan_migration(
    root: &Path,
    plans: &[OptimizationPlan],
    rel: &str,
) -> Result<ariadne_transform::MigrationPreview> {
    let plan = plan_for(plans, rel);
    let registry = default_registry();
    let mut pipeline = Pipeline::new(root, None)?;
    pipeline.get_or_build_reduced()?;
    let targets = plan_consumer_discovery_targets(&registry, plan)?;
    let all_names: Vec<String> = plan
        .proposed_extractions
        .iter()
        .map(|proposal| proposal.symbol_name.clone())
        .collect();
    let consumers = plan_target_consumers(
        &registry,
        &catalog_languages(&registry, &pipeline.discovery),
        &pipeline.build.translation_units,
        root,
        &targets,
        &plan.target_file,
        &all_names,
    );
    let (symbol_name_to_header, units) = plan_rewrite_targets(plan);
    Ok(IncludeRewriter::new(symbol_name_to_header, units)
        .build_preview(plan.id.0.clone(), &consumers))
}

// ---------------------------------------------------------------------------
// 10. INV-10 - the probe contract re-driven through the granted dev-dep
//     seam (owner hand R-9b1; the tier-mapping consumption itself stays
//     owned by the isolation suite's tier-three test, cited not duplicated).
// ---------------------------------------------------------------------------

/// A never-spawning probe recorder: every probe call is logged, every answer
/// is None (the offline machine). The isolation suite's scripted runner,
/// reduced to its all-None leg for this re-drive.
struct NoneProbeRecorder {
    call_log: Arc<Mutex<Vec<String>>>,
}

impl CommandRunner for NoneProbeRecorder {
    fn run(&self, binary: &str, args: &[&str]) -> Option<CommandOutcome> {
        self.call_log
            .lock()
            .expect("the call log lock")
            .push(format!("{binary} {}", args.join(" ")));
        None
    }
}

#[test]
fn parity_toolchain_probe_contract_never_unknown() {
    let call_log = Arc::new(Mutex::new(Vec::new()));
    let validator = RustToolchainValidator::with_runner(Box::new(NoneProbeRecorder {
        call_log: Arc::clone(&call_log),
    }));
    let status = validator.toolchain_status();
    assert!(
        matches!(status, ToolchainStatus::Available(_) | ToolchainStatus::Missing),
        "INV-10: toolchain_status is NEVER Unknown once the probe validator \
         shipped (owner hand R-9b1); this scripted walk answered {status:?}"
    );
    assert_eq!(
        status,
        ToolchainStatus::Missing,
        "an all-None probe walk answers Missing - the class crates/validate \
         maps to Skipped"
    );
    let error = validator
        .validate_compile(&[PathBuf::from("src/lib.rs")], &["--edition".into(), "2021".into()])
        .expect_err("offline validate_compile never reports a vacuous Ok");
    let message = error.to_string();
    assert!(
        message.contains("D-M23-12") && message.contains("cargo") && message.contains("rustc"),
        "the deferral names its class and its probe subjects: {message}"
    );
    assert!(
        matches!(error, PluginError::ToolchainUnavailable { .. }),
        "offline is an absence (Skipped-mappable), not a Fail kind: {error:?}"
    );
    assert_eq!(
        call_log.lock().expect("the call log lock").clone(),
        vec![
            "cargo --version".to_string(),
            "rustc --version".to_string(),
            "cargo --version".to_string(),
            "rustc --version".to_string(),
        ],
        "INV-10 b/c: BOTH walks (the status probe above and this compile's \
         own probe-first descent) spawn exactly the probe-order entries with \
         --version, and NOTHING beyond the probe walks ran - four probe \
         calls, zero compile-shaped spawns"
    );
}

// ---------------------------------------------------------------------------
// 11. RISK-EMPTY-CLAIMANT - mixed-language honesty as a CURRENT-behavior
//     pin: cpp+rust in one repo keeps the honest M21 multi-language error,
//     unchanged by this milestone. This asserts what the tree does TODAY and
//     claims no end-to-end mixed-language planning (shape sec. 6 names any
//     such claim FALSE for this milestone).
// ---------------------------------------------------------------------------

#[test]
fn parity_mixed_language_repo_keeps_the_m21_error() {
    let unit = |rel: &str, language: SourceLanguage| TranslationUnit {
        file_id: FileId::from_relative_path(rel),
        source_path: PathBuf::from(rel),
        context_name: "root".to_string(),
        compile_command: CompileCommand {
            directory: PathBuf::from("."),
            file: PathBuf::from(rel),
            command: None,
            arguments: Vec::new(),
            context_name: "root".to_string(),
            output: None,
        },
        language,
    };
    let build = BuildDiscoveryResult {
        repo_root: PathBuf::from("."),
        contexts: Vec::new(),
        translation_units: vec![
            unit("src/main.cpp", SourceLanguage::Cpp),
            unit("src/lib.rs", SourceLanguage::Rust),
        ],
    };
    let registry = default_registry();
    let error_text = incremental_backend(&registry, &build)
        .err()
        .expect("CURRENT behavior (owner of the stance: shape sec. 6): a \
                 cpp+rust repo still ERRORS at the incremental seam - the \
                 milestone never claimed mixed-language dispatch")
        .to_string();
    assert!(
        error_text.contains("multi-language")
            && error_text.contains("language-scoped cache (M21)"),
        "the honest M21 message is UNCHANGED behavior, now naming Rust in \
         its discovered set: {error_text}"
    );
    assert!(
        !error_text.contains("unhandled language"),
        "Rust is CLAIMED (registered) - the error is the M21 dispatch gap, \
         never an unclaimed-language refusal: {error_text}"
    );
}

// ---------------------------------------------------------------------------
// 12. RISK-SELFSCAN (D-3) - every rust_language_* suite points analysis at
//     `tests/fixtures/rust-basic` COPIES only: the source-level evidence the
//     shape demands for the final hand (the grep digest is journalled by the
//     R-10 journal; this test keeps it standing in CI).
// ---------------------------------------------------------------------------

#[test]
fn selfscan_rust_suites_operate_on_fixture_copies_only() -> Result<()> {
    // Fragments concatenated at runtime keep this file from tripping its
    // own scan. The forbidden idioms are the two constructors of an
    // OWN-TREE ANALYSIS ROOT: a process-working-directory root and the
    // upward ancestor walk that lands on the repository itself. (A plain
    // field named like a root, e.g. a synthetic build result's, always
    // pointing at "." for a temp copy, is not an analysis root and is not
    // scanned for.)
    let own_tree_root_idioms: Vec<String> = vec![
        ["current", "_dir"].concat(),
        [".par", "ent", "()"].concat(),
    ];
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut suite_files: Vec<String> = Vec::new();
    for entry in fs::read_dir(&tests_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("rust_language_") && name.ends_with(".rs") {
            suite_files.push(name);
        }
    }
    suite_files.sort();
    assert_eq!(
        suite_files,
        vec![
            "rust_language_dependency_graph_test.rs".to_string(),
            "rust_language_discovery_test.rs".to_string(),
            "rust_language_load_analysis_test.rs".to_string(),
            "rust_language_parity_test.rs".to_string(),
            "rust_language_semantic_index_test.rs".to_string(),
            "rust_language_symbol_isolation_test.rs".to_string(),
        ],
        "the milestone's full suite roster (five per-capability locks + this \
         parity lock) - the scan's non-vacuity set"
    );
    for name in &suite_files {
        let text = fs::read_to_string(tests_dir.join(name))?;
        assert!(
            text.contains("fixtures"),
            "{name}: every milestone suite takes its analysis input from the \
             frozen tests/fixtures set (fixture_path idiom)"
        );
        for own_tree_root in &own_tree_root_idioms {
            assert!(
                !text.contains(own_tree_root.as_str()),
                "{name}: own-tree root idiom {own_tree_root:?} would let a \
                 milestone suite scan THIS repository as analysis input \
                 (D-3/RISK-SELFSCAN forbids it)"
            );
        }
    }
    Ok(())
}
