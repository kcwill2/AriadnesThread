//! `ariadne-lang-scala` - the Scala language plugin (plan M19.1/M19.2).
//!
//! Kickoff milestone: file-kind discovery, modeling literals, capability
//! gating, and sbt-layout unit shaping. Every pipeline component beyond
//! those is a documented stub owned by a later milestone (resolver M20,
//! analyzer M21, extractor/rewriter/validator M23). The crate depends
//! inward only (repo -> lang-api -> lang); the m17 architecture guard
//! enforces this edge rule from the moment the crate exists.

pub mod detector;
pub mod plugin;
pub mod shaper;
mod stubs;

pub use detector::ScalaDetector;
pub use plugin::ScalaPlugin;
pub use shaper::ScalaUnitShaper;
