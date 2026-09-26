use sha2::{Digest, Sha256};
use std::path::Path;

use crate::cache::CURRENT_ANALYSIS_VERSION;
use ariadne_build::{CompilationContext, TranslationUnit};
use ariadne_repo::FileId;

/// Hash a set of labeled parts into one hex digest.
pub fn hash_parts(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update(b"|");
    }
    format!("{:x}", hasher.finalize())
}

/// Deterministic serialization of the build discovery used as the contexts
/// stage material. Must stay identical everywhere the contexts fingerprint is
/// derived (scheduler incremental path and the pipeline alike).
pub fn contexts_material(contexts: &[CompilationContext], tus: &[TranslationUnit]) -> String {
    serde_json::to_string(&(contexts, tus)).unwrap_or_default()
}

pub fn contexts_fingerprint(repository_fingerprint: &str, material: &str) -> String {
    Fingerprint::chain(
        "contexts",
        &[
            repository_fingerprint.to_string(),
            hash_parts(&["contexts-material", material]),
        ],
    )
}

/// Repository stage fingerprint over a discovery result.
pub fn repository_fingerprint(root: &Path, files: &[(FileId, String)]) -> String {
    Fingerprint::inputs(root, files)
}

/// Stage-level input fingerprint for the artifact pipeline.
pub struct Fingerprint;

impl Fingerprint {
    /// Fingerprint over the discovered file set (sorted by FileId) plus the
    /// repository root and the current analysis version.
    pub fn inputs(root: &Path, files: &[(FileId, String)]) -> String {
        let mut sorted: Vec<&(FileId, String)> = files.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        let mut hasher = Sha256::new();
        hasher.update(root.to_string_lossy().as_bytes());
        hasher.update(b"|");
        hasher.update(CURRENT_ANALYSIS_VERSION.as_bytes());
        hasher.update(b"|");
        for (id, hash) in sorted {
            hasher.update(id.0.to_be_bytes());
            hasher.update(b":");
            hasher.update(hash.as_bytes());
            hasher.update(b",");
        }
        format!("{:x}", hasher.finalize())
    }

    /// Chain an upstream fingerprint (or several) with a stage label and the
    /// current analysis version.
    pub fn chain(label: &str, upstream: &[String]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(label.as_bytes());
        hasher.update(b"|");
        hasher.update(CURRENT_ANALYSIS_VERSION.as_bytes());
        hasher.update(b"|");
        for fp in upstream {
            hasher.update(fp.as_bytes());
            hasher.update(b"|");
        }
        format!("{:x}", hasher.finalize())
    }
}

/// The fingerprint chain shared by all pipeline stages. Each downstream stage
/// is keyed by the fingerprints of its inputs, so a mismatch invalidates the
/// stage wholesale while TU-level content hashes still allow parse reuse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageFingerprints {
    pub repository: String,
    pub contexts: String,
    pub include_graph: String,
    pub index: String,
    pub reduced: String,
    pub callgraph: String,
    pub load: String,
    pub plan: String,
}

impl StageFingerprints {
    pub fn compute(repository: String, contexts: String) -> Self {
        let include_graph =
            Fingerprint::chain("include-graph", &[repository.clone(), contexts.clone()]);
        let index = Fingerprint::chain("index", &[repository.clone(), contexts.clone()]);
        let reduced = Fingerprint::chain("reduced", std::slice::from_ref(&index));
        let callgraph = Fingerprint::chain("callgraph", &[index.clone(), reduced.clone()]);
        let load = Fingerprint::chain("load", &[include_graph.clone(), reduced.clone()]);
        let plan = Fingerprint::chain("plan", &[include_graph.clone(), reduced.clone()]);
        Self {
            repository,
            contexts,
            include_graph,
            index,
            reduced,
            callgraph,
            load,
            plan,
        }
    }

    /// Compute the full stage chain from the discovery and build inputs.
    pub fn from_inputs(
        root: &Path,
        discovery: &ariadne_repo::RepositoryDiscoveryResult,
        contexts: &[CompilationContext],
        tus: &[TranslationUnit],
    ) -> Self {
        let hashes: Vec<(FileId, String)> = discovery
            .files
            .iter()
            .map(|f| (f.id, f.content_hash.clone()))
            .collect();
        let repository = Fingerprint::inputs(root, &hashes);
        let contexts = contexts_fingerprint(&repository, &contexts_material(contexts, tus));
        Self::compute(repository, contexts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_inputs_order_independent() {
        let a = (FileId::from_relative_path("a.h"), "h1".to_string());
        let b = (FileId::from_relative_path("b.h"), "h2".to_string());
        let root = PathBuf::from("/repo");
        assert_eq!(
            Fingerprint::inputs(&root, &[a.clone(), b.clone()]),
            Fingerprint::inputs(&root, &[b, a])
        );
    }

    #[test]
    fn test_inputs_change_on_hash_change() {
        let a = (FileId::from_relative_path("a.h"), "h1".to_string());
        let root = PathBuf::from("/repo");
        let before = Fingerprint::inputs(&root, std::slice::from_ref(&a));
        let changed = (FileId::from_relative_path("a.h"), "h2".to_string());
        let after = Fingerprint::inputs(&root, &[changed]);
        assert_ne!(before, after);
    }

    #[test]
    fn test_chain_labels_differ() {
        let upstream = vec!["abc".to_string()];
        assert_ne!(
            Fingerprint::chain("index", &upstream),
            Fingerprint::chain("reduced", &upstream)
        );
    }

    #[test]
    fn test_hash_parts_deterministic() {
        assert_eq!(hash_parts(&["a", "b"]), hash_parts(&["a", "b"]));
        assert_ne!(hash_parts(&["ab"]), hash_parts(&["a", "b"]));
    }
}
