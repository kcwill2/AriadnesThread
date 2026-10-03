use anyhow::Result;
use ariadne_pipeline::Pipeline;
use ariadne_repo::Repository;
use ariadne_report::ArtifactManager;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

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

/// Per-language planning hooks (plan M23, D-M23-7 i). The planner takes
/// exactly ONE aggregation filter and ONE target namer, so language dispatch
/// lives INSIDE the closures, keyed by the language the REGISTRY resolved for
/// the path the planner asks about.
///
/// M24 Part A, D-A4: this used to be `Some(Scala) => scala, _ => cpp`, so a
/// third language silently received C++ aggregation and naming rules — a wrong
/// answer rather than a compile error. C++ paths keep their hooks
/// byte-identically; a language with no rule here is now reported honestly
/// (not an aggregation unit, and an unchanged target) instead of being claimed
/// by C++.
struct PlanningHooks {
    aggregation: Box<dyn Fn(&Path) -> bool + Send + Sync>,
    target: Box<dyn Fn(&Path, &str) -> PathBuf + Send + Sync>,
}

/// `FileRecord.path` → language, for the planning dispatch closures.
///
/// M24 Part A, D-A5: resolved through the registry's `detect_language`, not
/// by copying `FileRecord.language`. A path no registered plugin claims is
/// simply absent, which is what lets the dispatch below tell "unknown" from
/// "C++".
fn language_of_path(
    registry: &ariadne_lang::LanguageRegistry,
    files: &[ariadne_repo::FileRecord],
) -> BTreeMap<PathBuf, ariadne_lang::LanguageId> {
    files
        .iter()
        .filter_map(|record| {
            registry
                .detect_language(&record.path)
                .map(|language| (record.path.clone(), language))
        })
        .collect()
}

fn planning_hooks(
    repo_root: PathBuf,
    languages: BTreeMap<PathBuf, ariadne_lang::LanguageId>,
) -> PlanningHooks {
    let agg_languages = languages.clone();
    let aggregation = Box::new(move |path: &Path| -> bool {
        match agg_languages.get(path) {
            Some(language) if *language == ariadne_lang::LanguageId::Scala => {
                ariadne_lang_scala::scala_aggregation_filter(&repo_root)(path)
            }
            Some(language) if *language == ariadne_lang::LanguageId::Cpp => {
                ariadne_lang_cpp::cpp_aggregation_filter(path)
            }
            // D-A4: no C++ fallback survives. A path whose language is neither
            // C++ nor Scala -- which today means a third language, and also a
            // path no registered plugin claims at all -- has no aggregation rule
            // in this closure. Answering "not an aggregation unit" is the
            // honest result; the `_ => Cpp` arm this replaces claimed it by
            // default and handed the file C++ aggregation semantics.
            Some(_) | None => false,
        }
    });
    let target = Box::new(move |candidate: &Path, symbol_name: &str| -> PathBuf {
        match languages.get(candidate) {
            Some(language) if *language == ariadne_lang::LanguageId::Scala => {
                ariadne_lang_scala::scala_target_path_for(candidate, symbol_name)
            }
            Some(language) if *language == ariadne_lang::LanguageId::Cpp => {
                ariadne_lang_cpp::cpp_target_path_for(candidate, symbol_name)
            }
            // D-A4: same discipline for the namer. The hook signature returns a
            // `PathBuf` and cannot report an error, so for a language with no
            // rule here the candidate is returned UNCHANGED -- no C++ naming
            // rule is applied to a file that is not C++. The aggregation
            // closure above already keeps such paths out of the candidate set;
            // this is the defensive half.
            Some(_) | None => candidate.to_path_buf(),
        }
    });
    PlanningHooks {
        aggregation,
        target,
    }
}

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

    // D-M23-7 i: per-language planning dispatch replaces the pre-M23 C++
    // hardcode; C++ files keep byte-identical hooks (see PlanningHooks).
    let hooks = planning_hooks(
        pipeline.repo.root.clone(),
        language_of_path(&registry, &pipeline.discovery.files),
    );
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
    use super::{language_of_path, planning_gaps, LanguageRoutingConstraints};
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
    fn a_third_language_no_longer_silently_receives_the_cpp_planning_hooks() {
        // M24 Part A, D-A4 -- THE POINT OF THIS TASK. Before it, both planning
        // hooks were `Some(Scala) => scala, _ => cpp_...`, so a file in ANY
        // other language silently received C++ aggregation and C++ naming
        // rules: a wrong answer rather than a compile error.
        //
        // The test uses a deliberately fake third language (no third-language
        // plugin ships in Part A; that is Part C) so the assertion is about the
        // DISPATCH, not about a real analyzer.
        let third_language = ariadne_lang::LanguageId::new("Fortran");
        let claimed = PathBuf::from("src/solver.f90");
        let mut languages: BTreeMap<PathBuf, ariadne_lang::LanguageId> = BTreeMap::new();
        languages.insert(claimed.clone(), third_language);

        let hooks = super::planning_hooks(PathBuf::from("."), languages);

        assert!(
            !(hooks.aggregation)(&claimed),
            "a language with no planning rule here is NOT an aggregation unit; \
             the C++ filter claimed it by default before D-A4"
        );
        assert_eq!(
            (hooks.target)(&claimed, "Solver"),
            claimed,
            "no C++ naming rule may be applied to a file that is not C++"
        );
        assert_ne!(
            (hooks.target)(&claimed, "Solver"),
            ariadne_lang_cpp::cpp_target_path_for(&claimed, "Solver"),
            "explicitly NOT the C++ target path - this is the misroute that died"
        );
    }

    #[test]
    fn planning_hooks_give_an_unclaimed_path_no_cpp_hooks() {
        // The other half of D-A4: a path with NO registered plugin used to fall
        // through to C++ too.
        let hooks = super::planning_hooks(PathBuf::from("."), BTreeMap::new());
        let orphan = PathBuf::from("src/unclaimed.rs");
        assert!(!(hooks.aggregation)(&orphan));
        assert_eq!((hooks.target)(&orphan, "Thing"), orphan);
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
    fn language_of_path_resolves_through_the_registry_not_the_record_label() {
        // SANCTIONED EXPECTATION CHANGE (M24 Part A, D-A5): the map is now
        // built from `registry.detect_language`, so a record's `language`
        // LABEL no longer determines dispatch. The `B.scala` record below
        // deliberately carries the WRONG label (`Other`) - the shape every
        // third language takes today, because no FileKind variant names it -
        // and still resolves to the plugin that claims it.
        let registry = crate::registry::default_registry();
        let files = vec![
            record("a.cpp", FileKind::Source, SourceLanguage::Cpp),
            record("B.scala", FileKind::ScalaSource, SourceLanguage::Other),
        ];
        let map = language_of_path(&registry, &files);
        assert_eq!(
            map.get(&PathBuf::from("B.scala")),
            Some(&ariadne_lang::LanguageId::Scala),
            "the claiming plugin wins over the record's Other label"
        );
        assert_eq!(
            map.get(&PathBuf::from("a.cpp")),
            Some(&ariadne_lang::LanguageId::Cpp)
        );
        assert_eq!(map.get(&PathBuf::from("ghost.h")), None);
    }

    #[test]
    fn language_of_path_omits_a_path_no_plugin_claims() {
        let registry = crate::registry::default_registry();
        let files = vec![record("notes.md", FileKind::Other, SourceLanguage::Other)];
        assert!(
            language_of_path(&registry, &files).is_empty(),
            "an unclaimed path is absent, which is what lets the hooks tell \
             'unknown' from 'C++' instead of defaulting"
        );
    }

    #[test]
    fn planning_hooks_route_scala_by_language_and_keep_cpp_hook_bytes() {
        let scala_rel = PathBuf::from("src/main/scala/com/example/models/all.scala");
        let cpp_rel = PathBuf::from("include/all.h");
        let mut languages: BTreeMap<PathBuf, ariadne_lang::LanguageId> = BTreeMap::new();
        languages.insert(scala_rel.clone(), ariadne_lang::LanguageId::Scala);
        languages.insert(cpp_rel.clone(), ariadne_lang::LanguageId::Cpp);
        let fixture_root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/scala-basic");
        let hooks = super::planning_hooks(fixture_root, languages);
        assert!(
            (hooks.aggregation)(&scala_rel),
            "the real fixture all.scala is a Scala aggregation unit through \
             the CLI dispatch (scala_aggregation_filter, not test injection)"
        );
        assert!(
            !(hooks.aggregation)(&PathBuf::from(
                "src/main/scala/com/example/models/User.scala"
            )),
            "a Scala file without a package-object clause is NOT an \
             aggregation unit - the C++ extension filter never claims it"
        );
        assert_eq!(
            (hooks.target)(&scala_rel, "com.example.models.User"),
            ariadne_lang_scala::scala_target_path_for(&scala_rel, "com.example.models.User"),
            "Scala candidates get the Scala target namer verbatim"
        );
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
}
