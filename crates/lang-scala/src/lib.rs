//! `ariadne-lang-scala` - the Scala language plugin (plan M19.1/M19.2).
//!
//! Kickoff milestone: file-kind discovery, modeling literals, capability
//! gating, and sbt-layout unit shaping; M20.1 added the import/export
//! dependency resolver. Every pipeline component is landed by milestone
//! (analyzer M21, probe validator M23.4, extractor M23.5/T5, rewriter
//! M23.6/T6); the kickoff stubs are deleted (D-M23-6), the capability
//! flip itself is T7. The crate depends
//! inward only (repo -> lang-api -> lang); the m17 architecture guard
//! enforces this edge rule from the moment the crate exists.

pub mod analyzer;
pub mod boundary;
pub mod constraints;
pub mod detector;
mod dependency_rewriter;
mod directive_parser;
pub mod extractor;
mod imports;
mod locator;
mod scanner;
pub mod parser;
pub mod planning;
pub mod plugin;
pub mod resolver;
pub mod shaper;
pub mod validator;

pub use analyzer::ScalaAnalyzer;
pub use boundary::EXTERNAL_PATH_MARKERS;
pub use dependency_rewriter::ScalaDependencyRewriter;
pub use constraints::ScalaClusteringConstraints;
pub use detector::ScalaDetector;
pub use extractor::ScalaSourceExtractor;
pub use imports::referenced_imports;
pub use parser::ScalaSourceParser;
pub use planning::{scala_aggregation_filter, scala_declares_package_object, scala_target_path_for};
pub use plugin::ScalaPlugin;
pub use resolver::ScalaImportResolver;
pub use shaper::ScalaUnitShaper;
pub use validator::{
    CommandOutcome, CommandRunner, ProcessCommandRunner, ScalaToolchainValidator,
    TOOLCHAIN_PROBE_ORDER,
};
