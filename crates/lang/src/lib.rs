//! Language-agnostic plugin traits for AriadnesThread.
//!
//! Each supported language implements [`LanguagePlugin`] and registers with
//! [`LanguageRegistry`]. The pipeline dispatches through the registry so new
//! languages can be added without touching core pipeline code.

pub mod analyzer;
pub mod constraints;
pub mod detector;
pub mod extractor;
pub mod registry;
pub mod resolver;
pub mod rewriter;
pub mod validator;

pub use analyzer::SemanticAnalyzer;
pub use constraints::ClusteringConstraints;
pub use detector::{LanguageDetector, SourceFileKind};
pub use extractor::{GeneratedSourceFile, SourceExtractor, SymbolBody};
pub use registry::{LanguagePlugin, LanguageRegistry};
pub use resolver::{DependencyResolver, RawDepEdge, ResolutionContext};
pub use rewriter::{DependencyRewriter, RewriteSpec};
pub use validator::CompilerValidator;
