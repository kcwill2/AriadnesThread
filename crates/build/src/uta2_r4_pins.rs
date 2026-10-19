//! UTA2-R4 characterization pins for build discovery/measurement residuals
//! (docs/specs/unit-test-audit.md Round 2, handoff item 3). The plain
//! candidate deletion ladder (root > build/ > out/) is already locked by the
//! round-1 `uta_t6_pins` suite; this file adds the CONFLICT dimension the
//! audit row names: precedence short-circuits on EXISTENCE, so the losers
//! are never parsed even when unparseable - and an existing-but-malformed
//! winner fails the whole discovery instead of falling through to a valid
//! lower rung. Plus the measure fallback attribution. Crate-level child
//! module (the `uta_t6_pins.rs` model); all seams exercised are public.

use crate::BuildDiscoverer;
use crate::BuildMeasurer;
use ariadne_repo::FileId;
use std::fs;
use std::path::{Path, PathBuf};

/// One compilation database whose single entry names its slot via a
/// distinct file, so the winner is recognizable from the TU's file id.
fn marker_database(marker: &str, directory: &Path) -> String {
    serde_json::json!([
        {
            "directory": directory.to_string_lossy(),
            "file": format!("src/{marker}"),
            "arguments": ["cl.exe", "/c", format!("src/{marker}")],
        }
    ])
    .to_string()
}

#[test]
fn pin_conflicting_databases_losers_never_parsed_and_malformed_winner_never_falls_through()
{
    let repo = tempfile::tempdir().unwrap();
    let root = repo.path();
    fs::create_dir_all(root.join("build")).unwrap();
    fs::create_dir_all(root.join("out")).unwrap();

    fs::write(
        root.join("compile_commands.json"),
        marker_database("from-root.cpp", root),
    )
    .unwrap();
    // Conflicting losers that CANNOT parse: were the ladder ever to parse or
    // probe content, discovery would fail - so a clean root parse proves the
    // lower rungs are never touched while a higher rung exists.
    fs::write(
        root.join("build").join("compile_commands.json"),
        "build rung: not JSON at all",
    )
    .unwrap();
    fs::write(
        root.join("out").join("compile_commands.json"),
        "out rung: also not JSON",
    )
    .unwrap();

    let result = BuildDiscoverer::new(root)
        .discover()
        .expect("pin: unparseable LOSERS must not affect the winning root db");
    assert_eq!(
        result.translation_units[0].file_id,
        FileId::from_relative_path("src/from-root.cpp"),
        "pin: root still wins the conflict and parses alone"
    );

    // The flip side: the first EXISTING candidate is parsed even when it is
    // malformed - a broken higher rung never falls through to a valid lower
    // rung. (build/ malformed, out/ perfectly parseable.)
    fs::remove_file(root.join("compile_commands.json")).unwrap();
    fs::write(
        root.join("out").join("compile_commands.json"),
        marker_database("from-out.cpp", &root.join("out")),
    )
    .unwrap();
    let outcome = BuildDiscoverer::new(root).discover();
    match outcome {
        Ok(result) => panic!(
            "pin: discovery must NOT fall through to the valid out/ db; \
             it returned {} TU(s) instead",
            result.translation_units.len()
        ),
        Err(err) => {
            let chain = format!("{err:#}");
            assert!(
                chain.contains("Failed to parse compile_commands.json at")
                    && chain.contains("build"),
                "the Err must name the malformed build/ rung: {chain}"
            );
        }
    }
}

#[test]
fn pin_malformed_compile_db_err_names_the_offending_file() {
    let repo = tempfile::tempdir().unwrap();
    let root = repo.path();
    let db_path = root.join("compile_commands.json");
    fs::write(&db_path, "this is not a JSON array").unwrap();

    let outcome = BuildDiscoverer::new(root).discover();
    let err = match outcome {
        Ok(_) => panic!("malformed JSON must never parse into an empty success"),
        Err(err) => err,
    };
    let chain = format!("{err:#}");
    assert!(
        chain.contains("Failed to parse compile_commands.json at"),
        "the top context must name the parse gate: {chain}"
    );
    // The offending path is named in its Debug rendering (escaped
    // backslashes on Windows) - the full path, not merely the basename.
    let escaped_path = db_path.to_string_lossy().replace('\\', "\\\\");
    assert!(
        chain.contains(&escaped_path),
        "the Err must carry the offending file's full path: {chain}"
    );
}

#[test]
fn pin_measure_tu_simulation_attributes_wall_clock_to_frontend_cpu() {
    // Honest attribution: with no compile command to run (empty `arguments`
    // skips the spawn entirely) the measurer SIMULATES, and the simulation
    // ATTRIBUTES the elapsed wall clock to `frontend_cpu_time`
    // (`measure.rs:186-188`). The reported "front-end CPU" is therefore NOT
    // measured compiler time in this mode, and a TU that never ran carries
    // exit code 0 (success).
    let repo = tempfile::tempdir().unwrap();
    let measurer = BuildMeasurer::new(repo.path().to_path_buf());
    let rel = PathBuf::from("simulated.cpp");
    let tu = crate::TranslationUnit {
        file_id: FileId::from_relative_path("simulated.cpp"),
        source_path: rel.clone(),
        context_name: "sim-context".into(),
        compile_command: crate::CompileCommand {
            directory: repo.path().to_path_buf(),
            file: rel,
            command: None,
            arguments: vec![],
            context_name: "sim-context".into(),
            output: None,
        },
        language: Default::default(),
    };

    let measurement = measurer
        .measure_tu(&tu)
        .expect("the simulation path never fails");
    assert_eq!(measurement.tu_file_id, FileId::from_relative_path("simulated.cpp"));
    assert_eq!(
        measurement.frontend_cpu_time, measurement.wall_clock_time,
        "pin: the fallback copies wall_clock into frontend_cpu verbatim"
    );
    assert!(
        measurement.wall_clock_time.as_millis() >= 1,
        "wall clock carries the 1ms floor clamp"
    );
    assert_eq!(measurement.preprocessing_time.as_millis(), 0);
    assert_eq!(measurement.memory_peak_bytes, 1024 * 1024);
    assert_eq!(
        measurement.exit_code, 0,
        "pin: a TU that never ran is reported with exit code 0 (success)"
    );
}
