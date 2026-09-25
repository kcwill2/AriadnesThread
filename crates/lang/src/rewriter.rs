//! Consumer dependency rewriting trait for language plugins.

/// Specifies which dependency directives in a consumer file should be replaced.
///
/// For C++ these are `#include` lines; for Scala they are `import` statements.
#[derive(Debug, Clone)]
pub struct RewriteSpec {
    /// The dependency strings to remove (e.g. `["#include \"models/all.h\""]`).
    pub old_deps: Vec<String>,
    /// The replacement dependency strings (e.g. `["#include \"models/order.h\""]`).
    pub new_deps: Vec<String>,
}

/// Rewrites dependency directives in a consumer source file.
///
/// Given the existing source text and a [`RewriteSpec`], returns the new
/// source text with old directives replaced by new ones. The implementation
/// must preserve all other content exactly.
pub trait DependencyRewriter: Send + Sync {
    /// Apply `spec` to `source` and return the rewritten source text.
    fn rewrite(&self, source: &str, spec: &RewriteSpec) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_spec_fields() {
        let spec = RewriteSpec {
            old_deps: vec!["#include \"all.h\"".into()],
            new_deps: vec!["#include \"order.h\"".into(), "#include \"user.h\"".into()],
        };
        assert_eq!(spec.old_deps.len(), 1);
        assert_eq!(spec.new_deps.len(), 2);
    }
}
