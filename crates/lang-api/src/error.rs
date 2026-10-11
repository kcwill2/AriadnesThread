//! Typed errors at the plugin/extensibility seam (audit finding A7, AR-24).
//!
//! [`PluginError`] replaces the `Result<_, String>` traffic on the plugin
//! trait seam. The contract is twofold:
//!
//! - **Kind**: callers `match` on the variant (a tier-3 validator can
//!   decide `Skipped` vs `Fail` structurally, and AR-25 consumes the
//!   downcast path through `anyhow`), never on rendered text.
//! - **Byte-stability**: each variant's `Display` renders the producing
//!   plugin's message VERBATIM. The wording is owned by the plugin that
//!   produced it -- a neutral API must not hardcode a language's spelling
//!   (the same rule that keeps `Other` out of `LanguageId`) -- and every
//!   CLI-visible string is locked byte-for-byte by the `ar24_*_is_byte_locked`
//!   characterization tests in `lang-cpp`/`lang-scala`.

use thiserror::Error;

/// The error type crossing the plugin seam today: the compiler-validation
/// failures of the `CompilerValidator` trait (`ariadne-lang`).
///
/// The three kinds answer exactly one consumer question: *could the
/// validation run at all?* [`PluginError::ValidationNotWired`] and
/// [`PluginError::ToolchainUnavailable`] say no (report honestly, skip if
/// the capability contract allows), [`PluginError::CompileFailure`] says
/// yes and the compiler said no (a real failure).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PluginError {
    /// The plugin has not implemented compiler validation yet; the message
    /// names the owning milestone. Distinct from a compile failure: nothing
    /// was compiled.
    #[error("{message}")]
    ValidationNotWired { message: String },

    /// The toolchain the validation needs is absent, deliberately deferred,
    /// or became unrunnable; the message is the plugin's honest explanation.
    /// Distinct from [`PluginError::CompileFailure`]: nothing was compiled.
    #[error("{message}")]
    ToolchainUnavailable { message: String },

    /// The compiler ran and exited unsuccessfully; the message carries the
    /// compiler's own output exactly as rendered before the typed seam
    /// (stderr-first wording is the plugin's, not this enum's).
    #[error("{message}")]
    CompileFailure { message: String },
}
