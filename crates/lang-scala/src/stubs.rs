//! Placeholder pipeline components owned by later milestones (M19.1).
//!
//! Capability gating lives in the pipeline, not here: the literals of
//! [`crate::plugin::ScalaPlugin::capabilities`] keep `planning`,
//! `extraction`, `rewriting`, and `validation` switched off, so calling
//! the extractor/rewriter/validator stubs before M23 is a capability-gated
//! error before it could ever reach these bodies. The analyzer and resolver
//! stubs sit behind capabilities that are already `true`: until their
//! milestones land, the pipeline legitimately receives empty results from
//! them, which is why each stub returns an honest empty value rather than
//! failing loudly.

use ariadne_lang::{
    ClusteringConstraints, CompilerValidator, DependencyResolver, DependencyRewriter,
    GeneratedSourceFile, RawDepEdge, ResolutionContext, RewriteSpec, SemanticAnalyzer,
    SourceExtractor, SymbolBody, SymbolId, SymbolRecord, TuAnalysis,
};
use ariadne_repo::{FileId, SourceLanguage};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Reports an empty [`TuAnalysis`] until the M21 Scala parser replaces it;
/// until M21 the semantic index legitimately holds zero Scala symbols, so
/// this stays an empty result, never a parser error.
pub struct StubAnalyzer;

impl SemanticAnalyzer for StubAnalyzer {
    fn analyze(&self, file_id: FileId, _source: &str, _context: &str) -> TuAnalysis {
        TuAnalysis {
            tu_file_id: file_id,
            language: SourceLanguage::Scala,
            ..TuAnalysis::default()
        }
    }
}

/// Reports no dependency edges until the M20 Scala resolver replaces it;
/// the dependency graph legitimately holds no Scala edges until then.
pub struct StubResolver;

impl DependencyResolver for StubResolver {
    fn resolve_edges(
        &self,
        _file_id: FileId,
        _path: &Path,
        _source: &str,
        _context: &ResolutionContext<'_>,
    ) -> Vec<RawDepEdge> {
        vec![]
    }
}

/// Scala has no atomic symbol groups and no language-level frozen symbols
/// until its clustering milestone; the policy layer owns freezing (plan
/// sec 3.4, same stance as C++).
pub struct StubConstraints;

impl ClusteringConstraints for StubConstraints {
    fn colocated_groups(&self, _symbols: &[SymbolRecord]) -> Vec<Vec<SymbolId>> {
        Vec::new()
    }

    fn frozen_symbols(&self, _symbols: &[SymbolRecord]) -> HashSet<SymbolId> {
        HashSet::new()
    }
}

/// Empty-source placeholder until the M23 Scala extractor replaces it;
/// `extraction: false` gates the pipeline away from this body until then.
pub struct StubExtractor;

impl SourceExtractor for StubExtractor {
    fn extract(
        &self,
        _bodies: &[SymbolBody],
        _namespace: &str,
        _dependencies: &[String],
    ) -> GeneratedSourceFile {
        GeneratedSourceFile {
            path: PathBuf::from("out.scala"),
            content: String::new(),
        }
    }
}

/// Source passthrough until the M23 Scala rewriter replaces it;
/// `rewriting: false` gates the pipeline away from this body until then.
pub struct StubRewriter;

impl DependencyRewriter for StubRewriter {
    fn rewrite(&self, source: &str, _spec: &RewriteSpec) -> String {
        source.to_string()
    }
}

/// Reports success without compiling until the M23 Scala validator
/// replaces it; `validation: false` gates the pipeline away from this body
/// until then. The trait-default `toolchain_status()` (`Unknown`) is kept:
/// no Scala toolchain probe exists before M23.
pub struct StubValidator;

impl CompilerValidator for StubValidator {
    fn validate_compile(&self, _files: &[PathBuf], _args: &[String]) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_analyzer_reports_an_empty_scala_analysis() {
        let file_id = FileId::from_relative_path("src/main/scala/com/example/Main.scala");
        let analysis = StubAnalyzer.analyze(file_id, "object Main {}", "root");
        assert_eq!(analysis.tu_file_id, file_id);
        assert_eq!(analysis.language, SourceLanguage::Scala);
        assert!(analysis.symbols.is_empty());
        assert!(analysis.declarations.is_empty());
        assert!(analysis.edges.is_empty());
    }

    #[test]
    fn stub_rewriter_passes_source_through() {
        let source = "object Main extends App {}";
        let spec = RewriteSpec {
            old_deps: vec!["com.example.orders".to_string()],
            new_deps: vec!["com.example.orders.model".to_string()],
        };
        assert_eq!(StubRewriter.rewrite(source, &spec), source);
    }

    #[test]
    fn stub_validator_reports_ok_until_m23() {
        assert!(StubValidator
            .validate_compile(&[], &[])
            .is_ok());
    }
}
