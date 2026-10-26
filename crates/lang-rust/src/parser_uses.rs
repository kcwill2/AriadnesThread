//! `use`-statement extraction for the syn analyzer (rust-language R-5; the
//! shape sec. 7 pre-split, uses half).
//!
//! Every `use` statement yields RAW PATH FACTS: one [`SymbolReference`] per
//! leaf path, carrying the path exactly as WRITTEN plus the 1-based span of
//! the leaf's own name token (globs: the position of the wildcard star). The
//! kind mirrors the Scala analyzer's Incomplete stance - the path target is
//! NOT attributed to a file here; resolution happens later through the R-6
//! resolver (the R-4 registration hop's `use`-edge stance, shape sec. 3.7).
//!
//! Deliberate no-fabrication rules:
//! - renames (`use a::b as c;`) record the ORIGINAL left-side path at the
//!   left name's span - the alias is binding-site information the reference
//!   channel does not claim (Scala's frozen rename grammar made the same
//!   left-side choice at m20);
//! - the `references` channel is the honest carrier: a top-level `use` has
//!   no enclosing symbol, and a `SemanticEdge` would demand inventing one,
//!   which the handoff's "nothing invented" clause forbids.

use crate::parser_items::location_of_span;
use ariadne_lang_api::types::{FileLocation, SymbolId, SymbolReference, TuAnalysis};
use ariadne_repo::FileId;
use syn::spanned::Spanned;
use syn::UseTree;

/// Record every raw path fact of one `use` statement, in source order.
pub(crate) fn record_use(use_item: &syn::ItemUse, file_id: FileId, analysis: &mut TuAnalysis) {
    let absolute_rooted = use_item.leading_colon.is_some();
    collect_tree(
        &use_item.tree,
        &mut Vec::new(),
        absolute_rooted,
        file_id,
        analysis,
    );
}

/// Walk the use tree, carrying the accumulated segments; leaves emit facts.
fn collect_tree(
    tree: &UseTree,
    segments: &mut Vec<String>,
    absolute_rooted: bool,
    file_id: FileId,
    analysis: &mut TuAnalysis,
) {
    match tree {
        UseTree::Path(use_path) => {
            let depth = segments.len();
            segments.push(use_path.ident.to_string());
            collect_tree(&use_path.tree, segments, absolute_rooted, file_id, analysis);
            segments.truncate(depth);
        }
        UseTree::Name(use_name) => {
            let depth = segments.len();
            segments.push(use_name.ident.to_string());
            push_reference(segments, absolute_rooted, use_name.ident.span(), file_id, analysis);
            segments.truncate(depth);
        }
        UseTree::Rename(use_rename) => {
            // The ORIGINAL left-side name is the fact; the alias stays unread.
            let depth = segments.len();
            segments.push(use_rename.ident.to_string());
            push_reference(segments, absolute_rooted, use_rename.ident.span(), file_id, analysis);
            segments.truncate(depth);
        }
        UseTree::Group(use_group) => {
            for item in &use_group.items {
                collect_tree(item, segments, absolute_rooted, file_id, analysis);
            }
        }
        UseTree::Glob(use_glob) => {
            let depth = segments.len();
            segments.push("*".to_string());
            push_reference(
                segments,
                absolute_rooted,
                use_glob.star_token.span(),
                file_id,
                analysis,
            );
            segments.truncate(depth);
        }
    }
}

fn push_reference(
    segments: &[String],
    absolute_rooted: bool,
    name_span: proc_macro2::Span,
    file_id: FileId,
    analysis: &mut TuAnalysis,
) {
    let root = if absolute_rooted { "::" } else { "" };
    let raw_path = format!("{root}{}", segments.join("::"));
    let (line, column) = location_of_span(name_span);
    analysis.references.push(SymbolReference {
        symbol_id: SymbolId::new(raw_path),
        file_id,
        location: FileLocation {
            file_id,
            line,
            column,
        },
        is_write: false,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_surface(source: &str) -> (TuAnalysis, Vec<(String, usize, usize)>) {
        let mut analysis = TuAnalysis::default();
        let parsed = syn::parse_file(source).expect("test source parses");
        for item in &parsed.items {
            if let syn::Item::Use(use_item) = item {
                record_use(use_item, FileId::from_relative_path("src/lib.rs"), &mut analysis);
            }
        }
        let surface = analysis
            .references
            .iter()
            .map(|reference| {
                (
                    reference.symbol_id.0.clone(),
                    reference.location.line,
                    reference.location.column,
                )
            })
            .collect();
        (analysis, surface)
    }

    #[test]
    fn plain_use_yields_the_raw_written_path_at_its_leaf_name() {
        let (analysis, surface) = reference_surface("use crate::models::user::User;\n");
        assert_eq!(
            surface,
            vec![("crate::models::user::User".to_string(), 1, 26)],
            "one fact, spelled exactly as written, at the leaf name's span"
        );
        assert!(analysis.symbols.is_empty(), "a `use` is never a symbol");
        assert!(analysis.declarations.is_empty());
    }

    #[test]
    fn grouped_use_expands_to_one_fact_per_leaf_at_its_own_span() {
        let (_analysis, surface) = reference_surface("use crate::a::{b, c};\n");
        assert_eq!(
            surface,
            vec![
                ("crate::a::b".to_string(), 1, 16),
                ("crate::a::c".to_string(), 1, 19),
            ],
            "group members share the prefix, each sits at its own column"
        );
    }

    #[test]
    fn nested_groups_carry_their_shared_prefixes() {
        let (_analysis, surface) = reference_surface("use a::{b::{c, d}, e};\n");
        assert_eq!(
            surface,
            vec![
                ("a::b::c".to_string(), 1, 13),
                ("a::b::d".to_string(), 1, 16),
                ("a::e".to_string(), 1, 20),
            ]
        );
    }

    #[test]
    fn glob_keeps_the_wildcard_spelling() {
        let (_analysis, surface) = reference_surface("use crate::a::*;\n");
        assert_eq!(
            surface,
            vec![("crate::a::*".to_string(), 1, 15)],
            "the wildcard is never provably unreferenced - it rides the \
             facts as written, positioned at its star"
        );
    }

    #[test]
    fn rename_records_the_original_left_side_only() {
        let (_analysis, surface) = reference_surface("use crate::a::b as c;\n");
        assert_eq!(
            surface,
            vec![("crate::a::b".to_string(), 1, 15)],
            "LOCK: the fact is the ORIGINAL path; the alias binds nowhere \
             in the reference channel"
        );
    }

    #[test]
    fn relative_self_and_absolute_paths_stay_verbatically_raw() {
        let (_analysis, surface) =
            reference_surface("use models::user::User;\nuse self::helper;\nuse ::serde::Serialize;\n");
        assert_eq!(
            surface,
            vec![
                ("models::user::User".to_string(), 1, 19),
                ("self::helper".to_string(), 2, 11),
                ("::serde::Serialize".to_string(), 3, 14),
            ],
            "no rewriting happens here - the resolver decides what a path means"
        );
    }

    #[test]
    fn facts_of_several_statements_stay_in_source_order() {
        let (_analysis, surface) = reference_surface("use z::Last;\nuse a::First;\n");
        assert_eq!(
            surface.iter().map(|fact| fact.0.as_str()).collect::<Vec<&str>>(),
            vec!["z::Last", "a::First"],
            "references keep every occurrence and write-order (load metric \
             consumes them all - Scala's reference discipline)"
        );
    }
}
