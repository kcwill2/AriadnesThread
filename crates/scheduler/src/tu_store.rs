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
use ariadne_graph::IncludeResolver;
use ariadne_index::{
    CppSemanticAnalyzer, TuAnalysis, TuCacheFingerprint, CURRENT_ANALYSIS_VERSION,
};
use ariadne_repo::{compute_file_hash, FileId, RepositoryDiscoveryResult};

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

/// Walk a TU's include closure (canonical semantics: TU body plus every
/// transitive header, all parsed and merged) and compute its fingerprint
/// without running the semantic parser.
pub fn compute_tu_closure(
    tu: &TranslationUnit,
    repo_root: &Path,
    ctx: &TuStoreContext,
    analysis_version: &str,
) -> Result<TuClosure> {
    let context = ctx
        .contexts
        .iter()
        .find(|c| c.name == tu.context_name)
        .cloned()
        .unwrap_or_else(|| CompilationContext::new(&tu.context_name, "clang++"));

    let resolver = IncludeResolver::new(repo_root)
        .with_repo_discovery(&ctx.discovery)
        .with_context(&context);

    let source_abs = repo_root.join(&tu.source_path);
    // An unreadable TU source (deleted mid-run, binary, non-UTF-8) is a
    // normal miss: analyze an empty body and stamp the same sentinel used
    // for unreadable headers, so the entry invalidates once it becomes
    // readable again instead of failing the whole run.
    let (source_content, source_hash) = match fs::read_to_string(&source_abs) {
        Ok(content) => (
            content,
            record_hash(ctx, repo_root, &tu.file_id, Some(&tu.source_path)),
        ),
        Err(_) => (String::new(), unreadable_hash(tu.file_id)),
    };

    let edges = resolver.resolve_file_includes(tu.file_id, &tu.source_path, &source_content);

    let mut files = vec![(tu.file_id, tu.source_path.clone(), source_content)];
    let mut visited: HashSet<FileId> = HashSet::new();
    visited.insert(tu.file_id);
    let mut header_hashes: Vec<(FileId, String)> = Vec::new();

    let mut queue = edges;
    while let Some(edge) = queue.pop() {
        if let Some(dest_id) = edge.destination {
            if visited.insert(dest_id) {
                let dest_path = ctx
                    .path_by_id
                    .get(&dest_id)
                    .cloned()
                    .or_else(|| edge.resolved_path.clone());
                let Some(dest_path) = dest_path else { continue };
                let header_abs = repo_root.join(&dest_path);
                match fs::read_to_string(&header_abs) {
                    Ok(header_content) => {
                        files.push((dest_id, dest_path.clone(), header_content.clone()));
                        header_hashes.push((
                            dest_id,
                            record_hash(ctx, repo_root, &dest_id, Some(&dest_path)),
                        ));
                        queue.extend(resolver.resolve_file_includes(
                            dest_id,
                            &dest_path,
                            &header_content,
                        ));
                    }
                    Err(_) => {
                        header_hashes.push((dest_id, unreadable_hash(dest_id)));
                    }
                }
            }
        }
    }

    let fingerprint = TuCacheFingerprint::compute(
        analysis_version,
        &context,
        &tu.file_id,
        &source_hash,
        &header_hashes,
        None,
    );

    Ok(TuClosure { files, fingerprint })
}

/// Canonical merged analysis for a TU closure (TU + transitive headers).
pub fn analyze_closure(tu: &TranslationUnit, closure: &TuClosure) -> TuAnalysis {
    let mut combined_symbols = Vec::new();
    let mut combined_declarations = Vec::new();
    let mut combined_edges = Vec::new();
    let mut combined_call_sites = Vec::new();
    let mut combined_references = Vec::new();

    for (file_id, _, file_content) in &closure.files {
        let mut file_analysis =
            CppSemanticAnalyzer::analyze_source(*file_id, file_content, &tu.context_name);
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
    }
}

/// Full closure-merge analysis (canonical semantics shared by every producer).
pub fn analyze_tu_full(
    tu: &TranslationUnit,
    repo_root: &Path,
    ctx: &TuStoreContext,
) -> Result<(TuAnalysis, TuCacheFingerprint)> {
    let closure = compute_tu_closure(tu, repo_root, ctx, CURRENT_ANALYSIS_VERSION)?;
    let analysis = analyze_closure(tu, &closure);
    Ok((analysis, closure.fingerprint))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_build::{CompileCommand, TranslationUnit};

    fn make_ctx(root: &Path) -> SharedTuStoreContext {
        let files = ariadne_repo::discover_repository_files(root).unwrap();
        let discovery = ariadne_repo::build_discovery_result(root.to_path_buf(), files);
        TuStoreContext::new(&discovery, vec![])
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

        let fp1 = compute_tu_closure(&tu, root, &ctx, CURRENT_ANALYSIS_VERSION)
            .unwrap()
            .fingerprint;
        let fp_same = compute_tu_closure(&tu, root, &ctx, CURRENT_ANALYSIS_VERSION)
            .unwrap()
            .fingerprint;
        assert_eq!(fp1, fp_same);

        fs::write(root.join("include/common.h"), "struct Common { int x; };").unwrap();
        let ctx2 = make_ctx(root);
        let fp2 = compute_tu_closure(&tu, root, &ctx2, CURRENT_ANALYSIS_VERSION)
            .unwrap()
            .fingerprint;
        assert_ne!(fp1.combined_hash, fp2.combined_hash);
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
        let (analysis, _fp) = analyze_tu_full(&tu, root, &ctx).unwrap();
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
        let res = compute_tu_closure(&tu, root, &ctx, CURRENT_ANALYSIS_VERSION);
        assert!(res.is_ok());
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
        let (bad_analysis, bad_fp) = analyze_tu_full(&bad, root, &ctx).unwrap();
        assert!(bad_analysis.symbols.is_empty());
        assert_eq!(bad_fp.source_hash, unreadable_hash(bad.file_id));

        // A sibling TU in the same store is unaffected.
        let good = tu_for(root, "src/good.cpp");
        let (good_analysis, _good_fp) = analyze_tu_full(&good, root, &ctx).unwrap();
        assert!(good_analysis.symbols.iter().any(|s| s.name == "Good"));

        // Once readable, the sentinel is replaced, invalidating the shard.
        fs::write(root.join("bad.cpp"), "struct Bad {};").unwrap();
        let ctx2 = make_ctx(root);
        let (readable_analysis, readable_fp) = analyze_tu_full(&bad, root, &ctx2).unwrap();
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
        let (analysis, fp) = analyze_tu_full(&bin_tu, root, &ctx).unwrap();
        assert_eq!(fp.source_hash, unreadable_hash(bin_tu.file_id));
        assert!(analysis.symbols.is_empty());

        let ok_tu = tu_for(root, "ok.cpp");
        let (ok_analysis, _fp) = analyze_tu_full(&ok_tu, root, &ctx).unwrap();
        assert!(ok_analysis.symbols.iter().any(|s| s.name == "Ok"));
    }
}
