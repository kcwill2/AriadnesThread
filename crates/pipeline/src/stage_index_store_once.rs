//! Store-once rebuild of the index stage.
//!
//! Every header a unit reaches used to be analyzed again, and a full copy of
//! its records stored, for every unit that reached it. On a large MSVC repo
//! (thousands of units, hundreds of headers each) that made the index
//! hundreds of times larger than the code it describes. Here each
//! `(file, context)` pair is analyzed once (cached per file on disk) and
//! stored once: in the FIRST unit, in unit order, whose closure reaches it.
//! Later units name the file in `TuAnalysis::shared_files` instead.
//!
//! Three steps, the first and last in parallel:
//! 1. walk every unit's closure (memoized, so each file is scanned once);
//! 2. assign each file to the first unit that reaches it (sequential, so the
//!    result is deterministic whatever the thread count);
//! 3. analyze each unit's owned files and merge them in closure order.

use std::collections::HashSet;
use std::path::Path;

use anyhow::{anyhow, Result};
use rayon::prelude::*;

use ariadne_build::TranslationUnit;
use ariadne_index::{IncrementalStats, IndexResult, TuAnalysis, CURRENT_ANALYSIS_VERSION};
use ariadne_lang::{LanguagePlugin, LanguageRegistry};
use ariadne_repo::FileId;
use ariadne_scheduler::closure_memo::{walk_tu_closure, ClosureFile, ClosureMemo, WalkInputs};
use ariadne_scheduler::file_store::{analyze_file_cached, append_file_analysis, FileAnalysisKey};
use ariadne_scheduler::TuStoreContext;

use crate::ProgressEvent;

/// Units analyzed per progress step. Progress events stay in unit order (the
/// `TuStarted` contract), so they are emitted per chunk, before it runs.
const UNITS_PER_PROGRESS_CHUNK: usize = 64;

/// One unit with its plugin and its closure.
struct UnitPlan<'a> {
    tu: &'a TranslationUnit,
    plugin: &'a dyn LanguagePlugin,
    files: Vec<ClosureFile>,
}

/// Which of a unit's closure files it stores, and which it only names.
struct UnitAssignment {
    owned: Vec<bool>,
    shared_files: Vec<FileId>,
}

/// Shared, read-only inputs of a store-once rebuild.
pub(crate) struct StoreOnceInputs<'a> {
    pub repo_root: &'a Path,
    pub store_ctx: &'a TuStoreContext,
    pub registry: &'a LanguageRegistry,
    pub pool: Option<&'a rayon::ThreadPool>,
}

impl StoreOnceInputs<'_> {
    fn run<T: Send>(&self, work: impl FnOnce() -> T + Send) -> T {
        match self.pool {
            Some(pool) => pool.install(work),
            None => work(),
        }
    }
}

/// Rebuild the index for `units`, storing each file's analysis once.
pub(crate) fn rebuild_index_store_once(
    units: &[TranslationUnit],
    inputs: &StoreOnceInputs<'_>,
    notify: &mut dyn FnMut(&ProgressEvent),
) -> Result<(IndexResult, IncrementalStats)> {
    let memo = ClosureMemo::new();
    let plans = inputs.run(|| plan_units(units, inputs, &memo))?;
    let assignments = assign_files(&plans);

    let total = plans.len();
    let mut analyses = Vec::with_capacity(total);
    let mut reused = 0usize;
    let mut chunk_start = 0usize;
    for chunk in plans.chunks(UNITS_PER_PROGRESS_CHUNK) {
        for (offset, plan) in chunk.iter().enumerate() {
            notify(&started_event(plan.tu, chunk_start + offset, total, inputs.repo_root));
        }
        let chunk_assignments = &assignments[chunk_start..chunk_start + chunk.len()];
        let results: Vec<Result<(TuAnalysis, bool)>> = inputs.run(|| {
            chunk
                .par_iter()
                .zip(chunk_assignments.par_iter())
                .map(|(plan, assignment)| analyze_unit(plan, assignment, inputs.repo_root))
                .collect()
        });
        for result in results {
            let (analysis, unit_reused) = result?;
            reused += usize::from(unit_reused);
            analyses.push(analysis);
        }
        chunk_start += chunk.len();
    }
    notify(&ProgressEvent::TuIndex { done: total, total });

    let index_result = IndexResult::new(inputs.repo_root.to_path_buf(), analyses);
    Ok((index_result, IncrementalStats::calculate(reused, total - reused)))
}

fn started_event(tu: &TranslationUnit, done: usize, total: usize, repo_root: &Path) -> ProgressEvent {
    ProgressEvent::TuStarted {
        done,
        total,
        source: tu
            .source_path
            .strip_prefix(repo_root)
            .unwrap_or(&tu.source_path)
            .to_path_buf(),
    }
}

fn plan_units<'a>(
    units: &'a [TranslationUnit],
    inputs: &StoreOnceInputs<'a>,
    memo: &ClosureMemo,
) -> Result<Vec<UnitPlan<'a>>> {
    units
        .par_iter()
        .map(|tu| {
            let plugin = plugin_for_path(inputs.registry, &tu.source_path)?;
            let walk = WalkInputs {
                repo_root: inputs.repo_root,
                store_ctx: inputs.store_ctx,
                resolver: plugin.resolver(),
            };
            Ok(UnitPlan {
                tu,
                plugin,
                files: walk_tu_closure(tu, &walk, memo),
            })
        })
        .collect()
}

/// Give each `(file, context)` pair to the first unit, in unit order, whose
/// closure reaches it; every later unit lists it as shared.
fn assign_files(plans: &[UnitPlan<'_>]) -> Vec<UnitAssignment> {
    let mut claimed: HashSet<(FileId, &str)> = HashSet::new();
    plans
        .iter()
        .map(|plan| {
            let context = plan.tu.context_name.as_str();
            let mut assignment = UnitAssignment {
                owned: Vec::with_capacity(plan.files.len()),
                shared_files: Vec::new(),
            };
            for file in &plan.files {
                let first_claim = claimed.insert((file.file_id, context));
                assignment.owned.push(first_claim);
                if !first_claim {
                    assignment.shared_files.push(file.file_id);
                }
            }
            assignment
        })
        .collect()
}

/// The unit's stored analysis (its owned files merged in closure order) and
/// whether every owned file came from the per-file cache.
fn analyze_unit(
    plan: &UnitPlan<'_>,
    assignment: &UnitAssignment,
    repo_root: &Path,
) -> Result<(TuAnalysis, bool)> {
    let language = plan.plugin.id();
    let plugin_fingerprint = plan.plugin.analysis_fingerprint();
    let key = FileAnalysisKey {
        analysis_version: CURRENT_ANALYSIS_VERSION,
        language: &language,
        plugin_fingerprint: &plugin_fingerprint,
        context_name: &plan.tu.context_name,
    };
    let mut merged = TuAnalysis {
        tu_file_id: plan.tu.file_id,
        context_name: plan.tu.context_name.clone(),
        language: language.clone(),
        shared_files: assignment.shared_files.clone(),
        ..TuAnalysis::default()
    };
    let mut all_reused = true;
    for (index, file) in plan.files.iter().enumerate() {
        if !assignment.owned[index] {
            continue;
        }
        let outcome = analyze_file_cached(file, &key, plan.plugin.analyzer(), repo_root)?;
        all_reused &= outcome.reused;
        if index == 0 {
            merged.language = outcome.analysis.language.clone();
        }
        append_file_analysis(&mut merged, outcome.analysis);
    }
    Ok((merged, all_reused))
}

/// Plugin that owns `path`'s language; an unclaimed path is an error rather
/// than being parsed by some other plugin.
pub(crate) fn plugin_for_path<'a>(
    registry: &'a LanguageRegistry,
    path: &Path,
) -> Result<&'a dyn LanguagePlugin> {
    let language = registry.detect_language(path).ok_or_else(|| {
        anyhow!(
            "unhandled language for {} (no registered plugin claims it)",
            path.display()
        )
    })?;
    registry
        .plugin_for_id(language.clone())
        .ok_or_else(|| anyhow!("language plugin not registered: {language:?}"))
}
