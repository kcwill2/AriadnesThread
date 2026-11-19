//! Per-file analysis store for store-once indexing.
//!
//! A file's analysis depends only on its content, its id, the context name
//! it is analyzed under, and the analyzer itself, never on which unit
//! reached it. So each `(file, context)` pair is analyzed once and cached in
//! `artifacts/file_shards/`, instead of once per including unit inside every
//! unit's merged shard (which repeated a widely included header thousands of
//! times in memory and on disk).

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use ariadne_index::TuAnalysis;
use ariadne_lang::{LanguageId, SemanticAnalyzer};
use ariadne_repo::FileId;

use crate::closure_memo::ClosureFile;

/// Cache envelope for one file's analysis; a fingerprint mismatch is a miss.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileShard {
    pub fingerprint: String,
    pub analysis: TuAnalysis,
}

/// What identifies one cached analysis: the analyzer and the context name.
pub struct FileAnalysisKey<'a> {
    pub analysis_version: &'a str,
    pub language: &'a LanguageId,
    pub plugin_fingerprint: &'a str,
    pub context_name: &'a str,
}

impl FileAnalysisKey<'_> {
    /// Cache fingerprint of `file` under this key.
    pub fn fingerprint(&self, file: &ClosureFile) -> String {
        [
            self.analysis_version,
            self.language.as_str(),
            self.plugin_fingerprint,
            self.context_name,
            file.content_hash.as_str(),
        ]
        .join("\n")
    }
}

pub fn file_shard_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("artifacts").join("file_shards")
}

/// Shard file name: the file id plus a stable hash of the context name, so
/// one header analyzed under two contexts keeps two entries.
pub fn file_shard_path(dir: &Path, file_id: FileId, context_name: &str) -> PathBuf {
    dir.join(format!("{}-{:016x}.json", file_id.0, fnv1a(context_name)))
}

fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

fn load_file_shard(path: &Path) -> Option<FileShard> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_file_shard(path: &Path, shard: &FileShard) -> Result<()> {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir)?;
    let temp_path = dir.join(format!(
        ".{}.{}.{}.tmp",
        shard.analysis.tu_file_id.0,
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    fs::write(&temp_path, serde_json::to_string(shard)?)?;
    fs::rename(&temp_path, path)?;
    Ok(())
}

/// One file's analysis and whether it came from the cache.
pub struct FileAnalysisOutcome {
    pub analysis: TuAnalysis,
    pub reused: bool,
}

/// The analysis of `file` under `key`: the cached shard when its fingerprint
/// matches, otherwise a fresh analysis (stored for next time). An unreadable
/// file is analyzed as empty source, as the canonical merge does.
pub fn analyze_file_cached(
    file: &ClosureFile,
    key: &FileAnalysisKey<'_>,
    analyzer: &dyn SemanticAnalyzer,
    repo_root: &Path,
) -> Result<FileAnalysisOutcome> {
    let shard_path = file_shard_path(&file_shard_dir(repo_root), file.file_id, key.context_name);
    let fingerprint = key.fingerprint(file);
    if let Some(shard) = load_file_shard(&shard_path) {
        if shard.fingerprint == fingerprint {
            return Ok(FileAnalysisOutcome {
                analysis: shard.analysis,
                reused: true,
            });
        }
    }
    let source = fs::read_to_string(repo_root.join(&file.rel_path)).unwrap_or_default();
    let analysis = analyzer.analyze(file.file_id, &source, key.context_name);
    save_file_shard(
        &shard_path,
        &FileShard {
            fingerprint,
            analysis: analysis.clone(),
        },
    )?;
    Ok(FileAnalysisOutcome {
        analysis,
        reused: false,
    })
}

/// Append `part`'s records onto `merged` in the canonical merge order.
pub fn append_file_analysis(merged: &mut TuAnalysis, mut part: TuAnalysis) {
    merged.symbols.append(&mut part.symbols);
    merged.declarations.append(&mut part.declarations);
    merged.edges.append(&mut part.edges);
    merged.call_sites.append(&mut part.call_sites);
    merged.references.append(&mut part.references);
    merged.extends_edges.append(&mut part.extends_edges);
}
