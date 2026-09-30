//! Single plugin enumeration site for the CLI (plan M18.3).
//!
//! `default_registry()` is THE place every language plugin is registered;
//! no other CLI code may reference a concrete plugin type (or the
//! transitional `cpp_default_shaper()`) directly — indexing, unit shaping,
//! and worker backends all route through what is enumerated here.
//! `ScalaPlugin` arrives with M19.

use ariadne_lang::LanguageRegistry;
use ariadne_repo::SourceLanguage;

/// The default language registry: C++ (plan §3.4 lifecycle table).
pub fn default_registry() -> LanguageRegistry {
    LanguageRegistry::new().register(Box::new(ariadne_lang_cpp::CppPlugin::new()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_enums_cpp_only() {
        let registry = default_registry();
        assert_eq!(registry.languages(), vec![SourceLanguage::Cpp]);
        assert!(registry.plugin_for_id(SourceLanguage::Cpp).is_some());
        assert!(registry.detect_language(std::path::Path::new("src/a.cpp")).is_some());
        assert!(registry.detect_language(std::path::Path::new("src/Main.scala")).is_none());
    }

    #[test]
    fn ensure_language_registered_accepts_cpp_rejects_scala() {
        let registry = default_registry();
        ensure_language_registered(&registry, SourceLanguage::Cpp).unwrap();
        let err = ensure_language_registered(&registry, SourceLanguage::Scala).unwrap_err();
        assert!(
            err.to_string().contains("language plugin not registered"),
            "clear registration error: {err}"
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
}
