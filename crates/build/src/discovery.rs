use anyhow::{Context, Result};
use ariadne_lang::{UnitDescriptor, UnitShaper};
use ariadne_repo::{FileId, RepositoryDiscoveryResult, SourceLanguage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{BuildDiscoveryResult, CompilationContext, CompileCommand, TranslationUnit};

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
///
/// Language knowledge enters only through the optional [`UnitShaper`]
/// (plan M18.4): with a shaper injected and no compilation database present,
/// units come from `shape_units()`; without one, discovery that finds no
/// `compile_commands.json` yields an empty result instead of a language
/// default.
pub struct BuildDiscoverer<'a> {
    pub repo_root: &'a Path,
    pub repo_discovery: Option<&'a RepositoryDiscoveryResult>,
    pub unit_shaper: Option<&'a dyn UnitShaper>,
}

impl<'a> BuildDiscoverer<'a> {
    pub fn new(repo_root: &'a Path) -> Self {
        Self {
            repo_root,
            repo_discovery: None,
            unit_shaper: None,
        }
    }

    pub fn with_repo_discovery(mut self, discovery: &'a RepositoryDiscoveryResult) -> Self {
        self.repo_discovery = Some(discovery);
        self
    }

    /// Inject the language-specific [`UnitShaper`] that turns discovered
    /// source files into translation units and derives context names
    /// (plan M18.4). This is the ONLY channel through which language knowledge
    /// (unit extensions, the synthetic compile invocation, the default-context
    /// naming) enters build discovery; the crate itself carries no C++/Scala
    /// vocabulary. Pass the registry's `unit_shaper_for()` (or, during the M18
    /// transition, the plugin crate's default shaper). With `None` and no
    /// `compile_commands.json`, discovery honestly yields no units instead of
    /// silently guessing a language default.
    pub fn with_unit_shaper(mut self, shaper: Option<&'a dyn UnitShaper>) -> Self {
        self.unit_shaper = shaper;
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

        // 2. Fallback: shape units through the injected language shaper.
        if let Some(shaper) = self.unit_shaper {
            return self.discover_through_shaper(shaper, &canonical_root);
        }

        // 3. No compilation database and no shaper: stay neutral (M18.4 —
        // the old infer_default_discovery C++ default lives in
        // lang-cpp::CppUnitShaper now and must be injected by the caller).
        eprintln!(
            "warning: no compile_commands.json found under {:?} and no unit shaper \
             injected; producing no build contexts or translation units",
            canonical_root
        );
        Ok(BuildDiscoveryResult {
            repo_root: canonical_root,
            contexts: Vec::new(),
            translation_units: Vec::new(),
        })
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

            let context_name = match self.unit_shaper {
                Some(shaper) => shaper.context_name_for_defines(&defines),
                None => infer_context_name(&defines),
            };

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

            let rel_path = relative_path_for_source(&entry.file, &source_path, root);

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

            // compile_commands.json is a C/C++-specific artifact; a non-C++
            // unit arriving here indicates plugin wiring lost (audit M18,
            // item 3) — shaper injection for this path is an M19+ item
            translation_units.push(TranslationUnit {
                file_id,
                source_path,
                context_name,
                compile_command: compile_cmd,
                language: SourceLanguage::Cpp,
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

    /// Synthesizes the discovery result from a shaper's unit descriptors:
    /// one [`TranslationUnit`] per descriptor plus a [`CompilationContext`]
    /// per distinct context name, assembled purely from the shaper's
    /// `defines` / `include_dirs` / `build_spec` ingredients — this crate
    /// keeps no language literals of its own (plan M18.4).
    fn discover_through_shaper(
        &self,
        shaper: &dyn UnitShaper,
        root: &Path,
    ) -> Result<BuildDiscoveryResult> {
        let empty_files: Vec<ariadne_repo::FileRecord> = Vec::new();
        let files = self.repo_discovery.map_or(&empty_files, |d| &d.files);
        let records_by_id: HashMap<FileId, &ariadne_repo::FileRecord> =
            files.iter().map(|f| (f.id, f)).collect();

        let mut contexts_map: HashMap<String, CompilationContext> = HashMap::new();
        let mut translation_units = Vec::new();

        for descriptor in shaper.shape_units(root, files) {
            let Some(record) = records_by_id.get(&descriptor.file_id) else {
                continue;
            };
            let unit =
                Self::unit_from_descriptor(&descriptor, record, root, &mut contexts_map, shaper);
            translation_units.push(unit);
        }

        let mut result = BuildDiscoveryResult {
            repo_root: root.to_path_buf(),
            contexts: contexts_map.into_values().collect(),
            translation_units,
        };

        result.normalize();
        Ok(result)
    }

    fn unit_from_descriptor(
        descriptor: &UnitDescriptor,
        record: &ariadne_repo::FileRecord,
        root: &Path,
        contexts_map: &mut HashMap<String, CompilationContext>,
        shaper: &dyn UnitShaper,
    ) -> TranslationUnit {
        let ctx = contexts_map
            .entry(descriptor.context_name.clone())
            .or_insert_with(|| {
                let mut ctx =
                    CompilationContext::new(&descriptor.context_name, shaper.default_compiler());
                ctx.language = descriptor.language;
                ctx
            });
        ctx.defines.extend(descriptor.defines.iter().cloned());
        ctx.include_dirs
            .extend(descriptor.include_dirs.iter().cloned());

        TranslationUnit {
            file_id: descriptor.file_id,
            source_path: record.canonical_path.clone(),
            context_name: descriptor.context_name.clone(),
            compile_command: CompileCommand {
                directory: root.to_path_buf(),
                file: record.canonical_path.clone(),
                command: Some(descriptor.build_spec.join(" ")),
                arguments: descriptor.build_spec.clone(),
                context_name: descriptor.context_name.clone(),
                output: None,
            },
            language: descriptor.language,
        }
    }
}

pub fn discover_build_contexts(repo_root: &Path) -> Result<BuildDiscoveryResult> {
    BuildDiscoverer::new(repo_root).discover()
}

/// Repo-relative path of a compilation-database source, mirroring the
/// verbatim strip / normalized-slash fallback chain of the original parser.
fn relative_path_for_source(entry_file: &Path, source_path: &Path, root: &Path) -> PathBuf {
    if entry_file.is_relative() {
        return entry_file.to_path_buf();
    }
    if let Ok(stripped) = source_path.strip_prefix(root) {
        return stripped.to_path_buf();
    }
    let norm_source = source_path.to_string_lossy().replace('\\', "/");
    let norm_root = root.to_string_lossy().replace('\\', "/");
    let clean_root = norm_root.trim_start_matches("//?/");
    if let Some(stripped) = norm_source.strip_prefix(clean_root) {
        PathBuf::from(stripped.trim_start_matches('/'))
    } else {
        source_path.to_path_buf()
    }
}

/// Context naming for the shaper-less `compile_commands.json` path: the
/// m2/characterization suites lock these names without a shaper, so the
/// neutral plumbing keeps a default. Callers with a language plugin should
/// inject one — `UnitShaper::context_name_for_defines` then drives naming
/// and this default is not consulted.
fn infer_context_name(defines: &[String]) -> String {
    if defines.iter().any(|d| d == "DEBUG" || d == "_DEBUG") {
        "Debug|x64".to_string()
    } else if defines.iter().any(|d| d == "NDEBUG") {
        "Release|x64".to_string()
    } else {
        "Default|x64".to_string()
    }
}

fn split_command_string(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;

    for c in cmd.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' | '\n' | '\r' if !in_quotes => {
                if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            _ => current_arg.push(c),
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}

/// Peekable cursor over the raw argument list consumed by `parse_args`.
type ArgCursor<'a> = std::iter::Peekable<std::slice::Iter<'a, String>>;

fn raw_argument_list(command: &Option<String>, arguments: &[String]) -> Vec<String> {
    if !arguments.is_empty() {
        arguments.to_vec()
    } else if let Some(ref cmd) = command {
        split_command_string(cmd)
    } else {
        Vec::new()
    }
}

/// Heuristic from the original parser: `/`-prefixed unix-ish paths that
/// contain "usr"/"bin" or end in a compiler name are treated as paths, not
/// MSVC-style flags.
fn is_compilation_flag(s: &str) -> bool {
    s.starts_with('-')
        || (s.starts_with('/')
            && !s.contains("usr")
            && !s.contains("bin")
            && !s.ends_with("c++")
            && !s.ends_with("g++")
            && !s.ends_with("clang")
            && !s.ends_with("clang++"))
}

fn take_compiler_executable(iter: &mut ArgCursor<'_>) -> String {
    if let Some(first) = iter.peek() {
        if !is_compilation_flag(first) {
            return iter
                .next()
                .expect("peeked Some: next() on the same cursor cannot be None")
                .clone();
        }
    }
    "cl.exe".to_string()
}

/// Value of a flag family whose value may be joined to the flag (`-DFOO`)
/// or the next token (`-D FOO`). A bare flag consumes the next token even
/// when it is another flag; at end-of-args it yields an empty value.
fn take_prefixed_value(arg: &str, prefixes: &[&str], iter: &mut ArgCursor<'_>) -> Option<String> {
    let prefix = prefixes.iter().find(|p| arg.starts_with(**p))?;
    let value = if arg.len() > prefix.len() {
        arg[prefix.len()..].to_string()
    } else if iter.peek().is_some() {
        iter.next()
            .expect("peeked Some: next() on the same cursor cannot be None")
            .clone()
    } else {
        String::new()
    };
    Some(value)
}

fn language_standard_from_flag(arg: &str) -> Option<String> {
    if !arg.starts_with("-std=") && !arg.starts_with("/std:") {
        return None;
    }
    let pos = arg.find('=').or_else(|| arg.find(':'))?;
    Some(arg[pos + 1..].to_string())
}

fn arch_target_from_flag(arg: &str) -> Option<String> {
    Some(arg.strip_prefix("/arch:")?.to_string())
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
    let raw_args = raw_argument_list(command, arguments);
    let mut iter = raw_args.iter().peekable();

    let compiler = take_compiler_executable(&mut iter);
    let mut defines = Vec::new();
    let mut include_dirs = Vec::new();
    let mut system_include_dirs = Vec::new();
    let mut forced_includes = Vec::new();
    let mut lang_std = None;
    let mut arch = None;
    let mut raw_flags = Vec::new();

    while let Some(arg) = iter.next() {
        if let Some(value) = take_prefixed_value(arg, &["-D", "/D"], &mut iter) {
            if !value.is_empty() {
                defines.push(value);
            }
        } else if let Some(value) = take_prefixed_value(arg, &["-isystem"], &mut iter) {
            if !value.is_empty() {
                system_include_dirs.push(PathBuf::from(value));
            }
        } else if let Some(value) = arg.strip_prefix("/external:I") {
            // No next-token form: a bare /external:I only strips an empty
            // suffix here and its intended path token is dropped downstream.
            if !value.is_empty() {
                system_include_dirs.push(PathBuf::from(value));
            }
        } else if let Some(value) = take_prefixed_value(arg, &["-I", "/I"], &mut iter) {
            if !value.is_empty() {
                include_dirs.push(PathBuf::from(value));
            }
        } else if let Some(value) = take_prefixed_value(arg, &["-FI", "/FI"], &mut iter) {
            if !value.is_empty() {
                forced_includes.push(PathBuf::from(value));
            }
        } else if let Some(spec) = language_standard_from_flag(arg) {
            lang_std = Some(spec);
        } else if let Some(target) = arch_target_from_flag(arg) {
            arch = Some(target);
        } else if is_compilation_flag(arg) {
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
    fn test_parse_args_gcc_clang() {
        let cmd = None;
        let args = vec![
            "/usr/bin/c++".to_string(),
            "-isystem".to_string(),
            "/usr/include".to_string(),
            "-isystem/usr/local/include".to_string(),
            "-I".to_string(),
            "/path/to/include".to_string(),
            "-Iinclude".to_string(),
            "-D".to_string(),
            "MACRO=1".to_string(),
            "-DMACRO_2".to_string(),
            "-std=c++20".to_string(),
            "-FI".to_string(),
            "/path/to/prefix.h".to_string(),
        ];

        let (compiler, defines, inc_dirs, sys_inc_dirs, forced_incs, lang_std, arch, _raw_flags) =
            parse_args(&cmd, &args);

        assert_eq!(compiler, "/usr/bin/c++");
        assert_eq!(
            sys_inc_dirs,
            vec![
                PathBuf::from("/usr/include"),
                PathBuf::from("/usr/local/include")
            ]
        );
        assert_eq!(
            inc_dirs,
            vec![PathBuf::from("/path/to/include"), PathBuf::from("include")]
        );
        assert_eq!(defines, vec!["MACRO=1".to_string(), "MACRO_2".to_string()]);
        assert_eq!(lang_std, Some("c++20".to_string()));
        assert_eq!(forced_incs, vec![PathBuf::from("/path/to/prefix.h")]);
        assert_eq!(arch, None);
    }

    #[test]
    fn test_parse_args_msvc() {
        let cmd = None;
        let args = vec![
            "cl.exe".to_string(),
            "/I".to_string(),
            "include".to_string(),
            "/I".to_string(),
            "C:\\Program Files\\include".to_string(),
            "/D".to_string(),
            "NDEBUG".to_string(),
            "/DDEBUG".to_string(),
            "/std:c++latest".to_string(),
            "/arch:AVX2".to_string(),
            "/FI".to_string(),
            "stdafx.h".to_string(),
        ];

        let (compiler, defines, inc_dirs, __sys_inc_dirs, forced_incs, lang_std, arch, _raw_flags) =
            parse_args(&cmd, &args);

        assert_eq!(compiler, "cl.exe");
        assert_eq!(
            inc_dirs,
            vec![
                PathBuf::from("include"),
                PathBuf::from("C:\\Program Files\\include")
            ]
        );
        assert_eq!(defines, vec!["NDEBUG".to_string(), "DEBUG".to_string()]);
        assert_eq!(lang_std, Some("c++latest".to_string()));
        assert_eq!(arch, Some("AVX2".to_string()));
        assert_eq!(forced_incs, vec![PathBuf::from("stdafx.h")]);
    }

    #[test]
    fn test_parse_args_quote_handling_in_command() {
        let cmd = Some(
            "cl.exe /I \"C:\\Program Files\\include\" /D MACRO=\"spaced val\" src.cpp".to_string(),
        );
        let args = vec![];

        let (
            compiler,
            defines,
            inc_dirs,
            _sys_inc_dirs,
            _forced_incs,
            _lang_std,
            _arch,
            _raw_flags,
        ) = parse_args(&cmd, &args);

        assert_eq!(compiler, "cl.exe");
        assert_eq!(inc_dirs, vec![PathBuf::from("C:\\Program Files\\include")]);
        assert_eq!(defines, vec!["MACRO=spaced val".to_string()]);
    }

    #[test]
    fn test_infer_context_name() {
        assert_eq!(infer_context_name(&["DEBUG".to_string()]), "Debug|x64");
        assert_eq!(infer_context_name(&["_DEBUG".to_string()]), "Debug|x64");
        assert_eq!(infer_context_name(&["NDEBUG".to_string()]), "Release|x64");
        assert_eq!(infer_context_name(&["OTHER".to_string()]), "Default|x64");
        assert_eq!(infer_context_name(&[]), "Default|x64");
    }

    #[test]
    fn test_build_discoverer_compile_commands_with_args() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let json_content = r#"[
            {
                "directory": "C:/src/project",
                "arguments": ["clang++", "-I", "include", "-D", "DEBUG", "src/thing.cpp"],
                "file": "src/thing.cpp"
            }
        ]"#;

        let json_path = root.join("compile_commands.json");
        fs::write(&json_path, json_content).unwrap();

        let discoverer = BuildDiscoverer::new(root);
        let result = discoverer.discover().unwrap();

        assert_eq!(result.contexts.len(), 1);
        assert_eq!(result.translation_units.len(), 1);

        let ctx = &result.contexts[0];
        assert_eq!(ctx.name, "Debug|x64");
        assert_eq!(ctx.compiler, "clang++");
        assert!(ctx.defines.contains(&"DEBUG".to_string()));

        let tu = &result.translation_units[0];
        assert_eq!(
            tu.compile_command.arguments,
            vec!["clang++", "-I", "include", "-D", "DEBUG", "src/thing.cpp"]
        );
    }

    #[test]
    fn test_build_discoverer_infer_default() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // No compile_commands.json
        // Mock a repository discovery result
        let file_path = root.join("src").join("main.cpp");
        let disc = RepositoryDiscoveryResult {
            root: root.to_path_buf(),
            files: vec![ariadne_repo::FileRecord {
                id: ariadne_repo::FileId::from_relative_path("src/main.cpp"),
                path: std::path::PathBuf::from("src/main.cpp"),
                canonical_path: file_path.clone(),
                kind: ariadne_repo::FileKind::Source,
                size: 0,
                content_hash: "dummy".to_string(),
                ownership: ariadne_repo::FileOwnership::Repository,
                optimization_policy: ariadne_repo::OptimizationPolicy::Optimizable,
                language: ariadne_repo::SourceLanguage::Cpp,
            }],
            ownership_summary: Default::default(),
            policy_summary: Default::default(),
        };

        // M18.4: the Default|x64 shaping is lang-cpp knowledge; the caller
        // injects the C++ shaper to keep today's behaviour.
        let shaper = ariadne_lang_cpp::cpp_default_shaper();
        let discoverer = BuildDiscoverer::new(root)
            .with_repo_discovery(&disc)
            .with_unit_shaper(Some(shaper));
        let result = discoverer.discover().unwrap();

        assert_eq!(result.contexts.len(), 1);
        let ctx = &result.contexts[0];
        assert_eq!(ctx.name, "Default|x64");
        assert_eq!(ctx.compiler, "cl.exe");
        assert!(ctx.defines.contains(&"NDEBUG".to_string()));
        assert_eq!(ctx.language, ariadne_repo::SourceLanguage::Cpp);

        assert_eq!(result.translation_units.len(), 1);
        let tu = &result.translation_units[0];
        assert_eq!(tu.source_path, file_path);
        assert_eq!(tu.language, ariadne_repo::SourceLanguage::Cpp);
        assert_eq!(
            tu.compile_command.arguments,
            vec![
                "cl.exe".to_string(),
                "/c".to_string(),
                file_path.to_string_lossy().to_string(),
            ]
        );
    }

    #[test]
    fn test_no_shaper_without_compile_commands_yields_no_contexts() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.cpp"), "int main(){}").unwrap();

        let disc = RepositoryDiscoveryResult {
            root: root.to_path_buf(),
            files: vec![ariadne_repo::FileRecord {
                id: ariadne_repo::FileId::from_relative_path("src/main.cpp"),
                path: std::path::PathBuf::from("src/main.cpp"),
                canonical_path: root.join("src/main.cpp"),
                kind: ariadne_repo::FileKind::Source,
                size: 0,
                content_hash: "dummy".to_string(),
                ownership: ariadne_repo::FileOwnership::Repository,
                optimization_policy: ariadne_repo::OptimizationPolicy::Optimizable,
                language: ariadne_repo::SourceLanguage::Cpp,
            }],
            ownership_summary: Default::default(),
            policy_summary: Default::default(),
        };

        // Without a shaper the fallback is gone: no C++ default, no contexts,
        // no units (M18.4 — callers opt into a language explicitly).
        let result = BuildDiscoverer::new(root)
            .with_repo_discovery(&disc)
            .discover()
            .unwrap();
        assert!(result.contexts.is_empty());
        assert!(result.translation_units.is_empty());
        assert_eq!(
            result.repo_root,
            root.canonicalize().unwrap_or_else(|_| root.to_path_buf())
        );
    }

    #[test]
    fn shaper_drives_context_naming_on_compile_commands_path() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        let json_content = r#"[
            {
                "directory": "C:/src/project",
                "arguments": ["cl.exe", "/D", "NDEBUG", "src/thing.cpp"],
                "file": "src/thing.cpp"
            }
        ]"#;
        std::fs::write(root.join("compile_commands.json"), json_content).unwrap();

        let result = BuildDiscoverer::new(root)
            .with_unit_shaper(Some(ariadne_lang_cpp::cpp_default_shaper()))
            .discover()
            .unwrap();

        assert_eq!(result.contexts.len(), 1);
        assert_eq!(result.contexts[0].name, "Release|x64");
        assert_eq!(result.translation_units[0].language, SourceLanguage::Cpp);
    }
}

/// Characterization suite: locks CURRENT behavior of `parse_args` and
/// `parse_compile_commands_json` before the Pass 3 decomposition.
/// Where a case encodes a quirk (dropped token, swallowed flag, empty-value
/// record), the assertion documents today's behavior, not desired behavior.
#[cfg(test)]
#[path = "discovery_characterization_tests.rs"]
mod characterization;
