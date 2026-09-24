//! Textual `#include` directive parser and search path resolver.

use std::path::{Path, PathBuf};
use ariadne_repo::{FileId, FileOwnership, RepositoryDiscoveryResult};
use ariadne_build::CompilationContext;
use crate::types::{IncludeEdge, IncludeKind, SourceLocation};

/// Parsed raw include directive from source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawIncludeDirective {
    pub raw_path: String,
    pub is_angled: bool,
    pub line: u32,
    pub column: u32,
}

/// Helper to parse `#include` lines from C/C++ source/header text.
pub fn parse_include_directives(content: &str) -> Vec<RawIncludeDirective> {
    let mut directives = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }

        let after_hash = trimmed[1..].trim_start();
        if !after_hash.starts_with("include") {
            continue;
        }

        let after_include = after_hash["include".len()..].trim_start();
        let (raw_path, is_angled, col_offset) = if after_include.starts_with('"') {
            if let Some(end_idx) = after_include[1..].find('"') {
                let path = &after_include[1..=end_idx];
                (path.to_string(), false, line.find('"').unwrap_or(0) + 1)
            } else {
                continue;
            }
        } else if after_include.starts_with('<') {
            if let Some(end_idx) = after_include[1..].find('>') {
                let path = &after_include[1..=end_idx];
                (path.to_string(), true, line.find('<').unwrap_or(0) + 1)
            } else {
                continue;
            }
        } else {
            continue;
        };

        directives.push(RawIncludeDirective {
            raw_path,
            is_angled,
            line: (line_idx + 1) as u32,
            column: col_offset as u32,
        });
    }

    directives
}

/// Include path resolver against repository file records and compilation context.
pub struct IncludeResolver<'a> {
    pub repo_root: &'a Path,
    pub repo_discovery: Option<&'a RepositoryDiscoveryResult>,
    pub context: Option<&'a CompilationContext>,
}

impl<'a> IncludeResolver<'a> {
    pub fn new(repo_root: &'a Path) -> Self {
        Self {
            repo_root,
            repo_discovery: None,
            context: None,
        }
    }

    pub fn with_repo_discovery(mut self, repo_discovery: &'a RepositoryDiscoveryResult) -> Self {
        self.repo_discovery = Some(repo_discovery);
        self
    }

    pub fn with_context(mut self, context: &'a CompilationContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Normalizes a path by resolving `.` and `..` relative components without requiring existing files.
    pub fn normalize_path(path: &Path) -> PathBuf {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    components.pop();
                }
                c => components.push(c.as_os_str()),
            }
        }
        let mut normalized = PathBuf::new();
        for c in components {
            normalized.push(c);
        }
        normalized
    }

    /// Resolves a single raw include directive for a source file.
    pub fn resolve_directive(
        &self,
        source_id: FileId,
        source_rel_path: &Path,
        directive: &RawIncludeDirective,
    ) -> IncludeEdge {
        let context_name = self.context.map(|c| c.name.as_str()).unwrap_or("Default").to_string();
        let source_dir = source_rel_path.parent().unwrap_or_else(|| Path::new(""));
        let include_raw = Path::new(&directive.raw_path);

        let mut candidate_paths = Vec::new();

        // 1. If non-angled (`"..."`), check source directory first
        if !directive.is_angled {
            candidate_paths.push(source_dir.join(include_raw));
        }

        // 2. Check compilation context user include dirs
        if let Some(ctx) = self.context {
            for inc_dir in &ctx.include_dirs {
                let candidate = if inc_dir.is_absolute() {
                    if let Ok(rel) = inc_dir.strip_prefix(self.repo_root) {
                        rel.join(include_raw)
                    } else {
                        inc_dir.join(include_raw)
                    }
                } else {
                    inc_dir.join(include_raw)
                };
                candidate_paths.push(candidate);
            }
        }

        // 3. Check compilation context system include dirs
        if let Some(ctx) = self.context {
            for sys_dir in &ctx.system_include_dirs {
                let candidate = if sys_dir.is_absolute() {
                    if let Ok(rel) = sys_dir.strip_prefix(self.repo_root) {
                        rel.join(include_raw)
                    } else {
                        sys_dir.join(include_raw)
                    }
                } else {
                    sys_dir.join(include_raw)
                };
                candidate_paths.push(candidate);
            }
        }

        // Search in repository discovery
        let mut resolved_dest: Option<FileId> = None;
        let mut resolved_path: Option<PathBuf> = None;
        let mut kind = if directive.is_angled { IncludeKind::System } else { IncludeKind::Local };

        if let Some(disc) = self.repo_discovery {
            for cand in &candidate_paths {
                let norm_cand = Self::normalize_path(cand);
                if let Some(file_record) = disc.files.iter().find(|f| {
                    Self::normalize_path(&f.path) == norm_cand
                        || Self::normalize_path(&f.canonical_path) == norm_cand
                }) {
                    resolved_dest = Some(file_record.id);
                    resolved_path = Some(file_record.path.clone());
                    kind = match file_record.ownership {
                        FileOwnership::ThirdParty => IncludeKind::External,
                        FileOwnership::Toolchain => IncludeKind::System,
                        _ => IncludeKind::Local,
                    };
                    break;
                }
            }
        }

        // Fallback: check filesystem on disk under repo_root if discovery missing or unmatched
        if resolved_dest.is_none() {
            for cand in &candidate_paths {
                let norm_cand = Self::normalize_path(cand);
                let full_cand = self.repo_root.join(&norm_cand);
                if full_cand.exists() {
                    let rel_str = norm_cand.to_string_lossy().replace('\\', "/");
                    let file_id = FileId::from_relative_path(&rel_str);
                    resolved_dest = Some(file_id);
                    resolved_path = Some(norm_cand);
                    break;
                }
            }
        }

        IncludeEdge {
            source: source_id,
            destination: resolved_dest,
            context_name,
            source_location: SourceLocation::new(directive.line, directive.column),
            kind,
            raw_path: directive.raw_path.clone(),
            resolved_path,
        }
    }

    /// Resolves all includes in a file given its content and relative path.
    pub fn resolve_file_includes(
        &self,
        source_id: FileId,
        source_rel_path: &Path,
        content: &str,
    ) -> Vec<IncludeEdge> {
        let directives = parse_include_directives(content);
        directives
            .into_iter()
            .map(|dir| self.resolve_directive(source_id, source_rel_path, &dir))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_include_directives() {
        let content = r#"
#pragma once
#include "thing_types.h"
#  include <vector>
#include "heavy.h" // comment
"#;

        let directives = parse_include_directives(content);
        assert_eq!(directives.len(), 3);
        assert_eq!(directives[0].raw_path, "thing_types.h");
        assert!(!directives[0].is_angled);
        assert_eq!(directives[0].line, 3);

        assert_eq!(directives[1].raw_path, "vector");
        assert!(directives[1].is_angled);
        assert_eq!(directives[1].line, 4);

        assert_eq!(directives[2].raw_path, "heavy.h");
        assert!(!directives[2].is_angled);
        assert_eq!(directives[2].line, 5);
    }
}
