//! C++ language plugin for AriadnesThread (plan
//! `multi-language-support-implementation_plan.md` §3.4 / M18).
//!
//! The `#include` directive machinery was relocated verbatim from
//! `ariadne-graph` (`directive_parser`, `IncludeResolver`), the in-process
//! semantic analyzer (tokenizer + `ParserCtx` + `CppSemanticAnalyzer`) from
//! `ariadne-index`, and the clang-worker sidecar backend
//! ([`backend::ClangWorkerBackend`]) from `ariadne-scheduler` — all
//! byte-identical to the pre-M18 code, permanently home here since the M18.11
//! shim removal (the `#[deprecated]` re-export shims in those crates are
//! gone; the permanent `m18_cpp_plugin_parity_test` locks the bytes).
//!
//! [`plugin::CppPlugin`] is the registry entry point: it wires the relocated
//! components into `ariadne_lang::LanguagePlugin`.

pub mod analyzer;
pub mod backend;
pub mod dependency_rewriter;
pub mod directive_parser;
pub mod parser;
pub mod planning;
pub mod plugin;
pub mod resolver;
pub mod source_extractor;
pub mod shaper;
mod move_retention;
mod symbol_locator;
mod tokenizer;

pub use analyzer::CppSemanticAnalyzer;
pub use backend::ClangWorkerBackend;
pub use dependency_rewriter::CppDependencyRewriter;
pub use directive_parser::{parse_include_directives, RawIncludeDirective};
pub use planning::{cpp_aggregation_filter, cpp_target_path_for};
pub use plugin::{CppPlugin, CPP_ANALYSIS_FINGERPRINT};
pub use source_extractor::CppSourceExtractor;
pub use resolver::CppDependencyResolver;
// AR-21: the deprecated legacy `IncludeResolver` is an oracle-only surface
// (module docs in `resolver`): compiled for lang-cpp's own tests or under
// the test-only `test-resolver` feature (dev-consumer wiring in
// tests/integration/Cargo.toml); production builds compile it out.
#[cfg(any(test, feature = "test-resolver"))]
#[allow(deprecated)]
pub use resolver::IncludeResolver;
pub use shaper::CppUnitShaper;

/// The process-wide C++ unit shaper (plan M18.4). Call sites that relied on
/// `BuildDiscoverer`'s old implicit C++ default pass this shaper so their
/// behaviour stays byte-identical; registry-driven selection (M18.3) is the
/// long-term source.
pub fn cpp_default_shaper() -> &'static CppUnitShaper {
    &CppUnitShaper
}

#[cfg(test)]
mod extension_case_matrix_tests;
