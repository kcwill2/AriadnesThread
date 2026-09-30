//! Consumer dependency rewriting trait for language plugins.

/// Specifies which dependency directives in a consumer file should be replaced.
///
/// The dependency lists carry **bare dependency identifiers** (paths for
/// C/C++ includes, qualified names for Scala imports) — the concrete
/// directive syntax (`#include "..."`, `import ...`) belongs to the
/// language implementation, never to the pipeline that builds the spec.
#[derive(Debug, Clone)]
pub struct RewriteSpec {
    /// The dependency identifiers to remove (e.g. `["models/all.h"]`).
    pub old_deps: Vec<String>,
    /// The replacement dependency identifiers (e.g. `["models/order.h"]`).
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

    /// Render a unified-diff-style fragment listing dependencies removed,
    /// added, and preserved (side effects). The default is a language-neutral
    /// marker rendering; language plugins reproduce their directive text.
    fn render_diff(&self, removed: &[String], added: &[String], preserved: &[String]) -> String {
        let mut out = String::new();
        for dep in removed {
            out.push_str(&format!(" - {dep}\n"));
        }
        for dep in added {
            out.push_str(&format!(" + {dep}\n"));
        }
        for dep in preserved {
            out.push_str(&format!(" ~ {dep}\n"));
        }
        out
    }

    /// Text emitted at the very top of a generated file, followed by one
    /// directive line per `required_deps` (the language decides both). The
    /// default emits nothing.
    fn emit_file_preamble(&self, required_deps: &[String]) -> String {
        let _ = required_deps;
        String::new()
    }
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
