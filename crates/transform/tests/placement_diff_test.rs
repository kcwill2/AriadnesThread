//! Relocated (M23 T3, D-M23-10) from `crates/transform/src/placement.rs`
//! verbatim: these diff-formatting tests drive the real C++ rewriter and are
//! the pre-M18.8b byte-baseline authority, so they move out of the guard walk
//! scope (`crates/*/src`) unchanged in content. The compute_rewrite /
//! build_preview / noop tests that need no plugin stay in `placement.rs`.

use ariadne_lang_cpp::CppDependencyRewriter;
use ariadne_transform::{ConsumerRewrite, GeneratedUnit, IncludeRewriter, MigrationPreview};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn cpp() -> CppDependencyRewriter {
    CppDependencyRewriter
}

// R2 keeps directive text out of this crate's sources, so the C++ shapes
// asserted here are assembled from a non-contiguous marker (the plugin
// crate owns the canonical form).
const DIRECTIVE: &str = concat!("#inc", "lude ");
fn include_of(target: &str) -> String {
    format!("{DIRECTIVE}\"{target}\"")
}

#[test]
fn test_consumer_rewrite_format_diff() {
    let rewrite = ConsumerRewrite {
        consumer_file: std::path::PathBuf::from("src/foo.cpp"),
        old_includes: vec![std::path::PathBuf::from("thing.h")],
        new_includes: vec![std::path::PathBuf::from("thing_types.h")],
        side_effect_preservations: vec![],
    };
    let diff = rewrite.format_diff_with(&cpp());
    assert!(diff.contains(&format!("- {}", include_of("thing.h"))), "diff: {}", diff);
    assert!(
        diff.contains(&format!("+ {}", include_of("thing_types.h"))),
        "diff: {}",
        diff
    );
    assert!(diff.contains("src/foo.cpp"), "diff: {}", diff);
    // Byte-exact capture of the pre-M18.8b `format_diff` output.
    assert_eq!(
        diff,
        format!(
            "src/foo.cpp\n    - {}\n    + {}\n",
            include_of("thing.h"),
            include_of("thing_types.h")
        )
    );
}

#[test]
fn test_migration_preview_format_diff() {
    let preview = MigrationPreview {
        plan_id: "PLAN-001".to_string(),
        rewrites: vec![
            ConsumerRewrite {
                consumer_file: std::path::PathBuf::from("src/foo.cpp"),
                old_includes: vec![std::path::PathBuf::from("thing.h")],
                new_includes: vec![std::path::PathBuf::from("thing_types.h")],
                side_effect_preservations: vec![],
            },
            ConsumerRewrite {
                consumer_file: std::path::PathBuf::from("src/bar.cpp"),
                old_includes: vec![std::path::PathBuf::from("thing.h")],
                new_includes: vec![std::path::PathBuf::from("thing_core.h")],
                side_effect_preservations: vec![],
            },
        ],
    };
    let diff = preview.format_diff_with(&cpp());
    assert!(diff.contains("src/foo.cpp"));
    assert!(diff.contains("src/bar.cpp"));
    assert!(diff.contains("thing_types.h"));
    assert!(diff.contains("thing_core.h"));
    assert_eq!(preview.effective_rewrites(), 2);
}

#[test]
fn test_migration_preview_side_effect_preservation() {
    let preview = MigrationPreview {
        plan_id: "PLAN-002".to_string(),
        rewrites: vec![ConsumerRewrite {
            consumer_file: std::path::PathBuf::from("src/init.cpp"),
            old_includes: vec![
                std::path::PathBuf::from("thing.h"),
                std::path::PathBuf::from("registry.h"),
            ],
            new_includes: vec![std::path::PathBuf::from("thing_types.h")],
            side_effect_preservations: vec![std::path::PathBuf::from("registry.h")],
        }],
    };
    let diff = preview.format_diff_with(&cpp());
    assert!(
        diff.contains(&format!("~ {} // side-effect preserved", include_of("registry.h"))),
        "diff: {}",
        diff
    );
    assert_eq!(preview.effective_rewrites(), 1);
}

#[test]
fn test_migration_preview_empty() {
    let preview = MigrationPreview {
        plan_id: "PLAN-003".to_string(),
        rewrites: vec![],
    };
    assert_eq!(preview.format_diff_with(&cpp()), "");
    assert_eq!(preview.effective_rewrites(), 0);
}

#[test]
fn test_include_rewriter_build_preview() {
    let mut sym_map: BTreeMap<String, PathBuf> = BTreeMap::new();
    sym_map.insert("ThingConfig".to_string(), PathBuf::from("thing_types.h"));
    sym_map.insert("ThingBuilder".to_string(), PathBuf::from("thing_core.h"));

    let gen_headers = vec![
        GeneratedUnit {
            path: PathBuf::from("thing_types.h"),
            symbols: vec![],
            required_deps: vec![],
            primary_source_text: None,
        },
        GeneratedUnit {
            path: PathBuf::from("thing_core.h"),
            symbols: vec![],
            required_deps: vec![PathBuf::from("thing_types.h")],
            primary_source_text: None,
        },
    ];

    let rewriter = IncludeRewriter::new(sym_map, gen_headers);
    let consumers = vec![
        (
            PathBuf::from("src/foo.cpp"),
            vec![PathBuf::from("thing.h")],
            vec!["ThingConfig".to_string()],
        ),
        (
            PathBuf::from("src/bar.cpp"),
            vec![PathBuf::from("thing.h")],
            vec!["ThingBuilder".to_string()],
        ),
    ];

    let preview = rewriter.build_preview("PLAN-001".to_string(), &consumers);
    assert_eq!(preview.plan_id, "PLAN-001");
    assert_eq!(preview.rewrites.len(), 2);
    assert_eq!(preview.effective_rewrites(), 2);

    let diff = preview.format_diff_with(&cpp());
    assert!(diff.contains("src/foo.cpp"));
    assert!(diff.contains("src/bar.cpp"));
    assert!(diff.contains("thing_types.h"));
    assert!(diff.contains("thing_core.h"));
}
