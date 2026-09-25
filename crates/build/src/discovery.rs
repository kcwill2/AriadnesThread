use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use ariadne_repo::{FileId, RepositoryDiscoveryResult};

use crate::types::{
    BuildDiscoveryResult, CompilationContext, CompileCommand, TranslationUnit,
};

/// Clang compilation database raw entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClangCompileCommandEntry {
    pub directory: PathBuf,
    pub file: PathBuf,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub arguments: Vec<String>,
    #[serde(default)]
    pub output: Option<PathBuf>,
}

/// Discovers compilation contexts and translation units for a repository.
pub struct BuildDiscoverer<'a> {
    pub repo_root: &'a Path,
    pub repo_discovery: Option<&'a RepositoryDiscoveryResult>,
}

impl<'a> BuildDiscoverer<'a> {
    pub fn new(repo_root: &'a Path) -> Self {
        Self {
            repo_root,
            repo_discovery: None,
        }
    }

    pub fn with_repo_discovery(mut self, discovery: &'a RepositoryDiscoveryResult) -> Self {
        self.repo_discovery = Some(discovery);
        self
    }

    pub fn discover(&self) -> Result<BuildDiscoveryResult> {
        let canonical_root = self
            .repo_root
            .canonicalize()
            .unwrap_or_else(|_| self.repo_root.to_path_buf());

        // 1. Try finding compile_commands.json in root or build directories
        let candidates = [
            canonical_root.join("compile_commands.json"),
            canonical_root.join("build").join("compile_commands.json"),
            canonical_root.join("out").join("compile_commands.json"),
        ];

        for path in &candidates {
            if path.exists() {
                return self.parse_compile_commands_json(path, &canonical_root);
            }
        }

        // 2. Fallback: Default infer discovery
        self.infer_default_discovery(&canonical_root)
    }

    fn parse_compile_commands_json(
        &self,
        json_path: &Path,
        root: &Path,
    ) -> Result<BuildDiscoveryResult> {
        let content = fs::read_to_string(json_path)
            .with_context(|| format!("Failed to read compile_commands.json at {:?}", json_path))?;

        let entries: Vec<ClangCompileCommandEntry> = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse compile_commands.json at {:?}", json_path))?;

        let mut contexts_map: HashMap<String, CompilationContext> = HashMap::new();
        let mut translation_units = Vec::new();

        for entry in entries {
            let (compiler, defines, inc_dirs, sys_inc_dirs, forced_incs, lang_std, arch, raw_flags) =
                parse_args(&entry.command, &entry.arguments);

            let context_name = infer_context_name(&defines);

            let ctx = contexts_map
                .entry(context_name.clone())
                .or_insert_with(|| CompilationContext::new(&context_name, &compiler));

            if ctx.target_arch.is_none() {
                ctx.target_arch = arch;
            }
            if ctx.language_standard.is_none() {
                ctx.language_standard = lang_std;
            }

            ctx.defines.extend(defines);
            ctx.include_dirs.extend(inc_dirs);
            ctx.system_include_dirs.extend(sys_inc_dirs);
            ctx.forced_includes.extend(forced_incs);
            ctx.raw_flags.extend(raw_flags);

            let source_path = if entry.file.is_absolute() {
                entry.file.clone()
            } else {
                entry.directory.join(&entry.file)
            };

            let rel_path = if entry.file.is_relative() {
                entry.file.clone()
            } else if let Ok(stripped) = source_path.strip_prefix(root) {
                stripped.to_path_buf()
            } else {
                let norm_source = source_path.to_string_lossy().replace('\\', "/");
                let norm_root = root.to_string_lossy().replace('\\', "/");
                let clean_root = norm_root.trim_start_matches("//?/");
                if let Some(stripped) = norm_source.strip_prefix(clean_root) {
                    PathBuf::from(stripped.trim_start_matches('/'))
                } else {
                    source_path.clone()
                }
            };

            let norm_rel_str = rel_path.to_string_lossy().replace('\\', "/");
            let file_id = FileId::from_relative_path(&norm_rel_str);

            let cmd_args = if !entry.arguments.is_empty() {
                entry.arguments.clone()
            } else if let Some(ref cmd) = entry.command {
                cmd.split_whitespace().map(|s| s.to_string()).collect()
            } else {
                Vec::new()
            };

            let compile_cmd = CompileCommand {
                directory: entry.directory,
                file: source_path.clone(),
                command: entry.command,
                arguments: cmd_args,
                context_name: context_name.clone(),
                output: entry.output,
            };

            translation_units.push(TranslationUnit {
                file_id,
                source_path,
                context_name,
                compile_command: compile_cmd,
            });
        }

        let contexts: Vec<CompilationContext> = contexts_map.into_values().collect();

        let mut result = BuildDiscoveryResult {
            repo_root: root.to_path_buf(),
            contexts,
            translation_units,
        };

        result.normalize();
        Ok(result)
    }

    fn infer_default_discovery(&self, root: &Path) -> Result<BuildDiscoveryResult> {
        let mut default_ctx = CompilationContext::new("Default|x64", "cl.exe");
        default_ctx.defines.push("NDEBUG".into());

        let include_dir = root.join("include");
        if include_dir.exists() {
            default_ctx.include_dirs.push(include_dir);
        }

        let mut translation_units = Vec::new();

        if let Some(disc) = self.repo_discovery {
            for f in &disc.files {
                let is_cpp_source = f
                    .path
                    .extension()
                    .map_or(false, |ext| ext == "cpp" || ext == "cc" || ext == "cxx" || ext == "c");

                if is_cpp_source {
                    let cmd_args = vec![
                        "cl.exe".to_string(),
                        "/c".to_string(),
                        f.canonical_path.to_string_lossy().to_string(),
                    ];

                    let compile_cmd = CompileCommand {
                        directory: root.to_path_buf(),
                        file: f.canonical_path.clone(),
                        command: Some(cmd_args.join(" ")),
                        arguments: cmd_args,
                        context_name: "Default|x64".into(),
                        output: None,
                    };

                    translation_units.push(TranslationUnit {
                        file_id: f.id.clone(),
                        source_path: f.canonical_path.clone(),
                        context_name: "Default|x64".into(),
                        compile_command: compile_cmd,
                    });
                }
            }
        }

        let mut result = BuildDiscoveryResult {
            repo_root: root.to_path_buf(),
            contexts: vec![default_ctx],
            translation_units,
        };

        result.normalize();
        Ok(result)
    }
}

pub fn discover_build_contexts(repo_root: &Path) -> Result<BuildDiscoveryResult> {
    BuildDiscoverer::new(repo_root).discover()
}

fn infer_context_name(defines: &[String]) -> String {
    if defines.iter().any(|d| d == "DEBUG" || d == "_DEBUG") {
        "Debug|x64".to_string()
    } else if defines.iter().any(|d| d == "NDEBUG") {
        "Release|x64".to_string()
    } else {
        "Default|x64".to_string()
    }
}

fn parse_args(
    command: &Option<String>,
    arguments: &[String],
) -> (
    String,
    Vec<String>,
    Vec<PathBuf>,
    Vec<PathBuf>,
    Vec<PathBuf>,
    Option<String>,
    Option<String>,
    Vec<String>,
) {
    let raw_args: Vec<String> = if !arguments.is_empty() {
        arguments.to_vec()
    } else if let Some(ref cmd) = command {
        cmd.split_whitespace().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    };

    let mut compiler = "cl.exe".to_string();
    let mut defines = Vec::new();
    let mut include_dirs = Vec::new();
    let mut system_include_dirs = Vec::new();
    let mut forced_includes = Vec::new();
    let mut lang_std = None;
    let mut arch = None;
    let mut raw_flags = Vec::new();

    if let Some(first) = raw_args.first() {
        if !first.starts_with('-') && !first.starts_with('/') {
            compiler = first.clone();
        }
    }

    let mut iter = raw_args.iter().peekable();
    while let Some(arg) = iter.next() {
        if arg.starts_with("-D") || arg.starts_with("/D") {
            let val = if arg.len() > 2 {
                &arg[2..]
            } else if let Some(next) = iter.peek() {
                if !next.starts_with('-') && !next.starts_with('/') {
                    iter.next().unwrap()
                } else {
                    ""
                }
            } else {
                ""
            };
            if !val.is_empty() {
                defines.push(val.to_string());
            }
        } else if arg.starts_with("-isystem") {
            let val = if arg.len() > 8 {
                &arg[8..]
            } else if iter.peek().is_some() {
                iter.next().unwrap()
            } else {
                ""
            };
            if !val.is_empty() {
                system_include_dirs.push(PathBuf::from(val));
            }
        } else if arg.starts_with("/external:I") {
            let val = &arg[11..];
            if !val.is_empty() {
                system_include_dirs.push(PathBuf::from(val));
            }
        } else if arg.starts_with("-I") || arg.starts_with("/I") {
            let val = if arg.len() > 2 {
                &arg[2..]
            } else if let Some(next) = iter.peek() {
                if !next.starts_with('-') && !next.starts_with('/') {
                    iter.next().unwrap()
                } else {
                    ""
                }
            } else {
                ""
            };
            if !val.is_empty() {
                include_dirs.push(PathBuf::from(val));
            }
        } else if arg.starts_with("-FI") || arg.starts_with("/FI") {
            let val = if arg.len() > 3 {
                &arg[3..]
            } else if let Some(next) = iter.peek() {
                if !next.starts_with('-') && !next.starts_with('/') {
                    iter.next().unwrap()
                } else {
                    ""
                }
            } else {
                ""
            };
            if !val.is_empty() {
                forced_includes.push(PathBuf::from(val));
            }
        } else if arg.starts_with("-std=") || arg.starts_with("/std:") {
            let pos = arg.find('=').or_else(|| arg.find(':')).unwrap();
            lang_std = Some(arg[pos + 1..].to_string());
        } else if arg.starts_with("/arch:") {
            arch = Some(arg[6..].to_string());
        } else if arg.starts_with('-') || arg.starts_with('/') {
            raw_flags.push(arg.clone());
        }
    }

    (
        compiler,
        defines,
        include_dirs,
        system_include_dirs,
        forced_includes,
        lang_std,
        arch,
        raw_flags,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_compile_commands_debug_and_release() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let json_content = r#"[
            {
                "directory": "C:/src/project",
                "command": "cl.exe /c /Iinclude /DDEBUG /std:c++17 src/thing.cpp",
                "file": "src/thing.cpp"
            },
            {
                "directory": "C:/src/project",
                "command": "cl.exe /c /Iinclude /DNDEBUG /std:c++17 src/thing.cpp",
                "file": "src/thing.cpp"
            }
        ]"#;

        let json_path = root.join("compile_commands.json");
        fs::write(&json_path, json_content).unwrap();

        let discoverer = BuildDiscoverer::new(root);
        let result = discoverer.discover().unwrap();

        assert_eq!(result.contexts.len(), 2);
        assert_eq!(result.translation_units.len(), 2);

        let debug_ctx = result.contexts.iter().find(|c| c.name == "Debug|x64").unwrap();
        assert!(debug_ctx.defines.contains(&"DEBUG".to_string()));

        let release_ctx = result.contexts.iter().find(|c| c.name == "Release|x64").unwrap();
        assert!(release_ctx.defines.contains(&"NDEBUG".to_string()));
    }
}
