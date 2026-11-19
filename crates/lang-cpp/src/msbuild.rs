//! MSBuild include-dir harvesting for Visual Studio repos.
//!
//! A repo built from `.vcxproj` files (and the `.props` sheets they import)
//! keeps its header search path in `AdditionalIncludeDirectories` and the
//! VC++ Directories properties, not in any compile command AT can read.
//! Without them, includes such as `"Common/Foo.h"` stay unresolved and the
//! headers behind them are never walked. This module recovers those dirs
//! with a deliberately small evaluator:
//!
//! - Project files are found next to (or above) the discovered sources, so
//!   `.atignore`d trees are never visited.
//! - Properties are evaluated in document order through `<Import>` chains
//!   and an auto-imported `Directory.Build.props`, with the reserved macros
//!   AT can derive (`$(ProjectDir)`, `$(MSBuildThisFileDirectory)`,
//!   `$(SolutionDir)`, ...) and environment variables as fallback.
//! - `Condition` attributes are ignored, so every configuration's dirs are
//!   unioned. Entries that still hold an unknown macro, item metadata
//!   (`%(...)`) or point at a missing directory are dropped.

mod xml_scan;

use ariadne_lang::normalize_relative_path;
use ariadne_repo::FileRecord;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use xml_scan::{scan, XmlEvent};

/// Imports nest a handful of levels in practice; this only bounds
/// pathological chains (cycles are already cut by the visited set).
const MAX_IMPORT_DEPTH: usize = 32;

/// Properties whose final value is a `;`-separated header search path.
const INCLUDE_PATH_PROPERTIES: [&str; 2] = ["includepath", "externalincludepath"];

/// Existing header search dirs declared by the repo's `.vcxproj` files, in
/// first-seen order without duplicates. Empty when the repo has none.
pub(crate) fn msbuild_include_dirs(repo_root: &Path, files: &[FileRecord]) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut dirs = Vec::new();
    for project_path in find_project_files(repo_root, files) {
        for dir in ProjectEvaluation::run(repo_root, &project_path) {
            if dir.is_dir() && seen.insert(dir.clone()) {
                dirs.push(dir);
            }
        }
    }
    dirs
}

/// `.vcxproj` files in any directory that holds, or is an ancestor of, a
/// discovered file; sorted for deterministic evaluation order.
fn find_project_files(repo_root: &Path, files: &[FileRecord]) -> Vec<PathBuf> {
    let mut candidate_dirs = BTreeSet::new();
    for record in files {
        let mut current = record.path.parent();
        while let Some(dir) = current {
            if !candidate_dirs.insert(dir.to_path_buf()) {
                break; // its ancestors were inserted with it
            }
            current = dir.parent();
        }
    }
    let mut projects: Vec<PathBuf> = candidate_dirs
        .iter()
        .flat_map(|dir| files_with_extension(&repo_root.join(dir), "vcxproj"))
        .collect();
    projects.sort();
    projects
}

fn files_with_extension(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
                && path.is_file()
        })
        .collect()
}

/// `dir` and its ancestors up to and including `repo_root`, nearest first.
fn dirs_up_to_root<'a>(dir: &'a Path, repo_root: &'a Path) -> impl Iterator<Item = &'a Path> {
    dir.ancestors()
        .take_while(move |ancestor| ancestor.starts_with(repo_root))
}

/// Directory spelling with a trailing `/`, as MSBuild's `*Dir` macros carry.
fn dir_with_separator(dir: &Path) -> String {
    let spelled = dir.to_string_lossy().replace('\\', "/");
    if spelled.ends_with('/') {
        spelled
    } else {
        format!("{spelled}/")
    }
}

/// Project text with UTF-16 (BOM-marked) and UTF-8-with-BOM handled; VS
/// writes both.
fn read_project_text(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let utf16 = |to_unit: fn([u8; 2]) -> u16| {
        let units: Vec<u16> = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| to_unit(*pair))
            .collect();
        String::from_utf16_lossy(&units)
    };
    let text = match bytes.get(..2) {
        Some([0xFF, 0xFE]) => utf16(u16::from_le_bytes),
        Some([0xFE, 0xFF]) => utf16(u16::from_be_bytes),
        _ => String::from_utf8_lossy(&bytes).into_owned(),
    };
    Some(text.trim_start_matches('\u{feff}').to_string())
}

/// Evaluation state for one `.vcxproj` and everything it imports.
struct ProjectEvaluation<'a> {
    repo_root: &'a Path,
    project_dir: PathBuf,
    /// Property values keyed by lower-cased name (MSBuild names are
    /// case-insensitive).
    properties: HashMap<String, String>,
    visited_files: HashSet<PathBuf>,
    /// Expanded, still `;`-joined include-dir lists in discovery order.
    include_lists: Vec<String>,
}

impl<'a> ProjectEvaluation<'a> {
    fn run(repo_root: &'a Path, project_path: &Path) -> Vec<PathBuf> {
        let project_dir = project_path.parent().unwrap_or(repo_root).to_path_buf();
        let mut evaluation = Self {
            repo_root,
            project_dir,
            properties: HashMap::new(),
            visited_files: HashSet::new(),
            include_lists: Vec::new(),
        };
        evaluation.seed_reserved_properties(project_path);
        if let Some(build_props) = evaluation.directory_build_props() {
            evaluation.evaluate_file(&build_props, 0);
        }
        evaluation.evaluate_file(project_path, 0);
        for name in INCLUDE_PATH_PROPERTIES {
            if let Some(value) = evaluation.properties.get(name).cloned() {
                evaluation.include_lists.push(value);
            }
        }
        evaluation.resolve_include_lists()
    }

    fn seed_reserved_properties(&mut self, project_path: &Path) {
        let project_dir = dir_with_separator(&self.project_dir);
        let project_name = project_path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let solution_dir = dir_with_separator(&self.solution_dir());
        let reserved = [
            ("projectdir", project_dir.clone()),
            ("msbuildprojectdirectory", project_dir.trim_end_matches('/').to_string()),
            ("projectname", project_name.clone()),
            ("msbuildprojectname", project_name),
            ("solutiondir", solution_dir),
        ];
        for (name, value) in reserved {
            self.properties.insert(name.to_string(), value);
        }
    }

    /// Nearest ancestor holding a `.sln` (VS sets `$(SolutionDir)` from the
    /// solution being built); the repo root when there is none.
    fn solution_dir(&self) -> PathBuf {
        dirs_up_to_root(&self.project_dir, self.repo_root)
            .find(|dir| !files_with_extension(dir, "sln").is_empty())
            .unwrap_or(self.repo_root)
            .to_path_buf()
    }

    /// MSBuild auto-imports the nearest `Directory.Build.props` above the
    /// project; AT stops looking at the repo root.
    fn directory_build_props(&self) -> Option<PathBuf> {
        dirs_up_to_root(&self.project_dir, self.repo_root)
            .map(|dir| dir.join("Directory.Build.props"))
            .find(|candidate| candidate.is_file())
    }

    fn evaluate_file(&mut self, path: &Path, depth: usize) {
        let key = normalize_relative_path(path);
        if depth > MAX_IMPORT_DEPTH || !self.visited_files.insert(key) {
            return;
        }
        let Some(document) = read_project_text(path) else {
            return;
        };
        let file_dir = path.parent().unwrap_or(self.repo_root).to_path_buf();
        let this_dir_key = "msbuildthisfiledirectory".to_string();
        let outer_this_dir = self
            .properties
            .insert(this_dir_key.clone(), dir_with_separator(&file_dir));
        self.evaluate_events(&scan(&document), &file_dir, depth);
        match outer_this_dir {
            Some(value) => self.properties.insert(this_dir_key, value),
            None => self.properties.remove(&this_dir_key),
        };
    }

    /// Walks the events once, tracking open elements so a text-only element
    /// (a "leaf") can be told apart from a container.
    fn evaluate_events(&mut self, events: &[XmlEvent], file_dir: &Path, depth: usize) {
        // (element name, has child elements)
        let mut open_elements: Vec<(&str, bool)> = Vec::new();
        let mut pending_text = "";
        for event in events {
            match event {
                XmlEvent::Open {
                    name,
                    attributes,
                    self_closing,
                } => {
                    if let Some(parent) = open_elements.last_mut() {
                        parent.1 = true;
                    }
                    if name.eq_ignore_ascii_case("Import") {
                        self.import(attributes, file_dir, depth);
                    }
                    if !self_closing {
                        open_elements.push((name, false));
                    }
                    pending_text = "";
                }
                XmlEvent::Text(text) => pending_text = text,
                XmlEvent::Close { .. } => {
                    if let Some((name, has_children)) = open_elements.pop() {
                        if !has_children {
                            let parent = open_elements.last().map(|(parent, _)| *parent);
                            self.on_leaf(parent, name, pending_text);
                        }
                    }
                    pending_text = "";
                }
            }
        }
    }

    fn on_leaf(&mut self, parent: Option<&str>, name: &str, text: &str) {
        if name.eq_ignore_ascii_case("AdditionalIncludeDirectories") {
            let expanded = self.expand(text);
            self.include_lists.push(expanded);
        } else if parent.is_some_and(|parent| parent.eq_ignore_ascii_case("PropertyGroup")) {
            let expanded = self.expand(text);
            self.properties.insert(name.to_ascii_lowercase(), expanded);
        }
    }

    fn import(&mut self, attributes: &[(String, String)], file_dir: &Path, depth: usize) {
        let Some((_, project)) = attributes
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case("Project"))
        else {
            return;
        };
        let expanded = self.expand(project);
        if has_unresolved_reference(&expanded) || expanded.contains('*') {
            return;
        }
        let target = normalize_relative_path(&file_dir.join(expanded.trim().replace('\\', "/")));
        if target.is_file() {
            self.evaluate_file(&target, depth + 1);
        }
    }

    /// Substitutes `$(Name)` from the property table, then the environment.
    /// Unknown references (and property functions) are left in place so the
    /// entry carrying them is dropped later.
    fn expand(&self, text: &str) -> String {
        let mut expanded = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(start) = rest.find("$(") {
            expanded.push_str(&rest[..start]);
            let after_open = &rest[start + 2..];
            let Some(end) = after_open.find(')') else {
                expanded.push_str(&rest[start..]);
                return expanded;
            };
            let name = &after_open[..end];
            match self.lookup(name) {
                Some(value) => expanded.push_str(&value),
                None => expanded.push_str(&rest[start..start + 2 + end + 1]),
            }
            rest = &after_open[end + 1..];
        }
        expanded.push_str(rest);
        expanded
    }

    fn lookup(&self, name: &str) -> Option<String> {
        self.properties
            .get(&name.to_ascii_lowercase())
            .cloned()
            .or_else(|| std::env::var(name).ok())
    }

    /// Splits the collected lists into paths, resolving relative entries
    /// against the project directory (cl.exe's working directory).
    fn resolve_include_lists(&self) -> Vec<PathBuf> {
        self.include_lists
            .iter()
            .flat_map(|list| list.split(';'))
            .map(|entry| entry.trim().trim_matches('"').trim())
            .filter(|entry| !entry.is_empty() && !has_unresolved_reference(entry))
            .map(|entry| {
                let path = PathBuf::from(entry.replace('\\', "/"));
                normalize_relative_path(&self.project_dir.join(path))
            })
            .collect()
    }
}

fn has_unresolved_reference(text: &str) -> bool {
    text.contains("$(") || text.contains("%(") || text.contains("@(")
}

#[cfg(test)]
mod tests;
