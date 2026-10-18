use anyhow::Result;
use ariadne_pipeline::Pipeline;
use ariadne_repo::Repository;
use ariadne_report::ArtifactManager;
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

/// Distinct discovered languages whose registered plugin disables the
/// planning stage (plan M22, D23). Registry lookup only (rule R3): a language
/// with no plugin (e.g. `Other`) is not a gap — the planner never hears of
/// it; a plugin with `planning == true` never is either. Each gap carries the
/// first discovered path of that language so the rejection names an honest
/// `target_file`. Pure so the rule is testable without a repository.
/// Post-M23-flip every registered plugin plans, so this yields no gaps for
/// the default registry; the rule (and the cluster-side gap→rejection
/// mechanism it feeds) stays for any future planning=false plugin.
fn planning_gaps(
    files: &[ariadne_repo::FileRecord],
    registry: &ariadne_lang::LanguageRegistry,
) -> Vec<ariadne_cluster::planner::PlanningGap> {
    let mut gaps: Vec<ariadne_cluster::planner::PlanningGap> = Vec::new();
    for record in files {
        // D-A5: the registry decides, not the record's file-kind label.
        let Some(language) = registry.detect_language(&record.path) else {
            continue;
        };
        let Some(plugin) = registry.plugin_for_id(language.clone()) else {
            continue;
        };
        if plugin.capabilities().planning {
            continue;
        }
        if gaps.iter().any(|gap| gap.language == language) {
            continue;
        }
        gaps.push(ariadne_cluster::planner::PlanningGap {
            language,
            sample_path: record.path.clone(),
        });
    }
    gaps
}

/// Per-language planning hooks (AR-19, A1 part 2): the dispatch retired the
/// CLI-side 2-arm language match (the D-M23-7 i closures) in favor of the
/// registry-plugin hooks in [`crate::registry::planning_hooks`] -- the same
/// seam the optimize and UI lanes now share. C++ paths keep byte-identical
/// answers (the cpp plugin's trait hook IS the old free function, AR-18
/// parity); the honest no-rule answers (not an aggregation unit; candidate
/// unchanged) come from the absence of a claiming plugin, not from an arm.

/// The single [`ClusteringConstraints`](ariadne_lang::ClusteringConstraints)
/// injected into the planner (D-M23-7 i): resolves each record to its
/// owning file's language (record `owning_file` → catalog), groups the
/// batch per language, and delegates to that language's plugin constraints.
/// Lives in the CLI — the dispatch DATA is injected from here, the core
/// planner stays plugin-blind (I2). Records whose file language no plugin
/// claims are simply unconstrained. Symbol NAMES are never dispatch keys
/// (G2b advisory 5: duplicate top-level names are not excluded).
struct LanguageRoutingConstraints<'a> {
    registry: &'a ariadne_lang::LanguageRegistry,
    language_of_file: &'a BTreeMap<ariadne_repo::FileId, ariadne_lang::LanguageId>,
}

impl<'a> ariadne_lang::ClusteringConstraints for LanguageRoutingConstraints<'a> {
    fn colocated_groups(
        &self,
        symbols: &[ariadne_lang::SymbolRecord],
    ) -> Vec<Vec<ariadne_lang::SymbolId>> {
        let mut groups = Vec::new();
        for language in self.registry.languages() {
            let Some(plugin) = self.registry.plugin_for_id(language.clone()) else {
                continue;
            };
            let subset: Vec<ariadne_lang::SymbolRecord> = symbols
                .iter()
                .filter(|record| {
                    self.language_of_file
                        .get(&record.owning_file)
                        .is_some_and(|label| *label == language)
                })
                .cloned()
                .collect();
            if !subset.is_empty() {
                groups.extend(plugin.constraints().colocated_groups(&subset));
            }
        }
        groups
    }

    /// D-M23-8 routing: the typed extends channel must reach the plugin that
    /// owns each edge — without this override the trait default would drop
    /// the channel for every language and the wired CLI route could never
    /// co-locate sealed pairs. Empty edges (every C++ model, D-M23-8 fold)
    /// take the byte-identical `colocated_groups` path; otherwise edges are
    /// filtered to the records of each language's file language before
    /// delegation.
    fn colocated_groups_with_edges(
        &self,
        symbols: &[ariadne_lang::SymbolRecord],
        extends_edges: &[(ariadne_lang::SymbolId, ariadne_lang::SymbolId)],
    ) -> Vec<Vec<ariadne_lang::SymbolId>> {
        if extends_edges.is_empty() {
            return self.colocated_groups(symbols);
        }
        let mut groups = Vec::new();
        for language in self.registry.languages() {
            let Some(plugin) = self.registry.plugin_for_id(language.clone()) else {
                continue;
            };
            let subset: Vec<ariadne_lang::SymbolRecord> = symbols
                .iter()
                .filter(|record| {
                    self.language_of_file
                        .get(&record.owning_file)
                        .is_some_and(|label| *label == language)
                })
                .cloned()
                .collect();
            if subset.is_empty() {
                continue;
            }
            let ids: std::collections::HashSet<&ariadne_lang::SymbolId> =
                subset.iter().map(|record| &record.id).collect();
            let subset_edges: Vec<(ariadne_lang::SymbolId, ariadne_lang::SymbolId)> =
                extends_edges
                    .iter()
                    .filter(|(src, dst)| ids.contains(src) && ids.contains(dst))
                    .cloned()
                    .collect();
            groups.extend(
                plugin
                    .constraints()
                    .colocated_groups_with_edges(&subset, &subset_edges),
            );
        }
        groups
    }

    fn frozen_symbols(
        &self,
        symbols: &[ariadne_lang::SymbolRecord],
    ) -> HashSet<ariadne_lang::SymbolId> {
        let mut frozen = HashSet::new();
        for language in self.registry.languages() {
            let Some(plugin) = self.registry.plugin_for_id(language.clone()) else {
                continue;
            };
            let subset: Vec<ariadne_lang::SymbolRecord> = symbols
                .iter()
                .filter(|record| {
                    self.language_of_file
                        .get(&record.owning_file)
                        .is_some_and(|label| *label == language)
                })
                .cloned()
                .collect();
            if !subset.is_empty() {
                frozen.extend(plugin.constraints().frozen_symbols(&subset));
            }
        }
        frozen
    }
}

pub fn plan_extractions(
    repo_path: &PathBuf,
) -> Result<Vec<ariadne_cluster::plan::OptimizationPlan>> {
    let mut pipeline = Pipeline::new(repo_path, None)?;
    let (graph, _closures) = pipeline.get_or_build_include_graph()?;
    let model = pipeline.get_or_build_reduced()?;
    let tu_file_ids: Vec<_> = pipeline
        .build
        .translation_units
        .iter()
        .map(|tu| tu.file_id)
        .collect();

    let registry = crate::registry::default_registry();
    let gaps = planning_gaps(&pipeline.discovery.files, &registry);

    // AR-19 (A1 part 2): registry-dispatched planning hooks (see the module
    // note above) -- one dispatch for the plan/optimize/UI lanes; C++ files
    // keep byte-identical hooks (locked in registry.rs + m00).
    let hooks = crate::registry::planning_hooks(&registry, &pipeline.repo.root);
    let catalog = crate::registry::catalog_languages(&registry, &pipeline.discovery);
    let constraints = LanguageRoutingConstraints {
        registry: &registry,
        language_of_file: &catalog,
    };
    let mut planner = ariadne_cluster::planner::OptimizationPlanner::new(
        &graph,
        &model,
        &pipeline.discovery.files,
        &tu_file_ids,
        &hooks.aggregation,
        &hooks.target,
        Some(&constraints),
        // cycle_policy is INFORMATIONAL to the planner: every policy shares
        // the SCC co-proposal path (m18.7 — locked by
        // `cycle_policy_variants_share_the_scc_coproposal_path_for_now`);
        // the stance only steers the report-side cycle classification.
        ariadne_lang::CyclePolicy::Reject,
    );
    planner.with_planning_gaps(gaps);
    let (plans, rejections) = planner.plan_with_rejections();

    pipeline
        .manager
        .save_plan_artifacts(&pipeline.repo.root, &plans, &pipeline.fps.plan)?;

    println!("Optimization plans: {}", plans.len());
    if !rejections.is_empty() {
        println!();
        for rejection in &rejections {
            println!("Planning rejected: {}", rejection.explanation);
        }
    }
    println!();
    println!("{}", pipeline.stats.format_summary());
    println!();
    for plan in &plans {
        println!("{}", plan.format_detail());
    }

    Ok(plans)
}

pub fn show_optimization_plan(repo_path: &PathBuf, id: &str) -> Result<()> {
    let repository = Repository::locate(repo_path)?;
    let artifacts = ArtifactManager::prepare_for_repo(&repository)?;

    let plans = match artifacts.load_plan_artifacts() {
        Ok(p) => p,
        Err(_) => plan_extractions(repo_path)?,
    };

    if let Some(plan) = plans.iter().find(|p| p.id.0 == id) {
        println!("{}", plan.format_detail());
    } else {
        println!("Plan {} not found.", id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{planning_gaps, LanguageRoutingConstraints};
    use ariadne_lang::ClusteringConstraints;
    use ariadne_repo::{
        FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn record(rel: &str, kind: FileKind, language: SourceLanguage) -> FileRecord {
        let path = PathBuf::from(rel);
        FileRecord {
            id: FileId::from_relative_path(rel),
            path: path.clone(),
            canonical_path: path,
            kind,
            size: 0,
            content_hash: String::new(),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
            language,
        }
    }

    fn symbol(
        id: &str,
        name: &str,
        kind: ariadne_lang::SymbolKind,
        owning_file: FileId,
    ) -> ariadne_lang::SymbolRecord {
        ariadne_lang::SymbolRecord {
            id: ariadne_lang::SymbolId::new(id),
            name: name.to_string(),
            kind,
            owning_file,
            declarations: vec![],
            definition: None,

            modifiers: Vec::new(),
        }
    }

    #[test]
    fn plan_lane_hooks_are_the_registry_dispatch_cpp_and_scala_stay_correct() {
        // AR-19 (A1 part 2): the D-M23-7 i 2-arm table and its
        // `language_of_path` lookup retired into
        // `crate::registry::planning_hooks`; the shared hooks themselves are
        // locked at that seam (registry.rs `planning_hooks_*` tests, which
        // absorbed the D-A4/D-A5 witnesses: third language gets no C++
        // semantics, the registry - never a label - dispatches) and on the
        // real repo path by `m24a_scala_plan_path_test`. This keeps the
        // PLAN-LANE witness: the hooks `plan_extractions` feeds the planner
        // route Scala paths to the Scala plugin and keep C++ bytes.
        let registry = crate::registry::default_registry();
        let fixture_root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/scala-basic");
        let hooks = crate::registry::planning_hooks(&registry, &fixture_root);
        let scala_agg = PathBuf::from("src/main/scala/com/example/models/all.scala");
        assert!(
            (hooks.aggregation)(&scala_agg),
            "the real fixture all.scala is a Scala aggregation unit through \
             the CLI plan dispatch (plugin trait hook, not test injection)"
        );
        assert!(
            !(hooks.aggregation)(&PathBuf::from(
                "src/main/scala/com/example/models/User.scala"
            )),
            "a Scala file without a package-object clause is NOT an \
             aggregation unit - the C++ extension filter never claims it"
        );
        assert_eq!(
            (hooks.target)(&scala_agg, "com.example.models.User"),
            ariadne_lang_scala::scala_target_path_for(&scala_agg, "com.example.models.User"),
            "Scala candidates get the Scala target namer verbatim"
        );
        let cpp_rel = PathBuf::from("include/all.h");
        assert_eq!(
            (hooks.target)(&cpp_rel, "Widget"),
            ariadne_lang_cpp::cpp_target_path_for(&cpp_rel, "Widget"),
            "non-Scala candidates keep cpp_target_path_for byte-identically"
        );
        assert_eq!(
            (hooks.aggregation)(&cpp_rel),
            ariadne_lang_cpp::cpp_aggregation_filter(&cpp_rel),
            "non-Scala aggregation decisions keep cpp_aggregation_filter \
             byte-identically (pre-dispatch behavior for C++ repos)"
        );
    }

    #[test]
    fn routing_constraints_delegate_per_language_scala_pairs_and_cpp_stays_empty() {
        let registry = crate::registry::default_registry();
        let scala_file = FileId::from_relative_path("src/main/scala/p/Legacy.scala");
        let cpp_file = FileId::from_relative_path("include/all.h");
        let mut catalog: BTreeMap<FileId, ariadne_lang::LanguageId> = BTreeMap::new();
        catalog.insert(scala_file, ariadne_lang::LanguageId::Scala);
        catalog.insert(cpp_file, ariadne_lang::LanguageId::Cpp);
        let dispatcher = LanguageRoutingConstraints {
            registry: &registry,
            language_of_file: &catalog,
        };
        let companion = ariadne_lang::SymbolRecord {
            name: "Foo".to_string(),
            ..symbol(
                "p.Foo$",
                "Foo$",
                ariadne_lang::SymbolKind::Object,
                scala_file,
            )
        };
        let records = vec![
            symbol("p.Foo", "Foo", ariadne_lang::SymbolKind::Class, scala_file),
            companion,
            symbol(
                "Widget",
                "Widget",
                ariadne_lang::SymbolKind::Class,
                cpp_file,
            ),
        ];
        let groups = dispatcher.colocated_groups(&records);
        let group_ids: Vec<Vec<String>> = groups
            .iter()
            .map(|g| g.iter().map(|s| s.0.clone()).collect())
            .collect();
        assert_eq!(
            group_ids,
            vec![vec!["p.Foo".to_string(), "p.Foo$".to_string()]],
            "the dispatcher hands the Scala subset to the Scala plugin \
             (D17 companion pairs) and the C++ subset to the C++ plugin \
             (no groups) - per-language delegation, never cross-wired"
        );
        assert!(dispatcher.frozen_symbols(&records).is_empty());
    }

    #[test]
    fn planning_gaps_flags_planning_unsupported_language_once() {
        // SANCTIONED EXPECTATION EVOLUTION (plan M23, D-M23-6/D-M23-9): the
        // pre-flip form of this lock used Scala as the planning=false
        // witness (m22 D23). The T7 flip turns Scala's planning bit on, so
        // no registered language is a gap any more — the gap→rejection
        // MECHANISM stays locked cluster-side (planner.rs planning_gap_tests
        // with a neutral `Other` gap); this CLI-side rule now locks that a
        // fully planned registry yields NO gaps, and keeps its once-per-
        // language aggregation intact for any future planning=false plugin.
        let registry = crate::registry::default_registry();
        let files = vec![
            record("a.cpp", FileKind::Source, SourceLanguage::Cpp),
            record("B.scala", FileKind::ScalaSource, SourceLanguage::Scala),
            record("C.scala", FileKind::ScalaSource, SourceLanguage::Scala),
        ];

        let gaps = planning_gaps(&files, &registry);

        assert!(
            gaps.is_empty(),
            "post-flip every registered plugin plans: {gaps:?}"
        );
        assert!(
            registry
                .plugin_for_id(ariadne_lang::LanguageId::Scala)
                .expect("scala registered")
                .capabilities()
                .planning,
            "the flip is what retired the Scala gap (D-M23-6)"
        );
    }

    #[test]
    fn planning_gaps_empty_when_every_language_plans() {
        let registry = crate::registry::default_registry();
        let files = vec![record("a.cpp", FileKind::Source, SourceLanguage::Cpp)];

        assert!(planning_gaps(&files, &registry).is_empty());
    }

    #[test]
    fn planning_gaps_skips_unclaimed_languages_without_a_plugin() {
        // `Other` (e.g. a discovered `.md`/`.txt` path) has no registered
        // plugin, so it is not a capability gap — the planner never hears of
        // it. Only a REGISTERED plugin with planning=false is one.
        let registry = crate::registry::default_registry();
        let files = vec![
            record("notes.md", FileKind::Other, SourceLanguage::Other),
            record("data.txt", FileKind::Other, SourceLanguage::Other),
            record("a.cpp", FileKind::Source, SourceLanguage::Cpp),
        ];

        assert!(
            planning_gaps(&files, &registry).is_empty(),
            "a language no plugin claims must not produce a planning rejection"
        );
    }

    // UTA-T7b row 4: `colocated_groups_with_edges` routing pins.

    fn routing_over<'a>(
        registry: &'a ariadne_lang::LanguageRegistry,
        catalog: &'a BTreeMap<FileId, ariadne_lang::LanguageId>,
    ) -> LanguageRoutingConstraints<'a> {
        LanguageRoutingConstraints {
            registry,
            language_of_file: catalog,
        }
    }

    fn group_id_strings(groups: &[Vec<ariadne_lang::SymbolId>]) -> Vec<Vec<String>> {
        groups
            .iter()
            .map(|group| group.iter().map(|id| id.0.clone()).collect())
            .collect()
    }

    fn mixed_scala_cpp_catalog(
        scala_file: FileId,
        cpp_file: FileId,
    ) -> BTreeMap<FileId, ariadne_lang::LanguageId> {
        let mut catalog: BTreeMap<FileId, ariadne_lang::LanguageId> = BTreeMap::new();
        catalog.insert(scala_file, ariadne_lang::LanguageId::Scala);
        catalog.insert(cpp_file, ariadne_lang::LanguageId::Cpp);
        catalog
    }

    #[test]
    fn pin_with_edges_empty_edge_input_equals_colocated_groups_byte_for_byte() {
        // D-M23-8 fold: an empty extends channel must take the identical
        // `colocated_groups` path (the dispatch must not even reach the
        // per-language edge machinery). Non-empty result required, or the
        // equality would be vacuous.
        let scala_file = FileId::from_relative_path("src/main/scala/p/Legacy.scala");
        let cpp_file = FileId::from_relative_path("include/all.h");
        let registry = crate::registry::default_registry();
        let catalog = mixed_scala_cpp_catalog(scala_file, cpp_file);
        let dispatcher = routing_over(&registry, &catalog);
        let records = vec![
            symbol("p.Foo", "Foo", ariadne_lang::SymbolKind::Class, scala_file),
            ariadne_lang::SymbolRecord {
                name: "Foo".to_string(),
                ..symbol("p.Foo$", "Foo$", ariadne_lang::SymbolKind::Object, scala_file)
            },
            symbol("Widget", "Widget", ariadne_lang::SymbolKind::Class, cpp_file),
        ];

        let plain = dispatcher.colocated_groups(&records);
        let via_empty_edges = dispatcher.colocated_groups_with_edges(&records, &[]);
        assert_eq!(
            group_id_strings(&via_empty_edges),
            group_id_strings(&plain),
            "empty edges must yield the byte-identical colocated_groups answer"
        );
        assert_eq!(
            group_id_strings(&plain),
            vec![vec!["p.Foo".to_string(), "p.Foo$".to_string()]],
            "the equality above must not be vacuous over an empty result"
        );
    }

    #[test]
    fn pin_with_edges_filters_edges_to_the_language_subset_they_live_in() {
        // Per-language edge filtering: an edge reaches a plugin only when
        // BOTH endpoints are records of that language's file subset. The
        // cross-language edge (Widget -> m.Shape) must reach NEITHER the
        // Scala plugin (Widget is not in the Scala subset: a misrouted
        // channel would pull `Widget` into the sealed group) nor the C++
        // one (Shape is not in the C++ subset; the C++ default ignores
        // edges anyway).
        let scala_file = FileId::from_relative_path("src/main/scala/m/Shapes.scala");
        let cpp_file = FileId::from_relative_path("include/widget.h");
        let registry = crate::registry::default_registry();
        let catalog = mixed_scala_cpp_catalog(scala_file, cpp_file);
        let dispatcher = routing_over(&registry, &catalog);

        let sealed_shape = ariadne_lang::SymbolRecord {
            modifiers: vec!["sealed".to_string()],
            ..symbol("m.Shape", "Shape", ariadne_lang::SymbolKind::Trait, scala_file)
        };
        let circle = symbol("m.Circle", "Circle", ariadne_lang::SymbolKind::Struct, scala_file);
        let square = symbol("m.Square", "Square", ariadne_lang::SymbolKind::Struct, scala_file);
        let widget = symbol("Widget", "Widget", ariadne_lang::SymbolKind::Class, cpp_file);
        let records = vec![sealed_shape, circle, square, widget];

        let intra_scala_edges = vec![
            (
                ariadne_lang::SymbolId::new("m.Circle"),
                ariadne_lang::SymbolId::new("m.Shape"),
            ),
            (
                ariadne_lang::SymbolId::new("m.Square"),
                ariadne_lang::SymbolId::new("m.Shape"),
            ),
        ];
        let cross_language_edge = (
            ariadne_lang::SymbolId::new("Widget"),
            ariadne_lang::SymbolId::new("m.Shape"),
        );

        let mut all_edges = intra_scala_edges.clone();
        all_edges.push(cross_language_edge.clone());
        let groups = dispatcher.colocated_groups_with_edges(&records, &all_edges);
        assert_eq!(
            group_id_strings(&groups),
            vec![vec![
                "m.Circle".to_string(),
                "m.Shape".to_string(),
                "m.Square".to_string(),
            ]],
            "the sealed group carries exactly the intra-Scala edge children; \
             Widget is never pulled across the language boundary"
        );

        // The cross-language edge ALONE must reach no plugin usefully: the
        // Scala subset receives zero usable edges (Shape has no present
        // child), the C++ subset none, and the sealed modifier alone is not
        // a constraint - so no groups at all.
        let cross_only =
            dispatcher.colocated_groups_with_edges(&records, std::slice::from_ref(&cross_language_edge));
        assert!(
            cross_only.is_empty(),
            "a cross-language edge alone must colocate nothing, got {:?}",
            group_id_strings(&cross_only)
        );
    }
}
