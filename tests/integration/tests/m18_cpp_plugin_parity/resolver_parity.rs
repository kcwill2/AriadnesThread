//! Section 1 — resolver parity (plan §5 M18 acceptance, first clause).
//!
//! `CppDependencyResolver::resolve_edges` / `resolve_target` must reproduce
//! the legacy `#include` behavior on both C++ fixtures:
//!
//! 1. the DIRECTIVE set `resolve_edges` emits per fixture file equals a
//!    vendored capture taken at HEAD by running the current
//!    `ariadne_lang_cpp::parse_include_directives` (public, non-deprecated);
//! 2. the full file closure reached through the plugin resolver equals a
//!    vendored copy of the pre-M18 LIFO closure walk
//!    (`git show ceac55f:crates/scheduler/src/tu_store.rs::compute_tu_closure`
//!    — the walk body, vendored verbatim below and driven through the
//!    lang-cpp `IncludeResolver` that stays public after M18.11-B);
//! 3. the `resolve_target` outcome table (through the documented
//!    filesystem-fallback mapping shared with the permanent walks) equals the
//!    legacy resolver's per-directive destination on every fixture directive.

use super::{analyzable_records, isolated_repo, normalized_rel, repo_inputs};
use anyhow::Result;
use ariadne_build::{CompilationContext, TranslationUnit};
use ariadne_index::CURRENT_ANALYSIS_VERSION;
use ariadne_lang::{
    DependencyResolver, DirectiveStyle, FileCatalog, RawDepEdge, ResolutionContext, ResolveOutcome,
};
use ariadne_lang_cpp::{parse_include_directives, CppDependencyResolver, CPP_ANALYSIS_FINGERPRINT};
use ariadne_repo::{FileId, RepositoryDiscoveryResult, SourceLanguage};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// `(rel_path, raw_target, is_angled, line, column)` per directive, captured
/// at ceac55f by running the current `ariadne_lang_cpp::parse_include_directives`
/// over every analyzable fixture file (see the M18.11 capture run; the same
/// parser is re-checked against this table by each test below, so the oracle
/// itself is locked too).
const BASIC_DIRECTIVES: &[(&str, &str, bool, u32, u32)] = &[
    ("include/thing.h", "thing_types.h", false, 3, 10),
    ("include/thing.h", "heavy.h", false, 4, 10),
    ("src/thing.cpp", "thing.h", false, 1, 10),
    ("tests/consumer.cpp", "thing.h", false, 1, 10),
];

/// Same capture for the `expensive` fixture.
const EXPENSIVE_DIRECTIVES: &[(&str, &str, bool, u32, u32)] = &[
    ("include/heavy.h", "iostream", true, 3, 10),
    ("include/heavy.h", "vector", true, 4, 10),
    ("include/heavy.h", "string", true, 5, 10),
    ("include/heavy.h", "map", true, 6, 10),
    ("include/thing.h", "heavy.h", false, 3, 10),
    ("src/consumer1.cpp", "../include/thing.h", false, 2, 10),
    ("src/consumer2.cpp", "../include/thing.h", false, 2, 10),
    ("src/consumer2.cpp", "../include/heavy.h", false, 3, 10),
];

#[test]
fn resolve_edges_matches_vendored_directive_capture_basic() -> Result<()> {
    directive_capture_parity("basic", BASIC_DIRECTIVES)
}

#[test]
fn resolve_edges_matches_vendored_directive_capture_expensive() -> Result<()> {
    directive_capture_parity("expensive", EXPENSIVE_DIRECTIVES)
}

#[test]
fn plugin_closure_matches_vendored_legacy_bfs_basic() -> Result<()> {
    closure_parity("basic")
}

#[test]
fn plugin_closure_matches_vendored_legacy_bfs_expensive() -> Result<()> {
    closure_parity("expensive")
}

#[test]
fn resolve_target_outcome_table_matches_legacy_resolver_basic() -> Result<()> {
    outcome_table_parity("basic")
}

#[test]
fn resolve_target_outcome_table_matches_legacy_resolver_expensive() -> Result<()> {
    outcome_table_parity("expensive")
}

#[test]
fn zz_capture_directives() -> Result<()> {
    for fixture in ["basic", "expensive"] {
        let repo = isolated_repo(fixture, "capture-dir")?;
        let (discovery, _build) = repo_inputs(&repo.root)?;
        for record in analyzable_records(&discovery) {
            let content = fs::read_to_string(record.canonical_path.clone())?;
            for directive in parse_include_directives(&content) {
                println!(
                    "@@DIR\t{}\t{}\t{}\t{}\t{}\t{}",
                    fixture,
                    serde_json::to_string(&normalized_rel(&record.path))?,
                    serde_json::to_string(&directive.raw_path)?,
                    directive.is_angled,
                    directive.line,
                    directive.column
                );
            }
        }
    }
    Ok(())
}

fn directive_capture_parity(
    fixture: &str,
    expected: &[(&str, &str, bool, u32, u32)],
) -> Result<()> {
    let repo = isolated_repo(fixture, "resolver")?;
    let (discovery, _build) = repo_inputs(&repo.root)?;
    let resolver = CppDependencyResolver::new();
    let ctx = ResolutionContext {
        repo_root: repo.root.as_path(),
        search_paths: Vec::new(),
    };
    let mut checked = 0usize;
    for record in analyzable_records(&discovery) {
        let rel = normalized_rel(&record.path);
        let content = fs::read_to_string(record.canonical_path.clone())?;
        let wanted: Vec<&(&str, &str, bool, u32, u32)> =
            expected.iter().filter(|entry| entry.0 == rel).collect();

        // Oracle self-lock: the public lang-cpp parser must still produce the
        // vendored capture (a parser change trips here, not on the plugin).
        let parsed = parse_include_directives(&content);
        assert_eq!(parsed.len(), wanted.len(), "{rel}: directive count drifted");
        for (directive, entry) in parsed.iter().zip(&wanted) {
            let &(want_rel, want_raw, want_angled, want_line, want_column) = *entry;
            assert_eq!(want_rel, rel, "capture rows must be grouped by file");
            assert_eq!(directive.raw_path, want_raw, "{rel}");
            assert_eq!(directive.is_angled, want_angled, "{rel}:{want_raw}");
            assert_eq!(directive.line, want_line, "{rel}:{want_raw}");
            assert_eq!(directive.column, want_column, "{rel}:{want_raw}");
        }

        // The plugin surface must reproduce the vendored capture exactly.
        let edges = resolver.resolve_edges(record.id, &record.path, &content, &ctx);
        assert_eq!(edges.len(), wanted.len(), "{rel}: resolve_edges drifted");
        for (edge, entry) in edges.iter().zip(&wanted) {
            let &(want_rel, want_raw, want_angled, want_line, want_column) = *entry;
            assert_eq!(want_rel, rel, "capture rows must be grouped by file");
            assert_eq!(edge.source_file, record.id, "{rel}");
            assert_eq!(edge.source_rel_path, record.path, "{rel}");
            assert_eq!(edge.raw_target, want_raw, "{rel}");
            assert!(!edge.is_wildcard, "C++ has no wildcard directives");
            assert_eq!(
                matches!(edge.style, DirectiveStyle::Angled),
                want_angled,
                "{rel}:{}",
                edge.raw_target
            );
            assert_eq!(edge.line, want_line, "{rel}:{}", edge.raw_target);
            assert_eq!(edge.column, want_column, "{rel}:{}", edge.raw_target);
        }
        checked += edges.len();
    }
    assert!(checked >= 3, "fixture directives were checked: {checked}");
    Ok(())
}

fn closure_parity(fixture: &str) -> Result<()> {
    let repo = isolated_repo(fixture, "closure")?;
    let (discovery, build) = repo_inputs(&repo.root)?;
    assert!(
        !build.translation_units.is_empty(),
        "fixture must shape at least one TU"
    );
    for tu in &build.translation_units {
        let vendored = vendored_legacy_closure(tu, &repo.root, &discovery, &build.contexts);
        let plugin = plugin_driven_closure(tu, &repo.root, &discovery, &build.contexts)?;
        let describe = |files: &[(u64, String)]| {
            files
                .iter()
                .map(|(id, path)| format!("{id}:{path}"))
                .collect::<Vec<_>>()
                .join(" | ")
        };
        assert_eq!(
            vendored,
            plugin,
            "{}: closure ORDER/membership diverged\n  vendored: {}\n  plugin:   {}",
            normalized_rel(&tu.source_path),
            describe(&vendored),
            describe(&plugin)
        );
    }
    Ok(())
}

fn outcome_table_parity(fixture: &str) -> Result<()> {
    let repo = isolated_repo(fixture, "outcomes")?;
    let (discovery, build) = repo_inputs(&repo.root)?;
    // Both fixtures shape exactly one compilation context; the sweep resolves
    // every directive under it (the closure walks resolve under the matching
    // context, which on a single-context fixture is the same context).
    let context = build
        .contexts
        .first()
        .cloned()
        .unwrap_or_else(|| CompilationContext::new("parity", "clang++"));
    let search_paths: Vec<std::path::PathBuf> = context
        .include_dirs
        .iter()
        .chain(context.system_include_dirs.iter())
        .cloned()
        .collect();
    let ctx = ResolutionContext {
        repo_root: repo.root.as_path(),
        search_paths,
    };
    let catalog = FileCatalog::from_records(&discovery.files);
    let legacy = ariadne_lang_cpp::IncludeResolver::new(&repo.root)
        .with_repo_discovery(&discovery)
        .with_context(&context);
    let plugin = CppDependencyResolver::new();

    for record in analyzable_records(&discovery) {
        let content = fs::read_to_string(record.canonical_path.clone())?;
        for directive in parse_include_directives(&content) {
            let legacy_edge = legacy.resolve_directive(record.id, &record.path, &directive);
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
            let outcome = match plugin.resolve_target(&edge, &catalog, &ctx) {
                ResolveOutcome::Resolved(id) => Some(id),
                ResolveOutcome::Wildcard(_) => {
                    panic!("C++ resolvers never emit wildcard outcomes")
                }
                // The documented parity table: catalog misses run the legacy
                // on-disk candidate fallback shared with the permanent walks.
                ResolveOutcome::External | ResolveOutcome::Unresolved => {
                    ariadne_lang::first_ondisk_include_candidate(
                        &edge.raw_target,
                        edge.style,
                        &edge.source_rel_path,
                        &ctx.search_paths,
                        &repo.root,
                        &|path: &Path| path.exists(),
                    )
                    .map(|(id, _)| id)
                }
            };
            assert_eq!(
                legacy_edge.destination, outcome,
                "{}:{} directive {:?} outcome table diverged",
                normalized_rel(&record.path),
                directive.line,
                directive.raw_path
            );
        }
    }
    Ok(())
}

/// Vendored verbatim from `git show ceac55f:crates/scheduler/src/tu_store.rs`
/// (`compute_tu_closure`'s walk): the legacy LIFO edge-stack include closure
/// through the lang-cpp `IncludeResolver` (public, non-deprecated after
/// M18.11-B). Only the file-visit bookkeeping is kept — the fingerprint
/// hashing it also performed is locked across the seam by the closure file
/// order itself here plus the fingerprint parity suites (M18.10).
fn vendored_legacy_closure(
    tu: &TranslationUnit,
    repo_root: &Path,
    discovery: &RepositoryDiscoveryResult,
    contexts: &[CompilationContext],
) -> Vec<(u64, String)> {
    let context = contexts
        .iter()
        .find(|candidate| candidate.name == tu.context_name)
        .cloned()
        .unwrap_or_else(|| CompilationContext::new(&tu.context_name, "clang++"));
    let path_by_id: HashMap<FileId, std::path::PathBuf> = discovery
        .files
        .iter()
        .map(|record| (record.id, record.path.clone()))
        .collect();
    let resolver = ariadne_lang_cpp::IncludeResolver::new(repo_root)
        .with_repo_discovery(discovery)
        .with_context(&context);

    let source_abs = repo_root.join(&tu.source_path);
    let source_content = fs::read_to_string(&source_abs).unwrap_or_default();
    let edges = resolver.resolve_file_includes(tu.file_id, &tu.source_path, &source_content);

    let mut files = vec![(tu.file_id, tu.source_path.clone())];
    let mut visited: HashSet<FileId> = HashSet::new();
    visited.insert(tu.file_id);

    let mut queue = edges;
    while let Some(edge) = queue.pop() {
        if let Some(dest_id) = edge.destination {
            if visited.insert(dest_id) {
                let dest_path = path_by_id
                    .get(&dest_id)
                    .cloned()
                    .or_else(|| edge.resolved_path.clone());
                let Some(dest_path) = dest_path else { continue };
                let header_abs = repo_root.join(&dest_path);
                if let Ok(header_content) = fs::read_to_string(&header_abs) {
                    files.push((dest_id, dest_path.clone()));
                    queue.extend(resolver.resolve_file_includes(
                        dest_id,
                        &dest_path,
                        &header_content,
                    ));
                }
            }
        }
    }

    files
        .into_iter()
        .map(|(id, path)| (id.0, normalized_rel(&path)))
        .collect()
}

/// The same closure computed through the permanent registry path
/// (`scheduler::compute_tu_closure_plugin` driven by `CppDependencyResolver`).
fn plugin_driven_closure(
    tu: &TranslationUnit,
    repo_root: &Path,
    discovery: &RepositoryDiscoveryResult,
    contexts: &[CompilationContext],
) -> Result<Vec<(u64, String)>> {
    let store_ctx = ariadne_scheduler::TuStoreContext::new(discovery, contexts.to_vec());
    let resolver = CppDependencyResolver::new();
    let closure = ariadne_scheduler::compute_tu_closure_plugin(
        tu,
        repo_root,
        &store_ctx,
        CURRENT_ANALYSIS_VERSION,
        &resolver,
        SourceLanguage::Cpp,
        CPP_ANALYSIS_FINGERPRINT,
    )?;
    Ok(closure
        .files
        .into_iter()
        .map(|(id, path, _content)| (id.0, normalized_rel(&path)))
        .collect())
}
