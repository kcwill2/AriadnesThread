//! UTA-T2 pin lock for the C++ extension case matrix (handoff item 12,
//! audit row B3-C). Three surfaces classify C++ file extensions and they do
//! NOT agree on casing: the detector lowercases, the planning aggregation
//! filter is case-sensitive, and the shaper source filter is case-sensitive.
//! Behavior is pinned, not changed; the divergence is reported for an owner
//! decision.

use crate::planning::cpp_aggregation_filter;
use crate::plugin::CppDetector;
use crate::shaper::CppUnitShaper;
use ariadne_lang::{LanguageDetector, SourceFileKind, UnitShaper};
use ariadne_repo::{FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage};
use std::path::{Path, PathBuf};

fn shaper_record(root: &Path, rel: &str) -> FileRecord {
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

#[test]
fn pin_cpp_extension_case_matrix() {
    // Detector: lowercases the extension, so every case variant is admitted.
    assert_eq!(CppDetector.detect(Path::new("a.H")), SourceFileKind::Header);
    assert_eq!(CppDetector.detect(Path::new("a.CPP")), SourceFileKind::Source);
    assert_eq!(CppDetector.detect(Path::new("a.Scala")), SourceFileKind::Other);

    // Planning aggregation filter: case-sensitive, uppercase never passes.
    assert!(cpp_aggregation_filter(Path::new("a.h")), "control: lowercase passes");
    assert!(!cpp_aggregation_filter(Path::new("a.H")));
    assert!(!cpp_aggregation_filter(Path::new("a.HPP")));
    assert!(!cpp_aggregation_filter(Path::new("a.Scala")));

    // Shaper source filter (driven through the public trait entry): also
    // case-sensitive; `.CPP` is skipped while lowercase `.cpp` is shaped.
    let dir = tempfile::tempdir().expect("tempdir for the shaper leg");
    let root = dir.path();
    let records = vec![
        shaper_record(root, "keep.cpp"),
        shaper_record(root, "skip.CPP"),
        shaper_record(root, "skip.H"),
        shaper_record(root, "skip.Scala"),
    ];
    let units = CppUnitShaper.shape_units(root, &records);
    assert_eq!(units.len(), 1, "the shaper admits lowercase cpp only");
    assert_eq!(units[0].file_id, FileId::from_relative_path("keep.cpp"));
}
