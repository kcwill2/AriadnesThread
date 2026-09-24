//! Build and compilation context discovery.

pub mod discovery;
pub mod types;

pub use discovery::{discover_build_contexts, BuildDiscoverer, ClangCompileCommandEntry};
pub use types::{
    BuildDiscoveryResult, CompilationContext, CompileCommand, TranslationUnit,
};
