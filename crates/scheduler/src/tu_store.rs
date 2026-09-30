//! Unified, content-keyed TU analysis store.
//!
//! One persistent store (`artifacts/shards/{file_id}.json`) shared by every
//! producer (single-thread `analyze`, `index`, parallel workers, UI). Entries
//! are envelope-checked: `ShardCheckpoint { fingerprint, analysis }` keyed by
//! the combined `TuCacheFingerprint` hash, so edits invalidate exactly the
//! translation units whose closure changed.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ariadne_build::{CompilationContext, TranslationUnit};
use ariadne_index::{TuAnalysis, TuCacheFingerprint, CURRENT_ANALYSIS_VERSION};
use ariadne_lang::{
    DependencyResolver, FileCatalog, LanguagePlugin, RawDepEdge, ResolutionContext, ResolveOutcome,
    SemanticAnalyzer,
};
use ariadne_repo::{compute_file_hash, FileId, RepositoryDiscoveryResult, SourceLanguage};

/// Envelope written to the shard store. Missing or mismatched fingerprint
/// means miss; pre-envelope plain-`TuAnalysis` shards fail deserialization
/// and are treated as stale (one-time full rebuild).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardCheckpoint {
    pub fingerprint: String,
    pub analysis: TuAnalysis,
}

pub fn shard_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts").join("shards")
}

fn shard_file(dir: &Path, file_id: FileId) -> PathBuf {
    dir.join(format!("{}.json", file_id.0))
}

/// Load a shard envelope; anything unreadable (absent, corrupt, old plain
/// format) is a miss rather than an error.
pub fn load_shard(dir: &Path, file_id: FileId) -> Option<ShardCheckpoint> {
    let path = shard_file(dir, file_id);
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_shard(dir: &Path, checkpoint: &ShardCheckpoint) -> Result<()> {
    fs::create_dir_all(dir)?;
    let path = shard_file(dir, checkpoint.analysis.tu_file_id);
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let tmp = dir.join(format!(
        ".{}.{}.{}.tmp",
        checkpoint.analysis.tu_file_id.0,
        std::process::id(),
        id
    ));
    let json = serde_json::to_string(&checkpoint)?;
    fs::write(&tmp, json)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Shared inputs every TU analysis needs: discovery for content hashes and
/// compilation contexts for include resolution. Discovery is additionally
/// pre-indexed by FileId so per-header lookups are O(1) instead of a linear
/// scan of the file list per header (closures x headers x files would be
/// quadratic across a run).
#[derive(Debug, Clone)]
pub struct TuStoreContext {
    pub discovery: RepositoryDiscoveryResult,
    pub contexts: Vec<CompilationContext>,
    hash_by_id: Arc<HashMap<FileId, String>>,
    path_by_id: Arc<HashMap<FileId, PathBuf>>,
}

pub type SharedTuStoreContext = Arc<TuStoreContext>;

impl TuStoreContext {
    pub fn new(
        discovery: &RepositoryDiscoveryResult,
        contexts: Vec<CompilationContext>,
    ) -> SharedTuStoreContext {
        let mut hash_by_id = HashMap::with_capacity(discovery.files.len());
        let mut path_by_id = HashMap::with_capacity(discovery.files.len());
        for file in &discovery.files {
            hash_by_id.insert(file.id, file.content_hash.clone());
            path_by_id.insert(file.id, file.path.clone());
        }
        Arc::new(Self {
            discovery: discovery.clone(),
            contexts,
            hash_by_id: Arc::new(hash_by_id),
            path_by_id: Arc::new(path_by_id),
        })
    }
}

/// The include closure of a TU with all file contents already read, plus the
/// cache fingerprint derived from their hashes.
#[derive(Debug, Clone)]
pub struct TuClosure {
    pub files: Vec<(FileId, PathBuf, String)>,
    pub fingerprint: TuCacheFingerprint,
}

fn unreadable_hash(file_id: FileId) -> String {
    format!("unreadable:{}", file_id.0)
}

fn record_hash(
    ctx: &TuStoreContext,
    repo_root: &Path,
    file_id: &FileId,
    rel_path: Option<&Path>,
) -> String {
    if let Some(hash) = ctx.hash_by_id.get(file_id) {
        return hash.clone();
    }
    // Files outside repo discovery (resolved via context include dirs):
    // hash them from their resolved path so edits invalidate dependents;
    // mark unreadable files with a sentinel so they invalidate when they appear.
    if let Some(path) = rel_path {
        let abs = repo_root.join(path);
        if let Ok(hash) = compute_file_hash(&abs) {
            return hash;
        }
    }
    unreadable_hash(*file_id)
}

/// Compilation context a TU's closure analysis runs under: the context with
/// the matching name, or a bare one named after the TU.
fn context_for_tu(tu: &TranslationUnit, ctx: &TuStoreContext) -> CompilationContext {
    ctx.contexts
        .iter()
        .find(|c| c.name == tu.context_name)
        .cloned()
        .unwrap_or_else(|| CompilationContext::new(&tu.context_name, "clang++"))
}

/// Rewrite one context path into its repo-relative form when it lies under
/// `repo_root` (raw, or after filesystem canonicalization to match the
/// `\\?\`-verbatim paths discovery emits); `None` keeps it verbatim.
fn repo_relative_form(path: &Path, repo_root: &Path, canonical_root: &Path) -> Option<PathBuf> {
    for root in [repo_root, canonical_root] {
        if let Ok(relative) = path.strip_prefix(root) {
            return Some(relative.to_path_buf());
        }
    }
    let canonical = fs::canonicalize(path).ok()?;
    canonical
        .strip_prefix(canonical_root)
        .ok()
        .map(|relative| relative.to_path_buf())
}

/// Context form fed to the TU cache fingerprint. Paths under `repo_root` are
/// rewritten repo-relative so the fingerprint keys on repository CONTENT and
/// configuration, never on the machine-specific location of the checkout:
/// the shard store is content-keyed (module docs), and M18's byte-parity
/// contract compares cold runs of repo copies in different roots (a raw
/// temp-root inside the bincode-hashed context would make that impossible).
/// Paths outside the root (system SDK includes) stay verbatim.
fn fingerprint_scoped_context(context: &CompilationContext, repo_root: &Path) -> CompilationContext {
    let canonical_root = fs::canonicalize(repo_root).unwrap_or_else(|_| repo_root.to_path_buf());
    let mut scoped = context.clone();
    for dirs in [
        &mut scoped.include_dirs,
        &mut scoped.system_include_dirs,
        &mut scoped.forced_includes,
    ] {
        for dir in dirs {
            if let Some(relative) = repo_relative_form(dir, repo_root, &canonical_root) {
                *dir = relative;
            }
        }
    }
    scoped
}

/// Canonical merged analysis for a TU closure (TU + transitive headers)
/// through an injected [`SemanticAnalyzer`]: identical merge order and merged
/// fields to the pre-M18 C++-hardcoded merge, with each file parsed through
/// the plugin analyzer. For C++ the plugin adapter is the same
/// `CppSemanticAnalyzer::analyze_source`, so the result is byte-identical;
/// the `language` stamp comes from the TU body's own analysis (also C++ for
/// the C++ plugin).
pub fn analyze_closure_with(
    tu: &TranslationUnit,
    closure: &TuClosure,
    analyzer: &dyn SemanticAnalyzer,
) -> TuAnalysis {
    let mut combined_symbols = Vec::new();
    let mut combined_declarations = Vec::new();
    let mut combined_edges = Vec::new();
    let mut combined_call_sites = Vec::new();
    let mut combined_references = Vec::new();
    let mut language = SourceLanguage::default();

    for (index, (file_id, _, file_content)) in closure.files.iter().enumerate() {
        let mut file_analysis = analyzer.analyze(*file_id, file_content, &tu.context_name);
        if index == 0 {
            language = file_analysis.language;
        }
        combined_symbols.append(&mut file_analysis.symbols);
        combined_declarations.append(&mut file_analysis.declarations);
        combined_edges.append(&mut file_analysis.edges);
        combined_call_sites.append(&mut file_analysis.call_sites);
        combined_references.append(&mut file_analysis.references);
    }

    TuAnalysis {
        tu_file_id: tu.file_id,
        context_name: tu.context_name.clone(),
        symbols: combined_symbols,
        declarations: combined_declarations,
        edges: combined_edges,
        references: combined_references,
        call_sites: combined_call_sites,
        language,
    }
}

/// Full closure-merge analysis (canonical semantics shared by every
/// producer), driven entirely through `plugin`'s resolver/analyzer and its
/// fingerprint identity — the required injection since the M18.11 removal of
/// the hardcoded-C++ walk. Byte-identical to the pre-M18 canonical merge for
/// the C++ plugin (locked by the vendored oracles in
/// `m18_cpp_plugin_parity_test`).
pub fn analyze_tu_full_with_plugin(
    tu: &TranslationUnit,
    repo_root: &Path,
    ctx: &TuStoreContext,
    plugin: &dyn LanguagePlugin,
) -> Result<(TuAnalysis, TuCacheFingerprint)> {
    let closure = compute_tu_closure_plugin(
        tu,
        repo_root,
        ctx,
        CURRENT_ANALYSIS_VERSION,
        plugin.resolver(),
        plugin.id(),
        &plugin.analysis_fingerprint(),
    )?;
    let analysis = analyze_closure_with(tu, &closure, plugin.analyzer());
    Ok((analysis, closure.fingerprint))
}

/// On-disk fallback for a directive the neutral resolver could not map, shared
/// with the graph BFS via [`ariadne_lang::first_ondisk_include_candidate`].
/// Reproduces the legacy resolver's full search-path candidate loop (includer
/// directory for quoted spellings plus every compilation-context search path,
/// style-agnostic) so the plugin closure keeps the pre-M18 coverage — an
/// angled `<hdr>` reachable only through an `-I`/system dir outside repo
/// discovery still resolves and enters the closure. Returns
/// `(FileId, repo-relative path)`.
fn on_disk_include_fallback(
    edge: &RawDepEdge,
    resolution: &ResolutionContext<'_>,
    repo_root: &Path,
) -> Option<(FileId, PathBuf)> {
    ariadne_lang::first_ondisk_include_candidate(
        &edge.raw_target,
        edge.style,
        &edge.source_rel_path,
        &resolution.search_paths,
        repo_root,
        &|path: &Path| path.exists(),
    )
}

/// Resolve one directive edge to the `(FileId, repo-relative path)` pairs the
/// plugin walk must visit, through the outcome-parity table documented on
/// [`compute_tu_closure_plugin`].
fn edge_destinations(
    edge: &RawDepEdge,
    resolver: &dyn DependencyResolver,
    catalog: &FileCatalog,
    resolution: &ResolutionContext,
    ctx: &TuStoreContext,
    repo_root: &Path,
) -> Vec<(FileId, PathBuf)> {
    let mut destinations = Vec::new();
    match resolver.resolve_target(edge, catalog, resolution) {
        ResolveOutcome::Resolved(id) => {
            if let Some(path) = ctx.path_by_id.get(&id) {
                destinations.push((id, path.clone()));
            }
        }
        ResolveOutcome::Wildcard(ids) => {
            for id in ids {
                if let Some(path) = ctx.path_by_id.get(&id) {
                    destinations.push((id, path.clone()));
                }
            }
        }
        // Legacy resolver's on-disk fallback: the includer directory (quoted
        // only) plus EVERY compilation-context search path, style-agnostic —
        // shared with `graph::builder` via [`on_disk_include_fallback`] so both
        // plugin walks keep the pre-M18 reachability. This is the M18.5
        // finding-1 fix: the pre-M18.3 walk here checked only the includer
        // directory, so angled `<hdr>` hits in `-I` dirs (out-of-root
        // SDK/vcpkg paths) were dropped from the plugin closure while the
        // legacy BFS visited them.
        ResolveOutcome::External | ResolveOutcome::Unresolved => {
            if let Some(hit) = on_disk_include_fallback(edge, resolution, repo_root) {
                destinations.push(hit);
            }
        }
    }
    destinations
}

/// Registry-driven closure walk (plan M18.2 / M18.3 / M18.5 seam — the only
/// closure walk since the M18.11 shim removal): the same closure `files`
/// order, membership, and fingerprint inputs as the pre-M18 canonical walk
/// (locked byte-for-byte by the vendored oracles in
/// `m18_cpp_plugin_parity_test`), with directive resolution running through
/// the injected [`DependencyResolver`] + [`FileCatalog`] instead of a
/// hardcoded include resolver.
///
/// Order contract (M18.3): the canonical walk is a LIFO stack of *edges*, so
/// the last directive of the most recently visited file is always visited
/// first and that chain's descendants are appended before earlier siblings.
/// The stack below mirrors that shape exactly — merged-analysis concatenation
/// order (part of every shard envelope and `index.json`) depends on it.
///
/// Outcome mapping (see the `lang-cpp::resolver` module docs for the full
/// parity table, applied via [`edge_destinations`]): `Resolved`/`Wildcard`
/// are visited via their discovery paths; `External`/`Unresolved` directives
/// run the legacy filesystem fallback shared with `graph::builder`
/// ([`on_disk_include_fallback`]) — the includer directory for quoted spellings
/// plus every compilation-context search path, style-agnostic, so angled
/// `<hdr>` includes found only under an `-I`/system dir are visited exactly as
/// the pre-M18 resolver visited them.
///
/// `language`/`plugin_fingerprint` (M18.10) are the fingerprint's language
/// inputs: callers must feed the unit's plugin `id()`/`analysis_fingerprint()`
/// — for C++ that is exactly the `(Cpp, CPP_ANALYSIS_FINGERPRINT)` tuple the
/// pre-M18 walk hardcoded, preserving shard-envelope parity.
pub fn compute_tu_closure_plugin(
    tu: &TranslationUnit,
    repo_root: &Path,
    ctx: &TuStoreContext,
    analysis_version: &str,
    resolver: &dyn DependencyResolver,
    language: SourceLanguage,
    plugin_fingerprint: &str,
) -> Result<TuClosure> {
    let context = context_for_tu(tu, ctx);
    // M18.5: the stateless plugin resolver receives the compilation
    // context's search paths per call (include dirs then system include
    // dirs, the legacy `collect_include_candidates` order).
    let search_paths: Vec<PathBuf> = context
        .include_dirs
        .iter()
        .chain(context.system_include_dirs.iter())
        .cloned()
        .collect();
    let resolution = ResolutionContext {
        repo_root,
        search_paths,
    };
    let catalog = FileCatalog::from_records(&ctx.discovery.files);

    let source_abs = repo_root.join(&tu.source_path);
    // Unreadable TU source is a normal miss: same sentinel as the legacy path.
    let (source_content, source_hash) = match fs::read_to_string(&source_abs) {
        Ok(content) => (
            content,
            record_hash(ctx, repo_root, &tu.file_id, Some(&tu.source_path)),
        ),
        Err(_) => (String::new(), unreadable_hash(tu.file_id)),
    };

    let mut files = vec![(tu.file_id, tu.source_path.clone(), source_content)];
    let mut visited: HashSet<FileId> = HashSet::new();
    visited.insert(tu.file_id);
    let mut header_hashes: Vec<(FileId, String)> = Vec::new();

    let mut stack: Vec<RawDepEdge> =
        resolver.resolve_edges(tu.file_id, &tu.source_path, &files[0].2, &resolution);
    while let Some(edge) = stack.pop() {
        let destinations = edge_destinations(&edge, resolver, &catalog, &resolution, ctx, repo_root);
        for (dest_id, dest_path) in destinations {
            if !visited.insert(dest_id) {
                continue;
            }
            let header_abs = repo_root.join(&dest_path);
            match fs::read_to_string(&header_abs) {
                Ok(header_content) => {
                    files.push((dest_id, dest_path.clone(), header_content.clone()));
                    header_hashes.push((
                        dest_id,
                        record_hash(ctx, repo_root, &dest_id, Some(&dest_path)),
                    ));
                    // Legacy appends the visited file's edges to the queue
                    // tail and pops from the tail: same stack, same order.
                    stack.extend(resolver.resolve_edges(
                        dest_id,
                        &dest_path,
                        &header_content,
                        &resolution,
                    ));
                }
                Err(_) => {
                    header_hashes.push((dest_id, unreadable_hash(dest_id)));
                }
            }
        }
    }

    let fingerprint = TuCacheFingerprint::compute(
        analysis_version,
        &fingerprint_scoped_context(&context, repo_root),
        &tu.file_id,
        &source_hash,
        &header_hashes,
        None,
        language,
        plugin_fingerprint,
    );

    Ok(TuClosure { files, fingerprint })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_build::{CompileCommand, TranslationUnit};
    use ariadne_lang_cpp::CppPlugin;

    fn make_ctx(root: &Path) -> SharedTuStoreContext {
        let files = ariadne_repo::discover_repository_files(root).unwrap();
        let discovery = ariadne_repo::build_discovery_result(root.to_path_buf(), files);
        TuStoreContext::new(&discovery, vec![])
    }

    /// The canonical closure walk with the C++ plugin injected — the only
    /// walk that exists since the M18.11 shim removal.
    fn plugin_closure(
        tu: &TranslationUnit,
        root: &Path,
        ctx: &TuStoreContext,
    ) -> Result<TuClosure> {
        let plugin = CppPlugin::new();
        compute_tu_closure_plugin(
            tu,
            root,
            ctx,
            CURRENT_ANALYSIS_VERSION,
            plugin.resolver(),
            plugin.id(),
            &plugin.analysis_fingerprint(),
        )
    }

    fn full_analysis(
        tu: &TranslationUnit,
        root: &Path,
        ctx: &TuStoreContext,
    ) -> Result<(TuAnalysis, TuCacheFingerprint)> {
        analyze_tu_full_with_plugin(tu, root, ctx, &CppPlugin::new())
    }

    fn tu_for(root: &Path, rel: &str) -> TranslationUnit {
        TranslationUnit {
            file_id: FileId::from_relative_path(rel),
            source_path: PathBuf::from(rel),
            context_name: "default".into(),
            compile_command: CompileCommand {
                directory: root.to_path_buf(),
                file: PathBuf::from(rel),
                command: None,
                arguments: vec![],
                context_name: "default".into(),
                output: None,
            },
            language: Default::default(),
        }
    }

    #[test]
    fn test_shard_envelope_roundtrip_and_old_format_miss() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path();

        // Old plain-TuAnalysis shards must fail as misses.
        let old = TuAnalysis {
            tu_file_id: FileId::from_relative_path("src/a.cpp"),
            context_name: "default".into(),
            symbols: vec![],
            declarations: vec![],
            edges: vec![],
            references: vec![],
            call_sites: vec![],
            language: ariadne_repo::SourceLanguage::Cpp,
        };
        fs::write(
            shard_file(path, old.tu_file_id),
            serde_json::to_string(&old).unwrap(),
        )
        .unwrap();
        assert!(load_shard(path, old.tu_file_id).is_none());

        let cp = ShardCheckpoint {
            fingerprint: "abc".into(),
            analysis: old.clone(),
        };
        save_shard(path, &cp).unwrap();
        let loaded = load_shard(path, old.tu_file_id).unwrap();
        assert_eq!(loaded.fingerprint, "abc");
    }

    #[test]
    fn test_closure_tracks_header_change() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("include")).unwrap();
        fs::write(root.join("include/common.h"), "struct Common {};").unwrap();
        fs::write(
            root.join("a.cpp"),
            "#include \"include/common.h\"\nvoid use(Common) {}",
        )
        .unwrap();

        let ctx = make_ctx(root);
        let tu = tu_for(root, "a.cpp");

        let fp1 = plugin_closure(&tu, root, &ctx).unwrap().fingerprint;
        let fp_same = plugin_closure(&tu, root, &ctx).unwrap().fingerprint;
        assert_eq!(fp1, fp_same);

        fs::write(root.join("include/common.h"), "struct Common { int x; };").unwrap();
        let ctx2 = make_ctx(root);
        let fp2 = plugin_closure(&tu, root, &ctx2).unwrap().fingerprint;
        assert_ne!(fp1.combined_hash, fp2.combined_hash);
    }

    #[test]
    fn test_fingerprint_survives_repo_copy() {
        // M18.3 byte-parity contract: the fingerprint keys on content and
        // configuration, not on the absolute location of the checkout — the
        // same repo copied to a second root must hash identically (cold
        // parity runs happen in fresh temp roots).
        let original = tempfile::tempdir().unwrap();
        let copy = tempfile::tempdir().unwrap();
        for root in [original.path(), copy.path()] {
            fs::create_dir_all(root.join("include")).unwrap();
            fs::write(root.join("include/common.h"), "struct Common {};").unwrap();
            fs::write(
                root.join("a.cpp"),
                "#include \"common.h\"\nvoid use(Common) {}",
            )
            .unwrap();
        }

        let fingerprints = [original.path(), copy.path()].map(|root| {
            let files = ariadne_repo::discover_repository_files(root).unwrap();
            let discovery = ariadne_repo::build_discovery_result(root.to_path_buf(), files);
            let mut context = CompilationContext::new("default", "clang++");
            context.include_dirs = vec![root.canonicalize().unwrap().join("include")];
            let ctx = TuStoreContext::new(&discovery, vec![context]);
            let tu = tu_for(root, "a.cpp");
            plugin_closure(&tu, root, &ctx).unwrap().fingerprint
        });

        assert_eq!(fingerprints[0].combined_hash, fingerprints[1].combined_hash);
    }

    #[test]
    fn test_closure_merges_header_symbols() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("include")).unwrap();
        fs::write(root.join("include/common.h"), "struct Common {};").unwrap();
        fs::write(
            root.join("a.cpp"),
            "#include \"include/common.h\"\nvoid use(Common) {}",
        )
        .unwrap();

        let ctx = make_ctx(root);
        let tu = tu_for(root, "a.cpp");
        let (analysis, _fp) = full_analysis(&tu, root, &ctx).unwrap();
        assert!(
            analysis.symbols.iter().any(|s| s.name == "Common"),
            "closure merge must include symbols from headers"
        );
    }

    #[test]
    fn test_unreadable_header_sentinel() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("a.cpp"), "#include \"ghost.h\"\nvoid f() {}").unwrap();
        // ghost.h missing: dest resolves nowhere, fingerprint still stable.
        let ctx = make_ctx(root);
        let tu = tu_for(root, "a.cpp");
        let closure_result = plugin_closure(&tu, root, &ctx);
        assert!(closure_result.is_ok());
    }

    #[test]
    fn test_missing_tu_source_analyzes_as_miss() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/good.cpp"), "struct Good {};").unwrap();
        // bad.cpp has no file on disk: must not error the run.
        let ctx = make_ctx(root);

        let bad = tu_for(root, "bad.cpp");
        let (bad_analysis, bad_fp) = full_analysis(&bad, root, &ctx).unwrap();
        assert!(bad_analysis.symbols.is_empty());
        assert_eq!(bad_fp.source_hash, unreadable_hash(bad.file_id));

        // A sibling TU in the same store is unaffected.
        let good = tu_for(root, "src/good.cpp");
        let (good_analysis, _good_fp) = full_analysis(&good, root, &ctx).unwrap();
        assert!(good_analysis.symbols.iter().any(|s| s.name == "Good"));

        // Once readable, the sentinel is replaced, invalidating the shard.
        fs::write(root.join("bad.cpp"), "struct Bad {};").unwrap();
        let ctx2 = make_ctx(root);
        let (readable_analysis, readable_fp) = full_analysis(&bad, root, &ctx2).unwrap();
        assert_ne!(readable_fp.combined_hash, bad_fp.combined_hash);
        assert_ne!(readable_fp.source_hash, unreadable_hash(bad.file_id));
        assert!(readable_analysis.symbols.iter().any(|s| s.name == "Bad"));
    }

    #[test]
    fn test_non_utf8_tu_source_analyzes_as_miss() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("bin.cpp"), [0xFFu8, 0xFE, 0x00, 0x01, 0xFD]).unwrap();
        fs::write(root.join("ok.cpp"), "struct Ok {};").unwrap();

        let ctx = make_ctx(root);
        let bin_tu = tu_for(root, "bin.cpp");
        let (analysis, fp) = full_analysis(&bin_tu, root, &ctx).unwrap();
        assert_eq!(fp.source_hash, unreadable_hash(bin_tu.file_id));
        assert!(analysis.symbols.is_empty());

        let ok_tu = tu_for(root, "ok.cpp");
        let (ok_analysis, _fp) = full_analysis(&ok_tu, root, &ctx).unwrap();
        assert!(ok_analysis.symbols.iter().any(|s| s.name == "Ok"));
    }
}
