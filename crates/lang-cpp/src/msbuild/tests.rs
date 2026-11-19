//! Fixture tests for MSBuild include-dir harvesting: each test lays out a
//! small Visual Studio tree in a tempdir and asserts the resolved search
//! dirs `msbuild_include_dirs` hands to the shaper.

use super::{msbuild_include_dirs, msbuild_notes};
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
fn notes_list_imported_props_files_with_project_counts() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let files = vec![source_record(root, "App/main.cpp")];
    write(root, "Directory.Build.props", &project(""));
    write(root, "Props/Common.props", &project(""));
    let import_common = "<ImportGroup><Import Project=\"..\\Props\\Common.props\" /></ImportGroup>";
    write(root, "App/App.vcxproj", &project(import_common));
    write(root, "Lib/Lib.vcxproj", &project(import_common));
    write(root, "Solo/Solo.vcxproj", &project(""));

    let notes = msbuild_notes(root, &files);

    let listing: Vec<&str> = notes
        .iter()
        .map(String::as_str)
        .skip_while(|line| !line.starts_with("MSBuild: 2 props file(s) imported"))
        .take(3)
        .collect();
    assert_eq!(
        listing,
        vec![
            "MSBuild: 2 props file(s) imported:",
            "  Directory.Build.props (3 projects)",
            "  Props/Common.props (2 projects)",
        ],
        "notes were: {notes:#?}"
    );
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

#[test]
fn notes_name_unresolved_macros_missing_dirs_and_imports() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    mkdir(root, "App/inc");
    let files = vec![source_record(root, "App/main.cpp")];
    write(
        root,
        "Props/Sdk.props",
        &project(&compile_dirs(
            "$(EnGenSDKRoot)\\include;$(EnGenSDKRoot)\\lib\\include;%(AdditionalIncludeDirectories)",
        )),
    );
    write(
        root,
        "App/App.vcxproj",
        &project(&format!(
            "<Import Project=\"..\\Props\\Sdk.props\" />\
             <Import Project=\"$(UserRootDir)\\Microsoft.Cpp.x64.user.props\" />{}",
            compile_dirs("inc;gone\\dir")
        )),
    );

    let notes = msbuild_notes(root, &files).join("\\n");

    assert!(notes.contains("1 project file"), "{notes}");
    assert!(notes.contains("1 include dir"), "{notes}");
    assert!(
        notes.contains("$(EnGenSDKRoot)") && notes.contains("2 entries")
            && notes.contains("Props/Sdk.props"),
        "unknown macro, its drop count and where it was used: {notes}"
    );
    assert!(
        !notes.contains("%(AdditionalIncludeDirectories)"),
        "item-metadata inheritance is normal, not a miss: {notes}"
    );
    assert!(notes.contains("gone/dir") && notes.contains("App/App.vcxproj"), "{notes}");
    assert!(notes.contains("$(UserRootDir)"), "unresolvable import: {notes}");
}

#[test]
fn notes_say_so_when_cpp_files_have_no_projects() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let files = vec![source_record(root, "src/main.cpp")];
    let notes = msbuild_notes(root, &files);
    assert_eq!(notes.len(), 1, "{notes:?}");
    assert!(notes[0].contains("no .vcxproj files found"), "{notes:?}");
}

#[test]
fn notes_are_empty_for_repos_without_cpp_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    write(root, "tools/stray.vcxproj", &project(""));
    let mut scala = source_record(root, "src/main/scala/App.scala");
    scala.kind = FileKind::ScalaSource;
    assert!(msbuild_notes(root, &[scala]).is_empty());
}

/// MSBuild evaluates every property (through all imports) before any item
/// definition, so a macro defined in a props imported AFTER the item
/// definition that uses it still resolves.
#[test]
fn item_definitions_see_properties_defined_by_later_imports() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let sdk = mkdir(root, "sdk/include");
    let shared = mkdir(root, "Props/shared");
    let files = vec![source_record(root, "App/main.cpp")];
    write(
        root,
        "Props/Uses.props",
        &project(&compile_dirs(
            "$(EnGenSDKRoot)include;$(MSBuildThisFileDirectory)shared",
        )),
    );
    write(
        root,
        "Props/Defines.props",
        &project(
            "<PropertyGroup Label=\"UserMacros\">\
             <EnGenSDKRoot>$(MSBuildThisFileDirectory)..\\sdk\\</EnGenSDKRoot>\
             </PropertyGroup>",
        ),
    );
    write(
        root,
        "App/App.vcxproj",
        &project(
            "<Import Project=\"..\\Props\\Uses.props\" />\
             <Import Project=\"..\\Props\\Defines.props\" />",
        ),
    );

    let dirs = msbuild_include_dirs(root, &files);

    assert_eq!(sorted(dirs), sorted(vec![sdk, shared]));
}

/// Visual Studio solutions often keep projects in their own folder (no
/// sources there) pointing at the code via relative ClCompile paths; such
/// projects must still be found.
#[test]
fn projects_in_folders_without_sources_are_found() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    let api = mkdir(root, "baselibs/core/include");
    let files = vec![source_record(root, "baselibs/core/src/core.cpp")];
    write(
        root,
        "baselibs/msvc/core/core.vcxproj",
        &project(&compile_dirs("$(ProjectDir)../../core/include")),
    );

    let dirs = msbuild_include_dirs(root, &files);

    assert_eq!(dirs, vec![api]);
}

#[test]
fn atignored_and_hidden_folders_are_not_searched_for_projects() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();
    mkdir(root, "kept/include");
    mkdir(root, "skipped/include");
    let files = vec![source_record(root, "src/main.cpp")];
    write(root, ".atignore", "vendor/\n");
    write(root, "vendor/v.vcxproj", &project(&compile_dirs("$(SolutionDir)skipped/include")));
    write(root, ".git/g.vcxproj", &project(&compile_dirs("$(SolutionDir)skipped/include")));
    write(root, "tools/k.vcxproj", &project(&compile_dirs("$(SolutionDir)kept/include")));

    let dirs = msbuild_include_dirs(root, &files);

    assert_eq!(dirs, vec![root.join("kept/include")]);
}
