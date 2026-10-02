//! `ariadne-lang-scala` - the Scala language plugin (plan M19.1/M19.2).
//!
//! Kickoff milestone: file-kind discovery, modeling literals, capability
//! gating, and sbt-layout unit shaping; M20.1 added the import/export
//! dependency resolver. Remaining pipeline components are documented stubs
//! or landed components by milestone (analyzer M21, probe validator M23.4,
//! extractor/rewriter M23 T5/T6). The crate depends
//! inward only (repo -> lang-api -> lang); the m17 architecture guard
//! enforces this edge rule from the moment the crate exists.

pub mod analyzer;
pub mod boundary;
pub mod constraints;
pub mod detector;
mod directive_parser;
pub mod extractor;
mod locator;
mod scanner;
pub mod parser;
pub mod planning;
pub mod plugin;
pub mod resolver;
pub mod shaper;
mod stubs;
pub mod validator;

pub use analyzer::ScalaAnalyzer;
pub use boundary::EXTERNAL_PATH_MARKERS;
pub use constraints::ScalaClusteringConstraints;
pub use detector::ScalaDetector;
pub use extractor::ScalaSourceExtractor;
pub use parser::ScalaSourceParser;
pub use planning::{scala_aggregation_filter, scala_declares_package_object, scala_target_path_for};
pub use plugin::ScalaPlugin;
pub use resolver::ScalaImportResolver;
pub use shaper::ScalaUnitShaper;
pub use validator::{
    CommandOutcome, CommandRunner, ProcessCommandRunner, ScalaToolchainValidator,
    TOOLCHAIN_PROBE_ORDER,
};
