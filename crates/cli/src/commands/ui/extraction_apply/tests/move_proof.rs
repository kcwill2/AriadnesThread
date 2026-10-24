//! C5 move-proof oracle (extraction-symbol-move SM-4): the milestone's
//! end-to-end machine witness for SM-I2 (the grown faithfulness equality)
//! and SM-I3 (dry-run/apply move-claim parity), plus three non-vacuity
//! guards proving the marker-proof helpers themselves reject broken
//! specimens.
//!
//! Independence (shape R4: the grown oracle, never an amended rider, is
//! the machine witness): markers are parsed per the PUBLISHED grammar
//! (shape C2) with this module's own constants, never through `lang-cpp`
//! internals. The candidate set is the reader's own
//! `move_removal_candidates` mirror, so the oracle reads the same
//! derivation validate runs inside apply (SM-I4 byte truth) without
//! calling validate. The offer order mirrors `strip_moved_symbols`:
//! BTreeSet order with a per-path running-text fold (adjacent candidates
//! for one path see the earlier candidate's marked text - the idempotence
//! refusals that make the fixture merge to ONE pair).
//!
//! M3 terminator deviation (SM-2 verdict finding M3, `move_retention`
//! accepted deviation 4, binding on C5.4): a marked region ending on a
//! file's FINAL UNTERMINATED line gains the dominant terminator, so the
//! uncomment reconstruction of such a file equals the original PLUS that
//! added terminator. The basic fixture's files are terminated, so
//! reconstruct-equality holds as written there; the synthetic specimens in
//! this module are authored terminated.

use super::canonicalized_lossy;
use ariadne_cluster::plan::OptimizationPlan;
use ariadne_lang::DependencyRewriter;
use ariadne_transform::{ExtractionResult, IsolationEngine, MigrationPreview};
use ariadne_ui::AnalysisGeneration;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// The published marker grammar (shape C2) re-declared on purpose: the
/// oracle recognizes markers by the grammar, not by the producing crate.
const BEGIN_HEAD: &str = "// ARIADNE-EXTRACT-OUT BEGIN plan=";
const SYMBOLS_SEPARATOR: &str = " symbols=";
const END_HEAD: &str = "// ARIADNE-EXTRACT-OUT END plan=";

/// The pre-apply move derivation of record: the reader's own candidate
/// mirror plus the strip-order fold's successful hook offers.
pub(crate) struct MoveDerivation {
    /// Every candidate in BTreeSet order: (source path, unit names).
    pub(crate) candidates: Vec<(PathBuf, Vec<String>)>,
    /// The fold's hook answers `Some` per source path: each entry is one
    /// offer's name list, i.e. one marker pair the apply will bind.
    pub(crate) succeeded_by_path: BTreeMap<PathBuf, Vec<Vec<String>>>,
}

/// Re-run validate's strip-offer sequence PRE-APPLY over the reader's own
/// derivation: same candidates, same BTreeSet order, same per-path running
/// text, same hook, same plan-id marker binding (`plan.id.0` renders the
/// fixture's `PLAN-001`, m20's byte truth). Read-only: the fold lives in
/// memory, nothing is written.
pub(crate) fn derive_move_fold(
    repo_root: &Path,
    plans: &[OptimizationPlan],
    plan_id: &str,
    generation: &AnalysisGeneration,
) -> MoveDerivation {
    let plan = find_plan(plans, plan_id);
    let migration = migration_preview_via_cli(repo_root, plan_id);
    let extraction = engine_extraction(plan, generation);
    let candidates: Vec<(PathBuf, Vec<String>)> =
        crate::commands::ui::extraction_reader::move_removal_candidates(
            repo_root, &migration, &extraction,
        )
        .into_iter()
        .collect();
    let registry = crate::registry::default_registry();
    let rewriter = crate::registry::plan_backend(
        &registry,
        &plan.target_file,
        |caps| caps.rewriting,
        "rewriting",
    )
    .expect("the plan's language registers a rewriting backend")
    .rewriter();
    let mut folded_text: HashMap<PathBuf, String> = HashMap::new();
    let mut succeeded_by_path: BTreeMap<PathBuf, Vec<Vec<String>>> = BTreeMap::new();
    for (source_path, moved_names) in &candidates {
        let current_text = match folded_text.get(source_path) {
            Some(text) => text.clone(),
            None => {
                let Ok(fresh) = std::fs::read_to_string(repo_root.join(source_path)) else {
                    continue;
                };
                fresh
            }
        };
        if let Some(post_move) =
            rewriter.move_out_top_level_symbols(&current_text, moved_names, &plan.id.0)
        {
            folded_text.insert(source_path.clone(), post_move);
            succeeded_by_path
                .entry(source_path.clone())
                .or_default()
                .push(moved_names.clone());
        }
    }
    MoveDerivation { candidates, succeeded_by_path }
}

/// The whole PRE-apply expected oracle of C5.1: the strip-order fold, the
/// surviving consumers-only sub-claim list (from the tests module's own
/// helper), and their union - all read before the apply, computed once and
/// reused by the marker proof's hook-success count (C5.4).
pub(crate) struct PreApplyExpectation {
    pub(crate) fold: MoveDerivation,
    pub(crate) consumers: Vec<String>,
    pub(crate) union: Vec<String>,
}

pub(crate) fn expectation_pre_apply(
    repo_root: &Path,
    plans: &[OptimizationPlan],
    plan_id: &str,
    rewrites: &[ariadne_ui::ExtractionRewriteDto],
    generation: &AnalysisGeneration,
) -> PreApplyExpectation {
    let fold = derive_move_fold(repo_root, plans, plan_id, generation);
    let consumers = super::expected_modified_consumers(rewrites, plans, plan_id);
    let union = expected_modified_with_move_sources(&fold, &consumers, repo_root);
    PreApplyExpectation { fold, consumers, union }
}

/// C5.1: the grown expected-modified set - the consumers-with-hits list
/// (the surviving sub-claim, computed by the caller) UNION every source
/// path the fold retired, host-canonical via the walk's own rendering,
/// sorted. All reads are PRE-apply (shape C4/C5).
pub(crate) fn expected_modified_with_move_sources(
    fold: &MoveDerivation,
    expected_consumers: &[String],
    repo_root: &Path,
) -> Vec<String> {
    let mut expected: Vec<String> = expected_consumers.to_vec();
    for source_path in fold.succeeded_by_path.keys() {
        let canonical = canonicalized_lossy(repo_root, source_path);
        if !expected.contains(&canonical) {
            expected.push(canonical);
        }
    }
    expected.sort();
    expected
}

/// C5.2's fixture equality: the modified set IS the approved consumers
/// UNION the canonicalized `include/thing.h` move source - the original
/// file became a modified file (SM-I2's fixture half).
pub(crate) fn assert_fixture_union(
    modified: &[String],
    approved_consumers: &[String],
    repo_root: &Path,
) {
    let mut expected_union = approved_consumers.to_vec();
    expected_union.push(canonicalized_lossy(repo_root, Path::new("include/thing.h")));
    expected_union.sort();
    assert_eq!(
        modified, expected_union,
        "fixture equality (C5.2): modified IS the approved consumers UNION \
         the canonicalized include/thing.h move source - the original file \
         became a modified file (extraction-symbol-move)"
    );
}

/// C5.4 + C5.5: the marker proof per marked original file, from the
/// pre/post walk pair. Claims: balanced BEGIN/END pairs; the pair count
/// equals the fold's hook-success count for this file; every marker line
/// carries the approved plan id; each symbols field equals the succeeded
/// offer's names (fixture truth: `Thing`); prefix-stripping the regions
/// reconstructs the pre-apply bytes byte-exactly (M3: fixture files are
/// terminated); every marker byte ASCII (A9); re-offering the hook over
/// the post-apply text with the same names answers `None` (the region is
/// inert, SM-I1); and the dry run's move claim equals the total symbol
/// names bound into the markers (SM-I3 parity, fixture: 1 == 1).
pub(crate) fn assert_marked_regions(
    plans: &[OptimizationPlan],
    plan_id: &str,
    relative: &Path,
    before: &BTreeMap<PathBuf, Option<Vec<u8>>>,
    after: &BTreeMap<PathBuf, Option<Vec<u8>>>,
    approved_move_claim: usize,
    fold: &MoveDerivation,
) {
    let before_text = walked_text(before, relative);
    let after_text = walked_text(after, relative);
    let successes = fold.succeeded_by_path.get(relative).cloned().unwrap_or_default();
    assert_balanced_pairs(&after_text);
    let (begins, ends) = marker_lines(&after_text);
    assert_eq!(
        begins.len(),
        successes.len(),
        "C5.4: the marked pair count must equal this file's hook-success \
         calls (fixture: ONE merged pair)"
    );
    assert_marker_plan_ids(&begins, &ends, plan_id);
    assert_symbols_fields(&begins, &successes);
    assert_prefix_strip_reconstructs(&after_text, &before_text);
    assert_markers_ascii(&begins, &ends);
    assert_hook_inert_after_apply(plans, plan_id, relative, &after_text, fold);
    assert_bound_names_total(&begins, approved_move_claim);
}

/// The walk's readable text for one relative path, panicking honestly
/// when the walk lacks it (a missing file is a rejection, not a pass).
fn walked_text(walk: &BTreeMap<PathBuf, Option<Vec<u8>>>, relative: &Path) -> String {
    let bytes = walk
        .get(relative)
        .unwrap_or_else(|| panic!("the walk must contain the file {relative:?}"))
        .as_ref()
        .cloned()
        .unwrap_or_else(|| panic!("the file {relative:?} must be readable"));
    String::from_utf8(bytes).expect("the fixture file text is UTF-8")
}

/// Lines recognized as markers by the published grammar (terminator-free).
fn marker_lines(text: &str) -> (Vec<&str>, Vec<&str>) {
    let begins: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with(BEGIN_HEAD))
        .collect();
    let ends: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with(END_HEAD))
        .collect();
    (begins, ends)
}

/// The BEGIN marker's plan and symbols fields per the grammar.
fn begin_fields(line: &str) -> (&str, &str) {
    let rest = line
        .strip_prefix(BEGIN_HEAD)
        .expect("a BEGIN marker line carries the BEGIN head");
    rest.split_once(SYMBOLS_SEPARATOR)
        .expect("a BEGIN marker carries a symbols field")
}

/// The END marker's plan field per the grammar.
fn end_fields(line: &str) -> &str {
    let plan = line
        .strip_prefix(END_HEAD)
        .expect("an END marker line carries the END head");
    assert!(!plan.contains(SYMBOLS_SEPARATOR), "an END marker binds only the plan id");
    plan
}

fn assert_balanced_pairs(text: &str) {
    let (begins, ends) = marker_lines(text);
    assert_eq!(
        begins.len(),
        ends.len(),
        "the marked regions are balanced BEGIN/END pairs: {} BEGIN vs {} END",
        begins.len(),
        ends.len()
    );
}

fn assert_marker_plan_ids(begins: &[&str], ends: &[&str], plan_id: &str) {
    for line in begins {
        let (plan, _symbols) = begin_fields(line);
        assert_eq!(plan, plan_id, "every BEGIN marker binds the approved plan id");
    }
    for line in ends {
        assert_eq!(end_fields(line), plan_id, "every END marker binds the approved plan id");
    }
}

fn assert_symbols_fields(begins: &[&str], successes: &[Vec<String>]) {
    for (line, names) in begins.iter().zip(successes) {
        let (_plan, symbols) = begin_fields(line);
        assert_eq!(
            symbols,
            names.join(";"),
            "the symbols field carries the succeeded offer's names \
             (fixture truth: symbols=Thing)"
        );
    }
}

fn assert_prefix_strip_reconstructs(after: &str, before: &str) {
    assert_eq!(
        uncomment_regions(after).as_bytes(),
        before.as_bytes(),
        "C5.4: prefix-stripping every marked region reconstructs the \
         pre-apply bytes byte-exactly (M3: the SM-2 deviation-4 terminator \
         rule - a terminator-less final region line gains the dominant \
         terminator; these specimens are terminated, so equality holds as \
         written)"
    );
}

fn assert_markers_ascii(begins: &[&str], ends: &[&str]) {
    for line in begins.iter().chain(ends) {
        assert!(line.is_ascii(), "A9: every marker byte must be ASCII, got {line:?}");
    }
}

fn assert_hook_inert_after_apply(
    plans: &[OptimizationPlan],
    plan_id: &str,
    relative: &Path,
    after_text: &str,
    fold: &MoveDerivation,
) {
    let plan = find_plan(plans, plan_id);
    let registry = crate::registry::default_registry();
    let rewriter = crate::registry::plan_backend(
        &registry,
        &plan.target_file,
        |caps| caps.rewriting,
        "rewriting",
    )
    .expect("the plan's language registers a rewriting backend")
    .rewriter();
    let mut reoffers = 0usize;
    for (source_path, moved_names) in &fold.candidates {
        if source_path != relative {
            continue;
        }
        reoffers += 1;
        assert!(
            rewriter
                .move_out_top_level_symbols(after_text, moved_names, plan_id)
                .is_none(),
            "C5.4/SM-I1: re-offering the hook over the post-apply text with \
             the same names must answer None (the marked region is inert): \
             {moved_names:?}"
        );
    }
    assert!(reoffers > 0, "the fold's candidates must cover the marked file {relative:?}");
}

fn assert_bound_names_total(begins: &[&str], approved_move_claim: usize) {
    let bound: usize = begins
        .iter()
        .map(|line| begin_fields(line).1.split(';').count())
        .sum();
    assert_eq!(
        bound, approved_move_claim,
        "SM-I3 parity (C5.5): the dry-run move claim must equal the total \
         symbol names bound into the written markers (fixture: 1 == 1)"
    );
}

/// The line-prefix undo of the retention form: drop the marker lines and
/// strip exactly one leading `//` from every region line (the m20 rider's
/// \r-tolerant idiom; an empty original line is the bare `//`).
fn uncomment_regions(text: &str) -> String {
    let mut restored = String::new();
    let mut inside_region = false;
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\r', '\n']);
        if body.starts_with(BEGIN_HEAD) {
            inside_region = true;
            continue;
        }
        if body.starts_with(END_HEAD) {
            inside_region = false;
            continue;
        }
        restored.push_str(if inside_region {
            line.strip_prefix("//").unwrap_or(line)
        } else {
            line
        });
    }
    restored
}

fn find_plan<'a>(plans: &'a [OptimizationPlan], plan_id: &str) -> &'a OptimizationPlan {
    plans
        .iter()
        .find(|candidate| candidate.id.0 == plan_id)
        .expect("the approved plan resolves in the generation's plan set")
}

/// The real CLI preview path (frozen `transform.rs`, call-only) serialized
/// to `artifacts/migration/preview.json` - the same bytes the approved
/// dry-run rows carry (the reader's C4 parity test locks that equality),
/// without fabricating a MigrationPreview in test code.
fn migration_preview_via_cli(repo_root: &Path, plan_id: &str) -> MigrationPreview {
    crate::commands::transform::build_migration_preview(&repo_root.to_path_buf(), plan_id, false)
        .expect("the fixture plan previews through the frozen CLI path");
    let preview_bytes = std::fs::read(
        repo_root
            .join("artifacts")
            .join("migration")
            .join("preview.json"),
    )
    .expect("the CLI preview artifact");
    serde_json::from_slice(&preview_bytes).expect("preview.json decodes")
}

/// The reader's own engine pass (run_engine idiom): the generation's
/// reduced model through the isolation engine with the placeholder-body
/// source reader (invariant I4), so the candidate units keep the engine's
/// path spellings - the same inputs validate's strip consumes.
fn engine_extraction(plan: &OptimizationPlan, generation: &AnalysisGeneration) -> ExtractionResult {
    let registry = crate::registry::default_registry();
    let backend = crate::registry::plan_backend(
        &registry,
        &plan.target_file,
        |caps| caps.extraction,
        "extraction",
    )
    .expect("the plan's language registers an extraction backend");
    let extractor = backend.extractor();
    IsolationEngine::new(&generation.reduced_model).extract(plan, &|_source_file| None, extractor)
}

// ---------------------------------------------------------------------------
// Non-vacuity trio (declared Green-at-birth, SM-1 justified-Red-skip
// precedent): pure synthetic-byte self-checks of the proof helpers - they
// cannot exist before their helpers and guard the markers proof against
// proving nothing. Not behavior claims on the production seams.
// ---------------------------------------------------------------------------

/// PRE-apply witness: the union helper equals the approved consumers UNION
/// the canonicalized `include/thing.h`, derived from the reader's own
/// candidate mirror - nothing applied, the original file still unmarked.
#[test]
fn move_removal_union_helper_matches_the_fixture_derivation() {
    let (_temp_dir, state, proposals, plans) = super::boot_swapped_fixture("basic");
    let plan_id = super::first_proposal_id(&proposals);
    let reader = super::installed_reader(&state);
    let dry_run = reader
        .dry_run(&plan_id)
        .expect("the boot proposal id must resolve within its generation");
    let fold = derive_move_fold(
        &state.repo_root,
        &plans,
        &plan_id,
        &state.analysis().expect("boot installed the generation"),
    );
    let union = expected_modified_with_move_sources(
        &fold,
        &super::expected_modified_consumers(&dry_run.rewrites, &plans, &plan_id),
        &state.repo_root,
    );
    let mut expected: Vec<String> = dry_run
        .rewrites
        .iter()
        .map(|row| row.consumer_file.clone())
        .collect();
    expected.push(canonicalized_lossy(&state.repo_root, Path::new("include/thing.h")));
    expected.sort();
    assert_eq!(
        union, expected,
        "the derived union must equal the approved consumers UNION the \
         canonicalized include/thing.h move source (PRE-apply, nothing \
         applied)"
    );
    let thing_text =
        std::fs::read_to_string(state.repo_root.join("include").join("thing.h"))
            .expect("the fixture header is readable");
    assert!(
        !thing_text.contains(BEGIN_HEAD),
        "nothing was applied: the original file is still unmarked"
    );
}

/// Non-vacuity guard: the balance claim must reject an unbalanced
/// specimen - a proof helper that never rejects proves nothing.
#[test]
fn assert_marked_regions_rejects_an_unbalanced_pair() {
    let unbalanced = concat!(
        "// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-001 symbols=Thing\n",
        "//class Thing {};\n",
    );
    let rejected = std::panic::catch_unwind(|| assert_balanced_pairs(unbalanced));
    assert!(
        rejected.is_err(),
        "the balance claim must reject a BEGIN marker without its END marker"
    );
}

/// Non-vacuity guard: the prefix-strip reconstruct must fail the byte
/// equality on tampered region bytes (the specimen is terminated per the
/// M3 carrier - author terminated, never rely on the added terminator).
#[test]
fn assert_marked_regions_rejects_tampered_region_bytes() {
    let before = concat!("int kept() {}\n", "class Thing {};\n");
    let after = concat!(
        "int kept() {}\n",
        "// ARIADNE-EXTRACT-OUT BEGIN plan=PLAN-001 symbols=Thing\n",
        "//class Thing { };\n",
        "// ARIADNE-EXTRACT-OUT END plan=PLAN-001\n",
    );
    let rejected =
        std::panic::catch_unwind(|| assert_prefix_strip_reconstructs(after, before));
    assert!(
        rejected.is_err(),
        "prefix-stripping tampered region bytes must fail the byte equality"
    );
}
