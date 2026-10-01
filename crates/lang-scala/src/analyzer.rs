//! [`ScalaAnalyzer`] — the thin `SemanticAnalyzer` wrapper over the M21
//! [`crate::parser::ScalaSourceParser`] (kickoff D18: "analyzer is a thin
//! wrapper"). The parser owns `tu_file_id` and everything derivable from
//! `source`; this adapter only stamps the two fields the parser cannot know:
//! the build `context` and (defensively, for the `tu_store` index-0 language
//! rule) the `language`. The analyzer is stateless, hence `Send + Sync` as
//! the trait requires.

use crate::parser::ScalaSourceParser;
use ariadne_lang::{SemanticAnalyzer, TuAnalysis};
use ariadne_repo::{FileId, SourceLanguage};

/// Stateless Scala analysis entry point wired into
/// [`crate::plugin::ScalaPlugin`] as of M21; replaces the placeholder
/// analyzer stub retired in the same commit.
pub struct ScalaAnalyzer {
    parser: ScalaSourceParser,
}

impl ScalaAnalyzer {
    pub fn new() -> Self {
        Self {
            parser: ScalaSourceParser,
        }
    }
}

impl Default for ScalaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticAnalyzer for ScalaAnalyzer {
    fn analyze(&self, file_id: FileId, source: &str, context: &str) -> TuAnalysis {
        let mut analysis = self.parser.parse(file_id, std::path::Path::new(""), source);
        analysis.context_name = context.to_string();
        analysis.language = SourceLanguage::Scala;
        analysis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyzer_output(source: &str, context: &str) -> (FileId, TuAnalysis) {
        let file_id = FileId::from_relative_path("src/main/scala/Foo.scala");
        (file_id, ScalaAnalyzer::new().analyze(file_id, source, context))
    }

    fn symbol_ids(analysis: &TuAnalysis) -> Vec<String> {
        analysis
            .symbols
            .iter()
            .map(|symbol| symbol.id.0.clone())
            .collect()
    }

    #[test]
    fn analyze_stamps_scala_language_context_and_keeps_parser_output() {
        let (file_id, analysis) = analyzer_output("class Foo {}", "root");
        assert_eq!(analysis.tu_file_id, file_id, "parser owns tu_file_id");
        assert_eq!(analysis.language, SourceLanguage::Scala);
        assert_eq!(analysis.context_name, "root");
        assert_eq!(symbol_ids(&analysis), vec!["Foo"], "symbols non-empty");
    }

    #[test]
    fn analyzer_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ScalaAnalyzer>();
    }
}
