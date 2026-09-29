//! Compiler validation trait for language plugins.

use ariadne_lang_api::capability::ToolchainStatus;
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

    /// Whether the language toolchain is present, so tier-3 validation can
    /// report `Skipped` instead of `Fail` when the compiler is absent.
    fn toolchain_status(&self) -> ToolchainStatus {
        ToolchainStatus::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopValidator;
    impl CompilerValidator for NoopValidator {
        fn validate_compile(
            &self,
            _source_files: &[PathBuf],
            _compiler_args: &[String],
        ) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn default_toolchain_status_is_unknown() {
        assert_eq!(NoopValidator.toolchain_status(), ToolchainStatus::Unknown);
    }
}
