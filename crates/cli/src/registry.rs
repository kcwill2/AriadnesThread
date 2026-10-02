//! Plugin access point for the CLI (plan M18.3, D9/M19.3 delegation).
//!
//! The plugin ENUMERATION moved to `ariadne_pipeline::build_default_registry`
//! (D9): pipeline owns the single enumeration site because cli depends on
//! pipeline, never the reverse (cycle rule). This module delegates to it and
//! stays the CLI's only registry-facing seam - no other CLI code may
//! reference a concrete plugin type; indexing, unit shaping, and worker
//! backends all route through what the builder enumerates (C++ first, then
//! Scala; registration order is a contract).

use ariadne_lang::LanguageRegistry;
use ariadne_repo::SourceLanguage;

/// The default language registry - delegation to the shared D9 builder
/// (C++ per plan sec 3.4 lifecycle table, Scala per M19.3).
pub fn default_registry() -> LanguageRegistry {
    ariadne_pipeline::build_default_registry()
}

/// Validate an explicit `--lang` selection: naming a language whose plugin
/// is not registered is a clear error, never a silently empty run (§3.3
/// fact 3's registration half — the per-unit half lives in the pipeline,
/// reporting "unhandled language" for units no plugin claims).
pub fn ensure_language_registered(
    registry: &LanguageRegistry,
    language: SourceLanguage,
) -> anyhow::Result<()> {
    if registry.plugin_for_id(language).is_none() {
        anyhow::bail!("language plugin not registered: {language:?}");
    }
    Ok(())
}

/// Parse a `--lang` value through `ariadne_repo::SourceLanguage`'s serde
/// variant names (the stored PascalCase names), also accepting the exact
/// lowercase short form (`--lang cpp` ≡ `--lang Cpp`) for ergonomics.
pub fn parse_language_arg(raw: &str) -> Result<SourceLanguage, String> {
    let mut chars = raw.chars();
    let capitalized = match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    };
    for candidate in [raw, capitalized.as_str()] {
        if let Ok(language) =
            serde_json::from_value::<SourceLanguage>(serde_json::Value::String(candidate.into()))
        {
            return Ok(language);
        }
    }
    Err(format!(
        "unknown language '{raw}'; expected cpp, scala, or other \
         (serde names: Cpp, Scala, Other)"
    ))
}

/// Language-scoped stage backend (plan M23, D-M23-7 ii, T7): the plugin
/// **of `language`** when that plugin's own capabilities enable
/// `capability`; `None` for an unregistered language or a false capability
/// (callers keep their honest error, never a C++ default). Replaces the
/// pre-flip exactly-one-claim helper `capability_backend`, which the
/// [`ScalaPlugin`](ariadne_lang_scala::ScalaPlugin) flip made ambiguous by
/// construction (two plugins claim most stages).
pub fn capability_backend_for_language<'a>(
    registry: &'a ariadne_lang::LanguageRegistry,
    language: SourceLanguage,
    capability: fn(&ariadne_lang::LanguageCapabilities) -> bool,
) -> Option<&'a dyn ariadne_lang::LanguagePlugin> {
    let plugin = registry.plugin_for_id(language)?;
    capability(&plugin.capabilities()).then_some(plugin)
}

/// Stage backend for ONE optimization plan (D-M23-7 ii): plans are cut from
/// one aggregation file, so their units and consumers share the target
/// file's language — resolved through the registry detector, never
/// defaulted to C++. The error names the stage so a failure is honest about
/// which backend was missing.
pub fn plan_backend<'a>(
    registry: &'a ariadne_lang::LanguageRegistry,
    plan_target_file: &std::path::Path,
    capability: fn(&ariadne_lang::LanguageCapabilities) -> bool,
    stage: &str,
) -> anyhow::Result<&'a dyn ariadne_lang::LanguagePlugin> {
    let language = registry.detect_language(plan_target_file).ok_or_else(|| {
        anyhow::anyhow!(
            "no language plugin claims the plan target {} — cannot route the {stage} backend",
            plan_target_file.display()
        )
    })?;
    capability_backend_for_language(registry, language, capability).ok_or_else(|| {
        anyhow::anyhow!(
            "the {language:?} plugin does not enable the {stage} capability for plan target {}",
            plan_target_file.display()
        )
    })
}

/// The incremental-analysis backend for a build's discovered units: distinct
/// `tu.language` set dispatch (plan sec 3.3 fact 3 - a unit language with no
/// registered plugin is an "unhandled language" error, never a C++ default);
/// zero units fall back to the sole plugin (empty-repo behavior unchanged);
/// more than one distinct language errors until the M21 language-scoped cache.
pub fn incremental_backend<'a>(
    registry: &'a ariadne_lang::LanguageRegistry,
    build_discovery: &ariadne_build::BuildDiscoveryResult,
) -> Result<&'a dyn ariadne_lang::LanguagePlugin, String> {
    let distinct = distinct_unit_languages(build_discovery);
    match distinct.len() {
        0 => registry.sole_plugin().ok_or_else(|| {
            "incremental indexing needs exactly one registered language plugin \
             (multi-language dispatch lands with the language-scoped cache (M21))"
                .to_string()
        }),
        1 => {
            let language = distinct
                .into_iter()
                .next()
                .expect("one distinct language present");
            registry.plugin_for_id(language).ok_or_else(|| {
                format!("unhandled language: no plugin is registered for {language:?} units")
            })
        }
        _ => Err(format!(
            "multi-language incremental analysis lands with the language-scoped \
             cache (M21); discovered languages: {distinct:?}"
        )),
    }
}

/// Distinct `tu.language` values over a build's units, first-seen order.
fn distinct_unit_languages(
    build_discovery: &ariadne_build::BuildDiscoveryResult,
) -> Vec<ariadne_repo::SourceLanguage> {
    let mut distinct: Vec<ariadne_repo::SourceLanguage> = Vec::new();
    for unit in &build_discovery.translation_units {
        if !distinct.contains(&unit.language) {
            distinct.push(unit.language);
        }
    }
    distinct
}

/// Catalog lookup over the discovered files: `FileId` → its
/// [`SourceLanguage`](ariadne_repo::SourceLanguage) (plan M23, D-M23-7).
/// The CLI's language-routed dispatch resolves plugins through this map —
/// consumer discovery (T7 S2) and the per-language plan/backend routing
/// (T7 S3) share it; files absent from the catalog have no language and
/// are never dispatched (no C++ default, per §3.3 fact 3's discipline).
pub fn catalog_languages(
    discovery: &ariadne_repo::RepositoryDiscoveryResult,
) -> std::collections::BTreeMap<ariadne_repo::FileId, SourceLanguage> {
    discovery
        .files
        .iter()
        .map(|record| (record.id, record.language))
        .collect()
}

/// The plugin registered for a file's catalog language, or `None` when the
/// file is not in the catalog or its language has no plugin (D-M23-7).
pub fn plugin_for_catalog_file<'a>(
    registry: &'a LanguageRegistry,
    catalog: &std::collections::BTreeMap<ariadne_repo::FileId, SourceLanguage>,
    file_id: ariadne_repo::FileId,
) -> Option<&'a dyn ariadne_lang::LanguagePlugin> {
    registry.plugin_for_id(*catalog.get(&file_id)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_enums_cpp_then_scala() {
        let registry = default_registry();
        assert_eq!(
            registry.languages(),
            vec![SourceLanguage::Cpp, SourceLanguage::Scala],
            "registration ORDER is a contract: C++ first keeps unit_shaper_for()/\
             detect_language C++ precedence byte-identical (M19.3)"
        );
        assert!(registry.plugin_for_id(SourceLanguage::Cpp).is_some());
        assert!(registry.plugin_for_id(SourceLanguage::Scala).is_some());
        assert!(registry.detect_language(std::path::Path::new("src/a.cpp")).is_some());
        assert_eq!(
            registry.detect_language(std::path::Path::new("src/Main.scala")),
            Some(SourceLanguage::Scala)
        );
    }

    #[test]
    fn ensure_language_registered_accepts_cpp_and_scala_rejects_other() {
        let registry = default_registry();
        ensure_language_registered(&registry, SourceLanguage::Cpp).unwrap();
        ensure_language_registered(&registry, SourceLanguage::Scala)
            .expect("Scala plugin registers in M19.3");
        let err = ensure_language_registered(&registry, SourceLanguage::Other).unwrap_err();
        assert!(
            err.to_string().contains("language plugin not registered"),
            "clear registration error: {err}"
        );
    }

    #[test]
    fn registries_enumerate_identical_languages() {
        // D9 (plan note (c)): cli and pipeline must never enumerate divergent
        // plugin sets. Both must agree on the language vec AND on the
        // registry-selected shaper's build markers.
        let cli_registry = default_registry();
        let pipeline_registry = ariadne_pipeline::default_registry();
        assert_eq!(
            cli_registry.languages(),
            pipeline_registry.languages(),
            "D9: cli and pipeline registries enumerate identical languages"
        );
        let cli_markers = cli_registry
            .unit_shaper_for()
            .expect("cli registry selects a unit shaper")
            .build_markers();
        let pipeline_markers = pipeline_registry
            .unit_shaper_for()
            .expect("pipeline registry selects a unit shaper")
            .build_markers();
        assert_eq!(
            cli_markers, pipeline_markers,
            "D9: registries select the same unit shaper"
        );
    }

    #[test]
    fn parse_language_arg_accepts_serde_names_case_insensitively() {
        assert_eq!(parse_language_arg("cpp").unwrap(), SourceLanguage::Cpp);
        assert_eq!(parse_language_arg("Cpp").unwrap(), SourceLanguage::Cpp);
        assert_eq!(parse_language_arg("scala").unwrap(), SourceLanguage::Scala);
        assert_eq!(parse_language_arg("other").unwrap(), SourceLanguage::Other);
    }

    #[test]
    fn parse_language_arg_rejects_unknown_values() {
        for bad in ["", "rust", "py", "Cpp20"] {
            assert!(parse_language_arg(bad).is_err(), "'{bad}' must be rejected");
        }
    }

    // m19 capability-backend locks, EVOLVED at the T7 flip (D-M23-7 ii):
    // the pre-flip form asserted C++ as the SOLE claimant (Scala's false
    // plan:381 literals); the flip makes resolution language-scoped, so
    // each stage resolves to the claiming language's OWN plugin — the
    // m19 spirit (no ambiguity, no C++ default) survives, the sole-
    // claimant wording does not.
    fn backend_resolves_to_owning_language(
        capability: &str,
        claim: fn(&ariadne_lang::LanguageCapabilities) -> bool,
        language: SourceLanguage,
    ) {
        let registry = default_registry();
        let backend = capability_backend_for_language(&registry, language, claim)
            .unwrap_or_else(|| panic!("the {language:?} plugin must own its {capability} stage"));
        assert_eq!(
            backend.id(),
            language,
            "D-M23-7 ii: language-scoped resolution routes {capability} to the \
             {language:?} plugin itself — never shared, never defaulted"
        );
    }

    #[test]
    fn capability_backend_resolves_extraction_to_cpp() {
        backend_resolves_to_owning_language("extraction", |caps| caps.extraction, SourceLanguage::Cpp);
    }

    #[test]
    fn capability_backend_resolves_rewriting_to_cpp() {
        backend_resolves_to_owning_language("rewriting", |caps| caps.rewriting, SourceLanguage::Cpp);
    }

    #[test]
    fn capability_backend_resolves_validation_to_cpp() {
        backend_resolves_to_owning_language("validation", |caps| caps.validation, SourceLanguage::Cpp);
    }

    #[test]
    fn capability_backend_resolves_all_gated_stages_to_scala() {
        for (stage, claim) in [
            ("extraction", (|caps: &ariadne_lang::LanguageCapabilities| caps.extraction) as fn(&_) -> bool),
            ("rewriting", |caps| caps.rewriting),
            ("validation", |caps| caps.validation),
            ("planning", |caps| caps.planning),
        ] {
            backend_resolves_to_owning_language(stage, claim, SourceLanguage::Scala);
        }
    }

    #[test]
    fn capability_backend_for_language_unregistered_language_is_none_never_cpp_default() {
        let registry = default_registry();
        assert!(
            capability_backend_for_language(&registry, SourceLanguage::Other, |caps| caps.extraction)
                .is_none(),
            "an unclaimed language has NO backend — never a C++ fallback"
        );
        let empty = ariadne_lang::LanguageRegistry::new();
        assert!(
            capability_backend_for_language(&empty, SourceLanguage::Cpp, |caps| caps.extraction)
                .is_none()
        );
    }

    fn unit_with_language(
        rel: &str,
        language: SourceLanguage,
    ) -> ariadne_build::TranslationUnit {
        ariadne_build::TranslationUnit {
            file_id: ariadne_repo::FileId::from_relative_path(rel),
            source_path: std::path::PathBuf::from(rel),
            context_name: "root".to_string(),
            compile_command: ariadne_build::CompileCommand {
                directory: std::path::PathBuf::from("."),
                file: std::path::PathBuf::from(rel),
                command: None,
                arguments: vec![],
                context_name: "root".to_string(),
                output: None,
            },
            language,
        }
    }

    fn build_of(languages: &[(&str, SourceLanguage)]) -> ariadne_build::BuildDiscoveryResult {
        ariadne_build::BuildDiscoveryResult {
            repo_root: std::path::PathBuf::from("."),
            contexts: vec![],
            translation_units: languages
                .iter()
                .map(|(rel, language)| unit_with_language(rel, *language))
                .collect(),
        }
    }

    #[test]
    fn incremental_backend_picks_cpp_for_cpp_units_despite_scala_registered() {
        let registry = default_registry();
        let build = build_of(&[("a.cpp", SourceLanguage::Cpp)]);
        let plugin = incremental_backend(&registry, &build).expect("Cpp units dispatch to Cpp");
        assert_eq!(plugin.id(), SourceLanguage::Cpp);
    }

    #[test]
    fn incremental_backend_reports_unhandled_language_never_cpp_default() {
        let registry = default_registry();
        let build = build_of(&[("mystery.x", SourceLanguage::Other)]);
        let err = incremental_backend(&registry, &build)
            .err()
            .expect("sec 3.3 fact 3: unregistered unit language must error");
        assert!(
            err.contains("unhandled language"),
            "sec 3.3 fact 3 requires the unhandled-language error, got: {err}"
        );
    }

    #[test]
    fn incremental_backend_rejects_mixed_languages_with_m21_message() {
        let registry = default_registry();
        let build = build_of(&[
            ("a.cpp", SourceLanguage::Cpp),
            ("B.scala", SourceLanguage::Scala),
        ]);
        let err = incremental_backend(&registry, &build)
            .err()
            .expect("mixed languages must error until M21");
        assert!(
            err.contains("language-scoped cache (M21)"),
            "mixed languages must name the M21 cache, got: {err}"
        );
    }

    #[test]
    fn incremental_backend_zero_units_keeps_sole_plugin_fallback() {
        let registry = default_registry();
        let build = build_of(&[]);
        let err = incremental_backend(&registry, &build)
            .err()
            .expect("empty repo with multi-plugin registry must error, not guess");
        assert!(
            err.contains("exactly one registered language plugin"),
            "empty-repo fallback stays sole-plugin semantics, got: {err}"
        );
    }

    fn discovery_of(rels: &[(&str, SourceLanguage)]) -> ariadne_repo::RepositoryDiscoveryResult {
        ariadne_repo::RepositoryDiscoveryResult {
            root: std::path::PathBuf::from("."),
            files: rels
                .iter()
                .map(|(rel, language)| ariadne_repo::FileRecord {
                    id: ariadne_repo::FileId::from_relative_path(rel),
                    path: std::path::PathBuf::from(rel),
                    canonical_path: std::path::PathBuf::from(rel),
                    kind: ariadne_repo::FileKind::Source,
                    size: 0,
                    content_hash: String::new(),
                    ownership: ariadne_repo::FileOwnership::Repository,
                    optimization_policy: ariadne_repo::OptimizationPolicy::Optimizable,
                    language: *language,
                })
                .collect(),
            ownership_summary: Default::default(),
            policy_summary: Default::default(),
        }
    }

    #[test]
    fn catalog_helpers_route_plugin_per_file_language_never_cpp_default() {
        let registry = default_registry();
        let discovery = discovery_of(&[
            ("a.cpp", SourceLanguage::Cpp),
            ("B.scala", SourceLanguage::Scala),
        ]);
        let catalog = catalog_languages(&discovery);
        assert_eq!(
            plugin_for_catalog_file(&registry, &catalog, ariadne_repo::FileId::from_relative_path("a.cpp"))
                .expect("cpp file dispatches to the cpp plugin")
                .id(),
            SourceLanguage::Cpp
        );
        assert_eq!(
            plugin_for_catalog_file(&registry, &catalog, ariadne_repo::FileId::from_relative_path("B.scala"))
                .expect("scala file dispatches to the scala plugin")
                .id(),
            SourceLanguage::Scala
        );
        assert!(
            plugin_for_catalog_file(
                &registry,
                &catalog,
                ariadne_repo::FileId::from_relative_path("ghost.cpp")
            )
            .is_none(),
            "a file outside the catalog has NO plugin — never a C++ default (D-M23-7)"
        );
    }
}
