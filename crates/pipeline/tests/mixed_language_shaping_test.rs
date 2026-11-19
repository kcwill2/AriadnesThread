//! Mixed C++/Scala repos: build discovery must shape every language's
//! units, not only the language whose build marker sits at the root.
//!
//! The fixture mirrors a Bazel-built Visual Studio + Scala monorepo with
//! leftover sbt files: a root `build.sbt` (which used to hand the whole
//! repo to the Scala shaper), `.vcxproj` files nested under feature
//! folders, and a shared `.props` sheet carrying the include dirs.

use ariadne_build::BuildDiscoverer;
use ariadne_pipeline::build_default_registry;
use ariadne_repo::Repository;
use std::fs;
use std::path::Path;

fn write(root: &Path, rel: &str, contents: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn mixed_monorepo(root: &Path) {
    write(root, "build.sbt", "lazy val root = project\n");
    write(root, "features/build.sbt", "lazy val features = project\n");
    write(
        root,
        "features/Features.props",
        "<Project><ItemDefinitionGroup><ClCompile>\
         <AdditionalIncludeDirectories>$(MSBuildThisFileDirectory)shared\\include\
         </AdditionalIncludeDirectories></ClCompile></ItemDefinitionGroup></Project>",
    );
    write(
        root,
        "features/radar/radar.vcxproj",
        "<Project><Import Project=\"..\\Features.props\" /></Project>",
    );
    write(root, "features/shared/include/shared/track.h", "struct Track {};\n");
    write(
        root,
        "features/radar/src/radar.cpp",
        "#include \"shared/track.h\"\nTrack radar_track;\n",
    );
    write(
        root,
        "features/sim/src/main/scala/sim/Sim.scala",
        "package sim\nobject Sim\n",
    );
}

#[test]
fn a_root_sbt_file_no_longer_hides_the_cpp_units() {
    let temp_dir = tempfile::tempdir().unwrap();
    mixed_monorepo(temp_dir.path());
    let repository = Repository::locate(temp_dir.path()).unwrap();
    let discovery = repository.discover().unwrap();
    let registry = build_default_registry();

    let build = BuildDiscoverer::new(&repository.root)
        .with_repo_discovery(&discovery)
        .with_unit_shaper(registry.unit_shaper_for_repo(&repository.root))
        .discover()
        .unwrap();

    let unit_files: Vec<String> = build
        .translation_units
        .iter()
        .map(|unit| unit.source_path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(unit_files, vec!["radar.cpp", "Sim.scala"]);

    let cpp_context = build
        .contexts
        .iter()
        .find(|context| context.name == "Default|x64")
        .expect("the C++ units get their own context");
    assert_eq!(cpp_context.compiler, "cl.exe");
    // Compared by spelling tail: the evaluator reports the plain `C:\...`
    // form while `repository.root` is canonical (verbatim on Windows).
    assert!(
        cpp_context.include_dirs.iter().any(|dir| {
            dir.to_string_lossy()
                .replace('\\', "/")
                .ends_with("/features/shared/include")
        }),
        "the .props include dir reaches the C++ context: {:?}",
        cpp_context.include_dirs
    );

    let scala_context = build
        .contexts
        .iter()
        .find(|context| context.name == "features")
        .expect("the Scala unit keeps its sbt-named context");
    assert_eq!(scala_context.compiler, "scalac");
    assert!(scala_context.include_dirs.is_empty());
}
