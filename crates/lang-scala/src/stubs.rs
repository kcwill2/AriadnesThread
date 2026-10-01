//! Placeholder pipeline components owned by later milestones (M19.1).
//!
//! Capability gating lives in the pipeline, not here: the literals of
//! [`crate::plugin::ScalaPlugin::capabilities`] keep `planning`,
//! `extraction`, `rewriting`, and `validation` switched off, so calling
//! the extractor/rewriter/validator stubs before M23 is a capability-gated
//! error before it could ever reach these bodies. M20.1 retired the
//! resolver stub ([`crate::resolver::ScalaImportResolver`]) and M21 closed
//! the analyzer gap ([`crate::analyzer::ScalaAnalyzer`] over the M21
//! parser); both are wired in [`crate::plugin`] and their stubs deleted.
//! What remains here is owned by M22 (constraints) and M23 (extraction,
//! rewriting, validation).

use ariadne_lang::{
    ClusteringConstraints, CompilerValidator, DependencyRewriter, GeneratedSourceFile,
    RewriteSpec, SourceExtractor, SymbolBody, SymbolId, SymbolRecord,
};
use std::collections::HashSet;
use std::path::PathBuf;

/// Scala has no atomic symbol groups and no language-level frozen symbols
/// until its clustering milestone; the policy layer owns freezing (plan
/// sec 3.4, same stance as C++).
pub struct StubConstraints;

impl ClusteringConstraints for StubConstraints {
    fn colocated_groups(&self, _symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
        Vec::new()
    }

    fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> HashSet<SymbolId> {
        HashSet::new()
    }
}

/// Empty-source placeholder until the M23 Scala extractor replaces it;
/// `extraction: false` gates the pipeline away from this body until then.
pub struct StubExtractor;

impl SourceExtractor for StubExtractor {
    fn extract(
        &self,
        _bodies: &[SymbolBody],
        _namespace: &str,
        _dependencies: &[String],
    ) -> GeneratedSourceFile {
        GeneratedSourceFile {
            path: PathBuf::from("out.scala"),
            content: String::new(),
        }
    }
}

/// Source passthrough until the M23 Scala rewriter replaces it;
/// `rewriting: false` gates the pipeline away from this body until then.
pub struct StubRewriter;

impl DependencyRewriter for StubRewriter {
    fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
        source.to_string()
    }
}

/// Reports success without compiling until the M23 Scala validator
/// replaces it; `validation: false` gates the pipeline away from this body
/// until then. The trait-default `toolchain_status()` (`Unknown`) is kept:
/// no Scala toolchain probe exists before M23.
pub struct StubValidator;

impl CompilerValidator for StubValidator {
    fn validate_compile(&self, _files: &[PathBuf], _args: &[String]) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_rewriter_passes_source_through() {
        let source = "object Main extends App {}";
        let spec = RewriteSpec {
            old_deps: vec!["com.example.orders".to_string()],
            new_deps: vec!["com.example.orders.model".to_string()],
        };
        assert_eq!(StubRewriter.rewrite(source, &spec), source);
    }

    #[test]
    fn stub_validator_reports_ok_until_m23() {
        assert!(StubValidator
            .validate_compile(&[], &[])
            .is_ok());
    }
}
