//! [`CppDependencyRewriter`] — byte-identical C++ directive text (plan §3.6,
//! M18.8b). Reproduces exactly what `TransactionApplier::apply` and
//! `ConsumerRewrite::format_diff` did inline before M18.8b: per old
//! dependency, both the quoted and the angled directive spelling are replaced
//! (all occurrences) with the newline-joined quoted directives of the new
//! dependencies; diff fragments use the 4-space `+`/`-`/`~` include lines
//! (preserved entries carry the side-effect comment); generated files start
//! with the `#pragma once` guard block. Directive spellings are therefore
//! owned here and nowhere in the transform/validate crates.

use ariadne_lang::{DependencyRewriter, RewriteSpec};

/// Rewrites C/C++ dependency directives in consumer sources.
pub struct CppDependencyRewriter;

fn quoted_directive(dep: &str) -> String {
    format!("#include \"{}\"", dep)
}

fn angled_directive(dep: &str) -> String {
    format!("#include <{}>", dep)
}

impl DependencyRewriter for CppDependencyRewriter {
    fn rewrite(&self, source: &str, spec: &RewriteSpec) -> String {
        // Byte-for-byte the pre-M18.8b `TransactionApplier::apply` mutation:
        // per old dependency, the quoted and the angled spelling are each
        // replace-all'ed with the quoted directive of every new dependency
        // joined by newlines (an empty new list deletes the directive text).
        let replacement: Vec<String> = spec.new_deps.iter().map(|d| quoted_directive(d)).collect();
        let replacement = replacement.join("\n");
        let mut content = source.to_string();
        for old_dep in &spec.old_deps {
            content = content.replace(&quoted_directive(old_dep), &replacement);
            content = content.replace(&angled_directive(old_dep), &replacement);
        }
        content
    }

    fn render_diff(&self, removed: &[String], added: &[String], preserved: &[String]) -> String {
        // Byte-for-byte the pre-M18.8b `ConsumerRewrite::format_diff` lines.
        let mut out = String::new();
        for dep in removed {
            out.push_str(&format!("    - {}\n", quoted_directive(dep)));
        }
        for dep in added {
            out.push_str(&format!("    + {}\n", quoted_directive(dep)));
        }
        for dep in preserved {
            out.push_str(&format!(
                "    ~ {} // side-effect preserved\n",
                quoted_directive(dep)
            ));
        }
        out
    }

    fn emit_file_preamble(&self, required_deps: &[String]) -> String {
        // Same bytes as `CppSourceExtractor::extract` with no symbol bodies
        // (guard, quoted include lines, blank separator iff any include).
        ariadne_lang::SourceExtractor::extract(
            &crate::source_extractor::CppSourceExtractor,
            &[],
            "",
            required_deps,
        )
        .content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(old: &[&str], new: &[&str]) -> RewriteSpec {
        RewriteSpec {
            old_deps: old.iter().map(|s| s.to_string()).collect(),
            new_deps: new.iter().map(|s| s.to_string()).collect(),
        }
    }

    // Baselines below were captured from `crates/validate/src/lib.rs`
    // TransactionApplier::apply (consumer mutation loop) and
    // `crates/transform/src/placement.rs` ConsumerRewrite::format_diff
    // BEFORE M18.8b moved the code behind this rewriter.

    #[test]
    fn rewrite_replaces_quoted_directive_with_new_block_byte_exact() {
        let source = "#include \"thing.h\"\nint main() { return 0; }\n";
        let out =
            CppDependencyRewriter.rewrite(source, &spec(&["thing.h"], &["thing_types.h", "thing_core.h"]));
        assert_eq!(
            out,
            "#include \"thing_types.h\"\n#include \"thing_core.h\"\nint main() { return 0; }\n"
        );
    }

    #[test]
    fn rewrite_replaces_every_occurrence_and_the_angled_spelling() {
        // Today's apply replaces ALL occurrences and derives the angled
        // spelling from the bare dependency: both get the same new block.
        let source = "#include \"thing.h\"\nclass A {};\n#include <thing.h>\n";
        let out = CppDependencyRewriter.rewrite(source, &spec(&["thing.h"], &["thing_core.h"]));
        assert_eq!(out, "#include \"thing_core.h\"\nclass A {};\n#include \"thing_core.h\"\n");
    }

    #[test]
    fn rewrite_without_new_deps_removes_directive_text() {
        let source = "#include \"thing.h\"\nint x;\n";
        let out = CppDependencyRewriter.rewrite(source, &spec(&["thing.h"], &[]));
        assert_eq!(out, "\nint x;\n");
    }

    #[test]
    fn rewrite_preserves_all_unrelated_bytes() {
        let source = "// keep me\nnot a directive: thing.h\n\n\n";
        let out = CppDependencyRewriter.rewrite(source, &spec(&["thing.h"], &["new.h"]));
        assert_eq!(out, source);
    }

    #[test]
    fn rewrite_replaces_each_old_dep_in_order() {
        let source = "#include \"a.h\"\n#include \"b.h\"\n";
        let out = CppDependencyRewriter.rewrite(source, &spec(&["a.h", "b.h"], &["all.h"]));
        assert_eq!(out, "#include \"all.h\"\n#include \"all.h\"\n");
    }

    #[test]
    fn render_diff_reproduces_placement_format_diff_bytes() {
        // Baseline: placement.rs emitted per old `    - #include "{old}"\n`,
        // per new `    + #include "{new}"\n`, per preserved
        // `    ~ #include "{p}" // side-effect preserved\n` (4-space indent).
        let out = CppDependencyRewriter.render_diff(
            &["thing.h".to_string()],
            &["thing_types.h".to_string()],
            &["registry.h".to_string()],
        );
        let expected = "    - #include \"thing.h\"\n    + #include \"thing_types.h\"\n    ~ #include \"registry.h\" // side-effect preserved\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn render_diff_with_no_entries_is_empty() {
        assert_eq!(CppDependencyRewriter.render_diff(&[], &[], &[]), "");
    }

    #[test]
    fn emit_file_preamble_reproduces_apply_guard_block_bytes() {
        // Baseline: apply emitted `#pragma once\n\n`, then one quoted
        // include line per required dep, then a blank line iff any dep.
        assert_eq!(CppDependencyRewriter.emit_file_preamble(&[]), "#pragma once\n\n");
        assert_eq!(
            CppDependencyRewriter.emit_file_preamble(&["a.h".to_string(), "b.h".to_string()]),
            "#pragma once\n\n#include \"a.h\"\n#include \"b.h\"\n\n"
        );
    }

    // D-M23-7 iii (T7): consumer discovery baselines captured from the CLI
    // substring scans in `commands/transform.rs::find_plan_consumers` and
    // `optimize.rs::find_including_translation_units` — `content.contains(
    // "#include \"<target>\"") || content.contains("#include <<target>>")`.
    // FIDELITY NOTE (for G3): the C++ hits reproduce that RAW substring
    // predicate line-by-line, INCLUDING occurrences inside comments — unlike
    // the lexical Scala scan, byte-identical consumer SETS (I1) outrank
    // lexical purity here; the neutral trait default stays empty.

    fn hits(source: &str, target: &str) -> Vec<ariadne_lang::DirectiveHit> {
        CppDependencyRewriter.find_dependency_targets(source, target)
    }

    #[test]
    fn find_dependency_targets_finds_quoted_and_angled_spellings_with_lines() {
        let source = "#include <sys.h>\nint x;\n#include \"thing.h\"\n";
        let found: Vec<(usize, usize, String)> = hits(source, "thing.h")
            .into_iter()
            .map(|h| (h.start_line, h.end_line, h.raw_directive))
            .collect();
        assert_eq!(
            found,
            vec![(3, 3, "#include \"thing.h\"".to_string())],
            "quoted spelling hits with 1-based inclusive line and verbatim line"
        );
        let angled: Vec<usize> = hits("#include <thing.h>\n", "thing.h")
            .into_iter()
            .map(|h| h.start_line)
            .collect();
        assert_eq!(angled, vec![1], "angled spelling is a hit too");
    }

    #[test]
    fn find_dependency_targets_requires_the_exact_substring_spelling() {
        assert!(hits("#include \"sub/thing.h\"\n", "thing.h").is_empty());
        assert!(hits("#include  \"thing.h\"\n", "thing.h").is_empty());
        assert!(hits("thing.h\n", "thing.h").is_empty());
        assert!(hits("#include \"other.h\"\n", "thing.h").is_empty());
        assert!(
            hits("// #include \"thing.h\"\n", "thing.h").len() == 1,
            "LOCK (substring fidelity): the pre-T7 CLI scan matched this \
             too — reproducing it byte-identically keeps consumer sets equal"
        );
    }

    #[test]
    fn find_dependency_targets_reports_every_matching_line_and_survives_crlf() {
        let source = "#include \"thing.h\"\nbody\n#include <thing.h>\n";
        assert_eq!(hits(source, "thing.h").len(), 2);
        let crlf = "#include \"thing.h\"\r\nint y;\r\n";
        let found = hits(crlf, "thing.h");
        assert_eq!(found.len(), 1, "CRLF line endings still hit");
        assert_eq!(found[0].raw_directive, "#include \"thing.h\"");
    }

    #[test]
    fn find_dependency_targets_no_match_is_empty() {
        assert!(hits("int main() {}\n", "thing.h").is_empty());
        assert!(hits("", "thing.h").is_empty());
    }
}
