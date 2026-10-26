//! `ariadne-lang-rust` - the Rust language plugin (rust-language R-3).
//!
//! Third language plugin after C++ and Scala: parses real Rust syntax with
//! `syn` (D-2). This hop lands the COMPLETE required trait surface while the
//! crate is UNREGISTERED (registration lands at R-4): the detector, shaper,
//! boundary markers, and modeling literals are real; the analyzer, resolver,
//! extractor, rewriter, and validator are honest never-dispatched stubs
//! behind an ALL-FALSE capability literal (INV-5 in its strongest form -
//! nothing on the pipeline side can be dispatched to an unregistered plugin
//! declaring no capabilities). The real `syn` analyzer lands at R-5 together
//! with the fingerprint bump "rust-1" (INV-8).
//!
//! Honest limit (D-2): macro expansion is out of scope. `include!` and
//! `macro_rules!`-generated `mod` structure is not expanded; `syn` is exact
//! about written code, not about macro output. The limit is carried as the
//! modeling `analysis_caveat`, never silently.
//!
//! The crate depends inward only (repo -> lang-api -> lang); the m17
//! architecture guards enforce this edge rule from the moment the crate
//! exists (A8).

pub mod analyzer;
pub mod boundary;
pub mod constraints;
pub mod detector;
mod dependency_rewriter;
pub mod extractor;
mod mod_tree;
mod move_retention;
mod parser_items;
mod parser_uses;
pub mod plugin;
pub mod resolver;
pub mod shaper;
pub mod validator;

pub use analyzer::RustAnalyzer;
pub use boundary::EXTERNAL_PATH_MARKERS;
pub use constraints::RustClusteringConstraints;
pub use dependency_rewriter::RustDependencyRewriter;
pub use detector::RustDetector;
pub use extractor::RustSourceExtractor;
pub use plugin::RustPlugin;
pub use resolver::RustImportResolver;
pub use shaper::RustUnitShaper;
pub use validator::RustToolchainValidator;
