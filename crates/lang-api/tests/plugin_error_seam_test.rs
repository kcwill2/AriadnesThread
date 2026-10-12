//! AR-24 (audit finding A7): the plugin-seam error contract itself.
//!
//! `PluginError` must (1) branch by VARIANT -- callers never pattern-match
//! rendered text -- and (2) render the producing plugin's message VERBATIM
//! through `Display`. The concrete CLI-visible strings are byte-locked at
//! their source in `lang-cpp`/`lang-scala` (`ar24_*_is_byte_locked`); this
//! suite locks the enum's structural promises with sentinel text, so the
//! two locks cannot mask each other.

use ariadne_lang_api::error::PluginError;

/// Kind branching without string matching: the three kinds answer exactly
/// one consumer question -- could the validation run at all? This is the
/// capability AR-25 consumes through the anyhow downcast path.
#[test]
fn plugin_error_branches_by_variant_not_by_string() {
    let errors = [
        PluginError::ValidationNotWired {
            message: "same text".into(),
        },
        PluginError::ToolchainUnavailable {
            message: "same text".into(),
        },
        PluginError::CompileFailure {
            message: "same text".into(),
        },
    ];
    let kinds: Vec<&str> = errors
        .iter()
        .map(|error| match error {
            PluginError::ValidationNotWired { .. } => "not-wired",
            PluginError::ToolchainUnavailable { .. } => "toolchain",
            PluginError::CompileFailure { .. } => "failure",
        })
        .collect();
    assert_eq!(
        kinds,
        vec!["not-wired", "toolchain", "failure"],
        "identical texts still discriminate by kind: the seam is typed, not stringly"
    );
}

/// Display renders the message verbatim for every variant -- the seam adds
/// no prefix, suffix, or rewording; the bytes ARE the CLI-visible contract.
#[test]
fn display_renders_the_plugin_message_verbatim_for_every_variant() {
    let sentinel = "AR24-SENTINEL: exact text must survive Display unchanged";
    let renders = [
        PluginError::ValidationNotWired { message: sentinel.into() }.to_string(),
        PluginError::ToolchainUnavailable { message: sentinel.into() }.to_string(),
        PluginError::CompileFailure { message: sentinel.into() }.to_string(),
    ];
    for rendered in renders {
        assert_eq!(rendered, sentinel, "Display must be pure passthrough");
    }
}

/// The transport AR-25 relies on: erased through `anyhow`, the typed kind
/// stays downcastable (the chain `downcast_ref::<PluginError>` will walk).
/// `anyhow` is a dev-dependency here only to prove the transport; the
/// production seam stays thiserror-typed.
#[test]
fn typed_error_downcasts_through_the_anyhow_chain() {
    let typed = PluginError::CompileFailure {
        message: "scalac exited unsuccessfully: lock".into(),
    };
    let erased = anyhow::Error::new(typed.clone());
    let recovered = erased
        .downcast_ref::<PluginError>()
        .expect("the seam error keeps its typed identity through anyhow");
    assert_eq!(recovered, &typed);
}

/// Std-level identity: usable as `&dyn std::error::Error` and downcastable
/// back to the exact value.
#[test]
fn typed_error_downcasts_as_std_error() {
    let typed = PluginError::ToolchainUnavailable {
        message: "toolchain absent".into(),
    };
    let erased: &dyn std::error::Error = &typed;
    assert_eq!(erased.to_string(), "toolchain absent");
    assert_eq!(erased.downcast_ref::<PluginError>(), Some(&typed));
}
