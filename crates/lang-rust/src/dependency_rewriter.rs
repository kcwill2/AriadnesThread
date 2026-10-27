//! [`RustDependencyRewriter`] - Rust `use`-path rewriting plus the
//! RISK-MODTREE parent mechanism (rust-language R-8).
//!
//! One grammar split carries the contract (shape sec. 3.7, sec. 6): a
//! dependency spelling that is a bare single identifier is MOD-space (it
//! names a `mod <name>;` declaration); any multi-segment spelling
//! (`crate::a::B`, `::serde::Serialize`) is USE-space (it names a `use`
//! leaf). One [`RewriteSpec`] carries both spaces; each file applies the
//! half it has.
//!
//! - [`DependencyRewriter::rewrite`] maps `use` statements whose leaf path
//!   EQUALS an old dep (all-occurrence discipline, `syn`-backed so
//!   commented/quoted spellings are structurally unreachable): a touched
//!   statement expands to one line per new dep at the first matched slot
//!   plus survivors in source order (renames kept via `as`, globs
//!   re-rendered), each at the statement's own indent; an old dep with an
//!   EMPTY new list deletes the statement, its whole line when alone. The
//!   mod-space half grafts every bare new spelling NOT yet declared as
//!   `mod <sanitized>;` right after the anchored out-of-line
//!   `mod <old-stem>;`, copying its visibility and indent - anchored ONLY
//!   where the old spelling is declared, so a plain `use` consumer can
//!   never receive an out-of-place `mod`; already-declared names make the
//!   pass a byte-identical no-op (IDEMPOTENT, pinned).
//! - [`DependencyRewriter::find_dependency_targets`] reports real
//!   statements only: a `use` statement hits for its exact leaves, the
//!   wildcard spelling `base::*`, and any target UNDER the wildcard base
//!   (Scala's qualified-access rule); a bare probe hits the parent's
//!   out-of-line `mod <name>;` verbatim. Unparseable input yields no hits:
//!   a scan that cannot guarantee "never inside comments or strings" must
//!   not report hits at all.
//! - [`DependencyRewriter::consumer_discovery_targets`] probes the moved
//!   ids (use-space), PLUS - for each FLAT id on a nested parent - its
//!   honest parent-qualified written spelling derived from the target
//!   file's module path (R-9b3, AMENDMENT-1 sec. 1a(2) grammar: discovery's
//!   predicate and the rewrite's matcher agree by construction, CW-1),
//!   PLUS the plan target file's sanitized stem (mod-space): the parent
//!   declaring `mod <source-stem>;` becomes a consumer-side target of the
//!   extraction transaction - the grown-oracle discipline (SM-4) then sees
//!   it in the dry-run MODIFIED set.
//! - [`DependencyRewriter::consumer_rewrite_spec`] derives that
//!   dual-space spec: moved ids under a source module are retargeted onto
//!   its new sibling module (ids under no source module are DROPPED -
//!   never invent an edge the spec does not name, Scala's discipline; the
//!   target is the single new module or the one whose sanitized stem
//!   matches the id leaf, else the id stays untouched) and the old/new
//!   path stems become the bare mod-space pair.
//!
//! - Amendment sec. 1a(2) (R-9a): when the prefix-match leg answers `None`
//!   for a FLAT moved id (`crate::X` - the only shape the wired route ever
//!   produces), a DERIVATION leg derives the written spellings from the
//!   payload's index-aligned file pairing: old `M(old_includes[i])::X`,
//!   new `M(new_includes[i])::X` (module paths per
//!   [`module_path_of_src_file`], so the crate-root and `mod.rs` parents
//!   spell honestly). Unequal lists cannot name an honest pairing and add
//!   nothing; multi-segment and foreign-root ids take the prefix-match leg
//!   only. Matches key on the FULL derived spelling (leaf equality), so a
//!   colliding flat name in another module is never reached.
//!
//! `move_out_top_level_symbols` is overridden (R-9a, amendment sec. 3):
//! deletion + declaration insert, Scala parity; the machinery lives in
//! [`crate::move_retention`] and this file keeps the one delegating method.
//! Hooks left at their trait defaults: `remove_top_level_symbols` (the
//! engine's copy-only channel - Rust retention is the move hook's job),
//! `render_diff` (neutral marker grammar), and `emit_file_preamble`.

use crate::mod_tree::{
    apply_splices, is_bare_mod_spelling, line_indent_at, line_start_offsets, located_span,
    module_path_of_src_file, sanitize_module_stem, use_leaves, SpliceEdit, UseLeaf,
};
use ariadne_lang::{DependencyRewriter, DirectiveHit, RewriteSpec};
use std::path::Path;
use syn::spanned::Spanned;

/// Rewrites Rust `use` paths and keeps extracted modules declared (R-8).
pub struct RustDependencyRewriter;

impl DependencyRewriter for RustDependencyRewriter {
    fn rewrite(&self, source: &str, spec: &RewriteSpec) -> String {
        insert_declared_mods(&rewrite_use_paths(source, spec), spec)
    }

    fn find_dependency_targets(&self, source: &str, target: &str) -> Vec<DirectiveHit> {
        let Ok(parsed) = syn::parse_file(source) else {
            return Vec::new();
        };
        let line_starts = line_start_offsets(source);
        let mut hits = Vec::new();
        for item in &parsed.items {
            let mentions = match item {
                syn::Item::Use(use_item) => use_leaves(use_item)
                    .iter()
                    .any(|leaf| leaf_affected_by(leaf, target, true)),
                syn::Item::Mod(item_mod) => {
                    item_mod.content.is_none() && item_mod.ident == target
                }
                _ => false,
            };
            if !mentions {
                continue;
            }
            if let Some((start_line, end_line, from, to)) =
                located_span(source, &line_starts, item.span())
            {
                hits.push(DirectiveHit {
                    start_line,
                    end_line,
                    raw_directive: source[from..to].trim_end().to_string(),
                });
            }
        }
        hits
    }

    /// Use-space probes (the moved ids) plus the mod-space probe (the
    /// sanitized stem of the plan's target file): the parent that declares
    /// `mod <stem>;` is discovered as a consumer so its declaration insert
    /// rides the same transaction. For a FLAT id `crate::X` the honest
    /// written spelling `M::X` (M = the target file's module path) rides
    /// too - exactly the spec leg's derived old spelling (CW-1
    /// agree-by-construction); a crate root spells `crate::X` already, so
    /// the dedupe keeps those probe lists byte-identical.
    fn consumer_discovery_targets(
        &self,
        target_file: &Path,
        moved_module_ids: &[String],
    ) -> Vec<String> {
        let mut probes: Vec<String> = Vec::new();
        let parent_module = module_path_of_src_file(target_file);
        for id in moved_module_ids {
            probes.push(id.clone());
            if let (Some(flat_leaf), Some(module)) = (flat_id_leaf(id), parent_module.as_deref()) {
                let honest = format!("{module}::{flat_leaf}");
                if !probes.contains(&honest) {
                    probes.push(honest);
                }
            }
        }
        if let Some(stem) = target_file.file_stem() {
            probes.push(sanitize_module_stem(&stem.to_string_lossy()));
        }
        probes
    }

    fn consumer_rewrite_spec(
        &self,
        old_includes: &[std::path::PathBuf],
        new_includes: &[std::path::PathBuf],
        moved_module_ids: &[String],
    ) -> Option<RewriteSpec> {
        old_includes.first()?;
        let source_modules: Vec<String> =
            old_includes.iter().filter_map(|path| module_path_of_src_file(path)).collect();
        let target_modules: Vec<String> =
            new_includes.iter().filter_map(|path| module_path_of_src_file(path)).collect();
        let mut old_deps: Vec<String> = Vec::new();
        let mut new_deps: Vec<String> = Vec::new();
        for id in moved_module_ids {
            // The prefix-match leg stays byte-identical and is tried FIRST,
            // so every R-8 witness keeps its exact old_deps/new_deps bytes;
            // the derivation leg below only answers where it refused.
            let mapped: Vec<(String, String)> = match retarget_id(id, &source_modules, &target_modules)
            {
                Some(pair) => vec![pair],
                None => derive_flat_id_pairs(id, old_includes, new_includes),
            };
            for (old_space_id, new_space_id) in mapped {
                old_deps.push(old_space_id);
                new_deps.push(new_space_id);
            }
        }
        old_deps.extend(unique_stems(old_includes));
        new_deps.extend(new_stems_except_old(new_includes, &old_deps));
        Some(RewriteSpec { old_deps, new_deps })
    }

    /// Rust retention (amendment sec. 3): deletion + declaration insert,
    /// Scala parity; `plan_id` is ignored (this retention needs no
    /// provenance). The machinery lives in [`crate::move_retention`].
    fn move_out_top_level_symbols(
        &self,
        source: &str,
        qualified_names: &[String],
        _plan_id: &str,
    ) -> Option<String> {
        crate::move_retention::move_out_retained(source, qualified_names)
    }
}

/// Derive the use-space pairs for a FLAT moved id (amendment sec. 1a(2)):
/// the id must be `crate::X` with a single segment `X` (multi-segment ids
/// and foreign roots take the prefix-match leg only); each index-aligned
/// `(old_includes[i], new_includes[i])` pairing yields the written-spelling
/// pair `M_old::X` -> `M_new::X` (a crate-root `M_old` spells `crate::X`).
/// Unequal list lengths cannot name an honest pairing and add nothing; a
/// pairing whose paths are outside a `src` root contributes nothing.
fn derive_flat_id_pairs(
    id: &str,
    old_includes: &[std::path::PathBuf],
    new_includes: &[std::path::PathBuf],
) -> Vec<(String, String)> {
    let Some(flat_leaf) = flat_id_leaf(id) else {
        return Vec::new();
    };
    if old_includes.len() != new_includes.len() {
        return Vec::new();
    }
    old_includes
        .iter()
        .zip(new_includes)
        .filter_map(|(old_path, new_path)| {
            let old_module = module_path_of_src_file(old_path)?;
            let new_module = module_path_of_src_file(new_path)?;
            Some((format!("{old_module}::{flat_leaf}"), format!("{new_module}::{flat_leaf}")))
        })
        .collect()
}

/// The single segment of a flat analyzer id (`crate::X` -> `Some("X")`);
/// every other shape (bare, multi-segment, foreign root) is `None` - the
/// derivation leg's eligibility grammar.
fn flat_id_leaf(id: &str) -> Option<&str> {
    let remainder = id.strip_prefix("crate::")?;
    (!remainder.is_empty() && !remainder.contains("::")).then_some(remainder)
}

/// Retarget one moved id: longest old-module prefix wins; the new module is
/// the single target, or the target whose sanitized leaf stem matches the
/// id's leaf. No source prefix or no target match => `None` (the id rides
/// no spec - its written paths are left untouched, never invented).
fn retarget_id(
    id: &str,
    source_modules: &[String],
    target_modules: &[String],
) -> Option<(String, String)> {
    let source_module = source_modules
        .iter()
        .filter(|module| id == *module || id.starts_with(&format!("{module}::")))
        .max_by_key(|module| module.len())?;
    let suffix = &id[source_module.len() + 2..];
    let leaf = suffix.rsplit("::").next()?;
    let target_module = match target_modules.len() {
        0 => return None,
        1 => target_modules[0].clone(),
        _ => target_modules
            .iter()
            .find(|target| {
                sanitize_module_stem(target.rsplit("::").next().unwrap_or_default())
                    == sanitize_module_stem(leaf)
            })?
            .clone(),
    };
    Some((id.to_string(), format!("{target_module}::{suffix}")))
}

fn unique_stems(paths: &[std::path::PathBuf]) -> Vec<String> {
    let mut stems: Vec<String> = Vec::new();
    for stem in paths.iter().filter_map(|path| path.file_stem()) {
        let sanitized = sanitize_module_stem(&stem.to_string_lossy());
        if !stems.contains(&sanitized) {
            stems.push(sanitized);
        }
    }
    stems
}

fn new_stems_except_old(new_includes: &[std::path::PathBuf], old_deps: &[String]) -> Vec<String> {
    unique_stems(new_includes).into_iter().filter(|stem| !old_deps.contains(stem)).collect()
}

/// The use-space half of `rewrite`: affected top-level `use` statements are
/// replaced through byte splices, so every untouched byte (comments
/// included) survives verbatim. Unparseable input returns unchanged.
fn rewrite_use_paths(source: &str, spec: &RewriteSpec) -> String {
    let old_uses: Vec<&String> =
        spec.old_deps.iter().filter(|spelling| !is_bare_mod_spelling(spelling)).collect();
    if old_uses.is_empty() {
        return source.to_string();
    }
    let new_uses: Vec<&String> =
        spec.new_deps.iter().filter(|spelling| !is_bare_mod_spelling(spelling)).collect();
    let Ok(parsed) = syn::parse_file(source) else {
        return source.to_string();
    };
    let line_starts = line_start_offsets(source);
    let edits: Vec<SpliceEdit> = parsed
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Use(use_item) => {
                use_statement_edit(source, &line_starts, use_item, &old_uses, &new_uses)
            }
            _ => None,
        })
        .collect();
    apply_splices(source, edits)
}

/// The replacement for one affected `use` statement, or `None` when the
/// statement is untouched. New dep lines land once, at the first matched
/// leaf's slot (each new dep at most once per statement - an honest
/// refinement of Scala's per-dep re-scan, which pinned the duplicate-
/// expansion alternative).
fn use_statement_edit(source: &str, line_starts: &[usize], use_item: &syn::ItemUse, old_uses: &[&String], new_uses: &[&String]) -> Option<SpliceEdit> {
    let leaves = use_leaves(use_item);
    let affected: Vec<bool> = leaves
        .iter()
        .map(|leaf| old_uses.iter().any(|old_dep| leaf_affected_by(leaf, old_dep, false)))
        .collect();
    if !affected.iter().any(|hit| *hit) {
        return None;
    }
    let visibility =
        matches!(use_item.vis, syn::Visibility::Public(_)).then_some("pub ").unwrap_or("");
    let mut lines: Vec<String> = Vec::new();
    let mut new_deps_emitted = false;
    for (leaf, is_affected) in leaves.iter().zip(affected) {
        if is_affected {
            if !new_deps_emitted {
                lines.extend(new_uses.iter().map(|dep| format!("{visibility}use {dep};")));
                new_deps_emitted = true;
            }
        } else {
            lines.push(render_leaf(visibility, leaf));
        }
    }
    let Some((_start_line, _end_line, from, to)) = located_span(source, line_starts, use_item.span()) else {
        return None;
    };
    let indent = line_indent_at(source, from);
    Some(SpliceEdit { from, to, replacement: lines.join(&format!("\n{indent}")) })
}

fn leaf_affected_by(leaf: &UseLeaf, spelling: &str, allow_glob_prefix: bool) -> bool {
    if leaf.star {
        spelling == leaf.spelling
            || spelling == format!("{}::*", leaf.spelling)
            || (allow_glob_prefix && spelling.starts_with(&format!("{}::", leaf.spelling)))
    } else {
        leaf.spelling == spelling
    }
}

fn render_leaf(visibility: &str, leaf: &UseLeaf) -> String {
    if leaf.star {
        format!("{visibility}use {}::*;", leaf.spelling)
    } else if let Some(alias) = &leaf.alias {
        format!("{visibility}use {} as {alias};", leaf.spelling)
    } else {
        format!("{visibility}use {};", leaf.spelling)
    }
}

/// The mod-space half of `rewrite`: graft `mod <sanitized>;` for every
/// bare new spelling the file does not declare yet (inline blocks count as
/// declared - idempotence), anchored after the out-of-line
/// `mod <old-stem>;`. No anchor => NO insertion anywhere: only the file
/// declaring the source module can honestly host the sibling.
fn insert_declared_mods(content: &str, spec: &RewriteSpec) -> String {
    let new_names: Vec<&String> =
        spec.new_deps.iter().filter(|spelling| is_bare_mod_spelling(spelling)).collect();
    if new_names.is_empty() {
        return content.to_string();
    }
    let old_names: Vec<&String> =
        spec.old_deps.iter().filter(|spelling| is_bare_mod_spelling(spelling)).collect();
    let Ok(parsed) = syn::parse_file(content) else {
        return content.to_string();
    };
    let declared: Vec<String> = parsed
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Mod(item_mod) => Some(item_mod.ident.to_string()),
            _ => None,
        })
        .collect();
    let to_insert: Vec<&String> = new_names
        .into_iter()
        .filter(|name| !declared.iter().any(|existing| existing == *name))
        .collect();
    if to_insert.is_empty() {
        return content.to_string();
    }
    let Some(anchor) = parsed.items.iter().find_map(|item| match item {
        syn::Item::Mod(item_mod)
            if item_mod.content.is_none() && old_names.iter().any(|old| item_mod.ident == **old) =>
        {
            Some(item_mod)
        }
        _ => None,
    }) else {
        return content.to_string();
    };
    let line_starts = line_start_offsets(content);
    let Some((_start_line, _end_line, from, to)) = located_span(content, &line_starts, anchor.span()) else {
        return content.to_string();
    };
    let visibility =
        matches!(anchor.vis, syn::Visibility::Public(_)).then_some("pub ").unwrap_or("");
    let indent = line_indent_at(content, from);
    let terminator = line_terminator(content, to);
    let mut block = String::new();
    for name in to_insert {
        block.push_str(&format!("{indent}{visibility}mod {name};{terminator}"));
    }
    let insert_pos = match content[to..].find('\n') {
        Some(offset) => to + offset + 1,
        None => content.len(),
    };
    let dangling_newline =
        (insert_pos == content.len() && !content.ends_with('\n')).then_some(terminator).unwrap_or_default();
    format!("{}{dangling_newline}{block}{}", &content[..insert_pos], &content[insert_pos..])
}

/// The newline style closing the line that contains/ends at byte `to`
/// (shared with [`crate::move_retention`]: one CRLF discipline, one source).
pub(crate) fn line_terminator(content: &str, to: usize) -> &'static str {
    match content[to..].find('\n') {
        Some(offset) if to + offset > 0 && content.as_bytes()[to + offset - 1] == b'\r' => "\r\n",
        Some(_) => "\n",
        None => "\n",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn spec(old: &[&str], new: &[&str]) -> RewriteSpec {
        RewriteSpec {
            old_deps: old.iter().map(|dep| dep.to_string()).collect(),
            new_deps: new.iter().map(|dep| dep.to_string()).collect(),
        }
    }

    fn rw(source: &str, old: &[&str], new: &[&str]) -> String {
        RustDependencyRewriter.rewrite(source, &spec(old, new))
    }

    /// The split specimen (R-9's apply shape, exercised in-crate here):
    /// `src/models/user.rs` loses `User` to `src/models/user_record.rs`; the
    /// use-space ids and the mod-space bare stems ride ONE spec.
    fn split_spec() -> RewriteSpec {
        RustDependencyRewriter
            .consumer_rewrite_spec(
                &[PathBuf::from("src/models/user.rs")],
                &[PathBuf::from("src/models/user_record.rs")],
                &["crate::models::user::User".to_string()],
            )
            .expect("an src-rooted split derives a Rust spec")
    }

    #[test]
    fn consumer_rewrite_spec_maps_the_use_space_and_the_mod_space() {
        let derived = split_spec();
        assert_eq!(
            derived.old_deps,
            vec!["crate::models::user::User".to_string(), "user".to_string()],
            "use-space ids for consumers + the bare source stem as the \
             parent's mod anchor"
        );
        assert_eq!(
            derived.new_deps,
            vec![
                "crate::models::user_record::User".to_string(),
                "user_record".to_string(),
            ],
            "retargeted id + the bare new stem to declare"
        );
    }

    #[test]
    fn consumer_rewrite_spec_is_none_without_old_paths() {
        assert!(
            RustDependencyRewriter
                .consumer_rewrite_spec(&[], &[], &["crate::a::B".to_string()])
                .is_none(),
            "no payload: the caller keeps the path strings (Scala's \
             conservative None precedent)"
        );
    }

    #[test]
    fn consumer_rewrite_spec_drops_ids_outside_the_source_module() {
        let derived = RustDependencyRewriter
            .consumer_rewrite_spec(
                &[PathBuf::from("src/models/user.rs")],
                &[PathBuf::from("src/models/user_record.rs")],
                &[
                    "crate::models::user::User".to_string(),
                    "other::place::CoProposed".to_string(),
                ],
            )
            .expect("spec derives");
        assert!(
            !derived.old_deps.iter().any(|dep| dep.contains("CoProposed"))
                && !derived.new_deps.iter().any(|dep| dep.contains("CoProposed")),
            "never invent an edge the spec does not name (Scala discipline)"
        );
    }

    #[test]
    fn use_path_maps_before_after_literals() {
        let consumer = "use crate::models::user::User;\n\nfn main() {}\n";
        assert_eq!(
            RustDependencyRewriter.rewrite(consumer, &split_spec()),
            "use crate::models::user_record::User;\n\nfn main() {}\n",
            "byte-pinned before/after literals: only the path text moves"
        );
    }

    #[test]
    fn every_occurrence_of_the_old_path_is_rewritten() {
        assert_eq!(
            rw(
                "use crate::a::Old;\nuse other::T;\nuse crate::a::Old;\n",
                &["crate::a::Old"],
                &["crate::b::New"],
            ),
            "use crate::b::New;\nuse other::T;\nuse crate::b::New;\n",
            "C++/Scala all-occurrence discipline"
        );
    }

    #[test]
    fn grouped_survivors_keep_their_renames_and_the_new_dep_lands_at_the_slot() {
        assert_eq!(
            rw(
                "use crate::models::user::{User, Helper as H};\n",
                &["crate::models::user::User"],
                &["crate::models::user_record::User"],
            ),
            "use crate::models::user_record::User;\nuse crate::models::user::Helper as H;\n",
            "LOCK: one leaf per line, the rename survives (`as H`), the spec \
             names no path for Helper so its module prefix is NOT touched"
        );
    }

    #[test]
    fn glob_narrows_on_the_aggregate_old_dep_never_on_a_symbol_dep() {
        assert_eq!(
            rw("use crate::a::*;\n", &["crate::a"], &["crate::a::deep::C"]),
            "use crate::a::deep::C;\n",
            "Scala wildcard-narrowing analog: the old dep IS the base"
        );
        assert_eq!(
            rw("use crate::a::*;\n", &["crate::a::C"], &["crate::a::deep::C"]),
            "use crate::a::*;\n",
            "a symbol-level old dep does not touch the wildcard"
        );
    }

    #[test]
    fn old_dep_with_empty_new_deps_deletes_the_statement_line() {
        assert_eq!(
            rw(
                "fn above() {}\n\nuse crate::models::user::User;\n\nfn below() {}\n",
                &["crate::models::user::User"],
                &[],
            ),
            "fn above() {}\n\n\nfn below() {}\n",
            "the whole line goes when the statement is alone on it"
        );
    }

    #[test]
    fn single_segment_old_dep_is_mod_space_never_use_space() {
        assert_eq!(
            rw("use user;\n", &["user"], &["user_record"]),
            "use user;\n",
            "LOCK (documented grammar): bare spellings drive the mod-tree \
             clause only; a bare `use` statement is never touched"
        );
    }

    #[test]
    fn unparseable_source_is_returned_unchanged() {
        assert_eq!(
            rw("fn broken( {{{\n", &["crate::a::Old"], &["crate::b::New"]),
            "fn broken( {{{\n",
            "no syn tree, no located directives: conservative no-op, never \
             corrupt (the spec-consumption analog of Scala's scanner guarantee)"
        );
    }

    #[test]
    fn pin_multibyte_bytes_before_a_touched_use_survive_byte_exact() {
        // Specimen from escapes (authored file stays pure ASCII): two-byte
        // u-umlaut before the touched use statement.
        let source = "// \u{fc}ber note\nuse crate::a::Old;\n";
        let out = rw(source, &["crate::a::Old"], &["crate::b::New"]);
        assert_eq!(
            out.replace("crate::b::New", "crate::a::Old"),
            source,
            "byte-preservation proof: char/byte offsets neither shifted nor \
             corrupted the multibyte sequence"
        );
    }

    #[test]
    fn parent_gets_the_sibling_mod_declared_once_in_the_anchor_style() {
        let parent = "pub mod user;\n\npub fn models_marker() {}\n";
        assert_eq!(
            RustDependencyRewriter.rewrite(parent, &split_spec()),
            "pub mod user;\npub mod user_record;\n\npub fn models_marker() {}\n",
            "RISK-MODTREE insert-once: the sibling declaration is grafted \
             right after the anchor, visibility copied, ASCII"
        );
        assert_eq!(
            rw("mod user;\n", &["user"], &["user_record"]),
            "mod user;\nmod user_record;\n",
            "a private anchor grafts a private declaration (block style)"
        );
    }

    #[test]
    fn parent_mod_insertion_is_idempotent_noop() {
        let parent = "mod user;\n";
        let once = RustDependencyRewriter.rewrite(parent, &split_spec());
        let twice = RustDependencyRewriter.rewrite(&once, &split_spec());
        assert_eq!(
            once, "mod user;\nmod user_record;\n",
            "first application inserts"
        );
        assert_eq!(
            twice, once,
            "already declared -> byte-identical no-op (pinned idempotence)"
        );
    }

    #[test]
    fn use_consumer_without_the_mod_anchor_never_receives_an_insertion() {
        let out = RustDependencyRewriter.rewrite("use crate::models::user::User;\n", &split_spec());
        assert_eq!(
            out, "use crate::models::user_record::User;\n",
            "the mod-space clause is anchored at a declared mod statement: \
             a file that does not declare the source module stays \
             use-rewrite-only - never corrupted with an out-of-place mod"
        );
    }

    #[test]
    fn discovery_targets_are_the_moved_paths_plus_the_source_module_stem() {
        assert_eq!(
            RustDependencyRewriter.consumer_discovery_targets(
                Path::new("src/models/user.rs"),
                &["crate::models::user::User".to_string()],
            ),
            vec![
                "crate::models::user::User".to_string(),
                "user".to_string(),
            ],
            "RISK-MODTREE: the parent that declares `mod user;` is a \
             consumer-side target through the mod-space probe"
        );
    }

    #[test]
    fn find_dependency_targets_hits_use_leaves_globs_and_mod_declarations() {
        let source = "pub mod user;\nuse crate::models::user::User;\nuse crate::models::user::*;\n";
        let named = RustDependencyRewriter.find_dependency_targets(source, "crate::models::user::User");
        assert_eq!(
            named.iter().map(|hit| hit.start_line).collect::<Vec<usize>>(),
            vec![2, 3],
            "the exact leaf hits its statement AND the wildcard under its \
             base (Scala statement_mentions parity: a glob is target-\
             affected for every id under its base)"
        );
        assert_eq!(named[0].raw_directive, "use crate::models::user::User;");
        assert_eq!((named[0].start_line, named[0].end_line), (2, 2));
        let glob = RustDependencyRewriter.find_dependency_targets(source, "crate::models::user::Ghost");
        assert_eq!(
            glob.iter().map(|hit| hit.start_line).collect::<Vec<usize>>(),
            vec![3],
            "a qualified target is target-affected under the wildcard"
        );
        let mods = RustDependencyRewriter.find_dependency_targets(source, "user");
        assert_eq!(
            mods.iter().map(|hit| hit.raw_directive.as_str()).collect::<Vec<&str>>(),
            vec!["pub mod user;"],
            "the mod-space probe hits the parent's declaration verbatim"
        );
    }

    #[test]
    fn commented_quoted_and_sibling_spellings_are_never_hits() {
        let source = "// use crate::models::user::User;\nconst DOC: &str = \"use crate::models::user::User;\";\nuse crate::models::users::User;\n";
        assert!(
            RustDependencyRewriter
                .find_dependency_targets(source, "crate::models::user::User")
                .is_empty(),
            "syn scans real statements only: comments and string bodies \
             never yield hits"
        );
        assert_eq!(
            RustDependencyRewriter
                .find_dependency_targets(source, "crate::models::users::User")
                .len(),
            1,
            "the sibling spelling IS a real statement - it just is not the \
             probe above; prefix collisions stay conservative"
        );
    }

    /// R-9a F1 (amendment sec. 1b red 2): a FLAT moved id on a crate-root
    /// payload retargets from its honest written spelling `crate::X` onto
    /// the new file's module path - the m23-mirror consumer rewrite made
    /// satisfiable. Pre-mechanism this fails with `left: ["lib"]`.
    #[test]
    fn derived_spec_retargets_the_flat_id_from_the_root_parent() {
        let derived = RustDependencyRewriter
            .consumer_rewrite_spec(
                &[PathBuf::from("src/lib.rs")],
                &[PathBuf::from("src/fixture_marker.rs")],
                &["crate::fixture_marker".to_string()],
            )
            .expect("an src-rooted payload derives a Rust spec");
        assert_eq!(
            derived.old_deps,
            vec!["crate::fixture_marker".to_string(), "lib".to_string()],
            "the flat id's honest written spelling at the crate root, then \
             the inert `lib` stem anchor"
        );
        assert_eq!(
            derived.new_deps,
            vec![
                "crate::fixture_marker::fixture_marker".to_string(),
                "fixture_marker".to_string(),
            ],
            "the honest Rust spelling repeats the segment when the new \
             file's stem equals the moved name (suite-pinned byte)"
        );
    }

    /// R-9a F1 (amendment sec. 1b red 3): the `mod.rs` parent class - the
    /// flat id's written spelling is the DIRECTORY's module path.
    #[test]
    fn derived_spec_retargets_the_flat_id_from_a_mod_rs_parent() {
        let derived = RustDependencyRewriter
            .consumer_rewrite_spec(
                &[PathBuf::from("src/models/mod.rs")],
                &[PathBuf::from("src/models/models_marker.rs")],
                &["crate::models_marker".to_string()],
            )
            .expect("an src-rooted payload derives a Rust spec");
        assert_eq!(
            derived.old_deps,
            vec![
                "crate::models::models_marker".to_string(),
                "mod".to_string(),
            ],
            "old spelling is the directory module + the flat id, not the \
             written file name"
        );
        assert_eq!(
            derived.new_deps,
            vec![
                "crate::models::models_marker::models_marker".to_string(),
                "models_marker".to_string(),
            ],
            "the segment repeat again (m23-mirror apply byte)"
        );
    }

    /// R-9a F1 (amendment sec. 1b red 4): collision safety - two modules
    /// declaring the same flat name derive DIFFERENT written spellings (the
    /// payload file keys the match), so a rewrite never reaches across.
    #[test]
    fn derived_spelling_never_touches_a_colliding_flat_name() {
        let derived = RustDependencyRewriter
            .consumer_rewrite_spec(
                &[PathBuf::from("src/a/m.rs")],
                &[PathBuf::from("src/a/m_record.rs")],
                &["crate::Thing".to_string()],
            )
            .expect("an src-rooted payload derives a Rust spec");
        let consumer = "use crate::a::m::Thing;\nuse crate::b::m::Thing;\n";
        assert_eq!(
            RustDependencyRewriter.rewrite(consumer, &derived),
            "use crate::a::m_record::Thing;\nuse crate::b::m::Thing;\n",
            "only the payload module's consumer line is rewritten; the \
             colliding flat name in another module stays BYTE-IDENTICAL"
        );
    }

    /// R-9a F1 (amendment sec. 1b red 5, GREEN-at-birth characterization
    /// justified per SM-3's clause): the derivation leg takes FLAT ids
    /// only; foreign roots are dropped by both legs. Today's refusal, and
    /// it stays - the prefix-match leg (byte-identical, tried first) may
    /// pair a `crate::`-rooted multi-segment id on a crate-root payload,
    /// pairing its old spelling verbatim; the derivation leg contributes
    /// no pair for it, pinned where the prefix leg cannot fire at all.
    #[test]
    fn derived_leg_refuses_multisegment_ids_and_foreign_roots() {
        let on_root_parent = RustDependencyRewriter
            .consumer_rewrite_spec(
                &[PathBuf::from("src/lib.rs")],
                &[PathBuf::from("src/fixture_marker.rs")],
                &[
                    "crate::inner::Helper".to_string(),
                    "other::place::CoProposed".to_string(),
                ],
            )
            .expect("spec derives");
        assert!(
            !on_root_parent.old_deps.iter().any(|dep| dep.contains("CoProposed"))
                && !on_root_parent.new_deps.iter().any(|dep| dep.contains("CoProposed")),
            "ids without a `crate::` root are dropped (Scala discipline, UNEDITED)"
        );
        let helper_old_spellings: Vec<&String> = on_root_parent
            .old_deps
            .iter()
            .filter(|dep| dep.contains("Helper"))
            .collect();
        assert!(
            helper_old_spellings
                .iter()
                .all(|dep| dep.as_str() == "crate::inner::Helper"),
            "any Helper pairing present is the prefix-match leg's verbatim-id \
             pair (old spelling == the id itself); the derivation leg never \
             rewrites a multi-segment id's old spelling: {helper_old_spellings:?}"
        );
        // The refusal where only the derivation leg could fire: a mod.rs
        // parent gives the multi-segment id no prefix match, and the
        // derivation leg must answer no pair at all.
        let on_mod_rs_parent = RustDependencyRewriter
            .consumer_rewrite_spec(
                &[PathBuf::from("src/models/mod.rs")],
                &[PathBuf::from("src/models/models_marker.rs")],
                &[
                    "crate::inner::Helper".to_string(),
                    "other::place::CoProposed".to_string(),
                ],
            )
            .expect("spec derives");
        assert_eq!(
            on_mod_rs_parent.old_deps,
            vec!["mod".to_string()],
            "multi-segment ids take the prefix-match leg ONLY: no pair, only \
             the inert stem anchor remains"
        );
        assert_eq!(
            on_mod_rs_parent.new_deps,
            vec!["models_marker".to_string()],
            "no derived new spelling either"
        );
    }

    /// R-9a F1 (green-at-birth characterization of amendment sec. 1a(2)):
    /// unequal payload lists cannot name an honest pairing, so the
    /// derivation leg adds nothing in either direction.
    #[test]
    fn derived_leg_refuses_unequal_payload_pairing() {
        let more_new = RustDependencyRewriter
            .consumer_rewrite_spec(
                &[PathBuf::from("src/models/mod.rs")],
                &[
                    PathBuf::from("src/models/one.rs"),
                    PathBuf::from("src/models/two.rs"),
                ],
                &["crate::marker".to_string()],
            )
            .expect("spec derives");
        assert_eq!(
            more_new.old_deps,
            vec!["mod".to_string()],
            "no honest pairing exists: no derived spelling"
        );
        assert_eq!(
            more_new.new_deps,
            vec!["one".to_string(), "two".to_string()],
            "only the mod-space stems ride the spec"
        );
        let more_old = RustDependencyRewriter
            .consumer_rewrite_spec(
                &[
                    PathBuf::from("src/models/mod.rs"),
                    PathBuf::from("src/models/extra.rs"),
                ],
                &[PathBuf::from("src/models/models_marker.rs")],
                &["crate::marker".to_string()],
            )
            .expect("spec derives");
        assert!(
            !more_old.old_deps.iter().any(|dep| dep.contains("marker")),
            "the mirror direction refuses identically: {:?}",
            more_old.old_deps
        );
    }

    /// R-9a F1 (amendment sec. 1a(1): "`lib`/`mod` stems are inert anchors
    /// - pinned"): the mod-space half still anchors sub-directory grafts on
    /// bare stems, but no real file declares `mod lib;`/`mod mod;`, so
    /// those spellings never graft anywhere. Green-at-birth: the refusal is
    /// today's behavior and stays.
    #[test]
    fn lib_and_mod_stems_are_inert_mod_space_anchors() {
        assert_eq!(
            rw("pub mod greeting;\npub fn keeper() {}\n", &["lib"], &["fixture_marker"]),
            "pub mod greeting;\npub fn keeper() {}\n",
            "no file declares `mod lib;`: the root-file stem anchors nothing"
        );
        assert_eq!(
            rw("pub mod models;\npub fn keeper() {}\n", &["mod"], &["models_marker"]),
            "pub mod models;\npub fn keeper() {}\n",
            "no file declares `mod mod;`: the mod.rs stem anchors nothing"
        );
    }

    /// R-9b3 W1 (AMENDMENT-3 sec. 1d): the nested-parent probe list carries
    /// the honest parent-qualified written spelling of the FLAT id - the
    /// exact old spelling the spec leg derives (:729-737), so discovery's
    /// predicate and the rewrite's matcher agree by construction (CW-1).
    #[test]
    fn discovery_probes_the_honest_module_spelling_for_a_nested_parent() {
        assert_eq!(
            RustDependencyRewriter.consumer_discovery_targets(
                Path::new("src/models/mod.rs"),
                &["crate::models_marker".to_string()],
            ),
            vec![
                "crate::models_marker".to_string(),
                "crate::models::models_marker".to_string(),
                "mod".to_string(),
            ],
            "the flat id, then its honest M::X spelling, then the inert mod.rs stem anchor"
        );
    }

    /// R-9b3 W3 (AMENDMENT-3 sec. 1d, INV-R9B3-2): GREEN-at-birth
    /// characterization, justified per SM-3's clause (AMENDMENT-2 sec. 2
    /// usage): the crate-root parent dedupes by equality - the flat id
    /// already IS the honest spelling - so crate-root probe lists stay
    /// byte-identical before and after the derivation.
    #[test]
    fn crate_root_discovery_probes_stay_byte_identical() {
        assert_eq!(
            RustDependencyRewriter.consumer_discovery_targets(
                Path::new("src/lib.rs"),
                &["crate::fixture_marker".to_string()],
            ),
            vec!["crate::fixture_marker".to_string(), "lib".to_string()],
            "plan-1 pin (suite :527-531): id leg then the inert `lib` anchor, nothing derived"
        );
        assert_eq!(
            RustDependencyRewriter.consumer_discovery_targets(
                Path::new("src/main.rs"),
                &["crate::fixture_marker".to_string()],
            ),
            vec!["crate::fixture_marker".to_string(), "main".to_string()],
            "main.rs is a crate-root file too: same dedupe, the stem leg unchanged"
        );
    }
}
