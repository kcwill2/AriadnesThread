//! M18.2 incremental-injection parity (plan §M18, `incremental.rs` row):
//! the registry-driven `tu_store` variants consumed through
//! `IncrementalAnalyzer::with_analyzer`/`with_resolver` must reproduce the
//! transitional hardcoded path — identical closure membership, identical
//! cache fingerprint, and identical merged analysis. The M18.2 third-party
//! divergence (`ResolveOutcome::External` dropped the boundary file from the
//! plugin closure) is CLOSED in M18.5: the stateless plugin resolver maps any
//! catalog hit — including ThirdParty records — to `Resolved(id)`, so the
//! fixture below includes a third-party header and still expects parity.

use ariadne_build::{CompilationContext, CompileCommand, TranslationUnit};
use ariadne_lang_cpp::{
    analyzer::CppSemanticAnalyzer, CppDependencyResolver, CPP_ANALYSIS_FINGERPRINT,
};
use ariadne_repo::{FileId, SourceLanguage};
use ariadne_scheduler::{
    analyze_closure, analyze_closure_with, compute_tu_closure, compute_tu_closure_plugin,
    TuStoreContext,
};
use std::path::{Path, PathBuf};

const VERSION: &str = "parity-version";

fn write_file(root: &Path, rel: &str, content: &str) {
    let full = root.join(rel);
    std::fs::create_dir_all(full.parent().expect("parent dir")).unwrap();
    std::fs::write(full, content).unwrap();
}

/// Two-chain closure (`tu1 -> common.h -> base.h`), a third-party header
/// (the M18.5 reconciliation case: both paths must include it), plus a TU
/// whose single include resolves nowhere, exercising the miss path.
fn sample_repo(root: &Path) {
    write_file(root, "include/base.h", "struct Base { int b; };");
    write_file(
        root,
        "include/common.h",
        "#include \"base.h\"\nstruct Common { Base base; };",
    );
    write_file(
        root,
        "third_party/vendored/blind.h",
        "struct Vendored { int v; };",
    );
    write_file(
        root,
        "src/tu1.cpp",
        "#include \"../include/common.h\"\n#include \"../third_party/vendored/blind.h\"\nvoid use(Common c) { (void)c; }",
    );
    write_file(
        root,
        "src/tu2.cpp",
        "#include <nowhere_to_be_found.h>\nvoid alone() {}",
    );
}

/// Parity compilation context: the repo's `include/` directory as an ABSOLUTE
/// in-root include dir. Relative/non-existent search dirs made
/// `fingerprint_scoped_context` a no-op on both paths, leaving the plugin-side
/// scoping call unlocked; an absolute in-root dir forces relativization to fire
/// on BOTH paths, so a plugin path that skipped scoping would key the
/// fingerprint on the (machine-specific) absolute checkout location and diverge
/// from the legacy digest (Finding 6).
fn parity_context(root: &Path) -> CompilationContext {
    let mut context = CompilationContext::new("parity", "clang++");
    context.include_dirs = vec![root.join("include")];
    context
}

fn translation_unit(rel: &str) -> TranslationUnit {
    TranslationUnit {
        file_id: FileId::from_relative_path(rel),
        source_path: PathBuf::from(rel),
        context_name: "parity".to_string(),
        compile_command: CompileCommand {
            directory: PathBuf::new(),
            file: PathBuf::from(rel),
            command: None,
            arguments: vec![],
            context_name: "parity".to_string(),
            output: None,
        },
        language: Default::default(),
    }
}

fn given_repo(root: &Path) -> ariadne_repo::RepositoryDiscoveryResult {
    let files = ariadne_repo::discover_repository_files(root).unwrap();
    ariadne_repo::build_discovery_result(root.to_path_buf(), files)
}

fn sorted_files(closure: &ariadne_scheduler::TuClosure) -> Vec<(FileId, PathBuf, String)> {
    let mut files = closure.files.clone();
    files.sort();
    files
}

#[test]
fn plugin_closure_matches_legacy_closure_and_fingerprint() {
    let repo = tempfile::tempdir().unwrap();
    let root = repo.path();
    sample_repo(root);
    let discovery = given_repo(root);
    let context = parity_context(root);
    let store_ctx = TuStoreContext::new(&discovery, vec![context.clone()]);
    let plugin_resolver = CppDependencyResolver::new();

    for rel in ["src/tu1.cpp", "src/tu2.cpp"] {
        let tu = translation_unit(rel);
        let legacy = compute_tu_closure(&tu, root, &store_ctx, VERSION).unwrap();
        let plugin = compute_tu_closure_plugin(
            &tu,
            root,
            &store_ctx,
            VERSION,
            &plugin_resolver,
            SourceLanguage::Cpp,
            CPP_ANALYSIS_FINGERPRINT,
        )
        .unwrap();

        assert_eq!(
            legacy.fingerprint.combined_hash, plugin.fingerprint.combined_hash,
            "cache fingerprint must be identical for {rel}"
        );
        assert_eq!(
            legacy.fingerprint.language, plugin.fingerprint.language,
            "fingerprint language input must be identical for {rel}"
        );
        assert_eq!(
            legacy.fingerprint.plugin_fingerprint, plugin.fingerprint.plugin_fingerprint,
            "fingerprint plugin_fingerprint input must be identical for {rel}"
        );
        assert_eq!(
            sorted_files(&legacy),
            sorted_files(&plugin),
            "closure file membership and content must be identical for {rel}"
        );
        // M18.3 strengthened: the merged-analysis concatenation reads
        // `closure.files` in walk order (part of every shard envelope and
        // `index.json`), so ORDER is part of the byte-parity contract — the
        // plugin walk must reproduce the legacy LIFO edge-stack order exactly
        // (tu1: common.h-then-blind.h ⇒ legacy visits blind.h before the
        // common.h chain).
        assert_eq!(
            legacy.files, plugin.files,
            "closure file ORDER must be identical for {rel}: legacy={:?} plugin={:?}",
            legacy.files.iter().map(|(id, _, _)| *id).collect::<Vec<_>>(),
            plugin.files.iter().map(|(id, _, _)| *id).collect::<Vec<_>>(),
        );
    }

    // M18.5 reconciliation: the third-party header stays in BOTH closures
    // (legacy `External`-kind edge and plugin `Resolved` outcome coincide).
    let tu = translation_unit("src/tu1.cpp");
    let closure = compute_tu_closure_plugin(
        &tu,
        root,
        &store_ctx,
        VERSION,
        &plugin_resolver,
        SourceLanguage::Cpp,
        CPP_ANALYSIS_FINGERPRINT,
    )
    .unwrap();
    let third_party = FileId::from_relative_path("third_party/vendored/blind.h");
    assert!(
        closure.files.iter().any(|(id, _, _)| *id == third_party),
        "third-party closure member dropped: {:?}",
        closure
            .files
            .iter()
            .map(|(id, _, _)| *id)
            .collect::<Vec<_>>()
    );
}

#[test]
fn injected_analyzer_merge_matches_hardcoded_merge() {
    let repo = tempfile::tempdir().unwrap();
    let root = repo.path();
    sample_repo(root);
    let discovery = given_repo(root);
    let context = parity_context(root);
    let store_ctx = TuStoreContext::new(&discovery, vec![context.clone()]);

    let tu = translation_unit("src/tu1.cpp");
    let closure = compute_tu_closure(&tu, root, &store_ctx, VERSION).unwrap();

    let legacy = analyze_closure(&tu, &closure);
    let injected = analyze_closure_with(&tu, &closure, &CppSemanticAnalyzer);
    assert_eq!(
        serde_json::to_string(&legacy).unwrap(),
        serde_json::to_string(&injected).unwrap(),
        "injected `&dyn SemanticAnalyzer` merge must be byte-identical"
    );
}

/// A repo whose single TU pulls a header purely through the `-I` search dir
/// (the header is NOT next to the includer), so both resolution and the
/// fingerprint's context term travel the include-dir path.
fn write_search_chain_repo(root: &Path) {
    write_file(root, "include/root_chain.h", "struct RootChain { int r; };");
    write_file(root, "src/chain.cpp", "#include \"root_chain.h\"\nvoid chain() {}");
}

/// The plugin-side cache fingerprint for `src/chain.cpp` under a given root with
/// an explicit include-dir search-path set (so a test controls the context term
/// without touching discovery).
fn plugin_fingerprint_for(root: &Path, include_dirs: Vec<PathBuf>) -> String {
    let discovery = given_repo(root);
    let mut context = CompilationContext::new("parity", "clang++");
    context.include_dirs = include_dirs;
    let store_ctx = TuStoreContext::new(&discovery, vec![context.clone()]);
    let tu = translation_unit("src/chain.cpp");
    compute_tu_closure_plugin(
        &tu,
        root,
        &store_ctx,
        VERSION,
        &CppDependencyResolver::new(),
        SourceLanguage::Cpp,
        CPP_ANALYSIS_FINGERPRINT,
    )
    .unwrap()
    .fingerprint
    .combined_hash
}

/// M18.5 Finding 1 lock: an angled `<hdr>` include whose only candidate is an
/// ABSOLUTE include dir OUTSIDE repo discovery. The transitional resolver visited
/// it via the on-disk search-path fallback (the candidate exists on disk but is
/// absent from discovery, so it gets a synthetic id and is hashed from its
/// resolved path — the `record_hash` note in `tu_store.rs`); the pre-M18.5 plugin
/// closure walk checked ONLY the includer directory for quoted directives and
/// dropped angled outcomes entirely, so its closure order and fingerprint
/// diverged. Both paths must now visit the header and agree byte-for-byte.
#[test]
fn plugin_closure_resolves_angled_include_from_out_of_root_search_path() {
    let repo = tempfile::tempdir().unwrap();
    let root = repo.path();
    // A distinct temp dir stands in for an out-of-root SDK/vcpkg include path:
    // its headers are never in `root`'s discovery, reachable only via `-I`.
    let sdk = tempfile::tempdir().unwrap();
    let sdk_root = sdk.path();

    write_file(
        root,
        "src/uses_sdk.cpp",
        "#include <sdk_header.h>\nvoid use() {}",
    );
    write_file(sdk_root, "sdk_header.h", "struct SdkHeader { int x; };");

    let discovery = given_repo(root);
    let mut context = CompilationContext::new("parity", "clang++");
    context.include_dirs = vec![sdk_root.to_path_buf()];
    let store_ctx = TuStoreContext::new(&discovery, vec![context.clone()]);
    let plugin_resolver = CppDependencyResolver::new();

    // Fixture precondition: the header is genuinely absent from discovery, so
    // this exercises the disk fallback, not a catalog `Resolved` hit.
    assert!(
        discovery
            .files
            .iter()
            .all(|f| !f.path.to_string_lossy().ends_with("sdk_header.h")),
        "the -I header must be absent from repo discovery for this to be the disk-fallback path"
    );

    let tu = translation_unit("src/uses_sdk.cpp");
    let legacy = compute_tu_closure(&tu, root, &store_ctx, VERSION).unwrap();
    let plugin = compute_tu_closure_plugin(
        &tu,
        root,
        &store_ctx,
        VERSION,
        &plugin_resolver,
        SourceLanguage::Cpp,
        CPP_ANALYSIS_FINGERPRINT,
    )
    .unwrap();

    // The out-of-discovery header must have been VISITED (not merely named by an
    // unresolved edge) on both walks, in identical order, with identical digest.
    assert_eq!(
        legacy.files, plugin.files,
        "angled search-path header diverged: legacy={:?} plugin={:?}",
        legacy
            .files
            .iter()
            .map(|(id, path, _)| (*id, path.clone()))
            .collect::<Vec<_>>(),
        plugin
            .files
            .iter()
            .map(|(id, path, _)| (*id, path.clone()))
            .collect::<Vec<_>>(),
    );
    assert_eq!(
        legacy.fingerprint.combined_hash, plugin.fingerprint.combined_hash,
        "angled search-path header fingerprint diverged"
    );
    assert!(
        legacy
            .files
            .iter()
            .any(|(_, path, _)| path.to_string_lossy().ends_with("sdk_header.h")),
        "legacy must visit the -I header via the disk fallback: {:?}",
        legacy
            .files
            .iter()
            .map(|(_, p, _)| p.clone())
            .collect::<Vec<_>>()
    );
}

/// Finding 6 lock: the plugin path must relativize an ABSOLUTE in-root include
/// dir out of the cache fingerprint exactly like the legacy path. Two identical
/// repo copies at different absolute roots (each with an absolute `include/`
/// search dir) must yield the SAME plugin fingerprint — otherwise the shard store
/// keys on the machine-specific checkout location and the M18 byte-parity contract
/// across repo copies breaks. The out-of-root counterpart is the opposite: it
/// stays verbatim, so two contexts differing ONLY by an out-of-root dir must give
/// DIFFERENT digests.
#[test]
fn plugin_fingerprint_relativizes_in_root_dir_and_keeps_out_of_root_verbatim() {
    let copy_a = tempfile::tempdir().unwrap();
    let copy_b = tempfile::tempdir().unwrap();
    write_search_chain_repo(copy_a.path());
    write_search_chain_repo(copy_b.path());

    // Same content, distinct absolute in-root include dirs.
    let fp_a = plugin_fingerprint_for(copy_a.path(), vec![copy_a.path().join("include")]);
    let fp_b = plugin_fingerprint_for(copy_b.path(), vec![copy_b.path().join("include")]);
    assert_eq!(
        fp_a, fp_b,
        "in-root include dir must relativize on the plugin path (distinct roots gave distinct digests)"
    );

    // Out-of-root counterpart: identical content, two distinct out-of-root dirs.
    let external_root = tempfile::tempdir().unwrap();
    write_search_chain_repo(external_root.path());
    let out_dir_one = tempfile::tempdir().unwrap();
    let out_dir_two = tempfile::tempdir().unwrap();
    let fp_one = plugin_fingerprint_for(
        external_root.path(),
        vec![
            external_root.path().join("include"),
            out_dir_one.path().to_path_buf(),
        ],
    );
    let fp_two = plugin_fingerprint_for(
        external_root.path(),
        vec![
            external_root.path().join("include"),
            out_dir_two.path().to_path_buf(),
        ],
    );
    assert_ne!(
        fp_one, fp_two,
        "out-of-root include dir is kept verbatim, so differing out-of-root dirs must give differing digests"
    );
}
