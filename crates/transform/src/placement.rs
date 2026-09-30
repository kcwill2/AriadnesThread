//! Placement: mapping generated headers onto consumer include migrations.

use crate::GeneratedUnit;
use ariadne_lang::DependencyRewriter;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

fn path_strings(paths: &[PathBuf]) -> Vec<String> {
    paths.iter().map(|p| p.display().to_string()).collect()
}

/// Represents one consumer file's include migration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConsumerRewrite {
    /// The consumer source/header file being rewritten.
    pub consumer_file: std::path::PathBuf,
    /// The old monolithic include(s) being replaced.
    pub old_includes: Vec<std::path::PathBuf>,
    /// The new targeted include(s) that replace them.
    pub new_includes: Vec<std::path::PathBuf>,
    /// Includes that must be preserved due to side effects (macros, global init, etc.).
    pub side_effect_preservations: Vec<std::path::PathBuf>,
}

impl ConsumerRewrite {
    /// Returns true if this rewrite actually changes anything.
    pub fn is_noop(&self) -> bool {
        self.old_includes == self.new_includes && self.side_effect_preservations.is_empty()
    }

    /// Format as a unified-diff-style patch fragment for this consumer.
    ///
    /// The directive-line text (and its exact spelling) belongs to the
    /// language plugin's [`DependencyRewriter`]; this engine only owns the
    /// neutral redirect logic and the consumer-file header line.
    pub fn format_diff_with(&self, rewriter: &dyn DependencyRewriter) -> String {
        let mut out = String::new();
        out.push_str(&format!("{}\n", self.consumer_file.display()));
        out.push_str(&rewriter.render_diff(
            &path_strings(&self.old_includes),
            &path_strings(&self.new_includes),
            &path_strings(&self.side_effect_preservations),
        ));
        out
    }
}

/// A preview of include migrations for all consumers affected by a plan.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MigrationPreview {
    pub plan_id: String,
    pub rewrites: Vec<ConsumerRewrite>,
}

impl MigrationPreview {
    /// Format the full preview as a human-readable diff, rendering each
    /// rewrite's directive lines through the plugin's rewriter.
    pub fn format_diff_with(&self, rewriter: &dyn DependencyRewriter) -> String {
        let mut out = String::new();
        for rewrite in &self.rewrites {
            out.push_str(&rewrite.format_diff_with(rewriter));
            out.push('\n');
        }
        out
    }

    /// Returns the number of rewrites that actually change something.
    pub fn effective_rewrites(&self) -> usize {
        self.rewrites.iter().filter(|r| !r.is_noop()).count()
    }
}

/// Rewrites consumer include directives based on symbol-to-header mappings.
pub struct IncludeRewriter {
    /// Maps symbol name -> the generated header filename that owns it
    symbol_name_to_header: BTreeMap<String, PathBuf>,
    /// All generated units, needed to resolve required deps (predecessors)
    generated_units: Vec<GeneratedUnit>,
}

impl IncludeRewriter {
    pub fn new(
        symbol_name_to_header: BTreeMap<String, PathBuf>,
        generated_units: Vec<GeneratedUnit>,
    ) -> Self {
        Self {
            symbol_name_to_header,
            generated_units,
        }
    }

    /// Build a ConsumerRewrite for one consumer file.
    ///
    /// - `consumer_file`: path of the consumer (.cpp or .h)
    /// - `old_includes`: the monolithic include(s) being replaced
    /// - `referenced_symbols`: symbol names from those includes that this consumer actually uses
    pub fn compute_rewrite(
        &self,
        consumer_file: PathBuf,
        old_includes: Vec<PathBuf>,
        referenced_symbols: &[String],
    ) -> ConsumerRewrite {
        // 1. Map each referenced symbol to its target header
        let mut target_headers: BTreeSet<PathBuf> = BTreeSet::new();
        let mut covered_old_includes: BTreeSet<PathBuf> = BTreeSet::new();

        for sym_name in referenced_symbols {
            if let Some(header) = self.symbol_name_to_header.get(sym_name) {
                target_headers.insert(header.clone());
                // Find which old_include this symbol came from by checking generated header's symbols
                for old_inc in &old_includes {
                    // If the target header is in the symbol_name_to_header map, this old_include is covered
                    covered_old_includes.insert(old_inc.clone());
                }
            }
        }

        // 2. Add required predecessor includes for each target header
        let mut all_new_includes: BTreeSet<PathBuf> = BTreeSet::new();
        for target in &target_headers {
            all_new_includes.insert(target.clone());
            // Find the GeneratedUnit to get its required deps
            if let Some(gen_unit) = self.generated_units.iter().find(|u| &u.path == target) {
                for prereq in &gen_unit.required_deps {
                    all_new_includes.insert(prereq.clone());
                }
            }
        }

        // 3. Side-effect guard: old includes with NO referenced symbols -> preserve
        let mut side_effect_preservations: Vec<PathBuf> = Vec::new();
        if referenced_symbols.is_empty() {
            // No symbols at all -> preserve all old includes
            for old_inc in &old_includes {
                side_effect_preservations.push(old_inc.clone());
            }
        } else {
            // Preserve old includes not covered by any referenced symbol's target header
            for old_inc in &old_includes {
                if !covered_old_includes.contains(old_inc) {
                    side_effect_preservations.push(old_inc.clone());
                }
            }
        }

        let new_includes: Vec<PathBuf> = all_new_includes.into_iter().collect();

        ConsumerRewrite {
            consumer_file,
            old_includes,
            new_includes,
            side_effect_preservations,
        }
    }

    /// Build a full MigrationPreview for all consumers of a plan.
    ///
    /// - `plan_id`: the plan identifier string
    /// - `consumers`: slice of (consumer_file, old_includes, referenced_symbol_names)
    pub fn build_preview(
        &self,
        plan_id: String,
        consumers: &[(PathBuf, Vec<PathBuf>, Vec<String>)],
    ) -> MigrationPreview {
        let rewrites = consumers
            .iter()
            .map(|(file, old_incs, syms)| {
                self.compute_rewrite(file.clone(), old_incs.clone(), syms)
            })
            .collect();
        MigrationPreview { plan_id, rewrites }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ariadne_lang_cpp::CppDependencyRewriter;

    fn cpp() -> CppDependencyRewriter {
        CppDependencyRewriter
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
        assert!(diff.contains("- #include \"thing.h\""), "diff: {}", diff);
        assert!(
            diff.contains("+ #include \"thing_types.h\""),
            "diff: {}",
            diff
        );
        assert!(diff.contains("src/foo.cpp"), "diff: {}", diff);
        // Byte-exact capture of the pre-M18.8b `format_diff` output.
        assert_eq!(
            diff,
            "src/foo.cpp\n    - #include \"thing.h\"\n    + #include \"thing_types.h\"\n"
        );
    }

    #[test]
    fn test_consumer_rewrite_is_noop() {
        let noop = ConsumerRewrite {
            consumer_file: std::path::PathBuf::from("src/foo.cpp"),
            old_includes: vec![std::path::PathBuf::from("thing.h")],
            new_includes: vec![std::path::PathBuf::from("thing.h")],
            side_effect_preservations: vec![],
        };
        assert!(noop.is_noop());

        let not_noop = ConsumerRewrite {
            consumer_file: std::path::PathBuf::from("src/bar.cpp"),
            old_includes: vec![std::path::PathBuf::from("thing.h")],
            new_includes: vec![std::path::PathBuf::from("thing_types.h")],
            side_effect_preservations: vec![],
        };
        assert!(!not_noop.is_noop());
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
            diff.contains("~ #include \"registry.h\" // side-effect preserved"),
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
    fn test_include_rewriter_single_symbol_to_single_header() {
        let mut sym_map: BTreeMap<String, PathBuf> = BTreeMap::new();
        sym_map.insert("ThingConfig".to_string(), PathBuf::from("thing_types.h"));

        let gen_headers = vec![GeneratedUnit {
            path: PathBuf::from("thing_types.h"),
            symbols: vec![],
            required_deps: vec![],
        }];

        let rewriter = IncludeRewriter::new(sym_map, gen_headers);
        let rewrite = rewriter.compute_rewrite(
            PathBuf::from("src/foo.cpp"),
            vec![PathBuf::from("thing.h")],
            &["ThingConfig".to_string()],
        );

        assert_eq!(rewrite.consumer_file, PathBuf::from("src/foo.cpp"));
        assert!(
            rewrite
                .new_includes
                .contains(&PathBuf::from("thing_types.h")),
            "new_includes: {:?}",
            rewrite.new_includes
        );
        assert!(rewrite.side_effect_preservations.is_empty());
    }

    #[test]
    fn test_include_rewriter_multiple_symbols_to_multiple_headers() {
        let mut sym_map: BTreeMap<String, PathBuf> = BTreeMap::new();
        sym_map.insert("ThingConfig".to_string(), PathBuf::from("thing_types.h"));
        sym_map.insert("ThingBuilder".to_string(), PathBuf::from("thing_core.h"));

        let gen_headers = vec![
            GeneratedUnit {
                path: PathBuf::from("thing_types.h"),
                symbols: vec![],
                required_deps: vec![],
            },
            GeneratedUnit {
                path: PathBuf::from("thing_core.h"),
                symbols: vec![],
                required_deps: vec![PathBuf::from("thing_types.h")],
            },
        ];

        let rewriter = IncludeRewriter::new(sym_map, gen_headers);
        let rewrite = rewriter.compute_rewrite(
            PathBuf::from("src/bar.cpp"),
            vec![PathBuf::from("thing.h")],
            &["ThingConfig".to_string(), "ThingBuilder".to_string()],
        );

        // Should have both target headers (and thing_types.h as predecessor of thing_core.h)
        assert!(
            rewrite
                .new_includes
                .contains(&PathBuf::from("thing_types.h")),
            "new_includes: {:?}",
            rewrite.new_includes
        );
        assert!(
            rewrite
                .new_includes
                .contains(&PathBuf::from("thing_core.h")),
            "new_includes: {:?}",
            rewrite.new_includes
        );
        assert!(rewrite.side_effect_preservations.is_empty());
    }

    #[test]
    fn test_include_rewriter_side_effect_guard_no_symbols() {
        // Critical negative test: if referenced_symbols is empty, the old include must be preserved
        let sym_map: BTreeMap<String, PathBuf> = BTreeMap::new();
        let gen_headers = vec![];

        let rewriter = IncludeRewriter::new(sym_map, gen_headers);
        let rewrite = rewriter.compute_rewrite(
            PathBuf::from("src/init.cpp"),
            vec![PathBuf::from("registry.h")],
            &[], // No referenced symbols
        );

        // Must NOT silently remove registry.h
        assert!(
            rewrite
                .side_effect_preservations
                .contains(&PathBuf::from("registry.h")),
            "side_effect_preservations: {:?}",
            rewrite.side_effect_preservations
        );
        assert!(
            rewrite.new_includes.is_empty(),
            "new_includes should be empty: {:?}",
            rewrite.new_includes
        );
    }

    #[test]
    fn test_include_rewriter_predecessor_includes() {
        // thing_core.h requires thing_types.h — both should appear in new_includes
        let mut sym_map: BTreeMap<String, PathBuf> = BTreeMap::new();
        sym_map.insert("ThingBuilder".to_string(), PathBuf::from("thing_core.h"));

        let gen_headers = vec![GeneratedUnit {
            path: PathBuf::from("thing_core.h"),
            symbols: vec![],
            required_deps: vec![PathBuf::from("thing_types.h")],
        }];

        let rewriter = IncludeRewriter::new(sym_map, gen_headers);
        let rewrite = rewriter.compute_rewrite(
            PathBuf::from("src/builder_user.cpp"),
            vec![PathBuf::from("thing.h")],
            &["ThingBuilder".to_string()],
        );

        assert!(rewrite
            .new_includes
            .contains(&PathBuf::from("thing_core.h")));
        assert!(
            rewrite
                .new_includes
                .contains(&PathBuf::from("thing_types.h")),
            "Predecessor thing_types.h should be included. new_includes: {:?}",
            rewrite.new_includes
        );
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
            },
            GeneratedUnit {
                path: PathBuf::from("thing_core.h"),
                symbols: vec![],
                required_deps: vec![PathBuf::from("thing_types.h")],
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
}
