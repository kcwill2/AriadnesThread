//! UTA-T5 characterization pins for the private `tu_store` seams, extracted
//! from `tu_store.rs`'s test module when that file approached the 1000-line
//! hygiene ceiling (extraction moved test code only; the seams under test are
//! byte-untouched). As a child module of `tu_store` this file keeps legal
//! access to the private functions the pins exercise
//! (`edge_destinations`, `fingerprint_scoped_context`).

use super::*;

fn pins_ctx(root: &Path) -> SharedTuStoreContext {
    let files = ariadne_repo::discover_repository_files(root).unwrap();
    let discovery = ariadne_repo::build_discovery_result(root.to_path_buf(), files);
    TuStoreContext::new(&discovery, vec![])
}

// ---- UTA-T5 item 2: concurrent savers must not share tmp names ---------
// save_shard names its temp file `.{file_id}.{pid}.{counter}.tmp`, unique
// per save within a process (atomic counter) and across processes (pid).
// This lock drives two threads saving the SAME FileId through a barrier
// and requires the surviving shard to always be one writer's complete
// checkpoint: mixed/corrupt bytes or leaked tmp residue would fail here.

#[test]
fn concurrent_savers_of_one_file_id_never_mix_checkpoint_bytes() {
    use std::sync::{Arc, Barrier};

    fn symbol(rel: &str, name: &str) -> ariadne_index::types::SymbolRecord {
        ariadne_index::types::SymbolRecord {
            id: ariadne_index::types::SymbolId::new(name),
            name: name.to_string(),
            kind: ariadne_index::types::SymbolKind::Function,
            owning_file: FileId::from_relative_path(rel),
            declarations: vec![],
            definition: None,

            modifiers: Vec::new(),
        }
    }

    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(dir.path().to_path_buf());
    let tu_id = FileId::from_relative_path("src/racy.cpp");

    // Two structurally distinguishable writers: alpha has zero symbols,
    // beta exactly three. A fingerprint/analysis mismatch in the loaded
    // shard means bytes from both writers were spliced together.
    let alpha = ShardCheckpoint {
        fingerprint: "alpha-fp".into(),
        analysis: TuAnalysis {
            tu_file_id: tu_id,
            ..Default::default()
        },
    };
    let beta = ShardCheckpoint {
        fingerprint: "beta-fp".into(),
        analysis: TuAnalysis {
            tu_file_id: tu_id,
            symbols: vec![
                symbol("src/racy.cpp", "beta_one"),
                symbol("src/racy.cpp", "beta_two"),
                symbol("src/racy.cpp", "beta_three"),
            ],
            ..Default::default()
        },
    };

    for round in 0..25u32 {
        let barrier = Arc::new(Barrier::new(2));
        let mut handles = Vec::new();
        for checkpoint in [alpha.clone(), beta.clone()] {
            let store_writer = Arc::clone(&store);
            let writer_barrier = Arc::clone(&barrier);
            handles.push(std::thread::spawn(move || {
                writer_barrier.wait();
                save_shard(&store_writer, &checkpoint).unwrap();
            }));
        }
        for handle in handles {
            handle.join().expect("concurrent saver thread must not panic");
        }
        let loaded = load_shard(&store, tu_id)
            .unwrap_or_else(|| panic!("round {round}: final shard must be a complete envelope"));
        if loaded.fingerprint == "alpha-fp" {
            assert!(
                loaded.analysis.symbols.is_empty(),
                "round {round}: alpha's fingerprint over beta's analysis is spliced bytes"
            );
        } else {
            assert_eq!(loaded.fingerprint, "beta-fp");
            assert_eq!(
                loaded.analysis.symbols.len(),
                3,
                "round {round}: beta's fingerprint must carry beta's three symbols"
            );
        }
    }

    let tmp_residue: Vec<String> = fs::read_dir(store.as_path())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".tmp"))
        .collect();
    assert!(
        tmp_residue.is_empty(),
        "every tmp must be consumed by its own rename: {tmp_residue:?}"
    );
}

// ---- UTA-T5 item 10: corrupt shards and unwritable dirs ---------------

#[test]
fn truncated_and_non_utf8_shards_load_as_miss() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();
    let tu_id = FileId::from_relative_path("src/corrupt.cpp");

    // Truncated envelope JSON: parse failure is a miss, not an error.
    fs::write(shard_file(path, tu_id), "{\"fingerprint\":\"abc\"").unwrap();
    assert!(load_shard(path, tu_id).is_none());

    // Non-UTF-8 bytes: read failure is a miss, not an error.
    fs::write(shard_file(path, tu_id), [0xFFu8, 0xFE, 0x00, 0x01, 0xFD]).unwrap();
    assert!(load_shard(path, tu_id).is_none());
}

#[test]
fn save_shard_surfaces_create_dir_all_failure_as_error() {
    let dir = tempfile::tempdir().unwrap();
    // A plain file blocking the parent path makes create_dir_all fail.
    let blocker = dir.path().join("blocked");
    fs::write(&blocker, "not a directory").unwrap();
    let unwritable_dir = blocker.join("shards");

    let checkpoint = ShardCheckpoint {
        fingerprint: "never-written".into(),
        analysis: TuAnalysis::default(),
    };
    let outcome = save_shard(&unwritable_dir, &checkpoint);
    assert!(
        outcome.is_err(),
        "a directory that cannot be created must surface an error, \
         and nothing may have been written"
    );
}

// ---- UTA-T5 item 12: edge_destinations outcome arms --------------------

struct ScriptedOutcomeResolver {
    outcome: ResolveOutcome,
}

impl DependencyResolver for ScriptedOutcomeResolver {
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

fn pinned_edge(raw_target: &str, style: ariadne_lang::DirectiveStyle) -> RawDepEdge {
    RawDepEdge {
        source_file: FileId::from_relative_path("src/main.cpp"),
        source_rel_path: PathBuf::from("src/main.cpp"),
        raw_target: raw_target.to_string(),
        is_wildcard: false,
        style,
        line: 1,
        column: 1,
    }
}

#[test]
fn edge_destinations_wildcard_fanout_preserves_resolver_id_order() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("w_first.h"), "struct First {};").unwrap();
    fs::write(root.join("w_second.h"), "struct Second {};").unwrap();
    fs::write(root.join("w_third.h"), "struct Third {};").unwrap();
    let ctx = pins_ctx(root);

    // Resolver answers in an order DIFFERENT from lexicographic FileId
    // order: fan-out must keep the resolver's order verbatim (closure
    // merge order feeds every shard envelope, M18.3 contract).
    let given_order = vec![
        FileId::from_relative_path("w_third.h"),
        FileId::from_relative_path("w_first.h"),
        FileId::from_relative_path("w_second.h"),
    ];
    let resolver = ScriptedOutcomeResolver {
        outcome: ResolveOutcome::Wildcard(given_order.clone()),
    };
    let catalog = FileCatalog::from_records(&ctx.discovery.files);
    let resolution = ResolutionContext {
        repo_root: root,
        search_paths: vec![],
    };
    let edge = pinned_edge("w_*.h", ariadne_lang::DirectiveStyle::Quoted);

    let destinations = edge_destinations(&edge, &resolver, &catalog, &resolution, &ctx, root);
    let got_ids: Vec<FileId> = destinations.iter().map(|(id, _)| *id).collect();
    assert_eq!(got_ids, given_order);
    assert_eq!(
        destinations[0].1,
        PathBuf::from("w_third.h"),
        "each fan-out id pairs with its discovery path"
    );
}

#[test]
fn edge_destinations_external_fallback_reaches_include_dirs_and_includer_dir() {
    let repo = tempfile::tempdir().unwrap();
    let root = repo.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.cpp"), "#include <sdk_only.h>").unwrap();
    // Quoted-includer-dir twin of the same fallback loop.
    fs::write(root.join("src/local.h"), "struct Local {};").unwrap();
    // An out-of-root -I dir holding an angled-only header (M18.5 finding).
    let sdk = tempfile::tempdir().unwrap();
    fs::write(sdk.path().join("sdk_only.h"), "struct SdkOnly {};").unwrap();

    let ctx = pins_ctx(root);
    let catalog = FileCatalog::from_records(&ctx.discovery.files);
    let resolution = ResolutionContext {
        repo_root: root,
        search_paths: vec![sdk.path().to_path_buf()],
    };

    // External outcome + angled spelling: found ONLY through the -I dir.
    let resolver = ScriptedOutcomeResolver {
        outcome: ResolveOutcome::External,
    };
    let edge = pinned_edge("sdk_only.h", ariadne_lang::DirectiveStyle::Angled);
    let destinations = edge_destinations(&edge, &resolver, &catalog, &resolution, &ctx, root);
    assert_eq!(destinations.len(), 1, "the -I hit must be visited");
    let hit_path = destinations[0].1.to_string_lossy().replace('\\', "/");
    let sdk_path = sdk.path().to_string_lossy().replace('\\', "/");
    assert!(
        hit_path.contains(&sdk_path) && hit_path.ends_with("sdk_only.h"),
        "angled on-disk fallback must resolve through the -I dir: {hit_path}"
    );

    // Unresolved outcome + quoted spelling: includer directory is checked
    // (the quoted-only borrow) with the same synthetic (FileId, rel path)
    // shape the legacy resolver emitted.
    let resolver = ScriptedOutcomeResolver {
        outcome: ResolveOutcome::Unresolved,
    };
    let edge = pinned_edge("local.h", ariadne_lang::DirectiveStyle::Quoted);
    let destinations = edge_destinations(&edge, &resolver, &catalog, &resolution, &ctx, root);
    assert_eq!(destinations.len(), 1);
    assert_eq!(destinations[0].0, FileId::from_relative_path("src/local.h"));
    assert_eq!(destinations[0].1, PathBuf::from("src/local.h"));

    // Control: a directive found nowhere (not includer dir, not any -I
    // dir) contributes no destination.
    let edge = pinned_edge("nowhere.h", ariadne_lang::DirectiveStyle::Angled);
    let destinations = edge_destinations(&edge, &resolver, &catalog, &resolution, &ctx, root);
    assert!(destinations.is_empty(), "unfindable directive visits nothing");
}

// ---- UTA-T5 item 13: fingerprint_scoped_context rewriting --------------

fn scoped_fixture_context(root: &Path) -> CompilationContext {
    let mut context = CompilationContext::new("scoped-pin", "fake-toolchain");
    context.include_dirs = vec![root.join("inc")];
    context.forced_includes = vec![root.join("prelude.h")];
    // A path that lies outside any temp root and cannot canonicalize:
    // it must stay verbatim (system SDK shape).
    context.system_include_dirs = vec![PathBuf::from("/definitely-outside-repo/sdk")];
    context
}

#[test]
fn fingerprint_scoped_context_rewrites_in_root_but_keeps_system_verbatim() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let scoped = fingerprint_scoped_context(&scoped_fixture_context(root), root);
    assert_eq!(
        scoped.include_dirs,
        vec![PathBuf::from("inc")],
        "in-root include dirs are rewritten repo-relative"
    );
    assert_eq!(
        scoped.forced_includes,
        vec![PathBuf::from("prelude.h")],
        "in-root forced includes are rewritten repo-relative"
    );
    assert_eq!(
        scoped.system_include_dirs,
        vec![PathBuf::from("/definitely-outside-repo/sdk")],
        "out-of-root system paths stay verbatim"
    );
}

#[test]
fn fingerprint_scoped_context_equal_across_copied_roots() {
    // The M18 byte-parity contract at its narrowest: the same repo in two
    // different roots must fingerprint through IDENTICAL scoped contexts.
    let original = tempfile::tempdir().unwrap();
    let copy = tempfile::tempdir().unwrap();
    let scoped_original =
        fingerprint_scoped_context(&scoped_fixture_context(original.path()), original.path());
    let scoped_copy =
        fingerprint_scoped_context(&scoped_fixture_context(copy.path()), copy.path());
    assert_eq!(scoped_original, scoped_copy);
}
