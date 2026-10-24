//! Validation harness and transactional verification.

use anyhow::Result;
use ariadne_lang::{
    render_unit, CompilerValidator, DependencyRewriter, RewriteSpec, SourceExtractor, SymbolBody,
    ToolchainStatus,
};
use ariadne_transform::{ExtractionResult, MigrationPreview};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
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
    /// M18.10 (plan §3.5 transactions row): the tier could not run at all
    /// (e.g. the language toolchain is legitimately absent). NOT a failure —
    /// rollback fires only on [`ValidationStatus::Fail`].
    Skipped(String),
}

impl ValidationStatus {
    /// Single source of the rollback decision (plan §6.2 `validate::lib` row):
    /// only a `Fail` counts as failure — `Pass` and `Skipped` never roll back.
    pub fn is_fail(&self) -> bool {
        matches!(self, ValidationStatus::Fail(_))
    }
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
    let status = match &tr.status {
        ValidationStatus::Pass => "PASS".to_string(),
        ValidationStatus::Fail(_) => "FAIL".to_string(),
        ValidationStatus::Skipped(reason) => format!("SKIPPED: {}", reason),
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

pub struct TransactionApplier;

impl TransactionApplier {
    /// Transactional apply: snapshot the touched files, write generated
    /// headers through `extractor` (the plugin owns every generated-file
    /// byte, plan M23 I3) and consumer rewrites through `rewriter` (which
    /// owns every directive spelling), and return the snapshot plus created
    /// paths. AR-04: the snapshot is captured before any mutation, and any
    /// error from the write/rewriter/strip phases rolls every written target
    /// back through [`TransactionApplier::rollback`] before propagating, so
    /// an `Err` never leaves a partial mutation on disk (mirrors the
    /// caller-side tier-failure rollback in cli `optimize`/`transform`).
    ///
    /// T7b move-removal (D-M23-5, I5): when a rewrite points at moved
    /// symbols generated into new files, the symbols' existing source files
    /// are also snapshotted, and the plugin's move hook
    /// `rewriter.move_out_top_level_symbols` - offered the plan id as marker
    /// provenance - decides the strip inside the same transaction: `Some`
    /// writes the moved-out text back (rollback restores the original),
    /// `None` (a refusal, or a copy-only language's default) leaves the file
    /// byte-identical, so the copy fallback survives. The
    /// decision lives entirely in the hook result; validate names no
    /// language (I2).
    pub fn apply_with(
        repo_root: &Path,
        extraction: &ExtractionResult,
        migration: &MigrationPreview,
        rewriter: &dyn DependencyRewriter,
        extractor: &dyn SourceExtractor,
    ) -> Result<(FileSnapshot, Vec<PathBuf>)> {
        let mut paths_to_snapshot = Vec::new();
        let mut new_paths = Vec::new();

        let gen_dir = if extractor.places_generated_units_in_place() {
            repo_root.to_path_buf()
        } else {
            repo_root.join("generated")
        };
        for unit in &extraction.headers {
            paths_to_snapshot.push(gen_dir.join(&unit.path));
        }

        for rw in &migration.rewrites {
            paths_to_snapshot.push(repo_root.join(&rw.consumer_file));
        }

        let move_removals = collect_move_removals(repo_root, migration, extraction);
        for (source, _) in &move_removals {
            paths_to_snapshot.push(repo_root.join(source));
        }

        let snapshot = FileSnapshot::capture(&paths_to_snapshot)?;

        // AR-04 (audit T1): the tier-failure rollback pattern the callers
        // apply around this function (`if fail { rollback(&snapshot,
        // &new_paths) }`) is enforced INSIDE it too: any error from the
        // write/rewriter/strip phases restores every written target before
        // the error propagates, so an `Err` return can never leak a partial
        // mutation. Success-path behavior is unchanged (phases moved
        // verbatim into `write_all_targets`).
        if let Err(cause) = Self::write_all_targets(
            repo_root,
            &gen_dir,
            extraction,
            migration,
            rewriter,
            extractor,
            &move_removals,
            &mut new_paths,
        ) {
            return Err(Self::rolled_back_error(cause, &snapshot, &new_paths));
        }

        Ok((snapshot, new_paths))
    }

    /// The write/rewriter/strip phases of `apply_with`, moved verbatim:
    /// stage the generated dir, render+write each unit through `extractor`,
    /// rewrite each consumer through `rewriter`, then offer the move-removal
    /// strip. Any `Err` here means the tree is partially mutated; the caller
    /// owns the rollback (AR-04).
    fn write_all_targets(
        repo_root: &Path,
        gen_dir: &Path,
        extraction: &ExtractionResult,
        migration: &MigrationPreview,
        rewriter: &dyn DependencyRewriter,
        extractor: &dyn SourceExtractor,
        move_removals: &BTreeSet<(PathBuf, Vec<String>)>,
        new_paths: &mut Vec<PathBuf>,
    ) -> Result<()> {
        fs::create_dir_all(gen_dir)?;

        for unit in &extraction.headers {
            let path = gen_dir.join(&unit.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            if !path.exists() {
                new_paths.push(path.clone());
            }

            let bodies: Vec<SymbolBody> = unit
                .symbols
                .iter()
                .map(|sym| SymbolBody {
                    name: sym.name.clone(),
                    body: sym.body.clone(),
                })
                .collect();
            let deps: Vec<String> = unit
                .required_deps
                .iter()
                .map(|dep| dep.display().to_string())
                .collect();
            let content = render_unit(extractor, &bodies, unit.primary_source_text.as_deref(), &deps);
            fs::write(&path, content)?;
        }

        for rw in &migration.rewrites {
            let path = repo_root.join(&rw.consumer_file);
            let source = fs::read_to_string(&path)?;
            let moved_module_ids: Vec<String> = extraction
                .headers
                .iter()
                .filter(|unit| rw.new_includes.contains(&unit.path))
                .flat_map(|unit| unit.symbols.iter().map(|sym| sym.qualified_id.clone()))
                .collect();
            let spec = rewriter
                .consumer_rewrite_spec(&rw.old_includes, &rw.new_includes, &moved_module_ids)
                .unwrap_or_else(|| RewriteSpec {
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
                });
            let content = rewriter.rewrite(&source, &spec);
            fs::write(&path, content)?;
        }

        strip_moved_symbols(repo_root, move_removals, rewriter, &migration.plan_id)
    }

    /// AR-04 error decoration: run the same rollback the tier-failure path
    /// runs, then rethrow the original cause with a "rolled back N files"
    /// line, where N counts every path the rollback pass processes (each
    /// snapshot entry restored/removed, each created path removed). If the
    /// rollback itself fails, that failure is propagated with the original
    /// cause as context: an unrestored tree must never be reported as
    /// rolled back.
    fn rolled_back_error(
        cause: anyhow::Error,
        snapshot: &FileSnapshot,
        new_paths: &[PathBuf],
    ) -> anyhow::Error {
        let processed_paths = snapshot.entries.len() + new_paths.len();
        match TransactionApplier::rollback(snapshot, new_paths) {
            Ok(()) => cause.context(format!("rolled back {processed_paths} files")),
            Err(rollback_err) => {
                rollback_err.context(format!("rollback incomplete after: {cause}"))
            }
        }
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

/// The existing source files whose moved symbols `apply_with` must offer to
/// `move_out_top_level_symbols`, as (repo-relative path, one unit's moved
/// symbol names sorted) pairs — one entry per moved UNIT per source file
/// (T8b): the unit is the atomic move region, so an unlocatable unit (an
/// external batch member that kept its placeholder fallback, I4) is skipped
/// whole by its own `None` hook result without blocking the strip of the
/// units that genuinely live in that file. Derivation stays fully neutral
/// (I2): a rewrite's `old_includes` entries that exist on disk, are not the
/// consumer itself, and are not a generated target (the T7b self-target
/// mirror — stripping a unit's own output would corrupt it); the names come
/// from the generated units the rewrite's `new_includes` point at.
fn collect_move_removals(
    repo_root: &Path,
    migration: &MigrationPreview,
    extraction: &ExtractionResult,
) -> BTreeSet<(PathBuf, Vec<String>)> {
    let mut removals: BTreeSet<(PathBuf, Vec<String>)> = BTreeSet::new();
    for rewrite in &migration.rewrites {
        for old_dep in &rewrite.old_includes {
            if old_dep == &rewrite.consumer_file || is_generated_target(old_dep, extraction) {
                continue;
            }
            if !repo_root.join(old_dep).is_file() {
                continue;
            }
            for unit in &extraction.headers {
                if !rewrite.new_includes.contains(&unit.path) || unit.symbols.is_empty() {
                    continue;
                }
                let names: Vec<String> =
                    unit.symbols.iter().map(|sym| sym.name.clone()).collect();
                removals.insert((old_dep.clone(), names));
            }
        }
    }
    removals
}

/// A rewrite's old dependency path that is itself one of this plan's
/// generated unit targets (shape-impossible for C++: targets are
/// `{name}_extracted.h` under `generated/`).
fn is_generated_target(path: &Path, extraction: &ExtractionResult) -> bool {
    extraction.headers.iter().any(|unit| unit.path == path)
}

/// Offer each moved unit to the plugin's move hook
/// `move_out_top_level_symbols` inside the caller's snapshot, binding the
/// plan id into the region markers as provenance; a `Some` result is written
/// back (I5 move), a `None` result (a refusal, a copy-only language's
/// default, or a unit the file does not declare) leaves bytes untouched.
fn strip_moved_symbols(
    repo_root: &Path,
    removals: &BTreeSet<(PathBuf, Vec<String>)>,
    rewriter: &dyn DependencyRewriter,
    plan_id: &str,
) -> Result<()> {
    for (relative, names) in removals {
        let path = repo_root.join(relative);
        let content = fs::read_to_string(&path)?;
        if let Some(stripped) = rewriter.move_out_top_level_symbols(&content, names, plan_id) {
            fs::write(&path, stripped)?;
        }
    }
    Ok(())
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

    /// Tier 3. Pre-M18.10 behavior was the include-existence walk only — this
    /// tier never ran a compiler. Kickoff decision D-M23-12 (plan
    /// `plans/m23-symbol-isolation-kickoff.md` §3) defers real compile
    /// invocation out of M23 for BOTH languages: extracted headers are not
    /// yet self-contained (extraction include-closure is its own workstream),
    /// so the honest M23 tier-3 deliverable is the probe-backed skip below.
    /// M18.10 adds the honest skip: when the injected plugin validator's
    /// [`CompilerValidator::toolchain_status`] reports `Missing`, a would-be
    /// `Pass` becomes `Skipped`, so this tier can no longer claim a TU
    /// compiles when no compiler was consulted. Existence-check `Fail`s are
    /// toolchain-independent and unchanged, so rollback semantics are
    /// identical. `Unknown`/absent validator keeps today's `Pass` behavior
    /// (absence unproven); probing lives in the plugin, never here.
    pub fn validate_affected_tus(
        migration: &MigrationPreview,
        header_output_dir: &Path,
        validator: Option<&dyn CompilerValidator>,
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
        let status = match validator.map(|v| v.toolchain_status()) {
            Some(ToolchainStatus::Missing) => ValidationStatus::Skipped(
                "no compiler toolchain available to compile affected TUs".into(),
            ),
            _ => ValidationStatus::Pass,
        };
        TierResult {
            tier: ValidationTier::AffectedTUs,
            status,
        }
    }
}

#[cfg(test)]
mod orchestration_tests;

#[cfg(test)]
mod uta2_r4_pins;

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_lang::{CompilerValidator, ToolchainStatus};
    use ariadne_transform::{ConsumerRewrite, GeneratedUnit};

    // The applier-orchestration cluster (the neutral fakes, the apply_with
    // writes/namespace/rollback tests, and the UTA-T4 item 5/6 coverage)
    // lives in `orchestration_tests.rs` (UTA-T4: growth guard - lib.rs was
    // one test module away of crossing the 1000-line hygiene ceiling).

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

    // test_transaction_apply_and_rollback (real C++ rewriter/extractor
    // byte-parity baselines) moved verbatim to
    // `crates/validate/tests/transaction_apply_test.rs` (M23 T3, D-M23-10):
    // it names the C++ plugin and lives outside the guard walk scope.

    // The neutral rewriter/extractor fakes and the apply_with writes,
    // namespace and rollback tests were moved verbatim to
    // `orchestration_tests.rs` at UTA-T4 (growth guard).

    // ── M18.10: ValidationStatus::Skipped (§6.2 `validate::lib` row) ────────

    fn tier3_migration(new_include: &str) -> MigrationPreview {
        MigrationPreview {
            plan_id: "P1".into(),
            rewrites: vec![ConsumerRewrite {
                consumer_file: PathBuf::from("src/main.cpp"),
                old_includes: vec![PathBuf::from("old.h")],
                new_includes: vec![PathBuf::from(new_include)],
                side_effect_preservations: vec![],
            }],
        }
    }

    struct FixedToolchain(ToolchainStatus);

    impl CompilerValidator for FixedToolchain {
        // AR-24: compile-forced seam signature only; body and locks untouched.
        fn validate_compile(
            &self,
            _files: &[PathBuf],
            _args: &[String],
        ) -> Result<(), ariadne_lang::PluginError> {
            Ok(())
        }

        fn toolchain_status(&self) -> ToolchainStatus {
            self.0.clone()
        }
    }

    #[test]
    fn skipped_status_formats_distinctly() {
        let outcome = ApplyOutcome::Committed {
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
                    status: ValidationStatus::Skipped("no compiler toolchain".into()),
                },
            ],
        };
        assert_eq!(
            outcome.format_output(),
            "Structural validation: PASS\nGenerated header validation: PASS\n\
             Affected TU validation: SKIPPED: no compiler toolchain\n\nCommitted."
        );
    }

    #[test]
    fn status_serde_names_stable_and_skipped_is_additive() {
        assert_eq!(
            serde_json::to_string(&ValidationStatus::Pass).unwrap(),
            "\"Pass\""
        );
        assert_eq!(
            serde_json::to_string(&ValidationStatus::Fail("boom".into())).unwrap(),
            "{\"Fail\":\"boom\"}"
        );
        let skipped = ValidationStatus::Skipped("toolchain missing".into());
        let json = serde_json::to_string(&skipped).unwrap();
        assert_eq!(json, "{\"Skipped\":\"toolchain missing\"}");
        assert_eq!(
            serde_json::from_str::<ValidationStatus>(&json).unwrap(),
            skipped
        );
    }

    #[test]
    fn rollback_predicate_fires_only_on_fail() {
        assert!(ValidationStatus::Fail("x".into()).is_fail());
        assert!(!ValidationStatus::Pass.is_fail());
        assert!(!ValidationStatus::Skipped("toolchain missing".into()).is_fail());
    }

    #[test]
    fn affected_tier_skips_when_injected_toolchain_is_missing() {
        let dir = tempfile::tempdir().unwrap();
        let gen = dir.path().join("generated");
        fs::create_dir_all(&gen).unwrap();
        fs::write(gen.join("new.h"), "#pragma once\n").unwrap();

        let result = TierValidator::validate_affected_tus(
            &tier3_migration("new.h"),
            &gen,
            Some(&FixedToolchain(ToolchainStatus::Missing)),
        );
        assert_eq!(result.tier, ValidationTier::AffectedTUs);
        match &result.status {
            ValidationStatus::Skipped(reason) => assert!(!reason.is_empty()),
            other => panic!("missing toolchain must yield Skipped, got {other:?}"),
        }
        assert!(!result.status.is_fail(), "Skipped must never roll back");
    }

    #[test]
    fn affected_tier_fail_precedes_toolchain_skip() {
        let dir = tempfile::tempdir().unwrap();
        let gen = dir.path().join("generated");
        fs::create_dir_all(&gen).unwrap();

        let result = TierValidator::validate_affected_tus(
            &tier3_migration("ghost.h"),
            &gen,
            Some(&FixedToolchain(ToolchainStatus::Missing)),
        );
        match &result.status {
            ValidationStatus::Fail(msg) => assert!(msg.contains("ghost.h")),
            other => panic!("missing include must still Fail with no toolchain: {other:?}"),
        }
    }

    #[test]
    fn affected_tier_keeps_pass_for_available_unknown_or_absent_validator() {
        let dir = tempfile::tempdir().unwrap();
        let gen = dir.path().join("generated");
        fs::create_dir_all(&gen).unwrap();
        fs::write(gen.join("new.h"), "#pragma once\n").unwrap();

        for validator in [
            None,
            Some(Box::new(FixedToolchain(ToolchainStatus::Unknown)) as Box<dyn CompilerValidator>),
            Some(Box::new(FixedToolchain(ToolchainStatus::Available(
                "testcc 1.2".into(),
            )))),
        ] {
            let result = TierValidator::validate_affected_tus(
                &tier3_migration("new.h"),
                &gen,
                validator.as_deref(),
            );
            assert_eq!(
                result.status,
                ValidationStatus::Pass,
                "pre-existing behavior must survive for this toolchain case"
            );
        }
    }

    // ── UTA-T4 item 4: tier 1/2 pass-fail tables ─────────────────────────────

    fn extraction_with_errors(errors: Vec<String>) -> ExtractionResult {
        ExtractionResult {
            plan_id: "P-TIER".into(),
            headers: vec![],
            errors,
        }
    }

    fn extraction_requiring(dep: &str) -> ExtractionResult {
        ExtractionResult {
            plan_id: "P-TIER".into(),
            headers: vec![GeneratedUnit {
                path: "unit.h".into(),
                symbols: vec![],
                required_deps: vec![PathBuf::from(dep)],
                primary_source_text: None,
            }],
            errors: vec![],
        }
    }

    #[test]
    fn structural_tier_passes_on_clean_extraction() {
        assert_eq!(
            TierValidator::validate_structural(&extraction_with_errors(vec![])),
            TierResult {
                tier: ValidationTier::Structural,
                status: ValidationStatus::Pass,
            }
        );
    }

    #[test]
    fn structural_tier_fails_with_joined_engine_errors() {
        let result = TierValidator::validate_structural(&extraction_with_errors(vec![
            "Cycle detected in generated headers".into(),
            "second problem".into(),
        ]));
        assert_eq!(
            result,
            TierResult {
                tier: ValidationTier::Structural,
                // Pin the join shape too: tier text is what users read back.
                status: ValidationStatus::Fail(
                    "Cycle detected in generated headers, second problem".into()
                ),
            }
        );
        assert!(result.status.is_fail());
    }

    #[test]
    fn generated_header_tier_passes_when_dep_lives_under_output_dir() {
        let dir = tempfile::tempdir().unwrap();
        let gen = dir.path().join("generated");
        fs::create_dir_all(&gen).unwrap();
        fs::write(gen.join("dep.h"), "// dep\n").unwrap();

        assert_eq!(
            TierValidator::validate_generated_headers(&extraction_requiring("dep.h"), &gen),
            TierResult {
                tier: ValidationTier::GeneratedHeaders,
                status: ValidationStatus::Pass,
            }
        );
    }

    #[test]
    fn generated_header_tier_passes_when_dep_lives_only_under_parent() {
        // The p2 branch: a dep resolved against the repo root (the parent
        // of the generated dir in the staging scheme) must also Pass.
        let dir = tempfile::tempdir().unwrap();
        let gen = dir.path().join("generated");
        fs::create_dir_all(&gen).unwrap();
        fs::write(dir.path().join("dep.h"), "// dep at repo root\n").unwrap();

        assert_eq!(
            TierValidator::validate_generated_headers(&extraction_requiring("dep.h"), &gen),
            TierResult {
                tier: ValidationTier::GeneratedHeaders,
                status: ValidationStatus::Pass,
            }
        );
    }

    #[test]
    fn generated_header_tier_fails_when_dep_is_nowhere() {
        let dir = tempfile::tempdir().unwrap();
        let gen = dir.path().join("generated");
        fs::create_dir_all(&gen).unwrap();

        let result = TierValidator::validate_generated_headers(
            &extraction_requiring("ghost.h"),
            &gen,
        );
        assert_eq!(
            result,
            TierResult {
                tier: ValidationTier::GeneratedHeaders,
                status: ValidationStatus::Fail("Missing include: ghost.h".into()),
            }
        );
    }

    #[test]
    fn generated_header_tier_no_parent_dir_falls_back_without_panicking() {
        // The PathBuf::new() branch: an output dir with no parent (an
        // empty path) must exercise the p2 fallback cleanly - a missing
        // dep still fails with the standard message, never a panic.
        let result = TierValidator::validate_generated_headers(
            &extraction_requiring("uta_t4_no_parent_ghost.h"),
            Path::new(""),
        );
        match &result.status {
            ValidationStatus::Fail(msg) => assert_eq!(
                msg,
                "Missing include: uta_t4_no_parent_ghost.h",
                "the no-parent branch still reports the missing dep"
            ),
            other => panic!("no-parent dir with a missing dep must Fail, got {other:?}"),
        }
        assert_eq!(result.tier, ValidationTier::GeneratedHeaders);
    }
}
