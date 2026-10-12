//! AR-25 lock: the CLI incremental seam propagates TYPED errors through
//! `anyhow` (A7 rider - the consumer side of AR-24's typed seam errors).
//!
//! Two legs, one per error family the mission names:
//!
//! 1. The dispatch seam itself. Each forced `incremental_backend` failure is
//!    propagated through the call site's live form (the `forced_seam_error`
//!    helper mirrors `commands/analyze.rs` verbatim at each commit) and must
//!    downcast back to the typed `IncrementalDispatchError` variant.
//!    Pre-AR-25 the seam returned `String` and the call site flattened it
//!    with `.map_err(anyhow::Error::msg)` - a form that preserves the message
//!    but destroys every type - so the downcast assertions are the layer-drop
//!    proof (RED before the fix, GREEN after). The `assert_eq` on
//!    `to_string()` FIRST in each test is the byte lock: it held pre-fix (the
//!    red run reached the downcast line) and holds post-fix - the forced
//!    scenario captured before/after with the same expected bytes.
//! 2. The kind-erasure witness vs the consuming form: the pre-AR-25 seam put
//!    a `String` into the anyhow payload (via `Error::msg` at the call site),
//!    so no typed dispatch KIND existed to recover - downcasting to a seam
//!    kind returned `None` while the text round-tripped. anyhow 1.0.104
//!    preserves the payload's own type identity, so what the dropped form
//!    lost is the branchable kind, not the bytes. The consuming form
//!    (`?` == `Error::new` over `std::error::Error`) transports a real
//!    `ariadne_lang::PluginError` intact - downcastable, Display verbatim.

use ariadne_build::BuildDiscoveryResult;
use ariadne_cli::registry::{IncrementalDispatchError, default_registry};
use ariadne_repo::SourceLanguage;

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

fn build_of(languages: &[(&str, SourceLanguage)]) -> BuildDiscoveryResult {
    BuildDiscoveryResult {
        repo_root: std::path::PathBuf::from("."),
        contexts: vec![],
        translation_units: languages
            .iter()
            .map(|(rel, language)| unit_with_language(rel, language.clone()))
            .collect(),
    }
}

/// Propagates the seam exactly the way the live consumer does at this commit:
/// plain `?` over the seam's `anyhow::Result`, verbatim mirror of
/// `commands/analyze.rs` post-AR-25 (pre-AR-25 this body carried the
/// `.map_err(anyhow::Error::msg)` that the production line dropped). With the
/// re-wrap gone, the seam error's typed source survives to this call site.
fn forced_seam_error(build: &BuildDiscoveryResult) -> anyhow::Error {
    route_build(build).err().expect("these scenarios must error")
}

fn route_build(build: &BuildDiscoveryResult) -> anyhow::Result<()> {
    let registry = default_registry();
    let _plugin = ariadne_cli::registry::incremental_backend(&registry, build)?;
    Ok(())
}

#[test]
fn ar25_sole_plugin_rejection_is_a_typed_source_through_the_seam() {
    let err = forced_seam_error(&build_of(&[]));
    assert_eq!(
        err.to_string(),
        "incremental indexing needs exactly one registered language plugin \
         (multi-language dispatch lands with the language-scoped cache (M21))",
        "AR-25 byte lock: the CLI-visible string is unchanged by the typed seam"
    );
    let typed = err
        .downcast_ref::<IncrementalDispatchError>()
        .expect("AR-25 layer-drop proof: the seam error survives anyhow as a typed source");
    assert!(
        matches!(typed, IncrementalDispatchError::SolePluginRequired),
        "the kind discriminates without string matching: {typed:?}"
    );
}

#[test]
fn ar25_unhandled_language_is_a_typed_source_through_the_seam() {
    let err = forced_seam_error(&build_of(&[("mystery.x", SourceLanguage::Other)]));
    assert_eq!(
        err.to_string(),
        "unhandled language: no plugin is registered for Other units",
        "AR-25 byte lock: the CLI-visible string is unchanged by the typed seam"
    );
    let typed = err
        .downcast_ref::<IncrementalDispatchError>()
        .expect("AR-25 layer-drop proof: the seam error survives anyhow as a typed source");
    assert_eq!(
        typed,
        &IncrementalDispatchError::UnhandledLanguage {
            language: SourceLanguage::Other,
        },
        "the kind names the rejected language as data, not as rendered text"
    );
}

#[test]
fn ar25_mixed_languages_is_a_typed_source_through_the_seam() {
    let err = forced_seam_error(&build_of(&[
        ("a.cpp", SourceLanguage::Cpp),
        ("B.scala", SourceLanguage::Scala),
    ]));
    assert_eq!(
        err.to_string(),
        "multi-language incremental analysis lands with the language-scoped \
         cache (M21); discovered languages: [Cpp, Scala]",
        "AR-25 byte lock: the CLI-visible string is unchanged by the typed seam"
    );
    let typed = err
        .downcast_ref::<IncrementalDispatchError>()
        .expect("AR-25 layer-drop proof: the seam error survives anyhow as a typed source");
    assert_eq!(
        typed,
        &IncrementalDispatchError::MultiLanguageDispatch {
            languages: vec![SourceLanguage::Cpp, SourceLanguage::Scala],
        },
        "the kind carries the discovered set in first-seen order"
    );
}

#[test]
fn ar25_msg_string_payload_carries_no_kind_the_consuming_form_transports_one() {
    // Witness of the erasure the mission removes: the pre-AR-25 seam payload
    // was a String (Error::msg at the call site). The text round-trips - that
    // is why byte-string locks could not see the damage - but no typed
    // dispatch KIND exists in the chain to branch on. (anyhow 1.0.104 keeps
    // the payload's own type identity, so the loss is the kind, not bytes;
    // adjudicated on this run's own evidence, see journal.)
    let pre_form = Err::<(), _>(
        "unhandled language: no plugin is registered for Other units".to_string(),
    )
    .map_err(anyhow::Error::msg)
    .unwrap_err();
    assert!(
        pre_form
            .downcast_ref::<IncrementalDispatchError>()
            .is_none(),
        "witness: the pre-AR-25 String payload carries no seam kind to branch on"
    );
    assert_eq!(
        pre_form.to_string(),
        "unhandled language: no plugin is registered for Other units",
        "the text survived while the kind never existed - string locks alone stay blind to this"
    );

    // The plugin-seam leg of the mission: the form the seam now uses
    // (`?` == Error::new over std::error::Error) transports a real
    // `PluginError` intact - downcastable by callers, Display verbatim.
    let typed = ariadne_lang::PluginError::CompileFailure {
        message: "scalac exited unsuccessfully: AR-25 sentinel".to_string(),
    };
    let propagated = Err::<(), _>(typed.clone())
        .map_err(anyhow::Error::new)
        .unwrap_err();
    assert_eq!(
        propagated.downcast_ref::<ariadne_lang::PluginError>(),
        Some(&typed),
        "the consuming form (?-conversion == Error::new) transports the seam error intact"
    );
    assert_eq!(
        propagated.to_string(),
        typed.to_string(),
        "Display stays the plugin's verbatim text through the transport"
    );
}
