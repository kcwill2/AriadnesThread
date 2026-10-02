//! M23 T7b (kickoff §8 item 2, I5): move-removal transaction wiring.
//! `apply_with` must run `DependencyRewriter::remove_top_level_symbols` on
//! each moved symbol's existing source file INSIDE the FileSnapshot
//! transaction: `Some` ⇒ stripped content written back and rollback
//! restores the original bytes; `None` (the C++ default) ⇒ the source file
//! is byte-identical (copy-only semantics preserved). The decision lives
//! entirely in the hook result — validate names no language (I2).

use ariadne_index::SymbolKind;
use ariadne_lang::{DependencyRewriter, RewriteSpec};
use ariadne_transform::{
    ConsumerRewrite, ExtractedSymbol, ExtractionResult, GeneratedUnit, MigrationPreview,
};
use ariadne_validate::TransactionApplier;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

const SOURCE_REL: &str = "pkg/legacy.src";
const CONSUMER_REL: &str = "apps/consumer.src";
const MOVED_NAME: &str = "pkg.mod.LegacyReport";
const ORIGINAL_SOURCE: &str = "header line\nSYM pkg.mod.LegacyReport end\nkeeper line\n";
const STRIPPED_SOURCE: &str = "header line\nkeeper line\n";

/// Neutral fake: strips `SYM <name> end` lines (a `Some` move-capable
/// language) and records the name lists it was handed.
struct MoveRewriter {
    seen_names: Mutex<Vec<Vec<String>>>,
}

impl MoveRewriter {
    fn new() -> Self {
        Self {
            seen_names: Mutex::new(Vec::new()),
        }
    }
}

impl DependencyRewriter for MoveRewriter {
    fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
        source.to_string()
    }

    fn remove_top_level_symbols(&self, source: &str, qualified_names: &[String]) -> Option<String> {
        self.seen_names
            .lock()
            .expect("fake names lock")
            .push(qualified_names.to_vec());
        let kept: Vec<&str> = source
            .lines()
            .filter(|line| !qualified_names.iter().any(|n| line.contains(n.rsplit('.').next().unwrap_or(n))))
            .collect();
        Some(format!("{}\n", kept.join("\n")))
    }
}

/// Copy-only fake: the trait default `None` stands.
struct CopyRewriter;

impl DependencyRewriter for CopyRewriter {
    fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
        source.to_string()
    }
}

struct FixtureRepo {
    _temp: tempfile::TempDir,
    root: PathBuf,
}

fn staged_repo() -> FixtureRepo {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().to_path_buf();
    fs::create_dir_all(root.join("pkg")).unwrap();
    fs::create_dir_all(root.join("apps")).unwrap();
    fs::write(root.join(SOURCE_REL), ORIGINAL_SOURCE).unwrap();
    fs::write(root.join(CONSUMER_REL), "consumer body\n").unwrap();
    FixtureRepo { _temp: temp, root }
}

fn extraction_with_moved_symbol() -> ExtractionResult {
    ExtractionResult {
        plan_id: "P7B".into(),
        headers: vec![GeneratedUnit {
            path: "pkg/LegacyReport.src".into(),
            symbols: vec![ExtractedSymbol {
                name: MOVED_NAME.to_string(),
                kind: SymbolKind::Class,
                body: "SYM body".into(),
            }],
            required_deps: vec![],
            primary_source_text: None,
        }],
        errors: vec![],
    }
}

fn migration_pointing_at_the_moved_symbol() -> MigrationPreview {
    MigrationPreview {
        plan_id: "P7B".into(),
        rewrites: vec![ConsumerRewrite {
            consumer_file: CONSUMER_REL.into(),
            old_includes: vec![SOURCE_REL.into()],
            new_includes: vec!["pkg/LegacyReport.src".into()],
            side_effect_preservations: vec![],
        }],
    }
}

#[test]
fn some_hook_result_strips_the_source_file_and_rollback_restores_it() {
    let repo = staged_repo();
    let extraction = extraction_with_moved_symbol();
    let migration = migration_pointing_at_the_moved_symbol();
    let rewriter = MoveRewriter::new();

    let (snapshot, new_paths) = TransactionApplier::apply_with(
        &repo.root,
        &extraction,
        &migration,
        &rewriter,
        &ariadne_lang_cpp::CppSourceExtractor,
    )
    .unwrap();

    assert_eq!(
        fs::read_to_string(repo.root.join(SOURCE_REL)).unwrap(),
        STRIPPED_SOURCE,
        "Some ⇒ the moved symbol is removed from its source file (I5 move)"
    );
    assert_eq!(
        rewriter.seen_names.lock().unwrap().as_slice(),
        &[vec![MOVED_NAME.to_string()]][..],
        "the hook is handed the moved unit's qualified symbol names"
    );

    TransactionApplier::rollback(&snapshot, &new_paths).unwrap();
    assert_eq!(
        fs::read(repo.root.join(SOURCE_REL)).unwrap(),
        ORIGINAL_SOURCE.as_bytes(),
        "the strip rode inside the FileSnapshot transaction — rollback restores it"
    );
    assert!(TransactionApplier::verify_restored(&snapshot).is_ok());
}

#[test]
fn none_hook_result_leaves_the_source_file_byte_identical() {
    let repo = staged_repo();
    let before = fs::read(repo.root.join(SOURCE_REL)).unwrap();

    let (snapshot, new_paths) = TransactionApplier::apply_with(
        &repo.root,
        &extraction_with_moved_symbol(),
        &migration_pointing_at_the_moved_symbol(),
        &CopyRewriter,
        &ariadne_lang_cpp::CppSourceExtractor,
    )
    .unwrap();

    assert_eq!(
        fs::read(repo.root.join(SOURCE_REL)).unwrap(),
        before,
        "None (C++ default) ⇒ the source file is untouched, copy-only semantics (I5)"
    );
    assert!(
        !new_paths
            .iter()
            .any(|p| p == &repo.root.join("generated").join(SOURCE_REL)),
        "the untouched source is never registered as a created path"
    );
    TransactionApplier::rollback(&snapshot, &new_paths).unwrap();
    assert!(TransactionApplier::verify_restored(&snapshot).is_ok());
}
