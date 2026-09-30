//! Transitional `#[deprecated]` re-export shim (plan M18.2): the tokenizer,
//! `ParserCtx`, and `CppSemanticAnalyzer` were relocated verbatim to
//! `ariadne-lang-cpp` (with the `ariadne_lang::SemanticAnalyzer` adapter
//! implemented there). This file contains ONLY the re-export so existing
//! call sites keep compiling; it is removed in M18.11.

#[deprecated(note = "moved to ariadne-lang-cpp; shim removed in M18.11")]
pub use ariadne_lang_cpp::analyzer::CppSemanticAnalyzer;

/// M18.10: transitional re-export (same TODO) of the single source of truth
/// for the C++ `plugin_fingerprint` cache input, so the legacy shim walk in
/// `scheduler::tu_store` and the registry walk via `CppPlugin` provably feed
/// the identical string without new crate edges.
pub use ariadne_lang_cpp::CPP_ANALYSIS_FINGERPRINT;
