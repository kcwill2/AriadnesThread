//! rust-language R-6 dependency-graph lock (milestone-named per the guide
//! Step-5 rule - the track key `rust_language_`, never an mNN name). Model
//! suite: `m20_scala_dependency_graph_test.rs` (the read-only zero-diff
//! idiom: fixture -> TempDir -> real registry graph -> edge-kind, boundary,
//! and cycle-report asserts). The caps lock added with the flip commit
//! mirrors m19's integration-side capability idiom.
//!
//! What this suite locks at the resolver hop:
//! shape sec. 3.7 resolver row - `mod x;` resolves to the child file in BOTH
//! layouts (`src/models/mod.rs` sibling and `src/greeting.rs` flat sibling);
//! `use` paths resolve to repo-discovered targets carrying the written
//! line/col (INV-7 spans honored end to end); out-of-repo paths resolve to
//! NOTHING (external marker, never an invented destination); the `vendor/`
//! boundary stays marker-only (no unit, no edge either way); the RISK-CYCLE
//! oracle - a legal intra-crate `use` cycle stays informational through the
//! existing `cycle_policy_by_file`-shaped registry lookup (zero core edits,
//! zero UI edits - the report-data fields are asserted, not produced here).

use anyhow::Result;
use ariadne_graph::{build_dependency_graph, IncludeGraph, IncludeKind};
use ariadne_lang::{
    DependencyResolver, DirectiveStyle, FileCatalog, LanguageId, LanguagePlugin, LanguageRegistry,
    RawDepEdge, ResolutionContext, ResolveOutcome,
};
use ariadne_lang_api::modeling::CyclePolicy;
use ariadne_repo::{FileId, Repository, SourceLanguage};
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE: &str = "rust-basic";
const LIB_REL: &str = "src/lib.rs";
const GREETING_REL: &str = "src/greeting.rs";
const MODELS_MOD_REL: &str = "src/models/mod.rs";
const USER_REL: &str = "src/models/user.rs";
const VENDOR_REL: &str = "vendor/legacy/legacy.rs";

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

/// Throwaway copy of the frozen fixture (m19/m20 isolation - the fixture set
/// stays READ-ONLY; every synthetic file lands only inside the copy).
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

fn rel_of(graph: &IncludeGraph, id: FileId) -> Option<String> {
    graph
        .file_paths
        .get(&id)
        .map(|path| normalized_rel(path))
}

/// The production graph path: CLI default registry, marker-based shaper
/// selection (the cargo `Cargo.toml` at the fixture root picks the Rust
/// shaper - the R-4 suite locked that selection through this same seam).
fn rust_graph(root: &Path) -> Result<IncludeGraph> {
    let registry = ariadne_cli::registry::default_registry();
    build_dependency_graph(root, &registry)
}

/// Outgoing resolved edges of one file as (destination rel, line, column),
/// in edge order - the vec-equality shape the Red protocol predicts
/// (`left: []` against the stub's empty edge vec).
fn resolved_edges_of(graph: &IncludeGraph, source: FileId) -> Vec<(String, u32, u32)> {
    graph
        .edges
        .iter()
        .filter(|edge| edge.source == source && edge.destination.is_some())
        .map(|edge| {
            (
                rel_of(graph, edge.destination.expect("filtered above")).expect("registered"),
                edge.source_location.line,
                edge.source_location.column,
            )
        })
        .collect()
}

/// The registered plugin's resolver - the production dispatch target taken
/// through the CLI default registry (the R-4/R-5 suite's registry seam).
fn registered_resolver<'a>(registry: &'a ariadne_lang::LanguageRegistry) -> &'a dyn DependencyResolver {
    registry
        .plugin_for_id(LanguageId::Rust)
        .expect("Rust registered in the CLI default registry (R-4)")
        .resolver()
}

// ---------------------------------------------------------------------------
// 0. INV-5 / shape sec. 3.4 - the integration-side byte-exact flip lock
//    (m19 idiom, "EVOLVED at flip" authority; authored WITH the flip commit,
//    never red-in-isolation - journal D5, R-5 precedent)
// ---------------------------------------------------------------------------

#[test]
fn capabilities_lock_the_dependency_graph_flip_literal() {
    let registry = ariadne_cli::registry::default_registry();
    let rust = registry
        .plugin_for_id(LanguageId::Rust)
        .expect("Rust registered in the CLI default registry (R-4)");
    assert_eq!(
        rust.capabilities(),
        ariadne_lang::LanguageCapabilities {
            discovery: true,
            dependency_graph: true,
            semantic_index: true,
            load_metric: false,
            planning: false,
            extraction: false,
            rewriting: false,
            validation: false,
            requires_toolchain: false,
        },
        "the resolver hop turns dependency_graph on alongside discovery + \
         semantic_index and ONLY those three (INV-5 monotonicity); \
         requires_toolchain stays false at every hop (A3/D-2)"
    );
}

// ---------------------------------------------------------------------------
// 1-2. mod-tree truth through the REAL graph path, both layouts (shape 3.7)
// ---------------------------------------------------------------------------

#[test]
fn mod_tree_edges_both_layouts_carry_kind_and_spans() -> Result<()> {
    let repo = isolated_rust_repo("mod-tree")?;
    let graph = rust_graph(&repo.root)?;

    let lib = FileId::from_relative_path(LIB_REL);
    assert_eq!(
        resolved_edges_of(&graph, lib),
        vec![
            (GREETING_REL.to_string(), 6, 5),
            (MODELS_MOD_REL.to_string(), 7, 5),
        ],
        "`mod greeting;` resolves to the flat sibling and `mod models;` to the \
         mod.rs sibling, each at its written name-token span (the handoff's \
         fixture-form oracle; the stub emits nothing - Red `left: []`)"
    );
    let models_mod = FileId::from_relative_path(MODELS_MOD_REL);
    assert_eq!(
        resolved_edges_of(&graph, models_mod),
        vec![(USER_REL.to_string(), 4, 9)],
        "`pub mod user;` in the mod.rs sibling resolves inside its own directory"
    );
    for edge in graph
        .edges
        .iter()
        .filter(|edge| edge.source == lib || edge.source == models_mod)
    {
        assert_eq!(edge.kind, IncludeKind::Import, "one relation per directive (IV-1 mirror)");
        assert_eq!(edge.language, SourceLanguage::Rust.into());
        assert!(
            edge.raw_path == "greeting"
                || edge.raw_path == "models"
                || edge.raw_path == "user",
            "the raw spelling is the written module name, nothing invented: {}",
            edge.raw_path
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 3-4. the resolver seam itself: written facts verbatim (fixture + authored)
// ---------------------------------------------------------------------------

#[test]
fn resolver_emits_mod_edges_at_written_spans() -> Result<()> {
    let source = fs::read_to_string(fixture_path(FIXTURE).join(LIB_REL))?;
    let registry = ariadne_cli::registry::default_registry();
    let edges = registered_resolver(&registry).resolve_edges(
        FileId::from_relative_path(LIB_REL),
        Path::new(LIB_REL),
        &source,
        &ResolutionContext::default(),
    );
    let surface: Vec<(String, bool, u32, u32)> = edges
        .iter()
        .map(|edge| {
            (
                edge.raw_target.clone(),
                edge.is_wildcard,
                edge.line,
                edge.column,
            )
        })
        .collect();
    assert_eq!(
        surface,
        vec![
            ("greeting".to_string(), false, 6, 5),
            ("models".to_string(), false, 7, 5),
        ],
        "mod declarations ride the fixture bytes at their exact name-token \
         spans (INV-7 honored at the resolver seam; Red `left: []`)"
    );
    assert!(
        edges
            .iter()
            .all(|edge| matches!(edge.style, DirectiveStyle::Quoted)),
        "a `mod x;` declaration is the LOCAL file-relative directive spelling \
         - the style channel the neutral seam reserves to the plugin"
    );
    Ok(())
}

#[test]
fn resolver_emits_use_facts_verbatim_with_spans() -> Result<()> {
    // The R-5 use-grammar pinned at parser_uses: raw written path, leaf-name
    // span, glob at the STAR span, rename keeps the LEFT side, `::`-absolute
    // stays verbatim. Here the same facts must arrive as RawDepEdges.
    let source = "use crate::models::user::User;\n\
                  use crate::legacy::helper::*;\n\
                  use ::serde::Serialize;\n\
                  use crate::depth::leaf as alias;\n";
    let registry = ariadne_cli::registry::default_registry();
    let edges = registered_resolver(&registry).resolve_edges(
        FileId::from_relative_path(LIB_REL),
        Path::new(LIB_REL),
        source,
        &ResolutionContext::default(),
    );
    let surface: Vec<(String, bool, u32, u32)> = edges
        .iter()
        .map(|edge| {
            (
                edge.raw_target.clone(),
                edge.is_wildcard,
                edge.line,
                edge.column,
            )
        })
        .collect();
    assert_eq!(
        surface,
        vec![
            ("crate::models::user::User".to_string(), false, 1, 26),
            ("crate::legacy::helper::*".to_string(), true, 2, 28),
            ("::serde::Serialize".to_string(), false, 3, 14),
            ("crate::depth::leaf".to_string(), false, 4, 19),
        ],
        "the resolver consumes the analyzer's raw use-facts verbatim - no \
         rewriting, no invented alias edges (Red `left: []`)"
    );
    assert!(
        edges
            .iter()
            .all(|edge| matches!(edge.style, DirectiveStyle::Unspecified)),
        "`use` paths are package-style directives - no local/external spelling"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 5-6. the external boundary stance (out-of-repo resolves to NOTHING)
// ---------------------------------------------------------------------------

fn target_of(raw_target: &str, is_wildcard: bool, style: DirectiveStyle) -> ResolveOutcome {
    let registry = ariadne_cli::registry::default_registry();
    registered_resolver(&registry).resolve_target(
        &RawDepEdge {
            source_file: FileId::from_relative_path(LIB_REL),
            source_rel_path: PathBuf::from(LIB_REL),
            raw_target: raw_target.to_string(),
            is_wildcard,
            style,
            line: 1,
            column: 1,
        },
        &fixture_catalog(),
        &ResolutionContext::default(),
    )
}

fn fixture_catalog() -> FileCatalog {
    let repo = isolated_rust_repo("catalog").expect("fixture copy");
    let discovery = Repository::locate(&repo.root)
        .expect("fixture copy locates")
        .discover()
        .expect("fixture copy discovers");
    FileCatalog::from_records(&discovery.files)
}

#[test]
fn out_of_repo_use_paths_resolve_to_external() {
    for spelling in ["::serde::Serialize", "serde::Serialize", "anyhow::Result"] {
        assert_eq!(
            target_of(spelling, false, DirectiveStyle::Unspecified),
            ResolveOutcome::External,
            "an out-of-repo crate path is the honest boundary marker - never \
             a destination, never an invented edge (the handoff's stance \
             mirroring the C++ system-header / Scala-lib posture)"
        );
    }
}

#[test]
fn vendor_boundary_markers_resolve_external_without_lookup() {
    for marker in ["target", "vendor", ".cargo"] {
        assert_eq!(
            target_of(&format!("{marker}::legacy::Legacy"), false, DirectiveStyle::Unspecified),
            ResolveOutcome::External,
            "a marker first segment wins before any catalog lookup"
        );
    }
    assert_eq!(
        target_of("legacy::Legacy", false, DirectiveStyle::Unspecified),
        ResolveOutcome::External,
        "the fixture's only `legacy` file lives under vendor/ - the matched \
         record's marker component keeps it external (Scala's record rule)"
    );
}

// ---------------------------------------------------------------------------
// 7. the vendor side stays untouched by the graph (no unit, no edge)
// ---------------------------------------------------------------------------

#[test]
fn no_edges_touch_the_vendor_boundary() -> Result<()> {
    let repo = isolated_rust_repo("boundary")?;
    let graph = rust_graph(&repo.root)?;
    let vendor = FileId::from_relative_path(VENDOR_REL);
    assert!(
        graph.file_paths.contains_key(&vendor),
        "the vendor file stays a DISCOVERY node (the R-4 ownership lock \
         cites it as ThirdParty/Frozen) - what the resolver forbids is any \
         EDGE crossing the boundary, not discovery visibility"
    );
    for edge in &graph.edges {
        assert_ne!(edge.source, vendor, "no unit edges originate in vendor/");
        assert_ne!(edge.destination, Some(vendor), "no resolution targets vendor/");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 8. use-edges over fixture paths, end to end (temp-copy harness files only)
// ---------------------------------------------------------------------------

#[test]
fn use_edges_over_fixture_paths_point_at_files() -> Result<()> {
    let repo = isolated_rust_repo("use-edges")?;
    fs::write(
        repo.root.join("src").join("consumer.rs"),
        "use crate::greeting;\n\
         use crate::models::user::User;\n\
         use serde::Serialize;\n",
    )?;
    let graph = rust_graph(&repo.root)?;
    let consumer = FileId::from_relative_path("src/consumer.rs");

    assert_eq!(
        resolved_edges_of(&graph, consumer),
        vec![
            (GREETING_REL.to_string(), 1, 12),
            (USER_REL.to_string(), 2, 26),
        ],
        "`use crate::greeting;` names the module file; the symbol leaf of \
         `use crate::models::user::User;` is consumed and the longest written \
         module prefix names src/models/user.rs (Red `left: []`)"
    );

    let externals: Vec<_> = graph
        .edges
        .iter()
        .filter(|edge| edge.source == consumer && matches!(edge.kind, IncludeKind::External))
        .collect();
    assert_eq!(externals.len(), 1, "the registry path stays external");
    assert_eq!(externals[0].destination, None, "External never carries a destination");
    assert_eq!(externals[0].raw_path, "serde::Serialize");
    assert_eq!(
        (externals[0].source_location.line, externals[0].source_location.column),
        (3, 12)
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 9. RISK-CYCLE oracle - report-classification data, zero core edits
// ---------------------------------------------------------------------------

#[test]
fn legal_use_cycle_is_informational_in_report_data() -> Result<()> {
    let repo = isolated_rust_repo("risk-cycle")?;
    fs::write(
        repo.root.join("src").join("cycle_a.rs"),
        "use crate::cycle_b::b_marker;\n\npub fn a_marker() -> u32 {\n    b_marker()\n}\n",
    )?;
    fs::write(
        repo.root.join("src").join("cycle_b.rs"),
        "use crate::cycle_a::a_marker;\n\npub fn b_marker() -> u32 {\n    a_marker()\n}\n",
    )?;
    let graph = rust_graph(&repo.root)?;
    let cycle_a = FileId::from_relative_path("src/cycle_a.rs");
    let cycle_b = FileId::from_relative_path("src/cycle_b.rs");
    assert_import_edge(&graph, cycle_a, cycle_b);
    assert_import_edge(&graph, cycle_b, cycle_a);

    assert!(
        !graph.detect_cycle_groups().is_empty(),
        "the legal intra-crate cycle is DETECTED (policy-free, m20 idiom; the \
         stub has no edges so no cycle - the Red reason here)"
    );

    // The pipeline's cycle_policy_by_file lookup (pipeline/src/lib.rs:672-687)
    // re-walked from graph nodes through the SAME registry dispatch - the
    // existing data path, zero core edits: a file's language answers through
    // detection/plugin_for_id and the plugin modeling carries the stance.
    let registry = ariadne_cli::registry::default_registry();
    assert_eq!(
        registry
            .plugin_for_id(LanguageId::Rust)
            .expect("Rust registered")
            .modeling()
            .cycle_policy,
        CyclePolicy::TolerateWithinPackage,
        "the report side consumes the modeling stance as DATA"
    );
    let policy_of = |file_id: FileId| -> CyclePolicy {
        graph
            .file_paths
            .get(&file_id)
            .and_then(|path| registry.detect_language(path))
            .and_then(|id| registry.plugin_for_id(id))
            .map(|plugin| plugin.modeling().cycle_policy)
            .unwrap_or(CyclePolicy::Reject)
    };
    let tolerated = graph.generate_report_with_cycle_policies(&policy_of);
    assert!(tolerated.cycle_count >= 1);
    assert_eq!(
        tolerated.informational_cycle_count, tolerated.cycle_count,
        "every cycle is informational when every member file tolerates (IV-3 \
         mirror); the planner stance stays informational - the m18.7 SCC-\
         sharing lock `cycle_policy_variants_share_the_scc_coproposal_path_for_\
         now` stays green UNEDITED inside the census (planning capability is \
         false at this hop, so no planner runs on Rust)"
    );

    let blocking = graph.generate_report();
    assert_eq!(blocking.informational_cycle_count, 0, "the all-Reject delegate classifies nothing informational");
    assert_eq!(blocking.cycle_count, tolerated.cycle_count, "cycle_count keeps counting every detected cycle (m3 lock)");

    let blocking_json = serde_json::to_string(&blocking).expect("report serializes");
    assert!(!blocking_json.contains("informational_cycle_count"), "skip-when-zero keeps the pre-M20 artifact shape: {blocking_json}");
    let tolerated_json = serde_json::to_string(&tolerated).expect("report serializes");
    assert!(
        tolerated_json.contains("informational_cycle_count"),
        "the report-data field surfaces for the Rust cycle through the existing \
         serialization - no manifest or UI edit: {tolerated_json}"
    );
    Ok(())
}

fn assert_import_edge(graph: &IncludeGraph, source: FileId, destination: FileId) {
    let found = graph.edges.iter().any(|edge| {
        edge.source == source && edge.destination == Some(destination) && edge.kind == IncludeKind::Import
    });
    assert!(found, "expected an Import edge {} -> {}", source.0, destination.0);
}
