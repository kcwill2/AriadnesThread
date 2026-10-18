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
        // Write containment (UTA-T4): every unit stays inside output_dir.
        // Checked across the whole batch before any IO, so a rejected batch
        // never leaves a partial write behind.
        for unit in &self.headers {
            reject_escaping_unit_path(&unit.path)?;
        }

        let mut written_paths = Vec::new();
        std::fs::create_dir_all(output_dir)?;

        for unit in &self.headers {
            written_paths.push(write_unit(output_dir, unit, extractor)?);
        }

        Ok(written_paths)
    }
}

/// A unit path must be relative and free of parent-dir hops: `Path::join`
/// lets an absolute path replace `output_dir` entirely, and `..` components
/// reach above it. Both shapes are rejected naming the offending path.
fn reject_escaping_unit_path(unit_path: &std::path::Path) -> anyhow::Result<()> {
    let escapes = unit_path.is_absolute()
        || unit_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir));
    if escapes {
        anyhow::bail!(
            "refusing to write generated unit outside the output dir: {}",
            unit_path.display()
        );
    }
    Ok(())
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

    /// Unit factory for the containment tests: one symbol, caller-chosen path.
    fn unit_at_path(path: std::path::PathBuf) -> GeneratedUnit {
        GeneratedUnit {
            path,
            symbols: vec![ExtractedSymbol {
                name: "S".to_string(),
                qualified_id: "S".to_string(),
                kind: ariadne_index::SymbolKind::Class,
                body: "// Symbol: S (Class)".to_string(),
            }],
            required_deps: vec![],
            primary_source_text: None,
        }
    }

    #[test]
    fn write_to_disk_rejects_absolute_unit_path_before_any_io() {
        // UTA-T4 item 2 (write containment): a unit.path that is absolute
        // silently replaces output_dir under join - it must be rejected with
        // the offending path named, before a single byte of IO happens.
        let dir = tempfile::tempdir().unwrap();
        let leak_path = std::env::current_dir()
            .expect("cwd readable")
            .join("uta_t4_absolute_leak.h");
        let _ = std::fs::remove_file(&leak_path);

        let result = ExtractionResult {
            plan_id: "PLAN-ABS".to_string(),
            headers: vec![unit_at_path(leak_path.clone())],
            errors: vec![],
        };
        let output_dir = dir.path().join("out");

        let outcome = result.write_to_disk(&output_dir, &RecordingFakeExtractor::default());

        // Clean up before asserting so a Red run never leaves a stray file.
        let leaked = leak_path.exists();
        if leaked {
            let _ = std::fs::remove_file(&leak_path);
        }

        let err = match outcome {
            Ok(paths) => panic!("absolute unit.path must be rejected, wrote: {paths:?}"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains(&leak_path.display().to_string()),
            "error must name the offending path, got: {err}"
        );
        assert!(
            !leaked,
            "an absolute unit.path must not write outside output_dir"
        );
        assert!(
            !output_dir.exists(),
            "the containment check must run before any IO"
        );
    }

    #[test]
    fn write_to_disk_rejects_parent_escaping_unit_path() {
        // UTA-T4 item 2 (write containment): a unit.path reaching above
        // output_dir through `..` must be rejected too, naming the offender.
        let dir = tempfile::tempdir().unwrap();
        let output_dir = dir.path().join("repo").join("out");
        let escape_target = dir.path().join("uta_t4_escaped.h");
        let escaping_path = std::path::PathBuf::from("..")
            .join("..")
            .join("uta_t4_escaped.h");

        let result = ExtractionResult {
            plan_id: "PLAN-ESC".to_string(),
            headers: vec![unit_at_path(escaping_path)],
            errors: vec![],
        };

        let outcome = result.write_to_disk(&output_dir, &RecordingFakeExtractor::default());

        let escaped = escape_target.exists();
        if escaped {
            let _ = std::fs::remove_file(&escape_target);
        }

        let err = match outcome {
            Ok(paths) => panic!("parent-escaping unit.path must be rejected, wrote: {paths:?}"),
            Err(err) => err,
        };
        assert!(
            err.to_string().contains("uta_t4_escaped.h"),
            "error must name the offending path, got: {err}"
        );
        assert!(
            !escaped,
            "a parent-escaping unit.path must not land outside output_dir"
        );
        assert!(
            !output_dir.exists(),
            "the containment check must run before any IO"
        );
    }

    #[test]
    fn write_to_disk_midbatch_io_failure_keeps_earlier_writes() {
        // UTA-T4 item 7 (characterization pin): write_to_disk owns no journal
        // or rollback. A mid-batch IO failure surfaces as `Err` and the files
        // already written stay on disk untouched - the caller
        // (`TransactionApplier::apply_with`) owns cleanup, not this function.
        let dir = tempfile::tempdir().unwrap();
        let output_dir = dir.path().join("out");
        std::fs::create_dir_all(&output_dir).unwrap();
        // A plain file sitting where the second unit needs its parent dir:
        // create_dir_all(parent) fails deterministically for that unit.
        std::fs::write(output_dir.join("blocker.h"), "existing file").unwrap();

        let result = ExtractionResult {
            plan_id: "PLAN-MID".to_string(),
            headers: vec![
                unit_at_path(std::path::PathBuf::from("ok.h")),
                unit_at_path(std::path::PathBuf::from("blocker.h").join("deep.h")),
            ],
            errors: vec![],
        };

        let outcome = result.write_to_disk(&output_dir, &RecordingFakeExtractor::default());

        assert!(
            outcome.is_err(),
            "a unit blocked by a same-named file must fail the batch, got: {outcome:?}"
        );
        assert!(
            output_dir.join("ok.h").exists(),
            "earlier writes must survive the mid-batch error: no hidden rollback here"
        );
        assert!(
            !output_dir.join("blocker.h").join("deep.h").exists(),
            "the failing unit itself must not have written anything"
        );
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
