//! [`ScalaDependencyRewriter`] — Scala import-directive text (plan M23, T6).
//!
//! Mirrors the C++ sibling's responsibilities with `import` grammar:
//! - [`DependencyRewriter::rewrite`] replaces every statement whose selector
//!   module path EQUALS an old dep (all-occurrence discipline, scanner-backed
//!   so commented/quoted imports are never touched). The locked statement
//!   strategy is **line expansion**: a touched statement is replaced by one
//!   `import <path>` line per surviving selector plus one per new dep (new
//!   deps land at the matched selector's slot, survivors keep source order),
//!   all indented to the statement's first column. An old dep with an EMPTY
//!   new list deletes the statement — its whole line when nothing else lives
//!   on it. A wildcard `import base._` matches when the old dep IS the
//!   aggregate (`base`) or the wildcard spelling (`base._`): it narrows to
//!   the new file's module path, D-M23-5.
//! - [`DependencyRewriter::render_diff`] uses the C++ diff grammar with
//!   `import` spelling: `    - import <dep>` / `    + import <dep>` /
//!   `    ~ import <dep> // side-effect preserved`.
//! - [`DependencyRewriter::find_dependency_targets`] reports named/grouped
//!   statements whose module path equals the target, plus wildcard
//!   statements whose base is the target's package prefix (D-M23-7 iii).
//! - [`DependencyRewriter::remove_top_level_symbols`] deletes top-level
//!   regions via the T5 locator, companion-paired (D17), for move semantics
//!   (D-M23-5); every non-removed byte stays verbatim.

use crate::imports::{
    char_to_byte_map, scan_imports, ImportPass, ImportStatement, ParsedSelector, SelectorForm,
};
use ariadne_lang::{DependencyRewriter, DirectiveHit, RewriteSpec};
use std::path::{Path, PathBuf};

/// The package a standard sbt-layout source path declares: the directory
/// components between the `src/main/scala` root marker and the file name
/// (the same shape `ScalaUnitShaper` shapes units from). `None` outside a
/// recognized root — callers stay conservative there.
fn source_package(path: &Path) -> Option<String> {
    let parts: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    let root = ["src".to_string(), "main".to_string(), "scala".to_string()];
    let start = parts.windows(root.len()).position(|w| w == root)? + root.len();
    if start >= parts.len().saturating_sub(1) {
        return None;
    }
    let package = parts[start..parts.len() - 1].join(".");
    (!package.is_empty()).then_some(package)
}

/// Statement-level target relation (D-M23-7 iii consumer discovery): a named
/// or grouped selector whose module path EQUALS the target, or a wildcard
/// whose base is the target itself or the target's package prefix — LOCKED:
/// `import a.b._` is target-affected for `a.b`, for `a.b.C`, and for
/// `a.b.deep.D` (qualified access through the wildcard), never for a sibling
/// package sharing only a character prefix (`ab.C` is not under `a`).
fn statement_mentions(form: &SelectorForm, target: &str) -> bool {
    match form {
        SelectorForm::Plain { .. } | SelectorForm::Grouped { .. } => {
            form.module_path().as_deref() == Some(target)
        }
        SelectorForm::Wildcard { base } => {
            target == base || target.starts_with(&format!("{base}."))
        }
    }
}

/// Rewrites Scala import directives in consumer sources.
pub struct ScalaDependencyRewriter;

/// True when a selector is the old dep: exact module-path equality, or for
/// a wildcard the aggregate path (`base`) / wildcard spelling (`base._`).
fn selector_affected(form: &SelectorForm, old_dep: &str) -> bool {
    match form {
        SelectorForm::Plain { .. } | SelectorForm::Grouped { .. } => {
            form.module_path().as_deref() == Some(old_dep)
        }
        SelectorForm::Wildcard { base } => {
            old_dep == base || old_dep == format!("{base}._")
        }
    }
}

impl DependencyRewriter for ScalaDependencyRewriter {
    fn rewrite(&self, source: &str, spec: &RewriteSpec) -> String {
        let mut content = source.to_string();
        for old_dep in &spec.old_deps {
            content = rewrite_for_dep(&content, old_dep, &spec.new_deps);
        }
        content
    }

    fn find_dependency_targets(&self, source: &str, target: &str) -> Vec<DirectiveHit> {
        let pass = scan_imports(source);
        pass.statements
            .iter()
            .filter(|statement| {
                statement
                    .selectors
                    .iter()
                    .any(|selector| statement_mentions(&selector.form, target))
            })
            .map(|statement| DirectiveHit {
                start_line: pass.tokens[statement.start_tok].line,
                end_line: pass.tokens[statement.end_tok - 1].line,
                raw_directive: pass.statement_text(source, statement),
            })
            .collect()
    }

    fn remove_top_level_symbols(
        &self,
        source: &str,
        qualified_names: &[String],
    ) -> Option<String> {
        crate::locator::removal::removal_text(source, qualified_names)
    }

    fn render_diff(&self, removed: &[String], added: &[String], preserved: &[String]) -> String {
        // The C++ diff grammar's shape (4-space indent + marker), Scala
        // spelling; preserved entries carry the side-effect comment.
        let mut out = String::new();
        for dep in removed {
            out.push_str(&format!("    - import {dep}\n"));
        }
        for dep in added {
            out.push_str(&format!("    + import {dep}\n"));
        }
        for dep in preserved {
            out.push_str(&format!("    ~ import {dep} // side-effect preserved\n"));
        }
        out
    }

    /// Consumer discovery probes with the moved units' module paths (T8b
    /// fence lift, D-M23-7 iii): a wildcard `import <base>._` is target-
    /// affected for every id under `base`, a named statement for its exact
    /// id. Companion-object ids (`X$`) are not import syntax — the class id
    /// already covers the companion (D17).
    fn consumer_discovery_targets(
        &self,
        _target_file: &Path,
        moved_module_ids: &[String],
    ) -> Vec<String> {
        moved_module_ids
            .iter()
            .filter(|id| !id.contains('$'))
            .cloned()
            .collect()
    }

    /// The path-space rewrite payload translated into import grammar (T8b,
    /// the mapping C++ never needed because its directive text IS the path):
    /// the shared file's old dep is its package (the wildcard's aggregate —
    /// `selector_affected` equality), and the new deps are the moved units'
    /// module paths under that package. Ids outside the package (co-proposed
    /// symbols that keep their own package) are NOT importable through the
    /// narrowed directive and are dropped; `$`-suffixed companion ids are
    /// import syntax only via their class id. A path outside the standard
    /// source root yields `None`: the caller keeps the path strings, which
    /// match no import and rewrite nothing (conservative, never corrupt).
    fn consumer_rewrite_spec(
        &self,
        old_includes: &[PathBuf],
        _new_includes: &[PathBuf],
        moved_module_ids: &[String],
    ) -> Option<RewriteSpec> {
        let package = source_package(old_includes.first()?)?;
        let mut new_deps: Vec<String> = moved_module_ids
            .iter()
            .filter(|id| {
                !id.contains('$')
                    && id.rsplit_once('.')
                        .is_some_and(|(parent, _)| parent == package)
            })
            .cloned()
            .collect();
        new_deps.sort();
        new_deps.dedup();
        Some(RewriteSpec {
            old_deps: vec![package],
            new_deps,
        })
    }

}

/// Replace every statement affected by `old_dep` (re-scan per dep: later
/// deps see the expanded survivors of earlier rewrites, mirroring the C++
/// per-dep sequential replace-all).
fn rewrite_for_dep(source: &str, old_dep: &str, new_deps: &[String]) -> String {
    let pass = scan_imports(source);
    let bytes = char_to_byte_map(source);
    let mut edits: Vec<(usize, usize, String)> = Vec::new();
    for statement in &pass.statements {
        if !statement
            .selectors
            .iter()
            .any(|selector| selector_affected(&selector.form, old_dep))
        {
            continue;
        }
        let from = bytes[pass.tokens[statement.start_tok].start];
        let to = bytes[pass.tokens[statement.end_tok - 1].end];
        edits.push((from, to, statement_replacement(source, &pass, statement, old_dep, new_deps)));
    }
    apply_edits(source, edits)
}

/// The replacement text for one affected statement: survivor lines plus new
/// dep lines at the matched slot; empty text deletes the statement.
fn statement_replacement(
    source: &str,
    pass: &ImportPass,
    statement: &ImportStatement,
    old_dep: &str,
    new_deps: &[String],
) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut new_deps_emitted = false;
    for selector in &statement.selectors {
        if selector_affected(&selector.form, old_dep) {
            if !new_deps_emitted {
                for dep in new_deps {
                    lines.push(format!("{} {dep}", statement.keyword));
                }
                new_deps_emitted = true;
            }
            continue;
        }
        lines.push(survivor_line(source, pass, statement, selector));
    }
    let from = pass.tokens[statement.start_tok].start;
    let joined = join_indented(&lines, line_indent(source, from));
    if joined.is_empty() {
        return String::new();
    }
    joined
}

/// Re-render one untouched selector, preserving its spelling shape: plain
/// chains as the full path, grouped members inside braces verbatim (renames
/// keep meaning), wildcards as `base._`.
fn survivor_line(
    source: &str,
    pass: &ImportPass,
    statement: &ImportStatement,
    selector: &ParsedSelector,
) -> String {
    let keyword = &statement.keyword;
    match &selector.form {
        SelectorForm::Plain { module_path, .. } => format!("{keyword} {module_path}"),
        SelectorForm::Grouped { base, .. } => {
            let text = pass.selector_text(source, selector);
            format!("{keyword} {base}.{{{text}}}")
        }
        SelectorForm::Wildcard { base } => format!("{keyword} {base}._"),
    }
}

fn join_indented(lines: &[String], indent: String) -> String {
    lines.join(&format!("\n{indent}"))
}

/// Whitespace prefix of the line holding char offset `char_from`.
fn line_indent(source: &str, char_from: usize) -> String {
    let byte_from = char_to_byte_map(source)[char_from];
    let head = &source[..byte_from];
    match head.rfind('\n') {
        Some(nl) if head[nl + 1..].chars().all(char::is_whitespace) => {
            head[nl + 1..].to_string()
        }
        None if head.chars().all(char::is_whitespace) => head.to_string(),
        _ => String::new(),
    }
}

/// Splice the edits (descending by start) into `source`. An edit whose
/// replacement is empty removes the statement's whole line when the rest of
/// that line is whitespace, including its terminator (CRLF-aware).
fn apply_edits(source: &str, edits: Vec<(usize, usize, String)>) -> String {
    let mut content = source.to_string();
    let mut edits = edits;
    edits.sort_by_key(|(from, _, _)| std::cmp::Reverse(*from));
    for (from, to, replacement) in edits {
        let (from, to, replacement) = if replacement.is_empty() {
            widen_to_full_line(&content, from, to)
        } else {
            (from, to, replacement)
        };
        content.replace_range(from..to, &replacement);
    }
    content
}

/// Extend an empty replacement to the whole line when the statement is
/// alone on it: the line's leading whitespace and trailing `\r?\n` join the
/// removal span.
fn widen_to_full_line(content: &str, from: usize, to: usize) -> (usize, usize, String) {
    let head = &content[..from];
    let line_start = head.rfind('\n').map_or(0, |nl| nl + 1);
    let tail = &content[to..];
    let line_end_relative = tail.find('\n').map_or(tail.len(), |nl| nl + 1);
    let tail_of_line = &content[to..to + line_end_relative];
    let alone = head[line_start..].chars().all(char::is_whitespace)
        && tail_of_line.chars().all(char::is_whitespace);
    if alone {
        (line_start, to + line_end_relative, String::new())
    } else {
        (from, to, String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(old: &[&str], new: &[&str]) -> RewriteSpec {
        RewriteSpec {
            old_deps: old.iter().map(|d| d.to_string()).collect(),
            new_deps: new.iter().map(|d| d.to_string()).collect(),
        }
    }

    fn rw(source: &str, old: &[&str], new: &[&str]) -> String {
        ScalaDependencyRewriter.rewrite(source, &spec(old, new))
    }

    #[test]
    fn rewrite_named_import_keeps_every_other_byte_of_the_file() {
        let source = "package p\n\nimport a.b.Old // trailing note\n\nclass Use {\n  val s = \"import a.b.Old\"\n  val o: Old = null\n}\n";
        let out = rw(source, &["a.b.Old"], &["a.b.extraction.OldModel"]);
        let expected = "package p\n\nimport a.b.extraction.OldModel // trailing note\n\nclass Use {\n  val s = \"import a.b.Old\"\n  val o: Old = null\n}\n";
        assert_eq!(out, expected);
        assert_eq!(
            expected.replace("import a.b.extraction.OldModel", "import a.b.Old"),
            source,
            "byte-preservation proof: the ONLY difference is the statement text"
        );
    }

    #[test]
    fn rewrite_locks_grouped_line_expansion_with_new_deps_at_the_matched_slot() {
        assert_eq!(
            rw("import a.b.{Old, Keep}\n", &["a.b.Old"], &["a.b.New1", "other.pkg.New2"]),
            "import a.b.New1\nimport other.pkg.New2\nimport a.b.{Keep}\n",
            "LOCK: touched statements expand to one import line per new dep \
             (at the matched selector's slot) plus survivors in source order; \
             the survivor keeps brace spelling so renames cannot change meaning"
        );
        assert_eq!(
            rw("import a.b.{Keep, Old}\n", &["a.b.Old"], &["x.y.Z"]),
            "import a.b.{Keep}\nimport x.y.Z\n",
            "LOCK: slot order — survivors before the match stay before it"
        );
    }

    #[test]
    fn empty_new_deps_deletes_the_statement_line_cleanly() {
        let source = "class Above\n\nimport a.b.Old\n\nclass Below\n";
        assert_eq!(rw(source, &["a.b.Old"], &[]), "class Above\n\n\nclass Below\n");
        assert_eq!(
            rw("import a.b.{Old, Keep}\n", &["a.b.Old"], &[]),
            "import a.b.{Keep}\n",
            "a partially emptied group keeps its survivors"
        );
    }

    #[test]
    fn rewrite_keeps_the_statements_indentation() {
        assert_eq!(
            rw("object Wrapper {\n  import a.b.Old\n}\n", &["a.b.Old"], &["x.Y", "z.Z"]),
            "object Wrapper {\n  import x.Y\n  import z.Z\n}\n"
        );
    }

    #[test]
    fn every_occurrence_of_the_old_dep_is_replaced() {
        assert_eq!(
            rw("import a.b.Old\nimport other.T\nimport a.b.Old\n", &["a.b.Old"], &["q.New"]),
            "import q.New\nimport other.T\nimport q.New\n",
            "C++ all-occurrence discipline, scanner-backed"
        );
    }

    #[test]
    fn commented_and_quoted_imports_are_never_rewritten() {
        let source = "// import a.b.Old\nval doc = \"import a.b.Old\"\nimport a.b.Old\n";
        assert_eq!(
            rw(source, &["a.b.Old"], &["q.New"]),
            "// import a.b.Old\nval doc = \"import a.b.Old\"\nimport q.New\n"
        );
    }

    #[test]
    fn wildcard_statement_narrows_to_the_new_module_path() {
        assert_eq!(
            rw("import a.b._\nclass Use\n", &["a.b"], &["a.b.extraction.C"]),
            "import a.b.extraction.C\nclass Use\n",
            "D-M23-5: the aggregate-wide wildcard narrows when the old dep IS \
             the aggregate; the wildcard spelling matches too"
        );
        assert_eq!(
            rw("import a.b._\n", &["a.b._"], &["a.b.extraction.C"]),
            "import a.b.extraction.C\n"
        );
        assert_eq!(
            rw("import a.b._\n", &["a.b.C"], &["a.b.extraction.C"]),
            "import a.b._\n",
            "a symbol-level old dep does NOT touch the wildcard: package \
             resolution keeps it valid (D-M23-5 first clause)"
        );
    }

    #[test]
    fn crlf_line_terminator_is_removed_with_a_deleted_statement() {
        assert_eq!(
            rw("class A\r\nimport a.b.Old\r\nclass B\r\n", &["a.b.Old"], &[]),
            "class A\r\nclass B\r\n",
            "LOCK: the widened deletion consumes CRLF as one terminator"
        );
    }

    #[test]
    fn unrelated_bytes_survive_a_multi_dep_rewrite() {
        let source = "package com.example\n\nimport com.example.models.{Order, User}\nimport scala.collection.mutable._\n\nobject Svc {\n  def f(u: User): Unit = ???\n}\n";
        let out = ScalaDependencyRewriter.rewrite(
            source,
            &spec(
                &["com.example.models.Order"],
                &["com.example.models.order.Order"],
            ),
        );
        assert_eq!(
            out,
            "package com.example\n\nimport com.example.models.order.Order\n\
             import com.example.models.{User}\nimport scala.collection.mutable._\n\n\
             object Svc {\n  def f(u: User): Unit = ???\n}\n"
        );
    }

    fn hits(source: &str, target: &str) -> Vec<DirectiveHit> {
        ScalaDependencyRewriter.find_dependency_targets(source, target)
    }

    #[test]
    fn named_grouped_and_wildcard_statements_are_found_for_their_targets() {
        let source = "package p\n\nimport a.b.C\nimport d.e.{F, G}\nimport h._\n";
        let named: Vec<(usize, usize, String)> = hits(source, "a.b.C")
            .into_iter()
            .map(|h| (h.start_line, h.end_line, h.raw_directive))
            .collect();
        assert_eq!(named, vec![(3, 3, "import a.b.C".to_string())]);
        assert_eq!(hits(source, "d.e.G").len(), 1, "a grouped member hits its statement");
        assert_eq!(hits(source, "h").len(), 1, "the aggregate itself is target-affected");
        assert_eq!(hits(source, "h.K").len(), 1, "a direct member is under the wildcard");
        assert_eq!(hits(source, "h.m.N").len(), 1, "qualified access through the wildcard");
        assert!(hits(source, "h2.K").is_empty());
    }

    #[test]
    fn wildcard_prefix_rule_never_matches_a_sibling_package() {
        let source = "import a._\nimport ab.C\n";
        assert_eq!(hits(source, "ab.C")[0].raw_directive, "import ab.C");
        assert_eq!(hits(source, "a.K")[0].raw_directive, "import a._");
        assert!(hits(source, "z.z.C").is_empty());
    }

    #[test]
    fn commented_and_quoted_occurrences_are_never_hits() {
        assert!(hits("// import a.b.C\nval doc = \"import a.b.C\"\n", "a.b.C").is_empty());
    }

    #[test]
    fn multiline_grouped_hit_reports_full_line_range_and_verbatim_text() {
        let source = "import a.b.{\n  Old,\n  Keep\n}\nclass Use\n";
        let hits = hits(source, "a.b.Old");
        assert_eq!(hits.len(), 1);
        assert_eq!((hits[0].start_line, hits[0].end_line), (1, 4));
        assert_eq!(hits[0].raw_directive, "import a.b.{\n  Old,\n  Keep\n}");
    }

    #[test]
    fn multiple_occurrences_yield_multiple_hits_in_source_order() {
        let hits = hits("import a.b.C\nimport other.T\nimport a.b.C\n", "a.b.C");
        assert_eq!(
            hits.iter().map(|h| h.start_line).collect::<Vec<usize>>(),
            vec![1, 3]
        );
    }

    #[test]
    fn bare_import_hits_only_its_exact_module_path() {
        assert_eq!(hits("import a.b.util\n", "a.b.util").len(), 1);
        assert!(hits("import a.b.util\n", "a.b").is_empty());
    }

    #[test]
    fn remove_top_level_symbols_delegates_to_the_locator_removal() {
        let names = ["Drop".to_string()];
        assert_eq!(
            ScalaDependencyRewriter
                .remove_top_level_symbols("class Keep\nclass Drop\n", &names)
                .as_deref(),
            Some("class Keep\n")
        );
        assert_eq!(
            ScalaDependencyRewriter
                .remove_top_level_symbols("class Keep\n", &names),
            None,
            "an unremovable name yields None: the caller falls back without removal"
        );
    }

    #[test]
    fn render_diff_uses_the_cpp_grammar_with_import_spelling() {
        assert_eq!(
            ScalaDependencyRewriter.render_diff(
                &["a.b.C".to_string()],
                &["a.b.Cmodel".to_string()],
                &["scala.collection.mutable._".to_string()],
            ),
            "    - import a.b.C\n    + import a.b.Cmodel\n    ~ import scala.collection.mutable._ // side-effect preserved\n"
        );
        assert_eq!(ScalaDependencyRewriter.render_diff(&[], &[], &[]), "");
    }

    #[test]
    fn consumer_discovery_targets_are_the_module_paths_without_companions() {
        let ids = vec![
            "com.example.legacy.LegacyReport".to_string(),
            "com.example.legacy.LegacyLedger".to_string(),
            "com.example.legacy.LegacyLedger$".to_string(),
        ];
        assert_eq!(
            ScalaDependencyRewriter.consumer_discovery_targets(
                Path::new("src/main/scala/com/example/legacy/legacy.scala"),
                &ids,
            ),
            vec![
                "com.example.legacy.LegacyReport".to_string(),
                "com.example.legacy.LegacyLedger".to_string(),
            ],
            "D-M23-7 iii: named module paths; the companion `$` id is not import syntax"
        );
    }

    #[test]
    fn consumer_rewrite_spec_maps_shared_file_to_package_and_moved_units() {
        let spec = ScalaDependencyRewriter
            .consumer_rewrite_spec(
                &[PathBuf::from("src/main/scala/com/example/legacy/legacy.scala")],
                &[PathBuf::from("src/main/scala/com/example/legacy/LegacyReport.scala")],
                &[
                    "com.example.legacy.LegacyReport".to_string(),
                    "com.example.legacy.LegacyLedger$".to_string(),
                    "com.example.legacy.records.AuditTrail".to_string(),
                ],
            )
            .expect("an sbt-layout source path names a package");
        assert_eq!(spec.old_deps, vec!["com.example.legacy".to_string()]);
        assert_eq!(
            spec.new_deps,
            vec!["com.example.legacy.LegacyReport".to_string()],
            "only ids under the shared package narrow into the directive"
        );
    }

    #[test]
    fn consumer_rewrite_spec_is_none_outside_the_source_root() {
        assert!(ScalaDependencyRewriter
            .consumer_rewrite_spec(
                &[PathBuf::from("weird/place/legacy.scala")],
                &[],
                &["com.example.LegacyReport".to_string()],
            )
            .is_none());
    }

    #[test]
    fn pin_multibyte_bytes_before_a_touched_import_survive_byte_exact() {
        // Specimen from escapes (authored file stays pure ASCII): two-byte
        // u-umlaut before the touched import, two-byte e-acute plus three-byte
        // arrow inside a string after it -- char/byte offsets matter on both sides.
        let source = "// \u{fc}ber note\nimport a.b.Old\n\nclass Use {\n  val label = \"caf\u{e9} \u{2192} v2\"\n}\n";
        let out = rw(source, &["a.b.Old"], &["a.b.New"]);
        assert_eq!(
            out.replace("a.b.New", "a.b.Old"),
            source,
            "byte-preservation proof: the ONLY difference is the statement text -- \
             the char_to_byte_map splice neither shifted nor corrupted any multibyte sequence"
        );
    }

    #[test]
    fn pin_export_statement_survivors_keep_the_export_keyword() {
        assert_eq!(
            rw("export a.b.{Old, Keep}\n", &["a.b.Old"], &["x.Y"]),
            "export x.Y\nexport a.b.{Keep}\n",
            "survivor_line and the new-dep lines both render with the statement's \
             own keyword -- an export is never rewritten into an import"
        );
    }

    #[test]
    fn pin_two_old_deps_on_one_statement_replay_expansion_once_per_matching_dep() {
        assert_eq!(
            rw("import a.b.{Old1, Old2}\n", &["a.b.Old1", "a.b.Old2"], &["n.New"]),
            "import n.New\nimport n.New\n",
            "LOCK (current behavior): rewrite_for_dep runs sequentially per dep and \
             re-scans after each, so dep 2 sees dep 1's expanded survivor statement \
             (the grouped `import a.b.Old2`) and expands it too -- the new dep lands \
             once per matching dep, an honest duplicate, not a merge"
        );
        assert_eq!(
            rw("import a.b.{Old1, Old2}\n", &["a.b.Old1", "a.b.Old2"], &[]),
            "",
            "with empty new deps the first pass leaves the survivor statement and \
             the second pass empties and line-deletes it"
        );
    }
}
