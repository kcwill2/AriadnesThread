//! What the MSBuild evaluator could NOT use, rendered as discovery notes.
//!
//! Dropping an include entry is silent by design (a guess that points at
//! the wrong directory is worse than no guess), but on a large Visual
//! Studio repo the drops are exactly what a user needs to see: which
//! macros AT never learned a value for, which named directories do not
//! exist, and which `<Import>`s it could not follow.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Example rows printed per finding kind; the counts above them are total.
const EXAMPLE_ROWS: usize = 15;

/// One macro that stayed unexpanded, with a representative use.
#[derive(Debug, Clone)]
struct MacroMiss {
    dropped_entries: usize,
    example_entry: String,
    example_file: PathBuf,
}

/// Drops collected across every evaluated project.
#[derive(Debug, Default, Clone)]
pub(super) struct Findings {
    unresolved_macros: BTreeMap<String, MacroMiss>,
    /// Missing directory -> the file whose entry named it.
    missing_dirs: BTreeMap<PathBuf, PathBuf>,
    /// Import spelling (after expansion) -> the file that imported it.
    unresolved_imports: BTreeMap<String, PathBuf>,
}

impl Findings {
    pub(super) fn record_unresolved_entry(&mut self, entry: &str, defining_file: &Path) {
        let macro_name = first_macro_reference(entry);
        let miss = self
            .unresolved_macros
            .entry(macro_name)
            .or_insert_with(|| MacroMiss {
                dropped_entries: 0,
                example_entry: entry.to_string(),
                example_file: defining_file.to_path_buf(),
            });
        miss.dropped_entries += 1;
    }

    pub(super) fn record_missing_dir(&mut self, dir: &Path, defining_file: &Path) {
        self.missing_dirs
            .entry(dir.to_path_buf())
            .or_insert_with(|| defining_file.to_path_buf());
    }

    pub(super) fn record_unresolved_import(&mut self, spelling: &str, importing_file: &Path) {
        self.unresolved_imports
            .entry(spelling.trim().to_string())
            .or_insert_with(|| importing_file.to_path_buf());
    }

    pub(super) fn merge(&mut self, other: Findings) {
        for (name, miss) in other.unresolved_macros {
            match self.unresolved_macros.get_mut(&name) {
                Some(existing) => existing.dropped_entries += miss.dropped_entries,
                None => {
                    self.unresolved_macros.insert(name, miss);
                }
            }
        }
        for (dir, file) in other.missing_dirs {
            self.missing_dirs.entry(dir).or_insert(file);
        }
        for (spelling, file) in other.unresolved_imports {
            self.unresolved_imports.entry(spelling).or_insert(file);
        }
    }

    /// Note lines for `build discover`; paths shown repo-relative.
    pub(super) fn note_lines(&self, repo_root: &Path) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.unresolved_macros.is_empty() {
            lines.push(format!(
                "MSBuild: {} unresolved macro(s); include entries using them were dropped:",
                self.unresolved_macros.len()
            ));
            let mut ranked: Vec<_> = self.unresolved_macros.iter().collect();
            ranked.sort_by(|left, right| {
                right.1.dropped_entries.cmp(&left.1.dropped_entries).then(left.0.cmp(right.0))
            });
            for (name, miss) in ranked.into_iter().take(EXAMPLE_ROWS) {
                lines.push(format!(
                    "  {name}: {} entries, e.g. \"{}\" in {}",
                    miss.dropped_entries,
                    miss.example_entry,
                    shown(&miss.example_file, repo_root)
                ));
            }
        }
        push_examples(
            &mut lines,
            "include dir(s) not found on disk",
            self.missing_dirs
                .iter()
                .map(|(dir, file)| format!("{} (named in {})", shown(dir, repo_root), shown(file, repo_root))),
            self.missing_dirs.len(),
        );
        push_examples(
            &mut lines,
            "import(s) not followed",
            self.unresolved_imports
                .iter()
                .map(|(spelling, file)| format!("{spelling} (in {})", shown(file, repo_root))),
            self.unresolved_imports.len(),
        );
        lines
    }
}

fn push_examples(
    lines: &mut Vec<String>,
    what: &str,
    rows: impl Iterator<Item = String>,
    total: usize,
) {
    if total == 0 {
        return;
    }
    lines.push(format!("MSBuild: {total} {what}:"));
    lines.extend(rows.take(EXAMPLE_ROWS).map(|row| format!("  {row}")));
    if total > EXAMPLE_ROWS {
        lines.push(format!("  ... and {} more", total - EXAMPLE_ROWS));
    }
}

/// Repo-relative, `/`-separated spelling for display.
fn shown(path: &Path, repo_root: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// The first `$(...)` reference in `entry` (the macro that sank it), or
/// the item-list/metadata marker when there is no property reference.
fn first_macro_reference(entry: &str) -> String {
    let Some(start) = entry.find("$(") else {
        return entry.to_string();
    };
    let tail = &entry[start..];
    match tail.find(')') {
        Some(end) => tail[..=end].to_string(),
        None => tail.to_string(),
    }
}
