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
pub mod unit;
pub mod validator;

// Neutral vocabulary now lives in `ariadne-lang-api`; these targeted
// re-exports (no glob: they would collide with this crate's own trait
// names) keep `ariadne_lang::SymbolId`, `ariadne_lang::TuAnalysis`, ...
// resolving for existing users after the `lang -> index` edge was removed.
pub use ariadne_lang_api::capability::{LanguageCapabilities, ToolchainStatus};
pub use ariadne_lang_api::catalog::{normalize_relative_path, FileCatalog, ResolveOutcome};
pub use ariadne_lang_api::identity::{language_of_file_kind, LanguageId};
pub use ariadne_lang_api::modeling::{
    AggregationKind, CyclePolicy, DependencyVocabulary, GuardKind, LanguageModeling,
    SymbolPlacement, UnitKind,
};
pub use ariadne_lang_api::types::{
    CallSite, DeclId, DependencyKind, FileLocation, SemanticEdge, SymbolDecl, SymbolId, SymbolKind,
    SymbolRecord, TuAnalysis,
};
pub use ariadne_lang_api::unit::UnitDescriptor;

pub use analyzer::SemanticAnalyzer;
pub use constraints::ClusteringConstraints;
pub use detector::{LanguageDetector, SourceFileKind};
pub use extractor::{render_unit, GeneratedSourceFile, SourceExtractor, SymbolBody};
pub use registry::{LanguagePlugin, LanguageRegistry};
pub use resolver::{
    first_ondisk_include_candidate, DependencyResolver, DirectiveStyle, RawDepEdge,
    ResolutionContext,
};
pub use rewriter::{DependencyRewriter, DirectiveHit, RewriteSpec};
pub use unit::UnitShaper;
pub use validator::CompilerValidator;
