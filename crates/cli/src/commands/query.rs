use crate::commands::index::get_or_build_reduced_model;
use anyhow::Result;
use ariadne_report::ArtifactManager;
use std::path::PathBuf;

pub fn handle_query_symbol(repo_path: &PathBuf, name: &str) -> Result<()> {
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let sym = match model.find_symbol(name) {
        Some(s) => s,
        None => {
            println!("Symbol {} not found.", name);
            return Ok(());
        }
    };

    let file_map = {
        let mut map = std::collections::HashMap::new();
        if let Ok(manager) = ArtifactManager::init(repo_path) {
            if let Ok(records) = manager.load_repository_artifacts() {
                for r in records {
                    map.insert(r.id, r.path.to_string_lossy().into_owned());
                }
            }
        }
        map
    };

    println!("{}", sym.name);
    println!("  Definition:");
    if let Some(def_file) = &sym.defining_file {
        if let Some(path) = file_map.get(def_file) {
            println!("    {}", path);
        } else {
            println!("    {}", def_file);
        }
    } else {
        println!("    (none)");
    }

    println!();
    println!("  Semantic dependencies:");
    let deps = model.get_dependencies(&sym.id);
    if deps.is_empty() {
        println!("    (none)");
    } else {
        for dep in deps {
            let dep_sym = model.symbols.get(&dep.dst_symbol);
            if let Some(ds) = dep_sym {
                println!("    {}", ds.name);
            } else {
                println!("    {}", dep.dst_symbol.0);
            }
        }
    }

    println!();
    println!("  Used by:");
    let users = model.get_users(&sym.id);
    if users.is_empty() {
        println!("    (none)");
    } else {
        for u in users {
            if let Some(path) = file_map.get(&u) {
                println!("    {}", path);
            } else {
                println!("    {}", u);
            }
        }
    }

    Ok(())
}

pub fn handle_query_deps(repo_path: &PathBuf, name: &str) -> Result<()> {
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let sym = match model.find_symbol(name) {
        Some(s) => s,
        None => {
            println!("Symbol {} not found.", name);
            return Ok(());
        }
    };

    println!("{} dependencies:", sym.name);
    let deps = model.get_dependencies(&sym.id);
    if deps.is_empty() {
        println!("  (none)");
    } else {
        for dep in deps {
            let dep_sym = model.symbols.get(&dep.dst_symbol);
            if let Some(ds) = dep_sym {
                println!("  {}", ds.name);
            } else {
                println!("  {}", dep.dst_symbol.0);
            }
        }
    }

    Ok(())
}

pub fn handle_query_users(repo_path: &PathBuf, name: &str) -> Result<()> {
    let (_, model) = get_or_build_reduced_model(repo_path)?;

    let sym = match model.find_symbol(name) {
        Some(s) => s,
        None => {
            println!("Symbol {} not found.", name);
            return Ok(());
        }
    };

    let file_map = {
        let mut map = std::collections::HashMap::new();
        if let Ok(manager) = ArtifactManager::init(repo_path) {
            if let Ok(records) = manager.load_repository_artifacts() {
                for r in records {
                    map.insert(r.id, r.path.to_string_lossy().into_owned());
                }
            }
        }
        map
    };

    println!("{} used by:", sym.name);
    let users = model.get_users(&sym.id);
    if users.is_empty() {
        println!("  (none)");
    } else {
        for u in users {
            if let Some(path) = file_map.get(&u) {
                println!("  {}", path);
            } else {
                println!("  {}", u);
            }
        }
    }

    Ok(())
}
