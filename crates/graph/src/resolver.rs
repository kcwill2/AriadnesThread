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
                    match components.last() {
                        Some(&std::path::Component::ParentDir) | None => {
                            components.push(component);
                        }
                        Some(&std::path::Component::RootDir) | Some(&std::path::Component::Prefix(_)) => {
                            // Cannot resolve `..` above root, so do nothing.
                        }
                        Some(&std::path::Component::Normal(_)) => {
                            components.pop();
                        }
                        _ => {}
                    }
                }
                c => components.push(c),
            }
        }
        let mut normalized = PathBuf::new();
        for c in components {
            normalized.push(c.as_os_str());
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
    use std::collections::HashMap;
    use ariadne_repo::{FileKind, FileOwnership, OptimizationPolicy, FileRecord};
    use ariadne_build::CompilationContext;

    fn mock_file(root: &Path, rel: &str, ownership: FileOwnership) -> FileRecord {
        FileRecord {
            id: FileId::from_relative_path(rel),
            path: PathBuf::from(rel),
            canonical_path: root.join(rel),
            kind: FileKind::Header,
            size: 0,
            content_hash: String::new(),
            ownership,
            optimization_policy: OptimizationPolicy::Optimizable,
        }
    }

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

    #[test]
    fn test_normalize_path() {
        // Redundant separators
        assert_eq!(IncludeResolver::normalize_path(Path::new("a//b///c")), PathBuf::from("a/b/c"));
        
        // Single dots
        assert_eq!(IncludeResolver::normalize_path(Path::new("a/./b/./c")), PathBuf::from("a/b/c"));
        
        // Double dots (traversal)
        assert_eq!(IncludeResolver::normalize_path(Path::new("a/b/../c")), PathBuf::from("a/c"));
        
        // Multiple double dots
        assert_eq!(IncludeResolver::normalize_path(Path::new("a/b/../../c")), PathBuf::from("c"));
        
        // Relative traversal starting with .. should keep the ..
        assert_eq!(IncludeResolver::normalize_path(Path::new("../a")), PathBuf::from("../a"));
        
        // Traversal above current dir from a known path
        assert_eq!(IncludeResolver::normalize_path(Path::new("a/../../b")), PathBuf::from("../b"));
    }

    #[test]
    fn test_resolve_directive() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        
        let mut files = Vec::new();
        files.push(mock_file(root, "common/helper.h", FileOwnership::Repository));
        files.push(mock_file(root, "third_party/lib/lib.h", FileOwnership::ThirdParty));
        files.push(mock_file(root, "usr/include/system.h", FileOwnership::Toolchain));
        
        let discovery = RepositoryDiscoveryResult {
            root: root.to_path_buf(),
            files,
            ownership_summary: HashMap::new(),
            policy_summary: HashMap::new(),
        };
        
        let mut context = CompilationContext::new("test", "clang");
        context.include_dirs = vec![PathBuf::from("third_party/lib")];
        context.system_include_dirs = vec![PathBuf::from("usr/include")];
        
        let resolver = IncludeResolver::new(root)
            .with_repo_discovery(&discovery)
            .with_context(&context);
            
        let source_id = FileId::from_relative_path("src/main.cpp");
        let source_rel_path = Path::new("src/main.cpp");
        
        // Setup a real disk file for the fallback test
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/sibling.h"), "").unwrap();

        // 1. Quoted local relative includes
        // Disk fallback (sibling.h is not in discovery)
        let dir1 = RawIncludeDirective { raw_path: "sibling.h".into(), is_angled: false, line: 1, column: 1 };
        let edge1 = resolver.resolve_directive(source_id, source_rel_path, &dir1);
        assert_eq!(edge1.kind, IncludeKind::Local);
        assert_eq!(edge1.destination, Some(FileId::from_relative_path("src/sibling.h")));
        assert_eq!(edge1.resolved_path, Some(PathBuf::from("src/sibling.h")));

        // Found in discovery (repo local traversal)
        let dir2 = RawIncludeDirective { raw_path: "../common/helper.h".into(), is_angled: false, line: 2, column: 1 };
        let edge2 = resolver.resolve_directive(source_id, source_rel_path, &dir2);
        assert_eq!(edge2.kind, IncludeKind::Local);
        assert_eq!(edge2.destination, Some(FileId::from_relative_path("common/helper.h")));

        // 2. Angled includes
        // Toolchain (System)
        let dir3 = RawIncludeDirective { raw_path: "system.h".into(), is_angled: true, line: 3, column: 1 };
        let edge3 = resolver.resolve_directive(source_id, source_rel_path, &dir3);
        assert_eq!(edge3.kind, IncludeKind::System);
        assert_eq!(edge3.destination, Some(FileId::from_relative_path("usr/include/system.h")));

        // ThirdParty (External) - Note: include_dirs maps to External if the file is ThirdParty
        let dir4 = RawIncludeDirective { raw_path: "lib.h".into(), is_angled: true, line: 4, column: 1 };
        let edge4 = resolver.resolve_directive(source_id, source_rel_path, &dir4);
        assert_eq!(edge4.kind, IncludeKind::External);
        assert_eq!(edge4.destination, Some(FileId::from_relative_path("third_party/lib/lib.h")));

        // 3. Unresolved include
        let dir5 = RawIncludeDirective { raw_path: "nonexistent.h".into(), is_angled: false, line: 5, column: 1 };
        let edge5 = resolver.resolve_directive(source_id, source_rel_path, &dir5);
        assert_eq!(edge5.destination, None);
        assert_eq!(edge5.resolved_path, None);
    }

    #[test]
    fn test_resolve_file_includes() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        
        let mut files = Vec::new();
        files.push(mock_file(root, "include/header.h", FileOwnership::Repository));
        
        let discovery = RepositoryDiscoveryResult {
            root: root.to_path_buf(),
            files,
            ownership_summary: HashMap::new(),
            policy_summary: HashMap::new(),
        };
        
        let mut context = CompilationContext::new("test", "clang");
        context.include_dirs = vec![PathBuf::from("include")];
        
        let resolver = IncludeResolver::new(root)
            .with_repo_discovery(&discovery)
            .with_context(&context);
            
        let content = r#"
#include "header.h"
#include <missing.h>
"#;
        let edges = resolver.resolve_file_includes(
            FileId::from_relative_path("src/lib.cpp"),
            Path::new("src/lib.cpp"),
            content,
        );
        
        assert_eq!(edges.len(), 2);
        
        assert_eq!(edges[0].raw_path, "header.h");
        assert_eq!(edges[0].destination, Some(FileId::from_relative_path("include/header.h")));
        
        assert_eq!(edges[1].raw_path, "missing.h");
        assert_eq!(edges[1].destination, None);
    }
}
