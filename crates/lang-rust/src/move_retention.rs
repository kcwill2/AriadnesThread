//! Rust move-retention hook for `move_out_top_level_symbols`
//! (rust-language R-9a, amendment sec. 3).
//!
//! RETENTION FORM: deletion + declaration insert (Scala parity - the user
//! decision of record is full parity with Scala, whose retention deletes;
//! C++'s marked `ARIADNE-EXTRACT-OUT` form belongs to the
//! extraction-symbol-move decision for C++ headers and is NOT copied here).
//! `plan_id` is ignored: a retention that needs no provenance ignores it
//! (the trait doc's own allowance).
//!
//! Refusals answer `None` so the engine's copy fallback leaves bytes
//! untouched: source unparseable; a requested name locating no top-level
//! item AND no sanitized `mod <name>;` sibling declaration (nothing moved
//! and nothing declared); a name locating only a `mod`-ident item
//! (structure never moves); a re-offer after a completed move (all names
//! absent, all declarations present - the byte-preserving idempotence
//! outcome, C++'s R-B outcome-shape).
//!
//! Work: remove each located top-level item by syn byte-span (attrs
//! included, the `apply_splices` + full-line widening discipline; comments
//! adjacent to an item are NOT item bytes - the R-8 printing caveat,
//! documented); then insert `mod <sanitize_module_stem(name)>;` for every
//! moved name not yet declared - the stem grammar is the SAME function as
//! the extraction-target grammar, one source of truth (`fixture_marker` ->
//! `mod fixture_marker;`), anchored after the LAST existing out-of-line
//! `mod` declaration (visibility/indent copied from it; file-private and
//! end-of-file when none exists), CRLF-aware like the R-8 grafts;
//! already-declared names never gain a second insert.

use crate::dependency_rewriter::line_terminator;
use crate::mod_tree::{
    apply_splices, line_indent_at, line_start_offsets, located_span, sanitize_module_stem, SpliceEdit,
};
use syn::spanned::Spanned;

/// Retention for a completed move: delete the named top-level items and
/// declare their extracted modules (amendment sec. 3). `None` refuses the
/// move and the engine's copy fallback leaves every byte untouched.
pub(crate) fn move_out_retained(source: &str, names: &[String]) -> Option<String> {
    let parsed = syn::parse_file(source).ok()?;
    let located: Vec<LocatedNames<'_>> = names
        .iter()
        .map(|name| LocatedNames {
            name,
            items: top_level_items_named(&parsed, name),
            declaration: mod_declaration_idents(&parsed).into_iter().any(|declared| {
                declared == sanitize_module_stem(name)
            }),
        })
        .collect();
    if refuses_the_move(&located) {
        return None;
    }
    // Deletions never remove a `mod` item (structure never moves), so the
    // declaration set computed on the original parse is also the post-removal set.
    let declared_stems: Vec<String> = mod_declaration_idents(&parsed);
    let removal = remove_items_by_span(source, &located)?;
    let moved_declarations: Vec<String> = located
        .iter()
        .filter(|entry| !entry.items.is_empty())
        .map(|entry| sanitize_module_stem(entry.name))
        .collect();
    let to_insert: Vec<String> = moved_declarations
        .into_iter()
        .filter(|stem| !declared_stems.iter().any(|declared| declared == stem))
        .collect();
    if to_insert.is_empty() {
        return Some(removal);
    }
    let reparsed = syn::parse_file(&removal).ok()?;
    Some(declare_modules_at(&removal, &reparsed, &to_insert))
}

/// A requested name with its located top-level items and whether the file
/// already declares the sanitized sibling `mod` declaration.
struct LocatedNames<'a> {
    name: &'a String,
    items: Vec<&'a syn::Item>,
    declaration: bool,
}

/// The honest refusals (amendment sec. 3), each answering `None`:
/// - a name locating no item AND no sibling declaration: nothing would
///   move and nothing needs declaring - honest refusal (not a no-op `Some`);
/// - a name locating only `mod`-ident items: structure never moves;
/// - a re-offer after a completed move: every item absent, every
///   declaration present - byte-preserving (the caller keeps the bytes).
/// An empty name batch is refused by the completed-move clause itself (the
/// all-quantifiers are vacuously true): nothing to move, nothing to keep.
fn refuses_the_move(located: &[LocatedNames<'_>]) -> bool {
    located.iter().any(|entry| {
        (!entry.items.is_empty() && entry.items.iter().all(|item| matches!(item, syn::Item::Mod(_))))
            || (entry.items.is_empty() && !entry.declaration)
    }) || located.iter().all(|entry| entry.items.is_empty() && entry.declaration)
}

/// The top-level items whose identifier equals `name` (items without an
/// identifier - `use`, `impl`, `const` - are not locatable move candidates).
fn top_level_items_named<'a>(parsed: &'a syn::File, name: &str) -> Vec<&'a syn::Item> {
    let ident_of: fn(&syn::Item) -> Option<&syn::Ident> = |item| match item {
        syn::Item::Fn(item_fn) => Some(&item_fn.sig.ident),
        syn::Item::Struct(item_struct) => Some(&item_struct.ident),
        syn::Item::Enum(item_enum) => Some(&item_enum.ident),
        syn::Item::Trait(item_trait) => Some(&item_trait.ident),
        syn::Item::Mod(item_mod) => Some(&item_mod.ident),
        syn::Item::Type(item_type) => Some(&item_type.ident),
        syn::Item::Union(item_union) => Some(&item_union.ident),
        _ => None,
    };
    parsed
        .items
        .iter()
        .filter(|item| ident_of(item).is_some_and(|ident| ident == name))
        .collect()
}

fn mod_declaration_idents(parsed: &syn::File) -> Vec<String> {
    parsed
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Mod(item_mod) => Some(item_mod.ident.to_string()),
            _ => None,
        })
        .collect()
}

/// Delete each located NON-mod item through its syn byte span (attrs
/// included; the `apply_splices` full-line widening removes the whole lines;
/// adjacent comments are not item bytes and stay - documented R-8 caveat).
/// An unresolvable span refuses the whole move: an unlocated deletion is
/// never attempted.
fn remove_items_by_span(source: &str, located: &[LocatedNames<'_>]) -> Option<String> {
    let line_starts = line_start_offsets(source);
    let mut edits: Vec<SpliceEdit> = Vec::new();
    for entry in located {
        for item in &entry.items {
            if matches!(**item, syn::Item::Mod(_)) {
                continue;
            }
            let (_start, _end, from, to) = located_span(source, &line_starts, item.span())?;
            edits.push(SpliceEdit { from, to, replacement: String::new() });
        }
    }
    Some(apply_splices(source, edits))
}

/// Declare the extracted modules: anchored after the LAST existing
/// out-of-line `mod` declaration, copying its visibility and indent,
/// CRLF-aware; with no out-of-line declaration the lines land at the file
/// end, file-private. Already-declared stems are filtered by the caller.
fn declare_modules_at(content: &str, parsed_after: &syn::File, declarations: &[String]) -> String {
    let anchor: Option<&syn::ItemMod> = parsed_after.items.iter().rev().find_map(|item| {
        match item {
            syn::Item::Mod(item_mod) if item_mod.content.is_none() => Some(item_mod),
            _ => None,
        }
    });
    let Some(anchor) = anchor else {
        return append_file_private_at_end(content, declarations);
    };
    let line_starts = line_start_offsets(content);
    let Some((_start, _end, from, to)) = located_span(content, &line_starts, anchor.span()) else {
        return append_file_private_at_end(content, declarations);
    };
    let visibility =
        matches!(anchor.vis, syn::Visibility::Public(_)).then_some("pub ").unwrap_or("");
    let indent = line_indent_at(content, from);
    let terminator = line_terminator(content, to);
    let mut block = String::new();
    for stem in declarations {
        block.push_str(&format!("{indent}{visibility}mod {stem};{terminator}"));
    }
    let insert_pos = match content[to..].find('\n') {
        Some(offset) => to + offset + 1,
        None => content.len(),
    };
    let dangling_newline =
        (insert_pos == content.len() && !content.ends_with('\n')).then_some(terminator).unwrap_or_default();
    format!("{}{dangling_newline}{block}{}", &content[..insert_pos], &content[insert_pos..])
}

/// The no-anchor fallback: the declarations land at the file end,
/// file-private, riding the terminator the file ends with (or `\n`).
fn append_file_private_at_end(content: &str, declarations: &[String]) -> String {
    let terminator = if content.ends_with("\r\n") { "\r\n" } else { "\n" };
    let dangling_newline =
        (!content.is_empty() && !content.ends_with('\n')).then_some(terminator).unwrap_or_default();
    let mut block = String::new();
    for stem in declarations {
        block.push_str(&format!("mod {stem};{terminator}"));
    }
    format!("{content}{dangling_newline}{block}")
}

#[cfg(test)]
mod tests {
    use crate::dependency_rewriter::RustDependencyRewriter;
    use ariadne_lang::DependencyRewriter;

    fn move_out(source: &str, names: &[&str]) -> Option<String> {
        let qualified: Vec<String> = names.iter().map(|name| name.to_string()).collect();
        RustDependencyRewriter.move_out_top_level_symbols(source, &qualified, "PLAN-R9A")
    }

    /// Amendment sec. 1b red 6, leg 1: the item bytes leave (attrs ride the
    /// span), the declaration lands ONCE after the last out-of-line `mod`
    /// in that anchor's style; everything else is byte-identical.
    #[test]
    fn move_out_removes_the_item_and_declares_the_module_once() {
        let source = "pub mod greeting;\npub mod models;\n\nfn fixture_marker() -> u32 {\n    42\n}\n\npub fn keeper() {}\n";
        let moved = move_out(source, &["fixture_marker"])
            .expect("a locatable top-level fn with a mod anchor must be retained by deletion");
        assert_eq!(
            moved,
            "pub mod greeting;\npub mod models;\npub mod fixture_marker;\n\n\npub fn keeper() {}\n",
            "byte-exact: the fn bytes are gone, the declaration lands once \
             after the LAST out-of-line mod, its `pub` visibility copied"
        );
        // Attrs ride the item span: the attribute lines leave with the item.
        let attrs = "#[derive(Clone)]\nstruct Marker;\n\nfn keeper() {}\n";
        let moved_attrs = move_out(attrs, &["Marker"])
            .expect("attrs are item bytes and must leave with the item");
        assert_eq!(
            moved_attrs,
            "\nfn keeper() {}\nmod Marker;\n",
            "no out-of-line mod anchor exists: the declaration lands at the \
             file end, file-private (no visibility), attrs removed with the item"
        );
        // CRLF specimen (RISK-R9-HOOK-SPICE oracle): removal and insert
        // keep the terminator grammar.
        let crlf = "pub mod a;\r\nfn moved() {}\r\n";
        assert_eq!(
            move_out(crlf, &["moved"]),
            Some("pub mod a;\r\npub mod moved;\r\n".to_string()),
            "CRLF-aware like the R-8 grafts: the inserted line rides the \
             file's \\r\\n terminator"
        );
    }

    /// Amendment sec. 1b red 6, leg 2: re-applying the retention after a
    /// completed move is byte-preserving (None -> copy fallback keeps the
    /// bytes), and an already-declared name never gains a second insert.
    #[test]
    fn move_out_reinsert_is_idempotent() {
        let source = "mod greeting;\n\nfn marker() {}\n";
        let once = move_out(source, &["marker"]).expect("the clean variant moves");
        assert_eq!(
            once, "mod greeting;\nmod marker;\n\n",
            "first application: deletion + the single declaration"
        );
        assert_eq!(
            move_out(&once, &["marker"]),
            None,
            "re-offer after a completed move answers None: the engine's \
             copy-fallback leaves the bytes untouched (byte-preserving \
             idempotence, C++'s R-B outcome-shape)"
        );
        assert_eq!(
            move_out("mod greeting;\nmod marker;\n\nfn marker() {}\n", &["marker"]),
            Some("mod greeting;\nmod marker;\n\n".to_string()),
            "already declared -> the item is removed and NO second insert \
             happens (byte-identical pin)"
        );
    }

    /// Amendment sec. 1b red 6, leg 3: honest refusals - an unparseable
    /// source, and a name locating neither an item nor a sibling
    /// declaration. Paired clean variant first (red non-vacuity).
    #[test]
    fn move_out_refuses_unparseable_or_unlocatable() {
        let clean = "mod greeting;\n\nfn marker() {}\n";
        assert!(
            move_out(clean, &["marker"]).is_some(),
            "the paired clean variant moves (red non-vacuity)"
        );
        assert_eq!(
            move_out("fn broken( {{{\n", &["marker"]),
            None,
            "unparseable source: no syn tree, no certified spans - honest \
             refusal, the engine proceeds without the move"
        );
        assert_eq!(
            move_out(clean, &["ghost"]),
            None,
            "no top-level item AND no `mod ghost;` sibling declaration: \
             nothing moved and nothing declared - honest refusal"
        );
    }

    /// Amendment sec. 1b red 6, leg 4: structure never moves (a name
    /// locating only a `mod`-ident item refuses, inline or out-of-line),
    /// and a fully completed move refuses the re-offer. Paired clean
    /// variant first (red non-vacuity).
    #[test]
    fn move_out_refuses_module_ident_and_already_moved() {
        let clean = "mod greeting;\n\nfn marker() {}\n";
        assert!(
            move_out(clean, &["marker"]).is_some(),
            "the paired clean variant moves (red non-vacuity)"
        );
        assert_eq!(
            move_out("mod hidden;\n\nfn keeper() {}\n", &["hidden"]),
            None,
            "a name locating only an out-of-line `mod` declaration is \
             structure, never a move candidate"
        );
        assert_eq!(
            move_out("mod hidden {\n    fn deep() {}\n}\n", &["hidden"]),
            None,
            "an inline `mod` block with the ident is equally untouchable"
        );
        let moved = move_out(clean, &["marker"]).expect("the paired clean variant moves");
        assert_eq!(
            move_out(&moved, &["marker"]),
            None,
            "all names absent, all declarations present: the completed move \
             answers None (the already-moved refusal)"
        );
    }
}
