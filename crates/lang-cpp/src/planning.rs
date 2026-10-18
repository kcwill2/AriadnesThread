//! C++ planner naming (plan M18.6 / §3.6 "Target file naming").
//!
//! The aggregation-file filter and the `{symbol}_extracted.h` extraction
//! target template moved verbatim out of `ariadne-cluster`'s planner so the
//! planning engine names no languages or extensions (plan §7 rule R2).
//! Behaviour is byte-identical to the cluster code it replaces, locked by
//! `crates/cluster/tests/planner_characterization.rs`.

use std::path::{Path, PathBuf};

/// C++ aggregation units are headers: the `h`/`hpp`/`hxx`/`hh`/`inl`
/// extension set (moved verbatim from
/// `ariadne_cluster::planner::is_header_file`).
pub fn cpp_aggregation_filter(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext, "h" | "hpp" | "hxx" | "hh" | "inl"))
        .unwrap_or(false)
}

/// The C++ extraction target: `{sanitized_symbol}_extracted.h` alongside the
/// aggregation file (template and sanitizer moved verbatim from the cluster
/// planner).
pub fn cpp_target_path_for(aggregation_path: &Path, symbol_name: &str) -> PathBuf {
    aggregation_path
        .with_file_name(format!("{}_extracted.h", sanitize_symbol_stem(symbol_name)))
}

/// Turn a symbol name into a filesystem-safe header stem. Closure-merged
/// models surface qualified/template names (`std::vector`, `Thing<T>`) that
/// are invalid in file names on Windows.
fn sanitize_symbol_stem(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut pending_underscore = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            if pending_underscore {
                out.push('_');
                pending_underscore = false;
            }
            out.push(ch);
        } else {
            pending_underscore = true;
        }
    }
    let stem = out.trim_matches('_').to_string();
    if stem.is_empty() {
        "symbol".to_string()
    } else {
        stem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregation_filter_accepts_the_header_extension_set() {
        for name in ["a.h", "a.hpp", "a.hxx", "a.hh", "a.inl"] {
            assert!(cpp_aggregation_filter(Path::new(name)), "{name} is a header");
        }
        for name in ["a.cpp", "a.cc", "a.c", "a.scala", "a.agg", "noext", "a.H"] {
            assert!(
                !cpp_aggregation_filter(Path::new(name)),
                "{name} is not a header"
            );
        }
    }

    #[test]
    fn target_path_template_places_extracted_header_beside_the_aggregation() {
        assert_eq!(
            cpp_target_path_for(Path::new("src/pkg.h"), "Widget"),
            PathBuf::from("src/Widget_extracted.h")
        );
    }

    #[test]
    fn sanitizer_locks_the_moved_stem_behaviour() {
        let cases = [
            ("std::vector", "std_vector"),
            ("Thing<T>", "Thing_T"),
            (":::", "symbol"),
            ("A_B", "A_B"),
            ("A::B", "A_B"),
        ];
        for (name, stem) in cases {
            assert_eq!(sanitize_symbol_stem(name), stem, "stem for {name:?}");
        }
    }
}

#[cfg(test)]
mod sanitize_pin_tests;
