//! Repository discovery and filesystem canonicalization.

pub mod discovery;
pub mod identity;
pub mod policy;
pub mod types;

pub use discovery::{build_discovery_result, compute_file_hash, discover_repository_files};
pub use identity::LanguageId;
pub use policy::classify_ownership_and_policy;
pub use types::{
    is_includable_header, FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy,
    RepositoryDiscoveryResult, SourceLanguage,
};

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Repository {
    pub root: PathBuf,
}

impl Repository {
    pub fn locate<P: AsRef<Path>>(path: P) -> Result<Self> {
        let root = path
            .as_ref()
            .canonicalize()
            .with_context(|| format!("Failed to locate repository path: {:?}", path.as_ref()))?;
        Ok(Self { root })
    }

    pub fn discover(&self) -> Result<RepositoryDiscoveryResult> {
        self.discover_with_progress(&mut |_hashed, _discovered| {})
    }

    /// Additive; per hashed file, (hashed_so_far, discovered_so_far).
    /// discover() delegates no-op. Record ORDER unchanged.
    pub fn discover_with_progress(
        &self,
        on_hashed: &mut dyn FnMut(usize, usize),
    ) -> Result<RepositoryDiscoveryResult> {
        let files = discovery::discover_repository_files_with_progress(&self.root, on_hashed)?;
        Ok(build_discovery_result(self.root.clone(), files))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_repository_locate() {
        let dir = tempdir().unwrap();
        let repo = Repository::locate(dir.path());
        assert!(repo.is_ok());
    }

    #[test]
    fn test_stable_file_ids() {
        let id1 = FileId::from_relative_path("include/thing.h");
        let id2 = FileId::from_relative_path("include/thing.h");
        let id3 = FileId::from_relative_path("src/thing.cpp");

        assert_eq!(id1, id2, "Same path must produce identical FileId");
        assert_ne!(id1, id3, "Different paths must produce different FileIds");
    }

    #[test]
    fn test_path_canonicalization_and_discovery() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let inc_dir = root.join("include");
        let src_dir = root.join("src");
        let gen_dir = root.join("src/generated");
        fs::create_dir_all(&inc_dir).unwrap();
        fs::create_dir_all(&src_dir).unwrap();
        fs::create_dir_all(&gen_dir).unwrap();

        let mut f1 = File::create(inc_dir.join("header.h")).unwrap();
        writeln!(f1, "#pragma once\nvoid foo();").unwrap();

        let mut f2 = File::create(src_dir.join("main.cpp")).unwrap();
        writeln!(f2, "#include \"header.h\"\nint main() {{ return 0; }}").unwrap();

        let mut f3 = File::create(gen_dir.join("gen_code.cpp")).unwrap();
        writeln!(f3, "// Generated code").unwrap();

        let repo = Repository::locate(root).unwrap();
        let res = repo.discover().unwrap();

        assert_eq!(res.files.len(), 3);

        let gen_file = res
            .files
            .iter()
            .find(|f| f.path.to_string_lossy().contains("gen_code"))
            .unwrap();
        assert_eq!(gen_file.ownership, FileOwnership::Generated);
        assert_eq!(gen_file.optimization_policy, OptimizationPolicy::Frozen);
    }
}
