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

/// One dependency directive located in a source file (D-M23-7 iii, M23 T6).
///
/// Consumer-discovery call sites (CLI `transform`/`optimize`) previously ran
/// a raw `#include "<target>"` substring scan and only used the hit as a
/// yes/no; this record carries what a language plugin can honestly report:
/// - `start_line` / `end_line`: **1-based, inclusive** line numbers of the
///   directive statement in the source file (one line for single-line
///   directives; a multi-line grouped directive spans its whole text),
/// - `raw_directive`: the directive statement **verbatim** as it appears in
///   the source, excluding any trailing newline (language plugins that
///   cannot recover the exact bytes must not report a hit at all).
///
/// Hits must never come from inside comments or string literals; a plugin
/// that scans lexically guarantees this (Scala: the M21 scanner; C++: the
/// tokenizer scan reproducing the old substring semantics, T7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectiveHit {
    pub start_line: usize,
    pub end_line: usize,
    pub raw_directive: String,
}

/// Rewrites dependency directives in a consumer source file.
///
/// Given the existing source text and a [`RewriteSpec`], returns the new
/// source text with old directives replaced by new ones. The implementation
/// must preserve all other content exactly.
pub trait DependencyRewriter: Send + Sync {
    /// Apply `spec` to `source` and return the rewritten source text.
    fn rewrite(&self, source: &str, spec: &RewriteSpec) -> String;

    /// Locate dependency directives in `source` that mention the extraction
    /// `target` (D-M23-7 iii). The default is **empty** — deliberately NOT
    /// the old C++ substring scan, so a language without an implementation
    /// reports no consumers rather than false positives; `lang-cpp`
    /// reproduces its substring scan explicitly in T7, and Scala scans its
    /// import directives (M23 T6).
    fn find_dependency_targets(&self, _source: &str, _target: &str) -> Vec<DirectiveHit> {
        Vec::new()
    }

    /// The dependency-space spellings a consumer file is probed with (via
    /// [`Self::find_dependency_targets`]) when discovering consumers of an
    /// extraction plan (plan M23, T8b fence lift). The default reports the
    /// target file's bare name — today's C++ `#include "<file>"` scan
    /// spelling, byte-identical (I1). Move languages override with the moved
    /// units' dependency identifiers (module paths for Scala imports);
    /// `moved_module_ids` are the plan's fully-qualified symbol ids, neutral
    /// plan data the CLI passes through untouched.
    fn consumer_discovery_targets(
        &self,
        target_file: &std::path::Path,
        _moved_module_ids: &[String],
    ) -> Vec<String> {
        target_file
            .file_name()
            .map(|name| vec![name.to_string_lossy().into_owned()])
            .unwrap_or_default()
    }

    /// Build the consumer's [`RewriteSpec`] from a rewrite's file-path
    /// payload. Consumer rewrites are computed path-neutral (old/new include
    /// paths), but `RewriteSpec` carries **bare dependency identifiers** —
    /// equal to the paths where a language's directive text *is* the path
    /// (C++), and not where it isn't (Scala imports name module paths, never
    /// files). The default is `None`: the caller keeps the path strings —
    /// today's C++ behavior byte-identical (I1). `moved_module_ids` holds the
    /// plan's fully-qualified ids of the units this rewrite points at.
    fn consumer_rewrite_spec(
        &self,
        _old_includes: &[std::path::PathBuf],
        _new_includes: &[std::path::PathBuf],
        _moved_module_ids: &[String],
    ) -> Option<RewriteSpec> {
        None
    }

    /// Delete the named top-level symbols from `source` (D-M23-5 move
    /// semantics), or `None` when any name is not safely removable - the
    /// caller then proceeds **without** removal. The default is still
    /// `None`: the engine never removes for languages that do not opt in.
    /// Since `extraction-symbol-move` the retention decision lives one
    /// level up, in [`Self::move_out_top_level_symbols`], whose default
    /// delegates here (deletion), so Scala keeps its deletion behavior
    /// through this hook and C++ chooses marked comment-out.
    fn remove_top_level_symbols(&self, _source: &str, _qualified_names: &[String]) -> Option<String> {
        None
    }

    /// The post-move source-text contract: return the text that REPLACES
    /// `source` once the named top-level symbols have moved into their
    /// extracted units, or `None` when any name cannot be safely retired -
    /// the caller then proceeds WITHOUT the move (copy fallback). The
    /// RETENTION FORM is the language's choice: Scala deletes via the
    /// inherited parser spans (through the default below); C++ comments
    /// regions out under `ARIADNE-EXTRACT-OUT` markers bound to `plan_id`
    /// (extraction-symbol-move, the user's reversible-move decision of
    /// record). `plan_id` is marker provenance only - a language whose
    /// retention needs no provenance ignores it. The default delegates to
    /// `remove_top_level_symbols` (deletion), so every existing
    /// implementation keeps byte-identical behavior.
    fn move_out_top_level_symbols(
        &self,
        source: &str,
        qualified_names: &[String],
        _plan_id: &str,
    ) -> Option<String> {
        self.remove_top_level_symbols(source, qualified_names)
    }

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

    struct InertRewriter;

    impl DependencyRewriter for InertRewriter {
        fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
            source.to_string()
        }
    }

    #[test]
    fn find_dependency_targets_default_is_empty_not_a_substring_scan() {
        let hits = InertRewriter.find_dependency_targets("#include \"a.h\"", "a.h");
        assert!(
            hits.is_empty(),
            "D-M23-7 iii: the neutral default must be empty — C++ opts into its \
             substring scan explicitly in T7, no language inherits it silently"
        );
    }

    #[test]
    fn remove_top_level_symbols_default_is_none_copy_semantics_preserved() {
        assert_eq!(
            InertRewriter.remove_top_level_symbols("int x;", &["x".to_string()]),
            None,
            "I5: default None — the engine never removes for copy-only languages (C++)."
        );
    }

    #[test]
    fn directive_hit_carries_line_range_and_verbatim_text() {
        let hit = DirectiveHit {
            start_line: 3,
            end_line: 3,
            raw_directive: "import a.b.C".to_string(),
        };
        assert_eq!(hit.start_line, hit.end_line);
        assert_eq!(hit.raw_directive, "import a.b.C");
    }

    #[test]
    fn pin_neutral_consumer_discovery_targets_default_is_the_bare_file_name() {
        assert_eq!(
            InertRewriter.consumer_discovery_targets(
                std::path::Path::new("a/b.h"),
                &[],
            ),
            vec!["b.h".to_string()],
            "I1: the neutral default probes with the target file's BARE name - \
             today's C++ `#include \"<file>\"` scan spelling, byte-identical"
        );
        assert!(
            InertRewriter.consumer_discovery_targets(std::path::Path::new("."), &[]).is_empty(),
            "a path without a file name (`.`) yields NO probe strings rather than \
             a bogus directory-name probe"
        );
    }

    #[test]
    fn pin_neutral_consumer_rewrite_spec_default_is_none() {
        assert!(
            InertRewriter.consumer_rewrite_spec(&[], &[], &[]).is_none(),
            "the path payload keeps its path strings when the language cannot map \
             them to directive identifiers (C++ behavior byte-identical)"
        );
    }

    #[test]
    fn pin_neutral_render_diff_default_uses_bare_marker_grammar() {
        assert_eq!(
            InertRewriter.render_diff(
                &["old/x".to_string()],
                &["new/x".to_string()],
                &["kept/x".to_string()],
            ),
            " - old/x\n + new/x\n ~ kept/x\n",
            "the language-neutral default is marker + dependency ONLY: no `#include` \
             or `import` spelling leaks into the neutral trait (plan §7 R2)"
        );
    }

    /// Fake implementing the OLD hook with a visible byte transform, so the
    /// new default's delegation is observable, not just type-checked.
    struct DeletingRewriter;

    impl DependencyRewriter for DeletingRewriter {
        fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
            source.to_string()
        }

        fn remove_top_level_symbols(
            &self,
            source: &str,
            _qualified_names: &[String],
        ) -> Option<String> {
            Some(source.replace("class Drop\n", ""))
        }
    }

    #[test]
    fn default_move_out_delegates_byte_identically_to_remove_hook() {
        let source = "class Keep {}\nclass Drop\n{\n};\n";
        let names = ["Drop".to_string()];
        let moved = DeletingRewriter.move_out_top_level_symbols(source, &names, "PLAN-TEST");
        let deleted = DeletingRewriter.remove_top_level_symbols(source, &names);
        assert!(
            matches!((&moved, &deleted), (Some(_), Some(_))),
            "SM-I5: the default IS the old hook - every existing implementation \
             keeps byte-identical behavior"
        );
        assert_eq!(
            moved, deleted,
            "SM-I5: the default IS the old hook - every existing implementation \
             keeps byte-identical behavior"
        );
    }

    #[test]
    fn default_move_out_refuses_exactly_where_the_remove_hook_refuses() {
        assert_eq!(
            InertRewriter.move_out_top_level_symbols("int x;", &["x".to_string()], "PLAN-TEST"),
            None,
            "SM-I5: the default refuses exactly where the old hook refuses - the \
             caller then proceeds WITHOUT the move (copy fallback)"
        );
    }
}
