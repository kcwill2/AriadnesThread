//! [`RustToolchainValidator`] - honest never-dispatched stub
//! (rust-language R-3).
//!
//! `capabilities().validation` is `false` at this hop. `toolchain_status()`
//! answers `Missing` (never `Unknown` - the INV-10 class: while the status is
//! `Missing`, TierValidator reports tier-3 `Skipped` and never reaches
//! `validate_compile`, so the deferral below is never surfaced mid-run).
//! The probe-only validator - mirroring `ScalaToolchainValidator`'s
//! CommandRunner injection seam with probe order `["cargo", "rustc"]`, so
//! the R-9 probe tests need no real spawns - lands at R-9.

use ariadne_lang::{CompilerValidator, PluginError, ToolchainStatus};
use std::path::PathBuf;

/// The message names the deferral and its owning hop (ValidationNotWired
/// contract: the message names the owning milestone).
const VALIDATION_DEFERRAL: &str = "rust compiler validation is not wired: the \
                                   probe-only validator lands at rust-language R-9";

/// Never-dispatched validator stub: reports the toolchain `Missing` and
/// refuses compilation, naming the deferral.
pub struct RustToolchainValidator;

impl CompilerValidator for RustToolchainValidator {
    fn validate_compile(
        &self,
        _source_files: &[PathBuf],
        _compiler_args: &[String],
    ) -> Result<(), PluginError> {
        Err(PluginError::ValidationNotWired {
            message: VALIDATION_DEFERRAL.to_string(),
        })
    }

    fn toolchain_status(&self) -> ToolchainStatus {
        ToolchainStatus::Missing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolchain_status_is_missing_never_unknown() {
        assert_eq!(
            RustToolchainValidator.toolchain_status(),
            ToolchainStatus::Missing,
            "INV-10 class: the trait default Unknown is replaced - TierValidator \
             maps Missing to a Skipped tier, never a spurious Fail"
        );
    }

    #[test]
    fn validate_compile_errors_naming_the_deferral() {
        let result = RustToolchainValidator.validate_compile(&[], &[]);
        let error = result.expect_err("the stub never claims a compile");
        match error {
            PluginError::ValidationNotWired { message } => {
                assert!(
                    message.contains("probe-only validator lands at rust-language R-9"),
                    "the message names the deferral and its hop: {message}"
                );
            }
            other => panic!("not-wired is the honest kind, got: {other:?}"),
        }
    }
}
