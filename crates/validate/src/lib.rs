//! Validation harness and transactional verification.

use anyhow::Result;
use ariadne_lang::{DependencyRewriter, RewriteSpec};
use ariadne_transform::{ExtractionResult, MigrationPreview};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationTier {
    Structural,
    GeneratedHeaders,
    AffectedTUs,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    Pass,
    Fail(String),
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierResult {
    pub tier: ValidationTier,
    pub status: ValidationStatus,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplyOutcome {
    Committed {
        tiers_passed: Vec<TierResult>,
    },
    RolledBack {
        failed_tier: TierResult,
        tiers_passed: Vec<TierResult>,
    },
}

impl ApplyOutcome {
    pub fn format_output(&self) -> String {
        let mut out = String::new();
        match self {
            ApplyOutcome::Committed { tiers_passed } => {
                for tr in tiers_passed {
                    out.push_str(&format_tier(tr));
                }
                out.push_str("\nCommitted.");
            }
            ApplyOutcome::RolledBack {
                failed_tier,
                tiers_passed,
            } => {
                for tr in tiers_passed {
                    out.push_str(&format_tier(tr));
                }
                out.push_str(&format_tier(failed_tier));
                out.push_str("\nRolling back...\n\nRepository restored.");
            }
        }
        out
    }
}

fn format_tier(tr: &TierResult) -> String {
    let name = match tr.tier {
        ValidationTier::Structural => "Structural validation",
        ValidationTier::GeneratedHeaders => "Generated header validation",
        ValidationTier::AffectedTUs => "Affected TU validation",
    };
    let status = match tr.status {
        ValidationStatus::Pass => "PASS",
        ValidationStatus::Fail(_) => "FAIL",
    };
    format!("{}: {}\n", name, status)
}

pub struct FileSnapshot {
    pub entries: BTreeMap<PathBuf, Option<Vec<u8>>>,
}

impl FileSnapshot {
    pub fn capture(paths: &[PathBuf]) -> std::io::Result<Self> {
        let mut entries = BTreeMap::new();
        for p in paths {
            if p.exists() {
                entries.insert(p.clone(), Some(fs::read(p)?));
            } else {
                entries.insert(p.clone(), None);
            }
        }
        Ok(Self { entries })
    }

    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        for (path, content_opt) in &self.entries {
            hasher.update(path.to_string_lossy().as_bytes());
            if let Some(content) = content_opt {
                hasher.update([1]);
                hasher.update(content);
            } else {
                hasher.update([0]);
            }
        }
        format!("{:x}", hasher.finalize())
    }
}

/// Transitional text-free rewriter (M18.8b): keeps the frozen 3-argument
/// `TransactionApplier::apply` entry point compiling while all directive
/// *text* lives behind the injected rewriter. It performs no directive
/// mutation and emits no preamble; language-correct applications must go
/// through [`TransactionApplier::apply_with`] with a registered plugin's
/// rewriter (the CLI does). Removed when the frozen call sites move (M23).
struct TransitionalDefaultRewriter;

impl DependencyRewriter for TransitionalDefaultRewriter {
    fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
        source.to_string()
    }
}

pub struct TransactionApplier;

impl TransactionApplier {
    /// Frozen-shape entry point: orchestration only, no directive text.
    /// Production pipeline call sites (CLI) use
    /// [`TransactionApplier::apply_with`] with a plugin rewriter.
    #[deprecated = "use apply_with with a plugin rewriter (removed M23)"]
    pub fn apply(
        repo_root: &Path,
        extraction: &ExtractionResult,
        migration: &MigrationPreview,
    ) -> Result<(FileSnapshot, Vec<PathBuf>)> {
        Self::apply_with(repo_root, extraction, migration, &TransitionalDefaultRewriter)
    }

    /// Transactional apply: snapshot the touched files, write generated
    /// headers and consumer rewrites through `rewriter` (which owns every
    /// directive spelling), and return the snapshot plus created paths.
    pub fn apply_with(
        repo_root: &Path,
        extraction: &ExtractionResult,
        migration: &MigrationPreview,
        rewriter: &dyn DependencyRewriter,
    ) -> Result<(FileSnapshot, Vec<PathBuf>)> {
        let mut paths_to_snapshot = Vec::new();
        let mut new_paths = Vec::new();

        let gen_dir = repo_root.join("generated");
        for unit in &extraction.headers {
            paths_to_snapshot.push(gen_dir.join(&unit.path));
        }

        for rw in &migration.rewrites {
            paths_to_snapshot.push(repo_root.join(&rw.consumer_file));
        }

        let snapshot = FileSnapshot::capture(&paths_to_snapshot)?;

        fs::create_dir_all(&gen_dir)?;

        for unit in &extraction.headers {
            let path = gen_dir.join(&unit.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            if !path.exists() {
                new_paths.push(path.clone());
            }

            let deps: Vec<String> = unit
                .required_deps
                .iter()
                .map(|dep| dep.display().to_string())
                .collect();
            let mut content = rewriter.emit_file_preamble(&deps);
            for sym in &unit.symbols {
                content.push_str(&sym.body);
                content.push('\n');
            }
            fs::write(&path, content)?;
        }

        for rw in &migration.rewrites {
            let path = repo_root.join(&rw.consumer_file);
            let source = fs::read_to_string(&path)?;
            let spec = RewriteSpec {
                old_deps: rw
                    .old_includes
                    .iter()
                    .map(|dep| dep.display().to_string())
                    .collect(),
                new_deps: rw
                    .new_includes
                    .iter()
                    .map(|dep| dep.display().to_string())
                    .collect(),
            };
            let content = rewriter.rewrite(&source, &spec);
            fs::write(&path, content)?;
        }

        Ok((snapshot, new_paths))
    }

    pub fn rollback(snapshot: &FileSnapshot, new_paths: &[PathBuf]) -> Result<()> {
        for path in new_paths {
            if path.exists() {
                fs::remove_file(path)?;
            }
        }

        for (path, content_opt) in &snapshot.entries {
            match content_opt {
                Some(content) => {
                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(path, content)?;
                }
                None => {
                    if path.exists() {
                        fs::remove_file(path)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn verify_restored(snapshot: &FileSnapshot) -> Result<(), String> {
        let current: Vec<PathBuf> = snapshot.entries.keys().cloned().collect();
        let current_snapshot = FileSnapshot::capture(&current).map_err(|e| e.to_string())?;
        if snapshot.fingerprint() == current_snapshot.fingerprint() {
            Ok(())
        } else {
            Err("State mismatch".into())
        }
    }
}

pub struct TierValidator;

impl TierValidator {
    pub fn validate_structural(extraction: &ExtractionResult) -> TierResult {
        let status = if extraction.errors.is_empty() {
            ValidationStatus::Pass
        } else {
            ValidationStatus::Fail(extraction.errors.join(", "))
        };
        TierResult {
            tier: ValidationTier::Structural,
            status,
        }
    }

    pub fn validate_generated_headers(
        extraction: &ExtractionResult,
        header_output_dir: &Path,
    ) -> TierResult {
        for unit in &extraction.headers {
            for req in &unit.required_deps {
                let p1 = header_output_dir.join(req);
                // Also check repo root, which is parent of generated dir in our scheme
                let p2 = if let Some(parent) = header_output_dir.parent() {
                    parent.join(req)
                } else {
                    PathBuf::new()
                };
                if !p1.exists() && !p2.exists() {
                    return TierResult {
                        tier: ValidationTier::GeneratedHeaders,
                        status: ValidationStatus::Fail(format!(
                            "Missing include: {}",
                            req.display()
                        )),
                    };
                }
            }
        }
        TierResult {
            tier: ValidationTier::GeneratedHeaders,
            status: ValidationStatus::Pass,
        }
    }

    pub fn validate_affected_tus(
        migration: &MigrationPreview,
        header_output_dir: &Path,
    ) -> TierResult {
        for rw in &migration.rewrites {
            for new_inc in &rw.new_includes {
                let p1 = header_output_dir.join(new_inc);
                let p2 = if let Some(parent) = header_output_dir.parent() {
                    parent.join(new_inc)
                } else {
                    PathBuf::new()
                };
                if !p1.exists() && !p2.exists() && new_inc.extension().is_some() {
                    return TierResult {
                        tier: ValidationTier::AffectedTUs,
                        status: ValidationStatus::Fail(format!(
                            "Missing TU include: {}",
                            new_inc.display()
                        )),
                    };
                }
            }
        }
        TierResult {
            tier: ValidationTier::AffectedTUs,
            status: ValidationStatus::Pass,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_lang_cpp::CppDependencyRewriter;
    use ariadne_transform::{ConsumerRewrite, GeneratedUnit};

    #[test]
    fn test_format_output() {
        let outcome1 = ApplyOutcome::Committed {
            tiers_passed: vec![
                TierResult {
                    tier: ValidationTier::Structural,
                    status: ValidationStatus::Pass,
                },
                TierResult {
                    tier: ValidationTier::GeneratedHeaders,
                    status: ValidationStatus::Pass,
                },
                TierResult {
                    tier: ValidationTier::AffectedTUs,
                    status: ValidationStatus::Pass,
                },
            ],
        };
        assert_eq!(
            outcome1.format_output(),
            "Structural validation: PASS\nGenerated header validation: PASS\nAffected TU validation: PASS\n\nCommitted."
        );

        let outcome2 = ApplyOutcome::RolledBack {
            failed_tier: TierResult {
                tier: ValidationTier::GeneratedHeaders,
                status: ValidationStatus::Fail("Missing include".into()),
            },
            tiers_passed: vec![TierResult {
                tier: ValidationTier::Structural,
                status: ValidationStatus::Pass,
            }],
        };
        assert_eq!(
            outcome2.format_output(),
            "Structural validation: PASS\nGenerated header validation: FAIL\n\nRolling back...\n\nRepository restored."
        );
    }

    #[test]
    fn test_file_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let p1 = dir.path().join("a.txt");
        let p2 = dir.path().join("b.txt");

        fs::write(&p1, "hello").unwrap();

        let snap = FileSnapshot::capture(&[p1.clone(), p2.clone()]).unwrap();
        assert_eq!(snap.entries[&p1], Some(b"hello".to_vec()));
        assert_eq!(snap.entries[&p2], None);

        let f1 = snap.fingerprint();

        fs::write(&p1, "world").unwrap();
        let snap2 = FileSnapshot::capture(&[p1, p2]).unwrap();
        assert_ne!(f1, snap2.fingerprint());
    }

    #[test]
    fn test_transaction_apply_and_rollback() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();

        let src_dir = repo.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let consumer = src_dir.join("main.cpp");
        fs::write(&consumer, "#include \"old.h\"\nint main() {}").unwrap();

        let extraction = ExtractionResult {
            plan_id: "P1".into(),
            headers: vec![GeneratedUnit {
                path: "new.h".into(),
                symbols: vec![],
                required_deps: vec![],
            }],
            errors: vec![],
        };

        let migration = MigrationPreview {
            plan_id: "P1".into(),
            rewrites: vec![ConsumerRewrite {
                consumer_file: PathBuf::from("src/main.cpp"),
                old_includes: vec![PathBuf::from("old.h")],
                new_includes: vec![PathBuf::from("new.h")],
                side_effect_preservations: vec![],
            }],
        };

        let (snap, new_paths) =
            TransactionApplier::apply_with(repo, &extraction, &migration, &CppDependencyRewriter)
                .unwrap();

        assert!(repo.join("generated/new.h").exists());
        // Byte-parity with the pre-M18.8b apply emit (guard block + bodies).
        assert_eq!(
            fs::read_to_string(repo.join("generated/new.h")).unwrap(),
            "#pragma once\n\n"
        );
        let consumer_content = fs::read_to_string(&consumer).unwrap();
        assert!(consumer_content.contains("#include \"new.h\""));
        assert!(!consumer_content.contains("#include \"old.h\""));
        // Byte-parity with the pre-M18.8b replace loop.
        assert_eq!(consumer_content, "#include \"new.h\"\nint main() {}");

        assert!(TransactionApplier::verify_restored(&snap).is_err());

        TransactionApplier::rollback(&snap, &new_paths).unwrap();
        assert!(TransactionApplier::verify_restored(&snap).is_ok());
        assert!(!repo.join("generated/new.h").exists());
        let restored_content = fs::read_to_string(&consumer).unwrap();
        assert!(restored_content.contains("#include \"old.h\""));
    }

    #[test]
    #[allow(deprecated)] // pins the transitional 3-arg shim until M23 removal
    fn transitional_apply_performs_orchestration_only() {
        // The frozen 3-argument entry point still snapshots/creates/rolls
        // back, but performs no directive text work (plugin-free default).
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path();

        let src_dir = repo.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let consumer = src_dir.join("main.cpp");
        fs::write(&consumer, "#include \"old.h\"\nint main() {}").unwrap();

        let extraction = ExtractionResult {
            plan_id: "P1".into(),
            headers: vec![GeneratedUnit {
                path: "new.h".into(),
                symbols: vec![],
                required_deps: vec![],
            }],
            errors: vec![],
        };

        let migration = MigrationPreview {
            plan_id: "P1".into(),
            rewrites: vec![ConsumerRewrite {
                consumer_file: PathBuf::from("src/main.cpp"),
                old_includes: vec![PathBuf::from("old.h")],
                new_includes: vec![PathBuf::from("new.h")],
                side_effect_preservations: vec![],
            }],
        };

        let (snap, new_paths) = TransactionApplier::apply(repo, &extraction, &migration).unwrap();

        // Created + rolled back, but text untouched and preamble-free.
        assert_eq!(
            fs::read_to_string(&consumer).unwrap(),
            "#include \"old.h\"\nint main() {}"
        );
        assert_eq!(fs::read_to_string(repo.join("generated/new.h")).unwrap(), "");

        TransactionApplier::rollback(&snap, &new_paths).unwrap();
        assert!(TransactionApplier::verify_restored(&snap).is_ok());
        assert!(!repo.join("generated/new.h").exists());
    }
}
