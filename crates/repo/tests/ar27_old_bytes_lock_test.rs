//! AR-27a byte-oracle lock (repo home): the OLD enum-era bytes for
//! `FileRecord`/`SourceLanguage` and the three legacy language ids.
//!
//! RED-FREE GREEN-FIRST (shape ruling section 5 step 1): every fixture here
//! came from the live serializers of the UNTOUCHED types BEFORE the home
//! move / alias / dual-format serde landed (now-deleted temp printer, then
//! mechanically cross-checked against the byte-locked repo fixtures; this
//! file's own load/rewrite equality proved it green pre-mutation;
//! the values agree with `tasks/ar-27-handoff-journal.md` section 5:
//! Cpp=[0,0,0,0], Scala=[1,0,0,0], Other=[2,0,0,0]). The same assertions
//! MUST stay green after the mutation: they are the oracle for every
//! byte claim of AR-27a (`repository/files.bin` keeps its u32 language
//! index; every JSON surface keeps its bare name).

use ariadne_repo::{
    FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage,
};
use std::path::PathBuf;

/// bincode 1.3.3 DEFAULT options (fixint, little-endian): `Vec` = u64 LE
/// length prefix. The empty vector is the stream's zero-shape witness.
const OLD_VEC_EMPTY: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0];

/// Two records: language `Cpp` (the `#[serde(default)]` value, written
/// explicitly because positional formats always carry the field) and
/// `Scala`. The language region of each record is the pinned 4-byte u32 LE
/// variant index - exactly what the dual-format positional arm must keep
/// writing for legacy ids (shape ruling section 2.2).
const OLD_VEC_PAIR: &[u8] = &[
    2, 0, 0, 0, 0, 0, 0, 0, 60, 234, 20, 198, 243, 26, 210, 52, 12, 0, 0, 0, 0, 0, 0, 0, 115, 114,
    99, 47, 109, 97, 105, 110, 46, 99, 112, 112, 12, 0, 0, 0, 0, 0, 0, 0, 115, 114, 99, 47, 109,
    97, 105, 110, 46, 99, 112, 112, 1, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
    104, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 48, 38, 156, 249, 212, 166, 124, 98, 14, 0, 0, 0, 0,
    0, 0, 0, 115, 114, 99, 47, 109, 97, 105, 110, 46, 115, 99, 97, 108, 97, 14, 0, 0, 0, 0, 0, 0,
    0, 115, 114, 99, 47, 109, 97, 105, 110, 46, 115, 99, 97, 108, 97, 2, 0, 0, 0, 7, 0, 0, 0, 0, 0,
    0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 104, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
];

/// Legacy id positional bytes: the pinned u32 LE variant indices
/// (shape ruling section 2.2/2.5). Tag 3 is UNREACHABLE in old bytes.
const OLD_ID_CPP: &[u8] = &[0, 0, 0, 0];
const OLD_ID_SCALA: &[u8] = &[1, 0, 0, 0];
const OLD_ID_OTHER: &[u8] = &[2, 0, 0, 0];

fn record(rel: &str, kind: FileKind, language: SourceLanguage) -> FileRecord {
    let path = PathBuf::from(rel);
    FileRecord {
        id: FileId::from_relative_path(rel),
        path: path.clone(),
        canonical_path: path,
        kind,
        size: 7,
        content_hash: "h".to_string(),
        ownership: FileOwnership::Repository,
        optimization_policy: OptimizationPolicy::Optimizable,
        language,
    }
}

fn cpp_record() -> FileRecord {
    record("src/main.cpp", FileKind::Source, SourceLanguage::Cpp)
}

fn scala_record() -> FileRecord {
    record("src/main.scala", FileKind::ScalaSource, SourceLanguage::Scala)
}

#[test]
fn old_bytes_empty_file_record_vector_loads_and_rewrites_identically() {
    let decoded: Vec<FileRecord> = bincode::deserialize(OLD_VEC_EMPTY).unwrap();
    assert!(decoded.is_empty());
    assert_eq!(bincode::serialize(&decoded).unwrap(), OLD_VEC_EMPTY);
}

#[test]
fn old_bytes_two_record_vector_loads_and_rewrites_identically() {
    let expected = vec![cpp_record(), scala_record()];
    let decoded: Vec<FileRecord> = bincode::deserialize(OLD_VEC_PAIR).unwrap();
    assert_eq!(decoded, expected, "the old u32-index language bytes load");
    assert_eq!(
        bincode::serialize(&expected).unwrap(),
        OLD_VEC_PAIR,
        "the CURRENT code rewrites the old bytes byte-identically \
         (files.bin language stays a u32 index)"
    );
}

#[test]
fn file_record_json_bytes_are_bare_language_names() {
    let cpp_json = serde_json::to_string(&cpp_record()).unwrap();
    assert_eq!(
        cpp_json,
        "{\"id\":3806134269384124988,\"path\":\"src/main.cpp\",\"canonical_path\":\"src/main.cpp\",\
         \"kind\":\"Source\",\"size\":7,\"content_hash\":\"h\",\"ownership\":\"Repository\",\
         \"optimization_policy\":\"Optimizable\",\"language\":\"Cpp\"}"
    );
    let scala_json = serde_json::to_string(&scala_record()).unwrap();
    assert_eq!(
        scala_json,
        "{\"id\":7096730546480096816,\"path\":\"src/main.scala\",\"canonical_path\":\"src/main.scala\",\
         \"kind\":\"ScalaSource\",\"size\":7,\"content_hash\":\"h\",\"ownership\":\"Repository\",\
         \"optimization_policy\":\"Optimizable\",\"language\":\"Scala\"}"
    );
    assert_eq!(serde_json::from_str::<FileRecord>(&cpp_json).unwrap(), cpp_record());
    assert_eq!(
        serde_json::from_str::<FileRecord>(&scala_json).unwrap(),
        scala_record()
    );

    // Additive-field contract (plan 3.5): manifests written before the
    // `language` key existed keep reading back as the default (Cpp).
    let without_language = cpp_json.replace(",\"language\":\"Cpp\"", "");
    let back: FileRecord = serde_json::from_str(&without_language).unwrap();
    assert_eq!(back, cpp_record());
    assert_eq!(back.language, SourceLanguage::default());
}

#[test]
fn legacy_ids_are_bare_names_in_json_and_u32_indices_positionally() {
    for (id, json, positional) in [
        (SourceLanguage::Cpp, "\"Cpp\"", OLD_ID_CPP),
        (SourceLanguage::Scala, "\"Scala\"", OLD_ID_SCALA),
        (SourceLanguage::Other, "\"Other\"", OLD_ID_OTHER),
    ] {
        assert_eq!(serde_json::to_string(&id).unwrap(), json);
        assert_eq!(serde_json::from_str::<SourceLanguage>(json).unwrap(), id);
        assert_eq!(bincode::serialize(&id).unwrap(), positional);
        assert_eq!(bincode::deserialize::<SourceLanguage>(positional).unwrap(), id);
    }
}
