//! Scala planning surface (M22.1a, decision D24).
//!
//! Mirrors the `lang-cpp` planning split: the aggregation-file filter and
//! the extraction-target naming live here (in the plugin crate), never in
//! the language-neutral cluster engine (plan §7 rule R2).
//!
//! Aggregation rule (plan:421): **only a file that declares a top-level
//! (brace-depth-0) `package object` is an aggregation unit** — a package
//! without one gets no aggregation and the planner invents no fake header.
//! Detection is scanner-backed through the M21 [`crate::scanner`]: comment
//! and string content is removed by lexing, so a commented or quoted
//! `package object` can never fire the predicate (IV1). The file name
//! `all.scala` is a fixture accident, not the rule — the filter reads
//! source, never the name.
//!
//! Path contract: the cluster planner applies the aggregation filter to
//! `IncludeGraph::file_paths` values
//! (`crates/cluster/src/planner.rs` `sorted_candidate_file_ids`), and the
//! graph registers those values straight from `FileRecord::path`
//! (`crates/graph/src/builder.rs` `register_file(file_record.id,
//! file_record.path.clone())`), which repo discovery stores repo-relative
//! with forward-slash normalization (`crates/repo/src/discovery.rs`
//! `strip_prefix(root)` + `norm_rel_str`). So the closure receives
//! **repo-relative** paths; [`scala_aggregation_filter`] captures the repo
//! root and reads `repo_root.join(path)`. An already-absolute path is used
//! as given (tolerance, not the planner's contract).
//!
//! Target naming (IV2): `{sanitized_symbol}.scala` beside the aggregation
//! file, sanitizing with the same discipline as `lang-cpp`'s
//! `sanitize_symbol_stem`. A qualified symbol id loses its package
//! qualifier first — the target lands beside the aggregation file, i.e.
//! inside the package directory, so the prefix would only duplicate the
//! directory it already sits in. The planner passes the anchor's simple
//! `SymbolRecord::name` (`planner.rs` `target_path_for(&candidate.path,
//! &anchor.name)`), so the strip is a no-op on today's real input.

use crate::scanner::{Token, TokenKind};
use std::path::{Path, PathBuf};

/// True when `source` declares a `package object` at brace depth 0.
///
/// Scanner-level by construction (IV1): [`crate::scanner::tokenize`]
/// removes comments and collapses string literals before this sees a
/// token, so neither can ever fire the predicate; and only the adjacent
/// identifier pair `package` `object` at depth 0 counts — a package
/// clause (`package` + qualified name) and a `package x { ... }` block
/// (depth > 0 inside) both miss it. A plain `object` under a package
/// clause likewise never pairs with a `package` token.
pub fn scala_declares_package_object(source: &str) -> bool {
    package_object_pair_at_depth_zero(&crate::scanner::tokenize(source))
}

fn package_object_pair_at_depth_zero(tokens: &[Token]) -> bool {
    let mut depth = 0i32;
    let mut previous_token_is_package_keyword = false;
    for token in tokens {
        match &token.kind {
            TokenKind::Punct('{') => {
                depth += 1;
                previous_token_is_package_keyword = false;
            }
            TokenKind::Punct('}') => {
                depth = (depth - 1).max(0);
                previous_token_is_package_keyword = false;
            }
            TokenKind::Ident(name) => {
                if depth == 0 && previous_token_is_package_keyword && name == "object" {
                    return true;
                }
                previous_token_is_package_keyword = name == "package";
            }
            _ => previous_token_is_package_keyword = false,
        }
    }
    false
}

/// The planner-ready aggregation filter over one captured repository root.
///
/// The closure receives the repo-relative `IncludeGraph::file_paths`
/// values the cluster planner feeds an aggregation filter (see the module
/// docs' path contract) and reads `repo_root.join(path)`; an
/// already-absolute path is used as given. An unreadable file is simply
/// not an aggregation unit — the filter invents nothing.
pub fn scala_aggregation_filter(repo_root: &Path) -> impl Fn(&Path) -> bool + '_ {
    move |candidate_path: &Path| {
        let resolved = if candidate_path.is_absolute() {
            candidate_path.to_path_buf()
        } else {
            repo_root.join(candidate_path)
        };
        std::fs::read_to_string(resolved)
            .is_ok_and(|source| scala_declares_package_object(&source))
    }
}

/// The Scala extraction target: `{sanitized_symbol}.scala` alongside the
/// aggregation file (plan §3.6 "Target file naming", Scala spelling).
///
/// A qualified symbol id is reduced to its simple name first (the target
/// already lives in the package directory); the remaining stem goes
/// through the lang-cpp sanitize discipline (see [`sanitize_symbol_stem`]).
pub fn scala_target_path_for(aggregation_path: &Path, symbol_name: &str) -> PathBuf {
    let simple_name = symbol_name.rsplit('.').next().unwrap_or(symbol_name);
    aggregation_path.with_file_name(format!("{}.scala", sanitize_symbol_stem(simple_name)))
}

/// Turn a symbol name into a filesystem-safe stem — the discipline
/// `lang-cpp`'s private `sanitize_symbol_stem` applies (mirrored here
/// because it is private there and a plugin crate never depends on
/// another language's plugin): ASCII alphanumeric/underscore survive, any
/// other run (`.`, `$`, `[`, `:` …) collapses to one `_`, edge underscores
/// are trimmed, and an all-illegal name falls back to `"symbol"`.
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
    use std::fs;
    use std::path::{Path, PathBuf};

    fn write_tree(root: &Path, files: &[(&str, &str)]) {
        for (rel, content) in files {
            let full = root.join(rel);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, content).unwrap();
        }
    }

    #[test]
    fn aggregation_detects_package_object_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write_tree(
            root,
            &[
                (
                    "src/main/scala/com/example/models/all.scala",
                    "package com.example\n\npackage object models {\n  def order(id: Int): Order = new Order(id)\n}\n",
                ),
                (
                    "src/main/scala/com/example/models/Plain.scala",
                    "package com.example.models\n\nobject models {\n  def helper(): Int = 1\n}\n",
                ),
                (
                    "src/main/scala/com/example/models/Order.scala",
                    "package com.example.models\n\nclass Order(val id: Int)\n",
                ),
                (
                    "src/main/scala/com/example/models/Shape.scala",
                    "package com.example.models\n\ntrait Shape\n",
                ),
                (
                    "src/main/scala/com/example/models/Doc.scala",
                    "package com.example.models\n\nobject Doc {\n  // package object models {}\n  val quoted = \"package object models {}\"\n}\n",
                ),
                (
                    "src/main/scala/com/example/models/Blocked.scala",
                    "package com.example {\n  package object models {}\n}\n",
                ),
            ],
        );
        let filter = scala_aggregation_filter(root);
        assert!(
            filter(Path::new("src/main/scala/com/example/models/all.scala")),
            "a depth-0 `package object` declaration makes the file the package's aggregation unit"
        );
        assert!(
            !filter(Path::new("src/main/scala/com/example/models/Plain.scala")),
            "a plain `object models` under the package clause is not a package object"
        );
        assert!(
            !filter(Path::new("src/main/scala/com/example/models/Order.scala")),
            "a class-only file gets no aggregation"
        );
        assert!(
            !filter(Path::new("src/main/scala/com/example/models/Shape.scala")),
            "a trait-only file gets no aggregation"
        );
        assert!(
            !filter(Path::new("src/main/scala/com/example/models/Doc.scala")),
            "IV1: `package object` inside a line comment or a string literal never fires (scanner-backed)"
        );
        assert!(
            !filter(Path::new("src/main/scala/com/example/models/Blocked.scala")),
            "a `package object` nested inside a `package x {{ ... }}` block is not at depth 0"
        );
        assert!(
            !filter(Path::new("src/main/scala/com/example/models/Missing.scala")),
            "an unreadable path is not an aggregation unit; the filter invents nothing"
        );
        assert!(
            filter(&root.join("src/main/scala/com/example/models/all.scala")),
            "tolerance: an already-absolute path is used as given"
        );
    }

    #[test]
    fn target_path_lands_in_package_dir() {
        assert_eq!(
            scala_target_path_for(
                Path::new("src/main/scala/com/example/models/all.scala"),
                "com.example.models.Order",
            ),
            PathBuf::from("src/main/scala/com/example/models/Order.scala"),
            "IV2: the package qualifier is stripped and the sanitized simple name lands beside the aggregation file"
        );
        assert_eq!(
            scala_target_path_for(Path::new("models/all.scala"), "Order"),
            PathBuf::from("models/Order.scala"),
            "the simple anchor name the planner actually passes (planner.rs target_path_for(&candidate.path, &anchor.name)) passes through untouched"
        );
        assert_eq!(
            scala_target_path_for(Path::new("models/all.scala"), "Foo$"),
            PathBuf::from("models/Foo.scala"),
            "the D17 companion `$` suffix is not a filesystem-safe stem character"
        );
        assert_eq!(
            scala_target_path_for(Path::new("models/all.scala"), "Thing[T]"),
            PathBuf::from("models/Thing_T.scala"),
            "same sanitize discipline as lang-cpp: illegal runs collapse to one underscore"
        );
        assert_eq!(
            scala_target_path_for(Path::new("models/all.scala"), ":::"),
            PathBuf::from("models/symbol.scala"),
            "empty-after-sanitize falls back to `symbol`, like lang-cpp"
        );
    }

    #[test]
    fn pin_sanitize_symbol_stem_edge_collisions() {
        // Multibyte specimens from escapes: authored file stays pure ASCII.
        assert_eq!(sanitize_symbol_stem("\u{fc}ber"), "ber");
        assert_eq!(sanitize_symbol_stem("\u{65e5}\u{672c}\u{8a9e}"), "symbol");
        assert_eq!(
            sanitize_symbol_stem("\u{65e5}\u{672c}\u{8a9e}"),
            sanitize_symbol_stem("\u{d55c}\u{ad00}\u{ae00}"),
            "LOCK (pinned collision): every fully non-ASCII name collapses to the \
             same `symbol` fallback stem, so distinct symbols can collide in the \
             target file name -- current behavior, reported, not fixed"
        );
        assert_eq!(
            sanitize_symbol_stem("\u{fc}ber"),
            "ber",
            "RED-phase finding, pinned honestly: a LEADING non-ASCII run collapses \
             to one underscore which the edge trim then removes -- only ALL-illegal \
             names reach the `symbol` fallback, matching the function's own docs"
        );
        assert_eq!(sanitize_symbol_stem(":::"), "symbol", "operator names empty out");
        assert_eq!(sanitize_symbol_stem("_x_"), "x", "edge underscores are trimmed");
    }
}
