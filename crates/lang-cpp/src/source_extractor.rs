//! [`CppSourceExtractor`] — byte-identical C++ new-file text (plan §3.6,
//! M18.8). Reproduces the header bytes the transform engine emitted directly
//! before M18.8: `#pragma once` guard, quoted `#include` lines, blank
//! separator when any include was emitted, then each symbol body plus one
//! newline. The engine historically always set `pragma_once = true`, so the
//! guard is emitted unconditionally here; the symbol target path is chosen
//! by the engine (`unit.path`), so [`GeneratedSourceFile::path`] stays empty.

use ariadne_lang::{GeneratedSourceFile, SourceExtractor, SymbolBody};
use std::path::PathBuf;

/// Generates the C++ text of a newly extracted header.
pub struct CppSourceExtractor;

impl SourceExtractor for CppSourceExtractor {
    fn extract(
        &self,
        symbols: &[SymbolBody],
        _namespace: &str,
        required_deps: &[String],
    ) -> GeneratedSourceFile {
        let mut content = String::new();
        content.push_str("#pragma once\n\n");
        for dep in required_deps {
            content.push_str(&format!("#include \"{}\"\n", dep));
        }
        if !required_deps.is_empty() {
            content.push('\n');
        }
        for sym in symbols {
            content.push_str(&sym.body);
            content.push('\n');
        }
        GeneratedSourceFile {
            path: PathBuf::new(),
            content,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(name: &str, text: &str) -> SymbolBody {
        SymbolBody {
            name: name.to_string(),
            body: text.to_string(),
        }
    }

    #[test]
    fn pragma_guard_and_includes_and_bodies_byte_exact() {
        let file = CppSourceExtractor.extract(
            &[body("MyClass", "// body1")],
            "",
            &["other.h".to_string()],
        );
        assert_eq!(
            file.content,
            "#pragma once\n\n#include \"other.h\"\n\n// body1\n"
        );
    }

    #[test]
    fn no_includes_case_omits_directives_and_blank_line() {
        let file = CppSourceExtractor.extract(&[body("MyFunc", "// body2")], "", &[]);
        assert_eq!(file.content, "#pragma once\n\n// body2\n");
        assert!(!file.content.contains("#include"));
    }

    #[test]
    fn multiple_includes_and_bodies_keep_order() {
        let file = CppSourceExtractor.extract(
            &[body("A", "// a"), body("B", "// b")],
            "",
            &["a.h".to_string(), "b.h".to_string()],
        );
        assert_eq!(
            file.content,
            "#pragma once\n\n#include \"a.h\"\n#include \"b.h\"\n\n// a\n// b\n"
        );
    }

    #[test]
    fn namespace_argument_is_ignored() {
        let with_ns = CppSourceExtractor
            .extract(&[body("A", "// a")], "ns::deep", &[])
            .content;
        let without_ns = CppSourceExtractor
            .extract(&[body("A", "// a")], "", &[])
            .content;
        assert_eq!(with_ns, without_ns);
    }

    #[test]
    fn empty_symbol_list_still_emits_guard() {
        let file = CppSourceExtractor.extract(&[], "", &[]);
        assert_eq!(file.content, "#pragma once\n\n");
    }

    #[test]
    fn plugin_extractor_is_the_real_emitter() {
        use crate::CppPlugin;
        use ariadne_lang::LanguagePlugin;
        let plugin = CppPlugin::new();
        let content = plugin
            .extractor()
            .extract(&[body("A", "// a")], "", &["dep.h".to_string()])
            .content;
        assert_eq!(content, "#pragma once\n\n#include \"dep.h\"\n\n// a\n");
    }
}
