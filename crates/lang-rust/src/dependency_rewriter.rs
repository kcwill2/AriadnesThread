//! [`RustDependencyRewriter`] - honest never-dispatched stub
//! (rust-language R-3).
//!
//! `capabilities().rewriting` is `false` at this hop, so the engine never
//! dispatches here. `rewrite` is the identity - honest passthrough, mirroring
//! the neutral defaults of the rest of the surface. The `use`-path rewrite
//! (the consumer-rewrite hook over `use x.y::Z;` spellings) plus the
//! mod-tree mechanism (RISK-MODTREE: keeping an extracted file declared in
//! its parent) land together at R-8; every other hook stays at its
//! trait default (reports nothing rather than false positives).

use ariadne_lang::{DependencyRewriter, RewriteSpec};

/// Never-dispatched rewriter stub: returns `source` unchanged.
pub struct RustDependencyRewriter;

impl DependencyRewriter for RustDependencyRewriter {
    fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
        source.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_is_the_honest_identity() {
        let source = "use serde::Serialize;\n\npub fn greet() {}\n";
        let spec = RewriteSpec {
            old_deps: vec!["crate::old".to_string()],
            new_deps: vec!["crate::new".to_string()],
        };
        assert_eq!(
            RustDependencyRewriter.rewrite(source, &spec),
            source,
            "rewriting is false, so this is never dispatched; the identity is \
             honest - the use-path rewrite + mod-tree mechanism lands at R-8"
        );
    }
}
