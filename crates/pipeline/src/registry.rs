//! The single plugin enumeration site (D9 consolidation), extracted from
//! `lib.rs` by M23 T3 (D-M23-10).
//!
//! This file is the NAMED composition root of the analysis-core crates: it
//! is the one file under the R2 guard's walk scope (`crates/{index,graph,
//! cluster,transform,validate,report,pipeline}/src`) that is allowed to name
//! plugins, because plugin enumeration is its single duty — the same
//! privilege class as `cli/src/registry.rs`, which delegates here (D9
//! direction: cli depends on pipeline, never the reverse). The guard
//! allowlists this exact path; every other core `src` file stays neutral.

use ariadne_lang::LanguageRegistry;
use std::sync::Arc;

/// The single plugin enumeration site (D9 consolidation): C++ first, then
/// Scala, then Rust appended last. Registration ORDER is a contract - it
/// drives `unit_shaper_for()`/`unit_shaper_for_repo()` fallback and
/// `plugin_for()` precedence so existing C++ discovery stays byte-identical
/// (rust-language R-4: C++ first stays first; Rust appends last,
/// INV-1/INV-2). Marker-based shaper auto-selection
/// (the deferred M18.5 gap) landed with plan M22 D22: unfiltered discovery
/// picks the first registered plugin whose build markers sit at the repo root.
/// The CLI delegates here (`ariadne-cli::registry::default_registry`); cli
/// depends on pipeline, never the reverse (D9 direction ruling).
pub fn build_default_registry() -> LanguageRegistry {
    LanguageRegistry::new()
        .register(Box::new(ariadne_lang_cpp::CppPlugin::new()))
        .register(Box::new(ariadne_lang_scala::ScalaPlugin::new()))
        .register(Box::new(ariadne_lang_rust::RustPlugin::new()))
}

/// Shared registry for library-level [`Pipeline::new`] entry points and
/// [`Pipeline::rehash_inputs`] - the [`build_default_registry`] plugin set.
/// C++ behavior stays byte-identical: C++ registers first, and marker-based
/// shaper selection (D22) only diverges from first-wins where a later
/// plugin's markers are present while every earlier plugin's are absent -
/// never true of an existing C++ repository. The Scala plugin's `false`
/// capability literals (plan:381) keep every extraction-side stage on the
/// C++ backend (plan sec 3.1 handler note).
pub fn default_registry() -> Arc<LanguageRegistry> {
    Arc::new(build_default_registry())
}
