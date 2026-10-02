//! Placeholder pipeline components owned by later milestones (M19.1).
//!
//! Capability gating lives in the pipeline, not here: the literals of
//! [`crate::plugin::ScalaPlugin::capabilities`] keep `planning`,
//! `extraction`, and `rewriting` switched off, so calling the
//! extractor/rewriter stubs before their milestones is a capability-gated
//! error before it could ever reach these bodies. M20.1 retired the
//! resolver stub ([`crate::resolver::ScalaImportResolver`]) and M21 closed
//! the analyzer gap ([`crate::analyzer::ScalaAnalyzer`] over the M21
//! parser); both are wired in [`crate::plugin`] and their stubs deleted.
//! M22.1a retired the constraints stub for
//! [`crate::constraints::ScalaClusteringConstraints`]. M23.4 (T4) retired
//! the validator stub for
//! [`crate::validator::ScalaToolchainValidator`] (probe-backed, D-M23-6).
//! M23.5 (T5) retired the extractor stub for
//! [`crate::extractor::ScalaSourceExtractor`] (verbatim regions + package
//! clause + rendered imports, D-M23-1/2). What remains here is owned by
//! M23 T6 (rewriting).

use ariadne_lang::{DependencyRewriter, RewriteSpec};

/// Source passthrough until the M23 Scala rewriter replaces it;
/// `rewriting: false` gates the pipeline away from this body until then.
pub struct StubRewriter;

impl DependencyRewriter for StubRewriter {
    fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
        source.to_string()
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
}
