use anyhow::Result;
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use crate::IncludeGraph;
use crate::types::{IncludeEdge, IncludeKind, SourceLocation};
use ariadne_build::BuildDiscoverer;
use ariadne_lang::{
    first_ondisk_include_candidate, DirectiveStyle, FileCatalog, LanguageId, LanguageRegistry,
    RawDepEdge, ResolutionContext, ResolveOutcome, UnitShaper,
};
use ariadne_repo::{FileId, FileOwnership, Repository};

/// Registry-driven dependency graph (plan M18.5): the BFS walks each file's
/// directives through the [`ariadne_lang::DependencyResolver`] of the plugin
/// the registry detects for that file, with per-unit shaping and search-path
/// context supplied by the registry as well. With no plugin registered the
/// walk honestly yields an edgeless graph (no units to seed it) — never a
/// silently-C++-parsed one (§3.3 fact 3). C++ output is byte-identical to
/// the pre-M18 BFS (locked by the vendored oracles in
/// `m18_cpp_plugin_parity_test` and the `m00` include-graph goldens).
///
/// Since plan M22 D22 the seeding shaper is the registry's marker-based
/// `unit_shaper_for_repo` selection against the located repository root
/// (first-wins fallback preserved); a repo carrying another plugin's build
/// markers now shapes through that plugin's shaper without injection.
pub fn build_dependency_graph(
    repo_path: &Path,
    registry: &LanguageRegistry,
) -> Result<IncludeGraph> {
    let repository = Repository::locate(repo_path)?;
    dependency_graph(
        &repository,
        registry,
        registry.unit_shaper_for_repo(&repository.root),
    )
}

/// The registry-driven BFS with an explicitly injected [`UnitShaper`] seeding
/// the scan (plan M20 D14): the hook for callers that hold a shaper the
/// registry's marker-based `unit_shaper_for_repo()` selection would not pick
/// for this repository — the shaper arrives as a neutral `&dyn UnitShaper`,
/// never as a language name. With the shaper marker selection yields, this is
/// exactly [`build_dependency_graph`].
pub fn build_dependency_graph_with_unit_shaper(
    repo_path: &Path,
    registry: &LanguageRegistry,
    shaper: &dyn UnitShaper,
) -> Result<IncludeGraph> {
    let repository = Repository::locate(repo_path)?;
    dependency_graph(&repository, registry, Some(shaper))
}

/// The language-neutral BFS: register every discovered file as a node, seed
/// a FIFO scan from the shaped translation units, and expand each scanned
/// file's directives through its owning plugin's resolver.
fn dependency_graph(
    repository: &Repository,
    registry: &LanguageRegistry,
    unit_shaper: Option<&dyn UnitShaper>,
) -> Result<IncludeGraph> {
    let repo_discovery = repository.discover()?;
    let build_result = BuildDiscoverer::new(&repository.root)
        .with_repo_discovery(&repo_discovery)
        .with_unit_shaper(unit_shaper)
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
        let Some(plugin) = registry.plugin_for_id(plugin_id.clone()) else {
            continue;
        };

        let resolution = resolution_context(&repository.root, target_ctx);
        let context_name = target_ctx
            .map(|ctx| ctx.name.as_str())
            .unwrap_or("Default");
        // The plugin's directive kind (plan M20 D10) is a resolver property,
        // read once per file from the handle the loop already holds.
        let directive_kind = plugin.resolver().directive_edge_kind();
        let raw_edges =
            plugin
                .resolver()
                .resolve_edges(file_id, &rel_path, &content, &resolution);

        for raw in raw_edges {
            let outcome = plugin.resolver().resolve_target(&raw, &catalog, &resolution);
            for edge in edge_from_outcome(&raw, outcome, &catalog, &resolution, &repository.root, plugin_id.clone(), context_name, directive_kind) {
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
/// property, not an outcome that hides the destination), unless the plugin
/// dictates a directive kind; `Unresolved` runs the legacy filesystem
/// fallback (quoted directives may land on disk outside discovery), and a
/// still-missing target keeps the legacy unresolved edge shape with the
/// spelling-derived kind; `External` is an honest out-of-boundary marker:
/// an unresolved edge of kind `External`, never probed on disk (plan M20
/// D10b — `CppDependencyResolver::resolve_target` in `crates/lang-cpp/src/
/// resolver.rs` returns only `Resolved`/`Unresolved`, so C++ never enters
/// this arm).
#[allow(clippy::too_many_arguments)]
fn edge_from_outcome(
    raw: &RawDepEdge,
    outcome: ResolveOutcome,
    catalog: &FileCatalog,
    resolution: &ResolutionContext<'_>,
    repo_root: &Path,
    language: LanguageId,
    context_name: &str,
    directive_kind: Option<IncludeKind>,
) -> Vec<IncludeEdge> {
    match outcome {
        ResolveOutcome::Resolved(id) => {
            vec![resolved_edge(raw, catalog, id, language, context_name, directive_kind)]
        }
        ResolveOutcome::Wildcard(ids) => ids
            .into_iter()
            .map(|id| resolved_edge(raw, catalog, id, language.clone(), context_name, directive_kind))
            .collect(),
        ResolveOutcome::External => vec![include_edge(
            raw,
            language,
            context_name,
            None,
            IncludeKind::External,
            None,
        )],
        ResolveOutcome::Unresolved => match on_disk_fallback(raw, resolution, repo_root) {
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
        },
    }
}

/// One edge to a catalog-resolved destination: kind from the plugin's
/// directive kind when it dictates one (plan M20 D10), else from the
/// record's ownership (the legacy resolver's discovery classification);
/// path from the record (the legacy `resolved_path`).
fn resolved_edge(
    raw: &RawDepEdge,
    catalog: &FileCatalog,
    id: FileId,
    language: LanguageId,
    context_name: &str,
    directive_kind: Option<IncludeKind>,
) -> IncludeEdge {
    let kind = match directive_kind {
        Some(kind) => kind,
        None => catalog
            .record(id)
            .map(|record| ownership_kind(record.ownership))
            .unwrap_or_else(|| style_kind(raw.style)),
    };
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
    language: LanguageId,
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

#[cfg(test)]
mod builder_seam_tests {
    use super::*;
    use ariadne_lang::{
        AggregationKind, ClusteringConstraints, CompilerValidator, CyclePolicy, DependencyResolver,
        DependencyRewriter, GeneratedSourceFile, GuardKind, LanguageCapabilities, LanguageDetector,
        LanguageModeling, LanguagePlugin, RewriteSpec, SemanticAnalyzer, SourceExtractor,
        SourceFileKind, SymbolBody, SymbolId, SymbolPlacement, SymbolRecord, UnitDescriptor,
        UnitKind, UnitShaper,
    };
    use std::collections::HashSet;

    // Language-neutral fixtures for the kind/outcome/shaper seams: a synthetic
    // plugin whose resolver dictates outcomes and (optionally) a directive
    // edge kind, plus a shaper that seeds exactly one unit. Plugin crates are
    // deliberately not referenced — the graph layer must work through trait
    // objects only (plan §7 R2/R3).

    const ENTRY_REL: &str = "src/entry.cpp";
    const LEAF_REL: &str = "src/leaf.cpp";
    const SIBLING_REL: &str = "src/sibling.cpp";
    const OFFSITE_RAW: &str = "vendor/dep";

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum StubOutcome {
        ResolvedLeaf,
        WildcardLeaves,
        OutOfRepo,
        MissedTarget,
    }

    struct SeamResolver {
        outcome: StubOutcome,
        edge_kind: Option<IncludeKind>,
    }

    impl SeamResolver {
        fn new(outcome: StubOutcome, edge_kind: Option<IncludeKind>) -> Self {
            Self { outcome, edge_kind }
        }
    }

    impl DependencyResolver for SeamResolver {
        fn resolve_edges(
            &self,
            file_id: FileId,
            rel_path: &Path,
            _source: &str,
            _ctx: &ResolutionContext<'_>,
        ) -> Vec<RawDepEdge> {
            if rel_path.extension().and_then(|e| e.to_str()) != Some("cpp") {
                return vec![];
            }
            if rel_path.file_name().and_then(|n| n.to_str()) != Some("sibling.cpp")
                && rel_path.file_name().and_then(|n| n.to_str()) != Some("leaf.cpp")
            {
                let raw_target = match self.outcome {
                    StubOutcome::MissedTarget | StubOutcome::OutOfRepo => OFFSITE_RAW,
                    _ => "dep-target",
                };
                return vec![RawDepEdge {
                    source_file: file_id,
                    source_rel_path: rel_path.to_path_buf(),
                    raw_target: raw_target.to_string(),
                    is_wildcard: false,
                    style: DirectiveStyle::Unspecified,
                    line: 1,
                    column: 1,
                }];
            }
            vec![]
        }

        fn resolve_target(
            &self,
            _edge: &RawDepEdge,
            _catalog: &FileCatalog,
            _ctx: &ResolutionContext<'_>,
        ) -> ResolveOutcome {
            match self.outcome {
                StubOutcome::ResolvedLeaf => {
                    ResolveOutcome::Resolved(FileId::from_relative_path(LEAF_REL))
                }
                StubOutcome::WildcardLeaves => ResolveOutcome::Wildcard(vec![
                    FileId::from_relative_path(LEAF_REL),
                    FileId::from_relative_path(SIBLING_REL),
                ]),
                StubOutcome::OutOfRepo => ResolveOutcome::External,
                StubOutcome::MissedTarget => ResolveOutcome::Unresolved,
            }
        }

        fn directive_edge_kind(&self) -> Option<IncludeKind> {
            self.edge_kind
        }
    }

    struct SeamDetector;
    impl LanguageDetector for SeamDetector {
        fn name(&self) -> &str {
            "seam-stub"
        }
        fn detect(&self, path: &Path) -> SourceFileKind {
            match path.extension().and_then(|e| e.to_str()) {
                Some("cpp") => SourceFileKind::Source,
                _ => SourceFileKind::Other,
            }
        }
    }

    struct SeamAnalyzer;
    impl SemanticAnalyzer for SeamAnalyzer {
        fn analyze(&self, file_id: FileId, _source: &str, _ctx: &str) -> ariadne_lang::TuAnalysis {
            ariadne_lang::TuAnalysis {
                tu_file_id: file_id,
                ..Default::default()
            }
        }
    }

    struct SeamConstraints;
    impl ClusteringConstraints for SeamConstraints {
        fn colocated_groups(&self, _symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
            vec![]
        }
        fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> HashSet<SymbolId> {
            HashSet::new()
        }
    }

    struct SeamExtractor;
    impl SourceExtractor for SeamExtractor {
        fn extract(&self, _bodies: &[SymbolBody], _ns: &str, _deps: &[String]) -> GeneratedSourceFile {
            GeneratedSourceFile {
                path: PathBuf::from("out.stub"),
                content: String::new(),
            }
        }
    }

    struct SeamRewriter;
    impl DependencyRewriter for SeamRewriter {
        fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
            source.to_string()
        }
    }

    struct SeamValidator;
    impl CompilerValidator for SeamValidator {
        fn validate_compile(&self, _files: &[PathBuf], _args: &[String]) -> Result<(), String> {
            Ok(())
        }
    }

    struct SeamPlugin {
        resolver: SeamResolver,
    }

    impl LanguagePlugin for SeamPlugin {
        fn detector(&self) -> &dyn LanguageDetector {
            &SeamDetector
        }
        fn analyzer(&self) -> &dyn SemanticAnalyzer {
            &SeamAnalyzer
        }
        fn resolver(&self) -> &dyn DependencyResolver {
            &self.resolver
        }
        fn constraints(&self) -> &dyn ClusteringConstraints {
            &SeamConstraints
        }
        fn extractor(&self) -> &dyn SourceExtractor {
            &SeamExtractor
        }
        fn rewriter(&self) -> &dyn DependencyRewriter {
            &SeamRewriter
        }
        fn validator(&self) -> &dyn CompilerValidator {
            &SeamValidator
        }
fn id(&self) -> ariadne_lang::LanguageId {
            ariadne_lang::LanguageId::cpp()
        }
        fn modeling(&self) -> LanguageModeling {
            LanguageModeling {
                language: ariadne_lang::LanguageId::cpp(),
                unit: UnitKind::TranslationUnit,
                aggregation: AggregationKind::None,
                guard: GuardKind::None,
                symbol_placement: SymbolPlacement::SingleFile,
                aggregation_reexports: false,
                cycle_policy: CyclePolicy::Reject,
                external_path_markers: vec![],
                // D26 fields: default-path wording, no caveat (the mock
                // makes no wording assertions).
                dependency_vocabulary: ariadne_lang::DependencyVocabulary {
                    directive_label: "Include".into(),
                    directive_passive_verb: "included".into(),
                    aggregation_noun: "header".into(),
                },
                analysis_caveat: None,
            }
        }
        fn capabilities(&self) -> LanguageCapabilities {
            LanguageCapabilities {
                discovery: true,
                dependency_graph: true,
                semantic_index: true,
                load_metric: true,
                planning: false,
                extraction: false,
                rewriting: false,
                validation: false,
                requires_toolchain: false,
            }
        }
        // `shaper()` stays at the None default: the D14 seam test injects the
        // shaper through the builder entry instead.
    }

    struct SeedEntryShaper;
    impl UnitShaper for SeedEntryShaper {
        fn build_markers(&self) -> Vec<&'static str> {
            vec![]
        }
        fn default_compiler(&self) -> &str {
            "stubc"
        }
        fn shape_units(&self, _repo_root: &Path, files: &[ariadne_repo::FileRecord]) -> Vec<UnitDescriptor> {
            files
                .iter()
                .filter(|record| record.path.to_string_lossy().replace('\\', "/") == ENTRY_REL)
                .map(|record| UnitDescriptor {
                    file_id: record.id,
                    language: ariadne_lang::LanguageId::cpp(),
                    unit_kind: UnitKind::TranslationUnit,
                    context_name: "Stub".to_string(),
                    aggregation_target: None,
                    build_spec: vec!["stub".to_string()],
                    defines: vec![],
                    include_dirs: vec![],
                })
                .collect()
        }
    }

    fn repo_with_files(repo_root: &Path) {
        fs::create_dir_all(repo_root.join("src").join("vendor")).expect("src/vendor");
        fs::write(repo_root.join(ENTRY_REL), "seed unit").expect("entry");
        fs::write(repo_root.join(LEAF_REL), "leaf").expect("leaf");
        fs::write(repo_root.join(SIBLING_REL), "sibling").expect("sibling");
        // Exists on disk only (no analyzable extension): reachable by the
        // on-disk fallback through the includer directory, never by discovery.
        fs::write(repo_root.join("src").join(OFFSITE_RAW), "offsite").expect("offsite");
    }

    fn registry_with(outcome: StubOutcome, edge_kind: Option<IncludeKind>) -> LanguageRegistry {
        LanguageRegistry::new().register(Box::new(SeamPlugin {
            resolver: SeamResolver::new(outcome, edge_kind),
        }))
    }

    fn build_with(outcome: StubOutcome, edge_kind: Option<IncludeKind>) -> Result<IncludeGraph> {
        let tmp = tempfile::tempdir().expect("temp repo");
        repo_with_files(tmp.path());
        let registry = registry_with(outcome, edge_kind);
        build_dependency_graph_with_unit_shaper(tmp.path(), &registry, &SeedEntryShaper)
    }

    #[test]
    fn directive_edge_kind_overrides_ownership_classification_on_resolved_edges() {
        let graph = build_with(StubOutcome::ResolvedLeaf, Some(IncludeKind::Import)).expect("build");
        assert_eq!(graph.edges.len(), 1, "the seeded unit yields exactly one edge");
        let edge = &graph.edges[0];
        assert_eq!(
            edge.source,
            FileId::from_relative_path(ENTRY_REL),
            "the seeded unit is the edge's source"
        );
        assert_eq!(edge.destination, Some(FileId::from_relative_path(LEAF_REL)));
        assert_eq!(
            edge.kind,
            IncludeKind::Import,
            "Some(directive kind) replaces the ownership-kind classification"
        );
        assert_eq!(edge.context_name, "Stub", "shaper-provided context travels");
    }

    #[test]
    fn directive_edge_kind_none_keeps_ownership_kind_path() {
        let graph = build_with(StubOutcome::ResolvedLeaf, None).expect("build");
        let edge = &graph.edges[0];
        assert_eq!(
            edge.kind,
            IncludeKind::Local,
            "None keeps the repository-ownership classification exactly"
        );
        assert_eq!(edge.destination, Some(FileId::from_relative_path(LEAF_REL)));
    }

    #[test]
    fn directive_edge_kind_applies_to_wildcard_expansion() {
        let graph = build_with(StubOutcome::WildcardLeaves, Some(IncludeKind::Import)).expect("build");
        assert_eq!(graph.edges.len(), 2, "one edge per wildcard destination");
        assert!(
            graph.edges.iter().all(|edge| edge.kind == IncludeKind::Import),
            "wildcard-expanded edges carry the directive kind too"
        );
    }

    #[test]
    fn external_outcome_never_consults_the_on_disk_fallback() {
        // The raw target maps onto a file that EXISTS on disk one directory
        // below the includer, so the legacy fallback would resolve it; the
        // External arm must emit an unresolved-boundary edge untouched.
        let graph = build_with(StubOutcome::OutOfRepo, None).expect("build");
        assert_eq!(graph.edges.len(), 1);
        let edge = &graph.edges[0];
        assert_eq!(edge.kind, IncludeKind::External);
        assert_eq!(edge.destination, None, "no filesystem probe, no destination");
        assert_eq!(edge.resolved_path, None);
        assert_eq!(edge.raw_path, OFFSITE_RAW);
    }

    #[test]
    fn unresolved_outcome_still_uses_the_on_disk_fallback_unchanged() {
        let graph = build_with(StubOutcome::MissedTarget, None).expect("build");
        assert_eq!(graph.edges.len(), 1);
        let edge = &graph.edges[0];
        assert_eq!(
            edge.kind,
            IncludeKind::Local,
            "the spelling-derived kind stays for the fallback hit"
        );
        assert!(
            edge.destination.is_some(),
            "an existing on-disk candidate still resolves as before"
        );
        assert!(edge
            .resolved_path
            .as_ref()
            .expect("fallback resolved path")
            .ends_with(Path::new("vendor").join("dep")));
    }

    #[test]
    fn default_entry_seeds_only_through_the_registry_shaper() {
        let tmp = tempfile::tempdir().expect("temp repo");
        repo_with_files(tmp.path());
        let registry = registry_with(StubOutcome::ResolvedLeaf, Some(IncludeKind::Import));

        let default_built = build_dependency_graph(tmp.path(), &registry).expect("default build");
        assert!(
            default_built.edges.is_empty(),
            "no shaper in the registry seeds means no units to walk from"
        );

        let injected =
            build_dependency_graph_with_unit_shaper(tmp.path(), &registry, &SeedEntryShaper)
                .expect("injected build");
        assert_eq!(
            injected.edges.len(),
            1,
            "the injected shaper alone seeds the BFS"
        );
        assert_eq!(injected.edges[0].kind, IncludeKind::Import);
    }
}
