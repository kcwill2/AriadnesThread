//! Fixture tests for MSBuild include-dir harvesting: each test lays out a
//! small Visual Studio tree in a tempdir and asserts the resolved search
//! dirs `msbuild_include_dirs` hands to the shaper.

use super::msbuild_include_dirs;
use ariadne_repo::{FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage};
use std::fs;
use std::path::{Path, PathBuf};

fn write(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn mkdir(root: &Path, rel: &str) -> PathBuf {
    let dir = root.join(rel);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn source_record(root: &Path, rel: &str) -> FileRecord {
    write(root, rel, "int x;\n");
    FileRecord {
        id: FileId::from_relative_path(rel),
        path: PathBuf::from(rel),
        canonical_path: root.join(rel),
        kind: FileKind::Source,
        size: 0,
        content_hash: String::new(),
        ownership: FileOwnership::Repository,
        optimization_policy: OptimizationPolicy::Optimizable,
        language: SourceLanguage::Cpp,
    }
}

fn project(body: &str) -> String {
    format!(
        "\u{feff}<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <Project DefaultTargets=\"Build\" \
         xmlns=\"http://schemas.microsoft.com/developer/msbuild/2003\">\n{body}\n</Project>\n"
    )
}

fn compile_dirs(dirs: &str) -> String {
    format!(
        "<ItemDefinitionGroup Condition=\"'$(Configuration)|$(Platform)'=='Debug|x64'\">\
         <ClCompile><AdditionalIncludeDirectories>{dirs}</AdditionalIncludeDirectories>\
         </ClCompile></ItemDefinitionGroup>"
    )
}

/// Canonical spelling for comparison: the evaluator normalizes `..` but
/// tempdirs may differ from their canonical form on some hosts.
fn sorted(mut dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    dirs.sort();
    dirs
}

#[test]
fn relative_and_project_dir_entries_resolve_against_the_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let common = mkdir(root, "Common/Include");
    let local = mkdir(root, "App/inc");
    let files = vec![source_record(root, "App/src/main.cpp")];
    write(
        root,
        "App/App.vcxproj",
        &project(&compile_dirs(
            "..\\Common\\Include;$(ProjectDir)inc;%(AdditionalIncludeDirectories)",
        )),
    );

    let dirs = msbuild_include_dirs(root, &files);

    assert_eq!(sorted(dirs), sorted(vec![common, local]));
}

#[test]
fn imported_props_supply_user_macros_and_this_file_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let third_party = mkdir(root, "ThirdParty/zlib/include");
    let shared = mkdir(root, "Props/shared");
    let files = vec![source_record(root, "App/main.cpp")];
    write(
        root,
        "Props/Common.props",
        &project(
            "<PropertyGroup Label=\"UserMacros\">\
             <ThirdPartyRoot>$(MSBuildThisFileDirectory)..\\ThirdParty\\</ThirdPartyRoot>\
             </PropertyGroup>\
             <ItemDefinitionGroup><ClCompile><AdditionalIncludeDirectories>\
             $(MSBuildThisFileDirectory)shared;%(AdditionalIncludeDirectories)\
             </AdditionalIncludeDirectories></ClCompile></ItemDefinitionGroup>",
        ),
    );
    write(
        root,
        "App/App.vcxproj",
        &project(&format!(
            "<ImportGroup Label=\"PropertySheets\">\
             <Import Project=\"..\\Props\\Common.props\" />\
             </ImportGroup>{}",
            compile_dirs("$(ThirdPartyRoot)zlib\\include")
        )),
    );

    let dirs = msbuild_include_dirs(root, &files);

    assert_eq!(sorted(dirs), sorted(vec![third_party, shared]));
}

#[test]
fn directory_build_props_is_auto_imported_from_an_ancestor() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let global = mkdir(root, "global/include");
    let files = vec![source_record(root, "libs/core/core.cpp")];
    write(
        root,
        "Directory.Build.props",
        &project(&compile_dirs("$(MSBuildThisFileDirectory)global\\include")),
    );
    write(root, "libs/core/core.vcxproj", &project(""));

    let dirs = msbuild_include_dirs(root, &files);

    assert_eq!(dirs, vec![global]);
}

#[test]
fn solution_dir_is_the_nearest_ancestor_holding_a_sln() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let api = mkdir(root, "product/api");
    let files = vec![source_record(root, "product/engine/engine.cpp")];
    write(root, "product/Product.sln", "");
    write(
        root,
        "product/engine/engine.vcxproj",
        &project(&compile_dirs("$(SolutionDir)api")),
    );

    let dirs = msbuild_include_dirs(root, &files);

    assert_eq!(dirs, vec![api]);
}

#[test]
fn include_path_properties_count_and_unknown_macros_are_dropped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let vc_dir = mkdir(root, "App/vcdirs");
    let external = mkdir(root, "App/external");
    let files = vec![source_record(root, "App/main.cpp")];
    write(
        root,
        "App/App.vcxproj",
        &project(
            "<PropertyGroup Condition=\"'$(Configuration)'=='Release'\">\
             <IncludePath>$(ProjectDir)vcdirs;$(VC_IncludePath);$(IncludePath)</IncludePath>\
             <ExternalIncludePath>external;$(ExternalIncludePath)</ExternalIncludePath>\
             </PropertyGroup>",
        ),
    );

    let dirs = msbuild_include_dirs(root, &files);

    assert_eq!(sorted(dirs), sorted(vec![vc_dir, external]));
}

#[test]
fn missing_dirs_comments_and_entities_are_handled() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let real = mkdir(root, "App/R&D");
    let files = vec![source_record(root, "App/main.cpp")];
    write(
        root,
        "App/App.vcxproj",
        &project(&format!(
            "<!-- <AdditionalIncludeDirectories>commented</AdditionalIncludeDirectories> -->{}",
            compile_dirs("does\\not\\exist; \"R&amp;D\" ;;")
        )),
    );

    let dirs = msbuild_include_dirs(root, &files);

    assert_eq!(dirs, vec![real]);
}

#[test]
fn import_cycles_terminate_and_missing_imports_are_skipped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let found = mkdir(root, "App/found");
    let files = vec![source_record(root, "App/main.cpp")];
    write(
        root,
        "App/a.props",
        &project("<Import Project=\"b.props\" /><Import Project=\"$(VCTargetsPath)\\x.props\" />"),
    );
    write(
        root,
        "App/b.props",
        &project(&format!("<Import Project=\"a.props\" />{}", compile_dirs("found"))),
    );
    write(
        root,
        "App/App.vcxproj",
        &project("<Import Project=\"a.props\" /><Import Project=\"gone.props\" />"),
    );

    let dirs = msbuild_include_dirs(root, &files);

    assert_eq!(dirs, vec![found]);
}

#[test]
fn environment_variables_expand_as_properties() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let sdk = mkdir(root, "sdk/include");
    let files = vec![source_record(root, "App/main.cpp")];
    let var_name = "ARIADNE_MSBUILD_TEST_SDK_ROOT";
    std::env::set_var(var_name, root.join("sdk"));
    write(
        root,
        "App/App.vcxproj",
        &project(&compile_dirs(&format!("$({var_name})\\include"))),
    );

    let dirs = msbuild_include_dirs(root, &files);

    assert_eq!(dirs, vec![sdk]);
}

#[test]
fn repos_without_projects_yield_nothing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    mkdir(root, "include");
    let files = vec![source_record(root, "src/main.cpp")];

    assert!(msbuild_include_dirs(root, &files).is_empty());
}
