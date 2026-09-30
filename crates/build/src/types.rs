use ariadne_repo::{FileId, SourceLanguage};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a compiler environment and build settings configuration.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CompilationContext {
    pub name: String,
    pub compiler: String,
    pub target_arch: Option<String>,
    pub language_standard: Option<String>,
    pub defines: Vec<String>,
    pub include_dirs: Vec<PathBuf>,
    pub system_include_dirs: Vec<PathBuf>,
    pub forced_includes: Vec<PathBuf>,
    pub raw_flags: Vec<String>,
    /// Plan §3.5 additive artifact field: artifacts written before this key
    /// existed read as `Cpp` (`SourceLanguage::default()`), the only language
    /// the pipeline could produce then.
    #[serde(default)]
    pub language: SourceLanguage,
}

impl CompilationContext {
    pub fn new(name: impl Into<String>, compiler: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            compiler: compiler.into(),
            target_arch: None,
            language_standard: None,
            defines: Vec::new(),
            include_dirs: Vec::new(),
            system_include_dirs: Vec::new(),
            forced_includes: Vec::new(),
            raw_flags: Vec::new(),
            language: SourceLanguage::default(),
        }
    }

    /// Normalizes internal lists (deduplicates and sorts) to guarantee determinism.
    pub fn normalize(&mut self) {
        self.defines.sort();
        self.defines.dedup();

        self.include_dirs.sort();
        self.include_dirs.dedup();

        self.system_include_dirs.sort();
        self.system_include_dirs.dedup();

        self.forced_includes.sort();
        self.forced_includes.dedup();

        self.raw_flags.sort();
        self.raw_flags.dedup();
    }
}

/// Compilation command entry representing a single source file invocation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CompileCommand {
    pub directory: PathBuf,
    pub file: PathBuf,
    pub command: Option<String>,
    pub arguments: Vec<String>,
    pub context_name: String,
    pub output: Option<PathBuf>,
}

/// A translation unit bound to a file ID and its build context.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TranslationUnit {
    pub file_id: FileId,
    pub source_path: PathBuf,
    pub context_name: String,
    pub compile_command: CompileCommand,
    /// Plan §3.5 additive artifact field; `#[serde(default)]` keeps old
    /// JSON reading back as `Cpp`.
    #[serde(default)]
    pub language: SourceLanguage,
}

/// Aggregated result of build context discovery across a repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildDiscoveryResult {
    pub repo_root: PathBuf,
    pub contexts: Vec<CompilationContext>,
    pub translation_units: Vec<TranslationUnit>,
}

impl BuildDiscoveryResult {
    pub fn normalize(&mut self) {
        for ctx in &mut self.contexts {
            ctx.normalize();
        }
        self.contexts.sort_by(|a, b| a.name.cmp(&b.name));
        self.contexts.dedup_by(|a, b| a.name == b.name);

        self.translation_units.sort_by(|a, b| {
            a.source_path
                .cmp(&b.source_path)
                .then_with(|| a.context_name.cmp(&b.context_name))
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_context_normalization() {
        let mut ctx = CompilationContext::new("Debug|x64", "cl.exe");
        ctx.defines = vec!["FOO".into(), "DEBUG".into(), "FOO".into()];
        ctx.include_dirs = vec![PathBuf::from("include/b"), PathBuf::from("include/a")];
        ctx.normalize();

        assert_eq!(ctx.defines, vec!["DEBUG", "FOO"]);
        assert_eq!(
            ctx.include_dirs,
            vec![PathBuf::from("include/a"), PathBuf::from("include/b")]
        );
    }

    #[test]
    fn test_build_discovery_result_determinism() {
        let mut ctx1 = CompilationContext::new("Release|x64", "cl.exe");
        ctx1.defines = vec!["NDEBUG".into()];

        let mut ctx2 = CompilationContext::new("Debug|x64", "cl.exe");
        ctx2.defines = vec!["DEBUG".into()];

        let file_path = PathBuf::from("src/main.cpp");
        let file_id = FileId::from_relative_path("src/main.cpp");

        let cmd1 = CompileCommand {
            directory: PathBuf::from("/repo"),
            file: file_path.clone(),
            command: Some("cl.exe /c src/main.cpp".into()),
            arguments: vec!["cl.exe".into(), "/c".into(), "src/main.cpp".into()],
            context_name: "Release|x64".into(),
            output: None,
        };

        let cmd2 = CompileCommand {
            directory: PathBuf::from("/repo"),
            file: file_path.clone(),
            command: Some("cl.exe /c /DDEBUG src/main.cpp".into()),
            arguments: vec![
                "cl.exe".into(),
                "/c".into(),
                "/DDEBUG".into(),
                "src/main.cpp".into(),
            ],
            context_name: "Debug|x64".into(),
            output: None,
        };

        let tu1 = TranslationUnit {
            file_id: file_id.clone(),
            source_path: file_path.clone(),
            context_name: "Release|x64".into(),
            compile_command: cmd1,
            language: SourceLanguage::Cpp,
        };

        let tu2 = TranslationUnit {
            file_id,
            source_path: file_path.clone(),
            context_name: "Debug|x64".into(),
            compile_command: cmd2,
            language: SourceLanguage::Cpp,
        };

        let mut discovery_result = BuildDiscoveryResult {
            repo_root: PathBuf::from("/repo"),
            contexts: vec![ctx1, ctx2],
            translation_units: vec![tu1, tu2],
        };

        discovery_result.normalize();

        assert_eq!(discovery_result.contexts[0].name, "Debug|x64");
        assert_eq!(discovery_result.contexts[1].name, "Release|x64");
        assert_eq!(
            discovery_result.translation_units[0].context_name,
            "Debug|x64"
        );
        assert_eq!(
            discovery_result.translation_units[1].context_name,
            "Release|x64"
        );
    }
}
