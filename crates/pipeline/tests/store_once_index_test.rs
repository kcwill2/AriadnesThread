//! Store-once indexing: a header shared by many units is analyzed and stored
//! once, yet every downstream answer matches the old full-copy index.
//!
//! The fixture has three units over a header chain (`api.h` -> `base.h`),
//! included through a `.vcxproj` include dir, plus one unit-private header.

use ariadne_build::TranslationUnit;
use ariadne_graph::ReducedSemanticModel;
use ariadne_index::{IndexResult, TuAnalysis};
use ariadne_lang::LanguageId;
use ariadne_pipeline::{build_default_registry, Pipeline};
use ariadne_repo::FileId;
use ariadne_scheduler::closure_memo::{walk_tu_closure, ClosureMemo, WalkInputs};
use ariadne_scheduler::tu_store::{analyze_tu_full_with_plugin, compute_tu_closure_plugin};
use ariadne_index::CURRENT_ANALYSIS_VERSION;
use ariadne_scheduler::TuStoreContext;
use std::fs;
use std::path::Path;

fn write(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn shared_header_repo(root: &Path) {
    write(
        root,
        "app/app.vcxproj",
        "<Project><ItemDefinitionGroup><ClCompile>\
         <AdditionalIncludeDirectories>..\\lib\\include</AdditionalIncludeDirectories>\
         </ClCompile></ItemDefinitionGroup></Project>",
    );
    write(
        root,
        "lib/include/lib/base.h",
        "#pragma once\nstruct Base { int id; };\ninline int base_id(Base b) { return b.id; }\n",
    );
    write(
        root,
        "lib/include/lib/api.h",
        "#pragma once\n#include \"lib/base.h\"\nstruct Api : Base { int value; };\n\
         inline int api_value(Api a) { return a.value + base_id(a); }\n",
    );
    write(root, "app/src/private.h", "#pragma once\nstruct Private { int p; };\n");
    write(
        root,
        "app/src/one.cpp",
        "#include \"lib/api.h\"\nint one() { Api a{}; return api_value(a); }\n",
    );
    write(
        root,
        "app/src/two.cpp",
        "#include \"lib/api.h\"\n#include \"private.h\"\n\
         int two() { Private p{}; Api a{}; return api_value(a) + p.p; }\n",
    );
    write(
        root,
        "app/src/three.cpp",
        "#include \"lib/base.h\"\nint three() { Base b{}; return base_id(b); }\n",
    );
}

/// The old index: every unit carries its full closure's records.
fn full_copy_index(pipeline: &Pipeline) -> IndexResult {
    let registry = build_default_registry();
    let plugin = registry.plugin_for_id(LanguageId::cpp()).unwrap();
    let store_ctx = TuStoreContext::new(&pipeline.discovery, pipeline.build.contexts.clone());
    let analyses: Vec<TuAnalysis> = pipeline
        .build
        .translation_units
        .iter()
        .map(|tu| analyze_tu_full_with_plugin(tu, &pipeline.repo.root, &store_ctx, plugin).unwrap().0)
        .collect();
    IndexResult::new(pipeline.repo.root.clone(), analyses)
}

fn header_id(pipeline: &Pipeline, rel: &str) -> FileId {
    pipeline
        .discovery
        .files
        .iter()
        .find(|file| file.path.to_string_lossy().replace('\\', "/") == rel)
        .unwrap_or_else(|| panic!("{rel} not discovered"))
        .id
}

fn units_storing(index: &IndexResult, file: FileId) -> usize {
    index
        .tu_analyses
        .iter()
        .filter(|tu| tu.symbols.iter().any(|symbol| symbol.owning_file == file))
        .count()
}

#[test]
fn shared_headers_are_stored_once_and_named_by_later_units() {
    let dir = tempfile::tempdir().unwrap();
    shared_header_repo(dir.path());
    let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
    let index = pipeline.get_or_build_index().unwrap();
    assert_eq!(index.tu_analyses.len(), 3);

    let base = header_id(&pipeline, "lib/include/lib/base.h");
    let api = header_id(&pipeline, "lib/include/lib/api.h");
    assert_eq!(units_storing(&index, base), 1, "base.h stored once");
    assert_eq!(units_storing(&index, api), 1, "api.h stored once");
    let sharing_base = index
        .tu_analyses
        .iter()
        .filter(|tu| tu.shared_files.contains(&base))
        .count();
    assert_eq!(sharing_base, 2, "the two later units name base.h as shared");

    let full = full_copy_index(&pipeline);
    assert!(
        index.total_edges < full.total_edges,
        "store-once keeps fewer edge records ({} vs {})",
        index.total_edges,
        full.total_edges
    );
}

#[test]
fn store_once_index_reduces_to_the_same_model_as_full_copies() {
    let dir = tempfile::tempdir().unwrap();
    shared_header_repo(dir.path());
    let mut pipeline = Pipeline::new(dir.path(), None).unwrap();
    let store_once = pipeline.get_or_build_index().unwrap();
    let full = full_copy_index(&pipeline);

    let reduced_once = ReducedSemanticModel::reduce(&store_once);
    let reduced_full = ReducedSemanticModel::reduce(&full);
    assert!(!reduced_full.tu_usage.is_empty());
    assert_eq!(reduced_once.symbols, reduced_full.symbols);
    assert_eq!(reduced_once.edges, reduced_full.edges);
    assert_eq!(reduced_once.tu_usage, reduced_full.tu_usage);
    assert_eq!(reduced_once.file_symbols, reduced_full.file_symbols);

    // A warm run loads the saved index; the shared lists must come back.
    let mut warm = Pipeline::new(dir.path(), None).unwrap();
    let loaded = warm.get_or_build_index().unwrap();
    assert_eq!(loaded.tu_analyses, store_once.tu_analyses);
    assert_eq!(ReducedSemanticModel::reduce(&loaded).tu_usage, reduced_full.tu_usage);
}

#[test]
fn memoized_walk_visits_the_canonical_closure_order() {
    let dir = tempfile::tempdir().unwrap();
    shared_header_repo(dir.path());
    let pipeline = Pipeline::new(dir.path(), None).unwrap();
    let registry = build_default_registry();
    let plugin = registry.plugin_for_id(LanguageId::cpp()).unwrap();
    let store_ctx = TuStoreContext::new(&pipeline.discovery, pipeline.build.contexts.clone());
    let memo = ClosureMemo::new();
    let inputs = WalkInputs {
        repo_root: &pipeline.repo.root,
        store_ctx: &store_ctx,
        resolver: plugin.resolver(),
    };
    for tu in &pipeline.build.translation_units {
        let canonical: Vec<FileId> = canonical_files(tu, &pipeline, &store_ctx);
        let memoized: Vec<FileId> = walk_tu_closure(tu, &inputs, &memo)
            .into_iter()
            .map(|file| file.file_id)
            .collect();
        assert_eq!(memoized, canonical, "closure order for {:?}", tu.source_path);
    }
    assert_eq!(memo.scanned_count(), 6, "3 sources + 3 headers, each scanned once");
}

fn canonical_files(tu: &TranslationUnit, pipeline: &Pipeline, ctx: &TuStoreContext) -> Vec<FileId> {
    let registry = build_default_registry();
    let plugin = registry.plugin_for_id(LanguageId::cpp()).unwrap();
    compute_tu_closure_plugin(
        tu,
        &pipeline.repo.root,
        ctx,
        CURRENT_ANALYSIS_VERSION,
        plugin.resolver(),
        plugin.id(),
        &plugin.analysis_fingerprint(),
    )
    .unwrap()
    .files
    .into_iter()
    .map(|(file_id, _, _)| file_id)
    .collect()
}

#[test]
fn editing_a_shared_header_reanalyzes_only_that_header() {
    let dir = tempfile::tempdir().unwrap();
    shared_header_repo(dir.path());
    Pipeline::new(dir.path(), None).unwrap().get_or_build_index().unwrap();

    write(
        dir.path(),
        "lib/include/lib/api.h",
        "#pragma once\n#include \"lib/base.h\"\nstruct Api : Base { int value; int extra; };\n\
         inline int api_value(Api a) { return a.value + base_id(a); }\n",
    );
    let mut warm = Pipeline::new(dir.path(), None).unwrap();
    warm.get_or_build_index().unwrap();
    assert_eq!(
        (warm.stats.reused_tus, warm.stats.reanalyzed_tus),
        (2, 1),
        "only the unit storing api.h has anything to re-analyze"
    );
}
