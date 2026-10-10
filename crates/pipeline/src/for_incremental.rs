//! A6 (AR-22): the `Pipeline::for_incremental` construction seam.
//!
//! Encapsulates the exact sequence the CLI mirrors by hand today in
//! `crates/cli/src/commands/analyze.rs` (`run_incremental_analysis_filtered`
//! lines 32-56 plus the `restrict_and_validate_units` and
//! `artifacts_are_authoritative` helpers, and the
//! `ensure_language_registered` guard from `crates/cli/src/registry.rs`):
//! locate, repo discovery, registry-shaper build discovery, the optional
//! `--lang` unit restriction with the registration and §3.3 fact-3
//! unhandled-language checks, and the artifact-authority rule the mirror
//! carries. AR-23 switches the CLI to this constructor and deletes the
//! mirror; until then nothing consumes this seam (zero behavior impact).
//!
//! Deliberate scope (journal `tasks/ar-22-handoff-journal.md`): construction
//! only - no backend plugin selection (the CLI keeps
//! `registry::incremental_backend` over `pipeline.build`), no workers knob
//! (the mirror has none), no repository/contexts artifact refresh (the
//! mirror's sequence writes nothing at construction time; only the
//! analyzer's own authoritative `save_artifacts` branch persists), and no
//! stage run.

use anyhow::{bail, Result};
use std::path::Path;

use ariadne_build::BuildDiscoveryResult;
use ariadne_lang::{LanguageId, LanguageRegistry};
use ariadne_repo::SourceLanguage;
use ariadne_report::ArtifactManager;
use ariadne_scheduler::TuStoreContext;

use crate::{default_registry, Pipeline, RunStats};

/// The unit-restriction + artifact-authority state of a `for_incremental`
/// run - the data the CLI's incremental mirror derives inline
/// (`restrict_and_validate_units`' retained set + the
/// `artifacts_are_authoritative` inputs). Public so the parity surface is
/// inspectable from tests and AR-23's switch site; `None` on every other
/// constructor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalView {
    /// The active `--lang` restriction, if any (same closed file-kind label
    /// `Pipeline::language_filter` carries).
    pub language_filter: Option<SourceLanguage>,
    /// Units in the discovered build set, before any restriction.
    pub discovered_units: usize,
    /// Units retained after the `--lang` restriction.
    pub retained_units: usize,
}

impl IncrementalView {
    /// The mirror's artifact-authority rule (analyze.rs
    /// `artifacts_are_authoritative`): only an unfiltered run whose unit
    /// set was never narrowed produces the authoritative full-repo
    /// artifacts; a filtered view must never persist under the full-stage
    /// fingerprint.
    pub fn artifacts_are_authoritative(&self) -> bool {
        self.language_filter.is_none() && self.retained_units >= self.discovered_units
    }
}

/// Registration guard - the message-text mirror of
/// `ariadne_cli::registry::ensure_language_registered` (§3.3 fact 3's
/// registration half: naming an unregistered language is a clear error,
/// never a silently empty run). Registry lookup only (R3).
fn ensure_language_registered(registry: &LanguageRegistry, language: SourceLanguage) -> Result<()> {
    if registry.plugin_for_id(LanguageId::from(language)).is_none() {
        bail!("language plugin not registered: {language:?}");
    }
    Ok(())
}

/// Restriction + validation mirror of analyze.rs
/// `restrict_and_validate_units`: with an active `--lang`, keep only units
/// the registry detects as that language (open-identity comparison, M24
/// Part A), then require every retained unit to be claimed by some
/// registered plugin. Registry lookups only (R3).
fn restrict_and_validate_units(
    registry: &LanguageRegistry,
    language: Option<SourceLanguage>,
    build: &mut BuildDiscoveryResult,
) -> Result<()> {
    if let Some(language) = language {
        build
            .translation_units
            .retain(|tu| registry.detect_language(&tu.source_path) == Some(LanguageId::from(language)));
    }
    for tu in &build.translation_units {
        if registry.detect_language(&tu.source_path).is_none() {
            bail!(
                "unhandled language for {} (no registered plugin claims it)",
                tu.source_path.display()
            );
        }
    }
    Ok(())
}

impl Pipeline {
    /// Pipeline configured for the incremental analysis path (A6, AR-22):
    /// the single authority site for the construction sequence the CLI
    /// mirrors by hand in `analyze.rs::run_incremental_analysis_filtered`
    /// today. Runs the registry's default plugin set (the same enumeration
    /// the CLI's delegating registry builds, locked identical by the D9
    /// test), validates an explicit `--lang` against the registry, computes
    /// the shared discovery/shaper/fingerprint inputs, then narrows and
    /// validates the unit set exactly as the mirror does, recording the
    /// restriction + artifact-authority state in
    /// [`Pipeline::incremental_view`].
    ///
    /// The result is INPUT state for `ariadne_scheduler::IncrementalAnalyzer`
    /// (`&pipeline.repo`, `&pipeline.discovery`, `&pipeline.build`) plus the
    /// authority decision that gates `save_artifacts`; no pipeline stage is
    /// run or persisted here, and the backend plugin stays the caller's
    /// registry dispatch (`incremental_backend`) until AR-23 wires it.
    pub fn for_incremental(
        repo_path: &Path,
        language: Option<SourceLanguage>,
    ) -> Result<Self> {
        let registry = default_registry();
        if let Some(language) = language {
            ensure_language_registered(&registry, language)?;
        }
        let (repo, discovery, mut build, fps) = Self::compute_inputs(repo_path, &registry)?;
        let discovered_units = build.translation_units.len();
        restrict_and_validate_units(&registry, language, &mut build)?;
        let retained_units = build.translation_units.len();
        let manager = ArtifactManager::prepare_for_repo(&repo)?;
        let store_ctx = TuStoreContext::new(&discovery, build.contexts.clone());
        Ok(Self {
            repo,
            manager,
            discovery,
            build,
            fps,
            stats: RunStats::default(),
            production_stats: None,
            store_ctx,
            workers: None,
            progress: None,
            registry,
            language_filter: language,
            incremental_view: Some(IncrementalView {
                language_filter: language,
                discovered_units,
                retained_units,
            }),
            include_cache: None,
            index_cache: None,
            reduced_cache: None,
            callgraph_cache: None,
            load_cache: None,
        })
    }
}
