//! AR-04 (audit T1, HIGH): mid-apply failure must roll the tree back.
//!
//! `apply_with` advertises a transactional guarantee: an `Err` return leaves
//! no partial-repo mutation behind. Before AR-04 the guarantee only held for
//! tier failures handled by callers (the `optimize.rs` / `transform.rs`
//! pattern `if fail { rollback(&snapshot, &new_paths) }`); an IO or hook
//! failure INSIDE the write phases propagated with the earlier writes left
//! on disk. These locks mirror that tier-failure pattern as an invariant of
//! `apply_with` itself: on ANY error during the write/extract/rewriter
//! phases, ALL previously-written targets are restored before the error
//! propagates, and the error text gains a "rolled back N files" line where N
//! counts every path the rollback pass processes (snapshot entries + created
//! paths).

use ariadne_index::SymbolKind;
use ariadne_lang::{
    DependencyRewriter, GeneratedSourceFile, RewriteSpec, SourceExtractor, SymbolBody,
};
use ariadne_transform::{
    ConsumerRewrite, ExtractedSymbol, ExtractionResult, GeneratedUnit, MigrationPreview,
};
use ariadne_validate::TransactionApplier;
use std::fs;
use std::path::{Path, PathBuf};

const CONSUMER_REL: &str = "apps/consumer.src";
const CONSUMER_TEXT: &str = "CONSUMER ORIGINAL\n";

/// Neutral fake extractor: emits bytes no C++ preamble concat could produce,
/// so any file it authored is unmistakably an apply write.
struct NeutralExtractor;

impl SourceExtractor for NeutralExtractor {
    fn extract(
        &self,
        symbols: &[SymbolBody],
        _namespace: &str,
        required_deps: &[String],
    ) -> GeneratedSourceFile {
        let mut content = format!("FAKE-UNIT deps={}\n", required_deps.len());
        for sym in symbols {
            content.push_str(&format!("FAKE-BODY {}\n", sym.name));
        }
        GeneratedSourceFile {
            path: PathBuf::from("unused.fake"),
            content,
        }
    }
}

/// Neutral fake rewriter that genuinely mutates the consumer text, so a
/// restored consumer is observably different from a rewritten one.
struct RenamingRewriter;

impl DependencyRewriter for RenamingRewriter {
    fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
        source.replace("OLD-DEP", "NEW-DEP")
    }
}

fn extracted_symbol(name: &str) -> ExtractedSymbol {
    ExtractedSymbol {
        name: name.to_string(),
        qualified_id: format!("pkg.{name}"),
        kind: SymbolKind::Class,
        body: format!("SYM {name} end"),
    }
}

fn unit(rel_path: &str, symbols: Vec<ExtractedSymbol>) -> GeneratedUnit {
    GeneratedUnit {
        path: PathBuf::from(rel_path),
        symbols,
        required_deps: vec![],
        primary_source_text: None,
    }
}

fn rewrite(consumer_rel: &str, new_unit_rel: &str) -> ConsumerRewrite {
    ConsumerRewrite {
        consumer_file: PathBuf::from(consumer_rel),
        old_includes: vec![PathBuf::from("legacy.src")],
        new_includes: vec![PathBuf::from(new_unit_rel)],
        side_effect_preservations: vec![],
    }
}

fn extraction(plan_id: &str, headers: Vec<GeneratedUnit>) -> ExtractionResult {
    ExtractionResult {
        plan_id: plan_id.to_string(),
        headers,
        errors: vec![],
    }
}

fn migration(plan_id: &str, rewrites: Vec<ConsumerRewrite>) -> MigrationPreview {
    MigrationPreview {
        plan_id: plan_id.to_string(),
        rewrites,
    }
}

fn staged_consumer_repo(temp: &Path) {
    fs::create_dir_all(temp.join("apps")).unwrap();
    fs::write(temp.join(CONSUMER_REL), CONSUMER_TEXT).unwrap();
}

/// Lock 1 (handoff regression lock, first write phase): the second unit's
/// write fails because the slot for its parent directory is occupied by a
/// plain file -- `create_dir_all`/`write` cannot succeed for that target on
/// any platform, deterministically, and the planted file itself is untouched
/// by apply. The first unit's write HAS landed before the failure, so on
/// `Err` the tree must read exactly as before apply.
#[test]
fn failing_write_on_second_unit_restores_the_tree_and_reports_rollback() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path();
    staged_consumer_repo(repo);
    fs::create_dir_all(repo.join("generated")).unwrap();
    // Occupies the parent-directory slot of the second unit's path, making
    // the second unit's creation an guaranteed IO failure.
    fs::write(repo.join("generated").join("blocked"), b"blocker").unwrap();

    let extraction = extraction(
        "P-AR04-A",
        vec![
            unit("first.src", vec![extracted_symbol("One")]),
            unit("blocked/second.src", vec![]),
        ],
    );
    let migration = migration("P-AR04-A", vec![rewrite(CONSUMER_REL, "first.src")]);

    let outcome = TransactionApplier::apply_with(
        repo,
        &extraction,
        &migration,
        &RenamingRewriter,
        &NeutralExtractor,
    );

    let apply_error = match outcome {
        Ok(_) => panic!("the blocked parent must make apply fail, not succeed"),
        Err(err) => err,
    };
    // Snapshot entries: generated/first.src (absent), generated/blocked/
    // second.src (absent), apps/consumer.src (present) = 3. Created paths:
    // generated/first.src = 1. The second unit fails before its creation.
    assert!(
        apply_error
            .to_string()
            .contains("rolled back 4 files"),
        "the propagated error must report the rollback: got {apply_error}"
    );
    assert!(
        !repo.join("generated").join("first.src").exists(),
        "the file written before the failure must not survive an Err apply"
    );
    assert_eq!(
        fs::read(repo.join(CONSUMER_REL)).unwrap(),
        CONSUMER_TEXT.as_bytes(),
        "the untouched consumer must be byte-identical after a rolled-back apply"
    );
    assert_eq!(
        fs::read(repo.join("generated").join("blocked")).unwrap(),
        b"blocker",
        "the fixture blocker is test state, never an apply target"
    );
}

/// Lock 2 (handoff: "even on partial failure, all previously written files
/// are restored"): two write phases succeed -- the generated unit is created
/// and the first consumer is genuinely rewritten -- and only the SECOND
/// consumer file's rewrite fails, via invalid UTF-8 bytes staged through byte
/// escapes (never a literal). Rollback must DELETE the created unit and
/// RESTORE the mutated consumer; an error that leaves either behind breaks
/// the transactional promise.
#[test]
fn failing_second_consumer_rewrite_restores_every_earlier_write() {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path();
    fs::create_dir_all(repo.join("apps")).unwrap();
    let first_consumer = repo.join("apps").join("one.src");
    let second_consumer = repo.join("apps").join("two.src");
    fs::write(&first_consumer, b"OLD-DEP line\n").unwrap();
    // Corruption signature built from byte escapes, per repo rule: the read
    // inside the rewrite phase cannot decode it, so the failure lands after
    // two successful writes.
    fs::write(&second_consumer, b"PRE \xff\xfe post\n").unwrap();

    let extraction = extraction("P-AR04-B", vec![unit("one.src", vec![extracted_symbol("Two")])]);
    let migration = migration(
        "P-AR04-B",
        vec![
            rewrite("apps/one.src", "one.src"),
            rewrite("apps/two.src", "one.src"),
        ],
    );

    let outcome = TransactionApplier::apply_with(
        repo,
        &extraction,
        &migration,
        &RenamingRewriter,
        &NeutralExtractor,
    );

    let apply_error = match outcome {
        Ok(_) => panic!("the undecodable consumer must make apply fail, not succeed"),
        Err(err) => err,
    };
    // Snapshot entries: generated/one.src (absent), apps/one.src (present),
    // apps/two.src (present) = 3. Created paths: generated/one.src = 1.
    assert!(
        apply_error
            .to_string()
            .contains("rolled back 4 files"),
        "the propagated error must report the rollback: got {apply_error}"
    );
    assert!(
        !repo.join("generated").join("one.src").exists(),
        "the created unit written in phase one must be removed on Err"
    );
    assert_eq!(
        fs::read(&first_consumer).unwrap(),
        b"OLD-DEP line\n",
        "the consumer rewritten before the failure must be restored, not left mutated"
    );
    assert_eq!(
        fs::read(&second_consumer).unwrap(),
        b"PRE \xff\xfe post\n",
        "the failing consumer must be left exactly as staged"
    );
}
