//! Inter-file dependency resolution trait for language plugins.

use ariadne_lang_api::catalog::{normalize_relative_path, FileCatalog, ResolveOutcome};
use ariadne_lang_api::types::IncludeKind;
use ariadne_repo::FileId;
use std::path::{Path, PathBuf};

/// First on-disk dependency candidate for a directive the [`FileCatalog`]
/// could not map, yielded in the legacy `collect_include_candidates` order:
/// the includer directory first for quoted spellings (angled directives never
/// borrow it), then every caller-supplied search path — an absolute search
/// path under `repo_root` reduced to repository-relative form exactly like the
/// transitional `IncludeResolver` did. Returns the first candidate `path_exists`
/// accepts, as the synthetic `(FileId, path)` pair the legacy resolver's
/// filesystem fallback emitted for a header absent from discovery.
///
/// Shared by both plugin closure walks (`graph::builder`'s on-disk fallback and
/// `scheduler::tu_store`'s directive walker) so they agree with each other and
/// with the transitional `IncludeResolver`. The fallback is style-agnostic:
/// an angled `<hdr>` reachable only through an `-I`/system dir (out-of-root
/// SDK/vcpkg include paths) resolves here, matching the pre-M18.3 walk that fed
/// those files' symbols and hashes into shards. `path_exists` keeps the
/// `DependencyResolver` trait boundary filesystem-free — callers pass
/// `|p| p.exists()`.
pub fn first_ondisk_include_candidate(
    raw_target: &str,
    style: DirectiveStyle,
    source_rel_path: &Path,
    search_paths: &[PathBuf],
    repo_root: &Path,
    path_exists: &dyn Fn(&Path) -> bool,
) -> Option<(FileId, PathBuf)> {
    let include_raw = Path::new(raw_target);
    let source_dir = source_rel_path.parent().unwrap_or_else(|| Path::new(""));
    let check = |candidate: PathBuf| -> Option<(FileId, PathBuf)> {
        let normalized = normalize_relative_path(&candidate);
        if path_exists(&repo_root.join(&normalized)) {
            let rel_str = normalized.to_string_lossy().replace('\\', "/");
            Some((FileId::from_relative_path(&rel_str), normalized))
        } else {
            None
        }
    };

    if !matches!(style, DirectiveStyle::Angled) {
        if let Some(hit) = check(source_dir.join(include_raw)) {
            return Some(hit);
        }
    }
    for search_dir in search_paths {
        let base = if search_dir.is_absolute() {
            search_dir
                .strip_prefix(repo_root)
                .unwrap_or(search_dir.as_path())
        } else {
            search_dir.as_path()
        };
        if let Some(hit) = check(base.join(include_raw)) {
            return Some(hit);
        }
    }
    None
}

/// How a dependency directive is *written* in source, in language-neutral
/// vocabulary (plan §7 rule R2). The spelling drives only the artifact
/// classification (which boundary the edge documents), never resolution
/// itself — plugins decide candidate search per style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DirectiveStyle {
    /// Explicitly written as a local/relative directive (C++ `#include "x"`).
    Quoted,
    /// Explicitly written as an external/system-boundary directive
    /// (C++ `#include <x>`).
    Angled,
    /// The language has no local/external spelling distinction.
    #[default]
    Unspecified,
}

/// A single raw dependency directive parsed from source text, before any
/// path resolution to a concrete [`FileId`] is attempted.
///
/// For C++ this represents a `#include` directive; for Scala an `import`
/// statement; for other languages the equivalent concept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawDepEdge {
    /// The file containing the directive.
    pub source_file: FileId,
    /// Repository-relative path of the includer, recorded by the parser
    /// (`resolve_edges` receives it). Additive in M18.5: `resolve_target`
    /// is a separate trait call, so a stateless plugin resolver needs the
    /// includer directory carried on the edge to reproduce source-relative
    /// search.
    pub source_rel_path: PathBuf,
    /// The raw target string as it appears in source (e.g. `"util/hash.h"` or
    /// `"com.example.models.User"`).
    pub raw_target: String,
    /// `true` for wildcard/glob imports (`import com.example._`,
    /// `#include <*>`).
    pub is_wildcard: bool,
    /// Spelling of the directive (M18.5, plan §7 R2): lets the graph layer
    /// reproduce the artifact's local-vs-system classification without
    /// naming a directive syntax (angle-style reconciliation, plan M18.5
    /// checklist item 3).
    pub style: DirectiveStyle,
    /// 1-based source line of the directive.
    pub line: u32,
    /// 1-based source column of the directive's target start (M18.5
    /// additive: completes the artifact's source-location record).
    pub column: u32,
}

/// Minimal context passed to a [`DependencyResolver`] during edge resolution.
#[derive(Debug)]
pub struct ResolutionContext<'a> {
    /// Absolute path to the repository root.
    pub repo_root: &'a Path,
    /// Language-specific extra search paths for the file being resolved
    /// (M18.5 additive; C++: the compilation context's include + system
    /// include dirs, Scala: source roots). A stateless plugin resolver
    /// receives them instead of holding repository compilation state, so
    /// `LanguagePlugin::resolver()` can be repository-independent.
    pub search_paths: Vec<PathBuf>,
}

impl<'a> Default for ResolutionContext<'a> {
    fn default() -> Self {
        Self {
            repo_root: Path::new("."),
            search_paths: Vec::new(),
        }
    }
}

/// Parses dependency directives from source text and emits [`RawDepEdge`]s.
///
/// Resolution to concrete [`FileId`]s is intentionally left to the graph
/// layer so that this trait stays pure and testable without a live filesystem.
pub trait DependencyResolver: Send + Sync {
    /// Return all dependency edges found in `source`.
    fn resolve_edges(
        &self,
        file_id: FileId,
        rel_path: &Path,
        source: &str,
        ctx: &ResolutionContext<'_>,
    ) -> Vec<RawDepEdge>;

    /// Maps a raw directive to a repository file. Default: unresolved.
    /// C++: include-dir + basename search. Scala: package → directory mapping.
    fn resolve_target(
        &self,
        edge: &RawDepEdge,
        catalog: &FileCatalog,
        ctx: &ResolutionContext<'_>,
    ) -> ResolveOutcome {
        let _ = (edge, catalog, ctx);
        ResolveOutcome::Unresolved
    }

    /// Edge kind this plugin's resolved directives should carry (plan M20
    /// D10). `None` (the default) keeps the graph layer's own destination
    /// classification; a plugin whose directives are a fixed relation in its
    /// language (an include-graph edge kind, not a discovered ownership
    /// property) overrides with `Some(kind)`. Consulted for resolved and
    /// wildcard outcomes only — the on-disk fallback stays spelling-derived.
    fn directive_edge_kind(&self) -> Option<IncludeKind> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_dep_edge_fields() {
        let file_id = FileId::from_relative_path("src/main.scala");
        let edge = RawDepEdge {
            source_file: file_id,
            source_rel_path: PathBuf::from("src/main.scala"),
            raw_target: "com.example.models._".into(),
            is_wildcard: true,
            style: DirectiveStyle::Unspecified,
            line: 3,
            column: 8,
        };
        assert!(edge.is_wildcard);
        assert_eq!(edge.line, 3);
        assert_eq!(edge.style, DirectiveStyle::Unspecified);
    }

    struct ParseOnlyResolver;
    impl DependencyResolver for ParseOnlyResolver {
        fn resolve_edges(
            &self,
            _fid: FileId,
            _p: &Path,
            _src: &str,
            _ctx: &ResolutionContext<'_>,
        ) -> Vec<RawDepEdge> {
            vec![]
        }
    }

    #[test]
    fn default_resolve_target_is_unresolved() {
        let ctx = ResolutionContext {
            repo_root: Path::new("."),
            ..ResolutionContext::default()
        };
        let edge = RawDepEdge {
            source_file: FileId::from_relative_path("src/a.cpp"),
            source_rel_path: PathBuf::from("src/a.cpp"),
            raw_target: "b.h".into(),
            is_wildcard: false,
            style: DirectiveStyle::Unspecified,
            line: 1,
            column: 1,
        };
        assert_eq!(
            ParseOnlyResolver.resolve_target(&edge, &FileCatalog::default(), &ctx),
            ResolveOutcome::Unresolved
        );
    }

    // ── first_ondisk_include_candidate: shared legacy search-order contract ──

    fn candidate_order(style: DirectiveStyle, search_paths: &[PathBuf]) -> Vec<PathBuf> {
        // Faked filesystem: the predicate records every absolute candidate the
        // helper presents and always misses, so the recorded list is exactly the
        // candidate ORDER the legacy `collect_include_candidates` must produce.
        let root = Path::new("/repo");
        let order = std::cell::RefCell::new(Vec::new());
        let miss = |path: &Path| -> bool {
            order.borrow_mut().push(path.to_path_buf());
            false
        };
        let hit = first_ondisk_include_candidate(
            "hdr.h",
            style,
            Path::new("src/main.cpp"),
            search_paths,
            root,
            &miss,
        );
        assert!(hit.is_none(), "all candidates miss");
        order.into_inner()
    }

    #[test]
    fn candidate_order_is_includer_dir_then_search_paths_for_quoted() {
        let order = candidate_order(
            DirectiveStyle::Quoted,
            &[PathBuf::from("inc"), PathBuf::from("sys")],
        );
        assert_eq!(
            order,
            vec![
                PathBuf::from("/repo/src/hdr.h"),
                PathBuf::from("/repo/inc/hdr.h"),
                PathBuf::from("/repo/sys/hdr.h"),
            ],
            "quoted: includer dir first, then user then system include dirs"
        );
    }

    #[test]
    fn candidate_order_skips_includer_dir_for_angled_but_keeps_search_paths() {
        // Finding 1 core: the angled spelling does not borrow the includer
        // directory, but the search paths are STILL searched (style-agnostic),
        // so an angled `<hdr>` under `-I` is a live candidate. (The in-root
        // reduction and out-of-root-verbatim handling of absolute search dirs
        // are locked by the real-root `tu_store_plugin_parity` tests.)
        let order = candidate_order(DirectiveStyle::Angled, &[PathBuf::from("inc")]);
        assert_eq!(order, vec![PathBuf::from("/repo/inc/hdr.h")]);
    }

    #[test]
    fn pin_unspecified_style_behaves_quoted_includer_dir_first() {
        let order = candidate_order(DirectiveStyle::Unspecified, &[PathBuf::from("inc")]);
        assert_eq!(
            order,
            vec![PathBuf::from("/repo/src/hdr.h"), PathBuf::from("/repo/inc/hdr.h")],
            "only Angled skips the includer directory: the default Unspecified \
             style behaves Quoted, includer dir first (Scala imports have no \
             local/external spelling and still get source-relative search)"
        );
    }

    #[test]
    fn pin_absolute_search_path_reduces_in_root_and_keeps_out_of_root_verbatim() {
        let order = candidate_order(
            DirectiveStyle::Quoted,
            &[PathBuf::from("/repo/inc"), PathBuf::from("/opt/sdk")],
        );
        assert_eq!(
            order,
            vec![
                PathBuf::from("/repo/src/hdr.h"),
                PathBuf::from("/repo/inc/hdr.h"),
                PathBuf::from("/opt/sdk/hdr.h"),
            ],
            "an absolute search dir under repo_root is reduced to repository-\
             relative form (strip_prefix), one outside is used verbatim - the \
             join of an absolute normalized candidate replaces the root, so SDK \
             dirs stay reachable"
        );
    }

    #[test]
    fn pin_neutral_directive_edge_kind_default_is_none() {
        assert_eq!(
            ParseOnlyResolver.directive_edge_kind(),
            None,
            "plan M20 D10: the default keeps the graph layer's own destination \
             classification - only a plugin whose directives name a fixed \
             relation overrides (Scala does, C++ keeps the default)"
        );
    }
}
