//! `.atignore` exclude globs: an optional file at the analyzed root whose
//! gitignore-flavoured patterns drop paths from repository discovery.
//! Motivating case: a vendored boost tree whose ~14k unit-test `.cpp` files
//! each became a synthesized translation unit.

use ariadne_repo::discover_repository_files;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn write_file(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, body).unwrap();
}

fn discovered_paths(root: &Path) -> Vec<String> {
    discover_repository_files(root)
        .expect("discovery succeeds")
        .into_iter()
        .map(|record| record.path.to_string_lossy().replace('\\', "/"))
        .collect()
}

/// A boost-shaped tree: real library sources, headers, tests and examples.
fn boost_shaped_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "boost/config.hpp", "#pragma once\n");
    write_file(root, "libs/system/src/error_code.cpp", "int a;\n");
    write_file(root, "libs/system/test/error_code_test.cpp", "int b;\n");
    write_file(
        root,
        "libs/system/test/deep/nested/more_test.cpp",
        "int c;\n",
    );
    write_file(root, "libs/regex/example/snippet.cpp", "int d;\n");
    write_file(root, "libs/regex/src/regex.cpp", "int e;\n");
    write_file(root, "doc/sample.cpp", "int f;\n");
    write_file(root, "tools/bcp/scratch.cpp", "int g;\n");
    dir
}

#[test]
fn without_atignore_discovery_is_unchanged() {
    let repo = boost_shaped_repo();
    assert_eq!(discovered_paths(repo.path()).len(), 8);
}

#[test]
fn atignore_prunes_matching_directories_and_files() {
    let repo = boost_shaped_repo();
    let atignore = "# boost: keep headers and real library sources only\n\
                    \n\
                    libs/*/test/**\n\
                    libs/*/example/\n\
                    /doc\n\
                    scratch.cpp";
    write_file(repo.path(), ".atignore", atignore);
    assert_eq!(
        discovered_paths(repo.path()),
        vec![
            "boost/config.hpp",
            "libs/regex/src/regex.cpp",
            "libs/system/src/error_code.cpp",
        ]
    );
}

#[test]
fn trailing_slash_pattern_matches_directories_only() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "src/test.cpp", "int a;\n");
    write_file(root, "test/unit.cpp", "int b;\n");
    write_file(root, ".atignore", "test/\n");
    assert_eq!(discovered_paths(root), vec!["src/test.cpp"]);
}

#[test]
fn double_star_matches_zero_or_more_segments() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "bench/top.cpp", "int a;\n");
    write_file(root, "a/b/bench/deep.cpp", "int b;\n");
    write_file(root, "a/keep.cpp", "int c;\n");
    write_file(root, ".atignore", "**/bench/**\n");
    assert_eq!(discovered_paths(root), vec!["a/keep.cpp"]);
}

#[test]
fn crlf_atignore_lines_are_honoured() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    write_file(root, "keep.cpp", "int a;\n");
    write_file(root, "skip/one.cpp", "int b;\n");
    write_file(root, ".atignore", "# windows editor\r\nskip/\r\n");
    assert_eq!(discovered_paths(root), vec!["keep.cpp"]);
}
