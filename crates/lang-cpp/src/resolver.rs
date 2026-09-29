//! C++ include search-path resolver (plan M18.1: relocated verbatim from
//! `crates/graph/src/resolver.rs`).
//!
//! Two surfaces live here:
//!
//! * [`IncludeResolver`] — the transitional, byte-identical public API kept so
//!   existing call sites compile unchanged through the `#[deprecated]`
//!   `ariadne-graph` re-export (removed in M18.11).
//! * [`CppDependencyResolver`] — the plugin implementation of
//!   `ariadne_lang::DependencyResolver` (`resolve_edges` = parse,
//!   `resolve_target` = the same include-directory search logic adapted to
//!   [`FileCatalog`]/[`ResolveOutcome`] per plan §3.3.
//!
//! Mapping decisions for `resolve_target` (vs today's `resolve_directive`):
//! * candidate search order is identical (source dir for quoted directives,
//!   then context user/system include dirs);
//! * a repository-record match owned by `Repository`/`Toolchain` maps to
//!   `Resolved(FileId)` — today's `Local` **and** `System` edges both carry a
//!   destination `FileId`, and `ResolveOutcome` has no "system" flavour;
//! * a `ThirdParty` match maps to `External` (the out-of-repo boundary, plan
//!   §3.4); today the same edge additionally carries the destination id — the
//!   neutral outcome intentionally drops it (graph builder adoption is
//!   M18.5's job);
//! * no match maps to `Unresolved` (today: `destination: None`). The on-disk
//!   fallback of the legacy resolver needs the *including file's* directory,
//!   which `RawDepEdge` does not carry; closure-merge callers apply their own
//!   source-relative fallback (see `scheduler::tu_store`), so the trait stays
//!   filesystem-free like its default implementation.
//! * `RawDepEdge` does not model angle-vs-quote style, so a *quoted*-style
//!   candidate search is applied; an angled directive could therefore only
//!   diverge where a same-named file sits next to the includer (none in any
//!   fixture). Widening `RawDepEdge` is M18.5 follow-up material.

use crate::directive_parser::{parse_include_directives, RawIncludeDirective};
use ariadne_build::CompilationContext;
use ariadne_lang::{DependencyResolver, FileCatalog, RawDepEdge, ResolutionContext, ResolveOutcome};
use ariadne_lang_api::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_repo::{FileId, FileOwnership, FileRecord, RepositoryDiscoveryResult};
use std::path::{Path, PathBuf};

/// Normalizes a path by resolving `.` and `..` relative components without requiring existing files.
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                match components.last() {
                    Some(&std::path::Component::ParentDir) | None => {
                        components.push(component);
                    }
                    Some(&std::path::Component::RootDir)
                    | Some(&std::path::Component::Prefix(_)) => {
                        // Cannot resolve `..` above root, so do nothing.
                    }
                    Some(&std::path::Component::Normal(_)) => {
                        components.pop();
                    }
                    _ => {}
                }
            }
            c => components.push(c),
        }
    }
    let mut normalized = PathBuf::new();
    for c in components {
        normalized.push(c.as_os_str());
    }
    normalized
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

/// Plugin-side `DependencyResolver` for C++ (plan M18.1): `resolve_edges`
/// parses `#include` directives into [`RawDepEdge`]s and `resolve_target`
/// applies the same include-directory search logic through
/// [`FileCatalog`]/[`ResolveOutcome`].
#[derive(Clone)]
pub struct CppDependencyResolver<'a> {
    repo_root: &'a Path,
    repo_discovery: Option<&'a RepositoryDiscoveryResult>,
    context: Option<&'a CompilationContext>,
}

impl<'a> CppDependencyResolver<'a> {
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

    /// Directory of the includer, derived from the resolver's own discovery
    /// (`RawDepEdge` only carries the source [`FileId`]); empty when the
    /// source is unknown to discovery.
    fn source_dir_of(&self, source_file: &FileId) -> PathBuf {
        self.repo_discovery
            .and_then(|disc| disc.files.iter().find(|f| f.id == *source_file))
            .and_then(|f| f.path.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_default()
    }

    fn candidates_for(&self, source_file: &FileId, raw_target: &str) -> Vec<PathBuf> {
        // RawDepEdge models no angle/quote style: search quoted-style (source
        // dir first). See the module docs for the divergence analysis.
        collect_include_candidates(
            self.repo_root,
            self.context,
            &self.source_dir_of(source_file),
            Path::new(raw_target),
            false,
        )
    }
}

impl<'a> DependencyResolver for CppDependencyResolver<'a> {
    fn resolve_edges(
        &self,
        file_id: FileId,
        _rel_path: &Path,
        source: &str,
        _ctx: &ResolutionContext<'_>,
    ) -> Vec<RawDepEdge> {
        parse_include_directives(source)
            .into_iter()
            .map(|dir| RawDepEdge {
                source_file: file_id,
                raw_target: dir.raw_path,
                is_wildcard: false,
                line: dir.line,
            })
            .collect()
    }

    fn resolve_target(
        &self,
        edge: &RawDepEdge,
        catalog: &FileCatalog,
        _ctx: &ResolutionContext<'_>,
    ) -> ResolveOutcome {
        let candidates = self.candidates_for(&edge.source_file, &edge.raw_target);

        // Discovery (when wired) keeps the legacy path/canonical match and
        // exposes the ownership needed for the External boundary decision.
        if let Some(disc) = self.repo_discovery {
            if let Some(record) = find_in_discovery(disc, &candidates) {
                return match record.ownership {
                    FileOwnership::ThirdParty => ResolveOutcome::External,
                    _ => ResolveOutcome::Resolved(record.id),
                };
            }
        } else {
            // Catalog-only configuration: first candidate present in the
            // catalog wins (no ownership data available ⇒ always Resolved).
            for cand in &candidates {
                if let Some(id) = catalog.by_rel_path(&cand.to_string_lossy()) {
                    return ResolveOutcome::Resolved(id);
                }
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
        // have TU sources discovered, and `resolve_target` recovers the
        // includer directory (which `RawDepEdge` does not carry) from it.
        files.push(mock_file(
            root,
            "src/main.cpp",
            FileOwnership::Repository,
        ));
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

    fn res_ctx(root: &Path) -> ResolutionContext<'_> {
        ResolutionContext { repo_root: root }
    }

    fn raw_edge(rel_source: &str, raw_target: &str) -> RawDepEdge {
        RawDepEdge {
            source_file: FileId::from_relative_path(rel_source),
            raw_target: raw_target.into(),
            is_wildcard: false,
            line: 1,
        }
    }

    #[test]
    fn resolve_target_parity_table_matches_resolve_directive() {
        let (temp, discovery, context) = given_resolver_fixture();
        let root = temp.path();
        let catalog = FileCatalog::from_records(&discovery.files);
        let ctx = res_ctx(root);

        let legacy = IncludeResolver::new(root)
            .with_repo_discovery(&discovery)
            .with_context(&context);
        let plugin = CppDependencyResolver::new(root)
            .with_repo_discovery(&discovery)
            .with_context(&context);
        let source = Path::new("src/main.cpp");

        // Repository match: Local + destination id == Resolved(same id)
        let helper = FileId::from_relative_path("common/helper.h");
        let edge = raw_edge("src/main.cpp", "../common/helper.h");
        let legacy_edge = legacy.resolve_directive(edge.source_file, source, &include_directive("../common/helper.h", false, 1));
        assert_eq!(legacy_edge.kind, IncludeKind::Local);
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Resolved(helper)
        );

        // Toolchain match: System + destination id == Resolved(same id)
        let sys = FileId::from_relative_path("usr/include/system.h");
        let edge = raw_edge("src/main.cpp", "system.h");
        let legacy_edge = legacy.resolve_directive(edge.source_file, source, &include_directive("system.h", true, 1));
        assert_eq!(legacy_edge.kind, IncludeKind::System);
        assert_eq!(legacy_edge.destination, Some(sys));
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Resolved(sys)
        );

        // ThirdParty match: kind External + destination id == External (no id)
        let edge = raw_edge("src/main.cpp", "lib.h");
        let legacy_edge = legacy.resolve_directive(edge.source_file, source, &include_directive("lib.h", true, 1));
        assert_eq!(legacy_edge.kind, IncludeKind::External);
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::External
        );

        // Miss: destination None == Unresolved
        let edge = raw_edge("src/main.cpp", "nonexistent.h");
        let legacy_edge = legacy.resolve_directive(edge.source_file, source, &include_directive("nonexistent.h", false, 1));
        assert_eq!(legacy_edge.destination, None);
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Unresolved
        );

        // On-disk fallback (sibling.h): legacy resolves via filesystem; the
        // trait path stays filesystem-free (caller-side fallback documented).
        let edge = raw_edge("src/main.cpp", "sibling.h");
        let legacy_edge = legacy.resolve_directive(edge.source_file, source, &include_directive("sibling.h", false, 1));
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
    fn resolve_target_catalog_only_config_resolves_without_discovery() {
        let (temp, discovery, _fixture_context) = given_resolver_fixture();
        let root = temp.path();
        let catalog = FileCatalog::from_records(&discovery.files);
        let ctx = res_ctx(root);

        // No repo_discovery on the resolver: the FileCatalog alone answers the
        // candidate paths. With the source directory unknown (RawDepEdge
        // carries no path and there is no discovery to recover it from), a
        // bare `"helper.h"` can only resolve through an include dir, mirroring
        // the legacy search that runs the compilation-context include dirs.
        let mut context = CompilationContext::new("catalog-only", "clang");
        context.include_dirs = vec![PathBuf::from("common"), PathBuf::from("third_party/lib")];
        let plugin = CppDependencyResolver::new(root).with_context(&context);

        let edge = raw_edge("src/main.cpp", "helper.h");
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Resolved(FileId::from_relative_path("common/helper.h"))
        );

        // Catalog-only never sees ownership, so a third-party file reached via
        // its include dir still resolves (never External) — the documented
        // difference from the discovery-backed path.
        let edge = raw_edge("src/main.cpp", "lib.h");
        assert_eq!(
            plugin.resolve_target(&edge, &catalog, &ctx),
            ResolveOutcome::Resolved(FileId::from_relative_path("third_party/lib/lib.h"))
        );
    }

    #[test]
    fn resolve_edges_maps_parse_output_to_raw_dep_edges() {
        let temp = tempfile::tempdir().unwrap();
        let plugin = CppDependencyResolver::new(temp.path());
        let ctx = ResolutionContext {
            repo_root: temp.path(),
        };
        let content = r#"
#include "thing_types.h"
#  include <vector>
#include "heavy.h" // comment
"#;
        let file_id = FileId::from_relative_path("src/thing.cpp");
        let edges = plugin.resolve_edges(
            file_id,
            Path::new("src/thing.cpp"),
            content,
            &ctx,
        );
        let parsed = parse_include_directives(content);
        assert_eq!(edges.len(), parsed.len());
        for (edge, parsed) in edges.iter().zip(&parsed) {
            assert_eq!(edge.source_file, file_id);
            assert_eq!(edge.raw_target, parsed.raw_path);
            assert!(!edge.is_wildcard);
            assert_eq!(edge.line, parsed.line);
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
        let ctx = res_ctx(&root);

        let mut context = CompilationContext::new("parity", "clang++");
        context.include_dirs = vec![PathBuf::from("include")];

        let legacy = IncludeResolver::new(&root)
            .with_repo_discovery(&discovery)
            .with_context(&context);
        let plugin = CppDependencyResolver::new(&root)
            .with_repo_discovery(&discovery)
            .with_context(&context);

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
                let legacy_edge =
                    legacy.resolve_directive(record.id, &record.path, directive);
                let edge = RawDepEdge {
                    source_file: record.id,
                    raw_target: directive.raw_path.clone(),
                    is_wildcard: false,
                    line: directive.line,
                };
                let outcome = plugin.resolve_target(&edge, &catalog, &ctx);
                match (legacy_edge.kind, legacy_edge.destination) {
                    (IncludeKind::External, Some(_)) => {
                        assert_eq!(outcome, ResolveOutcome::External);
                    }
                    (_, Some(dest)) => {
                        assert_eq!(outcome, ResolveOutcome::Resolved(dest));
                    }
                    (_, None) => {
                        assert_eq!(outcome, ResolveOutcome::Unresolved);
                    }
                }
                checked += 1;
            }
        }
        assert!(checked >= 3, "fixture directives were checked: {checked}");
    }
}
