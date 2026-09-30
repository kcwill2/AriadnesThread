//! C++ language plugin for AriadnesThread (plan
//! `multi-language-support-implementation_plan.md` §3.4 / M18).
//!
//! The `#include` directive machinery was relocated verbatim from
//! `ariadne-graph` (`directive_parser`, `IncludeResolver`), and the in-process
//! semantic analyzer (tokenizer + `ParserCtx` + `CppSemanticAnalyzer`) from
//! `ariadne-index`. Behaviour is byte-identical to the pre-M18 code: the
//! `#[deprecated]` re-export shims in those crates keep existing call sites
//! compiling until M18.11.
//!
//! [`plugin::CppPlugin`] is the registry entry point: it wires the relocated
//! components into `ariadne_lang::LanguagePlugin`.

pub mod analyzer;
pub mod dependency_rewriter;
pub mod directive_parser;
pub mod parser;
pub mod planning;
pub mod plugin;
pub mod resolver;
pub mod source_extractor;
pub mod shaper;
mod tokenizer;

pub use analyzer::CppSemanticAnalyzer;
pub use dependency_rewriter::CppDependencyRewriter;
pub use directive_parser::{parse_include_directives, RawIncludeDirective};
pub use planning::{cpp_aggregation_filter, cpp_target_path_for};
pub use plugin::CppPlugin;
pub use source_extractor::CppSourceExtractor;
pub use resolver::{CppDependencyResolver, IncludeResolver};
pub use shaper::CppUnitShaper;

/// The process-wide C++ unit shaper (plan M18.4). Call sites that relied on
/// `BuildDiscoverer`'s old implicit C++ default pass this shaper so their
/// behaviour stays byte-identical; registry-driven selection (M18.3) is the
/// long-term source.
pub fn cpp_default_shaper() -> &'static CppUnitShaper {
    &CppUnitShaper
}
