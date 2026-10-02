//! D27 payload-map plumbing (T-F). The cli payload builder owns every
//! FileId→display resolution (IV1: dto.rs stays free of registry/repo
//! concerns): paths come from the include graph, languages from discovery
//! records, unit kinds from the plugin modeling via registry lookup only
//! (IV2, R3-legal — no match on LanguageId).

use ariadne_lang::LanguageRegistry;
use ariadne_repo::{FileRecord, SourceLanguage};
use std::collections::HashMap;
use std::path::PathBuf;

/// The three id→display decoration maps the D27 dto builders consume. Keys
/// are numeric FileIds; consumers degrade on absent entries (dto.rs contract).
pub struct NodeDecorationMaps {
    pub paths: HashMap<u64, String>,
    pub languages: HashMap<u64, String>,
    pub unit_kinds: HashMap<u64, String>,
}

/// Shared path→numeric-FileId lookup used by every per-file payload builder.
pub fn build_path_to_id(include_graph: &ariadne_graph::IncludeGraph) -> HashMap<PathBuf, u64> {
    include_graph
        .file_paths
        .iter()
        .map(|(id, path)| (path.clone(), id.0))
        .collect()
}

/// Numeric FileId → repo-relative path string: the resolution the symbol-node
/// `defining_file`/`path`/`declaring_files` fields need (D27 defect fix).
pub fn paths_map(include_graph: &ariadne_graph::IncludeGraph) -> HashMap<u64, String> {
    include_graph
        .file_paths
        .iter()
        .map(|(id, path)| (id.0, path.to_string_lossy().into_owned()))
        .collect()
}

/// Numeric FileId → language display string from discovery records
/// (`SourceLanguage` Debug form, e.g. "Cpp"/"Scala" — data, no language nouns).
pub fn language_map(files: &[FileRecord]) -> HashMap<u64, String> {
    files
        .iter()
        .map(|file| (file.id.0, format!("{:?}", file.language)))
        .collect()
}

/// Numeric FileId → unit-kind display string from each language's plugin
/// `modeling().unit` (IV2: registry lookup only, R3-legal). Languages without
/// a plugin entry get NO key — the dto then omits the field (locked contract).
pub fn unit_kind_map(
    files: &[FileRecord],
    registry: &LanguageRegistry,
) -> HashMap<u64, String> {
    let mut by_id = HashMap::new();
    let mut by_language: HashMap<SourceLanguage, Option<String>> = HashMap::new();
    for file in files {
        let unit = by_language
            .entry(file.language)
            .or_insert_with_key(|language| {
                registry
                    .plugin_for_id(*language)
                    .map(|plugin| format!("{:?}", plugin.modeling().unit))
            });
        if let Some(unit) = unit {
            by_id.insert(file.id.0, unit.clone());
        }
    }
    by_id
}

/// Assemble every D27 decoration map in one call from the payload's sources.
pub fn build(
    include_graph: &ariadne_graph::IncludeGraph,
    discovery_files: &[FileRecord],
    registry: &LanguageRegistry,
) -> NodeDecorationMaps {
    NodeDecorationMaps {
        paths: paths_map(include_graph),
        languages: language_map(discovery_files),
        unit_kinds: unit_kind_map(discovery_files, registry),
    }
}

#[cfg(test)]
mod tests {
    use super::{build, build_path_to_id, language_map, paths_map, unit_kind_map};
    use ariadne_repo::{
        FileId, FileKind, FileOwnership, FileRecord, OptimizationPolicy, SourceLanguage,
    };
    use std::path::PathBuf;

    fn record(id: u64, path: &str, kind: FileKind, language: SourceLanguage) -> FileRecord {
        FileRecord {
            id: FileId(id),
            path: PathBuf::from(path),
            canonical_path: PathBuf::from(path),
            kind,
            size: 10,
            content_hash: format!("h{id}"),
            ownership: FileOwnership::Repository,
            optimization_policy: OptimizationPolicy::Optimizable,
            language,
        }
    }

    fn scala_repo_records() -> Vec<FileRecord> {
        vec![
            record(
                1,
                "src/main/scala/Main.scala",
                FileKind::ScalaSource,
                SourceLanguage::Scala,
            ),
            record(
                2,
                "src/main/scala/models/all.scala",
                FileKind::ScalaSource,
                SourceLanguage::Scala,
            ),
            record(3, "src/App.cpp", FileKind::Source, SourceLanguage::Cpp),
        ]
    }

    #[test]
    fn scala_record_yields_scala_language_on_the_file_node_map() {
        let languages = language_map(&scala_repo_records());
        assert_eq!(languages.get(&1).map(String::as_str), Some("Scala"));
        assert_eq!(languages.get(&2).map(String::as_str), Some("Scala"));
        assert_eq!(languages.get(&3).map(String::as_str), Some("Cpp"));
    }

    #[test]
    fn unit_kind_map_uses_plugin_modeling_per_language() {
        let registry = crate::registry::default_registry();
        let units = unit_kind_map(&scala_repo_records(), &registry);
        assert_eq!(units.get(&1).map(String::as_str), Some("SourceFile"));
        assert_eq!(units.get(&2).map(String::as_str), Some("SourceFile"));
        assert_eq!(units.get(&3).map(String::as_str), Some("TranslationUnit"));
    }

    #[test]
    fn language_without_plugin_gets_no_unit_kind_entry() {
        // Locked missing-entry contract (dto omits the key): Other-language
        // files have no plugin, so they carry no unit_kind at all.
        let registry = crate::registry::default_registry();
        let files = vec![record(9, "notes.md", FileKind::Other, SourceLanguage::Other)];
        assert!(unit_kind_map(&files, &registry).is_empty());
        assert_eq!(
            language_map(&files).get(&9).map(String::as_str),
            Some("Other"),
            "language stays honest even when no plugin models it"
        );
    }

    #[test]
    fn paths_map_resolves_graph_file_ids_to_repo_relative_paths() {
        let mut graph = ariadne_graph::IncludeGraph::new();
        graph
            .file_paths
            .insert(FileId(5), PathBuf::from("src/main/scala/Main.scala"));
        let paths = paths_map(&graph);
        assert_eq!(
            paths.get(&5).map(String::as_str),
            Some("src/main/scala/Main.scala")
        );
        assert!(paths.get(&6).is_none(), "unknown ids stay absent (dto degrades)");

        let path_to_id = build_path_to_id(&graph);
        assert_eq!(
            path_to_id.get(&PathBuf::from("src/main/scala/Main.scala")),
            Some(&5)
        );
    }

    #[test]
    fn build_assembles_all_three_decoration_maps() {
        let mut graph = ariadne_graph::IncludeGraph::new();
        for record in scala_repo_records() {
            graph.file_paths.insert(record.id, record.path.clone());
        }
        let registry = crate::registry::default_registry();
        let maps = build(&graph, &scala_repo_records(), &registry);
        assert_eq!(maps.paths.len(), 3);
        assert_eq!(maps.languages.len(), 3);
        assert_eq!(maps.unit_kinds.len(), 3);
        assert_eq!(maps.paths.get(&1).map(String::as_str), Some("src/main/scala/Main.scala"));
        assert_eq!(maps.languages.get(&1).map(String::as_str), Some("Scala"));
        assert_eq!(maps.unit_kinds.get(&1).map(String::as_str), Some("SourceFile"));
    }
}
