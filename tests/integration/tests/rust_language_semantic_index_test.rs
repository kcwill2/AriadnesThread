//! rust-language R-5 semantic-index lock (milestone-named per the guide
//! Step-5 rule - the track key `rust_language_`, never an mNN name). Model
//! suite: `m21_scala_semantic_index_test.rs` (m21 idiom: fixture-driven
//! symbol/edge truth + the cold/warm shard-witness pair); the production
//! index path is `build_source_index_filtered`, the plugin analyzer is taken
//! from the registered plugin (no lang-rust import needed by this package).
//!
//! What this suite locks at the analyzer hop:
//! INV-8 the `"rust-1"` fingerprint from the registered plugin side;
//! shape sec. 3.7 symbol-surface truth on `rust-basic` (written items only,
//! qualified ids); INV-7 SPAN TRUTH - exact 1-based name-token line/column
//! literals (this is the test that fails if `span-locations` vanishes);
//! D-2 macro honesty - no symbol without written source; the m21 cold/warm
//! cache idiom end to end (envelope-per-unit, cache-served reuse, envelopes
//! never rewritten).

use anyhow::Result;
use ariadne_cli::commands::index::build_source_index_filtered;
use ariadne_cli::registry::default_registry;
use ariadne_index::IndexResult;
use ariadne_lang::{LanguageId, SymbolKind, TuAnalysis};
use ariadne_repo::{FileId, SourceLanguage};
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURE: &str = "rust-basic";
const LIB_REL: &str = "src/lib.rs";
const GREETING_REL: &str = "src/greeting.rs";
const MODELS_REL: &str = "src/models/mod.rs";
const USER_REL: &str = "src/models/user.rs";
const MACRO_REL: &str = "src/macro_specimen.rs";

/// The five `src/**.rs` units the shaper turns into translation units
/// (R-4 discovery suite carries the same const set - cited, not edited).
const SRC_UNITS: [&str; 5] = [LIB_REL, GREETING_REL, MODELS_REL, USER_REL, MACRO_REL];

/// Sentinel id planted into one cached envelope: if the warm run surfaces
/// it, the analysis was served from the shard envelope, never re-parsed.
const CACHE_SENTINEL: &str = "sentinel_from_cached_envelope";

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fixtures")
        .join(name)
}

struct IsolatedRustRepo {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

/// Throwaway copy of the frozen fixture (m19/m21 isolation pattern - the
/// frozen fixture stays READ-ONLY; every envelope mutation happens inside
/// the copy).
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

/// The registered plugin's analyzer - the production dispatch target, taken
/// through the CLI default registry (the R-4 suite's registry seam).
fn analyze_registered(rel: &str) -> Result<TuAnalysis> {
    let source = fs::read_to_string(fixture_path(FIXTURE).join(rel))?;
    let file_id = FileId::from_relative_path(rel);
    let registry = default_registry();
    let plugin = registry
        .plugin_for_id(LanguageId::Rust)
        .expect("the Rust plugin is registered (R-4)");
    Ok(plugin.analyzer().analyze(file_id, &source, FIXTURE))
}

fn symbol_surface(analysis: &TuAnalysis) -> Vec<(String, SymbolKind)> {
    analysis
        .symbols
        .iter()
        .map(|record| (record.id.0.clone(), record.kind))
        .collect()
}

/// (qualified id, line, column) per declaration - the INV-7 span channel.
fn declaration_spans(analysis: &TuAnalysis) -> Vec<(String, usize, usize)> {
    analysis
        .declarations
        .iter()
        .map(|decl| {
            (
                decl.symbol_id.0.clone(),
                decl.location.line,
                decl.location.column,
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Shard envelope helpers (m21 mechanism, cited not edited)
// ---------------------------------------------------------------------------

fn shard_file_name(rel_path: &str) -> String {
    format!("{}.json", FileId::from_relative_path(rel_path).0)
}

fn envelope_path(root: &Path, rel: &str) -> PathBuf {
    root.join("artifacts").join("shards").join(shard_file_name(rel))
}

fn envelope_bytes(root: &Path, rel: &str) -> Vec<u8> {
    fs::read(envelope_path(root, rel)).expect("a shard envelope exists for the unit")
}

fn index_rust(root: &Path) -> Result<IndexResult> {
    build_source_index_filtered(&root.to_path_buf(), Some(SourceLanguage::Rust))
}

fn index_carries_symbol(result: &IndexResult, symbol_id: &str) -> bool {
    result
        .tu_analyses
        .iter()
        .flat_map(|tu| tu.symbols.iter())
        .any(|record| record.id.0 == symbol_id)
}

// ---------------------------------------------------------------------------
// 1. INV-8 - the fingerprint from the registered-plugin side
// ---------------------------------------------------------------------------

#[test]
fn registered_plugin_fingerprint_is_rust_1() {
    let registry = default_registry();
    let plugin = registry
        .plugin_for_id(LanguageId::Rust)
        .expect("Rust registered in the CLI default registry (R-4)");
    assert_eq!(
        plugin.analysis_fingerprint(),
        "rust-1",
        "INV-8: the fingerprint literal rides the first analyzer-shipping commit; \
         C++/Scala fingerprint literals are separate, unedited digest inputs"
    );
}

// ---------------------------------------------------------------------------
// 2. INV-5 / shape sec. 3.4 - the integration-side byte-exact flip lock
//    (m19 `plugin_contract_in_default_registry` idiom, "EVOLVED at flip"
//    authority; authored with the flip commit, never red-in-isolation)
// ---------------------------------------------------------------------------

#[test]
fn capabilities_lock_the_semantic_index_flip_literal() {
    let registry = default_registry();
    let rust = registry
        .plugin_for_id(LanguageId::Rust)
        .expect("Rust registered in the CLI default registry (R-4)");
    assert_eq!(
        rust.capabilities(),
        ariadne_lang::LanguageCapabilities {
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
        "full parity: all eight stage bits true, every one behind a shipped \
         component (INV-5 monotonicity); requires_toolchain stays false at \
         every hop (A3/D-2)"
    );
}

// ---------------------------------------------------------------------------
// 2-5. shape sec. 3.7 symbol-surface truth, per written fixture file
// ---------------------------------------------------------------------------

#[test]
fn lib_rs_locks_the_crate_root_surface() -> Result<()> {
    let analysis = analyze_registered(LIB_REL)?;
    assert_eq!(analysis.tu_file_id, FileId::from_relative_path(LIB_REL));
    assert_eq!(analysis.context_name, FIXTURE, "context is stamped verbatim");
    assert_eq!(
        analysis.language,
        SourceLanguage::Rust,
        "every analysis carries its own language stamp (Scala-m21 idiom)"
    );
    assert_eq!(
        symbol_surface(&analysis),
        vec![
            ("crate::greeting".to_string(), SymbolKind::Module),
            ("crate::models".to_string(), SymbolKind::Module),
            ("crate::fixture_marker".to_string(), SymbolKind::Function),
        ],
        "source-order records for the two declared modules and the fn"
    );
    Ok(())
}

#[test]
fn sibling_module_file_locks_its_written_surface() -> Result<()> {
    let analysis = analyze_registered(GREETING_REL)?;
    assert_eq!(
        symbol_surface(&analysis),
        vec![("crate::greeting".to_string(), SymbolKind::Function)],
        "the flat-sibling file contributes exactly its written fn"
    );
    Ok(())
}

#[test]
fn mod_rs_sibling_locks_its_written_surface() -> Result<()> {
    let analysis = analyze_registered(MODELS_REL)?;
    assert_eq!(
        symbol_surface(&analysis),
        vec![
            ("crate::user".to_string(), SymbolKind::Module),
            ("crate::models_marker".to_string(), SymbolKind::Function),
        ],
        "the mod.rs sibling contributes its declared module and its fn"
    );
    Ok(())
}

#[test]
fn nested_user_module_locks_its_written_surface() -> Result<()> {
    let analysis = analyze_registered(USER_REL)?;
    assert_eq!(
        symbol_surface(&analysis),
        vec![
            ("crate::User".to_string(), SymbolKind::Struct),
            ("crate::default_user".to_string(), SymbolKind::Function),
        ],
        "the derive attribute does not hide the struct; methods of nothing \
         are not fabricated"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 6. INV-7 SPAN TRUTH - exact literals, the span-locations vanishing oracle
// ---------------------------------------------------------------------------

#[test]
fn fixture_spans_carry_exact_line_and_column() -> Result<()> {
    // 1-based name-token positions read off the frozen fixture bytes (the
    // reviewer re-reads these files): line is `Span::start().line`, column is
    // `Span::start().column + 1` - the pinned tokenizer convention.
    let lib_spans: Vec<(String, usize, usize)> = vec![
        ("crate::greeting".to_string(), 6, 5),
        ("crate::models".to_string(), 7, 5),
        ("crate::fixture_marker".to_string(), 9, 8),
    ];
    assert_eq!(
        declaration_spans(&analyze_registered(LIB_REL)?),
        lib_spans,
        "INV-7: the `mod models;` declaration and every sibling sit at their \
         literal written position (the handoff's named oracle)"
    );
    assert_eq!(
        declaration_spans(&analyze_registered(GREETING_REL)?),
        vec![("crate::greeting".to_string(), 4, 8)]
    );
    assert_eq!(
        declaration_spans(&analyze_registered(MODELS_REL)?),
        vec![
            ("crate::user".to_string(), 4, 9),
            ("crate::models_marker".to_string(), 6, 8),
        ]
    );
    assert_eq!(
        declaration_spans(&analyze_registered(USER_REL)?),
        vec![
            ("crate::User".to_string(), 4, 12),
            ("crate::default_user".to_string(), 8, 8),
        ],
        "the name span is attribute-immune: the line-3 derive never moves it"
    );
    assert_eq!(
        declaration_spans(&analyze_registered(MACRO_REL)?),
        vec![("crate::written_code_only".to_string(), 20, 8)]
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 7. D-2 macro honesty - no symbol without written source
// ---------------------------------------------------------------------------

#[test]
fn macro_specimen_surfaces_only_written_items() -> Result<()> {
    let analysis = analyze_registered(MACRO_REL)?;
    assert_eq!(
        symbol_surface(&analysis),
        vec![("crate::written_code_only".to_string(), SymbolKind::Function)],
        "the specimen's only surface is its written fn: neither the \
         macro_rules! definition nor its invocation may fabricate the \
         generated `mod generated_by_macro` or its inner fn"
    );
    let mut written_ids: Vec<&str> = analysis
        .symbols
        .iter()
        .map(|record| record.id.0.as_str())
        .collect();
    written_ids.extend(
        analysis
            .declarations
            .iter()
            .map(|decl| decl.symbol_id.0.as_str()),
    );
    let fabricated = written_ids
        .iter()
        .any(|text| text.contains("generated_by_macro") || text.contains("invisible_without_expansion"));
    assert!(
        !fabricated,
        "macro-generated content must never surface as a symbol (D-2 caveat)"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 8-10. the m21 cold/warm cache idiom on the production path
// ---------------------------------------------------------------------------

#[test]
fn cold_index_writes_one_envelope_per_unit() -> Result<()> {
    let repo = isolated_rust_repo("cold")?;
    let result = index_rust(&repo.root)?;
    assert_eq!(result.tu_analyses.len(), SRC_UNITS.len(), "five units");
    for tu in &result.tu_analyses {
        assert_eq!(tu.language, SourceLanguage::Rust);
    }
    assert!(
        index_carries_symbol(&result, "crate::fixture_marker"),
        "the production index path must surface the written fixture surface \
         (fails on the stub's empty analyses - the R-5 flip is what dispatches)"
    );
    for rel in SRC_UNITS {
        let bytes = envelope_bytes(&repo.root, rel);
        let value: serde_json::Value = serde_json::from_slice(&bytes)?;
        let embedded = value["analysis"]["tu_file_id"]
            .as_u64()
            .expect("the envelope embeds its tu_file_id (m21 idiom)");
        assert_eq!(
            embedded,
            FileId::from_relative_path(rel).0,
            "the envelope's embedded tu_file_id matches the path-derived \
             FileId for {rel} - bidirectional shard mapping witness"
        );
    }
    Ok(())
}

#[test]
fn warm_reindex_of_unchanged_bytes_is_cache_served() -> Result<()> {
    let repo = isolated_rust_repo("warm")?;
    let cold = index_rust(&repo.root)?;
    assert_eq!(cold.tu_analyses.len(), SRC_UNITS.len());

    // Plant the sentinel INSIDE the cached envelope (fingerprint untouched):
    // only a run that serves the envelope instead of re-parsing the source
    // can ever surface it. The host is `macro_specimen.rs`, NOT `user.rs`:
    // since R-6 the resolver's mod-tree edges merge declared children into
    // parent envelopes, so a planted id in a reachable file would ADD a
    // unique symbol (the original id survives in the child envelope) and
    // break the count-equality witness below. `macro_specimen.rs` is
    // declared nowhere in the fixture, is never merged, and holds exactly
    // one written symbol - the sentinel replaces it in place (R-6 rider,
    // merged-closure truth; the pre-R-6 count held on any unit).
    let path = envelope_path(&repo.root, MACRO_REL);
    let mut envelope: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
    let surface_written = envelope["analysis"]["symbols"]
        .as_array()
        .is_some_and(|symbols| !symbols.is_empty());
    assert!(
        surface_written,
        "the cold envelope for {MACRO_REL} carries the written symbols the \
         sentinel can ride (fails on the stub's empty surface - Red reason)"
    );
    envelope["analysis"]["symbols"][0]["id"] =
        serde_json::Value::String(CACHE_SENTINEL.to_string());
    fs::write(&path, serde_json::to_vec(&envelope)?)?;

    let warm = index_rust(&repo.root)?;
    assert!(
        index_carries_symbol(&warm, CACHE_SENTINEL),
        "the warm pass on UNCHANGED bytes served the stored envelope - a \
         re-analysis would have re-parsed and lost the sentinel"
    );
    assert_eq!(
        warm.total_symbols, cold.total_symbols,
        "the reused analyses are content-identical apart from the planted id"
    );
    Ok(())
}

#[test]
fn warm_reindex_never_rewrites_envelope_bytes() -> Result<()> {
    let repo = isolated_rust_repo("reuse")?;
    index_rust(&repo.root)?;
    let before: Vec<(String, Vec<u8>)> = SRC_UNITS
        .iter()
        .map(|rel| (rel.to_string(), envelope_bytes(&repo.root, rel)))
        .collect();

    let warm = index_rust(&repo.root)?;
    assert_eq!(warm.tu_analyses.len(), SRC_UNITS.len());
    for (rel, bytes) in before {
        assert_eq!(
            envelope_bytes(&repo.root, &rel),
            bytes,
            "{rel}: every envelope is byte-identical after the warm run \
             (reused shards are never rewritten - m21's witness)"
        );
    }
    Ok(())
}
