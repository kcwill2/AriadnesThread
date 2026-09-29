//! Transitional `#[deprecated]` re-export shim (plan M18.2): the tokenizer,
//! `ParserCtx`, and `CppSemanticAnalyzer` were relocated verbatim to
//! `ariadne-lang-cpp` (with the `ariadne_lang::SemanticAnalyzer` adapter
//! implemented there). This file contains ONLY the re-export so existing
//! call sites keep compiling; it is removed in M18.11.

#[deprecated(note = "moved to ariadne-lang-cpp; shim removed in M18.11")]
pub use ariadne_lang_cpp::analyzer::CppSemanticAnalyzer;
