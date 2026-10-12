//! AR-27a byte-oracle + dual-format lock (lang-api home): the OLD enum-era
//! bytes for `TuAnalysis` vectors (the `tu_analyses.bin` warm shape), the
//! STRING-FRAMED `Vec<IncludeEdge>` warm bytes (`includes/edges.bin`,
//! string-framed since M24 Part A at the current epoch), and the bare-name
//! JSON of the legacy ids.
//!
//! RED-FREE GREEN-FIRST (shape ruling section 5 step 1): every fixture is the
//! VERBATIM output of the current, UNTOUCHED code (temp printer run before
//! the home move / alias / dual-format serde landed, since deleted; the
//! three `TuAnalysis` vectors, the `Vec<IncludeEdge>` warm bytes, and the
//! `FileRecord` pair were each dumped by the live `bincode::serialize` and
//! spliced in byte-for-byte). The same assertions MUST stay green after the
//! mutation: they are the oracle for every byte claim of AR-27a
//! (`tu_analyses.bin` keeps its u32 language index; the
//! `language_transparent_frame` exception keeps `includes/edges.bin` byte
//! frozen; JSON keeps bare names).
//!
//! Placement note (journal D1): this file also locks the `Vec<IncludeEdge>`
//! BINCODE bytes because the integration package where the field probe
//! lives has no bincode dependency (its Cargo.toml is outside the AR-27a
//! touch-set); the JSON locks live in `ar27_field_probe_test.rs`.

use ariadne_lang_api::identity::LanguageId;
use ariadne_lang_api::types::{
    IncludeEdge, IncludeKind, SourceLocation, SymbolId, SymbolRecord, TuAnalysis,
};
use ariadne_repo::{FileId, SourceLanguage};
use std::path::PathBuf;

/// One `TuAnalysis::default()` under the old enum: the language region is
/// the pinned u32 LE index (`[0,0,0,0]`) and the empty `extends_edges`
/// channel is PRESENT (positional formats never omit it - the FIX-F lock
/// family in `m23_bincode_serde_locks.rs` guards the same mechanism).
const OLD_TU_VEC_DEFAULT: &[u8] = &[
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0,
];

/// One `TuAnalysis` with a non-empty `extends_edges` channel and the default
/// (Cpp) language: language region `[0,0,0,0]`, channel appended last.
const OLD_TU_VEC_EXTENDS: &[u8] = &[
    1, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0,
    0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0,
    68, 101, 98, 117, 103, 124, 120, 54, 52, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 0, 0, 0, 0, 0, 0, 97, 1, 0,
    0, 0, 0, 0, 0, 0, 98,
];

/// Scala-language analysis with a sealed symbol and a non-empty extends
/// channel: language region is the pinned `[1,0,0,0]` Scala index.
const OLD_TU_VEC_SCALA_SEALED: &[u8] = &[
    1, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
    0, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0,
    83, 99, 97, 108, 97, 1, 0, 0, 0, 0, 0, 0,
    0, 7, 0, 0, 0, 0, 0, 0, 0, 112, 46, 83,
    104, 97, 112, 101, 7, 0, 0, 0, 0, 0, 0, 0,
    112, 46, 83, 104, 97, 112, 101, 10, 0, 0, 0, 7,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
    6, 0, 0, 0, 0, 0, 0, 0, 115, 101, 97, 108,
    101, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0,
    0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 8, 0,
    0, 0, 0, 0, 0, 0, 112, 46, 67, 105, 114, 99,
    108, 101, 7, 0, 0, 0, 0, 0, 0, 0, 112, 46,
    83, 104, 97, 112, 101,
];

/// `Vec<IncludeEdge>` in today's WARM shape (epoch 2.2.0): the language is
/// STRING-FRAMED (u64 LE length + UTF-8 bytes) for every id - `Cpp`,
/// `Scala`, and the open id `Fortran`. The global positional arm would move
/// these bytes, which is why `IncludeEdge.language` carries the
/// field-level `language_transparent_frame` exception (shape ruling
/// section 2.7); this fixture is its byte freeze.
const OLD_INCLUDE_VEC_BIN: &[u8] = &[
    3, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 1, 20, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0,
    0, 0, 68, 101, 98, 117, 103, 124, 120, 54, 52, 4, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 13, 0, 0, 0,
    0, 0, 0, 0, 116, 104, 105, 110, 103, 95, 116, 121, 112, 101, 115, 46, 104, 1, 21, 0, 0, 0, 0, 0,
    0, 0, 105, 110, 99, 108, 117, 100, 101, 47, 116, 104, 105, 110, 103, 95, 116, 121, 112, 101,
    115, 46, 104, 3, 0, 0, 0, 0, 0, 0, 0, 67, 112, 112, 11, 0, 0, 0, 0, 0, 0, 0, 1, 22, 0, 0, 0, 0,
    0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 68, 101, 98, 117, 103, 124, 120, 54, 52, 4, 0, 0, 0, 1, 0, 0,
    0, 0, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 79, 114, 100, 101, 114, 46, 115, 99, 97, 108, 97, 1,
    21, 0, 0, 0, 0, 0, 0, 0, 105, 110, 99, 108, 117, 100, 101, 47, 116, 104, 105, 110, 103, 95, 116,
    121, 112, 101, 115, 46, 104, 5, 0, 0, 0, 0, 0, 0, 0, 83, 99, 97, 108, 97, 12, 0, 0, 0, 0, 0, 0,
    0, 1, 23, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 68, 101, 98, 117, 103, 124, 120, 54, 52,
    4, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 115, 111, 108, 118, 101, 114, 46,
    102, 57, 48, 1, 21, 0, 0, 0, 0, 0, 0, 0, 105, 110, 99, 108, 117, 100, 101, 47, 116, 104, 105,
    110, 103, 95, 116, 121, 112, 101, 115, 46, 104, 7, 0, 0, 0, 0, 0, 0, 0, 70, 111, 114, 116, 114,
    97, 110,
];

fn record_with_modifiers(name: &str, modifiers: Vec<&str>) -> SymbolRecord {
    SymbolRecord {
        id: SymbolId::new(name),
        name: name.to_string(),
        kind: ariadne_lang_api::types::SymbolKind::Trait,
        owning_file: FileId(7),
        declarations: Vec::new(),
        definition: None,
        modifiers: modifiers.into_iter().map(String::from).collect(),
    }
}

fn default_analysis() -> TuAnalysis {
    TuAnalysis::default()
}

fn extends_analysis() -> TuAnalysis {
    let mut analysis = TuAnalysis::default();
    analysis.tu_file_id = FileId(3);
    analysis.context_name = "Debug|x64".to_string();
    analysis.extends_edges = vec![(SymbolId::new("a"), SymbolId::new("b"))];
    analysis
}

fn sealed_scala_analysis() -> TuAnalysis {
    let mut analysis = TuAnalysis::default();
    analysis.tu_file_id = FileId(4);
    analysis.context_name = "Scala".to_string();
    analysis.symbols = vec![record_with_modifiers("p.Shape", vec!["sealed"])];
    analysis.extends_edges = vec![(SymbolId::new("p.Circle"), SymbolId::new("p.Shape"))];
    analysis.language = SourceLanguage::Scala;
    analysis
}

fn edge(source: u64, destination: u64, raw_path: &str, language: LanguageId) -> IncludeEdge {
    IncludeEdge {
        source: FileId(source),
        destination: Some(FileId(destination)),
        context_name: "Debug|x64".to_string(),
        source_location: SourceLocation::new(4, 1),
        kind: IncludeKind::Local,
        raw_path: raw_path.to_string(),
        resolved_path: Some(PathBuf::from("include/thing_types.h")),
        language,
    }
}

fn warm_edges() -> Vec<IncludeEdge> {
    vec![
        edge(10, 20, "thing_types.h", LanguageId::Cpp),
        edge(11, 22, "Order.scala", LanguageId::Scala),
        edge(12, 23, "solver.f90", LanguageId::new("Fortran")),
    ]
}

#[test]
fn old_bytes_tu_analysis_vectors_load_and_rewrite_identically() {
    for (fixture, expected) in [
        (OLD_TU_VEC_DEFAULT, vec![default_analysis()]),
        (OLD_TU_VEC_EXTENDS, vec![extends_analysis()]),
        (OLD_TU_VEC_SCALA_SEALED, vec![sealed_scala_analysis()]),
    ] {
        let decoded: Vec<TuAnalysis> = bincode::deserialize(fixture).unwrap_or_else(|err| {
            panic!("old tu_analyses.bin-era bytes must load at the same epoch: {err}")
        });
        assert_eq!(decoded, expected);
        assert_eq!(
            bincode::serialize(&expected).unwrap(),
            fixture,
            "the CURRENT code rewrites the old bytes byte-identically \
             (tu_analyses.bin language stays a u32 index)"
        );
    }
}

#[test]
fn include_edge_bincode_warm_shape_stays_string_framed() {
    // Shape ruling section 2.7: `includes/edges.bin` carries the language
    // STRING-FRAMED at the current epoch. The field-level exception module
    // must reproduce these bytes in both eras and after the dual-format
    // global arm lands (a moved byte here would hard-error every warm blob).
    let expected = warm_edges();
    let decoded: Vec<IncludeEdge> = bincode::deserialize(OLD_INCLUDE_VEC_BIN)
        .unwrap_or_else(|err| panic!("string-framed edges.bin bytes must load: {err}"));
    assert_eq!(decoded, expected, "string-framed language bytes load for all ids");
    assert_eq!(bincode::serialize(&expected).unwrap(), OLD_INCLUDE_VEC_BIN);
}

#[test]
fn legacy_ids_json_as_bare_names() {
    for (id, json) in [
        (LanguageId::Cpp, "\"Cpp\""),
        (LanguageId::Scala, "\"Scala\""),
        (LanguageId::Other, "\"Other\""),
    ] {
        assert_eq!(serde_json::to_string(&id).unwrap(), json);
        assert_eq!(serde_json::from_str::<LanguageId>(json).unwrap(), id);
    }
}
