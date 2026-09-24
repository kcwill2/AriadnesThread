//! Build and compilation context discovery.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompilationContext {
    pub name: String,
    pub compiler: String,
    pub defines: Vec<String>,
    pub include_dirs: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_context_creation() {
        let ctx = CompilationContext {
            name: "Debug|x64".into(),
            compiler: "cl.exe".into(),
            defines: vec!["DEBUG".into()],
            include_dirs: vec!["include".into()],
        };
        assert_eq!(ctx.name, "Debug|x64");
    }
}
