use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use ariadne_repo::FileId;
use ariadne_build::CompilationContext;
use crate::TuAnalysis;

pub const CURRENT_ANALYSIS_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuCacheFingerprint {
    pub analysis_version: String,
    pub context_hash: String,
    pub source_hash: String,
    pub header_hashes: Vec<(FileId, String)>,
    pub config_hash: String,
    pub combined_hash: String,
}

impl TuCacheFingerprint {
    pub fn compute(
        analysis_version: &str,
        context: &CompilationContext,
        _source_id: &FileId,
        source_hash: &str,
        headers: &[(FileId, String)],
        config_str: Option<&str>,
    ) -> Self {
        // Serialize context for context_hash
        let context_str = bincode::serialize(context).unwrap_or_default();
        let context_hash = format!("{:x}", Sha256::digest(&context_str));
        
        let mut sorted_headers = headers.to_vec();
        sorted_headers.sort_by(|a, b| a.0.cmp(&b.0));
        
        let config_hash = config_str.map(|s| format!("{:x}", Sha256::digest(s.as_bytes()))).unwrap_or_default();
        
        // Combined hash
        let mut hasher = Sha256::new();
        hasher.update(analysis_version.as_bytes());
        hasher.update(b"|");
        hasher.update(context_hash.as_bytes());
        hasher.update(b"|");
        hasher.update(source_hash.as_bytes());
        hasher.update(b"|");
        for (f, h) in &sorted_headers {
            hasher.update(f.0.to_be_bytes());
            hasher.update(b":");
            hasher.update(h.as_bytes());
            hasher.update(b",");
        }
        hasher.update(b"|");
        hasher.update(config_hash.as_bytes());
        
        let combined_hash = format!("{:x}", hasher.finalize());

        Self {
            analysis_version: analysis_version.to_string(),
            context_hash,
            source_hash: source_hash.to_string(),
            header_hashes: sorted_headers,
            config_hash,
            combined_hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuCacheEntry {
    pub tu_file_id: FileId,
    pub source_path: PathBuf,
    pub context_name: String,
    pub fingerprint: TuCacheFingerprint,
    pub analysis: TuAnalysis,
}

impl TuCacheEntry {
    pub fn is_valid(&self, current_fingerprint: &TuCacheFingerprint) -> bool {
        self.fingerprint.combined_hash == current_fingerprint.combined_hash
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IncrementalCache {
    pub analysis_version: String,
    pub entries: HashMap<(FileId, String), TuCacheEntry>,
}

impl IncrementalCache {
    pub fn new(analysis_version: impl Into<String>) -> Self {
        Self {
            analysis_version: analysis_version.into(),
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, tu_file_id: &FileId, context_name: &str) -> Option<&TuCacheEntry> {
        self.entries.get(&(*tu_file_id, context_name.to_string()))
    }

    pub fn insert(&mut self, entry: TuCacheEntry) {
        self.entries.insert((entry.tu_file_id, entry.context_name.clone()), entry);
    }

    pub fn validate_and_get(&self, tu_file_id: &FileId, context_name: &str, current_fingerprint: &TuCacheFingerprint) -> Option<&TuAnalysis> {
        if let Some(entry) = self.get(tu_file_id, context_name) {
            if entry.is_valid(current_fingerprint) {
                return Some(&entry.analysis);
            }
        }
        None
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IncrementalStats {
    pub total_tus: usize,
    pub reused_tus: usize,
    pub reanalyzed_tus: usize,
    pub hit_rate: f64,
}

impl IncrementalStats {
    pub fn calculate(reused: usize, reanalyzed: usize) -> Self {
        let total = reused + reanalyzed;
        let hit_rate = if total > 0 {
            (reused as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        Self {
            total_tus: total,
            reused_tus: reused,
            reanalyzed_tus: reanalyzed,
            hit_rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> CompilationContext {
        CompilationContext {
            name: "test".into(),
            compiler: "clang".into(),
            target_arch: Some("x86_64".into()),
            language_standard: Some("c++20".into()),
            defines: vec!["DEBUG=1".into()],
            include_dirs: vec![PathBuf::from("/usr/include")],
            system_include_dirs: vec![],
            forced_includes: vec![],
            raw_flags: vec!["-O2".into()],
        }
    }

    #[test]
    fn test_fingerprint_determinism() {
        let ctx = create_test_context();
        let src_id = FileId::from_relative_path("src/main.cpp");
        let hdrs = vec![
            (FileId::from_relative_path("src/b.h"), "hash_b".into()),
            (FileId::from_relative_path("src/a.h"), "hash_a".into()),
        ];
        let fp1 = TuCacheFingerprint::compute(
            CURRENT_ANALYSIS_VERSION, &ctx, &src_id, "src_hash", &hdrs, Some("cfg")
        );
        let fp2 = TuCacheFingerprint::compute(
            CURRENT_ANALYSIS_VERSION, &ctx, &src_id, "src_hash", &hdrs, Some("cfg")
        );
        assert_eq!(fp1, fp2);
        // They are ordered by FileId, which is an opaque hash, so we just check it sorted them.
        assert!(fp1.header_hashes[0].0 <= fp1.header_hashes[1].0);
    }

    #[test]
    fn test_entry_validation() {
        let ctx = create_test_context();
        let src_id = FileId::from_relative_path("src/main.cpp");
        let fp1 = TuCacheFingerprint::compute(
            CURRENT_ANALYSIS_VERSION, &ctx, &src_id, "src_hash", &[], None
        );
        
        let fp2 = TuCacheFingerprint::compute(
            CURRENT_ANALYSIS_VERSION, &ctx, &src_id, "different", &[], None
        );
        
        assert!(TuCacheEntry {
            tu_file_id: src_id.clone(),
            source_path: PathBuf::from("src/main.cpp"),
            context_name: "test".into(),
            fingerprint: fp1.clone(),
            analysis: TuAnalysis {
                tu_file_id: src_id.clone(),
                context_name: "test".into(),
                symbols: vec![],
                declarations: vec![],
                edges: vec![],
                references: vec![],
            },
        }.is_valid(&fp1));

        assert!(!TuCacheEntry {
            tu_file_id: src_id.clone(),
            source_path: PathBuf::from("src/main.cpp"),
            context_name: "test".into(),
            fingerprint: fp1.clone(),
            analysis: TuAnalysis {
                tu_file_id: src_id.clone(),
                context_name: "test".into(),
                symbols: vec![],
                declarations: vec![],
                edges: vec![],
                references: vec![],
            },
        }.is_valid(&fp2));
    }

    #[test]
    fn test_incremental_stats() {
        let stats = IncrementalStats::calculate(8, 2);
        assert_eq!(stats.total_tus, 10);
        assert_eq!(stats.reused_tus, 8);
        assert_eq!(stats.reanalyzed_tus, 2);
        assert_eq!(stats.hit_rate, 80.0);
        
        let stats_zero = IncrementalStats::calculate(0, 0);
        assert_eq!(stats_zero.total_tus, 0);
        assert_eq!(stats_zero.hit_rate, 0.0);
    }
}
