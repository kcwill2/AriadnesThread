//! [`CppUnitShaper`] — the C++ half of build discovery (plan M18.4).
//!
//! Every literal here was lifted from `ariadne-build`'s old
//! `infer_default_discovery` / `infer_context_name` so the pipeline's
//! `BuildDiscoverer` keeps no C++ vocabulary: unit extensions, the
//! `cl.exe /c <source>` synthesized invocation, the `Default|x64` context
//! with its `NDEBUG` define and `include/` search dir, and the
//! `DEBUG`/`NDEBUG`-driven context naming.

use ariadne_lang::{UnitDescriptor, UnitKind, UnitShaper};
use ariadne_repo::FileRecord;
use std::path::{Path, PathBuf};

/// Extension test mirroring the old `infer_default_discovery` filter
/// byte-for-byte (case-sensitive, on the repo-relative path).
fn is_cpp_source(record: &FileRecord) -> bool {
    record
        .path
        .extension()
        .is_some_and(|ext| ext == "cpp" || ext == "cc" || ext == "cxx" || ext == "c")
}

fn default_context_defines() -> Vec<String> {
    // Today's fallback context always carried NDEBUG.
    vec!["NDEBUG".to_string()]
}

fn default_include_dirs(root: &Path) -> Vec<PathBuf> {
    let include_dir = root.join("include");
    if include_dir.exists() {
        vec![include_dir]
    } else {
        Vec::new()
    }
}

fn synthesized_invocation(canonical_path: &Path) -> Vec<String> {
    // Byte-identical to the old `vec!["cl.exe", "/c", <lossy canonical>]`.
    vec![
        "cl.exe".to_string(),
        "/c".to_string(),
        canonical_path.to_string_lossy().to_string(),
    ]
}

/// Shapes every discovered C/C++ source into one translation unit of the
/// synthetic `Default|x64` context — the MSVC-less-repo default.
pub struct CppUnitShaper;

impl UnitShaper for CppUnitShaper {
    /// The C++ build manifests of plan §3.4. Of these, today's discovery
    /// machinery actually consumes only `compile_commands.json` (directly,
    /// plus its `build/` and `out/` placements); the others mark a repo as
    /// C++ for registry-driven routing (M18.3+).
    fn build_markers(&self) -> Vec<&'static str> {
        vec!["CMakeLists.txt", "compile_commands.json", ".vcxproj"]
    }

    fn default_compiler(&self) -> &str {
        "cl.exe"
    }

    fn shape_units(&self, repo_root: &Path, files: &[FileRecord]) -> Vec<UnitDescriptor> {
        let context_name = self.context_name_for_defines(&[]);
        let defines = default_context_defines();
        let include_dirs = default_include_dirs(repo_root);

        files
            .iter()
            .filter(|record| is_cpp_source(record))
            .map(|record| descriptor_for(record, &context_name, &defines, &include_dirs))
            .collect()
    }

    /// The old `infer_context_name` naming semantics, now plugin knowledge.
    fn context_name_for_defines(&self, defines: &[String]) -> String {
        if defines.iter().any(|d| d == "DEBUG" || d == "_DEBUG") {
            "Debug|x64".to_string()
        } else if defines.iter().any(|d| d == "NDEBUG") {
            "Release|x64".to_string()
        } else {
            "Default|x64".to_string()
        }
    }
}

fn descriptor_for(
    record: &FileRecord,
    context_name: &str,
    defines: &[String],
    include_dirs: &[PathBuf],
) -> UnitDescriptor {
    UnitDescriptor {
        file_id: record.id,
        language: ariadne_lang::LanguageId::Cpp,
        unit_kind: UnitKind::TranslationUnit,
        context_name: context_name.to_string(),
        aggregation_target: None,
        build_spec: synthesized_invocation(&record.canonical_path),
        defines: defines.to_vec(),
        include_dirs: include_dirs.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_lang::LanguageId;
    use ariadne_repo::{FileId, FileKind, FileOwnership, OptimizationPolicy, SourceLanguage};

    fn record(root: &Path, rel: &str) -> FileRecord {
        let rel_path = PathBuf::from(rel);
        FileRecord {
            id: FileId::from_relative_path(rel),
            path: rel_path.clone(),
            canonical_path: root.join(&rel_path),
            kind: FileKind::Source,
            size: 0,
            content_hash: String::new(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
            language: SourceLanguage::Cpp,
        }
    }

    fn basic_fixture_files(root: &Path) -> Vec<FileRecord> {
        vec![
            record(root, "include/heavy.h"),
            record(root, "include/thing.h"),
            record(root, "include/thing_types.h"),
            record(root, "src/thing.cpp"),
            record(root, "tests/consumer.cpp"),
        ]
    }

    fn expensive_fixture_files(root: &Path) -> Vec<FileRecord> {
        vec![
            record(root, "include/heavy.h"),
            record(root, "include/thing.h"),
            record(root, "src/consumer1.cpp"),
            record(root, "src/consumer2.cpp"),
        ]
    }

    /// Parity table against the removed `infer_default_discovery`: every
    /// discovered `.cpp` becomes one `Default|x64` / `cl.exe /c <canonical>`
    /// unit; non-sources are skipped.
    #[test]
    fn shape_units_reproduces_infer_default_discovery_on_basic_fixture() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("include")).unwrap();

        let units = CppUnitShaper.shape_units(root, &basic_fixture_files(root));

        let parity = [
            ("src/thing.cpp", "thing.cpp"),
            ("tests/consumer.cpp", "consumer.cpp"),
        ];
        assert_eq!(units.len(), parity.len(), "headers are not units");
        for (unit, (rel, _)) in units.iter().zip(parity.iter()) {
            assert_eq!(unit.context_name, "Default|x64");
            assert_eq!(unit.language, LanguageId::Cpp);
            assert_eq!(unit.unit_kind, UnitKind::TranslationUnit);
            assert_eq!(unit.file_id, FileId::from_relative_path(rel));
            assert_eq!(
                unit.build_spec,
                vec![
                    "cl.exe".to_string(),
                    "/c".to_string(),
                    root.join(rel).to_string_lossy().to_string(),
                ]
            );
            assert_eq!(unit.defines, vec!["NDEBUG".to_string()]);
            assert_eq!(unit.include_dirs, vec![root.join("include")]);
            assert_eq!(CppUnitShaper.default_compiler(), "cl.exe");
        }
    }

    #[test]
    fn shape_units_reproduces_infer_default_discovery_on_expensive_fixture() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("include")).unwrap();

        let units = CppUnitShaper.shape_units(root, &expensive_fixture_files(root));

        assert_eq!(units.len(), 2);
        assert!(units.iter().all(|u| u.context_name == "Default|x64"
            && u.build_spec[0] == "cl.exe"
            && u.build_spec[1] == "/c"
            && u.defines == ["NDEBUG".to_string()]
            && u.include_dirs == [root.join("include")]));
    }

    #[test]
    fn shape_units_accepts_all_source_extensions_and_skips_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        let files = vec![
            record(root, "a.cpp"),
            record(root, "b.cc"),
            record(root, "c.cxx"),
            record(root, "d.c"),
            record(root, "e.h"),
            record(root, "f.hpp"),
            record(root, "notes.txt"),
            record(root, "g.CPP"),
        ];
        let units = CppUnitShaper.shape_units(root, &files);

        assert_eq!(units.len(), 4, "cpp|cc|cxx|c only, headers excluded");
        // No include/ dir on this root: no search dirs, exactly as today.
        assert!(units.iter().all(|u| u.include_dirs.is_empty()));
    }

    #[test]
    fn shape_units_without_matching_files_yields_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let units = CppUnitShaper.shape_units(dir.path(), &[]);
        assert!(units.is_empty());
    }

    #[test]
    fn build_markers_cover_the_cpp_build_manifests() {
        let markers = CppUnitShaper.build_markers();
        assert_eq!(
            markers,
            vec!["CMakeLists.txt", "compile_commands.json", ".vcxproj"]
        );
    }

    #[test]
    fn context_name_matches_old_infer_context_name_table() {
        // Parity with build::infer_context_name (locked by m2 + the
        // characterization suite).
        assert_eq!(
            CppUnitShaper.context_name_for_defines(&["DEBUG".to_string()]),
            "Debug|x64"
        );
        assert_eq!(
            CppUnitShaper.context_name_for_defines(&["_DEBUG".to_string()]),
            "Debug|x64"
        );
        assert_eq!(
            CppUnitShaper.context_name_for_defines(&["NDEBUG".to_string()]),
            "Release|x64"
        );
        assert_eq!(
            CppUnitShaper.context_name_for_defines(&["OTHER".to_string()]),
            "Default|x64"
        );
        assert_eq!(CppUnitShaper.context_name_for_defines(&[]), "Default|x64");
    }
}
