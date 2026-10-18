use crate::commands::index::get_or_build_reduced_model;
use anyhow::Result;
use ariadne_graph::{ReducedSemanticModel, ReducedSymbolRecord};
use ariadne_repo::{FileId};
use ariadne_report::ArtifactManager;
use std::collections::HashMap;
use std::path::PathBuf;

/// One line of the `Symbol {name} not found.` report shared by all three
/// query commands (UTA-T7b row 7 render seam): the miss is a REPORT, not an
/// error - every command exits `Ok(())` over it.
fn render_not_found(name: &str) -> String {
    format!("Symbol {} not found.", name)
}

/// `FileId -> repo artifact path` map for the definition/user rows. An
/// unreadable or missing artifact store degrades to an empty map (the raw
/// FileId Display is printed instead) - never an error.
fn file_path_map(repo_path: &PathBuf) -> HashMap<FileId, String> {
    let mut map = HashMap::new();
    if let Ok(manager) = ArtifactManager::init(repo_path) {
        if let Ok(records) = manager.load_repository_artifacts() {
            for r in records {
                map.insert(r.id, r.path.to_string_lossy().into_owned());
            }
        }
    }
    map
}

/// Full `query symbol` block: name, definition (artifact path, raw FileId,
/// or `(none)`), then dependency and user sections, `(none)` when empty.
fn render_symbol_report(
    model: &ReducedSemanticModel,
    sym: &ReducedSymbolRecord,
    file_map: &HashMap<FileId, String>,
) -> Vec<String> {
    let mut lines = vec![sym.name.clone(), "  Definition:".to_string()];
    if let Some(def_file) = &sym.defining_file {
        if let Some(path) = file_map.get(def_file) {
            lines.push(format!("    {}", path));
        } else {
            lines.push(format!("    {}", def_file));
        }
    } else {
        lines.push("    (none)".to_string());
    }
    lines.push(String::new());
    lines.push("  Semantic dependencies:".to_string());
    let deps = model.get_dependencies(&sym.id);
    if deps.is_empty() {
        lines.push("    (none)".to_string());
    } else {
        for dep in deps {
            match model.symbols.get(&dep.dst_symbol) {
                Some(ds) => lines.push(format!("    {}", ds.name)),
                None => lines.push(format!("    {}", dep.dst_symbol.0)),
            }
        }
    }
    lines.push(String::new());
    lines.push("  Used by:".to_string());
    let users = model.get_users(&sym.id);
    if users.is_empty() {
        lines.push("    (none)".to_string());
    } else {
        for u in users {
            match file_map.get(&u) {
                Some(path) => lines.push(format!("    {}", path)),
                None => lines.push(format!("    {}", u)),
            }
        }
    }
    lines
}

/// `query deps` block: `{name} dependencies:` then 2-space dependency rows.
fn render_dependencies_report(model: &ReducedSemanticModel, sym: &ReducedSymbolRecord) -> Vec<String> {
    let mut lines = vec![format!("{} dependencies:", sym.name)];
    let deps = model.get_dependencies(&sym.id);
    if deps.is_empty() {
        lines.push("  (none)".to_string());
    } else {
        for dep in deps {
            match model.symbols.get(&dep.dst_symbol) {
                Some(ds) => lines.push(format!("  {}", ds.name)),
                None => lines.push(format!("  {}", dep.dst_symbol.0)),
            }
        }
    }
    lines
}

/// `query users` block: `{name} used by:` then artifact-path or raw-FileId
/// rows, `(none)` when nobody uses the symbol.
fn render_users_report(
    model: &ReducedSemanticModel,
    sym: &ReducedSymbolRecord,
    file_map: &HashMap<FileId, String>,
) -> Vec<String> {
    let mut lines = vec![format!("{} used by:", sym.name)];
    let users = model.get_users(&sym.id);
    if users.is_empty() {
        lines.push("  (none)".to_string());
    } else {
        for u in users {
            match file_map.get(&u) {
                Some(path) => lines.push(format!("  {}", path)),
                None => lines.push(format!("  {}", u)),
            }
        }
    }
    lines
}

pub fn query_symbol_by_name(repo_path: &PathBuf, name: &str) -> Result<()> {
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let sym = match model.find_symbol(name) {
        Some(s) => s,
        None => {
            println!("{}", render_not_found(name));
            return Ok(());
        }
    };

    let file_map = file_path_map(repo_path);
    for line in render_symbol_report(&model, sym, &file_map) {
        println!("{}", line);
    }

    Ok(())
}

pub fn query_symbol_dependencies(repo_path: &PathBuf, name: &str) -> Result<()> {
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let sym = match model.find_symbol(name) {
        Some(s) => s,
        None => {
            println!("{}", render_not_found(name));
            return Ok(());
        }
    };

    for line in render_dependencies_report(&model, sym) {
        println!("{}", line);
    }

    Ok(())
}

pub fn query_symbol_users(repo_path: &PathBuf, name: &str) -> Result<()> {
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let sym = match model.find_symbol(name) {
        Some(s) => s,
        None => {
            println!("{}", render_not_found(name));
            return Ok(());
        }
    };

    let file_map = file_path_map(repo_path);
    for line in render_users_report(&model, sym, &file_map) {
        println!("{}", line);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        render_dependencies_report, render_not_found, render_symbol_report, render_users_report,
    };
    use ariadne_graph::{ReducedSemanticEdge, ReducedSemanticModel, ReducedSymbolRecord};
    use ariadne_index::types::{DependencyKind, SymbolId, SymbolKind};
    use ariadne_repo::FileId;
    use std::collections::{BTreeMap, HashMap};

    fn record(id: &str, name: &str, defining_file: Option<FileId>) -> ReducedSymbolRecord {
        ReducedSymbolRecord {
            id: SymbolId::new(id),
            name: name.to_string(),
            kind: SymbolKind::Class,
            defining_file,
            declarations: vec![],
            definition: None,
            declaring_files: vec![],
        }
    }

    fn model_with(symbols: Vec<ReducedSymbolRecord>, edges: Vec<ReducedSemanticEdge>) -> ReducedSemanticModel {
        ReducedSemanticModel {
            symbols: symbols
                .into_iter()
                .map(|symbol| (symbol.id.clone(), symbol))
                .collect::<BTreeMap<_, _>>(),
            edges,
            ..ReducedSemanticModel::default()
        }
    }

    fn edge(from: &str, to: &str) -> ReducedSemanticEdge {
        ReducedSemanticEdge {
            src_symbol: SymbolId::new(from),
            dst_symbol: SymbolId::new(to),
            kind: DependencyKind::Complete,
        }
    }

    fn mapped(filed: FileId, path: &str) -> HashMap<FileId, String> {
        let mut map = HashMap::new();
        map.insert(filed, path.to_string());
        map
    }

    #[test]
    fn pin_not_found_report_is_a_plain_line_not_an_error() {
        // The miss is printed as a sentence and the command exits Ok - this
        // line (pinned verbatim here) is what the three query commands print
        // before returning Ok(()).
        assert_eq!(render_not_found("Ghost"), "Symbol Ghost not found.");
        assert!(
            !render_not_found("Ghost").contains('\n'),
            "single line, no list scaffolding on a miss"
        );
    }

    #[test]
    fn pin_symbol_report_scaffolds_none_placeholders_in_fixed_order() {
        let sym = record("Ghost", "Ghost", None);
        let model = ReducedSemanticModel::default();
        assert_eq!(
            render_symbol_report(&model, &sym, &HashMap::new()),
            vec![
                "Ghost".to_string(),
                "  Definition:".to_string(),
                "    (none)".to_string(),
                String::new(),
                "  Semantic dependencies:".to_string(),
                "    (none)".to_string(),
                String::new(),
                "  Used by:".to_string(),
                "    (none)".to_string(),
            ],
            "empty model: definition, deps and users each print (none) under \
             their fixed section order (two blank separator lines)"
        );
    }

    #[test]
    fn pin_symbol_report_definition_and_user_rows_fall_back_to_raw_file_id() {
        // Mapped artifact paths print first; an UNMAPPED FileId prints its
        // 16-hex Display (4-space indent in the full report), never an
        // error or a silent skip.
        let def_file = FileId::from_relative_path("src/ghost.cpp");
        let user_file = FileId::from_relative_path("src/consumer.cpp");
        let sym = record("Ghost", "Ghost", Some(def_file));
        let mut model = model_with(vec![sym.clone()], vec![]);
        model
            .tu_usage
            .insert(sym.id.clone(), vec![user_file]);

        let lines = render_symbol_report(&model, &sym, &HashMap::new());
        assert_eq!(lines[2], format!("    {:016x}", def_file.0));
        assert_eq!(lines[8], format!("    {:016x}", user_file.0));

        let mapped_lines = render_symbol_report(
            &model,
            &sym,
            &mapped(def_file, "src/ghost.cpp"),
        );
        assert_eq!(mapped_lines[2], "    src/ghost.cpp".to_string());
    }

    #[test]
    fn pin_dependencies_report_resolves_named_targets_and_prints_dangling_raw_ids() {
        // A dependency edge whose target symbol IS in the model prints the
        // target's NAME; a dangling edge prints the raw dst id string (not
        // hex, not a skip); an empty dep set prints "  (none)".
        let sym = record("Ghost", "Ghost", None);
        let known = record("Known", "Known", None);
        let model = model_with(
            vec![sym.clone(), known],
            vec![edge("Ghost", "Known"), edge("Ghost", "Missing")],
        );
        assert_eq!(
            render_dependencies_report(&model, &sym),
            vec![
                "Ghost dependencies:".to_string(),
                "  Known".to_string(),
                "  Missing".to_string(),
            ],
            "named target by name, dangling target by raw id, edges in model order"
        );

        let bare = model_with(vec![sym.clone()], vec![]);
        assert_eq!(
            render_dependencies_report(&bare, &sym),
            vec![
                "Ghost dependencies:".to_string(),
                "  (none)".to_string(),
            ]
        );
    }

    #[test]
    fn pin_users_report_maps_paths_prints_raw_ids_and_none_when_empty() {
        let sym = record("Ghost", "Ghost", None);
        let known_user = FileId::from_relative_path("src/known_user.cpp");
        let unmapped_user = FileId::from_relative_path("src/unmapped_user.cpp");
        let mut model = model_with(vec![sym.clone()], vec![]);
        model
            .tu_usage
            .insert(sym.id.clone(), vec![known_user, unmapped_user]);

        let lines = render_users_report(&model, &sym, &mapped(known_user, "src/known_user.cpp"));
        assert_eq!(
            lines,
            vec![
                "Ghost used by:".to_string(),
                "  src/known_user.cpp".to_string(),
                format!("  {:016x}", unmapped_user.0),
            ],
            "mapped users print their artifact path, unmapped print the raw \
             16-hex FileId, both at 2-space indent"
        );

        let empty = model_with(vec![sym.clone()], vec![]);
        assert_eq!(
            render_users_report(&empty, &sym, &HashMap::new()),
            vec![
                "Ghost used by:".to_string(),
                "  (none)".to_string(),
            ]
        );
    }
}
