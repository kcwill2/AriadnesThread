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

    /// Reproduces the pre-T7 CLI consumer scan (`content.contains(
    /// "#include \"<target>\"") || content.contains("#include <<target>>")`)
    /// line-by-line, D-M23-7 iii. A pattern spanning no newline, so
    /// per-line containment is set-equal to the old whole-content
    /// containment; `raw_directive` is the matched line verbatim without
    /// its line terminator. Deliberately NOT lexical: the old scan matched
    /// comment-embedded directives too, and byte-identical consumer sets
    /// (I1) outrank lexical purity for C++ (the lexical Scala scan opts
    /// into comment/string invisibility instead).
    fn find_dependency_targets(&self, source: &str, target: &str) -> Vec<ariadne_lang::DirectiveHit> {
        let quoted = quoted_directive(target);
        let angled = angled_directive(target);
        source
            .split('\n')
            .enumerate()
            .filter_map(|(index, line)| {
                let text = line.strip_suffix('\r').unwrap_or(line);
                (text.contains(&quoted) || text.contains(&angled)).then(|| ariadne_lang::DirectiveHit {
                    start_line: index + 1,
                    end_line: index + 1,
                    raw_directive: text.to_string(),
                })
            })
            .collect()
    }

    /// Map the path-space rewrite payload onto directive identifiers with the
    /// SAME bare-file-name rule `consumer_discovery_targets` probes with
    /// (the trait default, `crates/lang/src/rewriter.rs:66-75`), so
    /// discovery's predicate and the rewrite's matcher agree by construction
    /// (extraction-consumer-rewrite CW-1). Always `Some`: C++ always has
    /// directive grammar. `moved_module_ids` is unused - the C++ directive
    /// space names files, not modules.
    fn consumer_rewrite_spec(
        &self,
        old_includes: &[std::path::PathBuf],
        new_includes: &[std::path::PathBuf],
        _moved_module_ids: &[String],
    ) -> Option<ariadne_lang::RewriteSpec> {
        Some(RewriteSpec {
            old_deps: old_includes.iter().map(|path| dependency_spelling(path)).collect(),
            new_deps: new_includes.iter().map(|path| dependency_spelling(path)).collect(),
        })
    }

    /// Marked comment-out retention is C++'s chosen form of the
    /// extraction-symbol-move decision of record: the located declaration
    /// span stays in the file commented out under `ARIADNE-EXTRACT-OUT`
    /// line-prefix markers bound to `plan_id` (provenance), never block
    /// comments. Refusals answer `None` so the caller proceeds without the
    /// move (copy fallback). Grammar and refusal contract:
    /// `crate::move_retention`.
    fn move_out_top_level_symbols(
        &self,
        source: &str,
        qualified_names: &[String],
        plan_id: &str,
    ) -> Option<String> {
        crate::move_retention::move_out_text(source, qualified_names, plan_id)
    }
}

/// Path -> bare directive spelling: the file name when the path has one,
/// else its display string - the honest fallback for a path without a file
/// name, mirroring the discovery default's refusal to invent a probe and
/// using the same lossy-ASCII channel as today's fallback (X7).
fn dependency_spelling(path: &std::path::Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
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

    // CW-1 (extraction-consumer-rewrite): the consumer_rewrite_spec hook
    // maps the path-space rewrite payload onto directive spellings with the
    // SAME bare-file-name rule consumer_discovery_targets probes with.

    fn hook_spec(old_paths: &[&str], new_paths: &[&str]) -> RewriteSpec {
        let old_includes: Vec<std::path::PathBuf> =
            old_paths.iter().map(std::path::PathBuf::from).collect();
        let new_includes: Vec<std::path::PathBuf> =
            new_paths.iter().map(std::path::PathBuf::from).collect();
        CppDependencyRewriter
            .consumer_rewrite_spec(&old_includes, &new_includes, &[])
            .expect("C++ always has directive grammar")
    }

    #[test]
    fn consumer_rewrite_spec_maps_bare_name_paths_identity() {
        let built = hook_spec(&["old.h"], &["new.h"]);
        assert_eq!(
            built.old_deps,
            vec!["old.h".to_string()],
            "a bare path is already its directive spelling - exactly why the byte-parity \
             test crates/validate/tests/transaction_apply_test.rs:48 (bare `old.h` payload) \
             survives UNEDITED: the hook spec equals the path fallback there"
        );
        assert_eq!(built.new_deps, vec!["new.h".to_string()]);
    }

    #[test]
    fn consumer_rewrite_spec_maps_paths_to_bare_file_names() {
        let built = hook_spec(
            &["include/thing.h"],
            &["generated/include/Thing_extracted.h", "include/ThingConfig_extracted.h"],
        );
        assert_eq!(built.old_deps, vec!["thing.h".to_string()]);
        assert_eq!(
            built.new_deps,
            vec!["Thing_extracted.h".to_string(), "ThingConfig_extracted.h".to_string()],
            "order preserved, no sorting"
        );
    }

    #[test]
    fn consumer_rewrite_spec_fallback_keeps_display_string_without_file_name() {
        let pathless_old = std::path::Path::new(".").to_path_buf();
        let pathless_new = std::path::Path::new("..").to_path_buf();
        let built = CppDependencyRewriter
            .consumer_rewrite_spec(&[pathless_old], &[pathless_new], &[])
            .expect("C++ always has directive grammar");
        assert_eq!(
            built.old_deps,
            vec![".".to_string()],
            "a path without a file name keeps its display string - never a fabricated probe"
        );
        assert_eq!(built.new_deps, vec!["..".to_string()]);
    }

    #[test]
    fn discovery_hit_iff_rewrite_changes_text_over_the_spelling_classes() {
        // X1(a) fidelity pair (kills the D6 class forever): C++ discovery
        // hits exactly when the hook-built spec makes `rewrite` change the
        // text, over every spelling class (non-empty new deps assumed).
        let target_file = std::path::Path::new("include/thing.h");
        let discovery_targets = CppDependencyRewriter.consumer_discovery_targets(target_file, &[]);
        assert_eq!(
            discovery_targets,
            vec!["thing.h".to_string()],
            "discovery probes with the bare file name (trait default)"
        );
        let spec = hook_spec(&["include/thing.h"], &["generated/include/Thing_extracted.h"]);
        assert_eq!(spec.new_deps, vec!["Thing_extracted.h".to_string()]);
        let probe = &discovery_targets[0];
        let discovery_hits = |text: &str| !hits(text, probe).is_empty();

        let bare_quoted = "#include \"thing.h\"\nint main() {}\n";
        assert!(discovery_hits(bare_quoted));
        assert_ne!(CppDependencyRewriter.rewrite(bare_quoted, &spec), bare_quoted);

        let angled = "#include <thing.h>\nint main() {}\n";
        assert!(discovery_hits(angled));
        assert_ne!(CppDependencyRewriter.rewrite(angled, &spec), angled);

        let crlf = "#include \"thing.h\"\r\nint y;\r\n";
        assert!(discovery_hits(crlf));
        assert_ne!(CppDependencyRewriter.rewrite(crlf, &spec), crlf);

        // R3: comment-embedded directive, pinned on BOTH sides explicitly:
        // C++ discovery reproduces the raw substring scan (comments included)
        // and replace-all matches the same text - the symmetry IS the
        // invariant, not an accident.
        let comment_embedded = "// #include \"thing.h\"\nint main() {}\n";
        assert!(
            discovery_hits(comment_embedded),
            "C++ discovery matches comment-embedded directives (substring fidelity lock)"
        );
        assert_ne!(
            CppDependencyRewriter.rewrite(comment_embedded, &spec),
            comment_embedded,
            "the rewrite matches them too - substring symmetry (R3)"
        );

        let unrelated_path_spelling = "#include \"sub/thing.h\"\n";
        assert!(!discovery_hits(unrelated_path_spelling));
        assert_eq!(
            CppDependencyRewriter.rewrite(unrelated_path_spelling, &spec),
            unrelated_path_spelling,
            "unrelated path spelling neither hits nor changes"
        );

        let no_match = "int main() {}\n";
        assert!(!discovery_hits(no_match));
        assert_eq!(
            CppDependencyRewriter.rewrite(no_match, &spec),
            no_match,
            "no-match text stays byte-identical"
        );
    }
}
