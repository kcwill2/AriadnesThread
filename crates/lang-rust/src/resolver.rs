//! [`RustImportResolver`] - the `mod x;` + `use`-path resolver (rust-language
//! R-6; replaces the never-dispatched stub from R-3).
//!
//! Contract (shape sec. 3.7 resolver row): `mod x;` resolves to the child file
//! - `x.rs` (flat sibling) or `x/mod.rs` (directory sibling); `use` paths
//! resolve to repo-discovered targets through the [`FileCatalog`] +
//! [`ResolutionContext`] seam, never the filesystem. External/registry paths
//! fall to the [`EXTERNAL_PATH_MARKERS`] boundary; out-of-repo `::serde::...`
//! resolves to NOTHING - `ResolveOutcome::External`, mirroring the C++
//! system-header / Scala-lib stance (inventing an edge is the violation this
//! hand names). Every `RawDepEdge` carries the written line/col (INV-7 spans
//! honored end to end).
//!
//! Stance decisions (journal-locked before authoring):
//! - Directives are captured at the written TOP LEVEL of each file: inline
//!   `mod x { .. }` blocks are descended by the analyzer for symbols, but the
//!   resolver emits no edges for directives nested inside them - a nested
//!   `self::`/`super::` anchor is a module-path position the neutral
//!   `RawDepEdge` seam cannot carry, and anchoring it at the includer file
//!   would invent a wrong target.
//! - The style channel splits the edge families: `mod` declarations are the
//!   LOCAL file-relative spelling (`DirectiveStyle::Quoted`), `use` paths are
//!   package-style (`Unspecified`) - the spelling classification the trait
//!   doc reserves to plugins.
//! - `resolve_target` mirrors the Scala IV-5 stance: a marker first segment is
//!   `External` without any lookup; a matched record whose rel-path carries a
//!   marker is `External`; zero matches is `External` (out-of-repo); several
//!   matches yield the sorted `Wildcard` (honest ambiguity). A `use` path
//!   consumes the LONGEST written prefix that names a discovered module file,
//!   so a trailing symbol segment (`...::user::User`) is consumed and the
//!   module file answers.
//! - Whole-scope globs with no remaining segments (`use crate::*;`,
//!   `use self::*;`) are `External`: the catalog seam has no directory-listing
//!   query and touching the filesystem at this boundary is forbidden.
//! - `syn` cannot parse the source -> no edges (the analyzer's honest-empty
//!   stance: a failed parse has no written surface to name).

use crate::boundary::EXTERNAL_PATH_MARKERS;
use crate::parser_items::location_of_span;
use crate::parser_uses;
use ariadne_lang::{
    DependencyResolver, DirectiveStyle, FileCatalog, RawDepEdge, ResolutionContext, ResolveOutcome,
};
use ariadne_lang_api::types::{IncludeKind, TuAnalysis};
use ariadne_repo::FileId;
use std::path::{Path, PathBuf};

/// Stateless Rust directive resolver wired into [`crate::plugin::RustPlugin`]
/// (mirrors `ScalaImportResolver`: no repository state, no filesystem access,
/// everything flows through the arguments and the catalog).
pub struct RustImportResolver;

impl DependencyResolver for RustImportResolver {
    fn resolve_edges(
        &self,
        file_id: FileId,
        rel_path: &Path,
        source: &str,
        _ctx: &ResolutionContext<'_>,
    ) -> Vec<RawDepEdge> {
        let Ok(parsed) = syn::parse_file(source) else {
            return Vec::new();
        };
        let mut edges = Vec::new();
        for item in &parsed.items {
            match item {
                syn::Item::Mod(mod_item) if mod_item.content.is_none() => {
                    edges.push(mod_edge(file_id, rel_path, mod_item));
                }
                syn::Item::Use(use_item) => edges.extend(use_edges(use_item, file_id, rel_path)),
                _ => {}
            }
        }
        edges
    }

    fn resolve_target(
        &self,
        edge: &RawDepEdge,
        catalog: &FileCatalog,
        _ctx: &ResolutionContext<'_>,
    ) -> ResolveOutcome {
        if matches!(edge.style, DirectiveStyle::Quoted) {
            resolve_mod_target(edge, catalog)
        } else {
            resolve_use_target(edge, catalog)
        }
    }

    /// IV-1 mirror at the Rust seam: every mod/use directive names exactly one
    /// dependency relation, so resolved edges are `Import` edges (plan M20
    /// D10 - a plugin-dictated edge kind, consulted for Resolved/Wildcard).
    fn directive_edge_kind(&self) -> Option<IncludeKind> {
        Some(IncludeKind::Import)
    }
}

/// One `mod x;` declaration: raw target is the written module ident at its
/// name-token span (INV-7, same convention as the analyzer's records).
fn mod_edge(
    file_id: FileId,
    rel_path: &Path,
    mod_item: &syn::ItemMod,
) -> RawDepEdge {
    let (line, column) = location_of_span(mod_item.ident.span());
    RawDepEdge {
        source_file: file_id,
        source_rel_path: rel_path.to_path_buf(),
        raw_target: mod_item.ident.to_string(),
        is_wildcard: false,
        style: DirectiveStyle::Quoted,
        line: to_edge_position(line),
        column: to_edge_position(column),
    }
}

/// One `use` statement's raw facts, verbatim from `parser_uses` (R-5
/// discipline: written path, leaf-name span, glob at the star span, rename
/// keeps the left side). The reference channel is drained per statement so
/// facts keep source order and every occurrence.
fn use_edges(
    use_item: &syn::ItemUse,
    file_id: FileId,
    rel_path: &Path,
) -> Vec<RawDepEdge> {
    let mut facts = TuAnalysis::default();
    parser_uses::record_use(use_item, file_id, &mut facts);
    facts
        .references
        .into_iter()
        .map(|reference| {
            let raw_target = reference.symbol_id.0;
            let is_wildcard = raw_target.ends_with("::*");
            RawDepEdge {
                source_file: file_id,
                source_rel_path: rel_path.to_path_buf(),
                raw_target,
                is_wildcard,
                style: DirectiveStyle::Unspecified,
                line: to_edge_position(reference.location.line),
                column: to_edge_position(reference.location.column),
            }
        })
        .collect()
}

fn to_edge_position(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

/// `mod x;` maps onto the includer's module directory: `x.rs` first, then
/// `x/mod.rs` (the fixture's two written layouts). A child file outside the
/// discovered set is `External` - Scala's zero-match stance, never a
/// filesystem probe.
fn resolve_mod_target(edge: &RawDepEdge, catalog: &FileCatalog) -> ResolveOutcome {
    let module_dir = edge.source_rel_path.parent().unwrap_or(Path::new(""));
    let name = edge.raw_target.trim();
    if name.is_empty() {
        return ResolveOutcome::External;
    }
    let flat_sibling = module_dir.join(format!("{name}.rs"));
    let dir_sibling = module_dir.join(name).join("mod.rs");
    for candidate in [flat_sibling, dir_sibling] {
        if let Some(matched) = catalog.by_path(&candidate) {
            return outcome_for_record(catalog, matched);
        }
    }
    ResolveOutcome::External
}

/// `use`-path mapping: marker-first, root-word split (`crate` = whole-repo
/// suffix lookup like Scala's package suffix rule; `self`/`super` = directory
/// anchors at the includer module dir), longest-written-prefix wins.
fn resolve_use_target(edge: &RawDepEdge, catalog: &FileCatalog) -> ResolveOutcome {
    let written = edge.raw_target.trim();
    if written
        .trim_start_matches(':')
        .split("::")
        .next()
        .is_some_and(is_boundary_component)
    {
        return ResolveOutcome::External;
    }
    let path = written.strip_suffix("::*").unwrap_or(written);
    let segments: Vec<&str> = path.split("::").filter(|s| !s.is_empty()).collect();
    match segments.first().copied() {
        Some("crate") => resolve_by_module_suffix(&segments[1..], catalog),
        Some("self") => {
            resolve_from_dir(&module_dir_of(&edge.source_rel_path), &segments[1..], catalog)
        }
        Some("super") => {
            let mut anchor = parent_dir(module_dir_of(&edge.source_rel_path));
            let mut rest = &segments[1..];
            while rest.first().copied() == Some("super") {
                anchor = parent_dir(anchor);
                rest = &rest[1..];
            }
            resolve_from_dir(&anchor, rest, catalog)
        }
        _ => resolve_by_module_suffix(&segments, catalog),
    }
}

fn module_dir_of(rel_path: &Path) -> PathBuf {
    rel_path.parent().unwrap_or(Path::new("")).to_path_buf()
}

fn parent_dir(dir: PathBuf) -> PathBuf {
    let parent = Path::new(&dir).parent().unwrap_or(Path::new(""));
    parent.to_path_buf()
}

/// Longest written prefix of the remaining path that names discovered module
/// files: full path first, trailing (symbol) segments dropped one by one.
/// Boundary records never expand (Scala IV-5), and a fully-boundary match set
/// is `External`.
fn resolve_by_module_suffix(segments: &[&str], catalog: &FileCatalog) -> ResolveOutcome {
    for depth in (1..=segments.len()).rev() {
        let hits = catalog.by_module_path_suffix(&segments[..depth].join("."));
        if hits.is_empty() {
            continue;
        }
        let in_repo: Vec<FileId> = hits
            .into_iter()
            .filter(|id| !crosses_external_boundary(catalog, *id))
            .collect();
        return match in_repo.len() {
            0 => ResolveOutcome::External,
            1 => ResolveOutcome::Resolved(in_repo[0]),
            _ => ResolveOutcome::Wildcard(in_repo),
        };
    }
    ResolveOutcome::External
}

/// Directory-anchored descent for `self::`/`super::` paths: each segment must
/// answer as `dir/x.rs` or `dir/x/mod.rs`; the descent stops at the first
/// miss, so a trailing symbol segment (`self::helper::Thing`) is consumed and
/// the deepest module file answers.
fn resolve_from_dir(anchor: &Path, segments: &[&str], catalog: &FileCatalog) -> ResolveOutcome {
    let mut dir = anchor.to_path_buf();
    let mut last_hit: Option<FileId> = None;
    for segment in segments {
        let flat = dir.join(format!("{segment}.rs"));
        if let Some(matched) = catalog.by_path(&flat) {
            last_hit = Some(matched);
            dir = dir.join(segment);
            continue;
        }
        let sibling = dir.join(segment).join("mod.rs");
        if let Some(matched) = catalog.by_path(&sibling) {
            last_hit = Some(matched);
            dir = dir.join(segment);
            continue;
        }
        break;
    }
    match last_hit {
        Some(matched) => outcome_for_record(catalog, matched),
        None => ResolveOutcome::External,
    }
}

/// A matched record under a boundary component is `External` (Scala's record
/// rule - `vendor/legacy/legacy.rs` is never a dependency target); otherwise
/// the record is the resolution.
fn outcome_for_record(catalog: &FileCatalog, matched: FileId) -> ResolveOutcome {
    if crosses_external_boundary(catalog, matched) {
        ResolveOutcome::External
    } else {
        ResolveOutcome::Resolved(matched)
    }
}

fn crosses_external_boundary(catalog: &FileCatalog, file_id: FileId) -> bool {
    catalog
        .rel_path(file_id)
        .into_iter()
        .flat_map(|rel_path| rel_path.components())
        .any(|component| {
            is_boundary_component(component.as_os_str().to_string_lossy().as_ref())
        })
}

fn is_boundary_component(name: &str) -> bool {
    EXTERNAL_PATH_MARKERS.iter().any(|marker| *marker == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_repo::{FileKind, FileOwnership, OptimizationPolicy, FileRecord, SourceLanguage};

    fn record(rel: &str) -> FileRecord {
        FileRecord {
            id: FileId::from_relative_path(rel),
            path: PathBuf::from(rel),
            canonical_path: PathBuf::from(rel),
            kind: FileKind::RustSource,
            size: 0,
            content_hash: String::new(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
            language: SourceLanguage::Rust,
        }
    }

    fn fixture_catalog() -> FileCatalog {
        FileCatalog::from_records(&[
            record("src/lib.rs"),
            record("src/greeting.rs"),
            record("src/models/mod.rs"),
            record("src/models/user.rs"),
            record("vendor/legacy/legacy.rs"),
        ])
    }

    fn edges_of(rel: &str, source: &str) -> Vec<RawDepEdge> {
        RustImportResolver.resolve_edges(
            FileId::from_relative_path(rel),
            Path::new(rel),
            source,
            &ResolutionContext::default(),
        )
    }

    fn target(rel: &str, raw_target: &str, style: DirectiveStyle) -> ResolveOutcome {
        let edge = RawDepEdge {
            source_file: FileId::from_relative_path(rel),
            source_rel_path: PathBuf::from(rel),
            raw_target: raw_target.to_string(),
            is_wildcard: false,
            style,
            line: 1,
            column: 1,
        };
        RustImportResolver.resolve_target(
            &edge,
            &fixture_catalog(),
            &ResolutionContext::default(),
        )
    }

    fn id(rel: &str) -> FileId {
        FileId::from_relative_path(rel)
    }

    #[test]
    fn mod_flat_sibling_layout_resolves_with_span_at_the_name_token() {
        let edges = edges_of("src/lib.rs", "mod greeting;\n");
        assert_eq!(edges.len(), 1);
        let edge = &edges[0];
        assert_eq!(edge.raw_target, "greeting");
        assert!(!edge.is_wildcard);
        assert!(matches!(edge.style, DirectiveStyle::Quoted));
        assert_eq!((edge.line, edge.column), (1, 5), "INV-7 at the resolver seam");
        assert_eq!(
            RustImportResolver.resolve_target(edge, &fixture_catalog(), &ResolutionContext::default()),
            ResolveOutcome::Resolved(id("src/greeting.rs"))
        );
    }

    #[test]
    fn mod_dir_sibling_layout_resolves_to_the_mod_rs_file() {
        let edges = edges_of("src/lib.rs", "mod models;\n");
        assert_eq!(
            RustImportResolver.resolve_target(&edges[0], &fixture_catalog(), &ResolutionContext::default()),
            ResolveOutcome::Resolved(id("src/models/mod.rs")),
            "`mod models;` finds src/models/mod.rs when no src/models.rs exists"
        );
    }

    #[test]
    fn mod_declaration_with_no_discovered_child_is_external() {
        let edges = edges_of("src/lib.rs", "mod absent;\n");
        assert_eq!(
            RustImportResolver.resolve_target(&edges[0], &fixture_catalog(), &ResolutionContext::default()),
            ResolveOutcome::External,
            "a declared module with no discovered file is the honest boundary \
             - zero-match stance mirroring Scala, never a filesystem probe"
        );
    }

    #[test]
    fn mod_child_under_a_boundary_component_is_external() {
        let catalog = FileCatalog::from_records(&[
            record("src/target/parent.rs"),
            record("src/target/child.rs"),
        ]);
        let edges = edges_of("src/target/parent.rs", "mod child;\n");
        assert_eq!(
            RustImportResolver.resolve_target(&edges[0], &catalog, &ResolutionContext::default()),
            ResolveOutcome::External,
            "the `target` component of the candidate rel-path trips the marker rule"
        );
    }

    #[test]
    fn use_symbol_leaf_consumed_and_module_file_answers() {
        assert_eq!(
            target("src/lib.rs", "crate::models::user::User", DirectiveStyle::Unspecified),
            ResolveOutcome::Resolved(id("src/models/user.rs")),
            "the longest written module prefix names src/models/user.rs; the \
             symbol leaf is never a file claim"
        );
    }

    #[test]
    fn out_of_repo_spellings_are_external_never_unresolved() {
        for spelling in ["::serde::Serialize", "serde::Serialize"] {
            assert_eq!(
                target("src/lib.rs", spelling, DirectiveStyle::Unspecified),
                ResolveOutcome::External,
                "out-of-repo paths resolve to NOTHING - Unresolved would run \
                 the builder's on-disk fallback and invent a candidate"
            );
        }
    }

    #[test]
    fn module_path_with_several_matches_is_a_sorted_wildcard() {
        let models_dir = vec![id("src/models/mod.rs"), id("src/models/user.rs")];
        assert_eq!(
            target("src/lib.rs", "crate::models", DirectiveStyle::Unspecified),
            ResolveOutcome::Wildcard(models_dir.clone()),
            "IV-5 mirror: several matches yield the sorted Wildcard even for \
             a named directive (module-key order)"
        );
        assert_eq!(
            target("src/lib.rs", "crate::models::*", DirectiveStyle::Unspecified),
            ResolveOutcome::Wildcard(models_dir),
            "a glob expands to the same module-dir set"
        );
    }

    #[test]
    fn whole_scope_globs_stay_external() {
        assert_eq!(
            target("src/lib.rs", "crate::*", DirectiveStyle::Unspecified),
            ResolveOutcome::External,
            "no directory-listing query exists at the catalog seam; an honest \
             External beats an invented crate-root expansion"
        );
    }

    #[test]
    fn self_and_super_anchor_at_the_includer_module_dir() {
        assert_eq!(
            target("src/models/mod.rs", "self::user", DirectiveStyle::Unspecified),
            ResolveOutcome::Resolved(id("src/models/user.rs"))
        );
        assert_eq!(
            target("src/models/user.rs", "super::greeting", DirectiveStyle::Unspecified),
            ResolveOutcome::Resolved(id("src/greeting.rs")),
            "`super::` steps one directory up from the includer module dir"
        );
        assert_eq!(
            target("src/models/mod.rs", "self::absent::Thing", DirectiveStyle::Unspecified),
            ResolveOutcome::External,
            "a descent that never hits a module file resolves to nothing"
        );
    }

    #[test]
    fn marker_first_segment_is_external_without_lookup() {
        for marker in ["target", "vendor", ".cargo"] {
            assert_eq!(
                target(
                    "src/lib.rs",
                    &format!("{marker}::legacy::Legacy"),
                    DirectiveStyle::Unspecified
                ),
                ResolveOutcome::External,
                "`{marker}` wins before any catalog lookup (Scala's rule)"
            );
        }
        assert_eq!(
            target("src/lib.rs", "legacy::Legacy", DirectiveStyle::Unspecified),
            ResolveOutcome::External,
            "the only `legacy` module lives under vendor/ - the matched \
             record's marker component keeps it external"
        );
    }

    #[test]
    fn glob_rename_and_absolute_facts_arrive_verbatim() {
        let edges = edges_of(
            "src/lib.rs",
            "use crate::a::b as c;\nuse crate::d::*;\nuse ::serde::Serialize;\n",
        );
        let surface: Vec<(String, bool, u32, u32)> = edges
            .iter()
            .map(|edge| {
                (
                    edge.raw_target.clone(),
                    edge.is_wildcard,
                    edge.line,
                    edge.column,
                )
            })
            .collect();
        assert_eq!(
            surface,
            vec![
                ("crate::a::b".to_string(), false, 1, 15),
                ("crate::d::*".to_string(), true, 2, 15),
                ("::serde::Serialize".to_string(), false, 3, 14),
            ],
            "rename keeps the left side, glob rides at its star span, the \
             absolute spelling stays written - parser_uses facts verbatim"
        );
    }

    #[test]
    fn inline_mod_blocks_yield_no_edges() {
        let edges = edges_of("src/lib.rs", "mod a { mod b; pub fn inner() {} }\n");
        assert!(
            edges.is_empty(),
            "the top-level stance is deliberate: a nested declaration's \
             module-path anchor cannot ride RawDepEdge, and mis-anchoring it \
             at src/lib.rs would invent a wrong target"
        );
    }

    #[test]
    fn unparseable_source_yields_no_edges() {
        assert!(
            edges_of("src/lib.rs", "pub fn broken(").is_empty(),
            "a failed parse has no written surface - the analyzer's \
             honest-empty stance at the resolver seam"
        );
    }

    #[test]
    fn directive_edge_kind_is_import() {
        assert_eq!(
            RustImportResolver.directive_edge_kind(),
            Some(IncludeKind::Import),
            "IV-1 mirror: every mod/use directive names one relation"
        );
    }
}
