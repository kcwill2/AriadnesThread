//! Section 4 — reduced-model provenance fixture lock (plan §5 M18
//! acceptance, fourth clause / §3.6.1).
//!
//! On both fixtures through the public registry pipeline
//! (`handle_index_with_registry` + `ReducedSemanticModel::reduce`), every
//! symbol's `defining_file` must equal projections recomputed INDEPENDENTLY
//! in this test — never by calling the rule under lock:
//!
//! * [`ceac55f_rule_projection`] re-mirrors the ceac55f provenance rule
//!   (per-TU `DeclId → FileId` registry from `tu.declarations`; defining
//!   file = the definition declaration's single resolved file, fallback
//!   `owning_file`; first accepted definition wins with Class/Struct
//!   upgrading a weaker definition), and
//! * [`legacy_owning_file_projection`] re-mirrors the OLDER M5 rule
//!   (`defining_file` = `owning_file` of the first accepted definition,
//!   same first-wins/Class-upgrade order) — the acceptance requirement
//!   that `defining_file` values are unchanged across the fix.
//!
//! `declaring_files` must be populated for every symbol that carries
//! declarations, sorted, and deduped; the spot table locks concrete
//! fixture values captured at ceac55f through this same pipeline.

use super::{isolated_repo, normalized_rel};
use anyhow::Result;
use ariadne_cli::handle_index_with_registry;
use ariadne_graph::ReducedSemanticModel;
use ariadne_index::IndexResult;
use ariadne_index::types::{DeclId, SymbolId, SymbolKind, TuAnalysis};
use ariadne_repo::FileId;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Per-fixture provenance spot lock: symbol → (`defining_file` repo path
/// with forward slashes, or `"-"` for none) plus its sorted `declaring_files`
/// — captured at ceac55f through the registry pipeline (same run as the
/// section 3 byte captures).
const BASIC_SPOT_PROVENANCE: &[(&str, &str, &[&str])] = &[
    ("HeavyData", "include/heavy.h", &["include/heavy.h"]),
    ("Thing", "include/thing.h", &["include/thing.h"]),
    (
        "Thing::Thing",
        "src/thing.cpp",
        &["include/thing.h", "src/thing.cpp"],
    ),
    (
        "Thing::get_id",
        "src/thing.cpp",
        &["include/thing.h", "src/thing.cpp"],
    ),
    (
        "ThingConfig",
        "include/thing_types.h",
        &["include/thing_types.h"],
    ),
    (
        "heavy_computation",
        "include/heavy.h",
        &["include/heavy.h"],
    ),
    ("main", "tests/consumer.cpp", &["tests/consumer.cpp"]),
];

const EXPENSIVE_SPOT_PROVENANCE: &[(&str, &str, &[&str])] = &[
    (
        "HeavyTemplate",
        "include/heavy.h",
        &["include/heavy.h"],
    ),
    (
        "HeavyTemplate::do_something",
        "include/heavy.h",
        &["include/heavy.h"],
    ),
    ("Thing", "include/thing.h", &["include/thing.h"]),
    ("Thing::use_heavy", "include/thing.h", &["include/thing.h"]),
    (
        "another_func",
        "src/consumer2.cpp",
        &["src/consumer2.cpp"],
    ),
    ("heavy_func", "include/heavy.h", &["include/heavy.h"]),
    ("main", "src/consumer1.cpp", &["src/consumer1.cpp"]),
];

struct PipelineRun {
    _repo: super::IsolatedRepo,
    index: IndexResult,
    model: ReducedSemanticModel,
    path_by_id: BTreeMap<FileId, String>,
}

/// The public registry pipeline, cold on a throwaway fixture copy.
fn registry_pipeline(fixture: &str, label: &str) -> Result<PipelineRun> {
    let repo = isolated_repo(fixture, label)?;
    let discovery = super::repo_inputs(&repo.root)?.0;
    let index = handle_index_with_registry(
        &repo.root,
        Arc::new(ariadne_cli::registry::default_registry()),
    )?;
    let model = ReducedSemanticModel::reduce(&index);
    let path_by_id = discovery
        .files
        .iter()
        .map(|record| (record.id, normalized_rel(&record.path)))
        .collect();
    Ok(PipelineRun {
        _repo: repo,
        index,
        model,
        path_by_id,
    })
}

#[test]
fn reduced_defining_file_matches_vendored_projections_basic() -> Result<()> {
    provenance_parity("basic", BASIC_SPOT_PROVENANCE)
}

#[test]
fn reduced_defining_file_matches_vendored_projections_expensive() -> Result<()> {
    provenance_parity("expensive", EXPENSIVE_SPOT_PROVENANCE)
}

fn provenance_parity(fixture: &str, spot: &[(&str, &str, &[&str])]) -> Result<()> {
    let run = registry_pipeline(fixture, "provenance")?;
    assert!(
        !run.model.symbols.is_empty(),
        "{fixture}: reduced model must contain symbols"
    );
    let rule = ceac55f_rule_projection(&run.index.tu_analyses);
    let legacy = legacy_owning_file_projection(&run.index.tu_analyses);

    for (id, record) in &run.model.symbols {
        assert_eq!(
            record.defining_file,
            *rule.get(id).unwrap_or(&None),
            "{fixture}: {} defining_file drifted from the independently \
             re-mirrored ceac55f decl-location rule",
            record.name
        );
        if record.definition.is_some() {
            assert_eq!(
                record.defining_file,
                legacy.get(id).copied(),
                "{fixture}: {} defining_file drifted from the legacy \
                 first-wins/Class-upgrade owning-file value (plan §5 \
                 \"defining_file values are unchanged\")",
                record.name
            );
        }
        assert_declaring_files(&record.name, record);
    }
    assert_spot_locks(fixture, &run, spot);
    Ok(())
}

fn assert_declaring_files(fixture: &str, record: &ariadne_graph::reduced::ReducedSymbolRecord) {
    let files = &record.declaring_files;
    assert!(
        files.windows(2).all(|window| window[0] <= window[1]),
        "{fixture}: {} declaring_files not sorted: {files:?}",
        record.name
    );
    let unique: BTreeSet<FileId> = files.iter().copied().collect();
    assert_eq!(
        unique.len(),
        files.len(),
        "{fixture}: {} declaring_files not deduped: {files:?}",
        record.name
    );
    assert_eq!(
        !files.is_empty(),
        !record.declarations.is_empty(),
        "{fixture}: {} declaring_files must be populated exactly when the \
         symbol carries declarations (declarations: {:?}, declaring: {files:?})",
        record.name,
        record.declarations
    );
}

fn assert_spot_locks(
    fixture: &str,
    run: &PipelineRun,
    spot: &[(&str, &str, &[&str])],
) {
    assert!(!spot.is_empty(), "{fixture}: spot lock table is empty");
    for (symbol, expected_defining, expected_declaring) in spot {
        let record = run
            .model
            .find_symbol(symbol)
            .unwrap_or_else(|| panic!("{fixture}: symbol {symbol} missing"));
        let actual_defining = record
            .defining_file
            .and_then(|id| run.path_by_id.get(&id))
            .map(String::as_str)
            .unwrap_or("-");
        assert_eq!(
            actual_defining, *expected_defining,
            "{fixture}: {symbol} defining_file spot lock drifted"
        );
        let actual_declaring: Vec<&str> = record
            .declaring_files
            .iter()
            .map(|id| run.path_by_id.get(id).map(String::as_str).unwrap_or("?"))
            .collect();
        assert_eq!(
            actual_declaring,
            expected_declaring.iter().copied().collect::<Vec<_>>(),
            "{fixture}: {symbol} declaring_files spot lock drifted"
        );
    }
}

/// Independently re-mirrored provenance rule as implemented at ceac55f (plan
/// §3.6.1): per-TU declaration registry (sentinel `FileId(0)` locations
/// skipped, files deduped per `(symbol, decl)` because the parser's
/// `decl_<counter>` ids collide across files); the defining file is the
/// definition decl's single resolved file, else `owning_file`; the first
/// accepted definition wins, upgraded by Class/Struct definitions over
/// weaker kinds. Written from the rule, NOT by calling the graph code.
fn ceac55f_rule_projection(analyses: &[TuAnalysis]) -> BTreeMap<SymbolId, Option<FileId>> {
    struct Entry {
        definition: Option<DeclId>,
        kind: SymbolKind,
        defining_file: Option<FileId>,
    }
    let mut entries: BTreeMap<SymbolId, Entry> = BTreeMap::new();
    for tu in analyses {
        let mut decl_locations: BTreeMap<(SymbolId, DeclId), Vec<FileId>> = BTreeMap::new();
        for decl in &tu.declarations {
            if decl.location.file_id.0 == 0 {
                continue;
            }
            let files = decl_locations
                .entry((decl.symbol_id.clone(), decl.id.clone()))
                .or_default();
            if !files.contains(&decl.location.file_id) {
                files.push(decl.location.file_id);
            }
        }
        for symbol in &tu.symbols {
            let entry = entries
                .entry(symbol.id.clone())
                .or_insert_with(|| Entry {
                    definition: None,
                    kind: symbol.kind,
                    defining_file: None,
                });
            let Some(def) = &symbol.definition else {
                continue;
            };
            let accepts_first = entry.definition.is_none();
            let class_upgrades = entry.kind != SymbolKind::Class
                && entry.kind != SymbolKind::Struct
                && (symbol.kind == SymbolKind::Class || symbol.kind == SymbolKind::Struct);
            if accepts_first || class_upgrades {
                entry.definition = Some(def.clone());
                entry.defining_file = Some(match decl_locations
                    .get(&(symbol.id.clone(), def.clone()))
                    .map(Vec::as_slice)
                {
                    Some([single]) => *single,
                    _ => symbol.owning_file,
                });
                if class_upgrades {
                    entry.kind = symbol.kind;
                }
            }
        }
    }
    entries
        .into_iter()
        .map(|(id, entry)| (id, entry.defining_file))
        .collect()
}

/// The older M5 rule vendored from the ceac55f characterization: the
/// defining file was simply the `owning_file` of the first accepted
/// definition, with Class/Struct upgrading a weaker definition.
fn legacy_owning_file_projection(
    analyses: &[TuAnalysis],
) -> BTreeMap<SymbolId, FileId> {
    let mut result: BTreeMap<SymbolId, FileId> = BTreeMap::new();
    let mut kinds: BTreeMap<SymbolId, SymbolKind> = BTreeMap::new();
    for analysis in analyses {
        for record in &analysis.symbols {
            if record.definition.is_none() {
                continue;
            }
            let entry_kind = kinds.get(&record.id).copied().unwrap_or(record.kind);
            let has_def = result.contains_key(&record.id);
            let upgradeable = !matches!(entry_kind, SymbolKind::Class | SymbolKind::Struct)
                && matches!(record.kind, SymbolKind::Class | SymbolKind::Struct);
            if !has_def || upgradeable {
                result.insert(record.id.clone(), record.owning_file);
                kinds.insert(record.id.clone(), record.kind);
            }
        }
    }
    result
}

#[test]
fn zz_capture_provenance() -> Result<()> {
    for fixture in ["basic", "expensive"] {
        let run = registry_pipeline(fixture, "capture-prov")?;
        for (id, record) in &run.model.symbols {
            let defining = record
                .defining_file
                .and_then(|file_id| run.path_by_id.get(&file_id))
                .cloned()
                .unwrap_or_else(|| "-".to_string());
            let declaring: Vec<String> = record
                .declaring_files
                .iter()
                .map(|file_id| {
                    run.path_by_id
                        .get(file_id)
                        .cloned()
                        .unwrap_or_else(|| format!("?{}", file_id.0))
                })
                .collect();
            println!(
                "@@PROV\t{}\t{}\t{}\t{}\t{}",
                fixture,
                serde_json::to_string(&record.name)?,
                serde_json::to_string(&id.0)?,
                serde_json::to_string(&defining)?,
                serde_json::to_string(&declaring)?
            );
        }
    }
    Ok(())
}


