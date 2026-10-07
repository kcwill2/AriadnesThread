use crate::TuAnalysis;
use ariadne_build::CompilationContext;
use ariadne_lang_api::identity::LanguageId;
use ariadne_repo::FileId;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;

/// Cache epoch (plan §3.5 cache-epoch row, §9 M18.10). Bumped 1.0.0 → 2.0.0:
/// the bincode `artifacts/reduced/symbols.bin` layout changed with M18.9
/// (`ReducedSymbolRecord::declaring_files`; `#[serde(default)]` cannot rescue
/// bincode) AND `TuCacheFingerprint` gained the language/plugin inputs below.
/// `ArtifactManager::check_manifest` rejects the stale epoch before any blob
/// deserialization, forcing a fresh rebuild.
/// M27 T2 epoch bump 2.0.0 -> 2.1.0 (user DP-6, 2026-10-05): the tokenizer
/// column-underflow class - shards written by RELEASE builds over non-ASCII
/// corpora carry silently-wrapped token columns, and the fingerprint chain
/// hashes CONTENT, not analyzer code, so this version bump is the
/// invalidation that forces those shards to rebuild.
/// M28 T4 epoch bump 2.1.0 -> 2.2.0 (user decision 2026-10-06, the
/// registered DP-2 exception to the 2.1.0 freeze): the load-artifact
/// payload reshape dropped the never-consumed per-symbol detail from
/// `load/report.json`/`report.bin` (bincode layout changed; bincode is
/// strict and `#[serde(default)]` cannot rescue it), so this bump is the
/// explicit invalidation - `check_manifest` sends 2.1.0-era artifacts to
/// `StaleArtifact` (rebuild) before any blob deserialization. The fingerprint
/// chain hashes content, not code (the standing M27 lesson), so the epoch is
/// the only lever.
pub const CURRENT_ANALYSIS_VERSION: &str = "2.2.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TuCacheFingerprint {
    pub analysis_version: String,
    pub context_hash: String,
    pub source_hash: String,
    pub header_hashes: Vec<(FileId, String)>,
    pub config_hash: String,
    /// M18.10: owning language, so one store cannot poison entries across
    /// languages. `serde(default)` keeps v1 `cache.json` files readable (⇒ Cpp).
    /// M24 Part A (ADAPT-5): open-ended, because this field records the
    /// owning PLUGIN's identity, not a file-kind label — a cache namespace that
    /// cannot name a third language cannot keep its documented promise.
    /// `#[serde(transparent)]` keeps the stored bytes identical, and the
    /// hand-written bare-name `Debug` below keeps the digest at line 78
    /// byte-identical, so `CURRENT_ANALYSIS_VERSION` does NOT need to move.
    #[serde(default)]
    pub language: LanguageId,
    /// M18.10: the owning plugin's `analysis_fingerprint()`; bumping a plugin
    /// invalidates exactly its language's shard entries.
    #[serde(default)]
    pub plugin_fingerprint: String,
    pub combined_hash: String,
}

impl TuCacheFingerprint {
    #[allow(clippy::too_many_arguments)]
    pub fn compute(
        analysis_version: &str,
        context: &CompilationContext,
        _source_id: &FileId,
        source_hash: &str,
        headers: &[(FileId, String)],
        config_str: Option<&str>,
        language: LanguageId,
        plugin_fingerprint: &str,
    ) -> Self {
        // Serialize context for context_hash
        let context_str = bincode::serialize(context).unwrap_or_default();
        let context_hash = format!("{:x}", Sha256::digest(&context_str));

        let mut sorted_headers = headers.to_vec();
        sorted_headers.sort_by(|a, b| a.0.cmp(&b.0));

        let config_hash = config_str
            .map(|s| format!("{:x}", Sha256::digest(s.as_bytes())))
            .unwrap_or_default();

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
        // M18.10: language + plugin fingerprint join the digest (appended
        // after config_hash), so a Scala run can neither collide with nor
        // invalidate C++ entries (§7 cache-poisoning risk row).
        hasher.update(b"|");
        hasher.update(format!("{language:?}").as_bytes());
        hasher.update(b"|");
        hasher.update(plugin_fingerprint.as_bytes());

        let combined_hash = format!("{:x}", hasher.finalize());

        Self {
            analysis_version: analysis_version.to_string(),
            context_hash,
            source_hash: source_hash.to_string(),
            header_hashes: sorted_headers,
            config_hash,
            language,
            plugin_fingerprint: plugin_fingerprint.to_string(),
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
        self.entries
            .insert((entry.tu_file_id, entry.context_name.clone()), entry);
    }

    pub fn validate_and_get(
        &self,
        tu_file_id: &FileId,
        context_name: &str,
        current_fingerprint: &TuCacheFingerprint,
    ) -> Option<&TuAnalysis> {
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
            // Test-only stand-in toolchain name; the R2 guard forbids naming
            // a real compiler in this crate (plan §6.3, item: guard un-ignore).
            compiler: "fake-cc".into(),
            target_arch: Some("x86_64".into()),
            language_standard: Some("c++20".into()),
            defines: vec!["DEBUG=1".into()],
            include_dirs: vec![PathBuf::from("/usr/include")],
            system_include_dirs: vec![],
            forced_includes: vec![],
            raw_flags: vec!["-O2".into()],
            language: Default::default(),
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
            CURRENT_ANALYSIS_VERSION,
            &ctx,
            &src_id,
            "src_hash",
            &hdrs,
            Some("cfg"),
            LanguageId::cpp(),
            "test-plugin",
        );
        let fp2 = TuCacheFingerprint::compute(
            CURRENT_ANALYSIS_VERSION,
            &ctx,
            &src_id,
            "src_hash",
            &hdrs,
            Some("cfg"),
            LanguageId::cpp(),
            "test-plugin",
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
            CURRENT_ANALYSIS_VERSION,
            &ctx,
            &src_id,
            "src_hash",
            &[],
            None,
            LanguageId::cpp(),
            "test-plugin",
        );

        let fp2 = TuCacheFingerprint::compute(
            CURRENT_ANALYSIS_VERSION,
            &ctx,
            &src_id,
            "different",
            &[],
            None,
            LanguageId::cpp(),
            "test-plugin",
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
                call_sites: vec![],
                language: ariadne_repo::SourceLanguage::Cpp,

                extends_edges: Vec::new(),
            },
        }
        .is_valid(&fp1));

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
                call_sites: vec![],
                language: ariadne_repo::SourceLanguage::Cpp,

                extends_edges: Vec::new(),
            },
        }
        .is_valid(&fp2));
    }

    fn fingerprint_with(
        language: LanguageId,
        plugin_fingerprint: &str,
    ) -> TuCacheFingerprint {
        TuCacheFingerprint::compute(
            CURRENT_ANALYSIS_VERSION,
            &create_test_context(),
            &FileId::from_relative_path("src/main.cpp"),
            "src_hash",
            &[],
            None,
            language,
            plugin_fingerprint,
        )
    }

    #[test]
    fn test_epoch_bumped_for_m18() {
        // §9 M18.10: bincode symbols.bin layout change (M18.9) + fingerprint
        // input change (this milestone) force cache-wide regeneration.
        // M27 T2 raised this pin to 2.1.0 (tokenizer shards); M28 T4 raises
        // it to 2.2.0 (load-artifact payload reshape, registered DP-2).
        assert_eq!(CURRENT_ANALYSIS_VERSION, "2.2.0");
    }

    #[test]
    fn test_differing_language_or_plugin_fingerprint_changes_digest() {
        let cpp = fingerprint_with(LanguageId::cpp(), "fp-a");
        let scala = fingerprint_with(LanguageId::scala(), "fp-a");
        let other_plugin = fingerprint_with(LanguageId::cpp(), "fp-b");
        let same = fingerprint_with(LanguageId::cpp(), "fp-a");

        assert_eq!(cpp, same, "identical inputs must be deterministic");
        assert_ne!(
            cpp.combined_hash, scala.combined_hash,
            "differing language must produce a different digest"
        );
        assert_ne!(
            cpp.combined_hash, other_plugin.combined_hash,
            "differing plugin_fingerprint must produce a different digest"
        );
        assert_eq!(cpp.language, LanguageId::cpp());
        assert_eq!(cpp.plugin_fingerprint, "fp-a");
    }

    #[test]
    fn test_v1_fingerprint_json_still_deserializes() {
        // cache.json is additive: v1 files lack the two new fields.
        let legacy = fingerprint_with(LanguageId::cpp(), "fp-a");
        let mut value = serde_json::to_value(&legacy).unwrap();
        value.as_object_mut().unwrap().remove("language");
        value.as_object_mut().unwrap().remove("plugin_fingerprint");
        let back: TuCacheFingerprint = serde_json::from_value(value).unwrap();
        assert_eq!(back.language, LanguageId::cpp());
        assert_eq!(back.plugin_fingerprint, "");
        assert_eq!(back.combined_hash, legacy.combined_hash);
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
