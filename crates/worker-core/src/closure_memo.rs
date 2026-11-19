//! Memoized closure walk for store-once indexing.
//!
//! The canonical walk ([`crate::tu_store::compute_tu_closure_plugin`]) reads,
//! hashes, scans, and resolves every header again for every unit that
//! reaches it. With real include dirs a header is reached by thousands of
//! units, so that per-unit work dominated indexing. This walk visits the same
//! files in the same order, but scans and resolves each `(file, context)`
//! pair only once per run and shares the result across units and threads.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ariadne_build::TranslationUnit;
use ariadne_lang::{DependencyResolver, ResolutionContext};
use ariadne_repo::FileId;

use crate::tu_store::{context_for_tu, edge_destinations, record_hash, TuStoreContext};

/// One file as seen from one compilation context: its content hash and,
/// per include directive in source order, the files that directive reaches.
/// `readable == false` marks a file that could not be read; it is visited
/// (so it is never retried within a unit) but contributes nothing.
#[derive(Debug)]
pub struct ScannedFile {
    pub rel_path: PathBuf,
    pub content_hash: String,
    pub readable: bool,
    pub directive_targets: Vec<Vec<(FileId, PathBuf)>>,
}

/// A file a unit's closure contains, in canonical closure order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosureFile {
    pub file_id: FileId,
    pub rel_path: PathBuf,
    pub content_hash: String,
}

type MemoKey = (FileId, String);

/// Per-run cache of scanned files, shared by every unit and worker thread.
#[derive(Debug, Default)]
pub struct ClosureMemo {
    scanned: Mutex<HashMap<MemoKey, Arc<ScannedFile>>>,
}

impl ClosureMemo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of distinct `(file, context)` pairs scanned so far.
    pub fn scanned_count(&self) -> usize {
        self.scanned.lock().map(|map| map.len()).unwrap_or(0)
    }

    fn lookup(&self, key: &MemoKey) -> Option<Arc<ScannedFile>> {
        self.scanned.lock().ok()?.get(key).cloned()
    }

    fn remember(&self, key: MemoKey, scanned: ScannedFile) -> Arc<ScannedFile> {
        let scanned = Arc::new(scanned);
        match self.scanned.lock() {
            Ok(mut map) => Arc::clone(map.entry(key).or_insert(scanned)),
            Err(_) => scanned,
        }
    }
}

/// Everything one unit's walk needs besides the memo.
pub struct WalkInputs<'a> {
    pub repo_root: &'a Path,
    pub store_ctx: &'a TuStoreContext,
    pub resolver: &'a dyn DependencyResolver,
}

/// The readable files of `tu`'s include closure in canonical closure order
/// (the unit's own source first), identical to the `files` order of
/// [`crate::tu_store::compute_tu_closure_plugin`].
pub fn walk_tu_closure(
    tu: &TranslationUnit,
    inputs: &WalkInputs<'_>,
    memo: &ClosureMemo,
) -> Vec<ClosureFile> {
    let context = context_for_tu(tu, inputs.store_ctx);
    let resolution = ResolutionContext {
        repo_root: inputs.repo_root,
        search_paths: context
            .include_dirs
            .iter()
            .chain(context.system_include_dirs.iter())
            .cloned()
            .collect(),
    };
    let scan = |file_id: FileId, rel_path: &Path| {
        scan_file(file_id, rel_path, &tu.context_name, inputs, &resolution, memo)
    };

    let source = scan(tu.file_id, &tu.source_path);
    let mut files = vec![closure_file(tu.file_id, &source)];
    let mut visited: HashSet<FileId> = HashSet::from([tu.file_id]);
    let mut stack: Vec<(Arc<ScannedFile>, usize)> = Vec::new();
    push_directives(&mut stack, &source);

    while let Some((includer, directive)) = stack.pop() {
        for (dest_id, dest_path) in &includer.directive_targets[directive] {
            if !visited.insert(*dest_id) {
                continue;
            }
            let header = scan(*dest_id, dest_path);
            if header.readable {
                files.push(closure_file(*dest_id, &header));
                push_directives(&mut stack, &header);
            }
        }
    }
    files
}

fn closure_file(file_id: FileId, scanned: &ScannedFile) -> ClosureFile {
    ClosureFile {
        file_id,
        rel_path: scanned.rel_path.clone(),
        content_hash: scanned.content_hash.clone(),
    }
}

/// Same LIFO shape as the canonical walk: a file's directives go on the
/// stack in source order, so its LAST directive is visited first.
fn push_directives(stack: &mut Vec<(Arc<ScannedFile>, usize)>, scanned: &Arc<ScannedFile>) {
    for directive in 0..scanned.directive_targets.len() {
        stack.push((Arc::clone(scanned), directive));
    }
}

fn scan_file(
    file_id: FileId,
    rel_path: &Path,
    context_name: &str,
    inputs: &WalkInputs<'_>,
    resolution: &ResolutionContext<'_>,
    memo: &ClosureMemo,
) -> Arc<ScannedFile> {
    let key = (file_id, context_name.to_string());
    if let Some(found) = memo.lookup(&key) {
        return found;
    }
    let scanned = match fs::read_to_string(inputs.repo_root.join(rel_path)) {
        Ok(content) => {
            let edges = inputs
                .resolver
                .resolve_edges(file_id, rel_path, &content, resolution);
            let catalog = inputs.store_ctx.catalog();
            let directive_targets = edges
                .iter()
                .map(|edge| {
                    edge_destinations(
                        edge,
                        inputs.resolver,
                        catalog,
                        resolution,
                        inputs.store_ctx,
                        inputs.repo_root,
                    )
                })
                .collect();
            ScannedFile {
                rel_path: rel_path.to_path_buf(),
                content_hash: record_hash(inputs.store_ctx, inputs.repo_root, &file_id, Some(rel_path)),
                readable: true,
                directive_targets,
            }
        }
        Err(_) => ScannedFile {
            rel_path: rel_path.to_path_buf(),
            content_hash: String::new(),
            readable: false,
            directive_targets: Vec::new(),
        },
    };
    memo.remember(key, scanned)
}
