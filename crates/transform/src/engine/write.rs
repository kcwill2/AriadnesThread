//! Render + write of generated units (plan §3.6): `ExtractionResult` puts
//! every unit on disk through the plugin's `SourceExtractor` text hook,
//! creating each unit's parent directory chain (Scala targets are nested
//! package dirs, plan M23 D-M23-4).

use crate::{ExtractionResult, GeneratedUnit};
use ariadne_lang::{render_unit, SourceExtractor, SymbolBody};
use std::path::PathBuf;

impl ExtractionResult {
    pub fn write_to_disk(
        &self,
        output_dir: &std::path::Path,
        extractor: &dyn SourceExtractor,
    ) -> anyhow::Result<Vec<PathBuf>> {
        let mut written_paths = Vec::new();
        std::fs::create_dir_all(output_dir)?;

        for unit in &self.headers {
            written_paths.push(write_unit(output_dir, unit, extractor)?);
        }

        Ok(written_paths)
    }
}

fn write_unit(
    output_dir: &std::path::Path,
    unit: &GeneratedUnit,
    extractor: &dyn SourceExtractor,
) -> anyhow::Result<PathBuf> {
    let bodies: Vec<SymbolBody> = unit
        .symbols
        .iter()
        .map(|sym| SymbolBody {
            name: sym.name.clone(),
            body: sym.body.clone(),
        })
        .collect();
    let deps: Vec<String> = unit
        .required_deps
        .iter()
        .map(|dep| dep.display().to_string())
        .collect();

    let content = render_unit(extractor, &bodies, unit.primary_source_text.as_deref(), &deps);
    let file_path = output_dir.join(&unit.path);
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&file_path, content)?;
    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::tests::RecordingFakeExtractor;
    use crate::ExtractedSymbol;

    // The plugin-emitter byte-parity locks (test_write_to_disk,
    // test_write_to_disk_byte_parity_full_content) moved verbatim to
    // `crates/transform/tests/engine_write_test.rs` (M23 T3, D-M23-10):
    // they name the C++ plugin and live outside the guard walk scope.

    #[test]
    fn write_unit_passes_namespace_from_the_plugin_hook() {
        let fake = RecordingFakeExtractor::reporting_namespace("demo_ns");
        let source_text = "namespace demo { }\n";
        let unit = GeneratedUnit {
            path: std::path::PathBuf::from("ns_unit.h"),
            symbols: vec![ExtractedSymbol {
                name: "A".to_string(),
                qualified_id: "A".to_string(),
                kind: ariadne_index::SymbolKind::Class,
                body: "class A { int x; };".to_string(),
            }],
            required_deps: vec![],
            primary_source_text: Some(source_text.to_string()),
        };
        let result = ExtractionResult {
            plan_id: "PLAN-NS".to_string(),
            headers: vec![unit],
            errors: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        result.write_to_disk(dir.path(), &fake).unwrap();

        assert_eq!(fake.unit_namespace_inputs(), vec![source_text.to_string()]);
        assert_eq!(fake.extract_namespaces(), vec!["demo_ns".to_string()]);
        let written = std::fs::read_to_string(dir.path().join("ns_unit.h")).unwrap();
        assert!(written.starts_with("NS<demo_ns>"));
    }

    #[test]
    fn write_unit_passes_empty_namespace_without_source_text() {
        let fake = RecordingFakeExtractor::reporting_namespace("unused_ns");
        let unit = GeneratedUnit {
            path: std::path::PathBuf::from("bare_unit.h"),
            symbols: vec![ExtractedSymbol {
                name: "A".to_string(),
                qualified_id: "A".to_string(),
                kind: ariadne_index::SymbolKind::Class,
                body: "// Symbol: A (Class)".to_string(),
            }],
            required_deps: vec![],
            primary_source_text: None,
        };
        let result = ExtractionResult {
            plan_id: "PLAN-BARE".to_string(),
            headers: vec![unit],
            errors: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        result.write_to_disk(dir.path(), &fake).unwrap();

        assert!(fake.unit_namespace_inputs().is_empty());
        assert_eq!(fake.extract_namespaces(), vec![String::new()]);
    }

    #[test]
    fn write_to_disk_creates_nested_parent_directories() {
        let unit = GeneratedUnit {
            path: std::path::PathBuf::from("generated").join("nested").join("deep_x.h"),
            symbols: vec![ExtractedSymbol {
                name: "X".to_string(),
                qualified_id: "X".to_string(),
                kind: ariadne_index::SymbolKind::Class,
                body: "// Symbol: X (Class)".to_string(),
            }],
            required_deps: vec![],
            primary_source_text: None,
        };
        let result = ExtractionResult {
            plan_id: "PLAN-NEST".to_string(),
            headers: vec![unit],
            errors: vec![],
        };

        let dir = tempfile::tempdir().unwrap();
        let written = result
            .write_to_disk(dir.path(), &RecordingFakeExtractor::default())
            .unwrap();

        assert!(written[0].exists(), "nested parents must be created");
    }
}
