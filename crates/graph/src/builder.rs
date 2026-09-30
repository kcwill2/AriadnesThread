use anyhow::Result;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{IncludeClosure, IncludeGraph, IncludeReport};
use crate::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_build::BuildDiscoverer;
use ariadne_lang::{
    first_ondisk_include_candidate, DirectiveStyle, FileCatalog, LanguageRegistry, RawDepEdge,
    ResolutionContext, ResolveOutcome,
};
use ariadne_repo::{FileId, FileOwnership, Repository, SourceLanguage};

/// Transitional registry-less entry point (plan M18.5): identical signature
/// and output to the pre-M18 BFS, now delegating through the registry-driven
/// [`build_dependency_graph`] with the same C++ capability set the CLI
/// boundary registers via `ariadne_cli::registry::default_registry`.
/// TODO(M18.11): remove with the `#[deprecated]` shims once every caller
/// passes a registry explicitly — building a plugin set here is language
/// injection inside `graph`, grandfathered only for this milestone.
pub fn build_include_graph(
    repo_path: &Path,
) -> Result<(
    Repository,
    IncludeGraph,
    HashMap<FileId, IncludeClosure>,
    IncludeReport,
)> {
    let registry = transitional_cpp_registry();
    let repository = Repository::locate(repo_path)?;
    let graph = dependency_graph(&repository, &registry)?;
    let closures = graph.compute_closures();
    let report = graph.generate_report();
    Ok((repository, graph, closures, report))
}

/// Registry-driven dependency graph (plan M18.5): the BFS walks each file's
/// directives through the [`ariadne_lang::DependencyResolver`] of the plugin
/// the registry detects for that file, with per-unit shaping and search-path
/// context supplied by the registry as well. With no plugin registered the
/// walk honestly yields an edgeless graph (no units to seed it) — never a
/// silently-C++-parsed one (§3.3 fact 3). C++ output is byte-identical to
/// the transitional [`build_include_graph`] (locked by
/// `m18_graph_parity_test`).
pub fn build_dependency_graph(
    repo_path: &Path,
    registry: &LanguageRegistry,
) -> Result<IncludeGraph> {
    let repository = Repository::locate(repo_path)?;
    dependency_graph(&repository, registry)
}

/// The one grandfathered `lang-cpp` edge in `graph` production code: the
/// registry-less [`build_include_graph`] entry point needs *a* plugin set,
/// and today's C++-only callers expect the C++ one.
/// TODO(M18.11): delete with the transitional shims.
fn transitional_cpp_registry() -> LanguageRegistry {
    LanguageRegistry::new().register(Box::new(ariadne_lang_cpp::CppPlugin::new()))
}

/// The language-neutral BFS: register every discovered file as a node, seed
/// a FIFO scan from the registry-shaped translation units, and expand each
/// scanned file's directives through its owning plugin's resolver.
fn dependency_graph(repository: &Repository, registry: &LanguageRegistry) -> Result<IncludeGraph> {
    let repo_discovery = repository.discover()?;
    let build_result = BuildDiscoverer::new(&repository.root)
        .with_repo_discovery(&repo_discovery)
        .with_unit_shaper(registry.unit_shaper_for())
        .discover()?;

    let mut graph = IncludeGraph::new();
    for file_record in &repo_discovery.files {
        graph.register_file(file_record.id, file_record.path.clone());
    }
    let catalog = FileCatalog::from_records(&repo_discovery.files);

    let mut scanned_files: HashSet<FileId> = HashSet::new();
    let mut scan_queue: VecDeque<(FileId, PathBuf, Option<String>)> = VecDeque::new();
    for tu in &build_result.translation_units {
        scan_queue.push_back((
            tu.file_id,
            tu.source_path.clone(),
            Some(tu.context_name.clone()),
        ));
    }

    while let Some((file_id, rel_path, ctx_name_opt)) = scan_queue.pop_front() {
        if !scanned_files.insert(file_id) {
            continue;
        }
        let content = match fs::read_to_string(repository.root.join(&rel_path)) {
            Ok(content) => content,
            Err(_) => continue,
        };
        let target_ctx = ctx_name_opt
            .as_ref()
            .and_then(|name| build_result.contexts.iter().find(|c| &c.name == name))
            .or_else(|| build_result.contexts.first());
        // A file no registered plugin claims has no resolver to parse it
        // with: it contributes no outgoing edges (never parsed as some other
        // language's source), mirroring the index stage's §3.3-fact-3 rule.
        let Some(plugin_id) = registry.detect_language(&rel_path) else {
            continue;
        };
        let Some(plugin) = registry.plugin_for_id(plugin_id) else {
            continue;
        };

        let resolution = resolution_context(&repository.root, target_ctx);
        let context_name = target_ctx
            .map(|ctx| ctx.name.as_str())
            .unwrap_or("Default");
        let raw_edges =
            plugin
                .resolver()
                .resolve_edges(file_id, &rel_path, &content, &resolution);

        for raw in raw_edges {
            let outcome = plugin.resolver().resolve_target(&raw, &catalog, &resolution);
            for edge in edge_from_outcome(&raw, outcome, &catalog, &resolution, &repository.root, plugin_id, context_name) {
                if let Some(dest_id) = edge.destination {
                    if !scanned_files.contains(&dest_id) {
                        if let Some(dest_path) = graph.file_paths.get(&dest_id).cloned() {
                            scan_queue.push_back((dest_id, dest_path, ctx_name_opt.clone()));
                        }
                    }
                }
                graph.add_edge(edge);
            }
        }
    }

    Ok(graph)
}

/// Per-file search context for the stateless plugin resolver (plan M18.5):
/// the target compilation context's include dirs followed by its system
/// include dirs — the order the legacy `collect_include_candidates` used.
fn resolution_context<'a>(
    repo_root: &'a Path,
    target_ctx: Option<&ariadne_build::CompilationContext>,
) -> ResolutionContext<'a> {
    let search_paths = target_ctx
        .map(|ctx| {
            ctx.include_dirs
                .iter()
                .chain(ctx.system_include_dirs.iter())
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    ResolutionContext {
        repo_root,
        search_paths,
    }
}

/// Turns one resolved directive outcome into the graph edges the legacy BFS
/// produced (M18.5 divergence reconciliation, closing the M18.2 review):
/// `Resolved`/`Wildcard` destinations travel with their catalog ownership,
/// which classifies the edge kind (`ThirdParty` → `External` is an edge
/// property, not an outcome that hides the destination); `External` and
/// `Unresolved` run the legacy filesystem fallback (quoted directives may
/// land on disk outside discovery), and a still-missing target keeps the
/// legacy unresolved edge shape with the spelling-derived kind.
fn edge_from_outcome(
    raw: &RawDepEdge,
    outcome: ResolveOutcome,
    catalog: &FileCatalog,
    resolution: &ResolutionContext<'_>,
    repo_root: &Path,
    language: SourceLanguage,
    context_name: &str,
) -> Vec<IncludeEdge> {
    match outcome {
        ResolveOutcome::Resolved(id) => vec![resolved_edge(raw, catalog, id, language, context_name)],
        ResolveOutcome::Wildcard(ids) => ids
            .into_iter()
            .map(|id| resolved_edge(raw, catalog, id, language, context_name))
            .collect(),
        ResolveOutcome::External | ResolveOutcome::Unresolved => {
            match on_disk_fallback(raw, resolution, repo_root) {
                Some((fallback_id, fallback_path)) => vec![include_edge(
                    raw,
                    language,
                    context_name,
                    Some(fallback_id),
                    style_kind(raw.style),
                    Some(fallback_path),
                )],
                None => vec![include_edge(
                    raw,
                    language,
                    context_name,
                    None,
                    style_kind(raw.style),
                    None,
                )],
            }
        }
    }
}

/// One edge to a catalog-resolved destination: kind from the record's
/// ownership (the legacy resolver's discovery classification), path from the
/// record (the legacy `resolved_path`).
fn resolved_edge(
    raw: &RawDepEdge,
    catalog: &FileCatalog,
    id: FileId,
    language: SourceLanguage,
    context_name: &str,
) -> IncludeEdge {
    let kind = catalog
        .record(id)
        .map(|record| ownership_kind(record.ownership))
        .unwrap_or_else(|| style_kind(raw.style));
    let resolved_path = catalog.rel_path(id).map(PathBuf::from);
    include_edge(raw, language, context_name, Some(id), kind, resolved_path)
}

/// The legacy resolver's on-disk fallback (shared with `scheduler::tu_store`
/// via [`first_ondisk_include_candidate`], which reproduces the legacy
/// `collect_include_candidates` candidate order and synthetic-id shape):
/// includer directory first for non-angled spellings, then the caller-supplied
/// search paths (absolute paths reduced to repo-relative when under the root).
/// A candidate that exists on disk but is absent from discovery gets a
/// synthetic [`FileId`] and is not queued for scanning (unregistered ids never
/// enter `file_paths`).
fn on_disk_fallback(
    raw: &RawDepEdge,
    resolution: &ResolutionContext<'_>,
    repo_root: &Path,
) -> Option<(FileId, PathBuf)> {
    first_ondisk_include_candidate(
        &raw.raw_target,
        raw.style,
        &raw.source_rel_path,
        &resolution.search_paths,
        repo_root,
        &|path: &Path| path.exists(),
    )
}

/// Edge kind from the spelling the directive was written with: angled
/// directives document a system-boundary include, quoted ones a local one.
fn style_kind(style: DirectiveStyle) -> IncludeKind {
    match style {
        DirectiveStyle::Angled => IncludeKind::System,
        DirectiveStyle::Quoted | DirectiveStyle::Unspecified => IncludeKind::Local,
    }
}

/// Edge kind from catalog ownership for a destination found in discovery —
/// the legacy resolver's classification, moved to the graph layer so the
/// neutral outcome carries the destination (see [`edge_from_outcome`]).
fn ownership_kind(ownership: FileOwnership) -> IncludeKind {
    match ownership {
        FileOwnership::ThirdParty => IncludeKind::External,
        FileOwnership::Toolchain => IncludeKind::System,
        _ => IncludeKind::Local,
    }
}

fn include_edge(
    raw: &RawDepEdge,
    language: SourceLanguage,
    context_name: &str,
    destination: Option<FileId>,
    kind: IncludeKind,
    resolved_path: Option<PathBuf>,
) -> IncludeEdge {
    IncludeEdge {
        source: raw.source_file,
        destination,
        context_name: context_name.to_string(),
        source_location: SourceLocation::new(raw.line, raw.column),
        kind,
        raw_path: raw.raw_target.clone(),
        resolved_path,
        language,
    }
}
