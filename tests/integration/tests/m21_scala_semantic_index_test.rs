//! M21 acceptance suite (plan:413 verbatim, kickoff acceptance #2): the
//! `--lang scala` production path — `build_source_index_filtered` through
//! T-C's filter-driven shaper seam (D15) — locks the D16/D17 symbol and
//! edge shapes, the D21 cold/warm/edit shard behavior, the language-scoped
//! cache digests, and offline determinism (A3).
//!
//! A3 by construction: this suite spawns zero processes (no
//! `std::process` anywhere), so no scalac/sbt can run even if a toolchain
//! were on PATH.

use anyhow::Result;
use ariadne_build::CompilationContext;
use ariadne_cli::commands::index::{
    build_source_index_filtered, handle_index_with_registry_filtered,
};
use ariadne_index::cache::TuCacheFingerprint;
use ariadne_index::{
    CURRENT_ANALYSIS_VERSION, DependencyKind, IndexResult, SymbolKind, SymbolRecord, TuAnalysis,
};
use ariadne_lang::{
    ClusteringConstraints, CompilerValidator, DependencyResolver, DependencyRewriter, LanguageId,
    LanguageModeling, LanguagePlugin, LanguageRegistry, SemanticAnalyzer, SourceExtractor,
    UnitShaper,
};
use ariadne_lang::{LanguageCapabilities, LanguageDetector};
use ariadne_lang_cpp::CppPlugin;
use ariadne_lang_scala::{ScalaAnalyzer, ScalaPlugin};
use ariadne_repo::{FileId, SourceLanguage};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const FIXTURE: &str = "scala-basic";

/// The five main-root units the D15-filtered shaping indexes (the test-root
/// spec stays excluded per the M19 shaper).
const MAIN_ROOT_RELS: [&str; 5] = [
    "src/main/scala/com/example/Main.scala",
    "src/main/scala/com/example/models/Order.scala",
    "src/main/scala/com/example/models/User.scala",
    "src/main/scala/com/example/models/all.scala",
    "src/main/scala/com/example/services/OrderService.scala",
];
const ORDER_REL: &str = MAIN_ROOT_RELS[1];
const ORDER_SERVICE_REL: &str = MAIN_ROOT_RELS[4];

/// D17 ids/kinds and the D16 wildcard-edge dst — the approved T-A corpus
/// shapes, asserted here, not relitigated.
const ORDER_SERVICE_ID: &str = "com.example.services.OrderService";
const ORDER_ID: &str = "com.example.models.Order";
const USER_ID: &str = "com.example.models.User";
const MAIN_ID: &str = "com.example.Main";
const PACKAGE_OBJECT_ID: &str = "com.example.models";
/// D27 fixture evolution (`import com.example.models._`): the FIRST active
/// wildcard owns attribution (D16), so the `ArrayBuffer` use is now guessed
/// qualified into `com.example.models` — the offline heuristic cannot see
/// that the second wildcard (`scala.collection.mutable._`) is its real home,
/// and it never fabricates a non-attributed edge.
const ARRAY_BUFFER_ID: &str = "com.example.models.ArrayBuffer";

/// Line holding the fixture's only `s"…${…}…"` interpolation (D18: splices
/// contribute no references, edges, or call sites).
const SPLICE_LINE: usize = 14;

struct IsolatedScalaRepo {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fixtures")
        .join(name)
}

/// Throwaway copy of the frozen fixture (m19/m20 isolation pattern — every
/// test owns its tempdir and all edits happen inside the copy).
fn isolated_scala_repo(label: &str) -> Result<IsolatedScalaRepo> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join(label);
    copy_tree(&fixture_path(FIXTURE), &root)?;
    Ok(IsolatedScalaRepo { _temp: temp, root })
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

/// The `--lang scala` CLI path through the T-C seam — the milestone's
/// product entry point (`cli/commands/index.rs:86`).
fn index_scala(root: &Path) -> Result<IndexResult> {
    build_source_index_filtered(&root.to_path_buf(), Some(SourceLanguage::Scala))
}

// ---------------------------------------------------------------------------
// Shard store helpers — the shard -> source mapping mechanism.
//
// A shard envelope is `artifacts/shards/{file_id.0}.json`
// (`scheduler/src/tu_store.rs:37-39`, store dir from `tu_store.rs:33-35`
// joined onto `repo_root` at `pipeline/src/lib.rs:465`), and `FileId` is the
// deterministic SHA-256 prefix of the normalized repo-relative path
// (`repo/src/types.rs:15-21`, produced from exactly those paths at
// `repo/src/discovery.rs:58-60`). Therefore `FileId::from_relative_path`
// reconstructs every shard's identity across runs of the same tree with no
// mtime or write-order assumptions; the envelope's embedded
// `analysis.tu_file_id` (used by `save_shard`, `tu_store.rs:49-51`) is the
// same value, which the warm test below cross-checks.
// ---------------------------------------------------------------------------

fn shard_file_name(rel_path: &str) -> String {
    format!("{}.json", FileId::from_relative_path(rel_path).0)
}

/// Raw bytes of every `artifacts/shards/*.json` envelope, keyed by file name
/// (the write-tmp files end in `.tmp` and are excluded).
fn read_shard_envelopes(root: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let dir = root.join("artifacts").join("shards");
    let mut envelopes = BTreeMap::new();
    if dir.is_dir() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".json") {
                envelopes.insert(name, fs::read(entry.path())?);
            }
        }
    }
    Ok(envelopes)
}

fn envelope<'a>(envelopes: &'a BTreeMap<String, Vec<u8>>, rel: &str) -> &'a [u8] {
    let name = shard_file_name(rel);
    envelopes
        .get(&name)
        .unwrap_or_else(|| panic!("shard envelope for {rel} (expected {name}) exists"))
}

fn envelope_fingerprint(bytes: &[u8]) -> Result<String> {
    let value: serde_json::Value = serde_json::from_slice(bytes)?;
    value["fingerprint"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("shard envelope lacks a fingerprint field"))
        .map(str::to_string)
}

fn envelope_tu_file_id(bytes: &[u8]) -> Result<u64> {
    let value: serde_json::Value = serde_json::from_slice(bytes)?;
    value["analysis"]["tu_file_id"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("shard envelope lacks analysis.tu_file_id"))
}

// ---------------------------------------------------------------------------
// D21 observability: a counting analyzer injected through a bespoke
// registry (handoff-authorized, zero src change). It wraps the real
// ScalaAnalyzer verbatim, so shard fingerprints and merged analyses are
// production output — the only addition is the invocation counter.
// ---------------------------------------------------------------------------

struct CountingAnalyzer {
    inner: ScalaAnalyzer,
    calls: Arc<AtomicUsize>,
}

impl SemanticAnalyzer for CountingAnalyzer {
    fn analyze(&self, file_id: FileId, source: &str, context: &str) -> TuAnalysis {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.inner.analyze(file_id, source, context)
    }
}

/// Scala's plugin entry verbatim except `analyzer()`; with C++ registered
/// first the registry enumeration matches the CLI default exactly.
struct CountingScalaPlugin {
    inner: ScalaPlugin,
    counting: CountingAnalyzer,
}

impl CountingScalaPlugin {
    fn new(calls: Arc<AtomicUsize>) -> Self {
        Self {
            inner: ScalaPlugin::new(),
            counting: CountingAnalyzer {
                inner: ScalaAnalyzer::new(),
                calls,
            },
        }
    }
}

impl LanguagePlugin for CountingScalaPlugin {
    fn detector(&self) -> &dyn LanguageDetector {
        self.inner.detector()
    }
    fn analyzer(&self) -> &dyn SemanticAnalyzer {
        &self.counting
    }
    fn resolver(&self) -> &dyn DependencyResolver {
        self.inner.resolver()
    }
    fn constraints(&self) -> &dyn ClusteringConstraints {
        self.inner.constraints()
    }
    fn extractor(&self) -> &dyn SourceExtractor {
        self.inner.extractor()
    }
    fn rewriter(&self) -> &dyn DependencyRewriter {
        self.inner.rewriter()
    }
    fn validator(&self) -> &dyn CompilerValidator {
        self.inner.validator()
    }
    fn id(&self) -> LanguageId {
        self.inner.id()
    }
    fn modeling(&self) -> LanguageModeling {
        self.inner.modeling()
    }
    fn capabilities(&self) -> LanguageCapabilities {
        self.inner.capabilities()
    }
    fn analysis_fingerprint(&self) -> String {
        self.inner.analysis_fingerprint()
    }
    fn shaper(&self) -> Option<&dyn UnitShaper> {
        self.inner.shaper()
    }
}

fn counting_registry(calls: Arc<AtomicUsize>) -> Arc<LanguageRegistry> {
    Arc::new(
        LanguageRegistry::new()
            .register(Box::new(CppPlugin::new()))
            .register(Box::new(CountingScalaPlugin::new(calls))),
    )
}

// ---------------------------------------------------------------------------
// Index-content helpers
// ---------------------------------------------------------------------------

fn symbol_in_index<'a>(result: &'a IndexResult, symbol_id: &str) -> Option<&'a SymbolRecord> {
    result
        .tu_analyses
        .iter()
        .flat_map(|tu| tu.symbols.iter())
        .find(|symbol| symbol.id.0 == symbol_id)
}

fn has_edge(result: &IndexResult, src: &str, dst: &str, kind: DependencyKind) -> bool {
    result.tu_analyses.iter().flat_map(|tu| tu.edges.iter()).any(|edge| {
        edge.kind == kind && edge.src_symbol.0 == src && edge.dst_symbol.0 == dst
    })
}

// ---------------------------------------------------------------------------
// 1. plan:413 literal — symbols and Complete edges through the product path
// ---------------------------------------------------------------------------

#[test]
fn filtered_index_locks_symbols_and_complete_edges() -> Result<()> {
    let repo = isolated_scala_repo("symbols")?;
    let result = index_scala(&repo.root)?;

    assert_eq!(
        result.tu_analyses.len(),
        5,
        "the five main-root units only — the test root stays excluded per the M19 shaper"
    );
    for tu in &result.tu_analyses {
        assert_eq!(tu.language, SourceLanguage::Scala, "every unit is Scala-stamped");
    }

    let service = symbol_in_index(&result, ORDER_SERVICE_ID)
        .expect("plan:413: com.example.services.OrderService is in the index");
    assert_eq!(service.kind, SymbolKind::Class, "D17: class -> Class");

    // D27 fixture evolution: the grouped import became
    // `import com.example.models._`, so the two named-usage edges ride the
    // active wildcard and carry D16's Incomplete kind (the plan:413
    // "edges to Order and User" coverage is unchanged, the kind moved).
    for dest in [USER_ID, ORDER_ID] {
        assert!(
            has_edge(&result, ORDER_SERVICE_ID, dest, DependencyKind::Incomplete),
            "plan:413: wildcard edge {ORDER_SERVICE_ID} -> {dest} exists in the index"
        );
    }

    let d17_shapes: [(&str, SymbolKind); 4] = [
        (MAIN_ID, SymbolKind::Object),
        (USER_ID, SymbolKind::Class),
        (ORDER_ID, SymbolKind::Class),
        (PACKAGE_OBJECT_ID, SymbolKind::Object),
    ];
    for (id, kind) in d17_shapes {
        let record = symbol_in_index(&result, id)
            .unwrap_or_else(|| panic!("D17: symbol {id} is in the index"));
        assert_eq!(record.kind, kind, "D17 kind for {id}");
    }

    assert!(
        has_edge(
            &result,
            ORDER_SERVICE_ID,
            ARRAY_BUFFER_ID,
            DependencyKind::Incomplete
        ),
        "D16: the wildcard-fed ArrayBuffer usage yields an Incomplete edge to the qualified guess"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 2. Cold / warm (D21) — witness: counted analyzer invocations (stronger
//    option the handoff allowed) AND envelope byte identity. The second run
//    invokes the plugin analyzer ZERO times; the five envelopes are never
//    rewritten.
// ---------------------------------------------------------------------------

#[test]
fn warm_run_reuses_every_unit_with_zero_analyzer_invocations() -> Result<()> {
    let repo = isolated_scala_repo("warm")?;
    let calls = Arc::new(AtomicUsize::new(0));
    let registry = counting_registry(Arc::clone(&calls));

    let cold = handle_index_with_registry_filtered(
        &repo.root,
        Arc::clone(&registry),
        Some(SourceLanguage::Scala),
    )?;
    assert!(
        calls.load(Ordering::SeqCst) > 0,
        "a cold run must actually invoke the plugin analyzer"
    );

    let first = read_shard_envelopes(&repo.root)?;
    let mut expected_names: Vec<String> = MAIN_ROOT_RELS.iter().map(|r| shard_file_name(r)).collect();
    expected_names.sort();
    let actual_names: Vec<String> = first.keys().cloned().collect();
    assert_eq!(
        actual_names, expected_names,
        "one envelope per unit, named {{FileId::from_relative_path(rel).0}}.json \
         (tu_store.rs:37-39 + repo/src/types.rs:15-21)"
    );
    for rel in MAIN_ROOT_RELS {
        let bytes = envelope(&first, rel);
        let mapped = envelope_tu_file_id(bytes)?;
        assert_eq!(
            mapped,
            FileId::from_relative_path(rel).0,
            "the envelope's embedded tu_file_id matches the path-derived FileId \
             for {rel} — bidirectional shard mapping witness"
        );
    }

    calls.store(0, Ordering::SeqCst);
    let warm = handle_index_with_registry_filtered(
        &repo.root,
        Arc::clone(&registry),
        Some(SourceLanguage::Scala),
    )?;

    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "D21 warm run: the analyzer is never invoked — all five units are reused from shards"
    );
    assert_eq!(
        warm.total_symbols, cold.total_symbols,
        "the reused analyses are content-identical to the cold run's"
    );
    assert_eq!(
        read_shard_envelopes(&repo.root)?,
        first,
        "all five envelopes are byte-identical after the warm run (never rewritten)"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 3. Edit reanalysis (D21) — a content-only edit to the copy's
//    models/Order.scala re-stamps exactly {Order.scala, OrderService.scala}
//    (OrderService's closure contains Order via the M20 import edge) and
//    leaves {Main, User, all} byte-identical to the first run.
// ---------------------------------------------------------------------------

#[test]
fn order_edit_reanalyzes_exactly_its_import_dependents() -> Result<()> {
    let repo = isolated_scala_repo("edit")?;
    index_scala(&repo.root)?;
    let baseline = read_shard_envelopes(&repo.root)?;
    assert_eq!(baseline.len(), 5, "cold run wrote exactly five envelopes");

    let order_path = repo.root.join(ORDER_REL);
    let mut order_source = fs::read_to_string(&order_path)?;
    order_source.push_str("\n// warmup edit\n");
    fs::write(&order_path, order_source)?;

    let result = index_scala(&repo.root)?;
    assert_eq!(result.tu_analyses.len(), 5, "still five units after the edit");
    assert!(
        has_edge(&result, ORDER_SERVICE_ID, ORDER_ID, DependencyKind::Incomplete),
        "the reanalyzed index still carries the plan:413 edge (Incomplete kind per the \
         D27 fixture evolution — grouped import replaced by `import com.example.models._`)"
    );

    let after = read_shard_envelopes(&repo.root)?;
    assert_eq!(after.len(), 5, "the edit run adds no new shard files");
    let reanalyzed: [&str; 2] = [ORDER_REL, ORDER_SERVICE_REL];
    for rel in MAIN_ROOT_RELS {
        let before = envelope(&baseline, rel);
        let current = envelope(&after, rel);
        let fingerprint_moved = envelope_fingerprint(before)? != envelope_fingerprint(current)?;
        if reanalyzed.contains(&rel) {
            assert!(
                fingerprint_moved,
                "{rel}: its own content or a reached import changed, so the envelope \
                 fingerprint must be new (D21 import-dependents)"
            );
        } else {
            assert!(
                !fingerprint_moved && before == current,
                "{rel}: unrelated to the edited file — envelope byte-identical to run 1"
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. Language-scoped cache digest (plan:413 last clause, D19/M18.10)
// ---------------------------------------------------------------------------

#[test]
fn scala_fingerprint_moves_only_the_scala_digest() -> Result<()> {
    // Test-only stand-in toolchain name (no real compiler is named; A3).
    let context = CompilationContext {
        name: "root".into(),
        compiler: "fake-toolchain".into(),
        target_arch: None,
        language_standard: None,
        defines: vec![],
        include_dirs: vec![],
        system_include_dirs: vec![],
        forced_includes: vec![],
        raw_flags: vec![],
        language: Default::default(),
    };
    let file_id = FileId::from_relative_path(ORDER_SERVICE_REL);
    let headers: Vec<(FileId, String)> = vec![];

    let digest = |language: SourceLanguage, plugin_fingerprint: &str| {
        TuCacheFingerprint::compute(
            CURRENT_ANALYSIS_VERSION,
            &context,
            &file_id,
            "source-hash",
            &headers,
            Some("config"),
            language,
            plugin_fingerprint,
        )
    };

    let cpp = digest(SourceLanguage::Cpp, "cpp-analyzer-1");
    // M23 evolution (D-M23-8, same-commit authority as the plugin bump): the
    // live Scala fingerprint literal moved "scala-parser-1" -> "scala-parser-2"
    // because Scala sources now persist modifiers + the typed extends channel.
    let scala = digest(SourceLanguage::Scala, "scala-parser-2");
    assert_ne!(
        cpp.combined_hash, scala.combined_hash,
        "plan:413: one store cannot poison entries across languages"
    );

    let scala_other = digest(SourceLanguage::Scala, "scala-other");
    let cpp_again = digest(SourceLanguage::Cpp, "cpp-analyzer-1");
    assert_ne!(
        scala.combined_hash, scala_other.combined_hash,
        "a Scala fingerprint bump must move the Scala digest"
    );
    assert_eq!(
        cpp, cpp_again,
        "the C++ digest recomputes IDENTICAL while Scala's fingerprint varies — the \
         languages are digest-decoupled (D19 blast radius)"
    );
    assert_eq!(CURRENT_ANALYSIS_VERSION, "2.0.0", "M18.10 cache epoch witness");
    Ok(())
}

// ---------------------------------------------------------------------------
// 5. Offline determinism (A3) — the real fixture's OrderService.scala
//    analyzed twice through the CLI default registry's plugin. No test in
//    this file shells out (no std::process import at all): scalac/sbt are
//    unreachable by construction.
// ---------------------------------------------------------------------------

#[test]
fn order_service_analysis_is_deterministic_offline() -> Result<()> {
    let registry = ariadne_cli::registry::default_registry();
    let service_path = Path::new(ORDER_SERVICE_REL);
    assert_eq!(
        registry.detect_language(service_path),
        Some(SourceLanguage::Scala),
        "the CLI registry claims the .scala path"
    );
    let plugin = registry
        .plugin_for_id(SourceLanguage::Scala)
        .expect("the Scala plugin is registered");

    let source = fs::read_to_string(fixture_path(FIXTURE).join(ORDER_SERVICE_REL))?;
    let file_id = FileId::from_relative_path(ORDER_SERVICE_REL);

    let first = plugin.analyzer().analyze(file_id, &source, "scala-basic");
    let second = plugin.analyzer().analyze(file_id, &source, "scala-basic");
    assert_eq!(
        serde_json::to_string(&first)?,
        serde_json::to_string(&second)?,
        "A3: identical inputs produce byte-identical analyses offline"
    );
    assert_eq!(first.language, SourceLanguage::Scala);

    // T-A corpus shapes (locked in lang-scala, mirrored not relitigated).
    let symbol_ids: Vec<String> = first.symbols.iter().map(|s| s.id.0.clone()).collect();
    assert_eq!(symbol_ids, vec![ORDER_SERVICE_ID]);
    let edge_keys: Vec<(String, String, DependencyKind)> = first
        .edges
        .iter()
        .map(|e| (e.src_symbol.0.clone(), e.dst_symbol.0.clone(), e.kind))
        .collect();
    // D27 fixture evolution (`import com.example.models._` replaced the
    // grouped import): all three usages ride the single active wildcard, so
    // every edge is D16 Incomplete with its qualified guess — ArrayBuffer
    // now guesses into the first wildcard's package (see ARRAY_BUFFER_ID).
    assert_eq!(
        edge_keys,
        vec![
            (ORDER_SERVICE_ID.into(), ARRAY_BUFFER_ID.into(), DependencyKind::Incomplete),
            (ORDER_SERVICE_ID.into(), USER_ID.into(), DependencyKind::Incomplete),
            (ORDER_SERVICE_ID.into(), ORDER_ID.into(), DependencyKind::Incomplete),
        ],
        "exactly the three wildcard-attributed Incomplete edges, source order"
    );

    assert_eq!(
        first.call_sites.len(),
        0,
        "ArrayBuffer[Int]() and def-headed lines must not fabricate call sites"
    );
    assert!(
        !first.references.iter().any(|r| r.location.line == SPLICE_LINE),
        "D18: no reference may originate inside the s-interpolation of summarize"
    );
    assert!(
        !first.call_sites.iter().any(|c| c.location.line == SPLICE_LINE),
        "D18: no call site may originate inside the s-interpolation of summarize"
    );
    Ok(())
}
