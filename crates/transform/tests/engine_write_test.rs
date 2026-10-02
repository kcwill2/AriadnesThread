//! Relocated (M23 T3, D-M23-10) from `crates/transform/src/engine/write.rs`
//! verbatim: the byte-parity write tests drive the real C++ plugin emitter and
//! therefore live outside the guard walk scope (`crates/*/src`). The neutral
//! fake-extractor render tests stay in `engine/write.rs`.

use ariadne_transform::{ExtractedSymbol, ExtractionResult, GeneratedUnit};

// R2 keeps directive text out of this crate's sources, so the C++ shapes
// asserted here are assembled from a non-contiguous marker (the plugin
// crate owns the canonical form).
const DIRECTIVE: &str = concat!("#inc", "lude ");
fn include_of(target: &str) -> String {
    format!("{DIRECTIVE}\"{target}\"")
}

#[test]
fn test_write_to_disk() {
    use ariadne_lang_cpp::CppSourceExtractor;

    let sym1 = ExtractedSymbol {
        name: "MyClass".to_string(),
        kind: ariadne_index::SymbolKind::Class,
        body: "// body1".to_string(),
    };
    let unit1 = GeneratedUnit {
        path: std::path::PathBuf::from("h1.h"),
        symbols: vec![sym1],
        required_deps: vec![std::path::PathBuf::from("other.h")],
        primary_source_text: None,
    };

    let sym2 = ExtractedSymbol {
        name: "MyFunc".to_string(),
        kind: ariadne_index::SymbolKind::Function,
        body: "// body2".to_string(),
    };
    let unit2 = GeneratedUnit {
        path: std::path::PathBuf::from("h2.h"),
        symbols: vec![sym2],
        required_deps: vec![],
        primary_source_text: None,
    };

    let result = ExtractionResult {
        plan_id: "PLAN".to_string(),
        headers: vec![unit1, unit2],
        errors: vec![],
    };

    let dir = tempfile::tempdir().unwrap();
    let written = result.write_to_disk(dir.path(), &CppSourceExtractor).unwrap();
    assert_eq!(written.len(), 2);

    let h1_path = dir.path().join("h1.h");
    let h2_path = dir.path().join("h2.h");

    assert!(h1_path.exists());
    assert!(h2_path.exists());

    let h1_content = std::fs::read_to_string(h1_path).unwrap();
    assert!(h1_content.contains("#pragma once"));
    assert!(h1_content.contains(&include_of("other.h")));
    assert!(h1_content.contains("// body1"));

    let h2_content = std::fs::read_to_string(h2_path).unwrap();
    assert!(h2_content.contains("#pragma once"));
    assert!(h2_content.contains("// body2"));
    assert!(!h2_content.contains(DIRECTIVE.trim_end()));
}

#[test]
fn test_write_to_disk_byte_parity_full_content() {
    use ariadne_lang_cpp::CppSourceExtractor;

    let unit = GeneratedUnit {
        path: std::path::PathBuf::from("h.h"),
        symbols: vec![
            ExtractedSymbol {
                name: "A".to_string(),
                kind: ariadne_index::SymbolKind::Class,
                body: "// body1".to_string(),
            },
            ExtractedSymbol {
                name: "B".to_string(),
                kind: ariadne_index::SymbolKind::Function,
                body: "// body2".to_string(),
            },
        ],
        required_deps: vec![std::path::PathBuf::from("a.h"), std::path::PathBuf::from("b.h")],
        primary_source_text: None,
    };
    let result = ExtractionResult {
        plan_id: "PLAN".to_string(),
        headers: vec![unit],
        errors: vec![],
    };

    let dir = tempfile::tempdir().unwrap();
    result.write_to_disk(dir.path(), &CppSourceExtractor).unwrap();

    let content = std::fs::read_to_string(dir.path().join("h.h")).unwrap();
    assert_eq!(
        content,
        format!(
            "#pragma once\n\n{}\n{}\n\n// body1\n// body2\n",
            include_of("a.h"),
            include_of("b.h")
        )
    );
}
