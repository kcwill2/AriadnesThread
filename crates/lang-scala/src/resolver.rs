//! Scala import/export directive resolution (plan M20.1).
//!
//! [`ScalaImportResolver`] is the stateless plugin implementation of
//! [`ariadne_lang::DependencyResolver`]: `resolve_edges` delegates to the
//! pure-text scanner in [`crate::directive_parser`], and `resolve_target`
//! maps dotted package paths onto [`FileCatalog`] records through the
//! deterministic [`FileCatalog::by_module_path_suffix`] lookup (D12).
//!
//! `resolve_target` policy (IV-5, in order): a raw target whose first
//! segment is an [`crate::boundary::EXTERNAL_PATH_MARKERS`] component is
//! `External` without any lookup; otherwise the suffix lookup decides —
//! 1 match ⇒ `Resolved`, >1 ⇒ `Wildcard(sorted)`, 0 ⇒ `External` (an
//! out-of-repo package prefix, plan §3.4 Scala column) — and a matched
//! record whose rel-path contains a marker component is `External`. The
//! `pkg._` spelling is tolerated on the target edge (it is stripped to the
//! package dir) even though `resolve_edges` never emits it (IV-2).

use crate::boundary::is_external_boundary_component;
use crate::directive_parser::parse_import_directives;
use ariadne_lang::{
    DependencyResolver, DirectiveStyle, FileCatalog, RawDepEdge, ResolutionContext, ResolveOutcome,
};
use ariadne_lang_api::types::IncludeKind;
use ariadne_repo::FileId;
use std::path::Path;

/// Stateless Scala directive resolver wired into
/// [`crate::plugin::ScalaPlugin`] (plan M20.1; IV-9 — no repository state,
/// no filesystem access, everything flows through the arguments).
#[derive(Debug, Clone, Copy, Default)]
pub struct ScalaImportResolver;

impl DependencyResolver for ScalaImportResolver {
    fn resolve_edges(
        &self,
        file_id: FileId,
        rel_path: &Path,
        source: &str,
        _ctx: &ResolutionContext<'_>,
    ) -> Vec<RawDepEdge> {
        parse_import_directives(source)
            .into_iter()
            .map(|parsed| RawDepEdge {
                source_file: file_id,
                source_rel_path: rel_path.to_path_buf(),
                raw_target: parsed.raw_target,
                is_wildcard: parsed.is_wildcard,
                style: DirectiveStyle::Unspecified,
                line: parsed.line,
                column: parsed.column,
            })
            .collect()
    }

    fn resolve_target(
        &self,
        edge: &RawDepEdge,
        catalog: &FileCatalog,
        _ctx: &ResolutionContext<'_>,
    ) -> ResolveOutcome {
        let target = strip_wildcard_suffix(edge.raw_target.trim());
        if target.is_empty() || is_external_boundary_component(first_path_segment(target)) {
            return ResolveOutcome::External;
        }
        let matches = catalog.by_module_path_suffix(target);
        match matches.len() {
            0 => ResolveOutcome::External,
            1 => resolve_single_match(catalog, matches[0]),
            _ => resolve_multiple_matches(catalog, &matches),
        }
    }

    /// IV-1 / D10: every directive this resolver emits names one dependency
    /// relation, so its resolved edges are `Import` edges regardless of the
    /// destination's catalog ownership.
    fn directive_edge_kind(&self) -> Option<IncludeKind> {
        Some(IncludeKind::Import)
    }
}

/// IV-5: the marker rule already excluded first-segment boundaries, so a
/// lone match is `Resolved` unless its rel-path crosses a boundary itself.
fn resolve_single_match(catalog: &FileCatalog, file_id: FileId) -> ResolveOutcome {
    if record_crosses_external_boundary(catalog, file_id) {
        ResolveOutcome::External
    } else {
        ResolveOutcome::Resolved(file_id)
    }
}

/// IV-5: several matches yield the sorted [`ResolveOutcome::Wildcard`] even
/// for a named directive (an honest ambiguity signal); wildcard expansion
/// never lists boundary-crossing files, and if the marker rule removes them
/// all the directive is `External`.
fn resolve_multiple_matches(catalog: &FileCatalog, matches: &[FileId]) -> ResolveOutcome {
    let in_repo: Vec<FileId> = matches
        .iter()
        .copied()
        .filter(|id| !record_crosses_external_boundary(catalog, *id))
        .collect();
    if in_repo.is_empty() {
        ResolveOutcome::External
    } else {
        ResolveOutcome::Wildcard(in_repo)
    }
}

fn record_crosses_external_boundary(catalog: &FileCatalog, file_id: FileId) -> bool {
    catalog
        .rel_path(file_id)
        .into_iter()
        .flat_map(|rel_path| rel_path.components())
        .any(|component| {
            is_external_boundary_component(component.as_os_str().to_string_lossy().as_ref())
        })
}

/// Accepts the `pkg._` spelling by resolving it to the package dir; the
/// wildcard edges emitted by `resolve_edges` already carry the bare package
/// (IV-2), so this only normalizes direct-caller spellings.
fn strip_wildcard_suffix(target: &str) -> &str {
    let without_wildcard = target.strip_suffix("._").unwrap_or(target);
    without_wildcard
        .strip_suffix('.')
        .unwrap_or(without_wildcard)
}

/// First path segment of a dotted/slash-separated target, for the marker test.
fn first_path_segment(target: &str) -> &str {
    let trimmed = target.trim_start_matches(['.', '/', '\\']);
    let end = trimmed.find(['.', '/', '\\']).unwrap_or(trimmed.len());
    &trimmed[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_lang::{DirectiveStyle, RawDepEdge, ResolutionContext, ResolveOutcome};
    use ariadne_repo::{
        FileId, FileKind, FileOwnership, OptimizationPolicy, FileRecord, SourceLanguage,
    };
    use std::path::{Path, PathBuf};

    fn record(rel: &str) -> FileRecord {
        FileRecord {
            id: FileId::from_relative_path(rel),
            path: PathBuf::from(rel),
            canonical_path: PathBuf::from(rel),
            kind: FileKind::Other,
            size: 0,
            content_hash: String::new(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
            language: SourceLanguage::Scala,
        }
    }

    fn fixture_catalog() -> FileCatalog {
        FileCatalog::from_records(&[
            record("src/main/scala/com/example/models/Order.scala"),
            record("src/main/scala/com/example/models/User.scala"),
            record("src/main/scala/com/example/models/all.scala"),
            record("src/main/scala/com/example/services/OrderService.scala"),
        ])
    }

    fn includer_rel_path() -> &'static Path {
        Path::new("src/main/scala/com/example/services/OrderService.scala")
    }

    fn edges_of(source: &str) -> Vec<RawDepEdge> {
        ScalaImportResolver::default().resolve_edges(
            FileId::from_relative_path("src/main/scala/com/example/services/OrderService.scala"),
            includer_rel_path(),
            source,
            &ResolutionContext::default(),
        )
    }

    fn targets_of(source: &str) -> Vec<(String, bool)> {
        edges_of(source)
            .into_iter()
            .map(|edge| (edge.raw_target, edge.is_wildcard))
            .collect()
    }

    fn resolve_in(catalog: &FileCatalog, raw_target: &str, is_wildcard: bool) -> ResolveOutcome {
        let edge = RawDepEdge {
            source_file: FileId::from_relative_path(
                "src/main/scala/com/example/services/OrderService.scala",
            ),
            source_rel_path: includer_rel_path().to_path_buf(),
            raw_target: raw_target.to_string(),
            is_wildcard,
            style: DirectiveStyle::Unspecified,
            line: 1,
            column: 1,
        };
        ScalaImportResolver::default().resolve_target(&edge, catalog, &ResolutionContext::default())
    }

    fn resolve(raw_target: &str, is_wildcard: bool) -> ResolveOutcome {
        resolve_in(&fixture_catalog(), raw_target, is_wildcard)
    }

    fn catalog_id(rel: &str) -> FileId {
        FileId::from_relative_path(rel)
    }

    fn models_dir_ids() -> Vec<FileId> {
        vec![
            catalog_id("src/main/scala/com/example/models/Order.scala"),
            catalog_id("src/main/scala/com/example/models/User.scala"),
            catalog_id("src/main/scala/com/example/models/all.scala"),
        ]
    }

    #[test]
    fn named_import_yields_one_unspecified_edge_at_the_keyword() {
        let edges = edges_of("import com.example.models.Order\n");
        assert_eq!(
            targets_of("import com.example.models.Order\n"),
            vec![("com.example.models.Order".to_string(), false)]
        );
        let edge = &edges[0];
        assert_eq!(edge.style, DirectiveStyle::Unspecified);
        assert_eq!((edge.line, edge.column), (1, 1));
        assert_eq!(edge.source_rel_path, includer_rel_path());
    }

    #[test]
    fn grouped_selectors_yield_one_named_edge_each() {
        assert_eq!(
            targets_of("import com.example.models.{Order, User}\n"),
            vec![
                ("com.example.models.Order".to_string(), false),
                ("com.example.models.User".to_string(), false),
            ]
        );
    }

    #[test]
    fn grouped_rename_selector_keeps_the_selected_name() {
        assert_eq!(
            targets_of("import com.example.models.{Order => O, User}\n"),
            vec![
                ("com.example.models.Order".to_string(), false),
                ("com.example.models.User".to_string(), false),
            ],
            "the edge names the selected symbol, not the rename alias"
        );
    }

    #[test]
    fn brace_group_wildcard_selector_is_a_wildcard_edge() {
        assert_eq!(
            targets_of("import com.example.models.{_}\n"),
            vec![("com.example.models".to_string(), true)]
        );
    }

    #[test]
    fn wildcard_import_yields_a_single_wildcard_edge_for_the_package() {
        assert_eq!(
            targets_of("import com.example.models._\n"),
            vec![("com.example.models".to_string(), true)],
            "IV-2: the wildcard edge carries the package, not the `._`"
        );
    }

    #[test]
    fn bare_import_is_treated_as_a_wildcard_of_its_package() {
        assert_eq!(
            targets_of("import com.example.models\n"),
            vec![("com.example.models".to_string(), true)]
        );
    }

    #[test]
    fn scala3_export_yields_edges_like_import() {
        assert_eq!(
            targets_of("export com.example.syntax.Prelude\n"),
            vec![("com.example.syntax.Prelude".to_string(), false)]
        );
    }

    #[test]
    fn package_clause_is_scope_context_never_an_edge() {
        assert!(edges_of("package com.example.models\n\nclass Foo\n").is_empty());
    }

    #[test]
    fn commented_directives_yield_no_edges() {
        assert!(edges_of("// import com.example.models.Order\n").is_empty());
        assert!(edges_of("/* import com.example.models.Order */\n").is_empty());
        assert!(
            edges_of("/*\nimport com.example.models.Order\n*/\n").is_empty(),
            "block comments span lines"
        );
        assert!(
            edges_of("/* /* */ import com.example.models.Order */\n").is_empty(),
            "block comments nest in Scala"
        );
    }

    #[test]
    fn directives_inside_strings_and_interpolation_yield_no_edges() {
        assert!(edges_of("val s = \"import com.example.models.Order\"\n").is_empty());
        assert!(
            edges_of("val t = s\"import x ${y} import a.b\"\n").is_empty(),
            "interpolated strings suppress directives"
        );
        assert!(
            edges_of("val u = s\"${\"import com.example.models.Order\"}\"\n").is_empty(),
            "quotes inside a ${{...}} splice stay inside the string"
        );
    }

    #[test]
    fn directive_after_a_string_is_still_found_with_true_position() {
        let edges = edges_of("val s = \"noise import x.y\"\n  import com.example.models.Order\n");
        assert_eq!(edges.len(), 1, "only the real directive is emitted");
        assert_eq!((edges[0].line, edges[0].column), (2, 3));
    }

    #[test]
    fn multiple_directives_across_lines_are_all_emitted_in_order() {
        let source = "package com.example.services\n\nimport com.example.models.Order\nimport com.example.models.User\n";
        assert_eq!(
            targets_of(source),
            vec![
                ("com.example.models.Order".to_string(), false),
                ("com.example.models.User".to_string(), false),
            ]
        );
    }

    #[test]
    fn named_target_resolves_the_single_matching_file() {
        assert_eq!(
            resolve("com.example.models.Order", false),
            ResolveOutcome::Resolved(catalog_id("src/main/scala/com/example/models/Order.scala"))
        );
    }

    #[test]
    fn wildcard_package_lists_all_files_sorted() {
        assert_eq!(
            resolve("com.example.models._", true),
            ResolveOutcome::Wildcard(models_dir_ids()),
            "the `._` spelling resolves to the whole package dir"
        );
    }

    #[test]
    fn plain_package_target_with_several_matches_is_a_sorted_wildcard() {
        assert_eq!(
            resolve("com.example.models", false),
            ResolveOutcome::Wildcard(models_dir_ids()),
            "IV-5: >1 matches yields Wildcard even for a named directive"
        );
    }

    #[test]
    fn out_of_repo_package_prefix_is_external() {
        assert_eq!(
            resolve("scala.collection.mutable", true),
            ResolveOutcome::External
        );
    }

    #[test]
    fn marker_first_segment_is_external_without_lookup() {
        for marker in ["lib", "target", ".ivy2", ".cache"] {
            assert_eq!(
                resolve(&format!("{marker}.legacy.Foo"), false),
                ResolveOutcome::External,
                "`{marker}` wins before any catalog lookup"
            );
        }
    }

    #[test]
    fn matched_record_under_a_marker_component_is_external() {
        let boundary_catalog = FileCatalog::from_records(&[record("lib/jars/Foo.scala")]);
        assert_eq!(
            resolve_in(&boundary_catalog, "jars.Foo", false),
            ResolveOutcome::External,
            "the `lib` rel-path component trips the marker rule"
        );
    }

    #[test]
    fn zero_match_named_target_is_external() {
        assert_eq!(
            resolve("com.example.legacy.Gone", false),
            ResolveOutcome::External
        );
    }

    #[test]
    fn pin_edge_spellings_trailing_dot_whitespace_only_and_leading_dot_targets() {
        assert_eq!(
            resolve("com.example.models.", false),
            ResolveOutcome::Wildcard(models_dir_ids()),
            "strip_wildcard_suffix's trailing-dot clause reduces `pkg.` to the bare \
             package -- identical answer to the undotted spelling"
        );
        assert_eq!(
            resolve("   ", false),
            ResolveOutcome::External,
            "a whitespace-only target is empty after the trim: External, no lookup"
        );
        assert_eq!(
            resolve(".com.example.models.Order", false),
            resolve("com.example.models.Order", false),
            "the catalog's unify_segments trims edge dots, so the leading-dot target \
             resolves EXACTLY like the plain one (pinned current behavior, honest: \
             neither first_path_segment nor the lookup rejects it)"
        );
    }
}
