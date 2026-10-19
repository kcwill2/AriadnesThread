//! UTA2-R4 characterization pins for the private `edge_destinations` seam
//! (docs/specs/unit-test-audit.md Round 2, pin-and-report class:
//! `tu_store.rs` silent drop of ids missing from `path_by_id`). Extracted as
//! a separate `#[path]` child of `tu_store` (the `tu_store_t5_pins.rs`
//! precedent) so the pins keep legal access to the private function while
//! `tu_store.rs` stays away from the 1000-line hygiene ceiling.

use super::*;

/// Resolver double: answers every `resolve_target` call with one scripted
/// outcome (the `tu_store_t5_pins` double, kept local so round-1 pins stay
/// byte-untouched).
struct ScriptedR4Resolver {
    outcome: ResolveOutcome,
}

impl DependencyResolver for ScriptedR4Resolver {
    fn resolve_edges(
        &self,
        _file_id: FileId,
        _rel_path: &Path,
        _source: &str,
        _context: &ResolutionContext<'_>,
    ) -> Vec<RawDepEdge> {
        Vec::new()
    }

    fn resolve_target(
        &self,
        _edge: &RawDepEdge,
        _catalog: &FileCatalog,
        _context: &ResolutionContext<'_>,
    ) -> ResolveOutcome {
        self.outcome.clone()
    }
}

fn r4_ctx(root: &Path) -> SharedTuStoreContext {
    let files = ariadne_repo::discover_repository_files(root).unwrap();
    let discovery = ariadne_repo::build_discovery_result(root.to_path_buf(), files);
    TuStoreContext::new(&discovery, vec![])
}

fn r4_edge(raw_target: &str) -> RawDepEdge {
    RawDepEdge {
        source_file: FileId::from_relative_path("src/main.cpp"),
        source_rel_path: PathBuf::from("src/main.cpp"),
        raw_target: raw_target.to_string(),
        is_wildcard: false,
        style: ariadne_lang::DirectiveStyle::Quoted,
        line: 1,
        column: 1,
    }
}

/// REPORT (UTA-B6 class, no fix - the shape decision is owed): ids the
/// resolver returns that are absent from `path_by_id` are dropped WITHOUT a
/// counter, warning, or error. The recommended surface is a dropped-id count
/// (and reason) on the closure walk output, mirroring how `External` /
/// `Unresolved` directives keep their on-disk fallback visibly.
#[test]
fn pin_edge_destination_drops_unknown_id() {
    let repo = tempfile::tempdir().unwrap();
    let root = repo.path();
    fs::write(root.join("known_a.h"), "struct A {};").unwrap();
    fs::write(root.join("known_b.h"), "struct B {};").unwrap();
    let ctx = r4_ctx(root);

    // The ghost header's BYTES are planted after discovery: an id absent
    // from `path_by_id` must drop even when the file exists on disk, which
    // pins that the Resolved/Wildcard arms never fall through to the
    // `on_disk_include_fallback` (that fallback belongs to External /
    // Unresolved only).
    fs::write(root.join("ghost.h"), "struct Ghost {};").unwrap();
    let ghost_id = FileId::from_relative_path("ghost.h");
    let other_ghost_id = FileId::from_relative_path("nested/ghost2.h");

    // Search paths widened to the root itself: were the fallback reachable
    // from these arms, the ghost WOULD resolve. It must not.
    let catalog = FileCatalog::from_records(&ctx.discovery.files);
    let resolution = ResolutionContext {
        repo_root: root,
        search_paths: vec![root.to_path_buf()],
    };
    let edge = r4_edge("ghost.h");

    // Arm 1: Resolved(ghost) -> silent drop, no destination, no error.
    let resolver = ScriptedR4Resolver {
        outcome: ResolveOutcome::Resolved(ghost_id),
    };
    let destinations =
        edge_destinations(&edge, &resolver, &catalog, &resolution, &ctx, root);
    assert!(
        destinations.is_empty(),
        "pin: a Resolved id absent from path_by_id is dropped silently - \
         no counter, no warning, no on-disk fallback: {destinations:?}"
    );

    // Arm 2: Wildcard fan-out keeps the KNOWN ids in resolver order and
    // drops only the unknown ones (the survivors pair with their discovery
    // paths unchanged).
    let known_a = FileId::from_relative_path("known_a.h");
    let known_b = FileId::from_relative_path("known_b.h");
    let resolver = ScriptedR4Resolver {
        outcome: ResolveOutcome::Wildcard(vec![known_a, ghost_id, known_b, other_ghost_id]),
    };
    let destinations =
        edge_destinations(&edge, &resolver, &catalog, &resolution, &ctx, root);
    let got_ids: Vec<FileId> = destinations.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        got_ids,
        vec![known_a, known_b],
        "pin: the drop removes exactly the unknown ids and preserves order"
    );
    assert_eq!(destinations[0].1, PathBuf::from("known_a.h"));
    assert_eq!(destinations[1].1, PathBuf::from("known_b.h"));

    // Arm 2 boundary: an all-unknown fan-out contributes nothing at all.
    let resolver = ScriptedR4Resolver {
        outcome: ResolveOutcome::Wildcard(vec![ghost_id, other_ghost_id]),
    };
    let destinations =
        edge_destinations(&edge, &resolver, &catalog, &resolution, &ctx, root);
    assert!(
        destinations.is_empty(),
        "pin: a fully-unknown Wildcard walk visits nothing, still silently"
    );
}
