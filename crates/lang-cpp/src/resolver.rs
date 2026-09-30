//! C++ include search-path resolver (plan M18.1: relocated verbatim from
//! `crates/graph/src/resolver.rs`).
//!
//! Two surfaces live here:
//!
//! * [`IncludeResolver`] — the transitional, byte-identical public API kept so
//!   existing call sites compile unchanged through the `#[deprecated]`
//!   `ariadne-graph` re-export (removed in M18.11).
//! * [`CppDependencyResolver`] — the stateless plugin implementation of
//!   `ariadne_lang::DependencyResolver` (`resolve_edges` = parse,
//!   `resolve_target` = the same include-directory search logic driven
//!   through [`FileCatalog`]/[`ResolveOutcome`] per plan §3.3.
//!
//! `resolve_target` mapping (M18.5 reconciliation, closes the M18.2-review
//! "External-drop" divergence):
//! * candidate search order is identical to `resolve_directive`: source dir
//!   first for quoted directives (the directive's spelling travels on the
//!   edge as `DirectiveStyle`, so angled directives no longer borrow the
//!   quoted source-dir candidate — the M18.2 divergence is closed), then the
//!   caller-supplied search paths in `ResolutionContext::search_paths`
//!   (the compilation context's include dirs followed by system include
//!   dirs, the same order `collect_include_candidates` uses);
//! * any [`FileCatalog`] hit — repository, toolchain, or third-party record —
//!   maps to `Resolved(FileId)`. The legacy walk visited every directive
//!   whose `resolve_directive` produced a destination `FileId`, including
//!   `Local`/`System` *and* `External`(ThirdParty)-kind edges, and merged
//!   them into closures/graphs; returning `External` (which carries no id)
//!   for third-party records dropped exactly those files from the plugin
//!   walk. Kind classification (Local/System/External) is a property of the
//!   *edge*, derived from catalog ownership by the graph layer — not a
//!   reason to hide the destination;
//! * `External` is reserved for a genuinely out-of-repo boundary — a file
//!   the resolver locates outside `repo_root` (legacy attached only a
//!   synthetic in-repo path id, so no legacy fixture behavior maps to it;
//!   Scala's unresolved package prefix will, M20);
//! * no match maps to `Unresolved`. The trait stays filesystem-free like its
//!   default: the legacy resolver's on-disk fallback needs the filesystem,
//!   so closure-merge and graph-walk callers run the legacy full candidate
//!   loop themselves through the shared
//!   `ariadne_lang::first_ondisk_include_candidate` (`scheduler::tu_store` and
//!   `graph::builder`): includer directory for quoted spellings plus every
//!   search path, style-agnostic. An include-dir candidate that exists on disk
//!   but is absent from the catalog is therefore visited on BOTH plugin walks —
//!   the M18.5 finding-1 divergence (the plugin closure walk checked only the
//!   includer directory, dropping the search-path fallback and angled `<hdr>`
//!   hits from `-I` dirs) is CLOSED and locked by the `tu_store_plugin_parity`
//!   search-path fixture.

use crate::directive_parser::{parse_include_directives, RawIncludeDirective};
use ariadne_build::CompilationContext;
use ariadne_lang::{
    normalize_relative_path, DependencyResolver, DirectiveStyle, FileCatalog, RawDepEdge,
    ResolutionContext, ResolveOutcome,
};
use ariadne_lang_api::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_repo::{FileId, FileOwnership, FileRecord, RepositoryDiscoveryResult, SourceLanguage};
use std::path::{Path, PathBuf};

/// Normalizes a path by resolving `.` and `..` relative components without requiring existing files.
pub fn normalize_path(path: &Path) -> PathBuf {
    normalize_relative_path(path)
}

/// Ordered include search paths for one directive: source dir first for
/// quoted includes, then compilation-context user and system include dirs.
fn collect_include_candidates(
    repo_root: &Path,
    context: Option<&CompilationContext>,
    source_dir: &Path,
    include_raw: &Path,
    is_angled: bool,
) -> Vec<PathBuf> {
    let mut candidate_paths = Vec::new();

    // 1. If non-angled (`"..."`), check source directory first
    if !is_angled {
        candidate_paths.push(source_dir.join(include_raw));
    }

    // 2. Check compilation context user include dirs
    if let Some(ctx) = context {
        for inc_dir in &ctx.include_dirs {
            let candidate = if inc_dir.is_absolute() {
                if let Ok(rel) = inc_dir.strip_prefix(repo_root) {
                    rel.join(include_raw)
                } else {
                    inc_dir.join(include_raw)
                }
            } else {
                inc_dir.join(include_raw)
            };
            candidate_paths.push(candidate);
        }
    }

    // 3. Check compilation context system include dirs
    if let Some(ctx) = context {
        for sys_dir in &ctx.system_include_dirs {
            let candidate = if sys_dir.is_absolute() {
                if let Ok(rel) = sys_dir.strip_prefix(repo_root) {
                    rel.join(include_raw)
                } else {
                    sys_dir.join(include_raw)
                }
            } else {
                sys_dir.join(include_raw)
            };
            candidate_paths.push(candidate);
        }
    }

    candidate_paths
}

/// First discovery record whose (normalized) relative or canonical path
/// equals one of the candidates — verbatim search semantics of the legacy
/// resolver.
fn find_in_discovery<'a>(
    discovery: &'a RepositoryDiscoveryResult,
    candidate_paths: &[PathBuf],
) -> Option<&'a FileRecord> {
    for cand in candidate_paths {
        let norm_cand = normalize_path(cand);
        if let Some(file_record) = discovery.files.iter().find(|f| {
            normalize_path(&f.path) == norm_cand || normalize_path(&f.canonical_path) == norm_cand
        }) {
            return Some(file_record);
        }
    }
    None
}

/// Include path resolver against repository file records and compilation context.
pub struct IncludeResolver<'a> {
    pub repo_root: &'a Path,
    pub repo_discovery: Option<&'a RepositoryDiscoveryResult>,
    pub context: Option<&'a CompilationContext>,
}

impl<'a> IncludeResolver<'a> {
    pub fn new(repo_root: &'a Path) -> Self {
        Self {
            repo_root,
            repo_discovery: None,
            context: None,
        }
    }

    pub fn with_repo_discovery(mut self, repo_discovery: &'a RepositoryDiscoveryResult) -> Self {
        self.repo_discovery = Some(repo_discovery);
        self
    }

    pub fn with_context(mut self, context: &'a CompilationContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Normalizes a path by resolving `.` and `..` relative components without requiring existing files.
    pub fn normalize_path(path: &Path) -> PathBuf {
        normalize_path(path)
    }

    /// Resolves a single raw include directive for a source file.
    pub fn resolve_directive(
        &self,
        source_id: FileId,
        source_rel_path: &Path,
        directive: &RawIncludeDirective,
    ) -> IncludeEdge {
        let context_name = self
            .context
            .map(|c| c.name.as_str())
            .unwrap_or("Default")
            .to_string();
        let source_dir = source_rel_path.parent().unwrap_or_else(|| Path::new(""));
        let include_raw = Path::new(&directive.raw_path);

        let candidate_paths = collect_include_candidates(
            self.repo_root,
            self.context,
            source_dir,
            include_raw,
            directive.is_angled,
        );

        // Search in repository discovery
        let mut resolved_dest: Option<FileId> = None;
        let mut resolved_path: Option<PathBuf> = None;
        let mut kind = if directive.is_angled {
            IncludeKind::System
        } else {
            IncludeKind::Local
        };

        if let Some(disc) = self.repo_discovery {
            if let Some(file_record) = find_in_discovery(disc, &candidate_paths) {
                resolved_dest = Some(file_record.id);
                resolved_path = Some(file_record.path.clone());
                kind = match file_record.ownership {
                    FileOwnership::ThirdParty => IncludeKind::External,
                    FileOwnership::Toolchain => IncludeKind::System,
                    _ => IncludeKind::Local,
                };
            }
        }

        // Fallback: check filesystem on disk under repo_root if discovery missing or unmatched
        if resolved_dest.is_none() {
            for cand in &candidate_paths {
                let norm_cand = normalize_path(cand);
                let full_cand = self.repo_root.join(&norm_cand);
                if full_cand.exists() {
                    let rel_str = norm_cand.to_string_lossy().replace('\\', "/");
                    let file_id = FileId::from_relative_path(&rel_str);
                    resolved_dest = Some(file_id);
                    resolved_path = Some(norm_cand);
                    break;
                }
            }
        }

        IncludeEdge {
            source: source_id,
            destination: resolved_dest,
            context_name,
            source_location: SourceLocation::new(directive.line, directive.column),
            kind,
            raw_path: directive.raw_path.clone(),
            resolved_path,
            language: SourceLanguage::Cpp,
        }
    }

    /// Resolves all includes in a file given its content and relative path.
    pub fn resolve_file_includes(
        &self,
        source_id: FileId,
        source_rel_path: &Path,
        content: &str,
    ) -> Vec<IncludeEdge> {
        let directives = parse_include_directives(content);
        directives
            .into_iter()
            .map(|dir| self.resolve_directive(source_id, source_rel_path, &dir))
            .collect()
    }
}

/// Plugin-side `DependencyResolver` for C++ (plan M18.1/M18.3): `resolve_edges`
/// parses `#include` directives into [`RawDepEdge`]s (carrying the includer
/// path, directive spelling, and column so the trait needs no repository
/// state) and `resolve_target` applies the legacy include-directory search
/// through the caller's [`FileCatalog`] + [`ResolutionContext::search_paths`].
///
/// Stateless by design: a single instance backs
/// [`crate::plugin::CppPlugin::resolver`]`()` for every repository; all
/// per-repository inputs arrive per call.
#[derive(Clone, Copy, Default)]
pub struct CppDependencyResolver;

impl CppDependencyResolver {
    pub fn new() -> Self {
        Self
    }

    /// Legacy-shaped ordered candidates (module docs): quoted directives get
    /// the includer's directory first; search paths (the compilation
    /// context's include dirs then system include dirs) follow, each
    /// absolute path under `repo_root` reduced to repository-relative form
    /// exactly like `collect_include_candidates`.
    fn candidates_for(&self, edge: &RawDepEdge, ctx: &ResolutionContext<'_>) -> Vec<PathBuf> {
        let source_dir = edge
            .source_rel_path
            .parent()
            .unwrap_or_else(|| Path::new(""));
        let is_angled = matches!(edge.style, DirectiveStyle::Angled);
        let mut candidates = Vec::new();
        if !is_angled {
            candidates.push(source_dir.join(Path::new(&edge.raw_target)));
        }
        let include_raw = Path::new(&edge.raw_target);
        for search_dir in &ctx.search_paths {
            let candidate = if search_dir.is_absolute() {
                match search_dir.strip_prefix(ctx.repo_root) {
                    Ok(rel) => rel.join(include_raw),
                    Err(_) => search_dir.join(include_raw),
                }
            } else {
                search_dir.join(include_raw)
            };
            candidates.push(candidate);
        }
        candidates
    }
}

impl DependencyResolver for CppDependencyResolver {
    fn resolve_edges(
        &self,
        file_id: FileId,
        rel_path: &Path,
        source: &str,
        _ctx: &ResolutionContext<'_>,
    ) -> Vec<RawDepEdge> {
        parse_include_directives(source)
            .into_iter()
            .map(|dir| RawDepEdge {
                source_file: file_id,
                source_rel_path: rel_path.to_path_buf(),
                raw_target: dir.raw_path,
                is_wildcard: false,
                style: if dir.is_angled {
                    DirectiveStyle::Angled
                } else {
                    DirectiveStyle::Quoted
                },
                line: dir.line,
                column: dir.column,
            })
            .collect()
    }

    fn resolve_target(
        &self,
        edge: &RawDepEdge,
        catalog: &FileCatalog,
        ctx: &ResolutionContext<'_>,
    ) -> ResolveOutcome {
        for candidate in self.candidates_for(edge, ctx) {
            // `FileCatalog::by_path` reproduces the legacy two-way match
            // (normalized relative path, then canonical path) verbatim.
            if let Some(id) = catalog.by_path(&candidate) {
                return ResolveOutcome::Resolved(id);
            }
        }
        ResolveOutcome::Unresolved
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_repo::{FileKind, OptimizationPolicy, SourceLanguage};
    use std::collections::HashMap;

    fn mock_file(root: &Path, rel: &str, ownership: FileOwnership) -> FileRecord {
        FileRecord {
            id: FileId::from_relative_path(rel),
            path: PathBuf::from(rel),
            canonical_path: root.join(rel),
            kind: FileKind::Header,
            size: 0,
            content_hash: String::new(),
            ownership,
            optimization_policy: OptimizationPolicy::Optimizable,
            language: SourceLanguage::Cpp,
        }
    }

    #[test]
    fn test_normalize_path() {
        // Redundant separators
        assert_eq!(
            IncludeResolver::normalize_path(Path::new("a//b///c")),
            PathBuf::from("a/b/c")
        );

        // Single dots
        assert_eq!(
            IncludeResolver::normalize_path(Path::new("a/./b/./c")),
            PathBuf::from("a/b/c")
        );

        // Double dots (traversal)
        assert_eq!(
            IncludeResolver::normalize_path(Path::new("a/b/../c")),
            PathBuf::from("a/c")
        );

        // Multiple double dots
        assert_eq!(
            IncludeResolver::normalize_path(Path::new("a/b/../../c")),
            PathBuf::from("c")
        );

        // Relative traversal starting with .. should keep the ..
        assert_eq!(
            IncludeResolver::normalize_path(Path::new("../a")),
            PathBuf::from("../a")
        );

        // Traversal above current dir from a known path
        assert_eq!(
            IncludeResolver::normalize_path(Path::new("a/../../b")),
            PathBuf::from("../b")
        );
    }

    fn include_directive(raw_path: &str, is_angled: bool, line: u32) -> RawIncludeDirective {
        RawIncludeDirective {
            raw_path: raw_path.into(),
            is_angled,
            line,
            column: 1,
        }
    }

    fn given_resolver_fixture() -> (
        tempfile::TempDir,
        RepositoryDiscoveryResult,
        CompilationContext,
    ) {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let mut files = Vec::new();
        // The includer itself is a discovery record: real call sites always
        // have TU sources discovered, so catalog and legacy discovery lookups
        // see the same file set (the plugin gets the includer directory from
        // `RawDepEdge::source_rel_path` instead).
        files.push(mock_file(root, "src/main.cpp", FileOwnership::Repository));
        files.push(mock_file(
            root,
            "common/helper.h",
            FileOwnership::Repository,
        ));
        files.push(mock_file(
            root,
            "third_party/lib/lib.h",
            FileOwnership::ThirdParty,
        ));
        files.push(mock_file(
            root,
            "usr/include/system.h",
            FileOwnership::Toolchain,
        ));

        let discovery = RepositoryDiscoveryResult {
            root: root.to_path_buf(),
            files,
            ownership_summary: HashMap::new(),
            policy_summary: HashMap::new(),
        };

        let mut context = CompilationContext::new("test", "clang");
        context.include_dirs = vec![PathBuf::from("third_party/lib")];
        context.system_include_dirs = vec![PathBuf::from("usr/include")];

        // Setup a real disk file for the fallback test
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/sibling.h"), "").unwrap();

        (temp, discovery, context)
    }

    #[test]
    fn test_resolve_directive() {
        let (temp, discovery, context) = given_resolver_fixture();
        let root = temp.path();

        let resolver = IncludeResolver::new(root)
            .with_repo_discovery(&discovery)
            .with_context(&context);

        let source_id = FileId::from_relative_path("src/main.cpp");
        let source_rel_path = Path::new("src/main.cpp");

        // 1. Quoted local relative includes
        // Disk fallback (sibling.h is not in discovery)
        let edge1 = resolver.resolve_directive(
            source_id,
            source_rel_path,
            &include_directive("sibling.h", false, 1),
        );
        assert_eq!(edge1.kind, IncludeKind::Local);
        assert_eq!(
            edge1.destination,
            Some(FileId::from_relative_path("src/sibling.h"))
        );
        assert_eq!(edge1.resolved_path, Some(PathBuf::from("src/sibling.h")));

        // Found in discovery (repo local traversal)
        let edge2 = resolver.resolve_directive(
            source_id,
            source_rel_path,
            &include_directive("../common/helper.h", false, 2),
        );
        assert_eq!(edge2.kind, IncludeKind::Local);
        assert_eq!(
            edge2.destination,
            Some(FileId::from_relative_path("common/helper.h"))
        );

        // 2. Angled includes
        // Toolchain (System)
        let edge3 = resolver.resolve_directive(
            source_id,
            source_rel_path,
            &include_directive("system.h", true, 3),
        );
        assert_eq!(edge3.kind, IncludeKind::System);
        assert_eq!(
            edge3.destination,
            Some(FileId::from_relative_path("usr/include/system.h"))
        );

        // ThirdParty (External) - Note: include_dirs maps to External if the file is ThirdParty
        let edge4 = resolver.resolve_directive(
            source_id,
            source_rel_path,
            &include_directive("lib.h", true, 4),
        );
        assert_eq!(edge4.kind, IncludeKind::External);
        assert_eq!(
            edge4.destination,
            Some(FileId::from_relative_path("third_party/lib/lib.h"))
        );

        // 3. Unresolved include
        let edge5 = resolver.resolve_directive(
            source_id,
            source_rel_path,
            &include_directive("nonexistent.h", false, 5),
        );
        assert_eq!(edge5.destination, None);
        assert_eq!(edge5.resolved_path, None);
    }

    #[test]
    fn test_resolve_file_includes() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let mut files = Vec::new();
        files.push(mock_file(
            root,
            "include/header.h",
            FileOwnership::Repository,
        ));

        let discovery = RepositoryDiscoveryResult {
            root: root.to_path_buf(),
            files,
            ownership_summary: HashMap::new(),
            policy_summary: HashMap::new(),
        };

        let mut context = CompilationContext::new("test", "clang");
        context.include_dirs = vec![PathBuf::from("include")];

        let resolver = IncludeResolver::new(root)
            .with_repo_discovery(&discovery)
            .with_context(&context);

        let content = r#"
#include "header.h"
#include <missing.h>
"#;
        let edges = resolver.resolve_file_includes(
            FileId::from_relative_path("src/lib.cpp"),
            Path::new("src/lib.cpp"),
            content,
        );

        assert_eq!(edges.len(), 2);

        assert_eq!(edges[0].raw_path, "header.h");
        assert_eq!(
            edges[0].destination,
            Some(FileId::from_relative_path("include/header.h"))
        );

        assert_eq!(edges[1].raw_path, "missing.h");
        assert_eq!(edges[1].destination, None);
    }

    // ── CppDependencyResolver parity table vs the legacy IncludeResolver ────

    fn res_ctx<'a>(root: &'a Path, search_paths: Vec<PathBuf>) -> ResolutionContext<'a> {
        ResolutionContext {
            repo_root: root,
            search_paths,
        }
    }

    fn raw_edge(rel_source: &str, raw_target: &str, angled: bool) -> RawDepEdge {
        RawDepEdge {
            source_file: FileId::from_relative_path(rel_source),
            source_rel_path: PathBuf::from(rel_source),
            raw_target: raw_target.into(),
            is_wildcard: false,
            style: if angled {
                DirectiveStyle::Angled
            } else {
                DirectiveStyle::Quoted
            },
            line: 1,
            column: 1,
        }
    }

    fn fixture_search_paths(context: &CompilationContext) -> Vec<PathBuf> {
        context
            .include_dirs
            .iter()
            .chain(context.system_include_dirs.iter())
            .cloned()
            .collect()
    }

    #[test]
    fn resolve_target_parity_table_matches_resolve_directive() {
        let (temp, discovery, context) = given_resolver_fixture();
        let root = temp.path();
        let catalog = FileCatalog::from_records(&discovery.files);
        let ctx = res_ctx(root, fixture_search_paths(&context));

        let legacy = IncludeResolver::new(root)
            .with_repo_discovery(&discovery)
            .with_context(&context);
        let plugin = CppDependencyResolver::new();
        let source = Path::new("src/main.cpp");

        // Repository match: Local + destination id == Resolved(same id)
        let helper = FileId::from_relative_path("common/helper.h");
        let edge = raw_edge("src/main.cpp", "../common/helper.h", false);
        let legacy_edge = legacy.resolve_directive(
            edge.source_file,
            source,
            &include_directive("../common/helper.h", false, 1),
        );
        assert_eq!(legacy_edge.kind, IncludeKind::Local);
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Resolved(helper)
        );

        // Toolchain match: System + destination id == Resolved(same id)
        let sys = FileId::from_relative_path("usr/include/system.h");
        let edge = raw_edge("src/main.cpp", "system.h", true);
        let legacy_edge = legacy.resolve_directive(
            edge.source_file,
            source,
            &include_directive("system.h", true, 1),
        );
        assert_eq!(legacy_edge.kind, IncludeKind::System);
        assert_eq!(legacy_edge.destination, Some(sys));
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Resolved(sys)
        );

        // ThirdParty match: legacy kind External + destination id ==
        // Resolved(same id). The M18.2 "External-drop" divergence is closed:
        // the destination travels, and kind classification is the graph
        // layer's job via `FileCatalog::record(...).ownership`.
        let third_party = FileId::from_relative_path("third_party/lib/lib.h");
        let edge = raw_edge("src/main.cpp", "lib.h", true);
        let legacy_edge = legacy.resolve_directive(
            edge.source_file,
            source,
            &include_directive("lib.h", true, 1),
        );
        assert_eq!(legacy_edge.kind, IncludeKind::External);
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Resolved(third_party)
        );
        assert_eq!(
            catalog
                .record(third_party)
                .map(|r| r.ownership)
                .unwrap_or(FileOwnership::Repository),
            FileOwnership::ThirdParty,
            "catalog ownership lets the graph layer re-derive kind External"
        );

        // Miss: destination None == Unresolved
        let edge = raw_edge("src/main.cpp", "nonexistent.h", false);
        let legacy_edge = legacy.resolve_directive(
            edge.source_file,
            source,
            &include_directive("nonexistent.h", false, 1),
        );
        assert_eq!(legacy_edge.destination, None);
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Unresolved
        );

        // On-disk fallback (sibling.h): legacy resolves via filesystem; the
        // trait path stays filesystem-free (caller-side fallback documented).
        let edge = raw_edge("src/main.cpp", "sibling.h", false);
        let legacy_edge = legacy.resolve_directive(
            edge.source_file,
            source,
            &include_directive("sibling.h", false, 1),
        );
        assert_eq!(
            legacy_edge.destination,
            Some(FileId::from_relative_path("src/sibling.h"))
        );
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Unresolved
        );
    }

    #[test]
    fn resolve_target_search_paths_resolve_without_context_state() {
        let (temp, discovery, _fixture_context) = given_resolver_fixture();
        let root = temp.path();
        let catalog = FileCatalog::from_records(&discovery.files);

        // The resolver holds no compilation state: the caller's search
        // paths alone drive the include-dir candidates (M18.5 stateless
        // surface, mirroring the legacy compilation-context search order).
        let ctx = res_ctx(
            root,
            vec![PathBuf::from("common"), PathBuf::from("third_party/lib")],
        );
        let plugin = CppDependencyResolver::new();

        let edge = raw_edge("src/main.cpp", "helper.h", false);
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Resolved(FileId::from_relative_path("common/helper.h"))
        );

        // A third-party record reached via a search path resolves like any
        // other hit (kind classification happens at the graph layer, not in
        // the outcome).
        let edge = raw_edge("src/main.cpp", "lib.h", true);
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Resolved(FileId::from_relative_path("third_party/lib/lib.h"))
        );
    }

    #[test]
    fn angled_style_skips_the_source_dir_candidate() {
        // M18.5 angle-style reconciliation: a same-named file next to the
        // includer is only a candidate for quoted directives, matching
        // `collect_include_candidates`' `is_angled` gate.
        let (temp, discovery, _fixture_context) = given_resolver_fixture();
        let root = temp.path();
        let mut record = mock_file(root, "src/helper.h", FileOwnership::Repository);
        record.canonical_path = root.join("src/helper.h");
        let mut records = discovery.files.clone();
        records.push(record);
        let catalog = FileCatalog::from_records(&records);
        let plugin = CppDependencyResolver::new();

        // Quoted: includer directory wins.
        let quoted = raw_edge("src/main.cpp", "helper.h", false);
        assert_eq!(
            plugin.resolve_target(&quoted, &catalog, &res_ctx(root, vec![])),
            ResolveOutcome::Resolved(FileId::from_relative_path("src/helper.h"))
        );

        // Angled: no source-dir candidate and no search paths ⇒ no match.
        let angled = raw_edge("src/main.cpp", "helper.h", true);
        assert_eq!(
            plugin.resolve_target(&angled, &catalog, &res_ctx(root, vec![])),
            ResolveOutcome::Unresolved
        );
    }

    #[test]
    fn resolve_edges_maps_parse_output_to_raw_dep_edges() {
        let temp = tempfile::tempdir().unwrap();
        let plugin = CppDependencyResolver::new();
        let ctx = res_ctx(temp.path(), vec![]);
        let content = r#"
#include "thing_types.h"
#  include <vector>
#include "heavy.h" // comment
"#;
        let file_id = FileId::from_relative_path("src/thing.cpp");
        let edges = plugin.resolve_edges(file_id, Path::new("src/thing.cpp"), content, &ctx);
        let parsed = parse_include_directives(content);
        assert_eq!(edges.len(), parsed.len());
        for (edge, parsed) in edges.iter().zip(&parsed) {
            assert_eq!(edge.source_file, file_id);
            assert_eq!(edge.source_rel_path, PathBuf::from("src/thing.cpp"));
            assert_eq!(edge.raw_target, parsed.raw_path);
            assert!(!edge.is_wildcard);
            assert_eq!(edge.line, parsed.line);
            assert_eq!(edge.column, parsed.column);
            assert_eq!(
                edge.style,
                if parsed.is_angled {
                    DirectiveStyle::Angled
                } else {
                    DirectiveStyle::Quoted
                }
            );
        }
    }

    fn fixture_basic_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/basic")
            .canonicalize()
            .expect("tests/fixtures/basic must exist")
    }

    #[test]
    fn resolve_target_reproduces_resolve_directive_on_basic_fixture() {
        // Parity-table unit test (plan §6.2 `lang-cpp::resolver`): the full
        // both-fixtures parity suite lands in M18.11; this exercises every
        // directive the `basic` fixture sources and headers actually contain.
        let root = fixture_basic_root();
        let files = ariadne_repo::discover_repository_files(&root).unwrap();
        let discovery = ariadne_repo::build_discovery_result(root.clone(), files);
        let catalog = FileCatalog::from_records(&discovery.files);
        let ctx = res_ctx(&root, vec![PathBuf::from("include")]);

        let mut context = CompilationContext::new("parity", "clang++");
        context.include_dirs = vec![PathBuf::from("include")];

        let legacy = IncludeResolver::new(&root)
            .with_repo_discovery(&discovery)
            .with_context(&context);
        let plugin = CppDependencyResolver::new();

        let analyzable = discovery
            .files
            .iter()
            .filter(|f| f.kind.is_analyzable())
            .collect::<Vec<_>>();
        assert!(!analyzable.is_empty(), "fixture must contain sources");

        let mut checked = 0usize;
        for record in &analyzable {
            let content = std::fs::read_to_string(record.canonical_path.clone()).unwrap();
            let directives = parse_include_directives(&content);
            for directive in &directives {
                let legacy_edge = legacy.resolve_directive(record.id, &record.path, directive);
                let edge = RawDepEdge {
                    source_file: record.id,
                    source_rel_path: record.path.clone(),
                    raw_target: directive.raw_path.clone(),
                    is_wildcard: false,
                    style: if directive.is_angled {
                        DirectiveStyle::Angled
                    } else {
                        DirectiveStyle::Quoted
                    },
                    line: directive.line,
                    column: directive.column,
                };
                let outcome = plugin.resolve_target(&edge, &catalog, &ctx);
                match legacy_edge.destination {
                    // Every destination id (Local, System, and External /
                    // ThirdParty) maps to Resolved(same id): the M18.2
                    // External-drop divergence is closed.
                    Some(dest) => {
                        assert_eq!(
                            outcome,
                            ResolveOutcome::Resolved(dest),
                            "{}: {}",
                            record.path.display(),
                            directive.raw_path
                        );
                    }
                    None => {
                        assert_eq!(outcome, ResolveOutcome::Unresolved);
                    }
                }
                checked += 1;
            }
        }
        assert!(checked >= 3, "fixture directives were checked: {checked}");
    }
}
