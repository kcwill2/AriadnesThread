//! Compiler validation trait for language plugins.

use std::path::PathBuf;

/// Invokes the real language compiler to verify that a set of source files
/// compiles without errors.
///
/// Used by the tier-3 validation step in [`TransactionApplier`]. If this
/// returns `Err`, the transaction is rolled back automatically.
pub trait CompilerValidator: Send + Sync {
    /// Attempt to compile `source_files` using `compiler_args`.
    ///
    /// Returns `Ok(())` on success or `Err(message)` on failure, where
    /// `message` contains the compiler's error output.
    fn validate_compile(
        &self,
        source_files: &[PathBuf],
        compiler_args: &[String],
    ) -> Result<(), String>;
}
