//! `ariadne-lang-scala` - the Scala language plugin (plan M19.1/M19.2).
//!
//! Kickoff milestone: file-kind discovery, modeling literals, capability
//! gating, and sbt-layout unit shaping; M20.1 added the import/export
//! dependency resolver. Every remaining pipeline component beyond those is
//! a documented stub owned by a later milestone (analyzer M21,
//! extractor/rewriter/validator M23). The crate depends
//! inward only (repo -> lang-api -> lang); the m17 architecture guard
//! enforces this edge rule from the moment the crate exists.

pub mod analyzer;
pub mod boundary;
pub mod detector;
mod directive_parser;
mod scanner;
pub mod parser;
pub mod plugin;
pub mod resolver;
pub mod shaper;
mod stubs;

pub use analyzer::ScalaAnalyzer;
pub use boundary::EXTERNAL_PATH_MARKERS;
pub use detector::ScalaDetector;
pub use parser::ScalaSourceParser;
pub use plugin::ScalaPlugin;
pub use resolver::ScalaImportResolver;
pub use shaper::ScalaUnitShaper;
