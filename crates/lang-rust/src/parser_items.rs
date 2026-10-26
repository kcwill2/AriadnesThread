//! Item-to-symbol walking for the syn analyzer (rust-language R-5; the
//! shape sec. 7 pre-split, analyzer half).
//!
//! Every WRITTEN top-level item of the contract set (fn/struct/enum/trait/
//! impl/type/const/static/mod) yields exactly one [`SymbolRecord`] plus one
//! [`SymbolDecl`]. Qualified ids are `crate::<path>::<Name>` where `<path>`
//! is the `mod`-chain VISIBLE IN THE FILE (inline `mod x { ... }` nesting);
//! the analyzer seam receives only a hashed [`FileId`] plus the build-context
//! name (verified at `scheduler/src/worker.rs:95` and
//! `worker-core/src/tu_store.rs:208`), so the TU's position in the crate
//! module tree is not an input and is never invented - cross-file module-tree
//! truth is R-6's resolver job (its `resolve_edges` does receive `rel_path`).
//!
//! Kind mapping (the neutral `SymbolKind` vocabulary, Scala-parity naming
//! decisions): fn -> `Function`, struct -> `Struct`, enum -> `Enum`,
//! trait -> `Trait`, type alias -> `TypeAlias`, const/static -> `Variable`,
//! mod -> `Module`, impl -> `Class` (the method-holder kind; id carries the
//! written form `impl <Type>` / `impl <Trait> for <Type>`).
//!
//! Honest absences (each is a deliberate no-record, never a fabricated
//! surface): `Item::Use` rides [`crate::parser_uses`]; `Item::Union`,
//! `Item::Macro`/`Item::Macro2`
//! (the D-2 macro caveat), `Item::ForeignMod`, `Item::Verbatim`, and impl
//! targets without a nameable path record nothing; methods of an impl and
//! trait-function signatures are not top-level items of the contract set.

use ariadne_lang_api::types::{
    DeclId, FileLocation, SymbolDecl, SymbolId, SymbolKind, SymbolRecord, TuAnalysis,
};
use ariadne_repo::FileId;
use syn::spanned::Spanned;

/// Root of every qualified id: the crate the analyzed file belongs to.
pub(crate) const ROOT_PREFIX: &str = "crate::";

/// The written items declared inside an inline `mod x { ... }` block, with
/// the child module prefix the walker descends to.
pub(crate) struct InlineModule<'a> {
    pub prefix: String,
    pub items: &'a [syn::Item],
}

/// Record one item and, when it is an inline `mod`, hand its written content
/// back to the walker. Source order is preserved: one record per written
/// item, one declaration per record, `definition` points at it.
pub(crate) fn record_item<'a>(
    item: &'a syn::Item,
    module_prefix: &str,
    file_id: FileId,
    analysis: &mut TuAnalysis,
) -> Option<InlineModule<'a>> {
    let name = item_written_name(item)?;
    let kind = item_kind(item);
    let qualified = format!("{module_prefix}{name}");
    let (line, column) = item_location(item);
    record_symbol(
        analysis,
        &qualified,
        name,
        kind,
        file_id,
        line,
        column,
    );
    match item {
        syn::Item::Mod(item_mod) => item_mod.content.as_ref().map(|(_brace, items)| InlineModule {
            prefix: inline_module_prefix(module_prefix, &item_mod.ident.to_string()),
            items: items.as_slice(),
        }),
        _ => None,
    }
}

fn inline_module_prefix(module_prefix: &str, module_name: &str) -> String {
    format!("{module_prefix}{module_name}::")
}

/// The written name of a contract-set item; `None` marks an honest absence.
fn item_written_name(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Fn(item_fn) => Some(item_fn.sig.ident.to_string()),
        syn::Item::Struct(item_struct) => Some(item_struct.ident.to_string()),
        syn::Item::Enum(item_enum) => Some(item_enum.ident.to_string()),
        syn::Item::Trait(item_trait) => Some(item_trait.ident.to_string()),
        syn::Item::Type(item_type) => Some(item_type.ident.to_string()),
        syn::Item::Const(item_const) => Some(item_const.ident.to_string()),
        syn::Item::Static(item_static) => Some(item_static.ident.to_string()),
        syn::Item::Mod(item_mod) => Some(item_mod.ident.to_string()),
        syn::Item::Impl(item_impl) => impl_written_name(item_impl),
        _ => None,
    }
}

/// The written `impl` form: `impl <Type>` or `impl <Trait> for <Type>`.
/// An impl whose target has no plain path (a bare generic, an array, a
/// macro) is not nameable without a printing dependency and records nothing.
fn impl_written_name(item_impl: &syn::ItemImpl) -> Option<String> {
    let type_name = match &*item_impl.self_ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        _ => None,
    }?;
    Some(match &item_impl.trait_ {
        Some((_not, trait_path, _for)) => {
            let trait_name = trait_path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<String>>()
                .join("::");
            format!("impl {trait_name} for {type_name}")
        }
        None => format!("impl {type_name}"),
    })
}

fn item_kind(item: &syn::Item) -> SymbolKind {
    match item {
        syn::Item::Fn(_) => SymbolKind::Function,
        syn::Item::Struct(_) => SymbolKind::Struct,
        syn::Item::Enum(_) => SymbolKind::Enum,
        syn::Item::Trait(_) => SymbolKind::Trait,
        syn::Item::Type(_) => SymbolKind::TypeAlias,
        syn::Item::Const(_) | syn::Item::Static(_) => SymbolKind::Variable,
        syn::Item::Mod(_) => SymbolKind::Module,
        syn::Item::Impl(_) => SymbolKind::Class,
        _ => SymbolKind::Other,
    }
}

/// INV-7 span discipline: named items sit at their NAME token; the impl
/// block (no name ident) at its span start. 1-based line and 1-based column
/// (`proc_macro2` `Span::start().column` is 0-based; both shipped tokenizer
/// conventions are 1-based - the `+ 1` is the pinned reconciliation).
fn item_location(item: &syn::Item) -> (usize, usize) {
    let span = match item {
        syn::Item::Fn(item_fn) => item_fn.sig.ident.span(),
        syn::Item::Struct(item_struct) => item_struct.ident.span(),
        syn::Item::Enum(item_enum) => item_enum.ident.span(),
        syn::Item::Trait(item_trait) => item_trait.ident.span(),
        syn::Item::Type(item_type) => item_type.ident.span(),
        syn::Item::Const(item_const) => item_const.ident.span(),
        syn::Item::Static(item_static) => item_static.ident.span(),
        syn::Item::Mod(item_mod) => item_mod.ident.span(),
        other => other.span(),
    };
    location_of_span(span)
}

/// Shared with `parser_uses`: 0-based `proc_macro2` column in, 1-based out.
pub(crate) fn location_of_span(span: proc_macro2::Span) -> (usize, usize) {
    (span.start().line, span.start().column + 1)
}

/// One symbol + its single definition-bearing declaration (the `DeclId`
/// idiom mirrors Scala at the same seam: `<qualified-id>#L<line>`).
fn record_symbol(
    analysis: &mut TuAnalysis,
    qualified: &str,
    name: String,
    kind: SymbolKind,
    file_id: FileId,
    line: usize,
    column: usize,
) {
    let decl_id = DeclId::new(format!("{qualified}#L{line}"));
    analysis.symbols.push(SymbolRecord {
        id: SymbolId::new(qualified),
        name,
        kind,
        owning_file: file_id,
        declarations: vec![decl_id.clone()],
        definition: Some(decl_id.clone()),
        modifiers: Vec::new(),
    });
    analysis.declarations.push(SymbolDecl {
        id: decl_id,
        symbol_id: SymbolId::new(qualified),
        location: FileLocation {
            file_id,
            line,
            column,
        },
        is_definition: true,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser_uses;

    fn surface(analysis: &TuAnalysis) -> Vec<(String, SymbolKind)> {
        analysis
            .symbols
            .iter()
            .map(|record| (record.id.0.clone(), record.kind))
            .collect()
    }

    fn recorded(source: &str) -> TuAnalysis {
        let mut analysis = TuAnalysis::default();
        let parsed = syn::parse_file(source).expect("test source parses");
        record_walk(&parsed.items, ROOT_PREFIX, &mut analysis);
        analysis
    }

    /// Mirrors `analyzer::walk_items` so the unit pins see the same descent.
    fn record_walk(items: &[syn::Item], module_prefix: &str, analysis: &mut TuAnalysis) {
        let file_id = FileId::from_relative_path("src/lib.rs");
        for item in items {
            match item {
                syn::Item::Use(use_item) => parser_uses::record_use(use_item, file_id, analysis),
                _ => {
                    if let Some(inline_module) = record_item(item, module_prefix, file_id, analysis)
                    {
                        record_walk(inline_module.items, &inline_module.prefix, analysis);
                    }
                }
            }
        }
    }

    #[test]
    fn fn_struct_enum_trait_map_to_their_written_kinds() {
        let analysis = recorded(
            "pub fn alpha() {}\npub struct Beta { pub field: u32 }\nenum Gamma {}\ntrait Delta {}\n",
        );
        assert_eq!(
            surface(&analysis),
            vec![
                ("crate::alpha".to_string(), SymbolKind::Function),
                ("crate::Beta".to_string(), SymbolKind::Struct),
                ("crate::Gamma".to_string(), SymbolKind::Enum),
                ("crate::Delta".to_string(), SymbolKind::Trait),
            ]
        );
    }

    #[test]
    fn const_static_and_type_alias_are_recorded() {
        let analysis = recorded("const MAX: u32 = 1;\nstatic COUNT: u32 = 0;\ntype Alias = u32;\n");
        assert_eq!(
            surface(&analysis),
            vec![
                ("crate::MAX".to_string(), SymbolKind::Variable),
                ("crate::COUNT".to_string(), SymbolKind::Variable),
                ("crate::Alias".to_string(), SymbolKind::TypeAlias),
            ]
        );
    }

    #[test]
    fn out_of_line_mod_declares_a_module_record_at_its_name_span() {
        let analysis = recorded("mod sibling;\n");
        assert_eq!(
            surface(&analysis),
            vec![("crate::sibling".to_string(), SymbolKind::Module)]
        );
        assert_eq!(
            analysis.declarations[0].location.line, 1, "written line (1-based)"
        );
        assert_eq!(
            analysis.declarations[0].location.column, 5, "`sibling` starts at 1-based column 5"
        );
    }

    #[test]
    fn inline_mod_nests_written_items_under_its_own_prefix() {
        let analysis = recorded("mod inner {\n    pub fn helper() {}\n}\n");
        assert_eq!(
            surface(&analysis),
            vec![
                ("crate::inner".to_string(), SymbolKind::Module),
                ("crate::inner::helper".to_string(), SymbolKind::Function),
            ],
            "inline content is written code: the walker descends one level"
        );
    }

    #[test]
    fn impl_blocks_carry_their_written_form() {
        let analysis = recorded("impl Gamma {}\ntrait Delta {}\nimpl Delta for Gamma {}\n");
        assert_eq!(
            surface(&analysis),
            vec![
                ("crate::impl Gamma".to_string(), SymbolKind::Class),
                ("crate::Delta".to_string(), SymbolKind::Trait),
                ("crate::impl Delta for Gamma".to_string(), SymbolKind::Class),
            ],
            "an impl is one Class-kind record with the written form as name; \
             its methods are not top-level items and are never listed here"
        );
        assert!(
            analysis.declarations.iter().all(|decl| decl.location.line > 0),
            "impl declarations still carry real span coordinates"
        );
    }

    #[test]
    fn macro_items_record_nothing_and_absorb_no_names() {
        let analysis = recorded(
            "macro_rules! m { () => { fn ghost() {} }; }\nm!();\npub fn real() {}\n",
        );
        assert_eq!(
            surface(&analysis),
            vec![("crate::real".to_string(), SymbolKind::Function)],
            "D-2: neither the macro definition nor an invocation may name \
             what only macro expansion would create"
        );
    }

    #[test]
    fn union_and_use_are_not_symbol_items() {
        let analysis = recorded("union U { a: u32 }\nuse x::Y;\n");
        assert_eq!(surface(&analysis), Vec::new(), "union is outside the contract item set");
        assert_eq!(
            analysis.references.len(),
            1,
            "the `use` fact leaves through the references channel, not symbols"
        );
    }

    #[test]
    fn record_and_declaration_are_wired_together() {
        let analysis = recorded("pub fn alpha() {}\n");
        let record = &analysis.symbols[0];
        let decl = &analysis.declarations[0];
        assert_eq!(record.declarations, vec![decl.id.clone()]);
        assert_eq!(record.definition.as_ref(), Some(&decl.id));
        assert_eq!(decl.id.0, "crate::alpha#L1", "Scala-seam DeclId idiom");
        assert!(decl.is_definition, "written items are definitions");
        assert!(record.modifiers.is_empty(), "no modifier words are recorded at this hop");
    }
}
