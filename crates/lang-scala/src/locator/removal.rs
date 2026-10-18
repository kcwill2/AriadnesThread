//! Whole-symbol removal for Scala move semantics (plan M23, T6, D-M23-5).
//!
//! Regions are the *same* declaration spans the T5 slicer uses
//! ([`crate::parser::program_of`]), then widened for deletion:
//! - a directly preceding comment block (doc comment, line comments, or a
//!   same-line `/* … */` prefix) joins the region, code never does;
//! - the region's own line indent and trailing terminator join the region;
//! - exactly ONE adjacent blank separator line is removed with it (the one
//!   after the region when present, else the one before) — unrelated blank
//!   lines are never collapsed;
//! - companion pairing (D17): removing `p.X` also removes `p.X$` (and vice
//!   versa) — the isolation unit is the whole top-level symbol.
//!
//! Any name that cannot be located fails the WHOLE call with `None`, so the
//! engine falls back without a partial removal.

use super::qualified_suffix_matches;
use crate::imports::char_to_byte_map;
use crate::parser::{program_of, ResolvedDecl, SourceProgram};

/// Remove every named top-level symbol (with companion) from `source`, or
/// `None` when any name is unlocatable.
pub(crate) fn removal_text(source: &str, names: &[String]) -> Option<String> {
    let program = program_of(source);
    let bytes = char_to_byte_map(source);
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for name in names {
        let decl = program
            .decls
            .iter()
            .find(|decl| qualified_suffix_matches(name, &decl.id.0))?;
        push_byte_span(&program, &bytes, decl, &mut spans);
        if let Some(companion) = companion_of(&program, decl) {
            push_byte_span(&program, &bytes, companion, &mut spans);
        }
    }
    let mut content = source.to_string();
    let mut ranges: Vec<(usize, usize)> = spans
        .into_iter()
        .map(|(start, end)| expand_removal_range(&content, start, end))
        .collect();
    ranges.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in ranges {
        match merged.last_mut() {
            Some(last) if last.1 >= start => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    for (start, end) in merged.iter().rev() {
        content.replace_range(*start..*end, "");
    }
    Some(content)
}

/// The D17 companion id (`X` ↔ `X$`) when the file also declares it.
fn companion_of<'a>(program: &'a SourceProgram, decl: &ResolvedDecl) -> Option<&'a ResolvedDecl> {
    let id = decl.id.0.as_str();
    let pair_id = match id.strip_suffix('$') {
        Some(stripped) => stripped.to_string(),
        None => format!("{id}$"),
    };
    program.decls.iter().find(|candidate| candidate.id.0 == pair_id)
}

fn push_byte_span(
    program: &SourceProgram,
    bytes: &[usize],
    decl: &ResolvedDecl,
    spans: &mut Vec<(usize, usize)>,
) {
    let last_token = decl
        .span_end_exclusive
        .min(program.tokens.len())
        .max(decl.span_start + 1)
        - 1;
    let from = bytes[program.tokens[decl.span_start].start];
    let to = bytes[program.tokens[last_token].end];
    spans.push((from, to));
}

/// Widen one region: comment prelude, full own lines, one blank separator.
fn expand_removal_range(content: &str, start: usize, end: usize) -> (usize, usize) {
    let mut start = include_comment_prelude(content, start);
    let mut end = end;
    let head = &content[..start];
    if let Some(line_start) = whole_line_start(head) {
        start = line_start;
    }
    end = include_own_terminator(content, end);
    if let Some(blank_end) = blank_line_after(content, end) {
        end = blank_end;
    } else if let Some(blank_start) = blank_line_before(content, start) {
        start = blank_start;
    }
    (start, end)
}

fn whole_line_start(head: &str) -> Option<usize> {
    let line_start = head.rfind('\n').map_or(0, |nl| nl + 1);
    head[line_start..]
        .chars()
        .all(char::is_whitespace)
        .then_some(line_start)
}

fn include_own_terminator(content: &str, end: usize) -> usize {
    let tail = &content[end..];
    let line_end = end + tail.find('\n').map_or(tail.len(), |nl| nl + 1);
    content[end..line_end]
        .chars()
        .all(char::is_whitespace)
        .then_some(line_end)
        .unwrap_or(end)
}

/// `end` sits at the start of the line after the region: consume it when it
/// is blank (whitespace-only to its terminator, or a whitespace-only EOF tail).
fn blank_line_after(content: &str, end: usize) -> Option<usize> {
    let rest = content.get(end..)?;
    match rest.find('\n') {
        Some(newline) if rest[..newline].chars().all(char::is_whitespace) => {
            Some(end + newline + 1)
        }
        None if rest.chars().all(char::is_whitespace) => Some(content.len()),
        _ => None,
    }
}

fn blank_line_before(content: &str, start: usize) -> Option<usize> {
    let head = content.get(..start)?;
    let blank_line_terminator = head.strip_suffix('\n')?;
    let previous_newline = blank_line_terminator.rfind('\n').map_or(0, |nl| nl + 1);
    blank_line_terminator[previous_newline..]
        .chars()
        .all(char::is_whitespace)
        .then_some(previous_newline)
}

/// Walk the region start back across a directly preceding comment block.
fn include_comment_prelude(content: &str, start: usize) -> usize {
    let mut start = start;
    loop {
        let head = &content[..start];
        let trimmed_end = head.trim_end().len();
        let trimmed = &content[..trimmed_end];
        if trimmed_end == 0 {
            return start;
        }
        if trimmed.ends_with("*/") {
            if let Some(open) = block_comment_start(trimmed) {
                start = open;
                continue;
            }
            return start;
        }
        let line_start = trimmed.rfind('\n').map_or(0, |nl| nl + 1);
        if content[line_start..trimmed_end].trim_start().starts_with("//") {
            start = line_start;
            continue;
        }
        return start;
    }
}

/// The opening `/*` of the block comment that closes exactly at the end of
/// `text` (nesting-aware); `None` when unbalanced.
fn block_comment_start(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut opens: Vec<usize> = Vec::new();
    let mut index = 0usize;
    while index + 1 < bytes.len() {
        if bytes[index] == b'/' && bytes[index + 1] == b'*' {
            opens.push(index);
            index += 2;
        } else if bytes[index] == b'*' && bytes[index + 1] == b'/' {
            let open = opens.pop()?;
            if opens.is_empty() && index + 2 == bytes.len() {
                return Some(open);
            }
            index += 2;
        } else {
            index += 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remove(source: &str, names: &[&str]) -> Option<String> {
        removal_text(source, &names.iter().map(|n| n.to_string()).collect::<Vec<String>>())
    }

    #[test]
    fn removal_leaves_every_other_byte_identical_and_consumes_one_blank() {
        let source = "package p\n\nclass Keep1 {\n  val x = 1\n}\n\nclass Drop {\n  val y = 2\n}\n\nclass Keep2\n";
        let expected = "package p\n\nclass Keep1 {\n  val x = 1\n}\n\nclass Keep2\n";
        assert_eq!(remove(source, &["p.Drop"]).as_deref(), Some(expected));
        assert_eq!(
            remove(source, &["Drop"]).as_deref(),
            Some(expected),
            "simple names suffix-match the qualified id like the T5 locator"
        );
    }

    #[test]
    fn doc_comment_and_modifiers_leave_with_the_region() {
        let source = "package p\n\n/** A doc\n  * second line */\nprivate final class Drop {\n}\n\nval keep = 1\n";
        assert_eq!(
            remove(source, &["Drop"]).as_deref(),
            Some("package p\n\nval keep = 1\n")
        );
    }

    #[test]
    fn line_comment_prelude_is_included_but_trailing_code_comments_are_not() {
        assert_eq!(
            remove("// doc for B\nclass B\nval a = 1 // mine\n", &["B"]).as_deref(),
            Some("val a = 1 // mine\n"),
            "a whole-line comment directly above belongs to the symbol"
        );
        assert_eq!(
            remove("val a = 1 // mine\nclass B\n", &["B"]).as_deref(),
            Some("val a = 1 // mine\n"),
            "a trailing comment on the PREVIOUS statement's line stays"
        );
    }

    #[test]
    fn companion_pairing_removes_class_and_dollar_object_together() {
        let source = "package p\n\nclass SomeClass {\n  val a = 1\n}\n\nobject SomeClass {\n  val b = 2\n}\n\nval keep = 3\n";
        let expected = "package p\n\nval keep = 3\n";
        assert_eq!(remove(source, &["p.SomeClass"]).as_deref(), Some(expected));
        assert_eq!(
            remove(source, &["p.SomeClass$"]).as_deref(),
            Some(expected),
            "LOCK (D17 / §3.4 atomic symbol groups): the removal unit is the \
             whole top-level symbol — naming either half removes both"
        );
    }

    #[test]
    fn lone_object_removal_touches_nothing_else() {
        assert_eq!(
            remove("object Main {\n  def go(): Unit = {}\n}\n", &["Main"]).as_deref(),
            Some("")
        );
    }

    #[test]
    fn braceless_def_region_ends_at_its_boundary_rule() {
        let source = "package p\n\ndef greet(): String = \"hi\"\n\nclass Real\n";
        assert_eq!(
            remove(source, &["p.greet"]).as_deref(),
            Some("package p\n\nclass Real\n")
        );
    }

    #[test]
    fn exactly_one_blank_separator_never_collapses_unrelated_blanks() {
        let source = "class A\n\n\nclass Drop\n\n\nval z\n";
        assert_eq!(
            remove(source, &["Drop"]).as_deref(),
            Some("class A\n\n\n\nval z\n"),
            "LOCK: region + exactly one adjacent blank; the unrelated double \
             blank before the region survives untouched"
        );
    }

    #[test]
    fn blank_separator_consumes_the_one_before_when_none_follows() {
        assert_eq!(
            remove("val a = 1\n\nclass Drop\nval b = 2\n", &["Drop"]).as_deref(),
            Some("val a = 1\nval b = 2\n")
        );
    }

    #[test]
    fn multiple_names_removals_are_atomic() {
        let source = "class A\nclass B\nclass C\n";
        assert_eq!(remove(source, &["A", "B"]).as_deref(), Some("class C\n"));
        assert_eq!(remove(source, &["A", "C", "Ghost"]).as_deref(), None);
        assert_eq!(remove(source, &[]).as_deref(), Some(source));
    }

    #[test]
    fn braced_body_with_strings_and_nested_braces_is_cut_exactly() {
        let source = "package p\n\nclass Drop {\n  val s = \"} { }\"\n  def f(): Unit = { println(\"}\") }\n}\n\nval keep = 1\n";
        assert_eq!(
            remove(source, &["Drop"]).as_deref(),
            Some("package p\n\nval keep = 1\n")
        );
    }

    #[test]
    fn pin_removal_under_crlf_consumes_one_blank_separator_byte_exact() {
        let source = "class A\r\n\r\nclass Drop\r\n\r\nval z\r\n";
        assert_eq!(
            remove(source, &["Drop"]).as_deref(),
            Some("class A\r\n\r\nval z\r\n"),
            "the CRLF blank separator counts as one blank line: its `\\r` is \
             whitespace, so the whole `\\r\\n` line joins the removal span"
        );
    }

    #[test]
    fn pin_nested_block_comment_prelude_leaves_with_the_region() {
        let source = "/** outer /* inner */ still prelude */\nclass Drop\n\nval keep = 1\n";
        assert_eq!(
            remove(source, &["Drop"]).as_deref(),
            Some("val keep = 1\n"),
            "block_comment_start is nesting-aware: it finds the outer opener that \
             matches the final closer, so the whole doc block goes with the region"
        );
    }

    #[test]
    fn pin_unbalanced_comment_prelude_bails_and_a_dangling_closer_is_not_claimed() {
        // Unbalanced OPENER: the scanner swallows the rest of the file into the
        // comment, so the declaration is invisible and the WHOLE removal bails.
        assert_eq!(remove("/* open /* inner */\nclass Drop\n", &["Drop"]), None);
        // Unbalanced CLOSER: block_comment_start refuses, so the dangling `*/`
        // is not comment prelude and stays in the file verbatim.
        assert_eq!(
            remove("val a = 1 */\nclass B\n", &["B"]).as_deref(),
            Some("val a = 1 */\n"),
            "the prelude walk only ever claims balanced block comments"
        );
    }
}
